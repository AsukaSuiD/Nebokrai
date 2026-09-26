//! Декодирование последовательности состояний из GameSave. native-сторона —
//! статический `CStateFactory::Unserialize`.
//!
//! Каталог записей разнесён на две независимые таблицы: размерная
//! (bytes/unserialize/normalize, без типизации) задаёт границы записей и
//! обход спанов, decoder — типизацию записи в enum-каталог. Неизвестная или
//! усечённая запись останавливает типизацию до спорного участка: исходный
//! хвост и остаток declared count сохраняются в LegacyStateCodec (техническая
//! страховка, не обещание round-trip повреждённого GameSave). Размер чтения и
//! размер cache-проекции могут различаться; продвижение курсора — только
//! читаемым layout.
//!
//! Каталог опирается только на Zone-данные: layout/decoder записей —
//! `effects`, идентичность — `regions`, factory-параметр — `skills::skillfactory`,
//! целевой enum-каталог — `skills::state` (`StateData`). Идентификаторы семей
//! без владельца в `effects` (SEAL/RUSH/RUSH2/STRIKE/KEROSENE и навыковые
//! HEAL/SUPER_HEAL/SPIDER_POISON/SPRITE_BURN/POISON_ARROW) — нативные
//! константы их исходных cpp-файлов, остаются здесь до переезда состояний.
//!
//! Исходный владелец PDB: `appserver/skills/statefactory.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#state--арена-кодек-и-каталог-состояний

use crate::effects::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AGILITY_STATE_2_BYTES, ATTACK_GAIN_STATE_BYTES,
    AUTOMATIC_RESTORE_STATE_BYTES, AgilityState2, AutomaticRestoreState,
    BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, BLIND_STATE_BYTES, BLIND_STATE_ID,
    BOA_LOCK_STATE_BYTES, BOA_LOCK_STATE_ID, BOSS_BLUE_FURY_STATE_BYTES,
    BOSS_BLUE_FURY_STATE_ID, BOSS_BLUE_QUAKE_STATE_BYTES, BOSS_BLUE_QUAKE_STATE_ID,
    BLOOD_LOSS_STATE_BYTES, BLOOD_LOSS_STATE_ID, BattleFairyAttributeState, BlindState,
    BloodLossState, BoaLockState, BossBlueFuryState, BossBlueQuakeState, CALLOSITY_2_SKILL_ID,
    CALLOSITY_SKILL_ID, CALLOSITY_STATE_BYTES, CHANGE_BODY_STATE_ID, CONSUMABLE_RESTORE_STATE_BYTES,
    CURE_STATE_BYTES, CURE_STATE_SKILL_ID, CTeamState, CallosityFamilyState, ChangeBodyState,
    ConsumableRestoreState, CureState, DAUB_POISON_STATE_BYTES, DAUB_POISON_STATE_ID,
    DaubPoisonState, DefenseShieldState, ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID,
    ENLARGE_FULL_MISS_STATE_BYTES, ENLARGE_FULL_MISS_STATE_ID, ENLARGE_MAX_HP_STATE_ID,
    ENLARGE_MAX_MP_STATE_ID, EX_STATE_ID, EX_STATE_NEW_ID, ELEMENT_STATE_BYTES,
    EnergyHoldingState, EnlargeFullMissState, EnlargeMaxHpState, EnlargeMaxMpState,
    FURY_STATE_SKILL_ID, ExtendedState, FuryState, GOD_BLESS_STATE_2_ID, GOD_BLESS_STATE_BYTES,
    GOD_BLESS_STATE_ID, GodBlessState, HEAL_STATE_BYTES, HEARTEN_STATE_BYTES, HEARTEN_STATE_ID,
    HealState, HeartenState, KNIGHT_CUT_STATE_BYTES, KNIGHT_CUT_STATE_ID, KNOCK_OUT_STATE_BYTES,
    KNOCK_OUT_STATE_ID, KnightCutState, KnockOutState, LEAF_CUT_2_STATE_ID, LEAF_CUT_3_STATE_ID,
    LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID, LIFE_SHIELD_SKILL_ID, LIFE_SHIELD_STATE_BYTES,
    LeafCutState, LifeShieldState, MACHINE_SHIELD_SKILL_ID, MACHINE_SHIELD_STATE_BYTES,
    MANA_SHIELD_SKILL_ID, MANA_SHIELD_STATE_BYTES, MAX_RESOURCE_STATE_BYTES,
    METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES, MachineShieldState, ManaShieldState,
    MeteorArrowState, NATURAL_SKILL_ID, ORIGIN_STATE_ID, OriginState, PARTICULAR_STATE_BYTES,
    PARTICULAR_STATE_ID, PERSISTENT_AGILITY_FAMILY_STATE_BYTES, PILLAR_STATE_BYTES,
    PILLAR_STATE_ID, POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID, POISON_STATE_BYTES,
    PROMOTION_STATE_BYTES, PROMOTION_STATE_ID,
    ParticularState, PersistentAgilityFamilyState, PillarState, PoisonFogState, PoisonState,
    PromotionState, RAGE_BREAK_STATE_ID, RAPTURE_SKILL_ID, RESTORE_HP_STATE_ID, RESTORE_MP_STATE_ID,
    RIDE_STATE_ID, ROAR_STATE_BYTES, ROAR_STATE_ID, RageBreakState, RideState, RoarState,
    SOUL_COLLECT_STATE_BYTES, SOUL_COLLECT_STATE_ID, SPIDER_WEB_STATE_BYTES, SPIDER_WEB_STATE_ID,
    SWORDSHIP_2_STATE_ID, SWORDSHIP_3_STATE_ID, SWORDSHIP_4_STATE_ID, SWORDSHIP_STATE_BYTES,
    SWORDSHIP_STATE_ID, ScriptMoveState, SoulCollectState, SpiderWebState, SwordshipState,
    TAIJI_STATE_ID, TEAM_STATE_ID, TIAN_SHEN_XIA_FAN_STATE_BYTES, TIAN_SHEN_XIA_FAN_STATE_ID,
    TaiJiState, TianShenXiaFanState, UNDEAD_STATE_ID, UndeadState, WANGSHENG_STATE_BYTES,
    WANGSHENG_STATE_ID, WangshengState, WEAK_STATE_BYTES, WEAK_STATE_ID, WUXING_STATE_BYTES,
    WeakState, WuXingState, is_automatic_restore_state_id,
};
use crate::regions::ShapeIdentity;

