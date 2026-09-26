//! Save-worker и save-runtime процесса WorldServer (`SaveThreadFunc` launcher
//! и barriers).
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает). Машинные основания save-пайплайна и контракт сохранения
//! зафиксированы у владельца пайплайна (см. `persistence::saveworker`); этот
//! файл — его process-проводка.
//!
//! `WorldSaveWorker::launch` строит двенадцать конкретных Tiberius save-DB
//! owner-ов и зовёт [`save_thread_func`]; thread-serialization, сборка
//! завершившихся handle-ов, join и trigger-guard повторяют исходный
//! process-порядок.

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use chrono::{Datelike, Timelike};

use crate::activities::rsgodsbattle::TiberiusRsGodsBattle;
use crate::activities::rsjjcsys::TiberiusRsJjcSys;
use crate::app::world_main_loop_contexts::WorldMainLoopContextBuildError;
use crate::app::world_process::WorldProcessDomainOwners;
use crate::app::world_process_init::WorldProcessInitContext;
use crate::app::worldserver::{
    WorldLogLocalTime, WorldLogTextOwner, WorldSaveThreadHandleState,
    WorldSaveThreadLaunchRequest,
};
use crate::content::dbgoods::TiberiusDbGoods;
use crate::organizations::dbcountry::TiberiusDbCountry;
use crate::organizations::rsenemyfactions::TiberiusRsEnemyFactions;
use crate::organizations::rsfaction::TiberiusRsFaction;
use crate::organizations::rsunion::TiberiusRsUnion;
use crate::persistence::largess::TiberiusLargess;
use crate::persistence::rsplayer::TiberiusRsPlayer;
use crate::persistence::rsgenvar::TiberiusRsGenVar;
use crate::persistence::rssetup::{TiberiusRsSetup, WorldDatabaseSettings};
use crate::persistence::savedb::{
    SaveDataLogPublisher, SaveDataMonitoringSnapshot, WorldSaveThreadJob,
};
use crate::persistence::saveworker::{
    WorldSaveRuntimeContext, WorldSaveThreadReport, save_thread_func,
};
use crate::regions::rsregion::TiberiusRsRegion;

pub struct WorldSaveWorkerCompletion {
    pub(crate) retained_job: Option<WorldSaveThreadJob>,
    retained_serialization: Option<tokio::sync::OwnedMutexGuard<()>>,
}

pub struct WorldSaveWorker {
    runtime: tokio::runtime::Handle,
    started_at: Instant,
    settings: WorldDatabaseSettings,
    largess: Arc<TiberiusLargess>,
    log: WorldLogTextOwner,
    serialization: Arc<tokio::sync::Mutex<()>>,
    handles: Vec<JoinHandle<WorldSaveWorkerCompletion>>,
    retained_jobs: Vec<WorldSaveThreadJob>,
    retained_serializations: Vec<tokio::sync::OwnedMutexGuard<()>>,
}

impl WorldSaveWorker {
    pub(crate) fn new(
        runtime: tokio::runtime::Handle,
        started_at: Instant,
        settings: WorldDatabaseSettings,
        largess: Arc<TiberiusLargess>,
        log: WorldLogTextOwner,
    ) -> Self {
        Self {
            runtime,
            started_at,
            settings,
            largess,
            log,
            serialization: Arc::new(tokio::sync::Mutex::new(())),
            handles: Vec::new(),
            retained_jobs: Vec::new(),
            retained_serializations: Vec::new(),
        }
    }

    pub(crate) fn serialization(&self) -> Arc<tokio::sync::Mutex<()>> {
        Arc::clone(&self.serialization)
    }

