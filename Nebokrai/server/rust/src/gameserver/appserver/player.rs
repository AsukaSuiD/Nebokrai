//! Достигнутая send-family проекция `CPlayer` исторического GameServer.
//!
//! PDB `GameServer/GameServer.pdb` подтверждает base `CMoveShape +0x0` и signed
//! `m_lTeamID +0xB20`, а также unsigned byte `m_btCountry +0xA5C`. Exact
//! `CMessage::SendToAround` RVA `0x00014420` и
//! `SendToRegionContryPlayer` RVA `0x00014760` читают inherited
//! `CBaseObject::m_lID +0x8` как numeric map/player identity, team ID и country
//! после RTTI `CShape/CMoveShape -> CPlayer`. Эти достигнутые поля имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; исходники
//! `server/gameserver/appserver/player.h/.cpp`.
//! `CMessage::Run` RVA `0x000149D0` дополнительно читает inherited father
//! `+0x40` как текущий `CServerRegion*`; удалённый raw pointer выражен
//! `Option<i32>` region identity в assembly-проекции.
//!
//! Материализована также подтверждённая setter-family: боевые scalar-ы
//! насыщаются до `INT_MAX`, contribution — до `±2_000_000_000`, а fetch power
//! сравнивается с unsigned-представлением setup limit. Это минимальный owned
//! player state для будущих equipment/battle-fairy side effects, но не замена
//! полного constructor-а, property recalc или runtime player lifecycle.
//! FourNation reward `0x7FE46` добавляет owned `dwExploit`: advertised client
//! value сохраняет wrapping addition, `SetExploit` отдельно применяет exact
//! unsigned CountryParam maximum, а virtual `UpdateProperty` остаётся
//! обязательным caller-runtime effect после мутации.
//! Total honor-rank startup материализует days/weeks/months counters и
//! nobility rank: reset меняет owned state, а пока RAW `PlayerRunScript`
//! выражен точным typed AdjustHonorRank script-effect-ом.
//! Silence-timeout, как и оригинал, проверяется лениво при query по
//! инъецируемому wrapping `timeGetTime`-значению; GM `0x7FC0B/0x7FC0E`
//! замыкают name lookup, mutation, двухпроходный ordered query и World
//! responses, поэтому отдельный scheduler не требуется.
//! Текущие HP/MP имеют собственные setter-и с clamp к текущим max-свойствам;
//! изменение самих max не выполняет этот clamp без конкретного caller-а.
//! Как в связном `RefreshContainerOwners`, достигнутые equipment и
//! battle-fairy containers принадлежат player type `400` с его numeric ID;
//! Depot-password vertical дополнительно материализует `m_eProgress`, оба
//! changing-guard-а, password byte-string и owned `CBank/CDepot`; numeric
//! значения внутреннего `eProgress` не выходят в wire и потому заменены typed
//! enum без выдуманного `repr`.
//! Exact `GetWarSoulGoods` читает headgear cell 10 и признаёт её боевой феей
//! только при addon `GAP_BF_BATTLE_FAIRY` value-id 1, равном единице.
//! `BatllteFairyCombine` соединяет container inputs, global BattleFairy gate,
//! fetch power, shared Game RNG/factory, `CMoveShape::AddSkill` и ordered
//! адресные object/skill/goods/audit effects. `BTreeMap` skill storage в
//! `CMoveShape` заменяет четыре pointer-vector-а только для общего confirmed
//! identity/level/type/name state; выполнение concrete skill owners не
//! перенесено сюда. Account для audit принадлежит player snapshot и пока
//! заполняется отдельным caller-ом при восстановлении player identity.
//! Поэтому `from_send_state` остаётся явной assembly-границей уже
//! восстановленного runtime. Figure передаётся как доказанный derived virtual
//! fact; владение spatial state остаётся у `CMoveShape`.
//! `SummonBF` RVA `0x00101CB0` материализован единым Player→CGame→region
//! проходом: guards, summon/recall state, ordered around effects и area-map
//! action. Active pet пока count-derived fact; codec и goods-message decoder
//! остаются явной границей и report не подменяет исторические packet bytes.
//! `BFPropertyAdd` соединяет восемь gear-ячеек с headgear battle fairy,
//! `GlobeSetup` occupation coefficients и player combat state. Сохранены
//! ранний effect до результата Add, post-remove `-1`, clamp текущих HP/MP,
//! двойное применение MaxHP/Str/Int/Dex и двойной `0xBF918` в Remove.
//! Goods-message `0x8FC2A` материализован до ordered potential mutation:
//! aggregate guard остаётся в клиентских единицах, отдельные allocation
//! умножаются на `10000`, одинаковые property keys имеют `std::map` first-win,
//! а каждый вызов и outer caller публикуют собственный `0xBF918`.
//! Upgrade `0x8FC28` замыкает validation, wallet snapshot, общий RNG,
//! factory level/growth mutation, target failure outcome, positional расход
//! gem-ов и ordered client/audit effects. Конкретный wallet-object codec и
//! полная audit-wire упаковка остаются transport boundary returned report-а.
//! `ResetPotential` использует owned packet `CVolumeLimitGoodsContainer` 8×12:
//! первый `ZHQLS01` расходуется до addon/player mutation, семь tracked-вкладов
//! возвращаются в общий potential и публикуется один итоговый `0xBF918`.
//! `ResetSkill` соединяет equipment headgear, optional packet-reset item,
//! общий Game RNG, exact несовместимые пары, полный detach/attach девяти
//! war-soul skills и подтверждения `0xBF71D/0xBF918`.
//! `skillmessage 0x90005` доведён до authorization и AI dispatch: feature/HP
//! guards, странный special-skill fallback `546/547`, self-target rewrite и
//! socket reject сохранены; concrete `CPlayerAI`, region symbol rule и полный
//! monster registry передаются как explicit facts.
//! Nation-war player lifecycle связывает exact `SetContendState`,
//! `OnDied`/`OnRelive` и millisecond-tail `PeriodicalUpdate`: owned state
//! хранит три PDB-поля `+0xBA5/+0xBA8/+0xBAC`, а конкретные self/around
//! маршруты сообщений остаются у `CGame`, владеющего network/session runtime.
//! Periodic `ComputeWarSoulXY` сохраняет float follow-state, exact dead/snap
//! thresholds, общий area-map tail и последующий `0xBF605`; restored-state
//! concrete skill остаётся входным фактом. Non-finite повреждённый float-state
//! блокируется typed outcome до старого x87 integer conversion.
//! Periodic HP-death prefix `CPlayer::AI` повторно нормализует summon/state и
//! recall/died флаги нулевой по HP equipped fairy, затем вызывает
//! `PropertiesChanged`. Оригинал в этой ветви не чистит stale area-map entry и
//! не посылает status broadcast; оба отсутствующих side effect сохранены.
//! `CEquipmentContainer::OnObjectRemoved` player-tail связывает снятие
//! headgear с exact `SetWarSoulStaus(0)`, девятью skill detach, пересчётом
//! свойств при уже отсутствующем slot-е, HP/MP clamp и `0xBF720`. Полный
//! virtual property owner остаётся injected callback-границей.
//! GodsBattle player snapshot теперь также хранит persisted faction/SZL;
//! faction membership появляется только в concrete region AddObject-tail и
//! удаляется его RemoveObject/DelObj-tail, не при восстановлении snapshot-а.
//! `UpdateSZL` проходит через `CGame`: player property/notice предшествуют
//! decrease-only appellation check и script-effect-у `RequestChangeAppellation`.
//! Симметричный `OnObjectAdded` сохраняет late-block partial mutations, после
//! commit добавляет девять war-soul skills, пересчитывает свойства, публикует
//! `0xBF720` с исключением owner-а и отражает даже zero-delta `PackExpand` log.

use super::area::WarSoulPoint;
use super::container::cbank::CBank;
use super::container::cbattlefairycontainer::{
    BattleFairyCell, BattleFairyCombineCheck, BattleFairyCombineRemovedInput,
    BattleFairyContainerAddOutcome, BattleFairyDefaultGoodsUpdate, BattleFairyDefaultSkill,
    BattleFairyPropertyAddEffect, BattleFairyUpgradeConsumedGem, CBattleFairyContainer,
};
use super::container::cdepot::CDepot;
use super::container::cequipmentcontainer::{
    CEquipmentContainer, EquipmentAddOutcome, EquipmentAddRuntimeFacts, EquipmentAroundUpdate,
    EquipmentColumn, EquipmentOwnerPlayerFacts, EquipmentRemoveOutcome,
    EquipmentRemoveRuntimeFacts,
};
use super::container::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeGoodsAddOutcome, VolumeGoodsRemoveOutcome,
};
use super::goods::cbattlefairyproperty::BattleFairyCompose;
use super::goods::cgoods::CGoods;
use super::goods::cgoodsbaseproperties::{
    GAP_BF_ABRAVE_ADDON, GAP_BF_AGILITY, GAP_BF_AGILITY_ADDON, GAP_BF_AGILITY_POTENTIAL,
    GAP_BF_ALL_SKILL, GAP_BF_ATTACK, GAP_BF_ATTACK_ADDON, GAP_BF_ATTACK_POTENTIAL,
    GAP_BF_BATTLE_FAIRY, GAP_BF_BLAST, GAP_BF_BLAST_ADDON, GAP_BF_BLAST_POTENTIAL, GAP_BF_BRAVE,
    GAP_BF_BRAVE_POTENTIAL, GAP_BF_CUT_HURT_ADDON, GAP_BF_CUT_HURT_SCALE, GAP_BF_EARTH,
    GAP_BF_EARTH_SKILL, GAP_BF_HP, GAP_BF_HUOXIESHU_SKILL, GAP_BF_LIFE_ADDON,
    GAP_BF_LINGZHISHU_SKILL, GAP_BF_MAN, GAP_BF_MAN_SKILL, GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP,
    GAP_BF_MP_ADDON, GAP_BF_POTENTIAL, GAP_BF_SKY, GAP_BF_SKY_SKILL, GAP_BF_SPRITE,
    GAP_BF_SPRITE_ADDON, GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITUALISE_ADDON, GAP_BF_SPRITUALISM,
    GAP_BF_SPRITUALISM_POTENTIAL, GAP_BF_STRENGH, GAP_BF_STRENGH_ADDON, GAP_BF_STRENGH_POTENTIAL,
    GAP_BF_WEAPON_LEVEL, GAP_GEM_LEVEL,
};
use super::goods::cgoodsfactory::CGoodsFactory;
use super::moveshape::{CMoveShape, MoveShapePositionFacts, MoveShapeSkill};
use super::shape::{CShape, ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapeView};
use super::skills::skillfactory::CSkillFactory;
use crate::public::guid::CGuid;
use crate::setup::globesetup::GlobePlayerPropertyCoefficients;
use std::collections::BTreeMap;

