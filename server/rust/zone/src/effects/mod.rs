//! Наложенные состояния Zone; живой владелец фигуры подключается переходным Game.

mod agility; // состояния ловкости CAgility/CNatural/CRapture и временная AgilityState2.
mod attackgain; // усиление атаки Fury и RageBreak.
mod automaticrestore; // четыре состояния автоматического восстановления HP/MP.
mod battlefairy; // данные и 12-байтная запись состояний боевой феи Po/Yu.
mod blind; // семейство CBlindState и родственные блокировки (BoaLock, KnockOut, SpiderWeb и др.).
mod bloodloss; // сохраняемые данные периодической потери крови.
mod bossbluefury; // состояние CBossBlueFuryState (0x1F7).
mod callosity; // CCallosityState/CCallosityState2 с общей записью и формулой.
mod changebody; // CHBYState (0x37): данные и 124-байтная запись смены тела.
mod consumablerestore; // восстановление HP/MP от расходуемых предметов.
mod cure; // данные, срок и сохраняемая запись CCureState.
mod daubpoison; // состояние смазанного ядом оружия CDaubPoisonState.
mod defenseshield; // диспетчер защитной ветви PreDefense.
mod element; // прибавки TaiJi и Origin к свойствам стихии.
mod energyholding; // счётчик и сохраняемые поля CEnergyHoldingState.
mod extended; // extended-state CExState (0x32) и CExStateNew (0x33).
mod fullmiss; // постоянная прибавка к полному уклонению.
mod godbless; // данные и числовые правила CGodBlessState/CGodBlessState2.
mod heal; // периодическое лечение CHealState.
mod hearten; // данные, срок и запись CHeartenState.
mod leafcut; // три периодических состояния LeafCut.
mod lifeshield; // поглощение урона щитом жизни.
mod machineshield; // поглощение урона машинным щитом.
mod manashield; // данные мана-щита и его защитная ветвь.
mod maxresource; // сохранённые прибавки к максимумам HP и MP.
mod meteorarrow; // запас метеорных стрел CMeteorArrowState.
mod periodicattack; // данные, часы и запись периодического урона.
mod pillar; // данные и срок защитной стойки CPillarState.
mod particular; // CParticularState (0x186A5).
mod poison; // четыре состояния периодического ядовитого урона.
mod poisonfog; // CPoisonFogState: данные, запись и расчёт ослабления.
mod promotion; // CPromotionState (0x142) и element-атака.
mod ride; // CRideState (100004): данные и wire-кодек верхового состояния.
mod roar; // числовое подавление атаки CRoarState.
mod scriptstate; // семь состояний AddState: пять UseGoods, ImproveExp и AutoProtect.
mod shieldabsorption; // общая числовая часть PreDefense мана- и машинного щитов.
mod soulcollect; // состояние сбора душ CSoulCollectState.
mod swordship; // постоянные прибавки мин./макс. атаки Swordship.
mod teamstate; // данные и запись командного CTeamState (0x186A6).
mod tianshenxiafan; // сохраняемая запись CTianShenXiaFanState (0x335).
mod time; // отображаемое оставшееся время наложенного состояния.
mod undead; // CNotDisappearAfterDead (внешний ID 0x38).
mod visualeffect; // базовый CVisualEffect состояний и навыков.
mod wangsheng; // сохранённые данные CWangshengState.
mod weak; // данные и правила CWeakState.
mod wuxing; // постоянные состояния У-син и их 96-байтная запись.

