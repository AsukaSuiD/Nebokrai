//! Сетевая прокладка одного хода процесса WorldServer — Tokio-воплощение
//! net-thread семейства `CServer`.
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_server`]
//! (идентификаторы — в evidence). Семантика snapshot, admission и I/O-операций
//! принадлежит владельцам `nebokrai_shared::network` и направлениям `app::world_server` /
//! `app::world_client`; этот файл переносит только планировку хода.
//!
//! Tokio-задача worker-а заменяет net-thread и объединяет accept с
//! worker-threads в одном `JoinSet`: ход снимает завершившиеся accept/I/O
//! задачи, ставит новый accept, применяет атомарный `process_network_snapshot`
//! и запускает I/O-действия; пауза 10 ms — между проходами, а не гарантия
//! такта. Завершение повторяет исходный порядок остановки: сигнал → подхват
//! задачи → abort accept → `quit_all` и досылка I/O до пустого реестра
//! clients → shutdown набора. Без worker-а release выполняет тот же flush на
//! текущей задаче.
//!
//! Владельцы направлений передаются явными параметрами; файл не знает `CGame`.
//! Две mut-ссылки на его поля нельзя получить одновременно через accessor-ы,
//! поэтому прежний единый `run_turn` разнесён на последовательные стадии
//! `run_game_server_worker` и `poll_login_client`; process-обвязка собирает
//! ход в исходном порядке, а типовой итог и отчёт остаются общими.
//!
//! Доказательства: docs/reconstruction/realm-services.md#world-процесс-и-lifecycle

use std::error::Error;
use std::fmt;
use std::future::{Future, poll_fn};
use std::io;
use std::task::Poll;

use tokio::task::{JoinHandle as TokioJoinHandle, JoinSet};

use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, ServerCommandHandle, ServerIoAction, ServerIoCompletion,
    ServerSnapshotError,
};

use super::world_client::{CMyNetClient, WorldClientIoError, WorldClientIoStep};
use super::world_server::CMyNetServer;
use super::world_server_client::GameServerReceiveError;

/// Типовой итог одного сетевого хода процесса World.
#[derive(Default)]
pub struct WorldProcessNetworkTurn {
    pub admissions: Vec<AdmissionOutcome>,
    pub accept_errors: Vec<io::Error>,
    pub io_completions: Vec<ServerIoCompletion>,
    pub server_errors: Vec<ServerSnapshotError<GameServerReceiveError>>,
    pub login: Option<Result<WorldClientIoStep, WorldClientIoError>>,
}

#[derive(Debug)]
pub enum WorldProcessNetworkError {
    Task(tokio::task::JoinError),
}

impl fmt::Display for WorldProcessNetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Task(error) => write!(formatter, "World network-задача завершилась: {error}"),
        }
    }
}

impl Error for WorldProcessNetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Task(error) => Some(error),
        }
    }
}

pub struct WorldProcessNetworkRuntime {
    worker: Option<WorldServerNetworkWorker>,
}

struct WorldServerNetworkWorker {
    stop: tokio::sync::oneshot::Sender<()>,
    task: TokioJoinHandle<Result<(), WorldProcessNetworkError>>,
}

impl WorldProcessNetworkRuntime {
    pub fn new() -> Self {
        Self { worker: None }
    }

    /// Подхватывает завершившийся worker и при наличии server-направления
    /// запускает его net-thread — прежний префикс `run_turn` до опроса Login.
    /// Завершившаяся с ошибкой задача останавливает ход прежней ошибкой.
    pub async fn run_game_server_worker(
        &mut self,
        game_server: Option<&mut CMyNetServer>,
        legacy_tick_ms: fn() -> u32,
    ) -> Result<(), WorldProcessNetworkError> {
        if self.worker.as_ref().is_some_and(|worker| worker.task.is_finished()) {
            let worker = self.worker.take().expect("завершившийся World network-worker проверен");
            worker.task.await.map_err(WorldProcessNetworkError::Task)??;
        }
        if self.worker.is_none() {
            if let Some(server) = game_server {
                let mut server = server.clone();
                let (stop, mut stopped) = tokio::sync::oneshot::channel();
                let task = tokio::spawn(async move {
                    let mut network = WorldServerNetworkRuntime::new(legacy_tick_ms);
                    let result = loop {
                        match network.run_turn(&mut server).await {
                            Ok(turn) => report_world_network_turn(&turn),
                            Err(error) => break Err(error),
                        }
                        tokio::select! {
                            _ = &mut stopped => break Ok(()),
                            _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {}
                        }
                    };
                    let released = network.release_server(&mut server).await;
                    result.and(released)
                });
                self.worker = Some(WorldServerNetworkWorker { stop, task });
            }
        }
        Ok(())
    }

    /// Выполняет один неблокирующий опрос исходящего Login-направления —
    /// прежний хвост `run_turn`; отсутствующий client оставляет опрос пустым.
    pub async fn poll_login_client(
        login_client: Option<&mut CMyNetClient>,
        legacy_tick_ms: fn() -> u32,
    ) -> Option<Result<WorldClientIoStep, WorldClientIoError>> {
        match login_client {
            Some(client) => poll_once(client.run_io_once(legacy_tick_ms)).await,
            None => None,
        }
    }

    /// Останавливает worker сигналом либо, когда его нет, выполняет flush
    /// текущего server-направления на вызывающей задаче.
    pub async fn release_server(
        &mut self,
        server: &mut CMyNetServer,
        legacy_tick_ms: fn() -> u32,
    ) -> Result<(), WorldProcessNetworkError> {
        if let Some(worker) = self.worker.take() {
            let _ = worker.stop.send(());
            worker.task.await.map_err(WorldProcessNetworkError::Task)?
        } else {
            WorldServerNetworkRuntime::new(legacy_tick_ms)
                .release_server(server)
                .await
        }
    }

    pub fn release_client(&mut self, client: &mut CMyNetClient) {
        let _ = client.close();
    }
}

struct WorldServerNetworkRuntime {
    accept_task: Option<TokioJoinHandle<io::Result<(tokio::net::TcpStream, std::net::SocketAddrV4)>>>,
    io_tasks: JoinSet<ServerIoCompletion>,
    legacy_tick_ms: fn() -> u32,
}

impl WorldServerNetworkRuntime {
    fn new(legacy_tick_ms: fn() -> u32) -> Self {
        Self {
            accept_task: None,
            io_tasks: JoinSet::new(),
            legacy_tick_ms,
        }
    }

    async fn run_turn(
        &mut self,
        server: &mut CMyNetServer,
    ) -> Result<WorldProcessNetworkTurn, WorldProcessNetworkError> {
        let mut turn = WorldProcessNetworkTurn::default();
        self.drain_completed(server, &mut turn).await?;
        self.start_accept(server);

        {
            let snapshot = server.process_network_snapshot((self.legacy_tick_ms)());
            let commands = server.command_handle();
            let (actions, errors) = snapshot.into_parts();
            turn.server_errors = errors;
            self.spawn_io_actions(actions, commands);
        }

        Ok(turn)
    }

    async fn release_server(
        &mut self,
        server: &mut CMyNetServer,
    ) -> Result<(), WorldProcessNetworkError> {
        if let Some(task) = self.accept_task.take() {
            task.abort();
            match task.await {
                Ok(result) => drop(result),
                Err(error) if error.is_cancelled() => {}
                Err(error) => return Err(WorldProcessNetworkError::Task(error)),
            }
        }

        let _ = server.command_handle().quit_all();
        while server.has_clients() {
            let snapshot = server.process_network_snapshot((self.legacy_tick_ms)());
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
        server: &mut CMyNetServer,
        turn: &mut WorldProcessNetworkTurn,
    ) -> Result<(), WorldProcessNetworkError> {
        if self
            .accept_task
            .as_ref()
            .is_some_and(TokioJoinHandle::is_finished)
        {
            let result = self
                .accept_task
                .take()
                .expect("завершившаяся World accept-задача проверена")
                .await
                .map_err(WorldProcessNetworkError::Task)?;
            match result {
                Ok((stream, peer)) => {
                    turn.admissions.push(server.queue_accepted(
                        stream,
                        peer,
                        (self.legacy_tick_ms)(),
                    ));
                }
                Err(error) => turn.accept_errors.push(error),
            }
        }
        self.drain_io_completions(Some(&mut turn.io_completions))
    }

    fn start_accept(&mut self, server: &mut CMyNetServer) {
        if self.accept_task.is_some() {
            return;
        }
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
    ) -> Result<(), WorldProcessNetworkError> {
        while let Some(completion) = self.io_tasks.try_join_next() {
            let completion = completion.map_err(WorldProcessNetworkError::Task)?;
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

/// Публикует заметные исходы хода в процессный журнал: не-Queued admission
/// и ошибки accept/transport обоих направлений; это stderr-форма, а не
/// доменное событие.
pub fn report_world_network_turn(turn: &WorldProcessNetworkTurn) {
    for admission in &turn.admissions {
        if !matches!(admission, AdmissionOutcome::Queued { .. }) {
            eprintln!("WorldServer: GameServer admission: {admission:?}");
        }
    }
    for error in &turn.accept_errors {
        eprintln!("WorldServer: ошибка GameServer accept: {error}");
    }
    for error in &turn.server_errors {
        eprintln!("WorldServer: ошибка GameServer transport: {error:?}");
    }
    if let Some(Err(error)) = &turn.login {
        eprintln!("WorldServer: ошибка Login transport: {error}");
    }
}
