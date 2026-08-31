//! Процессный владелец GameServer.
//!
//! Источник жизненного цикла — `gameserver.exe` и `GameServer.pdb`, исходный
//! owner `gameserver/gameserver.cpp`. Этот модуль хранит только действительно
//! процессное состояние: runtime-каталог, сигнал завершения и общий поток
//! legacy RNG. Игровые реестры, игроки, регионы и фабрики остаются у `CGame`;
//! их нельзя дублировать здесь ради формального `GameThreadRuntime`.

use std::error::Error;
use std::fmt;
use std::future::{Future, poll_fn};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;

use tokio::task::{JoinHandle, JoinSet};

use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::region::RegionRandomContext;
use crate::gameserver::appserver::script::function::{
    ScriptAwardAuthenticationContext, ScriptAwardAuthenticationSubmission,
};
use crate::nets::netserver::mynetclient::{GameClientIoError, GameClientIoStep};
use crate::nets::netserver::mynetserver::CMyNetServer;
use crate::nets::servers::{
    AcceptStart, AdmissionOutcome, ServerCommandHandle, ServerIoAction, ServerIoCompletion,
};

use super::game::{
    CGame, GameExitRuntime, GameMainLoopRuntime, GameNetworkRuntime, GameReleaseRuntime,
    GameRuntimePathOwner, GameRuntimePaths, GameThreadRuntime, game_tick_milliseconds,
};

#[derive(Clone)]
pub(crate) struct GameProcessControl {
    exit_requested: Arc<AtomicBool>,
}

impl GameProcessControl {
    pub(crate) fn request_exit(&self) {
        self.exit_requested.store(true, Ordering::Release);
    }
}

pub(crate) struct GameProcessRuntime {
    runtime: tokio::runtime::Handle,
    runtime_directory: PathBuf,
    exit_requested: Arc<AtomicBool>,
    random_state: u32,
    network: GameProcessNetworkRuntime,
}

impl GameProcessRuntime {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        runtime_directory: PathBuf,
        random_state: u32,
    ) -> (Self, GameProcessControl) {
        let exit_requested = Arc::new(AtomicBool::new(false));
        let control = GameProcessControl {
            exit_requested: Arc::clone(&exit_requested),
        };
        (
            Self {
                runtime,
                runtime_directory,
                exit_requested,
                random_state,
                network: GameProcessNetworkRuntime::new(),
            },
            control,
        )
    }

    pub(crate) fn runtime_paths(&self) -> GameRuntimePaths {
        GameRuntimePaths::from_runtime_directory(&self.runtime_directory)
    }

    pub(crate) fn exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Acquire)
    }

    pub(crate) async fn run_network_turn(
        &mut self,
        game: &mut CGame,
    ) -> Result<GameProcessNetworkTurn, GameProcessNetworkError> {
        self.network.run_turn(game).await
    }

    pub(crate) fn exit_network_server_worker(&mut self, server: &mut CMyNetServer) {
        let runtime = self.runtime.clone();
        let result =
            tokio::task::block_in_place(|| runtime.block_on(self.network.release_server(server)));
        if let Err(error) = result {
            tracing::warn!(?error, "сетевой worker GameServer не завершён");
        }
    }
}

#[derive(Default)]
pub(crate) struct GameProcessNetworkTurn {
    pub(crate) admissions: Vec<AdmissionOutcome>,
    pub(crate) accept_errors: Vec<io::Error>,
    pub(crate) io_completions: Vec<ServerIoCompletion>,
    pub(crate) server_errors: Vec<String>,
    pub(crate) world: Option<Result<GameClientIoStep, GameClientIoError>>,
    pub(crate) billing: Option<Result<GameClientIoStep, GameClientIoError>>,
}

#[derive(Debug)]
pub(crate) enum GameProcessNetworkError {
    Task(tokio::task::JoinError),
}

impl fmt::Display for GameProcessNetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Task(error) => {
                write!(formatter, "сетевая задача GameServer завершилась: {error}")
            }
        }
    }
}

impl Error for GameProcessNetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Task(error) => Some(error),
        }
    }
}

pub(crate) struct GameProcessNetworkRuntime {
    accept_task: Option<JoinHandle<io::Result<(tokio::net::TcpStream, std::net::SocketAddrV4)>>>,
    io_tasks: JoinSet<ServerIoCompletion>,
}

impl GameProcessNetworkRuntime {
    fn new() -> Self {
        Self {
            accept_task: None,
            io_tasks: JoinSet::new(),
        }
    }

    pub(crate) async fn run_turn(
        &mut self,
        game: &mut CGame,
    ) -> Result<GameProcessNetworkTurn, GameProcessNetworkError> {
        let mut turn = GameProcessNetworkTurn::default();
        self.drain_completed(game, &mut turn).await?;
        self.start_accept(game);

        if let Some(server) = game.current_net_server_mut() {
            let snapshot = server.process_network_snapshot(game_tick_milliseconds());
            let commands = server.command_handle();
            let (actions, errors) = snapshot.into_parts();
            turn.server_errors = errors
                .into_iter()
                .map(|error| format!("{error:?}"))
                .collect();
            self.spawn_io_actions(actions, commands);
        }

        if game.world_client().is_some() {
            turn.world = poll_once(game.run_world_io_once()).await;
        }
        if game.billing_client().is_some() {
            turn.billing = poll_once(game.run_billing_io_once()).await;
        }
        Ok(turn)
    }

