//! Безопасный fire-and-forget owner `CRsPlayer::ResetAllLeitingInDB`.
//!
//! worker `DbLetTingUpdate`.
//!
//! выделяет payload из двух DWORD, передаёт его в один
//! `_beginthreadex` и сразу возвращает success/failure создания. Ни retry, ни
//! queue, ни merge двух daily reset-ов не происходят. Каждый удачный dispatch
//! ниже так же запускает отдельный поток с собственным TDS-соединением.
//! `Arc<CThingSetup>` удерживает immutable setup до конца worker-а, а clone
//! globe snapshot заменяет чтение singleton-а. Намеренная утечка Win32 HANDLE
//! не сохраняется: сброшенный `JoinHandle` отсоединяет поток, но системный
//! ресурс Rust освобождает после завершения; DB-result передаётся log-owner-у
//! через неблокирующий канал.

use std::io;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use tokio::runtime::Handle;

use crate::dbaccess::worlddb::rsplayer::{
    LeiTingDatabaseResetOutcome, LeiTingDatabaseResetRequest, TiberiusRsPlayer,
};
use crate::dbaccess::worlddb::rssetup::WorldDatabaseSettings;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::setup::leitingsetup::CThingSetup;

/// Внешний log-owner получает те же begin/end границы worker-а без shared
/// mutable DB-owner-а между потоками.
#[derive(Debug)]
pub(crate) enum WorldLeiTingResetWorkerEvent {
    Started(LeiTingDatabaseResetRequest),
    Finished {
        request: LeiTingDatabaseResetRequest,
        outcome: LeiTingDatabaseResetOutcome,
    },
}

/// Владение неизменяемыми входами каждого отдельного LeiTing DB worker-а.
pub(crate) struct WorldLeiTingResetWorker {
    settings: WorldDatabaseSettings,
    thing_setup: Arc<CThingSetup>,
    globe_setup: GlobeSetupSnapshot,
    events: Receiver<WorldLeiTingResetWorkerEvent>,
    event_sender: mpsc::Sender<WorldLeiTingResetWorkerEvent>,
}

impl WorldLeiTingResetWorker {
    pub(crate) fn new(
        settings: WorldDatabaseSettings,
        thing_setup: Arc<CThingSetup>,
        globe_setup: GlobeSetupSnapshot,
    ) -> Self {
        let (event_sender, events) = mpsc::channel();
        Self {
            settings,
            thing_setup,
            globe_setup,
            events,
            event_sender,
        }
    }

 /// Повторяет один `ResetAllLeitingInDB` call: ошибка создания остаётся
 /// синхронным false-эквивалентом, DB итог приходит независимо позднее.
    pub(crate) fn dispatch(
        &self,
        request: LeiTingDatabaseResetRequest,
        runtime: Handle,
    ) -> Result<(), io::Error> {
        let settings = self.settings.clone();
        let thing_setup = Arc::clone(&self.thing_setup);
        let globe_setup = self.globe_setup.clone();
        let events = self.event_sender.clone();
        let handle = thread::Builder::new()
            .name("world-leiting-db-reset".to_owned())
            .spawn(move || {
                let _ = events.send(WorldLeiTingResetWorkerEvent::Started(request));
                let owner = TiberiusRsPlayer::new(&settings);
                let outcome = runtime.block_on(owner.db_lei_ting_update(
                    request,
                    thing_setup.as_ref(),
                    globe_setup.total_jing_li_dan_count(),
                ));
                let _ = events.send(WorldLeiTingResetWorkerEvent::Finished { request, outcome });
            })?;
        drop(handle);
        Ok(())
    }

    pub(crate) fn try_next_event(&self) -> Option<WorldLeiTingResetWorkerEvent> {
        match self.events.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }
}
