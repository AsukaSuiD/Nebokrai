//! DB-владелец и worker подарков `CLargess` WorldServer из `largess.cpp`,
//! подтверждённый точной парой `Nworldserver.exe` и `WorldServer.pdb`
//! (RSDS 289F1FB3-96A0-4FF4-8B5D-1FD17B50B751). Трейт `LargessOwner` и его
//! data-семья перенесены в Realm `persistence/`; Tiberius-реализация
//! `TiberiusLargess` остаётся в старом пакете (`dbaccess/worlddb/largess`)
//! и там же реэкспортирует этот модуль, потому что ветка
//! `LoadLargess`/`AddOneLargess` работает с `CPlayer`/`CGoods` и фабрикой
//! товаров старого владельца. Worker lifecycle transfer/cycle-load
//! (`StartWorkerThread`) остаётся с той же структурой: он разделяет с
//! load-веткой очереди `m_mapLargess` и обе настройки Cost DB, и его отрыв
//! разбил бы единое shared-состояние между потоками.
//!
//! Owner сохраняет lifecycle Init/UnInit, очереди transfer/cycle-load, порядок
//! `AddOneLargess`, календарные поля и обе формы `SaveLoadDetails`. Worker
//! переносит записи между очередями в исходном порядке; ошибка соединения или
//! команды не получает выдуманного rollback. Mutex/threads, Tiberius и owned
//! records заменяют Win32/ADO/STL, сохраняя locks, partial success и shutdown.
//!
//! Async-методы трейта записаны в desugared-форме по ADR-0013. Реализация
//! держит эквивалент `CriticalSectionmapLargess` — `parking_lot::MutexGuard`
//! (`!Send`) — захваченным поверх DB-вызовов, как исходная critical section
//! охватывала connect, оба SQL-вызова и erase. Поэтому `+ Send` на
//! возвращаемые future не добавлен: generic-потребитель `L: LargessOwner`
//! сохраняющего pipeline не требует Send и получает тот же `?Send`-контракт,
//! что давал исходный `async fn` трейта.

use std::error::Error;
use std::fmt;
use std::io;

use encoding_rs::WINDOWS_1251;
use tiberius::{AuthMethod, Config, EncryptionLevel};

use crate::persistence::rssetup::WorldTdsClient;

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
