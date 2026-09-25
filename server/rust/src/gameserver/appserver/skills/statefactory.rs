//! Каталог записей состояний GameSave перенесён в Zone `skills::statefactory`.
//! Здесь его реэкспорт для старого пакета и сварка с единым enum-каталогом
//! payload `StateData` арены `moveshape`, которая пока остаётся в переходном
//! Game; при переносе арены состояний блок impl уйдёт в Zone вместе с `StateData`.

pub(crate) use nebokrai_zone::skills::statefactory::*;

use crate::gameserver::appserver::moveshape::StateData;
use nebokrai_zone::effects::{
    AgilityState2, AutomaticRestoreState, BattleFairyAttributeState, BlindState, BloodLossState,
    BoaLockState, BossBlueFuryState, BossBlueQuakeState, CTeamState, CallosityFamilyState,
    ChangeBodyState, ConsumableRestoreState, CureState, DaubPoisonState, DefenseShieldState,
    EnergyHoldingState, EnlargeFullMissState, EnlargeMaxHpState, EnlargeMaxMpState, ExtendedState,
    FuryState, GodBlessState, HealState, HeartenState, KnightCutState, KnockOutState, LeafCutState,
    MeteorArrowState, OriginState, ParticularState, PersistentAgilityFamilyState, PillarState,
    PoisonFogState, RageBreakState, RideState, RoarState, ScriptMoveState, SoulCollectState,
    SpiderWebState,
    SwordshipState, TaiJiState, TianShenXiaFanState, UndeadState, WangshengState, WeakState,
    WuXingState,
};

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