const PLAYER_TYPE: i32 = 400;
const LEGACY_COMBAT_MAXIMUM: u32 = i32::MAX as u32;
const CONTRIBUTION_MINIMUM: i32 = -2_000_000_000;
const CONTRIBUTION_MAXIMUM: i32 = 2_000_000_000;
const BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE: u32 = 0x0b_f71d;
const BATTLE_FAIRY_FETCH_POWER_MESSAGE_TYPE: u32 = 0x0b_f80c;
const BATTLE_FAIRY_CONTAINER_EXTEND_ID: u32 = 0x0c;
const MONSTER_TAMING_SKILL_ID: u32 = 0xd4;
const BATTLE_FAIRY_MOVE_MESSAGE_TYPE: u32 = 0x0b_f605;
const BATTLE_FAIRY_STATUS_MESSAGE_TYPE: u32 = 0x0b_f930;
const BATTLE_FAIRY_SUMMON_MESSAGE_TYPE: u32 = 0x0b_f92e;
const BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE: u32 = 0x0b_f71e;
const BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING: &str = "ZHGS0022";
const SKILL_EFFECT_MESSAGE_TYPE: u32 = 0x0b_fe01;
const SKILL_REJECT_WAR_SOUL_REASON: u32 = 4;
const SKILL_REJECT_CODE: u8 = 0x0c;
const SKILL_POJIA: u32 = 530;
const SKILL_LEIMING: u32 = 543;
const SKILL_ID_MASK: u32 = i32::MAX as u32;

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
    pub(crate) old_client_payload: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillAdded {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) skill_type: u32,
    pub(crate) skill_name: Vec<u8>,
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
    pub(crate) removed_inputs: Vec<BattleFairyCombineRemovedInput>,
    pub(crate) effects: Vec<BattleFairyCombineEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyWarSoulAction {
    SetPosition {
        previous: WarSoulPoint,
        target: WarSoulPoint,
    },
    Delete {
        previous: WarSoulPoint,
        player_position: WarSoulPoint,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonOutcome {
    FeatureDisabled,
    AlreadySummoned,
    AlreadyRecalled,
    MissingHeadgear,
    InvalidHeadgear,
    NoHitPoints,
    ActivePet,
    MonsterTamingActive,
    CoordinateBlocked(ShapeCoordinateBlock),
    Summoned,
    Recalled,
    IgnoredMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySummonEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    AroundMessage {
        message_type: u32,
        player_id: i32,
        values: Vec<i32>,
    },
    PropertiesChanged {
        player_id: i32,
    },
}

#[must_use = "summon report хранит точный порядок адресных broadcast и property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySummonReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairySummonOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) spatial_applied: bool,
    pub(crate) effects: Vec<BattleFairySummonEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowOutcome {
    ActiveSkill,
    NotSummoned,
    CoordinateBlocked(ShapeCoordinateBlock),
    NonFiniteVisualState,
    InsideDeadZone,
    Moved,
    Snapped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyFollowEffect {
    AroundMove {
        message_type: u32,
        player_id: i32,
        object_type: i32,
        x: u32,
        y: u32,
    },
}

#[must_use = "follow report содержит spatial tail и обязательный move broadcast"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyFollowReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyFollowOutcome,
    pub(crate) region_id: Option<i32>,
    pub(crate) visual_x_bits: u32,
    pub(crate) visual_y_bits: u32,
    pub(crate) spatial_action: Option<BattleFairyWarSoulAction>,
    pub(crate) spatial_applied: bool,
    pub(crate) effects: Vec<BattleFairyFollowEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyDeathOutcome {
    MissingHeadgear,
    NotBattleFairy,
    Alive,
    Died,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyDeathEffect {
    PropertiesChanged { player_id: i32 },
}

#[must_use = "death report сохраняет periodic state transition и property effect"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyDeathReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyDeathOutcome,
    pub(crate) effects: Vec<BattleFairyDeathEffect>,
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
    pub(crate) effects: Vec<PlayerEquipmentRemoveEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddRuntimeFacts {
    pub(crate) can_mount_result: i32,
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
    },
    AroundUpdate(EquipmentAroundUpdate),
    PackageExtensionLogged {
        category: &'static str,
        string_id: &'static str,
        expanded_package_num: u32,
    },
}

#[must_use = "equipment add report сохраняет partial mutations и player/network tail"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEquipmentAddReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: EquipmentAddOutcome,
    pub(crate) effects: Vec<PlayerEquipmentAddEffect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationOutcome {
    Added(BattleFairyContainerAddOutcome),
    Removed(VolumeGoodsRemoveOutcome),
    MissingGoods,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyEquipmentMutationEffect {
    PropertiesChanged { player_id: i32 },
    BattleFairyUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "equipment report сохраняет container ownership и ранние property effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyEquipmentMutationReport {
    pub(crate) player_id: i32,
    pub(crate) cell: Option<BattleFairyCell>,
    pub(crate) delta: i32,
    pub(crate) property_applied: bool,
    pub(crate) outcome: BattleFairyEquipmentMutationOutcome,
    pub(crate) effects: Vec<BattleFairyEquipmentMutationEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationOutcome {
    MissingHeadgear,
    InvalidHeadgear,
    AggregateInsufficient,
    Processed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialAllocationEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "allocation report сохраняет ordered player и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialAllocationReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialAllocationOutcome,
    pub(crate) aggregate_client_points: i32,
    pub(crate) processed_properties: Vec<i32>,
    pub(crate) effects: Vec<BattleFairyPotentialAllocationEffect>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeLogGates {
    pub(crate) success: bool,
    pub(crate) failure: bool,
    pub(crate) lost_target: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeOutcome {
    MissingRegion,
    InsufficientMoney,
    InvalidEquipment,
    MissingBaseGem,
    GemLevelMismatch,
    MaximumLevel,
    Succeeded,
    FailedKept,
    FailedDowngraded,
    FailedReset,
    FailedDestroyed,
    ConsumptionStopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyUpgradeEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        format_value: Option<u32>,
    },
    MoneyChanged {
        player_id: i32,
        previous: u32,
        current: u32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
    GemConsumed(BattleFairyUpgradeConsumedGem),
    TargetDeleted {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        removal: VolumeGoodsRemoveOutcome,
    },
    Audit {
        message_type: u32,
        event: u8,
        player_id: i32,
        target: super::shape::ShapeIdentity,
        gems: [Option<super::shape::ShapeIdentity>; 4],
    },
}

#[must_use = "upgrade report содержит wallet, ownership и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyUpgradeReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyUpgradeOutcome,
    pub(crate) price: u32,
    pub(crate) probability: u32,
    pub(crate) previous_level: Option<i32>,
    pub(crate) resulting_level: Option<i32>,
    pub(crate) consumed_gems: Vec<BattleFairyUpgradeConsumedGem>,
    pub(crate) effects: Vec<BattleFairyUpgradeEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPotentialResetEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<VolumeGoodsRemoveOutcome>,
    },
    PropertiesChanged {
        player_id: i32,
    },
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "reset report содержит packet ownership и player/network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyPotentialResetReport {
    pub(crate) player_id: i32,
    pub(crate) outcome: BattleFairyPotentialResetOutcome,
    pub(crate) recovered_potential: i32,
    pub(crate) effects: Vec<BattleFairyPotentialResetEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetOutcome {
    FeatureDisabled,
    MissingHeadgear,
    InvalidHeadgear,
    MissingResetItem,
    InvalidPosition,
    SelectedSkillUnavailable,
    Reset,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRemoved {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_name: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillResetEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
    },
    PacketItemConsumed {
        player_id: i32,
        goods: super::shape::ShapeIdentity,
        previous_amount: u32,
        remaining_amount: u32,
        consumed: bool,
        removal: Option<VolumeGoodsRemoveOutcome>,
    },
    SkillRemoved(BattleFairySkillRemoved),
    SkillAdded(BattleFairySkillAdded),
    SelectedSkillLearned(BattleFairySkillAdded),
    GoodsUpdated(BattleFairyDefaultGoodsUpdate),
}

#[must_use = "skill reset report содержит packet, skill-state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillResetReport {
    pub(crate) player_id: i32,
    pub(crate) position: i32,
    pub(crate) outcome: BattleFairySkillResetOutcome,
    pub(crate) previous_skill: Option<u32>,
    pub(crate) selected_skill: Option<u32>,
    pub(crate) detached_skill_ids: Vec<u32>,
    pub(crate) attached_skill_ids: Vec<u32>,
    pub(crate) effects: Vec<BattleFairySkillResetEffect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequest {
    pub(crate) raw_skill_id: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) property_offset: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
}

impl BattleFairySkillRequest {
    pub(crate) const fn skill_id(self) -> u32 {
        self.raw_skill_id as u32 & SKILL_ID_MASK
    }
}

/// Facts ещё сырых virtual owner-ов `CServerRegion::SymbolIsAttackAble`,
/// `CPlayer::GetAI` и полного player/monster region registry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequestFacts {
    pub(crate) symbol_attackable: bool,
    pub(crate) player_ai_available: bool,
    pub(crate) object_target_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillDispatch {
    SelfTarget {
        skill_id: u32,
        player_id: i32,
    },
    Point {
        skill_id: u32,
        x: i32,
        y: i32,
    },
    Object {
        skill_id: u32,
        target: super::shape::ShapeIdentity,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillRequestOutcome {
    FeatureDisabled,
    MissingHeadgear,
    NoHitPoints,
    Unauthorized,
    CoordinateBlocked(ShapeCoordinateBlock),
    AiUnavailable,
    MissingRegion,
    MissingTarget,
    Queued,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairySkillRequestEffect {
    Notification {
        player_id: i32,
        string_id: &'static str,
        color: u32,
        message_type: u32,
    },
    SocketReject {
        message_type: u32,
        reason: u32,
        code: u8,
    },
    AiDispatch(BattleFairySkillDispatch),
}

#[must_use = "war-soul skill report содержит authorization, target rewrite и dispatch"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairySkillRequestReport {
    pub(crate) player_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) target_type: i32,
    pub(crate) target_id: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
    pub(crate) outcome: BattleFairySkillRequestOutcome,
    pub(crate) effects: Vec<BattleFairySkillRequestEffect>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct BattleFairyGearAddons {
    attack: i32,
    sprite: i32,
    strength: i32,
    brave: i32,
    agility: i32,
    spiritualism: i32,
    blast: i32,
    cut_hurt: i32,
    life: i32,
    mana: i32,
}

impl BattleFairyGearAddons {
    fn read(goods: &CGoods, factory: &CGoodsFactory) -> Self {
        let value = |property_type| goods.addon_property_value(factory, property_type, 1);
        Self {
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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerBaseProperties {
    pub(crate) level: u8,
    pub(crate) occupation: u8,
    pub(crate) pk_count: u16,
    pub(crate) experience: u32,
    pub(crate) health: u32,
    pub(crate) mana: u32,
    pub(crate) fetch_power: u32,
    pub(crate) battle_fairy_recall: bool,
    pub(crate) battle_fairy_died: bool,
    pub(crate) days_honor_eliminate: u32,
    pub(crate) weeks_honor_eliminate: u32,
    pub(crate) months_honor_eliminate: u32,
    pub(crate) total_honor_eliminate: u32,
    pub(crate) rank_of_nobility_id: u32,
    pub(crate) exploit: u32,
    pub(crate) gods_battle_faction: i32,
    pub(crate) szl: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerHonorResetReport {
    pub(crate) player_id: i32,
    pub(crate) reset_mask: u32,
    pub(crate) previous_days: u32,
    pub(crate) previous_weeks: u32,
    pub(crate) previous_months: u32,
    pub(crate) adjust_honor_rank_script: Option<&'static [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExploitMutationReport {
    pub(crate) player_id: i32,
    pub(crate) previous: u32,
    pub(crate) requested: u32,
    pub(crate) applied: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerCombatProperties {
    pub(crate) maximum_hp: u32,
    pub(crate) maximum_mp: u32,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) constitution: u32,
    pub(crate) intelligence: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) defense: u32,
    pub(crate) element_resistance: u32,
    pub(crate) burden: u16,
    pub(crate) reank: u16,
    pub(crate) element_modify: i32,
    pub(crate) blast_defense_scale_bits: u32,
    pub(crate) full_miss_scale_bits: u32,
    pub(crate) critical_rate_bits: u32,
}

impl PlayerCombatProperties {
    pub(crate) const fn blast_defense_scale(self) -> f32 {
        f32::from_bits(self.blast_defense_scale_bits)
    }

    pub(crate) const fn full_miss_scale(self) -> f32 {
        f32::from_bits(self.full_miss_scale_bits)
    }

    pub(crate) const fn critical_rate(self) -> f32 {
        f32::from_bits(self.critical_rate_bits)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PlayerProgress {
    #[default]
    None,
    Banking,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPlayer {
    move_shape: CMoveShape,
    figure: ShapeFigure,
    faction_id: i32,
    team_id: i32,
    country: u8,
    server_region_id: Option<i32>,
    in_changing_server: bool,
    in_changing_region: bool,
    current_progress: PlayerProgress,
    war_soul_state: u32,
    war_soul_point: WarSoulPoint,
    war_soul_visual_x_bits: u32,
    war_soul_visual_y_bits: u32,
    battle_fairy_summoned: bool,
    recreate_carriage: bool,
    active_pet_count: u32,
    attempt_appellation_id: u32,
    base_properties: PlayerBaseProperties,
    combat_properties: PlayerCombatProperties,
    ci_qing_open: bool,
    contend_state: bool,
    city_war_died_state: bool,
    city_war_died_state_time_ms: i32,
    died_state_start_time_ms: u32,
    contribution: i32,
    silence_minutes: i32,
    silence_timestamp_minutes: u32,
    money: u32,
    account: Vec<u8>,
    depot_password: Vec<u8>,
    bank: CBank,
    depot: CDepot,
    packet: CVolumeLimitGoodsContainer,
    equipment: CEquipmentContainer,
    battle_fairy_container: CBattleFairyContainer,
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

impl CPlayer {
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
        let mut packet = CVolumeLimitGoodsContainer::new();
        let _empty_release = packet.set_container_dimensions(8, 12);
        let mut player = Self {
            move_shape,
            figure,
            faction_id: 0,
            team_id,
            country,
            server_region_id,
            in_changing_server: false,
            in_changing_region: false,
            current_progress: PlayerProgress::None,
            war_soul_state: 0,
            war_soul_point: WarSoulPoint::default(),
            war_soul_visual_x_bits: 0.0f32.to_bits(),
            war_soul_visual_y_bits: 0.0f32.to_bits(),
            battle_fairy_summoned: false,
            recreate_carriage: false,
            active_pet_count: 0,
            attempt_appellation_id: 0,
            base_properties: PlayerBaseProperties::default(),
            combat_properties: PlayerCombatProperties::default(),
            ci_qing_open: false,
            contend_state: false,
            city_war_died_state: false,
            city_war_died_state_time_ms: 0,
            died_state_start_time_ms: 0,
            contribution: 0,
            silence_minutes: 0,
            silence_timestamp_minutes: 0,
            money: 0,
            account: Vec::new(),
            depot_password: Vec::new(),
            bank: CBank::new(),
            depot: CDepot::new(),
            packet,
            equipment: CEquipmentContainer::new(),
            battle_fairy_container: CBattleFairyContainer::new(),
        };
        player.refresh_reached_container_owners(owner_id);
        Some(player)
    }

    pub(crate) const fn shape(&self) -> &CShape {
        self.move_shape.shape()
    }

    pub(crate) const fn player_id(&self) -> i32 {
        self.shape().identity().id
    }

    pub(crate) const fn team_id(&self) -> i32 {
        self.team_id
    }

    pub(crate) const fn faction_id(&self) -> i32 {
        self.faction_id
    }

    pub(crate) const fn restore_faction_id(&mut self, faction_id: i32) {
        self.faction_id = faction_id;
    }

    pub(crate) const fn country(&self) -> u8 {
        self.country
    }

    /// Сохраняет player tail total-honor startup ветви: days сбрасывается
    /// всегда, weeks/months — по mask `2/4`, total не меняется, а ненулевой
    /// nobility rank требует точного `AdjustHonorRank` script-effect-а.
    pub(crate) fn reset_total_honor_eliminate(
        &mut self,
        reset_mask: u32,
    ) -> PlayerHonorResetReport {
        let report = PlayerHonorResetReport {
            player_id: self.player_id(),
            reset_mask,
            previous_days: self.base_properties.days_honor_eliminate,
            previous_weeks: self.base_properties.weeks_honor_eliminate,
            previous_months: self.base_properties.months_honor_eliminate,
            adjust_honor_rank_script: (self.base_properties.rank_of_nobility_id != 0)
                .then_some(b"scripts/circle/honorrank/adjusthonorrank.script"),
        };
        self.base_properties.days_honor_eliminate = 0;
        if reset_mask & 2 != 0 {
            self.base_properties.weeks_honor_eliminate = 0;
        }
        if reset_mask & 4 != 0 {
            self.base_properties.months_honor_eliminate = 0;
        }
        report
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

    pub(crate) const fn set_changing_state_snapshot(
        &mut self,
        in_changing_server: bool,
        in_changing_region: bool,
    ) {
        self.in_changing_server = in_changing_server;
        self.in_changing_region = in_changing_region;
    }

    pub(crate) const fn current_progress(&self) -> PlayerProgress {
        self.current_progress
    }

    pub(crate) const fn set_current_progress_snapshot(&mut self, progress: PlayerProgress) {
        self.current_progress = progress;
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

    pub(crate) const fn bank_locked(&self) -> bool {
        self.bank.is_locked()
    }

    pub(crate) const fn depot_locked(&self) -> bool {
        self.depot.is_locked()
    }

    pub(crate) const fn base_properties(&self) -> PlayerBaseProperties {
        self.base_properties
    }

    pub(crate) const fn gods_battle_faction(&self) -> i32 {
        self.base_properties.gods_battle_faction
    }

    pub(crate) const fn level(&self) -> u8 {
        self.base_properties.level
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

    pub(crate) const fn clear_attempt_appellation(&mut self) {
        self.attempt_appellation_id = 0;
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
    pub(crate) fn set_exploit(
        &mut self,
        requested: u32,
        maximum: i32,
    ) -> PlayerExploitMutationReport {
        let previous = self.base_properties.exploit;
        let applied = requested.min(maximum as u32);
        self.base_properties.exploit = applied;
        PlayerExploitMutationReport {
            player_id: self.player_id(),
            previous,
            requested,
            applied,
        }
    }

    /// Exact `SetValue("dwExploit", value)` из region reward path:
    /// generic property map пишет `DWORD` напрямую и не вызывает `SetExploit` clamp.
    pub(crate) fn set_exploit_property_value(
        &mut self,
        requested: u32,
    ) -> PlayerExploitMutationReport {
        let previous = self.base_properties.exploit;
        self.base_properties.exploit = requested;
        PlayerExploitMutationReport {
            player_id: self.player_id(),
            previous,
            requested,
            applied: requested,
        }
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

    pub(crate) fn can_enter_gods_battle_contend(&self) -> bool {
        self.shape().get_action() != 6 && !CMoveShape::is_died(self.base_properties.health)
    }

    /// Focused same-region branch `ChangeRegion`, которую вызывает
    /// `ServerNationRegion::KickOutAllPlayerToReturnPoint`.
    pub(crate) fn prepare_nation_relive(&mut self) {
        self.current_progress = PlayerProgress::None;
        self.recreate_carriage = false;
    }

    pub(crate) const fn nation_relive_position_facts(
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

    pub(crate) const fn nation_relive_shape_mut(&mut self) -> &mut CShape {
        self.move_shape.shape_mut()
    }

    pub(crate) const fn combat_properties(&self) -> PlayerCombatProperties {
        self.combat_properties
    }

    pub(crate) const fn ci_qing_open(&self) -> bool {
        self.ci_qing_open
    }

    pub(crate) const fn contend_state(&self) -> bool {
        self.contend_state
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

    pub(crate) const fn contribution(&self) -> i32 {
        self.contribution
    }

    pub(crate) const fn money(&self) -> u32 {
        self.money
    }

    pub(crate) const fn set_money_snapshot(&mut self, money: u32) {
        self.money = money;
    }

    pub(crate) const fn silence_minutes(&self) -> i32 {
        self.silence_minutes
    }

    pub(crate) const fn equipment(&self) -> &CEquipmentContainer {
        &self.equipment
    }

    pub(crate) const fn packet(&self) -> &CVolumeLimitGoodsContainer {
        &self.packet
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

    pub(crate) const fn battle_fairy_container_mut(&mut self) -> &mut CBattleFairyContainer {
        &mut self.battle_fairy_container
    }

    /// Account принадлежит player snapshot и используется exact audit-log
    /// combine; отсутствие ещё не загруженного account остаётся пустой строкой.
    pub(crate) fn set_account(&mut self, account: impl AsRef<[u8]>) {
        self.account.clear();
        self.account.extend_from_slice(account.as_ref());
    }

    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    /// Exact `GetWarSoulGoods`: боевой дух — только headgear в позиции 10,
    /// чьё первое значение `GAP_BF_BATTLE_FAIRY` равно единице.
    pub(crate) fn war_soul_goods(&self, factory: &CGoodsFactory) -> Option<&CGoods> {
        self.equipment
            .get_goods(10)
            .filter(|goods| goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) == 1)
    }

    /// Exact derived `bHasPet`: отдельный pet owner materializes list later;
    /// этому caller-у нужен только подтверждённый факт её непустоты.
    pub(crate) const fn set_active_pet_count(&mut self, count: u32) {
        self.active_pet_count = count;
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
    /// затем любое значение кроме единицы нормализуется к нулю.
    pub(crate) const fn set_war_soul_status(&mut self, value: u32) -> bool {
        let broadcast_previous = self.war_soul_state == 1;
        if value == 1 {
            self.battle_fairy_summoned = true;
            self.war_soul_state = 1;
        } else {
            self.battle_fairy_summoned = false;
            self.war_soul_state = 0;
        }
        broadcast_previous
    }

    /// Исполняет player-часть `CBattleFairyContainer::SummonBF`. Spatial map
    /// принадлежит `CServerRegion`, поэтому действие возвращается явным
    /// tail-ом для `CGame`; ordered notify/broadcast/property effects не
    /// сериализуются выдуманным transport-ом.
    pub(crate) fn summon_battle_fairy(
        &mut self,
        battle_fairy_enabled: bool,
        mode: i32,
        factory: &CGoodsFactory,
    ) -> BattleFairySummonReport {
        let player_id = self.player_id();
        let mut report = BattleFairySummonReport {
            player_id,
            outcome: BattleFairySummonOutcome::IgnoredMode,
            region_id: self.server_region_id,
            spatial_action: None,
            spatial_applied: false,
            effects: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySummonOutcome::FeatureDisabled;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0023", 0xffff_ffff);
            return report;
        }
        if mode == 1 && self.war_soul_state == 1 {
            report.outcome = BattleFairySummonOutcome::AlreadySummoned;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0024", 0xffff_ffff);
            return report;
        }
        if mode == -1 && self.base_properties.battle_fairy_recall {
            report.outcome = BattleFairySummonOutcome::AlreadyRecalled;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0025", 0xffff_ffff);
            return report;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            report.outcome = BattleFairySummonOutcome::MissingHeadgear;
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairySummonOutcome::InvalidHeadgear;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0009", 0xffff_ffff);
            return report;
        }
        if goods.addon_property_value(factory, GAP_BF_HP, 1) < 1 {
            report.outcome = BattleFairySummonOutcome::NoHitPoints;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0026", 0xffff_0000);
            return report;
        }
        if self.has_pet() {
            report.outcome = BattleFairySummonOutcome::ActivePet;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0027", 0xffff_ffff);
            return report;
        }
        if self.move_shape.current_skill_id() == Some(MONSTER_TAMING_SKILL_ID) {
            report.outcome = BattleFairySummonOutcome::MonsterTamingActive;
            push_battle_fairy_summon_notification(&mut report, "ZHGS0028", 0xffff_ffff);
            return report;
        }
        let player_position = match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
            (Ok(x), Ok(y)) => WarSoulPoint { x, y },
            (Err(error), _) | (_, Err(error)) => {
                report.outcome = BattleFairySummonOutcome::CoordinateBlocked(error);
                return report;
            }
        };

        match mode {
            1 => {
                self.battle_fairy_summoned = true;
                self.war_soul_state = 1;
                self.base_properties.battle_fairy_recall = false;
                self.base_properties.battle_fairy_died = false;
                self.war_soul_visual_x_bits = (player_position.x as f32).to_bits();
                self.war_soul_visual_y_bits = (player_position.y as f32).to_bits();
                report.outcome = BattleFairySummonOutcome::Summoned;
                report.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
                    previous: self.war_soul_point,
                    target: player_position,
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
                    player_id,
                    values: vec![player_id, 700, player_position.x, player_position.y],
                });
                // `SetWarSoulStaus(1)` наблюдает уже записанный state `1` и
                // поэтому публикует exact `0xbf930 {400, player_id}`.
                let _broadcast_previous = self.set_war_soul_status(1);
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, player_id],
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_SUMMON_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, 1],
                });
            }
            -1 => {
                self.battle_fairy_summoned = false;
                self.war_soul_state = 0;
                self.base_properties.battle_fairy_recall = true;
                self.base_properties.battle_fairy_died = false;
                self.war_soul_visual_x_bits = (-1.0f32).to_bits();
                self.war_soul_visual_y_bits = (-1.0f32).to_bits();
                report.outcome = BattleFairySummonOutcome::Recalled;
                report.spatial_action = Some(BattleFairyWarSoulAction::Delete {
                    previous: self.war_soul_point,
                    player_position,
                });
                report.effects.push(BattleFairySummonEffect::AroundMessage {
                    message_type: BATTLE_FAIRY_STATUS_MESSAGE_TYPE,
                    player_id,
                    values: vec![400, -1],
                });
            }
            _ => {}
        }
        report
            .effects
            .push(BattleFairySummonEffect::PropertiesChanged { player_id });
        report
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
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
    ) -> PlayerEquipmentRemoveReport {
        let player_id = self.player_id();
        let outcome = self.equipment.remove(
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
                for (skill_id, _) in war_soul_skill_entries_from_goods(&removed.goods, factory) {
                    let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
                    effects.push(PlayerEquipmentRemoveEffect::WarSoulSkillDetached { skill_id });
                    if let Some(skill) = self.move_shape.skill(skill_id) {
                        effects.push(PlayerEquipmentRemoveEffect::SkillRemoved(
                            BattleFairySkillRemoved {
                                message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                                player_id,
                                skill_id,
                                skill_name: skill.name().to_vec(),
                            },
                        ));
                    }
                }
            }
            if player_effects.recompute_without_removed_slot {
                self.combat_properties = recompute_properties(self);
                effects.push(
                    PlayerEquipmentRemoveEffect::PropertiesChangedWithoutRemovedSlot {
                        column: removed.event.column,
                        combat_properties: self.combat_properties,
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
            effects,
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
        register_with_goods_ai: &mut dyn FnMut(&CGoods),
        recompute_properties: &mut dyn FnMut(&CPlayer) -> PlayerCombatProperties,
    ) -> PlayerEquipmentAddReport {
        let player_id = self.player_id();
        let previous_expanded_package_num = self.equipment.expanded_package_num();
        let outcome = self.equipment.add_at(
            position,
            incoming,
            factory,
            EquipmentAddRuntimeFacts {
                owner_player: Some(EquipmentOwnerPlayerFacts {
                    can_mount_result: runtime.can_mount_result,
                }),
                pack_add_enabled: runtime.pack_add_enabled,
                now: runtime.now,
            },
            register_with_goods_ai,
        );
        let mut effects = Vec::new();
        if let EquipmentAddOutcome::Added(added) = &outcome
            && let Some(player_effects) = added.player_effects
        {
            if added.package_extension_applied {
                self.equipment
                    .set_expanded_package_num_snapshot(previous_expanded_package_num);
            }
            if player_effects.add_war_soul_skill
                && let Some(goods) = self.equipment.get_goods(added.column.position())
            {
                for (skill_id, level) in war_soul_skill_entries_from_goods(goods, factory) {
                    let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
                    effects
                        .push(PlayerEquipmentAddEffect::WarSoulSkillAttached { skill_id, level });
                    if let Some(skill) = self.move_shape.skill(skill_id) {
                        effects.push(PlayerEquipmentAddEffect::SkillAdded(
                            battle_fairy_skill_snapshot(player_id, skill),
                        ));
                    }
                }
            }
            if player_effects.recompute_properties {
                self.combat_properties = recompute_properties(self);
                effects.push(PlayerEquipmentAddEffect::PropertiesChanged {
                    combat_properties: self.combat_properties,
                });
            }
            effects.push(PlayerEquipmentAddEffect::AroundUpdate(
                player_effects.around_update,
            ));
            if added.package_extension_applied {
                self.equipment.set_expanded_package_num_snapshot(
                    previous_expanded_package_num.wrapping_add(added.package_extension_delta),
                );
                effects.push(PlayerEquipmentAddEffect::PackageExtensionLogged {
                    category: "PackExpand",
                    string_id: "KR002",
                    expanded_package_num: self.equipment.expanded_package_num(),
                });
            }
        }
        PlayerEquipmentAddReport {
            player_id,
            outcome,
            effects,
        }
    }

    /// Завершает CGame-owned area tail. Recall всегда копирует player point
    /// после попытки `DelWarSoul`, даже если old area отсутствовала; это
    /// literal последняя запись `CPlayer::DelWarSoul`.
    pub(crate) const fn apply_war_soul_action(
        &mut self,
        action: BattleFairyWarSoulAction,
        spatial_applied: bool,
    ) {
        match action {
            BattleFairyWarSoulAction::SetPosition { target, .. } if spatial_applied => {
                self.war_soul_point = target;
            }
            BattleFairyWarSoulAction::Delete {
                player_position, ..
            } => {
                self.war_soul_point = player_position;
            }
            BattleFairyWarSoulAction::SetPosition { .. } => {}
        }
    }

    /// Один живой `ComputeWarSoulXY` tick. `Some(false)` означает найденный
    /// current war-soul skill с `IsRestored()==0`; `None` точно соответствует
    /// отсутствующему skill и не блокирует follow.
    pub(crate) fn compute_war_soul_xy(
        &mut self,
        current_war_soul_skill_restored: Option<bool>,
    ) -> BattleFairyFollowReport {
        let player_id = self.player_id();
        let mut report = BattleFairyFollowReport {
            player_id,
            outcome: BattleFairyFollowOutcome::NotSummoned,
            region_id: self.server_region_id,
            visual_x_bits: self.war_soul_visual_x_bits,
            visual_y_bits: self.war_soul_visual_y_bits,
            spatial_action: None,
            spatial_applied: false,
            effects: Vec::new(),
        };
        if current_war_soul_skill_restored == Some(false) {
            report.outcome = BattleFairyFollowOutcome::ActiveSkill;
            return report;
        }
        if self.war_soul_state != 1 {
            return report;
        }
        let (tile_x, tile_y) = match (self.shape().get_tile_x(), self.shape().get_tile_y()) {
            (Ok(x), Ok(y)) => (x, y),
            (Err(error), _) | (_, Err(error)) => {
                report.outcome = BattleFairyFollowOutcome::CoordinateBlocked(error);
                return report;
            }
        };
        let current_x = tile_x as f32;
        let current_y = tile_y as f32;
        let mut visual_x = f32::from_bits(self.war_soul_visual_x_bits);
        let mut visual_y = f32::from_bits(self.war_soul_visual_y_bits);
        let delta_x = current_x - visual_x;
        let delta_y = current_y - visual_y;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt().abs();
        if !distance.is_finite() {
            report.outcome = BattleFairyFollowOutcome::NonFiniteVisualState;
            return report;
        }
        if distance < 0.5 {
            report.outcome = BattleFairyFollowOutcome::InsideDeadZone;
            return report;
        }

        let (target, outcome) = if distance <= 5.0 {
            let coefficient = if distance > 3.75 {
                0.265f32
            } else if distance > 0.75 {
                0.065f32
            } else {
                0.045f32
            };
            let step = distance * (coefficient + coefficient);
            if (current_x - visual_x).abs() > 0.1 {
                visual_x = if current_x <= visual_x {
                    visual_x - step
                } else {
                    visual_x + step
                };
            }
            if (current_y - visual_y).abs() > 0.1 {
                visual_y = if current_y <= visual_y {
                    visual_y - step
                } else {
                    visual_y + step
                };
            }
            (
                WarSoulPoint {
                    // EXE временно ставит x87 RC=truncate перед обоими fistp.
                    x: visual_x.trunc() as i32,
                    y: visual_y.trunc() as i32,
                },
                BattleFairyFollowOutcome::Moved,
            )
        } else {
            visual_x = current_x;
            visual_y = current_y;
            (
                WarSoulPoint {
                    x: tile_x,
                    y: tile_y,
                },
                BattleFairyFollowOutcome::Snapped,
            )
        };
        self.war_soul_visual_x_bits = visual_x.to_bits();
        self.war_soul_visual_y_bits = visual_y.to_bits();
        report.visual_x_bits = self.war_soul_visual_x_bits;
        report.visual_y_bits = self.war_soul_visual_y_bits;
        report.outcome = outcome;
        report.spatial_action = Some(BattleFairyWarSoulAction::SetPosition {
            previous: self.war_soul_point,
            target,
        });
        report.effects.push(BattleFairyFollowEffect::AroundMove {
            message_type: BATTLE_FAIRY_MOVE_MESSAGE_TYPE,
            player_id,
            object_type: 700,
            x: legacy_f32_to_u32(visual_x),
            y: legacy_f32_to_u32(visual_y),
        });
        report
    }

    /// Periodic prefix `CPlayer::AI`: нулевой HP equipped battle fairy каждый
    /// tick повторно нормализует четыре state-поля и вызывает PropertiesChanged.
    /// Исходник не удаляет stale area-map entry и не посылает status broadcast.
    pub(crate) fn refresh_battle_fairy_death(
        &mut self,
        factory: &CGoodsFactory,
    ) -> BattleFairyDeathReport {
        let player_id = self.player_id();
        let mut report = BattleFairyDeathReport {
            player_id,
            outcome: BattleFairyDeathOutcome::MissingHeadgear,
            effects: Vec::new(),
        };
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyDeathOutcome::NotBattleFairy;
            return report;
        }
        if goods.addon_property_value(factory, GAP_BF_HP, 1) != 0 {
            report.outcome = BattleFairyDeathOutcome::Alive;
            return report;
        }
        self.battle_fairy_summoned = false;
        self.war_soul_state = 0;
        self.set_battle_fairy_recall(true);
        self.set_battle_fairy_died(true);
        report.outcome = BattleFairyDeathOutcome::Died;
        report
            .effects
            .push(BattleFairyDeathEffect::PropertiesChanged { player_id });
        report
    }

    fn apply_battle_fairy_property(
        &mut self,
        cell: BattleFairyCell,
        addons: BattleFairyGearAddons,
        delta: i32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> Option<BattleFairyDefaultGoodsUpdate> {
        if delta == 0 || !is_battle_fairy_property_cell(cell) {
            return None;
        }
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let player_id = self.player_id();
        let battle_fairy = self.equipment.get_goods_mut(10)?;
        if battle_fairy.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            return None;
        }

        for (source, target) in [
            (addons.attack, GAP_BF_ATTACK),
            (addons.sprite, GAP_BF_SPRITE),
            (addons.strength, GAP_BF_STRENGH),
            (addons.brave, GAP_BF_BRAVE),
            (addons.agility, GAP_BF_AGILITY),
            (addons.spiritualism, GAP_BF_SPRITUALISM),
            (addons.blast, GAP_BF_BLAST),
            (addons.cut_hurt, GAP_BF_CUT_HURT_SCALE),
        ] {
            add_battle_fairy_addon(battle_fairy, factory, target, source.wrapping_mul(delta));
        }
        add_battle_fairy_addon(
            battle_fairy,
            factory,
            GAP_BF_MAX_HP,
            addons
                .strength
                .wrapping_add(addons.life)
                .wrapping_mul(delta),
        );
        add_battle_fairy_addon(
            battle_fairy,
            factory,
            GAP_BF_MAX_MP,
            addons
                .spiritualism
                .wrapping_add(addons.mana)
                .wrapping_mul(delta),
        );
        clamp_battle_fairy_current(battle_fairy, factory, GAP_BF_HP, GAP_BF_MAX_HP);
        clamp_battle_fairy_current(battle_fairy, factory, GAP_BF_MP, GAP_BF_MAX_MP);

        let strength = f64::from(addons.strength) * f64::from(delta) * 0.00001;
        let brave = f64::from(addons.brave) * f64::from(delta) * 0.00001;
        let agility = f64::from(addons.agility) * f64::from(delta) * 0.00001;
        let spiritualism = f64::from(addons.spiritualism) * f64::from(delta) * 0.00001;
        let combat = &mut self.combat_properties;
        combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
        combat.strength = add_battle_fairy_u32(combat.strength, brave);
        combat.maximum_attack = add_battle_fairy_u32(
            combat.maximum_attack,
            brave * f64::from(coefficients.str_to_max_attack[occupation]),
        );
        combat.burden = add_battle_fairy_u16(
            combat.burden,
            brave * f64::from(coefficients.str_to_burden[occupation]),
        );
        combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);
        combat.minimum_attack = add_battle_fairy_u32(
            combat.minimum_attack,
            agility * f64::from(coefficients.dex_to_min_attack[occupation]),
        );
        combat.reank = add_battle_fairy_u16(
            combat.reank,
            agility * f64::from(coefficients.dex_to_stiff[occupation]),
        );
        combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
        combat.element_modify = add_battle_fairy_i32(
            combat.element_modify,
            spiritualism * f64::from(coefficients.int_to_element[occupation]),
        );
        combat.maximum_mp = add_battle_fairy_u32(
            combat.maximum_mp,
            spiritualism * f64::from(coefficients.int_to_max_mp[occupation]),
        );
        combat.element_resistance = add_battle_fairy_u32(
            combat.element_resistance,
            spiritualism * f64::from(coefficients.int_to_resistant[occupation]),
        );

        // Подтверждённый RU quirk: четыре основных значения применяются
        // повторно после производных коэффициентов.
        combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, strength);
        combat.strength = add_battle_fairy_u32(combat.strength, brave);
        combat.intelligence = add_battle_fairy_u32(combat.intelligence, spiritualism);
        combat.dexterity = add_battle_fairy_u32(combat.dexterity, agility);

        Some(BattleFairyDefaultGoodsUpdate {
            message_type: 0x0b_f918,
            player_id,
            goods: battle_fairy.identity(),
            old_client_payload: encode_old_client(battle_fairy),
        })
    }

    /// Exact positional `CBattleFairyContainer::Add`: для gear-ячеек
    /// `BFPropertyAdd(+1)` является ранним partial effect и сохраняется даже
    /// если base storage затем отвергнет товар.
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
        let early_property = incoming.as_ref().and_then(|goods| {
            self.battle_fairy_container
                .property_effect_before_add(cell, goods, factory)
                .map(|effect| (effect, BattleFairyGearAddons::read(goods, factory)))
        });
        let mut property_applied = false;
        let mut effects = Vec::new();
        if let Some((BattleFairyPropertyAddEffect { cell, delta }, addons)) = early_property
            && let Some(update) = self.apply_battle_fairy_property(
                cell,
                addons,
                delta,
                factory,
                coefficients,
                encode_old_client,
            )
        {
            property_applied = true;
            effects.push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
            effects.push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                update,
            ));
        }
        let outcome =
            self.battle_fairy_container
                .add_at(cell, incoming, factory, owner_progress_allows);
        BattleFairyEquipmentMutationReport {
            player_id,
            cell: Some(cell),
            delta: 1,
            property_applied,
            outcome: BattleFairyEquipmentMutationOutcome::Added(outcome),
            effects,
        }
    }

    /// Exact `Remove`: base container отделяет goods до `BFPropertyAdd(-1)`;
    /// успешный property path сериализует battle fairy дважды — один раз в
    /// `BFPropertyAdd`, затем ещё раз в override `Remove`.
    pub(crate) fn remove_battle_fairy_goods(
        &mut self,
        ex_id: CGuid,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyEquipmentMutationReport {
        let player_id = self.player_id();
        let position = self
            .battle_fairy_container
            .base()
            .query_goods_position(ex_id);
        let cell = position.and_then(BattleFairyCell::from_position);
        let addons = position
            .and_then(|position| self.battle_fairy_container.base().get_goods(position))
            .map(|goods| BattleFairyGearAddons::read(goods, factory));
        let Some(outcome) = self.battle_fairy_container.base_mut().remove_goods(ex_id) else {
            return BattleFairyEquipmentMutationReport {
                player_id,
                cell,
                delta: -1,
                property_applied: false,
                outcome: BattleFairyEquipmentMutationOutcome::MissingGoods,
                effects: Vec::new(),
            };
        };
        let mut report = BattleFairyEquipmentMutationReport {
            player_id,
            cell,
            delta: -1,
            property_applied: false,
            outcome: BattleFairyEquipmentMutationOutcome::Removed(outcome),
            effects: Vec::new(),
        };
        if let (Some(cell), Some(addons)) = (cell, addons)
            && let Some(first_update) = self.apply_battle_fairy_property(
                cell,
                addons,
                -1,
                factory,
                coefficients,
                encode_old_client,
            )
        {
            report.property_applied = true;
            report
                .effects
                .push(BattleFairyEquipmentMutationEffect::PropertiesChanged { player_id });
            report
                .effects
                .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                    first_update,
                ));
            if let Some(battle_fairy) = self.war_soul_goods(factory) {
                report
                    .effects
                    .push(BattleFairyEquipmentMutationEffect::BattleFairyUpdated(
                        BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: battle_fairy.identity(),
                            old_client_payload: encode_old_client(battle_fairy),
                        },
                    ));
            }
        }
        report
    }

    /// Полный player-side opcode `0x8FC2A`. `allocations` содержат пары
    /// property/client-points прямо из packet-а: legacy outer caller суммирует
    /// unscaled points, но передаёт каждому `AllocatePotential` wrapping
    /// `points * 10000`. `std::map::insert` сохраняет первую запись ключа.
    pub(crate) fn allocate_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        allocations: &[(i32, i32)],
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialAllocationReport {
        let player_id = self.player_id();
        let aggregate_client_points = allocations
            .iter()
            .fold(0i32, |total, (_, points)| total.wrapping_add(*points));
        let mut report = BattleFairyPotentialAllocationReport {
            player_id,
            outcome: BattleFairyPotentialAllocationOutcome::MissingHeadgear,
            aggregate_client_points,
            processed_properties: Vec::new(),
            effects: Vec::new(),
        };
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyPotentialAllocationOutcome::InvalidHeadgear;
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::Notification {
                    player_id,
                    string_id: "ZHGS0009",
                    color: 0xffff_ffff,
                });
            return report;
        }
        if goods
            .addon_property_value(factory, GAP_BF_POTENTIAL, 1)
            .wrapping_sub(aggregate_client_points)
            < 0
        {
            report.outcome = BattleFairyPotentialAllocationOutcome::AggregateInsufficient;
            return report;
        }

        let mut ordered = BTreeMap::new();
        for &(property, points) in allocations {
            ordered.entry(property).or_insert(points);
        }
        for (property, points) in ordered {
            if !battle_fairy_enabled {
                report
                    .effects
                    .push(BattleFairyPotentialAllocationEffect::Notification {
                        player_id,
                        string_id: "ZHGS0008",
                        color: 0xffff_0000,
                    });
                continue;
            }
            let amount = points.wrapping_mul(10_000);
            self.allocate_one_battle_fairy_potential(property, amount, factory, coefficients);
            report.processed_properties.push(property);
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::PropertiesChanged { player_id });
            if let Some(goods) = self.war_soul_goods(factory) {
                report
                    .effects
                    .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(
                        BattleFairyDefaultGoodsUpdate {
                            message_type: 0x0b_f918,
                            player_id,
                            goods: goods.identity(),
                            old_client_payload: encode_old_client(goods),
                        },
                    ));
            }
        }

        // Outer goods-message сериализует headgear ещё раз независимо от
        // feature-disabled/unknown-property результата внутренних вызовов.
        if let Some(goods) = self.war_soul_goods(factory) {
            report
                .effects
                .push(BattleFairyPotentialAllocationEffect::GoodsUpdated(
                    BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: encode_old_client(goods),
                    },
                ));
        }
        report.outcome = BattleFairyPotentialAllocationOutcome::Processed;
        report
    }

    fn allocate_one_battle_fairy_potential(
        &mut self,
        property: i32,
        amount: i32,
        factory: &CGoodsFactory,
        coefficients: GlobePlayerPropertyCoefficients,
    ) {
        let occupation = usize::from(self.base_properties.occupation).min(2);
        let mut player_delta = None;
        {
            let Some(goods) = self.equipment.get_goods_mut(10) else {
                return;
            };
            let potential = goods.addon_property_value(factory, GAP_BF_POTENTIAL, 1);
            if potential.wrapping_sub(amount) < 0 {
                return;
            }
            let (tracked_property, applied_amount) = match property {
                GAP_BF_ATTACK => (
                    GAP_BF_ATTACK_POTENTIAL,
                    (f64::from(amount) * 1.5).round() as i32,
                ),
                GAP_BF_SPRITE => (
                    GAP_BF_SPRITE_POTENTIAL,
                    (f64::from(amount) * 1.5).round() as i32,
                ),
                GAP_BF_BLAST => (GAP_BF_BLAST_POTENTIAL, amount),
                GAP_BF_BRAVE => (GAP_BF_BRAVE_POTENTIAL, amount),
                GAP_BF_AGILITY => (GAP_BF_AGILITY_POTENTIAL, amount),
                GAP_BF_SPRITUALISM => (GAP_BF_SPRITUALISM_POTENTIAL, amount),
                GAP_BF_STRENGH => (GAP_BF_STRENGH_POTENTIAL, amount),
                _ => return,
            };
            add_battle_fairy_addon(goods, factory, property, applied_amount);
            add_battle_fairy_addon(goods, factory, tracked_property, applied_amount);
            let _stored = goods.set_addon_property_value_core(
                GAP_BF_POTENTIAL,
                1,
                potential.wrapping_sub(amount),
            );
            if property == GAP_BF_SPRITUALISM {
                add_battle_fairy_addon(goods, factory, GAP_BF_MAX_MP, amount);
            } else if property == GAP_BF_STRENGH {
                add_battle_fairy_addon(goods, factory, GAP_BF_MAX_HP, amount);
            }
            if matches!(
                property,
                GAP_BF_BRAVE | GAP_BF_AGILITY | GAP_BF_SPRITUALISM | GAP_BF_STRENGH
            ) {
                player_delta = Some((property, f64::from(amount) * 0.00001));
            }
        }

        let Some((property, delta)) = player_delta else {
            return;
        };
        let combat = &mut self.combat_properties;
        match property {
            GAP_BF_BRAVE => {
                combat.strength = add_battle_fairy_u32(combat.strength, delta);
                combat.maximum_attack = add_battle_fairy_u32(
                    combat.maximum_attack,
                    delta * f64::from(coefficients.str_to_max_attack[occupation]),
                );
                combat.burden = add_battle_fairy_u16(
                    combat.burden,
                    delta * f64::from(coefficients.str_to_burden[occupation]),
                );
            }
            GAP_BF_AGILITY => {
                combat.dexterity = add_battle_fairy_u32(combat.dexterity, delta);
                combat.minimum_attack = add_battle_fairy_u32(
                    combat.minimum_attack,
                    delta * f64::from(coefficients.dex_to_min_attack[occupation]),
                );
                combat.reank = add_battle_fairy_u16(
                    combat.reank,
                    delta * f64::from(coefficients.dex_to_stiff[occupation]),
                );
            }
            GAP_BF_SPRITUALISM => {
                combat.intelligence = add_battle_fairy_u32(combat.intelligence, delta);
                combat.element_modify = add_battle_fairy_i32(
                    combat.element_modify,
                    delta * f64::from(coefficients.int_to_element[occupation]),
                );
                combat.maximum_mp = add_battle_fairy_u32(
                    combat.maximum_mp,
                    delta * f64::from(coefficients.int_to_max_mp[occupation]),
                );
                combat.element_resistance = add_battle_fairy_u32(
                    combat.element_resistance,
                    delta * f64::from(coefficients.int_to_resistant[occupation]),
                );
            }
            GAP_BF_STRENGH => {
                combat.maximum_hp = add_battle_fairy_u32(combat.maximum_hp, delta);
            }
            _ => {}
        }
    }

    pub(crate) fn upgrade_battle_fairy_equipment(
        &mut self,
        factory: &CGoodsFactory,
        log_gates: BattleFairyUpgradeLogGates,
        random: &mut dyn FnMut(i32) -> i32,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyUpgradeReport {
        let player_id = self.player_id();
        let price = self.battle_fairy_container.upgrade_price(factory);
        let mut report = BattleFairyUpgradeReport {
            player_id,
            outcome: BattleFairyUpgradeOutcome::MissingRegion,
            price,
            probability: 0,
            previous_level: None,
            resulting_level: None,
            consumed_gems: Vec::new(),
            effects: Vec::new(),
        };
        if self.server_region_id.is_none() {
            return report;
        }
        if self.money < price {
            report.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0015", Some(price));
            return report;
        }
        let Some(equipment) = self
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::Equipment.position())
        else {
            report.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0014", None);
            return report;
        };
        if !equipment.can_battle_fairy_equipment_upgrade(factory) {
            report.outcome = BattleFairyUpgradeOutcome::InvalidEquipment;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0014", None);
            return report;
        }
        let current_level = equipment.addon_property_value(factory, GAP_BF_WEAPON_LEVEL, 1);
        report.previous_level = Some(current_level);
        let target_identity = equipment.identity();
        let Some(base_gem) = self
            .battle_fairy_container
            .base()
            .get_goods(BattleFairyCell::GemBase.position())
        else {
            report.outcome = BattleFairyUpgradeOutcome::MissingBaseGem;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0013", None);
            return report;
        };
        let minimum = base_gem.addon_property_value(factory, GAP_GEM_LEVEL, 1);
        let maximum = base_gem
            .addon_property_value(factory, GAP_GEM_LEVEL, 2)
            .max(minimum);
        if current_level < minimum || maximum < current_level {
            report.outcome = BattleFairyUpgradeOutcome::GemLevelMismatch;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0012", None);
            return report;
        }
        if 98 < current_level as u32 {
            report.outcome = BattleFairyUpgradeOutcome::MaximumLevel;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0021", None);
            return report;
        }
        report.probability = self.battle_fairy_container.probability(factory);
        if self.money < price {
            report.outcome = BattleFairyUpgradeOutcome::InsufficientMoney;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0020", None);
            return report;
        }
        let gem_identities = [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ]
        .map(|cell| {
            self.battle_fairy_container
                .base()
                .get_goods(cell.position())
                .map(CGoods::identity)
        });
        let previous_money = self.money;
        self.money = self.money.wrapping_sub(price);
        report.effects.push(BattleFairyUpgradeEffect::MoneyChanged {
            player_id,
            previous: previous_money,
            current: self.money,
        });

        let success = (random(100) as u32).wrapping_add(1) <= report.probability;
        let mut target_present = true;
        if success {
            let increase = self.battle_fairy_container.success_result(factory, random);
            let target_level = (current_level as u32).wrapping_add(increase).min(99) as i32;
            if let Some(goods) = self
                .battle_fairy_container
                .base_mut()
                .get_goods_mut(BattleFairyCell::Equipment.position())
            {
                let _upgraded = factory.upgrade_battle_fairy_equipment(goods, target_level);
            }
            report.outcome = BattleFairyUpgradeOutcome::Succeeded;
            push_battle_fairy_upgrade_notification(&mut report, "ZHGS0002", None);
            if log_gates.success {
                report.effects.push(BattleFairyUpgradeEffect::Audit {
                    message_type: 0x0006_0203,
                    event: 1,
                    player_id,
                    target: target_identity,
                    gems: gem_identities,
                });
            }
        } else {
            if log_gates.failure {
                report.effects.push(BattleFairyUpgradeEffect::Audit {
                    message_type: 0x0006_0203,
                    event: 2,
                    player_id,
                    target: target_identity,
                    gems: gem_identities,
                });
            }
            match self.battle_fairy_container.fail_result(factory) {
                1 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedKept;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0016", None);
                }
                2 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedDowngraded;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0017", None);
                    if current_level != 0
                        && let Some(goods) = self
                            .battle_fairy_container
                            .base_mut()
                            .get_goods_mut(BattleFairyCell::Equipment.position())
                    {
                        let _upgraded = factory
                            .upgrade_battle_fairy_equipment(goods, current_level.wrapping_sub(1));
                    }
                }
                3 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedReset;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0018", None);
                    if let Some(goods) = self
                        .battle_fairy_container
                        .base_mut()
                        .get_goods_mut(BattleFairyCell::Equipment.position())
                    {
                        let _upgraded = factory.upgrade_battle_fairy_equipment(goods, 0);
                    }
                }
                4 => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedDestroyed;
                    push_battle_fairy_upgrade_notification(&mut report, "ZHGS0019", None);
                    if log_gates.lost_target {
                        report.effects.push(BattleFairyUpgradeEffect::Audit {
                            message_type: 0x0006_0202,
                            event: 5,
                            player_id,
                            target: target_identity,
                            gems: gem_identities,
                        });
                    }
                    if let Some((goods, removal)) =
                        self.battle_fairy_container.delete_upgrade_target()
                    {
                        target_present = false;
                        report
                            .effects
                            .push(BattleFairyUpgradeEffect::TargetDeleted {
                                player_id,
                                goods,
                                removal,
                            });
                    }
                }
                _ => {
                    report.outcome = BattleFairyUpgradeOutcome::FailedKept;
                }
            }
        }
        if target_present
            && let Some(goods) = self
                .battle_fairy_container
                .base()
                .get_goods(BattleFairyCell::Equipment.position())
        {
            report.resulting_level =
                Some(goods.addon_property_value(factory, GAP_BF_WEAPON_LEVEL, 1));
            report.effects.push(BattleFairyUpgradeEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: goods.identity(),
                    old_client_payload: encode_old_client(goods),
                },
            ));
        }

        for cell in [
            BattleFairyCell::GemBase,
            BattleFairyCell::GemOne,
            BattleFairyCell::GemTwo,
            BattleFairyCell::GemThree,
        ] {
            let was_present = self
                .battle_fairy_container
                .base()
                .get_goods(cell.position())
                .is_some();
            let Some(consumed) = self.battle_fairy_container.consume_upgrade_gem(cell) else {
                if was_present || cell == BattleFairyCell::GemBase {
                    report.outcome = BattleFairyUpgradeOutcome::ConsumptionStopped;
                    break;
                }
                continue;
            };
            report.consumed_gems.push(consumed.clone());
            report
                .effects
                .push(BattleFairyUpgradeEffect::GemConsumed(consumed.clone()));
            if !consumed.removed
                && let Some(goods) = self
                    .battle_fairy_container
                    .base()
                    .get_goods(cell.position())
            {
                report.effects.push(BattleFairyUpgradeEffect::GoodsUpdated(
                    BattleFairyDefaultGoodsUpdate {
                        message_type: 0x0b_f918,
                        player_id,
                        goods: goods.identity(),
                        old_client_payload: encode_old_client(goods),
                    },
                ));
            }
        }
        report
    }

    pub(crate) fn reset_battle_fairy_potential(
        &mut self,
        battle_fairy_enabled: bool,
        factory: &CGoodsFactory,
        encode_old_client: &mut dyn FnMut(&CGoods) -> Vec<u8>,
    ) -> BattleFairyPotentialResetReport {
        let player_id = self.player_id();
        let mut report = BattleFairyPotentialResetReport {
            player_id,
            outcome: BattleFairyPotentialResetOutcome::MissingHeadgear,
            recovered_potential: 0,
            effects: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairyPotentialResetOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return report;
        }
        let Some(headgear) = self.equipment.get_goods(10) else {
            return report;
        };
        if headgear.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairyPotentialResetOutcome::InvalidHeadgear;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0009",
                    color: 0xffff_ffff,
                });
            return report;
        }

        let reset_index = factory.query_goods_id_by_original_name(Some(b"ZHQLS01"));
        let reset_item = self
            .packet
            .base()
            .traversing_goods()
            .find(|goods| goods.base_properties_index() == reset_index)
            .map(|goods| (goods.identity(), goods.amount()));
        let Some((reset_identity, reset_amount)) = reset_item else {
            report.outcome = BattleFairyPotentialResetOutcome::MissingResetItem;
            report
                .effects
                .push(BattleFairyPotentialResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0010",
                    color: 0xffff_ffff,
                });
            return report;
        };
        let reset_position = self.packet.query_goods_position(reset_identity.ex_id);
        let (remaining_amount, consumed, removal) = if reset_amount == 0 {
            (0, false, None)
        } else if reset_amount == 1 {
            let removal = self.packet.remove_goods(reset_identity.ex_id);
            (
                if removal.is_some() { 0 } else { reset_amount },
                removal.is_some(),
                removal,
            )
        } else {
            let remaining = reset_amount.wrapping_sub(1);
            let mut consumed = false;
            if let Some(position) = reset_position
                && let Some(goods) = self.packet.get_goods_mut(position)
            {
                goods.set_amount(remaining);
                consumed = true;
            }
            (
                if consumed { remaining } else { reset_amount },
                consumed,
                None,
            )
        };
        report
            .effects
            .push(BattleFairyPotentialResetEffect::PacketItemConsumed {
                player_id,
                goods: reset_identity,
                previous_amount: reset_amount,
                remaining_amount,
                consumed,
                removal,
            });

        let recovered = {
            let goods = self
                .equipment
                .get_goods_mut(10)
                .expect("headgear проверен до packet consumption");
            let mut take = |tracked, property| {
                let value = goods.addon_property_value(factory, tracked, 1);
                let _tracked_stored = goods.set_addon_property_value_core(tracked, 1, 0);
                let current = goods.addon_property_value(factory, property, 1);
                let _property_stored =
                    goods.set_addon_property_value_core(property, 1, current.wrapping_sub(value));
                value
            };
            let attack = take(GAP_BF_ATTACK_POTENTIAL, GAP_BF_ATTACK);
            let sprite = take(GAP_BF_SPRITE_POTENTIAL, GAP_BF_SPRITE);
            let blast = take(GAP_BF_BLAST_POTENTIAL, GAP_BF_BLAST);
            let brave = take(GAP_BF_BRAVE_POTENTIAL, GAP_BF_BRAVE);
            let agility = take(GAP_BF_AGILITY_POTENTIAL, GAP_BF_AGILITY);
            let spiritualism = take(GAP_BF_SPRITUALISM_POTENTIAL, GAP_BF_SPRITUALISM);
            let strength = take(GAP_BF_STRENGH_POTENTIAL, GAP_BF_STRENGH);
            let recovered = ((f64::from(sprite) + f64::from(attack)) * (2.0 / 3.0)
                + f64::from(blast)
                + f64::from(brave)
                + f64::from(agility)
                + f64::from(spiritualism)
                + f64::from(strength))
            .round() as i32;
            let potential = goods.addon_property_value(factory, GAP_BF_POTENTIAL, 1);
            let _stored = goods.set_addon_property_value_core(
                GAP_BF_POTENTIAL,
                1,
                potential.wrapping_add(recovered),
            );
            (recovered, brave, agility, spiritualism, strength)
        };
        report.recovered_potential = recovered.0;
        self.set_strength(
            self.combat_properties
                .strength
                .wrapping_sub((f64::from(recovered.1) * 0.00001).round() as u32),
        );
        self.set_dexterity(
            self.combat_properties
                .dexterity
                .wrapping_sub((f64::from(recovered.2) * 0.00001).round() as u32),
        );
        self.set_maximum_hp(
            self.combat_properties
                .maximum_hp
                .wrapping_sub((f64::from(recovered.4) * 0.00001).round() as u32),
        );
        self.set_intelligence(
            self.combat_properties
                .intelligence
                .wrapping_sub((f64::from(recovered.3) * 0.00001).round() as u32),
        );
        report
            .effects
            .push(BattleFairyPotentialResetEffect::PropertiesChanged { player_id });
        let headgear = self
            .equipment
            .get_goods(10)
            .expect("reset не отделяет equipped headgear");
        report
            .effects
            .push(BattleFairyPotentialResetEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: headgear.identity(),
                    old_client_payload: encode_old_client(headgear),
                },
            ));
        report.outcome = BattleFairyPotentialResetOutcome::Reset;
        report
    }

    /// Полный player-side `CBattleFairyContainer::ResetSkill`. `consume_item`
    /// соответствует третьему native аргументу: script allocation передаёт
    /// ноль, прямой gameplay caller может потребовать `ZHJNS01/02`.
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
        let mut report = BattleFairySkillResetReport {
            player_id,
            position,
            outcome: BattleFairySkillResetOutcome::MissingHeadgear,
            previous_skill: None,
            selected_skill: None,
            detached_skill_ids: Vec::new(),
            attached_skill_ids: Vec::new(),
            effects: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySkillResetOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairySkillResetEffect::Notification {
                    player_id,
                    string_id: "ZHGS0008",
                    color: 0xffff_0000,
                });
            return report;
        }
        let Some(headgear) = self.equipment.get_goods(10) else {
            return report;
        };
        if headgear.addon_property_value(factory, GAP_BF_BATTLE_FAIRY, 1) != 1 {
            report.outcome = BattleFairySkillResetOutcome::InvalidHeadgear;
            return report;
        }

        if consume_item {
            let reset_name = match position {
                3..=5 => Some(b"ZHJNS01".as_slice()),
                6 => Some(b"ZHJNS02".as_slice()),
                _ => None,
            };
            if let Some(reset_name) = reset_name {
                let reset_index = factory.query_goods_id_by_original_name(Some(reset_name));
                let reset_item = self
                    .packet
                    .base()
                    .traversing_goods()
                    .find(|goods| goods.base_properties_index() == reset_index)
                    .map(|goods| (goods.identity(), goods.amount()));
                let Some((reset_identity, reset_amount)) = reset_item else {
                    report.outcome = BattleFairySkillResetOutcome::MissingResetItem;
                    report
                        .effects
                        .push(BattleFairySkillResetEffect::Notification {
                            player_id,
                            string_id: BATTLE_FAIRY_SKILL_RESET_ITEM_MISSING,
                            color: 0xffff_ffff,
                        });
                    return report;
                };
                let reset_position = self.packet.query_goods_position(reset_identity.ex_id);
                let (remaining_amount, consumed, removal) = if reset_amount == 0 {
                    (0, false, None)
                } else if reset_amount == 1 {
                    let removal = self.packet.remove_goods(reset_identity.ex_id);
                    (
                        if removal.is_some() { 0 } else { reset_amount },
                        removal.is_some(),
                        removal,
                    )
                } else {
                    let remaining = reset_amount.wrapping_sub(1);
                    let mut consumed = false;
                    if let Some(reset_position) = reset_position
                        && let Some(goods) = self.packet.get_goods_mut(reset_position)
                    {
                        goods.set_amount(remaining);
                        consumed = true;
                    }
                    (
                        if consumed { remaining } else { reset_amount },
                        consumed,
                        None,
                    )
                };
                report
                    .effects
                    .push(BattleFairySkillResetEffect::PacketItemConsumed {
                        player_id,
                        goods: reset_identity,
                        previous_amount: reset_amount,
                        remaining_amount,
                        consumed,
                        removal,
                    });
            }
        }

        let (current_skills, current_all_skill) = {
            let goods = self
                .equipment
                .get_goods(10)
                .expect("headgear остаётся equipped после reset-item consumption");
            (
                [
                    goods.addon_property_value(factory, GAP_BF_SKY_SKILL, 2) as u32,
                    goods.addon_property_value(factory, GAP_BF_EARTH_SKILL, 2) as u32,
                    goods.addon_property_value(factory, GAP_BF_MAN_SKILL, 2) as u32,
                ],
                goods.addon_property_value(factory, GAP_BF_ALL_SKILL, 2) as u32,
            )
        };
        let (property, previous_skill, replaced) = match position {
            3..=5 => {
                let replaced = (position - 3) as usize;
                (
                    GAP_BF_SKY_SKILL + replaced as i32,
                    current_skills[replaced],
                    Some(replaced),
                )
            }
            6 => (GAP_BF_ALL_SKILL, current_all_skill, None),
            _ => {
                report.outcome = BattleFairySkillResetOutcome::InvalidPosition;
                return report;
            }
        };
        report.previous_skill = Some(previous_skill);

        // В каждом native switch-case полный detach расположен перед первым
        // random(), а не только перед addon mutation.
        let old_entries = self.war_soul_skill_entries(factory);
        for (skill_id, _) in old_entries {
            if skill_id == 0 {
                continue;
            }
            let _deleted = self.move_shape.delete_skill(skill_id, skill_factory);
            report.detached_skill_ids.push(skill_id);
            // Native `DelWarSoulSkillInPlayer` вызывает TellClient после
            // DelSkill. Поэтому packet удаления существует лишь если skill
            // пережил отказ category lookup.
            if let Some(skill) = self.move_shape.skill(skill_id) {
                report
                    .effects
                    .push(BattleFairySkillResetEffect::SkillRemoved(
                        BattleFairySkillRemoved {
                            message_type: BATTLE_FAIRY_SKILL_REMOVED_MESSAGE_TYPE,
                            player_id,
                            skill_id,
                            skill_name: skill.name().to_vec(),
                        },
                    ));
            }
        }

        let selected_skill = match replaced {
            Some(replaced) => loop {
                let candidate = SKILL_POJIA.wrapping_add(random(13) as u32);
                if current_skills.contains(&candidate) {
                    continue;
                }
                let conflicts = unpaired_battle_fairy_skill(candidate).is_some_and(|paired| {
                    current_skills
                        .iter()
                        .enumerate()
                        .any(|(index, &skill)| index != replaced && skill == paired)
                });
                if !conflicts {
                    break candidate;
                }
            },
            None => loop {
                let candidate = SKILL_LEIMING.wrapping_add(random(3) as u32);
                if candidate != current_all_skill {
                    break candidate;
                }
            },
        };
        report.selected_skill = Some(selected_skill);

        {
            let goods = self
                .equipment
                .get_goods_mut(10)
                .expect("skill detach не отделяет equipped headgear");
            let _level_cleared = goods.set_addon_property_value_core(property, 1, 0);
            let _skill_cleared = goods.set_addon_property_value_core(property, 2, 0);
            let _level_stored = goods.set_addon_property_value_core(property, 1, 1);
            let _skill_stored =
                goods.set_addon_property_value_core(property, 2, selected_skill as i32);
        }

        let new_entries = self.war_soul_skill_entries(factory);
        for (skill_id, level) in new_entries {
            let _added = self.move_shape.add_skill(skill_id, level, skill_factory);
            if let Some(skill) = self.move_shape.skill(skill_id) {
                report.attached_skill_ids.push(skill_id);
                report.effects.push(BattleFairySkillResetEffect::SkillAdded(
                    battle_fairy_skill_snapshot(player_id, skill),
                ));
            }
        }

        let Some(selected) = self.move_shape.skill(selected_skill) else {
            report.outcome = BattleFairySkillResetOutcome::SelectedSkillUnavailable;
            return report;
        };
        report
            .effects
            .push(BattleFairySkillResetEffect::SelectedSkillLearned(
                battle_fairy_skill_snapshot(player_id, selected),
            ));
        let headgear = self
            .equipment
            .get_goods(10)
            .expect("ResetSkill не отделяет equipped headgear");
        report
            .effects
            .push(BattleFairySkillResetEffect::GoodsUpdated(
                BattleFairyDefaultGoodsUpdate {
                    message_type: 0x0b_f918,
                    player_id,
                    goods: headgear.identity(),
                    old_client_payload: encode_old_client(headgear),
                },
            ));
        report.outcome = BattleFairySkillResetOutcome::Reset;
        report
    }

    fn war_soul_skill_entries(&self, factory: &CGoodsFactory) -> [(u32, i32); 9] {
        let Some(goods) = self.equipment.get_goods(10) else {
            return [(0, 0); 9];
        };
        war_soul_skill_entries_from_goods(goods, factory)
    }

    /// Полный player-side `skillmessage` opcode `0x90005` после успешного
    /// packet decode. Contend notification намеренно не блокирует запрос.
    pub(crate) fn request_battle_fairy_skill(
        &self,
        battle_fairy_enabled: bool,
        request: BattleFairySkillRequest,
        facts: BattleFairySkillRequestFacts,
        goods_factory: &CGoodsFactory,
        skill_factory: &CSkillFactory,
    ) -> BattleFairySkillRequestReport {
        let player_id = self.player_id();
        let skill_id = request.skill_id();
        let mut report = BattleFairySkillRequestReport {
            player_id,
            skill_id,
            skill_level: 0,
            target_type: request.target_type,
            target_id: request.target_id,
            target_x: request.target_x,
            target_y: request.target_y,
            outcome: BattleFairySkillRequestOutcome::MissingHeadgear,
            effects: Vec::new(),
        };
        if !battle_fairy_enabled {
            report.outcome = BattleFairySkillRequestOutcome::FeatureDisabled;
            report
                .effects
                .push(BattleFairySkillRequestEffect::Notification {
                    player_id,
                    string_id: "ZHGS0037",
                    color: 0xffff_0000,
                    message_type: 0,
                });
            return report;
        }
        let Some(goods) = self.equipment.get_goods(10) else {
            return report;
        };
        if goods.addon_property_value(goods_factory, GAP_BF_HP, 1) == 0 {
            report.outcome = BattleFairySkillRequestOutcome::NoHitPoints;
            return report;
        }
        if self.contend_state && facts.symbol_attackable {
            report
                .effects
                .push(BattleFairySkillRequestEffect::Notification {
                    player_id,
                    string_id: "ZHGS0038",
                    color: 0xffff_ffff,
                    message_type: 0xffff_0000,
                });
        }

        let skill_level =
            check_battle_fairy_skill(goods, goods_factory, request.property_offset, skill_id);
        report.skill_level = skill_level;
        if skill_level == 0 {
            report.outcome = BattleFairySkillRequestOutcome::Unauthorized;
            push_battle_fairy_skill_reject(&mut report);
            return report;
        }

        if skill_factory
            .query_skill_base_properties(skill_id, skill_level)
            .is_some_and(|properties| properties.is_target_self() != 0)
        {
            let (target_x, target_y) = match (self.shape().get_tile_x(), self.shape().get_tile_y())
            {
                (Ok(x), Ok(y)) => (x, y),
                (Err(error), _) | (_, Err(error)) => {
                    report.outcome = BattleFairySkillRequestOutcome::CoordinateBlocked(error);
                    return report;
                }
            };
            report.target_type = self.shape().identity().object_type;
            report.target_id = player_id;
            report.target_x = target_x;
            report.target_y = target_y;
        }
        if !facts.player_ai_available {
            report.outcome = BattleFairySkillRequestOutcome::AiUnavailable;
            return report;
        }

        let dispatch = if report.target_type == 0 || report.target_id == 0 {
            if report.target_x == 0 || report.target_y == 0 {
                BattleFairySkillDispatch::SelfTarget {
                    skill_id,
                    player_id,
                }
            } else {
                BattleFairySkillDispatch::Point {
                    skill_id,
                    x: report.target_x,
                    y: report.target_y,
                }
            }
        } else {
            if self.server_region_id.is_none() {
                report.outcome = BattleFairySkillRequestOutcome::MissingRegion;
                return report;
            }
            let target = ShapeIdentity {
                object_type: report.target_type,
                id: report.target_id,
                ex_id: CGuid::GUID_INVALID,
            };
            if !facts.object_target_available {
                report.outcome = BattleFairySkillRequestOutcome::MissingTarget;
                push_battle_fairy_skill_reject(&mut report);
                return report;
            }
            BattleFairySkillDispatch::Object { skill_id, target }
        };
        report
            .effects
            .push(BattleFairySkillRequestEffect::AiDispatch(dispatch));
        report.outcome = BattleFairySkillRequestOutcome::Queued;
        report
    }

    /// Достигнутая часть exact `RefreshContainerOwners`: owner ID должен быть
    /// перепривязан после создания player identity или его восстановления.
    pub(crate) const fn refresh_reached_container_owners(&mut self, player_id: i32) {
        self.bank.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.depot
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
        self.packet.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.equipment.base_mut().set_owner(PLAYER_TYPE, player_id);
        self.battle_fairy_container
            .base_mut()
            .base_mut()
            .base_mut()
            .set_owner(PLAYER_TYPE, player_id);
    }

    pub(crate) const fn set_pk_count(&mut self, value: u16) {
        self.base_properties.pk_count = value;
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

    pub(crate) const fn health(&self) -> u32 {
        self.base_properties.health
    }

    pub(crate) const fn maximum_health(&self) -> u32 {
        self.combat_properties.maximum_hp
    }

    pub(crate) const fn maximum_mana(&self) -> u32 {
        self.combat_properties.maximum_mp
    }

    /// Owned scalar tail `OnRelive` после external passive/enter/update hooks.
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

    pub(crate) const fn mana(&self) -> u32 {
        self.base_properties.mana
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

    pub(crate) const fn set_battle_fairy_recall(&mut self, value: bool) {
        self.base_properties.battle_fairy_recall = value;
    }

    pub(crate) const fn set_battle_fairy_died(&mut self, value: bool) {
        self.base_properties.battle_fairy_died = value;
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
    /// Transport получает уже ordered report, не подменяя неизвестные поля
    /// исторических packet-encoder-ов выдуманными нулями.
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
            removed_inputs: Vec::with_capacity(3),
            effects: Vec::new(),
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
            Ok(recipe) => recipe,
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
                    old_client_payload: None,
                },
            ));
            report.removed_inputs.push(removed);
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
                if !move_shape.add_skill(skill.id, skill.level, skill_factory) {
                    return false;
                }
                let stored = move_shape
                    .skill(skill.id)
                    .expect("успешный AddSkill публикует найденный skill");
                skill_effects.push(BattleFairyCombineEffect::SkillAdded(
                    BattleFairySkillAdded {
                        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
                        player_id,
                        skill_id: stored.id(),
                        skill_level: stored.level(),
                        skill_type: stored.skill_type(),
                        skill_name: stored.name().to_vec(),
                    },
                ));
                true
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
            default_properties.goods_update,
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
            figure: self.figure,
        })
    }
}

