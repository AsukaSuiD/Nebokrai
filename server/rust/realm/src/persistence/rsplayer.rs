//! DB-owner игрока мира: трейт `RsPlayerOwner` и его data-семьи, извлечённые
//! из старого `rsplayer`. Источник контракта — точная пара `Nworldserver.exe`
//! и `WorldServer.pdb`.
//!
//! Owner охватывает create/open/load/save игрока, honor ranks и lookup-операции
//! account/name/country. Generic-параметр `PlayerT` заменяет прямую ссылку на
//! игрока: связка с `CPlayer` остаётся у DB-реализации владельца. Generic
//! `Organizing` у `stat_ranks` проходит через существующий шов
//! `PlayerRankOrganizingLookup`, которым пользуется `CPlayerRanks::add_rank`.
//! Load-семья параметризована `AddonBlockT`/`InsertBlockT` и связана с трейтом
//! через associated types `DbGoodsOwner`, как и в `content::dbgoods`.
//!
//! `GetPlayerDeletionDate` различает `0` и catch-result `-1`; caller считает
//! `-1` ненулевым timestamp. Очередность SQL и уже выполненные partial effects
//! не откатываются автоматически. Параметризованный Tiberius, owned byte
//! strings и typed snapshots заменяют ADO/COM, globals и fixed buffers, не
//! меняя схемы, provider-order, значения отказа или wire-контракты caller-а.

use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::error::Error;
use std::fmt;

use nebokrai_shared::resources::CThingSetup;
use nebokrai_shared::values::TagTimeArithmeticBlock;

use crate::activities::rsjjcsys::{PlayerJjcDataSnapshot, PlayerJjcLoadFailure, RsJjcSysOwner};
use crate::characters::honordb::{
    HonorRanksDbDataSnapshot, HonorRanksLoadOutcome, HonorRanksLoadSink, HonorRanksSavePeriod,
    HonorRanksType,
};
use crate::characters::playerranks::{
    CPlayerRanks, PlayerRankAddBlock, PlayerRankOrganizingLookup,
};
use crate::content::dbgoods::{
    DbGoodsOwner, GoodsLoadBlock, GoodsLoadFailure, PlayerGoodsFiledSnapshot,
};
use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::goodslistener::GoodsTraversalBlock;
use crate::persistence::rssetup::{WorldDatabaseConnectionError, WorldTdsClient};

pub trait HonorRanksFieldSink {
    type Error;

    fn put_honor_ranks_field(
        &mut self,
        rank_type: HonorRanksType,
        blob: Vec<u8>,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug)]
pub struct HonorRanksBlobBlock {
    pub total_entries: usize,
}

#[derive(Debug)]
pub enum HonorRanksByTypeSaveOutcome<E> {
    Saved,
    FieldFailed(E),
    BlockedMissingFact(HonorRanksBlobBlock),
}

#[derive(Clone, Copy, Debug)]
pub struct HonorRanksSaveBlock {
    pub period: HonorRanksSavePeriod,
    pub rank_type: HonorRanksType,
    pub blob: HonorRanksBlobBlock,
}

#[derive(Debug)]
pub enum HonorRanksSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(HonorRanksSaveBlock),
}

#[derive(Default)]
pub struct CollectedHonorRanksFields {
    pub fields: Vec<(HonorRanksType, Vec<u8>)>,
}

impl HonorRanksFieldSink for CollectedHonorRanksFields {
    type Error = Infallible;

    fn put_honor_ranks_field(
        &mut self,
        rank_type: HonorRanksType,
        blob: Vec<u8>,
    ) -> Result<(), Self::Error> {
        self.fields.push((rank_type, blob));
        Ok(())
    }
}

pub struct PlayerCreationBaseSnapshot {
    pub id: i32,
    pub name: Vec<u8>,
    pub account: Vec<u8>,
    pub level: u8,
    pub occupation: u8,
    pub sex: u8,
    pub country: u8,
    pub head: u8,
    pub equipment_ids: [u32; 11],
    pub equipment_levels: [u8; 11],
    pub region_id: i32,
}

pub struct PlayerBaseSaveSnapshot<'a> {
    pub id: i32,
    pub name: &'a [u8],
    pub level: u8,
    pub occupation: u8,
    pub sex: u8,
    pub country: u8,
    pub head: u8,
    pub equipment_ids: [u32; 11],
    pub equipment_levels: [i32; 11],
    pub region_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerBaseDatabaseRow {
    pub id: u32,
    pub name: Vec<u8>,
    pub level: u8,
    pub occupation: u8,
    pub sex: u8,
    pub country: u8,
    pub head: u8,
    pub equipment_ids: [u32; 11],
    pub equipment_levels: [u8; 11],
    pub region_id: i32,
}

