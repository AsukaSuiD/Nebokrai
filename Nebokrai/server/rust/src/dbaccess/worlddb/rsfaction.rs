//! DB-владелец `CRsFaction` WorldServer из `rsfaction.cpp`.
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Сохраняются раздельные операции faction property, members, applications,
//! leave words, icon, pronounce и ability, их SQL-порядок и исходные bool
//! результаты. Load строит записи в порядке провайдера; save не добавляет
//! транзакцию или rollback поверх уже выполненных команд. Tiberius и owned
//! values заменяют ADO/COM и MSVC containers без изменения DB-семантики.

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
    CFaction, FactionBaseProperty, FactionDatabaseBaseState, FactionInitialBlock, TagApplyPerson,
    TagLeaveWord, TagPronounceWord, UnterminatedLeaveWordField,
};
use crate::worldserver::appworld::organizingsystem::organizing::{
    EPurviewOwnState, TagMemInfo, TagTimeValue, UnterminatedMemberField,
};
use crate::worldserver::appworld::organizingsystem::organizingparam::COrganizingParam;
use crate::worldserver::worldserver::game::CGame;

const SAVE_FACTION_PROPERTY_SQL: &str = "IF EXISTS (SELECT TOP 1 ID FROM CSL_FACTION_BaseProperty WHERE ID = @P16 ORDER BY ID) BEGIN UPDATE TOP (1) CSL_FACTION_BaseProperty SET MasterID = @P1, Levels = @P2, Experience = @P3, OffenseVictorCounts = @P4, DefenceVictorCounts = @P5, VillageWarVictorCounts = @P6, MemberNums = @P7, UnionID = @P8, bPermit = @P9, lPro1 = @P10, lPro2 = @P11, DelRemainTime = @P12, country = @P13, GoodsWarCount = @P14, GoodsWarLastTime = @P15 WHERE ID = @P16; SELECT CAST(@@ROWCOUNT AS int) AS UpdatedRows END ELSE SELECT CAST(0 AS int) AS UpdatedRows";
const SAVE_FACTION_ABILITY_SQL: &str = "IF EXISTS (SELECT TOP 1 FactionID FROM CSL_FACTION_Ability WHERE FactionID = @P1 ORDER BY FactionID) BEGIN UPDATE TOP (1) CSL_FACTION_Ability SET Pronounce = @P2, IconData = @P3, LastUploadIconDataTime = @P4 WHERE FactionID = @P1 END ELSE BEGIN INSERT INTO CSL_FACTION_Ability (FactionID, Pronounce, IconData, LastUploadIconDataTime) VALUES (@P1, @P2, @P3, @P4) END";
const LOAD_FACTION_PROPERTY_SQL: &str = "SELECT * FROM CSL_FACTION_BaseProperty ORDER BY id";
const LOAD_FACTION_MEMBERS_SQL: &str = "SELECT CSL_FACTION_Members.*, CSL_PLAYER_BASE.Name,CSL_PLAYER_BASE.Levels,CSL_PLAYER_BASE.Occupation FROM CSL_FACTION_Members join CSL_PLAYER_BASE on CSL_FACTION_Members.playerid = CSL_PLAYER_BASE.id ORDER BY FactionID";
const LOAD_FACTION_LEAVE_WORDS_SQL: &str = "SELECT CSL_FACTIONLeaveWord.*,csl_player_base.name FROM CSL_FACTIONLeaveWord join csl_player_base ON CSL_FACTIONLeaveWord.leavewordplayerid=csl_player_base.id WHERE DATEDIFF(day,LeaveWordTime,GETDATE())<=10 ORDER BY FactionID   ";
const LOAD_FACTION_ABILITY_SQL: &str = "SELECT * FROM CSL_FACTION_Ability ORDER BY FactionID";
const LOAD_FACTION_APPLY_PERSONS_SQL: &str = "SELECT CSL_Faction_Apply.*,CSL_PLAYER_BASE.Name,CSL_PLAYER_BASE.Levels,CSL_PLAYER_BASE.Occupation FROM CSL_Faction_Apply join csl_player_base on CSL_Faction_Apply.PlayerID=csl_player_base.id ORDER BY FactionID";

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionMemberLoadBlock {
    pub(crate) visible_len: usize,
    pub(crate) capacity: usize,
}