fn push_battle_fairy_summon_notification(
    report: &mut BattleFairySummonReport,
    string_id: &'static str,
    color: u32,
) {
    report.effects.push(BattleFairySummonEffect::Notification {
        player_id: report.player_id,
        string_id,
        color,
    });
}

fn push_battle_fairy_upgrade_notification(
    report: &mut BattleFairyUpgradeReport,
    string_id: &'static str,
    format_value: Option<u32>,
) {
    report.effects.push(BattleFairyUpgradeEffect::Notification {
        player_id: report.player_id,
        string_id,
        color: 0xffff_ffff,
        format_value,
    });
}

fn battle_fairy_skill_snapshot(player_id: i32, skill: &MoveShapeSkill) -> BattleFairySkillAdded {
    BattleFairySkillAdded {
        message_type: BATTLE_FAIRY_SKILL_ADDED_MESSAGE_TYPE,
        player_id,
        skill_id: skill.id(),
        skill_level: skill.level(),
        skill_type: skill.skill_type(),
        skill_name: skill.name().to_vec(),
    }
}

fn war_soul_skill_entries_from_goods(goods: &CGoods, factory: &CGoodsFactory) -> [(u32, i32); 9] {
    let entry = |property| {
        (
            goods.addon_property_value(factory, property, 2) as u32,
            goods.addon_property_value(factory, property, 1),
        )
    };
    let mut entries = [
        entry(GAP_BF_SKY),
        entry(GAP_BF_EARTH),
        entry(GAP_BF_MAN),
        entry(GAP_BF_SKY_SKILL),
        entry(GAP_BF_EARTH_SKILL),
        entry(GAP_BF_MAN_SKILL),
        entry(GAP_BF_ALL_SKILL),
        entry(GAP_BF_HUOXIESHU_SKILL),
        entry(GAP_BF_LINGZHISHU_SKILL),
    ];
    entries[7].1 = 1;
    entries[8].1 = 1;
    entries
}