#[derive(Debug)]
pub enum PlayerBaseLoadFailure {
    MissingConnection,
    Database,
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
pub enum PlayerBaseCreateOutcome {
    Created,
    Failed,
}

/// Полный caller-owned view трёх стадий `CRsPlayer::CreatePlayer`.
///
/// Все вложенные ID обязаны происходить из одного исходного `CPlayer`:
/// `base.id == abilities.scalar.id == goods.player_id`.
pub struct PlayerCreationSnapshot<'player, 'goods_snapshot> {
    pub base: PlayerCreationBaseSnapshot,
    pub abilities: PlayerAbilityCreationSnapshot<'player>,
    pub goods: PlayerGoodsFiledSnapshot<'goods_snapshot>,
}

#[derive(Debug)]
pub enum PlayerCreateBlock {
    Goods(GoodsTraversalBlock),
}

#[derive(Debug)]
pub enum PlayerCreateOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(PlayerCreateBlock),
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerDeleteTimeBlock {
    pub deletion_time: i32,
}

#[derive(Debug)]
pub enum PlayerDeleteOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(PlayerDeleteTimeBlock),
}

#[derive(Debug)]
pub struct RsPlayerNotice {
    pub operation: RsPlayerOperation,
    pub error: RsPlayerSaveError,
}

#[derive(Clone, Copy, Debug)]
pub enum RsPlayerOperation {
    OpenPlayerBaseCount,
    OpenPlayerBase,
    GetPlayerDeletionDate,
    GetPlayerCountryById,
    GetPlayerNameById,
    ValidatePlayerIdInCdkey,
    IsNameExist,
    GetPlayerId,
    GetCdKey,
    StatRanks,
    Outer,
    BaseRow,
    SaveBaseRow,
    AbilityRow,
    SaveAbilityRow,
    SaveQuestData,
    Restore,
    Delete,
    HonorRanksLoad,
    HonorRanksInsert,
    HonorRanksSave,
}

#[derive(Debug)]
pub enum RsPlayerSaveError {
    Database(RsPlayerDatabaseError),
    MissingConnection,
    PlayerRanksStatFailed,
    MissingBaseRow,
    MissingAbilityRow,
    MissingHonorRanksRow { period: HonorRanksSavePeriod },
    MalformedPlayerBaseRow,
    JjcSaveFailed,
}

impl fmt::Display for RsPlayerSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => write!(formatter, "не передано соединение World player DB"),
            Self::PlayerRanksStatFailed => {
                write!(formatter, "пересчёт рейтинга игроков завершился ошибкой")
            }
            Self::MissingBaseRow => write!(formatter, "не найдена строка CSL_PLAYER_BASE"),
            Self::MissingAbilityRow => write!(formatter, "не найдена строка CSL_PLAYER_ABILITY"),
            Self::MissingHonorRanksRow { period } => write!(
                formatter,
                "не найдена строка CSL_HonorRanks для {period:?}"
            ),
            Self::MalformedPlayerBaseRow => {
                write!(formatter, "некорректная строка CSL_PLAYER_BASE")
            }
            Self::JjcSaveFailed => {
                write!(formatter, "отдельное сохранение JJc завершилось ошибкой")
            }
        }
    }
}

impl Error for RsPlayerSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::MissingConnection
            | Self::PlayerRanksStatFailed
            | Self::MissingBaseRow
            | Self::MissingAbilityRow
            | Self::MissingHonorRanksRow { .. }
            | Self::MalformedPlayerBaseRow
            | Self::JjcSaveFailed => None,
        }
    }
}

#[derive(Debug)]
pub struct RsPlayerDatabaseError(tiberius::error::Error);

impl fmt::Display for RsPlayerDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World player DB: {}", self.0)
    }
}

impl Error for RsPlayerDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsPlayerDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

