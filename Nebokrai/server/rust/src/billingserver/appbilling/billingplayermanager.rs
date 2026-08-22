//! Общие FIFO и записи `CBillingPlayerManager` из
//! `appbilling/billingplayermanager.{h,cpp}`.
//!
//! Восстановлены записи, три общие FIFO и полный lifecycle worker-ов:
//! `BPMThreadFunc` `0x00012CC0`, `LogThreadFunc` `0x00012D70`, `Start`
//! `0x00012E10`, `CreateThread` `0x00012E60`, `Release` `0x00011840` и
//! `End` `0x000117E0`.
//!
//! Три очереди были process-static и общими для всех элементов `CGame::vecBPM`,
//! а каждый `Push` глубоко копировал запись в конец собственного `std::list`
//! под отдельным lock и безусловно возвращал `true`. Rust хранит один общий
//! owner с тремя независимыми `parking_lot::Mutex<VecDeque<_>>`; все workers
//! разделяют именно этот owner, а не получают собственные FIFO.
//! Owned аргумент заменяет временную C++ запись и её немедленную глубокую
//! копию, сохраняя bytes, порядок и lifetime без `new/delete`.
//!
//! `Run` и `OnLogProcess` копировали всю соответствующую глобальную очередь в
//! локальный список и очищали источник под тем же lock. `mem::take` переносит
//! тот же полный snapshot атомарно, не удерживая lock на будущих DB/send-
//! эффектах. Новые записи после снятия остаются следующему проходу. `Run`
//! всегда обрабатывает AC snapshot раньше отдельно снятого TR snapshot и
//! строит точные ответы `0xFF001..0xFF003`. В purchase-ветви `BuyItemCode`
//! выполняется до проверки `goods_number < 1001`: при превышении лимита уже
//! совершённый DB-эффект и optional IL-запись сохраняются, ответ не уходит,
//! остаток снятого TR snapshot теряется, а worker завершается.
//!
//! Каждый DB-worker имеет собственный `TiberiusRsPlayerAccount`, после
//! успешного `Run` спит ровно 1 ms и проверяет owned stop только в начале
//! следующего прохода. Atomic stop заменяет `PostThreadMessage(0x66A)` без
//! Windows message queue; последовательный `Release` по-прежнему публикует
//! stop и ждёт workers по одному. Cash-log worker сохраняет исходный do-while:
//! хотя бы один `OnLogProcess`, чтение текущего `dwSaveLogSvrTime` после DB и
//! проверка общего `g_bGameThreadExit` после сна. `Start` по-прежнему не делает
//! ошибку optional log-thread фатальной, а `End` только ждёт его при текущем
//! `bLogSvrSwitch`.
//!
//! COM/ADO заменены готовым DB-owner, Win32 threads/events — owned
//! `JoinHandle` и atomics, critical sections — `parking_lot::Mutex`. STL/CRT,
//! SEH, deleting destructors, allocators и static `$E/$L` cleanup удалены как
//! library/compiler noise. Технические DB/thread ошибки публикуются отдельными
//! notices без credentials; send-результаты, как в оригинале, игнорируются.

use std::collections::VecDeque;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use parking_lot::Mutex;

use crate::dbaccess::dbbilling::rsplayeraccount::{
    BillingDatabaseSettings, RsPlayerAccountInitializationError, RsPlayerAccountNotice,
    RsPlayerAccountOwner, TiberiusRsPlayerAccount,
};
use crate::nets::netbilling::message::CMessage;
use crate::nets::servers::ServerCommandHandle;
use crate::public::guid::CGuid;
use crate::public::tools::put_string_to_file;

/// Запрос баланса одного игрока из `0xEF201`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TagAccInfo {
    pub(crate) player_id: i32,
    pub(crate) player_identity: Vec<u8>,
    pub(crate) game_server_id: i32,
}

impl TagAccInfo {
    /// Глубоко владеет byte-exact строкой, которую C++ принимал по значению.
    pub(crate) fn new(player_id: i32, player_identity: Vec<u8>, game_server_id: i32) -> Self {
        Self {
            player_id,
            player_identity,
            game_server_id,
        }
    }
}

