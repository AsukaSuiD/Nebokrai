//! Системный owner повторного подключения WorldServer к LoginServer.
//!
//! Источник: `WorldServer/worldserver/game.cpp:2246,4883`, RVA
//! `0x00004320/0x000033B0`, точная пара `Nworldserver.exe + WorldServer.pdb`.
//! `CreateConnectLoginThread` сначала ставил exit-флаг предыдущего worker-а,
//! безусловно ждал его handle, закрывал handle и только затем создавал новый.
//! Сам worker проверял флаг до первой паузы, затем делал `Sleep(8000)`, одну
//! попытку reconnect и проверял флаг лишь после неуспеха. `JoinHandle`,
//! `AtomicBool` и Tokio `Handle` заменяют CRT/Win32 plumbing; восьмисекундная
//! cadence и порядок stop/join/start остаются явными.
//!
//! Worker не захватывает mutable `CGame`: `WorldLoginReconnectSpec` содержит
//! snapshot setup и клонируемый отправитель единственной World FIFO. Поэтому
//! новый client всё ещё проходит тот же `ProcessMessage` handoff, а фоновый
//! поток не получает доступ к player, setup или transport-server состоянию.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tokio::runtime::Handle;

use super::game::{WorldLoginReconnectSpec, WorldLoginReconnectWorkerOutcome};

const LOGIN_RECONNECT_INTERVAL: Duration = Duration::from_secs(8);

#[derive(Default)]
struct WorldLoginReconnectWorkerSignal {
    exit: AtomicBool,
}

/// Итог уже остановленного системного reconnect-worker-а.
#[derive(Debug)]
pub(crate) enum WorldLoginReconnectWorkerCompletion {
    Returned(WorldLoginReconnectWorkerOutcome),
    Panicked,
}

/// Owned Linux-замена одного Win32 `hConnectThread`.
pub(crate) struct WorldLoginReconnectWorker {
    signal: Arc<WorldLoginReconnectWorkerSignal>,
    handle: Option<JoinHandle<WorldLoginReconnectWorkerOutcome>>,
}

impl WorldLoginReconnectWorker {
    /// Создаёт новый worker после полного join предыдущего owner-а.
    pub(crate) fn start(
        spec: WorldLoginReconnectSpec,
        runtime: Handle,
    ) -> Result<Self, io::Error> {
        let signal = Arc::new(WorldLoginReconnectWorkerSignal::default());
        let worker_signal = Arc::clone(&signal);
        let handle = thread::Builder::new()
            .name("world-login-reconnect".to_owned())
            .spawn(move || run_worker(spec, runtime, worker_signal))?;
        Ok(Self {
            signal,
            handle: Some(handle),
        })
    }

    /// Соответствует записи `bConnectThreadExit = true`.
    pub(crate) fn request_exit(&self) {
        self.signal.exit.store(true, Ordering::Relaxed);
    }

    /// Выполняет exact wait/close-пару в safe форме ownership `JoinHandle`.
    pub(crate) fn join(&mut self) -> Option<WorldLoginReconnectWorkerCompletion> {
        self.handle.take().map(|handle| match handle.join() {
            Ok(outcome) => WorldLoginReconnectWorkerCompletion::Returned(outcome),
            Err(_) => WorldLoginReconnectWorkerCompletion::Panicked,
        })
    }

    /// Выставляет exit и ждёт worker; пауза 8 s намеренно не прерывается.
    pub(crate) fn stop(&mut self) -> Option<WorldLoginReconnectWorkerCompletion> {
        self.request_exit();
        self.join()
    }
}

impl Drop for WorldLoginReconnectWorker {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn run_worker(
    spec: WorldLoginReconnectSpec,
    runtime: Handle,
    signal: Arc<WorldLoginReconnectWorkerSignal>,
) -> WorldLoginReconnectWorkerOutcome {
    if signal.exit.load(Ordering::Relaxed) {
        return WorldLoginReconnectWorkerOutcome::StoppedBeforeRetry;
    }

    let mut attempts = 0_u32;
    loop {
        thread::sleep(LOGIN_RECONNECT_INTERVAL);
        attempts = attempts.wrapping_add(1);
        if let Ok(reconnect) = runtime.block_on(spec.reconnect_once()) {
            return WorldLoginReconnectWorkerOutcome::Reconnected {
                attempts,
                reconnect,
            };
        }
        if signal.exit.load(Ordering::Relaxed) {
            return WorldLoginReconnectWorkerOutcome::StoppedAfterFailedRetry { attempts };
        }
    }
}