use super::skillfactory::CSkillFactory;
use super::state::StateData;

// Нативные идентификаторы записей семей, чьи состояния ещё не выделили
// собственного владельца в `effects`; значения сверены с их cpp-файлами.
const SEAL_STATE_ID: u32 = 0x138;
const RUSH_STATE_ID: u32 = 0x73;
const RUSH_2_STATE_ID: u32 = 0x7c;
const STRIKE_STATE_ID: u32 = 0xdd;
const KEROSENE_STATE_ID: u32 = 0xf1;
const HEAL_SKILL_ID: u32 = 0xd3;
const HEAL_2_SKILL_ID: u32 = 0xe3;
const SUPER_HEAL_SKILL_ID: u32 = 0xd9;
const SUPER_HEAL_2_SKILL_ID: u32 = 0xe4;
const POISON_ARROW_SKILL_ID: u32 = 0x21e;
pub const SPIDER_POISON_SKILL_ID: u32 = 0x191;
const SPRITE_BURN_SKILL_ID: u32 = 0x1a6;

// Псевдонимы конкретных вариантов общих payload семей; в переходном Game тем
// же константам соответствуют type-псевдонимы appserver/skills.
pub type LeafCutState2 = LeafCutState<LEAF_CUT_2_STATE_ID>;
pub type LeafCutState3 = LeafCutState<LEAF_CUT_3_STATE_ID>;
pub type SealState = BlindState<SEAL_STATE_ID>;
pub type RushState = BlindState<RUSH_STATE_ID>;
pub type Rush2State = BlindState<RUSH_2_STATE_ID>;
pub type StrikeState = BlindState<STRIKE_STATE_ID>;
pub type KeroseneState = PoisonState<KEROSENE_STATE_ID>;
pub type PoisonArrowState = PoisonState<POISON_ARROW_SKILL_ID>;
pub type SpiderPoisonState = PoisonState<SPIDER_POISON_SKILL_ID>;
pub type SpriteBurnState = PoisonState<SPRITE_BURN_SKILL_ID>;

