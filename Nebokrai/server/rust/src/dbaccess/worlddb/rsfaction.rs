//! DB-владелец `CRsFaction` исторического WorldServer из `rsfaction.cpp`.
//!
//! Статус `DelFaction` RVA `0x000F9A00`, `SaveFaction` RVA `0x000FF070`,
//! `SaveFactionProperty` RVA
//! `0x000FA080`, `SaveFactionMembers` RVA `0x000FB700`, `SaveLeaveWords` RVA
//! `0x000FACA0`, `SaveFactionApplyPersons` RVA `0x000FB240`, `SaveIconData` RVA
//! `0x000FB3F0`, `SaveFactionPronounce` RVA `0x000FC860` и `SaveAbility` RVA
//! `0x000FECB0`, а также private `LoadFactionProperty` RVA `0x000FCB20` —
//! `IMPLEMENTED`; constructor, destructor и остальные функции ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp`.
//!
//! PDB задаёт `DelFaction(long, connection) -> bool`. Raw показывает один
//! batch `DELETE CSL_FACTION_BaseProperty WHERE ID = %d` на caller-connection.
//! Exact `0x004F9A00..0x004F9AD0` имеет статус `VERIFIED_DISASSEMBLY`: signed
//! `long` передаётся в `_sprintf` напрямую, успешный `ExecuteCn` явно ставит
//! `AL=1`, а его `false` и null connection — `AL=0`. Только Execute-error пишет
//! `delete faction ERROR`; null-путь исходно не имеет log. Affected rows не
//! проверяются.
//!
//! Параметризованный TDS сохраняет signed ID и тот же SQL Server DELETE,
//! заменяя `_sprintf`/ADO transport. Любой `i32` вместе с NUL заведомо
//! помещался в исходный `char[1024]`, поэтому buffer-границы нет. Метод не
//! начинает и не завершает транзакцию: соседний `DoSaveData` владеет
//! begin/commit. `Option` сохраняет тихий null-путь, structured notice создаётся
//! только для доказанного DB-log и не содержит SQL либо runtime faction ID.
//!
//! `SaveFactionProperty(CFaction*, connection)` сначала нормализует save-копию:
//! пустой `m_szFactionWarLastWinTime` становится буквальной строкой `NULL`, а
//! найденный через `GetFactionById` live count передаётся достигнутому
//! `CFaction::SetGoodsWarCount`. Последний снимает local time, ставит dirty-bit
//! `1` и clamp-ит signed count `<= 0` к `0`; Rust caller передаёт результат
//! lookup как `canonical_goods_war_count`, не возвращая global singleton.
//!
//! Затем ADO открывал updateable recordset единственной строки
//! `CSL_FACTION_BaseProperty`, делал `MoveFirst`, записывал пятнадцать полей в
//! исходном порядке и только в конце вызывал `Update`. TDS `UPDATE TOP (1)` с
//! параметрами сохраняет ту же одну строку и типы; ноль affected rows заменяет
//! доказанную ошибку `MoveFirst` на typed `MissingPropertyRow`. Exact epilogue
//! `0x004FABF3..0x004FAC59` подтверждает `true` только после Update и `false`
//! после catch; catch писал `Save Faction Property`. Null faction остаётся
//! тихим `false`, а non-null faction успевает выполнить state-нормализацию до
//! ошибки null connection. После ответов reverse прекращён.
//!
//! `SaveFactionMembers(CFaction*, connection)` сначала удаляет все строки
//! `CSL_FACTION_Members` заданной фракции, затем проходит исходный
//! `map<long, tagMemInfo>` в signed-key порядке и выполняет отдельный INSERT для
//! каждого участника. Ошибка DELETE пишет `delete CSL_FACTION_Members ERROR` и
//! возвращает `false`; ошибка одного INSERT попадает в catch `Save Faction
//! MemberData`, оставляет уже выполненные изменения caller-транзакции и также
//! возвращает `false`. Пустая map после успешного DELETE даёт `true`.
//!
//! Exact call-site `0x004FB7D9..0x004FB899` имеет статус
//! `VERIFIED_DISASSEMBLY`: потерянными raw vararg были `wSecond` и
//! `listPV[10]`; фактически time имеет порядок year/month/day/hour/minute/second,
//! а INSERT получает все одиннадцать signed purview. Эпилог
//! `0x004FB8E2..0x004FB940` отдельно подтверждает `AL=1` после полного обхода и
//! `AL=0` после catch. После этих ответов reverse прекращён.
//!
//! `BTreeMap` заменяет только внутренности STL и сохраняет порядок. Tiberius
//! заменяет ADO, а неэкранированный SQL literal сохранён намеренно: одинарная
//! кавычка в title должна по-прежнему менять результат SQL-парсера, а не
//! становиться успешным параметром. ANSI `strTitle` декодируется как Windows-
//! 1251; при 63 видимых байтах, максимальных i32/u16 и завершающем NUL исходный
//! `char[1024]` требовал не более 536 байт. Если NUL отсутствует, Rust возвращает
//! локальный `BLOCKED_MISSING_FACT` после уже выполненного DELETE/предыдущих
//! INSERT: исходный `%s` читал бы за массивом, но достижимость и результат этого
//! UB не доказаны. Null faction остаётся тихим `false`; null connection
//! представляется DELETE-stage notice без SQL и runtime ID.
//!
//! `SaveLeaveWords(CFaction*, connection)` так же сначала удаляет все строки
//! `CSL_FACTIONLeaveWord` faction ID, затем проходит `m_LeaveWords` в list-
//! порядке и выполняет отдельный INSERT без списка колонок:
//! `ID, FactionID, LeaveWordPlayerID, Content, LeaveWordTime`. До форматирования
//! content исходник вызывал `CGame::CheckPoint`, который удваивает одинарные
//! кавычки; Windows-1251 decode и сырой TDS batch сохраняют те же SQL-значения.
//! Успешный escaped content ограничен 255 ANSI-байтами, поэтому даже с
//! максимальными i32/u16 итог вместе с NUL требует не более 374 байт старого
//! `char[1024]`.
//!
//! Exact call-site `0x004FAD8C..0x004FADEB` имеет статус
//! `VERIFIED_DISASSEMBLY`: потерянными raw vararg были `wSecond` и готовая
//! time-строка; INSERT получает leave-word ID, faction ID, player ID, escaped
//! content и `year-month-day hour:minute:second` именно в этом порядке. Хвост
//! `0x004FAE4F..0x004FAEB6` подтверждает `true` после полного/пустого обхода и
//! `false` для insert/delete/catch. После этих ответов reverse прекращён.
//!
//! Ошибка DELETE пишет исходный `delete CSL_FACTIONLeaveWord ERROR `, ошибка
//! INSERT — `Save Faction LeaveWord ERROR`; неожиданный старый `_com_error`
//! имел catch `Save Faction LeaveWords`. Rust DB-ошибки явны и не выдаются за
//! unwind. Предыдущие INSERT остаются в caller-транзакции. Отсутствующий NUL в
//! `strContent` либо escaped length `>= 256` возвращает локальный
//! `BLOCKED_MISSING_FACT` после уже выполненных эффектов: в обоих случаях
//! оригинал читал бы за fixed/stack buffer. Null faction остаётся тихим
//! `false`, null connection относится к DELETE-stage notice. Raw owner, catch и
//! compiler cleanup удалены; `VecDeque`, owned bytes и `Drop` заменяют только
//! STL/string/stack lifetime.
//!
//! `SaveAbility(CFaction*, connection)` открывал первую ability-строку по
//! signed faction ID. При EOF он открывал updateable table-recordset, делал
//! `AddNew` и задавал `FactionID`; существующая строка сохраняла текущий row.
//! `SaveFactionPronounce` получал ровно `0x828` bytes через достигнутый
//! `CFaction::GetPronounceData` и назначал binary-поле `Pronounce`.
//! `SaveIconData` назначал byte-vector `IconData` и BSTR
//! `LastUploadIconDataTime` в формате `year-month-day hour:minute:second`.
//! Только после успеха обоих private owner-ов выполнялся один recordset
//! `Update`; затем ability-row уже не откатывался, даже если отдельный
//! `SaveFactionApplyPersons` завершался `false`.
//!
//! Apply-owner сначала выполнял literal
//! `DELETE FROM CSL_Faction_Apply WHERE FactionID='%d'`, затем проходил
//! `m_ApplyPersons` в signed map-порядке и делал отдельный
//! `INSERT INTO CSL_Faction_Apply VALUES(%d,%d)` для каждого key. Ошибка DELETE
//! писала `delete CSL_Faction_Apply ERROR`, ошибка INSERT — `Save Faction
//! ApplyList ERROR`; уже выполненные ability update, DELETE и предыдущие INSERT
//! оставались в caller-транзакции. Пустой map после DELETE давал `true`.
//! COM/SAFEARRAY исключения private helper-ов имели отдельные строки `Save
//! Faction Pronounce`, `Save Faction IconData` и `Save Faction Apply`, а внешний
//! recordset catch — `Save Faction Ability`; в Rust устранённые VARIANT/COM-
//! операции не получают вымышленных failure-ветвей.
//!
//! Exact диапазоны `0x004FB295..0x004FB39C`,
//! `0x004FB3F0..0x004FB6DF`, `0x004FC860..0x004FCA7C` и
//! `0x004FECD9..0x004FF04C` имеют статус `VERIFIED_DISASSEMBLY`: второй INSERT-
//! аргумент берётся из signed map-key по node `+0x0C`; все три private owner-а
//! и dispatcher возвращают `true` только после полного успеха, а dispatcher
//! передаёт тот же caller-connection в apply-owner после `Update`. Получив эти
//! ответы, reverse прекращён.
//!
//! Один параметризованный TDS `IF EXISTS UPDATE TOP (1) ELSE INSERT` заменяет
//! только updateable ADO recordset и сохраняет выбор одной строки, четыре
//! записываемых поля и один финальный DB-эффект. Binary-параметры сохраняют
//! byte-exact pronounce/icon, строковый параметр проходит то же преобразование
//! SQL Server в тип time-колонки. Apply SQL оставлен literal и выполняется
//! отдельными batch-ами в исходном порядке. Null faction остаётся тихим
//! `false`; null connection и ошибка ability batch соответствуют `Save Faction
//! Ability`. `BTreeSet`, slices, Tiberius и `Drop` заменяют только MSVC tree,
//! SAFEARRAY/VARIANT, ADO и compiler cleanup. Четыре заменённых owner-блока,
//! catch-и и cleanup-helper-ы удалены.
//!
//! Верхний `SaveFaction` один раз читает `m_ChangeDataType`, затем независимо и
//! строго в порядке bits `1, 2, 4, 8` вызывает property, members, leave-word и
//! ability owner-ы с одним caller-connection. Возвращаемые leaf-ами `false`
//! исходно не проверяются и не останавливают следующие ветви; после обычного
//! прохода dispatcher возвращает `true`, в том числе для нулевой mask. Null
//! faction остаётся тихим `false`, null connection пишет `Save Thread Sent
//! Connect Not Found.` и также возвращает `false`.
//!
//! Raw уверенно задавал ветвление, но не bool-результат своего catch. Поэтому
//! exact-проверка была ограничена эпилогами `0x004FF16E..0x004FF18D` и
//! `0x004FF190..0x004FF1BE`: normal path ставит `AL=1`, catch `Save Faction` —
//! `AL=0`. После ответа reverse прекращён. В Rust leaf DB-ошибки уже являются
//! явными `false` и поглощаются dispatcher-ом буквально. Только два ранее
//! зафиксированных overread-пути возвращают typed `BlockedMissingFact`: их
//! неизвестная достижимость не заменена продолжением следующих dirty-bits.
//!
//! `LoadFactionProperty` открывает отдельное World DB connection и буквально
//! выполняет `SELECT * FROM CSL_FACTION_BaseProperty ORDER BY id`. Для каждой
//! строки он сперва запускает public constructor `CFaction`, затем в исходном
//! порядке заменяет DB property/scalar поля и кладёт объект в signed-key map.
//! Поэтому staging сохраняет уже готовый prefix до DB/field failure, а duplicate
//! ID оставляет последнюю строку. Старый leaked overwritten pointer устранён:
//! он не был виден вне реализации. `GoodsWarLastTime` проходит SQL calendar
//! time в `SYSTEMTIME`-эквивалент и форматируется без zero-padding как старый
//! `_sprintf`; `country` сохраняется как `unsigned char`. Пять остальных
//! частей `LoadAllFaction` намеренно ещё не вызываются из этого narrow owner-а:
//! их нельзя подменять частично опубликованным `COrganizingCtrl`.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;