fn check_battle_fairy_skill(
    goods: &CGoods,
    factory: &CGoodsFactory,
    property_offset: i32,
    requested_skill: u32,
) -> i32 {
    if property_offset == 0 {
        return 1;
    }
    let property = GAP_BF_MAN.wrapping_add(property_offset);
    if goods.addon_property_value(factory, property, 2) as u32 == requested_skill {
        return goods.addon_property_value(factory, property, 1);
    }
    if goods.addon_property_value(factory, GAP_BF_HUOXIESHU_SKILL, 2) == 0x222
        || goods.addon_property_value(factory, GAP_BF_LINGZHISHU_SKILL, 2) == 0x223
    {
        return 1;
    }
    0
}

fn push_battle_fairy_skill_reject(report: &mut BattleFairySkillRequestReport) {
    report
        .effects
        .push(BattleFairySkillRequestEffect::SocketReject {
            message_type: SKILL_EFFECT_MESSAGE_TYPE,
            reason: SKILL_REJECT_WAR_SOUL_REASON,
            code: SKILL_REJECT_CODE,
        });
}

fn legacy_f32_to_u32(value: f32) -> u32 {
    (value.trunc() as i64) as u32
}

/// Exact constructor map `m_UnPairSkills`, подтверждённый immediate-ами
/// `gameserver.exe` по адресу `0x00504052..0x00504149`.
const fn unpaired_battle_fairy_skill(skill_id: u32) -> Option<u32> {
    Some(match skill_id {
        530 => 534,
        531 => 535,
        532 => 536,
        533 => 537,
        534 => 530,
        535 => 531,
        536 => 532,
        537 => 533,
        _ => return None,
    })
}