/// Цель типизации декодированной записи состояния. Trait не содержит игровых
/// ID и не дублирует каталог записей, а только передаёт декодированный
/// конкретный payload владельцу enum-каталога. Эта категория реализаций
/// описывает тот же enum, что порождает записи: для арены состояний impl
/// для `StateData` живёт ниже.
pub trait StateRecordTarget: Sized {
    fn change_body(state: ChangeBodyState) -> Self;
    fn extended(state: ExtendedState) -> Self;
    fn undead(state: UndeadState) -> Self;
    fn leaf_cut(state: LeafCutState) -> Self;
    fn leaf_cut_2(state: LeafCutState2) -> Self;
    fn leaf_cut_3(state: LeafCutState3) -> Self;
    fn kerosene(state: KeroseneState) -> Self;
    fn swordship(state: SwordshipState) -> Self;
    fn strike(state: StrikeState) -> Self;
    fn wu_xing(state: WuXingState) -> Self;
    fn poison_fog(state: PoisonFogState) -> Self;
    fn meteor_arrow(state: MeteorArrowState) -> Self;
    fn blind(state: BlindState) -> Self;
    fn knock_out(state: KnockOutState) -> Self;
    fn spider_web(state: SpiderWebState) -> Self;
    fn seal(state: SealState) -> Self;
    fn god_bless(state: GodBlessState) -> Self;
    fn weak(state: WeakState) -> Self;
    fn soul_collect(state: SoulCollectState) -> Self;
    fn sprite_burn(state: SpriteBurnState) -> Self;
    fn spider_poison(state: SpiderPoisonState) -> Self;
    fn daub_poison(state: DaubPoisonState) -> Self;
    fn boss_blue_quake(state: BossBlueQuakeState) -> Self;
    fn knight_cut(state: KnightCutState) -> Self;
    fn boa_lock(state: BoaLockState) -> Self;
    fn rush(state: RushState) -> Self;
    fn rush_2(state: Rush2State) -> Self;
    fn roar(state: RoarState) -> Self;
    fn pillar(state: PillarState) -> Self;
    fn rage_break(state: RageBreakState) -> Self;
    fn fury(state: FuryState) -> Self;
    fn heal(state: HealState) -> Self;
    fn cure(state: CureState) -> Self;
    fn enlarge_full_miss(state: EnlargeFullMissState) -> Self;
    fn tai_ji(state: TaiJiState) -> Self;
    fn enlarge_max_hp(state: EnlargeMaxHpState) -> Self;
    fn enlarge_max_mp(state: EnlargeMaxMpState) -> Self;
    fn origin(state: OriginState) -> Self;
    fn hearten(state: HeartenState) -> Self;
    fn persistent_agility(state: PersistentAgilityFamilyState) -> Self;
    fn agility_2(state: AgilityState2) -> Self;
    fn callosity(state: CallosityFamilyState) -> Self;
    fn blood_loss(state: BloodLossState) -> Self;
    fn boss_blue_fury(state: BossBlueFuryState) -> Self;
    fn battle_fairy_attribute(state: BattleFairyAttributeState) -> Self;
    fn tian_shen_xia_fan(state: TianShenXiaFanState) -> Self;
    fn wangsheng(state: WangshengState) -> Self;
    fn poison_arrow(state: PoisonArrowState) -> Self;
    fn consumable_restore(state: ConsumableRestoreState) -> Self;
    fn automatic_restore(state: AutomaticRestoreState) -> Self;
    fn particular(state: ParticularState) -> Self;
    fn team(state: CTeamState) -> Self;
    fn script(state: ScriptMoveState) -> Self;
    fn ride(state: RideState) -> Self;
    fn defense_shield(state: DefenseShieldState) -> Self;
    fn energy_holding(state: EnergyHoldingState) -> Self;

    /// Нормализованная Save-проекция для записей, у которых Load читает
    /// меньше байт, чем пишет Save (ветвь TianShenXiaFan). Остальные записи
    /// возвращают None и сохраняют прочитанный участок как есть.
    fn normalized_save_record(&self) -> Option<Vec<u8>> {
        None
    }
}