use chrono::{Datelike, NaiveDateTime, Timelike};
use encoding_rs::WINDOWS_1251;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};
use crate::worldserver::appworld::organizingsystem::faction::{
    CFaction, FactionBaseProperty, FactionDatabaseBaseState, FactionInitialBlock, TagLeaveWord,
    UnterminatedLeaveWordField,
};
use crate::worldserver::appworld::organizingsystem::organizing::{
    TagMemInfo, TagTimeValue, UnterminatedMemberField,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::worldserver::game::{CGame, WorldCheckPointBlock};

const SAVE_FACTION_PROPERTY_SQL: &str = "IF EXISTS (SELECT TOP 1 ID FROM CSL_FACTION_BaseProperty WHERE ID = @P16 ORDER BY ID) BEGIN UPDATE TOP (1) CSL_FACTION_BaseProperty SET MasterID = @P1, Levels = @P2, Experience = @P3, OffenseVictorCounts = @P4, DefenceVictorCounts = @P5, VillageWarVictorCounts = @P6, MemberNums = @P7, UnionID = @P8, bPermit = @P9, lPro1 = @P10, lPro2 = @P11, DelRemainTime = @P12, country = @P13, GoodsWarCount = @P14, GoodsWarLastTime = @P15 WHERE ID = @P16; SELECT CAST(@@ROWCOUNT AS int) AS UpdatedRows END ELSE SELECT CAST(0 AS int) AS UpdatedRows";
const SAVE_FACTION_ABILITY_SQL: &str = "IF EXISTS (SELECT TOP 1 FactionID FROM CSL_FACTION_Ability WHERE FactionID = @P1 ORDER BY FactionID) BEGIN UPDATE TOP (1) CSL_FACTION_Ability SET Pronounce = @P2, IconData = @P3, LastUploadIconDataTime = @P4 WHERE FactionID = @P1 END ELSE BEGIN INSERT INTO CSL_FACTION_Ability (FactionID, Pronounce, IconData, LastUploadIconDataTime) VALUES (@P1, @P2, @P3, @P4) END";
const LOAD_FACTION_PROPERTY_SQL: &str = "SELECT * FROM CSL_FACTION_BaseProperty ORDER BY id";

/// Безопасный prefix concrete `CFaction`, сформированный private owner-ом
/// `LoadFactionProperty` до следующих четырёх DB-этапов `LoadAllFaction`.
pub(crate) type FactionPropertyLoadStaging = BTreeMap<i32, CFaction>;

/// Наблюдаемый исход private owner-а `LoadFactionProperty`.
///
/// `factions` сохраняет уже вставленный map-prefix. Повторный ID заменяет
/// прежний объект, как `std::map::operator[]`; старая утечка pointer-а не
/// переносится, поскольку не меняла опубликованное последнее значение.
pub(crate) enum FactionPropertyLoadOutcome {
    ReturnedTrue { factions: FactionPropertyLoadStaging },
    ReturnedFalse { factions: FactionPropertyLoadStaging },
    BlockedMissingFact {
        factions: FactionPropertyLoadStaging,
        block: FactionInitialBlock,
    },
}

/// Scalar-поля property-группы одной достигнутой save-копии `CFaction`.
struct FactionPropertySaveSnapshot {
    master_id: i32,
    base_property: FactionBaseProperty,
    delete_remain_time: i32,
}

/// Локальная граница malformed save-копии перед первым DB-вызовом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionSaveProjectionBlock {
    MasterId,
    BaseProperty,
    DeleteRemainTime,
}

