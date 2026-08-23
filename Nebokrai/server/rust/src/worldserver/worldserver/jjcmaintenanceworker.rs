//! Безопасный detached owner недельного DB-сброса `CRsJJcSys::JJcWeekClear`.
//!
//! caller `CJJcSystem::ResetJJc`, `JJcWeekClear`, worker
//! `DbJJC`. Исходный bool отражает только результат
//! `_beginthreadex`; BAKE и Clear исполняются позднее и не меняют этот bool.
//! Rust отделяет DB settings от mutable World owner-а, освобождает thread
//! handle ownership-ом и возвращает поздний DB-итог через неблокирующий канал.

use std::io;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use tokio::runtime::Handle;

use crate::dbaccess::worlddb::rsjjcsys::{RsJjcSysNotice, RsJjcSysOwner, TiberiusRsJjcSys};
use crate::dbaccess::worlddb::rssetup::WorldDatabaseSettings;

#[derive(Debug)]
pub(crate) enum WorldJjcWeekClearWorkerEvent {
    WeekStarted,
    WeekFinished {
        returned_true: bool,
        notices: Vec<RsJjcSysNotice>,
    },
    SeasonFinished {
        returned_true: bool,
        notices: Vec<RsJjcSysNotice>,
    },
    SeasonExecutionThreadFailed(io::Error),
    SeasonExecutionThreadPanicked,
}

pub(crate) struct WorldJjcWeekClearWorker {
    settings: WorldDatabaseSettings,
    events: Receiver<WorldJjcWeekClearWorkerEvent>,
    event_sender: mpsc::Sender<WorldJjcWeekClearWorkerEvent>,
}

impl WorldJjcWeekClearWorker {
    pub(crate) fn new(settings: WorldDatabaseSettings) -> Self {
        let (event_sender, events) = mpsc::channel();
        Self {
            settings,
            events,
            event_sender,
        }
    }

 /// Возвращает успех только создания system thread, как `JJcWeekClear`.
    pub(crate) fn dispatch(&self, runtime: Handle) -> Result<(), io::Error> {
        let settings = self.settings.clone();
        let events = self.event_sender.clone();
        let handle = thread::Builder::new()
            .name("world-jjc-week-clear".to_owned())
            .spawn(move || {
                let _ = events.send(WorldJjcWeekClearWorkerEvent::WeekStarted);
                let mut owner = TiberiusRsJjcSys::new(&settings);
                let returned_true = runtime.block_on(owner.run_jjc_week_clear_database());
                let mut notices = Vec::new();
                while let Some(notice) = owner.pop_notice() {
                    notices.push(notice);
                }
                let _ = events.send(WorldJjcWeekClearWorkerEvent::WeekFinished {
                    returned_true,
                    notices,
                });
            })?;
        drop(handle);
        Ok(())
    }

 /// Сохраняет синхронный bool `JJcSeasonClear`. Технический helper-thread
 /// нужен только потому, что MainLoop уже исполняется внутри Tokio runtime;
 /// join оставляет внешний blocking/timing контракт исходного caller-а.
    pub(crate) fn clear_season(&self, runtime: Handle) -> bool {
        let settings = self.settings.clone();
        let handle = match thread::Builder::new()
            .name("world-jjc-season-clear".to_owned())
            .spawn(move || {
                let mut owner = TiberiusRsJjcSys::new(&settings);
                let returned_true = runtime.block_on(owner.clear_jjc_season());
                let mut notices = Vec::new();
                while let Some(notice) = owner.pop_notice() {
                    notices.push(notice);
                }
                (returned_true, notices)
            })
        {
            Ok(handle) => handle,
            Err(error) => {
                let _ = self
                    .event_sender
                    .send(WorldJjcWeekClearWorkerEvent::SeasonExecutionThreadFailed(error));
                return false;
            }
        };
        match handle.join() {
            Ok((returned_true, notices)) => {
                let _ = self
                    .event_sender
                    .send(WorldJjcWeekClearWorkerEvent::SeasonFinished {
                        returned_true,
                        notices,
                    });
                returned_true
            }
            Err(_) => {
                let _ = self
                    .event_sender
                    .send(WorldJjcWeekClearWorkerEvent::SeasonExecutionThreadPanicked);
                false
            }
        }
    }

    pub(crate) fn try_next_event(&self) -> Option<WorldJjcWeekClearWorkerEvent> {
        match self.events.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }
}