impl StateRecordTarget for StateData {
    fn change_body(state: ChangeBodyState) -> Self { Self::ChangeBody(state) }
    fn extended(state: ExtendedState) -> Self { Self::Extended(state) }
    fn undead(state: UndeadState) -> Self { Self::Undead(state) }
    fn leaf_cut(state: LeafCutState) -> Self { Self::LeafCut(state) }
    fn leaf_cut_2(state: LeafCutState2) -> Self { Self::LeafCut2(state) }
    fn leaf_cut_3(state: LeafCutState3) -> Self { Self::LeafCut3(state) }
    fn kerosene(state: KeroseneState) -> Self { Self::Kerosene(state) }
    fn swordship(state: SwordshipState) -> Self { Self::Swordship(state) }
    fn strike(state: StrikeState) -> Self { Self::Strike(state) }
    fn wu_xing(state: WuXingState) -> Self { Self::WuXing(state) }
    fn poison_fog(state: PoisonFogState) -> Self { Self::PoisonFog(state) }
    fn meteor_arrow(state: MeteorArrowState) -> Self { Self::MeteorArrow(state) }
    fn blind(state: BlindState) -> Self { Self::Blind(state) }
    fn knock_out(state: KnockOutState) -> Self { Self::KnockOut(state) }
    fn spider_web(state: SpiderWebState) -> Self { Self::SpiderWeb(state) }
    fn seal(state: SealState) -> Self { Self::Seal(state) }
    fn god_bless(state: GodBlessState) -> Self { Self::GodBless(state) }
    fn weak(state: WeakState) -> Self { Self::Weak(state) }
    fn soul_collect(state: SoulCollectState) -> Self { Self::SoulCollect(state) }
    fn sprite_burn(state: SpriteBurnState) -> Self { Self::SpriteBurn(state) }
    fn spider_poison(state: SpiderPoisonState) -> Self { Self::SpiderPoison(state) }
    fn daub_poison(state: DaubPoisonState) -> Self { Self::DaubPoison(state) }
    fn boss_blue_quake(state: BossBlueQuakeState) -> Self { Self::BossBlueQuake(state) }
    fn knight_cut(state: KnightCutState) -> Self { Self::KnightCut(state) }
    fn boa_lock(state: BoaLockState) -> Self { Self::BoaLock(state) }
    fn rush(state: RushState) -> Self { Self::Rush(state) }
    fn rush_2(state: Rush2State) -> Self { Self::Rush2(state) }
    fn roar(state: RoarState) -> Self { Self::Roar(state) }
    fn pillar(state: PillarState) -> Self { Self::Pillar(state) }
    fn rage_break(state: RageBreakState) -> Self { Self::RageBreak(state) }
    fn fury(state: FuryState) -> Self { Self::Fury(state) }
    fn heal(state: HealState) -> Self { Self::Heal(state) }
    fn cure(state: CureState) -> Self { Self::Cure(state) }
    fn enlarge_full_miss(state: EnlargeFullMissState) -> Self { Self::EnlargeFullMiss(state) }
    fn tai_ji(state: TaiJiState) -> Self { Self::TaiJi(state) }
    fn enlarge_max_hp(state: EnlargeMaxHpState) -> Self { Self::EnlargeMaxHp(state) }
    fn enlarge_max_mp(state: EnlargeMaxMpState) -> Self { Self::EnlargeMaxMp(state) }
    fn origin(state: OriginState) -> Self { Self::Origin(state) }
    fn hearten(state: HeartenState) -> Self { Self::Hearten(state) }
    fn persistent_agility(state: PersistentAgilityFamilyState) -> Self { Self::PersistentAgility(state) }
    fn agility_2(state: AgilityState2) -> Self { Self::Agility2(state) }
    fn callosity(state: CallosityFamilyState) -> Self { Self::Callosity(state) }
    fn blood_loss(state: BloodLossState) -> Self { Self::BloodLoss(state) }
    fn boss_blue_fury(state: BossBlueFuryState) -> Self { Self::BossBlueFury(state) }
    fn battle_fairy_attribute(state: BattleFairyAttributeState) -> Self { Self::BattleFairyAttribute(state) }
    fn tian_shen_xia_fan(state: TianShenXiaFanState) -> Self { Self::TianShenXiaFan(state) }
    fn wangsheng(state: WangshengState) -> Self { Self::Wangsheng(state) }
    fn poison_arrow(state: PoisonArrowState) -> Self { Self::PoisonArrow(state) }
    fn consumable_restore(state: ConsumableRestoreState) -> Self { Self::ConsumableRestore(state) }
    fn automatic_restore(state: AutomaticRestoreState) -> Self { Self::AutomaticRestore(state) }
    fn particular(state: ParticularState) -> Self { Self::Particular(state) }
    fn team(state: CTeamState) -> Self { Self::Team(state) }
    fn script(state: ScriptMoveState) -> Self { Self::Script(state) }
    fn ride(state: RideState) -> Self { Self::Ride(state) }
    fn defense_shield(state: DefenseShieldState) -> Self { Self::DefenseShield(state) }
    fn energy_holding(state: EnergyHoldingState) -> Self { Self::EnergyHolding(state) }

    fn normalized_save_record(&self) -> Option<Vec<u8>> {
        match self {
            Self::TianShenXiaFan(state) => Some(state.encoded().to_vec()),
            _ => None,
        }
    }
}

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    let bytes = payload.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

type StateDecoder<S> = fn(&[u8], usize, ShapeIdentity, &CSkillFactory, &mut dyn FnMut() -> u32) -> Option<S>;

/// Размерный контур записи без типизации: границы native-входа и поля
/// нормализации cache. Ветвь TianShenXiaFan читает 10 байт, хотя cache Save
/// пишет 12; остальные записи читают ровно свой layout.
struct StateRecordSizes {
    bytes: usize,
    unserialize_bytes: usize,
    normalize: bool,
}

impl StateRecordSizes {
    const fn typed(bytes: usize) -> Self {
        Self { bytes, unserialize_bytes: bytes, normalize: false }
    }
}