/// Входная покупка либо player-to-player trade из `0xEF202/0xEF203`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TagTradeNode {
    pub(crate) trade_type: i32,
    pub(crate) buyer_id: i32,
    pub(crate) seller_id: i32,
    pub(crate) buyer_identity: Vec<u8>,
    pub(crate) seller_identity: Vec<u8>,
    pub(crate) buyer_ip: Vec<u8>,
    pub(crate) seller_ip: Vec<u8>,
    pub(crate) buyer_name: Vec<u8>,
    pub(crate) seller_name: Vec<u8>,
    pub(crate) yuanbao: u32,
    pub(crate) goods_id: i32,
    pub(crate) goods_number: i32,
    pub(crate) game_server_id: i32,
    pub(crate) session_id: i32,
    pub(crate) plugin_id: i32,
    pub(crate) login_server_id: i32,
    pub(crate) world_server_id: i32,
    pub(crate) goods_guid: CGuid,
}

/// Аргументы полного value-конструктора `tagTradeNode`.
pub(crate) struct TagTradeNodeParts {
    pub(crate) trade_type: i32,
    pub(crate) buyer_id: i32,
    pub(crate) seller_id: i32,
    pub(crate) buyer_identity: Vec<u8>,
    pub(crate) seller_identity: Vec<u8>,
    pub(crate) buyer_ip: Vec<u8>,
    pub(crate) seller_ip: Vec<u8>,
    pub(crate) buyer_name: Vec<u8>,
    pub(crate) seller_name: Vec<u8>,
    pub(crate) yuanbao: u32,
    pub(crate) goods_id: i32,
    pub(crate) goods_number: i32,
    pub(crate) game_server_id: i32,
    pub(crate) session_id: i32,
    pub(crate) plugin_id: i32,
    pub(crate) login_server_id: i32,
    pub(crate) world_server_id: i32,
    pub(crate) goods_guid: CGuid,
}

impl TagTradeNode {
    /// Переносит все поля старого длинного конструктора без перестановки.
    pub(crate) fn from_parts(parts: TagTradeNodeParts) -> Self {
        Self {
            trade_type: parts.trade_type,
            buyer_id: parts.buyer_id,
            seller_id: parts.seller_id,
            buyer_identity: parts.buyer_identity,
            seller_identity: parts.seller_identity,
            buyer_ip: parts.buyer_ip,
            seller_ip: parts.seller_ip,
            buyer_name: parts.buyer_name,
            seller_name: parts.seller_name,
            yuanbao: parts.yuanbao,
            goods_id: parts.goods_id,
            goods_number: parts.goods_number,
            game_server_id: parts.game_server_id,
            session_id: parts.session_id,
            plugin_id: parts.plugin_id,
            login_server_id: parts.login_server_id,
            world_server_id: parts.world_server_id,
            goods_guid: parts.goods_guid,
        }
    }
}

/// Cash-log запись, которую DB-owner получает после trade.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagIncLogNode {
    pub(crate) log_time: f64,
    pub(crate) buyer_identity: Vec<u8>,
    pub(crate) buyer_ip: Vec<u8>,
    pub(crate) yuanbao: u32,
    pub(crate) goods_id: i32,
    pub(crate) goods_number: i32,
    pub(crate) login_server_id: i32,
    pub(crate) world_server_id: i32,
}

/// Аргументы value-конструктора `tagIncLogNode`.
pub(crate) struct TagIncLogNodeParts {
    pub(crate) log_time: f64,
    pub(crate) buyer_identity: Vec<u8>,
    pub(crate) buyer_ip: Vec<u8>,
    pub(crate) yuanbao: u32,
    pub(crate) goods_id: i32,
    pub(crate) goods_number: i32,
    pub(crate) login_server_id: i32,
    pub(crate) world_server_id: i32,
}

impl TagIncLogNode {
    /// Переносит все поля исходного cash-log конструктора.
    pub(crate) fn from_parts(parts: TagIncLogNodeParts) -> Self {
        Self {
            log_time: parts.log_time,
            buyer_identity: parts.buyer_identity,
            buyer_ip: parts.buyer_ip,
            yuanbao: parts.yuanbao,
            goods_id: parts.goods_id,
            goods_number: parts.goods_number,
            login_server_id: parts.login_server_id,
            world_server_id: parts.world_server_id,
        }
    }
}

/// Единый владелец трёх process-static FIFO старого класса.
#[derive(Default)]
pub(crate) struct CBillingPlayerManager {
    account_requests: Mutex<VecDeque<TagAccInfo>>,
    increment_logs: Mutex<VecDeque<TagIncLogNode>>,
    trade_requests: Mutex<VecDeque<TagTradeNode>>,
    notices: Mutex<VecDeque<BillingPlayerManagerNotice>>,
}

