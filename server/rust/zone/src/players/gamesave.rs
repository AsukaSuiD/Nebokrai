//! GameSave codec живого игрока Zone: единый World→Game блок
//! `CPlayer::DecordFromByteArray(..., true)` и обратный
//! `AddGameSaveToByteArray` над принадлежащими игроку частями Zone —
//! контейнеры `items/`, base/combat typed-снимки свойств, сценарные переменные
//! `scripts/`, прогресс заданий `quests/`, `MoveShapeState` `regions/moveshape`
//! и DB-кодек состояний `skills/state`.
//!
//! Hub `CPlayer` старого пакета (`gameserver/appserver/player.rs`) бережёт
//! делегаты прежних сигнатур и передаёт сюда проекции своих полей
//! [`PlayerGameSaveParts`]; сам агрегат сюда не переезжает (циклическая
//! зависимость). Проекции — суженный заёмный срез hub-а, владение остаётся у
//! него. Клиентский wire (`AddToByteArray_ForClient`) — соседний
//! `clientsnapshot.rs`; game.rs и message/* продолжают работать через
//! делегаты hub-а без изменения call-site-ов.
//!
//! Исходный owner: `server/gameserver/appserver/player.cpp/.h`, точная пара
//! GameServer/gameserver.exe + GameServer.pdb; layout GameSave VERIFIED
//! машинной разведкой форматов (fix D2/D4 применены; запись «машинная
//! разведка GameSave-форматов»). UNKNOWN: pre-effect `DelAllItemInDelList`
//! (`0x00440DDC` → `0x0043E4E0`) перед encode не воспроизводится — сущности
//! del-list в Rust нет.
//!
//! Узкие typed-швы до ухода кодов-владельцев из старого пакета:
//! realm-appellation bonus-предикат (каталог `skills/realmappellation`
//! старого пакета) и результат требований `CanMountEquip` (player-side
//! требования предметов hub-а); оба передаются generic-параметрами, без dyn.

use std::collections::{BTreeSet, VecDeque};

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};
use thiserror::Error;

use crate::combat::PlayerCombatProperties;
use crate::content::goodsfactory::CGoodsFactory;
use crate::items::camountlimitgoodscontainer::{
    AmountLimitGoodsCodecError, CAmountLimitGoodsContainer,
};
use crate::items::camountlimitgoodsshadowcontainer::CAmountLimitGoodsShadowContainer;
use crate::items::cbank::CBank;
use crate::items::cbattlefairycontainer::CBattleFairyContainer;
use crate::items::cdepot::CDepot;
use crate::items::cequipmentcontainer::{
    CEquipmentContainer, EquipmentAddRuntimeFacts, EquipmentContainerCodecError,
    EquipmentOwnerPlayerFacts, EquipmentUnserializedEntry,
};
use crate::items::cfairycontainer::{CFairyContainer, FairyContainerCodecError};
use crate::items::cgoods::CGoods;
use crate::items::cjifen::CJiFen;
use crate::items::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeGoodsCodecError};
use crate::items::cwallet::{CWallet, CurrencyCodecError};
use crate::items::cyuanbao::CYuanBao;
use crate::quests::{PlayerQuestAvailability, PlayerQuestProgress};
use crate::regions::moveshape::MoveShapeState;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::ShapeDecodeError;
use crate::regions::skillregistry::SKILL_BASE_DEFENSE;
use crate::scripts::{CVariableList, GameVariableSnapshotError};
use crate::skills::execution::MoveShapeSkill;
use crate::skills::skillfactory::{CSkillFactory, SkillCategory};

pub const PLAYER_BASE_PROPERTY_WIRE_SIZE: usize = 0x194;
pub const BASE_LEVEL_OFFSET: usize = 0x04;
pub const BASE_EXPERIENCE_OFFSET: usize = 0x08;
pub const BASE_HEAD_PICTURE_OFFSET: usize = 0x0c;
pub const BASE_FACE_PICTURE_OFFSET: usize = 0x0d;
pub const BASE_OCCUPATION_OFFSET: usize = 0x0e;
pub const BASE_SEX_OFFSET: usize = 0x0f;
pub const BASE_PK_COUNT_OFFSET: usize = 0x1c;
pub const BASE_KILL_COUNT_OFFSET: usize = 0x20;
pub const BASE_HIT_TOP_LOG_OFFSET: usize = 0x24;
pub const BASE_CHARGED_OFFSET: usize = 0x38;
pub const BASE_REMAIN_POINT_OFFSET: usize = 0x3a;
pub const BASE_HOTKEY_OFFSET: usize = 0x3c;
pub const BASE_PK_NORMAL_OFFSET: usize = 0x9c;
pub const BASE_PK_TEAM_OFFSET: usize = 0x9d;
pub const BASE_PK_UNION_OFFSET: usize = 0x9e;
pub const BASE_PK_BADMAN_OFFSET: usize = 0x9f;
pub const BASE_PK_COUNTRY_OFFSET: usize = 0xa0;
pub const BASE_HEALTH_OFFSET: usize = 0xa4;
pub const BASE_MANA_OFFSET: usize = 0xa8;
pub const BASE_RP_OFFSET: usize = 0xac;
pub const BASE_YP_OFFSET: usize = 0xae;
pub const BASE_MAXIMUM_HP_OFFSET: usize = 0xb0;
pub const BASE_MAXIMUM_MP_OFFSET: usize = 0xb4;
pub const BASE_MAXIMUM_YP_OFFSET: usize = 0xb8;
pub const BASE_MAXIMUM_RP_OFFSET: usize = 0xba;
pub const BASE_STRENGTH_OFFSET: usize = 0xbc;
pub const BASE_DEXTERITY_OFFSET: usize = 0xc0;
pub const BASE_CONSTITUTION_OFFSET: usize = 0xc4;
pub const BASE_INTELLIGENCE_OFFSET: usize = 0xc8;
pub const BASE_MINIMUM_ATTACK_OFFSET: usize = 0xcc;
pub const BASE_MAXIMUM_ATTACK_OFFSET: usize = 0xd0;
pub const BASE_HIT_OFFSET: usize = 0xd4;
pub const BASE_BURDEN_OFFSET: usize = 0xd6;
pub const BASE_CCH_OFFSET: usize = 0xd8;
pub const BASE_DEFENSE_OFFSET: usize = 0xdc;
pub const BASE_DODGE_OFFSET: usize = 0xe0;
pub const BASE_ATTACK_SPEED_OFFSET: usize = 0xe2;
pub const BASE_ELEMENT_RESISTANCE_OFFSET: usize = 0xe4;
pub const BASE_HP_RECOVERY_OFFSET: usize = 0xe8;
pub const BASE_MP_RECOVERY_OFFSET: usize = 0xea;
pub const BASE_VIGOUR_OFFSET: usize = 0xec;
pub const BASE_MAXIMUM_VIGOUR_OFFSET: usize = 0xf0;
pub const BASE_ENERGY_OFFSET: usize = 0xf4;
pub const BASE_MAXIMUM_ENERGY_OFFSET: usize = 0xf8;
pub const BASE_CREDIT_OFFSET: usize = 0xfc;
pub const BASE_EXALT_OFFSET: usize = 0x100;
pub const BASE_DISPLAY_HEAD_PIECE_OFFSET: usize = 0x104;
pub const BASE_QUEST_TIME_BEGIN_OFFSET: usize = 0x108;
pub const BASE_QUEST_TIME_LIMIT_OFFSET: usize = 0x10c;
pub const BASE_QUEST_ENABLED_OFFSET: usize = 0x110;
pub const BASE_EXPLOIT_OFFSET: usize = 0x114;
pub const BASE_FAIRY_CONTAINER_ENABLED_OFFSET: usize = 0x11c;
pub const BASE_BATTLE_FAIRY_ENABLED_OFFSET: usize = 0x128;
pub const BASE_BREAK_ARMOUR_OFFSET: usize = 0x12c;
pub const BASE_PUNCTURE_OFFSET: usize = 0x130;
pub const BASE_BREAK_ELEMENT_OFFSET: usize = 0x134;
pub const BASE_BREAK_BOUND_OFFSET: usize = 0x138;
pub const BASE_POWER_OF_GOLD_OFFSET: usize = 0x13c;
pub const BASE_DAYS_HONOR_OFFSET: usize = 0x140;
pub const BASE_WEEKS_HONOR_OFFSET: usize = 0x144;
pub const BASE_MONTHS_HONOR_OFFSET: usize = 0x148;
pub const BASE_TOTAL_HONOR_OFFSET: usize = 0x14c;
pub const BASE_RANK_OF_NOBILITY_OFFSET: usize = 0x150;
pub const BASE_APPELLATION_OFFSET: usize = 0x154;
pub const BASE_MODE_OFFSET: usize = 0x158;
pub const BASE_FETCH_POWER_OFFSET: usize = 0x164;
pub const BASE_BATTLE_FAIRY_SUMMONED_OFFSET: usize = 0x16c;
pub const BASE_BATTLE_FAIRY_RECALL_OFFSET: usize = 0x16d;
pub const BASE_BATTLE_FAIRY_DIED_OFFSET: usize = 0x16e;
pub const BASE_AUCTION_SPACE_OFFSET: usize = 0x170;
pub const BASE_JJC_LEVEL_OFFSET: usize = 0x174;
pub const BASE_JJC_SCORE_OFFSET: usize = 0x178;
pub const BASE_FY_ENERGY_OFFSET: usize = 0x17c;
pub const BASE_FY_ENABLE_FLAGS_OFFSET: usize = 0x180;
pub const BASE_LT_UP_60_COUNT_OFFSET: usize = 0x184;
pub const BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET: usize = 0x186;
pub const BASE_LT_60_STAMP_OFFSET: usize = 0x188;
pub const BASE_SZL_OFFSET: usize = 0x18c;
pub const BASE_GODS_BATTLE_FACTION_OFFSET: usize = 0x190;
pub const PLAYER_COMBAT_PROPERTY_WIRE_SIZE: usize = 0x9c;