pub(crate) enum FactionMembersLoadOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionMemberLoadBlock),
}

/// Безопасная граница короткого binary `Pronounce`: оригинал memcpy читал бы за
/// полученный DB chunk, а внешняя реакция повреждённой строки не определена.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactionPronounceLoadBlock {
    pub(crate) actual_size: usize,
}

pub(crate) enum FactionAbilityLoadOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionPronounceLoadBlock),
}

/// Итог верхнего `LoadAllFaction` до передачи готовой map парному controller-owner.
/// PDB задаёт `int` return; для raw temporary `std::map` это его число элементов
/// до cleanup. Неуспешная ветвь удаляет map, но возвращаемое в `Initialize` log
/// число относится к сохранённому до удаления staging-prefix.
pub(crate) enum FactionLoadOutcome {
    ReturnedTrue {
        factions: FactionPropertyLoadStaging,
        reported_count: i32,
    },
    ReturnedFalse {
        factions: FactionPropertyLoadStaging,
        reported_count: i32,
    },
    BlockedInitial {
        factions: FactionPropertyLoadStaging,
        block: FactionInitialBlock,
        reported_count: i32,
    },
    BlockedMember {
        factions: FactionPropertyLoadStaging,
        block: FactionMemberLoadBlock,
        reported_count: i32,
    },
    BlockedPronounce {
        factions: FactionPropertyLoadStaging,
        block: FactionPronounceLoadBlock,
        reported_count: i32,
    },
}

fn reported_faction_load_count(factions: &FactionPropertyLoadStaging) -> i32 {
    i32::try_from(factions.len()).unwrap_or(i32::MAX)
}

struct FactionPropertySaveSnapshot {
    master_id: i32,
    base_property: FactionBaseProperty,
    delete_remain_time: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionSaveProjectionBlock {
    MasterId,
    BaseProperty,
    DeleteRemainTime,
}

pub(crate) struct FactionSaveSnapshot<'faction> {
    pub(crate) faction: &'faction mut CFaction,
    canonical_goods_war_count: Option<i32>,
    property: Option<FactionPropertySaveSnapshot>,
}

impl<'faction> FactionSaveSnapshot<'faction> {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactionLeaveWordSaveBlock {
    UnterminatedContent(UnterminatedLeaveWordField),
}

impl fmt::Display for FactionLeaveWordSaveBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedContent(block) => block.fmt(formatter),
        }
    }
}

impl Error for FactionLeaveWordSaveBlock {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnterminatedContent(block) => Some(block),
        }
    }
}

impl From<UnterminatedLeaveWordField> for FactionLeaveWordSaveBlock {
    fn from(block: UnterminatedLeaveWordField) -> Self {
        Self::UnterminatedContent(block)
    }
}

#[derive(Debug)]
pub(crate) enum FactionLeaveWordsSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(FactionLeaveWordSaveBlock),
}

#[derive(Debug)]
pub(crate) enum FactionSaveBlock {
    Member(UnterminatedMemberField),
    LeaveWord(FactionLeaveWordSaveBlock),
}

#[derive(Debug)]
pub(crate) enum FactionSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(FactionSaveBlock),
}

