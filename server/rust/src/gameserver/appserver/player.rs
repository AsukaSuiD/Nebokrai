//! Hub `CPlayer` переходного пакета: владелец живого state игрока и его
//! адаптеров (send-family проекция исторического GameServer). Значимое
//! поведение держат части и их владельцы в Zone; этот hub хранит поля,
//! accessors и делегаты прежних сигнатур единого `CGame`/message-контура.
//!
//! GameSave codec и клиентские снимки игрока — Zone `players`
//! (`zone/src/players/`):
//! - `decode_game_save`/`encode_game_save`, base/combat property wire,
//!   organizing snapshot, LeiTing codec, init/refresh persistence-контейнеров
//!   и унаследованные машинные факты (`0x00441399` discard-цепочка,
//!   `DelAllItemInDelList` UNKNOWN, D1/D8 отложенные границы) —
//!   `zone::players::gamesave`; layout VERIFIED машинной разведкой
//!   GameSave-форматов (fix D2/D4).
//! - `encode_client_shape_snapshot` (short area/query) и
//!   `encode_initial_client_snapshot` (full login `0xBF401`) —
//!   `zone::players::clientsnapshot`.
//! - Codec-типы и wire-константы — re-export совместимости ниже для hub-а
//!   и его callers; делегаты передают заёмные проекции полей.
//! - Временные generic-швы: realm-appellation bonus-предикат и
//!   `CanMountEquip` из hub-а — см. шапку `zone::players::gamesave`.
//!
//! Машинные факты остающихся hub-частей (якоря той же точной пары):
//! ctor (`0x004590DC/0x00459365`) задаёт InChangingRegion=true и last-enter=0;
//! `InitSkills` вызывается из `OnLogMessage` после login-script (`0x0049FC0A`),
//! `OnChangeSkill` (`0x00508E6A..0x00508E7E`) выбирает GetDefaultAttackSkillID;
//! `UpdateProperty` (`0x004593E0`) держит формулы экипировки здесь, `+0x24`
//! состояний — общий проход CMoveShape; SelfTarget: virtual +0x78 в `0x00488E20`,
//! item — `0x00489109/0x00489547`, WarSoul — `0x0048953D/0x00489547`;
//! `SummonBF` RVA `0x00101CB0` — единый Player→CGame→region проход;
//! `LoadBFDefualtProperty` (`0x00502BC0`) проверяет GetSkill после AddSkill;
//! расход MP атрибутных Po/Yu — непосредственно equipment[10] (`CPojia::AI`
//! `0x0052a77f`); hand goods сериализуется перед экипировкой (`0x00440dc0`);
//! Particular `OnObjectAdded` (`0x004451A0`) Begin синхронно после commit и
//! GoodsAI; equipment-listener (`0x004EF6C0`) публикует skills/properties/
//! BF720/PackExpand до товара; `OnEnterRegion` (`0x0045A410`) — ordered unique,
//! packet-вектор отбрасывается; Packet Add (`0x004DE6E0`) регистрирует GoodsAI
//! до player-listener; `ComputeTicket` (`0x0043E820`) читает часы после
//! life/ticket/type/start gates; Mount (`0x00444E20`) — name перед ctor/Begin,
//! fight-state GS0154 без установки; CHBY `OnLost` (`0x0044183B..0x00441894`) —
//! !restore_online шлёт исходный BF806 и вызывает End сразу.
//!
//! Исходный владелец: `server/gameserver/appserver/player.h/.cpp`, точная
//! пара GameServer/gameserver.exe + GameServer.pdb (идентификаторы сборки —
//! COMPONENT_VARIANT в конце файла). Статусы перенесённых тел не повышаются
//! и живут рядом с кодом `zone::players`.

use super::ai::playerai::CPlayerAI;
use super::area::WarSoulPoint;
use super::container::camountlimitgoodscontainer::{
    AmountLimitGoodsAdded, AmountLimitGoodsRemoved, AmountLimitGoodsTaken,
    CAmountLimitGoodsContainer,
};
use super::container::camountlimitgoodsshadowcontainer::{
    AmountShadowAdded, CAmountLimitGoodsShadowContainer,
};
use super::container::cbank::{BankGoodsAddOutcome, CBank};
use super::container::cbattlefairycontainer::{
    BattleFairyCell, BattleFairyCombineCheck, BattleFairyContainerAddOutcome,
    BattleFairyDefaultGoodsUpdate, BattleFairyDefaultSkill, BattleFairyUpgradeConsumedGem,
    CBattleFairyContainer,
};
use super::container::ccontainer::ContainerListenerHandle;
use super::container::ccontainer::PreviousContainer;
use super::container::cdepot::CDepot;
use super::container::cequipmentcontainer::{
    CEquipmentContainer, EquipmentAddOutcome, EquipmentAddRuntimeFacts, EquipmentAroundUpdate,
    EquipmentColumn, EquipmentOwnerPlayerFacts,
    EquipmentRemoveOutcome, EquipmentRemoveRuntimeFacts,
};
use super::container::cfairycontainer::CFairyContainer;
use super::container::cgoodscontainer::GoodsStackMergeOutcome;
use super::container::cgoodsshadowcontainer::{PlacedShadowGoods, ShadowRecordBlock};
use super::container::cjifen::CJiFen;
use super::container::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome,
    VolumeGoodsRemoveOutcome, VolumeGoodsSwapOutcome,
};
use super::container::cwallet::{
    CWallet, CurrencyDecreaseOutcome, CurrencyGoodsAddOutcome, CurrencyGoodsTaken,
    CurrencyIncreaseOutcome,
};
use super::container::cyuanbao::CYuanBao;
use super::gameeffectjournal::{GameEffect, GameEffectJournal};
use super::goods::cbattlefairyproperty::BattleFairyCompose;
use super::goods::cgoods::CGoods;
use super::goods::cgoodsbaseproperties::{
    EQUIP_PLACE_HEADGEAR, GAP_AGILITY_CORRECTION, GAP_ANIMA_BIND, GAP_ARMOR_CORRECTION, GAP_ATTACK_AVOID,
    GAP_ATTACK_SPEED_CORRECTION, GAP_BF_ABRAVE_ADDON, GAP_BF_AGILITY, GAP_BF_AGILITY_ADDON,
    GAP_BF_AGILITY_BASE, GAP_BF_AGILITY_POTENTIAL, GAP_BF_ATTACK, GAP_BF_ATTACK_ADDON,
    GAP_BF_ATTACK_BASE, GAP_BF_ATTACK_POTENTIAL, GAP_BF_BATTLE_FAIRY, GAP_BF_BLAST,
    GAP_BF_BLAST_ADDON, GAP_BF_BLAST_POTENTIAL, GAP_BF_BRAVE, GAP_BF_BRAVE_BASE,
    GAP_BF_BRAVE_POTENTIAL, GAP_BF_CUT_HURT_ADDON,
    GAP_BF_CUT_HURT_SCALE, GAP_BF_HP, GAP_BF_LEVEL, GAP_BF_LIFE_ADDON,
    GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_MP_ADDON,
    GAP_BF_PULLULATERATE, GAP_BF_SPRITE, GAP_BF_SPRITE_ADDON,
    GAP_BF_SPRITE_BASE, GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITUALISE_ADDON,
    GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE, GAP_BF_SPRITUALISM_POTENTIAL,
    GAP_BF_STRENGH, GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_BASE, GAP_BF_STRENGH_POTENTIAL,
    GAP_BLAST_ATTACK, GAP_BLAST_ELEMENT_ATTACK,
    GAP_BREAK_ARMOUR, GAP_BREAK_BOUND, GAP_BREAK_ELEMENT, GAP_BURDEN_UPPER_LIMIT_CORRECTION,
    GAP_CIQING_PROPERTY1, GAP_CIQING_PROPERTY2,
    GAP_CONSTITUTION_CORRECTION, GAP_DODGE_CORRECTION, GAP_ELEMENT_ATTACK_CORRECTION,
    GAP_ELEMENT_AVOID, GAP_ELEMENT_RESISTANCE_CORRECTION, GAP_FATAL_BLOW_RATE_CORRECTION,
    GAP_EQUIP_ACTIVE, GAP_EXCEPTION_STATE, GAP_FULL_MISS, GAP_FUMO_PROPERTY,
    GAP_GOODS_BIND,
    GAP_FAIRY_AGILITY, GAP_FAIRY_HP, GAP_FAIRY_STRENGTH, GAP_FAIRY_WAKAN,
    GAP_GOODS_EQUIMENT_FLASH, GAP_GOODS_LIFE_TYPE, GAP_GOODS_MAXIMUM_DURABILITY,
    GAP_GOODS_PACKAGE_EXTENTION,
    GAP_GOLD_POWER, GAP_HIT_RATE_CORRECTION, GAP_HP_RESTORE_SPEED_CORRECTION,
    GAP_HP_UPPER_LIMIT_CORRECTION,
    GAP_MAXIMUM_ATTACK_CORRECTION, GAP_MINIMUM_ATTACK_CORRECTION, GAP_MOUNT_LEVEL, GAP_MOUNT_TYPE,
    GAP_MP_RESTORE_SPEED_CORRECTION, GAP_MP_UPPER_LIMIT_CORRECTION, GAP_PARTICULAR_ATTRIBUTE,
    GAP_REQUIRE_GENDER, GAP_REQUIRE_OCCUPATION, GAP_ROLE_MINIMUM_AGILITY_LIMIT,
    GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
    GAP_ROLE_MINIMUM_STRENGTH_LIMIT, GAP_ROLE_MINIMUM_WAKAN_LIMIT,
    GAP_PUNCTURE, GAP_STIFFEN_PROBABILITY_CORRECTION, GAP_STRENGTH_CORRECTION,
    GAP_WAKAN_CORRECTION,
    GAP_WEAPON_CATEGORY, GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE,
};
use super::goods::cgoodsfactory::CGoodsFactory;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use super::listener::cgoodsparticularpropertylistener::GoodsParticularPropertyListener;
use super::moveshape::{
    CMoveShape, MoveShapeCommandBlock, MoveShapePositionFacts,
    MoveShapeSkill, SKILL_BASE_DEFENSE,
};
use nebokrai_zone::scripts::{
    CVariableList, GameVariableMutationOutcome, GameVariableSnapshotError,
};
use nebokrai_zone::skills::{
    BattleFairySkillProperty, EQUIPPED_SKILL_PROPERTIES, battle_fairy_skill_entry,
    battle_fairy_skill_id, battle_fairy_skill_level,
};
use nebokrai_zone::skills::battlefairysummon::{BattleFairyHeadgearOperation, BattleFairyWarSoul};
use nebokrai_zone::skills::battlefairygear::{
    self, BattleFairyGearAddons, BattleFairyGearHost, BattleFairyMoneyChange,
    battle_fairy_skill_property_key,
};
use super::serverregion::CServerRegion;
use super::shape::{
    CShape, ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapeView,
};
use super::skills::archery::ARCHERY_SKILL_ID;
use super::skills::baseattack::BASE_ATTACK_SKILL_ID;
use super::skills::basemagic::BASE_MAGIC_SKILL_ID;
use super::skills::skillfactory::{CSkillFactory, UNKNOWN_SKILL_ID};
use super::states::automaticrestore::AutomaticRestoreMutation;
use super::teamstate::CTeamState;
use crate::nets::netserver::message::GameServerAroundRuntime;
use crate::public::auctionnode::CGoodsNode;
use nebokrai_shared::values::CGuid;
use crate::public::taozhuangsetup::CTaoZhuangSetup;
use crate::setup::globesetup::{GlobePlayerPropertyCoefficients, GlobeSetupSnapshot};
use nebokrai_shared::resources::HitLevelEntry;
use nebokrai_shared::resources::CQuestSystem;
use nebokrai_zone::players::{clientsnapshot, gamesave};
use nebokrai_zone::players::clientsnapshot::{PlayerClientShapeParts, PlayerInitialClientParts};
use nebokrai_zone::players::gamesave::{
    PlayerGameSaveParts, PlayerOrganizingParts, PlayerOrganizingSnapshot,
    read_player_wire_u16, read_player_wire_u32,
};
use nebokrai_zone::quests::PlayerQuestProgress;
use nebokrai_zone::trade::auction::{
    AuctionBuyGate, AuctionListingGate, AuctionMoneyMoveCapacity, PlayerAuction,
    check_auction_money_move, legacy_ipv4_text,
};
use nebokrai_zone::trade::ctrader::{
    TradeSourceContainer, trade_rollback_merge_reversible,
};
use nebokrai_zone::trade::currency::{
    BalanceSetDirection, BankCurrencyContainer, GroundCurrencyContainer, balance_set_direction,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use tracing::trace;

const PLAYER_TYPE: i32 = 400;
const LEGACY_COMBAT_MAXIMUM: u32 = i32::MAX as u32;
const CONTRIBUTION_MINIMUM: i32 = -2_000_000_000;
const CONTRIBUTION_MAXIMUM: i32 = 2_000_000_000;
const BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE: u32 = 0x0b_f80c;
const BATTLE_FAIRY_CONTAINER_EXTEND_ID: u32 = 0x0c;
const SKILL_EFFECT_MESSAGE_TYPE: u32 = 0x0b_fe01;
const SKILL_REJECT_REASON: u32 = 0;
const SKILL_REJECT_WAR_SOUL_REASON: u32 = 4;
const SKILL_REJECT_CODE: u8 = 0x0c;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyObjectMoveOperation {
    Delete,
    New,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyObjectMove {
    pub(crate) operation: BattleFairyObjectMoveOperation,
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) position: u32,
    pub(crate) amount: u32,
    pub(crate) old_client_payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyAuditLog {
    pub(crate) string_id: &'static str,
    pub(crate) account: Vec<u8>,
    pub(crate) goods_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    FetchPowerChanged {
        message_type: u32,
        player_id: i32,
        subject_id: i32,
        property_name: &'static str,
        value: u32,
    },
    ObjectMove(BattleFairyObjectMove),
    SkillAdded(BattleFairySkillAdded),
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
    Audit(BattleFairyAuditLog),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyCombineOutcome {
    FeatureDisabled,
    Rejected,
    InsufficientFetchPower,
    InputRemovalStopped,
    Failed,
    CreationFailed,
    CreationRejected,
    Created,
}

#[must_use = "combine report содержит последовательность адресных packet/log effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyCombineReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyCombineOutcome,
    pub(crate) effects: GameEffectJournal,
}

// Порция №7a: типы, wire-константы кадров и правила war-soul перенесены
// буквально в Zone `skills/battlefairysummon.rs`; report/plan-структуры с
// журнальным полем остаются здесь, потому что журнал — прежний
// `GameEffectJournal`.
pub(crate) use nebokrai_zone::skills::battlefairysummon::{
    BattleFairyDeathOutcome, BattleFairyFollowEffect, BattleFairyFollowOutcome,
    BattleFairySummonEffect, BattleFairySummonOutcome, BattleFairyWarSoulAction,
    BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WarSoulHitOutcome {
    pub(crate) broken: bool,
    pub(crate) broadcast_previous_status: bool,
    pub(crate) update: super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate,
}

#[must_use = "summon report хранит точный порядок адресных broadcast и property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySummonReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairySummonOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) effects: GameEffectJournal,
}

#[must_use = "план следования содержит пространственное действие и обязательную рассылку движения"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyFollowPlan {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyFollowOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentRemoveRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    pub(crate) player_goods_package_extension: Option<u32>,
    pub(crate) active_war_soul_blocks_headgear: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentRemoveEffect {
    WarSoulStatusAround {
        message_type: u32,
        player_id: i32,
        values: [i32; 2],
    },
    WarSoulSkillDetached {
        skill_id: u32,
    },
    SkillRemoved(BattleFairySkillRemoved),
    PropertiesChangedWithoutRemovedSlot {
        column: EquipmentColumn,
        combat_properties: PlayerCombatProperties,
        ci_qing_result_values: BTreeMap<u32, u32>,
    },
    VitalsClamped {
        previous_health: u32,
        current_health: u32,
        previous_mana: u32,
        current_mana: u32,
    },
    AroundUpdate(EquipmentAroundUpdate),
}

#[must_use = "equipment remove report сохраняет container ownership и player/network tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentRemoveReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentRemoveOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddRuntimeFacts {
    pub(crate) pack_add_enabled: bool,
    pub(crate) now: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerEquipmentAddEffect {
    WarSoulSkillAttached {
        skill_id: u32,
        level: i32,
    },
    SkillAdded(BattleFairySkillAdded),
    PropertiesChanged {
        combat_properties: PlayerCombatProperties,
        ci_qing_result_values: BTreeMap<u32, u32>,
    },
    AroundUpdate(EquipmentAroundUpdate),
    PackageExtensionLogged {
        category: &'static str,
        string_id: &'static str,
        expanded_package_num: u32,
    },
}

#[must_use = "equipment add report сохраняет исход частичных изменений контейнера"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentAddOutcome,
}

// Порция №7b: типы эффектов/исходов BF gear/property/upgrade/reset перенесены
// буквально в Zone `skills/battlefairygear.rs`; zone-перечисления
// параметризованы связанными типами hub-шва, подстановка прежних типов —
// alias-ами без изменения имён/полей (потребители без правок). Report-
// структуры с журнальным полем остаются здесь, потому что журнал — прежний
// `GameEffectJournal`.
pub(crate) type BattleFairyEquipmentMutationOutcome =
    battlefairygear::BattleFairyEquipmentMutationOutcome<
        BattleFairyContainerAddOutcome,
        VolumeGoodsRemoveOutcome,
    >;
pub(crate) type BattleFairyEquipmentMutationEffect =
    battlefairygear::BattleFairyEquipmentMutationEffect<BattleFairyDefaultGoodsUpdate>;
pub(crate) type BattleFairyPotentialAllocationEffect =
    battlefairygear::BattleFairyPotentialAllocationEffect<BattleFairyDefaultGoodsUpdate>;
pub(crate) type BattleFairyUpgradeEffect = battlefairygear::BattleFairyUpgradeEffect<
    BattleFairyDefaultGoodsUpdate,
    CurrencyDecreaseOutcome,
    BattleFairyUpgradeConsumedGem,
    VolumeGoodsRemoveOutcome,
>;
pub(crate) type BattleFairyPotentialResetEffect = battlefairygear::BattleFairyPotentialResetEffect<
    BattleFairyDefaultGoodsUpdate,
    VolumeGoodsRemoveOutcome,
>;
pub(crate) type BattleFairySkillResetEffect = battlefairygear::BattleFairySkillResetEffect<
    BattleFairyDefaultGoodsUpdate,
    VolumeGoodsRemoveOutcome,
>;

pub(crate) use nebokrai_zone::skills::battlefairygear::{
    BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE, BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
    BattleFairyPotentialAllocationOutcome, BattleFairyPotentialResetOutcome, BattleFairySkillAdded,
    BattleFairySkillRemoved, BattleFairySkillResetOutcome, BattleFairyUpgradeGoodsSnapshot,
    BattleFairyUpgradeLogGates, BattleFairyUpgradeOutcome, BattleFairyUpgradePlayerSnapshot,
};

#[must_use = "equipment report сохраняет container ownership и ранние property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyEquipmentMutationReport {
    pub(crate) player_id: i32,
    pub(crate) cell: Option<BattleFairyCell>,
    pub(crate) property_applied: bool,
    pub(crate) outcome: BattleFairyEquipmentMutationOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[must_use = "allocation report сохраняет ordered player и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialAllocationReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialAllocationOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[must_use = "upgrade report содержит wallet, ownership и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyUpgradeOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[must_use = "reset report содержит packet ownership и player/network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialResetReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialResetOutcome,
    pub(crate) effects: GameEffectJournal,
}

#[must_use = "skill reset report содержит packet, skill-state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillResetReport {
    pub(crate) player_id: i32,
    pub(crate) position: i32,
    pub(crate) outcome: BattleFairySkillResetOutcome,
    pub(crate) effects: GameEffectJournal,
}

pub(crate) use nebokrai_zone::skills::{
    BattleFairySkillDispatch, BattleFairySkillRequest, BattleFairySkillRequestFacts,
    PlayerSkillDispatch, PlayerSkillRequest, PlayerSkillRequestFacts, SkillTarget, SkillTargetForm,
};

/// Codec-типы GameSave и wires свойств игрока — Zone `players::gamesave`;
/// здесь их re-export совместимости для держателя hub-а и его callers.
pub(crate) use nebokrai_zone::players::gamesave::{
    BASE_ATTACK_SPEED_OFFSET, BASE_CCH_OFFSET, BASE_CONSTITUTION_OFFSET, BASE_DEFENSE_OFFSET,
    BASE_DEXTERITY_OFFSET, BASE_DODGE_OFFSET, BASE_ELEMENT_RESISTANCE_OFFSET, BASE_HEALTH_OFFSET,
    BASE_HIT_OFFSET, BASE_HP_RECOVERY_OFFSET, BASE_INTELLIGENCE_OFFSET, BASE_KILL_COUNT_OFFSET,
    BASE_MANA_OFFSET, BASE_MAXIMUM_ATTACK_OFFSET, BASE_MAXIMUM_HP_OFFSET, BASE_MAXIMUM_MP_OFFSET,
    BASE_MINIMUM_ATTACK_OFFSET, BASE_MP_RECOVERY_OFFSET, BASE_PK_COUNT_OFFSET, BASE_STRENGTH_OFFSET,
    LeiTingEnableFlags, PLAYER_BASE_PROPERTY_WIRE_SIZE, PLAYER_COMBAT_PROPERTY_WIRE_SIZE,
    PlayerBaseProperties, PlayerFriend, PlayerGameSaveCodecError, PlayerLeiTingDecodeBlock,
    PlayerLeiTingThing, PlayerUncreatedCarriage, PlayerUncreatedPet,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorSnapshot {
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) appellation_id: u32,
    pub(crate) days_eliminate: u32,
    pub(crate) weeks_eliminate: u32,
    pub(crate) months_eliminate: u32,
    pub(crate) total_eliminate: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLeiTingThingCountOutcome {
    Missing,
    Rejected {
        current: u16,
        requested: i32,
        maximum: u16,
    },
    Updated {
        previous_count: u16,
        current_count: u16,
        previous_energy: u32,
        current_energy: u32,
        daily_count_incremented: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLoginGoodsLocation {
    Equipment,
    Packet,
    Hand,
    Auction,
    Depot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerFriendAddOutcome {
    Added,
    AlreadyPresent,
    LimitReached,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerDeathGoodsCandidate {
    pub(crate) location: PlayerGoodsAiLocation,
    pub(crate) goods_id: CGuid,
    pub(crate) amount: u32,
    pub(crate) price: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) particular_on_death: bool,
    pub(crate) table_drop_allowed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerCriminalStateEndReason {
    Timeout,
    PkThresholdExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCriminalStateEnd {
    pub(crate) player_id: i32,
    pub(crate) previous_timestamp_ms: u32,
    pub(crate) checked_at_ms: u32,
    pub(crate) pk_count: u16,
    pub(crate) reason: PlayerCriminalStateEndReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurdererSignDecrease {
    pub(crate) player_id: i32,
    pub(crate) pk_count: u16,
    pub(crate) kill_count: u32,
    pub(crate) checked_at_ms: u32,
    pub(crate) next_timestamp_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLostDelayStarted {
    pub(crate) player_id: i32,
    pub(crate) fight_state_count: i32,
    pub(crate) timestamp_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExitSilenceUpdate {
    pub(crate) previous_minutes: i32,
    pub(crate) previous_timestamp_minutes: u32,
    pub(crate) sampled_minutes: [Option<u32>; 3],
    pub(crate) remaining_minutes: i32,
    pub(crate) timestamp_minutes: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerFightStateTransition {
    pub(crate) player_id: i32,
    pub(crate) previous_count: i32,
    pub(crate) current_count: i32,
    pub(crate) entered_peace: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRemoteSkillMutation {
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerRemoteLevelMutation {
    pub(crate) player_id: i32,
    pub(crate) faction_id: i32,
    pub(crate) previous_level: u8,
    pub(crate) level: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HotkeyHandTransferOutcome {
    MissingHandGoods,
    NotConsumable,
    UnsupportedSource,
    Moved,
    RolledBack,
    GarbageCollected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyHandOwnershipEvent {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) position: Option<u32>,
    pub(crate) amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "hand transfer report сохраняет ownership, fallback add и object-move исход"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HotkeyHandTransferReport {
    pub(crate) source_container_extend_id: u32,
    pub(crate) source_position: u32,
    pub(crate) goods: Option<ShapeIdentity>,
    pub(crate) hand_removal: Option<HotkeyHandOwnershipEvent>,
    pub(crate) packet_adds: Vec<PlayerPacketAddOutcome>,
    pub(crate) currency_adds: Vec<CurrencyGoodsAddOutcome>,
    pub(crate) hand_rollback: Option<AmountLimitGoodsAdded>,
    pub(crate) outcome: HotkeyHandTransferOutcome,
}

#[must_use = "packet add сохраняет исход передачи предмета контейнеру"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPacketAddOutcome {
    pub(crate) outcome: VolumeGoodsAddOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorEliminateMutation {
    pub(crate) player_id: i32,
    pub(crate) previous: [u32; 4],
    pub(crate) current: [u32; 4],
}

// Снимок боевых свойств игрока — формульный владелец Zone
// `combat/fightdefense`; CPlayer остаётся производителем (`combat_properties()`),
// переходник сохраняет прежние пути потребителей.
pub(crate) use nebokrai_zone::combat::PlayerCombatProperties;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPropertyRecompute {
    pub(crate) properties: PlayerCombatProperties,
    pub(crate) ci_qing_result_values: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaoZhuangSetEvaluation {
    pub(crate) set_id: u32,
    pub(crate) collected_all: bool,
    pub(crate) completion_script: Vec<u8>,
}

/// Exact `GetPlayerAllProperties` diagnostic projection. Числа хранят raw
/// DWORD vararg bits: конкретный `%d`/`%u` шаблона определяет их signed view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAllPropertiesDiagnosticSnapshot {
    pub(crate) name: Vec<u8>,
    pub(crate) summary_words: [u32; 15],
    pub(crate) base_combat_words: [u32; 15],
    pub(crate) current_combat_words: [u32; 20],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExpendableEffect {
    pub(crate) property_type: i32,
    pub(crate) value: i32,
    pub(crate) start_time_ms: u32,
    pub(crate) effect_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerStatAllocationState {
    pub(crate) sex: u8,
    pub(crate) occupation: u8,
    pub(crate) remain_point: u16,
    pub(crate) base_maximum_hp: u32,
    pub(crate) base_maximum_mp: u32,
    pub(crate) base_strength: u32,
    pub(crate) base_dexterity: u32,
    pub(crate) base_constitution: u32,
    pub(crate) base_intelligence: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerStatAllocationMutation {
    pub(crate) player_id: i32,
    pub(crate) selector: u8,
    pub(crate) stat_changed: bool,
    pub(crate) previous: PlayerStatAllocationState,
    pub(crate) current: PlayerStatAllocationState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerPkPermissions {
    pub(crate) player: bool,
    pub(crate) teammate: bool,
    pub(crate) guild_member: bool,
    pub(crate) criminal: bool,
    pub(crate) country: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPkPermissionMutation {
    pub(crate) player_id: i32,
    pub(crate) selector: i8,
    pub(crate) requested: bool,
    pub(crate) recognized: bool,
    pub(crate) changed: bool,
    pub(crate) previous: PlayerPkPermissions,
    pub(crate) current: PlayerPkPermissions,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PlayerProgress {
    #[default]
    None,
    Banking,
    Trading,
    Shopping,
    OpenStall,
    Increment,
    Upgrade,
    Synthesis,
    Mailing,
    DaKong,
    Compose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsSessionPlayerRelease {
    pub(crate) previous_progress: PlayerProgress,
    pub(crate) previous_moveable_count: i32,
    pub(crate) resulting_moveable_count: i32,
    pub(crate) moveable: bool,
}

pub(crate) use nebokrai_zone::trade::auction::AuctionSelfGoodsRefresh;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingPacketConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) position: u32,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingPacketAddition {
    pub(crate) player_id: i32,
    pub(crate) source: super::shape::ShapeIdentity,
    pub(crate) position: Option<u32>,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) old_client_payload: Option<Vec<u8>>,
    pub(crate) resulting_amount: Option<u32>,
}

#[must_use = "изменение YuanBao содержит обязательный container/client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerYuanBaoChange {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: PlayerYuanBaoChangeOutcome,
}

#[must_use = "списание денег содержит wallet outcome для обязательного client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMoneyDecrease {
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: CurrencyDecreaseOutcome,
}

#[must_use = "результат bank transfer определяет ownership и lock/add side effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerBankCurrencyAddOutcome {
    Wallet(CurrencyGoodsAddOutcome),
    Bank(BankGoodsAddOutcome),
    AuctionWallet(CurrencyGoodsAddOutcome),
}

#[must_use = "изменение аукционных денег содержит wallet outcome для client effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAuctionMoneyChange {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) current: u32,
    pub(crate) outcome: PlayerAuctionMoneyChangeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerAuctionMoneyChangeOutcome {
    Unchanged,
    Increased(CurrencyIncreaseOutcome),
    Decreased(CurrencyDecreaseOutcome),
}

#[must_use = "возврат с аукциона содержит container, bind и ownership outcome"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAuctionGoodsReturn {
    pub(crate) player_id: i32,
    pub(crate) position: u32,
    pub(crate) source: ShapeIdentity,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) resulting_goods: Option<ShapeIdentity>,
    pub(crate) resulting_amount: Option<u32>,
    pub(crate) bind_stored: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerYuanBaoChangeOutcome {
    Unchanged,
    Increased(CurrencyIncreaseOutcome),
    Decreased(CurrencyDecreaseOutcome),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingContainerAddition {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) position: u32,
    pub(crate) source: super::shape::ShapeIdentity,
    pub(crate) outcome: VolumeGoodsAddOutcome,
    pub(crate) old_client_payload: Option<Vec<u8>>,
    pub(crate) resulting_amount: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingContainerConsumption {
    pub(crate) player_id: i32,
    pub(crate) container_extend_id: u32,
    pub(crate) position: u32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<VolumeGoodsRemoveOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CiQingHandConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<AmountLimitGoodsRemoved>,
}

#[must_use = "уничтожение hand goods содержит ownership и listener-эффекты удаления"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoodsDestroyHandConsumption {
    pub(crate) player_id: i32,
    pub(crate) goods: super::shape::ShapeIdentity,
    pub(crate) previous_amount: u32,
    pub(crate) removed_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) removal: Option<AmountLimitGoodsRemoved>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementSelectionBlock {
    MissingGoods,
    UnsupportedSourceContainer,
    GoodsIdentityMismatch,
    GoodsAmountMismatch,
    StackableGoods,
    MissingBaseProperties,
    Shadow(ShadowRecordBlock),
}

#[must_use = "selection report связывает source container и AddShadow effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementSelectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) shadow: AmountShadowAdded,
    pub(crate) previous_last_operated: (u32, u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementDeselectionBlock {
    MissingShadow,
    GoodsIdentityMismatch,
    GoodsAmountMismatch,
    MissingSourceGoods,
}

#[must_use = "deselection report сохраняет original source и RemoveShadow effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnhancementDeselectionReport {
    pub(crate) goods: ShapeIdentity,
    pub(crate) source: PreviousContainer,
    pub(crate) removed: super::container::cgoodsshadowcontainer::ShadowRemovedReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerTalkChannel {
    Normal,
    Area,
    Country,
    World,
    Private,
    Team,
    Union,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CPlayer {
    move_shape: CMoveShape,
    player_ai: CPlayerAI,
    figure: ShapeFigure,
    faction_id: i32,
    faction_logo_id: i32,
    faction_level: u16,
    faction_experience: i32,
    faction_force: i32,
    faction_contribute: u32,
    faction_master_id: i32,
    faction_name: Vec<u8>,
    faction_title: Vec<u8>,
    enemy_factions: BTreeSet<i32>,
    city_war_enemy_factions: BTreeSet<i32>,
    faction_owned_regions: Vec<[u8; 8]>,
    union_id: i32,
    union_master_id: i32,
    team_id: i32,
    team_captain: bool,
    country: u8,
    server_region_id: Option<i32>,
    in_changing_server: bool,
    in_changing_region: bool,
    last_enter_region_tick_ms: u32,
    entered_region: bool,
    state_before_server_region_change: u16,
    current_progress: PlayerProgress,
    personal_shop_session_id: i32,
    personal_shop_plug_id: i32,
    war_soul_state: u32,
    war_soul_point: WarSoulPoint,
    war_soul_visual_x_bits: u32,
    war_soul_visual_y_bits: u32,
    current_ticket: u32,
    goods_ai_tree: BTreeMap<u32, BTreeSet<CGuid>>,
    goods_ai_delete_queue: VecDeque<BTreeSet<CGuid>>,
    flash_previous: [u32; 17],
    flash_current: [u32; 17],
    flash_changed: bool,
    battle_fairy_summoned: bool,
    recreate_carriage: bool,
    active_carriage_id: i32,
    create_faction_operator: bool,
    apply_join_faction_operator: bool,
    faction_declare_operator: bool,
    active_pet_count: u32,
    attempt_appellation_id: u32,
    realm_appellation_skill_id: u32,
    realm_appellation_skill_level: i32,
    heart_request_sent: i32,
    heart_received: bool,
    friends: Vec<PlayerFriend>,
    quest_progress: PlayerQuestProgress,
    lei_ting_things: VecDeque<PlayerLeiTingThing>,
    uncreated_pets: Vec<PlayerUncreatedPet>,
    uncreated_carriage: PlayerUncreatedCarriage,
    login: bool,
    session_id: Vec<u8>,
    title: Vec<u8>,
    base_property_wire: [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE],
    jjc_data: [u8; 0x10],
    jjc_pk_state: bool,
    fight_state_count: i32,
    auto_protected: bool,
    lost_time_stamp_ms: u32,
    continuous_kill_amount: u32,
    continuous_kill_timestamp_ms: u32,
    base_properties: PlayerBaseProperties,
    combat_properties: PlayerCombatProperties,
    combat_property_wire: [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    expendable_effects: BTreeMap<i32, PlayerExpendableEffect>,
    last_skill_item_use_ms: BTreeMap<u32, u32>,
    ci_qing_open: bool,
    ci_qing_list: BTreeSet<u32>,
    ci_qing_add_values: BTreeMap<u32, u32>,
    ci_qing_tao_zhuang_add_values: BTreeMap<u32, u32>,
    tao_zhuang_id: u32,
    equipment_changed: bool,
    tao_zhuang_setup_pending: bool,
    tao_zhuang_items: BTreeMap<u32, u32>,
    tao_zhuang_original_names: BTreeSet<Vec<u8>>,
    tao_zhuang_properties: BTreeMap<u32, u32>,
    ci_qing_tao_zhuang_properties: BTreeMap<u32, u32>,
    tao_zhuang_skills: BTreeMap<u32, u32>,
    contend_state: bool,
    emotion_index: i32,
    emotion_timestamp_ms: u32,
    city_war_died_state: bool,
    city_war_died_state_time_ms: i32,
    died_state_start_time_ms: u32,
    criminal_state_timestamp_ms: u32,
    murderer_time_stamp_ms: u32,
    ping_time: i32,
    last_ping_time_ms: u32,
    contribution: i32,
    silence_minutes: i32,
    silence_timestamp_minutes: u32,
    normal_talk_timestamp_ms: u32,
    area_talk_timestamp_ms: u32,
    world_talk_timestamp_ms: u32,
    country_talk_timestamp_ms: u32,
    private_talk_timestamp_ms: u32,
    team_talk_timestamp_ms: u32,
    union_talk_timestamp_ms: u32,
    money: u32,
    client_ip: u32,
    account: Vec<u8>,
    depot_password: Vec<u8>,
    last_container_script: Vec<u8>,
    variable_list: CVariableList,
    bank: CBank,
    depot: CDepot,
    hand: CAmountLimitGoodsContainer,
    enhancement: CAmountLimitGoodsShadowContainer,
    last_operated_container: u32,
    last_operated_goods_position: u32,
    packet: CVolumeLimitGoodsContainer,
    wallet: CWallet,
    yuan_bao: CYuanBao,
    ji_fen: CJiFen,
    equipment: CEquipmentContainer,
    auction_listing: CVolumeLimitGoodsContainer,
    auction_goods: CVolumeLimitGoodsContainer,
    auction_wallet: CWallet,
    auction: PlayerAuction<CGoodsNode>,
    ci_qing: CVolumeLimitGoodsContainer,
    ci_qing_compose: CVolumeLimitGoodsContainer,
    fairy_container: CFairyContainer,
    battle_fairy_container: CBattleFairyContainer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerGoodsAiLocation {
    pub(crate) extend_id: i32,
    pub(crate) position: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerParticularGoodsDrop {
    pub(crate) location: PlayerGoodsAiLocation,
    pub(crate) goods_id: CGuid,
    pub(crate) amount: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerGoodsAiDeletion {
    pub(crate) location: PlayerGoodsAiLocation,
    pub(crate) goods: CGoods,
    pub(crate) previous_amount: u32,
    pub(crate) removed_amount: u32,
    pub(crate) remaining_amount: u32,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerReliveMutation {
    pub(crate) player_id: i32,
    pub(crate) previous_x: i32,
    pub(crate) previous_y: i32,
    pub(crate) direction: i32,
    pub(crate) health: u32,
    pub(crate) mana: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerReliveOwnedPrelude {
    pub(crate) cleared_uncreated_pets: usize,
    pub(crate) cleared_uncreated_carriage: bool,
    pub(crate) previous_moveable_count: i32,
    pub(crate) resulting_moveable_count: i32,
    pub(crate) moveable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerContinuousKillUpdate {
    pub(crate) amount: u32,
    pub(crate) new_top_log: Option<u16>,
    pub(crate) bonus_experience: u32,
}

fn apply_equipment_goods_properties(
    properties: &mut PlayerCombatProperties,
    goods: &CGoods,
    factory: &CGoodsFactory,
    coefficients: GlobePlayerPropertyCoefficients,
    occupation: usize,
    include_fairy_properties: bool,
    active_level: Option<u8>,
) {
    fn add_u32(target: &mut u32, delta: i32) {
        *target = (i64::from(*target) + i64::from(delta)).clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn add_mount_u32(target: &mut u32, delta: i32) {
        let value = target.wrapping_add(delta as u32);
        *target = if delta < 0 && (value as i32) < 0 {
            0
        } else {
            value.min(i32::MAX as u32)
        };
    }
    fn add_mount_u16(target: &mut u16, delta: i32) {
        let value = i32::from(*target).wrapping_add(delta);
        *target = if delta < 0 && value < 0 { 0 } else { value as u16 };
    }
    // `AddPreItemToPlayer`, `MountFuMoProperty` и `ActiveEquip` перед FISTP
    // выставляют x87 RC=truncate; производные поля усекают полную сумму.
    fn scaled_delta(value: i32, coefficient: f32) -> i32 {
        ((value as f32) * coefficient).trunc() as i32
    }
    fn add_derived_u32(target: &mut u32, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i64;
        *target = value.clamp(0, i64::from(i32::MAX)) as u32;
    }
    fn add_derived_i32(target: &mut i32, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i32;
        *target = if delta < 0 && value < 0 { 0 } else { value };
    }
    fn add_derived_u16(target: &mut u16, delta: i32, coefficient: f32) {
        let value = (*target as f32 + delta as f32 * coefficient).trunc() as i32;
        *target = if value < 0 { 0 } else { value as u16 };
    }
    fn active_u32(current: u32, addition: i32, scale: f64) -> u32 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64;
        value.clamp(0, i64::from(i32::MAX)) as u32
    }
    fn active_signed(current: i32, addition: i32, scale: f64) -> i32 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64 as u32;
        if (value as i32) < 0 { 0 } else { value as i32 }
    }
    fn active_u16(current: u16, addition: i32, scale: f64) -> u16 {
        let value = (f64::from(current) + f64::from(addition) * scale).trunc() as i64;
        if value < 0 { 0 } else { value as u16 }
    }

    fn apply_active_equip(
        properties: &mut PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
        selector: i32,
        percentage: i32,
    ) {
        let scale = f64::from(percentage) * 0.01_f64;
        match selector {
            GAP_MINIMUM_ATTACK_CORRECTION
            | GAP_MAXIMUM_ATTACK_CORRECTION
            | GAP_ELEMENT_ATTACK_CORRECTION => {
                properties.minimum_attack = active_u32(
                    properties.minimum_attack,
                    goods.addon_property_value(factory, GAP_MINIMUM_ATTACK_CORRECTION, 1),
                    scale,
                );
                properties.maximum_attack = active_u32(
                    properties.maximum_attack,
                    goods.addon_property_value(factory, GAP_MAXIMUM_ATTACK_CORRECTION, 1),
                    scale,
                );
                properties.element_modify = active_signed(
                    properties.element_modify,
                    goods.addon_property_value(factory, GAP_ELEMENT_ATTACK_CORRECTION, 1),
                    scale,
                );
            }
            GAP_ARMOR_CORRECTION | GAP_ELEMENT_RESISTANCE_CORRECTION => {
                properties.defense = active_u32(
                    properties.defense,
                    goods.addon_property_value(factory, GAP_ARMOR_CORRECTION, 1),
                    scale,
                );
                properties.element_resistance = active_u32(
                    properties.element_resistance,
                    goods.addon_property_value(factory, GAP_ELEMENT_RESISTANCE_CORRECTION, 1),
                    scale,
                );
            }
            GAP_HP_UPPER_LIMIT_CORRECTION => {
                properties.maximum_hp = active_u32(
                    properties.maximum_hp,
                    goods.addon_property_value(factory, GAP_HP_UPPER_LIMIT_CORRECTION, 1),
                    scale,
                );
            }
            GAP_ATTACK_AVOID | GAP_ELEMENT_AVOID => {
                properties.attack_avoid = active_u16(
                    properties.attack_avoid,
                    goods.addon_property_value(factory, GAP_ATTACK_AVOID, 1),
                    scale,
                );
                properties.element_avoid = active_u16(
                    properties.element_avoid,
                    goods.addon_property_value(factory, GAP_ELEMENT_AVOID, 1),
                    scale,
                );
            }
            _ => {}
        }
    }

    let enabled = goods.enabled_addon_properties(factory);
    // Native equipment/ride owners вызывают positive, затем negative pass:
    // первый pass принимает неотрицательные addon-ы, второй — отрицательные.
    for positive_pass in [true, false] {
        for &stored_type in &enabled {
            if stored_type == GAP_EQUIP_ACTIVE {
                if positive_pass
                    && goods.has_addon_property_values(factory, GAP_ANIMA_BIND)
                    && goods.addon_property_value(factory, GAP_ANIMA_BIND, 1) != 0
                    && active_level.is_some_and(|level| {
                        goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
                            <= i32::from(level)
                    })
                {
                    apply_active_equip(
                        properties,
                        goods,
                        factory,
                        goods.addon_property_value(factory, GAP_EQUIP_ACTIVE, 1),
                        goods.addon_property_value(factory, GAP_EQUIP_ACTIVE, 2),
                    );
                }
                continue;
            }
            let fumo = stored_type == GAP_FUMO_PROPERTY;
            let (property_type, delta) = if fumo {
                (
                    goods.addon_property_value(factory, stored_type, 1),
                    goods.addon_property_value(factory, stored_type, 2),
                )
            } else {
                (
                    stored_type,
                    goods.addon_property_value(factory, stored_type, 1),
                )
            };
            // GAP_FUMO_PROPERTY native-ветка существует только в первом
            // `MountEquipRide(true)` pass и уже внутри принимает signed delta.
            if (fumo && !positive_pass) || (!fumo && (delta >= 0) != positive_pass) {
                continue;
            }
            // MountEquip0x00442610 и MountEquipRide0x0043C5E0 складывают
            // low32 до signed negative gate. Отдельный MountFuMoProperty
            // здесь сохраняет прежний адаптер, не наследуя этот контракт.
            let add_direct_u32 = if fumo { add_u32 } else { add_mount_u32 };
            let add_direct_u16 = if fumo { add_u16 } else { add_mount_u16 };
            match property_type {
                GAP_MINIMUM_ATTACK_CORRECTION => add_direct_u32(&mut properties.minimum_attack, delta),
                GAP_MAXIMUM_ATTACK_CORRECTION => add_direct_u32(&mut properties.maximum_attack, delta),
                GAP_ELEMENT_ATTACK_CORRECTION => {
                    let value = properties.element_modify.wrapping_add(delta);
                    properties.element_modify = if delta < 0 && value < 0 { 0 } else { value };
                }
                GAP_ARMOR_CORRECTION => add_direct_u32(&mut properties.defense, delta),
                GAP_ATTACK_SPEED_CORRECTION => {
                    if fumo {
                        add_u16(&mut properties.attack_speed, delta);
                    } else {
                        properties.attack_speed = properties.attack_speed.wrapping_add(delta as u16);
                    }
                }
                GAP_HIT_RATE_CORRECTION => add_direct_u16(&mut properties.hit, delta),
                GAP_FATAL_BLOW_RATE_CORRECTION => add_direct_u16(&mut properties.cch, delta),
                GAP_DODGE_CORRECTION => add_direct_u16(&mut properties.dodge, delta),
                GAP_ELEMENT_RESISTANCE_CORRECTION => {
                    add_direct_u32(&mut properties.element_resistance, delta)
                }
                GAP_HP_RESTORE_SPEED_CORRECTION => add_direct_u16(&mut properties.hp_recovery, delta),
                GAP_MP_RESTORE_SPEED_CORRECTION => add_direct_u16(&mut properties.mp_recovery, delta),
                GAP_STRENGTH_CORRECTION => {
                    add_direct_u32(&mut properties.strength, delta);
                    add_derived_u32(
                        &mut properties.maximum_attack,
                        delta,
                        coefficients.str_to_max_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.burden,
                        delta,
                        coefficients.str_to_burden[occupation],
                    );
                }
                GAP_AGILITY_CORRECTION => {
                    add_direct_u32(&mut properties.dexterity, delta);
                    add_derived_u32(
                        &mut properties.minimum_attack,
                        delta,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.reank,
                        delta,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                GAP_CONSTITUTION_CORRECTION => {
                    add_direct_u32(&mut properties.constitution, delta);
                    add_derived_u32(
                        &mut properties.maximum_hp,
                        delta,
                        coefficients.con_to_max_hp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.defense,
                        delta,
                        coefficients.con_to_defense[occupation],
                    );
                }
                GAP_WAKAN_CORRECTION => {
                    add_direct_u32(&mut properties.intelligence, delta);
                    add_derived_i32(
                        &mut properties.element_modify,
                        delta,
                        coefficients.int_to_element[occupation],
                    );
                    add_derived_u32(
                        &mut properties.maximum_mp,
                        delta,
                        coefficients.int_to_max_mp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.element_resistance,
                        delta,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                GAP_HP_UPPER_LIMIT_CORRECTION => add_direct_u32(&mut properties.maximum_hp, delta),
                GAP_MP_UPPER_LIMIT_CORRECTION => add_direct_u32(&mut properties.maximum_mp, delta),
                GAP_STIFFEN_PROBABILITY_CORRECTION => add_direct_u16(&mut properties.reank, delta),
                GAP_BURDEN_UPPER_LIMIT_CORRECTION => add_direct_u16(&mut properties.burden, delta),
                GAP_ATTACK_AVOID => add_direct_u16(&mut properties.attack_avoid, delta),
                GAP_ELEMENT_AVOID => add_direct_u16(&mut properties.element_avoid, delta),
                GAP_FULL_MISS => add_direct_u16(&mut properties.full_miss, delta),
                GAP_BLAST_ATTACK => add_direct_u16(&mut properties.blast_attack, delta),
                GAP_BLAST_ELEMENT_ATTACK => {
                    // Legacy case 96 берёт base из wBlastAttack, не из target.
                    let mut value = properties.blast_attack;
                    add_direct_u16(&mut value, delta);
                    properties.blast_element_attack = value;
                }
                GAP_FAIRY_STRENGTH if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_strength_to_player);
                    add_u32(&mut properties.strength, player_delta);
                    add_derived_u32(
                        &mut properties.maximum_attack,
                        player_delta,
                        coefficients.str_to_max_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.burden,
                        player_delta,
                        coefficients.str_to_burden[occupation],
                    );
                }
                GAP_FAIRY_AGILITY if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_agility_to_player);
                    add_u32(&mut properties.dexterity, player_delta);
                    add_derived_u32(
                        &mut properties.minimum_attack,
                        player_delta,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    add_derived_u16(
                        &mut properties.reank,
                        player_delta,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                GAP_FAIRY_WAKAN if include_fairy_properties => {
                    let player_delta = scaled_delta(delta, coefficients.fairy_wakan_to_player);
                    add_u32(&mut properties.intelligence, player_delta);
                    add_derived_i32(
                        &mut properties.element_modify,
                        player_delta,
                        coefficients.int_to_element[occupation],
                    );
                    add_derived_u32(
                        &mut properties.maximum_mp,
                        player_delta,
                        coefficients.int_to_max_mp[occupation],
                    );
                    add_derived_u32(
                        &mut properties.element_resistance,
                        player_delta,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                GAP_FAIRY_HP if include_fairy_properties => add_u32(
                    &mut properties.maximum_hp,
                    scaled_delta(delta, coefficients.fairy_hp_to_player),
                ),
                _ => {}
            }
        }
    }
}

impl CPlayer {
    /// Сбор заёмной persistence-проекции codec-владеемых частей hub-а для
    /// Zone `players::gamesave`; живёт только на время вызова codec-а.
    /// Порядок полей отражает persisted layout, сами значения — живые части.
    fn game_save_parts(&mut self) -> PlayerGameSaveParts<'_> {
        PlayerGameSaveParts {
            move_shape: &mut self.move_shape,
            base_property_wire: &mut self.base_property_wire,
            base_properties: &mut self.base_properties,
            combat_property_wire: &mut self.combat_property_wire,
            combat_properties: &mut self.combat_properties,
            battle_fairy_summoned: &mut self.battle_fairy_summoned,
            war_soul_state: &mut self.war_soul_state,
            realm_appellation_skill_id: &mut self.realm_appellation_skill_id,
            realm_appellation_skill_level: &mut self.realm_appellation_skill_level,
            auto_protected: &mut self.auto_protected,
            account: &mut self.account,
            title: &mut self.title,
            session_id: &mut self.session_id,
            team_id: &mut self.team_id,
            country: &mut self.country,
            contribution: &mut self.contribution,
            money: &mut self.money,
            depot_password: &mut self.depot_password,
            ci_qing_list: &mut self.ci_qing_list,
            friends: &mut self.friends,
            lei_ting_things: &mut self.lei_ting_things,
            quest_progress: &mut self.quest_progress,
            uncreated_pets: &mut self.uncreated_pets,
            uncreated_carriage: &mut self.uncreated_carriage,
            recreate_carriage: &mut self.recreate_carriage,
            login: &mut self.login,
            variable_list: &mut self.variable_list,
            silence_minutes: &mut self.silence_minutes,
            silence_timestamp_minutes: &mut self.silence_timestamp_minutes,
            murderer_time_stamp_ms: &mut self.murderer_time_stamp_ms,
            fight_state_count: &mut self.fight_state_count,
            city_war_died_state: &mut self.city_war_died_state,
            city_war_died_state_time_ms: &mut self.city_war_died_state_time_ms,
            died_state_start_time_ms: &mut self.died_state_start_time_ms,
            jjc_data: &mut self.jjc_data,
            jjc_pk_state: &mut self.jjc_pk_state,
            organizing: PlayerOrganizingParts {
                faction_id: &mut self.faction_id,
                faction_logo_id: &mut self.faction_logo_id,
                faction_level: &mut self.faction_level,
                faction_experience: &mut self.faction_experience,
                faction_force: &mut self.faction_force,
                faction_contribute: &mut self.faction_contribute,
                faction_master_id: &mut self.faction_master_id,
                faction_name: &mut self.faction_name,
                faction_title: &mut self.faction_title,
                union_id: &mut self.union_id,
                union_master_id: &mut self.union_master_id,
                enemy_factions: &mut self.enemy_factions,
                city_war_enemy_factions: &mut self.city_war_enemy_factions,
                faction_owned_regions: &mut self.faction_owned_regions,
            },
            hand: &mut self.hand,
            enhancement: &mut self.enhancement,
            packet: &mut self.packet,
            equipment: &mut self.equipment,
            wallet: &mut self.wallet,
            yuan_bao: &mut self.yuan_bao,
            ji_fen: &mut self.ji_fen,
            bank: &mut self.bank,
            depot: &mut self.depot,
            auction_goods: &mut self.auction_goods,
            auction_listing: &mut self.auction_listing,
            auction_wallet: &mut self.auction_wallet,
            fairy_container: &mut self.fairy_container,
            battle_fairy_container: &mut self.battle_fairy_container,
            ci_qing: &mut self.ci_qing,
            ci_qing_compose: &mut self.ci_qing_compose,
        }
    }

    /// Читающая проекция организационного блока для encode/клиентских снимков
    /// Zone `players` (тот же `0x7FE06` wire layout).
    fn organizing_snapshot(&self) -> PlayerOrganizingSnapshot<'_> {
        PlayerOrganizingSnapshot {
            faction_id: self.faction_id,
            faction_logo_id: self.faction_logo_id,
            faction_level: self.faction_level,
            faction_experience: self.faction_experience,
            faction_force: self.faction_force,
            faction_contribute: self.faction_contribute,
            faction_master_id: self.faction_master_id,
            faction_name: &self.faction_name,
            faction_title: &self.faction_title,
            union_id: self.union_id,
            union_master_id: self.union_master_id,
            enemy_factions: &self.enemy_factions,
            city_war_enemy_factions: &self.city_war_enemy_factions,
            faction_owned_regions: &self.faction_owned_regions,
        }
    }

    /// Собирает только достигнутый send-family state уже созданного игрока;
    /// identity другого object type отвергается до регистрации.
    pub(crate) fn from_send_state(
        move_shape: CMoveShape,
        figure: ShapeFigure,
        team_id: i32,
        country: u8,
        server_region_id: Option<i32>,
    ) -> Option<Self> {
        if move_shape.shape().identity().object_type != PLAYER_TYPE {
            return None;
        }
        let owner_id = move_shape.shape().identity().id;
        let realm_appellation_bonus = move_shape
            .skills()
            .filter(|skill| {
                super::skills::realmappellation::is_bonus_skill(skill.id())
                    && (1..=4).contains(&skill.level())
            })
            .min_by_key(|skill| skill.id())
            .map(|skill| (skill.id(), skill.level()));
        let mut player = Self {
            move_shape,
            player_ai: CPlayerAI::default(),
            figure,
            faction_id: 0,
            faction_logo_id: 0,
            faction_level: 0,
            faction_experience: 0,
            faction_force: 0,
            faction_contribute: 0,
            faction_master_id: 0,
            faction_name: Vec::new(),
            faction_title: Vec::new(),
            enemy_factions: BTreeSet::new(),
            city_war_enemy_factions: BTreeSet::new(),
            faction_owned_regions: Vec::new(),
            union_id: 0,
            union_master_id: 0,
            team_id,
            team_captain: false,
            country,
            server_region_id,
            in_changing_server: false,
            in_changing_region: true,
            last_enter_region_tick_ms: 0,
            entered_region: false,
            state_before_server_region_change: 0,
            current_progress: PlayerProgress::None,
            personal_shop_session_id: 0,
            personal_shop_plug_id: 0,
            war_soul_state: 0,
            war_soul_point: WarSoulPoint::default(),
            war_soul_visual_x_bits: 0.0f32.to_bits(),
            war_soul_visual_y_bits: 0.0f32.to_bits(),
            current_ticket: 0,
            goods_ai_tree: BTreeMap::new(),
            goods_ai_delete_queue: VecDeque::new(),
            flash_previous: [0; 17],
            flash_current: [0; 17],
            flash_changed: false,
            battle_fairy_summoned: false,
            recreate_carriage: false,
            active_carriage_id: 0,
            create_faction_operator: false,
            apply_join_faction_operator: false,
            faction_declare_operator: false,
            active_pet_count: 0,
            attempt_appellation_id: 0,
            realm_appellation_skill_id: realm_appellation_bonus
                .map_or(UNKNOWN_SKILL_ID, |identity| identity.0),
            realm_appellation_skill_level: realm_appellation_bonus.map_or(0, |identity| identity.1),
            heart_request_sent: 0,
            heart_received: false,
            friends: Vec::new(),
            quest_progress: PlayerQuestProgress::default(),
            lei_ting_things: VecDeque::new(),
            uncreated_pets: Vec::new(),
            uncreated_carriage: PlayerUncreatedCarriage::default(),
            login: false,
            session_id: Vec::new(),
            title: Vec::new(),
            base_property_wire: [0; PLAYER_BASE_PROPERTY_WIRE_SIZE],
            jjc_data: [0; 0x10],
            jjc_pk_state: false,
            fight_state_count: 0,
            auto_protected: false,
            lost_time_stamp_ms: 0,
            continuous_kill_amount: 0,
            continuous_kill_timestamp_ms: 0,
            base_properties: PlayerBaseProperties::default(),
            combat_properties: PlayerCombatProperties::default(),
            combat_property_wire: [0; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
            expendable_effects: BTreeMap::new(),
            last_skill_item_use_ms: BTreeMap::new(),
            ci_qing_open: false,
            ci_qing_list: BTreeSet::new(),
            ci_qing_add_values: BTreeMap::new(),
            ci_qing_tao_zhuang_add_values: BTreeMap::new(),
            tao_zhuang_id: 0,
            equipment_changed: false,
            tao_zhuang_setup_pending: true,
            tao_zhuang_items: BTreeMap::new(),
            tao_zhuang_original_names: BTreeSet::new(),
            tao_zhuang_properties: BTreeMap::new(),
            ci_qing_tao_zhuang_properties: BTreeMap::new(),
            tao_zhuang_skills: BTreeMap::new(),
            contend_state: false,
            emotion_index: 0,
            emotion_timestamp_ms: 0,
            city_war_died_state: false,
            city_war_died_state_time_ms: 0,
            died_state_start_time_ms: 0,
            criminal_state_timestamp_ms: 0,
            murderer_time_stamp_ms: 0,
            ping_time: 0,
            last_ping_time_ms: 0,
            contribution: 0,
            silence_minutes: 0,
            silence_timestamp_minutes: 0,
            normal_talk_timestamp_ms: 0,
            area_talk_timestamp_ms: 0,
            world_talk_timestamp_ms: 0,
            country_talk_timestamp_ms: 0,
            private_talk_timestamp_ms: 0,
            team_talk_timestamp_ms: 0,
            union_talk_timestamp_ms: 0,
            money: 0,
            client_ip: 0,
            account: Vec::new(),
            depot_password: Vec::new(),
            last_container_script: Vec::new(),
            variable_list: CVariableList::default(),
            bank: CBank::new(),
            depot: CDepot::new(),
            hand: CAmountLimitGoodsContainer::new(),
            enhancement: CAmountLimitGoodsShadowContainer::new(),
            last_operated_container: 0,
            last_operated_goods_position: 0,
            packet: CVolumeLimitGoodsContainer::new(),
            wallet: CWallet::new(),
            yuan_bao: CYuanBao::new(),
            ji_fen: CJiFen::new(),
            equipment: CEquipmentContainer::new(),
            auction_listing: CVolumeLimitGoodsContainer::new(),
            auction_goods: CVolumeLimitGoodsContainer::new(),
            auction_wallet: CWallet::new(),
            auction: PlayerAuction::default(),
            ci_qing: CVolumeLimitGoodsContainer::new(),
            ci_qing_compose: CVolumeLimitGoodsContainer::new(),
            fairy_container: CFairyContainer::new(),
            battle_fairy_container: CBattleFairyContainer::new(),
        };
        // Разметка persistence-контейнеров и перепривязка owner перенесены в
        // Zone players::gamesave (дизайн D4); hub только делегирует проекцию.
        let mut parts = player.game_save_parts();
        gamesave::init_player_persistence_containers(&mut parts);
        gamesave::refresh_player_container_owners(&mut parts, owner_id);
        Some(player)
    }

    /// Exact World→Game player handoff. Decoder восстанавливает единый
    /// `CPlayer::DecordFromByteArray(..., true)` блок, а не отдельные игровые
    /// фрагменты. Native GoodsAI tail намеренно отложен до успешной регистрации
    /// player-а в map/region и исполняется `CGame::complete_world_player_login`
    /// для достигнутых equipment и packet owners.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn decode_game_save(
        source: &[u8],
        cursor: &mut usize,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        variable_definitions: Option<&[u8]>,
        now_ms: u32,
        one_pk_count_time_ms: u32,
        state_now: &mut dyn FnMut() -> u32,
        pack_add_enabled: bool,
        ordinary_threshold: &mut dyn FnMut(u32, u32) -> u32,
        battle_threshold: &mut dyn FnMut(u32, u32) -> u32,
    ) -> Result<Self, PlayerGameSaveCodecError> {
        let start = *cursor;
        let mut move_shape = CMoveShape::default();
        move_shape
            .shape_mut()
            .decode_from_byte_array(source, cursor, true)?;
        let object_type = move_shape.shape().identity().object_type;
        if object_type != PLAYER_TYPE {
            return Err(PlayerGameSaveCodecError::WrongObjectType { object_type });
        }
        let server_region_id = move_shape.shape().get_region_id();

        let mut player = Self::from_send_state(
            move_shape,
            ShapeFigure::default(),
            0,
            0,
            Some(server_region_id),
        )
        .expect("player object type проверен до создания CPlayer");
        {
            let mut parts = player.game_save_parts();
            gamesave::decode_player_game_save(
                source,
                cursor,
                &mut parts,
                goods_factory,
                skill_factory,
                variable_definitions,
                now_ms,
                one_pk_count_time_ms,
                &mut *state_now,
                pack_add_enabled,
                &mut *ordinary_threshold,
                &mut *battle_threshold,
                super::skills::realmappellation::is_bonus_skill,
                |goods, base, combat| {
                    Self::can_mount_equip_from_properties(base, combat, goods, goods_factory)
                },
            )?;
        }

        let consumed_bytes = cursor.saturating_sub(start);
        tracing::trace!(
            player_id = player.player_id(),
            consumed_bytes,
            "сохранение игрока декодировано"
        );
        Ok(player)
    }

    /// Делегат прежней сигнатуры `AddGameSaveToByteArray`: persisted layout —
    /// Zone `players::gamesave::encode_player_game_save` (машинные факты
    /// `0x00441399` и `DelAllItemInDelList` UNKNOWN — там же).
    pub(crate) fn encode_game_save(
        &mut self,
        destination: &mut Vec<u8>,
        goods_factory: &CGoodsFactory,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
        one_pk_count_time_ms: u32,
        pets: &[PlayerUncreatedPet],
        carriage: &PlayerUncreatedCarriage,
        recreate_carriage: bool,
    ) -> Result<bool, PlayerGameSaveCodecError> {
        let mut parts = self.game_save_parts();
        gamesave::encode_player_game_save(
            destination,
            &mut parts,
            goods_factory,
            now_ms,
            timed_state_now_milliseconds,
            one_pk_count_time_ms,
            pets,
            carriage,
            recreate_carriage,
        )
        .map(|()| true)
    }

    fn synchronized_base_property_wire(&self) -> [u8; PLAYER_BASE_PROPERTY_WIRE_SIZE] {
        gamesave::synchronized_base_property_wire(
            &self.base_property_wire,
            &self.base_properties,
            self.battle_fairy_summoned,
        )
    }

    pub(crate) const fn shape(&self) -> &CShape {
        self.move_shape.shape()
    }

    /// Точный short-вариант `CPlayer::AddToByteArray_ForClient(false)` для
    /// area/query публикаций. Полный login/save вариант остаётся у отдельного
    /// GameSave codec; здесь нет container payload-ов и account-данных.
    pub(crate) fn encode_client_shape_snapshot(
        &self,
        goods_factory: &CGoodsFactory,
        country_identity: u8,
        personal_shop: Option<(i32, i32, &[u8])>,
        team_member_count: usize,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        clientsnapshot::encode_client_shape_snapshot(
            &PlayerClientShapeParts {
                move_shape: &self.move_shape,
                base_properties: &self.base_properties,
                combat_properties: &self.combat_properties,
                equipment: &self.equipment,
                organizing: self.organizing_snapshot(),
                murderer_time_stamp_ms: self.murderer_time_stamp_ms,
                contend_state: self.contend_state,
                city_war_died_state: self.city_war_died_state,
                emotion_index: self.emotion_index,
                emotion_timestamp_ms: self.emotion_timestamp_ms,
                country: self.country,
                war_soul_state: self.war_soul_state,
            },
            goods_factory,
            country_identity,
            personal_shop,
            team_member_count,
            now_milliseconds,
        )
    }

    /// Точный полный вариант `CPlayer::AddToByteArray_ForClient(true)`,
    /// который `OnLogMessage` вкладывает в успешный `0xBF401`. Persisted
    /// GameSave здесь неприменим: client wire иначе упорядочивает навыки,
    /// контейнеры, валюты, задания и завершающие country/CiQing поля.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_initial_client_snapshot(
        &mut self,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        quest_system: &CQuestSystem,
        da_kong_enabled: bool,
        level_experience: u32,
        country_identity: u8,
        team_member_count: usize,
        loan_time_limit: u32,
        ci_qing_quest_id: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let mut parts = PlayerInitialClientParts {
            move_shape: &self.move_shape,
            base_property_wire: &self.base_property_wire,
            base_properties: &self.base_properties,
            combat_property_wire: &self.combat_property_wire,
            account: &self.account,
            title: &self.title,
            friends: &self.friends,
            lei_ting_things: &self.lei_ting_things,
            // Поля ниже изменяемые — snapshot строится поэлементно, чтобы
            // не тянуть &self-заём метода под &mut-поля проекции.
            organizing: PlayerOrganizingSnapshot {
                faction_id: self.faction_id,
                faction_logo_id: self.faction_logo_id,
                faction_level: self.faction_level,
                faction_experience: self.faction_experience,
                faction_force: self.faction_force,
                faction_contribute: self.faction_contribute,
                faction_master_id: self.faction_master_id,
                faction_name: &self.faction_name,
                faction_title: &self.faction_title,
                union_id: self.union_id,
                union_master_id: self.union_master_id,
                enemy_factions: &self.enemy_factions,
                city_war_enemy_factions: &self.city_war_enemy_factions,
                faction_owned_regions: &self.faction_owned_regions,
            },
            contend_state: self.contend_state,
            city_war_died_state: self.city_war_died_state,
            quest_progress: &self.quest_progress,
            country: self.country,
            contribution: self.contribution,
            war_soul_state: self.war_soul_state,
            hand: &self.hand,
            equipment: &self.equipment,
            auction_goods: &self.auction_goods,
            packet: &self.packet,
            auction_listing: &self.auction_listing,
            fairy_container: &self.fairy_container,
            wallet: &self.wallet,
            auction_wallet: &self.auction_wallet,
            yuan_bao: &self.yuan_bao,
            ji_fen: &self.ji_fen,
            battle_fairy_container: &self.battle_fairy_container,
            ci_qing_compose: &self.ci_qing_compose,
            ci_qing: &self.ci_qing,
            ci_qing_open: &mut self.ci_qing_open,
            battle_fairy_summoned: &mut self.battle_fairy_summoned,
        };
        clientsnapshot::encode_initial_client_snapshot(
            &mut parts,
            goods_factory,
            skill_factory,
            quest_system,
            da_kong_enabled,
            level_experience,
            country_identity,
            team_member_count,
            loan_time_limit,
            ci_qing_quest_id,
            timed_state_now_milliseconds,
        )
    }

    pub(crate) const fn player_ai(&self) -> &CPlayerAI {
        &self.player_ai
    }

    pub(crate) const fn player_ai_mut(&mut self) -> &mut CPlayerAI {
        &mut self.player_ai
    }

    pub(crate) fn take_player_ai(&mut self) -> CPlayerAI {
        std::mem::take(&mut self.player_ai)
    }

    pub(crate) fn restore_player_ai(&mut self, player_ai: CPlayerAI) {
        self.player_ai = player_ai;
    }

    pub(crate) const fn restore_login_team(&mut self, captain: bool, team_id: i32) {
        self.team_captain = captain;
        self.team_id = team_id;
    }

    /// `CTeam::OnPlugInserted/OnPlugEnded` меняют canonical player team ID.
    pub(crate) const fn set_team_membership(&mut self, team_id: i32) {
        self.team_id = team_id;
        if team_id == 0 {
            self.team_captain = false;
        }
    }

    pub(crate) const fn set_team_captain(&mut self, captain: bool) {
        self.team_captain = captain;
    }

    pub(crate) const fn is_team_captain(&self) -> bool {
        self.team_captain
    }

    pub(crate) const fn mark_login_script_started(&mut self) -> bool {
        let first_login = !self.login;
        self.login = true;
        first_login
    }

    pub(crate) const fn player_id(&self) -> i32 {
        self.shape().identity().id
    }

    pub(crate) fn player_name(&self) -> &[u8] {
        self.shape().base_object().get_name()
    }

    pub(crate) fn friends(&self) -> &[PlayerFriend] {
        &self.friends
    }

    pub(crate) fn has_team_recruitment_state(&self) -> bool {
        self.move_shape.team_recruitment_states().next().is_some()
    }

    pub(crate) fn team_recruitment_state_count(&self) -> usize {
        self.move_shape.team_recruitment_states().count()
    }

    pub(crate) fn first_team_recruitment_state(&self) -> Option<&CTeamState> {
        self.move_shape.team_recruitment_states().next()
    }



    /// Возвращает состояние вложенного в игрока прогресса для сценарного входа.
    pub(crate) fn quest_state(&self, quest_id: u16) -> i32 {
        self.quest_progress.state(quest_id)
    }

    pub(crate) fn complete_quest_script_path<'a>(
        &self,
        quest_id: u16,
        catalog: &'a CQuestSystem,
    ) -> Option<&'a [u8]> {
        self.quest_progress.complete_script_path(quest_id, catalog)
    }

    pub(crate) fn abandon_quest_script_path<'a>(
        &self,
        quest_id: u16,
        catalog: &'a CQuestSystem,
    ) -> Option<&'a [u8]> {
        self.quest_progress.abandon_script_path(quest_id, catalog)
    }

    pub(crate) fn set_quest_state_snapshot(&mut self, quest_id: u16, state: u8) {
        self.quest_progress.insert_snapshot(quest_id, state);
    }

    pub(crate) fn accept_script_quest(&mut self, quest_id: u16, definition_exists: bool) -> bool {
        self.quest_progress.accept(quest_id, definition_exists)
    }

    pub(crate) fn complete_script_quest(&mut self, quest_id: u16) -> bool {
        self.quest_progress.complete(quest_id)
    }

    pub(crate) fn remove_script_quest(&mut self, quest_id: u16) -> bool {
        self.quest_progress.remove(quest_id)
    }

    pub(crate) fn has_script_quest(&self, quest_id: u16) -> bool {
        self.quest_progress.contains(quest_id)
    }

    pub(crate) fn valid_script_quest_count(
        &self,
        is_displayed_quest: impl FnMut(u16) -> bool,
    ) -> i32 {
        self.quest_progress.valid_count(is_displayed_quest)
    }

    pub(crate) fn add_friend_state(&mut self, name: &[u8]) -> PlayerFriendAddOutcome {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        if self.friends.len() >= 0x28 {
            return PlayerFriendAddOutcome::LimitReached;
        }
        if self.friends.iter().any(|friend| friend.name == name) {
            return PlayerFriendAddOutcome::AlreadyPresent;
        }
        self.friends.push(PlayerFriend {
            name: name.to_vec(),
            online: true,
        });
        PlayerFriendAddOutcome::Added
    }

    pub(crate) fn delete_friend_state(&mut self, name: &[u8]) -> bool {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        let Some(index) = self.friends.iter().position(|friend| friend.name == name) else {
            return false;
        };
        self.friends.remove(index);
        true
    }

    pub(crate) fn has_friend(&self, name: &[u8]) -> bool {
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        self.friends.iter().any(|friend| friend.name == name)
    }

    pub(crate) const fn team_id(&self) -> i32 {
        self.team_id
    }

    pub(crate) const fn is_charged(&self) -> bool {
        self.base_properties.charged
    }

    pub(crate) const fn set_charged(&mut self, charged: bool) {
        self.base_properties.charged = charged;
    }

    pub(crate) const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    pub(crate) const fn faction_level(&self) -> u16 {
        self.faction_level
    }

    pub(crate) const fn faction_experience(&self) -> i32 {
        self.faction_experience
    }

    pub(crate) const fn is_faction_master(&self) -> bool {
        self.faction_id > 0 && self.faction_master_id == self.player_id()
    }

    pub(crate) fn faction_name(&self) -> &[u8] {
        &self.faction_name
    }

    pub(crate) const fn union_id(&self) -> i32 {
        self.union_id
    }

    pub(crate) const fn union_master_id(&self) -> i32 {
        self.union_master_id
    }

    pub(crate) const fn is_union_master(&self) -> bool {
        self.union_id > 0 && self.union_master_id == self.player_id()
    }

    pub(crate) const fn create_faction_operator(&self) -> bool {
        self.create_faction_operator
    }

    pub(crate) const fn set_create_faction_operator(&mut self, value: bool) {
        self.create_faction_operator = value;
    }

    pub(crate) const fn apply_join_faction_operator(&self) -> bool {
        self.apply_join_faction_operator
    }

    pub(crate) const fn set_apply_join_faction_operator(&mut self, value: bool) {
        self.apply_join_faction_operator = value;
    }

    pub(crate) const fn faction_declare_operator(&self) -> bool {
        self.faction_declare_operator
    }

    pub(crate) const fn set_faction_declare_operator(&mut self, value: bool) {
        self.faction_declare_operator = value;
    }

    pub(crate) fn restore_faction_identity(
        &mut self,
        faction_id: i32,
        faction_logo_id: i32,
        faction_level: u16,
        faction_experience: i32,
        faction_force: i32,
        faction_contribute: u32,
        faction_master_id: i32,
        faction_name: &[u8],
        faction_title: &[u8],
        union_id: i32,
        union_master_id: i32,
        enemy_factions: BTreeSet<i32>,
        city_war_enemy_factions: BTreeSet<i32>,
        faction_owned_regions: Vec<[u8; 8]>,
    ) {
        self.faction_id = faction_id;
        self.faction_logo_id = faction_logo_id;
        self.faction_level = faction_level;
        self.faction_experience = faction_experience;
        self.faction_force = i32::from(faction_force != 0);
        self.faction_contribute = u32::from(faction_contribute != 0);
        self.faction_master_id = faction_master_id;
        self.faction_name.clear();
        self.faction_name.extend_from_slice(faction_name);
        self.faction_title.clear();
        self.faction_title.extend_from_slice(faction_title);
        self.union_id = union_id;
        self.union_master_id = union_master_id;
        self.enemy_factions = enemy_factions;
        self.city_war_enemy_factions = city_war_enemy_factions;
        self.faction_owned_regions = faction_owned_regions;
    }

    pub(crate) fn is_enemy_faction_member(&self, faction_id: i32) -> bool {
        self.faction_id > 0 && self.enemy_factions.contains(&faction_id)
    }

    pub(crate) fn is_city_war_enemy_faction_member(&self, faction_id: i32) -> bool {
        self.faction_id > 0 && self.city_war_enemy_factions.contains(&faction_id)
    }

    pub(crate) const fn country(&self) -> u8 {
        self.country
    }

    /// `CPlayer::SetScriptValue("btCountry")` сужает signed script value до
    /// исходного byte storage без country-range validation.
    pub(crate) const fn set_script_country(&mut self, requested: i32) -> i32 {
        self.country = requested as u8;
        self.country as i32
    }

    /// Ответ World `0x7FF01` меняет страну только для signed диапазона `1..4`;
    /// невалидное значение всё равно публикуется caller-ом в `0xC0301`.
    pub(crate) fn apply_world_country(&mut self, requested: i32) {
        let previous = self.country;
        if (1..5).contains(&requested) {
            self.country = requested as u8;
        }
        tracing::trace!(
            player_id = self.player_id(),
            previous,
            requested,
            applied = self.country,
            changed = self.country != previous,
            "страна игрока изменена ответом World"
        );
    }

    /// Сбрасывает подтверждённые счётчики чести и возвращает exact условие
    /// `AdjustHonorRank`: ненулевой nobility rank требует запуска сценария.
    pub(crate) fn reset_total_honor_eliminate(&mut self, reset_mask: u32) -> bool {
        let player_id = self.player_id();
        let previous_days = self.base_properties.days_honor_eliminate;
        let previous_weeks = self.base_properties.weeks_honor_eliminate;
        let previous_months = self.base_properties.months_honor_eliminate;
        let adjust_honor_rank = self.base_properties.rank_of_nobility_id != 0;
        self.base_properties.days_honor_eliminate = 0;
        if reset_mask & 2 != 0 {
            self.base_properties.weeks_honor_eliminate = 0;
        }
        if reset_mask & 4 != 0 {
            self.base_properties.months_honor_eliminate = 0;
        }
        tracing::trace!(
            player_id,
            reset_mask,
            previous_days,
            previous_weeks,
            previous_months,
            adjust_honor_rank,
            "счётчики чести игрока сброшены"
        );
        adjust_honor_rank
    }

    pub(crate) const fn server_region_id(&self) -> Option<i32> {
        self.server_region_id
    }

    pub(crate) const fn in_changing_server(&self) -> bool {
        self.in_changing_server
    }

    pub(crate) const fn in_changing_region(&self) -> bool {
        self.in_changing_region
    }

    /// Exact anti-repeat prefix `8F801`: timestamp записывается после
    /// успешного EnterTime gate, но ещё до проверки `m_bInChangingRegion`.
    pub(crate) fn accept_region_entry_ack(&mut self, now_ms: u32, enter_time_ms: u32) -> bool {
        if now_ms.wrapping_sub(self.last_enter_region_tick_ms) < enter_time_ms {
            return false;
        }
        self.last_enter_region_tick_ms = now_ms;
        self.in_changing_region
    }

    pub(crate) const fn begin_region_entry_states(&mut self) {
        self.move_shape.reset_region_entry_control();
    }

    pub(crate) const fn mark_entered_region(&mut self) {
        self.entered_region = true;
    }

    pub(crate) const fn set_changing_state_snapshot(
        &mut self,
        in_changing_server: bool,
        in_changing_region: bool,
    ) {
        self.in_changing_server = in_changing_server;
        self.in_changing_region = in_changing_region;
    }

    /// Player-owned scalar tail локального `ChangeRegion`; фактическое
    /// membership перемещение остаётся deferred у `CServerRegion::AI`.
    pub(crate) fn stage_local_region_change(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
    ) {
        self.in_changing_server = false;
        self.in_changing_region = true;
        self.entered_region = false;
        self.movement_shape_mut()
            .stage_region_change(region_id, tile_x, tile_y, direction);
    }

    pub(crate) fn begin_cross_region_companion_change(&mut self) {
        self.recreate_carriage = false;
        self.uncreated_carriage = PlayerUncreatedCarriage::default();
    }

    /// Снимки companion-ов принадлежат игроку между выходом из исходного
    /// region owner-а и созданием новых monster-owner-ов в назначении.
    pub(crate) fn store_uncreated_region_pets(&mut self, pets: Vec<PlayerUncreatedPet>) {
        self.uncreated_pets = pets;
    }

    pub(crate) fn store_uncreated_region_carriage(&mut self, carriage: PlayerUncreatedCarriage) {
        self.uncreated_carriage = carriage;
        self.recreate_carriage = true;
    }

    /// Same-region `ChangeRegion` не пересоздаёт повозку: возможный перенос
    /// живого monster-owner выполняет `CGame`, а persisted snapshot сбрасывает
    /// player-owner до его поиска, как исходный `m_bReCreateCarriage = false`.
    pub(crate) const fn begin_same_region_change(&mut self) {
        self.recreate_carriage = false;
    }

    pub(crate) fn begin_server_region_change(&mut self) {
        self.state_before_server_region_change = self.shape().get_state();
        self.movement_shape_mut().set_state(0);
        self.in_changing_server = true;
        self.in_changing_region = true;
        self.entered_region = false;
        self.recreate_carriage = false;
    }

    pub(crate) fn cancel_server_region_change(&mut self) {
        if self.in_changing_server {
            let state = self.state_before_server_region_change;
            self.movement_shape_mut().set_state(state);
        }
        self.in_changing_server = false;
        self.in_changing_region = false;
    }

    pub(crate) fn apply_staged_local_region_change(&mut self) -> (i32, i32, i32, i32) {
        let destination = self.movement_shape_mut().apply_staged_region_change();
        self.server_region_id = Some(destination.0);
        destination
    }

    pub(crate) const fn current_progress(&self) -> PlayerProgress {
        self.current_progress
    }

    pub(crate) const fn set_current_progress_snapshot(&mut self, progress: PlayerProgress) {
        self.current_progress = progress;
    }

    pub(crate) const fn set_personal_shop_flag(&mut self, session_id: i32, plug_id: i32) {
        self.personal_shop_session_id = session_id;
        self.personal_shop_plug_id = plug_id;
    }

    pub(crate) const fn personal_shop_flag(&self) -> Option<(i32, i32)> {
        if self.personal_shop_session_id == 0 || self.personal_shop_plug_id == 0 {
            None
        } else {
            Some((self.personal_shop_session_id, self.personal_shop_plug_id))
        }
    }

    pub(crate) fn begin_equipment_session(
        &mut self,
        progress: PlayerProgress,
        lock_movement: bool,
    ) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = progress;
        if lock_movement {
            self.move_shape.set_moveable(false);
        }
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn attach_equipment_session_listener(&mut self, plug_id: i32) -> [bool; 2] {
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let packet = self
            .packet
            .base_mut()
            .base_mut()
            .base_mut()
            .add_listener(listener);
        let equipment = self.equipment.base_mut().base_mut().add_listener(listener);
        [packet, equipment]
    }

    pub(crate) fn detach_equipment_session_listener(&mut self, plug_id: i32) -> [bool; 2] {
        let listener = usize::try_from(plug_id)
            .ok()
            .and_then(ContainerListenerHandle::from_legacy_identity);
        let packet = self
            .packet
            .base_mut()
            .base_mut()
            .base_mut()
            .remove_listener(listener);
        let equipment = self
            .equipment
            .base_mut()
            .base_mut()
            .remove_listener(listener);
        [packet, equipment]
    }

    pub(crate) const fn is_dead(&self) -> bool {
        CMoveShape::is_died(self.base_properties.health)
    }

    pub(crate) const fn set_god_mode(&mut self, enabled: bool) {
        self.move_shape.set_god(enabled);
    }

    pub(crate) const fn is_god_mode(&self) -> bool {
        self.move_shape.is_god()
    }

    pub(crate) fn release_goods_session_state(&mut self) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = PlayerProgress::None;
        self.move_shape.set_moveable(true);
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn begin_synthesis(&mut self) -> GoodsSessionPlayerRelease {
        let previous_progress = self.current_progress;
        let previous_moveable_count = self.move_shape.moveable_count();
        self.current_progress = PlayerProgress::Synthesis;
        self.move_shape.set_moveable(false);
        GoodsSessionPlayerRelease {
            previous_progress,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    pub(crate) fn close_synthesis(&mut self) -> Option<GoodsSessionPlayerRelease> {
        (self.current_progress == PlayerProgress::Synthesis)
            .then(|| self.release_goods_session_state())
    }

    pub(crate) fn depot_password(&self) -> &[u8] {
        &self.depot_password
    }

    /// Exact `SetDepotPassword`: nullable C-string уже разрешена caller-ом;
    /// сохраняются только bytes до первого NUL.
    pub(crate) fn set_depot_password(&mut self, password: &[u8]) {
        let length = password
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(password.len());
        self.depot_password.clear();
        self.depot_password.extend_from_slice(&password[..length]);
    }

    pub(crate) fn unlock_depot_storage(&mut self) {
        let _legacy_bank_result = self.bank.unlock_if_authenticated(true);
        let _legacy_depot_result = self.depot.unlock_if_authenticated(true);
    }

    pub(crate) fn close_depot_storage(&mut self) {
        let _legacy_bank_result = self.bank.lock();
        let _legacy_depot_result = self.depot.lock();
        self.current_progress = PlayerProgress::None;
    }

    pub(crate) fn prepare_depot_storage(&mut self, password_required: bool) {
        if password_required {
            let _ = self.bank.lock();
            let _ = self.depot.lock();
        } else {
            let _ = self.bank.unlock_if_authenticated(true);
            let _ = self.depot.unlock_if_authenticated(true);
        }
    }

    pub(crate) fn bank_snapshot_goods(&self, position: u32) -> Option<&CGoods> {
        self.bank.snapshot_goods(position)
    }

    pub(crate) const fn bank_locked(&self) -> bool {
        self.bank.is_locked()
    }

    pub(crate) const fn depot_locked(&self) -> bool {
        self.depot.is_locked()
    }

    pub(crate) const fn base_properties(&self) -> PlayerBaseProperties {
        self.base_properties
    }

    pub(crate) const fn set_display_head_piece(&mut self, display: bool) {
        self.base_properties.display_head_piece = display;
    }

    pub(crate) const fn display_head_piece(&self) -> bool {
        self.base_properties.display_head_piece
    }

    pub(crate) const fn quest_enabled(&self) -> bool {
        self.base_properties.quest_availability.enabled()
    }

    pub(crate) const fn set_quest_enabled(&mut self, enabled: bool) {
        self.base_properties.quest_availability.set_enabled(enabled);
    }

    pub(crate) const fn begin_quest_time(&mut self, now_seconds: i32, time_limit: i32) {
        self.base_properties.quest_availability.begin(now_seconds, time_limit);
    }

    pub(crate) const fn clear_quest_time(&mut self) {
        self.base_properties.quest_availability.clear_time();
    }

    pub(crate) const fn quest_time_remaining(&self, now_seconds: i32) -> i32 {
        self.base_properties.quest_availability.remaining(now_seconds)
    }

    pub(crate) const fn client_quest_time_remaining(&self, now_seconds: i32) -> i32 {
        self.base_properties
            .quest_availability
            .client_remaining(now_seconds)
    }

    pub(crate) const fn acknowledge_heartbeat(&mut self) {
        self.heart_request_sent = 0;
        self.heart_received = true;
    }

    /// LeiTing snapshot codec — Zone `players::gamesave` (совпадает с
    /// WorldServer `AddByteArrayLeiTing`); делегат прежней сигнатуры.
    pub(crate) fn encode_lei_ting(&self) -> Vec<u8> {
        gamesave::encode_player_lei_ting(&self.base_properties, &self.lei_ting_things)
    }

    pub(crate) fn decode_lei_ting(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerLeiTingDecodeBlock> {
        gamesave::decode_player_lei_ting(
            &mut self.base_properties,
            &mut self.lei_ting_things,
            source,
            cursor,
        )
    }

    pub(crate) const fn change_fy_energy_flag(&mut self, index: u16) -> bool {
        let threshold_reached = match index {
            0 => self.base_properties.fy_energy >= 20,
            1 => self.base_properties.fy_energy >= 60,
            2 => self.base_properties.fy_energy >= 80,
            3 => self.base_properties.fy_energy >= 100,
            4 => self.base_properties.lt_up_60_count >= 4,
            5 => self.base_properties.lt_up_60_count >= 10,
            6 => self.base_properties.lt_up_60_count >= 16,
            7 => self.base_properties.lt_up_60_count >= 22,
            8 => self.base_properties.lt_up_60_count >= 28,
            _ => return false,
        };
        let flag = LeiTingEnableFlags::from_bits_retain(1u32 << index);
        if !threshold_reached || self.base_properties.fy_enable_flags.contains(flag) {
            return false;
        }
        self.base_properties.fy_enable_flags = LeiTingEnableFlags::from_bits_retain(
            self.base_properties.fy_enable_flags.bits() | flag.bits(),
        );
        true
    }

    /// `GetOneThing((ushort)id)`: lookup намеренно сохраняет narrowing без
    /// последующей проверки исходного signed ID, как ветвь `GetThingCnt`.
    pub(crate) fn lei_ting_thing_count(&self, thing_id: i32) -> Option<u16> {
        let narrowed = thing_id as u16;
        self.lei_ting_things
            .iter()
            .find(|thing| thing.thing_id == narrowed)
            .map(|thing| thing.count)
    }

    /// Полный reached `AddThingCnt(id, count, false)`. В отличие от getter-а,
    /// setter после ushort lookup сравнивает сохранённый ID с исходным signed
    /// аргументом. Разрешено только строго положительное увеличение; энергия
    /// и суточные поля сохраняют wrapping-арифметику x86 owner-а. Closure —
    /// только CRT/local-time граница `AddLTUp60Cnt`; wire исполняет script
    /// caller после успешной mutation.
    pub(crate) fn set_lei_ting_thing_count(
        &mut self,
        thing_id: i32,
        requested_count: i32,
        next_daily_stamp_if_same_local_day: impl FnOnce(u32) -> Option<u32>,
    ) -> PlayerLeiTingThingCountOutcome {
        let narrowed = thing_id as u16;
        let Some(index) = self
            .lei_ting_things
            .iter()
            .position(|thing| thing.thing_id == narrowed)
        else {
            return PlayerLeiTingThingCountOutcome::Missing;
        };
        let current = self.lei_ting_things[index];
        if u32::from(current.thing_id) != thing_id as u32 {
            return PlayerLeiTingThingCountOutcome::Missing;
        }
        let difference = requested_count.wrapping_sub(i32::from(current.count));
        if difference < 1
            || difference > i32::from(current.max_count)
            || requested_count > i32::from(current.max_count)
        {
            return PlayerLeiTingThingCountOutcome::Rejected {
                current: current.count,
                requested: requested_count,
                maximum: current.max_count,
            };
        }

        self.lei_ting_things[index].count = requested_count as u16;
        let previous_energy = self.base_properties.fy_energy;
        self.base_properties.fy_energy =
            previous_energy.wrapping_add(u32::from(current.point).wrapping_mul(difference as u32));
        let daily_count_incremented = if self.base_properties.fy_energy >= 60 {
            if let Some(next_stamp) =
                next_daily_stamp_if_same_local_day(self.base_properties.lt_60_stamp)
            {
                self.base_properties.lt_up_60_count =
                    self.base_properties.lt_up_60_count.wrapping_add(1);
                self.base_properties.lt_60_stamp = next_stamp;
                true
            } else {
                false
            }
        } else {
            false
        };
        PlayerLeiTingThingCountOutcome::Updated {
            previous_count: current.count,
            current_count: requested_count as u16,
            previous_energy,
            current_energy: self.base_properties.fy_energy,
            daily_count_incremented,
        }
    }

    pub(crate) const fn honor_snapshot(&self) -> PlayerHonorSnapshot {
        PlayerHonorSnapshot {
            rank_of_nobility_id: self.base_properties.rank_of_nobility_id,
            appellation_id: self.base_properties.appellation_id,
            days_eliminate: self.base_properties.days_honor_eliminate,
            weeks_eliminate: self.base_properties.weeks_honor_eliminate,
            months_eliminate: self.base_properties.months_honor_eliminate,
            total_eliminate: self.base_properties.total_honor_eliminate,
        }
    }

    /// World acknowledgement `0x7FA16` подтверждает уже принятую honor-пару:
    /// все четыре счётчика увеличиваются независимо с DWORD wrapping.
    pub(crate) const fn acknowledge_honor_eliminate(&mut self) -> PlayerHonorEliminateMutation {
        let previous = [
            self.base_properties.days_honor_eliminate,
            self.base_properties.weeks_honor_eliminate,
            self.base_properties.months_honor_eliminate,
            self.base_properties.total_honor_eliminate,
        ];
        self.base_properties.days_honor_eliminate = previous[0].wrapping_add(1);
        self.base_properties.weeks_honor_eliminate = previous[1].wrapping_add(1);
        self.base_properties.months_honor_eliminate = previous[2].wrapping_add(1);
        self.base_properties.total_honor_eliminate = previous[3].wrapping_add(1);
        PlayerHonorEliminateMutation {
            player_id: self.player_id(),
            previous,
            current: [
                self.base_properties.days_honor_eliminate,
                self.base_properties.weeks_honor_eliminate,
                self.base_properties.months_honor_eliminate,
                self.base_properties.total_honor_eliminate,
            ],
        }
    }

    pub(crate) const fn request_change_appellation_state(&mut self, appellation_id: u32) {
        self.attempt_appellation_id = appellation_id;
    }


    pub(crate) const fn jjc_pk_state(&self) -> bool {
        self.jjc_pk_state
    }

    pub(crate) const fn set_jjc_pk_state(&mut self, active: bool) {
        self.jjc_pk_state = active;
    }

    pub(crate) const fn jjc_level(&self) -> u32 {
        self.base_properties.jjc_level
    }

    pub(crate) const fn jjc_score(&self) -> u32 {
        self.base_properties.jjc_score
    }

    pub(crate) const fn jjc_data(&self) -> &[u8; 0x10] {
        &self.jjc_data
    }

    /// Восемь `unsigned short` из точного `tagJJcData` лежат подряд в
    /// GameSave/wire-порядке: четыре недельных, затем четыре сезонных счётчика.
    pub(crate) fn jjc_counter(&self, selector: i32) -> Option<u16> {
        let index = usize::try_from(selector.checked_sub(1)?).ok()?;
        let offset = index.checked_mul(2)?;
        let bytes = self.jjc_data.get(offset..offset.checked_add(2)?)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn set_jjc_counter(&mut self, selector: i32, value: u16) -> bool {
        let Some(offset) = selector
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| index.checked_mul(2))
        else {
            return false;
        };
        let Some(bytes) = self
            .jjc_data
            .get_mut(offset..offset.saturating_add(2))
        else {
            return false;
        };
        bytes.copy_from_slice(&value.to_le_bytes());
        true
    }

    /// Exact `JJcWeekClear`: первые четыре WORD — weekly counters; при
    /// пятнадцати участиях score получает level-dependent award с x87
    /// усечением к нулю и cap 1500.
    pub(crate) fn clear_jjc_week(&mut self) {
        let joined = LegacyReader::new(&self.jjc_data)
            .read_u16()
            .expect("JJC data содержит флаг участия");
        if joined >= 15 {
            let factor = if self.base_properties.jjc_level < 1001 {
                100.0
            } else {
                300.0
            };
            let award = (f64::from(self.base_properties.jjc_level) * 0.001 * factor)
                .trunc()
                .clamp(0.0, 1500.0) as u32;
            self.base_properties.jjc_score = self.base_properties.jjc_score.wrapping_add(award);
        }
        self.jjc_data[..8].fill(0);
    }

    pub(crate) fn clear_jjc_season(&mut self, default_level: i32) {
        self.jjc_data.fill(0);
        self.base_properties.jjc_level = default_level as u32;
        self.base_properties.jjc_score = 0;
    }

    pub(crate) fn get_appellation_state(&self, state_id: u32) -> u32 {
        self.move_shape.get_undead_state(state_id)
    }



    pub(crate) fn change_body_check(&self) -> bool {
        self.base_properties.mode == 0
            && !matches!(
                self.current_progress,
                PlayerProgress::Trading | PlayerProgress::OpenStall
            )
            && self.team_id == 0
            && !self.is_rider()
            && !self.has_pet()
            && self.uncreated_carriage.original_name.is_empty()
    }

    pub(crate) fn has_change_body_state(&self) -> bool {
        self.move_shape.active_change_body_state().is_some()
    }


    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        self.move_shape.get_change_body_state(state_id)
    }

    pub(crate) fn first_change_body_state_id(&self) -> Option<u32> {
        self.move_shape.first_change_body_state_id()
    }





    pub(crate) fn is_rider(&self) -> bool {
        self.move_shape.has_ride_state()
    }







    pub(crate) fn replace_boss_blue_quake_state(
        &mut self,
        state: super::skills::bossbluequakestate::BossBlueQuakeState,
    ) -> Option<super::skills::bossbluequakestate::BossBlueQuakeState> {
        self.move_shape.replace_boss_blue_quake_state(state)
    }




    pub(crate) fn take_boss_blue_quake_state(
        &mut self,
    ) -> Option<super::skills::bossbluequakestate::BossBlueQuakeState> {
        self.move_shape.take_boss_blue_quake_state()
    }

    /// Базовая и equipment/CiQing половина `CPlayer::UpdateProperty` до
    /// виртуального `CMoveShape::UpdateProperty`. Два signed addon-pass-а
    /// сохраняют slot order `MountAllEquip`. Восемь recovery scalar-ов
    /// каждый раз восстанавливаются из `CGlobeSetup::tagSetup` до применения
    /// equipment, CiQing и state addon-ов, как остальные базовые свойства.
    pub(crate) fn recompute_base_and_equipment_properties(
        &self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        goods_factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        self.recompute_base_and_mounted_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            goods_factory,
            true,
        )
    }

    /// Базовые cases `0x80..0x84` исходного `MountCiQingEquip`. Они не входят
    /// в combat snapshot: owner меняет сохранённый `tagBaseProperty` в порядке
    /// восьми ячеек, сперва для неотрицательных, затем для отрицательных
    /// addon-ов каждого предмета. Durability здесь намеренно не проверяется.
    pub(crate) fn apply_ci_qing_base_properties(&mut self, factory: &CGoodsFactory) {
        fn add_property(target: &mut u32, delta: i32) {
            let next = target.wrapping_add(delta as u32);
            *target = if delta < 0 && (next as i32) < 0 {
                0
            } else {
                next
            };
        }

        for position in 0..self.ci_qing.size() {
            let additions = {
                let Some(goods) = self.ci_qing.get_goods(position) else {
                    continue;
                };
                goods
                    .enabled_addon_properties(factory)
                    .into_iter()
                    .map(|property_type| {
                        (
                            property_type,
                            goods.addon_property_value(factory, property_type, 1),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            for positive_pass in [true, false] {
                for &(property_type, delta) in &additions {
                    if (delta >= 0) != positive_pass {
                        continue;
                    }
                    match property_type {
                        GAP_BREAK_ARMOUR => {
                            add_property(&mut self.base_properties.break_armour, delta)
                        }
                        GAP_PUNCTURE => add_property(&mut self.base_properties.puncture, delta),
                        GAP_BREAK_ELEMENT => {
                            add_property(&mut self.base_properties.break_element, delta)
                        }
                        GAP_BREAK_BOUND => {
                            add_property(&mut self.base_properties.break_bound, delta)
                        }
                        GAP_GOLD_POWER => {
                            add_property(&mut self.base_properties.power_of_gold, delta)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Первый снимок `MountAllEquip`: обычная экипировка уже применена, а
    /// восемь CiQing-ячеек ещё нет. Он нужен для exact насыщенной разницы
    /// `UpdateCiQingProperty` и не создаёт теневого player state.
    pub(crate) fn recompute_without_ci_qing_properties(
        &self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        goods_factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        self.recompute_base_and_mounted_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            goods_factory,
            false,
        )
    }

    fn recompute_base_and_mounted_properties(
        &self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        goods_factory: &CGoodsFactory,
        include_ci_qing: bool,
    ) -> PlayerCombatProperties {
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let base_u16 = |offset| read_player_wire_u16(&self.base_property_wire, offset);
        let base_u32 = |offset| read_player_wire_u32(&self.base_property_wire, offset);
        let derived = |value: u32, coefficient: f32| {
            ((value as f32) * coefficient).trunc() as u32
        };
        let mut properties = PlayerCombatProperties {
            maximum_hp: self
                .base_properties
                .base_maximum_hp
                .wrapping_add(derived(
                    self.base_properties.base_constitution,
                    coefficients.con_to_max_hp[occupation],
                )),
            maximum_mp: self
                .base_properties
                .base_maximum_mp
                .wrapping_add(derived(
                    self.base_properties.base_intelligence,
                    coefficients.int_to_max_mp[occupation],
                )),
            maximum_yp: self.base_properties.maximum_yp,
            maximum_rp: self.base_properties.maximum_rp,
            strength: self.base_properties.base_strength,
            dexterity: self.base_properties.base_dexterity,
            constitution: self.base_properties.base_constitution,
            intelligence: self.base_properties.base_intelligence,
            minimum_attack: base_u32(BASE_MINIMUM_ATTACK_OFFSET).wrapping_add(derived(
                self.base_properties.base_dexterity,
                coefficients.dex_to_min_attack[occupation],
            )),
            maximum_attack: base_u32(BASE_MAXIMUM_ATTACK_OFFSET).wrapping_add(derived(
                self.base_properties.base_strength,
                coefficients.str_to_max_attack[occupation],
            )),
            attack_speed: base_u16(BASE_ATTACK_SPEED_OFFSET),
            hit: base_u16(BASE_HIT_OFFSET),
            dodge: base_u16(BASE_DODGE_OFFSET),
            cch: base_u16(BASE_CCH_OFFSET),
            defense: base_u32(BASE_DEFENSE_OFFSET).wrapping_add(derived(
                self.base_properties.base_constitution,
                coefficients.con_to_defense[occupation],
            )),
            element_resistance: base_u32(BASE_ELEMENT_RESISTANCE_OFFSET).wrapping_add(derived(
                self.base_properties.base_intelligence,
                coefficients.int_to_resistant[occupation],
            )),
            hp_recovery: base_u16(BASE_HP_RECOVERY_OFFSET),
            mp_recovery: base_u16(BASE_MP_RECOVERY_OFFSET),
            burden: self.base_properties.base_burden.wrapping_add(
                derived(
                    self.base_properties.base_strength,
                    coefficients.str_to_burden[occupation],
                ) as u16,
            ),
            reank: derived(
                self.base_properties.base_dexterity,
                coefficients.dex_to_stiff[occupation],
            ) as u16,
            element_modify: derived(
                self.base_properties.base_intelligence,
                coefficients.int_to_element[occupation],
            ) as i32,
            blast_attack_scale_bits: base_combat_scales[0].to_bits(),
            blast_defense_scale_bits: base_combat_scales[1].to_bits(),
            element_blast_attack_scale_bits: base_combat_scales[2].to_bits(),
            element_blast_defense_scale_bits: base_combat_scales[3].to_bits(),
            full_miss_scale_bits: base_combat_scales[4].to_bits(),
            critical_rate_bits: critical_rate.to_bits(),
            resume_hp_peace: coefficients.resume_hp_peace,
            resume_mp_peace: coefficients.resume_mp_peace,
            resume_hp_fight: coefficients.resume_hp_fight,
            resume_mp_fight: coefficients.resume_mp_fight,
            restored_hp_peace: coefficients.restored_hp_peace,
            restored_mp_peace: coefficients.restored_mp_peace,
            restored_hp_fight: coefficients.restored_hp_fight,
            restored_mp_fight: coefficients.restored_mp_fight,
            battle_fairy_summoned: self.battle_fairy_summoned,
            battle_fairy_recall: self.base_properties.battle_fairy_recall,
            battle_fairy_died: self.base_properties.battle_fairy_died,
            ..PlayerCombatProperties::default()
        };
        let mut active_levels = BTreeMap::<i32, u8>::new();
        for (_, goods) in self.equipment.traversing_goods() {
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(goods_factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            if !goods.has_addon_property_values(goods_factory, GAP_ANIMA_BIND)
                || goods.addon_property_value(goods_factory, GAP_ANIMA_BIND, 1) == 0
            {
                continue;
            }
            let key = goods.addon_property_value(goods_factory, GAP_ANIMA_BIND, 2);
            let mut level = goods
                .addon_property_value(goods_factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
                as u8;
            if level > 99 {
                level = level.wrapping_add(10);
            }
            active_levels
                .entry(key)
                .and_modify(|current| *current = (*current).max(level))
                .or_insert(level);
        }
        for (column, goods) in self.equipment.traversing_goods() {
            apply_equipment_goods_properties(
                &mut properties,
                goods,
                goods_factory,
                coefficients,
                occupation,
                true,
                active_levels.get(&(column.position() as i32)).copied(),
            );
        }
        if include_ci_qing {
            for position in 0..self.ci_qing.size() {
                let Some(goods) = self.ci_qing.get_goods(position) else {
                    continue;
                };
                apply_equipment_goods_properties(
                    &mut properties,
                    goods,
                    goods_factory,
                    coefficients,
                    occupation,
                    true,
                    active_levels.get(&(position as i32)).copied(),
                );
            }
        }
        properties
    }

    /// Мутирующая prelude исходного `CPlayer::UpdateProperty`: slot 10
    /// пересчитывает производные battle-fairy addon-ы до `MountAllEquip`.
    /// Значения `2` остаются instance-modifier-ами, а текущие HP/MP здесь
    /// намеренно не зажимаются — native owner обновляет только максимумы.
    /// Шесть FISTP-конверсий усекают полную сумму к нулю.
    pub(crate) fn refresh_battle_fairy_equipment_properties(
        &mut self,
        factory: &CGoodsFactory,
    ) {
        let Some(goods) = self.equipment.get_goods_mut(10) else {
            return;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 0 {
            return;
        }
        let level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1);
        let pullulate = goods.addon_property_value(factory, GAP_BF_PULLULATERATE, 1) as f32;
        let growth = (level.wrapping_sub(1) as f32) * pullulate * 0.0001_f32 + 1.0_f32;
        for (value_property, base_property, potential_property) in [
            (GAP_BF_BRAVE, GAP_BF_BRAVE_BASE, GAP_BF_BRAVE_POTENTIAL),
            (
                GAP_BF_AGILITY,
                GAP_BF_AGILITY_BASE,
                GAP_BF_AGILITY_POTENTIAL,
            ),
            (
                GAP_BF_SPRITUALISM,
                GAP_BF_SPRITUALISM_BASE,
                GAP_BF_SPRITUALISM_POTENTIAL,
            ),
            (
                GAP_BF_STRENGH,
                GAP_BF_STRENGH_BASE,
                GAP_BF_STRENGH_POTENTIAL,
            ),
            (
                GAP_BF_ATTACK,
                GAP_BF_ATTACK_BASE,
                GAP_BF_ATTACK_POTENTIAL,
            ),
            (
                GAP_BF_SPRITE,
                GAP_BF_SPRITE_BASE,
                GAP_BF_SPRITE_POTENTIAL,
            ),
        ] {
            let base = goods.addon_property_value(factory, base_property, 1) as f32;
            let potential = goods.addon_property_value(factory, potential_property, 1) as f32;
            let modifier = goods.addon_property_value(factory, value_property, 2) as f32;
            let value = (modifier + potential + base * growth).trunc() as i32;
            let _ = goods.set_addon_property_value_core(value_property, 1, value);
        }
        let blast = goods
            .addon_property_value(factory, GAP_BF_BLAST_POTENTIAL, 1)
            .wrapping_add(goods.addon_property_value(factory, GAP_BF_BLAST, 2));
        let _ = goods.set_addon_property_value_core(GAP_BF_BLAST, 1, blast);
        let maximum_hp = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
        let maximum_mp = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, maximum_hp);
        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, maximum_mp);
        let _ = goods.set_instance_addon_modifier(GAP_BF_CUT_HURT_SCALE, 1, 0);
    }

    /// `MountEquip` cases `0x9B/0x9C/0x9E..0xA1` после battle-fairy prelude.
    /// Текущие BF-атрибуты хранятся в масштабе 1/10000; нулевое производное
    /// значение восстанавливает base addon-ы, как native positive pass. Все
    /// FISTP-преобразования используют truncate полной суммы с live property.
    pub(crate) fn apply_battle_fairy_equipment_properties(
        &mut self,
        mut properties: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        fn add_u32(target: &mut u32, delta: i64) {
            *target = (i64::from(*target) + delta).clamp(0, i64::from(i32::MAX)) as u32;
        }
        fn add_scaled_u32(target: &mut u32, delta: f64) {
            let value = (f64::from(*target) + delta).trunc() as i64;
            *target = value.clamp(0, i64::from(i32::MAX)) as u32;
        }
        fn add_scaled_i32(target: &mut i32, delta: f64) {
            let value = (f64::from(*target) + delta).trunc() as i64;
            *target = value.clamp(0, i64::from(i32::MAX)) as i32;
        }
        fn add_scaled_u16(target: &mut u16, delta: f64) {
            let value = (f64::from(*target) + delta).trunc() as i64;
            *target = if value < 0 { 0 } else { value as u16 };
        }
        fn truncated_product(value: i32, coefficient: f32) -> i32 {
            (f64::from(value) * f64::from(coefficient)).trunc() as i32
        }
        fn scaled(value: i32) -> f64 {
            f64::from(value) * 0.0001_f64
        }

        let occupation = usize::from(self.base_properties.occupation).min(2);
        let Some(goods) = self.equipment.get_goods_mut(10) else {
            return properties;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 0 {
            return properties;
        }
        let enabled = goods.enabled_addon_properties(factory);
        for property in enabled {
            match property {
                GAP_BF_ATTACK => {
                    if goods.addon_property_value(factory, GAP_BF_ATTACK, 1) == 0 {
                        let base = goods.addon_property_value(factory, GAP_BF_ATTACK_BASE, 1);
                        let _ = goods.set_addon_property_value_core(GAP_BF_ATTACK, 1, base);
                    }
                }
                GAP_BF_SPRITE => {
                    if goods.addon_property_value(factory, GAP_BF_SPRITE, 1) == 0 {
                        let base = goods.addon_property_value(factory, GAP_BF_SPRITE_BASE, 1);
                        let _ = goods.set_addon_property_value_core(GAP_BF_SPRITE, 1, base);
                    }
                }
                GAP_BF_BRAVE => {
                    let base = goods.addon_property_value(factory, GAP_BF_BRAVE_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_BRAVE, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_brave_to_player);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_brave_to_player);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_BRAVE, 1, base);
                        add_u32(&mut properties.strength, i64::from(base_effect));
                        continue;
                    }
                    if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                        continue;
                    }
                    let effect = scaled(current_effect);
                    add_scaled_u32(&mut properties.strength, effect);
                    add_scaled_u32(
                        &mut properties.maximum_attack,
                        effect * f64::from(coefficients.str_to_max_attack[occupation]),
                    );
                    add_scaled_u16(
                        &mut properties.burden,
                        effect * f64::from(coefficients.str_to_burden[occupation]),
                    );
                }
                GAP_BF_AGILITY => {
                    let base = goods.addon_property_value(factory, GAP_BF_AGILITY_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_AGILITY, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_agility_to_player);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_agility_to_player);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_AGILITY, 1, base);
                        add_u32(&mut properties.dexterity, i64::from(base_effect));
                        continue;
                    }
                    if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                        continue;
                    }
                    let effect = scaled(current_effect);
                    add_scaled_u32(&mut properties.dexterity, effect);
                    add_scaled_u32(
                        &mut properties.minimum_attack,
                        effect * f64::from(coefficients.dex_to_min_attack[occupation]),
                    );
                    add_scaled_u16(
                        &mut properties.reank,
                        effect * f64::from(coefficients.dex_to_stiff[occupation]),
                    );
                }
                GAP_BF_SPRITUALISM => {
                    let base = goods.addon_property_value(factory, GAP_BF_SPRITUALISM_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_spiritualism_to_player);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_spiritualism_to_player);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_SPRITUALISM, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, base);
                        add_u32(&mut properties.intelligence, i64::from(base_effect));
                        continue;
                    }
                    if goods.addon_property_value(factory, GAP_BF_HP, 1) == 0 {
                        continue;
                    }
                    let effect = scaled(current_effect);
                    add_scaled_u32(&mut properties.intelligence, effect);
                    add_scaled_i32(
                        &mut properties.element_modify,
                        effect * f64::from(coefficients.int_to_element[occupation]),
                    );
                    add_scaled_u32(
                        &mut properties.maximum_mp,
                        effect * f64::from(coefficients.int_to_max_mp[occupation]),
                    );
                    add_scaled_u32(
                        &mut properties.element_resistance,
                        effect * f64::from(coefficients.int_to_resistant[occupation]),
                    );
                    clamp_battle_fairy_current(goods, factory, GAP_BF_MP, GAP_BF_MAX_MP);
                }
                GAP_BF_STRENGH => {
                    let base = goods.addon_property_value(factory, GAP_BF_STRENGH_BASE, 1);
                    let current = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
                    let base_effect =
                        truncated_product(base, coefficients.battle_fairy_strength_to_hp);
                    let current_effect =
                        truncated_product(current, coefficients.battle_fairy_strength_to_hp);
                    if current_effect == 0 {
                        let _ = goods.set_addon_property_value_core(GAP_BF_STRENGH, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, base);
                        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, base);
                        add_u32(&mut properties.maximum_hp, i64::from(base_effect));
                    } else if goods.addon_property_value(factory, GAP_BF_HP, 1) != 0 {
                        add_scaled_u32(&mut properties.maximum_hp, scaled(current_effect));
                    } else {
                        continue;
                    }
                    clamp_battle_fairy_current(goods, factory, GAP_BF_HP, GAP_BF_MAX_HP);
                }
                _ => {}
            }
        }
        properties
    }

    /// Два снимка `UpdateCiQingProperty` должны видеть один и тот же
    /// pre-Mount BF goods state. Первый property-pass выполняется на clone,
    /// второй оставляет canonical zero-init/clamp mutations в slot 10.
    pub(crate) fn apply_battle_fairy_equipment_property_pair(
        &mut self,
        previous: PlayerCombatProperties,
        current: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        factory: &CGoodsFactory,
    ) -> (PlayerCombatProperties, PlayerCombatProperties) {
        let saved = self.equipment.get_goods(10).cloned();
        let previous = self.apply_battle_fairy_equipment_properties(
            previous,
            coefficients,
            factory,
        );
        if let (Some(saved), Some(goods)) = (saved, self.equipment.get_goods_mut(10)) {
            *goods = saved;
        }
        let current = self.apply_battle_fairy_equipment_properties(
            current,
            coefficients,
            factory,
        );
        (previous, current)
    }

    /// Полный `MountAllEquip` строит два снимка из одного BF goods state,
    /// заменяет `m_mapCiQingAddValue` разностью и передаёт caller-у итог для
    /// обязательного `SendResultToClient` до `OnChangeProperties`.
    pub(crate) fn recompute_update_property(
        &mut self,
        coefficients: GlobePlayerPropertyCoefficients,
        base_combat_scales: [f32; 5],
        critical_rate: f32,
        factory: &CGoodsFactory,
    ) -> PlayerPropertyRecompute {
        self.equipment_changed = true;
        self.refresh_battle_fairy_equipment_properties(factory);
        self.apply_ci_qing_base_properties(factory);
        let previous = self.recompute_without_ci_qing_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            factory,
        );
        let current = self.recompute_base_and_equipment_properties(
            coefficients,
            base_combat_scales,
            critical_rate,
            factory,
        );
        let (previous, current) = self.apply_battle_fairy_equipment_property_pair(
            previous,
            current,
            coefficients,
            factory,
        );
        if let Some(values) = Self::update_ci_qing_property_difference(
            &Self::combat_type_values_from(previous),
            &Self::combat_type_values_from(current),
        ) {
            self.ci_qing_add_values = values;
        }
        PlayerPropertyRecompute {
            properties: current,
            ci_qing_result_values: self.ci_qing_property_result(),
        }
    }


    pub(crate) fn defense_shields(
        &self,
    ) -> impl Iterator<Item = &super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.defense_shields()
    }

    pub(crate) fn defense_shield_key(&self, skill_id: u32) -> Option<super::moveshape::StateKey> {
        self.move_shape.defense_shield_key(skill_id)
    }

    pub(crate) fn defense_shield(
        &self,
        key: super::moveshape::StateKey,
    ) -> Option<&super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.defense_shield(key)
    }

    pub(crate) fn remove_defense_shield_key(
        &mut self,
        key: super::moveshape::StateKey,
    ) -> Option<super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.remove_defense_shield_key(key)
    }

    pub(crate) fn remove_defense_shield(
        &mut self,
        skill_id: u32,
    ) -> Option<super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.remove_defense_shield(skill_id)
    }


    pub(crate) fn take_defense_shields(
        &mut self,
    ) -> super::moveshape::StateBatch<super::skills::shieldstate::DefenseShieldState> {
        self.move_shape.take_defense_shields()
    }

    pub(crate) fn restore_defense_shields(
        &mut self,
        states: super::moveshape::StateBatch<super::skills::shieldstate::DefenseShieldState>,
    ) {
        self.move_shape.restore_defense_shields(states);
    }

    pub(crate) fn cure_state(&self) -> Option<super::skills::curestate::CureState> {
        self.move_shape.cure_state()
    }

    pub(crate) fn take_cure_state(&mut self) -> Option<super::skills::curestate::CureState> {
        self.move_shape.take_cure_state()
    }



    pub(crate) fn curable_state_ids(&self) -> Vec<u32> {
        self.move_shape.curable_state_ids()
    }

















    pub(crate) fn replace_spider_web_state(
        &mut self,
        state: super::skills::spiderwebstate::SpiderWebState,
    ) -> Option<super::skills::spiderwebstate::SpiderWebState> {
        self.move_shape.replace_spider_web_state(state)
    }




    pub(crate) fn take_boss_blue_fury_state(
        &mut self,
    ) -> Option<super::skills::bossbluefurystate::BossBlueFuryState> {
        self.move_shape.take_boss_blue_fury_state()
    }

    pub(crate) fn begin_boss_blue_fury_state(
        &mut self,
        state: super::skills::bossbluefurystate::BossBlueFuryState,
    ) {
        self.move_shape.begin_boss_blue_fury_state(state);
    }




    pub(crate) fn take_spider_web_state(
        &mut self,
    ) -> Option<super::skills::spiderwebstate::SpiderWebState> {
        self.move_shape.take_spider_web_state()
    }









    pub(crate) fn pillar_state(&self) -> Option<super::skills::pillarstate::PillarState> {
        self.move_shape.pillar_state()
    }



    pub(crate) fn blind_state_order(&self) -> Vec<u32> {
        self.move_shape.blind_state_order()
    }


    pub(crate) fn take_expired_battle_fairy_attribute_state(&mut self, key: crate::gameserver::appserver::moveshape::StateKey, now_ms: u32) -> Option<super::skills::battlefairyattributestate::BattleFairyAttributeState> {
        self.move_shape.take_expired_battle_fairy_attribute_state(key, now_ms)
    }








    pub(crate) fn script_move_state_count(&self, state_id: i32) -> u32 {
        self.move_shape.state_count_by_state_id(state_id)
    }








    pub(crate) const fn is_auto_protected(&self) -> bool {
        self.auto_protected
    }

    pub(crate) const fn set_auto_protected(&mut self, value: bool) {
        self.auto_protected = value;
    }

    pub(crate) fn improve_experience_multiplier(&self) -> f64 {
        self.move_shape
            .script_states()
            .fold(1.0_f64, |multiplier, state| {
                multiplier + super::scriptstate::script_experience_multiplier_delta(state)
            })
    }

    pub(crate) const fn fight_state_count(&self) -> i32 {
        self.fight_state_count
    }

    /// Exact `EnterPeaceState` state half: virtual `CShape::SetState(0)` and
    /// fight countdown reset precede the caller-owned around publication.
    pub(crate) fn enter_peace_state(&mut self) -> PlayerFightStateTransition {
        let previous_count = self.fight_state_count;
        self.movement_shape_mut().set_state(0);
        self.fight_state_count = 0;
        PlayerFightStateTransition {
            player_id: self.player_id(),
            previous_count,
            current_count: 0,
            entered_peace: true,
        }
    }

    /// Exact `OnBeginSkill -> EnterCombatState`: base defense — единственное
    /// исключение; concrete skill caller уже отфильтровал его. Countdown
    /// хранится в simulation frames (`g_ms == 80`), around `0xBF607` остаётся
    /// у CGame рядом с live region transport.
    pub(crate) fn enter_combat_state(
        &mut self,
        fight_state_timer_ms: i32,
    ) -> PlayerFightStateTransition {
        let previous_count = self.fight_state_count;
        self.movement_shape_mut().set_state(1);
        self.fight_state_count = fight_state_timer_ms / 80;
        PlayerFightStateTransition {
            player_id: self.player_id(),
            previous_count,
            current_count: self.fight_state_count,
            entered_peace: false,
        }
    }

    pub(crate) const fn can_fight(&self) -> bool {
        self.move_shape.can_fight()
    }

    /// Reached `UpdateCurrentState` combat half. Только положительный counter
    /// декрементируется; переход через zero исполняет exact peace mutation.
    pub(crate) fn update_fight_state(&mut self) -> Option<PlayerFightStateTransition> {
        if self.fight_state_count <= 0 {
            return None;
        }
        let previous_count = self.fight_state_count;
        self.fight_state_count = self.fight_state_count.wrapping_sub(1);
        if self.fight_state_count < 1 {
            self.movement_shape_mut().set_state(0);
            self.fight_state_count = 0;
            return Some(PlayerFightStateTransition {
                player_id: self.player_id(),
                previous_count,
                current_count: 0,
                entered_peace: true,
            });
        }
        Some(PlayerFightStateTransition {
            player_id: self.player_id(),
            previous_count,
            current_count: self.fight_state_count,
            entered_peace: false,
        })
    }

    /// Exact delayed `OnLost` timestamp. Native formula deliberately keeps
    /// signed/wrapping intermediate arithmetic with fixed process `g_ms=80`.
    pub(crate) fn begin_lost_delay(
        &mut self,
        now_ms: u32,
        fight_state_timer_ms: i32,
    ) -> Option<PlayerLostDelayStarted> {
        if self.fight_state_count <= 0 {
            return None;
        }
        let remaining = self
            .fight_state_count
            .wrapping_mul(80)
            .wrapping_sub(fight_state_timer_ms);
        self.lost_time_stamp_ms = now_ms.wrapping_add(remaining as u32);
        Some(PlayerLostDelayStarted {
            player_id: self.player_id(),
            fight_state_count: self.fight_state_count,
            timestamp_ms: self.lost_time_stamp_ms,
        })
    }

    pub(crate) const fn lost_delay_due(&self, now_ms: u32, fight_state_timer_ms: i32) -> bool {
        self.lost_time_stamp_ms != 0
            && self
                .lost_time_stamp_ms
                .wrapping_add(fight_state_timer_ms as u32)
                <= now_ms
    }

    pub(crate) const fn has_lost_delay(&self) -> bool {
        self.lost_time_stamp_ms != 0
    }

    /// CRideState::AI (0x004F9110): listener сначала обходит весь packet
    /// с GAP_MOUNT_TYPE != 0, затем первый совпавший type/level завершает поиск.
    /// Имя, role limit, предыдущий GUID и локальный StateKey здесь не участвуют.
    pub(crate) fn has_ride_goods(
        &self,
        mount_type: u32,
        level: u32,
        factory: &CGoodsFactory,
    ) -> bool {
        let packet = self.packet.base();
        let mut listener = GoodsParticularPropertyListener::new(GAP_MOUNT_TYPE);
        for goods in packet.traversing_goods() {
            listener.visit(factory, goods);
        }
        listener.goods_ids().iter().any(|goods_id| {
            packet.find(*goods_id).is_some_and(|goods| {
                goods.addon_property_value(factory, GAP_MOUNT_TYPE, 1) as u32 == mount_type
                    && goods.addon_property_value(factory, GAP_MOUNT_LEVEL, 1) as u32 == level
            })
        })
    }

    fn ride_goods(
        &self,
        state: &super::ridestate::RideState,
        factory: &CGoodsFactory,
    ) -> Option<&CGoods> {
        let base_index = factory.query_goods_id_by_original_name(Some(state.goods_name()));
        self.packet
            .base()
            .traversing_goods()
            .find(|goods| goods.base_properties_index() == base_index)
    }

    /// `CNotDisappearAfterDead::OnUpdateProperties` применяет прямые поля до
    /// базовых характеристик, а отрицательные STR/DEX/CON/INT проецирует как
    /// разность производных старого и нового значения. IMUL сохраняет low32
    /// до unsigned /100; каждый native setter ограничивает DWORD INT_MAX.
    /// X87-преобразования усекают дробную часть; в отрицательных HP/MP/DEF-ветках сохраняется
    /// исходное знаковое WORD-сужение. Поле usage `20_001` соответствует
    /// `tagProperty.wHit +0x24`, а не соседнему `wAtcSpeed +0x32`.
    pub(crate) fn apply_undead_state_properties(
        &self,
        mut properties: PlayerCombatProperties,
        coefficients: GlobePlayerPropertyCoefficients,
        state: &super::moveshape::UndeadState,
    ) -> PlayerCombatProperties {
        fn clamp_wrapped_min_one(value: u32) -> u32 {
            if (value as i32) < 1 { 1 } else { value }
        }

        fn set_property(value: u32) -> u32 {
            value.min(i32::MAX as u32)
        }

        fn trunc_product(value: f64, coefficient: f32) -> i32 {
            super::skills::fightdefense::truncate_original(
                (value * f64::from(coefficient)).trunc(),
            )
        }

        fn ftol_word_product(value: f64, coefficient: f32) -> i16 {
            let value = (value * f64::from(coefficient)).trunc();
            // Только отрицательная INT→MP ветвь вызывает __ftol2 (FISTP64),
            // а затем использует AX. Integer-indefinite поэтому даёт WORD 0.
            if !value.is_finite()
                || value < -9_223_372_036_854_775_808.0
                || value >= 9_223_372_036_854_775_808.0
            {
                0
            } else {
                value as i64 as i16
            }
        }

        fn direct_value(current: u32, value: i16, percentage: bool) -> u32 {
            if value < 0 {
                let decrement = if percentage {
                    (current.wrapping_mul(value.wrapping_neg() as i32 as u32) / 100) as i16
                } else {
                    value.wrapping_neg()
                };
                let narrowed = (current as i16).wrapping_sub(decrement);
                if narrowed < 1 { 1 } else { narrowed as u32 }
            } else if value > 0 {
                let increment = if percentage {
                    current.wrapping_mul(value as u32) / 100
                } else {
                    value as u32
                };
                set_property(current.wrapping_add(increment))
            } else {
                current
            }
        }

        fn changed_stat(current: u32, value: i32, percentage: bool) -> (u32, u32, bool) {
            if value < 0 {
                let decrement = if percentage {
                    current.wrapping_mul(value.wrapping_neg() as u32) / 100
                } else {
                    value.wrapping_neg() as u32
                };
                let new = current.wrapping_sub(decrement);
                (clamp_wrapped_min_one(new), decrement, false)
            } else {
                let increment = if percentage {
                    current.wrapping_mul(value as u32) / 100
                } else {
                    value as u32
                };
                (set_property(current.wrapping_add(increment)), increment, true)
            }
        }

        fn add_positive_derived(current: u32, delta: f64, coefficient: f32) -> u32 {
            set_property(current.wrapping_add(trunc_product(delta, coefficient) as u32))
        }

        fn add_derived_difference(current: u32, old: f64, new: f64, coefficient: f32) -> u32 {
            let difference = trunc_product(new, coefficient)
                .wrapping_sub(trunc_product(old, coefficient));
            clamp_wrapped_min_one(current.wrapping_add(difference as u32))
        }

        fn add_short_derived_difference(
            current: u32,
            old: f64,
            new: f64,
            coefficient: f32,
        ) -> u32 {
            let difference = (trunc_product(new, coefficient) as i16)
                .wrapping_sub(trunc_product(old, coefficient) as i16);
            let narrowed = (current as i16).wrapping_add(difference);
            if narrowed < 1 { 1 } else { narrowed as u32 }
        }

        let occupation = usize::from(self.base_properties.occupation).min(2);
        properties.maximum_hp =
            direct_value(properties.maximum_hp, state.maximum_hp, state.percentage);
        properties.maximum_mp =
            direct_value(properties.maximum_mp, state.maximum_mp, state.percentage);
        properties.defense = direct_value(properties.defense, state.defense, state.percentage);
        properties.element_resistance = direct_value(
            properties.element_resistance,
            state.element_resistance,
            state.percentage,
        );

        if state.strength != 0 {
            let old = properties.strength;
            let (new, delta, positive) = changed_stat(old, state.strength, state.percentage);
            properties.strength = new;
            properties.maximum_attack = if positive {
                add_positive_derived(
                    properties.maximum_attack,
                    f64::from(delta),
                    coefficients.str_to_max_attack[occupation],
                )
            } else {
                add_derived_difference(
                    properties.maximum_attack,
                    f64::from(old),
                    f64::from(new),
                    coefficients.str_to_max_attack[occupation],
                )
            };
        }

        if state.dexterity != 0 {
            let old = properties.dexterity;
            let (new, delta, positive) = changed_stat(old, state.dexterity, state.percentage);
            properties.dexterity = new;
            properties.minimum_attack = if positive {
                add_positive_derived(
                    properties.minimum_attack,
                    f64::from(delta),
                    coefficients.dex_to_min_attack[occupation],
                )
            } else {
                add_derived_difference(
                    properties.minimum_attack,
                    f64::from(old),
                    f64::from(new),
                    coefficients.dex_to_min_attack[occupation],
                )
            };
        }

        if state.constitution != 0 {
            let old = properties.constitution;
            let (new, delta, positive) =
                changed_stat(old, state.constitution, state.percentage);
            properties.constitution = new;
            if positive {
                let projected = if state.percentage { f64::from(delta as f32) } else { f64::from(delta) };
                properties.maximum_hp = add_positive_derived(
                    properties.maximum_hp,
                    projected,
                    coefficients.con_to_max_hp[occupation],
                );
                properties.defense = add_positive_derived(
                    properties.defense,
                    projected,
                    coefficients.con_to_defense[occupation],
                );
            } else {
                properties.maximum_hp = add_short_derived_difference(
                    properties.maximum_hp,
                    f64::from(old),
                    f64::from(new),
                    coefficients.con_to_max_hp[occupation],
                );
                properties.defense = add_short_derived_difference(
                    properties.defense,
                    f64::from(old as f32),
                    f64::from(new as f32),
                    coefficients.con_to_defense[occupation],
                );
            }
        }

        if state.intelligence != 0 {
            let old = properties.intelligence;
            let (new, delta, positive) =
                changed_stat(old, state.intelligence, state.percentage);
            properties.intelligence = new;
            if positive {
                // В абсолютной ветке оригинал повторно проецирует уже новое
                // полное INT; процентная ветка проецирует только приращение.
                let projected = if state.percentage { f64::from(delta as f32) } else { f64::from(new) };
                properties.maximum_mp = add_positive_derived(
                    properties.maximum_mp,
                    projected,
                    coefficients.int_to_max_mp[occupation],
                );
                properties.element_resistance = add_positive_derived(
                    properties.element_resistance,
                    projected,
                    coefficients.int_to_resistant[occupation],
                );
                properties.element_modify = properties.element_modify.wrapping_add(
                    trunc_product(projected, coefficients.int_to_element[occupation]),
                );
            } else {
                let old_mp = ftol_word_product(f64::from(old), coefficients.int_to_max_mp[occupation]);
                let new_mp = ftol_word_product(f64::from(new as f32), coefficients.int_to_max_mp[occupation]);
                let maximum_mp = (properties.maximum_mp as i16)
                    .wrapping_add(new_mp.wrapping_sub(old_mp));
                properties.maximum_mp = maximum_mp.max(1) as u32;
                properties.element_resistance = add_derived_difference(
                    properties.element_resistance,
                    f64::from(old as f32),
                    f64::from(new as f32),
                    coefficients.int_to_resistant[occupation],
                );
                let difference = trunc_product(f64::from(new as f32), coefficients.int_to_element[occupation])
                    .wrapping_sub(trunc_product(
                        f64::from(old as f32),
                        coefficients.int_to_element[occupation],
                    ));
                properties.element_modify = properties.element_modify.wrapping_add(difference);
                if properties.element_modify < 1 {
                    properties.element_modify = 1;
                }
            }
        }

        properties.minimum_attack =
            direct_value(properties.minimum_attack, state.minimum_attack, false);
        properties.maximum_attack =
            direct_value(properties.maximum_attack, state.maximum_attack, false);
        if state.element_modify < 0 {
            let narrowed = (properties.element_modify as i16).wrapping_add(state.element_modify);
            properties.element_modify = if narrowed < 1 { 1 } else { i32::from(narrowed) };
        } else {
            properties.element_modify = properties
                .element_modify
                .wrapping_add(i32::from(state.element_modify));
        }
        properties.blast_attack = properties
            .blast_attack
            .wrapping_add(state.blast_attack as u16);
        properties.blast_element_attack = properties
            .blast_element_attack
            .wrapping_add(state.blast_element_attack as u16);
        properties.cch = properties.cch.wrapping_add(state.cch as u16);
        properties.full_miss = properties.full_miss.wrapping_add(state.full_miss as u16);
        properties.attack_avoid = properties
            .attack_avoid
            .wrapping_add(state.attack_avoid as u16);
        properties.element_avoid = properties
            .element_avoid
            .wrapping_add(state.element_avoid as u16);
        properties.hit = properties.hit.wrapping_add(state.hit as u16);
        properties.dodge = properties.dodge.wrapping_add(state.dodge as u16);
        properties
    }

    pub(crate) fn apply_extended_state_properties(
        mut properties: PlayerCombatProperties,
        state: &super::exstate::ExtendedState,
    ) -> PlayerCombatProperties {
        let add = |target: &mut u32, value: u16| {
            if value != 0 {
                *target = target.wrapping_add(u32::from(value)).min(i32::MAX as u32);
            }
        };
        add(&mut properties.maximum_hp, state.maximum_hp);
        add(&mut properties.maximum_mp, state.maximum_mp);
        add(&mut properties.minimum_attack, state.minimum_attack);
        add(&mut properties.maximum_attack, state.maximum_attack);
        add(&mut properties.defense, state.defense);
        add(&mut properties.element_resistance, state.element_resistance);
        properties.element_modify = properties
            .element_modify
            .wrapping_add(i32::from(state.element_modify));
        properties.cch = properties.cch.wrapping_add(state.cch);
        properties.full_miss = properties.full_miss.wrapping_add(state.full_miss);
        properties.attack_avoid = properties.attack_avoid.wrapping_add(state.attack_avoid);
        properties.element_avoid = properties.element_avoid.wrapping_add(state.element_avoid);
        properties.hit = properties.hit.wrapping_add(state.hit);
        properties.dodge = properties.dodge.wrapping_add(state.dodge);
        properties
    }

    pub(crate) fn apply_active_change_body_state_properties(
        mut properties: PlayerCombatProperties,
        state: &super::chbystate::ChangeBodyState,
    ) -> PlayerCombatProperties {
        let add = |target: &mut u32, value: u32| {
            if value != 0 {
                *target = target.wrapping_add(value).min(i32::MAX as u32);
            }
        };
        add(&mut properties.maximum_hp, state.maximum_hp);
        add(&mut properties.maximum_mp, state.maximum_mp);
        add(&mut properties.minimum_attack, state.minimum_attack);
        add(&mut properties.maximum_attack, state.maximum_attack);
        add(&mut properties.defense, state.defense);
        add(&mut properties.element_resistance, state.element_resistance);
        if state.cch != 0 {
            properties.cch = properties.cch.wrapping_add(state.cch);
        }
        if state.blast_attack != 0 {
            properties.blast_attack = properties.blast_attack.wrapping_add(state.blast_attack);
        }
        if state.blast_element_attack != 0 {
            properties.blast_element_attack = properties
                .blast_element_attack
                .wrapping_add(state.blast_element_attack);
        }
        properties
    }

    /// CRideState::OnUpdateProperties0x004F9000 выбирает первый packet goods
    /// по имени именно этого экземпляра, не по AI-cache. Общий расчёт делает
    /// MountEquipRide(true), затем(false), не копируя state или CGoods.
    pub(crate) fn apply_ride_state_properties(
        &self,
        mut properties: PlayerCombatProperties,
        state: &super::ridestate::RideState,
        coefficients: GlobePlayerPropertyCoefficients,
        goods_factory: &CGoodsFactory,
    ) -> PlayerCombatProperties {
        if let Some(goods) = self.ride_goods(state, goods_factory) {
            apply_equipment_goods_properties(
                &mut properties,
                goods,
                goods_factory,
                coefficients,
                usize::from(self.base_properties.occupation).min(2),
                false,
                None,
            );
        }
        properties
    }


    pub(crate) const fn remain_jing_li_dan_count(&self) -> u16 {
        self.base_properties.remain_jing_li_dan_count
    }

    pub(crate) const fn set_remain_jing_li_dan_count(&mut self, count: u16) {
        self.base_properties.remain_jing_li_dan_count = count;
    }

    pub(crate) fn get_extended_state(
        &self,
        kind: super::exstate::ExtendedStateKind,
        state_id: u32,
    ) -> u32 {
        self.move_shape.get_extended_state(kind, state_id)
    }



    pub(crate) fn realm_appellation_bonus_identity(
        &self,
    ) -> Option<super::skills::realmappellation::RealmBonusIdentity> {
        (self.realm_appellation_skill_id != UNKNOWN_SKILL_ID
            && (1..=4).contains(&self.realm_appellation_skill_level))
        .then_some(super::skills::realmappellation::RealmBonusIdentity {
            skill_id: self.realm_appellation_skill_id,
            level: self.realm_appellation_skill_level,
        })
    }

    pub(crate) const fn set_realm_appellation_bonus_identity(&mut self, skill_id: u32, level: i32) {
        self.realm_appellation_skill_id = skill_id;
        self.realm_appellation_skill_level = level;
    }

    pub(crate) fn realm_appellation_entitled(&self, appellation_id: u32, factory: &CSkillFactory) -> bool {
        super::skills::realmappellation::is_title(appellation_id)
            && self
                .move_shape
                .skill(appellation_id, factory)
                .is_some_and(|skill| skill.level() > 0)
    }

    /// Достигнутая часть единого `m_mapNameValue/GetScriptValue` catalog.
    /// DWORD возвращаются теми же битами в signed script integer; неизвестное
    /// имя остаётся `None`, а сам GetMe преобразует его в legacy zero.
    pub(crate) fn script_value(&self, property: &[u8]) -> Option<i32> {
        if property.eq_ignore_ascii_case(b"lRegionID") {
            Some(self.server_region_id().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lID") {
            Some(self.player_id())
        } else if property.eq_ignore_ascii_case(b"lTileX") {
            Some(self.shape().get_tile_x().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lTileY") {
            Some(self.shape().get_tile_y().unwrap_or_default())
        } else if property.eq_ignore_ascii_case(b"lDir") {
            Some(self.shape().get_direction())
        } else if property.eq_ignore_ascii_case(b"wState") {
            Some(i32::from(self.shape().get_state()))
        } else if property.eq_ignore_ascii_case(b"wAction") {
            Some(i32::from(self.shape().get_action()))
        } else if property.eq_ignore_ascii_case(b"btCountry") {
            Some(i32::from(self.country()))
        } else if property.eq_ignore_ascii_case(b"lPos") {
            Some(self.shape().get_position())
        } else if property.eq_ignore_ascii_case(b"dwVigour") {
            Some(self.vigour() as i32)
        } else if property.eq_ignore_ascii_case(b"lLevel") {
            Some(i32::from(self.level()))
        } else if property.eq_ignore_ascii_case(b"lSex") {
            Some(i32::from(self.base_properties.sex))
        } else if property.eq_ignore_ascii_case(b"dwExp") {
            Some(self.experience() as i32)
        } else if property.eq_ignore_ascii_case(b"wPkCount") {
            Some(i32::from(self.pk_count()))
        } else if property.eq_ignore_ascii_case(b"lOccupation") {
            Some(i32::from(self.occupation()))
        } else if property.eq_ignore_ascii_case(b"dwAppellationID") {
            Some(self.base_properties.appellation_id as i32)
        } else if property.eq_ignore_ascii_case(b"dwRankOfNobilityID") {
            Some(self.base_properties.rank_of_nobility_id as i32)
        } else if property.eq_ignore_ascii_case(b"dwCredit") {
            Some(self.base_properties.credit as i32)
        } else if property.eq_ignore_ascii_case(b"dwSZL") {
            Some(self.base_properties.szl as i32)
        } else if property.eq_ignore_ascii_case(b"lContribute") {
            Some(self.contribution)
        } else if property.eq_ignore_ascii_case(b"bFairyContainerEnabled") {
            Some(i32::from(self.base_properties.fairy_container_enabled))
        } else if property.eq_ignore_ascii_case(b"bBattleFairyEnabled") {
            Some(i32::from(self.base_properties.battle_fairy_enabled))
        } else {
            None
        }
    }

    /// Достигнутая writable-часть того же `m_mapNameValue/SetValue` catalog.
    /// Узкие поля сохраняют исходное integer narrowing, DWORD — все биты.
    pub(crate) fn set_script_value(&mut self, property: &[u8], value: i32) -> Option<i32> {
        if property.eq_ignore_ascii_case(b"btCountry") {
            Some(self.set_script_country(value))
        } else if property.eq_ignore_ascii_case(b"dwVigour") {
            Some(self.set_script_vigour(value))
        } else if property.eq_ignore_ascii_case(b"lLevel") {
            self.base_properties.level = value as u8;
            Some(i32::from(self.base_properties.level))
        } else if property.eq_ignore_ascii_case(b"lSex") {
            self.base_properties.sex = value as u8;
            Some(i32::from(self.base_properties.sex))
        } else if property.eq_ignore_ascii_case(b"dwExp") {
            Some(self.set_script_experience(value))
        } else if property.eq_ignore_ascii_case(b"lOccupation") {
            self.base_properties.occupation = value as u8;
            Some(i32::from(self.base_properties.occupation))
        } else if property.eq_ignore_ascii_case(b"dwAppellationID") {
            self.base_properties.appellation_id = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwRankOfNobilityID") {
            self.base_properties.rank_of_nobility_id = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwCredit") {
            self.base_properties.credit = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"dwSZL") {
            self.base_properties.szl = value as u32;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"lContribute") {
            self.contribution = value;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"bFairyContainerEnabled") {
            self.base_properties.fairy_container_enabled = value != 0;
            Some(value)
        } else if property.eq_ignore_ascii_case(b"bBattleFairyEnabled") {
            self.base_properties.battle_fairy_enabled = value != 0;
            Some(value)
        } else {
            None
        }
    }

    /// `ChangeValue` применяет wrapping arithmetic ширины фактического поля.
    /// Shipped GM-script использует исторический алиас `Experience` для
    /// canonical `dwExp`.
    pub(crate) fn change_script_value(&mut self, property: &[u8], delta: i32) -> Option<i32> {
        let canonical = if property.eq_ignore_ascii_case(b"Experience") {
            b"dwExp".as_slice()
        } else {
            property
        };
        let current = self.script_value(canonical)?;
        if canonical.eq_ignore_ascii_case(b"bFairyContainerEnabled")
            || canonical.eq_ignore_ascii_case(b"bBattleFairyEnabled")
        {
            let changed = i32::from(current.wrapping_add(delta) != 0);
            let _ = self.set_script_value(canonical, changed)?;
            return Some(changed);
        }
        self.set_script_value(canonical, current.wrapping_add(delta))
    }

    pub(crate) const fn gods_battle_faction(&self) -> i32 {
        self.base_properties.gods_battle_faction
    }

    pub(crate) const fn level(&self) -> u8 {
        self.base_properties.level
    }

    pub(crate) const fn set_level(&mut self, level: u8) {
        self.base_properties.level = level;
    }

    pub(crate) fn apply_level_property_upgrade(
        &mut self,
        upgrade: &nebokrai_shared::resources::PlayerPropertiesUpgrade,
    ) {
        self.base_properties.base_maximum_hp = upgrade.base_maximum_hp;
        self.base_properties.base_dexterity = upgrade.base_dexterity;
        self.base_properties.base_maximum_mp = upgrade.base_maximum_mp;
        self.base_properties.base_strength = upgrade.base_strength;
        self.base_properties.base_burden = upgrade.base_burden;
        self.base_properties.base_constitution = upgrade.base_constitution;
        self.base_properties.base_intelligence = upgrade.base_intelligence;
    }

    pub(crate) const fn set_base_maximum_rp(&mut self, value: u16) {
        self.base_properties.maximum_rp = value;
    }

    pub(crate) const fn level_wire_properties(&self) -> (u32, u32, u16, u16) {
        (
            self.base_properties.base_maximum_hp,
            self.base_properties.base_maximum_mp,
            self.base_properties.base_burden,
            self.base_properties.maximum_rp,
        )
    }

    pub(crate) const fn energy(&self) -> u32 {
        self.base_properties.energy
    }

    pub(crate) const fn maximum_energy(&self) -> u32 {
        self.base_properties.maximum_energy
    }

    /// Exact `CPlayer::SetEnergy`: unsigned caller arithmetic сохраняется,
    /// затем значение ограничивается текущим `dwMaxEnergy`.
    pub(crate) fn set_energy(&mut self, energy: u32) {
        self.base_properties.energy = energy.min(self.base_properties.maximum_energy);
    }

    /// `CPlayer::SetMaxEnergy` сохраняет новый максимум и сразу ограничивает
    /// им текущее значение энергии.
    pub(crate) fn set_maximum_energy(&mut self, energy: u32) {
        self.base_properties.maximum_energy = energy;
        self.base_properties.energy = self.base_properties.energy.min(energy);
    }

    pub(crate) const fn occupation(&self) -> u8 {
        self.base_properties.occupation
    }

    /// Cross-Game level relay сохраняет `SetLevel` mutation и отдельный
    /// caller-side faction publication; experience сбрасывается после неё.
    pub(crate) fn apply_remote_level(&mut self, level: u8) -> PlayerRemoteLevelMutation {
        let mutation = PlayerRemoteLevelMutation {
            player_id: self.player_id(),
            faction_id: self.faction_id,
            previous_level: self.base_properties.level,
            level,
        };
        self.base_properties.level = level;
        self.base_properties.experience = 0;
        mutation
    }

    pub(crate) const fn szl(&self) -> u32 {
        self.base_properties.szl
    }

    pub(crate) const fn set_szl(&mut self, value: u32) {
        self.base_properties.szl = value;
    }

    pub(crate) const fn attempt_appellation_id(&self) -> u32 {
        self.attempt_appellation_id
    }

    pub(crate) const fn set_gods_battle_faction(&mut self, faction: i32) {
        self.base_properties.gods_battle_faction = faction;
    }

    /// Assembly/load boundary для persisted player tail; faction membership
    /// сам region восстанавливает только после фактического `AddObject`.
    pub(crate) const fn restore_gods_battle_state(&mut self, faction: i32, szl: u32) {
        self.base_properties.gods_battle_faction = faction;
        self.base_properties.szl = szl;
    }

    pub(crate) const fn restore_level_and_attempt_appellation(
        &mut self,
        level: u8,
        attempt_appellation_id: u32,
    ) {
        self.base_properties.level = level;
        self.attempt_appellation_id = attempt_appellation_id;
    }

    /// Exact `SetExploit`: signed CountryParam storage сравнивается как
    /// `unsigned long`, затем значение зажимается только сверху.
    pub(crate) const fn exploit(&self) -> u32 {
        self.base_properties.exploit
    }

    pub(crate) fn set_exploit(&mut self, requested: u32, maximum: i32) -> u32 {
        let previous = self.base_properties.exploit;
        let applied = requested.min(maximum as u32);
        self.base_properties.exploit = applied;
        tracing::trace!(
            player_id = self.player_id(),
            previous,
            requested,
            applied,
            "подвиг игрока изменён"
        );
        applied
    }

    /// Exact `SetValue("dwExploit", value)` из region reward path:
    /// generic property map пишет `DWORD` напрямую и не вызывает `SetExploit` clamp.
    pub(crate) fn set_exploit_property_value(&mut self, requested: u32) {
        let previous = self.base_properties.exploit;
        self.base_properties.exploit = requested;
        tracing::trace!(
            player_id = self.player_id(),
            previous,
            requested,
            "свойство подвига игрока изменено напрямую"
        );
    }

    /// `OnPlayerTimgingStart` отсеивает action `ACT_DIED == 6`
    /// отдельно от health-based `CMoveShape::IsDied`.
    pub(crate) fn can_start_nation_war_timing(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Тот же double guard использует `ServerNationRegion::OnMonsterDamage`.
    pub(crate) fn can_attack_nation_monster(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Focused same-region branch `ChangeRegion`, которую вызывает
    /// `ServerNationRegion::KickOutAllPlayerToReturnPoint`.
    pub(crate) fn prepare_nation_relive(&mut self) {
        self.current_progress = PlayerProgress::None;
        self.recreate_carriage = false;
    }

    pub(crate) const fn movement_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        MoveShapePositionFacts {
            current_hit_points: self.base_properties.health,
            figure: self.figure,
            current_area: None,
            area_width,
            area_height,
        }
    }

    pub(crate) const fn movement_shape_mut(&mut self) -> &mut CShape {
        self.move_shape.shape_mut()
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        &self.move_shape
    }

    pub(crate) const fn move_shape_mut(&mut self) -> &mut CMoveShape {
        &mut self.move_shape
    }

    pub(crate) fn back_stage_skill_id(&self, index: usize) -> Option<u32> {
        self.player_ai.base_ai().back_stage_skill_id(index)
    }

    pub(crate) fn begin_pending_back_stage_skill_ids(&mut self, factory: &CSkillFactory) -> Vec<u32> {
        self.player_ai.base_ai_mut().begin_pending_back_stage_skill_ids()
            .into_iter()
            .filter(|skill_id| self.move_shape.skill(*skill_id, factory).is_some())
            .collect()
    }

    pub(crate) fn auto_start_passive_skills(&mut self) -> usize {
        self.move_shape.auto_start_passive_skills(self.player_ai.base_ai_mut())
    }

    pub(crate) const fn can_process_ai_destination(&self) -> bool {
        !CMoveShape::is_died(self.base_properties.health)
    }

    pub(crate) const fn is_movement_allowed(&self) -> bool {
        self.move_shape.is_moveable()
    }

    pub(crate) fn movement_speed(&self) -> f32 {
        self.shape().get_speed()
    }

    pub(crate) const fn set_skill_moveable(&mut self, moveable: bool) {
        self.move_shape.set_moveable(moveable);
    }

    pub(crate) const fn set_skill_fightable(&mut self, fightable: bool) {
        self.move_shape.set_fightable(fightable);
    }

    pub(crate) fn has_state_by_skill_id(&self, state_id: u32) -> bool {
        self.move_shape.has_state_by_skill_id(state_id)
    }

    /// Клиентский ИИ и сценарные `WalkStep`/`RunStep` используют обычного
    /// владельца движения `CMoveShape`, поэтому сетевой маршрут `0xBF605` и
    /// перестановка в регионе остаются единым действием.
    pub(crate) fn move_step(
        &mut self,
        server_region: &mut CServerRegion,
        destination_x: i32,
        destination_y: i32,
        run: i32,
        area_width: i32,
        area_height: i32,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<(), MoveShapeCommandBlock> {
        let facts = self.movement_position_facts(area_width, area_height);
        self.move_shape.on_move(
            Some(server_region),
            destination_x,
            destination_y,
            run,
            facts,
            around,
        )
    }

    pub(crate) const fn figure(&self) -> ShapeFigure {
        self.figure
    }

    pub(crate) const fn nation_relive_position_facts(
        &self,
        area_width: i32,
        area_height: i32,
    ) -> MoveShapePositionFacts {
        self.movement_position_facts(area_width, area_height)
    }

    pub(crate) const fn nation_relive_shape_mut(&mut self) -> &mut CShape {
        self.movement_shape_mut()
    }

    pub(crate) const fn combat_properties(&self) -> PlayerCombatProperties {
        self.combat_properties
    }

    /// Один concrete OnUpdateProperties меняет живой tagProperty.
    /// Синхронизация wire — чистая проекция полей, без equipment callbacks,
    /// пересчёта остальных состояний или отложенной публикации visual.
    pub(crate) fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    ) {
        self.combat_properties = update(self.combat_properties);
        self.sync_combat_property_wire();
    }

    /// Exact scalar checks `CanUseItem`; catalog/instance addon fallback
    /// остаётся у `CGoods`, а result-коды являются частью `0xBF709` wire.
    pub(crate) fn can_use_item(&self, goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        Self::can_use_item_from_properties(
            self.base_properties,
            self.combat_properties,
            goods,
            factory,
        )
    }

    /// Exact `CPlayer::CanMountEquip`: для headgear сначала согласует persisted
    /// ordinary/battle-fairy enable flags с `GAP_BF_BATTLE_FAIRY`, затем
    /// возвращает те же requirement-коды `1..7` или magic success `9`.
    pub(crate) fn can_mount_equip(&self, goods: &CGoods, factory: &CGoodsFactory) -> i32 {
        Self::can_mount_equip_from_properties(
            self.base_properties,
            self.combat_properties,
            goods,
            factory,
        )
    }

    fn can_mount_equip_from_properties(
        base_properties: PlayerBaseProperties,
        combat_properties: PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> i32 {
        if factory
            .query_goods_base_properties(goods.base_properties_index())
            .is_some_and(|properties| properties.equip_place() == EQUIP_PLACE_HEADGEAR)
        {
            let battle_fairy_headgear =
                goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1;
            if (!base_properties.fairy_container_enabled
                && (!base_properties.battle_fairy_enabled || !battle_fairy_headgear))
                || (base_properties.fairy_container_enabled
                    && !base_properties.battle_fairy_enabled
                    && battle_fairy_headgear)
            {
                return 0;
            }
        }
        Self::can_use_item_from_properties(base_properties, combat_properties, goods, factory)
    }

    fn can_use_item_from_properties(
        base_properties: PlayerBaseProperties,
        combat_properties: PlayerCombatProperties,
        goods: &CGoods,
        factory: &CGoodsFactory,
    ) -> i32 {
        let required = |property| goods.addon_property_value(factory, property, 1) as u32;
        let level = required(GAP_ROLE_MINIMUM_LEVEL_LIMIT);
        if level != 0 && u32::from(base_properties.level) < level {
            return 1;
        }
        for (property, actual, result) in [
            (
                GAP_ROLE_MINIMUM_STRENGTH_LIMIT,
                combat_properties.strength,
                2,
            ),
            (
                GAP_ROLE_MINIMUM_AGILITY_LIMIT,
                combat_properties.dexterity,
                3,
            ),
            (
                GAP_ROLE_MINIMUM_CONSTITUTION_LIMIT,
                combat_properties.constitution,
                4,
            ),
            (
                GAP_ROLE_MINIMUM_WAKAN_LIMIT,
                combat_properties.intelligence,
                5,
            ),
        ] {
            let minimum = required(property);
            if minimum != 0 && actual < minimum {
                return result;
            }
        }
        let occupation = required(GAP_REQUIRE_OCCUPATION);
        if occupation != 0 && occupation != u32::from(base_properties.occupation) + 1 {
            return 6;
        }
        let gender = required(GAP_REQUIRE_GENDER);
        if gender != 0 && gender != u32::from(base_properties.sex) {
            return 7;
        }
        9
    }

    pub(crate) fn item_skill_level(&self, skill_id: u32, factory: &CSkillFactory) -> i32 {
        self.move_shape
            .skill(skill_id, factory)
            .map_or(0, MoveShapeSkill::level)
    }

    pub(crate) fn learn_item_skill(
        &mut self,
        skill_id: u32,
        level: i32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.add_skill(skill_id, level, factory)
    }

    pub(crate) fn set_item_skill_position(&mut self, skill_id: u32, position: i32, factory: &CSkillFactory) -> bool {
        self.move_shape.set_item_skill_position(skill_id, position, factory)
    }

    pub(crate) fn item_skill_position(&self, skill_id: u32, factory: &CSkillFactory) -> Option<i32> {
        self.move_shape.skill(skill_id, factory).map(MoveShapeSkill::item_position)
    }

    /// `CPlayer::ReUseSkillItem` различает отсутствующий map-ключ и сохранённый
    /// нулевой timestamp после переполнения `timeGetTime`.
    pub(crate) fn last_skill_item_use_ms(&self, item_index: u32) -> Option<u32> {
        self.last_skill_item_use_ms.get(&item_index).copied()
    }

    pub(crate) fn mark_skill_item_used(&mut self, item_index: u32, now_ms: u32) {
        self.last_skill_item_use_ms.insert(item_index, now_ms);
    }

    /// `DeleteSkillItem` работает только с сохранённой ячейкой и не ищет
    /// подходящий stack в остальных ячейках пакета.
    pub(crate) fn consume_skill_item_at(
        &mut self,
        position: u32,
        item_index: u32,
        amount: u32,
    ) -> Option<CiQingPacketConsumption> {
        let goods = self.packet.get_goods(position)?;
        if goods.base_properties_index() != item_index || goods.amount() < amount || amount == 0 {
            return None;
        }
        self.remove_packet_goods_by_id(goods.identity().ex_id, amount)
    }

    /// UseItem, tagExpendableEffect 0x4A..4D: часы новой записи находятся в
    /// 0x4545F4/45473B/454887/4549CE, замены — 0x4546BC/454808/45494F/454A8D.
    /// Два value(1) обслуживают разные native reads: payload и live property.
    pub(crate) fn apply_expendable_item_effect(
        &mut self,
        property_type: i32,
        mut value: impl FnMut(u32) -> i32,
        mut now: impl FnMut() -> u32,
    ) -> i32 {
        if !matches!(property_type, 0x4a..=0x4d) {
            return 0;
        }
        let previous = self.expendable_effects.get(&property_type).copied();
        let mut adjust_property = |amount: i32, subtract: bool| {
            match property_type {
                0x4a => {
                    let current = self.combat_properties.maximum_attack;
                    self.combat_properties.maximum_attack = if subtract {
                        current.wrapping_sub(amount as u32)
                    } else {
                        current.wrapping_add(amount as u32)
                    };
                }
                0x4b => {
                    let current = self.combat_properties.attack_speed;
                    self.combat_properties.attack_speed = if subtract {
                        current.wrapping_sub(amount as u16)
                    } else {
                        current.wrapping_add(amount as u16)
                    };
                }
                0x4c => {
                    let current = self.combat_properties.defense;
                    self.combat_properties.defense = if subtract {
                        current.wrapping_sub(amount as u32)
                    } else {
                        current.wrapping_add(amount as u32)
                    };
                }
                0x4d => {
                    let current = self.combat_properties.element_modify;
                    self.combat_properties.element_modify = if subtract {
                        current.wrapping_sub(amount)
                    } else {
                        current.wrapping_add(amount)
                    };
                }
                _ => unreachable!("поддержанный expendable property проверен до чтения значений"),
            }
        };
        if let Some(previous) = previous {
            adjust_property(previous.value, true);
            adjust_property(value(1), false);
            let effect = self.expendable_effects.get_mut(&property_type)
                .expect("замена сохраняет найденный expendable effect");
            effect.start_time_ms = now();
            effect.effect_time_ms = value(2) as u32;
            effect.value = value(1);
        } else {
            let stored_value = value(1);
            let start_time_ms = now();
            let effect_time_ms = value(2) as u32;
            adjust_property(value(1), false);
            self.expendable_effects.insert(property_type, PlayerExpendableEffect {
                property_type,
                value: stored_value,
                start_time_ms,
                effect_time_ms,
            });
        }
        match property_type {
            0x4a => LegacyWriter::write_u32_at(
                &mut self.combat_property_wire,
                0x20,
                self.combat_properties.maximum_attack,
            )
            .expect("combat wire содержит maximum attack"),
            0x4b => LegacyWriter::write_u16_at(
                &mut self.combat_property_wire,
                0x32,
                self.combat_properties.attack_speed,
            )
            .expect("combat wire содержит attack speed"),
            0x4c => LegacyWriter::write_u32_at(
                &mut self.combat_property_wire,
                0x2c,
                self.combat_properties.defense,
            )
            .expect("combat wire содержит defense"),
            0x4d => LegacyWriter::write_i32_at(
                &mut self.combat_property_wire,
                0x48,
                self.combat_properties.element_modify,
            )
            .expect("combat wire содержит element modify"),
            _ => {}
        }
        match property_type {
            0x4a => self.combat_properties.maximum_attack as i32,
            0x4b => i32::from(self.combat_properties.attack_speed),
            0x4c => self.combat_properties.defense as i32,
            0x4d => self.combat_properties.element_modify,
            _ => 0,
        }
    }

    pub(crate) const fn combat_property_wire(&self) -> &[u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE] {
        &self.combat_property_wire
    }

    /// Selector `3009` читает не производную Rust-модель, а те же concrete
    /// `tagBaseProperty`/`tagProperty` slots, которые использует EXE. Поэтому
    /// неизвестные, но загруженные и пересчитанные поля сохраняются в выводе.
    pub(crate) fn all_properties_diagnostic_snapshot(
        &self,
    ) -> PlayerAllPropertiesDiagnosticSnapshot {
        let base = self.synchronized_base_property_wire();
        let current = &self.combat_property_wire;
        let base_u16 = |offset| u32::from(read_player_wire_u16(&base, offset));
        let base_u32 = |offset| read_player_wire_u32(&base, offset);
        let current_u16 = |offset| u32::from(read_player_wire_u16(current, offset));
        let current_u32 = |offset| read_player_wire_u32(current, offset);
        PlayerAllPropertiesDiagnosticSnapshot {
            name: self.shape().base_object().get_name().to_vec(),
            summary_words: [
                u32::from(self.occupation()),
                u32::from(self.level()),
                self.experience(),
                base_u32(BASE_HEALTH_OFFSET),
                current_u32(0x00),
                base_u32(BASE_MANA_OFFSET),
                current_u32(0x04),
                base_u16(0xac),
                current_u16(0x0a),
                base_u32(BASE_MAXIMUM_HP_OFFSET),
                base_u32(BASE_MAXIMUM_MP_OFFSET),
                base_u16(0xba),
                base_u16(BASE_PK_COUNT_OFFSET),
                base_u32(BASE_KILL_COUNT_OFFSET),
                self.money(),
            ],
            base_combat_words: [
                base_u32(BASE_STRENGTH_OFFSET),
                base_u32(BASE_DEXTERITY_OFFSET),
                base_u32(BASE_CONSTITUTION_OFFSET),
                base_u32(BASE_INTELLIGENCE_OFFSET),
                base_u32(0xcc),
                base_u32(0xd0),
                base_u16(0xd4),
                base_u16(0xd6),
                base_u16(0xd8),
                base_u32(0xdc),
                base_u16(0xe0),
                base_u16(0xe2),
                base_u32(0xe4),
                base_u16(0xe8),
                base_u16(0xea),
            ],
            current_combat_words: [
                current_u32(0x0c),
                current_u32(0x10),
                current_u32(0x14),
                current_u32(0x18),
                current_u32(0x1c),
                current_u32(0x20),
                current_u16(0x24),
                current_u16(0x26),
                current_u16(0x28),
                current_u32(0x2c),
                current_u16(0x30),
                i32::from(read_player_wire_u16(current, 0x32) as i16) as u32,
                current_u32(0x34),
                current_u16(0x38),
                current_u16(0x3a),
                current_u16(0x3c),
                current_u32(0x40),
                current_u16(0x44),
                current_u32(0x48),
                current_u16(0x4c),
            ],
        }
    }

    /// Граница восстановления exact `m_Property` из persisted player state.
    /// Последующие reached-пересчёты заменяют только известные поля layout.
    pub(crate) const fn restore_combat_property_wire(
        &mut self,
        wire: [u8; PLAYER_COMBAT_PROPERTY_WIRE_SIZE],
    ) {
        self.combat_property_wire = wire;
    }

    /// Применяет результат виртуального `UpdateProperty` и синхронизирует
    /// подтверждённые поля 0x9c-byte `tagProperty`, сохраняя неизвестные байты.
    pub(crate) fn apply_recomputed_combat_properties(
        &mut self,
        properties: PlayerCombatProperties,
        goods_factory: &CGoodsFactory,
    ) {
        self.combat_properties = properties;
        self.sync_combat_property_wire();
        self.refresh_equipment_flash(goods_factory);
    }

    fn sync_combat_property_wire(&mut self) {
        let properties = self.combat_properties;
        let write_u16 = |wire: &mut [u8], offset: usize, value: u16| {
            LegacyWriter::write_u16_at(wire, offset, value)
                .expect("combat wire offset проверен layout-константой");
        };
        let write_u32 = |wire: &mut [u8], offset: usize, value: u32| {
            LegacyWriter::write_u32_at(wire, offset, value)
                .expect("combat wire offset проверен layout-константой");
        };
        write_u32(&mut self.combat_property_wire, 0x00, properties.maximum_hp);
        write_u32(&mut self.combat_property_wire, 0x04, properties.maximum_mp);
        write_u16(&mut self.combat_property_wire, 0x08, properties.maximum_yp);
        write_u16(&mut self.combat_property_wire, 0x0a, properties.maximum_rp);
        write_u32(&mut self.combat_property_wire, 0x0c, properties.strength);
        write_u32(&mut self.combat_property_wire, 0x10, properties.dexterity);
        write_u32(
            &mut self.combat_property_wire,
            0x14,
            properties.constitution,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x18,
            properties.intelligence,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x1c,
            properties.minimum_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x20,
            properties.maximum_attack,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x32,
            properties.attack_speed,
        );
        write_u16(&mut self.combat_property_wire, 0x24, properties.hit);
        write_u16(&mut self.combat_property_wire, 0x30, properties.dodge);
        write_u16(&mut self.combat_property_wire, 0x28, properties.cch);
        write_u16(&mut self.combat_property_wire, 0x26, properties.burden);
        write_u32(&mut self.combat_property_wire, 0x2c, properties.defense);
        write_u32(
            &mut self.combat_property_wire,
            0x34,
            properties.element_resistance,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x3c,
            properties.soul_resistance,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x44,
            properties.add_soul_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x58,
            properties.blast_attack_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x40,
            properties.add_element_attack,
        );
        write_u16(&mut self.combat_property_wire, 0x38, properties.hp_recovery);
        write_u16(&mut self.combat_property_wire, 0x3a, properties.mp_recovery);
        write_u32(
            &mut self.combat_property_wire,
            0x48,
            properties.element_modify as u32,
        );
        write_u16(&mut self.combat_property_wire, 0x4c, properties.reank);
        write_u16(
            &mut self.combat_property_wire,
            0x4e,
            properties.attack_avoid,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x50,
            properties.element_avoid,
        );
        write_u16(&mut self.combat_property_wire, 0x52, properties.full_miss);
        write_u16(
            &mut self.combat_property_wire,
            0x54,
            properties.blast_attack,
        );
        write_u16(
            &mut self.combat_property_wire,
            0x56,
            properties.blast_element_attack,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x5c,
            properties.blast_defense_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x60,
            properties.element_blast_attack_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x64,
            properties.element_blast_defense_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x68,
            properties.full_miss_scale_bits,
        );
        write_u32(
            &mut self.combat_property_wire,
            0x6c,
            properties.critical_rate_bits,
        );
        write_u32(&mut self.combat_property_wire, 0x70, properties.resume_hp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x74, properties.resume_mp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x78, properties.resume_hp_fight as u32);
        write_u32(&mut self.combat_property_wire, 0x7c, properties.resume_mp_fight as u32);
        write_u32(&mut self.combat_property_wire, 0x80, properties.restored_hp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x84, properties.restored_mp_peace as u32);
        write_u32(&mut self.combat_property_wire, 0x88, properties.restored_hp_fight as u32);
        write_u32(&mut self.combat_property_wire, 0x8c, properties.restored_mp_fight as u32);
        self.combat_property_wire[0x90] = u8::from(properties.battle_fairy_summoned);
        self.combat_property_wire[0x91] = u8::from(properties.battle_fairy_recall);
        self.combat_property_wire[0x92] = u8::from(properties.battle_fairy_died);
    }

    /// `MountAllEquip -> SetCurFlash`: пересобирает 17 flash-ячеек после
    /// каждого полного property commit. Исторический system clock заменён
    /// `SystemTime`; damaged equipment намеренно сохраняет прежнее значение,
    /// потому что original loop пропускает `SetCurFlash` для нулевой прочности.
    fn refresh_equipment_flash(&mut self, factory: &CGoodsFactory) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as u32);
        for position in 0..17usize {
            let Some(goods) = self.equipment.get_goods(position as u32) else {
                self.flash_current[position] = 0;
                self.flash_changed = true;
                continue;
            };
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            let flash = if goods.query_attribute(GAP_GOODS_LIFE_TYPE) {
                match goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 2) {
                    1 => {
                        let expiry = goods
                            .start_point(factory)
                            .wrapping_add(u64::from(goods.goods_lifetime(factory)));
                        if (expiry >> 32) as i32 == 0 && expiry as u32 <= now {
                            0
                        } else {
                            goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32
                        }
                    }
                    2 => goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32,
                    _ => continue,
                }
            } else {
                goods.addon_property_value(factory, GAP_GOODS_EQUIMENT_FLASH, 1) as u32
            };
            self.flash_current[position] = flash;
            self.flash_changed = true;
        }
    }

    /// `DoneFlash` забирает один pending snapshot и одновременно продвигает
    /// previous-table. Повторный tick без нового `MountAllEquip` ничего не шлёт.
    pub(crate) fn take_flash_update(&mut self) -> Option<[(u32, u32); 17]> {
        if !self.flash_changed {
            return None;
        }
        let pairs = std::array::from_fn(|position| {
            (self.flash_previous[position], self.flash_current[position])
        });
        self.flash_previous = self.flash_current;
        self.flash_changed = false;
        Some(pairs)
    }

    pub(crate) const fn stat_allocation_state(&self) -> PlayerStatAllocationState {
        PlayerStatAllocationState {
            sex: self.base_properties.sex,
            occupation: self.base_properties.occupation,
            remain_point: self.base_properties.remain_point,
            base_maximum_hp: self.base_properties.base_maximum_hp,
            base_maximum_mp: self.base_properties.base_maximum_mp,
            base_strength: self.base_properties.base_strength,
            base_dexterity: self.base_properties.base_dexterity,
            base_constitution: self.base_properties.base_constitution,
            base_intelligence: self.base_properties.base_intelligence,
        }
    }

    /// Восстанавливает owned поля `m_BaseProperty`, участвующие в client
    /// allocation `0x8FA01`; полный decoder игрока остаётся отдельным owner-ом.
    pub(crate) const fn restore_stat_allocation_state(&mut self, state: PlayerStatAllocationState) {
        self.base_properties.sex = state.sex;
        self.base_properties.occupation = state.occupation;
        self.base_properties.remain_point = state.remain_point;
        self.base_properties.base_maximum_hp = state.base_maximum_hp;
        self.base_properties.base_maximum_mp = state.base_maximum_mp;
        self.base_properties.base_strength = state.base_strength;
        self.base_properties.base_dexterity = state.base_dexterity;
        self.base_properties.base_constitution = state.base_constitution;
        self.base_properties.base_intelligence = state.base_intelligence;
    }

    /// Exact mutation-tail `0x8FA01`: DEX/CON/INT используют legacy STR gate;
    /// неизвестный selector всё равно расходует одно очко и ведёт к recompute.
    pub(crate) fn allocate_stat_point(
        &mut self,
        selector: u8,
        constitution_hp: u16,
        intelligence_mp: u16,
    ) -> Option<PlayerStatAllocationMutation> {
        if self.base_properties.remain_point == 0 {
            return None;
        }
        let previous = self.stat_allocation_state();
        let strength_gate = self.base_properties.base_strength < i32::MAX as u32;
        let stat_changed = match selector {
            0 if strength_gate => {
                self.base_properties.base_strength =
                    self.base_properties.base_strength.wrapping_add(1);
                true
            }
            1 if strength_gate => {
                self.base_properties.base_dexterity =
                    self.base_properties.base_dexterity.wrapping_add(1);
                true
            }
            2 => {
                if strength_gate {
                    self.base_properties.base_constitution =
                        self.base_properties.base_constitution.wrapping_add(1);
                }
                self.base_properties.base_maximum_hp = self
                    .base_properties
                    .base_maximum_hp
                    .wrapping_add(u32::from(constitution_hp));
                strength_gate
            }
            3 => {
                if strength_gate {
                    self.base_properties.base_intelligence =
                        self.base_properties.base_intelligence.wrapping_add(1);
                }
                self.base_properties.base_maximum_mp = self
                    .base_properties
                    .base_maximum_mp
                    .wrapping_add(u32::from(intelligence_mp));
                strength_gate
            }
            _ => false,
        };
        self.base_properties.remain_point = self.base_properties.remain_point.wrapping_sub(1);
        Some(PlayerStatAllocationMutation {
            player_id: self.player_id(),
            selector,
            stat_changed,
            previous,
            current: self.stat_allocation_state(),
        })
    }

    pub(crate) const fn pk_permissions(&self) -> PlayerPkPermissions {
        PlayerPkPermissions {
            player: self.base_properties.pk_normal,
            teammate: self.base_properties.pk_team,
            guild_member: self.base_properties.pk_union,
            criminal: self.base_properties.pk_badman,
            country: self.base_properties.pk_country,
        }
    }

    /// Граница восстановления пяти persisted `bPk_*` перед skill/AI use.
    pub(crate) const fn restore_pk_permissions(&mut self, permissions: PlayerPkPermissions) {
        self.base_properties.pk_normal = permissions.player;
        self.base_properties.pk_team = permissions.teammate;
        self.base_properties.pk_union = permissions.guild_member;
        self.base_properties.pk_badman = permissions.criminal;
        self.base_properties.pk_country = permissions.country;
    }

    /// Exact selector `0x8FA05`; неизвестное signed-char значение не меняет
    /// state, но caller уже прочитал оба входных байта.
    pub(crate) fn set_pk_permission(
        &mut self,
        selector: i8,
        requested: bool,
    ) -> PlayerPkPermissionMutation {
        let previous = self.pk_permissions();
        let recognized = match selector {
            0 => {
                self.base_properties.pk_normal = requested;
                true
            }
            1 => {
                self.base_properties.pk_team = requested;
                true
            }
            2 => {
                self.base_properties.pk_union = requested;
                true
            }
            3 => {
                self.base_properties.pk_badman = requested;
                true
            }
            4 => {
                self.base_properties.pk_country = requested;
                true
            }
            _ => false,
        };
        let current = self.pk_permissions();
        PlayerPkPermissionMutation {
            player_id: self.player_id(),
            selector,
            requested,
            recognized,
            changed: previous != current,
            previous,
            current,
        }
    }

    /// Exact `GetCurBurden`: только equipment, packet и hand, в исходном
    /// wrapping-порядке. Временные auction/fairy/session containers не входят.
    pub(crate) fn current_burden(&self, factory: &CGoodsFactory) -> u32 {
        self.equipment
            .contents_weight(factory)
            .wrapping_add(self.packet.base().contents_weight(factory))
            .wrapping_add(self.hand.contents_weight(factory))
    }

    pub(crate) const fn ci_qing_open(&self) -> bool {
        self.ci_qing_open
    }

    pub(crate) fn ci_qing_list(&self) -> impl ExactSizeIterator<Item = u32> + '_ {
        self.ci_qing_list.iter().copied()
    }

    pub(crate) fn restore_ci_qing_entry(&mut self, base_index: u32) -> bool {
        self.ci_qing_list.insert(base_index)
    }

    pub(crate) fn ci_qing_property_snapshot(
        &self,
    ) -> (&BTreeMap<u32, u32>, &BTreeMap<u32, u32>, u32) {
        (
            &self.ci_qing_add_values,
            &self.ci_qing_tao_zhuang_add_values,
            self.tao_zhuang_id,
        )
    }

    pub(crate) fn apply_ci_qing_property_snapshot(
        &mut self,
        add_values: BTreeMap<u32, u32>,
        tao_zhuang_add_values: BTreeMap<u32, u32>,
        tao_zhuang_id: u32,
    ) {
        self.ci_qing_add_values = add_values;
        self.ci_qing_tao_zhuang_add_values = tao_zhuang_add_values;
        self.tao_zhuang_id = tao_zhuang_id;
    }

    pub(crate) fn ci_qing_property_result(&self) -> BTreeMap<u32, u32> {
        let mut result = self.ci_qing_add_values.clone();
        for (&property, &value) in &self.ci_qing_tao_zhuang_add_values {
            let current = result.entry(property).or_default();
            *current = current.wrapping_add(value);
        }
        result
    }

    pub(crate) fn update_ci_qing_property_difference(
        previous: &BTreeMap<u32, u32>,
        current: &BTreeMap<u32, u32>,
    ) -> Option<BTreeMap<u32, u32>> {
        if previous.len() != current.len() {
            return None;
        }
        let mut destination = BTreeMap::new();
        for ((_, &previous), (&property, &current)) in previous.iter().zip(current) {
            destination.insert(property, current.saturating_sub(previous));
        }
        Some(destination)
    }

    pub(crate) const fn tao_zhuang_is_pending(&self) -> bool {
        self.equipment_changed
    }

    pub(crate) const fn mark_tao_zhuang_pending(&mut self) {
        self.equipment_changed = true;
    }

    pub(crate) const fn tao_zhuang_setup_is_pending(&self) -> bool {
        self.tao_zhuang_setup_pending
    }

    pub(crate) const fn mark_tao_zhuang_setup_sent(&mut self) {
        self.tao_zhuang_setup_pending = false;
    }

    /// `MountAllEquip` собирает только уникальные original-name активных
    /// equipment/CiQing goods; одинаковое имя в двух слотах считается один раз.
    pub(crate) fn rebuild_tao_zhuang_items(
        &mut self,
        setup: &CTaoZhuangSetup,
        factory: &CGoodsFactory,
    ) {
        self.tao_zhuang_items.clear();
        self.tao_zhuang_original_names.clear();
        let mut names = Vec::new();
        for (_, goods) in self.equipment.traversing_goods() {
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            if let Some(name) = factory.query_goods_original_name(goods.base_properties_index()) {
                names.push(name.to_vec());
            }
        }
        for position in 0..self.ci_qing.size() {
            let Some(goods) = self.ci_qing.get_goods(position) else {
                continue;
            };
            if goods.query_attribute(GAP_GOODS_MAXIMUM_DURABILITY)
                && goods.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2) < 1
            {
                continue;
            }
            if let Some(name) = factory.query_goods_original_name(goods.base_properties_index()) {
                names.push(name.to_vec());
            }
        }
        for name in names {
            if self.tao_zhuang_original_names.insert(name.clone())
                && let Some(set_id) = setup.query_id_by_equipment_name(&name)
            {
                let count = self.tao_zhuang_items.entry(set_id).or_default();
                *count = count.wrapping_add(1);
            }
        }
    }

    pub(crate) fn tao_zhuang_skills_for_removal(
        &self,
        setup: &CTaoZhuangSetup,
        factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillRemoved> {
        setup
            .skill_ids()
            .iter()
            .filter_map(|&skill_id| {
                self.move_shape
                    .skill(skill_id, factory)
                    .map(|skill| BattleFairySkillRemoved {
                        message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                        player_id: self.player_id(),
                        skill_id,
                        skill_name: skill.name(factory).map(<[u8]>::to_vec),
                    })
            })
            .collect()
    }

    pub(crate) fn delete_tao_zhuang_skill(
        &mut self,
        skill_id: u32,
        factory: &CSkillFactory,
    ) -> bool {
        self.move_shape.delete_skill(skill_id, factory)
    }

    /// `ComputerAddValue`: каждый достигнутый threshold применяется, а первый
    /// threshold выше collected count немедленно завершает set-prefix.
    pub(crate) fn compute_tao_zhuang_bonuses(
        &mut self,
        setup: &CTaoZhuangSetup,
    ) -> Vec<TaoZhuangSetEvaluation> {
        self.tao_zhuang_properties.clear();
        self.ci_qing_tao_zhuang_properties.clear();
        self.tao_zhuang_skills.clear();
        let mut evaluations = Vec::new();
        for (&set_id, &item_count) in &self.tao_zhuang_items {
            let Some(item) = setup.item(set_id) else {
                continue;
            };
            for addition in item.additions().values() {
                if item_count < addition.number {
                    break;
                }
                for (&skill_id, &level) in &addition.skills {
                    self.tao_zhuang_skills
                        .entry(skill_id)
                        .and_modify(|current| *current = (*current).max(level))
                        .or_insert(level);
                }
                let destination = if set_id < 100 {
                    &mut self.tao_zhuang_properties
                } else {
                    &mut self.ci_qing_tao_zhuang_properties
                };
                for (&property, &value) in &addition.properties {
                    let current = destination.entry(property).or_default();
                    *current = current.wrapping_add(value);
                }
            }
            let collected_all = item.declared_equipment_count == item_count;
            evaluations.push(TaoZhuangSetEvaluation {
                set_id,
                collected_all,
                completion_script: if collected_all {
                    item.script.clone()
                } else {
                    Vec::new()
                },
            });
        }
        evaluations
    }

    pub(crate) const fn set_tao_zhuang_id(&mut self, set_id: u32) {
        self.tao_zhuang_id = set_id;
    }

    pub(crate) fn replace_ci_qing_tao_zhuang_add_values(&mut self, values: BTreeMap<u32, u32>) {
        self.ci_qing_tao_zhuang_add_values = values;
    }

    pub(crate) fn combat_type_values(&self) -> BTreeMap<u32, u32> {
        Self::combat_type_values_from(self.combat_properties)
    }

    /// Серверный домен `GetCurrentTypeValue` — ровно 15 ключей `2..0x10`:
    /// 2/3/4/5 = strength/dexterity/constitution/intelligence, 6/7 = min/max
    /// attack, 8 = element_modify, 9 = cch, 0xa = defense,
    /// 0xb = element_resistance, 0xc/0xd = max HP/MP, 0xe/0xf = blast attack /
    /// blast element attack, 0x10 = full_miss. Ключи в провод не уходят:
    /// клиент (CiQing-ветвь `0x53990E`) читает 15 позиционных DWORD после
    /// `(type, id)`, порядок задаётся возрастанием ключей BTreeMap. Соседний
    /// домен `0x0e..0x60` живёт отдельно в `apply_tao_zhuang_properties`
    /// (серверный `AddPreItemToPlayer`) и здесь не используется.
    pub(crate) fn combat_type_values_from(
        properties: PlayerCombatProperties,
    ) -> BTreeMap<u32, u32> {
        BTreeMap::from([
            (0x02, properties.strength),
            (0x03, properties.dexterity),
            (0x04, properties.constitution),
            (0x05, properties.intelligence),
            (0x06, properties.minimum_attack),
            (0x07, properties.maximum_attack),
            (0x08, properties.element_modify as u32),
            (0x09, u32::from(properties.cch)),
            (0x0a, properties.defense),
            (0x0b, properties.element_resistance),
            (0x0c, properties.maximum_hp),
            (0x0d, properties.maximum_mp),
            (0x0e, u32::from(properties.blast_attack)),
            (0x0f, u32::from(properties.blast_element_attack)),
            (0x10, u32::from(properties.full_miss)),
        ])
    }

    pub(crate) fn apply_tao_zhuang_properties(
        &mut self,
        ci_qing: bool,
        coefficients: GlobePlayerPropertyCoefficients,
    ) {
        let properties = if ci_qing {
            self.ci_qing_tao_zhuang_properties.clone()
        } else {
            self.tao_zhuang_properties.clone()
        };
        let occupation = usize::from(self.base_properties.occupation).min(2);
        // Оба TaoZhuang owner-а вызывают точный `AddPreItemToPlayer`: x87
        // усекает полную сумму live property и производной signed delta.
        let derived_u32 = |current: u32, value: u32, coefficient: f32| {
            (current as f32 + value as i32 as f32 * coefficient).trunc() as i32 as u32
        };
        let derived_i32 = |current: i32, value: u32, coefficient: f32| {
            (current as f32 + value as i32 as f32 * coefficient).trunc() as i32
        };
        let derived_u16 = |current: u16, value: u32, coefficient: f32| {
            (current as f32 + value as i32 as f32 * coefficient).trunc() as i32 as u16
        };
        for (property, value) in properties {
            match property {
                0x0e => {
                    self.combat_properties.minimum_attack =
                        self.combat_properties.minimum_attack.wrapping_add(value)
                }
                0x0f => {
                    self.combat_properties.maximum_attack =
                        self.combat_properties.maximum_attack.wrapping_add(value)
                }
                0x10 => {
                    self.combat_properties.element_modify = self
                        .combat_properties
                        .element_modify
                        .wrapping_add(value as i32)
                }
                0x11 => {
                    self.combat_properties.defense =
                        self.combat_properties.defense.wrapping_add(value)
                }
                0x12 => {
                    self.combat_properties.attack_speed = self
                        .combat_properties
                        .attack_speed
                        .wrapping_add(value as u16)
                }
                0x13 => {
                    self.combat_properties.hit =
                        self.combat_properties.hit.wrapping_add(value as u16)
                }
                0x14 => {
                    self.combat_properties.cch =
                        self.combat_properties.cch.wrapping_add(value as u16)
                }
                0x15 => {
                    self.combat_properties.dodge =
                        self.combat_properties.dodge.wrapping_add(value as u16)
                }
                0x17 => {
                    self.combat_properties.element_resistance = self
                        .combat_properties
                        .element_resistance
                        .wrapping_add(value)
                }
                0x19 => {
                    self.combat_properties.hp_recovery = self
                        .combat_properties
                        .hp_recovery
                        .wrapping_add(value as u16)
                }
                0x1a => {
                    self.combat_properties.mp_recovery = self
                        .combat_properties
                        .mp_recovery
                        .wrapping_add(value as u16)
                }
                0x1b => {
                    self.combat_properties.strength =
                        self.combat_properties.strength.wrapping_add(value);
                    self.combat_properties.maximum_attack = derived_u32(
                        self.combat_properties.maximum_attack,
                        value,
                        coefficients.str_to_max_attack[occupation],
                    );
                    self.combat_properties.burden = derived_u16(
                        self.combat_properties.burden,
                        value,
                        coefficients.str_to_burden[occupation],
                    );
                }
                0x1c => {
                    self.combat_properties.dexterity =
                        self.combat_properties.dexterity.wrapping_add(value);
                    self.combat_properties.minimum_attack = derived_u32(
                        self.combat_properties.minimum_attack,
                        value,
                        coefficients.dex_to_min_attack[occupation],
                    );
                    self.combat_properties.reank = derived_u16(
                        self.combat_properties.reank,
                        value,
                        coefficients.dex_to_stiff[occupation],
                    );
                }
                0x1d => {
                    self.combat_properties.constitution =
                        self.combat_properties.constitution.wrapping_add(value);
                    self.combat_properties.maximum_hp = derived_u32(
                        self.combat_properties.maximum_hp,
                        value,
                        coefficients.con_to_max_hp[occupation],
                    );
                    self.combat_properties.defense = derived_u32(
                        self.combat_properties.defense,
                        value,
                        coefficients.con_to_defense[occupation],
                    );
                }
                0x1e => {
                    self.combat_properties.intelligence =
                        self.combat_properties.intelligence.wrapping_add(value);
                    self.combat_properties.element_modify = derived_i32(
                        self.combat_properties.element_modify,
                        value,
                        coefficients.int_to_element[occupation],
                    );
                    self.combat_properties.maximum_mp = derived_u32(
                        self.combat_properties.maximum_mp,
                        value,
                        coefficients.int_to_max_mp[occupation],
                    );
                    self.combat_properties.element_resistance = derived_u32(
                        self.combat_properties.element_resistance,
                        value,
                        coefficients.int_to_resistant[occupation],
                    );
                }
                0x1f => {
                    self.combat_properties.maximum_hp =
                        self.combat_properties.maximum_hp.wrapping_add(value)
                }
                0x20 => {
                    self.combat_properties.maximum_mp =
                        self.combat_properties.maximum_mp.wrapping_add(value)
                }
                0x33 => {
                    self.combat_properties.reank =
                        self.combat_properties.reank.wrapping_add(value as u16)
                }
                0x34 => {
                    self.combat_properties.burden =
                        self.combat_properties.burden.wrapping_add(value as u16)
                }
                0x5b => {
                    self.combat_properties.attack_avoid = self
                        .combat_properties
                        .attack_avoid
                        .wrapping_add(value as u16)
                }
                0x5c => {
                    self.combat_properties.element_avoid = self
                        .combat_properties
                        .element_avoid
                        .wrapping_add(value as u16)
                }
                0x5d => {
                    self.combat_properties.full_miss =
                        self.combat_properties.full_miss.wrapping_add(value as u16)
                }
                0x5f => {
                    self.combat_properties.blast_attack = self
                        .combat_properties
                        .blast_attack
                        .wrapping_add(value as u16)
                }
                0x60 => {
                    self.combat_properties.blast_element_attack = self
                        .combat_properties
                        .blast_element_attack
                        .wrapping_add(value as u16)
                }
                _ => {}
            }
        }
        self.sync_combat_property_wire();
    }

    pub(crate) fn add_tao_zhuang_skills(
        &mut self,
        factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillAdded> {
        let player_id = self.player_id();
        let skills = self.tao_zhuang_skills.clone();
        let mut added = Vec::new();
        for (skill_id, level) in skills {
            if self.move_shape.add_skill(skill_id, level as i32, factory)
                && let Some(skill) = self.move_shape.skill(skill_id, factory)
            {
                added.push(battle_fairy_skill_snapshot(player_id, skill, factory));
            }
        }
        added
    }

    pub(crate) fn finish_tao_zhuang_update(&mut self) {
        self.equipment_changed = false;
        self.tao_zhuang_items.clear();
        self.tao_zhuang_original_names.clear();
        self.tao_zhuang_properties.clear();
        self.ci_qing_tao_zhuang_properties.clear();
        self.tao_zhuang_skills.clear();
    }

    pub(crate) fn check_item_in_packet(&self, base_index: u32) -> u32 {
        if base_index == 0 {
            return 0;
        }
        self.packet
            .base()
            .get_goods_by_base_properties(base_index)
            .into_iter()
            .fold(0u32, |total, goods| total.wrapping_add(goods.amount()))
    }

    /// Exact insertion-order `remove_item_in_packet`: каждый stack проходит
    /// через семантику `DeleteGoods(PEI_PACKET, ..., remaining, false)`.
    pub(crate) fn remove_item_in_packet(
        &mut self,
        base_index: u32,
        requested: u32,
    ) -> Vec<CiQingPacketConsumption> {
        if base_index == 0 || requested == 0 {
            return Vec::new();
        }
        let candidates: Vec<_> = self
            .packet
            .base()
            .get_goods_by_base_properties(base_index)
            .into_iter()
            .map(|goods| (goods.identity(), goods.amount()))
            .collect();
        let player_id = self.player_id();
        let mut removed_amount = 0u32;
        let mut consumptions = Vec::new();
        for (identity, previous_amount) in candidates {
            let remaining_request = requested.wrapping_sub(removed_amount);
            if remaining_request == 0 {
                break;
            }
            let position = self
                .packet
                .query_goods_position(identity.ex_id)
                .unwrap_or_default();
            let consumed = previous_amount.min(remaining_request);
            if consumed == 0 {
                continue;
            }
            let remaining_amount = previous_amount.wrapping_sub(consumed);
            let removal = if remaining_amount == 0 {
                self.packet.remove_goods(identity.ex_id)
            } else {
                let position = self.packet.query_goods_position(identity.ex_id);
                if let Some(goods) =
                    position.and_then(|position| self.packet.get_goods_mut(position))
                {
                    goods.set_amount(remaining_amount);
                }
                None
            };
            removed_amount = removed_amount.wrapping_add(consumed);
            consumptions.push(CiQingPacketConsumption {
                player_id,
                goods: identity,
                position,
                previous_amount,
                remaining_amount,
                removal,
            });
        }
        consumptions
    }

    pub(crate) fn remove_packet_goods_by_id(
        &mut self,
        goods_id: CGuid,
        requested: u32,
    ) -> Option<CiQingPacketConsumption> {
        if requested == 0 {
            return None;
        }
        let goods = self.packet.base().find(goods_id)?;
        let identity = goods.identity();
        let position = self.packet.query_goods_position(goods_id)?;
        let previous_amount = goods.amount();
        let consumed = previous_amount.min(requested);
        let remaining_amount = previous_amount.wrapping_sub(consumed);
        let removal = if remaining_amount == 0 {
            self.packet.remove_goods(goods_id)
        } else {
            let position = self.packet.query_goods_position(goods_id)?;
            self.packet
                .get_goods_mut(position)?
                .set_amount(remaining_amount);
            None
        };
        Some(CiQingPacketConsumption {
            player_id: self.player_id(),
            goods: identity,
            position,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn add_packet_goods_at(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> PlayerPacketAddOutcome {
        let outcome =
            self.packet
                .add_goods_at(position, incoming, factory, owner_progress_allows);
        self.notify_packet_goods_added(&outcome, factory, on_goods_added);
        PlayerPacketAddOutcome { outcome }
    }

    pub(crate) fn add_packet_goods(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> PlayerPacketAddOutcome {
        let outcome = self
            .packet
            .add_goods(incoming, factory, owner_progress_allows);
        self.notify_packet_goods_added(&outcome, factory, on_goods_added);
        PlayerPacketAddOutcome { outcome }
    }

    pub(crate) fn swap_packet_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> Option<VolumeGoodsSwapOutcome> {
        CVolumeLimitGoodsContainer::swap_goods_with_owner(
            self,
            position,
            incoming,
            owner_progress_allows,
            |player| &mut player.packet,
            |player, position, incoming, owner_progress_allows| {
                player
                    .add_packet_goods_at(
                        position,
                        incoming,
                        factory,
                        owner_progress_allows,
                        on_goods_added,
                    )
                    .outcome
            },
        )
    }

    fn notify_packet_goods_added(
        &mut self,
        outcome: &VolumeGoodsAddOutcome,
        factory: &CGoodsFactory,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) {
        let VolumeGoodsAddOutcome::Added(added) = outcome else {
            return;
        };
        let current_ticket = self.current_ticket;
        let registration = self
            .packet
            .base_mut()
            .find_mut(added.identity.ex_id)
            .and_then(|goods| {
                Self::prepare_goods_ai_registration_with_clock(
                    current_ticket,
                    goods,
                    factory,
                    &mut crate::gameserver::gameserver::game::game_wall_time_seconds,
                )
            });
        if let Some((ticket, goods_id)) = registration {
            self.record_goods_ai_registration(ticket, goods_id);
        }
        if let Some(goods) = self.packet.base().find(added.identity.ex_id) {
            let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
            on_goods_added(self, additional);
        }
    }

    /// Player-side `AddGoodsToPacket`: успешный add забирает ownership из
    /// входного vector, rejected/несовместимый stack остаётся у caller-а.
    pub(crate) fn add_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        let owner_progress_allows = self.current_progress == PlayerProgress::None;
        self.add_goods_to_packet_with_progress(
            goods,
            factory,
            encode_old_client,
            owner_progress_allows,
            on_goods_added,
        )
    }

    /// Billing Increment response приходит при `PROGRESS_INCREMENT`, но
    /// исходный `BillOfIncShop` добавляет batch напрямую в packet и не
    /// применяет progress-lock к stack merge.
    pub(crate) fn add_increment_shop_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// Script `2249` выполняется при занятом script progress, но native owner
    /// после созревания добавляет replacement напрямую в packet.
    pub(crate) fn add_script_fairy_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// `GetPreciousItem` исполняется внутри script progress, но native owner
    /// также добавляет награду напрямую и не применяет ordinary progress-lock.
    pub(crate) fn add_precious_box_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// `CTrader::Trade` добавляет contrary goods при
    /// `PROGRESS_TRADING`; этот owner намеренно обходит общий progress-lock,
    /// как прямой packet `Add` исходной функции.
    pub(crate) fn add_traded_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// NPC shop добавляет batch напрямую при `PROGRESS_SHOPPING`.
    pub(crate) fn add_shop_goods_to_packet(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        self.add_goods_to_packet_with_progress(
            goods, factory, encode_old_client, true, on_goods_added,
        )
    }

    /// Обратная половина `CTrader::RollBack`: отменяет уже выполненный
    /// contrary packet add, включая direct stack merge, и возвращает client
    /// consumption fact. Сам исходный goods caller хранит отдельно до commit.
    /// Машинный rollback commit `0x1B9470` обращает только ПОЛНЫЙ merge
    /// (`merged amount == original`); неслитый частичный merge не обращается —
    /// guard принадлежит Zone `trade/ctrader` (`trade_rollback_merge_reversible`,
    /// risk-note: как release-форма оригинала, возврат `None`).
    pub(crate) fn rollback_traded_packet_addition(
        &mut self,
        addition: &CiQingPacketAddition,
        original_amount: u32,
    ) -> Option<CiQingPacketConsumption> {
        let position = addition.position?;
        match &addition.outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let removed = self.packet.remove_goods(added.identity.ex_id)?;
                let taken = match removed {
                    VolumeGoodsRemoveOutcome::Removed(taken)
                    | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
                };
                let removed = match taken {
                    AmountLimitGoodsTaken::Removed(removed) => removed,
                    AmountLimitGoodsTaken::Split(_) => return None,
                };
                Some(CiQingPacketConsumption {
                    player_id: self.player_id(),
                    goods: removed.goods.identity(),
                    position,
                    previous_amount: removed.amount,
                    remaining_amount: 0,
                    removal: None,
                })
            }
            VolumeGoodsAddOutcome::Stack(
                super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                    target,
                    amount,
                },
            ) if trade_rollback_merge_reversible(*amount, original_amount) => {
                let goods = self.packet.get_goods_mut(position)?;
                if goods.identity() != *target || goods.amount() < original_amount {
                    return None;
                }
                let previous_amount = goods.amount();
                let remaining_amount = previous_amount.wrapping_sub(original_amount);
                goods.set_amount(remaining_amount);
                Some(CiQingPacketConsumption {
                    player_id: self.player_id(),
                    goods: *target,
                    position,
                    previous_amount,
                    remaining_amount,
                    removal: None,
                })
            }
            _ => None,
        }
    }

    fn add_goods_to_packet_with_progress(
        &mut self,
        goods: Vec<CGoods>,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
        owner_progress_allows: bool,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> (Vec<CiQingPacketAddition>, Vec<CGoods>) {
        let player_id = self.player_id();
        let mut additions = Vec::new();
        let mut remaining = Vec::new();
        for goods in goods {
            let source = goods.identity();
            let mut incoming = Some(goods);
            let packet_add = self.add_packet_goods(
                &mut incoming, factory, owner_progress_allows, on_goods_added,
            );
            let outcome = packet_add.outcome;
            let (old_client_payload, resulting_amount) = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => {
                    let stored = self
                        .packet
                        .base()
                        .find(added.identity.ex_id)
                        .expect("успешный packet add сохранил новый goods");
                    let payload = Some(encode_old_client(stored));
                    let amount = Some(stored.amount());
                    (payload, amount)
                }
                VolumeGoodsAddOutcome::Stack(stack) => {
                    let target = match stack {
                        super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                            target,
                            ..
                        } => self.packet.base().find(target.ex_id),
                        _ => None,
                    };
                    (None, target.map(CGoods::amount))
                }
                VolumeGoodsAddOutcome::Rejected(_) => (None, None),
            };
            let position = match &outcome {
                VolumeGoodsAddOutcome::Added(added) => added.position,
                VolumeGoodsAddOutcome::Stack(
                    super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                        target,
                        ..
                    },
                ) => self.packet.query_goods_position(target.ex_id),
                _ => None,
            };
            additions.push(CiQingPacketAddition {
                player_id,
                source,
                position,
                outcome,
                old_client_payload,
                resulting_amount,
            });
            if let Some(goods) = incoming {
                remaining.push(goods);
            }
        }
        (additions, remaining)
    }

    pub(crate) fn ci_qing_compose_goods(&self, position: u32) -> Option<&CGoods> {
        self.ci_qing_compose.get_goods(position)
    }

    pub(crate) fn ci_qing_goods(&self, position: u32) -> Option<&CGoods> {
        self.ci_qing.get_goods(position)
    }

    pub(crate) fn ci_qing_goods_amount(&self, factory: &CGoodsFactory) -> u32 {
        self.ci_qing.goods_amount(factory)
    }

    pub(crate) fn add_goods_to_ci_qing(
        &mut self,
        goods: CGoods,
        position: u32,
        compose_container: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> (CiQingContainerAddition, Option<CGoods>) {
        let player_id = self.player_id();
        let source = goods.identity();
        let container_extend_id = if compose_container { 17 } else { 16 };
        let container = if compose_container {
            &mut self.ci_qing_compose
        } else {
            &mut self.ci_qing
        };
        let mut incoming = Some(goods);
        let outcome = container.add_goods_at(
            position,
            &mut incoming,
            factory,
            self.current_progress == PlayerProgress::None,
        );
        let (old_client_payload, resulting_amount) = match &outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let stored = container
                    .base()
                    .find(added.identity.ex_id)
                    .expect("успешный CiQing add сохранил goods");
                (Some(encode_old_client(stored)), Some(stored.amount()))
            }
            VolumeGoodsAddOutcome::Stack(stack) => {
                let target = match stack {
                    super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                        target,
                        ..
                    } => container.base().find(target.ex_id),
                    _ => None,
                };
                (None, target.map(CGoods::amount))
            }
            VolumeGoodsAddOutcome::Rejected(_) => (None, None),
        };
        (
            CiQingContainerAddition {
                player_id,
                container_extend_id,
                position,
                source,
                outcome,
                old_client_payload,
                resulting_amount,
            },
            incoming,
        )
    }

    /// Ownership-ветвь generic `CC2SContainerObjectMove` для временного
    /// compose-контейнера. В отличие от CiQing gameplay packets здесь не
    /// нужен old-client object stream: итоговый `0xC0101` несёт только GUID
    /// и amount, но positional/automatic Add и listener state остаются у
    /// concrete container owner-а.
    pub(crate) fn add_ci_qing_compose_transfer_goods(
        &mut self,
        incoming: &mut Option<CGoods>,
        position: u32,
        factory: &CGoodsFactory,
    ) -> CiQingContainerAddition {
        let player_id = self.player_id();
        let source = incoming
            .as_ref()
            .expect("CiQing compose transfer add получает detached goods")
            .identity();
        let outcome = if position == u32::MAX {
            self.ci_qing_compose.add_goods(
                incoming,
                factory,
                self.current_progress == PlayerProgress::None,
            )
        } else {
            self.ci_qing_compose.add_goods_at(
                position,
                incoming,
                factory,
                self.current_progress == PlayerProgress::None,
            )
        };
        let (actual_position, resulting_amount) = match &outcome {
            VolumeGoodsAddOutcome::Added(added) => {
                let actual_position = added.position.unwrap_or(position);
                let amount = self
                    .ci_qing_compose
                    .get_goods(actual_position)
                    .map(CGoods::amount);
                (actual_position, amount)
            }
            VolumeGoodsAddOutcome::Stack(
                super::container::cgoodscontainer::GoodsStackMergeOutcome::Merged {
                    target, ..
                },
            ) => {
                let actual_position = self
                    .ci_qing_compose
                    .query_goods_position(target.ex_id)
                    .unwrap_or(position);
                let amount = self
                    .ci_qing_compose
                    .get_goods(actual_position)
                    .map(CGoods::amount);
                (actual_position, amount)
            }
            _ => (position, None),
        };
        CiQingContainerAddition {
            player_id,
            container_extend_id: 17,
            position: actual_position,
            source,
            outcome,
            old_client_payload: None,
            resulting_amount,
        }
    }

    pub(crate) fn take_ci_qing_compose_transfer_goods<Create>(
        &mut self,
        position: u32,
        requested_amount: u32,
        factory: &CGoodsFactory,
        create_goods: Create,
    ) -> Option<VolumeGoodsRemoveOutcome>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        self.ci_qing_compose
            .take_goods(position, requested_amount, factory, create_goods)
    }

    pub(crate) fn remove_ci_qing_compose_goods(
        &mut self,
        position: u32,
    ) -> Option<CiQingContainerConsumption> {
        let goods = self.ci_qing_compose.get_goods(position)?;
        let identity = goods.identity();
        let amount = goods.amount();
        let removal = self.ci_qing_compose.remove_goods(identity.ex_id)?;
        Some(CiQingContainerConsumption {
            player_id: self.player_id(),
            container_extend_id: 17,
            position,
            goods: identity,
            previous_amount: amount,
            remaining_amount: 0,
            removal: Some(removal),
        })
    }

    pub(crate) fn remove_ci_qing_goods(
        &mut self,
        position: u32,
        requested: u32,
    ) -> Option<CiQingContainerConsumption> {
        let goods = self.ci_qing.get_goods(position)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        let consumed = previous_amount.min(requested);
        if consumed == 0 {
            return None;
        }
        let remaining_amount = previous_amount.wrapping_sub(consumed);
        let removal = if remaining_amount == 0 {
            self.ci_qing.remove_goods(identity.ex_id)
        } else {
            self.ci_qing
                .get_goods_mut(position)
                .map(|goods| goods.set_amount(remaining_amount));
            None
        };
        Some(CiQingContainerConsumption {
            player_id: self.player_id(),
            container_extend_id: 16,
            position,
            goods: identity,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn ci_qing_hand_goods(&self) -> Option<&CGoods> {
        self.hand.get_goods(0)
    }

    pub(crate) fn hotkey(&self, slot: u8) -> Option<u32> {
        self.base_properties.hotkeys.get(usize::from(slot)).copied()
    }

    pub(crate) fn set_hotkey(&mut self, slot: u8, value: u32) -> bool {
        let Some(hotkey) = self.base_properties.hotkeys.get_mut(usize::from(slot)) else {
            return false;
        };
        *hotkey = value;
        true
    }

    pub(crate) const fn last_operated_goods(&self) -> (u32, u32) {
        (
            self.last_operated_container,
            self.last_operated_goods_position,
        )
    }

    pub(crate) fn record_last_operated_goods(
        &mut self,
        source_extend_id: i32,
        source_position: u32,
    ) -> (u32, u32) {
        let previous = self.last_operated_goods();
        self.last_operated_container = source_extend_id as u32;
        self.last_operated_goods_position = source_position;
        previous
    }

    /// Storage core назначения hotkey из hand. Packet/hand/wallet/YuanBao
    /// замкнуты на owned containers; equipment для consumable доказательно
    /// отвергается до mutation.
    pub(crate) fn return_hotkey_hand_goods(
        &mut self,
        factory: &CGoodsFactory,
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> HotkeyHandTransferReport {
        let (source_container_extend_id, source_position) = self.last_operated_goods();
        let mut report = HotkeyHandTransferReport {
            source_container_extend_id,
            source_position,
            goods: None,
            hand_removal: None,
            packet_adds: Vec::new(),
            currency_adds: Vec::new(),
            hand_rollback: None,
            outcome: HotkeyHandTransferOutcome::MissingHandGoods,
        };
        let Some(hand_goods) = self.hand.get_goods(0) else {
            return report;
        };
        report.goods = Some(hand_goods.identity());
        let Some(properties) =
            factory.query_goods_base_properties(hand_goods.base_properties_index())
        else {
            report.outcome = HotkeyHandTransferOutcome::NotConsumable;
            return report;
        };
        if properties.goods_type() != GOODS_TYPE_CONSUMABLE {
            report.outcome = HotkeyHandTransferOutcome::NotConsumable;
            return report;
        }
        if !(1..=5).contains(&source_container_extend_id) {
            report.outcome = HotkeyHandTransferOutcome::UnsupportedSource;
            return report;
        }

        let removed = self
            .hand
            .remove_goods(hand_goods.identity().ex_id)
            .expect("unlocked hand goods проверен перед synchronous remove");
        report.hand_removal = Some(HotkeyHandOwnershipEvent {
            owner_type: removed.owner_type,
            owner_id: removed.owner_id,
            position: removed.position,
            amount: removed.amount,
            listeners: removed.listeners.clone(),
        });
        let mut incoming = Some(removed.goods);
        if source_container_extend_id == 1 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.packet_adds.push(self.add_packet_goods_at(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
                on_goods_added,
            ));
            if incoming.is_some() {
                report.packet_adds.push(self.add_packet_goods(
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                    on_goods_added,
                ));
            }
        } else if source_container_extend_id == 3 {
            let goods = incoming.take().expect("removed hand goods остаётся owned");
            match self.hand.add_goods(goods, factory) {
                Ok(added) => report.hand_rollback = Some(added),
                Err(goods) => incoming = Some(goods),
            }
        } else if source_container_extend_id == 4 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.currency_adds.push(self.wallet.add_goods(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.currency_adds.push(self.wallet.add_goods(
                    0,
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        } else if source_container_extend_id == 5 {
            let owner_progress_allows = self.current_progress == PlayerProgress::None;
            report.currency_adds.push(self.yuan_bao.add_goods(
                source_position,
                &mut incoming,
                factory,
                owner_progress_allows,
            ));
            if incoming.is_some() {
                report.currency_adds.push(self.yuan_bao.add_goods(
                    0,
                    &mut incoming,
                    factory,
                    owner_progress_allows,
                ));
            }
        }

        if incoming.is_none() {
            if source_container_extend_id == 4 {
                self.money = self.wallet.currency_amount();
            }
            report.outcome = HotkeyHandTransferOutcome::Moved;
            return report;
        }
        let goods = incoming
            .take()
            .expect("failed destination сохраняет incoming");
        match self.hand.add_goods(goods, factory) {
            Ok(added) => {
                report.hand_rollback = Some(added);
                report.outcome = HotkeyHandTransferOutcome::RolledBack;
            }
            Err(goods) => {
                report.goods = Some(goods.identity());
                drop(goods);
                report.outcome = HotkeyHandTransferOutcome::GarbageCollected;
            }
        }
        report
    }

    pub(crate) fn destroy_hand_goods(
        &mut self,
        goods_id: CGuid,
        requested: u32,
    ) -> Option<GoodsDestroyHandConsumption> {
        if requested == 0 {
            return None;
        }
        let goods = self.hand.find(goods_id)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        let removed_amount = previous_amount.min(requested);
        let remaining_amount = previous_amount.wrapping_sub(removed_amount);
        let removal = if remaining_amount == 0 {
            self.hand.remove_goods(goods_id)
        } else {
            self.hand.find_mut(goods_id)?.set_amount(remaining_amount);
            None
        };
        Some(GoodsDestroyHandConsumption {
            player_id: self.player_id(),
            goods: identity,
            previous_amount,
            removed_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn remove_ci_qing_hand_goods(&mut self) -> Option<CiQingHandConsumption> {
        let goods = self.hand.get_goods(0)?;
        let identity = goods.identity();
        let previous_amount = goods.amount();
        if previous_amount == 0 {
            return None;
        }
        let remaining_amount = previous_amount.wrapping_sub(1);
        let removal = if remaining_amount == 0 {
            self.hand.remove_goods(identity.ex_id)
        } else {
            self.hand.find_mut(identity.ex_id).map(|goods| {
                goods.set_amount(remaining_amount);
            });
            None
        };
        Some(CiQingHandConsumption {
            player_id: self.player_id(),
            goods: identity,
            previous_amount,
            remaining_amount,
            removal,
        })
    }

    pub(crate) fn ci_qing_mount_facts(&self, factory: &CGoodsFactory) -> Option<(u32, u32, u32)> {
        let goods = self.ci_qing_hand_goods()?;
        if goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
            > i32::from(self.level())
        {
            return None;
        }
        Some((
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY1, 1) as u32,
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY1, 2) as u32,
            goods.addon_property_value(factory, GAP_CIQING_PROPERTY2, 1) as u32,
        ))
    }

    pub(crate) const fn contend_state(&self) -> bool {
        self.contend_state
    }

    pub(crate) fn apply_client_direction(&mut self, direction: u8) -> i32 {
        self.move_shape
            .shape_mut()
            .set_direction(i32::from(direction));
        self.move_shape.shape().get_direction()
    }

    pub(crate) fn clear_emotion_state(&mut self) {
        self.emotion_index = 0;
        self.emotion_timestamp_ms = 0;
    }

    /// Exact reached `PerformEmotion` state: state очищается до guards;
    /// repeated emotion запоминается, но around publication выполняется для
    /// любого разрешённого AI/жизни вызова.
    pub(crate) fn perform_emotion_state(
        &mut self,
        emotion_id: i32,
        repeated: bool,
        now_ms: u32,
        ai_available: bool,
        ai_has_target: bool,
    ) -> bool {
        self.clear_emotion_state();
        if self.is_dead() || !ai_available || ai_has_target {
            return false;
        }
        if repeated {
            self.emotion_index = emotion_id;
            self.emotion_timestamp_ms = now_ms;
        }
        true
    }

    /// State-часть exact `SetContendState`: unchanged setter не публикуется.
    /// `0xBFF28` собирает и маршрутизирует caller после успешной мутации.
    pub(crate) const fn set_contend_state(&mut self, contend_state: bool) -> bool {
        if self.contend_state == contend_state {
            return false;
        }
        self.contend_state = contend_state;
        true
    }

    pub(crate) const fn city_war_died_state_time_ms(&self) -> i32 {
        self.city_war_died_state_time_ms
    }

    pub(crate) const fn city_war_died_state(&self) -> bool {
        self.city_war_died_state
    }

    /// Script `9313` читает этот byte напрямую без вычисления аргументов.
    pub(crate) const fn is_nation_war_player_weak(&self) -> bool {
        self.city_war_died_state
    }

    pub(crate) const fn died_state_start_time_ms(&self) -> u32 {
        self.died_state_start_time_ms
    }

    /// Direct assignment из `OnDied`: original не вызывает setter и поэтому
    /// не посылает `0xBFF2B`; clock стартует только для positive duration.
    pub(crate) const fn begin_city_war_death_countdown(&mut self, duration_ms: i32, now_ms: u32) {
        self.city_war_died_state_time_ms = duration_ms;
        if duration_ms > 0 {
            self.died_state_start_time_ms = now_ms;
        }
    }

    /// Достигнутый decode-tail восстановления player: persisted duration
    /// запускает новый local clock, а action `ACT_DIED == 6` оставляет state
    /// выключенным до `OnRelive`.
    pub(crate) fn restore_city_war_death_countdown(&mut self, duration_ms: i32, now_ms: u32) {
        self.city_war_died_state_time_ms = duration_ms;
        if duration_ms > 0 {
            self.died_state_start_time_ms = now_ms;
            if self.shape().get_action() != 6 {
                self.city_war_died_state = true;
            }
        }
    }

    /// State-часть exact `SetCityWarDiedStateTime`; caller использует return
    /// как gate `0xBFF2B`, который допустим лишь при active died state.
    pub(crate) const fn set_city_war_died_state_time_ms(&mut self, time_ms: i32) -> bool {
        if self.city_war_died_state_time_ms == time_ms {
            return false;
        }
        self.city_war_died_state_time_ms = time_ms;
        self.city_war_died_state
    }

    /// Exact `SetCityWarDiedState` всегда пишет state и всегда публикует обе
    /// копии `0xBFF2A`, даже если значение не изменилось.
    pub(crate) const fn set_city_war_died_state(&mut self, died_state: bool) {
        self.city_war_died_state = died_state;
    }

    pub(crate) const fn restart_died_state_clock(&mut self, now_ms: u32) {
        self.died_state_start_time_ms = now_ms;
    }

    /// Exact counter-prefix `CPlayer::PeriodicalUpdate`: исторический `long`
    /// увеличивается до строгого порога `250`; caller только после этого
    /// снимает `timeGetTime`, обнуляет counter и публикует `0xBF809`.
    pub(crate) const fn advance_periodical_ping(&mut self) -> i32 {
        self.ping_time = self.ping_time.wrapping_add(1);
        self.ping_time
    }

    pub(crate) const fn complete_periodical_ping(&mut self, now_ms: u32) {
        self.ping_time = 0;
        self.last_ping_time_ms = now_ms;
    }

    pub(crate) const fn contribution(&self) -> i32 {
        self.contribution
    }

    pub(crate) const fn money(&self) -> u32 {
        self.money
    }

    /// Маршрут живых контейнеров перевода банковской валюты; таблица
    /// extend-id принадлежит Zone `trade/currency`.
    pub(crate) fn bank_transfer_currency_goods(&self, extend_id: i32) -> Option<&CGoods> {
        match BankCurrencyContainer::from_extend_id(extend_id) {
            Some(BankCurrencyContainer::Wallet) => self.wallet.get_goods(0),
            Some(BankCurrencyContainer::Bank) => self.bank.get_goods(0),
            Some(BankCurrencyContainer::AuctionWallet) => self.auction_wallet.get_goods(0),
            None => None,
        }
    }

    pub(crate) fn take_bank_transfer_currency_goods<Create>(
        &mut self,
        extend_id: i32,
        requested: u32,
        factory: &CGoodsFactory,
        mut create_goods: Create,
    ) -> Option<CurrencyGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let container = BankCurrencyContainer::from_extend_id(extend_id);
        let taken = match container {
            Some(BankCurrencyContainer::Wallet) => self
                .wallet
                .take_goods(0, requested, factory, &mut create_goods),
            Some(BankCurrencyContainer::Bank) => self
                .bank
                .take_goods(0, requested, factory, &mut create_goods),
            Some(BankCurrencyContainer::AuctionWallet) => self
                .auction_wallet
                .take_goods(0, requested, factory, &mut create_goods),
            None => None,
        };
        if matches!(container, Some(BankCurrencyContainer::Wallet)) {
            self.money = self.wallet.currency_amount();
        }
        taken
    }

    pub(crate) fn add_bank_transfer_currency_goods(
        &mut self,
        extend_id: i32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
    ) -> Option<PlayerBankCurrencyAddOutcome> {
        let owner_progress_allows = !matches!(
            self.current_progress,
            PlayerProgress::OpenStall | PlayerProgress::Trading | PlayerProgress::Upgrade
        );
        let container = BankCurrencyContainer::from_extend_id(extend_id);
        let outcome = match container {
            Some(BankCurrencyContainer::Wallet) => PlayerBankCurrencyAddOutcome::Wallet(
                self.wallet
                    .add_goods(0, incoming, factory, owner_progress_allows),
            ),
            Some(BankCurrencyContainer::Bank) => PlayerBankCurrencyAddOutcome::Bank(
                self.bank
                    .add_goods(0, incoming, factory, owner_progress_allows),
            ),
            Some(BankCurrencyContainer::AuctionWallet) => {
                PlayerBankCurrencyAddOutcome::AuctionWallet(self.auction_wallet.add_goods(
                    0,
                    incoming,
                    factory,
                    owner_progress_allows,
                ))
            }
            None => return None,
        };
        if matches!(container, Some(BankCurrencyContainer::Wallet)) {
            self.money = self.wallet.currency_amount();
        }
        Some(outcome)
    }

    /// Маршрут живых контейнеров наземной валюты; таблица extend-id
    /// принадлежит Zone `trade/currency`.
    pub(crate) fn ground_currency_goods(&self, extend_id: i32) -> Option<&CGoods> {
        match GroundCurrencyContainer::from_extend_id(extend_id) {
            Some(GroundCurrencyContainer::Wallet) => self.wallet.get_goods(0),
            Some(GroundCurrencyContainer::YuanBao) => self.yuan_bao.get_goods(0),
            None => None,
        }
    }

    pub(crate) fn take_ground_currency_goods<Create>(
        &mut self,
        extend_id: i32,
        requested: u32,
        factory: &CGoodsFactory,
        mut create_goods: Create,
    ) -> Option<CurrencyGoodsTaken>
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let container = GroundCurrencyContainer::from_extend_id(extend_id);
        let taken = match container {
            Some(GroundCurrencyContainer::Wallet) => self
                .wallet
                .take_goods(0, requested, factory, &mut create_goods),
            Some(GroundCurrencyContainer::YuanBao) => self
                .yuan_bao
                .take_goods(0, requested, factory, &mut create_goods),
            None => None,
        };
        if matches!(container, Some(GroundCurrencyContainer::Wallet)) {
            self.money = self.wallet.currency_amount();
        }
        taken
    }

    pub(crate) fn add_ground_currency_goods(
        &mut self,
        extend_id: i32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        owner_progress_allows: bool,
    ) -> Option<CurrencyGoodsAddOutcome> {
        let container = GroundCurrencyContainer::from_extend_id(extend_id);
        let outcome = match container {
            Some(GroundCurrencyContainer::Wallet) => self
                .wallet
                .add_goods(0, incoming, factory, owner_progress_allows),
            Some(GroundCurrencyContainer::YuanBao) => self
                .yuan_bao
                .add_goods(0, incoming, factory, owner_progress_allows),
            None => return None,
        };
        if matches!(container, Some(GroundCurrencyContainer::Wallet)) {
            self.money = self.wallet.currency_amount();
        }
        Some(outcome)
    }

    pub(crate) fn decrease_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
    ) -> PlayerMoneyDecrease {
        let previous = self.wallet.currency_amount();
        let outcome = self.wallet.decrease_currency(requested, factory);
        self.money = self.wallet.currency_amount();
        PlayerMoneyDecrease {
            previous,
            current: self.money,
            outcome,
        }
    }

    pub(crate) fn increase_money(
        &mut self,
        requested: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> super::container::cwallet::CurrencyIncreaseOutcome {
        let mut created_currency = Some(created_currency);
        let outcome = self
            .wallet
            .increase_currency(requested, factory, move |_, _| {
                created_currency.take().unwrap_or_default()
            });
        self.money = self.wallet.currency_amount();
        outcome
    }

    pub(crate) fn yuan_bao(&self) -> u32 {
        self.yuan_bao.currency_amount()
    }

    /// State-owner exact `SetYuanBao`: однослотовый currency container
    /// сохраняет create/increase/decrease/delete outcome для сетевого
    /// caller-а; развилка направления принадлежит Zone `trade/currency`.
    pub(crate) fn set_yuan_bao(
        &mut self,
        current: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> PlayerYuanBaoChange {
        let previous = self.yuan_bao.currency_amount();
        let outcome = match balance_set_direction(previous, current) {
            BalanceSetDirection::Increase(delta) => {
                let mut created_currency = Some(created_currency);
                PlayerYuanBaoChangeOutcome::Increased(self.yuan_bao.increase_currency(
                    delta,
                    factory,
                    move |_, _| created_currency.take().unwrap_or_default(),
                ))
            }
            BalanceSetDirection::Decrease(delta) => PlayerYuanBaoChangeOutcome::Decreased(
                self.yuan_bao.decrease_currency(delta, factory),
            ),
            BalanceSetDirection::Unchanged => PlayerYuanBaoChangeOutcome::Unchanged,
        };
        PlayerYuanBaoChange {
            player_id: self.player_id(),
            previous,
            current: self.yuan_bao.currency_amount(),
            outcome,
        }
    }

    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub(crate) const fn set_client_ip_snapshot(&mut self, client_ip: u32) {
        self.client_ip = client_ip;
    }

    pub(crate) fn depot_money(&self) -> u32 {
        self.bank.gold_coins_amount()
    }

    pub(crate) const fn pk_count(&self) -> u16 {
        self.base_properties.pk_count
    }

    pub(crate) const fn kill_count(&self) -> u32 {
        self.base_properties.kill_count
    }

    pub(crate) const fn criminal_state_timestamp_ms(&self) -> u32 {
        self.criminal_state_timestamp_ms
    }

    pub(crate) const fn add_murder_kill_count(&mut self, amount: u32) -> u32 {
        self.base_properties.kill_count = self.base_properties.kill_count.wrapping_add(amount);
        self.base_properties.kill_count
    }

    /// `CPKSys::ReportMurderer` добавляет только PK count; kill count уже
    /// увеличен `OnBeenMurdered` перед вызовом policy.
    pub(crate) fn report_murderer(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> u16 {
        self.base_properties.pk_count = u32::from(self.base_properties.pk_count)
            .saturating_add(pk_count_per_kill)
            .min(u32::from(u16::MAX)) as u16;
        let murderer_timestamp_started =
            self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms == 0;
        if murderer_timestamp_started {
            self.murderer_time_stamp_ms = now_ms();
        }
        tracing::trace!(
            player_id = self.player_id(),
            pk_count = self.base_properties.pk_count,
            kill_count = self.base_properties.kill_count,
            murderer_timestamp_started,
            "убийца зарегистрирован"
        );
        self.base_properties.pk_count
    }

    pub(crate) fn is_badman(&self, pk_count_per_kill: u32) -> bool {
        u32::from(self.base_properties.pk_count) > pk_count_per_kill
            || self.criminal_state_timestamp_ms != 0
    }

    /// Exact scalar часть `EnterCriminalState`: PK threshold проверяется до
    /// clock; повторный вход обновляет timestamp, но не требует around-wire.
    pub(crate) fn enter_criminal_state(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> Option<bool> {
        if u32::from(self.base_properties.pk_count) > pk_count_per_kill {
            return None;
        }
        let started = self.criminal_state_timestamp_ms == 0;
        self.criminal_state_timestamp_ms = now_ms();
        Some(started)
    }

    pub(crate) fn reset_murder_counters(&mut self) {
        let previous_pk_count = self.base_properties.pk_count;
        let previous_kill_count = self.base_properties.kill_count;
        self.base_properties.pk_count = 0;
        self.base_properties.kill_count = 0;
        tracing::trace!(
            player_id = self.player_id(),
            previous_pk_count,
            previous_kill_count,
            "счётчики убийств игрока сброшены"
        );
    }

    /// World kill confirmation tail: unsigned saturation, wrapping kill count
    /// и `OnUpdateMurdererSign` с единственным clock sample при старте timer-а.
    pub(crate) fn apply_confirmed_kill(
        &mut self,
        pk_count_per_kill: u32,
        now_ms: impl FnOnce() -> u32,
    ) -> (u16, u32) {
        self.base_properties.pk_count = u32::from(self.base_properties.pk_count)
            .saturating_add(pk_count_per_kill)
            .min(u32::from(u16::MAX)) as u16;
        self.base_properties.kill_count = self.base_properties.kill_count.wrapping_add(1);
        let murderer_timestamp_started =
            self.base_properties.pk_count != 0 && self.murderer_time_stamp_ms == 0;
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
        } else if murderer_timestamp_started {
            self.murderer_time_stamp_ms = now_ms();
        }
        tracing::trace!(
            player_id = self.player_id(),
            pk_count = self.base_properties.pk_count,
            kill_count = self.base_properties.kill_count,
            murderer_timestamp_started,
            "подтверждённое убийство применено"
        );
        (
            self.base_properties.pk_count,
            self.base_properties.kill_count,
        )
    }

    /// Exact `OnDecreaseMurdererSign`: timer идёт только у живого murderer-а,
    /// сравнение сохраняет DWORD addition/order, а при оставшемся PK исходник
    /// повторно читает `timeGetTime` для начала следующего интервала.
    pub(crate) fn decrease_murderer_sign(
        &mut self,
        one_pk_count_time_ms: u32,
        mut now_ms: impl FnMut() -> u32,
    ) -> Option<PlayerMurdererSignDecrease> {
        if self.is_dead() || self.base_properties.pk_count == 0 {
            return None;
        }
        let checked_at_ms = now_ms();
        if self
            .murderer_time_stamp_ms
            .wrapping_add(one_pk_count_time_ms)
            > checked_at_ms
        {
            return None;
        }
        self.base_properties.pk_count -= 1;
        self.murderer_time_stamp_ms = if self.base_properties.pk_count == 0 {
            0
        } else {
            now_ms()
        };
        Some(PlayerMurdererSignDecrease {
            player_id: self.player_id(),
            pk_count: self.base_properties.pk_count,
            kill_count: self.base_properties.kill_count,
            checked_at_ms,
            next_timestamp_ms: self.murderer_time_stamp_ms,
        })
    }

    pub(crate) const fn set_money_snapshot(&mut self, money: u32) {
        self.money = money;
    }

    pub(crate) const fn silence_minutes(&self) -> i32 {
        self.silence_minutes
    }

    /// Exact chat cooldown: unsigned wrapping elapsed сравнивается до
    /// mutation. Caller сохраняет исходный порядок последующих проверок.
    pub(crate) fn begin_talk(
        &mut self,
        channel: PlayerTalkChannel,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool {
        let timestamp = match channel {
            PlayerTalkChannel::Normal => &mut self.normal_talk_timestamp_ms,
            PlayerTalkChannel::Area => &mut self.area_talk_timestamp_ms,
            PlayerTalkChannel::Country => &mut self.country_talk_timestamp_ms,
            PlayerTalkChannel::World => &mut self.world_talk_timestamp_ms,
            PlayerTalkChannel::Private => &mut self.private_talk_timestamp_ms,
            PlayerTalkChannel::Team => &mut self.team_talk_timestamp_ms,
            PlayerTalkChannel::Union => &mut self.union_talk_timestamp_ms,
        };
        if now_ms.wrapping_sub(*timestamp) < interval_ms {
            return false;
        }
        *timestamp = now_ms;
        true
    }

    pub(crate) const fn equipment(&self) -> &CEquipmentContainer {
        &self.equipment
    }

    pub(crate) fn weapon_damage_level(&self, factory: &CGoodsFactory) -> i32 {
        self.equipment.get_goods(2).map_or(0, |goods| {
            goods.addon_property_value(
                factory,
                crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL,
                1,
            )
        })
    }

    /// Точный `CPlayer::GetWeaponModifier`: отрицательная разница уровней
    /// обнуляется, затем результат последовательно ограничивается сверху и
    /// снизу обычными float-сравнениями. Отдельной защиты от нулевого divisor
    /// в EXE нет, поэтому `0 / 0` намеренно остаётся `NaN`.
    pub(crate) fn weapon_modifier(
        &self,
        factory: &CGoodsFactory,
        target_level: i32,
        divisor: f32,
        minimum: f32,
    ) -> f32 {
        let delta = self
            .weapon_damage_level(factory)
            .wrapping_sub(target_level)
            .max(0);
        let mut modifier = delta as f32 / divisor;
        if modifier > 1.0 {
            modifier = 1.0;
        }
        if modifier < minimum {
            modifier = minimum;
        }
        modifier
    }

    /// Точный обход GoodsAI при первом входе: позиционные equipment/packet,
    /// одиночный hand, позиционные auction и depot. Возврат `false` повторяет
    /// исходный `break` только внутри текущего контейнера; следующий владелец
    /// всё равно обрабатывается. Закрытый depot читается через собственное
    /// базовое хранилище без временной смены lock-флага — безопасная замена
    /// исходного `Unlock(saved password) → traversal → Lock`, не меняющая
    /// наблюдаемое итоговое состояние блокировки.
    pub(crate) fn visit_login_goods_mut(
        &mut self,
        mut visit: impl FnMut(PlayerLoginGoodsLocation, &mut CGoods) -> bool,
    ) {
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Equipment, goods)
            {
                break;
            }
        }
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Packet, goods)
            {
                break;
            }
        }
        let hand_id = self
            .hand
            .traversing_goods()
            .next()
            .map(|goods| goods.identity().ex_id);
        if let Some(goods_id) = hand_id
            && let Some(goods) = self.hand.find_mut(goods_id)
        {
            let _ = visit(PlayerLoginGoodsLocation::Hand, goods);
        }
        for position in 0..self.auction_listing.size() {
            if let Some(goods) = self.auction_listing.get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Auction, goods)
            {
                break;
            }
        }
        for position in 0..self.depot.base().size() {
            if let Some(goods) = self.depot.base_mut().get_goods_mut(position)
                && !visit(PlayerLoginGoodsLocation::Depot, goods)
            {
                break;
            }
        }
    }

    pub(crate) const fn packet(&self) -> &CVolumeLimitGoodsContainer {
        &self.packet
    }

    pub(crate) const fn depot(&self) -> &CDepot {
        &self.depot
    }

    pub(crate) const fn depot_mut(&mut self) -> &mut CDepot {
        &mut self.depot
    }

    /// Маршрут живых контейнеров источника предложения обмена; таблица
    /// extend-id принадлежит Zone `trade/ctrader`.
    pub(crate) fn trade_source_goods(
        &self,
        extend_id: i32,
        position: u32,
        goods_id: CGuid,
    ) -> Option<&CGoods> {
        let goods = match TradeSourceContainer::from_extend_id(extend_id) {
            Some(TradeSourceContainer::Packet) => self.packet.get_goods(position),
            Some(TradeSourceContainer::Equipment) => self.equipment.get_goods(position),
            Some(TradeSourceContainer::Wallet) => self.wallet.get_goods(position),
            Some(TradeSourceContainer::YuanBao) => self.yuan_bao.get_goods(position),
            None => None,
        }?;
        (goods.identity().ex_id == goods_id).then_some(goods)
    }

    pub(crate) fn enhancement_selected_goods_id(&self) -> Option<CGuid> {
        self.enhancement.base().goods_id_at(0)
    }

    /// Контейнер улучшения хранит только теневые метаданные; сценарии
    /// `9409/9411` каждый раз разрешают выбранный товар обратно в его живого
    /// владельца.
    pub(crate) fn enhancement_selected_goods_mut(&mut self) -> Option<&mut CGoods> {
        let goods_id = self.enhancement_selected_goods_id()?;
        self.get_goods_by_id_mut(goods_id)
    }

    /// Владелец сценарной функции записывает доверенный серверный путь; клиент
    /// `0x8FC11/12` никогда не передаёт имя исполняемого файла.
    pub(crate) fn set_last_container_script(&mut self, script: impl AsRef<[u8]>) {
        self.last_container_script.clear();
        self.last_container_script
            .extend_from_slice(script.as_ref());
    }

    pub(crate) fn last_container_script(&self) -> &[u8] {
        &self.last_container_script
    }

    pub(crate) const fn variable_list(&self) -> &CVariableList {
        &self.variable_list
    }

    pub(crate) fn initialize_variable_list(
        &mut self,
        definitions: Option<&[u8]>,
    ) -> Result<(), GameVariableSnapshotError> {
        self.variable_list.initialize_if_empty(definitions)
    }

    pub(crate) fn set_string_variable(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> GameVariableMutationOutcome {
        self.variable_list.set_string(name, value)
    }

    pub(crate) fn set_integer_variable(
        &mut self,
        name: &[u8],
        element_index: usize,
        value: i32,
    ) -> GameVariableMutationOutcome {
        self.variable_list.set_integer(name, element_index, value)
    }

    pub(crate) fn add_integer_variable(
        &mut self,
        name: &[u8],
        value: i32,
    ) -> GameVariableMutationOutcome {
        self.variable_list.add_integer(name, value)
    }

    pub(crate) fn add_string_variable(
        &mut self,
        name: &[u8],
        value: &[u8],
    ) -> GameVariableMutationOutcome {
        self.variable_list.add_string(name, value)
    }

    pub(crate) fn clear_all_enhancement_selection(&mut self) -> usize {
        self.enhancement.clear()
    }

    pub(crate) fn record_enhancement_selection(
        &mut self,
        goods_id: CGuid,
        previous: PreviousContainer,
        placed_position: u32,
    ) -> Result<AmountShadowAdded, EnhancementSelectionBlock> {
        let goods = self
            .get_goods_by_id(goods_id)
            .ok_or(EnhancementSelectionBlock::MissingGoods)?;
        let placed = PlacedShadowGoods {
            identity: goods_id,
            position: placed_position,
            base_properties_index: goods.base_properties_index(),
            amount: goods.amount(),
        };
        self.enhancement
            .record_placed_goods(previous, placed)
            .map_err(EnhancementSelectionBlock::Shadow)
    }

    /// Exact player→enhancement часть `CC2SContainerObjectMove`: shadow не
    /// владеет goods, поэтому успешный native remove→source add безопасно
    /// свёрнут в проверку live source и атомарную запись metadata.
    pub(crate) fn select_enhancement_goods(
        &mut self,
        source_extend_id: i32,
        source_position: u32,
        goods_id: CGuid,
        amount: u32,
        factory: &CGoodsFactory,
    ) -> Result<EnhancementSelectionReport, EnhancementSelectionBlock> {
        let goods = match source_extend_id {
            1 => self.packet.get_goods(source_position),
            2 => self.equipment.get_goods(source_position),
            _ => return Err(EnhancementSelectionBlock::UnsupportedSourceContainer),
        }
        .ok_or(EnhancementSelectionBlock::MissingGoods)?;
        if goods.identity().ex_id != goods_id {
            return Err(EnhancementSelectionBlock::GoodsIdentityMismatch);
        }
        if goods.amount() != amount {
            return Err(EnhancementSelectionBlock::GoodsAmountMismatch);
        }
        match goods.can_stack(factory) {
            Ok(true) => return Err(EnhancementSelectionBlock::StackableGoods),
            Ok(false) => {}
            Err(_) => return Err(EnhancementSelectionBlock::MissingBaseProperties),
        }

        let goods = goods.identity();
        let source = PreviousContainer {
            container_type: PLAYER_TYPE,
            container_id: self.player_id(),
            container_extend_id: source_extend_id,
            goods_position: source_position,
        };
        let shadow = self.record_enhancement_selection(goods_id, source, source_position)?;
        let previous_last_operated =
            self.record_last_operated_goods(source_extend_id, source_position);
        Ok(EnhancementSelectionReport {
            goods,
            source,
            shadow,
            previous_last_operated,
        })
    }

    pub(crate) fn enhancement_original_container(
        &self,
        shadow_position: u32,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        (self.enhancement.base().goods_id_at(shadow_position) == Some(goods_id))
            .then(|| {
                self.enhancement
                    .base()
                    .original_container_information(goods_id)
            })
            .flatten()
    }

    pub(crate) fn enhancement_remove_shadow(
        &mut self,
        goods_id: CGuid,
    ) -> Option<super::container::cgoodsshadowcontainer::ShadowRemovedReport> {
        self.enhancement.base_mut().remove_shadow(goods_id)
    }

    /// Same-original-slot ветвь native shadow Remove: underlying goods после
    /// remove→add остаётся у прежнего owner-а, а здесь удаляется только shadow
    /// metadata и формируется обязательный `OT_DELETE_OBJECT` report.
    pub(crate) fn clear_enhancement_selection(
        &mut self,
        shadow_position: u32,
        goods_id: CGuid,
        amount: u32,
    ) -> Result<EnhancementDeselectionReport, EnhancementDeselectionBlock> {
        let actual_id = self
            .enhancement
            .base()
            .goods_id_at(shadow_position)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        if actual_id != goods_id {
            return Err(EnhancementDeselectionBlock::GoodsIdentityMismatch);
        }
        let source = self
            .enhancement
            .base()
            .original_container_information(goods_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        let goods = match source.container_extend_id {
            1 => self.packet.get_goods(source.goods_position),
            2 => self.equipment.get_goods(source.goods_position),
            _ => None,
        }
        .filter(|goods| goods.identity().ex_id == goods_id)
        .ok_or(EnhancementDeselectionBlock::MissingSourceGoods)?;
        if goods.amount() != amount {
            return Err(EnhancementDeselectionBlock::GoodsAmountMismatch);
        }
        let goods = goods.identity();
        let removed = self
            .enhancement
            .base_mut()
            .remove_shadow(goods_id)
            .ok_or(EnhancementDeselectionBlock::MissingShadow)?;
        Ok(EnhancementDeselectionReport {
            goods,
            source,
            removed,
        })
    }

    pub(crate) const fn packet_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.packet
    }

    pub(crate) const fn equipment_mut(&mut self) -> &mut CEquipmentContainer {
        &mut self.equipment
    }

    pub(crate) const fn battle_fairy_container(&self) -> &CBattleFairyContainer {
        &self.battle_fairy_container
    }

    pub(crate) const fn fairy_container(&self) -> &CFairyContainer {
        &self.fairy_container
    }

    pub(crate) const fn fairy_container_mut(&mut self) -> &mut CFairyContainer {
        &mut self.fairy_container
    }

    pub(crate) const fn battle_fairy_container_mut(&mut self) -> &mut CBattleFairyContainer {
        &mut self.battle_fairy_container
    }

    /// Выполняет player-часть `LoadBFDefualtProperty` для ещё не добавленного
    /// сценарного предмета: изменяет сам предмет и регистрирует три начальных
    /// навыка в каноническом `CMoveShape` игрока.
    pub(crate) fn initialize_script_battle_fairy_goods(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<(BattleFairyDefaultGoodsUpdate, Vec<BattleFairySkillAdded>)> {
        let player_id = self.player_id();
        let mut skills = Vec::with_capacity(3);
        let mut register_skill = |skill: BattleFairyDefaultSkill| {
            let registered = self
                .move_shape
                .add_skill(skill.id, skill.level, skill_factory);
            if let Some(stored) = self.move_shape.skill(skill.id, skill_factory) {
                skills.push(battle_fairy_skill_snapshot(player_id, stored, skill_factory));
            }
            registered
        };
        let report = CBattleFairyContainer::load_default_properties(
            Some(player_id),
            goods,
            factory,
            &mut register_skill,
            encode_old_client,
        )?;
        Some((report, skills))
    }

    /// Учётная запись принадлежит снимку игрока и используется точным журналом
    /// объединения; до загрузки она остаётся пустой строкой.
    pub(crate) fn set_account(&mut self, account: impl AsRef<[u8]>) {
        self.account.clear();
        self.account.extend_from_slice(account.as_ref());
    }

    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    pub(crate) fn billing_session_id(&self) -> &[u8] {
        &self.session_id
    }

    /// Exact `GetWarSoulGoods`: боевой дух — только headgear в позиции 10,
    /// чьё первое значение `GAP_BF_BATTLE_FAIRY` равно единице.
    pub(crate) fn war_soul_goods(&self, factory: &CGoodsFactory) -> Option<&CGoods> {
        self.equipment
            .get_goods(10)
            .filter(|goods| goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1)
    }

    pub(crate) fn war_soul_mana(&self, factory: &CGoodsFactory) -> Option<i32> {
        self.war_soul_goods(factory)
            .map(|goods| goods.addon_property_value(factory, GAP_BF_MP, 1))
    }

    pub(crate) fn war_soul_attack(&self, factory: &CGoodsFactory) -> Option<i32> {
        self.war_soul_goods(factory)
            .map(|goods| goods.addon_property_value(factory, GAP_BF_ATTACK, 1))
    }

    /// Временная проекция `ReplacePlayerData` для единственного вызова
    /// base-defense. Возвращаемые scale — уже точный результат последующего
    /// `RestorePlayerData`, включая legacy truncate через signed DWORD.
    pub(crate) fn war_soul_defense_projection(
        &self,
        factory: &CGoodsFactory,
        full_miss_scale: f32,
        critical_rate: f32,
        blast_defense_scale: f32,
    ) -> Option<(PlayerCombatProperties, u8, [f32; 3])> {
        let goods = self.war_soul_goods(factory)?;
        let restored_scale = |value: f32, minimum: f32| {
            let truncated = value.trunc() as i32 as u32;
            (truncated as f32).max(minimum)
        };
        let restored = [
            restored_scale(self.combat_properties.full_miss_scale(), 0.01),
            restored_scale(self.combat_properties.critical_rate(), 1.0),
            restored_scale(self.combat_properties.blast_defense_scale(), 0.01),
        ];
        let mut properties = self.combat_properties;
        properties.full_miss_scale_bits = full_miss_scale.max(0.01).to_bits();
        properties.critical_rate_bits = critical_rate.max(1.0).to_bits();
        properties.blast_defense_scale_bits = blast_defense_scale.max(0.01).to_bits();
        properties.blast_attack = goods.addon_property_value(factory, GAP_BF_BLAST, 1) as u16;
        let level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1) as u8;
        Some((properties, level, restored))
    }

    pub(crate) fn restore_war_soul_defense_projection(&mut self, restored: [f32; 3]) {
        self.combat_properties.full_miss_scale_bits = restored[0].to_bits();
        self.combat_properties.critical_rate_bits = restored[1].to_bits();
        self.combat_properties.blast_defense_scale_bits = restored[2].to_bits();
    }

    /// Специальная ветвь `OnBeenAttacked(..., true)` сферы хаоса: урон
    /// сначала уменьшает `GAP_BF_HP` с точной wrapping-арифметикой, разрушение
    /// сбрасывает состояние, после чего формируется полный снимок старого клиента.
    pub(crate) fn apply_war_soul_hit(
        &mut self,
        damage: i32,
        factory: &CGoodsFactory,
        da_kong_key: bool,
    ) -> Option<WarSoulHitOutcome> {
        if self.war_soul_state == 0 { return None }
        let player_id = self.player_id();
        let goods = self.equipment_mut().get_goods_mut(10)?;
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 { return None }
        let cut_hurt = goods.addon_property_value(factory, GAP_BF_CUT_HURT_SCALE, 1);
        let health = goods.addon_property_value(factory, GAP_BF_HP, 1);
        let next = health.wrapping_sub(10_000i32.wrapping_sub(cut_hurt).wrapping_mul(damage));
        let broken = next < 1;
        let stored = if broken { 0 } else { next };
        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, stored);
        let identity = goods.identity();
        let mut old_client_payload = Vec::new();
        if !goods.serialize_for_old_client(&mut old_client_payload, factory, da_kong_key) { return None }
        let broadcast_previous_status = broken && self.set_war_soul_status(0);
        Some(WarSoulHitOutcome {
            broken,
            broadcast_previous_status,
            update: super::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate {
                message_type: 0x000b_f918,
                player_id,
                goods: identity,
                old_client_payload,
            },
        })
    }

    /// Exact `GetGoodsById` lookup order для goods-message `0x8FC2E`.
    /// Locked hand/packet/auction goods скрываются container `find`, equipment
    /// использует собственный positional storage.
    pub(crate) fn get_goods_by_id(&self, goods_id: CGuid) -> Option<&CGoods> {
        self.hand
            .find(goods_id)
            .or_else(|| self.packet.base().find(goods_id))
            .or_else(|| self.equipment.find(goods_id))
            .or_else(|| self.auction_listing.base().find(goods_id))
            .or_else(|| self.auction_goods.base().find(goods_id))
    }

    pub(crate) fn get_goods_by_id_mut(&mut self, goods_id: CGuid) -> Option<&mut CGoods> {
        if self.hand.find(goods_id).is_some() {
            return self.hand.find_mut(goods_id);
        }
        if self.packet.base().find(goods_id).is_some() {
            return self.packet.base_mut().find_mut(goods_id);
        }
        if self.equipment.find(goods_id).is_some() {
            return self.equipment.find_mut(goods_id);
        }
        if self.auction_listing.base().find(goods_id).is_some() {
            return self.auction_listing.base_mut().find_mut(goods_id);
        }
        self.auction_goods.base_mut().find_mut(goods_id)
    }

    pub(crate) const fn hand_mut(&mut self) -> &mut CAmountLimitGoodsContainer {
        &mut self.hand
    }

    pub(crate) const fn hand(&self) -> &CAmountLimitGoodsContainer {
        &self.hand
    }

    pub(crate) const fn auction_goods_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.auction_goods
    }

    pub(crate) const fn auction_goods(&self) -> &CVolumeLimitGoodsContainer {
        &self.auction_goods
    }

    /// Точная часть состояния `CPlayer::ModifyAuctionSpace`: значение должно
    /// находиться между текущим числом свободных расширенных ячеек и полным
    /// объёмом контейнера. Исходные методы контейнера работают только с
    /// расширенной областью, начинающейся с позиции 48.
    pub(crate) fn modify_auction_space(
        &mut self,
        requested: u32,
        pack_add_enabled: bool,
    ) -> Option<u32> {
        if requested > self.auction_goods.size() || requested < self.base_properties.auction_space {
            return None;
        }
        self.auction_goods.set_all_inactive();
        if pack_add_enabled {
            self.auction_goods
                .apply_player_expansion_limit(self.auction_goods.size().wrapping_sub(requested));
        }
        let current = self.auction_goods.expansion_available_space();
        self.base_properties.auction_space = current;
        Some(current)
    }

    pub(crate) const fn auction_listing(&self) -> &CVolumeLimitGoodsContainer {
        &self.auction_listing
    }

    pub(crate) const fn auction_listing_mut(&mut self) -> &mut CVolumeLimitGoodsContainer {
        &mut self.auction_listing
    }

    pub(crate) fn auction_listing_extension_bonus(&self, factory: &CGoodsFactory) -> i32 {
        let Some(goods) = self.auction_listing.get_goods(1) else {
            return 0;
        };
        if goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 1) != 3 {
            return 0;
        }
        goods.addon_property_value(factory, GAP_GOODS_PACKAGE_EXTENTION, 2)
    }

    pub(crate) fn take_auction_listing_goods(&mut self) -> Option<CGoods> {
        let goods_id = self.auction_listing.get_goods(0)?.identity().ex_id;
        let outcome = self.auction_listing.remove_goods(goods_id)?;
        let taken = match outcome {
            VolumeGoodsRemoveOutcome::Removed(taken)
            | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
        };
        Some(match taken {
            crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken::Removed(removed) => removed.goods,
            crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsTaken::Split(split) => split.goods,
        })
    }

    /// Exact state-owner возврата `0x80404`: позиция выбирается до Add,
    /// stack merge использует обычный player-progress gate, а bind value-id 2
    /// записывается уже в итоговый stored goods.
    pub(crate) fn return_auction_goods(
        &mut self,
        goods: CGoods,
        bind_type: i32,
        factory: &CGoodsFactory,
    ) -> Option<PlayerAuctionGoodsReturn> {
        let position = self
            .auction_goods
            .find_position_for_goods(&goods, factory)?;
        let source = goods.identity();
        let owner_progress_allows = self.current_progress == PlayerProgress::None;
        let mut incoming = Some(goods);
        let outcome = self.auction_goods.add_goods_at(
            position,
            &mut incoming,
            factory,
            owner_progress_allows,
        );
        let successful = matches!(
            &outcome,
            VolumeGoodsAddOutcome::Added(_)
                | VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged { .. })
        );
        let (resulting_goods, resulting_amount, bind_stored) = if successful {
            let stored = self
                .auction_goods
                .get_goods_mut(position)
                .expect("успешный auction Add обязан оставить stored goods");
            let bind_stored = stored.set_addon_property_value_core(GAP_GOODS_BIND, 2, bind_type);
            (Some(stored.identity()), Some(stored.amount()), bind_stored)
        } else {
            (None, None, false)
        };
        Some(PlayerAuctionGoodsReturn {
            player_id: self.player_id(),
            position,
            source,
            outcome,
            resulting_goods,
            resulting_amount,
            bind_stored,
        })
    }

    pub(crate) fn auction_money(&self) -> u32 {
        self.auction_wallet.currency_amount()
    }

    /// Правило `CheckAuctionMoneyMove` исполняет Zone trade над снятыми с
    /// кошельков и фабрики скалярами; уведомление `GPM015` у caller-а.
    pub(crate) fn auction_money_move_capacity(
        &self,
        factory: &CGoodsFactory,
    ) -> AuctionMoneyMoveCapacity {
        check_auction_money_move(
            self.wallet.currency_amount(),
            self.auction_wallet.currency_amount(),
            self.wallet.max_stack_number(factory),
        )
    }

    pub(crate) fn auction_money_goods(&self) -> Option<&CGoods> {
        self.auction_wallet.get_goods(0)
    }

    /// State-часть exact `SetAuctionMoney`; аргумент является новым абсолютным
    /// балансом, а caller создаёт недостающий MONEY и публикует extend-id 15;
    /// развилка направления принадлежит Zone `trade/currency`.
    pub(crate) fn set_auction_money(
        &mut self,
        current: u32,
        factory: &CGoodsFactory,
        created_currency: Vec<CGoods>,
    ) -> PlayerAuctionMoneyChange {
        let previous = self.auction_wallet.currency_amount();
        let outcome = match balance_set_direction(previous, current) {
            BalanceSetDirection::Increase(delta) => {
                let mut created_currency = Some(created_currency);
                PlayerAuctionMoneyChangeOutcome::Increased(
                    self.auction_wallet.increase_currency(
                        delta,
                        factory,
                        move |_, _| created_currency.take().unwrap_or_default(),
                    ),
                )
            }
            BalanceSetDirection::Decrease(delta) => PlayerAuctionMoneyChangeOutcome::Decreased(
                self.auction_wallet.decrease_currency(delta, factory),
            ),
            BalanceSetDirection::Unchanged => PlayerAuctionMoneyChangeOutcome::Unchanged,
        };
        PlayerAuctionMoneyChange {
            player_id: self.player_id(),
            previous,
            current: self.auction_wallet.currency_amount(),
            outcome,
        }
    }

    pub(crate) const fn set_auction_open(&mut self, open: bool) {
        self.auction.set_open(open);
    }

    /// State/container часть exact `TellClientScale`; закрытый аукцион не
    /// создаёт client-effect, открытый уменьшает положительные scale-счётчики
    /// и сохраняет container traversal order.
    pub(crate) fn auction_scale_goods_ids(
        &mut self,
        factory: &CGoodsFactory,
    ) -> Option<Vec<CGuid>> {
        self.auction
            .is_open()
            .then(|| self.auction_goods.get_scale_goods(factory))
    }

    pub(crate) fn auction_goods_identity_at(&self, position: u32) -> Option<ShapeIdentity> {
        self.auction_goods.get_goods(position).map(CGoods::identity)
    }

    /// Clock-гейт покупки лота исполняет вложенное в игрока состояние
    /// аукциона из Zone trade.
    pub(crate) fn begin_auction_buy(&mut self, tick_ms: impl FnMut() -> u32) -> AuctionBuyGate {
        self.auction.begin_buy(tick_ms)
    }

    /// Пятисекундный гейт выставления исполняет вложенное состояние аукциона
    /// из Zone trade.
    pub(crate) fn begin_auction_listing(
        &mut self,
        tick_ms: impl FnMut() -> u32,
    ) -> AuctionListingGate {
        self.auction.begin_listing(tick_ms)
    }

    /// Секундный limit-гейт исполняет вложенное состояние аукциона из Zone
    /// trade; счётчики комнаты и пределы setup снимает message caller.
    pub(crate) fn begin_auction_limit_check(
        &mut self,
        tick_ms: u32,
        owner_goods_count: usize,
        global_goods_count: usize,
        player_maximum: f32,
        global_maximum: f32,
        extension_bonus: i32,
    ) -> bool {
        self.auction.begin_limit_check(
            tick_ms,
            owner_goods_count,
            global_goods_count,
            player_maximum,
            global_maximum,
            extension_bonus,
        )
    }

    pub(crate) fn current_auction_node(&self) -> Option<&CGoodsNode> {
        self.auction.current_node()
    }

    pub(crate) fn set_current_auction_node(&mut self, node: CGoodsNode) -> bool {
        self.auction.set_current_node(node)
    }

    /// Делегирует запись узла вложенному состоянию аукциона из Zone trade
    /// (точное безусловное замещение `AutoAddAuctionGoods`).
    pub(crate) fn replace_current_auction_node(&mut self, node: CGoodsNode) {
        self.auction.replace_current_node(node);
    }

    pub(crate) fn take_current_auction_node(&mut self) -> Option<CGoodsNode> {
        self.auction.take_current_node()
    }

    pub(crate) const fn auction_listing_fee(&self) -> u32 {
        self.auction.listing_fee()
    }

    pub(crate) const fn set_auction_listing_fee(&mut self, fee: u32) {
        self.auction.set_listing_fee(fee);
    }

    pub(crate) fn current_auction_buy_node(&self) -> Option<&CGoodsNode> {
        self.auction.current_buy_node()
    }

    pub(crate) fn set_current_auction_buy_node(&mut self, node: CGoodsNode) -> bool {
        self.auction.set_current_buy_node(node)
    }

    pub(crate) fn take_current_auction_buy_node(&mut self) -> Option<CGoodsNode> {
        self.auction.take_current_buy_node()
    }

    pub(crate) fn client_ip_text(&self) -> Vec<u8> {
        legacy_ipv4_text(self.client_ip)
    }

    pub(crate) fn begin_auction_search(
        &mut self,
        name: &[u8],
        lower_level: i32,
        upper_level: i32,
        use_self: i32,
        money_type: i32,
        weapon_type: i32,
    ) {
        self.auction.begin_search(
            name,
            lower_level,
            upper_level,
            use_self,
            money_type,
            weapon_type,
        );
    }

    /// Гейт `ReFlushSelfGoods` исполняет вложенное состояние аукциона из Zone
    /// trade; оба space-скаляра снимаются с контейнеров до вызова.
    pub(crate) fn refresh_auction_self_goods(
        &mut self,
        factory: &CGoodsFactory,
        tick_ms: impl FnMut() -> u32,
    ) -> AuctionSelfGoodsRefresh {
        let wallet_amount = self.auction_wallet.currency_amount();
        let wallet_maximum = self.auction_wallet.max_stack_number(factory);
        self.auction.begin_self_goods_refresh(
            tick_ms,
            self.auction_goods.space(),
            wallet_maximum.wrapping_sub(wallet_amount),
        )
    }

    /// Exact derived `bHasPet`: отдельный pet owner materializes list later;
    /// этому caller-у нужен только подтверждённый факт её непустоты.
    pub(crate) const fn set_active_pet_count(&mut self, count: u32) {
        self.active_pet_count = count;
    }

    pub(crate) fn take_uncreated_pets(&mut self) -> Vec<PlayerUncreatedPet> {
        std::mem::take(&mut self.uncreated_pets)
    }

    pub(crate) fn login_carriage(&self) -> (&PlayerUncreatedCarriage, bool) {
        (&self.uncreated_carriage, self.recreate_carriage)
    }

    pub(crate) fn finish_login_carriage_recreation(&mut self, carriage_id: i32) {
        self.active_carriage_id = carriage_id;
        self.recreate_carriage = false;
        self.uncreated_carriage = PlayerUncreatedCarriage::default();
    }

    pub(crate) fn finish_empty_login_carriage_recreation(&mut self) {
        self.finish_login_carriage_recreation(0);
    }

    pub(crate) const fn bind_active_carriage(&mut self, carriage_id: i32) {
        self.active_carriage_id = carriage_id;
    }

    pub(crate) const fn active_carriage_id(&self) -> i32 {
        self.active_carriage_id
    }

    pub(crate) const fn clear_active_carriage(&mut self, carriage_id: i32) -> bool {
        if self.active_carriage_id != carriage_id {
            return false;
        }
        self.active_carriage_id = 0;
        true
    }

    pub(crate) const fn current_pets_mode(&self) -> i32 {
        self.move_shape.current_pets_mode()
    }

    pub(crate) fn set_current_pets_mode(&mut self, mode: i32) -> bool {
        self.move_shape.set_current_pets_mode(mode)
    }

    pub(crate) fn add_active_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        self.move_shape.add_pet(object_type, id, figure);
        self.active_pet_count = self.move_shape.pets().len() as u32;
    }

    pub(crate) fn remove_active_pet(&mut self, object_type: i32, id: i32) -> bool {
        let removed = self.move_shape.remove_pet(object_type, id);
        self.active_pet_count = self.move_shape.pets().len() as u32;
        removed
    }

    pub(crate) fn active_pets(&self) -> &[super::moveshape::MoveShapePet] {
        self.move_shape.pets()
    }

    pub(crate) fn learned_skill_level(&self, skill_id: u32, factory: &CSkillFactory) -> i32 {
        self.move_shape.skill_level(skill_id, factory)
    }

    pub(crate) fn learned_skill_level_if_present(&self, skill_id: u32, factory: &CSkillFactory) -> Option<i32> {
        self.move_shape.skill(skill_id, factory).map(|skill| skill.level())
    }

    /// Создание intrinsic skills из `CPlayer::InitSkills` (0x00440C30):
    /// отсутствующая базовая защита добавляется первой,
    /// затем профессии `0/1/2` получают соответственно обычную атаку,
    /// атаку со стрельбой либо базовую магию. Уже загруженные записи не
    /// заменяются. CGame вызывает это после login-script; последующий
    /// SetCurrentSkill(GetDefaultAttackSkillID) остаётся у опубликованного owner-а.
    pub(crate) fn initialize_intrinsic_skills(&mut self, factory: &CSkillFactory) {
        if self.move_shape.skill(SKILL_BASE_DEFENSE, factory).is_none() {
            self.move_shape.add_base_defense_skill(factory);
        }
        let intrinsic = match self.occupation() {
            0 => &[BASE_ATTACK_SKILL_ID][..],
            1 => &[BASE_ATTACK_SKILL_ID, ARCHERY_SKILL_ID][..],
            2 => &[BASE_MAGIC_SKILL_ID][..],
            _ => &[],
        };
        for &skill_id in intrinsic {
            if self.move_shape.skill(skill_id, factory).is_none() {
                let _ = self.move_shape.add_skill(skill_id, 1, factory);
            }
        }
    }

    pub(crate) const fn has_pet(&self) -> bool {
        self.active_pet_count != 0
    }

    /// Snapshot/skill caller передаёт только current ID, достаточный для
    /// `SummonBF` запрета `SKILL_MONSTER_TAMING`; concrete skill execution не
    /// становится частью player owner-а.
    pub(crate) const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.move_shape.set_current_skill_id(skill_id);
    }

    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.move_shape.current_skill_id()
    }

    /// Exact `CPlayer::GetDefaultAttackSkillID`: лучник использует базовую
    /// стрельбу только с луком, арбалетом либо weapon-category `8`; маг всегда
    /// возвращается к базовой магии, остальные варианты — к обычной атаке.
    pub(crate) fn default_attack_skill_id(&self, factory: &CGoodsFactory) -> u32 {
        match self.occupation() {
            1 if self.equipment.get_goods(2).is_some_and(|weapon| {
                matches!(
                    weapon.addon_property_value(factory, GAP_WEAPON_CATEGORY, 1),
                    3 | 4 | 8
                )
            }) => ARCHERY_SKILL_ID,
            2 => BASE_MAGIC_SKILL_ID,
            _ => BASE_ATTACK_SKILL_ID,
        }
    }

    /// Native `OnChangeSkill` и `OnLoseTarget` назначают вычисленный после
    /// concrete `End` default skill. Наличие выбранного ID не означает
    /// незавершённое исполнение: оно хранится отдельно в CPlayerAI.
    pub(crate) const fn restore_default_attack_skill_after_end(
        &mut self,
        default_attack_skill_id: u32,
    ) {
        self.move_shape.set_current_skill_id(Some(default_attack_skill_id));
    }

    pub(crate) const fn war_soul_state(&self) -> u32 {
        self.war_soul_state
    }

    pub(crate) const fn war_soul_point(&self) -> WarSoulPoint {
        self.war_soul_point
    }

    pub(crate) const fn battle_fairy_summoned(&self) -> bool {
        self.battle_fairy_summoned
    }

    /// Exact `SetWarSoulStaus`: around status публикуется по прежнему state,
    /// затем любое значение кроме единицы нормализуется к нулю. Правило
    /// перенесено буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) const fn set_war_soul_status(&mut self, value: u32) -> bool {
        nebokrai_zone::skills::battlefairysummon::set_war_soul_status(
            &mut self.battle_fairy_summoned,
            &mut self.war_soul_state,
            value,
        )
    }

    /// Исполняет player-часть `CBattleFairyContainer::SummonBF`. Spatial map
    /// принадлежит `CServerRegion`, поэтому действие возвращается явным
    /// tail-ом для `CGame`; ordered notify/broadcast/property effects там
    /// сериализуются concrete wire после spatial mutation. Правило перенесено
    /// буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) fn summon_battle_fairy(
        &mut self,
        battle_fairy_enabled: bool,
        mode: i32,
        factory: &CGoodsFactory,
    ) -> BattleFairySummonReport {
        let player_id = self.player_id();
        let has_pet = self.has_pet();
        let active_skill_id = self.move_shape.current_skill_id();
        let shape = self.move_shape.shape();
        let resolution = nebokrai_zone::skills::battlefairysummon::summon_battle_fairy(
            &mut BattleFairyWarSoul {
                summoned: &mut self.battle_fairy_summoned,
                state: &mut self.war_soul_state,
                recall: &mut self.base_properties.battle_fairy_recall,
                died: &mut self.base_properties.battle_fairy_died,
                visual_x_bits: &mut self.war_soul_visual_x_bits,
                visual_y_bits: &mut self.war_soul_visual_y_bits,
                point: &mut self.war_soul_point,
            },
            player_id,
            battle_fairy_enabled,
            mode,
            has_pet,
            active_skill_id,
            |property| {
                self.equipment
                    .get_goods(10)
                    .map(|goods| goods.addon_property_value(factory, property, 1))
            },
            || (shape.get_tile_x(), shape.get_tile_y()),
        );
        BattleFairySummonReport {
            player_id,
            outcome: resolution.outcome,
            region_id: self.server_region_id,
            spatial_action: resolution.spatial_action,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Полный player-tail успешного `CEquipmentContainer::Remove`: container
    /// mutation предшествует callback-ам, поэтому removed slot уже отсутствует
    /// во время injected результата virtual `PropertiesChanged`.
    pub(crate) fn remove_equipment_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        runtime: PlayerEquipmentRemoveRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&mut CPlayer) -> PlayerPropertyRecompute,
    ) -> PlayerEquipmentRemoveReport {
        let player_id = self.player_id();
        let mut outcome = self.equipment.remove(
            ex_id,
            factory,
            EquipmentRemoveRuntimeFacts {
                owner_player_present: true,
                pack_add_enabled: runtime.pack_add_enabled,
                player_goods_package_extension: runtime.player_goods_package_extension,
                active_war_soul_blocks_headgear: runtime.active_war_soul_blocks_headgear,
            },
        );
        let mut effects = Vec::new();
        if let EquipmentRemoveOutcome::Removed(removed) = &mut outcome {
            self.equipment_changed = true;
            let now_seconds = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.unregister_goods_ai(&mut removed.goods, factory, now_seconds);
        }
        if let EquipmentRemoveOutcome::Removed(removed) = &outcome
            && let Some(player_effects) = removed.event.player_effects
        {
            if player_effects.clear_war_soul_status && self.set_war_soul_status(0) {
                effects.push(PlayerEquipmentRemoveEffect::WarSoulStatusAround {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: [400, player_id],
                });
            }
            if player_effects.delete_war_soul_skill {
                for property in EQUIPPED_SKILL_PROPERTIES {
                    let skill_id = war_soul_skill_id_from_goods(&removed.goods, factory, property);
                    let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
                    effects.push(PlayerEquipmentRemoveEffect::WarSoulSkillDetached { skill_id });
                    if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                        effects.push(PlayerEquipmentRemoveEffect::SkillRemoved(
                            BattleFairySkillRemoved {
                                message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                                player_id,
                                skill_id,
                                skill_name: skill.name(skill_factory).map(<[u8]>::to_vec),
                            },
                        ));
                    }
                }
            }
            if player_effects.recompute_without_removed_slot {
                let recompute = recompute_properties(self);
                self.apply_recomputed_combat_properties(recompute.properties, factory);
                effects.push(
                    PlayerEquipmentRemoveEffect::PropertiesChangedWithoutRemovedSlot {
                        column: removed.event.column,
                        combat_properties: self.combat_properties,
                        ci_qing_result_values: recompute.ci_qing_result_values,
                    },
                );
            }
            if player_effects.clamp_hp_and_mp {
                let previous_health = self.health();
                let previous_mana = self.mana();
                self.set_health(previous_health);
                self.set_mana(previous_mana);
                effects.push(PlayerEquipmentRemoveEffect::VitalsClamped {
                    previous_health,
                    current_health: self.health(),
                    previous_mana,
                    current_mana: self.mana(),
                });
            }
            effects.push(PlayerEquipmentRemoveEffect::AroundUpdate(
                player_effects.around_update,
            ));
        }
        PlayerEquipmentRemoveReport {
            player_id,
            outcome,
            effects: effects.into_iter().collect(),
        }
    }

    /// Полный player-tail positional `CEquipmentContainer::Add`. Timed и
    /// goods-AI partial effects остаются наблюдаемы даже при late block; skill,
    /// properties, around и package-log выполняются только после commit.
    pub(crate) fn add_equipment_goods(
        &mut self,
        position: u32,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        runtime: PlayerEquipmentAddRuntimeFacts,
        recompute_properties: &mut dyn FnMut(&mut CPlayer) -> PlayerPropertyRecompute,
        publish_effect: &mut dyn FnMut(&CPlayer, PlayerEquipmentAddEffect),
        on_goods_added: &mut dyn FnMut(&mut CPlayer, u32),
    ) -> PlayerEquipmentAddReport {
        let player_id = self.player_id();
        let previous_expanded_package_num = self.equipment.expanded_package_num();
        let can_mount_result = incoming
            .as_ref()
            .map_or(0, |goods| self.can_mount_equip(goods, factory));
        let container_runtime = EquipmentAddRuntimeFacts {
            owner_player: Some(EquipmentOwnerPlayerFacts { can_mount_result }),
            pack_add_enabled: runtime.pack_add_enabled,
            now: runtime.now,
        };
        let outcome = {
            let current_ticket = self.current_ticket;
            let goods_ai_tree = &mut self.goods_ai_tree;
            let goods_ai_delete_queue = &mut self.goods_ai_delete_queue;
            let mut register_with_goods_ai = |goods: &mut CGoods| {
                if let Some((ticket, goods_id)) = Self::prepare_goods_ai_registration_with_clock(
                    current_ticket,
                    goods,
                    factory,
                    &mut crate::gameserver::gameserver::game::game_wall_time_seconds,
                ) {
                    Self::record_goods_ai_registration_in(
                        current_ticket,
                        goods_ai_tree,
                        goods_ai_delete_queue,
                        ticket,
                        goods_id,
                    );
                }
            };
            if position == u32::MAX {
                self.equipment.add_preferred(
                    incoming,
                    factory,
                    container_runtime,
                    &mut register_with_goods_ai,
                )
            } else {
                self.equipment.add_at(
                    position,
                    incoming,
                    factory,
                    container_runtime,
                    &mut register_with_goods_ai,
                )
            }
        };
        if matches!(&outcome, EquipmentAddOutcome::Added(_)) {
            self.equipment_changed = true;
        }
        if let EquipmentAddOutcome::Added(added) = &outcome
            && let Some(player_effects) = added.player_effects
        {
            if added.package_extension_applied {
                self.equipment
                    .set_expanded_package_num_snapshot(previous_expanded_package_num);
            }
            if player_effects.add_war_soul_skill
                && self.equipment.get_goods(added.column.position()).is_some()
            {
                for property in EQUIPPED_SKILL_PROPERTIES {
                    let (skill_id, level) = {
                        let goods = self
                            .equipment
                            .get_goods(added.column.position())
                            .expect("присоединение навыка не отделяет предмет экипировки");
                        war_soul_skill_entry_from_goods(goods, factory, property)
                    };
                    let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
                    publish_effect(
                        self,
                        PlayerEquipmentAddEffect::WarSoulSkillAttached { skill_id, level },
                    );
                    if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                        publish_effect(
                            self,
                            PlayerEquipmentAddEffect::SkillAdded(
                                battle_fairy_skill_snapshot(player_id, skill, skill_factory),
                            ),
                        );
                    }
                }
            }
            if player_effects.recompute_properties {
                let recompute = recompute_properties(self);
                self.apply_recomputed_combat_properties(recompute.properties, factory);
                publish_effect(
                    self,
                    PlayerEquipmentAddEffect::PropertiesChanged {
                        combat_properties: self.combat_properties,
                        ci_qing_result_values: recompute.ci_qing_result_values,
                    },
                );
            }
            publish_effect(
                self,
                PlayerEquipmentAddEffect::AroundUpdate(player_effects.around_update),
            );
            if added.package_extension_applied {
                self.equipment.set_expanded_package_num_snapshot(
                    previous_expanded_package_num.wrapping_add(added.package_extension_delta),
                );
                publish_effect(
                    self,
                    PlayerEquipmentAddEffect::PackageExtensionLogged {
                        category: "PackExpand",
                        string_id: "KR002",
                        expanded_package_num: self.equipment.expanded_package_num(),
                    },
                );
            }
        }
        if let EquipmentAddOutcome::Added(added) = &outcome
            && let Some(goods) = self.equipment.get_goods(added.column.position())
        {
            let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
            on_goods_added(self, additional);
        }
        PlayerEquipmentAddReport { player_id, outcome }
    }

    /// Завершает принадлежащий `CGame` хвост области: `spatial_applied`
    /// означает найденную нужную area, а не изменение её map entry. Правило
    /// перенесено буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) const fn apply_war_soul_action(
        &mut self,
        action: BattleFairyWarSoulAction,
        spatial_applied: bool,
    ) {
        nebokrai_zone::skills::battlefairysummon::apply_war_soul_action(
            &mut self.war_soul_point,
            action,
            spatial_applied,
        )
    }

    /// Active WarSoul tail `CPlayer::OnEnterRegion`: visual float координаты
    /// возвращаются к клетке хозяина; spatial point применяет координатор
    /// после target-area gate и End(int,0) выбранного навыка. Правило
    /// перенесено буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) fn prepare_war_soul_region_entry(
        &mut self,
    ) -> Option<(BattleFairyWarSoulAction, u32, u32)> {
        let shape = self.move_shape.shape();
        nebokrai_zone::skills::battlefairysummon::prepare_war_soul_region_entry(
            self.war_soul_state,
            &mut self.war_soul_visual_x_bits,
            &mut self.war_soul_visual_y_bits,
            self.war_soul_point,
            || -> Option<(i32, i32)> { Some((shape.get_tile_x().ok()?, shape.get_tile_y().ok()?)) },
        )
    }

    /// Один проход живой ветви `ComputeWarSoulXY`. `Some(false)` означает
    /// найденный текущий навык боевой феи с `IsRestored()==0`; `None` точно
    /// соответствует отсутствующему навыку и не блокирует следование. Правило
    /// перенесено буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) fn compute_war_soul_xy(
        &mut self,
        current_war_soul_skill_restored: Option<bool>,
    ) -> BattleFairyFollowPlan {
        let player_id = self.player_id();
        let shape = self.move_shape.shape();
        let resolution = nebokrai_zone::skills::battlefairysummon::compute_war_soul_xy(
            &mut BattleFairyWarSoul {
                summoned: &mut self.battle_fairy_summoned,
                state: &mut self.war_soul_state,
                recall: &mut self.base_properties.battle_fairy_recall,
                died: &mut self.base_properties.battle_fairy_died,
                visual_x_bits: &mut self.war_soul_visual_x_bits,
                visual_y_bits: &mut self.war_soul_visual_y_bits,
                point: &mut self.war_soul_point,
            },
            player_id,
            current_war_soul_skill_restored,
            || (shape.get_tile_x(), shape.get_tile_y()),
        );
        BattleFairyFollowPlan {
            player_id,
            outcome: resolution.outcome,
            region_id: self.server_region_id,
            spatial_action: resolution.spatial_action,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Мёртвая ветвь сразу после `CMoveShape::AI`: пространственная позиция
    /// получает точное `(-1,-1)`, обе визуальные координаты `float` становятся
    /// `-1.0`, но исходник не публикует пакет движения вокруг. Правило
    /// перенесено буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) fn clear_dead_war_soul_xy(&mut self) -> BattleFairyFollowPlan {
        let player_id = self.player_id();
        let resolution = nebokrai_zone::skills::battlefairysummon::clear_dead_war_soul_xy(
            &mut BattleFairyWarSoul {
                summoned: &mut self.battle_fairy_summoned,
                state: &mut self.war_soul_state,
                recall: &mut self.base_properties.battle_fairy_recall,
                died: &mut self.base_properties.battle_fairy_died,
                visual_x_bits: &mut self.war_soul_visual_x_bits,
                visual_y_bits: &mut self.war_soul_visual_y_bits,
                point: &mut self.war_soul_point,
            },
        );
        BattleFairyFollowPlan {
            player_id,
            outcome: resolution.outcome,
            region_id: self.server_region_id,
            spatial_action: resolution.spatial_action,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Владеющая state/container середина оставшегося `CPlayer::AI` tail.
    /// GoodsAI/delete выполняется соседним CGame caller-ом до ticket mutation;
    /// packet expansion принадлежит самому player-у. Flash и TaoZhuang затем
    /// завершаются concrete CGame owner-ом.
    pub(crate) fn advance_ai_ticket_and_packet(
        &mut self,
        pack_add_enabled: bool,
    ) -> (u32, Option<u32>) {
        self.current_ticket = self.current_ticket.wrapping_add(1);
        let expanded_package_num = pack_add_enabled.then(|| {
            let expanded = self.equipment.expanded_package_num();
            self.packet.set_all_inactive();
            self.packet.apply_player_expansion_limit(expanded);
            expanded
        });
        (self.current_ticket, expanded_package_num)
    }

    pub(crate) const fn current_ticket(&self) -> u32 {
        self.current_ticket
    }

    /// Exact `CheckAddGoods -> ComputeTicket`: remaining lifetime переводится
    /// в AI tickets множителем 12.5. Пара возвращается наружу, чтобы container
    /// borrow завершился до mutation player map-а.
    pub(crate) fn prepare_goods_ai_registration(
        current_ticket: u32,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) -> Option<(u32, CGuid)> {
        Self::prepare_goods_ai_registration_with_clock(
            current_ticket, goods, factory, &mut || now_seconds,
        )
    }

    fn prepare_goods_ai_registration_with_clock(
        current_ticket: u32,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: &mut dyn FnMut() -> u64,
    ) -> Option<(u32, CGuid)> {
        if !goods.query_attribute(GAP_GOODS_LIFE_TYPE) || goods.add_ticket() != 0 {
            return None;
        }
        let time_type = goods.goods_time_type(factory);
        let mut start = goods.start_point(factory);
        if !matches!(time_type, 1 | 3) && (!matches!(time_type, 2 | 4) || start == 0) {
            return None;
        }
        let now_seconds = now_seconds();
        if start == 0 {
            goods.set_start_point(now_seconds);
            start = now_seconds;
        }
        let elapsed = if start < now_seconds {
            (now_seconds as u32).wrapping_sub(start as u32)
        } else {
            0
        };
        let lifetime = goods.goods_lifetime(factory);
        if elapsed.wrapping_add(5) >= lifetime {
            return Some((current_ticket, goods.identity().ex_id));
        }
        let delta = ((u64::from(lifetime.wrapping_sub(elapsed)) * 25) / 2) as u32;
        if delta == 0 {
            return None;
        }
        let ticket = current_ticket.wrapping_add(delta);
        goods.set_add_ticket(ticket);
        (goods.add_ticket() != 0).then_some((ticket, goods.identity().ex_id))
    }

    pub(crate) fn record_goods_ai_registration(&mut self, ticket: u32, goods_id: CGuid) {
        Self::record_goods_ai_registration_in(
            self.current_ticket,
            &mut self.goods_ai_tree,
            &mut self.goods_ai_delete_queue,
            ticket,
            goods_id,
        );
    }

    fn record_goods_ai_registration_in(
        current_ticket: u32,
        goods_ai_tree: &mut BTreeMap<u32, BTreeSet<CGuid>>,
        goods_ai_delete_queue: &mut VecDeque<BTreeSet<CGuid>>,
        ticket: u32,
        goods_id: CGuid,
    ) {
        if ticket <= current_ticket {
            goods_ai_delete_queue.push_back(BTreeSet::from([goods_id]));
        } else {
            goods_ai_tree.entry(ticket).or_default().insert(goods_id);
        }
    }

    fn unregister_goods_ai(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) {
        let ticket = goods.add_ticket();
        if ticket == 0 {
            return;
        }
        let goods_id = goods.identity().ex_id;
        let mut registered = false;
        if let Some(bucket) = self.goods_ai_tree.get_mut(&ticket) {
            registered = bucket.remove(&goods_id);
            if bucket.is_empty() {
                self.goods_ai_tree.remove(&ticket);
            }
        }
        if !registered {
            return;
        }
        let start = goods.start_point(factory);
        let elapsed = if start < now_seconds {
            (now_seconds as u32).wrapping_sub(start as u32)
        } else {
            0
        };
        let lifetime = goods.goods_lifetime(factory);
        if elapsed.wrapping_add(5) >= lifetime {
            let _ = goods.set_addon_property_value_core(GAP_GOODS_LIFE_TYPE, 1, 0);
            self.goods_ai_delete_queue
                .push_back(BTreeSet::from([goods_id]));
            return;
        }
        let _ = goods.set_addon_property_value_core(
            GAP_GOODS_LIFE_TYPE,
            1,
            lifetime.wrapping_sub(elapsed) as i32,
        );
        goods.set_start_point(0);
        goods.set_add_ticket(0);
    }

    pub(crate) fn register_goods_ai_by_id(
        &mut self,
        goods_id: CGuid,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) -> bool {
        let current_ticket = self.current_ticket;
        let registration = self.get_goods_ai_by_id_mut(goods_id).and_then(|goods| {
            Self::prepare_goods_ai_registration(current_ticket, goods, factory, now_seconds)
        });
        let Some((ticket, goods_id)) = registration else {
            return false;
        };
        self.record_goods_ai_registration(ticket, goods_id);
        true
    }

    /// Выполняет `CPlayer::DelItemFromGoodsAiTree` для предмета, который
    /// остаётся в принадлежащем игроку контейнере. Это позволяет сценарному
    /// владельцу изменить временные поля между удалением и повторной
    /// регистрацией, не перемещая сам предмет.
    pub(crate) fn unregister_goods_ai_by_id(
        &mut self,
        goods_id: CGuid,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) -> bool {
        let Some(ticket) = self.get_goods_by_id(goods_id).map(CGoods::add_ticket) else {
            return false;
        };
        if ticket == 0 {
            return false;
        }
        let Some(bucket) = self.goods_ai_tree.get_mut(&ticket) else {
            return false;
        };
        let _ = bucket.remove(&goods_id);
        if bucket.is_empty() {
            self.goods_ai_tree.remove(&ticket);
        }

        let Some(goods) = self.get_goods_by_id_mut(goods_id) else {
            return false;
        };
        let start = goods.start_point(factory);
        let elapsed = if start < now_seconds {
            (now_seconds as u32).wrapping_sub(start as u32)
        } else {
            0
        };
        let lifetime = goods.goods_lifetime(factory);
        if elapsed.wrapping_add(5) >= lifetime {
            goods.set_goods_lifetime(0);
            self.goods_ai_delete_queue
                .push_back(BTreeSet::from([goods_id]));
            return true;
        }
        goods.set_goods_lifetime(lifetime.wrapping_sub(elapsed));
        goods.set_start_point(0);
        goods.set_add_ticket(0);
        true
    }

    /// Завершает удаление предмета со склада через тот же реестр `GoodsAI`,
    /// который обслуживает перемещения между контейнерами игрока.
    pub(crate) fn unregister_depot_goods_ai(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
        now_seconds: u64,
    ) {
        self.unregister_goods_ai(goods, factory, now_seconds);
    }

    fn get_goods_ai_by_id_mut(&mut self, goods_id: CGuid) -> Option<&mut CGoods> {
        if self.hand.find(goods_id).is_some() {
            return self.hand.find_mut(goods_id);
        }
        if self.packet.base().find(goods_id).is_some() {
            return self.packet.base_mut().find_mut(goods_id);
        }
        if self.equipment.find(goods_id).is_some() {
            return self.equipment.find_mut(goods_id);
        }
        if self.depot.base().base().find(goods_id).is_some() {
            return self.depot.base_mut().base_mut().find_mut(goods_id);
        }
        if self.fairy_container.base().base().find(goods_id).is_some() {
            return self
                .fairy_container
                .base_mut()
                .base_mut()
                .find_mut(goods_id);
        }
        if self
            .battle_fairy_container
            .base()
            .base()
            .find(goods_id)
            .is_some()
        {
            return self
                .battle_fairy_container
                .base_mut()
                .base_mut()
                .find_mut(goods_id);
        }
        if self.auction_listing.base().find(goods_id).is_some() {
            return self.auction_listing.base_mut().find_mut(goods_id);
        }
        if self.auction_goods.base().find(goods_id).is_some() {
            return self.auction_goods.base_mut().find_mut(goods_id);
        }
        if self.ci_qing.base().find(goods_id).is_some() {
            return self.ci_qing.base_mut().find_mut(goods_id);
        }
        self.ci_qing_compose.base_mut().find_mut(goods_id)
    }

    pub(crate) fn done_goods_ai_tree(&mut self) -> usize {
        let due_tickets: Vec<_> = self
            .goods_ai_tree
            .range(..=self.current_ticket)
            .map(|(&ticket, _)| ticket)
            .collect();
        let mut moved = 0;
        for ticket in due_tickets {
            if let Some(due) = self.goods_ai_tree.remove(&ticket)
                && !due.is_empty()
            {
                moved += due.len();
                self.goods_ai_delete_queue.push_back(due);
            }
        }
        moved
    }

    /// Exact `DoneDelList`: только front bucket и максимум четыре GUID за AI.
    pub(crate) fn take_goods_ai_deletions(&mut self) -> Vec<CGuid> {
        while self
            .goods_ai_delete_queue
            .front()
            .is_some_and(BTreeSet::is_empty)
        {
            self.goods_ai_delete_queue.pop_front();
        }
        let Some(front) = self.goods_ai_delete_queue.front_mut() else {
            return Vec::new();
        };
        let result: Vec<_> = front.iter().copied().take(4).collect();
        for goods_id in &result {
            front.remove(goods_id);
        }
        if front.is_empty() {
            self.goods_ai_delete_queue.pop_front();
        }
        result
    }

    pub(crate) fn goods_ai_location(&self, goods_id: CGuid) -> Option<PlayerGoodsAiLocation> {
        let located = |extend_id, position| PlayerGoodsAiLocation {
            extend_id,
            position,
        };
        self.packet
            .query_goods_position(goods_id)
            .map(|position| located(1, position))
            .or_else(|| {
                self.equipment
                    .query_goods_position_by_id(goods_id)
                    .map(|column| located(2, column.position()))
            })
            .or_else(|| {
                self.hand
                    .query_goods_position(goods_id)
                    .map(|p| located(3, p))
            })
            .or_else(|| {
                self.depot
                    .base()
                    .query_goods_position(goods_id)
                    .map(|p| located(9, p))
            })
            .or_else(|| {
                self.fairy_container
                    .base()
                    .query_goods_position(goods_id)
                    .map(|p| located(11, p))
            })
            .or_else(|| {
                self.battle_fairy_container
                    .base()
                    .query_goods_position(goods_id)
                    .map(|p| located(12, p))
            })
            .or_else(|| {
                self.auction_listing
                    .query_goods_position(goods_id)
                    .map(|p| located(13, p))
            })
            .or_else(|| {
                self.auction_goods
                    .query_goods_position(goods_id)
                    .map(|p| located(14, p))
            })
            .or_else(|| {
                self.ci_qing
                    .query_goods_position(goods_id)
                    .map(|p| located(16, p))
            })
            .or_else(|| {
                self.ci_qing_compose
                    .query_goods_position(goods_id)
                    .map(|p| located(17, p))
            })
    }

    /// Exact `DropParticularGoodsWhenLost` snapshot: packet, equipment, затем
    /// hand; выбирается бит `0x04`, а actual `DropGoods` mutation выполняется
    /// caller-ом после завершения обхода, чтобы container erase не сбивал его.
    pub(crate) fn particular_goods_drops(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<PlayerParticularGoodsDrop> {
        let mut drops = Vec::new();
        let mut push = |extend_id: i32, position: u32, goods: &CGoods| {
            if goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 0x04 != 0 {
                drops.push(PlayerParticularGoodsDrop {
                    location: PlayerGoodsAiLocation {
                        extend_id,
                        position,
                    },
                    goods_id: goods.identity().ex_id,
                    amount: goods.amount(),
                });
            }
        };
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods(position) {
                push(1, position, goods);
            }
        }
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods(position) {
                push(2, position, goods);
            }
        }
        for goods in self.hand.traversing_goods() {
            if let Some(position) = self.hand.query_goods_position(goods.identity().ex_id) {
                push(3, position, goods);
            }
        }
        drops
    }

    /// Точный снимок `DropParticularGoodsWhenRecall`: рюкзак, экипировка,
    /// затем рука; выбирается бит `0x80`. Фактическое перемещение на землю
    /// выполняет вызывающий владелец после обхода, чтобы удаления не меняли
    /// порядок кандидатов.
    pub(crate) fn particular_goods_recall_drops(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<PlayerParticularGoodsDrop> {
        let mut drops = Vec::new();
        let mut push = |extend_id: i32, position: u32, goods: &CGoods| {
            if goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 0x80 != 0 {
                drops.push(PlayerParticularGoodsDrop {
                    location: PlayerGoodsAiLocation { extend_id, position },
                    goods_id: goods.identity().ex_id,
                    amount: goods.amount(),
                });
            }
        };
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods(position) {
                push(1, position, goods);
            }
        }
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods(position) {
                push(2, position, goods);
            }
        }
        for goods in self.hand.traversing_goods() {
            if let Some(position) = self.hand.query_goods_position(goods.identity().ex_id) {
                push(3, position, goods);
            }
        }
        drops
    }

    pub(crate) fn script_equipment_base_index(&self, position: u32) -> Option<u32> {
        self.equipment
            .get_goods(position)
            .map(CGoods::base_properties_index)
    }

    pub(crate) fn upgrade_script_equipment(
        &mut self,
        position: u32,
        level_delta: i32,
        factory: &CGoodsFactory,
        mut random_below: impl FnMut(i32) -> i32,
    ) -> Option<ShapeIdentity> {
        let goods = self.equipment.get_goods_mut(position)?;
        let current_level = goods.addon_property_value(factory, GAP_WEAPON_LEVEL, 1);
        let target_level = (current_level as u32).wrapping_add(level_delta as u32) as i32;
        let _ = factory.upgrade_equipment(goods, target_level, &mut random_below);
        Some(goods.identity())
    }

    /// Stable snapshot контейнеров для `CPlayer::OnDied`: caller может
    /// удалять предметы, не инвалидируя исходный обход. Бит `0x08` задаёт
    /// unconditional death drop, бит `0x02` запрещает table-driven drop.
    pub(crate) fn death_goods_candidates(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<PlayerDeathGoodsCandidate> {
        let mut candidates = Vec::new();
        let mut push = |extend_id: i32, position: u32, goods: &CGoods| {
            let particular = goods.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1);
            candidates.push(PlayerDeathGoodsCandidate {
                location: PlayerGoodsAiLocation {
                    extend_id,
                    position,
                },
                goods_id: goods.identity().ex_id,
                amount: goods.amount(),
                price: goods.price(),
                name: goods.name().to_vec(),
                particular_on_death: particular & 0x08 != 0,
                table_drop_allowed: particular & 0x02 == 0,
            });
        };
        for position in 0..self.packet.size() {
            if let Some(goods) = self.packet.get_goods(position) {
                push(1, position, goods);
            }
        }
        for position in 0..17 {
            if let Some(goods) = self.equipment.get_goods(position) {
                push(2, position, goods);
            }
        }
        for goods in self.hand.traversing_goods() {
            if let Some(position) = self.hand.query_goods_position(goods.identity().ex_id) {
                push(3, position, goods);
            }
        }
        candidates
    }

    pub(crate) fn owned_goods_location(
        &self,
        extend_id: i32,
        goods_id: CGuid,
    ) -> Option<PlayerGoodsAiLocation> {
        let location = |position| PlayerGoodsAiLocation {
            extend_id,
            position,
        };
        match extend_id {
            1 | 2 | 3 | 9 | 11 | 12 | 13 | 14 | 16 | 17 => self
                .goods_ai_location(goods_id)
                .filter(|actual| actual.extend_id == extend_id),
            4 => self
                .wallet
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            5 => self
                .yuan_bao
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            6 => self
                .ji_fen
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            8 => self
                .bank
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            10 => (self.enhancement_selected_goods_id() == Some(goods_id))
                .then(|| self.goods_ai_location(goods_id))
                .flatten(),
            15 => self
                .auction_wallet
                .get_goods(0)
                .filter(|goods| goods.identity().ex_id == goods_id)
                .map(|_| location(0)),
            _ => None,
        }
    }

    /// Полный non-equipment `DeleteGoods(..., 1, true)` storage core. Client
    /// wire и World audit формирует CGame после сохранения удалённого snapshot.
    pub(crate) fn delete_owned_goods(
        &mut self,
        location: PlayerGoodsAiLocation,
        goods_id: CGuid,
        requested_amount: u32,
        factory: &CGoodsFactory,
    ) -> Option<PlayerGoodsAiDeletion> {
        macro_rules! delete_volume {
            ($container:expr) => {{
                let container = $container;
                let goods = container.get_goods(location.position)?.clone();
                (goods.identity().ex_id == goods_id).then_some(())?;
                let previous_amount = goods.amount();
                let removed_amount = requested_amount.min(previous_amount);
                let remaining_amount = previous_amount.wrapping_sub(removed_amount);
                let listeners = if remaining_amount != 0 {
                    container
                        .get_goods_mut(location.position)?
                        .set_amount(remaining_amount);
                    Vec::new()
                } else {
                    let removed = container.remove_goods(goods_id)?;
                    let taken = match removed {
                        VolumeGoodsRemoveOutcome::Removed(taken)
                        | VolumeGoodsRemoveOutcome::RemovedButCellMissing(taken) => taken,
                    };
                    match taken {
                        AmountLimitGoodsTaken::Removed(removed) => removed.listeners,
                        AmountLimitGoodsTaken::Split(_) => unreachable!("full GoodsAI delete"),
                    }
                };
                Some(PlayerGoodsAiDeletion {
                    location,
                    goods,
                    previous_amount,
                    removed_amount,
                    remaining_amount,
                    listeners,
                })
            }};
        }
        match location.extend_id {
            1 => delete_volume!(&mut self.packet),
            2 => {
                let goods = self.equipment.get_goods(location.position)?.clone();
                (goods.identity().ex_id == goods_id).then_some(())?;
                let previous_amount = goods.amount();
                (requested_amount < previous_amount).then_some(())?;
                self.equipment
                    .find_mut(goods_id)?
                    .set_amount(previous_amount.wrapping_sub(requested_amount));
                Some(PlayerGoodsAiDeletion {
                    location,
                    goods,
                    previous_amount,
                    removed_amount: requested_amount,
                    remaining_amount: previous_amount.wrapping_sub(requested_amount),
                    listeners: Vec::new(),
                })
            }
            3 => {
                let goods = self.hand.get_goods(location.position)?.clone();
                (goods.identity().ex_id == goods_id).then_some(())?;
                let previous_amount = goods.amount();
                let removed_amount = requested_amount.min(previous_amount);
                let remaining_amount = previous_amount.wrapping_sub(removed_amount);
                let listeners = if remaining_amount != 0 {
                    self.hand.find_mut(goods_id)?.set_amount(remaining_amount);
                    Vec::new()
                } else {
                    self.hand.remove_goods(goods_id)?.listeners
                };
                Some(PlayerGoodsAiDeletion {
                    location,
                    goods,
                    previous_amount,
                    removed_amount,
                    remaining_amount,
                    listeners,
                })
            }
            4 | 5 | 6 | 8 | 15 => {
                macro_rules! delete_currency {
                    ($container:expr) => {{
                        let container = $container;
                        let goods = container.get_goods(location.position)?.clone();
                        (goods.identity().ex_id == goods_id).then_some(())?;
                        let previous_amount = goods.amount();
                        let removed_amount = requested_amount.min(previous_amount);
                        let remaining_amount = previous_amount.wrapping_sub(removed_amount);
                        let listeners = if removed_amount == 0 {
                            Vec::new()
                        } else {
                            let template = goods.clone();
                            let taken = container.take_goods(
                                location.position,
                                removed_amount,
                                factory,
                                |_| Some(template.clone()),
                            )?;
                            match taken {
                                CurrencyGoodsTaken::Removed(removed) => removed.listeners,
                                CurrencyGoodsTaken::Split(_) => Vec::new(),
                            }
                        };
                        Some(PlayerGoodsAiDeletion {
                            location,
                            goods,
                            previous_amount,
                            removed_amount,
                            remaining_amount,
                            listeners,
                        })
                    }};
                }
                match location.extend_id {
                    4 => {
                        let deletion = delete_currency!(&mut self.wallet);
                        if deletion.is_some() {
                            self.money = self.wallet.currency_amount();
                        }
                        deletion
                    }
                    5 => delete_currency!(&mut self.yuan_bao),
                    6 => delete_currency!(&mut self.ji_fen),
                    8 => delete_currency!(&mut self.bank),
                    15 => delete_currency!(&mut self.auction_wallet),
                    _ => unreachable!(),
                }
            }
            9 => delete_volume!(self.depot.base_mut()),
            11 => delete_volume!(self.fairy_container.base_mut()),
            12 => delete_volume!(self.battle_fairy_container.base_mut()),
            13 => delete_volume!(&mut self.auction_listing),
            14 => delete_volume!(&mut self.auction_goods),
            16 => delete_volume!(&mut self.ci_qing),
            17 => delete_volume!(&mut self.ci_qing_compose),
            _ => None,
        }
    }

    /// Периодический префикс `CPlayer::AI`: нулевой HP надетой боевой феи при
    /// каждом проходе повторно нормализует четыре поля состояния и вызывает
    /// `PropertiesChanged`. Исходник не удаляет устаревшую запись карты области
    /// и не рассылает состояние. Правило перенесено буквально в Zone
    /// `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) fn refresh_battle_fairy_death(
        &mut self,
        factory: &CGoodsFactory,
    ) -> BattleFairyDeathOutcome {
        nebokrai_zone::skills::battlefairysummon::refresh_battle_fairy_death(
            &mut BattleFairyWarSoul {
                summoned: &mut self.battle_fairy_summoned,
                state: &mut self.war_soul_state,
                recall: &mut self.base_properties.battle_fairy_recall,
                died: &mut self.base_properties.battle_fairy_died,
                visual_x_bits: &mut self.war_soul_visual_x_bits,
                visual_y_bits: &mut self.war_soul_visual_y_bits,
                point: &mut self.war_soul_point,
            },
            |property| {
                self.equipment
                    .get_goods(10)
                    .map(|goods| goods.addon_property_value(factory, property, 1))
            },
        )
    }

    /// Exact positional `CBattleFairyContainer::Add`: для gear-ячеек
    /// `BFPropertyAdd(+1)` является ранним partial effect и сохраняется даже
    /// если base storage затем отвергнет товар. Правило перенесено буквально
    /// в Zone `skills/battlefairygear.rs` (порция №7b).
    pub(crate) fn add_battle_fairy_goods(
        &mut self,
        cell: BattleFairyCell,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let occupation = self.base_properties.occupation;
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::add_battle_fairy_goods(
                &mut adapter,
                player_id,
                cell.position(),
                incoming,
                occupation,
                coefficients,
                owner_progress_allows,
            )
        };
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: Some(cell),
            property_applied: resolution.property_applied,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Безпозиционный overload сначала читает catalog BF equip-place. Только
    /// валидная колонка достигает player property-tail; все typed reject-и
    /// остаются у container owner-а без выдуманного размещения. Правило
    /// перенесено буквально в Zone `skills/battlefairygear.rs` (порция №7b).
    pub(crate) fn add_battle_fairy_goods_auto(
        &mut self,
        incoming: &mut Option<CGoods>,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        owner_progress_allows: bool,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let occupation = self.base_properties.occupation;
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::add_battle_fairy_goods_auto(
                &mut adapter,
                player_id,
                incoming,
                occupation,
                coefficients,
                owner_progress_allows,
            )
        };
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: resolution
                .cell_position
                .and_then(BattleFairyCell::from_position),
            property_applied: resolution.property_applied,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Exact `Remove`: base container отделяет goods до `BFPropertyAdd(-1)`;
    /// успешный property path сериализует battle fairy дважды — один раз в
    /// `BFPropertyAdd`, затем ещё раз в override `Remove`. Правило перенесено
    /// буквально в Zone `skills/battlefairygear.rs` (порция №7b).
    pub(crate) fn remove_battle_fairy_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let occupation = self.base_properties.occupation;
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::remove_battle_fairy_goods(
                &mut adapter,
                player_id,
                ex_id,
                occupation,
                coefficients,
            )
        };
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: resolution
                .cell_position
                .and_then(BattleFairyCell::from_position),
            property_applied: resolution.property_applied,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Positional `Remove(position, amount)` использует полный player-tail
    /// для whole goods. Partial stack remove относится к material/gem cells и
    /// не запускает `BFPropertyAdd(-1)`, пока исходный slot остаётся занят.
    /// Правило перенесено буквально в Zone `skills/battlefairygear.rs`
    /// (порция №7b).
    pub(crate) fn take_battle_fairy_goods<Create>(
        &mut self,
        cell: BattleFairyCell,
        amount: u32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        mut create_goods: Create,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport
    where
        Create: FnMut(u32) -> Option<CGoods>,
    {
        let player_id = self.player_id();
        let occupation = self.base_properties.occupation;
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::take_battle_fairy_goods(
                &mut adapter,
                player_id,
                cell.position(),
                amount,
                occupation,
                coefficients,
                &mut create_goods,
            )
        };
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: resolution
                .cell_position
                .and_then(BattleFairyCell::from_position)
                .or(Some(cell)),
            property_applied: resolution.property_applied,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Полный player-side opcode `0x8FC2A`. `allocations` содержат пары
    /// property/client-points прямо из packet-а: legacy outer caller суммирует
    /// unscaled points, но передаёт каждому `AllocatePotential` wrapping
    /// `points * 10000`. `std::map::insert` сохраняет первую запись ключа.
    /// Правило перенесено буквально в Zone `skills/battlefairygear.rs`
    /// (порция №7b).
    pub(crate) fn allocate_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        allocations: &[(i32, i32)],
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialAllocationReport {
        let player_id = self.player_id();
        let occupation = self.base_properties.occupation;
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::allocate_battle_fairy_potential(
                &mut adapter,
                player_id,
                battle_fairy_enabled,
                allocations,
                occupation,
                coefficients,
            )
        };
        BattleFairyPotentialAllocationReport {
            player_id,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Полный player-side путь улучшения экипировки боевой феи с аудитом
    /// `0x60202/0x60203`. Правило перенесено буквально в Zone
    /// `skills/battlefairygear.rs` (порция №7b).
    pub(crate) fn upgrade_battle_fairy_equipment(
        &mut self,
        factory: &CGoodsFactory,
        log_gates: BattleFairyUpgradeLogGates,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyUpgradeReport {
        let player_id = self.player_id();
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::upgrade_battle_fairy_equipment(
                &mut adapter,
                player_id,
                log_gates,
                random,
            )
        };
        BattleFairyUpgradeReport {
            player_id,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Полный player-side сброс потенциала боевой феи предметом `ZHQLS01`.
    /// Правило перенесено буквально в Zone `skills/battlefairygear.rs`
    /// (порция №7b).
    pub(crate) fn reset_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialResetReport {
        let player_id = self.player_id();
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: None,
                encode_old_client,
            };
            battlefairygear::reset_battle_fairy_potential(
                &mut adapter,
                player_id,
                battle_fairy_enabled,
            )
        };
        BattleFairyPotentialResetReport {
            player_id,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Полный player-side `CBattleFairyContainer::ResetSkill`. `consume_item`
    /// соответствует третьему native аргументу: script allocation передаёт
    /// ноль, прямой gameplay caller может потребовать `ZHJNS01/02`. Правило
    /// перенесено буквально в Zone `skills/battlefairygear.rs` (порция №7b).
    pub(crate) fn reset_battle_fairy_skill(
        &mut self,
        battle_fairy_enabled: bool,
        position: i32,
        consume_item: bool,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairySkillResetReport {
        let player_id = self.player_id();
        let resolution = {
            let mut adapter = BattleFairyGearPlayerAdapter {
                player: self,
                factory,
                skill_factory: Some(skill_factory),
                encode_old_client,
            };
            battlefairygear::reset_battle_fairy_skill(
                &mut adapter,
                player_id,
                battle_fairy_enabled,
                position,
                consume_item,
                random,
            )
        };
        BattleFairySkillResetReport {
            player_id,
            position,
            outcome: resolution.outcome,
            effects: resolution.effects.into_iter().collect(),
        }
    }

    /// Player-owned `DelWarSoulSkillInPlayer` перед запуском reset-script.
    /// Native `TellClient(false)` уже после `DelSkill` не находит удалённый
    /// skill, поэтому наблюдаемым результатом остаётся ordered detach state.
    pub(crate) fn detach_battle_fairy_script_skills(
        &mut self,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> Vec<u32> {
        let mut detached = Vec::new();
        if self.equipment.get_goods(10).is_none() {
            return detached;
        }
        for property in EQUIPPED_SKILL_PROPERTIES {
            let skill_id = {
                let goods = self.equipment.get_goods(10).expect("снятие навыка сохраняет предмет");
                war_soul_skill_id_from_goods(goods, factory, property)
            };
            if skill_id == 0 {
                continue;
            }
            let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
            detached.push(skill_id);
        }
        detached
    }

    /// Player-owned `AddWarSoulSkillToPalyer` после reset-script: addon state
    /// перечитывается из того же equipped headgear, затем каждый достигнутый
    /// skill публикуется через обычный `TellClient(true)` snapshot.
    pub(crate) fn attach_battle_fairy_script_skills(
        &mut self,
        factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> Vec<BattleFairySkillAdded> {
        let player_id = self.player_id();
        let mut attached = Vec::new();
        if self.equipment.get_goods(10).is_none() {
            return attached;
        }
        for property in EQUIPPED_SKILL_PROPERTIES {
            let (skill_id, level) = {
                let goods = self.equipment.get_goods(10).expect("установка навыка сохраняет предмет");
                war_soul_skill_entry_from_goods(goods, factory, property)
            };
            if skill_id == 0 {
                continue;
            }
            let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
            if let Some(skill) = self.move_shape.skill(skill_id, skill_factory) {
                attached.push(battle_fairy_skill_snapshot(player_id, skill, skill_factory));
            }
        }
        attached
    }

    /// Полный player-side `skillmessage 0x90001` после успешного decoder-а.
    /// Contend notification не блокирует запрос; `ClearEmotion` всегда
    /// предшествует authorization и AI dispatch.
    pub(crate) fn request_player_skill(
        &mut self,
        socket_id: i32,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        self.request_player_skill_core(socket_id, request, facts, None, "GS0090", skill_factory)
    }

    /// Item-skill `0x90004` использует переданный client level, а успешная
    /// ветвь добавляет ID в native ordered item-skill vector перед AI effect.
    pub(crate) fn request_item_skill(
        &mut self,
        socket_id: i32,
        request: PlayerSkillRequest,
        skill_level: i32,
        facts: PlayerSkillRequestFacts,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        self.request_player_skill_core(
            socket_id,
            request,
            facts,
            Some(skill_level),
            "GS1039",
            skill_factory,
        )
    }

    fn request_player_skill_core(
        &mut self,
        socket_id: i32,
        request: PlayerSkillRequest,
        facts: PlayerSkillRequestFacts,
        item_skill_level: Option<i32>,
        contend_string_id: &'static str,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut journal = GameEffectJournal::default();
        let mut target_type = request.target_type;
        let mut target_id = request.target_id;
        let mut target_x = request.target_x;
        let mut target_y = request.target_y;
        if self.contend_state && facts.symbol_attackable {
            journal.push(GameEffect::SkillNotification {
                player_id,
                string_id: contend_string_id,
                color: 0xffff_ffff,
                message_type: 0xffff_0000,
            });
        }

        self.emotion_index = 0;
        self.emotion_timestamp_ms = 0;
        journal.push(GameEffect::ClearPlayerEmotion {
            player_id,
            region_id: self.server_region_id,
        });

        let skill_level = item_skill_level.unwrap_or_else(|| {
            self.move_shape
                .skill(skill_id, skill_factory)
                .map_or(0, MoveShapeSkill::level)
        });
        if skill_level == 0 {
            push_player_skill_reject(&mut journal, socket_id);
            trace!(
                player_id,
                skill_id, "Запрос навыка отклонён: навык не разрешён"
            );
            return journal;
        }
        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (resolved_x, resolved_y) =
                match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
                    (Ok(x), Ok(y)) => (x, y),
                    (Err(error), _) | (_, Err(error)) => {
                        trace!(
                            player_id,
                            skill_id,
                            ?error,
                            "Запрос навыка отклонён координатной границей"
                        );
                        return journal;
                    }
                };
            target_type = self.shape().identity().object_type;
            target_id = player_id;
            target_x = resolved_x;
            target_y = resolved_y;
        }
        if !facts.player_ai_available {
            trace!(
                player_id,
                skill_id, "Запрос навыка отклонён: отсутствует AI игрока"
            );
            return journal;
        }

        let dispatch = match (SkillTarget {
            object_type: target_type,
            object_id: target_id,
            x: target_x,
            y: target_y,
        })
        .form()
        {
            SkillTargetForm::SelfTarget => PlayerSkillDispatch::SelfTarget {
                skill_id,
                player_id,
            },
            SkillTargetForm::Point { x, y } => PlayerSkillDispatch::Point { skill_id, x, y },
            SkillTargetForm::Object { target } => {
                if self.server_region_id.is_none() {
                    trace!(
                        player_id,
                        skill_id, "Запрос навыка отклонён: отсутствует регион"
                    );
                    return journal;
                }
                if !facts.object_target_available {
                    push_player_skill_reject(&mut journal, socket_id);
                    trace!(
                        player_id,
                        skill_id, target_type, target_id, "Запрос навыка отклонён: цель отсутствует"
                    );
                    return journal;
                }
                PlayerSkillDispatch::Object { skill_id, target }
            }
        };
        if item_skill_level.is_some() {
            self.move_shape.set_item_skill(skill_id);
        }
        journal.push(GameEffect::QueuePlayerSkill {
            player_id,
            dispatch,
        });
        trace!(
            player_id,
            skill_id,
            skill_level,
            ?dispatch,
            "Запрос навыка передан AI"
        );
        journal
    }

    /// Полный player-side `skillmessage` opcode `0x90005` после успешного
    /// packet decode. Contend notification намеренно не блокирует запрос.
    pub(crate) fn request_battle_fairy_skill(
        &self,
        battle_fairy_enabled: bool,
        socket_id: i32,
        request: BattleFairySkillRequest,
        facts: BattleFairySkillRequestFacts,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> GameEffectJournal {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut journal = GameEffectJournal::default();
        let mut target_type = request.target_type;
        let mut target_id = request.target_id;
        let mut target_x = request.target_x;
        let mut target_y = request.target_y;
        if !battle_fairy_enabled {
            journal.push(GameEffect::SkillNotification {
                player_id,
                string_id: "ZHGS0037",
                color: 0xffff_0000,
                message_type: 0,
            });
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: подсистема выключена"
            );
            return journal;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: отсутствует головной предмет"
            );
            return journal;
        };
        if goods.addon_property_value(goods_factory, GAP_BF_HP, 1) == 0 {
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: нет здоровья"
            );
            return journal;
        }
        if self.contend_state && facts.symbol_attackable {
            journal.push(GameEffect::SkillNotification {
                player_id,
                string_id: "ZHGS0038",
                color: 0xffff_ffff,
                message_type: 0xffff_0000,
            });
        }

        let skill_level =
            battle_fairy_skill_level(request.property_offset, skill_id, |property, index| {
                battle_fairy_skill_property_value(goods, goods_factory, property, index)
            });
        if skill_level == 0 {
            push_battle_fairy_skill_reject(&mut journal, socket_id);
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: навык не разрешён"
            );
            return journal;
        }

        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (resolved_x, resolved_y) =
                match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
                    (Ok(x), Ok(y)) => (x, y),
                    (Err(error), _) | (_, Err(error)) => {
                        trace!(
                            player_id,
                            skill_id,
                            ?error,
                            "Запрос навыка боевой феи отклонён координатной границей"
                        );
                        return journal;
                    }
                };
            target_type = self.shape().identity().object_type;
            target_id = player_id;
            target_x = resolved_x;
            target_y = resolved_y;
        }
        if !facts.player_ai_available {
            trace!(
                player_id,
                skill_id, "Запрос навыка боевой феи отклонён: отсутствует AI игрока"
            );
            return journal;
        }

        let dispatch = match (SkillTarget {
            object_type: target_type,
            object_id: target_id,
            x: target_x,
            y: target_y,
        })
        .form()
        {
            SkillTargetForm::SelfTarget => BattleFairySkillDispatch::SelfTarget {
                skill_id,
                skill_level,
                player_id,
            },
            SkillTargetForm::Point { x, y } => BattleFairySkillDispatch::Point {
                skill_id,
                skill_level,
                x,
                y,
            },
            SkillTargetForm::Object { target } => {
                if self.server_region_id.is_none() {
                    trace!(
                        player_id,
                        skill_id, "Запрос навыка боевой феи отклонён: отсутствует регион"
                    );
                    return journal;
                }
                if !facts.object_target_available {
                    push_battle_fairy_skill_reject(&mut journal, socket_id);
                    trace!(
                        player_id,
                        skill_id,
                        target_type,
                        target_id,
                        "Запрос навыка боевой феи отклонён: цель отсутствует"
                    );
                    return journal;
                }
                BattleFairySkillDispatch::Object {
                    skill_id,
                    skill_level,
                    target,
                }
            }
        };
        journal.push(GameEffect::QueueBattleFairySkill {
            player_id,
            dispatch,
        });
        trace!(
            player_id,
            skill_id,
            skill_level,
            ?dispatch,
            "Запрос навыка боевой феи передан AI"
        );
        journal
    }

    pub(crate) const fn set_pk_count(&mut self, value: u16) {
        self.base_properties.pk_count = value;
    }

    /// `OnUpdateMurdererSign` сбрасывает часы при нулевом PK и запускает их
    /// только при первом переходе к ненулевому значению.
    pub(crate) fn update_murderer_sign(&mut self, now_ms: impl FnOnce() -> u32) {
        if self.base_properties.pk_count == 0 {
            self.murderer_time_stamp_ms = 0;
        } else if self.murderer_time_stamp_ms == 0 {
            self.murderer_time_stamp_ms = now_ms();
        }
    }

    pub(crate) const fn set_occupation(&mut self, occupation: u8) {
        self.base_properties.occupation = occupation;
    }

    pub(crate) const fn set_ci_qing_open(&mut self, value: bool) {
        self.ci_qing_open = value;
    }

    pub(crate) const fn set_experience(&mut self, value: u32) {
        self.base_properties.experience = value;
    }

    pub(crate) const fn experience(&self) -> u32 {
        self.base_properties.experience
    }

    /// Exact `CPlayer::IncreaseContinuousKill`: первый hit после истёкшего
    /// окна сбрасывает счётчик в ноль, а milestone меняет persisted
    /// `wHitTopLog` до начисления его bonus experience и client publications.
    pub(crate) fn increase_continuous_kill(
        &mut self,
        now_ms: u32,
        hit_time_ms: u32,
        hit_levels: &[HitLevelEntry],
    ) -> PlayerContinuousKillUpdate {
        let mut new_top_log = None;
        let mut bonus_experience = 0;
        if now_ms.wrapping_sub(self.continuous_kill_timestamp_ms) < hit_time_ms {
            self.continuous_kill_amount = self.continuous_kill_amount.wrapping_add(1);
            if u32::from(self.base_properties.hit_top_log) < self.continuous_kill_amount {
                if let Some(level) = hit_levels
                    .iter()
                    .find(|level| level.hit == self.continuous_kill_amount)
                {
                    self.base_properties.hit_top_log = level.hit as u16;
                    new_top_log = Some(self.base_properties.hit_top_log);
                    bonus_experience = level.experience;
                }
            }
        } else {
            self.continuous_kill_amount = 0;
        }
        self.continuous_kill_timestamp_ms = now_ms;
        PlayerContinuousKillUpdate {
            amount: self.continuous_kill_amount,
            new_top_log,
            bonus_experience,
        }
    }

    pub(crate) const fn continuous_kill_amount(&self) -> u32 {
        self.continuous_kill_amount
    }

    pub(crate) const fn vigour(&self) -> u32 {
        self.base_properties.vigour
    }

    /// Exact `CPlayer::SetVigour`: вход сначала записывается в base property,
    /// затем ограничивается текущим `dwMaxVigour` того же owner-а.
    pub(crate) const fn set_vigour(&mut self, value: u32) {
        self.base_properties.vigour = if self.base_properties.maximum_vigour < value {
            self.base_properties.maximum_vigour
        } else {
            value
        };
    }

    pub(crate) const fn set_script_vigour(&mut self, value: i32) -> i32 {
        self.base_properties.vigour = value as u32;
        value
    }

    pub(crate) const fn set_script_experience(&mut self, value: i32) -> i32 {
        self.base_properties.experience = value as u32;
        value
    }

    /// Прямое чтение `m_BaseProperty.bFairyContainerEnabled`; правило
    /// перенесено буквально в Zone `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) const fn fairy_container_enabled(&self) -> bool {
        nebokrai_zone::skills::battlefairysummon::fairy_container_enabled(
            self.base_properties.fairy_container_enabled,
        )
    }

    /// Граница восстановления `m_BaseProperty.bFairyContainerEnabled` из
    /// persisted player snapshot; default остаётся выключенным до decode.
    /// Правило перенесено буквально в Zone `skills/battlefairysummon.rs`
    /// (порция №7a).
    pub(crate) const fn set_fairy_container_enabled(&mut self, value: bool) {
        nebokrai_zone::skills::battlefairysummon::set_fairy_container_enabled(
            &mut self.base_properties.fairy_container_enabled,
            value,
        )
    }

    pub(crate) const fn restore_appearance_and_mode(
        &mut self,
        head_picture: i32,
        face_picture: i32,
        mode: u32,
    ) {
        self.base_properties.head_picture = head_picture;
        self.base_properties.face_picture = face_picture;
        self.base_properties.mode = mode;
    }

    pub(crate) const fn appearance_and_mode(&self) -> (i32, i32, u32) {
        (
            self.base_properties.head_picture,
            self.base_properties.face_picture,
            self.base_properties.mode,
        )
    }

    pub(crate) const fn health(&self) -> u32 {
        self.base_properties.health
    }

    pub(crate) const fn maximum_health(&self) -> u32 {
        self.combat_properties.maximum_hp
    }

    pub(crate) fn roll_stiffen(
        &mut self,
        damage: u32,
        setup: crate::setup::globesetup::GlobeStiffenSetup,
        now_ms: impl FnMut() -> u32,
        random: impl FnMut(i32) -> i32,
    ) -> u32 {
        let maximum_hp = self.combat_properties.maximum_hp;
        let reank = self.combat_properties.reank;
        self.move_shape
            .stiffen(damage as u16, maximum_hp, reank, setup, now_ms, random)
    }

    pub(crate) const fn maximum_mana(&self) -> u32 {
        self.combat_properties.maximum_mp
    }

    /// Собственный скалярный хвост `OnRelive` после внешних вызовов
    /// пассивных навыков, входа в регион и пересчёта свойств.
    pub(crate) fn apply_relive_scalars(
        &mut self,
    ) -> Result<PlayerReliveMutation, ShapeCoordinateBlock> {
        let previous_x = self.shape().get_tile_x()?;
        let previous_y = self.shape().get_tile_y()?;
        let direction = self.shape().get_direction();
        self.set_health(self.maximum_health());
        self.set_mana(self.maximum_mana());
        self.move_shape.shape_mut().set_action(0);
        self.move_shape.shape_mut().set_position(0);
        Ok(PlayerReliveMutation {
            player_id: self.player_id(),
            previous_x,
            previous_y,
            direction,
            health: self.health(),
            mana: self.mana(),
        })
    }

    /// Собственные неполиморфные изменения достигнутого `OnRelive`: временные
    /// снимки спутников очищаются до `OnEnterRegion/UpdateProperty`, а один
    /// уровень блокировки движения снимается после них.
    pub(crate) fn clear_relive_uncreated_companions(&mut self) -> (usize, bool) {
        let cleared_uncreated_pets = self.uncreated_pets.len();
        self.uncreated_pets.clear();
        let cleared_uncreated_carriage = !self.uncreated_carriage.original_name.is_empty()
            || self.uncreated_carriage.health != 0;
        self.uncreated_carriage.original_name.clear();
        self.uncreated_carriage.health = 0;
        (cleared_uncreated_pets, cleared_uncreated_carriage)
    }

    pub(crate) fn unlock_movement_after_relive(
        &mut self,
        cleared_uncreated_pets: usize,
        cleared_uncreated_carriage: bool,
    ) -> PlayerReliveOwnedPrelude {
        let previous_moveable_count = self.move_shape.moveable_count();
        self.move_shape.set_moveable(true);
        PlayerReliveOwnedPrelude {
            cleared_uncreated_pets,
            cleared_uncreated_carriage,
            previous_moveable_count,
            resulting_moveable_count: self.move_shape.moveable_count(),
            moveable: self.move_shape.is_moveable(),
        }
    }

    /// Точное скалярное изменение `EnterResidentState`; рассылкой соседям
    /// владеет `CGame`, где доступны действующие регион и сессия.
    pub(crate) fn enter_resident_state(&mut self) -> u32 {
        let previous = self.criminal_state_timestamp_ms;
        self.criminal_state_timestamp_ms = 0;
        previous
    }

    pub(crate) const fn criminal_state_active(&self) -> bool {
        self.criminal_state_timestamp_ms != 0
    }

    /// Exact criminal tail `UpdateCurrentState`: clock уже sampled caller-ом
    /// только при active timestamp; timeout использует wrapping DWORD sum,
    /// threshold сравнивает promoted `wPkCount` строго через `<`.
    pub(crate) fn criminal_state_end_due(
        &self,
        checked_at_ms: u32,
        criminal_time_ms: u32,
        pk_count_per_kill: u32,
    ) -> Option<PlayerCriminalStateEnd> {
        let previous_timestamp_ms = self.criminal_state_timestamp_ms;
        if previous_timestamp_ms == 0 {
            return None;
        }
        let reason = if previous_timestamp_ms.wrapping_add(criminal_time_ms) <= checked_at_ms {
            PlayerCriminalStateEndReason::Timeout
        } else if pk_count_per_kill < u32::from(self.base_properties.pk_count) {
            PlayerCriminalStateEndReason::PkThresholdExceeded
        } else {
            return None;
        };
        Some(PlayerCriminalStateEnd {
            player_id: self.player_id(),
            previous_timestamp_ms,
            checked_at_ms,
            pk_count: self.base_properties.pk_count,
            reason,
        })
    }

    pub(crate) const fn mana(&self) -> u32 {
        self.base_properties.mana
    }

    /// Добавляет четыре состояния из актуальных свойств после общего
    /// End-обхода Particular и AutomaticRestore у вызывающего владельца.
    pub(crate) fn append_automatic_hp_mp_states(&mut self) {
        self.move_shape
            .append_automatic_hp_mp_states(self.combat_properties);
    }

    pub(crate) fn particular_state_goods_present(
        &self,
        additional_data: u32,
        factory: &CGoodsFactory,
    ) -> bool {
        let mut listener = GoodsParticularPropertyListener::new(GAP_EXCEPTION_STATE);
        for goods in self.packet.base().traversing_goods() {
            listener.visit(factory, goods);
        }
        if listener.goods_ids().iter().any(|goods_id| {
            self.packet.base().find(*goods_id).is_some_and(|goods| {
                goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32
                    == additional_data
            })
        }) {
            return true;
        }
        for (_, goods) in self.equipment.traversing_goods() {
            listener.visit(factory, goods);
        }
        listener.goods_ids().iter().any(|goods_id| {
            self.equipment.find(*goods_id).is_some_and(|goods| {
                goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32
                    == additional_data
            })
        })
    }

    /// OnEnterRegion отбрасывает packet-значения, но дополняет тот же GUID-listener
    /// экипировкой. Уникальность относится к значениям, не к живым состояниям.
    pub(crate) fn equipment_particular_state_values(
        &self,
        factory: &CGoodsFactory,
    ) -> Vec<u32> {
        let mut listener = GoodsParticularPropertyListener::new(GAP_EXCEPTION_STATE);
        for goods in self.packet.base().traversing_goods() {
            listener.visit(factory, goods);
        }
        let mut values = Vec::new();
        for goods_id in listener.goods_ids() {
            if let Some(goods) = self.packet.base().find(*goods_id) {
                let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
                if additional != 0 && !values.contains(&additional) {
                    values.push(additional);
                }
            }
        }
        values.clear();
        for (_, goods) in self.equipment.traversing_goods() {
            listener.visit(factory, goods);
        }
        for goods_id in listener.goods_ids() {
            if let Some(goods) = self.equipment.find(*goods_id) {
                let additional = goods.addon_property_value(factory, GAP_EXCEPTION_STATE, 1) as u32;
                if additional != 0 && !values.contains(&additional) {
                    values.push(additional);
                }
            }
        }
        values
    }

    pub(crate) fn automatic_restore_needs_clock(&self, key: crate::gameserver::appserver::moveshape::StateKey) -> bool {
        self.move_shape
            .automatic_restore_state(key)
            .is_some_and(|state| {
                state.should_check(
                    self.is_dead(),
                    self.shape().get_state(),
                    self.health(),
                    self.maximum_health(),
                    self.mana(),
                    self.maximum_mana(),
                )
            })
    }

    pub(crate) fn automatic_restore_due(&self, key: crate::gameserver::appserver::moveshape::StateKey, checked_at_ms: u32) -> bool {
        self.move_shape
            .automatic_restore_state(key)
            .is_some_and(|state| state.due(checked_at_ms))
    }

    /// Фиксирует второе чтение часов даже при нулевом объёме. `true` означает,
    /// что исходный виртуальный `OnChangeStates` обязан быть вызван немедленно.
    pub(crate) fn apply_automatic_restore(
        &mut self,
        key: crate::gameserver::appserver::moveshape::StateKey,
        recorded_at_ms: u32,
    ) -> bool {
        let properties = self.combat_properties;
        let health = self.health();
        let maximum_health = self.maximum_health();
        let mana = self.mana();
        let maximum_mana = self.maximum_mana();
        let mutation = self
            .move_shape
            .automatic_restore_state_mut(key)
            .and_then(|state| {
                state.apply(
                    recorded_at_ms,
                    properties.into(),
                    health,
                    maximum_health,
                    mana,
                    maximum_mana,
                )
            });
        match mutation {
            Some(AutomaticRestoreMutation::Health(value)) => self.set_health(value),
            Some(AutomaticRestoreMutation::Mana(value)) => self.set_mana(value),
            None => return false,
        }
        true
    }

    pub(crate) const fn rp(&self) -> u16 {
        self.base_properties.rp
    }

    pub(crate) const fn maximum_rp(&self) -> u16 {
        self.base_properties.maximum_rp
    }

    pub(crate) const fn yp(&self) -> u16 {
        self.base_properties.yp
    }

    pub(crate) const fn set_maximum_hp(&mut self, value: u32) {
        self.combat_properties.maximum_hp = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_mp(&mut self, value: u32) {
        self.combat_properties.maximum_mp = clamp_combat_scalar(value);
    }

    /// Exact `SetHP` сначала записывает вход, затем перечитывает виртуальный
    /// `GetMaxHP`; в typed owner-е это текущее combat поле.
    pub(crate) const fn set_health(&mut self, value: u32) {
        self.base_properties.health = if self.combat_properties.maximum_hp < value {
            self.combat_properties.maximum_hp
        } else {
            value
        };
    }

    pub(crate) const fn set_mana(&mut self, value: u32) {
        self.base_properties.mana = if self.combat_properties.maximum_mp < value {
            self.combat_properties.maximum_mp
        } else {
            value
        };
    }

    pub(crate) const fn set_rp(&mut self, value: u16) {
        self.base_properties.rp = if self.base_properties.maximum_rp < value {
            self.base_properties.maximum_rp
        } else {
            value
        };
    }

    /// Exact `CPlayer::IncreaseRp`: только профессия 0 и достигший первого
    /// setup-порога игрок получают RP. За атаку прибавляется фиксированное
    /// значение, а защитная ветвь идёт по шести порогам доли снятого HP с
    /// конца массива. `true` означает исходный вызов `PropertiesChanged` даже
    /// при нулевой прибавке или уже достигнутом пределе.
    pub(crate) fn increase_rp(
        &mut self,
        attacking: bool,
        damage: u16,
        globe_setup: &GlobeSetupSnapshot,
    ) -> bool {
        if self.occupation() != 0 {
            return false;
        }
        let Some(policy) = globe_setup.player_rp_gain_policy(self.level()) else {
            return false;
        };
        let gain = if attacking {
            policy.attack_gain
        } else {
            // Native сначала материализует ushort damage как f32, а деление
            // выполняет в x87 перед FSTP dword. f64 сохраняет точный u32
            // знаменатель до финального округления к тому же f32-result.
            let ratio = (damage as f64 / self.maximum_health() as f64) as f32;
            let mut gain = 0;
            for index in (0..policy.damage_factors.len()).rev() {
                if !(ratio <= policy.damage_factors[index]) {
                    break;
                }
                gain = policy.damage_gains[index];
            }
            gain
        };
        let value = u32::from(self.rp())
            .wrapping_add(u32::from(gain))
            .min(u32::from(policy.maximum)) as u16;
        self.set_rp(value);
        true
    }

    /// Player-owned scalar части `OnExit` return-point tail. Восстановление
    /// смерти выполняется до virtual `GetReturnPoint`, а destination location
    /// записывается только после успешного выбора точки.
    pub(crate) fn prepare_exit_return(&mut self, died: bool) {
        if died {
            self.set_health(self.maximum_health());
            self.move_shape.shape_mut().set_action(0);
            self.move_shape.shape_mut().set_position(0);
        }
    }

    pub(crate) fn apply_exit_return_location(
        &mut self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
    ) {
        let shape = self.move_shape.shape_mut();
        shape.set_region_id(region_id);
        shape.set_direction(direction);
        shape.set_pos_xy_base(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
    }

    pub(crate) const fn set_strength(&mut self, value: u32) {
        self.combat_properties.strength = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_dexterity(&mut self, value: u32) {
        self.combat_properties.dexterity = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_constitution(&mut self, value: u32) {
        self.combat_properties.constitution = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_intelligence(&mut self, value: u32) {
        self.combat_properties.intelligence = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_minimum_attack(&mut self, value: u32) {
        self.combat_properties.minimum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_maximum_attack(&mut self, value: u32) {
        self.combat_properties.maximum_attack = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_defense(&mut self, value: u32) {
        self.combat_properties.defense = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_element_resistance(&mut self, value: u32) {
        self.combat_properties.element_resistance = clamp_combat_scalar(value);
    }

    pub(crate) const fn set_blast_defense_scale(&mut self, value: f32) {
        self.combat_properties.blast_defense_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_full_miss_scale(&mut self, value: f32) {
        self.combat_properties.full_miss_scale_bits =
            (if value < 0.01 { 0.01 } else { value }).to_bits();
    }

    pub(crate) const fn set_critical_rate(&mut self, value: f32) {
        self.combat_properties.critical_rate_bits =
            (if value < 1.0 { 1.0 } else { value }).to_bits();
    }

    pub(crate) const fn set_contribution(&mut self, value: i32) {
        self.contribution = if value < CONTRIBUTION_MINIMUM {
            CONTRIBUTION_MINIMUM
        } else if value > CONTRIBUTION_MAXIMUM {
            CONTRIBUTION_MAXIMUM
        } else {
            value
        };
    }

    /// `lMaxFetchPower` в exact сравнивался после unsigned cast, поэтому
    /// отрицательный setup limit становится большим unsigned пределом.
    pub(crate) const fn set_fetch_power(&mut self, value: u32, setup_maximum: i32) {
        let maximum = setup_maximum as u32;
        self.base_properties.fetch_power = if maximum < value { maximum } else { value };
    }

    pub(crate) const fn fetch_power(&self) -> u32 {
        self.base_properties.fetch_power
    }

    /// Player-owned mutation `ReviveBattleFairy`; client goods/state wire
    /// остаётся у вызывающего `CGame`, уже после изменения всех полей.
    /// Правило перенесено буквально в Zone `skills/battlefairysummon.rs`
    /// (порция №7a).
    pub(crate) fn revive_battle_fairy(&mut self, factory: &CGoodsFactory) -> bool {
        nebokrai_zone::skills::battlefairysummon::revive_battle_fairy(
            &mut BattleFairyWarSoul {
                summoned: &mut self.battle_fairy_summoned,
                state: &mut self.war_soul_state,
                recall: &mut self.base_properties.battle_fairy_recall,
                died: &mut self.base_properties.battle_fairy_died,
                visual_x_bits: &mut self.war_soul_visual_x_bits,
                visual_y_bits: &mut self.war_soul_visual_y_bits,
                point: &mut self.war_soul_point,
            },
            |operation| match operation {
                BattleFairyHeadgearOperation::Read(property) => self
                    .equipment
                    .get_goods(10)
                    .map(|goods| goods.addon_property_value(factory, property, 1)),
                BattleFairyHeadgearOperation::Write(property, value) => {
                    if let Some(goods) = self.equipment.get_goods_mut(10) {
                        let _ = goods.set_addon_property_value_core(property, 1, value);
                    }
                    None
                }
            },
        )
    }

    pub(crate) const fn set_battle_fairy_recall(&mut self, value: bool) {
        nebokrai_zone::skills::battlefairysummon::set_battle_fairy_recall(
            &mut self.base_properties.battle_fairy_recall,
            value,
        )
    }

    pub(crate) const fn set_battle_fairy_died(&mut self, value: bool) {
        nebokrai_zone::skills::battlefairysummon::set_battle_fairy_died(
            &mut self.base_properties.battle_fairy_died,
            value,
        )
    }

    /// Scalar tail `ApplyDeathFinalWarSoulReset`. В отличие от гибели самой
    /// боевой феи смерть хозяина снимает summon/state, разрешает recall и
    /// очищает `bBFDied`. Правило перенесено буквально в Zone
    /// `skills/battlefairysummon.rs` (порция №7a).
    pub(crate) const fn reset_war_soul_after_player_death(&mut self) {
        nebokrai_zone::skills::battlefairysummon::reset_war_soul_after_player_death(
            &mut self.battle_fairy_summoned,
            &mut self.war_soul_state,
            &mut self.base_properties.battle_fairy_recall,
            &mut self.base_properties.battle_fairy_died,
        )
    }

    /// Exact `SetSilence`: начало хранится в минутах `timeGetTime`, а
    /// не абсолютным deadline в миллисекундах.
    pub(crate) const fn set_silence(&mut self, minutes: i32, now_milliseconds: u32) {
        if minutes > 0 {
            self.silence_minutes = minutes;
            self.silence_timestamp_minutes = now_milliseconds / 60_000;
        } else {
            self.silence_minutes = 0;
            self.silence_timestamp_minutes = 0;
        }
    }

    /// Exact `IsInSilence`: равенство deadline ещё считается silence; после
    /// первой просроченной проверки оба legacy поля обнуляются.
    pub(crate) const fn is_in_silence(&mut self, now_milliseconds: u32) -> bool {
        if self.silence_minutes == 0 {
            return false;
        }
        let deadline =
            (self.silence_timestamp_minutes as i32).wrapping_add(self.silence_minutes) as u32;
        if now_milliseconds / 60_000 <= deadline {
            return true;
        }
        self.silence_minutes = 0;
        self.silence_timestamp_minutes = 0;
        false
    }

    /// Точная мутация `CPlayer::OnExit`: legacy owner отдельно считывает
    /// `timeGetTime` перед deadline, перед уменьшением остатка и перед новой
    /// отметкой. Поэтому caller передаёт часы как callback, а не один snapshot.
    pub(crate) fn update_silence_on_exit(
        &mut self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> PlayerExitSilenceUpdate {
        let previous_minutes = self.silence_minutes;
        let previous_timestamp_minutes = self.silence_timestamp_minutes;
        let mut sampled_minutes = [None; 3];
        if self.silence_minutes != 0 {
            let first = now_milliseconds() / 60_000;
            sampled_minutes[0] = Some(first);
            let deadline =
                (self.silence_timestamp_minutes as i32).wrapping_add(self.silence_minutes) as u32;
            if deadline < first {
                self.silence_timestamp_minutes = 0;
                self.silence_minutes = 0;
            } else {
                let second = now_milliseconds() / 60_000;
                sampled_minutes[1] = Some(second);
                self.silence_minutes = self.silence_minutes.wrapping_add(
                    (self.silence_timestamp_minutes as i32).wrapping_sub(second as i32),
                );
                let third = now_milliseconds() / 60_000;
                sampled_minutes[2] = Some(third);
                self.silence_timestamp_minutes = third;
            }
        }
        PlayerExitSilenceUpdate {
            previous_minutes,
            previous_timestamp_minutes,
            sampled_minutes,
            remaining_minutes: self.silence_minutes,
            timestamp_minutes: self.silence_timestamp_minutes,
        }
    }

    /// Player caller `CheckBattleFairyCombine` всегда передаёт собственный ID
    /// в исходный owner; global compose configuration остаётся явным входом.
    pub(crate) fn check_battle_fairy_combine(
        &self,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
    ) -> BattleFairyCombineCheck {
        self.battle_fairy_container.check_battle_fairy_combine(
            Some(self.player_id()),
            factory,
            compose,
        )
    }

    /// Полный player-side `BatllteFairyCombine`: gate, validation, exact
    /// random/deplete/remove order, creation, skill-state и адресные effects.
    /// Battle cell не является gear slot, поэтому этот caller намеренно не
    /// запускает `BFPropertyAdd` и общий player property recalc. Transport
    /// получает уже ordered report, не подменяя неизвестные поля исторических
    /// packet-encoder-ов выдуманными нулями.
    pub(crate) fn combine_battle_fairy<Create>(
        &mut self,
        battle_fairy_enabled: bool,
        setup_maximum_fetch_power: i32,
        factory: &CGoodsFactory,
        compose: &[BattleFairyCompose],
        skill_factory: &CSkillFactory,
        random: &mut dyn FnMut(i32) -> i32,
        create_goods: &mut Create,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyCombineReport
    where
        Create: FnMut(u32, &mut dyn FnMut(i32) -> i32) -> Option<CGoods>,
    {
        let player_id = self.player_id();
        let mut report = BattleFairyCombineReport {
            player_id,
            outcome: BattleFairyCombineOutcome::Rejected,
            effects: GameEffectJournal::default(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairyCombineOutcome::FeatureDisabled;
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0008",
                color: 0xffff_0000,
            });
            return report;
        }

        let recipe = match self
            .battle_fairy_container
            .battle_fairy_combine_recipe(factory, compose)
        {
            Ok(Some(recipe)) => recipe,
            // Execution no-match машинно завершает обработчик молча (tail
            // 0x503F81); notification ZHGS0060 здесь не отправляется.
            Ok(None) => return report,
            Err(notification) => {
                report.effects.push(BattleFairyCombineEffect::Notification {
                    player_id,
                    string_id: notification.string_id(),
                    color: 0xffff_ffff,
                });
                return report;
            }
        };
        let fetch_power = self.base_properties.fetch_power;
        if fetch_power < recipe.deplete_fetch {
            report.outcome = BattleFairyCombineOutcome::InsufficientFetchPower;
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0061",
                color: 0xffff_ffff,
            });
            return report;
        }

        let success = (random(100) as f32) < recipe.success_rate;
        if !success {
            report.effects.push(BattleFairyCombineEffect::Notification {
                player_id,
                string_id: "ZHGS0006",
                color: 0xffff_ffff,
            });
        }
        self.set_fetch_power(
            fetch_power.wrapping_sub(recipe.deplete_fetch),
            setup_maximum_fetch_power,
        );
        report
            .effects
            .push(BattleFairyCombineEffect::FetchPowerChanged {
                message_type: BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE,
                player_id,
                subject_id: player_id,
                property_name: "dwFetchPower",
                value: self.base_properties.fetch_power,
            });

        for cell in [
            super::container::cbattlefairycontainer::BattleFairyCell::FetchBody,
            super::container::cbattlefairycontainer::BattleFairyCell::FetchStone,
            super::container::cbattlefairycontainer::BattleFairyCell::Material,
        ] {
            let Some(removed) = self
                .battle_fairy_container
                .remove_battle_fairy_combine_input(cell)
            else {
                report.outcome = BattleFairyCombineOutcome::InputRemovalStopped;
                return report;
            };
            report.effects.push(BattleFairyCombineEffect::ObjectMove(
                BattleFairyObjectMove {
                    operation: BattleFairyObjectMoveOperation::Delete,
                    player_id,
                    container_extend_id: BATTLE_FAIRY_CONTAINER_EXTEND_ID,
                    goods: removed.goods,
                    position: removed.cell.position(),
                    amount: removed.amount,
                    old_client_payload: None,
                },
            ));
            tracing::trace!(player_id, cell = ?removed.cell, goods = ?removed.goods, amount = removed.amount, "материал соединения боевой феи удалён");
        }

        if !success {
            report.outcome = BattleFairyCombineOutcome::Failed;
            report
                .effects
                .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                    string_id: "ZHGS0007",
                    account: self.account.clone(),
                    goods_name: Vec::new(),
                }));
            return report;
        }

        let Some(created) = create_goods(recipe.index, random) else {
            report.outcome = BattleFairyCombineOutcome::CreationFailed;
            report
                .effects
                .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                    string_id: "ZHGS0003",
                    account: self.account.clone(),
                    goods_name: Vec::new(),
                }));
            return report;
        };
        let created_identity = created.identity();
        let created_amount = created.amount();
        let mut incoming = Some(created);
        let stored = matches!(
            self.battle_fairy_container.add_at(
                super::container::cbattlefairycontainer::BattleFairyCell::Battle,
                &mut incoming,
                factory,
                true,
            ),
            BattleFairyContainerAddOutcome::Stored {
                base: VolumeGoodsAddOutcome::Added(_),
                ..
            }
        );
        if !stored {
            report.outcome = BattleFairyCombineOutcome::CreationRejected;
            return report;
        }

        let old_client_payload = {
            let goods = self
                .battle_fairy_container
                .base()
                .get_goods(
                    super::container::cbattlefairycontainer::BattleFairyCell::Battle.position(),
                )
                .expect("успешный add боевой феи сохранил goods в Battle cell");
            encode_old_client(goods)
        };
        report.effects.push(BattleFairyCombineEffect::ObjectMove(
            BattleFairyObjectMove {
                operation: BattleFairyObjectMoveOperation::New,
                player_id,
                container_extend_id: BATTLE_FAIRY_CONTAINER_EXTEND_ID,
                goods: created_identity,
                position: BattleFairyCell::Battle.position(),
                amount: created_amount,
                old_client_payload: Some(old_client_payload),
            },
        ));
        report.effects.push(BattleFairyCombineEffect::Notification {
            player_id,
            string_id: "ZHGS0004",
            color: 0xffff_ffff,
        });

        let mut skill_effects = Vec::with_capacity(3);
        let default_properties = {
            let (move_shape, container) = (&mut self.move_shape, &mut self.battle_fairy_container);
            let goods = container
                .base_mut()
                .get_goods_mut(
                    super::container::cbattlefairycontainer::BattleFairyCell::Battle.position(),
                )
                .expect("успешный add боевой феи оставляет Battle cell доступной");
            let mut register_skill = |skill: BattleFairyDefaultSkill| {
                let registered = move_shape.add_skill(skill.id, skill.level, skill_factory);
                if let Some(stored) = move_shape.skill(skill.id, skill_factory) {
                    skill_effects.push(BattleFairyCombineEffect::SkillAdded(
                        battle_fairy_skill_snapshot(player_id, stored, skill_factory),
                    ));
                }
                registered
            };
            CBattleFairyContainer::load_default_properties(
                Some(player_id),
                goods,
                factory,
                &mut register_skill,
                encode_old_client,
            )
            .expect("existing player ID разрешает LoadBFDefualtProperty")
        };
        report.effects.extend(skill_effects);
        report.effects.push(BattleFairyCombineEffect::GoodsUpdated(
            default_properties,
        ));
        let goods_name = self
            .battle_fairy_container
            .base()
            .get_goods(super::container::cbattlefairycontainer::BattleFairyCell::Battle.position())
            .expect("созданная боевая фея остаётся в Battle cell")
            .name()
            .to_vec();
        report
            .effects
            .push(BattleFairyCombineEffect::Audit(BattleFairyAuditLog {
                string_id: "ZHGS0005",
                account: self.account.clone(),
                goods_name,
            }));
        report.outcome = BattleFairyCombineOutcome::Created;
        report
    }

    pub(crate) fn shape_view(&self) -> Option<ShapeView> {
        let identity = self.shape().identity();
        Some(ShapeView {
            identity,
            tile_x: self.shape().get_tile_x().ok()?,
            tile_y: self.shape().get_tile_y().ok()?,
            pos_x_bits: self.shape().get_pos_x().to_bits(),
            pos_y_bits: self.shape().get_pos_y().to_bits(),
            figure: self.figure,
        })
    }
}

/// Переходный адаптер hub-шва Zone `skills/battlefairygear::BattleFairyGearHost`
/// для прежнего `CPlayer` (порция №7b): typed-чтения и мутации живых
/// контейнеров (экипировка/пакет/контейнер боевой феи + base), головного
/// предмета ячейки 10, кошелька и навыков `CMoveShape`. Сборка кадра `0xBF918`
/// (identity + old-client payload одним доступом) остаётся здесь как
/// сериализационный шов прежнего владельца; literal `0x0b_f918` повторяет
/// перенесённые тела.
struct BattleFairyGearPlayerAdapter<'a> {
    player: &'a mut CPlayer,
    factory: &'a CGoodsFactory,
    skill_factory: Option<&'a CSkillFactory>,
    encode_old_client: &'a mut dyn FnMut(&CGoods) -> Vec<u8>,
}

impl<'a> BattleFairyGearPlayerAdapter<'a> {
    fn skill_factory(&self) -> &'a CSkillFactory {
        self.skill_factory
            .expect("шов reset-skill требует skill factory")
    }
}

impl BattleFairyGearHost for BattleFairyGearPlayerAdapter<'_> {
    type Goods = CGoods;
    type GoodsUpdate = BattleFairyDefaultGoodsUpdate;
    type AddOutcome = BattleFairyContainerAddOutcome;
    type Removal = VolumeGoodsRemoveOutcome;
    type CurrencyOutcome = CurrencyDecreaseOutcome;
    type ConsumedGem = BattleFairyUpgradeConsumedGem;

    fn headgear_addon(&self, property: i32, index: u32) -> Option<i32> {
        self.player
            .equipment
            .get_goods(10)
            .map(|goods| goods.addon_property_value(self.factory, property, index))
    }

    fn add_headgear_addon(&mut self, property: i32, delta: i32) {
        if let Some(goods) = self.player.equipment.get_goods_mut(10) {
            add_battle_fairy_addon(goods, self.factory, property, delta);
        }
    }

    fn set_headgear_addon(&mut self, property: i32, index: u32, value: i32) {
        if let Some(goods) = self.player.equipment.get_goods_mut(10) {
            let _stored = goods.set_addon_property_value_core(property, index, value);
        }
    }

    fn clamp_headgear_current(&mut self, current_property: i32, maximum_property: i32) {
        if let Some(goods) = self.player.equipment.get_goods_mut(10) {
            clamp_battle_fairy_current(goods, self.factory, current_property, maximum_property);
        }
    }

    fn headgear_goods_update(&mut self) -> Option<BattleFairyDefaultGoodsUpdate> {
        let player_id = self.player.player_id();
        let goods = self.player.equipment.get_goods(10)?;
        Some(BattleFairyDefaultGoodsUpdate {
            message_type: 0x0b_f918,
            player_id,
            goods: goods.identity(),
            old_client_payload: (self.encode_old_client)(goods),
        })
    }

    fn combat_properties(&mut self) -> &mut PlayerCombatProperties {
        &mut self.player.combat_properties
    }

    fn server_region_present(&self) -> bool {
        self.player.server_region_id.is_some()
    }

    fn gear_addons(&self, goods: &CGoods) -> BattleFairyGearAddons {
        let value = |property_type| goods.addon_property_value(self.factory, property_type, 1);
        BattleFairyGearAddons {
            attack: value(GAP_BF_ATTACK_ADDON),
            sprite: value(GAP_BF_SPRITE_ADDON),
            strength: value(GAP_BF_STRENGH_ADDON),
            brave: value(GAP_BF_ABRAVE_ADDON),
            agility: value(GAP_BF_AGILITY_ADDON),
            spiritualism: value(GAP_BF_SPRITUALISE_ADDON),
            blast: value(GAP_BF_BLAST_ADDON),
            cut_hurt: value(GAP_BF_CUT_HURT_ADDON),
            life: value(GAP_BF_LIFE_ADDON),
            mana: value(GAP_BF_MP_ADDON),
        }
    }

    fn battle_fairy_equip_cell(&self, goods: &CGoods) -> Option<u32> {
        self.factory
            .query_goods_base_properties(goods.base_properties_index())
            .and_then(|properties| properties.battle_fairy_equip_place())
            .and_then(|position| BattleFairyCell::from_position(position as u32))
            .map(BattleFairyCell::position)
    }

    fn property_effect_before_add(&self, cell_position: u32, goods: &CGoods) -> Option<(u32, i32)> {
        let cell = BattleFairyCell::from_position(cell_position)?;
        self.player
            .battle_fairy_container
            .property_effect_before_add(cell, goods, self.factory)
            .map(|effect| (effect.cell.position(), effect.delta))
    }

    fn battle_fairy_add_at(
        &mut self,
        cell_position: u32,
        incoming: &mut Option<CGoods>,
        owner_progress_allows: bool,
    ) -> BattleFairyContainerAddOutcome {
        let cell = BattleFairyCell::from_position(cell_position)
            .expect("positional add передаёт валидную ячейку боевой феи");
        self.player.battle_fairy_container.add_at(
            cell,
            incoming,
            self.factory,
            owner_progress_allows,
        )
    }

    fn battle_fairy_add(
        &mut self,
        incoming: &mut Option<CGoods>,
        owner_progress_allows: bool,
    ) -> BattleFairyContainerAddOutcome {
        self.player
            .battle_fairy_container
            .add(incoming, self.factory, owner_progress_allows)
    }

    fn battle_fairy_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.player
            .battle_fairy_container
            .base()
            .query_goods_position(ex_id)
    }

    fn battle_fairy_cell_addons(&self, position: u32) -> Option<BattleFairyGearAddons> {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(position)
            .map(|goods| self.gear_addons(goods))
    }

    fn battle_fairy_remove_goods(&mut self, ex_id: CGuid) -> Option<VolumeGoodsRemoveOutcome> {
        self.player
            .battle_fairy_container
            .base_mut()
            .remove_goods(ex_id)
    }

    fn battle_fairy_cell_amount(&self, position: u32) -> Option<u32> {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(position)
            .map(|goods| goods.amount())
    }

    fn battle_fairy_cell_ex_id(&self, position: u32) -> Option<CGuid> {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(position)
            .map(|goods| goods.identity().ex_id)
    }

    fn battle_fairy_take_goods(
        &mut self,
        position: u32,
        amount: u32,
        create_goods: &mut dyn FnMut(u32) -> Option<CGoods>,
    ) -> Option<VolumeGoodsRemoveOutcome> {
        self.player.battle_fairy_container.base_mut().take_goods(
            position,
            amount,
            self.factory,
            create_goods,
        )
    }

    fn battle_fairy_cell_goods_update(
        &mut self,
        position: u32,
    ) -> Option<BattleFairyDefaultGoodsUpdate> {
        let player_id = self.player.player_id();
        let goods = self
            .player
            .battle_fairy_container
            .base()
            .get_goods(position)?;
        Some(BattleFairyDefaultGoodsUpdate {
            message_type: 0x0b_f918,
            player_id,
            goods: goods.identity(),
            old_client_payload: (self.encode_old_client)(goods),
        })
    }

    fn battle_fairy_upgrade_price(&mut self) -> u32 {
        self.player
            .battle_fairy_container
            .upgrade_price(self.factory)
    }

    fn currency_amount(&self) -> u32 {
        self.player.wallet.currency_amount()
    }

    fn battle_fairy_cell_present(&self, position: u32) -> bool {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(position)
            .is_some()
    }

    fn battle_fairy_equipment_can_upgrade(&self) -> bool {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::Equipment.position())
            .is_some_and(|goods| goods.can_battle_fairy_equipment_upgrade(self.factory))
    }

    fn battle_fairy_cell_addon(&self, position: u32, property: i32, index: u32) -> Option<i32> {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(position)
            .map(|goods| goods.addon_property_value(self.factory, property, index))
    }

    fn battle_fairy_cell_snapshot(&self, position: u32) -> Option<BattleFairyUpgradeGoodsSnapshot> {
        self.player
            .battle_fairy_container
            .base()
            .get_goods(position)
            .map(|goods| BattleFairyUpgradeGoodsSnapshot {
                identity: goods.identity(),
                name: goods.name().to_vec(),
                price: goods.price(),
                amount: goods.amount(),
            })
    }

    fn battle_fairy_probability(&self) -> u32 {
        self.player.battle_fairy_container.probability(self.factory)
    }

    fn decrease_money(&mut self, amount: u32) -> BattleFairyMoneyChange<CurrencyDecreaseOutcome> {
        let money = self.player.decrease_money(amount, self.factory);
        BattleFairyMoneyChange {
            previous: money.previous,
            current: money.current,
            outcome: money.outcome,
        }
    }

    fn audit_player_snapshot(&self) -> BattleFairyUpgradePlayerSnapshot {
        BattleFairyUpgradePlayerSnapshot {
            pk_count: self.player.base_properties.pk_count,
            money: self.player.money,
            depot_money: self.player.depot_money(),
            region_id: self.player.server_region_id.unwrap_or_default(),
            tile_x: self.player.shape().get_tile_x().unwrap_or_default(),
            tile_y: self.player.shape().get_tile_y().unwrap_or_default(),
            client_ip: self.player.client_ip,
        }
    }

    fn battle_fairy_success_result(&self, random: &mut dyn FnMut(i32) -> i32) -> u32 {
        self.player
            .battle_fairy_container
            .success_result(self.factory, random)
    }

    fn set_battle_fairy_cell_goods_level(&mut self, position: u32, level: i32) {
        if let Some(goods) = self
            .player
            .battle_fairy_container
            .base_mut()
            .get_goods_mut(position)
        {
            let _upgraded = self.factory.upgrade_battle_fairy_equipment(goods, level);
        }
    }

    fn battle_fairy_fail_result(&self) -> u32 {
        self.player.battle_fairy_container.fail_result(self.factory)
    }

    fn battle_fairy_delete_upgrade_target(&mut self) -> Option<VolumeGoodsRemoveOutcome> {
        self.player
            .battle_fairy_container
            .delete_upgrade_target()
            .map(|(_identity, removal)| removal)
    }

    fn battle_fairy_consume_upgrade_gem(
        &mut self,
        cell_position: u32,
    ) -> Option<BattleFairyUpgradeConsumedGem> {
        let cell = BattleFairyCell::from_position(cell_position)?;
        self.player.battle_fairy_container.consume_upgrade_gem(cell)
    }

    fn consumed_gem_facts(gem: &BattleFairyUpgradeConsumedGem) -> (bool, u32, u32) {
        (gem.removed, gem.previous_amount, gem.remaining_amount)
    }

    fn find_reset_goods(&self, name: &[u8]) -> Option<(ShapeIdentity, u32)> {
        let reset_index = self.factory.query_goods_id_by_original_name(Some(name));
        self.player
            .packet
            .base()
            .traversing_goods()
            .find(|goods| goods.base_properties_index() == reset_index)
            .map(|goods| (goods.identity(), goods.amount()))
    }

    fn packet_goods_position(&self, ex_id: CGuid) -> Option<u32> {
        self.player.packet.query_goods_position(ex_id)
    }

    fn packet_remove_goods(&mut self, ex_id: CGuid) -> Option<VolumeGoodsRemoveOutcome> {
        self.player.packet.remove_goods(ex_id)
    }

    fn packet_set_goods_amount(&mut self, position: u32, remaining: u32) -> bool {
        let Some(goods) = self.player.packet.get_goods_mut(position) else {
            return false;
        };
        goods.set_amount(remaining);
        true
    }

    fn delete_war_soul_skill(&mut self, skill_id: u32) {
        let skill_factory = self.skill_factory();
        let _deleted = self.player.move_shape.delete_skill(skill_id, skill_factory);
    }

    fn war_soul_removed_skill_name(&self, skill_id: u32) -> Option<Option<Vec<u8>>> {
        let skill_factory = self.skill_factory();
        self.player
            .move_shape
            .skill(skill_id, skill_factory)
            .map(|skill| skill.name(skill_factory).map(<[u8]>::to_vec))
    }

    fn add_war_soul_skill(&mut self, skill_id: u32, level: i32) {
        let skill_factory = self.skill_factory();
        let _added = self
            .player
            .move_shape
            .add_skill(skill_id, level, skill_factory);
    }

    fn war_soul_added_skill_facts(&self, skill_id: u32) -> Option<(i32, u32, Option<Vec<u8>>)> {
        let skill_factory = self.skill_factory();
        self.player
            .move_shape
            .skill(skill_id, skill_factory)
            .map(|skill| {
                (
                    skill.level(),
                    skill.skill_type(),
                    skill.name(skill_factory).map(<[u8]>::to_vec),
                )
            })
    }
}

fn battle_fairy_skill_snapshot(
    player_id: i32,
    skill: &MoveShapeSkill,
    factory: &CSkillFactory,
) -> BattleFairySkillAdded {
    BattleFairySkillAdded {
        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
        player_id,
        skill_id: skill.id(),
        skill_level: skill.level(),
        skill_type: skill.skill_type(),
        skill_name: skill.name(factory).map(<[u8]>::to_vec),
    }
}

fn war_soul_skill_entry_from_goods(
    goods: &CGoods,
    factory: &CGoodsFactory,
    property: BattleFairySkillProperty,
) -> (u32, i32) {
    battle_fairy_skill_entry(property, |property, index| {
        battle_fairy_skill_property_value(goods, factory, property, index)
    })
}

fn war_soul_skill_id_from_goods(
    goods: &CGoods,
    factory: &CGoodsFactory,
    property: BattleFairySkillProperty,
) -> u32 {
    battle_fairy_skill_id(property, |property, index| {
        battle_fairy_skill_property_value(goods, factory, property, index)
    })
}

fn battle_fairy_skill_property_value(
    goods: &CGoods,
    factory: &CGoodsFactory,
    property: BattleFairySkillProperty,
    index: u32,
) -> i32 {
    goods.addon_property_value(factory, battle_fairy_skill_property_key(property), index)
}

fn push_battle_fairy_skill_reject(journal: &mut GameEffectJournal, socket_id: i32) {
    journal.push(GameEffect::SkillSocketReject {
        socket_id,
        message_type: SKILL_EFFECT_MESSAGE_TYPE,
        reason: SKILL_REJECT_WAR_SOUL_REASON,
        code: SKILL_REJECT_CODE,
    });
}

fn push_player_skill_reject(journal: &mut GameEffectJournal, socket_id: i32) {
    journal.push(GameEffect::SkillSocketReject {
        socket_id,
        message_type: SKILL_EFFECT_MESSAGE_TYPE,
        reason: SKILL_REJECT_REASON,
        code: SKILL_REJECT_CODE,
    });
}

fn add_battle_fairy_addon(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    property_type: i32,
    delta: i32,
) {
    let value = goods
        .addon_property_value(factory, property_type, 1)
        .wrapping_add(delta);
    let _stored = goods.set_addon_property_value_core(property_type, 1, value);
}

fn clamp_battle_fairy_current(
    goods: &mut CGoods,
    factory: &CGoodsFactory,
    current_property: i32,
    maximum_property: i32,
) {
    let maximum = goods.addon_property_value(factory, maximum_property, 1);
    if maximum < goods.addon_property_value(factory, current_property, 1) {
        let _stored = goods.set_addon_property_value_core(current_property, 1, maximum);
    }
}

const fn clamp_combat_scalar(value: u32) -> u32 {
    if LEGACY_COMBAT_MAXIMUM < value {
        LEGACY_COMBAT_MAXIMUM
    } else {
        value
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp

// ============================================================================
// ============================================================================
// COMPONENT_VARIANT_END: GameServer

// Граница реконструкции аукциона: AutoAddAuctionGoods остаётся отдельным
// неперенесённым producer; listing-completion уже выполняется owner-ом 0x90A02.
