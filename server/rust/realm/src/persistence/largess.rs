//! DB-владелец и worker подарков `CLargess` WorldServer из `largess.cpp`:
//! data-семья исходов очередей
//! transfer/cycle-load, настройки Cost DB, notice-семья сохранения, трейт
//! `LargessOwner` и Tiberius-реализация `TiberiusLargess` с load-веткой
//! `LoadLargess`/`AddOneLargess` и worker lifecycle (`StartWorkerThread`).
//! Источник контракта — та же точная пара, что у [`crate::persistence::savedb`].
//!
//! Owner сохраняет lifecycle Init/UnInit, очереди transfer/cycle-load, порядок
//! `AddOneLargess`, календарные поля и обе формы `SaveLoadDetails`. Worker
//! переносит записи между очередями в исходном порядке; ошибка соединения или
//! команды не получает выдуманного rollback. Mutex/threads, Tiberius и owned
//! records заменяют Win32/ADO/STL, сохраняя locks, partial success и shutdown.
//!
//! Load-ветка и worker разделяют очереди `m_mapLargess` и обе настройки Cost
//! DB и поэтому живут одним структурным владельцем; смежные типы — `CPlayer`
//! (`characters/player`), `CGoods` и фабрика товаров (`content/`). Chrono
//! заменяет локальное Win32-время, `std::thread` +
//! `JoinHandle` — жизненный цикл worker-а.
//!
//! Async-методы трейта записаны в desugared-форме по ADR-0013: реализация
//! держит эквивалент `CriticalSectionmapLargess` (`parking_lot::MutexGuard`,
//! `!Send`) захваченным поверх DB-вызовов, как исходная critical section
//! охватывала connect, оба SQL и erase. Generic-потребитель `L: LargessOwner`
//! сохраняющего pipeline не требует Send и получает тот же `?Send`-контракт,
//! что давал исходный `async fn` трейта.
//!
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use chrono::{Datelike, Local, NaiveDateTime, Timelike};
use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use parking_lot::Mutex;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, Query, Row};
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use nebokrai_shared::values::CGuid;

use crate::characters::player::{CPlayer, PlayerCodecError};
use crate::content::cgoods::{CGoods, GAP_GOODS_PACKAGE_EXTENTION};
use crate::content::cgoodsfactory::create_goods;
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::writelog::LargessWriteLog;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendLargessOutcome {
    Inserted,
    DuplicateSendId,
    ExistingPlayerKept,
}

#[derive(Debug)]
pub enum CycleLoadLargessFailure {
    Connection(LargessDatabaseError),
    Database {
        row_index: usize,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        row_index: usize,
        column: &'static str,
        value: i64,
    },
}

#[derive(Debug)]
pub enum CycleLoadLargessOutcome {
    ReturnedTrue {
        row_count: usize,
        inserted: usize,
        duplicate_send_ids: usize,
        existing_players_kept: usize,
    },
    ReturnedFalse(CycleLoadLargessFailure),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferLargessDatabaseStage {
    BeginWorkingTransaction,
    InsertWorkingRow,
    MarkIncomingRowProcessed,
    CommitWorkingTransaction,
}

#[derive(Debug)]
pub enum TransferLargessFailure {
    IncomingConnection(LargessDatabaseError),
    WorkingConnection(LargessDatabaseError),
    IncomingQuery(tiberius::error::Error),
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        row_index: usize,
        column: &'static str,
        value: i64,
    },
    IncomingRow {
        row_index: usize,
        column: &'static str,
        source: tiberius::error::Error,
    },
    Database {
        row_index: usize,
        stage: TransferLargessDatabaseStage,
        source: tiberius::error::Error,
        rollback: Option<tiberius::error::Error>,
    },
}

#[derive(Debug)]
pub enum TransferLargessOutcome {
    ReturnedTrue { row_count: usize },
    ReturnedFalse(TransferLargessFailure),
}

#[derive(Debug)]
pub struct LargessWorkerReport {
    pub transfer: TransferLargessOutcome,
    pub cycle_load: CycleLoadLargessOutcome,
}

#[derive(Debug)]
pub enum LargessWorkerCompletion {
    Returned(LargessWorkerReport),
    Panicked,
}

#[derive(Debug)]
pub enum LargessWorkerStartOutcome {
    Disabled,
    Busy,
    Started {
        previous: Option<LargessWorkerCompletion>,
    },
    MissingRuntime {
        previous: Option<LargessWorkerCompletion>,
    },
    SpawnFailed {
        previous: Option<LargessWorkerCompletion>,
        source: io::Error,
    },
}

/// Настройки одной Cost DB: incoming (входящие назначения мира) и working
/// (рабочая база выдачи). Отдельная пара от `WorldDatabaseSettings`, потому
/// что transfer ходит в обе базы по своим строкам подключения.
#[derive(Clone)]
pub struct CostDatabaseSettings {
    _provider: Vec<u8>,
    host: Vec<u8>,
    database: Vec<u8>,
    user: Vec<u8>,
    password: Vec<u8>,
}

pub struct CostDatabaseSettingsParts {
    pub provider: Vec<u8>,
    pub host: Vec<u8>,
    pub database: Vec<u8>,
    pub user: Vec<u8>,
    pub password: Vec<u8>,
}