#[derive(Debug)]
pub(crate) struct RsFactionNotice {
    pub(crate) operation: RsFactionOperation,
    pub(crate) error: RsFactionSaveError,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RsFactionOperation {
    LoadFactionProperty,
    LoadFactionMembers,
    LoadLeaveWords,
    LoadAbility,
    LoadFactionApplyPersons,
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
    MissingFaction { faction_id: i32 },
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
            Self::MissingFaction { faction_id } => {
                write!(formatter, "не найдена faction {faction_id} для member DB-строки")
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
            | Self::MissingRequiredValue(_)
            | Self::MissingFaction { .. } => None,
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
/// диагностическое сообщение. Runtime SQL и DB-значения намеренно не выдаются.
#[derive(Debug)]
enum FactionLoadReadError {
    Database(tiberius::error::Error),
    MissingRequiredValue(&'static str),
    MissingFaction { faction_id: i32 },
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
            FactionLoadReadError::MissingFaction { faction_id } => Self::MissingFaction { faction_id },
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

async fn load_faction_members_rows(
    connection: &mut WorldTdsClient,
    factions: &mut FactionPropertyLoadStaging,
    notices: &mut VecDeque<RsFactionNotice>,
) -> FactionMembersLoadOutcome {
    let stream = match connection.simple_query(LOAD_FACTION_MEMBERS_SQL).await {
        Ok(stream) => stream,
        Err(error) => {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadFactionMembers,
                error: RsFactionSaveError::Database(error.into()),
            });
            return FactionMembersLoadOutcome::ReturnedFalse;
        }
    };
    let rows = match stream.into_first_result().await {
        Ok(rows) => rows,
        Err(error) => {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadFactionMembers,
                error: RsFactionSaveError::Database(error.into()),
            });
            return FactionMembersLoadOutcome::ReturnedFalse;
        }
    };

    for row in &rows {
        let faction_id = match read_faction_i32(row, "FactionID") {
            Ok(faction_id) => faction_id,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadFactionMembers,
                    error: error.into(),
                });
                return FactionMembersLoadOutcome::ReturnedFalse;
            }
        };
        let Some(faction) = factions.get_mut(&faction_id) else {
 // Оригинал owner печатает `LoadGuildMembers: guild id not exist.` и
 // только MoveNext; остальные поля текущей строки он не читает.
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadFactionMembers,
                error: RsFactionSaveError::MissingFaction { faction_id },
            });
            continue;
        };
        let member = match read_faction_member(row) {
            Ok(member) => member,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadFactionMembers,
                    error: error.into(),
                });
                return FactionMembersLoadOutcome::ReturnedFalse;
            }
        };
        let name_length = visible_legacy_text_len(&member.name);
        if name_length >= 32 {
 // `strcpy(local_120[32], Name)` выходил за стек. Нет точного
 // внешнего результата corrupted row, поэтому не дополняем/режем.
            return FactionMembersLoadOutcome::BlockedMissingFact(FactionMemberLoadBlock {
                visible_len: name_length,
                capacity: 32,
            });
        }
        faction.load_database_member(member.member);
    }
    FactionMembersLoadOutcome::ReturnedTrue
}

struct FactionMemberDatabaseRow {
    name: Vec<u8>,
    member: TagMemInfo,
}

fn read_faction_member(row: &Row) -> Result<FactionMemberDatabaseRow, FactionLoadReadError> {
    let name = read_faction_text(row, "Name")?;
    let visible_name = visible_legacy_text(&name);
    if visible_name.len() >= 32 {
 // Возвращаем исходные bytes в row-промежутке, чтобы caller сохранил
 // prefix и выразил отдельную safe-границу без DB notice.
        return Ok(FactionMemberDatabaseRow {
            name,
            member: TagMemInfo::from_complete_fields(
                0,
                [0; 32],
                0,
                0,
                0,
                [0; 64],
                [EPurviewOwnState::No; 11],
                [0; 64],
                TagTimeValue {
                    year: 0,
                    month: 0,
                    day_of_week: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    milliseconds: 0,
                },
                false,
            ),
        });
    }
    let mut member_name = [0; 32];
    member_name[..visible_name.len()].copy_from_slice(visible_name);
    let title = read_faction_text(row, "Title")?;
    let mut member_title = [0; 64];
 // Оригинал проверял ANSI `std::string::size() < 0x15`; long title оставлял
 // C-string пустой, а не обрезанный.
    let visible_title = visible_legacy_text(&title);
    if visible_title.len() < 0x15 {
        member_title[..visible_title.len()].copy_from_slice(visible_title);
    }
    Ok(FactionMemberDatabaseRow {
        name,
        member: TagMemInfo::from_complete_fields(
            read_faction_i32(row, "PlayerID")?,
            member_name,
            read_faction_i32(row, "Levels")?,
            read_faction_i32(row, "Occupation")?,
            read_faction_i32(row, "MemberLvl")?,
            member_title,
            read_faction_member_purview(row)?,
            [0; 64],
            read_faction_time(row, "LastOnlineTime")?,
            read_faction_bool(row, "bControbute")?,
        ),
    })
}

