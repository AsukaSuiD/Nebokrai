//! Контракт и пул фоновых DB worker-ов загрузки игроков Realm.
//!
//! Owner сначала проверяет game-exit, затем player-load-exit и только после
//! этого делает `Sleep(1)`. Наличие работы продолжает обработку текущего FIFO,
//! а пустая очередь возвращает поток к началу polling-loop.
//!
//! `WorldPlayerLoadWorkerSpec` хранит cloneable пары load/data FIFO, поэтому
//! системным потокам не передаётся mutable `CGame` и не нужен process-global
//! singleton. `std::thread::JoinHandle` заменяет `_beginthreadex`/Win32 HANDLE;
//! Tokio runtime используется только для уже асинхронной Tiberius DB-границы.
//! Один общий signal сохраняет два исходных flags и их приоритет проверки.
//! Пустой slot после ошибки создания остаётся в vector-order, как null HANDLE,
//! а shutdown выставляет player-load-exit и join-ит все slots по порядку.
//!
//! Загружаемый игрок параметризован (`P`) и создаётся через узкий
//! `WorldPlayerLoadFactory`, поэтому worker-семья старого `CPlayer` не знает.

use std::future::Future;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tokio::runtime::Handle;

use nebokrai_shared::runtime::put_string_to_file;

use crate::characters::playerdataqueue::{CPlayerDataQueue, PlayerDataQueueEntry};
use crate::characters::playerloadqueue::CPlayerLoadQueue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGameInitWorkerHandleState {
    Open,
    Empty,
}

/// Построение загрузочной заглушки игрока для worker-а: worker создаёт пустого
/// игрока и записывает в него player/account ID до фактической DB-загрузки.
/// Реализация у владельца игрока (старый `CPlayer`; имя
/// `set_database_load_identity` совпадает с inherent-методом специально,
/// inherent priority исключает рекурсию).
pub trait WorldPlayerLoadFactory: Sized {
    fn new_database_load_player() -> Self;
    fn set_database_load_identity(&mut self, player_id: i32, account: &[u8]);
}