impl CostDatabaseSettings {
    pub fn from_parts(parts: CostDatabaseSettingsParts) -> Self {
        Self {
            _provider: parts.provider,
            host: parts.host,
            database: parts.database,
            user: parts.user,
            password: parts.password,
        }
    }

    pub fn tds_config(&self) -> Config {
        let mut config = Config::new();
        config.host(decode_ansi_c_string(&self.host));
        config.database(decode_ansi_c_string(&self.database));
        config.authentication(AuthMethod::sql_server(
            decode_ansi_c_string(&self.user),
            decode_ansi_c_string(&self.password),
        ));
        config.encryption(EncryptionLevel::NotSupported);
        config
    }
}

#[derive(Clone)]
pub struct LargessSnapshot {
    pub send_id: i32,
    pub goods_index: u32,
    pub send_num: i32,
    pub obtained_num: i32,
    pub goods_level: i32,
    pub sent_time: Vec<u8>,
    pub failed_reason: Vec<u8>,
}

/// ANSI cd-key, который не должен попадать в Debug-вывод notice. Поле открыто,
/// чтобы старая реализация конструировала значение буквальной tuple-формой.
#[derive(Clone)]
pub struct SensitiveCdKey(pub Vec<u8>);

impl SensitiveCdKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SensitiveCdKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SensitiveCdKey(<скрыто>)")
    }
}

#[derive(Debug)]
pub enum SaveLoadDetailsOutcome {
    ReturnedTrue,
    ReturnedFalse,
}

#[derive(Debug)]
pub enum LargessNotice {
    AddPresentDetail {
        cd_key: SensitiveCdKey,
        send_id: i32,
        obtained_num: i32,
        error: LargessDatabaseError,
    },
    UpdateObtainedNum {
        cd_key: SensitiveCdKey,
        send_id: i32,
        player_id: i32,
        obtained_num: i32,
        error: LargessDatabaseError,
    },
    SaveLoadDetails(LargessDatabaseError),
}

#[derive(Debug)]
pub enum LargessDatabaseError {
    MissingConnection,
    Connect(io::Error),
    Tds(tiberius::error::Error),
}

impl fmt::Display for LargessDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConnection => formatter.write_str("не передано соединение Cost DB"),
            Self::Connect(error) => write!(formatter, "не установлено соединение Cost DB: {error}"),
            Self::Tds(error) => write!(formatter, "ошибка TDS Cost DB: {error}"),
        }
    }
}

impl Error for LargessDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingConnection => None,
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
        }
    }
}

impl From<tiberius::error::Error> for LargessDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

