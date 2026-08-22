//! Владелец `CLargess` исторического WorldServer из `largess.cpp`.
//!
//! Статус `Init` RVA `0x000E6630`, `UnInit` RVA `0x000E6650`, двух перегрузок
//! `SaveLoadDetails` RVA `0x000E8CA0` и
//! `0x000E91B0`, `GetTime` RVA `0x000E6800`, `AddGoldCoin` RVA `0x000E6690`,
//! `AddOneLargess` RVA `0x000E6A60`, `TransferLargessThread` RVA `0x000E7500`,
//! `AppendLargessToMap` RVA `0x000E9D40`, `CycleLoadLargessThread` RVA
//! `0x000E9FD0`, `WorkerThread` RVA `0x000EAC80` и `StartWorkerThread` RVA
//! `0x000EACC0` — `IMPLEMENTED`; остальной
//! корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp`.
//!
//! PDB задаёт `tagLargess` размером `0x4C`: пять последовательных 32-битных
//! полей `lSendID`, `dwGoodsIndex`, `lSendNum`, `lObtainedNum`, `lGoodsLevel`,
//! затем `std::string strSendedTime` по `+0x14` и `strFailedReason` по `+0x30`.
//! Статическая `std::map<long, tagLargess>` поэтому представлена
//! `BTreeMap<i32, LargessSnapshot>`, а `parking_lot::Mutex` заменяет только
//! `CriticalSectionmapLargess` без poisoning. Обе перегрузки держали critical
//! section от первого `equal_range` до последнего SQL, условного erase и
//! cleanup; Rust намеренно держит тот же lock через все `await`. Он защищал не
//! только память map, но и порядок конкурирующих save/load/mutation операций,
//! поэтому локальная копия записи не является основанием снять lock раньше.
//!
//! При `dwLoadLargessTime == 0`, отсутствующем player-key либо пустом
//! `strFailedReason` DB-вызова нет и исходный bool равен `true`. Для непустой
//! причины owner сначала вставляет `LoadDetails` с полями `SendID`, signed
//! player ID, `lObtainedNum`, C-string view `strSendedTime`, C-string view
//! `strFailedReason` и текущим local time без leading zeroes. Затем он всегда
//! пытается обновить `Largess`: для точного полного значения
//! `"error goodsID!"` пишет `9999`, иначе `lObtainedNum`. Ошибка INSERT
//! поглощается локальным catch и не пропускает UPDATE; ошибка UPDATE также
//! поглощается, но сохраняет map-entry. После успешного UPDATE запись удаляется
//! только когда `lSendNum == lObtainedNum` либо причина точно равна
//! `"error goodsID!"`; иначе остаётся для следующей попытки.
//!
//! Exact EXE имеет статус `VERIFIED_DISASSEMBLY` только для потерянных raw-
//! фактов. `0x004E8E1A..0x004E8E52` и `0x004E95F4..0x004E962C` подтвердили
//! шесть аргументов INSERT в указанном порядке; `0x004E8F73..0x004E8FC5` и
//! `0x004E974F..0x004E97A1` — special/обычный UPDATE. Выходы
//! `0x004E8CF8`, `0x004E8D70`, `0x004E9169..0x004E9192` и
//! `0x004E924E`, `0x004E9942..0x004E99DA` возвращают `true`, outer catch-и
//! `0x004E914F`/`0x004E9968` — `false`. После этих ответов reverse прекращён.
//!
//! Перегрузка с caller-connection использует уже открытое Cost DB соединение и
//! не начинает/завершает транзакцию. Даже null connection попадал в два
//! локальных catch-а и обычно завершался `true`, поэтому `Option` не превращён
//! в ранний fail-closed. Старая перегрузка под тем же lock самостоятельно
//! открывает отдельное Cost DB соединение из пяти setup-строк, закрывает его до
//! unlock и возвращает `false` при ошибке открытия. Tiberius/Tokio TCP заменяют
//! ADO provider/COM/BSTR; provider сохраняется в setup snapshot, но Linux TDS
//! не интерпретирует его. ANSI SQL декодируется как Windows-1251, выбранная для
//! этой поставки, а literal SQL сознательно сохраняет исходный parser-эффект
//! одинарных кавычек вместо параметризации.
//!
//! Оба INSERT собирались небезопасным `_sprintf` в `char[256]`. Если точные
//! C-string prefixes вместе с NUL требуют больше 256 байт, результат старого
//! stack-overflow неизвестен; локальный `BLOCKED_MISSING_FACT` не назначает ему
//! `false`, truncation либо продолжение. Текущий local time заведомо помещается
//! в `char[32]`, а оба UPDATE — в `char[256]`. `Vec`, `BTreeMap`, Rust `Drop`
//! и structured notices заменяют только `std::string`, MSVC tree, COM lifetime
//! и log-механику; CD-key в `Debug` намеренно скрыт. `LargessWriteLog` хранит
//! исходные одиннадцать значений, `CGame::publish_largess_load_log` ставит их
//! в общий FIFO, а write-log worker выполняет параметризованный INSERT.
//!
//! Два leaf-а выдачи также восстановлены. `AddGoldCoin` вызывает bank-wallet
//! на позиции `0`. `AddOneLargess` обходит весь inherited depot limit; только
//! ячейки `96,109,122,135,148` требуют addon `GAP_GOODS_PACKAGE_EXTENTION/1`
//! со значением `1`. Rejected `Box<CGoods>` освобождается Rust `Drop` вместо
//! virtual deleting destructor; container codec-ошибка остаётся typed block.
//! Exact `0x004E6AB0..0x004E6AE3` подтверждает gate и numeric type `0xEA`;
//! `0x004E6B12` возвращает `AL=1` после вставки. Failure-tail читает в
//! `0x004E6B7A` тот самый локальный byte, который `0x004E6A80` заранее
//! обнулил, поэтому заполненный depot стабильно даёт `false`, а не мусор.
//! `AppendLargessToMap` сначала линейно проверяет `lSendID` по всей карте и
//! только затем делает unique insert по player ID; ни один из двух duplicate-
//! случаев не заменяет старую запись. `BTreeMap::entry` сохраняет этот контракт.
//! Cycle-load держит тот же map-lock от открытия Cost DB до EOF, выполняет
//! literal query без ORDER BY и сразу публикует каждую строку через append-
//! owner. Поэтому поздняя DB/row ошибка сохраняет уже вставленный prefix, а
//! donor staging/swap не переносится. Прочитанный `Cdkey` и его `_strlwr`
//! удалены как мёртвая локальная работа; exact вызов append передаёт
//! `ObtainedNum=0` независимо от выбранной DB-строки, и этот quirk сохранён.
//! Worker остаётся одним короткоживущим проходом: стандартный `JoinHandle`
//! заменяет CRT handle, его живое состояние — `TryEnterCriticalSection`, а
//! тело всегда вызывает transfer и затем cycle независимо от первого bool.

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

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::worldserver::appworld::goods::cgoods::{
    CGoods, GAP_GOODS_PACKAGE_EXTENTION,
};
use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods,
};
use crate::worldserver::appworld::player::{CPlayer, PlayerCodecError};