fn read_faction_member_purview(
    row: &Row,
) -> Result<[EPurviewOwnState; 11], FactionLoadReadError> {
    const COLUMNS: [&str; 11] = [
        "PV_Disband",
        "PV_Exit",
        "PV_DubJobLvl",
        "PV_ConMem",
        "PV_FireOut",
        "PV_Pronounce",
        "PV_LeaveWord",
        "PV_EditLeaveWord",
        "PV_ObtainTax",
        "PV_OperCityGate",
        "PV_EndueROR",
    ];
    let mut purview = [EPurviewOwnState::No; 11];
    for (index, column) in COLUMNS.into_iter().enumerate() {
        purview[index] = match read_faction_i32(row, column)? {
            0 => EPurviewOwnState::No,
            1 => EPurviewOwnState::Forbid,
            2 => EPurviewOwnState::Permit,
 // C++ записывал arbitrary signed `long` в enum storage. Safe
 // Rust не materialize-ит invalid enum и не выдумывает его wire.
            _ => return Err(FactionLoadReadError::MissingRequiredValue(column)),
        };
    }
    Ok(purview)
}

fn visible_legacy_text(source: &[u8]) -> &[u8] {
    source
        .iter()
        .position(|byte| *byte == 0)
        .map_or(source, |end| &source[..end])
}

fn visible_legacy_text_len(source: &[u8]) -> usize {
    visible_legacy_text(source).len()
}

async fn query_faction_rows(
    connection: &mut WorldTdsClient,
    sql: &str,
) -> Result<Vec<Row>, tiberius::error::Error> {
    connection.simple_query(sql).await?.into_first_result().await
}

async fn load_faction_property_rows(
    connection: &mut WorldTdsClient,
    master_title: &[u8],
    game: &CGame,
    parameters: &COrganizingParam,
    notices: &mut VecDeque<RsFactionNotice>,
) -> FactionPropertyLoadOutcome {
    let rows = match query_faction_rows(connection, LOAD_FACTION_PROPERTY_SQL).await {
        Ok(rows) => rows,
        Err(error) => {
            notices.push_back(RsFactionNotice { operation: RsFactionOperation::LoadFactionProperty, error: RsFactionSaveError::Database(error.into()) });
            return FactionPropertyLoadOutcome::ReturnedFalse { factions: BTreeMap::new() };
        }
    };
    let mut factions = BTreeMap::new();
    for row in &rows {
        let state = match read_faction_database_base_state(row) {
            Ok(state) => state,
            Err(error) => {
                notices.push_back(RsFactionNotice { operation: RsFactionOperation::LoadFactionProperty, error: error.into() });
                return FactionPropertyLoadOutcome::ReturnedFalse { factions };
            }
        };
        let faction_id = state.faction_id;
        match CFaction::from_database_base_state(state, master_title, game, parameters) {
            Ok(faction) => { factions.insert(faction_id, faction); }
            Err(block) => return FactionPropertyLoadOutcome::BlockedMissingFact { factions, block },
        }
    }
    FactionPropertyLoadOutcome::ReturnedTrue { factions }
}