pub trait LargessOwner {
    fn save_load_details_with_connection(
        &mut self,
        cd_key: &[u8],
        player_id: i32,
        connection: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = SaveLoadDetailsOutcome>;

    fn save_load_details(
        &mut self,
        cd_key: &[u8],
        player_id: i32,
    ) -> impl std::future::Future<Output = SaveLoadDetailsOutcome>;

    fn pop_notice(&mut self) -> Option<LargessNotice>;
}

fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn decode_ansi_c_string(bytes: &[u8]) -> String {
    let (decoded, _, _) = WINDOWS_1251.decode(c_string_prefix(bytes));
    decoded.into_owned()
}

const ERROR_GOODS_ID: &[u8] = b"error goodsID!";
const LARGESS_DEPOT_EXTENSION_FIRST_POSITION: u32 = 0x60;
const LARGESS_DEPOT_EXTENSION_STRIDE: u32 = 0x0D;
const LARGESS_DEPOT_EXTENSION_END: u32 = 0xA1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LargessDepotAddOutcome {
    Added { position: u32 },
    Rejected { position: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadLargessCompletion {
    Disabled,
    EmptyAccount,
    MissingEntry,
    AlreadyComplete,
    Processed,
}

#[derive(Clone, Debug)]
pub struct LoadLargessReport {
    pub completion: LoadLargessCompletion,
    pub charged: bool,
    pub current_sent_num: i32,
    pub obtained_num: Option<i32>,
    pub write_log: Option<LargessWriteLog>,
}

#[derive(Debug)]
pub enum LoadLargessBlock {
    Player(PlayerCodecError),
    Guid(getrandom::Error),
    ZeroMaximumStack { goods_index: u32 },
}

pub fn add_gold_coin(
    player: &mut CPlayer,
    goods: Box<CGoods>,
    gold_coin_limit: u32,
) -> Result<bool, PlayerCodecError> {
    player.add_largess_gold_coin(goods, gold_coin_limit)
}

pub fn add_one_largess(
    player: &mut CPlayer,
    goods: Box<CGoods>,
    registry: &GoodsBasePropertiesRegistry,
) -> Result<LargessDepotAddOutcome, PlayerCodecError> {
    let limit = player.largess_depot_limit();
    let extension_enabled =
        goods.get_addon_property_value(GAP_GOODS_PACKAGE_EXTENTION, 1) == 1;
    let mut goods = Some(goods);
    let mut position = 0_u32;
    while position < limit {
        let extension_cell = position >= LARGESS_DEPOT_EXTENSION_FIRST_POSITION
            && position < LARGESS_DEPOT_EXTENSION_END
            && (position - LARGESS_DEPOT_EXTENSION_FIRST_POSITION)
                % LARGESS_DEPOT_EXTENSION_STRIDE
                == 0;
        if !extension_cell || extension_enabled {
            goods = player.add_largess_to_depot(
                position,
                goods.take().expect("товар жив до успешной depot-вставки"),
                registry,
            )?;
            if goods.is_none() {
                return Ok(LargessDepotAddOutcome::Added { position });
            }
        }
        position = position.wrapping_add(1);
    }
    Ok(LargessDepotAddOutcome::Rejected { position: limit })
}

/// Linux/TDS-реализация `CLargess`: общие очереди `m_mapLargess`, обе
/// настройки Cost DB, load-ветка и worker lifecycle, перенесённые из старого
/// пакета без изменения порядка записей, partial success и shutdown.
pub struct TiberiusLargess {
    load_largess_time: u32,
    incoming_cost_database: CostDatabaseSettings,
    cost_database: CostDatabaseSettings,
    entries: Arc<Mutex<BTreeMap<i32, LargessSnapshot>>>,
    notices: VecDeque<LargessNotice>,
    worker: Mutex<Option<JoinHandle<LargessWorkerReport>>>,
}

impl TiberiusLargess {
 /// Принимает начальный map snapshot; дальнейшее наполнение выполняют
 /// готовые `AppendLargessToMap` и `CycleLoadLargessThread` owners.
    pub fn new(
        load_largess_time: u32,
        incoming_cost_database: CostDatabaseSettings,
        cost_database: CostDatabaseSettings,
        entries: BTreeMap<i32, LargessSnapshot>,
    ) -> Self {
        Self {
            load_largess_time,
            incoming_cost_database,
            cost_database,
            entries: Arc::new(Mutex::new(entries)),
            notices: VecDeque::new(),
            worker: Mutex::new(None),
        }
    }

    pub fn clone_save_owner(&self) -> Self {
        Self {
            load_largess_time: self.load_largess_time,
            incoming_cost_database: self.incoming_cost_database.clone(),
            cost_database: self.cost_database.clone(),
            entries: Arc::clone(&self.entries),
            notices: VecDeque::new(),
            worker: Mutex::new(None),
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.lock().len()
    }

 /// Эквивалент `StartWorkerThread`: пропускает запуск при нулевом интервале
 /// и пока предыдущий проход ещё владеет worker-slot.
    pub fn start_worker(&self, world_number: u32) -> LargessWorkerStartOutcome {
        if self.load_largess_time == 0 {
            return LargessWorkerStartOutcome::Disabled;
        }

        let mut worker = self.worker.lock();
        if worker.as_ref().is_some_and(|handle| !handle.is_finished()) {
            return LargessWorkerStartOutcome::Busy;
        }
        let previous = worker.take().map(join_largess_worker);
        let runtime = match Handle::try_current() {
            Ok(runtime) => runtime,
            Err(_) => return LargessWorkerStartOutcome::MissingRuntime { previous },
        };
        let load_largess_time = self.load_largess_time;
        let incoming_cost_database = self.incoming_cost_database.clone();
        let cost_database = self.cost_database.clone();
        let entries = Arc::clone(&self.entries);
        match thread::Builder::new()
            .name("world-largess".to_owned())
            .spawn(move || {
                let owner = TiberiusLargess {
                    load_largess_time,
                    incoming_cost_database,
                    cost_database,
                    entries,
                    notices: VecDeque::new(),
                    worker: Mutex::new(None),
                };
                runtime.block_on(async move {
                    let transfer = owner.transfer_largess(world_number).await;
                    let cycle_load = owner.cycle_load_largess().await;
                    LargessWorkerReport {
                        transfer,
                        cycle_load,
                    }
                })
            })
        {
            Ok(handle) => {
                *worker = Some(handle);
                LargessWorkerStartOutcome::Started { previous }
            }
            Err(source) => LargessWorkerStartOutcome::SpawnFailed { previous, source },
        }
    }

    pub fn wait_for_worker(&self) -> Option<LargessWorkerCompletion> {
        self.worker.lock().take().map(join_largess_worker)
    }

 /// Переносит входящие назначения текущего World в рабочую Cost DB.
 ///
 /// Tiberius-параметры заменяют небезопасные `_sprintf` SQL-буферы, но
 /// сохраняют исходный порядок: target BEGIN/INSERT, source IsProcessed=1,
 /// затем target COMMIT. Поэтому исходное окно потери при ошибке COMMIT
 /// после успешного source UPDATE намеренно не маскируется новой общей
 /// транзакцией между двумя базами.
    pub async fn transfer_largess(&self, world_number: u32) -> TransferLargessOutcome {
        const SELECT_INCOMING_LARGESS: &str =
            "SELECT * FROM Largess WHERE WorldID=@P1 AND IsProcessed=0";

        let mut incoming = match Self::connect_cost_database(
            self.incoming_cost_database.tds_config(),
        )
        .await
        {
            Ok(connection) => connection,
            Err(source) => {
                return TransferLargessOutcome::ReturnedFalse(
                    TransferLargessFailure::IncomingConnection(source),
                );
            }
        };
        let mut working =
            match Self::connect_cost_database(self.cost_database.tds_config()).await {
                Ok(connection) => connection,
                Err(source) => {
                    return TransferLargessOutcome::ReturnedFalse(
                        TransferLargessFailure::WorkingConnection(source),
                    );
                }
            };

        let mut select = Query::new(SELECT_INCOMING_LARGESS);
        select.bind(world_number as i32);
        let rows = match select.query(&mut incoming).await {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(source) => {
                    return TransferLargessOutcome::ReturnedFalse(
                        TransferLargessFailure::IncomingQuery(source),
                    );
                }
            },
            Err(source) => {
                return TransferLargessOutcome::ReturnedFalse(
                    TransferLargessFailure::IncomingQuery(source),
                );
            }
        };

        let mut row_count = 0usize;
        for row in rows {
            let transfer = match TransferLargessRow::read(&row, row_count) {
                Ok(transfer) => transfer,
                Err(source) => return TransferLargessOutcome::ReturnedFalse(source),
            };
            if let Err(source) = working.simple_query("BEGIN TRANSACTION").await {
                return TransferLargessOutcome::ReturnedFalse(
                    TransferLargessFailure::Database {
                        row_index: row_count,
                        stage: TransferLargessDatabaseStage::BeginWorkingTransaction,
                        source,
                        rollback: None,
                    },
                );
            }

            if let Err(source) = insert_transferred_largess(&mut working, &transfer).await {
                let rollback = rollback_working_largess(&mut working).await.err();
                return TransferLargessOutcome::ReturnedFalse(
                    TransferLargessFailure::Database {
                        row_index: row_count,
                        stage: TransferLargessDatabaseStage::InsertWorkingRow,
                        source,
                        rollback,
                    },
                );
            }
            if let Err(source) = mark_incoming_largess_processed(&mut incoming, transfer.send_id).await
            {
                let rollback = rollback_working_largess(&mut working).await.err();
                return TransferLargessOutcome::ReturnedFalse(
                    TransferLargessFailure::Database {
                        row_index: row_count,
                        stage: TransferLargessDatabaseStage::MarkIncomingRowProcessed,
                        source,
                        rollback,
                    },
                );
            }
            let commit = working
                .simple_query("COMMIT TRANSACTION")
                .await
                .map(|_| ());
            if let Err(source) = commit {
                let rollback = rollback_working_largess(&mut working).await.err();
                return TransferLargessOutcome::ReturnedFalse(
                    TransferLargessFailure::Database {
                        row_index: row_count,
                        stage: TransferLargessDatabaseStage::CommitWorkingTransaction,
                        source,
                        rollback,
                    },
                );
            }
            row_count += 1;
        }

        TransferLargessOutcome::ReturnedTrue { row_count }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append_largess_to_map(
        &self,
        send_id: i32,
        goods_index: u32,
        send_num: i32,
        obtained_num: i32,
        goods_level: i32,
        player_id: i32,
    ) -> AppendLargessOutcome {
        let mut entries = self.entries.lock();
        append_largess_entry(
            &mut entries,
            send_id,
            goods_index,
            send_num,
            obtained_num,
            goods_level,
            player_id,
        )
    }

    #[allow(
        clippy::await_holding_lock,
        reason = "exact CriticalSectionmapLargess охватывал connect, recordset и весь row-loop"
    )]
    pub async fn cycle_load_largess(&self) -> CycleLoadLargessOutcome {
        const SELECT_PENDING_LARGESS: &str =
            "SELECT * FROM Largess WHERE SendNum>ObtainedNum or ObtainedNum is NULL";

        let config = self.cost_database.tds_config();
        let mut entries = self.entries.lock();
        let mut connection = match Self::connect_cost_database(config).await {
            Ok(connection) => connection,
            Err(source) => {
                return CycleLoadLargessOutcome::ReturnedFalse(
                    CycleLoadLargessFailure::Connection(source),
                );
            }
        };
        let mut rows = match connection.query(SELECT_PENDING_LARGESS, &[]).await {
            Ok(rows) => rows,
            Err(source) => {
                return CycleLoadLargessOutcome::ReturnedFalse(
                    CycleLoadLargessFailure::Database {
                        row_index: 0,
                        source,
                    },
                );
            }
        };

        let mut row_count = 0usize;
        let mut inserted = 0usize;
        let mut duplicate_send_ids = 0usize;
        let mut existing_players_kept = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return CycleLoadLargessOutcome::ReturnedFalse(
                        CycleLoadLargessFailure::Database {
                            row_index: row_count,
                            source,
                        },
                    );
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let send_id = match required_largess_i32(&row, "SendID", row_count) {
                Ok(value) => value,
                Err(source) => return CycleLoadLargessOutcome::ReturnedFalse(source),
            };
            let send_num = match required_largess_i32(&row, "SendNum", row_count) {
                Ok(value) => value,
                Err(source) => return CycleLoadLargessOutcome::ReturnedFalse(source),
            };
            let goods_index = match required_largess_i32(&row, "GoodsIndex", row_count) {
                Ok(value) => value as u32,
                Err(source) => return CycleLoadLargessOutcome::ReturnedFalse(source),
            };
            let goods_level = match optional_largess_i32(&row, "GoodsLevel", row_count) {
                Ok(Some(value)) => value,
                Ok(None) => -1,
                Err(source) => return CycleLoadLargessOutcome::ReturnedFalse(source),
            };
            let player_id = match required_largess_i32(&row, "PlayerId", row_count) {
                Ok(value) => value,
                Err(source) => return CycleLoadLargessOutcome::ReturnedFalse(source),
            };

 // Оригинал owner читал/lowercase-ил Cdkey, но значение не покидало
 // локальный string и не участвовало ни в одном side effect.
            match append_largess_entry(
                &mut entries,
                send_id,
                goods_index,
                send_num,
                0,
                goods_level,
                player_id,
            ) {
                AppendLargessOutcome::Inserted => inserted += 1,
                AppendLargessOutcome::DuplicateSendId => duplicate_send_ids += 1,
                AppendLargessOutcome::ExistingPlayerKept => existing_players_kept += 1,
            }
            row_count += 1;
        }

        CycleLoadLargessOutcome::ReturnedTrue {
            row_count,
            inserted,
            duplicate_send_ids,
            existing_players_kept,
        }
    }

    async fn connect_cost_database(config: Config) -> Result<WorldTdsClient, LargessDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(LargessDatabaseError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(LargessDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(LargessDatabaseError::Tds)
    }

    pub fn load_largess<Random, Upgrade>(
        &self,
        player: &mut CPlayer,
        registry: &GoodsBasePropertiesRegistry,
        gold_coin_index: u32,
        gold_coin_limit: u32,
        use_log_system: bool,
        random: &mut Random,
        upgrade_equipment: &mut Upgrade,
    ) -> Result<LoadLargessReport, LoadLargessBlock>
    where
        Random: FnMut(i32) -> i32 + ?Sized,
        Upgrade: FnMut(&mut CGoods, i32) + ?Sized,
    {
        let unchanged = |completion| LoadLargessReport {
            completion,
            charged: false,
            current_sent_num: 0,
            obtained_num: None,
            write_log: None,
        };
        if self.load_largess_time == 0 {
            return Ok(unchanged(LoadLargessCompletion::Disabled));
        }
        let account = c_string_prefix(player.get_account()).to_vec();
        if account.is_empty() {
            return Ok(unchanged(LoadLargessCompletion::EmptyAccount));
        }
        let player_id = player.get_id();
        let mut entries = self.entries.lock();
        let Some(entry) = entries.get_mut(&player_id) else {
            return Ok(unchanged(LoadLargessCompletion::MissingEntry));
        };

        player.mark_largess_charged();
        let remaining = entry.send_num.wrapping_sub(entry.obtained_num) as u32;
        if remaining == 0 {
            return Ok(LoadLargessReport {
                completion: LoadLargessCompletion::AlreadyComplete,
                charged: true,
                current_sent_num: 0,
                obtained_num: Some(entry.obtained_num),
                write_log: None,
            });
        }

        let mut send_time = Vec::new();
        let mut goods_id = Vec::new();
        let mut goods_name = Vec::new();
        let mut sent_num = 0_i32;
        let mut current_sent_num = 0_i32;
        let mut result = Vec::new();

        if entry.goods_index == gold_coin_index {
            let Some(mut goods) = create_goods(registry, entry.goods_index, random) else {
                send_time = format_local_time().into_bytes();
                entry.sent_time.clone_from(&send_time);
                entry.failed_reason.clear();
                entry.failed_reason.extend_from_slice(ERROR_GOODS_ID);
                result.clone_from(&entry.failed_reason);
                return Ok(finish_largess_load(
                    entry,
                    account,
                    player_id,
                    send_time,
                    goods_id,
                    goods_name,
                    sent_num,
                    current_sent_num,
                    result,
                    use_log_system,
                ));
            };
            goods.set_amount(remaining);
            goods_name = goods.get_goods_name().to_vec();
            goods_id = goods.get_ex_id().to_string().into_bytes();
            if add_gold_coin(player, goods, gold_coin_limit)
                .map_err(LoadLargessBlock::Player)?
            {
 // Оригинал gold-ветка пишет literal `1`, а не количество монет.
                current_sent_num = 1;
                entry.obtained_num = entry.send_num;
                entry.failed_reason.clear();
                entry.failed_reason.extend_from_slice(b"money OK");
                send_time = format_local_time().into_bytes();
                result.clone_from(&entry.failed_reason);
                sent_num = entry.obtained_num;
            } else {
                send_time = format_local_time().into_bytes();
                entry.sent_time.clone_from(&send_time);
                entry.failed_reason.clear();
                entry
                    .failed_reason
                    .extend_from_slice(b"Money is excess for bank");
                result.clone_from(&entry.failed_reason);
                sent_num = entry.obtained_num;
            }
        } else {
            let mut prepared_num = 0_u32;
            while prepared_num < remaining {
                let Some(mut goods) = create_goods(registry, entry.goods_index, random) else {
                    goods_id.clear();
                    goods_name.clear();
                    send_time = format_local_time().into_bytes();
                    entry.sent_time.clone_from(&send_time);
                    entry.failed_reason.clear();
                    entry.failed_reason.extend_from_slice(ERROR_GOODS_ID);
                    result.clone_from(&entry.failed_reason);
                    sent_num = 0;
                    break;
                };
                let maximum_stack = goods
                    .get_max_stack_number(registry)
                    .map_err(|source| LoadLargessBlock::Player(source.into()))?;
                if maximum_stack == 0 {
                    return Err(LoadLargessBlock::ZeroMaximumStack {
                        goods_index: entry.goods_index,
                    });
                }
                let amount = maximum_stack.min(remaining.wrapping_sub(prepared_num));
                prepared_num = prepared_num.wrapping_add(amount);
                if amount > 1 {
                    goods.set_amount(amount);
                }
                let guid = CGuid::create().map_err(LoadLargessBlock::Guid)?;
                goods.set_ex_id(&guid);
                if entry.goods_level > 0 {
                    upgrade_equipment(&mut goods, entry.goods_level);
                }
                goods_name = goods.get_goods_name().to_vec();
                goods_id = goods.get_ex_id().to_string().into_bytes();
                match add_one_largess(player, goods, registry)
                    .map_err(LoadLargessBlock::Player)?
                {
                    LargessDepotAddOutcome::Added { .. } => {
                        current_sent_num = current_sent_num.wrapping_add(amount as i32);
                        entry.obtained_num = entry.obtained_num.wrapping_add(amount as i32);
                        send_time = format_local_time().into_bytes();
                        entry.sent_time.clone_from(&send_time);
                        entry.failed_reason.clear();
                        entry.failed_reason.extend_from_slice(b"goods OK");
                        result.clone_from(&entry.failed_reason);
                        sent_num = entry.obtained_num;
                    }
                    LargessDepotAddOutcome::Rejected { .. } => {
                        send_time = format_local_time().into_bytes();
                        entry.sent_time.clone_from(&send_time);
                        entry.failed_reason.clear();
                        entry
                            .failed_reason
                            .extend_from_slice(b"Depot space is not enough");
                        result.clone_from(&entry.failed_reason);
                        sent_num = entry.obtained_num;
                        break;
                    }
                }
            }
        }

        Ok(finish_largess_load(
            entry,
            account,
            player_id,
            send_time,
            goods_id,
            goods_name,
            sent_num,
            current_sent_num,
            result,
            use_log_system,
        ))
    }
}

impl Drop for TiberiusLargess {
    fn drop(&mut self) {
        if let Some(handle) = self.worker.get_mut().take() {
            let _ = join_largess_worker(handle);
        }
    }
}

fn join_largess_worker(handle: JoinHandle<LargessWorkerReport>) -> LargessWorkerCompletion {
    match handle.join() {
        Ok(report) => LargessWorkerCompletion::Returned(report),
        Err(_) => LargessWorkerCompletion::Panicked,
    }
}

struct TransferLargessRow {
    send_id: i32,
    cd_key: String,
    player_id: i32,
    goods_index: u32,
    goods_name: String,
    goods_level: i32,
    send_num: i32,
    world_id: i32,
    send_time: String,
}

impl TransferLargessRow {
    fn read(row: &Row, row_index: usize) -> Result<Self, TransferLargessFailure> {
        Ok(Self {
            send_id: required_transfer_i32(row, "SendID", row_index)?,
            cd_key: required_transfer_text(row, "Cdkey", row_index)?,
            player_id: required_transfer_i32(row, "PlayerId", row_index)?,
            goods_index: required_transfer_u32(row, "GoodsIndex", row_index)?,
            goods_name: required_transfer_text(row, "GoodsName", row_index)?,
            goods_level: required_transfer_i32(row, "GoodsLevel", row_index)?,
            send_num: required_transfer_i32(row, "SendNum", row_index)?,
            world_id: required_transfer_i32(row, "WorldID", row_index)?,
            send_time: required_transfer_send_time(row, row_index)?,
        })
    }
}

async fn insert_transferred_largess(
    working: &mut WorldTdsClient,
    row: &TransferLargessRow,
) -> Result<(), tiberius::error::Error> {
    const INSERT_WORKING_LARGESS: &str =
        "INSERT INTO Largess(SendID,Cdkey,PlayerId,GoodsIndex,GoodsName,GoodsLevel,SendNum,WorldID,SendTime,ObtainedNum) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,0)";

    let mut insert = Query::new(INSERT_WORKING_LARGESS);
    insert.bind(row.send_id);
    insert.bind(row.cd_key.as_str());
    insert.bind(row.player_id);
    insert.bind(i64::from(row.goods_index));
    insert.bind(row.goods_name.as_str());
    insert.bind(row.goods_level);
    insert.bind(row.send_num);
    insert.bind(row.world_id);
    insert.bind(row.send_time.as_str());
    insert.execute(working).await.map(|_| ())
}

async fn mark_incoming_largess_processed(
    incoming: &mut WorldTdsClient,
    send_id: i32,
) -> Result<(), tiberius::error::Error> {
 // LoginDB schema-аудит поздней Rust-ветки подтверждает identity/PK SendID;
 // это безопасный эквивалент ADO Recordset::Fields[IsProcessed]=1; Update().
    let mut update = Query::new(
        "UPDATE Largess SET IsProcessed=1 WHERE SendID=@P1 AND IsProcessed=0",
    );
    update.bind(send_id);
    update.execute(incoming).await.map(|_| ())
}

async fn rollback_working_largess(
    working: &mut WorldTdsClient,
) -> Result<(), tiberius::error::Error> {
    working
        .simple_query("IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION")
        .await
        .map(|_| ())
}

fn required_transfer_i32(
    row: &Row,
    column: &'static str,
    row_index: usize,
) -> Result<i32, TransferLargessFailure> {
    match row.try_get::<i32, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(TransferLargessFailure::MissingRequiredValue { row_index, column }),
        Err(first_error) => match row.try_get::<i64, _>(column) {
            Ok(Some(value)) => i32::try_from(value).map_err(|_| {
                TransferLargessFailure::NumericOutsideLegacyRange {
                    row_index,
                    column,
                    value,
                }
            }),
            Ok(None) => Err(TransferLargessFailure::MissingRequiredValue { row_index, column }),
            Err(_) => Err(TransferLargessFailure::IncomingRow {
                row_index,
                column,
                source: first_error,
            }),
        },
    }
}

fn required_transfer_u32(
    row: &Row,
    column: &'static str,
    row_index: usize,
) -> Result<u32, TransferLargessFailure> {
    match row.try_get::<i64, _>(column) {
        Ok(Some(value)) => u32::try_from(value).map_err(|_| {
            TransferLargessFailure::NumericOutsideLegacyRange {
                row_index,
                column,
                value,
            }
        }),
        Ok(None) => Err(TransferLargessFailure::MissingRequiredValue { row_index, column }),
        Err(first_error) => match row.try_get::<i32, _>(column) {
            Ok(Some(value)) => Ok(value as u32),
            Ok(None) => Err(TransferLargessFailure::MissingRequiredValue { row_index, column }),
            Err(_) => Err(TransferLargessFailure::IncomingRow {
                row_index,
                column,
                source: first_error,
            }),
        },
    }
}

fn required_transfer_text(
    row: &Row,
    column: &'static str,
    row_index: usize,
) -> Result<String, TransferLargessFailure> {
    match row.try_get::<&str, _>(column) {
        Ok(Some(value)) => Ok(value.to_owned()),
        Ok(None) => Err(TransferLargessFailure::MissingRequiredValue { row_index, column }),
        Err(source) => Err(TransferLargessFailure::IncomingRow {
            row_index,
            column,
            source,
        }),
    }
}

fn required_transfer_send_time(
    row: &Row,
    row_index: usize,
) -> Result<String, TransferLargessFailure> {
    const COLUMN: &str = "SendTime";
    match row.try_get::<NaiveDateTime, _>(COLUMN) {
        Ok(Some(value)) => Ok(value.to_string()),
        Ok(None) => Err(TransferLargessFailure::MissingRequiredValue {
            row_index,
            column: COLUMN,
        }),
        Err(first_error) => match row.try_get::<&str, _>(COLUMN) {
            Ok(Some(value)) => Ok(value.to_owned()),
            Ok(None) => Err(TransferLargessFailure::MissingRequiredValue {
                row_index,
                column: COLUMN,
            }),
            Err(_) => Err(TransferLargessFailure::IncomingRow {
                row_index,
                column: COLUMN,
                source: first_error,
            }),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn append_largess_entry(
    entries: &mut BTreeMap<i32, LargessSnapshot>,
    send_id: i32,
    goods_index: u32,
    send_num: i32,
    obtained_num: i32,
    goods_level: i32,
    player_id: i32,
) -> AppendLargessOutcome {
    if entries.values().any(|entry| entry.send_id == send_id) {
        return AppendLargessOutcome::DuplicateSendId;
    }
    let entry = LargessSnapshot {
        send_id,
        goods_index,
        send_num,
        obtained_num,
        goods_level,
        sent_time: Vec::new(),
        failed_reason: Vec::new(),
    };
    match entries.entry(player_id) {
        std::collections::btree_map::Entry::Vacant(slot) => {
            slot.insert(entry);
            AppendLargessOutcome::Inserted
        }
        std::collections::btree_map::Entry::Occupied(_) => {
            AppendLargessOutcome::ExistingPlayerKept
        }
    }
}

fn optional_largess_i32(
    row: &Row,
    column: &'static str,
    row_index: usize,
) -> Result<Option<i32>, CycleLoadLargessFailure> {
    match row.try_get::<i32, _>(column) {
        Ok(value) => Ok(value),
        Err(first_error) => match row.try_get::<i64, _>(column) {
            Ok(Some(value)) => i32::try_from(value).map(Some).map_err(|_| {
                CycleLoadLargessFailure::NumericOutsideLegacyRange {
                    row_index,
                    column,
                    value,
                }
            }),
            Ok(None) => Ok(None),
            Err(_) => Err(CycleLoadLargessFailure::Database {
                row_index,
                source: first_error,
            }),
        },
    }
}

fn required_largess_i32(
    row: &Row,
    column: &'static str,
    row_index: usize,
) -> Result<i32, CycleLoadLargessFailure> {
    optional_largess_i32(row, column, row_index)?.ok_or(
        CycleLoadLargessFailure::MissingRequiredValue {
            row_index,
            column,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_largess_load(
    entry: &LargessSnapshot,
    account: Vec<u8>,
    player_id: i32,
    send_time: Vec<u8>,
    goods_id: Vec<u8>,
    goods_name: Vec<u8>,
    sent_num: i32,
    current_sent_num: i32,
    result: Vec<u8>,
    use_log_system: bool,
) -> LoadLargessReport {
    let write_log = (use_log_system && current_sent_num != 0).then(|| LargessWriteLog {
        account,
        player_id,
        send_time,
        goods_id,
        goods_index: entry.goods_index,
        goods_name,
        goods_level: entry.goods_level,
        send_num: entry.send_num,
        sent_num,
        current_sent_num,
        result,
    });
    LoadLargessReport {
        completion: LoadLargessCompletion::Processed,
        charged: true,
        current_sent_num,
        obtained_num: Some(entry.obtained_num),
        write_log,
    }
}

impl LargessOwner for TiberiusLargess {
    #[allow(
        clippy::await_holding_lock,
        reason = "exact CriticalSectionmapLargess охватывал оба DB-вызова и erase"
    )]
    async fn save_load_details_with_connection(
        &mut self,
        cd_key: &[u8],
        player_id: i32,
        mut connection: Option<&mut WorldTdsClient>,
    ) -> SaveLoadDetailsOutcome {
        if self.load_largess_time == 0 {
            return SaveLoadDetailsOutcome::ReturnedTrue;
        }

        let entries = &self.entries;
        let notices = &mut self.notices;
        let mut entries = entries.lock();
        if !entries.contains_key(&player_id) {
            return SaveLoadDetailsOutcome::ReturnedTrue;
        }

        save_matching_entry(&mut entries, notices, cd_key, player_id, &mut connection).await
    }

    #[allow(
        clippy::await_holding_lock,
        reason = "exact CriticalSectionmapLargess охватывал connect, оба DB-вызова и erase"
    )]
    async fn save_load_details(&mut self, cd_key: &[u8], player_id: i32) -> SaveLoadDetailsOutcome {
        if self.load_largess_time == 0 {
            return SaveLoadDetailsOutcome::ReturnedTrue;
        }

        let config = self.cost_database.tds_config();
        let entries = &self.entries;
        let notices = &mut self.notices;
        let mut entries = entries.lock();
        if !entries.contains_key(&player_id) {
            return SaveLoadDetailsOutcome::ReturnedTrue;
        }

        let mut connection = match Self::connect_cost_database(config).await {
            Ok(connection) => connection,
            Err(error) => {
                notices.push_back(LargessNotice::SaveLoadDetails(error));
                return SaveLoadDetailsOutcome::ReturnedFalse;
            }
        };
        let mut connection = Some(&mut connection);
        save_matching_entry(&mut entries, notices, cd_key, player_id, &mut connection).await
    }

    fn pop_notice(&mut self) -> Option<LargessNotice> {
        self.notices.pop_front()
    }
}

async fn save_matching_entry(
    entries: &mut BTreeMap<i32, LargessSnapshot>,
    notices: &mut VecDeque<LargessNotice>,
    cd_key: &[u8],
    player_id: i32,
    connection: &mut Option<&mut WorldTdsClient>,
) -> SaveLoadDetailsOutcome {
    let Some(entry) = entries.get(&player_id).cloned() else {
        return SaveLoadDetailsOutcome::ReturnedTrue;
    };
    if entry.failed_reason.is_empty() {
        return SaveLoadDetailsOutcome::ReturnedTrue;
    }

    let save_time = format_local_time();
    let insert_sql = build_insert_sql(&entry, player_id, &save_time);

    if let Err(error) = execute_optional(connection, &insert_sql).await {
        notices.push_back(LargessNotice::AddPresentDetail {
            cd_key: SensitiveCdKey(c_string_prefix(cd_key).to_vec()),
            send_id: entry.send_id,
            obtained_num: entry.obtained_num,
            error,
        });
    }

    let error_goods_id = entry.failed_reason.as_slice() == ERROR_GOODS_ID;
    let stored_obtained_num = if error_goods_id {
        9999
    } else {
        entry.obtained_num
    };
    let update_sql = format!(
        "UPDATE Largess SET ObtainedNum={stored_obtained_num} WHERE SendID={} and PlayerId={player_id}",
        entry.send_id
    );
    let update_succeeded = match execute_optional(connection, &update_sql).await {
        Ok(()) => true,
        Err(error) => {
            notices.push_back(LargessNotice::UpdateObtainedNum {
                cd_key: SensitiveCdKey(c_string_prefix(cd_key).to_vec()),
                send_id: entry.send_id,
                player_id,
                obtained_num: entry.obtained_num,
                error,
            });
            false
        }
    };

    if update_succeeded && (entry.send_num == entry.obtained_num || error_goods_id) {
        entries.remove(&player_id);
    }

    SaveLoadDetailsOutcome::ReturnedTrue
}

async fn execute_optional(
    connection: &mut Option<&mut WorldTdsClient>,
    sql: &str,
) -> Result<(), LargessDatabaseError> {
    let Some(connection) = connection.as_deref_mut() else {
        return Err(LargessDatabaseError::MissingConnection);
    };
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}

fn build_insert_sql(entry: &LargessSnapshot, player_id: i32, save_time: &str) -> String {
    let mut sql = format!(
        "INSERT INTO LoadDetails(SendID,PlayerId,ObtainedNum,LoadTime,Failedreason,SaveTime) VALUES({},{},{},'",
        entry.send_id, player_id, entry.obtained_num
    )
    .into_bytes();
    sql.extend_from_slice(c_string_prefix(&entry.sent_time));
    sql.extend_from_slice(b"','");
    sql.extend_from_slice(c_string_prefix(&entry.failed_reason));
    sql.extend_from_slice(b"','");
    sql.extend_from_slice(save_time.as_bytes());
    sql.extend_from_slice(b"')");

    let (decoded, _, _) = WINDOWS_1251.decode(&sql);
    decoded.into_owned()
}

fn format_local_time() -> String {
    let now = Local::now();
    format!(
        "{}-{}-{} {}:{}:{}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}