pub use agility::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AGILITY_STATE_2_BYTES, AgilityState2, NATURAL_SKILL_ID,
    PERSISTENT_AGILITY_FAMILY_STATE_BYTES, PersistentAgilityFamilyState,
    PersistentAgilityProperties, RAPTURE_SKILL_ID,
}; // типы состояний ловкости с их skill ID и размерами записи.
pub use attackgain::{
    ATTACK_GAIN_STATE_BYTES, AttackGainState, FURY_STATE_SKILL_ID, FuryState, RAGE_BREAK_STATE_ID,
    RageBreakState,
}; // общее состояние усиления атаки с ID Fury и RageBreak.
pub use automaticrestore::{
    AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreKind, AutomaticRestoreMutation,
    AutomaticRestoreProperties, AutomaticRestoreState, is_automatic_restore_state_id,
}; // состояния авторегена HP/MP: вид, мутация и проверка state ID.
pub use battlefairy::{
    BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, BattleFairyAttributeKind, BattleFairyAttributePlayerView,
    BattleFairyAttributeState, battle_fairy_attribute_kind,
}; // состояние бонуса феи: вид, проекция игрока и выбор вида.
pub use blind::{
    BLIND_STATE_BYTES, BLIND_STATE_ID, BOA_LOCK_STATE_BYTES, BOA_LOCK_STATE_ID,
    BOSS_BLUE_QUAKE_STATE_BYTES, BOSS_BLUE_QUAKE_STATE_ID, BlindState, BoaLockState,
    BossBlueQuakeState, KNIGHT_CUT_STATE_BYTES, KNIGHT_CUT_STATE_ID, KNOCK_OUT_STATE_BYTES,
    KNOCK_OUT_STATE_ID, KnightCutState, KnockOutState, SPIDER_WEB_STATE_BYTES, SPIDER_WEB_STATE_ID,
    SpiderWebState,
}; // блокирующие состояния семейства Blind: типы, ID и размеры.
pub use bloodloss::{
    BLOOD_LOSS_STATE_BYTES, BLOOD_LOSS_STATE_ID, BloodLossAttackSeed, BloodLossState,
}; // состояние потери крови и исходные данные её атаки.
pub use bossbluefury::{BOSS_BLUE_FURY_STATE_BYTES, BOSS_BLUE_FURY_STATE_ID, BossBlueFuryState}; // состояние ярости синего босса (0x1F7).
pub use callosity::{
    CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID, CALLOSITY_STATE_BYTES, CallosityFamilyState,
}; // семейное состояние Callosity с ID обоих вариантов.
pub use changebody::{
    CHANGE_BODY_PARAMETER_BYTES, CHANGE_BODY_SKILL_TYPE, CHANGE_BODY_STATE_BYTES,
    CHANGE_BODY_STATE_ID, ChangeBodyState,
}; // состояние смены тела и константы его записи.
pub use consumablerestore::{
    CONSUMABLE_RESTORE_STATE_BYTES, ConsumableRestoreIntervals, ConsumableRestoreState,
    RESTORE_HP_STATE_ID, RESTORE_MP_STATE_ID, RestoreHpState, RestoreMpState, RestoreStateData,
}; // состояния восстановления от расходников и их интервалы.
pub use cure::{CURE_STATE_BYTES, CURE_STATE_SKILL_ID, CureState}; // состояние исцеления CCureState.
pub use daubpoison::{DAUB_POISON_STATE_BYTES, DAUB_POISON_STATE_ID, DaubPoisonState}; // состояние яда на оружии CDaubPoisonState.
pub use defenseshield::{DefenseShieldState, is_pre_defense_skipped_skill}; // состояние защитной ветви и предикат пропуска PreDefense.
pub use element::{
    ELEMENT_STATE_BYTES, ElementState, ORIGIN_STATE_ID, OriginState, TAIJI_STATE_ID, TaiJiState,
}; // прибавки стихии: состояния ElementState/TaiJi/Origin с ID.
pub use energyholding::{ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID, EnergyHoldingState}; // счётчик энергии CEnergyHoldingState с ID.
pub use extended::{
    EX_STATE_BYTES, EX_STATE_ID, EX_STATE_NEW_BYTES, EX_STATE_NEW_ID, ExtendedState,
    ExtendedStateKind,
}; // extended-состояние: вид Ex/ExNew и размеры обеих записей.
pub use fullmiss::{
    ENLARGE_FULL_MISS_STATE_BYTES, ENLARGE_FULL_MISS_STATE_ID, EnlargeFullMissState,
}; // состояние прибавки полного уклонения (0x25B).
pub use godbless::{
    GOD_BLESS_STATE_2_ID, GOD_BLESS_STATE_BYTES, GOD_BLESS_STATE_ID, GodBlessState,
}; // состояние CGodBlessState с ID обеих версий.
pub use heal::{HEAL_STATE_BYTES, HealState}; // состояние периодического лечения.
pub use hearten::{HEARTEN_STATE_BYTES, HEARTEN_STATE_ID, HeartenState}; // состояние CHeartenState с ID.
pub use leafcut::{
    LEAF_CUT_2_STATE_ID, LEAF_CUT_3_STATE_ID, LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID,
    LeafCutAttackSeed, LeafCutState,
}; // семейство LeafCut: три ID и исходные данные атаки.
pub use lifeshield::{LIFE_SHIELD_SKILL_ID, LIFE_SHIELD_STATE_BYTES, LifeShieldState}; // щит жизни: состояние и skill ID.
pub use machineshield::{MACHINE_SHIELD_SKILL_ID, MACHINE_SHIELD_STATE_BYTES, MachineShieldState}; // машинный щит: состояние и skill ID.
pub use manashield::{MANA_SHIELD_SKILL_ID, MANA_SHIELD_STATE_BYTES, ManaShieldState}; // мана-щит: состояние и skill ID.
pub use maxresource::{
    ENLARGE_MAX_HP_STATE_ID, ENLARGE_MAX_MP_STATE_ID, EnlargeMaxHpState, EnlargeMaxMpState,
    MAX_RESOURCE_STATE_BYTES, MaxResourceState,
}; // прибавки максимумов HP/MP: состояния, ID и общий размер.
pub use meteorarrow::{METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES, MeteorArrowState}; // состояние метеорных стрел и его skill ID.
pub use periodicattack::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
}; // ядро, запись и кодирование периодической атаки.
pub use pillar::{PILLAR_STATE_BYTES, PILLAR_STATE_ID, PillarState}; // защитная стойка CPillarState с ID.
pub use particular::{
    PARTICULAR_STATE_BYTES, PARTICULAR_STATE_CHECK_INTERVAL_MS, PARTICULAR_STATE_ID,
    ParticularState,
}; // состояние CParticularState: ID, размер и интервал проверки.
pub use poison::{POISON_STATE_BYTES, PoisonState}; // общее состояние периодического яда.
pub use poisonfog::{POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID, PoisonFogState}; // состояние ядовитого тумана с ID.
pub use promotion::{
    PROMOTION_STATE_BYTES, PROMOTION_STATE_ID, PromotionState, promotion_element_attack,
}; // состояние CPromotion и расчёт его element-атаки.
pub use ride::{RIDE_GOODS_CHECK_INTERVAL_MS, RIDE_STATE_ID, RideState}; // верховое состояние: ID и интервал проверки предмета.
pub use roar::{ROAR_STATE_BYTES, ROAR_STATE_ID, RoarState}; // подавление атаки CRoarState с ID.
pub use scriptstate::{
    AUTO_PROTECT_STATE_BYTES, AUTO_PROTECT_STATE_ID, IMPROVE_EXP_STATE_ID,
    SCRIPT_STATE_TIMED_BYTES, ScriptMoveState, ScriptStateKind,
    USE_GOODS_ENLARGE_DEF_STATE_ID, USE_GOODS_ENLARGE_ELM_DEF_STATE_ID,
    USE_GOODS_ENLARGE_FULL_MISS_STATE_ID, USE_GOODS_ENLARGE_MAX_HP_STATE_ID,
    USE_GOODS_ENLARGE_MAX_MP_STATE_ID,
}; // script-состояния: вид, move-данные и ID семи вариантов.
pub use soulcollect::{SOUL_COLLECT_STATE_BYTES, SOUL_COLLECT_STATE_ID, SoulCollectState}; // состояние сбора душ с ID.
pub use swordship::{
    SWORDSHIP_2_STATE_ID, SWORDSHIP_3_STATE_ID, SWORDSHIP_4_STATE_ID, SWORDSHIP_STATE_BYTES,
    SWORDSHIP_STATE_ID, SwordshipState, is_swordship_state_id,
}; // прибавки атаки Swordship: ID четырёх вариантов и проверка принадлежности.
pub use teamstate::{
    CTeamState, TEAM_STATE_CHECK_INTERVAL_MS, TEAM_STATE_ID, TEAM_STATE_STRING_CAPACITY,
}; // командное состояние CTeamState: ID, интервал и ёмкость строк.
pub use tianshenxiafan::{
    TIAN_SHEN_XIA_FAN_STATE_BYTES, TIAN_SHEN_XIA_FAN_STATE_ID, TianShenXiaFanPlayerView,
    TianShenXiaFanState,
}; // состояние CTianShenXiaFan и проекция игрока.
pub use time::{change_body_client_state_time, guarded_client_state_time, timed_client_state_time}; // вычисление отображаемого остатка срока состояния.
pub use undead::{UNDEAD_STATE_ID, UNDEAD_STATE_PARAMETER_BYTES, UndeadState}; // состояние CNotDisappearAfterDead: ID и размер параметра.
pub use visualeffect::CVisualEffect; // базовый тип визуального эффекта.
pub use wangsheng::{WANGSHENG_STATE_BYTES, WANGSHENG_STATE_ID, WangshengState}; // состояние CWangshengState с ID.
pub use weak::{WEAK_STATE_BYTES, WEAK_STATE_ID, WeakState}; // состояние ослабления CWeakState с ID.
pub use wuxing::{
    WUXING_EARTH_STATE_ID, WUXING_FIRE_STATE_ID, WUXING_METAL_STATE_ID, WUXING_STATE_BYTES,
    WUXING_WATER_STATE_ID, WUXING_WOOD_STATE_ID, WuXingCoefficients, WuXingKind, WuXingProperties,
    WuXingState, WuXingStateParameters, apply_wuxing_to_properties, kind_for_skill_id,
}; // состояния У-син: вид, параметры и применение к свойствам.
