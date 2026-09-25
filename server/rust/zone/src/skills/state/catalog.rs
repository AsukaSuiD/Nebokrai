//! Клиентская проекция живых состояний и runtime-план их visual, перенесённые
//! из переходного Game (`appserver/states/state.cpp`, клиентские getters
//! `CState::Serialize`-семейства) в Zone skills. Источник: пара
//! `gameserver.exe` SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ `GameServer.pdb` RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2
//! (CodeView RSDS GUID+age совпадают).
//!
//! Порядок полей записи (ID, time, additional, имя Team) принадлежит проходу
//! `CMoveShape::AddToByteArray_ForClient` (0x004CDD30, остаётся у владельца
//! hub moveshape); здесь только значения time/additional/team_name каждого
//! из 56 вариантов payload и план loop/updated `CVisualEffect` после runtime
//! Begin. PARTIAL: построчная машинная сверка значений в этом шаге не
//! повторялась — формулы getters сохраняют статусы своих владельцев в
//! `effects`, а порядок записей не переуверен, так как sender не менялся.
//! Присутствие пабликов `?Serialize@CState@@...` (RVA 0x1DAD00 по конвенции
//! pubs-json, +0x1000 к Ghidra-шкале; VA карты 0x005DAD00),
//! `?AddToByteArray_ForClient@CMoveShape@@...` (0xCCD30) и
//! `?GetTeamName@CTeamState@@QAEPADXZ` (0x1BF290) подтверждено по S_PUB32
//! (`.local/evidence/game-pubs.json`).
//!
//! Сварка: единый enum-каталог payload (`StateData`) пока принадлежит арене
//! `moveshape` переходного Game и переедет шагом B переноса хранилища
//! (`moveshape/state_storage.rs`). До переезда каталог обобщён над узким
//! трейтом `StateClientPayload`, отдающим заимствованный вид вариантов
//! `StatePayloadView`; impl для `StateData` живёт рядом со старым владельцем
//! (`src/gameserver/appserver/states/state.rs`) и снимается шагом B, после
//! чего оба match пойдут прямо по `&StateData` без трейта и вида.

use crate::effects::{
    AgilityState2, AutomaticRestoreState, BattleFairyAttributeState, BlindState, BloodLossState,
    BoaLockState, BossBlueFuryState, BossBlueQuakeState, CTeamState, CVisualEffect,
    CallosityFamilyState, ChangeBodyState, ConsumableRestoreState, CureState, DaubPoisonState,
    DefenseShieldState, EnergyHoldingState, EnlargeFullMissState, EnlargeMaxHpState,
    EnlargeMaxMpState, ExtendedState, FuryState, GOD_BLESS_STATE_2_ID, GodBlessState, HealState,
    HeartenState, KnightCutState, KnockOutState, LeafCutState, MeteorArrowState, OriginState,
    ParticularState, PersistentAgilityFamilyState, PillarState, PoisonFogState, RageBreakState,
    RideState, RoarState, ScriptMoveState, SoulCollectState, SpiderWebState, SwordshipState,
    TaiJiState, TianShenXiaFanState, UndeadState, WangshengState, WeakState, WuXingState,
};

use crate::skills::statefactory::{
    KeroseneState, LeafCutState2, LeafCutState3, PoisonArrowState, Rush2State, RushState,
    SealState, SpiderPoisonState, SpriteBurnState, StrikeState,
};

/// Заимствованная клиентская проекция одного живого экземпляра, без DB Serialize.
/// Только Team дописывает имя после общей тройки ID/time/additional.
#[derive(Clone, Copy, Debug, Default)]
pub struct StateClientRecord<'a> {
    pub time: i32,
    pub additional: u32,
    pub team_name: Option<&'a [u8]>,
}

impl StateClientRecord<'_> {
    fn timed(time: i32) -> Self {
        Self { time, ..Self::default() }
    }
}

/// Сварочный шов с hub-enum-каталогом payload состояний. Сегодня его реализует
/// `StateData` арены `moveshape` переходного Game; единственный метод отдаёт
/// заимствованный вид вариантов, которого достаточно обоим клиентским
/// каталогам. После переноса арены состояний impl переедет в Zone вместе с
/// enum-каталогом, и шов вместе с `StatePayloadView` будет удалён.
pub trait StateClientPayload<'a> {
    fn state_payload_view(&'a self) -> StatePayloadView<'a>;
}

