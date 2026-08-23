//! Управляемый reconnect worker направления WorldServer -> LoginServer.
//!
//! Источник контракта — точная пара WorldServer EXE/PDB. Worker выполняет
//! первый reconnect сразу, затем повторяет попытку с исходной cadence, пока
//! соединение не опубликовано либо owned shutdown не отменит ожидание.
//! Replacement client передаётся main-loop через typed handoff, поэтому смена
//! network-owner-а происходит в его исходной позиции и не обгоняет сообщения.
//!
//! Tokio task/cancellation заменяет Win32 thread message и handle; stop всегда
//! дожидается завершения задачи. Endpoint order, reconnect result и
//! control-send publication остаются у `CGame`.

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

 /// Выполняет wait/close-пару в safe форме ownership `JoinHandle`.
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