const fn is_battle_fairy_property_cell(cell: BattleFairyCell) -> bool {
    matches!(
        cell,
        BattleFairyCell::Weapon
            | BattleFairyCell::Body
            | BattleFairyCell::Huxinjing
            | BattleFairyCell::Jewelry
            | BattleFairyCell::Glove
            | BattleFairyCell::Pifeng
            | BattleFairyCell::Yaodai
            | BattleFairyCell::Xiezi
    )
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

fn add_battle_fairy_u32(current: u32, delta: f64) -> u32 {
    let next = f64::from(current) + delta;
    if next < 0.0 {
        0
    } else {
        clamp_combat_scalar(next.round() as u32)
    }
}

fn add_battle_fairy_u16(current: u16, delta: f64) -> u16 {
    let next = f64::from(current) + delta;
    if next < 0.0 { 0 } else { next.round() as u16 }
}

fn add_battle_fairy_i32(current: i32, delta: f64) -> i32 {
    let next = f64::from(current) + delta;
    if next < 0.0 { 0 } else { next.round() as i32 }
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
// FUNCTION: CPlayer::GetAccount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:371
// RVA: 0x00002860
// ADDRESS: 00402860
// PROTOTYPE: char * __thiscall GetAccount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetPkCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:304
// RVA: 0x0001E580
// ADDRESS: 0041e580
// PROTOTYPE: void __thiscall SetPkCount(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCiQingOpenFun
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:250
// RVA: 0x0002ACD0
// ADDRESS: 0042acd0
// PROTOTYPE: void __thiscall SetCiQingOpenFun(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:289
// RVA: 0x0002ACE0
// ADDRESS: 0042ace0
// PROTOTYPE: void __thiscall SetExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:390
// RVA: 0x0002ACF0
// ADDRESS: 0042acf0
// PROTOTYPE: void __thiscall SetMaxHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:392
// RVA: 0x0002AD10
// ADDRESS: 0042ad10
// PROTOTYPE: void __thiscall SetMaxMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:398
// RVA: 0x0002AD30
// ADDRESS: 0042ad30
// PROTOTYPE: void __thiscall SetStr(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:400
// RVA: 0x0002AD50
// ADDRESS: 0042ad50
// PROTOTYPE: void __thiscall SetDex(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:402
// RVA: 0x0002AD70
// ADDRESS: 0042ad70
// PROTOTYPE: void __thiscall SetCon(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetInt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:404
// RVA: 0x0002AD90
// ADDRESS: 0042ad90
// PROTOTYPE: void __thiscall SetInt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:406
// RVA: 0x0002ADB0
// ADDRESS: 0042adb0
// PROTOTYPE: void __thiscall SetMinAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:408
// RVA: 0x0002ADD0
// ADDRESS: 0042add0
// PROTOTYPE: void __thiscall SetMaxAtk(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:416
// RVA: 0x0002ADF0
// ADDRESS: 0042adf0
// PROTOTYPE: void __thiscall SetDef(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:422
// RVA: 0x0002AE10
// ADDRESS: 0042ae10
// PROTOTYPE: void __thiscall SetElementResistant(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:460
// RVA: 0x0002AE30
// ADDRESS: 0042ae30
// PROTOTYPE: void __thiscall SetBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFullMissScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:466
// RVA: 0x0002AE60
// ADDRESS: 0042ae60
// PROTOTYPE: void __thiscall SetFullMissScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCriticalRate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:471
// RVA: 0x0002AE90
// ADDRESS: 0042ae90
// PROTOTYPE: void __thiscall SetCriticalRate(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetContribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:546
// RVA: 0x0002AEC0
// ADDRESS: 0042aec0
// PROTOTYPE: void __thiscall SetContribute(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNextExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:570
// RVA: 0x0002AF00
// ADDRESS: 0042af00
// PROTOTYPE: ulong __thiscall GetNextExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFetchPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:680
// RVA: 0x0002AF20
// ADDRESS: 0042af20
// PROTOTYPE: void __thiscall SetFetchPower(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetPersonalShopFlag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:507
// RVA: 0x0002AF60
// ADDRESS: 0042af60
// PROTOTYPE: void __thiscall SetPersonalShopFlag(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDefaultAttackSkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:562
// RVA: 0x0002AFD0
// ADDRESS: 0042afd0
// PROTOTYPE: tagSkillID __thiscall GetDefaultAttackSkillID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDecreaseMurdererSign
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2941
// RVA: 0x0002B030
// ADDRESS: 0042b030
// PROTOTYPE: void __thiscall OnDecreaseMurdererSign(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnUpdateMurdererSign
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3037
// RVA: 0x0002B130
// ADDRESS: 0042b130
// PROTOTYPE: void __thiscall OnUpdateMurdererSign(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsBadman
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3089
// RVA: 0x0002B160
// ADDRESS: 0042b160
// PROTOTYPE: bool __thiscall IsBadman(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInArea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5078
// RVA: 0x0002B190
// ADDRESS: 0042b190
// PROTOTYPE: bool __thiscall IsInArea(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsInRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5123
// RVA: 0x0002B230
// ADDRESS: 0042b230
// PROTOTYPE: bool __thiscall IsInRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ActiveEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6432
// RVA: 0x0002B240
// ADDRESS: 0042b240
// PROTOTYPE: void __thiscall ActiveEquip(CGoods * param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountFuMoProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7230
// RVA: 0x0002B690
// ADDRESS: 0042b690
// PROTOTYPE: int __thiscall MountFuMoProperty(GOODS_ADDON_PROPERTIES param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurBurden
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8564
// RVA: 0x0002C2D0
// ADDRESS: 0042c2d0
// PROTOTYPE: long __thiscall GetCurBurden(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8582
// RVA: 0x0002C310
// ADDRESS: 0042c310
// PROTOTYPE: long __thiscall CanMountEquip(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanUseItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8650
// RVA: 0x0002C490
// ADDRESS: 0042c490
// PROTOTYPE: long __thiscall CanUseItem(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecodeSkillsFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9096
// RVA: 0x0002C5C0
// ADDRESS: 0042c5c0
// PROTOTYPE: void __thiscall DecodeSkillsFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9195
// RVA: 0x0002C620
// ADDRESS: 0042c620
// PROTOTYPE: void __thiscall OnChangeProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED, VERIFIED_DISASSEMBLY: `SetSilence/IsInSilence`
// RVA `0x0002C8A0/0x0002C8F0` материализованы выше и достигнуты GM
// `0x7FC0B/0x7FC0E`; покрытый raw удалён.
// ============================================================================
// FUNCTION: CPlayer::UpdateCurrentState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9313
// RVA: 0x0002C940
// ADDRESS: 0042c940
// PROTOTYPE: void __thiscall UpdateCurrentState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterCriminalState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9336
// RVA: 0x0002C9A0
// ADDRESS: 0042c9a0
// PROTOTYPE: void __thiscall EnterCriminalState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterResidentState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9351
// RVA: 0x0002CA40
// ADDRESS: 0042ca40
// PROTOTYPE: void __thiscall EnterResidentState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterPeaceState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9361
// RVA: 0x0002CAC0
// ADDRESS: 0042cac0
// PROTOTYPE: void __thiscall EnterPeaceState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::EnterCombatState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9373
// RVA: 0x0002CB50
// ADDRESS: 0042cb50
// PROTOTYPE: void __thiscall EnterCombatState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGoodsById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9478
// RVA: 0x0002CC00
// ADDRESS: 0042cc00
// PROTOTYPE: CGoods * __thiscall GetGoodsById(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGoodsById_FromPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9501
// RVA: 0x0002CC70
// ADDRESS: 0042cc70
// PROTOTYPE: CGoods * __thiscall GetGoodsById_FromPackage(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeginSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9513
// RVA: 0x0002CC90
// ADDRESS: 0042cc90
// PROTOTYPE: int __thiscall OnBeginSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendNotifyMessageA
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9558
// RVA: 0x0002CCD0
// ADDRESS: 0042ccd0
// PROTOTYPE: void __thiscall SendNotifyMessageA(char * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendSystemInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9570
// RVA: 0x0002CD70
// ADDRESS: 0042cd70
// PROTOTYPE: void __thiscall SendSystemInfo(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendOtherInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9581
// RVA: 0x0002CE00
// ADDRESS: 0042ce00
// PROTOTYPE: void __thiscall SendOtherInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CanMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9644
// RVA: 0x0002CE80
// ADDRESS: 0042ce80
// PROTOTYPE: int __thiscall CanMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnCannotMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9651
// RVA: 0x0002CEA0
// ADDRESS: 0042cea0
// PROTOTYPE: void __thiscall OnCannotMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10030
// RVA: 0x0002CF40
// ADDRESS: 0042cf40
// PROTOTYPE: ulong __thiscall GetMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10035
// RVA: 0x0002CF50
// ADDRESS: 0042cf50
// PROTOTYPE: ulong __thiscall GetYuanBao(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDepotMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10045
// RVA: 0x0002CF60
// ADDRESS: 0042cf60
// PROTOTYPE: ulong __thiscall GetDepotMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10050
// RVA: 0x0002CF70
// ADDRESS: 0042cf70
// PROTOTYPE: int __thiscall SetMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetYuanBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10094
// RVA: 0x0002D120
// ADDRESS: 0042d120
// PROTOTYPE: int __thiscall SetYuanBao(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10224
// RVA: 0x0002D2D0
// ADDRESS: 0042d2d0
// PROTOTYPE: undefined __thiscall CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::~CPacketListener
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10229
// RVA: 0x0002D2E0
// ADDRESS: 0042d2e0
// PROTOTYPE: void __thiscall ~CPacketListener(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPacketListener::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10234
// RVA: 0x0002D2F0
// ADDRESS: 0042d2f0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10285
// RVA: 0x0002D3F0
// ADDRESS: 0042d3f0
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10309
// RVA: 0x0002D430
// ADDRESS: 0042d430
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10327
// RVA: 0x0002D450
// ADDRESS: 0042d450
// PROTOTYPE: void __thiscall RejectUseSkillRequest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10892
// RVA: 0x0002D4D0
// ADDRESS: 0042d4d0
// PROTOTYPE: void __thiscall SetLevel(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PerformEmotion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11059
// RVA: 0x0002D590
// ADDRESS: 0042d590
// PROTOTYPE: void __thiscall PerformEmotion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ClearEmotion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11083
// RVA: 0x0002D680
// ADDRESS: 0042d680
// PROTOTYPE: void __thiscall ClearEmotion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainerListener::OnObjectRemoved
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11134
// RVA: 0x0002D720
// ADDRESS: 0042d720
// PROTOTYPE: int __thiscall OnObjectRemoved(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsFactionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11200
// RVA: 0x0002D930
// ADDRESS: 0042d930
// PROTOTYPE: bool __thiscall IsFactionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsUnionMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11211
// RVA: 0x0002D950
// ADDRESS: 0042d950
// PROTOTYPE: bool __thiscall IsUnionMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11410
// RVA: 0x0002D980
// ADDRESS: 0042d980
// PROTOTYPE: float __thiscall GetWeaponModifier(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetWeaponDamageLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11439
// RVA: 0x0002D9F0
// ADDRESS: 0042d9f0
// PROTOTYPE: ulong __thiscall GetWeaponDamageLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::QuestTimeBegin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11486
// RVA: 0x0002DA20
// ADDRESS: 0042da20
// PROTOTYPE: void __thiscall QuestTimeBegin(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::QuestTimeClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11497
// RVA: 0x0002DAC0
// ADDRESS: 0042dac0
// PROTOTYPE: void __thiscall QuestTimeClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetQuestOn
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11506
// RVA: 0x0002DB40
// ADDRESS: 0042db40
// PROTOTYPE: void __thiscall SetQuestOn(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::end_business
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12095
// RVA: 0x0002DBF0
// ADDRESS: 0042dbf0
// PROTOTYPE: void __thiscall end_business(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSessionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12173
// RVA: 0x0002DCC0
// ADDRESS: 0042dcc0
// PROTOTYPE: char * __thiscall GetSessionID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetIpAddress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12179
// RVA: 0x0002DCD0
// ADDRESS: 0042dcd0
// PROTOTYPE: char * __thiscall GetIpAddress(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12861
// RVA: 0x0002DD20
// ADDRESS: 0042dd20
// PROTOTYPE: int __thiscall DeleteSkillItem(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetWarSoulXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13071
// RVA: 0x0002DF50
// ADDRESS: 0042df50
// PROTOTYPE: void __thiscall SetWarSoulXY(tagPOINT param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13109
// RVA: 0x0002E0A0
// ADDRESS: 0042e0a0
// PROTOTYPE: void __thiscall DelWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReplacePlayerData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13276
// RVA: 0x0002E260
// ADDRESS: 0042e260
// PROTOTYPE: void __thiscall ReplacePlayerData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestorePlayerData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13300
// RVA: 0x0002E400
// ADDRESS: 0042e400
// PROTOTYPE: void __thiscall RestorePlayerData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClientMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13315
// RVA: 0x0002E4D0
// ADDRESS: 0042e4d0
// PROTOTYPE: void __thiscall TellClientMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13324
// RVA: 0x0002E570
// ADDRESS: 0042e570
// PROTOTYPE: void __thiscall TellClient(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RejectUseSkillRequestWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13358
// RVA: 0x0002E720
// ADDRESS: 0042e720
// PROTOTYPE: void __thiscall RejectUseSkillRequestWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::BuyItemFromAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13554
// RVA: 0x0002E820
// ADDRESS: 0042e820
// PROTOTYPE: void __thiscall BuyItemFromAuction(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13619
// RVA: 0x0002E8B0
// ADDRESS: 0042e8b0
// PROTOTYPE: bool __thiscall IsMoney(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsGoodAllowedInAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13682
// RVA: 0x0002EA50
// ADDRESS: 0042ea50
// PROTOTYPE: bool __thiscall IsGoodAllowedInAuction(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCurAucNodeOK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13941
// RVA: 0x0002EA80
// ADDRESS: 0042ea80
// PROTOTYPE: bool __thiscall IsCurAucNodeOK(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCurAucBuyNodeOK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13948
// RVA: 0x0002EB30
// ADDRESS: 0042eb30
// PROTOTYPE: bool __thiscall IsCurAucBuyNodeOK(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanCurAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13970
// RVA: 0x0002EBE0
// ADDRESS: 0042ebe0
// PROTOTYPE: void __thiscall CleanCurAucNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetOptMoneyYuan
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14536
// RVA: 0x0002EC10
// ADDRESS: 0042ec10
// PROTOTYPE: bool __thiscall GetOptMoneyYuan(long * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetOptMoneyJin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14575
// RVA: 0x0002ECF0
// ADDRESS: 0042ecf0
// PROTOTYPE: bool __thiscall GetOptMoneyJin(long * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::QuerySellerGoodsSelf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14715
// RVA: 0x0002EE80
// ADDRESS: 0042ee80
// PROTOTYPE: void __thiscall QuerySellerGoodsSelf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14782
// RVA: 0x0002EEF0
// ADDRESS: 0042eef0
// PROTOTYPE: ulong __thiscall GetAuctionMoney(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AuctionLimit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14998
// RVA: 0x0002EF00
// ADDRESS: 0042ef00
// PROTOTYPE: bool __thiscall AuctionLimit(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendSaleLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15094
// RVA: 0x0002EF60
// ADDRESS: 0042ef60
// PROTOTYPE: void __thiscall SendSaleLog(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCutLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15115
// RVA: 0x0002F0A0
// ADDRESS: 0042f0a0
// PROTOTYPE: void __thiscall SendCutLog(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::NoticyWS_BaiTan_Over
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15135
// RVA: 0x0002F140
// ADDRESS: 0042f140
// PROTOTYPE: void __thiscall NoticyWS_BaiTan_Over(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendToGSBaiTan
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15143
// RVA: 0x0002F1C0
// ADDRESS: 0042f1c0
// PROTOTYPE: void __thiscall SendToGSBaiTan(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CountScoreAdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15155
// RVA: 0x0002F250
// ADDRESS: 0042f250
// PROTOTYPE: int __cdecl CountScoreAdd(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JJcWeekClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15176
// RVA: 0x0002F2D0
// ADDRESS: 0042f2d0
// PROTOTYPE: void __thiscall JJcWeekClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JJcSeasonClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15190
// RVA: 0x0002F320
// ADDRESS: 0042f320
// PROTOTYPE: void __thiscall JJcSeasonClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddPreItemToPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15373
// RVA: 0x0002F360
// ADDRESS: 0042f360
// PROTOTYPE: void __thiscall AddPreItemToPlayer(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddGoodsToCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16045
// RVA: 0x0002F920
// ADDRESS: 0042f920
// PROTOTYPE: bool __thiscall AddGoodsToCiQing(CGoods * param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteCiQingLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16333
// RVA: 0x0002FAA0
// ADDRESS: 0042faa0
// PROTOTYPE: void __thiscall WriteCiQingLog(ulong param_1, ulong param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17480
// RVA: 0x0002FB50
// ADDRESS: 0042fb50
// PROTOTYPE: void __thiscall SetCurFlash(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneFlash
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17528
// RVA: 0x0002FC50
// ADDRESS: 0042fc50
// PROTOTYPE: void __thiscall DoneFlash(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddLTUp60Cnt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17679
// RVA: 0x0002FD10
// ADDRESS: 0042fd10
// PROTOTYPE: void __thiscall AddLTUp60Cnt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:368
// RVA: 0x000300D0
// ADDRESS: 004300d0
// PROTOTYPE: void __thiscall SetMP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetRP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:370
// RVA: 0x000300F0
// ADDRESS: 004300f0
// PROTOTYPE: void __thiscall SetRP(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetVigour
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:518
// RVA: 0x00030120
// ADDRESS: 00430120
// PROTOTYPE: void __thiscall SetVigour(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PeriodicalUpdate
// STATUS: PARTIALLY_IMPLEMENTED_DEATH_STATE_TAIL
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2348
// RVA: 0x00030140
// ADDRESS: 00430140
// PROTOTYPE: void __thiscall PeriodicalUpdate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseRp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9417
// RVA: 0x000302F0
// ADDRESS: 004302f0
// PROTOTYPE: void __thiscall IncreaseRp(int param_1, ushort param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteGoodsDelLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12633
// RVA: 0x00030430
// ADDRESS: 00430430
// PROTOTYPE: void __thiscall WriteGoodsDelLog(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::BuyItemFromAauction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13826
// RVA: 0x00030BC0
// ADDRESS: 00430bc0
// PROTOTYPE: bool __thiscall BuyItemFromAauction(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsDonePreNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13931
// RVA: 0x00030CF0
// ADDRESS: 00430cf0
// PROTOTYPE: bool __thiscall IsDonePreNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCurAucBuyNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13977
// RVA: 0x00030D00
// ADDRESS: 00430d00
// PROTOTYPE: bool __thiscall SetCurAucBuyNode(CGoodsNode param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReFlushSelfGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14723
// RVA: 0x00030D90
// ADDRESS: 00430d90
// PROTOTYPE: void __thiscall ReFlushSelfGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetAuctionMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14737
// RVA: 0x00030E60
// ADDRESS: 00430e60
// PROTOTYPE: bool __thiscall SetAuctionMoney(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsGM
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5044
// RVA: 0x00031430
// ADDRESS: 00431430
// PROTOTYPE: bool __thiscall IsGM(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetGMLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5052
// RVA: 0x00031480
// ADDRESS: 00431480
// PROTOTYPE: long __thiscall GetGMLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10594
// RVA: 0x000314E0
// ADDRESS: 004314e0
// PROTOTYPE: ulong __thiscall DeleteGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2, ulong param_3, bool param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsbyGuid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12330
// RVA: 0x00031800
// ADDRESS: 00431800
// PROTOTYPE: int __thiscall DeleteGoodsbyGuid(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendAucAbBuyOptTran
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14493
// RVA: 0x00031900
// ADDRESS: 00431900
// PROTOTYPE: void __thiscall SendAucAbBuyOptTran(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15294
// RVA: 0x00031DE0
// ADDRESS: 00431de0
// PROTOTYPE: void __thiscall AddTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15351
// RVA: 0x00031E70
// ADDRESS: 00431e70
// PROTOTYPE: void __thiscall AddTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddCiQingTaoZhuangPre
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15362
// RVA: 0x00031EE0
// ADDRESS: 00431ee0
// PROTOTYPE: void __thiscall AddCiQingTaoZhuangPre(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteArrayLeiTing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17554
// RVA: 0x00031F50
// ADDRESS: 00431f50
// PROTOTYPE: void __thiscall AddByteArrayLeiTing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetOneThing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17635
// RVA: 0x00032000
// ADDRESS: 00432000
// PROTOTYPE: tagThing * __thiscall GetOneThing(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:272
// RVA: 0x00032630
// ADDRESS: 00432630
// PROTOTYPE: int __thiscall DropGoods(PLAYER_EXTEND_ID param_1, CGUID * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::check_item_in_packet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:388
// RVA: 0x00032900
// ADDRESS: 00432900
// PROTOTYPE: uint __thiscall check_item_in_packet(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::remove_item_in_packet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:408
// RVA: 0x000329E0
// ADDRESS: 004329e0
// PROTOTYPE: uint __thiscall remove_item_in_packet(int param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNumSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8968
// RVA: 0x00032AA0
// ADDRESS: 00432aa0
// PROTOTYPE: long __thiscall GetNumSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddSkillsToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8998
// RVA: 0x00032B80
// ADDRESS: 00432b80
// PROTOTYPE: void __thiscall AddSkillsToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnChangeStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9234
// RVA: 0x00033080
// ADDRESS: 00433080
// PROTOTYPE: void __thiscall OnChangeStates(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsEnemyFactionMember
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9397
// RVA: 0x00033210
// ADDRESS: 00433210
// PROTOTYPE: long __thiscall IsEnemyFactionMember(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsCityWarEneymyFactionMemeber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9407
// RVA: 0x00033240
// ADDRESS: 00433240
// PROTOTYPE: long __thiscall IsCityWarEneymyFactionMemeber(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsInPacket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10857
// RVA: 0x00033270
// ADDRESS: 00433270
// PROTOTYPE: void __thiscall DeleteGoodsInPacket(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11226
// RVA: 0x00033310
// ADDRESS: 00433310
// PROTOTYPE: bool __thiscall AddQuestDataByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetQuestState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11293
// RVA: 0x000333B0
// ADDRESS: 004333b0
// PROTOTYPE: long __thiscall GetQuestState(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetValidQuestNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11301
// RVA: 0x000333F0
// ADDRESS: 004333f0
// PROTOTYPE: long __thiscall GetValidQuestNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CompleteQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11352
// RVA: 0x00033480
// ADDRESS: 00433480
// PROTOTYPE: void __thiscall CompleteQuest(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11383
// RVA: 0x00033550
// ADDRESS: 00433550
// PROTOTYPE: void __thiscall UpdateQuest(ushort param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ReUseSkillItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12832
// RVA: 0x00033610
// ADDRESS: 00433610
// PROTOTYPE: int __thiscall ReUseSkillItem(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::JudgeZhaoMuStatus
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13478
// RVA: 0x000336B0
// ADDRESS: 004336b0
// PROTOTYPE: bool __thiscall JudgeZhaoMuStatus(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanPreAndSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15265
// RVA: 0x00033700
// ADDRESS: 00433700
// PROTOTYPE: void __thiscall CleanPreAndSkillList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelTaoZhuangSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15273
// RVA: 0x000337A0
// ADDRESS: 004337a0
// PROTOTYPE: void __thiscall DelTaoZhuangSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15841
// RVA: 0x00033840
// ADDRESS: 00433840
// PROTOTYPE: void __thiscall AddByteCiQing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountCiQingFromHand
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16180
// RVA: 0x000338D0
// ADDRESS: 004338d0
// PROTOTYPE: bool __thiscall MountCiQingFromHand(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddOrgSysToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1518
// RVA: 0x00033C30
// ADDRESS: 00433c30
// PROTOTYPE: bool __thiscall AddOrgSysToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteGS2WS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13511
// RVA: 0x00033E40
// ADDRESS: 00433e40
// PROTOTYPE: void __thiscall AddByteGS2WS(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateLeiTingToWSandClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17701
// RVA: 0x00033ED0
// ADDRESS: 00433ed0
// PROTOTYPE: void __thiscall UpdateLeiTingToWSandClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10335
// RVA: 0x00034300
// ADDRESS: 00434300
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetTileXY
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10872
// RVA: 0x00034930
// ADDRESS: 00434930
// PROTOTYPE: void __thiscall SetTileXY(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::do_coutribute
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11514
// RVA: 0x00034A00
// ADDRESS: 00434a00
// PROTOTYPE: void __thiscall do_coutribute(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddExploitToMurdererInCountryWar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12038
// RVA: 0x00035DF0
// ADDRESS: 00435df0
// PROTOTYPE: void __thiscall AddExploitToMurdererInCountryWar(CServerRegion * param_1, CPlayer * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DrawAwards
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12205
// RVA: 0x00035F70
// ADDRESS: 00435f70
// PROTOTYPE: long __thiscall DrawAwards(long param_1, int param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeBodyCheck
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12918
// RVA: 0x00036080
// ADDRESS: 00436080
// PROTOTYPE: int __thiscall ChangeBodyCheck(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IsAollowAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13583
// RVA: 0x00036200
// ADDRESS: 00436200
// PROTOTYPE: bool __thiscall IsAollowAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::WriteBuyAuctionLog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14612
// RVA: 0x00036370
// ADDRESS: 00436370
// PROTOTYPE: void __thiscall WriteBuyAuctionLog(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OpenAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14788
// RVA: 0x000367B0
// ADDRESS: 004367b0
// PROTOTYPE: bool __thiscall OpenAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAuctionMoneyMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14846
// RVA: 0x000369B0
// ADDRESS: 004369b0
// PROTOTYPE: bool __thiscall CheckAuctionMoneyMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoodsFromCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16251
// RVA: 0x00036AA0
// ADDRESS: 00436aa0
// PROTOTYPE: void __thiscall DeleteGoodsFromCiQing(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddThingCnt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17649
// RVA: 0x00036BD0
// ADDRESS: 00436bd0
// PROTOTYPE: bool __thiscall AddThingCnt(int param_1, int param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RemoveQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11367
// RVA: 0x00038DC0
// ADDRESS: 00438dc0
// PROTOTYPE: void __thiscall RemoveQuest(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeByteCiQing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15853
// RVA: 0x00038EA0
// ADDRESS: 00438ea0
// PROTOTYPE: void __thiscall DeByteCiQing(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:357
// RVA: 0x0003A020
// ADDRESS: 0043a020
// PROTOTYPE: int __thiscall CheckGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelFriend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9163
// RVA: 0x0003A1A0
// ADDRESS: 0043a1a0
// PROTOTYPE: bool __thiscall DelFriend(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeleteGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10566
// RVA: 0x0003A300
// ADDRESS: 0043a300
// PROTOTYPE: CGUID __thiscall DeleteGoods(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenDead
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10712
// RVA: 0x0003A4D0
// ADDRESS: 0043a4d0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenDead(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10776
// RVA: 0x0003A860
// ADDRESS: 0043a860
// PROTOTYPE: void __thiscall DropParticularGoodsWhenLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DropParticularGoodsWhenRecall
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10818
// RVA: 0x0003AAC0
// ADDRESS: 0043aac0
// PROTOTYPE: void __thiscall DropParticularGoodsWhenRecall(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangSkillList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15230
// RVA: 0x0003AD30
// ADDRESS: 0043ad30
// PROTOTYPE: void __thiscall AddItemToTaoZhuangSkillList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15243
// RVA: 0x0003AD70
// ADDRESS: 0043ad70
// PROTOTYPE: void __thiscall AddItemToTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToCiQingTaoZhuangPreList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15254
// RVA: 0x0003ADB0
// ADDRESS: 0043adb0
// PROTOTYPE: void __thiscall AddItemToCiQingTaoZhuangPreList(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCurrentTypeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16080
// RVA: 0x0003ADF0
// ADDRESS: 0043adf0
// PROTOTYPE: void __thiscall GetCurrentTypeValue(map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddGoodsToPacket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:431
// RVA: 0x0003BFD0
// ADDRESS: 0043bfd0
// PROTOTYPE: bool __thiscall AddGoodsToPacket(vector<CGoods*,std::allocator<CGoods*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordOrgSysFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1459
// RVA: 0x0003C230
// ADDRESS: 0043c230
// PROTOTYPE: bool __thiscall DecordOrgSysFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquipRide
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6516
// RVA: 0x0003C5E0
// ADDRESS: 0043c5e0
// PROTOTYPE: void __thiscall MountEquipRide(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExitRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9661
// RVA: 0x0003DB50
// ADDRESS: 0043db50
// PROTOTYPE: void __thiscall OnExitRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:10910
// RVA: 0x0003DB60
// ADDRESS: 0043db60
// PROTOTYPE: ulong __thiscall IncExp(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11240
// RVA: 0x0003E1C0
// ADDRESS: 0043e1c0
// PROTOTYPE: bool __thiscall AddQuestDataByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12365
// RVA: 0x0003E3F0
// ADDRESS: 0043e3f0
// PROTOTYPE: bool __thiscall AddItemToDelList(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelAllItemInDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12401
// RVA: 0x0003E4E0
// ADDRESS: 0043e4e0
// PROTOTYPE: bool __thiscall DelAllItemInDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneDelList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12427
// RVA: 0x0003E670
// ADDRESS: 0043e670
// PROTOTYPE: void __thiscall DoneDelList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputeTicket
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12459
// RVA: 0x0003E820
// ADDRESS: 0043e820
// PROTOTYPE: ulong __thiscall ComputeTicket(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckAddGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12497
// RVA: 0x0003E920
// ADDRESS: 0043e920
// PROTOTYPE: ulong __thiscall CheckAddGoods(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12518
// RVA: 0x0003E980
// ADDRESS: 0043e980
// PROTOTYPE: bool __thiscall AddItemToMap(ulong param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12543
// RVA: 0x0003EA60
// ADDRESS: 0043ea60
// PROTOTYPE: bool __thiscall AddItemToGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DelItemFromGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12567
// RVA: 0x0003EAC0
// ADDRESS: 0043eac0
// PROTOTYPE: bool __thiscall DelItemFromGoodsAiTree(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneGoodsAiTree
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12616
// RVA: 0x0003EBD0
// ADDRESS: 0043ebd0
// PROTOTYPE: void __thiscall DoneGoodsAiTree(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateGoodsGS2C
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12784
// RVA: 0x0003EC40
// ADDRESS: 0043ec40
// PROTOTYPE: void __thiscall UpdateGoodsGS2C(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastUseSkillItemTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12826
// RVA: 0x0003ED30
// ADDRESS: 0043ed30
// PROTOTYPE: void __thiscall SetLastUseSkillItemTime(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClietAuctionOK
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13520
// RVA: 0x0003ED50
// ADDRESS: 0043ed50
// PROTOTYPE: void __thiscall TellClietAuctionOK(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteAuctionSelfToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13708
// RVA: 0x0003EF60
// ADDRESS: 0043ef60
// PROTOTYPE: void __thiscall AddByteAuctionSelfToClient(CMessage * param_1, CMessage * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteAuctionAllToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13764
// RVA: 0x0003F190
// ADDRESS: 0043f190
// PROTOTYPE: void __thiscall AddByteAuctionAllToClient(CMessage * param_1, CMessage * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendBackAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14323
// RVA: 0x0003F3D0
// ADDRESS: 0043f3d0
// PROTOTYPE: void __thiscall SendBackAucNode(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::TellClientScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14696
// RVA: 0x0003F500
// ADDRESS: 0043f500
// PROTOTYPE: void __thiscall TellClientScale(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendAuctionBangCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14938
// RVA: 0x0003F620
// ADDRESS: 0043f620
// PROTOTYPE: void __thiscall SendAuctionBangCondition(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ModifyAuctionSpace
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14977
// RVA: 0x0003F7A0
// ADDRESS: 0043f7a0
// PROTOTYPE: void __thiscall ModifyAuctionSpace(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OutputBinaryStream
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15063
// RVA: 0x0003F890
// ADDRESS: 0043f890
// PROTOTYPE: void __thiscall OutputBinaryStream(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendTaoZhuangSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15214
// RVA: 0x0003FA20
// ADDRESS: 0043fa20
// PROTOTYPE: void __thiscall SendTaoZhuangSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CleanTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15636
// RVA: 0x0003FAF0
// ADDRESS: 0043faf0
// PROTOTYPE: void __thiscall CleanTaoZhuangItemList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToTaoZhuangItemList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15642
// RVA: 0x0003FB60
// ADDRESS: 0043fb60
// PROTOTYPE: void __thiscall AddItemToTaoZhuangItemList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MakeCiQingNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15864
// RVA: 0x0003FCD0
// ADDRESS: 0043fcd0
// PROTOTYPE: ulong __thiscall MakeCiQingNode(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendCiQingGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16023
// RVA: 0x0003FE90
// ADDRESS: 0043fe90
// PROTOTYPE: void __thiscall SendCiQingGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecodeByteArrayLeiTing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17571
// RVA: 0x0003FFD0
// ADDRESS: 0043ffd0
// PROTOTYPE: void __thiscall DecodeByteArrayLeiTing(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeFyEnergyFlag
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17595
// RVA: 0x000400F0
// ADDRESS: 004400f0
// PROTOTYPE: bool __thiscall ChangeFyEnergyFlag(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:530
// RVA: 0x00040C30
// ADDRESS: 00440c30
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:742
// RVA: 0x00040DC0
// ADDRESS: 00440dc0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnExit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1692
// RVA: 0x00041460
// ADDRESS: 00441460
// PROTOTYPE: void __thiscall OnExit(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044156e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1712
// RVA: 0x0004156E
// ADDRESS: 0044156e
// PROTOTYPE: undefined Catch@0044156e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0044158b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1715
// RVA: 0x0004158B
// ADDRESS: 0044158b
// PROTOTYPE: undefined FUN_0044158b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnLost
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1780
// RVA: 0x000417A0
// ADDRESS: 004417a0
// PROTOTYPE: void __thiscall OnLost(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEquipmentWaste
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2731
// RVA: 0x000419B0
// ADDRESS: 004419b0
// PROTOTYPE: void __thiscall OnEquipmentWaste(EQUIPMENT_COLUMN param_1, CGoods * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnArmorDamaged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2818
// RVA: 0x00041AF0
// ADDRESS: 00441af0
// PROTOTYPE: void __thiscall OnArmorDamaged(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnWeaponDamaged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2932
// RVA: 0x00041D50
// ADDRESS: 00441d50
// PROTOTYPE: void __thiscall OnWeaponDamaged(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2966
// RVA: 0x00041D80
// ADDRESS: 00441d80
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnBeenMurdered
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3111
// RVA: 0x00042040
// ADDRESS: 00442040
// PROTOTYPE: void __thiscall OnBeenMurdered(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5239
// RVA: 0x00042610
// ADDRESS: 00442610
// PROTOTYPE: void __thiscall MountEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00443c5b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:6069
// RVA: 0x00043C5B
// ADDRESS: 00443c5b
// PROTOTYPE: undefined Catch@00443c5b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreHp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7785
// RVA: 0x00044C80
// ADDRESS: 00444c80
// PROTOTYPE: int __thiscall RestoreHp(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7804
// RVA: 0x00044D50
// ADDRESS: 00444d50
// PROTOTYPE: int __thiscall RestoreMp(ulong param_1, ulong param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::Mount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8513
// RVA: 0x00044E20
// ADDRESS: 00444e20
// PROTOTYPE: int __thiscall Mount(ulong param_1, ulong param_2, ulong param_3, char * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddFriend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9110
// RVA: 0x00044F80
// ADDRESS: 00444f80
// PROTOTYPE: void __thiscall AddFriend(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnObjectAdded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11095
// RVA: 0x000451A0
// ADDRESS: 004451a0
// PROTOTYPE: int __thiscall OnObjectAdded(CContainer * param_1, CBaseObject * param_2, ulong param_3, void * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordQuestDataFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11278
// RVA: 0x000452D0
// ADDRESS: 004452d0
// PROTOTYPE: bool __thiscall DecordQuestDataFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11316
// RVA: 0x00045360
// ADDRESS: 00445360
// PROTOTYPE: void __thiscall AddQuest(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::get_country_identity
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12150
// RVA: 0x00045540
// ADDRESS: 00445540
// PROTOTYPE: uchar __thiscall get_country_identity(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RestoreHpMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:12717
// RVA: 0x000455D0
// ADDRESS: 004455d0
// PROTOTYPE: void __thiscall RestoreHpMp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddItemToAuction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13492
// RVA: 0x00045910
// ADDRESS: 00445910
// PROTOTYPE: bool __thiscall AddItemToAuction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MakeCurAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:13999
// RVA: 0x00045A50
// ADDRESS: 00445a50
// PROTOTYPE: bool __thiscall MakeCurAucNode(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendBuyAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14353
// RVA: 0x00045EE0
// ADDRESS: 00445ee0
// PROTOTYPE: void __thiscall SendBuyAucNode(CGoodsNode * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendAucAbOpt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14397
// RVA: 0x00046080
// ADDRESS: 00446080
// PROTOTYPE: void __thiscall SendAucAbOpt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AutoAddAuctionGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14862
// RVA: 0x000466A0
// ADDRESS: 004466a0
// PROTOTYPE: void __thiscall AutoAddAuctionGoods(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComposeCiQingNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15901
// RVA: 0x00046A40
// ADDRESS: 00446a40
// PROTOTYPE: bool __thiscall ComposeCiQingNode(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateCiQingProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16110
// RVA: 0x00046E90
// ADDRESS: 00446e90
// PROTOTYPE: void __thiscall UpdateCiQingProperty(map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> param_1, map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> param_2, map<unsigned_long,unsigned_long,struct_std::less<unsigned_long>,class_std::allocator<struct_std::pair<unsigned_long_const_,unsigned_long>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountCiQingEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16353
// RVA: 0x00047000
// ADDRESS: 00447000
// PROTOTYPE: void __thiscall MountCiQingEquip(ulong param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00448346
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17020
// RVA: 0x00048346
// ADDRESS: 00448346
// PROTOTYPE: undefined Catch@00448346()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::~CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:256
// RVA: 0x00049860
// ADDRESS: 00449860
// PROTOTYPE: void __thiscall ~CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:286
// RVA: 0x0004A2D0
// ADDRESS: 0044a2d0
// PROTOTYPE: uchar __thiscall GetLevel(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:288
// RVA: 0x0004A2E0
// ADDRESS: 0044a2e0
// PROTOTYPE: ulong __thiscall GetExp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:365
// RVA: 0x0004A2F0
// ADDRESS: 0044a2f0
// PROTOTYPE: ulong __thiscall GetHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:366
// RVA: 0x0004A300
// ADDRESS: 0044a300
// PROTOTYPE: void __thiscall SetHP(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxHP
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:389
// RVA: 0x0004A340
// ADDRESS: 0044a340
// PROTOTYPE: ulong __thiscall GetMaxHP(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMinAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:405
// RVA: 0x0004A350
// ADDRESS: 0044a350
// PROTOTYPE: ulong __thiscall GetMinAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMaxAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:407
// RVA: 0x0004A360
// ADDRESS: 0044a360
// PROTOTYPE: ulong __thiscall GetMaxAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:409
// RVA: 0x0004A370
// ADDRESS: 0044a370
// PROTOTYPE: ushort __thiscall GetHit(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetCCH
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:413
// RVA: 0x0004A380
// ADDRESS: 0044a380
// PROTOTYPE: ushort __thiscall GetCCH(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDef
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:415
// RVA: 0x0004A390
// ADDRESS: 0044a390
// PROTOTYPE: ulong __thiscall GetDef(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetDodge
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:417
// RVA: 0x0004A3A0
// ADDRESS: 0044a3a0
// PROTOTYPE: ushort __thiscall GetDodge(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAtcSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:419
// RVA: 0x0004A3B0
// ADDRESS: 0044a3b0
// PROTOTYPE: short __thiscall GetAtcSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:421
// RVA: 0x0004A3C0
// ADDRESS: 0044a3c0
// PROTOTYPE: ulong __thiscall GetElementResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetHpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:423
// RVA: 0x0004A3D0
// ADDRESS: 0044a3d0
// PROTOTYPE: ushort __thiscall GetHpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMpRecoverSpeed
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:425
// RVA: 0x0004A3E0
// ADDRESS: 0044a3e0
// PROTOTYPE: ushort __thiscall GetMpRecoverSpeed(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:428
// RVA: 0x0004A3F0
// ADDRESS: 0044a3f0
// PROTOTYPE: ulong __thiscall GetExalt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetExalt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:429
// RVA: 0x0004A400
// ADDRESS: 0044a400
// PROTOTYPE: void __thiscall SetExalt(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetSoulResistant
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:432
// RVA: 0x0004A410
// ADDRESS: 0044a410
// PROTOTYPE: ushort __thiscall GetSoulResistant(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddElementAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:434
// RVA: 0x0004A420
// ADDRESS: 0044a420
// PROTOTYPE: ulong __thiscall GetAddElementAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAddSoulAtk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:436
// RVA: 0x0004A430
// ADDRESS: 0044a430
// PROTOTYPE: ushort __thiscall GetAddSoulAtk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetReAnk
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:441
// RVA: 0x0004A440
// ADDRESS: 0044a440
// PROTOTYPE: ushort __thiscall GetReAnk(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetAttackAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:444
// RVA: 0x0004A450
// ADDRESS: 0044a450
// PROTOTYPE: ushort __thiscall GetAttackAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetElementAvoid
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:446
// RVA: 0x0004A460
// ADDRESS: 0044a460
// PROTOTYPE: ushort __thiscall GetElementAvoid(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetFullMiss
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:448
// RVA: 0x0004A470
// ADDRESS: 0044a470
// PROTOTYPE: ushort __thiscall GetFullMiss(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:941
// RVA: 0x0004A480
// ADDRESS: 0044a480
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneCurAucNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14165
// RVA: 0x0004B100
// ADDRESS: 0044b100
// PROTOTYPE: void __thiscall DoneCurAucNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneCurAucBuyNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14258
// RVA: 0x0004B2A0
// ADDRESS: 0044b2a0
// PROTOTYPE: void __thiscall DoneCurAucBuyNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendBackCurBuyNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:14347
// RVA: 0x0004B370
// ADDRESS: 0044b370
// PROTOTYPE: void __thiscall SendBackCurBuyNode(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SendResultToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16137
// RVA: 0x0004B390
// ADDRESS: 0044b390
// PROTOTYPE: void __thiscall SendResultToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteToOtherPerson
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:16269
// RVA: 0x0004B5E0
// ADDRESS: 0044b5e0
// PROTOTYPE: void __thiscall AddByteToOtherPerson(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1252
// RVA: 0x0004BA80
// ADDRESS: 0044ba80
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1867
// RVA: 0x0004C400
// ADDRESS: 0044c400
// PROTOTYPE: bool __thiscall ChangeRegion(long param_1, long param_2, long param_3, long param_4, long param_5, long param_6, long param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnStandOnSwitchPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2596
// RVA: 0x0004D420
// ADDRESS: 0044d420
// PROTOTYPE: int __thiscall OnStandOnSwitchPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnDied
// STATUS: PARTIALLY_IMPLEMENTED_NATION_AND_GODS_BATTLE
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:3306
// RVA: 0x0004D850
// ADDRESS: 0044d850
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CheckLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:4930
// RVA: 0x00052DD0
// ADDRESS: 00452dd0
// PROTOTYPE: long __thiscall CheckLevel(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::MountAllEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5132
// RVA: 0x00053480
// ADDRESS: 00453480
// PROTOTYPE: void __thiscall MountAllEquip(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004535ee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:5183
// RVA: 0x000535EE
// ADDRESS: 004535ee
// PROTOTYPE: undefined Catch@004535ee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UseItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:7833
// RVA: 0x00053840
// ADDRESS: 00453840
// PROTOTYPE: void __thiscall UseItem(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::InitNameValueMap
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8693
// RVA: 0x00055000
// ADDRESS: 00455000
// PROTOTYPE: void __thiscall InitNameValueMap(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8858
// RVA: 0x000577B0
// ADDRESS: 004577b0
// PROTOTYPE: ulong __thiscall GetValue(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8889
// RVA: 0x00057AF0
// ADDRESS: 00457af0
// PROTOTYPE: ulong __thiscall SetValue(char * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:8927
// RVA: 0x00057E60
// ADDRESS: 00457e60
// PROTOTYPE: ulong __thiscall ChangeValue(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::IncreaseContinuousKill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9593
// RVA: 0x00058390
// ADDRESS: 00458390
// PROTOTYPE: void __thiscall IncreaseContinuousKill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PlayerRunScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:11397
// RVA: 0x000584E0
// ADDRESS: 004584e0
// PROTOTYPE: long __thiscall PlayerRunScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AdjustHonorRank
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15201
// RVA: 0x000585B0
// ADDRESS: 004585b0
// PROTOTYPE: bool __thiscall AdjustHonorRank(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RequestChangeAppellation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15208
// RVA: 0x000585C0
// ADDRESS: 004585c0
// PROTOTYPE: bool __thiscall RequestChangeAppellation(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ComputerAddValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15311
// RVA: 0x000585D0
// ADDRESS: 004585d0
// PROTOTYPE: void __thiscall ComputerAddValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DoneTaoZhuang
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:15678
// RVA: 0x00058810
// ADDRESS: 00458810
// PROTOTYPE: void __thiscall DoneTaoZhuang(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::RunQuestCompleteScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:17798
// RVA: 0x00058940
// ADDRESS: 00458940
// PROTOTYPE: long __thiscall RunQuestCompleteScript(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:89
// RVA: 0x000589B0
// ADDRESS: 004589b0
// PROTOTYPE: undefined __thiscall CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:601
// RVA: 0x000593E0
// ADDRESS: 004593e0
// PROTOTYPE: void __thiscall UpdateProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnRelive
// STATUS: IMPLEMENTED_WITH_RUNTIME_CONTEXT, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:1558
// RVA: 0x00059BB0
// ADDRESS: 00459bb0
// PROTOTYPE: void __thiscall OnRelive(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:2264
// RVA: 0x00059FF0
// ADDRESS: 00459ff0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::OnEnterRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.cpp:9681
// RVA: 0x0005A170
// ADDRESS: 0045a170
// PROTOTYPE: void __thiscall OnEnterRegion(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetNetExID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1037
// RVA: 0x0007BD10
// ADDRESS: 0047bd10
// PROTOTYPE: long __thiscall GetNetExID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetSeaGoodsName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1438
// RVA: 0x00087780
// ADDRESS: 00487780
// PROTOTYPE: void __thiscall SetSeaGoodsName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1173
// RVA: 0x00093440
// ADDRESS: 00493440
// PROTOTYPE: char * __thiscall GetLastContainerScript(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCharged
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:317
// RVA: 0x000AEB30
// ADDRESS: 004aeb30
// PROTOTYPE: void __thiscall SetCharged(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetMaxEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:526
// RVA: 0x000AEB40
// ADDRESS: 004aeb40
// PROTOTYPE: void __thiscall SetMaxEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetCreateFactionOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1023
// RVA: 0x000AEB60
// ADDRESS: 004aeb60
// PROTOTYPE: void __thiscall SetCreateFactionOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetApplyJoinOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1026
// RVA: 0x000AEB70
// ADDRESS: 004aeb70
// PROTOTYPE: void __thiscall SetApplyJoinOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFactionDeclareWarOperator
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1029
// RVA: 0x000AEB80
// ADDRESS: 004aeb80
// PROTOTYPE: void __thiscall SetFactionDeclareWarOperator(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetEnergy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:524
// RVA: 0x000AED90
// ADDRESS: 004aed90
// PROTOTYPE: void __thiscall SetEnergy(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetLastContainerScript
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:1172
// RVA: 0x000AEFC0
// ADDRESS: 004aefc0
// PROTOTYPE: void __thiscall SetLastContainerScript(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::PushItemToCiQingList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:252
// RVA: 0x000AF1E0
// ADDRESS: 004af1e0
// PROTOTYPE: void __thiscall PushItemToCiQingList(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddStr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:511
// RVA: 0x000FA9E0
// ADDRESS: 004fa9e0
// PROTOTYPE: void __thiscall AddStr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddDex
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:512
// RVA: 0x000FAA00
// ADDRESS: 004faa00
// PROTOTYPE: void __thiscall AddDex(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddCon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:513
// RVA: 0x000FAA20
// ADDRESS: 004faa20
// PROTOTYPE: void __thiscall AddCon(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddInt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:514
// RVA: 0x000FAA40
// ADDRESS: 004faa40
// PROTOTYPE: void __thiscall AddInt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:458
// RVA: 0x001DFD60
// ADDRESS: 005dfd60
// PROTOTYPE: void __thiscall SetBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastAttackScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:462
// RVA: 0x001DFD90
// ADDRESS: 005dfd90
// PROTOTYPE: void __thiscall SetElementBlastAttackScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetElementBlastDefendScale
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\player.h:464
// RVA: 0x001DFDC0
// ADDRESS: 005dfdc0
// PROTOTYPE: void __thiscall SetElementBlastDefendScale(float param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
