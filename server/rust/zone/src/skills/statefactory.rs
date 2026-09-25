//! Декодирование последовательности состояний из GameSave, перенесённое в Zone skills.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/statefactory.cpp;
//! native-сторона — статический `CStateFactory::Unserialize`
//! (`?Unserialize@CStateFactory@@SAPAVCState@@PAEAAJ@Z`, RVA `0x001D7D00`,
//! VA `0x005D7D00`): чтение declared count, switch по ID записи с переходом
//! к привязке владельца; таблица переходов начинается с `0x32`.
//!
//! Единая запись каталога связывает ID, layout и decoder. Состояния передаются
//! CMoveShape в исходном порядке, включая повторные ID. Неизвестная или
//! усечённая запись останавливает типизацию до спорного участка: исходный хвост
//! и остаток declared count сохраняются в LegacyStateCodec. Это техническая
//! страховка, а не обещание native round-trip повреждённого GameSave.
//! Размер чтения и размер cache-проекции могут различаться; продвижение
//! исходного курсора определяется только читаемым layout.
//!
//! Каталог опирается только на Zone-данные: layout и decoder каждой записи —
//! `effects`, идентичность владельца — `regions`, factory-параметр — соседний
//! `skills::skillfactory`. Единственный внешний шов — целевой enum-каталог
//! payload: он пока принадлежит арене `moveshape` переходного Game
//! (`StateData`), поэтому файл параметризован trait `StateRecordTarget`, а
//! gameserver-фасад сваривает каталог с единым `StateData` без второго
//! перечня игровых ID. Идентификаторы семей без собственного владельца в
//! `effects` (SEAL/RUSH/RUSH2/STRIKE/KEROSENE и навыковые HEAL/SUPER_HEAL/
//! SPIDER_POISON/SPRITE_BURN/POISON_ARROW) — нативные константы их исходных
//! cpp-файлов; они остаются здесь до переезда соответствующих состояний.

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
const SPIDER_POISON_SKILL_ID: u32 = 0x191;
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

/// Сварочный шов с единым enum-каталогом payload декодированных записей.
/// Сегодня его реализует `StateData` арены `moveshape` переходного Game;
/// trait не содержит игровых ID и не дублирует каталог записей, а только
/// передаёт декодированный конкретный payload владельцу арены. После переноса
/// арены состояний impl переедет в Zone вместе с её enum-каталогом.
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

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    let bytes = payload.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

type StateDecoder<S> = fn(&[u8], usize, ShapeIdentity, &CSkillFactory, &mut dyn FnMut() -> u32) -> Option<S>;

struct StateRecordLayout<S> {
    bytes: usize,
    unserialize_bytes: usize,
    decode: StateDecoder<S>,
    normalize: bool,
}

impl<S> StateRecordLayout<S> {
    fn typed(bytes: usize, decode: StateDecoder<S>) -> Self {
        Self { bytes, unserialize_bytes: bytes, decode, normalize: false }
    }
}