    async fn release_server(
        &mut self,
        server: &mut CMyNetServer,
    ) -> Result<(), GameProcessNetworkError> {
        if let Some(task) = self.accept_task.take() {
            task.abort();
            match task.await {
                Ok(result) => drop(result),
                Err(error) if error.is_cancelled() => {}
                Err(error) => return Err(GameProcessNetworkError::Task(error)),
            }
        }

        let _ = server.command_handle().quit_all();
        while server.client_count() != 0 {
            let snapshot = server.process_network_snapshot(game_tick_milliseconds());
            let commands = server.command_handle();
            let (actions, _errors) = snapshot.into_parts();
            self.spawn_io_actions(actions, commands);
            self.drain_io_completions(None)?;
            tokio::task::yield_now().await;
        }
        self.io_tasks.shutdown().await;
        Ok(())
    }

    async fn drain_completed(
        &mut self,
        game: &mut CGame,
        turn: &mut GameProcessNetworkTurn,
    ) -> Result<(), GameProcessNetworkError> {
        if self
            .accept_task
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            let result = self
                .accept_task
                .take()
                .expect("завершившаяся Game accept-задача проверена")
                .await
                .map_err(GameProcessNetworkError::Task)?;
            match result {
                Ok((stream, peer)) => {
                    if let Some(server) = game.current_net_server_mut() {
                        turn.admissions.push(server.queue_accepted(
                            stream,
                            peer,
                            game_tick_milliseconds(),
                        ));
                    }
                }
                Err(error) => turn.accept_errors.push(error),
            }
        }
        self.drain_io_completions(Some(&mut turn.io_completions))
    }

    fn start_accept(&mut self, game: &mut CGame) {
        if self.accept_task.is_some() {
            return;
        }
        let Some(server) = game.current_net_server_mut() else {
            return;
        };
        if let AcceptStart::Pending(accept) = server.begin_accept() {
            self.accept_task = Some(tokio::spawn(async move { accept.accept().await }));
        }
    }

    fn spawn_io_actions(&mut self, actions: Vec<ServerIoAction>, commands: ServerCommandHandle) {
        for action in actions {
            let commands = commands.clone();
            self.io_tasks
                .spawn(async move { action.run(commands).await });
        }
    }

    fn drain_io_completions(
        &mut self,
        mut output: Option<&mut Vec<ServerIoCompletion>>,
    ) -> Result<(), GameProcessNetworkError> {
        while let Some(completion) = self.io_tasks.try_join_next() {
            let completion = completion.map_err(GameProcessNetworkError::Task)?;
            if let Some(output) = &mut output {
                output.push(completion);
            }
        }
        Ok(())
    }
}

async fn poll_once<Output>(future: impl Future<Output = Output>) -> Option<Output> {
    let mut future = Box::pin(future);
    poll_fn(move |context| {
        Poll::Ready(match future.as_mut().poll(context) {
            Poll::Ready(output) => Some(output),
            Poll::Pending => None,
        })
    })
    .await
}

impl RegionRandomContext for GameProcessRuntime {
    fn random_below(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        self.random_state = self
            .random_state
            .wrapping_mul(214_013)
            .wrapping_add(2_531_011);
        (((self.random_state >> 16) & 0x7fff) as i32) % bound
    }
}

impl ScriptAwardAuthenticationContext for GameProcessRuntime {
    fn submit_script_award_authentication(
        &mut self,
        _player: &CPlayer,
        _patch_id: i32,
        _information_type: i32,
        _color: u32,
        _background: u32,
    ) -> ScriptAwardAuthenticationSubmission {
        // UniBill/Bsip был загружаемым внешним provider-ом исходного процесса.
        // В доступном комплекте нет ни его API, ни callback-а, поэтому здесь
        // нельзя создавать фиктивный order ID или изображать успешную заявку.
        ScriptAwardAuthenticationSubmission::ProviderUnavailable
    }
}

impl GameRuntimePathOwner for GameProcessRuntime {
    fn runtime_paths(&self) -> GameRuntimePaths {
        GameProcessRuntime::runtime_paths(self)
    }
}

impl GameExitRuntime for GameProcessRuntime {
    fn exit_requested(&self) -> bool {
        GameProcessRuntime::exit_requested(self)
    }
}

impl GameReleaseRuntime for GameProcessRuntime {
    fn exit_network_server_worker(&mut self, server: &mut CMyNetServer) {
        GameProcessRuntime::exit_network_server_worker(self, server);
    }
}

impl GameNetworkRuntime for GameProcessRuntime {
    fn process_network_turn(
        &mut self,
        game: &mut CGame,
    ) -> impl Future<Output = ()> {
        async move {
            match self.run_network_turn(game).await {
                Ok(turn) => {
                    for error in &turn.accept_errors {
                        tracing::warn!(?error, "listener GameServer не принял соединение");
                    }
                    for error in &turn.server_errors {
                        tracing::warn!(error, "сетевой сеанс GameServer завершил операцию с ошибкой");
                    }
                    if let Some(Err(error)) = &turn.world {
                        tracing::warn!(?error, "сетевой ход World-клиента GameServer не завершён");
                    }
                    if let Some(Err(error)) = &turn.billing {
                        tracing::warn!(?error, "сетевой ход Billing-клиента GameServer не завершён");
                    }
                    tracing::trace!(
                        admissions = turn.admissions.len(),
                        io_completions = turn.io_completions.len(),
                        world_polled = turn.world.is_some(),
                        billing_polled = turn.billing.is_some(),
                        "завершён неблокирующий сетевой ход GameServer"
                    );
                }
                Err(error) => {
                    tracing::warn!(?error, "process-owned сетевой ход GameServer прерван");
                }
            }
        }
    }
}

impl GameMainLoopRuntime for GameProcessRuntime {}

impl GameThreadRuntime for GameProcessRuntime {}
