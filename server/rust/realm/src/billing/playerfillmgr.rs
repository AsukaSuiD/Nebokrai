//! Optional worker `CPlayerFillMgr`, подтверждённый `billingserver.exe` и
//! `billingserver.pdb` по owner-у `appbilling/playerfillmgr.cpp`.
//!
//! Проход читает до 50 строк в DB-порядке, рассылает `0xFF004` для каждой и
//! лишь затем одним вызовом удаляет все прочитанные ID; ошибка отправки не
//! отменяет следующие ответы или delete. Worker проверяет общий exit перед
//! проходом и всегда ждёт 5000 ms после него. `End` только присоединяет поток;
//! Win32 thread API и ADO заменены owned `JoinHandle` и Tiberius.
//! Перенесён в Realm `billing/`.

use std::collections::VecDeque;
use std::ffi::CString;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;
use thiserror::Error;

use super::rsplayeraccount::{
    BillingDatabaseSettings, RsPlayerAccountInitializationError, RsPlayerAccountNotice,
    RsPlayerAccountOwner, TiberiusRsPlayerAccount,
};
use crate::app::billing_message::CMessage;
use crate::billing::rsplayerfillmgr::{
    PlayerFillDatabaseSettings, RsPlayerFillInitializationError, RsPlayerFillNotice,
    RsPlayerFillOwner, TiberiusRsPlayerFillMgr,
};
use nebokrai_shared::network::ServerCommandHandle;

const PLAYER_FILL_RESPONSE: i32 = 0x000F_F004;
const PLAYER_FILL_CADENCE: Duration = Duration::from_millis(5_000);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerFillWorkerOwner {
    FillTable,
    PlayerAccount,
}

#[derive(Debug)]
pub enum PlayerFillNotice {
    FillDatabase(RsPlayerFillNotice),
    AccountDatabase(RsPlayerAccountNotice),
    WorkerUnavailable {
        owner: PlayerFillWorkerOwner,
        reason: String,
    },
    WorkerPanicked,
}

#[derive(Debug, Error)]
#[error("не создан PlayerFill worker: {0}")]
pub struct StartPlayerFillError(#[source] io::Error);

#[derive(Default)]
pub struct CPlayerFillMgr {
    notices: Mutex<VecDeque<PlayerFillNotice>>,
}

impl CPlayerFillMgr {
    pub fn run(
        &self,
        fill_database: &mut dyn RsPlayerFillOwner,
        account_database: &mut dyn RsPlayerAccountOwner,
        sender: &ServerCommandHandle,
    ) -> bool {
        let entries = fill_database.get_player_fill_log();
        self.collect_fill_notices(fill_database);

        for entry in &entries {
            let outcome = account_database.get_user_point(&entry.player_account);
            self.collect_account_notices(account_database);

            let mut response = CMessage::new(PLAYER_FILL_RESPONSE);
            let account = legacy_c_string(&entry.player_account);
            response.base_mut().add_str(Some(&account));
            response.base_mut().add_long(outcome.result);
            if outcome.result == 0 {
                response.base_mut().add_long(outcome.point);
            }
            let _ = response.send_to_all_gs(sender);
        }

        if !entries.is_empty() {
            fill_database.delete_player_fill_log(&entries);
            self.collect_fill_notices(fill_database);
        }
        true
    }

    pub fn pop_notice(&self) -> Option<PlayerFillNotice> {
        self.notices.lock().pop_front()
    }

    fn collect_fill_notices(&self, database: &mut dyn RsPlayerFillOwner) {
        while let Some(notice) = database.pop_notice() {
            self.push_notice(PlayerFillNotice::FillDatabase(notice));
        }
    }

    fn collect_account_notices(&self, database: &mut dyn RsPlayerAccountOwner) {
        while let Some(notice) = database.pop_notice() {
            self.push_notice(PlayerFillNotice::AccountDatabase(notice));
        }
    }

    fn push_notice(&self, notice: PlayerFillNotice) {
        self.notices.lock().push_back(notice);
    }
}

pub struct PlayerFillRuntime {
    manager: Arc<CPlayerFillMgr>,
    fill_database_settings: PlayerFillDatabaseSettings,
    account_database_settings: BillingDatabaseSettings,
    sender: ServerCommandHandle,
    game_thread_exit: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl PlayerFillRuntime {
    pub fn new(
        manager: Arc<CPlayerFillMgr>,
        fill_database_settings: PlayerFillDatabaseSettings,
        account_database_settings: BillingDatabaseSettings,
        sender: ServerCommandHandle,
        game_thread_exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            manager,
            fill_database_settings,
            account_database_settings,
            sender,
            game_thread_exit,
            worker: None,
        }
    }

    pub fn start(&mut self) -> Result<(), StartPlayerFillError> {
        let manager = Arc::clone(&self.manager);
        let fill_settings = self.fill_database_settings.clone();
        let account_settings = self.account_database_settings.clone();
        let sender = self.sender.clone();
        let game_thread_exit = Arc::clone(&self.game_thread_exit);
        let worker = thread::Builder::new()
            .name("billing-player-fill".to_owned())
            .spawn(move || {
                let mut fill_database = match TiberiusRsPlayerFillMgr::new(fill_settings) {
                    Ok(database) => database,
                    Err(error) => {
                        manager.push_notice(worker_unavailable_fill(error));
                        return;
                    }
                };
                let mut account_database = match TiberiusRsPlayerAccount::new(account_settings) {
                    Ok(database) => database,
                    Err(error) => {
                        manager.push_notice(worker_unavailable_account(error));
                        return;
                    }
                };

                while !game_thread_exit.load(Ordering::Acquire) {
                    manager.run(&mut fill_database, &mut account_database, &sender);
                    thread::sleep(PLAYER_FILL_CADENCE);
                }
            })
            .map_err(StartPlayerFillError)?;
        self.worker = Some(worker);
        Ok(())
    }

    pub fn end(&mut self) -> bool {
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            self.manager.push_notice(PlayerFillNotice::WorkerPanicked);
        }
        true
    }
}

fn legacy_c_string(bytes: &[u8]) -> CString {
    let visible = bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end]);
    CString::new(visible).expect("slice обрезан по первому NUL")
}

fn worker_unavailable_fill(error: RsPlayerFillInitializationError) -> PlayerFillNotice {
    PlayerFillNotice::WorkerUnavailable {
        owner: PlayerFillWorkerOwner::FillTable,
        reason: error.to_string(),
    }
}

fn worker_unavailable_account(error: RsPlayerAccountInitializationError) -> PlayerFillNotice {
    PlayerFillNotice::WorkerUnavailable {
        owner: PlayerFillWorkerOwner::PlayerAccount,
        reason: error.to_string(),
    }
}