    /// Запускает очередной background role, не ожидая предыдущий поток.
    ///
    /// Исходный caller закрывал только kernel handle: уже запущенный save-thread
    /// продолжал работу и сериализовался внутри `SaveThreadFunc`. Rust хранит
    /// join-handle-ы до release, но собирает здесь лишь уже завершившиеся.
    pub(crate) fn launch(&mut self, job: WorldSaveThreadJob) -> WorldSaveThreadHandleState {
        self.collect_finished();

        let runtime = self.runtime.clone();
        let started_at = self.started_at;
        let settings = self.settings.clone();
        let mut largess = self.largess.clone_save_owner();
        let mut log = self.log.clone();
        let serialization = Arc::clone(&self.serialization);
        let pending_job = Arc::new(parking_lot::Mutex::new(Some(job)));
        let thread_job = Arc::clone(&pending_job);
        let handle = thread::Builder::new()
            .name("world-save".to_owned())
            .spawn(move || {
                let mut job = thread_job
                    .lock()
                    .take()
                    .expect("save job принадлежит единственному worker-у");
                let mut setup_database = TiberiusRsSetup::new_for_save(settings.clone());
                let mut variable_database = TiberiusRsGenVar::new(settings.clone());
                let mut player_database = TiberiusRsPlayer::new(&settings);
                let mut jjc_database = TiberiusRsJjcSys::new(&settings);
                let mut goods_database = TiberiusDbGoods::new(&settings);
                let mut union_database = TiberiusRsUnion::new(settings.clone());
                let mut faction_database = TiberiusRsFaction::new(settings.clone());
                let mut region_database = TiberiusRsRegion::new(settings.clone());
                let mut gods_battle_database = TiberiusRsGodsBattle::new(&settings);
                let mut enemy_factions_database = TiberiusRsEnemyFactions::new(settings.clone());
                let mut country_database = TiberiusDbCountry::default();
                let mut lifecycle = *job.lifecycle.lock();
                let shared_lifecycle = Arc::clone(&job.lifecycle);
                let write_log_queue = job.write_log_queue.clone();
                let server_name = job.server_name.clone();
                let server_id = job.server_id;
                let world_number_bits = job.world_number_bits;
                let save_info_time_ms = job.save_info_time_ms;
                let mut publisher = SaveDataLogPublisher::new(
                    false,
                    save_info_time_ms,
                    &mut log,
                    move || started_at.elapsed().as_millis() as u32,
                    || {
                        let now = chrono::Local::now();
                        WorldLogLocalTime {
                            year: now.year() as u16,
                            month: now.month() as u16,
                            day: now.day() as u16,
                            hour: now.hour() as u16,
                            minute: now.minute() as u16,
                            second: now.second() as u16,
                        }
                    },
                    |payload: &[u8]| {
                        eprintln!("WorldServer: {}", String::from_utf8_lossy(payload));
                    },
                );

                let completed = runtime.block_on(async {
                    let mut serialization = Some(serialization.lock_owned().await);
                    let report = save_thread_func(
                        &mut job.save,
                        &settings,
                        &mut lifecycle,
                        &job.variables,
                        &job.registry,
                        &mut job.honor_ranks,
                        job.gods_battle_faction_xyd,
                        &job.gods_battle_npc_factions,
                        job.use_old_save_largess_way,
                        &mut setup_database,
                        &mut variable_database,
                        &mut player_database,
                        &mut jjc_database,
                        &mut goods_database,
                        &mut union_database,
                        &mut faction_database,
                        &mut region_database,
                        &mut gods_battle_database,
                        &mut enemy_factions_database,
                        &mut country_database,
                        &mut largess,
                        &mut publisher,
                        move |snapshot| *shared_lifecycle.lock() = snapshot,
                        || drop(serialization.take()),
                        move || SaveDataMonitoringSnapshot {
                            server_name,
                            write_log_count: write_log_queue.len() as u32,
                            server_id,
                            world_number_bits,
                        },
                    )
                    .await;
                    let completed = matches!(&report, WorldSaveThreadReport::Complete { .. });
                    drop(report);
                    (completed, serialization)
                });

                WorldSaveWorkerCompletion {
                    retained_job: (!completed.0).then_some(job),
                    retained_serialization: completed.1,
                }
            });

        match handle {
            Ok(handle) => {
                self.handles.push(handle);
                WorldSaveThreadHandleState::Open
            }
            Err(error) => {
                eprintln!("WorldServer: не создан save-worker: {error}");
                if let Some(job) = pending_job.lock().take() {
                    self.retained_jobs.push(job);
                }
                WorldSaveThreadHandleState::Empty
            }
        }
    }

    fn collect_completion(&mut self, handle: JoinHandle<WorldSaveWorkerCompletion>) {
        match handle.join() {
            Ok(completion) => {
                if let Some(job) = completion.retained_job {
                    self.retained_jobs.push(job);
                }
                if let Some(serialization) = completion.retained_serialization {
                    self.retained_serializations.push(serialization);
                }
            }
            Err(_) => eprintln!("WorldServer: save-worker завершился panic"),
        }
    }

    fn collect_finished(&mut self) {
        let handles = std::mem::take(&mut self.handles);
        for handle in handles {
            if handle.is_finished() {
                self.collect_completion(handle);
            } else {
                self.handles.push(handle);
            }
        }
    }

    pub(crate) fn join(&mut self) -> WorldSaveThreadHandleState {
        let previous = if self.handles.is_empty() {
            WorldSaveThreadHandleState::Empty
        } else {
            WorldSaveThreadHandleState::Open
        };
        let handles = std::mem::take(&mut self.handles);
        for handle in handles {
            self.collect_completion(handle);
        }
        previous
    }
}

pub struct WorldProcessSaveRuntime {
    worker: WorldSaveWorker,
    trigger_guard: Option<tokio::sync::OwnedMutexGuard<()>>,
}

impl WorldProcessSaveRuntime {
    pub(crate) fn new(worker: WorldSaveWorker) -> Self {
        Self {
            worker,
            trigger_guard: None,
        }
    }

    pub(crate) fn wait_for_barrier(&self) {
        let runtime = self.worker.runtime.clone();
        let serialization = self.worker.serialization();
        tokio::task::block_in_place(|| {
            let guard = runtime.block_on(serialization.lock_owned());
            drop(guard);
        });
    }

    pub(crate) fn join(&mut self) -> WorldSaveThreadHandleState {
        self.worker.join()
    }

    pub(crate) fn after_game_init(
        runtime: tokio::runtime::Handle,
        init: &WorldProcessInitContext,
        domains: &WorldProcessDomainOwners,
    ) -> Result<Self, WorldMainLoopContextBuildError> {
        let settings = init
            .database_settings()
            .ok_or(WorldMainLoopContextBuildError::MissingDatabaseSettings)?;
        let largess = init
            .largess()
            .ok_or(WorldMainLoopContextBuildError::MissingLargessOwner)?;
        Ok(Self::new(WorldSaveWorker::new(
            runtime,
            init.started_at(),
            settings,
            largess,
            domains.log.clone(),
        )))
    }
}

impl WorldSaveRuntimeContext for WorldProcessSaveRuntime {
    fn try_enter_trigger(&mut self) -> bool {
        if self.trigger_guard.is_some() {
            return false;
        }
        match self.worker.serialization().try_lock_owned() {
            Ok(guard) => {
                self.trigger_guard = Some(guard);
                true
            }
            Err(_) => false,
        }
    }

    fn leave_trigger(&mut self) {
        drop(self.trigger_guard.take());
    }

    fn launch(
        &mut self,
        _request: &WorldSaveThreadLaunchRequest,
        job: WorldSaveThreadJob,
    ) -> WorldSaveThreadHandleState {
        self.worker.launch(job)
    }
}