async fn load_faction_leave_words_rows(
    connection: &mut WorldTdsClient,
    factions: &mut FactionPropertyLoadStaging,
    notices: &mut VecDeque<RsFactionNotice>,
) -> bool {
    let rows = match query_faction_rows(connection, LOAD_FACTION_LEAVE_WORDS_SQL).await {
        Ok(rows) => rows,
        Err(error) => {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadLeaveWords,
                error: RsFactionSaveError::Database(error.into()),
            });
            return false;
        }
    };
    for row in &rows {
        let faction_id = match read_faction_i32(row, "FactionID") {
            Ok(value) => value,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadLeaveWords,
                    error: error.into(),
                });
                return false;
            }
        };
        let Some(faction) = factions.get_mut(&faction_id) else {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadLeaveWords,
                error: RsFactionSaveError::MissingFaction { faction_id },
            });
            continue;
        };
        let leave_word = match read_faction_leave_word(row) {
            Ok(value) => value,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadLeaveWords,
                    error: error.into(),
                });
                return false;
            }
        };
        let _ = faction.load_leave_word(leave_word);
    }
    true
}

fn read_faction_leave_word(row: &Row) -> Result<TagLeaveWord, FactionLoadReadError> {
    let name = read_faction_text(row, "Name")?;
    let content = read_faction_text(row, "Content")?;
    let mut fixed_name = [0; 20];
    let mut fixed_content = [0; 212];
    let visible_name = visible_legacy_text(&name);
    let visible_content = visible_legacy_text(&content);
 // Оригинал оставлял пустое Content при length >= 201; Name был ограничен
 // DB-схемой. Для corrupt Name >=20 owner не создаёт unsafe C-string.
    if visible_name.len() < fixed_name.len() {
        fixed_name[..visible_name.len()].copy_from_slice(visible_name);
    }
    if visible_content.len() < 0xC9 {
        fixed_content[..visible_content.len()].copy_from_slice(visible_content);
    }
    Ok(TagLeaveWord::from_complete_fields(
        read_faction_i32(row, "ID")?,
        read_faction_i32(row, "LeaveWordPlayerID")?,
        fixed_name,
        read_faction_time(row, "LeaveWordTime")?,
        fixed_content,
    ))
}

async fn load_faction_apply_persons_rows(
    connection: &mut WorldTdsClient,
    factions: &mut FactionPropertyLoadStaging,
    notices: &mut VecDeque<RsFactionNotice>,
) -> bool {
    let rows = match query_faction_rows(connection, LOAD_FACTION_APPLY_PERSONS_SQL).await {
        Ok(rows) => rows,
        Err(error) => {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadFactionApplyPersons,
                error: RsFactionSaveError::Database(error.into()),
            });
            return false;
        }
    };
    for row in &rows {
        let faction_id = match read_faction_i32(row, "FactionID") {
            Ok(value) => value,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadFactionApplyPersons,
                    error: error.into(),
                });
                return false;
            }
        };
        let Some(faction) = factions.get_mut(&faction_id) else {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadFactionApplyPersons,
                error: RsFactionSaveError::MissingFaction { faction_id },
            });
            continue;
        };
        let name = match read_faction_text(row, "Name") {
            Ok(name) => name,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadFactionApplyPersons,
                    error: error.into(),
                });
                return false;
            }
        };
        let mut fixed_name = [0; 20];
        let visible_name = visible_legacy_text(&name);
        if visible_name.len() < fixed_name.len() {
            fixed_name[..visible_name.len()].copy_from_slice(visible_name);
        }
        let person = match (|| {
            Ok::<_, FactionLoadReadError>(TagApplyPerson::from_complete_fields(
                read_faction_i32(row, "PlayerID")?,
                fixed_name,
                read_faction_i32(row, "Occupation")?,
                read_faction_i32(row, "Levels")?,
            ))
        })() {
            Ok(value) => value,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadFactionApplyPersons,
                    error: error.into(),
                });
                return false;
            }
        };
        faction.load_database_apply_person(person);
    }
    true
}