fn record_layout<S: StateRecordTarget>(payload: &[u8], cursor: usize, state_id: u32) -> Option<StateRecordLayout<S>> {
    Some(match state_id {
        CHANGE_BODY_STATE_ID => StateRecordLayout::typed(
            124, |payload, offset, _owner, _factory, _now| {
                ChangeBodyState::decode_at(payload, offset, _now).map(S::change_body)
            },
        ),
        EX_STATE_ID => StateRecordLayout::typed(
            44, |payload, offset, _owner, _factory, _now| {
                ExtendedState::decode_at(payload, offset, _now).map(S::extended)
            },
        ),
        EX_STATE_NEW_ID => StateRecordLayout::typed(
            56, |payload, offset, _owner, _factory, _now| {
                ExtendedState::decode_at(payload, offset, _now).map(S::extended)
            },
        ),
        UNDEAD_STATE_ID => StateRecordLayout::typed(
            76, |payload, offset, _owner, _factory, _now| {
                UndeadState::decode_at(payload, offset, _now).map(S::undead)
            },
        ),
        LEAF_CUT_STATE_ID => StateRecordLayout::typed(
            LEAF_CUT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                LeafCutState::decode(payload, offset, _now).ok().map(S::leaf_cut)
            },
        ),
        LEAF_CUT_2_STATE_ID => StateRecordLayout::typed(
            LEAF_CUT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                LeafCutState2::decode(payload, offset, _now).ok().map(S::leaf_cut_2)
            },
        ),
        LEAF_CUT_3_STATE_ID => StateRecordLayout::typed(
            LEAF_CUT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                LeafCutState3::decode(payload, offset, _now).ok().map(S::leaf_cut_3)
            },
        ),
        KEROSENE_STATE_ID => StateRecordLayout::typed(
            POISON_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                KeroseneState::decode(payload, offset, _now).ok().map(S::kerosene)
            },
        ),
        SWORDSHIP_STATE_ID | SWORDSHIP_2_STATE_ID | SWORDSHIP_3_STATE_ID | SWORDSHIP_4_STATE_ID => StateRecordLayout::typed(
            SWORDSHIP_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                SwordshipState::decode(payload, offset).ok().map(S::swordship)
            },
        ),
        STRIKE_STATE_ID => StateRecordLayout::typed(
            BLIND_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                StrikeState::decode(payload, offset, _now).ok().map(S::strike)
            },
        ),
        0x353..=0x357 => StateRecordLayout::typed(
            WUXING_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                WuXingState::decode(payload, offset).ok().map(S::wu_xing)
            },
        ),
        POISON_FOG_STATE_ID => StateRecordLayout::typed(
            POISON_FOG_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                PoisonFogState::decode(payload, offset, _now).ok().map(S::poison_fog)
            },
        ),
        METEOR_ARROW_MASS_SKILL_ID => StateRecordLayout::typed(
            METEOR_ARROW_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                MeteorArrowState::decode(payload, offset).ok().map(S::meteor_arrow)
            },
        ),
        BLIND_STATE_ID => StateRecordLayout::typed(
            BLIND_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                BlindState::decode(payload, offset, _now).ok().map(S::blind)
            },
        ),
        KNOCK_OUT_STATE_ID => StateRecordLayout::typed(
            KNOCK_OUT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                KnockOutState::decode(payload, offset, _now).ok().map(S::knock_out)
            },
        ),
        SPIDER_WEB_STATE_ID => StateRecordLayout::typed(
            SPIDER_WEB_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                SpiderWebState::decode(payload, offset, _now).ok().map(S::spider_web)
            },
        ),
        SEAL_STATE_ID => StateRecordLayout::typed(
            BLIND_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                SealState::decode(payload, offset, _now).ok().map(S::seal)
            },
        ),
        GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID => StateRecordLayout::typed(
            GOD_BLESS_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                GodBlessState::decode(payload, offset, _now).ok().map(S::god_bless)
            },
        ),
        WEAK_STATE_ID => StateRecordLayout::typed(
            WEAK_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                WeakState::decode(payload, offset, _now).ok().map(S::weak)
            },
        ),
        SOUL_COLLECT_STATE_ID => StateRecordLayout::typed(
            SOUL_COLLECT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                SoulCollectState::decode(payload, offset).ok().map(S::soul_collect)
            },
        ),
        SPRITE_BURN_SKILL_ID => StateRecordLayout::typed(
            POISON_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                SpriteBurnState::decode(payload, offset, _now).ok().map(S::sprite_burn)
            },
        ),
        SPIDER_POISON_SKILL_ID => StateRecordLayout::typed(
            POISON_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                SpiderPoisonState::decode(payload, offset, _now).ok().map(S::spider_poison)
            },
        ),
        DAUB_POISON_STATE_ID => StateRecordLayout::typed(
            DAUB_POISON_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                DaubPoisonState::decode(payload, offset, _now).ok().map(S::daub_poison)
            },
        ),
        BOSS_BLUE_QUAKE_STATE_ID => StateRecordLayout::typed(
            BOSS_BLUE_QUAKE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                BossBlueQuakeState::decode(payload, offset, _now).ok().map(S::boss_blue_quake)
            },
        ),
        KNIGHT_CUT_STATE_ID => StateRecordLayout::typed(
            KNIGHT_CUT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                KnightCutState::decode(payload, offset, _now).ok().map(S::knight_cut)
            },
        ),
        BOA_LOCK_STATE_ID => StateRecordLayout::typed(
            BOA_LOCK_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                BoaLockState::decode(payload, offset, _now).ok().map(S::boa_lock)
            },
        ),
        RUSH_STATE_ID => StateRecordLayout::typed(
            BLIND_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                RushState::decode(payload, offset, _now).ok().map(S::rush)
            },
        ),
        RUSH_2_STATE_ID => StateRecordLayout::typed(
            BLIND_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                Rush2State::decode(payload, offset, _now).ok().map(S::rush_2)
            },
        ),
        ROAR_STATE_ID => StateRecordLayout::typed(
            ROAR_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                RoarState::decode(payload, offset, _now).ok().map(S::roar)
            },
        ),
        PILLAR_STATE_ID => StateRecordLayout::typed(
            PILLAR_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                PillarState::decode(payload, offset, _now).ok().map(S::pillar)
            },
        ),
        RAGE_BREAK_STATE_ID => StateRecordLayout::typed(
            ATTACK_GAIN_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                RageBreakState::decode(payload, offset, _now).ok().map(S::rage_break)
            },
        ),
        FURY_STATE_SKILL_ID => StateRecordLayout::typed(
            ATTACK_GAIN_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                FuryState::decode(payload, offset, _now).ok().map(S::fury)
            },
        ),
        HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID => StateRecordLayout::typed(
            HEAL_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                HealState::decode(payload, offset, _now).ok().map(S::heal)
            },
        ),
        CURE_STATE_SKILL_ID => StateRecordLayout::typed(
            CURE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                CureState::decode(payload, offset, _now).ok().map(S::cure)
            },
        ),
        ENLARGE_FULL_MISS_STATE_ID => StateRecordLayout::typed(
            ENLARGE_FULL_MISS_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                EnlargeFullMissState::decode(payload, offset).ok().map(S::enlarge_full_miss)
            },
        ),
        TAIJI_STATE_ID => StateRecordLayout::typed(
            ELEMENT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                TaiJiState::decode(payload, offset).ok().map(S::tai_ji)
            },
        ),
        ENLARGE_MAX_HP_STATE_ID => StateRecordLayout::typed(
            MAX_RESOURCE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                EnlargeMaxHpState::decode(payload, offset).ok().map(S::enlarge_max_hp)
            },
        ),
        ENLARGE_MAX_MP_STATE_ID => StateRecordLayout::typed(
            MAX_RESOURCE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                EnlargeMaxMpState::decode(payload, offset).ok().map(S::enlarge_max_mp)
            },
        ),
        ORIGIN_STATE_ID => StateRecordLayout::typed(
            ELEMENT_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                OriginState::decode(payload, offset).ok().map(S::origin)
            },
        ),
        HEARTEN_STATE_ID => StateRecordLayout::typed(
            HEARTEN_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                HeartenState::decode(payload, offset, _now).ok().map(S::hearten)
            },
        ),
        AGILITY_SKILL_ID | NATURAL_SKILL_ID | RAPTURE_SKILL_ID => StateRecordLayout::typed(
            PERSISTENT_AGILITY_FAMILY_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                PersistentAgilityFamilyState::decode(payload, offset).ok().map(S::persistent_agility)
            },
        ),
        AGILITY_2_SKILL_ID => StateRecordLayout::typed(
            AGILITY_STATE_2_BYTES, |payload, offset, _owner, _factory, _now| {
                AgilityState2::decode(payload, offset, _now()).ok().map(S::agility_2)
            },
        ),
        CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => StateRecordLayout::typed(
            CALLOSITY_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                CallosityFamilyState::decode(payload, offset, _now()).ok().map(S::callosity)
            },
        ),
        BLOOD_LOSS_STATE_ID => StateRecordLayout::typed(
            BLOOD_LOSS_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                BloodLossState::decode(payload, offset, _now).ok().map(S::blood_loss)
            },
        ),
        BOSS_BLUE_FURY_STATE_ID => StateRecordLayout::typed(
            BOSS_BLUE_FURY_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                BossBlueFuryState::decode(payload, offset, _now).ok().map(S::boss_blue_fury)
            },
        ),
        0x212..=0x219 => StateRecordLayout::typed(
            BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                BattleFairyAttributeState::decode(payload, offset, _now).ok().map(S::battle_fairy_attribute)
            },
        ),
        // Исходный Load читает 10 байт, а Save пишет 12. Два дополнительных
        // байта cache не должны сдвигать чтение следующего ID во входе.
        TIAN_SHEN_XIA_FAN_STATE_ID => StateRecordLayout {
            bytes: TIAN_SHEN_XIA_FAN_STATE_BYTES,
            unserialize_bytes: 10,
            decode: |payload, offset, _owner, _factory, _now| {
                TianShenXiaFanState::decode(payload, offset).ok().map(S::tian_shen_xia_fan)
            },
            normalize: true,
        },
        WANGSHENG_STATE_ID => StateRecordLayout::typed(
            WANGSHENG_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                WangshengState::decode(payload, offset, _now).ok().map(S::wangsheng)
            },
        ),
        POISON_ARROW_SKILL_ID => StateRecordLayout::typed(
            POISON_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                PoisonArrowState::decode(payload, offset, _now).ok().map(S::poison_arrow)
            },
        ),
        state_id if state_id == RESTORE_HP_STATE_ID as u32 => StateRecordLayout::typed(
            CONSUMABLE_RESTORE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                ConsumableRestoreState::decode(payload, offset, _now).map(S::consumable_restore)
            },
        ),
        state_id if state_id == RESTORE_MP_STATE_ID as u32 => StateRecordLayout::typed(
            CONSUMABLE_RESTORE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                ConsumableRestoreState::decode(payload, offset, _now).map(S::consumable_restore)
            },
        ),
        state_id if is_automatic_restore_state_id(state_id) => StateRecordLayout::typed(
            AUTOMATIC_RESTORE_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                AutomaticRestoreState::decode(payload, offset, _now()).map(S::automatic_restore)
            },
        ),
        PARTICULAR_STATE_ID => StateRecordLayout::typed(
            PARTICULAR_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                ParticularState::decode(payload, offset).ok().map(S::particular)
            },
        ),
        state_id if state_id == TEAM_STATE_ID as u32 => StateRecordLayout::typed(
            CTeamState::serialized_size(payload, cursor)?, |payload, offset, _owner, _factory, _now| {
                CTeamState::decode(payload, offset).ok().map(S::team)
            },
        ),
        state_id if ScriptMoveState::serialized_size(state_id as i32).is_some() => StateRecordLayout::typed(
            ScriptMoveState::serialized_size(state_id as i32)?, |payload, offset, _owner, _factory, _now| {
                ScriptMoveState::decode(payload, offset, _now).ok().map(S::script)
            },
        ),
        RIDE_STATE_ID => StateRecordLayout::typed(
            17 + payload.get(cursor.checked_add(16)?..)?.iter().take(256).position(|byte| *byte == 0)?, |payload, offset, _owner, _factory, _now| {
                RideState::decode_at(payload, offset).map(S::ride)
            },
        ),
        MACHINE_SHIELD_SKILL_ID => StateRecordLayout::typed(
            MACHINE_SHIELD_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                MachineShieldState::decode(payload, offset, _now()).ok().map(|state| S::defense_shield(DefenseShieldState::Machine(state)))
            },
        ),
        MANA_SHIELD_SKILL_ID => StateRecordLayout::typed(
            MANA_SHIELD_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                ManaShieldState::decode(payload, offset, _now()).ok().map(|state| S::defense_shield(DefenseShieldState::Mana(state)))
            },
        ),
        LIFE_SHIELD_SKILL_ID => StateRecordLayout::typed(
            LIFE_SHIELD_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                LifeShieldState::decode(payload, offset, _now).ok().map(|state| S::defense_shield(DefenseShieldState::Life(state)))
            },
        ),
        PROMOTION_STATE_ID => StateRecordLayout::typed(
            PROMOTION_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                PromotionState::decode(payload, offset, _now).ok().map(|state| S::defense_shield(DefenseShieldState::Promotion(state)))
            },
        ),
        ENERGY_HOLDING_STATE_ID => StateRecordLayout::typed(
            ENERGY_HOLDING_STATE_BYTES, |payload, offset, _owner, _factory, _now| {
                EnergyHoldingState::decode(payload, offset).ok().map(S::energy_holding)
            },
        ),
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
    let layout = record_layout::<S>(payload, offset, read_u32(payload, offset)?)?;
    let record = payload.get(offset..offset.checked_add(layout.unserialize_bytes)?)?;
    let cache_offset = cache.len();
    cache.extend_from_slice(record);
    let Some(state) = (layout.decode)(cache, cache_offset, owner, factory, now) else {
        cache.truncate(cache_offset);
        return None;
    };
    if layout.normalize {
        let Some(record) = state.normalized_save_record().filter(|record| record.len() == layout.bytes) else {
            cache.truncate(cache_offset);
            return None;
        };
        cache.truncate(cache_offset);
        cache.extend_from_slice(&record);
    }
    Some((state, layout.unserialize_bytes))
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
        let Some(layout) = record_layout::<Unrecorded>(payload, cursor, state_id) else { break };
        let Some(end) = cursor.checked_add(layout.bytes).filter(|end| *end <= payload.len()) else { break };
        offsets.push((cursor, layout.bytes));
        cursor = end;
    }
    offsets
}

