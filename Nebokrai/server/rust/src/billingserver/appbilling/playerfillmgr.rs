//! Владелец optional `CPlayerFillMgr` исторического BillingServer.
//!
//! Контракт выборки, broadcast, удаления и worker lifecycle подтверждён точной
//! парой BillingServer EXE/PDB.
//!
//! Один проход получает не более 50 строк `TBL_NeedUpdate` в DB-порядке. Для
//! каждой строки он вызывает фактический `CRsPlayerAccount::GetUserPoint`,
//! строит broadcast `0xFF004 + Account + result [+ point при result == 0]` и
//! только после всего snapshot удаляет из таблицы все прочитанные ID одним
//! вызовом. Ошибка отдельной отправки не отменяет последующие ответы или delete;
//! DB-fallback `GetUserPoint = -2` остаётся у его собственного владельца.
//!
//! Worker один раз выполняет технический `InitConn`, затем проверяет общий
//! `g_bGameThreadExit` перед каждым проходом и безусловно выдерживает `5000 ms`
//! после него. `End` только ждёт завершение: отдельного stop-сообщения эта ветвь
//! `CGame::Release` не посылала. Windows `PeekMessage(0x66A)` не имеет внешнего
//! sender в достигнутом lifecycle и не получает пустой Linux thread-message
//! аналог; фактическую остановку сохраняет общий atomic-флаг.
//!
//! `JoinHandle` заменяет `_beginthreadex/WaitForSingleObject/CloseHandle`,
//! Tiberius owners — COM apartment и статический ADO config, а `CString`, `Vec`
//! и `Drop` — C-string/STL/SEH cleanup. Recordset/ADO, iostream/STL internals,
//! deleting thunks, `$L/$E` cleanup и compiler catch-блоки удалены.

use std::collections::VecDeque;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;

use crate::dbaccess::dbbilling::rsplayeraccount::{
    BillingDatabaseSettings, RsPlayerAccountInitializationError, RsPlayerAccountNotice,
    RsPlayerAccountOwner, TiberiusRsPlayerAccount,
};
use crate::dbaccess::dbbilling::rsplayerfillmgr::{
    PlayerFillDatabaseSettings, RsPlayerFillInitializationError, RsPlayerFillNotice,
    RsPlayerFillOwner, TiberiusRsPlayerFillMgr,
};
use crate::nets::netbilling::message::CMessage;
use crate::nets::servers::ServerCommandHandle;

const PLAYER_FILL_RESPONSE: i32 = 0x000F_F004;
const PLAYER_FILL_CADENCE: Duration = Duration::from_millis(5_000);

/// DB-owner, который не смог начать работу внутри уже созданного thread.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerFillWorkerOwner {
    FillTable,
    PlayerAccount,
}

/// Operator-visible событие optional PlayerFill worker.
#[derive(Debug)]
pub(crate) enum PlayerFillNotice {
    FillDatabase(RsPlayerFillNotice),
    AccountDatabase(RsPlayerAccountNotice),
    WorkerUnavailable {
        owner: PlayerFillWorkerOwner,
        reason: String,
    },
    WorkerPanicked,
}

/// Точная фатальная граница `Start`: ОС не создала worker thread.
#[derive(Debug)]
pub(crate) struct StartPlayerFillError(io::Error);

impl fmt::Display for StartPlayerFillError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "не создан PlayerFill worker: {}", self.0)
    }
}

impl Error for StartPlayerFillError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

/// Общая static-часть исходного `CPlayerFillMgr`.
#[derive(Default)]
pub(crate) struct CPlayerFillMgr {
    notices: Mutex<VecDeque<PlayerFillNotice>>,
}

impl CPlayerFillMgr {
    /// Выполняет один fetch/send/delete проход исходного `Run`.
    pub(crate) fn run(
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

    /// Забирает следующее DB/worker-событие без credential/account values.
    pub(crate) fn pop_notice(&self) -> Option<PlayerFillNotice> {
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

/// Owned Linux lifecycle единственного optional PlayerFill thread.
pub(crate) struct PlayerFillRuntime {
    manager: Arc<CPlayerFillMgr>,
    fill_database_settings: PlayerFillDatabaseSettings,
    account_database_settings: BillingDatabaseSettings,
    sender: ServerCommandHandle,
    game_thread_exit: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl PlayerFillRuntime {
    /// Связывает DB settings, broadcast-owner и общий game-thread exit.
    pub(crate) fn new(
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

    /// Создаёт единственный worker; только ошибка thread spawn является `false`.
    pub(crate) fn start(&mut self) -> Result<(), StartPlayerFillError> {
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

    /// Ждёт thread без отдельного stop, как исходный `End`.
    pub(crate) fn end(&mut self) -> bool {
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