const LEGACY_SQL_BUFFER_SIZE: usize = 256;
const ERROR_GOODS_ID: &[u8] = b"error goodsID!";
const LARGESS_DEPOT_EXTENSION_FIRST_POSITION: u32 = 0x60;
const LARGESS_DEPOT_EXTENSION_STRIDE: u32 = 0x0D;
const LARGESS_DEPOT_EXTENSION_END: u32 = 0xA1;

/// Exact bool и достигнутая позиция `CLargess::AddOneLargess`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LargessDepotAddOutcome {
    Added { position: u32 },
    Rejected { position: u32 },
}

/// Typed запись исходного `goods_largess_log`, ещё до transport/FIFO owner-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LargessWriteLog {
    pub(crate) account: Vec<u8>,
    pub(crate) player_id: i32,
    pub(crate) send_time: Vec<u8>,
    pub(crate) goods_id: Vec<u8>,
    pub(crate) goods_index: u32,
    pub(crate) goods_name: Vec<u8>,
    pub(crate) goods_level: i32,
    pub(crate) send_num: i32,
    pub(crate) sent_num: i32,
    pub(crate) current_sent_num: i32,
    pub(crate) result: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoadLargessCompletion {
    Disabled,
    EmptyAccount,
    MissingEntry,
    AlreadyComplete,
    Processed,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadLargessReport {
    pub(crate) completion: LoadLargessCompletion,
    pub(crate) charged: bool,
    pub(crate) current_sent_num: i32,
    pub(crate) obtained_num: Option<i32>,
    pub(crate) write_log: Option<LargessWriteLog>,
}

#[derive(Debug)]
pub(crate) enum LoadLargessBlock {
    Player(PlayerCodecError),
    Guid(getrandom::Error),
    ZeroMaximumStack { goods_index: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppendLargessOutcome {
    Inserted,
    DuplicateSendId,
    ExistingPlayerKept,
}

#[derive(Debug)]
pub(crate) enum CycleLoadLargessFailure {
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
pub(crate) enum CycleLoadLargessOutcome {
    ReturnedTrue {
        row_count: usize,
        inserted: usize,
        duplicate_send_ids: usize,
        existing_players_kept: usize,
    },
    ReturnedFalse(CycleLoadLargessFailure),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TransferLargessDatabaseStage {
    BeginWorkingTransaction,
    InsertWorkingRow,
    MarkIncomingRowProcessed,
    CommitWorkingTransaction,
}

#[derive(Debug)]
pub(crate) enum TransferLargessFailure {
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
pub(crate) enum TransferLargessOutcome {
    ReturnedTrue { row_count: usize },
    ReturnedFalse(TransferLargessFailure),
}

#[derive(Debug)]
pub(crate) struct LargessWorkerReport {
    pub(crate) transfer: TransferLargessOutcome,
    pub(crate) cycle_load: CycleLoadLargessOutcome,
}

#[derive(Debug)]
pub(crate) enum LargessWorkerCompletion {
    Returned(LargessWorkerReport),
    Panicked,
}

#[derive(Debug)]
pub(crate) enum LargessWorkerStartOutcome {
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

/// Делегирует `CLargess::AddGoldCoin` готовому bank-wallet owner-у.
pub(crate) fn add_gold_coin(
    player: &mut CPlayer,
    goods: Box<CGoods>,
    gold_coin_limit: u32,
) -> Result<bool, PlayerCodecError> {
    player.add_largess_gold_coin(goods, gold_coin_limit)
}

/// Повторяет positional scan и пять package-extension gates depot-а.
pub(crate) fn add_one_largess(
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

/// Пять исходных Cost DB setup-строк без публикации credentials.
#[derive(Clone)]
pub(crate) struct CostDatabaseSettings {
    _provider: Vec<u8>,
    host: Vec<u8>,
    database: Vec<u8>,
    user: Vec<u8>,
    password: Vec<u8>,
}

/// Владеющие части Cost DB setup snapshot.
pub(crate) struct CostDatabaseSettingsParts {
    pub(crate) provider: Vec<u8>,
    pub(crate) host: Vec<u8>,
    pub(crate) database: Vec<u8>,
    pub(crate) user: Vec<u8>,
    pub(crate) password: Vec<u8>,
}

impl CostDatabaseSettings {
    /// Сохраняет byte-exact `strCostDB*` поля старого `CSetup`.
    pub(crate) fn from_parts(parts: CostDatabaseSettingsParts) -> Self {
        Self {
            _provider: parts.provider,
            host: parts.host,
            database: parts.database,
            user: parts.user,
            password: parts.password,
        }
    }

    fn tds_config(&self) -> Config {
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

/// Полный достигнутый value-layout одного `tagLargess` без MSVC ABI.
#[derive(Clone)]
pub(crate) struct LargessSnapshot {
    pub(crate) send_id: i32,
    pub(crate) goods_index: u32,
    pub(crate) send_num: i32,
    pub(crate) obtained_num: i32,
    pub(crate) goods_level: i32,
    pub(crate) sent_time: Vec<u8>,
    pub(crate) failed_reason: Vec<u8>,
}

/// CD-key нужен будущему log-owner-у, но не раскрывается обычным `Debug`.
#[derive(Clone)]
pub(crate) struct SensitiveCdKey(Vec<u8>);

impl SensitiveCdKey {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SensitiveCdKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SensitiveCdKey(<скрыто>)")
    }
}

/// Один конкретный неизвестный результат старого `_sprintf` за `char[256]`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LargessSqlBufferBlock {
    pub(crate) required_bytes_with_nul: usize,
}

/// Наблюдаемый bool либо локальная небезопасная граница старого owner-а.
#[derive(Debug)]
pub(crate) enum SaveLoadDetailsOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(LargessSqlBufferBlock),
}

/// Структурированные эквиваленты трёх исходных log-ветвей.
#[derive(Debug)]
pub(crate) enum LargessNotice {
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

/// Ошибка достигнутой ADO/TDS-границы без SQL и credentials.
#[derive(Debug)]
pub(crate) enum LargessDatabaseError {
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

/// Узкая граница двух достигнутых перегрузок `CLargess::SaveLoadDetails`.
pub(crate) trait LargessOwner {
    /// Сохраняет одну map-запись на caller-owned Cost DB connection.
    async fn save_load_details_with_connection(
        &mut self,
        cd_key: &[u8],
        player_id: i32,
        connection: Option<&mut WorldTdsClient>,
    ) -> SaveLoadDetailsOutcome;

    /// Открывает отдельное Cost DB connection и сохраняет одну map-запись.
    async fn save_load_details(&mut self, cd_key: &[u8], player_id: i32) -> SaveLoadDetailsOutcome;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<LargessNotice>;
}

/// Linux/TDS-замена достигнутой части статического `CLargess`.
pub(crate) struct TiberiusLargess {
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
    pub(crate) fn new(
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

    /// Эквивалент `StartWorkerThread`: пропускает запуск при нулевом интервале
    /// и пока предыдущий проход ещё владеет worker-slot.
    pub(crate) fn start_worker(&self, world_number: u32) -> LargessWorkerStartOutcome {
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

    /// Эквивалент `UnInit`-ожидания единственного worker handle.
    pub(crate) fn wait_for_worker(&self) -> Option<LargessWorkerCompletion> {
        self.worker.lock().take().map(join_largess_worker)
    }

    /// Переносит входящие назначения текущего World в рабочую Cost DB.
    ///
    /// Tiberius-параметры заменяют небезопасные `_sprintf` SQL-буферы, но
    /// сохраняют исходный порядок: target BEGIN/INSERT, source IsProcessed=1,
    /// затем target COMMIT. Поэтому доказанное окно потери при ошибке COMMIT
    /// после успешного source UPDATE намеренно не маскируется новой общей
    /// транзакцией между двумя базами.
    pub(crate) async fn transfer_largess(&self, world_number: u32) -> TransferLargessOutcome {
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

    /// Повторяет global SendID scan и последующий unique player-key insert.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_largess_to_map(
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

    /// Выполняет один exact Cost DB polling-проход без staging/swap карты.
    #[allow(
        clippy::await_holding_lock,
        reason = "exact CriticalSectionmapLargess охватывал connect, recordset и весь row-loop"
    )]
    pub(crate) async fn cycle_load_largess(&self) -> CycleLoadLargessOutcome {
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

            // Exact owner читал/lowercase-ил Cdkey, но значение не покидало
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

    /// Выполняет одну синхронную выдачу `CLargess::LoadLargess` под map-lock.
    pub(crate) fn load_largess<Random, Upgrade>(
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
                // Exact gold-ветка пишет literal `1`, а не количество монет.
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
    let insert_sql = match build_insert_sql(&entry, player_id, &save_time) {
        Ok(sql) => sql,
        Err(block) => {
            // BLOCKED_MISSING_FACT: что наблюдалось после `_sprintf` за
            // `char[256]` в WorldServer RVA 0x000E8DC3/0x000E95DF?
            // До overflow map не изменялась и SQL ещё не выполнялся.
            return SaveLoadDetailsOutcome::BlockedMissingFact(block);
        }
    };

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

fn build_insert_sql(
    entry: &LargessSnapshot,
    player_id: i32,
    save_time: &str,
) -> Result<String, LargessSqlBufferBlock> {
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

    let required_bytes_with_nul = sql.len() + 1;
    if required_bytes_with_nul > LEGACY_SQL_BUFFER_SIZE {
        return Err(LargessSqlBufferBlock {
            required_bytes_with_nul,
        });
    }

    let (decoded, _, _) = WINDOWS_1251.decode(&sql);
    Ok(decoded.into_owned())
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp

// ============================================================================
// FUNCTION: CLargess::Init
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:30
// RVA: 0x000E6630
// ADDRESS: 004e6630
// PROTOTYPE: bool __cdecl Init(void)
//
// IMPLEMENTED_OWNER: `TiberiusLargess::new`; `parking_lot::Mutex` и owned
// `JoinHandle` не требуют ручной Win32-инициализации. Неиспользуемый больше
// нигде `csConnectSatus` не перенесён.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::UnInit
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:42
// RVA: 0x000E6650
// ADDRESS: 004e6650
// PROTOTYPE: bool __cdecl UnInit(void)
//
// IMPLEMENTED_OWNER: `TiberiusLargess::wait_for_worker` сохраняет бесконечное
// ожидание последнего worker-а; Rust RAII освобождает обе mutex после owner-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::AddGoldCoin
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:79
// RVA: 0x000E6690
// ADDRESS: 004e6690
// PROTOTYPE: bool __cdecl AddGoldCoin(CPlayer * param_1, CGoods * param_2)
//
// IMPLEMENTED_OWNER: `add_gold_coin` выше делегирует bank-wallet позиции `0`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::AddOneLargess
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:92
// RVA: 0x000E6A60
// ADDRESS: 004e6a60
// PROTOTYPE: bool __cdecl AddOneLargess(CPlayer * param_1, CGoods * param_2)
//
// IMPLEMENTED_OWNER: `add_one_largess` выше сохраняет полный positional scan.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::LoadLargess
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:417
// RVA: 0x000E6EC0
// ADDRESS: 004e6ec0
// PROTOTYPE: void __cdecl LoadLargess(CPlayer * param_1)
//
// IMPLEMENTED_OWNER: `TiberiusLargess::load_largess` выше; typed write-log
// публикуется через `CGame::publish_largess_load_log` в общий World FIFO.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::TransferLargessThread
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:715
// RVA: 0x000E7500
// ADDRESS: 004e7500
// PROTOTYPE: bool __cdecl TransferLargessThread(void)
//
// IMPLEMENTED_OWNER: `TiberiusLargess::transfer_largess` выше сохраняет
// двухбазовый BEGIN/INSERT/source-Update/COMMIT порядок; параметры Tiberius
// заменяют небезопасные `_sprintf` SQL-буферы. EXE 0x004E7CE7..0x004E7CFF
// подтвердил `%d = CSetup::dwNumber`, а 0x004E81D5..0x004E8208 — точный порядок
// девяти INSERT-аргументов.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004e884b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:796
// RVA: 0x000E884B
// ADDRESS: 004e884b
// PROTOTYPE: undefined Catch@004e884b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::AppendLargessToMap
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:56
// RVA: 0x000E9D40
// ADDRESS: 004e9d40
// PROTOTYPE: void __cdecl AppendLargessToMap(long param_1, ulong param_2, long param_3, long param_4, long param_5, long param_6)
//
// IMPLEMENTED_OWNER: `TiberiusLargess::append_largess_to_map` выше сохраняет
// SendID scan и unique player-key insertion без замены существующего value.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::CycleLoadLargessThread
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:809
// RVA: 0x000E9FD0
// ADDRESS: 004e9fd0
// PROTOTYPE: bool __cdecl CycleLoadLargessThread(void)
//
// IMPLEMENTED_OWNER: `TiberiusLargess::cycle_load_largess` выше сохраняет
// lock/connect/query/row-порядок, prefix mutations и literal ObtainedNum `0`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eab52
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:864
// RVA: 0x000EAB52
// ADDRESS: 004eab52
// PROTOTYPE: undefined Catch@004eab52()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004eab91
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:871
// RVA: 0x000EAB91
// ADDRESS: 004eab91
// PROTOTYPE: undefined FUN_004eab91()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::WorkerThread
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:692
// RVA: 0x000EAC80
// ADDRESS: 004eac80
// PROTOTYPE: uint __stdcall WorkerThread(void * param_1)
//
// IMPLEMENTED_OWNER: closure внутри `TiberiusLargess::start_worker` выполняет
// `transfer_largess`, затем безусловно `cycle_load_largess`; runtime Handle
// заменяет COM apartment только для async TDS-драйвера.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::StartWorkerThread
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:674
// RVA: 0x000EACC0
// ADDRESS: 004eacc0
// PROTOTYPE: void __cdecl StartWorkerThread(void)
//
// сохраняет nonblocking TryEnter/skip, а `std::thread::Builder` заменяет
// `_beginthreadex`. `CGame::main_loop` вызывает owner напрямую.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