/// Пустая цель размерного обхода: layout-таблица одна и та же, но её
/// декодеры здесь не вызываются, поэтому конструкторы не реализованы.
enum Unrecorded {}

impl StateRecordTarget for Unrecorded {
    fn change_body(_: ChangeBodyState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn extended(_: ExtendedState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn undead(_: UndeadState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn leaf_cut(_: LeafCutState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn leaf_cut_2(_: LeafCutState2) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn leaf_cut_3(_: LeafCutState3) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn kerosene(_: KeroseneState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn swordship(_: SwordshipState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn strike(_: StrikeState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn wu_xing(_: WuXingState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn poison_fog(_: PoisonFogState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn meteor_arrow(_: MeteorArrowState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn blind(_: BlindState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn knock_out(_: KnockOutState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn spider_web(_: SpiderWebState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn seal(_: SealState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn god_bless(_: GodBlessState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn weak(_: WeakState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn soul_collect(_: SoulCollectState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn sprite_burn(_: SpriteBurnState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn spider_poison(_: SpiderPoisonState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn daub_poison(_: DaubPoisonState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn boss_blue_quake(_: BossBlueQuakeState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn knight_cut(_: KnightCutState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn boa_lock(_: BoaLockState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn rush(_: RushState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn rush_2(_: Rush2State) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn roar(_: RoarState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn pillar(_: PillarState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn rage_break(_: RageBreakState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn fury(_: FuryState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn heal(_: HealState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn cure(_: CureState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn enlarge_full_miss(_: EnlargeFullMissState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn tai_ji(_: TaiJiState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn enlarge_max_hp(_: EnlargeMaxHpState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn enlarge_max_mp(_: EnlargeMaxMpState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn origin(_: OriginState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn hearten(_: HeartenState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn persistent_agility(_: PersistentAgilityFamilyState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn agility_2(_: AgilityState2) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn callosity(_: CallosityFamilyState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn blood_loss(_: BloodLossState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn boss_blue_fury(_: BossBlueFuryState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn battle_fairy_attribute(_: BattleFairyAttributeState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn tian_shen_xia_fan(_: TianShenXiaFanState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn wangsheng(_: WangshengState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn poison_arrow(_: PoisonArrowState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn consumable_restore(_: ConsumableRestoreState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn automatic_restore(_: AutomaticRestoreState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn particular(_: ParticularState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn team(_: CTeamState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn script(_: ScriptMoveState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn ride(_: RideState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn defense_shield(_: DefenseShieldState) -> Self { unimplemented!("размерный обход не декодирует запись") }
    fn energy_holding(_: EnergyHoldingState) -> Self { unimplemented!("размерный обход не декодирует запись") }
}