/// Достигнутая save-копия `CFaction`; property materialизуется только для bit 1.
pub(crate) struct FactionSaveSnapshot<'faction> {
    pub(crate) faction: &'faction mut CFaction,
    /// Результат исходного `GetFactionById`; `None` сохраняет no-op lookup-а.
    canonical_goods_war_count: Option<i32>,
    property: Option<FactionPropertySaveSnapshot>,
}

impl<'faction> FactionSaveSnapshot<'faction> {
    /// Материализует ровно поля, читаемые `CRsFaction`, из frozen save-копии.
    pub(crate) fn from_faction(
        faction: &'faction mut CFaction,
        canonical_goods_war_count: Option<i32>,
    ) -> Result<Self, FactionSaveProjectionBlock> {
        let property = if faction.change_data_type() & 1 != 0 {
            Some(FactionPropertySaveSnapshot {
                master_id: faction
                    .master_id()
                    .ok_or(FactionSaveProjectionBlock::MasterId)?,
                base_property: faction
                    .base_property()
                    .ok_or(FactionSaveProjectionBlock::BaseProperty)?,
                delete_remain_time: faction
                    .delete_remain_time()
                    .ok_or(FactionSaveProjectionBlock::DeleteRemainTime)?,
            })
        } else {
            None
        };

        Ok(Self {
            faction,
            canonical_goods_war_count,
            property,
        })
    }
}