impl CBillingPlayerManager {
    /// Создаёт три пустые общие FIFO без запуска worker threads.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Добавляет account-запрос в конец общей AC FIFO.
    pub(crate) fn push_account_request(&self, request: TagAccInfo) -> bool {
        self.account_requests.lock().push_back(request);
        true
    }

    /// Добавляет cash-log в конец общей IL FIFO.
    pub(crate) fn push_increment_log(&self, record: TagIncLogNode) -> bool {
        self.increment_logs.lock().push_back(record);
        true
    }

    /// Добавляет purchase/trade запрос в конец общей TR FIFO.
    pub(crate) fn push_trade_request(&self, request: TagTradeNode) -> bool {
        self.trade_requests.lock().push_back(request);
        true
    }

    /// Атомарно снимает весь AC snapshot для одного `Run`.
    pub(crate) fn take_account_requests(&self) -> VecDeque<TagAccInfo> {
        mem::take(&mut *self.account_requests.lock())
    }

    /// Атомарно снимает весь IL snapshot для одного `OnLogProcess`.
    pub(crate) fn take_increment_logs(&self) -> VecDeque<TagIncLogNode> {
        mem::take(&mut *self.increment_logs.lock())
    }

    /// Атомарно снимает весь TR snapshot для одного `Run`.
    pub(crate) fn take_trade_requests(&self) -> VecDeque<TagTradeNode> {
        mem::take(&mut *self.trade_requests.lock())
    }
}

const ACCOUNT_RESPONSE: i32 = 0x000F_F001;
const INCREMENT_PURCHASE_RESPONSE: i32 = 0x000F_F002;
const PLAYER_TRADE_RESPONSE: i32 = 0x000F_F003;
const MAX_PURCHASE_GOODS_NUMBER: i32 = 1000;
const DATABASE_WORKER_CADENCE: Duration = Duration::from_millis(1);

/// Источник typed operator-записи manager runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BillingPlayerWorkerKind {
    Database,
    CashLog,
}

/// Наблюдаемая либо техническая запись worker lifecycle.
#[derive(Debug)]
pub(crate) enum BillingPlayerManagerNotice {
    Database(RsPlayerAccountNotice),
    IncrementPurchaseQuantity {
        player_id: i32,
        goods_id: i32,
        goods_number: i32,
    },
    WorkerUnavailable {
        worker: BillingPlayerWorkerKind,
        reason: String,
    },
    WorkerPanicked {
        worker: BillingPlayerWorkerKind,
    },
}

/// Ошибка Linux-замены исходного `CreateThread`.
#[derive(Debug)]
pub(crate) enum CreateBillingPlayerWorkerError {
    DatabaseOwner(RsPlayerAccountInitializationError),
    Spawn(io::Error),
}

impl fmt::Display for CreateBillingPlayerWorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseOwner(error) => error.fmt(formatter),
            Self::Spawn(error) => write!(formatter, "не создан Billing DB-worker: {error}"),
        }
    }
}

impl Error for CreateBillingPlayerWorkerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DatabaseOwner(error) => Some(error),
            Self::Spawn(error) => Some(error),
        }
    }
}