/// Заимствованный вид вариантов единого enum-каталога payload: тот же перечень
/// из 56 вариантов по ссылке, без владения данными и без второго списка ID.
pub enum StatePayloadView<'a> {
    PersistentAgility(&'a PersistentAgilityFamilyState),
    TaiJi(&'a TaiJiState),
    EnlargeFullMiss(&'a EnlargeFullMissState),
    EnlargeMaxHp(&'a EnlargeMaxHpState),
    EnlargeMaxMp(&'a EnlargeMaxMpState),
    Origin(&'a OriginState),
    MeteorArrow(&'a MeteorArrowState),
    EnergyHolding(&'a EnergyHoldingState),
    SoulCollect(&'a SoulCollectState),
    Swordship(&'a SwordshipState),
    WuXing(&'a WuXingState),
    Agility2(&'a AgilityState2),
    Callosity(&'a CallosityFamilyState),
    Hearten(&'a HeartenState),
    RageBreak(&'a RageBreakState),
    Pillar(&'a PillarState),
    TianShenXiaFan(&'a TianShenXiaFanState),
    Wangsheng(&'a WangshengState),
    Blind(&'a BlindState),
    Rush(&'a RushState),
    Rush2(&'a Rush2State),
    KnockOut(&'a KnockOutState),
    KnightCut(&'a KnightCutState),
    SpiderWeb(&'a SpiderWebState),
    Seal(&'a SealState),
    Strike(&'a StrikeState),
    Heal(&'a HealState),
    PoisonArrow(&'a PoisonArrowState),
    SpiderPoison(&'a SpiderPoisonState),
    SpriteBurn(&'a SpriteBurnState),
    BloodLoss(&'a BloodLossState),
    LeafCut(&'a LeafCutState),
    LeafCut2(&'a LeafCutState2),
    LeafCut3(&'a LeafCutState3),
    Kerosene(&'a KeroseneState),
    Cure(&'a CureState),
    BossBlueQuake(&'a BossBlueQuakeState),
    BoaLock(&'a BoaLockState),
    GodBless(&'a GodBlessState),
    Roar(&'a RoarState),
    Weak(&'a WeakState),
    Fury(&'a FuryState),
    BossBlueFury(&'a BossBlueFuryState),
    PoisonFog(&'a PoisonFogState),
    BattleFairyAttribute(&'a BattleFairyAttributeState),
    DefenseShield(&'a DefenseShieldState),
    DaubPoison(&'a DaubPoisonState),
    AutomaticRestore(&'a AutomaticRestoreState),
    ConsumableRestore(&'a ConsumableRestoreState),
    Particular(&'a ParticularState),
    Team(&'a CTeamState),
    Script(&'a ScriptMoveState),
    ChangeBody(&'a ChangeBodyState),
    Extended(&'a ExtendedState),
    Undead(&'a UndeadState),
    Ride(&'a RideState),
}

/// Клиентская запись одного состояния из `AddToByteArray_ForClient`: 56
/// вариантов в порядке прежнего callback-каталога переходного Game. Пустая
/// client-clause оригинального каталога даёт нулевую запись; часы и состав
/// команды получает только вариант, читавший их в исходных getters.
pub fn state_client_record<'a, S: StateClientPayload<'a>>(
    state: &'a S,
    team_member_count: usize,
    now: &mut dyn FnMut() -> u32,
) -> StateClientRecord<'a> {
    match state.state_payload_view() {
        StatePayloadView::PersistentAgility(_) => StateClientRecord::default(),
        StatePayloadView::TaiJi(_) => StateClientRecord::default(),
        StatePayloadView::EnlargeFullMiss(_) => StateClientRecord::default(),
        StatePayloadView::EnlargeMaxHp(_) => StateClientRecord::default(),
        StatePayloadView::EnlargeMaxMp(_) => StateClientRecord::default(),
        StatePayloadView::Origin(_) => StateClientRecord::default(),
        StatePayloadView::MeteorArrow(state) => StateClientRecord { additional: state.additional_data() as u32, ..StateClientRecord::default() },
        StatePayloadView::EnergyHolding(_) => StateClientRecord::default(),
        StatePayloadView::SoulCollect(state) => StateClientRecord { additional: state.souls() as u32, ..StateClientRecord::default() },
        StatePayloadView::Swordship(_) => StateClientRecord::default(),
        StatePayloadView::WuXing(_) => StateClientRecord::default(),
        StatePayloadView::Agility2(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Callosity(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Hearten(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::RageBreak(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Pillar(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::TianShenXiaFan(_) => StateClientRecord::default(),
        StatePayloadView::Wangsheng(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Blind(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Rush(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Rush2(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::KnockOut(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::KnightCut(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::SpiderWeb(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Seal(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Strike(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Heal(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::PoisonArrow(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::SpiderPoison(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::SpriteBurn(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::BloodLoss(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::LeafCut(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::LeafCut2(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::LeafCut3(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Kerosene(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Cure(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::BossBlueQuake(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::BoaLock(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::GodBless(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Roar(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Weak(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::Fury(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::BossBlueFury(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::PoisonFog(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StatePayloadView::BattleFairyAttribute(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::DefenseShield(state) => {
            let (time, additional) = match state {
                DefenseShieldState::Life(state) => (state.client_time(now), state.life() as u32),
                DefenseShieldState::Machine(state) => (state.client_time(now), state.life() as u32),
                DefenseShieldState::Mana(state) => (state.client_time(now), state.life() as u32),
                DefenseShieldState::Promotion(state) => (state.client_time(now), 0),
            };
            StateClientRecord { time, additional, team_name: None }
        }
        StatePayloadView::DaubPoison(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::AutomaticRestore(_) => StateClientRecord::default(),
        StatePayloadView::ConsumableRestore(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Particular(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None },
        StatePayloadView::Team(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(team_member_count), team_name: Some(state.team_name()) },
        StatePayloadView::Script(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::ChangeBody(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Extended(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Undead(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StatePayloadView::Ride(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None },
    }
}

/// План visual зарегистрированного после runtime Begin состояния: пара
/// loop/updated прежнего callback-каталога переходного Game. Варианты без
/// visual-ресурса возвращают None; прочие получают общий базовый план,
/// one-shot — немедленное завершение через update.
pub fn registered_runtime_state_visual<'a, S: StateClientPayload<'a>>(state: &'a S) -> Option<CVisualEffect> {
    let (loop_value, updated) = match state.state_payload_view() {
        StatePayloadView::PersistentAgility(_) => Some((1, false)),
        StatePayloadView::TaiJi(_) => None,
        StatePayloadView::EnlargeFullMiss(_) => None,
        StatePayloadView::EnlargeMaxHp(_) => None,
        StatePayloadView::EnlargeMaxMp(_) => None,
        StatePayloadView::Origin(_) => None,
        StatePayloadView::MeteorArrow(_) => Some((1, false)),
        StatePayloadView::EnergyHolding(_) => Some((1, false)),
        StatePayloadView::SoulCollect(_) => Some((1, false)),
        StatePayloadView::Swordship(_) => None,
        StatePayloadView::WuXing(_) => None,
        StatePayloadView::Agility2(_) => Some((0, true)),
        StatePayloadView::Callosity(_) => Some((1, false)),
        StatePayloadView::Hearten(_) => Some((1, false)),
        StatePayloadView::RageBreak(_) => Some((1, false)),
        StatePayloadView::Pillar(_) => Some((1, false)),
        StatePayloadView::TianShenXiaFan(_) => Some((1, false)),
        StatePayloadView::Wangsheng(_) => Some((1, false)),
        StatePayloadView::Blind(_) => Some((1, true)),
        StatePayloadView::Rush(_) => Some((1, true)),
        StatePayloadView::Rush2(_) => Some((1, true)),
        StatePayloadView::KnockOut(_) => Some((1, false)),
        StatePayloadView::KnightCut(_) => Some((1, false)),
        StatePayloadView::SpiderWeb(_) => Some((1, false)),
        StatePayloadView::Seal(_) => Some((1, false)),
        StatePayloadView::Strike(_) => Some((1, false)),
        StatePayloadView::Heal(_) => Some((1, false)),
        StatePayloadView::PoisonArrow(_) => Some((1, false)),
        StatePayloadView::SpiderPoison(_) => Some((1, false)),
        StatePayloadView::SpriteBurn(_) => Some((1, false)),
        StatePayloadView::BloodLoss(_) => Some((1, false)),
        StatePayloadView::LeafCut(_) => Some((1, false)),
        StatePayloadView::LeafCut2(_) => Some((1, false)),
        StatePayloadView::LeafCut3(_) => Some((1, false)),
        StatePayloadView::Kerosene(_) => Some((1, false)),
        StatePayloadView::Cure(_) => Some((1, false)),
        StatePayloadView::BossBlueQuake(_) => Some((1, false)),
        StatePayloadView::BoaLock(_) => Some((1, false)),
        StatePayloadView::GodBless(state) => Some((if state.skill_id() == GOD_BLESS_STATE_2_ID { 0 } else { 1 }, false)),
        StatePayloadView::Roar(_) => Some((1, false)),
        StatePayloadView::Weak(_) => Some((1, false)),
        StatePayloadView::Fury(_) => Some((1, false)),
        StatePayloadView::BossBlueFury(_) => Some((1, false)),
        StatePayloadView::PoisonFog(_) => Some((1, false)),
        StatePayloadView::BattleFairyAttribute(_) => Some((1, false)),
        StatePayloadView::DefenseShield(state) => {
            let once = matches!(state, DefenseShieldState::Promotion(_));
            Some((if once { 0 } else { 1 }, once))
        }
        StatePayloadView::DaubPoison(_) => Some((1, false)),
        StatePayloadView::AutomaticRestore(_) => Some((1, false)),
        StatePayloadView::ConsumableRestore(_) => Some((1, false)),
        StatePayloadView::Particular(_) => Some((1, true)),
        StatePayloadView::Team(_) => Some((1, true)),
        StatePayloadView::Script(state) => Some((if state.is_auto_protect() { 1 } else { 0 }, false)),
        StatePayloadView::ChangeBody(_) => Some((1, false)),
        StatePayloadView::Extended(_) => Some((1, false)),
        StatePayloadView::Undead(_) => Some((1, false)),
        StatePayloadView::Ride(_) => Some((1, false)),
    }?;
    let mut visual = CVisualEffect::new();
    visual.begin_visual_effect(loop_value);
    if updated { visual.update_visual_effect(); }
    Some(visual)
}