pub trait WorldPlayerDataLoadOwner<P> {
    fn load_player_data<'a>(
        &'a mut self,
        player: &'a mut P,
    ) -> impl Future<Output = bool> + 'a;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerLoadBatchRecordOutcome {
    SkippedZeroPlayerId,
    Loaded,
    LoadFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerLoadBatchRecordReport {
    pub player_id: i32,
    pub client_ip: u32,
    pub started_at_ms: Option<u32>,
    pub finished_at_ms: Option<u32>,
    pub elapsed_ms: Option<u32>,
    pub outcome: WorldPlayerLoadBatchRecordOutcome,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerLoadBatchReport {
    pub worker_index: u32,
    pub drained_records: usize,
    pub records: Vec<WorldPlayerLoadBatchRecordReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerLoadBatchBlock {
    pub worker_index: u32,
    pub player_id: i32,
    pub processed_records: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlayerLoadWorkerExit {
    GameThread,
    PlayerLoadThreads,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerLoadWorkerReport {
    pub worker_index: u32,
    pub completed_batches: u32,
    pub drained_records: u32,
    pub exit: WorldPlayerLoadWorkerExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldPlayerLoadWorkerBlock {
    pub worker_index: u32,
    pub completed_batches: u32,
    pub drained_records: u32,
    pub source: WorldPlayerLoadBatchBlock,
}

/// Cloneable queue-owner, который можно безопасно передать системному
/// `LoadPlayerDataFromDB` потоку без передачи всего mutable `CGame`.
pub struct WorldPlayerLoadWorkerSpec<P> {
    player_load_queue: CPlayerLoadQueue,
    player_data_queue: CPlayerDataQueue<P>,
}

// Ручная реализация вместо derive: оба FIFO — `Arc` clone и не требуют
// `P: Clone`.
impl<P> Clone for WorldPlayerLoadWorkerSpec<P> {
    fn clone(&self) -> Self {
        Self {
            player_load_queue: self.player_load_queue.clone(),
            player_data_queue: self.player_data_queue.clone(),
        }
    }
}

impl<P> WorldPlayerLoadWorkerSpec<P>
where
    P: WorldPlayerLoadFactory,
{
    pub const fn new(
        player_load_queue: CPlayerLoadQueue,
        player_data_queue: CPlayerDataQueue<P>,
    ) -> Self {
        Self {
            player_load_queue,
            player_data_queue,
        }
    }

    pub async fn process_batch<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        mut get_tick: GetTick,
    ) -> Result<WorldPlayerLoadBatchReport, WorldPlayerLoadBatchBlock>
    where
        Loader: WorldPlayerDataLoadOwner<P> + ?Sized,
        LoadLargess: FnMut(&mut P) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let mut drained = self.player_load_queue.pop_player_load_data_to_list();
        let drained_records = drained.len();
        if drained_records != 0 {
            put_string_to_file(
                "TemptLoadDataLog",
                format!(
                    "Thread {} Need To Process {} DB Request.",
                    worker_index as i32, drained_records as u32 as i32,
                )
                .as_bytes(),
            );
        }

        let mut records = Vec::with_capacity(drained_records);
        while let Some(entry) = drained.pop_front() {
            let player_id = entry.player_id();
            let client_ip = entry.client_ip();
            if player_id == 0 {
                records.push(WorldPlayerLoadBatchRecordReport {
                    player_id,
                    client_ip,
                    started_at_ms: None,
                    finished_at_ms: None,
                    elapsed_ms: None,
                    outcome: WorldPlayerLoadBatchRecordOutcome::SkippedZeroPlayerId,
                });
                continue;
            }
            let Some(account) = entry.cdkey().map(<[u8]>::to_vec) else {
                return Err(WorldPlayerLoadBatchBlock {
                    worker_index,
                    player_id,
                    processed_records: records.len(),
                });
            };

            let started_at_ms = get_tick();
            put_string_to_file(
                "TemptLoadDataLog",
                format!(
                    "{} Request Read DB Start. (PID:{})",
                    player_id, worker_index as i32,
                )
                .as_bytes(),
            );

            let mut player = Box::new(P::new_database_load_player());
            player.set_database_load_identity(player_id, &account);
            let loaded = loader.load_player_data(&mut player).await;
            let mut player = if loaded {
                Some(player)
            } else {
                put_string_to_file(
                    "debug-DB",
                    format!("Read Player DB Error. (ID:{player_id})").as_bytes(),
                );
                None
            };

            if let Some(player) = player.as_deref_mut() {
                load_largess(player);
            }
            let fixed_account = entry.fixed_cdkey();
            let _ = self
                .player_data_queue
                .push_player_data(PlayerDataQueueEntry::new(
                    fixed_account,
                    player_id as u32,
                    client_ip,
                    player,
                ));

            let finished_at_ms = get_tick();
            let elapsed_ms = finished_at_ms.wrapping_sub(started_at_ms);
            put_string_to_file(
                "TemptLoadDataLog",
                format!(
                    "{} Read DB End. (PID:{}) (time:{})",
                    player_id, worker_index as i32, elapsed_ms,
                )
                .as_bytes(),
            );
            records.push(WorldPlayerLoadBatchRecordReport {
                player_id,
                client_ip,
                started_at_ms: Some(started_at_ms),
                finished_at_ms: Some(finished_at_ms),
                elapsed_ms: Some(elapsed_ms),
                outcome: if loaded {
                    WorldPlayerLoadBatchRecordOutcome::Loaded
                } else {
                    WorldPlayerLoadBatchRecordOutcome::LoadFailed
                },
            });
        }

        Ok(WorldPlayerLoadBatchReport {
            worker_index,
            drained_records,
            records,
        })
    }

    pub async fn run<Loader, LoadLargess, GetTick>(
        &self,
        worker_index: u32,
        game_thread_exit: &AtomicBool,
        player_load_threads_exit: &AtomicBool,
        loader: &mut Loader,
        load_largess: &mut LoadLargess,
        mut get_tick: GetTick,
    ) -> Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>
    where
        Loader: WorldPlayerDataLoadOwner<P> + ?Sized,
        LoadLargess: FnMut(&mut P) + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let mut completed_batches = 0_u32;
        let mut drained_records = 0_u32;
        loop {
            let exit = if game_thread_exit.load(Ordering::Relaxed) {
                Some(WorldPlayerLoadWorkerExit::GameThread)
            } else if player_load_threads_exit.load(Ordering::Relaxed) {
                Some(WorldPlayerLoadWorkerExit::PlayerLoadThreads)
            } else {
                None
            };
            if let Some(exit) = exit {
                return Ok(WorldPlayerLoadWorkerReport {
                    worker_index,
                    completed_batches,
                    drained_records,
                    exit,
                });
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
            let batch = self
                .process_batch(worker_index, loader, load_largess, &mut get_tick)
                .await
                .map_err(|source| WorldPlayerLoadWorkerBlock {
                    worker_index,
                    completed_batches,
                    drained_records,
                    source,
                })?;
            completed_batches = completed_batches.wrapping_add(1);
            drained_records = drained_records.wrapping_add(batch.drained_records as u32);
        }
    }
}

#[derive(Default)]
struct WorldPlayerLoadWorkerSignal {
    game_thread_exit: AtomicBool,
    player_load_threads_exit: AtomicBool,
}

#[derive(Debug)]
pub enum WorldPlayerLoadWorkerCompletion {
    Empty,
    Returned(Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>),
    Panicked,
}

struct WorldPlayerLoadWorkerSlot {
    worker_index: u32,
    handle: Option<JoinHandle<Result<WorldPlayerLoadWorkerReport, WorldPlayerLoadWorkerBlock>>>,
}

pub struct WorldPlayerLoadWorkerPool {
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
    pub fn new() -> Self {
        Self::default()
    }

 /// Создаёт один worker-slot. Ошибка spawn сохраняет пустой slot,
 /// чтобы Release видел тот же ordered vector, что и исходный owner.
    pub fn start<P, Loader, LoadLargess, GetTick>(
        &mut self,
        spec: WorldPlayerLoadWorkerSpec<P>,
        worker_index: u32,
        runtime: Handle,
        mut loader: Loader,
        mut load_largess: LoadLargess,
        mut get_tick: GetTick,
    ) -> (WorldGameInitWorkerHandleState, Option<io::Error>)
    where
        P: WorldPlayerLoadFactory + Send + 'static,
        Loader: WorldPlayerDataLoadOwner<P> + Send + 'static,
        LoadLargess: FnMut(&mut P) + Send + 'static,
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

    pub fn request_game_thread_exit(&self) {
        self.signal.game_thread_exit.store(true, Ordering::Relaxed);
    }

    pub fn request_player_load_threads_exit(&self) {
        self.signal
            .player_load_threads_exit
            .store(true, Ordering::Relaxed);
    }

    pub fn worker_slots(&self) -> usize {
        self.workers.len()
    }

    pub fn stop(&mut self) -> Vec<(u32, WorldPlayerLoadWorkerCompletion)> {
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