/// Исходный bool-результат `SaveFactionMembers` либо безопасная граница старого
/// чтения `strTitle` за фиксированным массивом.
#[derive(Debug)]
pub(crate) enum FactionMembersSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(UnterminatedMemberField),
}

/// Неизвестная граница старого C-string/stack-buffer пути leave-word.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionLeaveWordSaveBlock {
    UnterminatedContent(UnterminatedLeaveWordField),
    CheckPointBuffer(WorldCheckPointBlock),
}

impl fmt::Display for FactionLeaveWordSaveBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedContent(block) => block.fmt(formatter),
            Self::CheckPointBuffer(block) => block.fmt(formatter),
        }
    }
}

impl Error for FactionLeaveWordSaveBlock {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnterminatedContent(block) => Some(block),
            Self::CheckPointBuffer(block) => Some(block),
        }
    }
}

impl From<UnterminatedLeaveWordField> for FactionLeaveWordSaveBlock {
    fn from(block: UnterminatedLeaveWordField) -> Self {
        Self::UnterminatedContent(block)
    }
}

impl From<WorldCheckPointBlock> for FactionLeaveWordSaveBlock {
    fn from(block: WorldCheckPointBlock) -> Self {
        Self::CheckPointBuffer(block)
    }
}

/// Исходный bool-результат `SaveLeaveWords` либо локальный missing-fact.
#[derive(Debug)]
pub(crate) enum FactionLeaveWordsSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(FactionLeaveWordSaveBlock),
}

/// Безопасная граница одного из двух старых overread-путей dispatcher-а.
#[derive(Debug)]
pub(crate) enum FactionSaveBlock {
    Member(UnterminatedMemberField),
    LeaveWord(FactionLeaveWordSaveBlock),
}

/// Наблюдаемый bool-результат `SaveFaction` либо локальный missing-fact.
#[derive(Debug)]
pub(crate) enum FactionSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionSaveBlock),
}

/// Структурированная замена достигнутых `CRsFaction` DB-log ветвей.
#[derive(Debug)]
pub(crate) struct RsFactionNotice {
    pub(crate) operation: RsFactionOperation,
    pub(crate) error: RsFactionSaveError,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RsFactionOperation {
    LoadFactionProperty,
    SaveFaction,
    DeleteFaction,
    SaveFactionProperty,
    DeleteFactionMembers,
    SaveFactionMember,
    DeleteFactionLeaveWords,
    SaveFactionLeaveWord,
    SaveFactionAbility,
    DeleteFactionApplyPersons,
    SaveFactionApplyPerson,
}

#[derive(Debug)]
pub(crate) enum RsFactionSaveError {
    Database(RsFactionDatabaseError),
    MissingConnection,
    MissingPropertyRow,
    MissingSettings,
    Connection(WorldDatabaseConnectionError),
    MissingRequiredValue(&'static str),
}

impl fmt::Display for RsFactionSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => {
                formatter.write_str("не передано соединение World faction DB")
            }
            Self::MissingPropertyRow => formatter
                .write_str("World faction DB не содержит base-property строку для обновления"),
            Self::MissingSettings => formatter.write_str("не заданы параметры World faction DB"),
            Self::Connection(error) => error.fmt(formatter),
            Self::MissingRequiredValue(column) => {
                write!(formatter, "в faction DB отсутствует обязательное поле {column}")
            }
        }
    }
}

impl Error for RsFactionSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Connection(error) => Some(error),
            Self::MissingConnection
            | Self::MissingPropertyRow
            | Self::MissingSettings
            | Self::MissingRequiredValue(_) => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RsFactionDatabaseError(tiberius::error::Error);

impl fmt::Display for RsFactionDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World faction DB: {}", self.0)
    }
}

impl Error for RsFactionDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsFactionDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Ошибка одного field/query шага `LoadFactionProperty` до преобразования в
/// operator-visible notice. Runtime SQL и DB-значения намеренно не выдаются.
#[derive(Debug)]
enum FactionLoadReadError {
    Database(tiberius::error::Error),
    MissingRequiredValue(&'static str),
}

impl From<tiberius::error::Error> for FactionLoadReadError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Database(error)
    }
}

impl From<FactionLoadReadError> for RsFactionSaveError {
    fn from(error: FactionLoadReadError) -> Self {
        match error {
            FactionLoadReadError::Database(error) => Self::Database(error.into()),
            FactionLoadReadError::MissingRequiredValue(column) => Self::MissingRequiredValue(column),
        }
    }
}

fn read_faction_i32(row: &Row, column: &'static str) -> Result<i32, FactionLoadReadError> {
    match row.try_get::<i32, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FactionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(FactionLoadReadError::Database(error)),
    }
}

fn read_faction_bool(row: &Row, column: &'static str) -> Result<bool, FactionLoadReadError> {
    match row.try_get::<bool, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FactionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(FactionLoadReadError::Database(error)),
    }
}

fn read_faction_u8(row: &Row, column: &'static str) -> Result<u8, FactionLoadReadError> {
    match row.try_get::<u8, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FactionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(FactionLoadReadError::Database(error)),
    }
}