struct DatabaseWorker {
    stop_requested: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

/// Owned Linux lifecycle общей static-части и элементов исходного `vecBPM`.
pub(crate) struct BillingPlayerManagerRuntime {
    queues: Arc<CBillingPlayerManager>,
    sender: ServerCommandHandle,
    database_settings: BillingDatabaseSettings,
    log_server_enabled: Arc<AtomicBool>,
    save_log_interval_ms: Arc<AtomicU32>,
    game_thread_exit: Arc<AtomicBool>,
    database_workers: Vec<DatabaseWorker>,
    log_worker: Option<JoinHandle<()>>,
}

impl CBillingPlayerManager {
    /// Выполняет один исходный `Run`: AC snapshot, затем отдельный TR snapshot.
    pub(crate) fn run(
        &self,
        stop_requested: &AtomicBool,
        database: &mut dyn RsPlayerAccountOwner,
        sender: &ServerCommandHandle,
        log_server_enabled: &AtomicBool,
    ) -> bool {
        if stop_requested.load(Ordering::Acquire) {
            return false;
        }

        for request in self.take_account_requests() {
            let outcome = database.get_user_point(&request.player_identity);
            self.collect_database_notices(database);

            let mut response = CMessage::new(ACCOUNT_RESPONSE);
            response.base_mut().add_long(request.player_id);
            response.base_mut().add_long(outcome.result);
            if outcome.result == 0 {
                response.base_mut().add_long(outcome.point);
            }
            let _ = response.send_to_gs(sender, request.game_server_id);
        }

        let mut trades = self.take_trade_requests();
        while let Some(trade) = trades.pop_front() {
            if trade.seller_identity.is_empty() {
                let outcome = database.buy_item_code(&trade, log_server_enabled);
                self.collect_database_notices(database);
                if let Some(record) = outcome.increment_log {
                    self.push_increment_log(record);
                }

                if trade.goods_number > MAX_PURCHASE_GOODS_NUMBER {
                    self.push_notice(BillingPlayerManagerNotice::IncrementPurchaseQuantity {
                        player_id: trade.buyer_id,
                        goods_id: trade.goods_id,
                        goods_number: trade.goods_number,
                    });
                    let line = format!(
                        "playerid:[{}] buy goods:[{}];goodsnumber:[{}]",
                        trade.buyer_id, trade.goods_id, trade.goods_number
                    );
                    put_string_to_file("Increment_error_log", line.as_bytes());
                    return false;
                }

                let mut response = CMessage::new(INCREMENT_PURCHASE_RESPONSE);
                response.base_mut().add_long(trade.buyer_id);
                response.base_mut().add_long(outcome.result);
                if outcome.result == 0 {
                    response.base_mut().add_long(outcome.last_point);
                    response.base_mut().add_ulong(trade.yuanbao);
                    response.base_mut().add_long(trade.goods_id);
                    response.base_mut().add_long(trade.goods_number);
                    response.base_mut().add_long(trade.seller_id);
                    add_transaction_code(&mut response, outcome.transaction_code);
                }
                let _ = response.send_to_gs(sender, trade.game_server_id);
                continue;
            }

            let outcome = database.buy_player_item(&trade);
            self.collect_database_notices(database);

            let mut response = CMessage::new(PLAYER_TRADE_RESPONSE);
            response.base_mut().add_long(trade.buyer_id);
            response.base_mut().add_long(trade.seller_id);
            response.base_mut().add_long(outcome.result);
            if outcome.result == 0 {
                response.base_mut().add_long(outcome.buyer_last_point);
                response.base_mut().add_long(outcome.seller_last_point);
                response.base_mut().add_long(trade.trade_type);
                response.base_mut().add_long(trade.session_id);
                response.base_mut().add_long(trade.plugin_id);
                response.base_mut().add_ulong(trade.yuanbao);
                response.base_mut().add_guid(trade.goods_guid);
                add_transaction_code(&mut response, outcome.transaction_code);
            }
            let _ = response.send_to_gs(sender, trade.game_server_id);
        }

        true
    }

    /// Снимает IL snapshot, передаёт его одной DB-сессии и выдерживает cadence.
    pub(crate) fn on_log_process(
        &self,
        database: &mut dyn RsPlayerAccountOwner,
        save_log_interval_ms: &AtomicU32,
    ) -> bool {
        let records: Vec<_> = self.take_increment_logs().into_iter().collect();
        if !records.is_empty() {
            database.put_cash_log(&records);
            self.collect_database_notices(database);
        }

        let interval = save_log_interval_ms.load(Ordering::Acquire);
        thread::sleep(Duration::from_millis(u64::from(interval)));
        true
    }

    /// Забирает следующую operator/DB-запись без runtime-значений credentials.
    pub(crate) fn pop_notice(&self) -> Option<BillingPlayerManagerNotice> {
        self.notices.lock().pop_front()
    }

    fn collect_database_notices(&self, database: &mut dyn RsPlayerAccountOwner) {
        while let Some(notice) = database.pop_notice() {
            self.push_notice(BillingPlayerManagerNotice::Database(notice));
        }
    }