async fn load_faction_ability_rows(
    connection: &mut WorldTdsClient,
    factions: &mut FactionPropertyLoadStaging,
    notices: &mut VecDeque<RsFactionNotice>,
) -> FactionAbilityLoadOutcome {
    let rows = match query_faction_rows(connection, LOAD_FACTION_ABILITY_SQL).await {
        Ok(rows) => rows,
        Err(error) => {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadAbility,
                error: RsFactionSaveError::Database(error.into()),
            });
            return FactionAbilityLoadOutcome::ReturnedFalse;
        }
    };
    for row in &rows {
        let faction_id = match read_faction_i32(row, "FactionID") {
            Ok(value) => value,
            Err(error) => {
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadAbility,
                    error: error.into(),
                });
                return FactionAbilityLoadOutcome::ReturnedFalse;
            }
        };
        let Some(faction) = factions.get_mut(&faction_id) else {
            notices.push_back(RsFactionNotice {
                operation: RsFactionOperation::LoadAbility,
                error: RsFactionSaveError::MissingFaction { faction_id },
            });
            continue;
        };
        match read_faction_ability(row) {
            Ok((pronounce, icon_time)) => {
                if let Some(pronounce) = pronounce {
                    faction.load_database_pronounce(pronounce);
                }
                faction.load_database_icon_upload_time(icon_time);
            }
            Err(FactionAbilityReadError::ShortPronounce(actual_size)) => {
                return FactionAbilityLoadOutcome::BlockedMissingFact(
                    FactionPronounceLoadBlock { actual_size },
                );
            }
            Err(FactionAbilityReadError::Read(error)) => {
 // Оригинал LoadAbility считал false private helper-а, печатал
 // failure и переходил к следующему recordset row.
                notices.push_back(RsFactionNotice {
                    operation: RsFactionOperation::LoadAbility,
                    error: error.into(),
                });
            }
        }
    }
    FactionAbilityLoadOutcome::ReturnedTrue
}

enum FactionAbilityReadError {
    Read(FactionLoadReadError),
    ShortPronounce(usize),
}

fn read_faction_binary<'row>(
    row: &'row Row,
    column: &'static str,
) -> Result<Option<&'row [u8]>, FactionLoadReadError> {
    match row.try_get::<&[u8], _>(column) {
        Ok(value) => Ok(value),
        Err(error) => Err(FactionLoadReadError::Database(error)),
    }
}

fn read_faction_ability(
    row: &Row,
) -> Result<(Option<TagPronounceWord>, TagTimeValue), FactionAbilityReadError> {
    let pronounce_bytes = read_faction_binary(row, "Pronounce")
        .map_err(FactionAbilityReadError::Read)?;
    let pronounce = match pronounce_bytes {
        None | Some([]) => None,
        Some(bytes) => TagPronounceWord::from_database_blob(bytes)
            .ok_or(FactionAbilityReadError::ShortPronounce(bytes.len()))
            .map(Some)?,
    };
            // `LoadIconData` читает chunk, но не записывает его в `m_IconData`;
            // само чтение сохраняет прежнюю DB-ошибку неверного типа.
    let _ = read_faction_binary(row, "IconData").map_err(FactionAbilityReadError::Read)?;
    let icon_time = read_faction_time(row, "LastUploadIconDataTime")
        .map_err(FactionAbilityReadError::Read)?;
    Ok((pronounce, icon_time))
}