/// Размерная таблица каталога: ID записи → границы записи в native-входе и
/// признак нормализации Save-проекции. Порядок ветвей повторяет бывший
/// общий `record_layout`; динамические размеры (Team/Script/Ride) читаются
/// из самого входа без типизации записи.
fn record_sizes(payload: &[u8], cursor: usize, state_id: u32) -> Option<StateRecordSizes> {
    Some(match state_id {
        CHANGE_BODY_STATE_ID => StateRecordSizes::typed(124),
        EX_STATE_ID => StateRecordSizes::typed(44),
        EX_STATE_NEW_ID => StateRecordSizes::typed(56),
        UNDEAD_STATE_ID => StateRecordSizes::typed(76),
        LEAF_CUT_STATE_ID => StateRecordSizes::typed(LEAF_CUT_STATE_BYTES),
        LEAF_CUT_2_STATE_ID => StateRecordSizes::typed(LEAF_CUT_STATE_BYTES),
        LEAF_CUT_3_STATE_ID => StateRecordSizes::typed(LEAF_CUT_STATE_BYTES),
        KEROSENE_STATE_ID => StateRecordSizes::typed(POISON_STATE_BYTES),
        SWORDSHIP_STATE_ID | SWORDSHIP_2_STATE_ID | SWORDSHIP_3_STATE_ID | SWORDSHIP_4_STATE_ID => StateRecordSizes::typed(SWORDSHIP_STATE_BYTES),
        STRIKE_STATE_ID => StateRecordSizes::typed(BLIND_STATE_BYTES),
        0x353..=0x357 => StateRecordSizes::typed(WUXING_STATE_BYTES),
        POISON_FOG_STATE_ID => StateRecordSizes::typed(POISON_FOG_STATE_BYTES),
        METEOR_ARROW_MASS_SKILL_ID => StateRecordSizes::typed(METEOR_ARROW_STATE_BYTES),
        BLIND_STATE_ID => StateRecordSizes::typed(BLIND_STATE_BYTES),
        KNOCK_OUT_STATE_ID => StateRecordSizes::typed(KNOCK_OUT_STATE_BYTES),
        SPIDER_WEB_STATE_ID => StateRecordSizes::typed(SPIDER_WEB_STATE_BYTES),
        SEAL_STATE_ID => StateRecordSizes::typed(BLIND_STATE_BYTES),
        GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID => StateRecordSizes::typed(GOD_BLESS_STATE_BYTES),
        WEAK_STATE_ID => StateRecordSizes::typed(WEAK_STATE_BYTES),
        SOUL_COLLECT_STATE_ID => StateRecordSizes::typed(SOUL_COLLECT_STATE_BYTES),
        SPRITE_BURN_SKILL_ID => StateRecordSizes::typed(POISON_STATE_BYTES),
        SPIDER_POISON_SKILL_ID => StateRecordSizes::typed(POISON_STATE_BYTES),
        DAUB_POISON_STATE_ID => StateRecordSizes::typed(DAUB_POISON_STATE_BYTES),
        BOSS_BLUE_QUAKE_STATE_ID => StateRecordSizes::typed(BOSS_BLUE_QUAKE_STATE_BYTES),
        KNIGHT_CUT_STATE_ID => StateRecordSizes::typed(KNIGHT_CUT_STATE_BYTES),
        BOA_LOCK_STATE_ID => StateRecordSizes::typed(BOA_LOCK_STATE_BYTES),
        RUSH_STATE_ID => StateRecordSizes::typed(BLIND_STATE_BYTES),
        RUSH_2_STATE_ID => StateRecordSizes::typed(BLIND_STATE_BYTES),
        ROAR_STATE_ID => StateRecordSizes::typed(ROAR_STATE_BYTES),
        PILLAR_STATE_ID => StateRecordSizes::typed(PILLAR_STATE_BYTES),
        RAGE_BREAK_STATE_ID => StateRecordSizes::typed(ATTACK_GAIN_STATE_BYTES),
        FURY_STATE_SKILL_ID => StateRecordSizes::typed(ATTACK_GAIN_STATE_BYTES),
        HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID => StateRecordSizes::typed(HEAL_STATE_BYTES),
        CURE_STATE_SKILL_ID => StateRecordSizes::typed(CURE_STATE_BYTES),
        ENLARGE_FULL_MISS_STATE_ID => StateRecordSizes::typed(ENLARGE_FULL_MISS_STATE_BYTES),
        TAIJI_STATE_ID => StateRecordSizes::typed(ELEMENT_STATE_BYTES),
        ENLARGE_MAX_HP_STATE_ID => StateRecordSizes::typed(MAX_RESOURCE_STATE_BYTES),
        ENLARGE_MAX_MP_STATE_ID => StateRecordSizes::typed(MAX_RESOURCE_STATE_BYTES),
        ORIGIN_STATE_ID => StateRecordSizes::typed(ELEMENT_STATE_BYTES),
        HEARTEN_STATE_ID => StateRecordSizes::typed(HEARTEN_STATE_BYTES),
        AGILITY_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID => StateRecordSizes::typed(PERSISTENT_AGILITY_FAMILY_STATE_BYTES),
        AGILITY_2_SKILL_ID => StateRecordSizes::typed(AGILITY_STATE_2_BYTES),
        CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => StateRecordSizes::typed(CALLOSITY_STATE_BYTES),
        BLOOD_LOSS_STATE_ID => StateRecordSizes::typed(BLOOD_LOSS_STATE_BYTES),
        BOSS_BLUE_FURY_STATE_ID => StateRecordSizes::typed(BOSS_BLUE_FURY_STATE_BYTES),
        0x212..=0x219 => StateRecordSizes::typed(BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES),
        // Исходный Load читает 10 байт, а Save пишет 12. Два дополнительных
        // байта cache не должны сдвигать чтение следующего ID во входе.
        TIAN_SHEN_XIA_FAN_STATE_ID => StateRecordSizes {
            bytes: TIAN_SHEN_XIA_FAN_STATE_BYTES,
            unserialize_bytes: 10,
            normalize: true,
        },
        WANGSHENG_STATE_ID => StateRecordSizes::typed(WANGSHENG_STATE_BYTES),
        POISON_ARROW_SKILL_ID => StateRecordSizes::typed(POISON_STATE_BYTES),
        state_id if state_id == RESTORE_HP_STATE_ID as u32 => StateRecordSizes::typed(CONSUMABLE_RESTORE_STATE_BYTES),
        state_id if state_id == RESTORE_MP_STATE_ID as u32 => StateRecordSizes::typed(CONSUMABLE_RESTORE_STATE_BYTES),
        state_id if is_automatic_restore_state_id(state_id) => StateRecordSizes::typed(AUTOMATIC_RESTORE_STATE_BYTES),
        PARTICULAR_STATE_ID => StateRecordSizes::typed(PARTICULAR_STATE_BYTES),
        state_id if state_id == TEAM_STATE_ID as u32 => StateRecordSizes::typed(CTeamState::serialized_size(payload, cursor)?),
        state_id if ScriptMoveState::serialized_size(state_id as i32).is_some() => StateRecordSizes::typed(ScriptMoveState::serialized_size(state_id as i32)?),
        RIDE_STATE_ID => StateRecordSizes::typed(17 + payload.get(cursor.checked_add(16)?..)?.iter().take(256).position(|byte| *byte == 0)?),
        MACHINE_SHIELD_SKILL_ID => StateRecordSizes::typed(MACHINE_SHIELD_STATE_BYTES),
        MANA_SHIELD_SKILL_ID => StateRecordSizes::typed(MANA_SHIELD_STATE_BYTES),
        LIFE_SHIELD_SKILL_ID => StateRecordSizes::typed(LIFE_SHIELD_STATE_BYTES),
        PROMOTION_STATE_ID => StateRecordSizes::typed(PROMOTION_STATE_BYTES),
        ENERGY_HOLDING_STATE_ID => StateRecordSizes::typed(ENERGY_HOLDING_STATE_BYTES),
        _ => return None,
    })
}

