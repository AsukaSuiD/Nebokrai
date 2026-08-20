//! Владелец `CLargess` исторического WorldServer из `largess.cpp`.
//!
//! Статус двух перегрузок `SaveLoadDetails` RVA `0x000E8CA0` и
//! `0x000E91B0`, а также достигнутого `GetTime` RVA `0x000E6800` —
//! `IMPLEMENTED`; остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
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
//! и log-механику; CD-key в `Debug` намеренно скрыт, но остаётся доступен
//! будущему точному log-owner-у.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::io;

use chrono::{Datelike, Local, Timelike};
use encoding_rs::WINDOWS_1251;
use parking_lot::Mutex;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

const LEGACY_SQL_BUFFER_SIZE: usize = 256;
const ERROR_GOODS_ID: &[u8] = b"error goodsID!";

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
    cost_database: CostDatabaseSettings,
    entries: Mutex<BTreeMap<i32, LargessSnapshot>>,
    notices: VecDeque<LargessNotice>,
}

impl TiberiusLargess {
    /// Принимает уже материализованный map snapshot; его наполнение принадлежит
    /// пока ещё сырому `LoadLargess`/`AppendLargessToMap`.
    pub(crate) fn new(
        load_largess_time: u32,
        cost_database: CostDatabaseSettings,
        entries: BTreeMap<i32, LargessSnapshot>,
    ) -> Self {
        Self {
            load_largess_time,
            cost_database,
            entries: Mutex::new(entries),
            notices: VecDeque::new(),
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:30
// RVA: 0x000E6630
// ADDRESS: 004e6630
// PROTOTYPE: bool __cdecl Init(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::UnInit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:42
// RVA: 0x000E6650
// ADDRESS: 004e6650
// PROTOTYPE: bool __cdecl UnInit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::AddGoldCoin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:79
// RVA: 0x000E6690
// ADDRESS: 004e6690
// PROTOTYPE: bool __cdecl AddGoldCoin(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::AddOneLargess
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:92
// RVA: 0x000E6A60
// ADDRESS: 004e6a60
// PROTOTYPE: bool __cdecl AddOneLargess(CPlayer * param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::LoadLargess
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:417
// RVA: 0x000E6EC0
// ADDRESS: 004e6ec0
// PROTOTYPE: void __cdecl LoadLargess(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::TransferLargessThread
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:715
// RVA: 0x000E7500
// ADDRESS: 004e7500
// PROTOTYPE: bool __cdecl TransferLargessThread(void)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:56
// RVA: 0x000E9D40
// ADDRESS: 004e9d40
// PROTOTYPE: void __cdecl AppendLargessToMap(long param_1, ulong param_2, long param_3, long param_4, long param_5, long param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::CycleLoadLargessThread
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:809
// RVA: 0x000E9FD0
// ADDRESS: 004e9fd0
// PROTOTYPE: bool __cdecl CycleLoadLargessThread(void)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:692
// RVA: 0x000EAC80
// ADDRESS: 004eac80
// PROTOTYPE: uint __stdcall WorkerThread(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLargess::StartWorkerThread
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\largess.cpp:674
// RVA: 0x000EACC0
// ADDRESS: 004eacc0
// PROTOTYPE: void __cdecl StartWorkerThread(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
