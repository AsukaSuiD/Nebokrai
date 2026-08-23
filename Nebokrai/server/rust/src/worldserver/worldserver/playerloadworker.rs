//! Жизненный цикл пула `LoadPlayerDataFromDB` исторического WorldServer.
//!
//! исходный owner
//! проверяет сначала
//! game-exit, затем player-load-exit и только после этого делает `Sleep(1)`.
//! продолжает текущий FIFO-list, а
//! возвращается к началу polling-loop.
//!
//! `WorldPlayerLoadWorkerSpec` хранит cloneable пары load/data FIFO, поэтому
//! системным потокам не передаётся mutable `CGame` и не нужен process-global
//! singleton. `std::thread::JoinHandle` заменяет `_beginthreadex`/Win32 HANDLE;
//! Tokio runtime используется только для уже асинхронной Tiberius DB-границы.
//! Один общий signal сохраняет два исходных flags и их приоритет проверки.
//! Пустой slot после ошибки создания остаётся в vector-order, как null HANDLE,
//! а shutdown выставляет player-load-exit и join-ит все slots по порядку.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use tokio::runtime::Handle;

use crate::worldserver::appworld::player::CPlayer;
use crate::worldserver::worldserver::game::{
    WorldGameInitWorkerHandleState, WorldPlayerDataLoadOwner, WorldPlayerLoadWorkerBlock,
    WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerSpec,
};

#[derive(Default)]
struct WorldPlayerLoadWorkerSignal {
    game_thread_exit: AtomicBool,
    player_load_threads_exit: AtomicBool,
}

/// Итог одного vector-slot после штатного ordered join.
#[derive(Debug)]
pub(crate) enum WorldPlayerLoadWorkerCompletion {
    Empty,
    Returned(Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>),
    Panicked,
}

struct WorldPlayerLoadWorkerSlot {
    worker_index: u32,
    handle: Option<JoinHandle<Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>>>,
}

/// Owned vector системных DB-load потоков и два общих exit-флага.
pub(crate) struct WorldPlayerLoadWorkerPool {
    signal: Arc<WorldPlayerLoadWorkerSignal>,
    workers: Vec<WorldPlayerLoadWorkerSlot>,
}

impl Default for WorldPlayerLoadWorkerPool {
    fn default() -> Self {
        Self {
            signal: Arc::new(WorldPlayerLoadWorkerSignal::default()),
            workers: Vec::new(),
        }
    }
}

impl WorldPlayerLoadWorkerPool {
    pub(crate) fn new() -> Self {
        Self::default()
    }

 /// Создаёт один worker-slot. Ошибка spawn сохраняет пустой slot,
 /// чтобы Release видел тот же ordered vector, что и исходный owner.
    pub(crate) fn start<Loader, LoadLargess, GetTick>(
        &mut self,
        spec: WorldPlayerLoadWorkerSpec,
        worker_index: u32,
        runtime: Handle,
        mut loader: Loader,
        mut load_largess: LoadLargess,
        mut get_tick: GetTick,
    ) -> (WorldGameInitWorkerHandleState, Option<io::Error>)
    where
        Loader: WorldPlayerDataLoadOwner + Send + 'static,
        LoadLargess: FnMut(&mut CPlayer) + Send + 'static,
        GetTick: FnMut() -> u32 + Send + 'static,
    {
        let signal = Arc::clone(&self.signal);
        let spawned = thread::Builder::new()
            .name(format!("world-player-load-{worker_index}"))
            .spawn(move || {
                runtime.block_on(spec.run(
                    worker_index,
                    &signal.game_thread_exit,
                    &signal.player_load_threads_exit,
                    &mut loader,
                    &mut load_largess,
                    &mut get_tick,
                ))
            });

        match spawned {
            Ok(handle) => {
                self.workers.push(WorldPlayerLoadWorkerSlot {
                    worker_index,
                    handle: Some(handle),
                });
                (WorldGameInitWorkerHandleState::Open, None)
            }
            Err(error) => {
                self.workers.push(WorldPlayerLoadWorkerSlot {
                    worker_index,
                    handle: None,
                });
                (WorldGameInitWorkerHandleState::Empty, Some(error))
            }
        }
    }

    pub(crate) fn request_game_thread_exit(&self) {
        self.signal.game_thread_exit.store(true, Ordering::Relaxed);
    }

    pub(crate) fn request_player_load_threads_exit(&self) {
        self.signal
            .player_load_threads_exit
            .store(true, Ordering::Relaxed);
    }

    pub(crate) fn worker_slots(&self) -> usize {
        self.workers.len()
    }

 /// Выставляет общий load-exit и ждёт все handles в insertion-order.
    pub(crate) fn stop(&mut self) -> Vec<(u32, WorldPlayerLoadWorkerCompletion)> {
        self.request_player_load_threads_exit();
        self.workers
            .drain(..)
            .map(|slot| {
                let completion = match slot.handle {
                    Some(handle) => match handle.join() {
                        Ok(result) => WorldPlayerLoadWorkerCompletion::Returned(result),
                        Err(_) => WorldPlayerLoadWorkerCompletion::Panicked,
                    },
                    None => WorldPlayerLoadWorkerCompletion::Empty,
                };
                (slot.worker_index, completion)
            })
            .collect()
    }
}

impl Drop for WorldPlayerLoadWorkerPool {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