/// Decoder-таблица каталога: ID записи → конструктор целевого enum-каталога
/// без размерного контура. Порядок ветвей повторяет `record_sizes`;
/// конструкторы получают cache-offset записи, у которой ещё нет сдвига
/// асимметричной TianShenXiaFan-нормализации.
fn record_decoder<S: StateRecordTarget>(state_id: u32) -> Option<StateDecoder<S>> {
    Some(match state_id {
        CHANGE_BODY_STATE_ID => |payload, offset, _owner, _factory, _now| {
            ChangeBodyState::decode_at(payload, offset, _now).map(S::change_body)
        },
        EX_STATE_ID => |payload, offset, _owner, _factory, _now| {
            ExtendedState::decode_at(payload, offset, _now).map(S::extended)
        },
        EX_STATE_NEW_ID => |payload, offset, _owner, _factory, _now| {
            ExtendedState::decode_at(payload, offset, _now).map(S::extended)
        },
        UNDEAD_STATE_ID => |payload, offset, _owner, _factory, _now| {
            UndeadState::decode_at(payload, offset, _now).map(S::undead)
        },
        LEAF_CUT_STATE_ID => |payload, offset, _owner, _factory, _now| {
            LeafCutState::decode(payload, offset, _now).ok().map(S::leaf_cut)
        },
        LEAF_CUT_2_STATE_ID => |payload, offset, _owner, _factory, _now| {
            LeafCutState2::decode(payload, offset, _now).ok().map(S::leaf_cut_2)
        },
        LEAF_CUT_3_STATE_ID => |payload, offset, _owner, _factory, _now| {
            LeafCutState3::decode(payload, offset, _now).ok().map(S::leaf_cut_3)
        },
        KEROSENE_STATE_ID => |payload, offset, _owner, _factory, _now| {
            KeroseneState::decode(payload, offset, _now).ok().map(S::kerosene)
        },
        SWORDSHIP_STATE_ID | SWORDSHIP_2_STATE_ID | SWORDSHIP_3_STATE_ID | SWORDSHIP_4_STATE_ID => |payload, offset, _owner, _factory, _now| {
            SwordshipState::decode(payload, offset).ok().map(S::swordship)
        },
        STRIKE_STATE_ID => |payload, offset, _owner, _factory, _now| {
            StrikeState::decode(payload, offset, _now).ok().map(S::strike)
        },
        0x353..=0x357 => |payload, offset, _owner, _factory, _now| {
            WuXingState::decode(payload, offset).ok().map(S::wu_xing)
        },
        POISON_FOG_STATE_ID => |payload, offset, _owner, _factory, _now| {
            PoisonFogState::decode(payload, offset, _now).ok().map(S::poison_fog)
        },
        METEOR_ARROW_MASS_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            MeteorArrowState::decode(payload, offset).ok().map(S::meteor_arrow)
        },
        BLIND_STATE_ID => |payload, offset, _owner, _factory, _now| {
            BlindState::decode(payload, offset, _now).ok().map(S::blind)
        },
        KNOCK_OUT_STATE_ID => |payload, offset, _owner, _factory, _now| {
            KnockOutState::decode(payload, offset, _now).ok().map(S::knock_out)
        },
        SPIDER_WEB_STATE_ID => |payload, offset, _owner, _factory, _now| {
            SpiderWebState::decode(payload, offset, _now).ok().map(S::spider_web)
        },
        SEAL_STATE_ID => |payload, offset, _owner, _factory, _now| {
            SealState::decode(payload, offset, _now).ok().map(S::seal)
        },
        GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID => |payload, offset, _owner, _factory, _now| {
            GodBlessState::decode(payload, offset, _now).ok().map(S::god_bless)
        },
        WEAK_STATE_ID => |payload, offset, _owner, _factory, _now| {
            WeakState::decode(payload, offset, _now).ok().map(S::weak)
        },
        SOUL_COLLECT_STATE_ID => |payload, offset, _owner, _factory, _now| {
            SoulCollectState::decode(payload, offset).ok().map(S::soul_collect)
        },
        SPRITE_BURN_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            SpriteBurnState::decode(payload, offset, _now).ok().map(S::sprite_burn)
        },
        SPIDER_POISON_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            SpiderPoisonState::decode(payload, offset, _now).ok().map(S::spider_poison)
        },
        DAUB_POISON_STATE_ID => |payload, offset, _owner, _factory, _now| {
            DaubPoisonState::decode(payload, offset, _now).ok().map(S::daub_poison)
        },
        BOSS_BLUE_QUAKE_STATE_ID => |payload, offset, _owner, _factory, _now| {
            BossBlueQuakeState::decode(payload, offset, _now).ok().map(S::boss_blue_quake)
        },
        KNIGHT_CUT_STATE_ID => |payload, offset, _owner, _factory, _now| {
            KnightCutState::decode(payload, offset, _now).ok().map(S::knight_cut)
        },
        BOA_LOCK_STATE_ID => |payload, offset, _owner, _factory, _now| {
            BoaLockState::decode(payload, offset, _now).ok().map(S::boa_lock)
        },
        RUSH_STATE_ID => |payload, offset, _owner, _factory, _now| {
            RushState::decode(payload, offset, _now).ok().map(S::rush)
        },
        RUSH_2_STATE_ID => |payload, offset, _owner, _factory, _now| {
            Rush2State::decode(payload, offset, _now).ok().map(S::rush_2)
        },
        ROAR_STATE_ID => |payload, offset, _owner, _factory, _now| {
            RoarState::decode(payload, offset, _now).ok().map(S::roar)
        },
        PILLAR_STATE_ID => |payload, offset, _owner, _factory, _now| {
            PillarState::decode(payload, offset, _now).ok().map(S::pillar)
        },
        RAGE_BREAK_STATE_ID => |payload, offset, _owner, _factory, _now| {
            RageBreakState::decode(payload, offset, _now).ok().map(S::rage_break)
        },
        FURY_STATE_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            FuryState::decode(payload, offset, _now).ok().map(S::fury)
        },
        HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            HealState::decode(payload, offset, _now).ok().map(S::heal)
        },
        CURE_STATE_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            CureState::decode(payload, offset, _now).ok().map(S::cure)
        },
        ENLARGE_FULL_MISS_STATE_ID => |payload, offset, _owner, _factory, _now| {
            EnlargeFullMissState::decode(payload, offset).ok().map(S::enlarge_full_miss)
        },
        TAIJI_STATE_ID => |payload, offset, _owner, _factory, _now| {
            TaiJiState::decode(payload, offset).ok().map(S::tai_ji)
        },
        ENLARGE_MAX_HP_STATE_ID => |payload, offset, _owner, _factory, _now| {
            EnlargeMaxHpState::decode(payload, offset).ok().map(S::enlarge_max_hp)
        },
        ENLARGE_MAX_MP_STATE_ID => |payload, offset, _owner, _factory, _now| {
            EnlargeMaxMpState::decode(payload, offset).ok().map(S::enlarge_max_mp)
        },
        ORIGIN_STATE_ID => |payload, offset, _owner, _factory, _now| {
            OriginState::decode(payload, offset).ok().map(S::origin)
        },
        HEARTEN_STATE_ID => |payload, offset, _owner, _factory, _now| {
            HeartenState::decode(payload, offset, _now).ok().map(S::hearten)
        },
        AGILITY_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            PersistentAgilityFamilyState::decode(payload, offset).ok().map(S::persistent_agility)
        },
        AGILITY_2_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            AgilityState2::decode(payload, offset, _now()).ok().map(S::agility_2)
        },
        CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            CallosityFamilyState::decode(payload, offset, _now()).ok().map(S::callosity)
        },
        BLOOD_LOSS_STATE_ID => |payload, offset, _owner, _factory, _now| {
            BloodLossState::decode(payload, offset, _now).ok().map(S::blood_loss)
        },
        BOSS_BLUE_FURY_STATE_ID => |payload, offset, _owner, _factory, _now| {
            BossBlueFuryState::decode(payload, offset, _now).ok().map(S::boss_blue_fury)
        },
        0x212..=0x219 => |payload, offset, _owner, _factory, _now| {
            BattleFairyAttributeState::decode(payload, offset, _now).ok().map(S::battle_fairy_attribute)
        },
        TIAN_SHEN_XIA_FAN_STATE_ID => |payload, offset, _owner, _factory, _now| {
            TianShenXiaFanState::decode(payload, offset).ok().map(S::tian_shen_xia_fan)
        },
        WANGSHENG_STATE_ID => |payload, offset, _owner, _factory, _now| {
            WangshengState::decode(payload, offset, _now).ok().map(S::wangsheng)
        },
        POISON_ARROW_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            PoisonArrowState::decode(payload, offset, _now).ok().map(S::poison_arrow)
        },
        state_id if state_id == RESTORE_HP_STATE_ID as u32 => |payload, offset, _owner, _factory, _now| {
            ConsumableRestoreState::decode(payload, offset, _now).map(S::consumable_restore)
        },
        state_id if state_id == RESTORE_MP_STATE_ID as u32 => |payload, offset, _owner, _factory, _now| {
            ConsumableRestoreState::decode(payload, offset, _now).map(S::consumable_restore)
        },
        state_id if is_automatic_restore_state_id(state_id) => |payload, offset, _owner, _factory, _now| {
            AutomaticRestoreState::decode(payload, offset, _now()).map(S::automatic_restore)
        },
        PARTICULAR_STATE_ID => |payload, offset, _owner, _factory, _now| {
            ParticularState::decode(payload, offset).ok().map(S::particular)
        },
        state_id if state_id == TEAM_STATE_ID as u32 => |payload, offset, _owner, _factory, _now| {
            CTeamState::decode(payload, offset).ok().map(S::team)
        },
        state_id if ScriptMoveState::serialized_size(state_id as i32).is_some() => |payload, offset, _owner, _factory, _now| {
            ScriptMoveState::decode(payload, offset, _now).ok().map(S::script)
        },
        RIDE_STATE_ID => |payload, offset, _owner, _factory, _now| {
            RideState::decode_at(payload, offset).map(S::ride)
        },
        MACHINE_SHIELD_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            MachineShieldState::decode(payload, offset, _now()).ok().map(|state| S::defense_shield(DefenseShieldState::Machine(state)))
        },
        MANA_SHIELD_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            ManaShieldState::decode(payload, offset, _now()).ok().map(|state| S::defense_shield(DefenseShieldState::Mana(state)))
        },
        LIFE_SHIELD_SKILL_ID => |payload, offset, _owner, _factory, _now| {
            LifeShieldState::decode(payload, offset, _now).ok().map(|state| S::defense_shield(DefenseShieldState::Life(state)))
        },
        PROMOTION_STATE_ID => |payload, offset, _owner, _factory, _now| {
            PromotionState::decode(payload, offset, _now).ok().map(|state| S::defense_shield(DefenseShieldState::Promotion(state)))
        },
        ENERGY_HOLDING_STATE_ID => |payload, offset, _owner, _factory, _now| {
            EnergyHoldingState::decode(payload, offset).ok().map(S::energy_holding)
        },
        _ => return None,
    })
}