pub(crate) trait RsFactionOwner {
 /// Выполняет полный оригинал порядок пяти load-owner-ов на одном World DB
 /// connection и возвращает ready-to-publish faction staging map.
    async fn load_all_factions(
        &mut self,
        master_title: &[u8],
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> FactionLoadOutcome;

 /// Открывает отдельное World DB соединение и materialize-ит только
 /// `LoadFactionProperty`; оставшиеся load-owner-ы пока не смешиваются с
 /// этим staging map.
    async fn load_faction_property(
        &mut self,
        master_title: &[u8],
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> FactionPropertyLoadOutcome;

    async fn save_faction(
        &mut self,
        snapshot: Option<&mut FactionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> FactionSaveOutcome;

    async fn del_faction(
        &mut self,
        faction_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

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

    fn pop_notice(&mut self) -> Option<RsFactionNotice>;
}

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
    async fn load_all_factions(
        &mut self,
        master_title: &[u8],
        game: &CGame,
        parameters: &COrganizingParam,
    ) -> FactionLoadOutcome {
        let Some(settings) = self.settings.clone() else {
            self.notices.push_back(RsFactionNotice { operation: RsFactionOperation::LoadFactionProperty, error: RsFactionSaveError::MissingSettings });
            return FactionLoadOutcome::ReturnedFalse {
                factions: BTreeMap::new(),
                reported_count: 0,
            };
        };
        let mut connection = match settings.connect().await {
            Ok(connection) => connection,
            Err(error) => {
                self.notices.push_back(RsFactionNotice { operation: RsFactionOperation::LoadFactionProperty, error: RsFactionSaveError::Connection(error) });
                return FactionLoadOutcome::ReturnedFalse {
                    factions: BTreeMap::new(),
                    reported_count: 0,
                };
            }
        };
        let mut factions = match load_faction_property_rows(&mut connection, master_title, game, parameters, &mut self.notices).await {
            FactionPropertyLoadOutcome::ReturnedTrue { factions } => factions,
            FactionPropertyLoadOutcome::ReturnedFalse { factions } => {
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::ReturnedFalse {
                    factions,
                    reported_count,
                };
            }
            FactionPropertyLoadOutcome::BlockedMissingFact { factions, block } => {
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::BlockedInitial {
                    factions,
                    block,
                    reported_count,
                };
            }
        };
        match load_faction_members_rows(&mut connection, &mut factions, &mut self.notices).await {
            FactionMembersLoadOutcome::ReturnedTrue => {}
            FactionMembersLoadOutcome::ReturnedFalse => {
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::ReturnedFalse {
                    factions,
                    reported_count,
                };
            }
            FactionMembersLoadOutcome::BlockedMissingFact(block) => {
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::BlockedMember {
                    factions,
                    block,
                    reported_count,
                };
            }
        }
        if !load_faction_leave_words_rows(&mut connection, &mut factions, &mut self.notices).await {
            let reported_count = reported_faction_load_count(&factions);
            return FactionLoadOutcome::ReturnedFalse {
                factions,
                reported_count,
            };
        }
        match load_faction_ability_rows(&mut connection, &mut factions, &mut self.notices).await {
            FactionAbilityLoadOutcome::ReturnedTrue => {}
            FactionAbilityLoadOutcome::ReturnedFalse => {
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::ReturnedFalse {
                    factions,
                    reported_count,
                };
            }
            FactionAbilityLoadOutcome::BlockedMissingFact(block) => {
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::BlockedPronounce {
                    factions,
                    block,
                    reported_count,
                };
            }
        }
        if !load_faction_apply_persons_rows(&mut connection, &mut factions, &mut self.notices).await {
            let reported_count = reported_faction_load_count(&factions);
            return FactionLoadOutcome::ReturnedFalse {
                factions,
                reported_count,
            };
        }
        for faction in factions.values_mut() {
            if faction.complete_database_load(parameters).is_err() {
                self.notices.push_back(RsFactionNotice { operation: RsFactionOperation::LoadFactionProperty, error: RsFactionSaveError::MissingRequiredValue("CFaction::m_Property") });
                let reported_count = reported_faction_load_count(&factions);
                return FactionLoadOutcome::ReturnedFalse {
                    factions,
                    reported_count,
                };
            }
        }
        let reported_count = reported_faction_load_count(&factions);
        FactionLoadOutcome::ReturnedTrue {
            factions,
            reported_count,
        }
    }

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

 // читает mask один раз: property-нормализация может снова
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
    let escaped_content = CGame::check_point(&content[..content.len() - 1]);
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