#[derive(Debug)]
pub enum PlayerRanksStatFailure {
    MissingConnection,
    Database {
        row_index: Option<usize>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerRanksStatBlock {
    MaximumCountUnknown,
    AddRank {
        row_index: usize,
        source: PlayerRankAddBlock,
    },
}

#[derive(Debug)]
pub enum PlayerRanksStatOutcome {
    ReturnedTrue { row_count: usize },
    ReturnedFalse(PlayerRanksStatFailure),
    BlockedMissingFact(PlayerRanksStatBlock),
}

pub trait RsPlayerOwner<PlayerT> {
    fn get_player_count_in_db_by_cdkey(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Option<u8>> + Send;

 /// Повторяет World-only `GetPlayerCountInCdkey`: DB sentinel сохраняется,
 /// successful byte складывается с live creation-count с x86 wrapping.
    fn get_player_count_in_cdkey(
        &mut self,
        account: &[u8],
        creation_count: u8,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Option<u8>> {
        async move {
            self.get_player_count_in_db_by_cdkey(account, active_transaction)
                .await
                .and_then(|database_count| {
                    let count = database_count.wrapping_add(creation_count);
                    (count != u8::MAX).then_some(count)
                })
        }
    }

    fn open_player_base_in_db(
        &mut self,
        account: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<
        Output = Result<Vec<PlayerBaseDatabaseRow>, PlayerBaseLoadFailure>,
    > + Send;

    fn get_player_deletion_date(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = i32> + Send;

    fn get_player_country_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = u8> + Send;

    fn get_player_name_by_id(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Vec<u8>> + Send;

    fn validate_player_id_in_cdkey(
        &mut self,
        account: &[u8],
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn is_name_exist(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn get_cd_key(
        &mut self,
        player_name: &[u8],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = Vec<u8>> + Send;

    fn stat_ranks<Organizing: PlayerRankOrganizingLookup>(
        &mut self,
        ranks: &mut CPlayerRanks,
        organizing: &Organizing,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = PlayerRanksStatOutcome>;

    fn load_player<J, G, WeekDay>(
        &mut self,
        player: &mut PlayerT,
        active_transaction: Option<&mut WorldTdsClient>,
        thing_setup: &CThingSetup,
        get_week_day: WeekDay,
        jjc_owner: &mut J,
        goods_owner: &mut G,
        goods_registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> impl std::future::Future<Output = PlayerLoadOutcome<G::AddonBlock, G::InsertBlock>>
    where
        J: RsJjcSysOwner,
        G: DbGoodsOwner<PlayerT>,
        WeekDay: FnMut() -> u16;

    fn create_player<J: RsJjcSysOwner, G: DbGoodsOwner<PlayerT>>(
        &mut self,
        snapshot: Option<&PlayerCreationSnapshot<'_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> impl std::future::Future<Output = PlayerCreateOutcome>;

    fn save_player<J: RsJjcSysOwner, G: DbGoodsOwner<PlayerT>>(
        &mut self,
        snapshot: Option<&PlayerSaveSnapshot<'_, '_, '_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
        goods_owner: &mut G,
    ) -> impl std::future::Future<Output = PlayerSaveOutcome>;

    fn create_player_base(
        &mut self,
        snapshot: &PlayerCreationBaseSnapshot,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = PlayerBaseCreateOutcome> + Send;

    fn save_player_base(
        &mut self,
        snapshot: Option<&PlayerBaseSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn create_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: &PlayerAbilityCreationSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
        jjc_owner: &mut J,
    ) -> impl std::future::Future<Output = bool>;

    fn save_player_abilities<J: RsJjcSysOwner>(
        &mut self,
        snapshot: Option<&PlayerAbilitySaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
        jjc_owner: &mut J,
    ) -> impl std::future::Future<Output = bool>;

    fn save_quest_data(
        &mut self,
        snapshot: Option<&PlayerQuestSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn restore_player(
        &mut self,
        player_id: u32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn delete_player(
        &mut self,
        player_id: u32,
        deletion_time: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = PlayerDeleteOutcome> + Send;

    fn load_honor_ranks<S: HonorRanksLoadSink>(
        &mut self,
        sink: &mut S,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = HonorRanksLoadOutcome>;

    fn insert_honor_ranks(
        &mut self,
        snapshot: &HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_honor_ranks_by_type<S: HonorRanksFieldSink>(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
        sink: &mut S,
    ) -> HonorRanksByTypeSaveOutcome<S::Error>;

    fn save_honor_ranks(
        &mut self,
        snapshot: &mut HonorRanksDbDataSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = HonorRanksSaveOutcome> + Send;

    fn pop_notice(&mut self) -> Option<RsPlayerNotice>;
}

pub struct PlayerAbilityScalarSnapshot<'a> {
    pub id: i32,
    pub name: &'a [u8],
    pub region_id: i32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub dir: i32,
    pub account: &'a [u8],
    pub title: &'a [u8],
    pub level: u8,
    pub exp: u32,
    pub head_pic: u8,
    pub face_pic: u8,
    pub occupation: u8,
    pub sex: u8,
    pub spouse_id: u32,
    pub union_id: u32,
    pub murderer_time: u32,
    pub pk_count: u16,
    pub kill_count: u32,
    pub hit_top_log: u16,
    pub hot_hit: u32,
    pub loan_max: u32,
    pub loan: u32,
    pub loan_time: i32,
    pub remain_point: u16,
    pub pk_normal: bool,
    pub pk_team: bool,
    pub pk_union: bool,
    pub pk_badman: bool,
    pub pk_country: bool,
    pub yp: u16,
    pub hp: u32,
    pub mp: u32,
    pub rp: u16,
    pub base_max_hp: u32,
    pub base_max_mp: u32,
    pub base_max_yp: u16,
    pub base_max_rp: u16,
    pub base_str: u32,
    pub base_dex: u32,
    pub base_con: u32,
    pub base_int: u32,
    pub base_min_atk: u32,
    pub base_max_atk: u32,
    pub base_hit: u16,
    pub base_burden: u16,
    pub base_cch: u16,
    pub base_def: u32,
    pub base_dodge: u16,
    pub base_atc_speed: u16,
    pub base_element_resistant: u32,
    pub base_hp_recover_speed: u16,
    pub base_mp_recover_speed: u16,
    pub base_vigour: u32,
    pub base_max_vigour: u32,
    pub base_energy: u32,
    pub base_max_energy: u32,
    pub base_credit: u32,
    pub display_head_piece: u8,
    pub country: u8,
    pub contribute: i32,
    pub is_charged: bool,
    pub quest_time_begin: i32,
    pub quest_time_limit: i32,
    pub quest: bool,
    pub depot_password: &'a [u8],
    pub exploit: u32,
    pub kudos: u32,
    pub mode: u32,
    pub fairy_enabled: bool,
    pub foster_num: u32,
    pub hatcher_num: u32,
    pub battle_fairy_enabled: bool,
    pub fetch_power: u32,
    pub max_fetch_power: u32,
    pub auction_space: u32,
    pub exalt: u32,
    pub szl: u32,
    pub gods_battle_faction: i32,
    pub base_fy_energy: u32,
    pub base_bl_fy_energy: u32,
    pub lt_up_60_count: u16,
    pub remain_jl_dan_count: u16,
    pub lt_60_stamp: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerAbilityBinaryField {
    HotKey,
    Skill,
    ScriptFlag,
    State,
    Friend,
    CiQing,
    Thing,
}

impl PlayerAbilityBinaryField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::HotKey => "HotKey",
            Self::Skill => "ListSkill",
            Self::ScriptFlag => "VariableList",
            Self::State => "ListState",
            Self::Friend => "ListFriendName",
            Self::CiQing => "ciqing",
            Self::Thing => "ListThing",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerAbilitySkill {
    pub id: u16,
    pub level: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerThing {
    pub tid: u16,
    pub count: u16,
    pub max_count: u16,
    pub point: u16,
}

pub struct PlayerScriptFlagSnapshot<'a> {
    pub variable_num: i32,
    pub variable_data: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerFriendName<'a>(pub &'a [u8]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddedFriendNameNul {
    pub offset: usize,
}

impl<'a> PlayerFriendName<'a> {
    pub fn from_legacy_bytes(bytes: &'a [u8]) -> Result<Self, EmbeddedFriendNameNul> {
        match bytes.iter().position(|byte| *byte == 0) {
            Some(offset) => Err(EmbeddedFriendNameNul { offset }),
            None => Ok(Self(bytes)),
        }
    }
}

/// Полный caller-owned view одной новой строки `CSL_PLAYER_ABILITY`.
///
/// Все части обязаны происходить из одного `CPlayer` snapshot; в частности,
/// `scalar.id == jjc.id`, как два чтения одного исходного объекта.
pub struct PlayerAbilityCreationSnapshot<'a> {
    pub scalar: PlayerAbilityScalarSnapshot<'a>,
    pub hot_keys: &'a [u32; 24],
    pub skills: &'a [PlayerAbilitySkill],
    pub script_flag: PlayerScriptFlagSnapshot<'a>,
    pub ex_states: &'a [u8],
    pub friend_names: &'a [PlayerFriendName<'a>],
    pub ci_qing_ids: &'a BTreeSet<u32>,
    pub things: &'a [PlayerThing],
    pub jjc: PlayerJjcDataSnapshot,
}

pub struct PlayerAbilitySaveSnapshot<'a> {
    pub ability: PlayerAbilityCreationSnapshot<'a>,
    pub silence_time: i32,
    pub days_honor_eliminate_num: u32,
    pub weeks_honor_eliminate_num: u32,
    pub months_honor_eliminate_num: u32,
    pub total_honor_eliminate_num: u32,
    pub rank_of_nobility_id: u32,
    pub appellation_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerQuestSaveEntry {
    pub quest_id: u16,
    pub complete: u8,
}

pub struct PlayerQuestSaveSnapshot<'a> {
    pub player_id: i32,
    pub quests: &'a BTreeMap<u16, PlayerQuestSaveEntry>,
}

/// Полный caller-owned view четырёх стадий `CRsPlayer::SavePlayer`.
///
/// Все вложенные ID обязаны происходить из одного исходного `CPlayer`:
/// `base.id == abilities.base.scalar.id == quest.player_id == goods.player_id`.
pub struct PlayerSaveSnapshot<'player, 'quest, 'goods_snapshot> {
    pub base: PlayerBaseSaveSnapshot<'player>,
    pub abilities: PlayerAbilitySaveSnapshot<'player>,
    pub quest: PlayerQuestSaveSnapshot<'quest>,
    pub goods: PlayerGoodsFiledSnapshot<'goods_snapshot>,
}

#[derive(Debug)]
pub enum PlayerSaveBlock {
    Goods(GoodsTraversalBlock),
}

#[derive(Debug)]
pub enum PlayerSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(PlayerSaveBlock),
}

#[derive(Debug)]
pub enum PlayerAbilityBlobDecodeBlock {
    HotKeySize {
        actual_bytes: usize,
    },
    FriendNameWithoutTerminator {
        offset: usize,
        available_bytes: usize,
    },
    ScriptPayloadTooLarge {
        actual_bytes: usize,
    },
}

impl fmt::Display for PlayerAbilityBlobDecodeBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HotKeySize { actual_bytes } => write!(
                formatter,
                "HotKey содержит {actual_bytes} байт вместо обязательных 96"
            ),
            Self::FriendNameWithoutTerminator {
                offset,
                available_bytes,
            } => write!(
                formatter,
                "ListFriendName с offset {offset} не содержит NUL в оставшихся {available_bytes} байтах"
            ),
            Self::ScriptPayloadTooLarge { actual_bytes } => write!(
                formatter,
                "VariableList payload содержит {actual_bytes} байт и не помещается в signed long"
            ),
        }
    }
}

impl Error for PlayerAbilityBlobDecodeBlock {}

#[derive(Debug)]
pub enum PlayerAbilityRowLoadFailure {
    Database {
        column: &'static str,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        column: &'static str,
        value: i64,
        target: &'static str,
    },
    Blob {
        field: PlayerAbilityBinaryField,
        source: PlayerAbilityBlobDecodeBlock,
    },
    HonorTime(TagTimeArithmeticBlock),
}

#[derive(Debug)]
pub enum PlayerAbilityQueryLoadFailure {
    ZeroPlayerId,
    MissingConnection,
    Database(tiberius::error::Error),
    MissingRow,
}

#[derive(Debug)]
pub enum PlayerAbilityQueryLoadOutcome {
    ReturnedTrue,
    ReturnedFalse(PlayerAbilityQueryLoadFailure),
    BlockedMalformed(PlayerAbilityRowLoadFailure),
}

#[derive(Debug)]
pub enum PlayerQuestQueryLoadFailure {
    ZeroPlayerId,
    MissingConnection,
    Database(tiberius::error::Error),
}

#[derive(Debug)]
pub enum PlayerQuestQueryLoadOutcome {
    ReturnedTrue { quest_count: usize },
    ReturnedFalse(PlayerQuestQueryLoadFailure),
}

#[derive(Debug)]
pub enum PlayerLoadFailure {
    Connection(WorldDatabaseConnectionError),
    Ability(PlayerAbilityQueryLoadFailure),
    Quest(PlayerQuestQueryLoadFailure),
    Goods(GoodsLoadFailure),
    Jjc(PlayerJjcLoadFailure),
}

#[derive(Debug)]
pub enum PlayerLoadBlock<AddonBlockT, InsertBlockT> {
    Ability(PlayerAbilityRowLoadFailure),
    Goods(GoodsLoadBlock<AddonBlockT, InsertBlockT>),
}

#[derive(Debug)]
pub enum PlayerLoadOutcome<AddonBlockT, InsertBlockT> {
    ReturnedTrue,
    ReturnedFalse(PlayerLoadFailure),
    BlockedMissingFact(PlayerLoadBlock<AddonBlockT, InsertBlockT>),
}