/// Читает одну native-запись и добавляет её отдельную Serialize-проекцию в cache.
/// Concrete decoder сразу получает cache-offset: его сохранённые смещения не
/// указывают внутрь исходного непрозрачного хвоста после асимметричной записи.
pub fn decode_state_record_into_cache<S: StateRecordTarget>(
    payload: &[u8],
    offset: usize,
    cache: &mut Vec<u8>,
    owner: ShapeIdentity,
    factory: &CSkillFactory,
    now: &mut dyn FnMut() -> u32,
) -> Option<(S, usize)> {
    let state_id = read_u32(payload, offset)?;
    let sizes = record_sizes(payload, offset, state_id)?;
    let decode = record_decoder::<S>(state_id)?;
    let record = payload.get(offset..offset.checked_add(sizes.unserialize_bytes)?)?;
    let cache_offset = cache.len();
    cache.extend_from_slice(record);
    let Some(state) = decode(cache, cache_offset, owner, factory, now) else {
        cache.truncate(cache_offset);
        return None;
    };
    if sizes.normalize {
        let Some(record) = state.normalized_save_record().filter(|record| record.len() == sizes.bytes) else {
            cache.truncate(cache_offset);
            return None;
        };
        cache.truncate(cache_offset);
        cache.extend_from_slice(&record);
    }
    Some((state, sizes.unserialize_bytes))
}

/// Начала записей нормализованного Serialize-cache, не native Unserialize-input.
pub fn known_state_record_offsets(payload: &[u8]) -> Vec<usize> {
    known_state_record_spans(payload).into_iter().map(|(offset, _)| offset).collect()
}

/// Границы Serialize-cache. Native input продвигает только decode_state_record_into_cache.
pub fn known_state_record_spans(payload: &[u8]) -> Vec<(usize, usize)> {
    let Some(declared_count) = read_u32(payload, 0) else {
        return Vec::new();
    };
    let mut offsets = Vec::new();
    let mut cursor = 4usize;
    for _ in 0..declared_count {
        let Some(state_id) = read_u32(payload, cursor) else { break };
        let Some(sizes) = record_sizes(payload, cursor, state_id) else { break };
        let Some(end) = cursor.checked_add(sizes.bytes).filter(|end| *end <= payload.len()) else { break };
        offsets.push((cursor, sizes.bytes));
        cursor = end;
    }
    offsets
}