fn read_faction_text(row: &Row, column: &'static str) -> Result<Vec<u8>, FactionLoadReadError> {
    match row.try_get::<&str, _>(column) {
        Ok(Some(value)) => {
            let (encoded, _, _) = WINDOWS_1251.encode(value);
            Ok(encoded.into_owned())
        }
        Ok(None) => Err(FactionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(FactionLoadReadError::Database(error)),
    }
}

fn read_faction_time(
    row: &Row,
    column: &'static str,
) -> Result<TagTimeValue, FactionLoadReadError> {
    let value = match row.try_get::<NaiveDateTime, _>(column) {
        Ok(Some(value)) => value,
        Ok(None) => return Err(FactionLoadReadError::MissingRequiredValue(column)),
        Err(error) => return Err(FactionLoadReadError::Database(error)),
    };
    Ok(TagTimeValue {
        year: u16::try_from(value.year())
            .expect("год NaiveDateTime помещается в SYSTEMTIME"),
        month: u16::try_from(value.month())
            .expect("месяц NaiveDateTime помещается в SYSTEMTIME"),
        day_of_week: u16::try_from(value.weekday().num_days_from_sunday())
            .expect("день недели NaiveDateTime помещается в SYSTEMTIME"),
        day: u16::try_from(value.day()).expect("день NaiveDateTime помещается в SYSTEMTIME"),
        hour: u16::try_from(value.hour()).expect("час NaiveDateTime помещается в SYSTEMTIME"),
        minute: u16::try_from(value.minute())
            .expect("минута NaiveDateTime помещается в SYSTEMTIME"),
        second: u16::try_from(value.second())
            .expect("секунда NaiveDateTime помещается в SYSTEMTIME"),
        milliseconds: u16::try_from(value.nanosecond() / 1_000_000)
            .expect("миллисекунда NaiveDateTime помещается в SYSTEMTIME"),
    })
}

fn read_faction_database_base_state(
    row: &Row,
) -> Result<FactionDatabaseBaseState, FactionLoadReadError> {
    let goods_war_last_win_time = read_faction_time(row, "GoodsWarLastTime")?;
    Ok(FactionDatabaseBaseState {
        faction_id: read_faction_i32(row, "ID")?,
        name: read_faction_text(row, "Name")?,
        master_id: read_faction_i32(row, "MasterID")?,
        established_time: read_faction_time(row, "EstablishedTime")?,
        level: read_faction_i32(row, "Levels")?,
        experience: read_faction_i32(row, "Experience")?,
        offense_victor_counts: read_faction_i32(row, "OffenseVictorCounts")?,
        defence_victor_counts: read_faction_i32(row, "DefenceVictorCounts")?,
        village_war_victor_counts: read_faction_i32(row, "VillageWarVictorCounts")?,
        member_count: read_faction_i32(row, "MemberNums")?,
        union_id: read_faction_i32(row, "UnionID")?,
        permit: read_faction_bool(row, "bPermit")?,
        property_1: read_faction_i32(row, "lPro1")?,
        property_2: read_faction_i32(row, "lPro2")?,
        delete_remain_time: read_faction_i32(row, "DelRemainTime")?,
        country: read_faction_u8(row, "country")?,
        goods_war_count: read_faction_i32(row, "GoodsWarCount")?,
        // `_sprintf` использовал year/month/day/hour/minute/second без zero
        // padding и не переносил weekday/milliseconds из SYSTEMTIME.
        goods_war_last_win_time: format!(
            "{}-{}-{} {}:{}:{}",
            goods_war_last_win_time.year,
            goods_war_last_win_time.month,
            goods_war_last_win_time.day,
            goods_war_last_win_time.hour,
            goods_war_last_win_time.minute,
            goods_war_last_win_time.second
        ),
    })
}

/// Узкая объектная граница достигнутой стадии исходного `CRsFaction`.
pub(crate) trait RsFactionOwner {
    /// Открывает отдельное World DB соединение и materialize-ит только
    /// `LoadFactionProperty`; оставшиеся load-owner-ы пока не смешиваются с
    /// этим staging map.
    async fn load_faction_property(
        &mut self,
        master_title: &[u8],
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> FactionPropertyLoadOutcome;

    /// Вызывает dirty-bit leaf-ы `1/2/4/8` внутри caller-транзакции.
    async fn save_faction(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionSaveOutcome;

    /// Удаляет base-property row фракции внутри caller-транзакции.
    async fn del_faction(
        &mut self,
        faction_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Сохраняет property dirty-bit `1` внутри caller-транзакции.
    async fn save_faction_property(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Перезаписывает ordered member-снимок для dirty-bit `2` внутри
    /// caller-транзакции.
    async fn save_faction_members(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionMembersSaveOutcome;

    /// Перезаписывает ordered leave-word snapshot для dirty-bit `4` внутри
    /// caller-транзакции.
    async fn save_leave_words(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionLeaveWordsSaveOutcome;

    /// Сохраняет ability-row, затем ordered apply-person snapshot внутри
    /// caller-транзакции.
    async fn save_ability(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Забирает следующий исходный DB-log эквивалент.
    fn pop_notice(&mut self) -> Option<RsFactionNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRsFaction`.
#[derive(Default)]
pub(crate) struct TiberiusRsFaction {
    settings: Option<WorldDatabaseSettings>,
    notices: VecDeque<RsFactionNotice>,
}

impl TiberiusRsFaction {
    pub(crate) fn new(settings: WorldDatabaseSettings) -> Self {
        Self {
            settings: Some(settings),
            notices: VecDeque::new(),
        }
    }

    fn property_load_failed(
        &mut self,
        error: FactionLoadReadError,
        factions: FactionPropertyLoadStaging,
    ) -> FactionPropertyLoadOutcome {
        self.notices.push_back(RsFactionNotice {
            operation: RsFactionOperation::LoadFactionProperty,
            error: error.into(),
        });
        FactionPropertyLoadOutcome::ReturnedFalse { factions }
    }
}

impl RsFactionOwner for TiberiusRsFaction {
    async fn load_faction_property(
        &mut self,
        master_title: &[u8],
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> FactionPropertyLoadOutcome {
        let Some(settings) = self.settings.clone() else {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadFactionProperty,
                error: RsFactionSaveError::MissingSettings,
            });
            return FactionPropertyLoadOutcome::ReturnedFalse {
                factions: BTreeMap::new(),
            };
        };
        let mut connection = match settings.connect().await {
            Ok(connection) => connection,
            Err(error) => {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadFactionProperty,
                    error: RsFactionSaveError::Connection(error),
                });
                return FactionPropertyLoadOutcome::ReturnedFalse {
                    factions: BTreeMap::new(),
                };
            }
        };
        let rows = match Query::new(LOAD_FACTION_PROPERTY_SQL)
            .query(&mut connection)
            .await
        {
            Ok(stream) => match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => return self.property_load_failed(error.into(), BTreeMap::new()),
            },
            Err(error) => return self.property_load_failed(error.into(), BTreeMap::new()),
        };

        let mut factions = BTreeMap::new();
        for row in &rows {
            let state = match read_faction_database_base_state(row) {
                Ok(state) => state,
                Err(error) => return self.property_load_failed(error, factions),
            };
            let faction_id = state.faction_id;
            let faction = match CFaction::from_database_base_state(
                state,
                master_title,
                game,
                parameters,
            ) {
                Ok(faction) => faction,
                Err(block) => {
                    return FactionPropertyLoadOutcome::BlockedMissingFact { factions, block };
                }
            };
            factions.insert(faction_id, faction);
        }
        FactionPropertyLoadOutcome::ReturnedTrue { factions }
    }

    async fn save_faction(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionSaveOutcome {
        let Some(snapshot) = snapshot else {
            return FactionSaveOutcome::ReturnedFalse;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::SaveFaction,
                error: RsFactionSaveError::MissingConnection,
            });
            return FactionSaveOutcome::ReturnedFalse;
        };

        // RVA 0x004FF0F4 читает mask один раз: property-нормализация может снова
        // поставить bit 1, но не должна менять набор ветвей текущего прохода.
        let change_data_type = snapshot.faction.change_data_type();

        if change_data_type & 1 != 0 {
            let _ = self
                .save_faction_property(Some(&mut *snapshot), Some(&mut *active_transaction))
                .await;
        }
        if change_data_type & 2 != 0 {
            match self
                .save_faction_members(Some(&*snapshot.faction), Some(&mut *active_transaction))
                .await
            {
                FactionMembersSaveOutcome::Saved | FactionMembersSaveOutcome::Failed => {}
                FactionMembersSaveOutcome::BlockedMissingFact(block) => {
                    return FactionSaveOutcome::BlockedMissingFact(FactionSaveBlock::Member(block));
                }
            }
        }
        if change_data_type & 4 != 0 {
            match self
                .save_leave_words(Some(&*snapshot.faction), Some(&mut *active_transaction))
                .await
            {
                FactionLeaveWordsSaveOutcome::Saved | FactionLeaveWordsSaveOutcome::Failed => {}
                FactionLeaveWordsSaveOutcome::BlockedMissingFact(block) => {
                    return FactionSaveOutcome::BlockedMissingFact(FactionSaveBlock::LeaveWord(
                        block,
                    ));
                }
            }
        }
        if change_data_type & 8 != 0 {
            let _ = self
                .save_ability(Some(&*snapshot.faction), Some(&mut *active_transaction))
                .await;
        }

        FactionSaveOutcome::ReturnedTrue
    }

    async fn del_faction(
        &mut self,
        faction_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(active_transaction) = active_transaction else {
            return false;
        };

        let mut query = Query::new("DELETE CSL_FACTION_BaseProperty WHERE ID = @P1");
        query.bind(faction_id);
        match query.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::DeleteFaction,
                    error: RsFactionSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn save_faction_property(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(snapshot) = snapshot else {
            return false;
        };
        let Some(property) = snapshot.property.as_ref() else {
            // Единственный caller вызывает owner только для dirty-bit 1, а
            // закрытый constructor не создаёт такой snapshot без property.
            return false;
        };

        if snapshot.faction.goods_war_last_win_time().is_empty() {
            snapshot
                .faction
                .set_goods_war_last_win_time(String::from("NULL"));
        }
        if let Some(canonical_goods_war_count) = snapshot.canonical_goods_war_count {
            snapshot
                .faction
                .set_goods_war_count(canonical_goods_war_count);
        }

        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::SaveFactionProperty,
                error: RsFactionSaveError::MissingConnection,
            });
            return false;
        };

        let mut query = Query::new(SAVE_FACTION_PROPERTY_SQL);
        query.bind(property.master_id);
        query.bind(property.base_property.level());
        query.bind(property.base_property.experience());
        query.bind(property.base_property.offense_victor_counts());
        query.bind(property.base_property.defence_victor_counts());
        query.bind(property.base_property.village_war_victor_counts());
        query.bind(property.base_property.member_count());
        query.bind(property.base_property.union_id());
        query.bind(property.base_property.permit());
        query.bind(property.base_property.property_1());
        query.bind(property.base_property.property_2());
        query.bind(property.delete_remain_time);
        query.bind(property.base_property.country());
        query.bind(snapshot.faction.goods_war_count());
        query.bind(snapshot.faction.goods_war_last_win_time());
        query.bind(snapshot.faction.faction_id());

        let update_result = match query.query(active_transaction).await {
            Ok(stream) => stream.into_row().await,
            Err(error) => Err(error),
        };
        match update_result {
            Ok(Some(row)) if row.get::<i32, _>(0).is_some_and(|updated| updated != 0) => true,
            Ok(_) => {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::SaveFactionProperty,
                    error: RsFactionSaveError::MissingPropertyRow,
                });
                false
            }
            Err(error) => {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::SaveFactionProperty,
                    error: RsFactionSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn save_faction_members(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionMembersSaveOutcome {
        let Some(faction) = faction else {
            return FactionMembersSaveOutcome::Failed;
        };
        let faction_id = faction.faction_id();
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::DeleteFactionMembers,
                error: RsFactionSaveError::MissingConnection,
            });
            return FactionMembersSaveOutcome::Failed;
        };

        let delete_sql = format!("DELETE FROM CSL_FACTION_Members WHERE FactionID='{faction_id}'");
        if let Err(error) = execute_faction_statement(active_transaction, delete_sql).await {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::DeleteFactionMembers,
                error: RsFactionSaveError::Database(error.into()),
            });
            return FactionMembersSaveOutcome::Failed;
        }

        for member in faction.get_members().values() {
            let insert_sql = match build_faction_member_insert(faction_id, member) {
                Ok(insert_sql) => insert_sql,
                Err(block) => return FactionMembersSaveOutcome::BlockedMissingFact(block),
            };
            if let Err(error) = execute_faction_statement(active_transaction, insert_sql).await {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::SaveFactionMember,
                    error: RsFactionSaveError::Database(error.into()),
                });
                return FactionMembersSaveOutcome::Failed;
            }
        }

        FactionMembersSaveOutcome::Saved
    }

    async fn save_leave_words(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionLeaveWordsSaveOutcome {
        let Some(faction) = faction else {
            return FactionLeaveWordsSaveOutcome::Failed;
        };
        let faction_id = faction.faction_id();
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::DeleteFactionLeaveWords,
                error: RsFactionSaveError::MissingConnection,
            });
            return FactionLeaveWordsSaveOutcome::Failed;
        };

        let delete_sql =
            format!("DELETE FROM CSL_FACTIONLeaveWord WHERE FactionID='{faction_id}' ");
        if let Err(error) = execute_faction_statement(active_transaction, delete_sql).await {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::DeleteFactionLeaveWords,
                error: RsFactionSaveError::Database(error.into()),
            });
            return FactionLeaveWordsSaveOutcome::Failed;
        }

        for leave_word in faction.get_leave_words() {
            let insert_sql = match build_faction_leave_word_insert(faction_id, leave_word) {
                Ok(insert_sql) => insert_sql,
                Err(block) => return FactionLeaveWordsSaveOutcome::BlockedMissingFact(block),
            };
            if let Err(error) = execute_faction_statement(active_transaction, insert_sql).await {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::SaveFactionLeaveWord,
                    error: RsFactionSaveError::Database(error.into()),
                });
                return FactionLeaveWordsSaveOutcome::Failed;
            }
        }

        FactionLeaveWordsSaveOutcome::Saved
    }

    async fn save_ability(
        &mut self,
        faction: Option<&CFaction>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(faction) = faction else {
            return false;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::SaveFactionAbility,
                error: RsFactionSaveError::MissingConnection,
            });
            return false;
        };

        let mut pronounce_data = Vec::new();
        faction.get_pronounce_data(&mut pronounce_data);
        let icon_time = faction.last_upload_icon_time();
        let last_upload_icon_time = format!(
            "{}-{}-{} {}:{}:{}",
            icon_time.year,
            icon_time.month,
            icon_time.day,
            icon_time.hour,
            icon_time.minute,
            icon_time.second
        );

        let mut query = Query::new(SAVE_FACTION_ABILITY_SQL);
        query.bind(faction.faction_id());
        query.bind(pronounce_data.as_slice());
        query.bind(faction.icon_data());
        query.bind(last_upload_icon_time.as_str());
        if let Err(error) = query.execute(&mut *active_transaction).await {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::SaveFactionAbility,
                error: RsFactionSaveError::Database(error.into()),
            });
            return false;
        }

        self.save_faction_apply_persons(faction, active_transaction)
            .await
    }

    fn pop_notice(&mut self) -> Option<RsFactionNotice> {
        self.notices.pop_front()
    }
}

impl TiberiusRsFaction {
    async fn save_faction_apply_persons(
        &mut self,
        faction: &CFaction,
        active_transaction: &mut WorldTdsClient,
    ) -> bool {
        let faction_id = faction.faction_id();
        let delete_sql = format!("DELETE FROM CSL_Faction_Apply WHERE FactionID='{faction_id}'");
        if let Err(error) = execute_faction_statement(active_transaction, delete_sql).await {
            self.notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::DeleteFactionApplyPersons,
                error: RsFactionSaveError::Database(error.into()),
            });
            return false;
        }

        for player_id in faction.get_apply_person_ids() {
            let insert_sql =
                format!("INSERT INTO CSL_Faction_Apply VALUES({faction_id},{player_id})");
            if let Err(error) = execute_faction_statement(active_transaction, insert_sql).await {
                self.notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::SaveFactionApplyPerson,
                    error: RsFactionSaveError::Database(error.into()),
                });
                return false;
            }
        }

        true
    }
}

fn build_faction_member_insert(
    faction_id: i32,
    member: &TagMemInfo,
) -> Result<String, UnterminatedMemberField> {
    let title = member.title_wire_bytes()?;
    let (title, _, _) = WINDOWS_1251.decode(&title[..title.len() - 1]);
    let time = member.last_online_time;
    let last_online_time = format!(
        "{}-{}-{} {}:{}:{}",
        time.year, time.month, time.day, time.hour, time.minute, time.second
    );
    let purview = member.purview.map(|state| state.wire_value());
    let contribute = if member.contribute { 1 } else { 0 };

    // Старый sprintf не экранировал Title. Сырой SQL сохраняет наблюдаемую
    // семантику SQL-парсера: кавычка могла дать ошибку либо изменить batch.
    // Tiberius заменяет только транспорт ADO. Даже при 63 ANSI-байтах Title и
    // максимальных i32/u16 нужно не более 536 байт исходного char[1024] с NUL.
    Ok(format!(
        "INSERT INTO CSL_FACTION_Members (FactionID,PlayerID,MemberLvl,Title,bControbute,LastOnlineTime, PV_Disband,PV_Exit,PV_DubJobLvl,PV_ConMem,PV_FireOut,PV_Pronounce,PV_LeaveWord, PV_EditLeaveWord,PV_ObtainTax,PV_OperCityGate,PV_EndueROR) VALUES ({},{},{},N'{}',{},'{}',{},{},{},{},{},{},{},{},{},{},{})",
        faction_id,
        member.id,
        member.job_level,
        title,
        contribute,
        last_online_time,
        purview[0],
        purview[1],
        purview[2],
        purview[3],
        purview[4],
        purview[5],
        purview[6],
        purview[7],
        purview[8],
        purview[9],
        purview[10]
    ))
}

fn build_faction_leave_word_insert(
    faction_id: i32,
    leave_word: &TagLeaveWord,
) -> Result<String, FactionLeaveWordSaveBlock> {
    let content = leave_word.content_wire_bytes()?;
    let escaped_content = CGame::check_point(&content[..content.len() - 1])?;
    let (escaped_content, _, _) = WINDOWS_1251.decode(&escaped_content);
    let time = leave_word.time;
    let leave_word_time = format!(
        "{}-{}-{} {}:{}:{}",
        time.year, time.month, time.day, time.hour, time.minute, time.second
    );

    Ok(format!(
        "INSERT INTO CSL_FACTIONLeaveWord VALUES({},{},{},N'{}','{}')",
        leave_word.id, faction_id, leave_word.player_id, escaped_content, leave_word_time
    ))
}

async fn execute_faction_statement(
    active_transaction: &mut WorldTdsClient,
    sql: String,
) -> Result<(), tiberius::error::Error> {
    active_transaction
        .simple_query(sql)
        .await?
        .into_results()
        .await?;
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp



// ============================================================================
// FUNCTION: CRsFaction::LoadFactionPronounce
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:565
// RVA: 0x000F9AD0
// ADDRESS: 004f9ad0
// PROTOTYPE: bool __thiscall LoadFactionPronounce(CFaction * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f9d15
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:585
// RVA: 0x000F9D15
// ADDRESS: 004f9d15
// PROTOTYPE: undefined Catch@004f9d15()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsFaction::LoadIconData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:662
// RVA: 0x000F9D60
// ADDRESS: 004f9d60
// PROTOTYPE: bool __thiscall LoadIconData(CFaction * param_1, _com_ptr_t<_com_IIID<_Recordset,&struct___s_GUID_const__GUID_00000556_0000_0010_8000_00aa006d2ea4>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004fa039
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:685
// RVA: 0x000FA039
// ADDRESS: 004fa039
// PROTOTYPE: undefined Catch@004fa039()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsFaction::LoadAbility
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:447
// RVA: 0x000FAEE0
// ADDRESS: 004faee0
// PROTOTYPE: bool __thiscall LoadAbility(map<long,CFaction*,std::less<long>,std::allocator<std::pair<long_const_,CFaction*>_>_> * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004fb1cd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:496
// RVA: 0x000FB1CD
// ADDRESS: 004fb1cd
// PROTOTYPE: undefined Catch@004fb1cd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004fb21e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:500
// RVA: 0x000FB21E
// ADDRESS: 004fb21e
// PROTOTYPE: undefined FUN_004fb21e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
// ============================================================================
// FUNCTION: CRsFaction::LoadLeaveWords
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:372
// RVA: 0x000FBDE0
// ADDRESS: 004fbde0
// PROTOTYPE: bool __thiscall LoadLeaveWords(map<long,CFaction*,std::less<long>,std::allocator<std::pair<long_const_,CFaction*>_>_> * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004fc50d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:440
// RVA: 0x000FC50D
// ADDRESS: 004fc50d
// PROTOTYPE: undefined Catch@004fc50d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004fc552
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:443
// RVA: 0x000FC552
// ADDRESS: 004fc552
// PROTOTYPE: undefined FUN_004fc552()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
// ============================================================================
// FUNCTION: CRsFaction::LoadFactionProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:196
// RVA: 0x000FCB20
// ADDRESS: 004fcb20
// PROTOTYPE: bool __thiscall LoadFactionProperty(map<long,CFaction*,std::less<long>,std::allocator<std::pair<long_const_,CFaction*>_>_> * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004fd7cc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:257
// RVA: 0x000FD7CC
// ADDRESS: 004fd7cc
// PROTOTYPE: undefined Catch@004fd7cc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004fd81c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:261
// RVA: 0x000FD81C
// ADDRESS: 004fd81c
// PROTOTYPE: undefined FUN_004fd81c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsFaction::LoadFactionMembers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:265
// RVA: 0x000FD840
// ADDRESS: 004fd840
// PROTOTYPE: bool __thiscall LoadFactionMembers(map<long,CFaction*,std::less<long>,std::allocator<std::pair<long_const_,CFaction*>_>_> * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004fe6a0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:363
// RVA: 0x000FE6A0
// ADDRESS: 004fe6a0
// PROTOTYPE: undefined Catch@004fe6a0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004fe6f0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:368
// RVA: 0x000FE6F0
// ADDRESS: 004fe6f0
// PROTOTYPE: undefined FUN_004fe6f0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsFaction::LoadFactionApplyPersons
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:505
// RVA: 0x000FE710
// ADDRESS: 004fe710
// PROTOTYPE: bool __thiscall LoadFactionApplyPersons(map<long,CFaction*,std::less<long>,std::allocator<std::pair<long_const_,CFaction*>_>_> * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004fec3f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:557
// RVA: 0x000FEC3F
// ADDRESS: 004fec3f
// PROTOTYPE: undefined Catch@004fec3f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004fec8f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:561
// RVA: 0x000FEC8F
// ADDRESS: 004fec8f
// PROTOTYPE: undefined FUN_004fec8f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
// ============================================================================
// FUNCTION: CRsFaction::LoadAllFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:141
// RVA: 0x000FF1D0
// ADDRESS: 004ff1d0
// PROTOTYPE: int __thiscall LoadAllFaction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ff4a4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp:188
// RVA: 0x000FF4A4
// ADDRESS: 004ff4a4
// PROTOTYPE: undefined Catch@004ff4a4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Field20::GetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsfaction.cpp
// RVA: 0x000FF4F0
// ADDRESS: 004ff4f0
// PROTOTYPE: _variant_t __thiscall GetValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//











// COMPONENT_VARIANT_END: WorldServer