/// Подтверждённые LeiTing/FY-флаги в `u32` legacy-формата игрока.
///
/// Неизвестные биты сохраняются через `from_bits_retain` и возвращаются в
/// сетевой/DB формат без усечения; известные `0..=8` соответствуют порогам
/// энергии и числу суточных подъёмов выше 60.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LeiTingEnableFlags(u32);

impl LeiTingEnableFlags {
    pub const ENERGY_20: Self = Self(1 << 0);
    pub const ENERGY_60: Self = Self(1 << 1);
    pub const ENERGY_80: Self = Self(1 << 2);
    pub const ENERGY_100: Self = Self(1 << 3);
    pub const LT_UP_60_4: Self = Self(1 << 4);
    pub const LT_UP_60_10: Self = Self(1 << 5);
    pub const LT_UP_60_16: Self = Self(1 << 6);
    pub const LT_UP_60_22: Self = Self(1 << 7);
    pub const LT_UP_60_28: Self = Self(1 << 8);

    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

/// Typed-снимок базовых свойств игрока из `m_BaseProperty[0x194]`.
/// Хранилище принадлежит hub CPlayer; этот тип — владелец persisted-layout
/// отражения. Прямые записи полей выполняют gameplay-сеттеры hub-а.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerBaseProperties {
    pub level: u8,
    pub occupation: u8,
    pub sex: u8,
    pub remain_point: u16,
    pub base_maximum_hp: u32,
    pub base_maximum_mp: u32,
    pub base_burden: u16,
    pub base_strength: u32,
    pub base_dexterity: u32,
    pub base_constitution: u32,
    pub base_intelligence: u32,
    pub pk_normal: bool,
    pub pk_team: bool,
    pub pk_union: bool,
    pub pk_badman: bool,
    pub pk_country: bool,
    pub pk_count: u16,
    pub kill_count: u32,
    pub hit_top_log: u16,
    pub experience: u32,
    pub vigour: u32,
    pub credit: u32,
    pub charged: bool,
    pub fairy_container_enabled: bool,
    pub battle_fairy_enabled: bool,
    pub break_armour: u32,
    pub puncture: u32,
    pub break_element: u32,
    pub break_bound: u32,
    pub power_of_gold: u32,
    pub hotkeys: [u32; 24],
    pub mode: u32,
    pub display_head_piece: bool,
    pub quest_availability: PlayerQuestAvailability,
    pub fy_enable_flags: LeiTingEnableFlags,
    pub fy_energy: u32,
    pub lt_60_stamp: u32,
    pub lt_up_60_count: u16,
    pub remain_jing_li_dan_count: u16,
    pub appellation_id: u32,
    pub head_picture: i32,
    pub face_picture: i32,
    pub health: u32,
    pub mana: u32,
    pub rp: u16,
    pub yp: u16,
    pub maximum_yp: u16,
    pub maximum_rp: u16,
    pub maximum_vigour: u32,
    pub energy: u32,
    pub maximum_energy: u32,
    pub exalt: u32,
    pub fetch_power: u32,
    pub battle_fairy_recall: bool,
    pub battle_fairy_died: bool,
    pub auction_space: u32,
    pub jjc_level: u32,
    pub jjc_score: u32,
    pub days_honor_eliminate: u32,
    pub weeks_honor_eliminate: u32,
    pub months_honor_eliminate: u32,
    pub total_honor_eliminate: u32,
    pub rank_of_nobility_id: u32,
    pub exploit: u32,
    pub gods_battle_faction: i32,
    pub szl: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerFriend {
    pub name: Vec<u8>,
    pub online: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerLeiTingThing {
    pub thing_id: u16,
    pub count: u16,
    pub max_count: u16,
    pub point: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerUncreatedPet {
    pub original_name: Vec<u8>,
    pub health: u32,
    pub level: u32,
    pub experience: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlayerUncreatedCarriage {
    pub original_name: Vec<u8>,
    pub script: Vec<u8>,
    pub health: u32,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("LeiTing snapshot обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
pub struct PlayerLeiTingDecodeBlock {
    pub field: &'static str,
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PlayerGameSaveCodecError {
    #[error(transparent)]
    Shape(#[from] ShapeDecodeError),
    #[error(transparent)]
    Goods(#[from] AmountLimitGoodsCodecError),
    #[error(transparent)]
    Volume(#[from] VolumeGoodsCodecError),
    #[error(transparent)]
    Equipment(EquipmentContainerCodecError),
    #[error(transparent)]
    Fairy(#[from] FairyContainerCodecError),
    #[error(transparent)]
    Currency(#[from] CurrencyCodecError),
    #[error(transparent)]
    Variables(#[from] GameVariableSnapshotError),
    #[error(transparent)]
    LeiTing(#[from] PlayerLeiTingDecodeBlock),
    #[error("player save обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("player save содержит отрицательное count {count} в {field}")]
    NegativeCount { field: &'static str, count: i32 },
    #[error("player save string {field} имеет длину {length} при максимуме {maximum}")]
    StringTooLong {
        field: &'static str,
        length: usize,
        maximum: usize,
    },
    #[error("player save collection {field} длиной {length} не представима")]
    CollectionTooLarge { field: &'static str, length: usize },
    #[error("player save содержит object type {object_type} вместо player")]
    WrongObjectType { object_type: i32 },
    #[error("player save отклонил equipment position {position}")]
    EquipmentRejected { position: u32 },
}

/// Организационный блок persisted-снимка игрока (`0x7FE06` wire-семья):
/// изменяемый срез для decode.
pub struct PlayerOrganizingParts<'a> {
    pub faction_id: &'a mut i32,
    pub faction_logo_id: &'a mut i32,
    pub faction_level: &'a mut u16,
    pub faction_experience: &'a mut i32,
    pub faction_force: &'a mut i32,
    pub faction_contribute: &'a mut u32,
    pub faction_master_id: &'a mut i32,
    pub faction_name: &'a mut Vec<u8>,
    pub faction_title: &'a mut Vec<u8>,
    pub union_id: &'a mut i32,
    pub union_master_id: &'a mut i32,
    pub enemy_factions: &'a mut BTreeSet<i32>,
    pub city_war_enemy_factions: &'a mut BTreeSet<i32>,
    pub faction_owned_regions: &'a mut Vec<[u8; 8]>,
}

/// Тот же организационный блок по чтению для encode и клиентских снимков.
#[derive(Clone, Copy)]
pub struct PlayerOrganizingSnapshot<'a> {
    pub faction_id: i32,
    pub faction_logo_id: i32,
    pub faction_level: u16,
    pub faction_experience: i32,
    pub faction_force: i32,
    pub faction_contribute: u32,
    pub faction_master_id: i32,
    pub faction_name: &'a [u8],
    pub faction_title: &'a [u8],
    pub union_id: i32,
    pub union_master_id: i32,
    pub enemy_factions: &'a BTreeSet<i32>,
    pub city_war_enemy_factions: &'a BTreeSet<i32>,
    pub faction_owned_regions: &'a [[u8; 8]],
}

/// Persistence-проекция hub `CPlayer`: заёмный срез всех codec-владеемых
/// полей. Владение остаётся у hub-а; проекция живёт только на время вызова
/// codec-а. Порядок полей отражает persisted layout, сами значения — живые
/// части Zone.
pub struct PlayerGameSaveParts<'a> {
    /// Фигура игрока: shape codec, реестр навыков и арена состояний.
    pub move_shape: &'a mut MoveShapeState,
    pub base_property_wire: &'a mut [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    pub base_properties: &'a mut PlayerBaseProperties,
    pub combat_property_wire: &'a mut [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    pub combat_properties: &'a mut PlayerCombatProperties,
    pub battle_fairy_summoned: &'a mut bool,
    pub war_soul_state: &'a mut u32,
    pub realm_appellation_skill_id: &'a mut u32,
    pub realm_appellation_skill_level: &'a mut i32,
    pub auto_protected: &'a mut bool,
    pub account: &'a mut Vec<u8>,
    pub title: &'a mut Vec<u8>,
    pub session_id: &'a mut Vec<u8>,
    pub team_id: &'a mut i32,
    pub country: &'a mut u8,
    pub contribution: &'a mut i32,
    pub money: &'a mut u32,
    pub depot_password: &'a mut Vec<u8>,
    pub ci_qing_list: &'a mut BTreeSet<u32>,
    pub friends: &'a mut Vec<PlayerFriend>,
    pub lei_ting_things: &'a mut VecDeque<PlayerLeiTingThing>,
    pub quest_progress: &'a mut PlayerQuestProgress,
    pub uncreated_pets: &'a mut Vec<PlayerUncreatedPet>,
    pub uncreated_carriage: &'a mut PlayerUncreatedCarriage,
    pub recreate_carriage: &'a mut bool,
    pub login: &'a mut bool,
    pub variable_list: &'a mut CVariableList,
    pub silence_minutes: &'a mut i32,
    pub silence_timestamp_minutes: &'a mut u32,
    pub murderer_time_stamp_ms: &'a mut u32,
    pub fight_state_count: &'a mut i32,
    pub city_war_died_state: &'a mut bool,
    pub city_war_died_state_time_ms: &'a mut i32,
    pub died_state_start_time_ms: &'a mut u32,
    pub jjc_data: &'a mut [u8; 0x10],
    pub jjc_pk_state: &'a mut bool,
    pub organizing: PlayerOrganizingParts<'a>,
    pub hand: &'a mut CAmountLimitGoodsContainer,
    pub enhancement: &'a mut CAmountLimitGoodsShadowContainer,
    pub packet: &'a mut CVolumeLimitGoodsContainer,
    pub equipment: &'a mut CEquipmentContainer,
    pub wallet: &'a mut CWallet,
    pub yuan_bao: &'a mut CYuanBao,
    pub ji_fen: &'a mut CJiFen,
    pub bank: &'a mut CBank,
    pub depot: &'a mut CDepot,
    pub auction_goods: &'a mut CVolumeLimitGoodsContainer,
    pub auction_listing: &'a mut CVolumeLimitGoodsContainer,
    pub auction_wallet: &'a mut CWallet,
    pub fairy_container: &'a mut CFairyContainer,
    pub battle_fairy_container: &'a mut CBattleFairyContainer,
    pub ci_qing: &'a mut CVolumeLimitGoodsContainer,
    pub ci_qing_compose: &'a mut CVolumeLimitGoodsContainer,
}

/// Исходная разметка persistence-контейнеров живого игрока
/// (`from_send_state` хвост): объёмы, ширина/высота ячеек, лимит руки и
/// extend-id enhancement. Конструкция самих контейнеров остаётся hub-у.
pub fn init_player_persistence_containers(parts: &mut PlayerGameSaveParts<'_>) {
    let _empty_release = parts.packet.set_container_dimensions(8, 12);
    parts.enhancement.set_goods_amount_limit(1);
    parts.enhancement.base_mut().set_container_extend_id(10);
    let _empty_release = parts.ci_qing.set_container_volume(8);
    let _empty_release = parts.ci_qing_compose.set_container_volume(3);
    let _empty_release = parts.fairy_container.base_mut().set_container_volume(14);
    let _empty_release = parts.auction_goods.set_container_volume(0x12);
    let _empty_release = parts.auction_listing.set_container_volume(2);
    let _empty_release = parts.depot.base_mut().set_container_dimensions(8, 12);
}

/// Достигнутая часть exact `RefreshContainerOwners`: owner ID должен быть
/// перепривязан после создания player identity или его восстановления.
pub const fn refresh_player_container_owners(
    parts: &mut PlayerGameSaveParts<'_>,
    player_id: i32,
) {
    parts.bank.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts
        .depot
        .base_mut()
        .base_mut()
        .base_mut()
        .set_owner(PLAYER_TYPE, player_id);
    parts.hand.set_owner(PLAYER_TYPE, player_id);
    parts
        .enhancement
        .base_mut()
        .base_mut()
        .set_owner(PLAYER_TYPE, player_id);
    parts.packet.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts.wallet.set_owner(PLAYER_TYPE, player_id);
    parts.yuan_bao.set_owner(PLAYER_TYPE, player_id);
    parts.ji_fen.set_owner(PLAYER_TYPE, player_id);
    parts.equipment.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts.auction_listing.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts.auction_goods.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts.auction_wallet.set_owner(PLAYER_TYPE, player_id);
    parts.ci_qing.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts.ci_qing_compose.base_mut().set_owner(PLAYER_TYPE, player_id);
    parts
        .fairy_container
        .base_mut()
        .base_mut()
        .set_owner(PLAYER_TYPE, player_id);
    parts
        .battle_fairy_container
        .base_mut()
        .base_mut()
        .set_owner(PLAYER_TYPE, player_id);
}

/// Exact World→Game player handoff, общая часть после shape-блока. Decoder
/// восстанавливает единый `CPlayer::DecordFromByteArray(..., true)` блок, а не
/// отдельные игровые фрагменты. Native GoodsAI tail намеренно отложен до
/// успешной регистрации player-а в map/region и исполняется
/// `CGame::complete_world_player_login` для достигнутых equipment и packet
/// owners. Сам shape codec и проверка object type выполняет делегат hub-а до
/// этого вызова — проекция уже указывает на сконструированную фигуру.
#[allow(
    clippy::too_many_arguments,
    reason = "literal handoff сохраняет достигнутые runtime-швы hub-а и пороги контейнеров"
)]
pub fn decode_player_game_save<OrdinaryThreshold, BattleThreshold, StateNow, RealmBonus, CanMount>(
    source: &[u8],
    cursor: &mut usize,
    parts: &mut PlayerGameSaveParts<'_>,
    goods_factory: &CGoodsFactory,
    skill_factory: &CSkillFactory,
    variable_definitions: Option<&[u8]>,
    now_ms: u32,
    one_pk_count_time_ms: u32,
    mut state_now: StateNow,
    pack_add_enabled: bool,
    mut ordinary_threshold: OrdinaryThreshold,
    mut battle_threshold: BattleThreshold,
    is_realm_bonus_skill: RealmBonus,
    can_mount_equip: CanMount,
) -> Result<(), PlayerGameSaveCodecError>
where
    OrdinaryThreshold: FnMut(u32, u32) -> u32,
    BattleThreshold: FnMut(u32, u32) -> u32,
    StateNow: FnMut() -> u32,
    RealmBonus: Fn(u32) -> bool,
    CanMount: Fn(&CGoods, PlayerBaseProperties, PlayerCombatProperties) -> i32,
{
    *parts.base_property_wire = read_player_game_save_array(source, cursor, "m_BaseProperty[0x194]")?;
    let (base_properties, battle_fairy_summoned) = apply_base_property_wire(parts.base_property_wire);
    *parts.base_properties = base_properties;
    // После чтения base-wire `bBFSummon` намеренно снова выводится из
    // локального `m_dwWarSoulState`, а не принимается как независимый
    // persisted fact.
    *parts.battle_fairy_summoned = battle_fairy_summoned;
    *parts.battle_fairy_summoned = *parts.war_soul_state != 0;
    *parts.account = read_player_game_save_string(source, cursor, "strAccount", 0x100)?;
    *parts.title = read_player_game_save_string(source, cursor, "strTitle", 0x100)?;
    *parts.combat_property_wire = read_player_game_save_array(source, cursor, "m_Property[0x9c]")?;
    *parts.combat_properties = apply_combat_property_wire(parts.combat_property_wire);
    *parts.team_id = read_player_game_save_i32(source, cursor, "m_lTeamID")?;

    parts.ci_qing_list.clear();
    let ci_qing_count = read_player_game_save_count(source, cursor, "m_setCiQingList")?;
    for _ in 0..ci_qing_count {
        parts.ci_qing_list.insert(read_player_game_save_u32(
            source,
            cursor,
            "m_setCiQingList entry",
        )?);
    }

    crate::skills::state::clear_persisted_runtime_state(
        &mut parts.move_shape.skills,
        &mut parts.move_shape.state_storage.ex_states,
        &mut parts.move_shape.state_storage.state_entries,
        &mut parts.move_shape.can_fight_count,
        &mut parts.move_shape.can_fight,
    );
    *parts.auto_protected = false;
    let skill_count = read_player_game_save_i32(source, cursor, "skill count")?;
    for _ in 0..skill_count.max(0) {
        let packed = read_player_game_save_u32(source, cursor, "tagSkillID")?;
        let skill_id = packed & 0xffff;
        let level = (packed >> 16) as i32;
        let loaded = parts.move_shape.skills.add_skill(skill_id, level, skill_factory, |owner| {
            MoveShapeSkill::registered(skill_id, level, owner)
        });
        if loaded && is_realm_bonus_skill(skill_id) && (1..=4).contains(&level) {
            *parts.realm_appellation_skill_id = skill_id;
            *parts.realm_appellation_skill_level = level;
        }
    }
    let ex_state_length = read_player_game_save_count(source, cursor, "m_vExStates length")?;
    let ex_states = read_player_game_save_slice(source, cursor, "m_vExStates", ex_state_length)?.to_vec();
    crate::skills::state::replace_ex_states(
        &mut parts.move_shape.state_storage.ex_states,
        &mut parts.move_shape.state_storage.state_entries,
        parts.move_shape.shape.identity(),
        ex_states,
        skill_factory,
        &mut state_now,
    );

    parts.friends.clear();
    let friend_count = read_player_game_save_i32(source, cursor, "m_listFriend")?;
    for _ in 0..friend_count.max(0) {
        parts.friends.push(PlayerFriend {
            name: read_player_game_save_string(source, cursor, "tagFriend.strName", 0x94)?,
            online: read_player_game_save_u8(source, cursor, "tagFriend.bOnline")? != 0,
        });
    }
    decode_player_lei_ting(parts.base_properties, parts.lei_ting_things, source, cursor)?;

    let _cleared_hand = parts.hand.clear_goods();
    parts.hand.set_goods_amount_limit(1);
    parts.hand.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;

    let base_properties = *parts.base_properties;
    let combat_properties = *parts.combat_properties;
    let equipment = parts.equipment.unserialize_with(
        source,
        cursor,
        goods_factory,
        true,
        |source, cursor| {
            let mut goods = CGoods::default();
            goods.unserialize(
                source,
                cursor,
                true,
                goods_factory,
                &mut ordinary_threshold,
                &mut battle_threshold,
            )?;
            Ok(Some(goods))
        },
        |goods| EquipmentAddRuntimeFacts {
            owner_player: Some(EquipmentOwnerPlayerFacts {
                can_mount_result: can_mount_equip(goods, base_properties, combat_properties),
            }),
            pack_add_enabled: false,
            now: u64::from(now_ms),
        },
        &mut |_| {},
        &mut |_, _, _| {},
    );
    let equipment =
        equipment.map_err(|failure| PlayerGameSaveCodecError::Equipment(failure.error))?;
    if let Some(position) = equipment.entries.iter().find_map(|entry| match entry {
        EquipmentUnserializedEntry::Rejected { position, .. }
        | EquipmentUnserializedEntry::DecoderReturnedNull { position } => Some(*position),
        EquipmentUnserializedEntry::Added { .. } => None,
    }) {
        return Err(PlayerGameSaveCodecError::EquipmentRejected { position });
    }

    let _released = parts.packet.set_container_dimensions(8, 12);
    parts.packet.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    parts
        .packet
        .apply_player_expansion_limit(parts.equipment.expanded_package_num());

    let _released = parts.auction_goods.set_container_volume(0x12);
    parts.auction_goods.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    // D1 машинной разведки: оригинал выполняет auction tail контейнера
    // (семья CleanCell/HaveCell, виртуалы `+0x80`/`+0x84` в
    // `0x0044C3AF/0x0044C3C6`) только после всего decoder-а; побайтовая
    // эквивалентность этого Rust-пути зависит от Unserialize-маркировки
    // самого container и отложена в container-порцию (запись «Zone player:
    // машинная разведка GameSave» в docs/status/audit.md).
    parts.auction_goods.set_all_inactive();
    if pack_add_enabled {
        let inactive = parts
            .auction_goods
            .size()
            .wrapping_sub(parts.base_properties.auction_space);
        parts.auction_goods.apply_player_expansion_limit(inactive);
    }
    let _released = parts.auction_listing.set_container_volume(2);
    parts.auction_listing.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    parts.wallet.unserialize(
        source,
        cursor,
        "m_cWallet marker",
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    parts.auction_wallet.unserialize(
        source,
        cursor,
        "m_cAuctionWallet marker",
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    parts.yuan_bao.unserialize(
        source,
        cursor,
        "m_cYuanBao marker",
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    parts.ji_fen.unserialize(
        source,
        cursor,
        "m_cJiFen marker",
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    *parts.money = parts.wallet.currency_amount();

    *parts.depot_password =
        read_player_game_save_string(source, cursor, "m_strDepotPassword", 0x6c)?;
    parts.bank.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    parts.depot.unserialize(
        source,
        cursor,
        goods_factory,
        pack_add_enabled,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    let _released = parts.fairy_container.base_mut().set_container_volume(0x0e);
    parts.fairy_container.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    let _released = parts
        .battle_fairy_container
        .base_mut()
        .set_container_volume(0x11);
    parts.battle_fairy_container.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    let _released = parts.ci_qing.set_container_volume(8);
    parts.ci_qing.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;
    let _released = parts.ci_qing_compose.set_container_volume(3);
    parts.ci_qing_compose.unserialize(
        source,
        cursor,
        goods_factory,
        &mut ordinary_threshold,
        &mut battle_threshold,
    )?;

    parts
        .variable_list
        .decode_world_snapshot(variable_definitions, source, cursor)?;
    // Машина восстанавливает всю silence-пару (`0x0044C0A7..0x0044C0E1` по
    // той же точной паре gameserver.exe + GameServer.pdb): wire>0 пишет
    // minutes и начало `timeGetTime()/60_000` (магический делитель
    // `0x45E7B273`, shr `0xE`), wire≤0 обнуляет оба поля. Контракт совпадает
    // с exact `SetSilence`; текущий tick уже передан как `now_ms`.
    let silence_minutes = read_player_game_save_i32(source, cursor, "m_lSilenceTime")?;
    if silence_minutes > 0 {
        *parts.silence_minutes = silence_minutes;
        *parts.silence_timestamp_minutes = now_ms / 60_000;
    } else {
        *parts.silence_minutes = 0;
        *parts.silence_timestamp_minutes = 0;
    }
    let murderer_state = read_player_game_save_u8(source, cursor, "murderer state")? != 0;
    let murderer_remain = read_player_game_save_u32(source, cursor, "murderer remain time")?;
    restore_player_murderer_timestamp(
        parts.base_properties.pk_count,
        parts.murderer_time_stamp_ms,
        murderer_state,
        murderer_remain,
        now_ms,
        one_pk_count_time_ms,
    );
    *parts.fight_state_count = read_player_game_save_i32(source, cursor, "m_lFightStateCount")?;

    parts.uncreated_pets.clear();
    let pet_count = read_player_game_save_count(source, cursor, "m_vUncreatedPets")?;
    for _ in 0..pet_count {
        parts.uncreated_pets.push(PlayerUncreatedPet {
            original_name: read_player_game_save_string(
                source,
                cursor,
                "tagPetInformation.strOriginalName",
                0x94,
            )?,
            health: read_player_game_save_u32(source, cursor, "tagPetInformation.dwHp")?,
            level: read_player_game_save_u32(source, cursor, "tagPetInformation.dwLevel")?,
            experience: read_player_game_save_u32(
                source,
                cursor,
                "tagPetInformation.dwExperience",
            )?,
        });
    }
    *parts.uncreated_carriage = PlayerUncreatedCarriage {
        original_name: read_player_game_save_string(
            source,
            cursor,
            "tagCarriageInfo.strOriginalName",
            0x94,
        )?,
        script: read_player_game_save_string(
            source,
            cursor,
            "tagCarriageInfo.strCarriageScript",
            0x94,
        )?,
        health: read_player_game_save_u32(source, cursor, "tagCarriageInfo.dwHp")?,
    };
    *parts.recreate_carriage =
        read_player_game_save_u8(source, cursor, "m_bReCreateCarriage")? != 0;
    *parts.login = read_player_game_save_u8(source, cursor, "m_bLogin")? != 0;
    *parts.city_war_died_state_time_ms =
        read_player_game_save_i32(source, cursor, "m_lCityWarDiedStateTime")?;
    *parts.died_state_start_time_ms =
        u32::from(*parts.city_war_died_state_time_ms > 0).wrapping_mul(now_ms);
    *parts.city_war_died_state =
        *parts.city_war_died_state_time_ms > 0 && parts.base_properties.occupation != 6;

    parts.quest_progress.clear();
    let quest_count = read_player_game_save_i32(source, cursor, "m_PlayerQuests")?;
    for _ in 0..quest_count.max(0) {
        let quest_id = read_player_game_save_u16(source, cursor, "tagPlayerQuest.wQuestID")?;
        let state = read_player_game_save_u8(source, cursor, "tagPlayerQuest.byComplete")?;
        parts.quest_progress.insert_snapshot(quest_id, state);
    }
    *parts.country = read_player_game_save_u8(source, cursor, "m_btCountry")?;
    *parts.contribution = read_player_game_save_i32(source, cursor, "m_lContribute")?;
    *parts.jjc_data = read_player_game_save_array(source, cursor, "m_jjcdata[0x10]")?;
    *parts.jjc_pk_state = read_player_game_save_u8(source, cursor, "bJJcPkState")? != 0;
    decode_player_organizing_snapshot(source, cursor, &mut parts.organizing)?;
    *parts.session_id = read_player_game_save_string(source, cursor, "m_strSessionID", 0x40)?;
    // D8 машинной разведки: SetOwner-цикл контейнеров и UpdateProperty
    // оригинал выполняет в других точках того же прохода (UpdateProperty —
    // виртуал `+0x9C` в самом хвосте decoder-а, `0x0044C3D0`); Rust
    // группирует owner-refresh здесь, а тот же UpdateProperty исполняет
    // `CGame::complete_world_player_login` — итоговое состояние совпадает.
    let player_id = parts.move_shape.shape.identity().id;
    refresh_player_container_owners(parts, player_id);
    Ok(())
}

/// `AddGameSaveToByteArray`: тот же persisted layout без organization
/// snapshot (World обновляет его самостоятельно перед следующим handoff).
/// Результаты `CShape::AddToByteArray`, всех 15 container serialize и
/// `CVariableList::AddToByteArray` машина не тестирует и всегда возвращает
/// 1 (`0x00441399`): false-подрезультаты цепочки отбрасываются, ошибки
/// ограничены безопасными границами Rust. Отдельный первый шаг оригинала
/// `DelAllItemInDelList` (`0x00440DDC` → `0x0043E4E0`) не воспроизводится —
/// сущности del-list в Rust нет (UNKNOWN).
pub fn encode_player_game_save(
    destination: &mut Vec<u8>,
    parts: &mut PlayerGameSaveParts<'_>,
    goods_factory: &CGoodsFactory,
    now_ms: u32,
    timed_state_now_milliseconds: impl FnMut() -> u32,
    one_pk_count_time_ms: u32,
    pets: &[PlayerUncreatedPet],
    carriage: &PlayerUncreatedCarriage,
    recreate_carriage: bool,
) -> Result<(), PlayerGameSaveCodecError> {
    // Оригинал не тестирует false-подрезультаты этой serialize-цепочки и
    // всегда возвращает 1 (`0x00441399`): все результаты ниже отбрасываются
    // именованными discard-ами, как того требует машинный факт.
    let _shape_serialized = parts.move_shape.shape.add_to_byte_array(destination, true);
    destination.extend_from_slice(&synchronized_base_property_wire(
        parts.base_property_wire,
        parts.base_properties,
        *parts.battle_fairy_summoned,
    ));
    append_player_game_save_string(destination, "strAccount", parts.account, 0x100)?;
    append_player_game_save_string(destination, "strTitle", parts.title, 0x100)?;
    destination.extend_from_slice(parts.combat_property_wire);
    LegacyWriter::new(destination).write_i32(*parts.team_id);

    append_player_game_save_count(destination, "m_setCiQingList", parts.ci_qing_list.len())?;
    for base_index in parts.ci_qing_list.iter() {
        LegacyWriter::new(destination).write_u32(*base_index);
    }
    let skills: Vec<_> = serializable_player_skills(parts.move_shape).collect();
    append_player_game_save_count(destination, "skill count", skills.len())?;
    for skill in skills {
        let packed = (skill.id() & 0xffff) | ((skill.level() as u32 & 0xffff) << 16);
        LegacyWriter::new(destination).write_u32(packed);
    }
    let ex_states = crate::skills::state::serialize_ex_states_for_save(
        &mut parts.move_shape.state_storage.ex_states,
        &mut parts.move_shape.state_storage.state_entries,
        now_ms,
        timed_state_now_milliseconds,
    );
    append_player_game_save_count(destination, "m_vExStates length", ex_states.len())?;
    destination.extend_from_slice(&ex_states);
    append_player_game_save_count(destination, "m_listFriend", parts.friends.len())?;
    for friend in parts.friends.iter() {
        append_player_game_save_string(destination, "tagFriend.strName", &friend.name, 0x94)?;
        destination.push(u8::from(friend.online));
    }
    destination.extend_from_slice(&encode_player_lei_ting(parts.base_properties, parts.lei_ting_things));

    let _hand_serialized = parts.hand.serialize(destination, goods_factory);
    parts.equipment.serialize_with(
        destination,
        goods_factory,
        true,
        |goods, include_child, destination| {
            let _goods_serialized = goods.serialize(destination, include_child);
        },
    );
    let _packet_serialized = parts.packet.serialize(destination, goods_factory);
    let _auction_goods_serialized = parts.auction_goods.serialize(destination, goods_factory);
    let _auction_listing_serialized = parts.auction_listing.serialize(destination, goods_factory);
    let _wallet_serialized = parts.wallet.serialize(destination);
    let _auction_wallet_serialized = parts.auction_wallet.serialize(destination);
    let _yuan_bao_serialized = parts.yuan_bao.serialize(destination);
    let _ji_fen_serialized = parts.ji_fen.serialize(destination);
    append_player_game_save_string(
        destination,
        "m_strDepotPassword",
        parts.depot_password,
        0x6c,
    )?;
    let _bank_serialized = parts.bank.serialize(destination);
    let _depot_serialized = parts.depot.serialize(destination, goods_factory);
    let _fairy_serialized = parts.fairy_container.serialize(destination, goods_factory);
    let _battle_fairy_serialized = parts
        .battle_fairy_container
        .serialize(destination, goods_factory);
    let _ci_qing_serialized = parts.ci_qing.serialize(destination, goods_factory);
    let _ci_qing_compose_serialized = parts.ci_qing_compose.serialize(destination, goods_factory);
    let _variables_serialized = parts.variable_list.encode_world_snapshot(destination);
    LegacyWriter::new(destination).write_i32(*parts.silence_minutes);
    // Оригинал round-trip-ит сырой byte murderer state (`+0x100`, запись в
    // `0x00441034..0x00441048`), сохранённый decoder-ом; Rust не хранит
    // отдельный флаг и пересчитывает его из pk_count и timestamp.
    // Расхождение наблюдаемо только на грязном wire (byte не согласован с
    // этой парой) — сознательная нормализация; записанный ниже remainder
    // побайтово совпадает с машинным clamp (`0x0044104C..0x00441077`).
    let murderer_state =
        parts.base_properties.pk_count != 0 && *parts.murderer_time_stamp_ms != 0;
    LegacyWriter::new(destination).write_u8(u8::from(murderer_state));
    let murderer_remain = if *parts.murderer_time_stamp_ms == 0 {
        0
    } else {
        parts
            .murderer_time_stamp_ms
            .wrapping_add(one_pk_count_time_ms)
            .wrapping_sub(now_ms)
            .min(one_pk_count_time_ms)
    };
    LegacyWriter::new(destination).write_u32(murderer_remain);
    LegacyWriter::new(destination).write_i32(*parts.fight_state_count);
    append_player_game_save_count(destination, "m_vUncreatedPets", pets.len())?;
    for pet in pets {
        append_player_game_save_string(
            destination,
            "tagPetInformation.strOriginalName",
            &pet.original_name,
            0x94,
        )?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_u32(pet.health);
        writer.write_u32(pet.level);
        writer.write_u32(pet.experience);
    }
    append_player_game_save_string(
        destination,
        "tagCarriageInfo.strOriginalName",
        &carriage.original_name,
        0x94,
    )?;
    append_player_game_save_string(
        destination,
        "tagCarriageInfo.strCarriageScript",
        &carriage.script,
        0x94,
    )?;
    let mut writer = LegacyWriter::new(destination);
    writer.write_u32(carriage.health);
    writer.write_u8(u8::from(recreate_carriage));
    writer.write_u8(u8::from(*parts.login));
    writer.write_i32(*parts.city_war_died_state_time_ms);
    append_player_game_save_count(destination, "m_PlayerQuests", parts.quest_progress.len())?;
    for (quest_id, state) in parts.quest_progress.iter() {
        let mut writer = LegacyWriter::new(destination);
        writer.write_u16(*quest_id);
        writer.write_u8(*state);
    }
    LegacyWriter::new(destination).write_u8(*parts.country);
    LegacyWriter::new(destination).write_i32(*parts.contribution);
    destination.extend_from_slice(parts.jjc_data);
    destination.push(u8::from(*parts.jjc_pk_state));
    append_player_game_save_string(destination, "m_strSessionID", parts.session_id, 0x40)?;
    Ok(())
}

/// Общий обход `AddSkillsToByteArray` (`0x00432b80`): навыки в порядке
/// Attack→Defense→Summon→State, внутри категории — в порядке регистрации;
/// фильтр `GetNumSkills` (`0x00432aa0`) исключает только Defense с ID 10.
/// GameSave и initial client используют один обход.
pub fn serializable_player_skills(
    move_shape: &MoveShapeState,
) -> impl Iterator<Item = &MoveShapeSkill> {
    [
        SkillCategory::Attack,
        SkillCategory::Defense,
        SkillCategory::Summon,
        SkillCategory::State,
    ]
    .into_iter()
    .flat_map(|category| {
        move_shape
            .skills
            .skills_in_category(category)
            .filter(move |skill| category != SkillCategory::Defense || skill.id() != SKILL_BASE_DEFENSE)
    })
}

/// Синхронизация typed-снимка базовых свойств обратно в base-wire перед
/// записью: обновляет только подтверждённые поля, не обнуляя хвост.
pub fn synchronized_base_property_wire(
    base_wire: &[u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    base_properties: &PlayerBaseProperties,
    battle_fairy_summoned: bool,
) -> [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE] {
    let mut wire = *base_wire;
    wire[BASE_LEVEL_OFFSET] = base_properties.level;
    write_player_wire_u32(&mut wire, BASE_EXPERIENCE_OFFSET, base_properties.experience);
    wire[BASE_HEAD_PICTURE_OFFSET] = base_properties.head_picture as u8;
    wire[BASE_FACE_PICTURE_OFFSET] = base_properties.face_picture as u8;
    wire[BASE_OCCUPATION_OFFSET] = base_properties.occupation;
    wire[BASE_SEX_OFFSET] = base_properties.sex;
    write_player_wire_u16(&mut wire, BASE_PK_COUNT_OFFSET, base_properties.pk_count);
    write_player_wire_u32(&mut wire, BASE_KILL_COUNT_OFFSET, base_properties.kill_count);
    write_player_wire_u16(&mut wire, BASE_HIT_TOP_LOG_OFFSET, base_properties.hit_top_log);
    write_player_wire_u16(&mut wire, BASE_REMAIN_POINT_OFFSET, base_properties.remain_point);
    wire[BASE_CHARGED_OFFSET] = u8::from(base_properties.charged);
    for (index, hotkey) in base_properties.hotkeys.iter().copied().enumerate() {
        write_player_wire_u32(&mut wire, BASE_HOTKEY_OFFSET + index * 4, hotkey);
    }
    for (offset, value) in [
        (BASE_PK_NORMAL_OFFSET, base_properties.pk_normal),
        (BASE_PK_TEAM_OFFSET, base_properties.pk_team),
        (BASE_PK_UNION_OFFSET, base_properties.pk_union),
        (BASE_PK_BADMAN_OFFSET, base_properties.pk_badman),
        (BASE_PK_COUNTRY_OFFSET, base_properties.pk_country),
        (
            BASE_FAIRY_CONTAINER_ENABLED_OFFSET,
            base_properties.fairy_container_enabled,
        ),
        (
            BASE_BATTLE_FAIRY_ENABLED_OFFSET,
            base_properties.battle_fairy_enabled,
        ),
        (
            BASE_DISPLAY_HEAD_PIECE_OFFSET,
            base_properties.display_head_piece,
        ),
        (BASE_BATTLE_FAIRY_SUMMONED_OFFSET, battle_fairy_summoned),
        (
            BASE_BATTLE_FAIRY_RECALL_OFFSET,
            base_properties.battle_fairy_recall,
        ),
        (
            BASE_BATTLE_FAIRY_DIED_OFFSET,
            base_properties.battle_fairy_died,
        ),
        (
            BASE_QUEST_ENABLED_OFFSET,
            base_properties.quest_availability.enabled(),
        ),
    ] {
        wire[offset] = u8::from(value);
    }
    for (offset, value) in [
        (BASE_HEALTH_OFFSET, base_properties.health),
        (BASE_MANA_OFFSET, base_properties.mana),
        (BASE_MAXIMUM_HP_OFFSET, base_properties.base_maximum_hp),
        (BASE_MAXIMUM_MP_OFFSET, base_properties.base_maximum_mp),
        (BASE_STRENGTH_OFFSET, base_properties.base_strength),
        (BASE_DEXTERITY_OFFSET, base_properties.base_dexterity),
        (
            BASE_CONSTITUTION_OFFSET,
            base_properties.base_constitution,
        ),
        (
            BASE_INTELLIGENCE_OFFSET,
            base_properties.base_intelligence,
        ),
        (BASE_VIGOUR_OFFSET, base_properties.vigour),
        (
            BASE_MAXIMUM_VIGOUR_OFFSET,
            base_properties.maximum_vigour,
        ),
        (BASE_ENERGY_OFFSET, base_properties.energy),
        (
            BASE_MAXIMUM_ENERGY_OFFSET,
            base_properties.maximum_energy,
        ),
        (BASE_CREDIT_OFFSET, base_properties.credit),
        (BASE_EXALT_OFFSET, base_properties.exalt),
        (
            BASE_QUEST_TIME_BEGIN_OFFSET,
            base_properties.quest_availability.time_begin() as u32,
        ),
        (
            BASE_QUEST_TIME_LIMIT_OFFSET,
            base_properties.quest_availability.time_limit() as u32,
        ),
        (BASE_EXPLOIT_OFFSET, base_properties.exploit),
        (BASE_BREAK_ARMOUR_OFFSET, base_properties.break_armour),
        (BASE_PUNCTURE_OFFSET, base_properties.puncture),
        (BASE_BREAK_ELEMENT_OFFSET, base_properties.break_element),
        (BASE_BREAK_BOUND_OFFSET, base_properties.break_bound),
        (BASE_POWER_OF_GOLD_OFFSET, base_properties.power_of_gold),
        (
            BASE_DAYS_HONOR_OFFSET,
            base_properties.days_honor_eliminate,
        ),
        (
            BASE_WEEKS_HONOR_OFFSET,
            base_properties.weeks_honor_eliminate,
        ),
        (
            BASE_MONTHS_HONOR_OFFSET,
            base_properties.months_honor_eliminate,
        ),
        (
            BASE_TOTAL_HONOR_OFFSET,
            base_properties.total_honor_eliminate,
        ),
        (
            BASE_RANK_OF_NOBILITY_OFFSET,
            base_properties.rank_of_nobility_id,
        ),
        (BASE_APPELLATION_OFFSET, base_properties.appellation_id),
        (BASE_MODE_OFFSET, base_properties.mode),
        (BASE_FETCH_POWER_OFFSET, base_properties.fetch_power),
        (
            BASE_AUCTION_SPACE_OFFSET,
            base_properties.auction_space,
        ),
        (BASE_JJC_LEVEL_OFFSET, base_properties.jjc_level),
        (BASE_JJC_SCORE_OFFSET, base_properties.jjc_score),
        (BASE_FY_ENERGY_OFFSET, base_properties.fy_energy),
        (
            BASE_FY_ENABLE_FLAGS_OFFSET,
            base_properties.fy_enable_flags.bits(),
        ),
        (BASE_LT_60_STAMP_OFFSET, base_properties.lt_60_stamp),
        (BASE_SZL_OFFSET, base_properties.szl),
        (
            BASE_GODS_BATTLE_FACTION_OFFSET,
            base_properties.gods_battle_faction as u32,
        ),
    ] {
        write_player_wire_u32(&mut wire, offset, value);
    }
    write_player_wire_u16(
        &mut wire,
        BASE_LT_UP_60_COUNT_OFFSET,
        base_properties.lt_up_60_count,
    );
    write_player_wire_u16(&mut wire, BASE_RP_OFFSET, base_properties.rp);
    write_player_wire_u16(&mut wire, BASE_YP_OFFSET, base_properties.yp);
    write_player_wire_u16(
        &mut wire,
        BASE_MAXIMUM_YP_OFFSET,
        base_properties.maximum_yp,
    );
    write_player_wire_u16(&mut wire, BASE_BURDEN_OFFSET, base_properties.base_burden);
    write_player_wire_u16(&mut wire, BASE_MAXIMUM_RP_OFFSET, base_properties.maximum_rp);
    write_player_wire_u16(
        &mut wire,
        BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET,
        base_properties.remain_jing_li_dan_count,
    );
    wire
}

fn restore_player_murderer_timestamp(
    pk_count: u16,
    murderer_time_stamp_ms: &mut u32,
    murderer_state: bool,
    murderer_remain: u32,
    now_ms: u32,
    one_pk_count_time_ms: u32,
) {
    if pk_count == 0 {
        *murderer_time_stamp_ms = 0;
        return;
    }
    if murderer_state || *murderer_time_stamp_ms == 0 {
        *murderer_time_stamp_ms = now_ms;
    }
    if murderer_remain != 0 {
        let remain = murderer_remain.min(one_pk_count_time_ms);
        *murderer_time_stamp_ms = now_ms.wrapping_sub(one_pk_count_time_ms.wrapping_sub(remain));
    }
}

/// Отражение base-wire в typed-снимок: тот же layout, что
/// [`synchronized_base_property_wire`]. Побочный флаг призыва боевой феи
/// читается из wire-байта; decode-владелец затем выводит его из
/// `m_dwWarSoulState`.
pub fn apply_base_property_wire(
    wire: &[u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
) -> (PlayerBaseProperties, bool) {
    let mut base_properties = PlayerBaseProperties {
        level: wire[BASE_LEVEL_OFFSET],
        experience: read_player_wire_u32(wire, BASE_EXPERIENCE_OFFSET),
        head_picture: i32::from(wire[BASE_HEAD_PICTURE_OFFSET]),
        face_picture: i32::from(wire[BASE_FACE_PICTURE_OFFSET]),
        occupation: wire[BASE_OCCUPATION_OFFSET],
        sex: wire[BASE_SEX_OFFSET],
        pk_count: read_player_wire_u16(wire, BASE_PK_COUNT_OFFSET),
        kill_count: read_player_wire_u32(wire, BASE_KILL_COUNT_OFFSET),
        hit_top_log: read_player_wire_u16(wire, BASE_HIT_TOP_LOG_OFFSET),
        charged: wire[BASE_CHARGED_OFFSET] != 0,
        remain_point: read_player_wire_u16(wire, BASE_REMAIN_POINT_OFFSET),
        pk_normal: wire[BASE_PK_NORMAL_OFFSET] != 0,
        pk_team: wire[BASE_PK_TEAM_OFFSET] != 0,
        pk_union: wire[BASE_PK_UNION_OFFSET] != 0,
        pk_badman: wire[BASE_PK_BADMAN_OFFSET] != 0,
        pk_country: wire[BASE_PK_COUNTRY_OFFSET] != 0,
        health: read_player_wire_u32(wire, BASE_HEALTH_OFFSET),
        mana: read_player_wire_u32(wire, BASE_MANA_OFFSET),
        rp: read_player_wire_u16(wire, BASE_RP_OFFSET),
        yp: read_player_wire_u16(wire, BASE_YP_OFFSET),
        maximum_yp: read_player_wire_u16(wire, BASE_MAXIMUM_YP_OFFSET),
        maximum_rp: read_player_wire_u16(wire, BASE_MAXIMUM_RP_OFFSET),
        base_maximum_hp: read_player_wire_u32(wire, BASE_MAXIMUM_HP_OFFSET),
        base_maximum_mp: read_player_wire_u32(wire, BASE_MAXIMUM_MP_OFFSET),
        base_burden: read_player_wire_u16(wire, BASE_BURDEN_OFFSET),
        base_strength: read_player_wire_u32(wire, BASE_STRENGTH_OFFSET),
        base_dexterity: read_player_wire_u32(wire, BASE_DEXTERITY_OFFSET),
        base_constitution: read_player_wire_u32(wire, BASE_CONSTITUTION_OFFSET),
        base_intelligence: read_player_wire_u32(wire, BASE_INTELLIGENCE_OFFSET),
        vigour: read_player_wire_u32(wire, BASE_VIGOUR_OFFSET),
        maximum_vigour: read_player_wire_u32(wire, BASE_MAXIMUM_VIGOUR_OFFSET),
        energy: read_player_wire_u32(wire, BASE_ENERGY_OFFSET),
        maximum_energy: read_player_wire_u32(wire, BASE_MAXIMUM_ENERGY_OFFSET),
        credit: read_player_wire_u32(wire, BASE_CREDIT_OFFSET),
        exalt: read_player_wire_u32(wire, BASE_EXALT_OFFSET),
        display_head_piece: wire[BASE_DISPLAY_HEAD_PIECE_OFFSET] != 0,
        quest_availability: PlayerQuestAvailability::from_snapshot(
            read_player_wire_u32(wire, BASE_QUEST_TIME_BEGIN_OFFSET) as i32,
            read_player_wire_u32(wire, BASE_QUEST_TIME_LIMIT_OFFSET) as i32,
            wire[BASE_QUEST_ENABLED_OFFSET] != 0,
        ),
        exploit: read_player_wire_u32(wire, BASE_EXPLOIT_OFFSET),
        fairy_container_enabled: wire[BASE_FAIRY_CONTAINER_ENABLED_OFFSET] != 0,
        battle_fairy_enabled: wire[BASE_BATTLE_FAIRY_ENABLED_OFFSET] != 0,
        break_armour: read_player_wire_u32(wire, BASE_BREAK_ARMOUR_OFFSET),
        puncture: read_player_wire_u32(wire, BASE_PUNCTURE_OFFSET),
        break_element: read_player_wire_u32(wire, BASE_BREAK_ELEMENT_OFFSET),
        break_bound: read_player_wire_u32(wire, BASE_BREAK_BOUND_OFFSET),
        power_of_gold: read_player_wire_u32(wire, BASE_POWER_OF_GOLD_OFFSET),
        days_honor_eliminate: read_player_wire_u32(wire, BASE_DAYS_HONOR_OFFSET),
        weeks_honor_eliminate: read_player_wire_u32(wire, BASE_WEEKS_HONOR_OFFSET),
        months_honor_eliminate: read_player_wire_u32(wire, BASE_MONTHS_HONOR_OFFSET),
        total_honor_eliminate: read_player_wire_u32(wire, BASE_TOTAL_HONOR_OFFSET),
        rank_of_nobility_id: read_player_wire_u32(wire, BASE_RANK_OF_NOBILITY_OFFSET),
        appellation_id: read_player_wire_u32(wire, BASE_APPELLATION_OFFSET),
        mode: read_player_wire_u32(wire, BASE_MODE_OFFSET),
        fetch_power: read_player_wire_u32(wire, BASE_FETCH_POWER_OFFSET),
        battle_fairy_recall: wire[BASE_BATTLE_FAIRY_RECALL_OFFSET] != 0,
        battle_fairy_died: wire[BASE_BATTLE_FAIRY_DIED_OFFSET] != 0,
        auction_space: read_player_wire_u32(wire, BASE_AUCTION_SPACE_OFFSET),
        jjc_level: read_player_wire_u32(wire, BASE_JJC_LEVEL_OFFSET),
        jjc_score: read_player_wire_u32(wire, BASE_JJC_SCORE_OFFSET),
        fy_energy: read_player_wire_u32(wire, BASE_FY_ENERGY_OFFSET),
        fy_enable_flags: LeiTingEnableFlags::from_bits_retain(read_player_wire_u32(
            wire,
            BASE_FY_ENABLE_FLAGS_OFFSET,
        )),
        lt_up_60_count: read_player_wire_u16(wire, BASE_LT_UP_60_COUNT_OFFSET),
        remain_jing_li_dan_count: read_player_wire_u16(wire, BASE_REMAIN_JING_LI_DAN_COUNT_OFFSET),
        lt_60_stamp: read_player_wire_u32(wire, BASE_LT_60_STAMP_OFFSET),
        szl: read_player_wire_u32(wire, BASE_SZL_OFFSET),
        gods_battle_faction: read_player_wire_u32(wire, BASE_GODS_BATTLE_FACTION_OFFSET) as i32,
        ..PlayerBaseProperties::default()
    };
    for (index, hotkey) in base_properties.hotkeys.iter_mut().enumerate() {
        *hotkey = read_player_wire_u32(wire, BASE_HOTKEY_OFFSET + index * 4);
    }
    let battle_fairy_summoned = wire[BASE_BATTLE_FAIRY_SUMMONED_OFFSET] != 0;
    (base_properties, battle_fairy_summoned)
}

/// Отражение `m_Property[0x9c]` combat-wire в typed-снимок боевых свойств.
pub fn apply_combat_property_wire(
    wire: &[u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
) -> PlayerCombatProperties {
    PlayerCombatProperties {
        maximum_hp: read_player_wire_u32(wire, 0x00),
        maximum_mp: read_player_wire_u32(wire, 0x04),
        maximum_yp: read_player_wire_u16(wire, 0x08),
        maximum_rp: read_player_wire_u16(wire, 0x0a),
        strength: read_player_wire_u32(wire, 0x0c),
        dexterity: read_player_wire_u32(wire, 0x10),
        constitution: read_player_wire_u32(wire, 0x14),
        intelligence: read_player_wire_u32(wire, 0x18),
        minimum_attack: read_player_wire_u32(wire, 0x1c),
        maximum_attack: read_player_wire_u32(wire, 0x20),
        attack_speed: read_player_wire_u16(wire, 0x32),
        hit: read_player_wire_u16(wire, 0x24),
        dodge: read_player_wire_u16(wire, 0x30),
        cch: read_player_wire_u16(wire, 0x28),
        burden: read_player_wire_u16(wire, 0x26),
        defense: read_player_wire_u32(wire, 0x2c),
        element_resistance: read_player_wire_u32(wire, 0x34),
        add_element_attack: read_player_wire_u32(wire, 0x40),
        hp_recovery: read_player_wire_u16(wire, 0x38),
        mp_recovery: read_player_wire_u16(wire, 0x3a),
        element_modify: read_player_wire_u32(wire, 0x48) as i32,
        reank: read_player_wire_u16(wire, 0x4c),
        attack_avoid: read_player_wire_u16(wire, 0x4e),
        element_avoid: read_player_wire_u16(wire, 0x50),
        full_miss: read_player_wire_u16(wire, 0x52),
        blast_attack: read_player_wire_u16(wire, 0x54),
        blast_element_attack: read_player_wire_u16(wire, 0x56),
        soul_resistance: read_player_wire_u16(wire, 0x3c),
        add_soul_attack: read_player_wire_u16(wire, 0x44),
        blast_attack_scale_bits: read_player_wire_u32(wire, 0x58),
        blast_defense_scale_bits: read_player_wire_u32(wire, 0x5c),
        element_blast_attack_scale_bits: read_player_wire_u32(wire, 0x60),
        element_blast_defense_scale_bits: read_player_wire_u32(wire, 0x64),
        full_miss_scale_bits: read_player_wire_u32(wire, 0x68),
        critical_rate_bits: read_player_wire_u32(wire, 0x6c),
        resume_hp_peace: read_player_wire_u32(wire, 0x70) as i32,
        resume_mp_peace: read_player_wire_u32(wire, 0x74) as i32,
        resume_hp_fight: read_player_wire_u32(wire, 0x78) as i32,
        resume_mp_fight: read_player_wire_u32(wire, 0x7c) as i32,
        restored_hp_peace: read_player_wire_u32(wire, 0x80) as i32,
        restored_mp_peace: read_player_wire_u32(wire, 0x84) as i32,
        restored_hp_fight: read_player_wire_u32(wire, 0x88) as i32,
        restored_mp_fight: read_player_wire_u32(wire, 0x8c) as i32,
        battle_fairy_summoned: wire[0x90] != 0,
        battle_fairy_recall: wire[0x91] != 0,
        battle_fairy_died: wire[0x92] != 0,
    }
}

/// Decode организационного блока GameSave. `Quest-map`, skill-list,
/// friend-list и три organization-list decoder-а сохраняют signed legacy
/// count: отрицательное значение очищает коллекцию и не отклоняет остальной
/// handoff.
pub fn decode_player_organizing_snapshot(
    source: &[u8],
    cursor: &mut usize,
    parts: &mut PlayerOrganizingParts<'_>,
) -> Result<(), PlayerGameSaveCodecError> {
    *parts.faction_id = read_player_game_save_i32(source, cursor, "m_lFactionID")?;
    if *parts.faction_id > 0 {
        *parts.faction_logo_id = read_player_game_save_i32(source, cursor, "m_lFactionLogoID")?;
        *parts.faction_level = read_player_game_save_u16(source, cursor, "m_wFactionLevel")?;
        *parts.faction_experience =
            read_player_game_save_i32(source, cursor, "m_lFactionExperience")?;
        *parts.faction_force = i32::from(read_player_game_save_i32(source, cursor, "m_lForce")? != 0);
        *parts.faction_contribute = u32::from(
            read_player_game_save_i32(source, cursor, "m_bFactionContribute")? != 0,
        );
        *parts.faction_name = read_player_game_save_string(source, cursor, "m_strFactionName", 0x100)?;
        *parts.faction_title =
            read_player_game_save_string(source, cursor, "m_strFactionTitle", 0x100)?;
        *parts.faction_master_id =
            read_player_game_save_i32(source, cursor, "m_lFactionMasterID")?;
        *parts.union_id = read_player_game_save_i32(source, cursor, "m_lUnionID")?;
        *parts.union_master_id = read_player_game_save_i32(source, cursor, "m_lUnionMasterID")?;
        for (field, destination) in [
            ("m_EnemyFactions", &mut *parts.enemy_factions),
            ("m_CityWarEnemyFactions", &mut *parts.city_war_enemy_factions),
        ] {
            let count = read_player_game_save_i32(source, cursor, field)?;
            destination.clear();
            for _ in 0..count.max(0) {
                destination.insert(read_player_game_save_i32(source, cursor, field)?);
            }
        }
        let count = read_player_game_save_i32(source, cursor, "m_OwnedRegions")?;
        parts.faction_owned_regions.clear();
        for _ in 0..count.max(0) {
            let wire = read_player_game_save_slice(source, cursor, "m_OwnedRegions", 8)?;
            parts.faction_owned_regions.push(wire.try_into().expect("размер проверен"));
        }
    } else {
        *parts.faction_logo_id = 0;
        *parts.faction_level = 0;
        *parts.faction_experience = 0;
        *parts.faction_force = 0;
        *parts.faction_contribute = 0;
        parts.faction_name.clear();
        parts.faction_title.clear();
        parts.enemy_factions.clear();
        parts.city_war_enemy_factions.clear();
        parts.faction_owned_regions.clear();
        *parts.faction_master_id = 0;
        *parts.union_id = 0;
        *parts.union_master_id = 0;
    }
    Ok(())
}

/// Encode организационного блока: идентичная проекция для area/login
/// клиентских снимков (`0x7FE06` wire layout).
pub fn encode_player_organizing_snapshot(
    snapshot: PlayerOrganizingSnapshot<'_>,
) -> Option<Vec<u8>> {
    let mut payload = Vec::new();
    let mut writer = LegacyWriter::new(&mut payload);
    writer.write_i32(snapshot.faction_id);
    if snapshot.faction_id <= 0 {
        return Some(payload);
    }
    writer.write_i32(snapshot.faction_logo_id);
    writer.write_u16(snapshot.faction_level);
    writer.write_i32(snapshot.faction_experience);
    writer.write_i32(snapshot.faction_force);
    writer.write_u32(snapshot.faction_contribute);
    writer.write_c_string(snapshot.faction_name);
    writer.write_c_string(snapshot.faction_title);
    writer.write_i32(snapshot.faction_master_id);
    writer.write_i32(snapshot.union_id);
    writer.write_i32(snapshot.union_master_id);
    writer.write_i32(i32::try_from(snapshot.enemy_factions.len()).ok()?);
    for faction_id in snapshot.enemy_factions {
        writer.write_i32(*faction_id);
    }
    writer.write_i32(i32::try_from(snapshot.city_war_enemy_factions.len()).ok()?);
    for faction_id in snapshot.city_war_enemy_factions {
        writer.write_i32(*faction_id);
    }
    writer.write_i32(i32::try_from(snapshot.faction_owned_regions.len()).ok()?);
    for region in snapshot.faction_owned_regions {
        writer.write_bytes(region);
    }
    Some(payload)
}

/// LeiTing snapshot codec: совпадает с WorldServer `AddByteArrayLeiTing`.
/// Пять scalar-полей игрока и ordered `tagThing` list.
pub fn encode_player_lei_ting(
    base_properties: &PlayerBaseProperties,
    lei_ting_things: &VecDeque<PlayerLeiTingThing>,
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(20 + lei_ting_things.len() * 8);
    let mut writer = LegacyWriter::new(&mut payload);
    writer.write_u32(base_properties.fy_enable_flags.bits());
    writer.write_u32(base_properties.fy_energy);
    writer.write_u32(base_properties.lt_60_stamp);
    writer.write_u16(base_properties.lt_up_60_count);
    writer.write_u16(base_properties.remain_jing_li_dan_count);
    writer.write_u32(lei_ting_things.len() as u32);
    for thing in lei_ting_things {
        writer.write_u16(thing.thing_id);
        writer.write_u16(thing.count);
        writer.write_u16(thing.max_count);
        writer.write_u16(thing.point);
    }
    payload
}

/// Парный decoder [`encode_player_lei_ting`].
pub fn decode_player_lei_ting(
    base_properties: &mut PlayerBaseProperties,
    lei_ting_things: &mut VecDeque<PlayerLeiTingThing>,
    source: &[u8],
    cursor: &mut usize,
) -> Result<(), PlayerLeiTingDecodeBlock> {
    fn read_u32(
        source: &[u8],
        cursor: &mut usize,
        field: &'static str,
    ) -> Result<u32, PlayerLeiTingDecodeBlock> {
        let offset = *cursor;
        let available = source.len().saturating_sub(offset);
        let mut reader = LegacyReader::at(source, offset).map_err(|_| PlayerLeiTingDecodeBlock {
            field,
            offset,
            needed: 4,
            available,
        })?;
        let value = reader.read_u32().map_err(|_| PlayerLeiTingDecodeBlock {
            field,
            offset,
            needed: 4,
            available,
        })?;
        *cursor = reader.position();
        Ok(value)
    }
    fn read_u16(
        source: &[u8],
        cursor: &mut usize,
        field: &'static str,
    ) -> Result<u16, PlayerLeiTingDecodeBlock> {
        let offset = *cursor;
        let available = source.len().saturating_sub(offset);
        let mut reader = LegacyReader::at(source, offset).map_err(|_| PlayerLeiTingDecodeBlock {
            field,
            offset,
            needed: 2,
            available,
        })?;
        let value = reader.read_u16().map_err(|_| PlayerLeiTingDecodeBlock {
            field,
            offset,
            needed: 2,
            available,
        })?;
        *cursor = reader.position();
        Ok(value)
    }

    base_properties.fy_enable_flags =
        LeiTingEnableFlags::from_bits_retain(read_u32(source, cursor, "dwfyenFlag")?);
    base_properties.fy_energy = read_u32(source, cursor, "dwfyEnergy")?;
    base_properties.lt_60_stamp = read_u32(source, cursor, "dwLT60Stamp")?;
    base_properties.lt_up_60_count = read_u16(source, cursor, "wLTUp60Cnt")?;
    base_properties.remain_jing_li_dan_count = read_u16(source, cursor, "wRemainJingLiDanCnt")?;
    lei_ting_things.clear();
    let count = read_u32(source, cursor, "m_listThing count")?;
    for _ in 0..count {
        lei_ting_things.push_back(PlayerLeiTingThing {
            thing_id: read_u16(source, cursor, "tagThing.wTID")?,
            count: read_u16(source, cursor, "tagThing.wCnt")?,
            max_count: read_u16(source, cursor, "tagThing.wMaxCnt")?,
            point: read_u16(source, cursor, "tagThing.wPoint")?,
        });
    }
    Ok(())
}

fn read_player_game_save_slice<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    field: &'static str,
    needed: usize,
) -> Result<&'a [u8], PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, needed)?;
    let bytes = reader
        .read_bytes(needed)
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn read_player_game_save_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], PlayerGameSaveCodecError> {
    Ok(read_player_game_save_slice(source, cursor, field, N)?
        .try_into()
        .expect("player wire slice имеет запрошенную длину"))
}

fn read_player_game_save_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 1)?;
    let value = reader
        .read_u8()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 2)?;
    let value = reader
        .read_u16()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_u32()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, PlayerGameSaveCodecError> {
    let mut reader = player_save_reader(source, *cursor, field, 4)?;
    let value = reader
        .read_i32()
        .map_err(|block| player_save_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_player_game_save_count(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<usize, PlayerGameSaveCodecError> {
    let count = read_player_game_save_i32(source, cursor, field)?;
    usize::try_from(count).map_err(|_| PlayerGameSaveCodecError::NegativeCount { field, count })
}

fn read_player_game_save_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    maximum: usize,
) -> Result<Vec<u8>, PlayerGameSaveCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let mut reader = player_save_reader(source, offset, field, 1)?;
    let bytes = reader
        .read_c_string(available)
        .map_err(|block| player_save_read_error(field, block))?;
    let length = bytes.len();
    if length >= maximum {
        return Err(PlayerGameSaveCodecError::StringTooLong {
            field,
            length,
            maximum,
        });
    }
    *cursor = reader.position();
    Ok(bytes.to_vec())
}

fn append_player_game_save_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    length: usize,
) -> Result<(), PlayerGameSaveCodecError> {
    let count = i32::try_from(length)
        .map_err(|_| PlayerGameSaveCodecError::CollectionTooLarge { field, length })?;
    LegacyWriter::new(destination).write_i32(count);
    Ok(())
}

fn append_player_game_save_string(
    destination: &mut Vec<u8>,
    field: &'static str,
    value: &[u8],
    maximum: usize,
) -> Result<(), PlayerGameSaveCodecError> {
    let length = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    if length >= maximum {
        return Err(PlayerGameSaveCodecError::StringTooLong {
            field,
            length,
            maximum,
        });
    }
    LegacyWriter::new(destination).write_c_string(&value[..length]);
    Ok(())
}

fn player_save_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, PlayerGameSaveCodecError> {
    LegacyReader::at(source, cursor).map_err(|block| PlayerGameSaveCodecError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn player_save_read_error(field: &'static str, block: LegacyReadBlock) -> PlayerGameSaveCodecError {
    PlayerGameSaveCodecError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}

pub fn read_player_wire_u16(wire: &[u8], offset: usize) -> u16 {
    LegacyReader::at(wire, offset)
        .and_then(|mut reader| reader.read_u16())
        .expect("base/property wire offset проверен layout-константой")
}

pub fn read_player_wire_u32(wire: &[u8], offset: usize) -> u32 {
    LegacyReader::at(wire, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("base/property wire offset проверен layout-константой")
}

fn write_player_wire_u16(wire: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(wire, offset, value)
        .expect("base/property wire offset проверен layout-константой");
}

fn write_player_wire_u32(wire: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(wire, offset, value)
        .expect("base/property wire offset проверен layout-константой");
}