    fn push_notice(&self, notice: BillingPlayerManagerNotice) {
        self.notices.lock().push_back(notice);
    }
}

impl BillingPlayerManagerRuntime {
    /// Связывает общие FIFO с текущими Billing setup-флагами и server-owner.
    pub(crate) fn new(
        queues: Arc<CBillingPlayerManager>,
        sender: ServerCommandHandle,
        database_settings: BillingDatabaseSettings,
        log_server_enabled: Arc<AtomicBool>,
        save_log_interval_ms: Arc<AtomicU32>,
        game_thread_exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            queues,
            sender,
            database_settings,
            log_server_enabled,
            save_log_interval_ms,
            game_thread_exit,
            database_workers: Vec::new(),
            log_worker: None,
        }
    }

    /// Возвращает единственный общий owner трёх process-static FIFO.
    pub(crate) fn queues(&self) -> &Arc<CBillingPlayerManager> {
        &self.queues
    }

    /// Выполняет исходный `Start`; ошибка optional log-thread не меняет `true`.
    pub(crate) fn start(&mut self, log_enabled_at_start: bool) -> bool {
        if !log_enabled_at_start {
            return true;
        }

        let database = match TiberiusRsPlayerAccount::new(self.database_settings.clone()) {
            Ok(database) => database,
            Err(error) => {
                self.queues
                    .push_notice(BillingPlayerManagerNotice::WorkerUnavailable {
                        worker: BillingPlayerWorkerKind::CashLog,
                        reason: error.to_string(),
                    });
                return true;
            }
        };
        let queues = Arc::clone(&self.queues);
        let save_log_interval_ms = Arc::clone(&self.save_log_interval_ms);
        let game_thread_exit = Arc::clone(&self.game_thread_exit);
        match thread::Builder::new()
            .name("billing-cash-log".to_owned())
            .spawn(move || {
                let mut database = database;
                loop {
                    queues.on_log_process(&mut database, &save_log_interval_ms);
                    if game_thread_exit.load(Ordering::Acquire) {
                        break;
                    }
                }
            }) {
            Ok(worker) => self.log_worker = Some(worker),
            Err(error) => {
                self.queues
                    .push_notice(BillingPlayerManagerNotice::WorkerUnavailable {
                        worker: BillingPlayerWorkerKind::CashLog,
                        reason: error.to_string(),
                    });
            }
        }
        true
    }

    /// Создаёт один owned аналог элемента `CGame::vecBPM`.
    pub(crate) fn create_thread(&mut self) -> Result<(), CreateBillingPlayerWorkerError> {
        let mut database = TiberiusRsPlayerAccount::new(self.database_settings.clone())
            .map_err(CreateBillingPlayerWorkerError::DatabaseOwner)?;
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop_requested);
        let queues = Arc::clone(&self.queues);
        let sender = self.sender.clone();
        let log_server_enabled = Arc::clone(&self.log_server_enabled);
        let worker = thread::Builder::new()
            .name("billing-database".to_owned())
            .spawn(move || {
                loop {
                    if !queues.run(&worker_stop, &mut database, &sender, &log_server_enabled) {
                        break;
                    }
                    thread::sleep(DATABASE_WORKER_CADENCE);
                }
            })
            .map_err(CreateBillingPlayerWorkerError::Spawn)?;
        self.database_workers.push(DatabaseWorker {
            stop_requested,
            thread: worker,
        });
        Ok(())
    }

    /// Последовательно публикует stop и ждёт каждый DB-worker, как `Release`.
    pub(crate) fn release(&mut self) {
        let workers = mem::take(&mut self.database_workers);
        for worker in workers {
            worker.stop_requested.store(true, Ordering::Release);
            if worker.thread.join().is_err() {
                self.queues
                    .push_notice(BillingPlayerManagerNotice::WorkerPanicked {
                        worker: BillingPlayerWorkerKind::Database,
                    });
            }
        }
    }

    /// Ждёт optional log-worker только при текущем `bLogSvrSwitch`, как `End`.
    pub(crate) fn end(&mut self, log_enabled_at_end: bool) -> bool {
        if log_enabled_at_end
            && let Some(worker) = self.log_worker.take()
            && worker.join().is_err()
        {
            self.queues
                .push_notice(BillingPlayerManagerNotice::WorkerPanicked {
                    worker: BillingPlayerWorkerKind::CashLog,
                });
        }
        true
    }
}

fn add_transaction_code(message: &mut CMessage, transaction_code: Vec<u8>) {
    let transaction_code = CString::new(transaction_code)
        .expect("CRsPlayerAccount обрезает transaction code по первому NUL");
    message.base_mut().add_str(Some(&transaction_code));
}
