//! Клиентская проекция живых состояний (значения time/additional/team_name
//! 56 вариантов enum-каталога) и runtime-план их visual; порядок полей записи
//! и двухпроходный писатель — в соседнем `snapshot`.
//!
//! Исходный владелец PDB: `appserver/states/state.cpp` (getters
//! `CState::Serialize`-семейства). Enum-каталог payload (`StateData`) живёт
//! рядом в `storage` арены состояний.
//!
//! Открытая неувязка (DISCREPANCY, поведение не менялось): щиты DefenseShield
//! Life/Machine/Mana пишут additional = `life()`, у оригинала vtable-слот — 0;
//! вопрос передан владельцу, исправление — отдельным решением.
//!
//! Доказательства: docs/reconstruction/gameserver-skills.md#statecatalog-клиентская-проекция-состояний

use crate::effects::{CVisualEffect, DefenseShieldState, GOD_BLESS_STATE_2_ID};

use super::storage::StateData;

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

/// Клиентская запись одного состояния из `AddToByteArray_ForClient`: 56
/// вариантов в порядке прежнего callback-каталога переходного Game. Пустая
/// client-clause оригинального каталога даёт нулевую запись; часы и состав
/// команды получает только вариант, читавший их в исходных getters.
pub fn state_client_record<'a>(
    state: &'a StateData,
    team_member_count: usize,
    now: &mut dyn FnMut() -> u32,
) -> StateClientRecord<'a> {
    match state {
        StateData::PersistentAgility(_) => StateClientRecord::default(),
        StateData::TaiJi(_) => StateClientRecord::default(),
        StateData::EnlargeFullMiss(_) => StateClientRecord::default(),
        StateData::EnlargeMaxHp(_) => StateClientRecord::default(),
        StateData::EnlargeMaxMp(_) => StateClientRecord::default(),
        StateData::Origin(_) => StateClientRecord::default(),
        StateData::MeteorArrow(state) => StateClientRecord { additional: state.additional_data() as u32, ..StateClientRecord::default() },
        StateData::EnergyHolding(_) => StateClientRecord::default(),
        StateData::SoulCollect(state) => StateClientRecord { additional: state.souls() as u32, ..StateClientRecord::default() },
        StateData::Swordship(_) => StateClientRecord::default(),
        StateData::WuXing(_) => StateClientRecord::default(),
        StateData::Agility2(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Callosity(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Hearten(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::RageBreak(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Pillar(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::TianShenXiaFan(_) => StateClientRecord::default(),
        StateData::Wangsheng(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Blind(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Rush(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Rush2(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::KnockOut(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::KnightCut(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::SpiderWeb(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Seal(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Strike(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Heal(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::PoisonArrow(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::SpiderPoison(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::SpriteBurn(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::BloodLoss(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::LeafCut(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::LeafCut2(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::LeafCut3(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Kerosene(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Cure(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::BossBlueQuake(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::BoaLock(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::GodBless(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Roar(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Weak(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::Fury(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::BossBlueFury(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::PoisonFog(state) => StateClientRecord::timed(state.client_time(now) as i32),
        StateData::BattleFairyAttribute(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::DefenseShield(state) => {
            // Машинная сверка `AddToByteArray_ForClient` (RVA 0xCDD30):
            // vtable+0x38 (GetAdditionalData) у CLife/CMachine/CMana указывает
            // на базовую `xor eax,eax; ret` (RVA 0x201200) — оригинал остаток
            // щита в клиентский снимок арены не пишет. Значение `life`
            // принадлежит layout DB-`Serialize`, а не клиентской записи.
            let (time, additional) = match state {
                DefenseShieldState::Life(state) => (state.client_time(now), 0),
                DefenseShieldState::Machine(state) => (state.client_time(now), 0),
                DefenseShieldState::Mana(state) => (state.client_time(now), 0),
                DefenseShieldState::Promotion(state) => (state.client_time(now), 0),
            };
            StateClientRecord { time, additional, team_name: None }
        }
        StateData::DaubPoison(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::AutomaticRestore(_) => StateClientRecord::default(),
        StateData::ConsumableRestore(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Particular(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None },
        StateData::Team(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(team_member_count), team_name: Some(state.team_name()) },
        StateData::Script(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::ChangeBody(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Extended(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Undead(state) => StateClientRecord::timed(state.client_state_time(now) as i32),
        StateData::Ride(state) => StateClientRecord { time: state.client_state_time(), additional: state.additional_data(), team_name: None },
    }
}

/// План visual зарегистрированного после runtime Begin состояния: пара
/// loop/updated прежнего callback-каталога переходного Game. Варианты без
/// visual-ресурса возвращают None; прочие получают общий базовый план,
/// one-shot — немедленное завершение через update.
pub fn registered_runtime_state_visual(state: &StateData) -> Option<CVisualEffect> {
    let (loop_value, updated) = match state {
        StateData::PersistentAgility(_) => Some((1, false)),
        StateData::TaiJi(_) => None,
        StateData::EnlargeFullMiss(_) => None,
        StateData::EnlargeMaxHp(_) => None,
        StateData::EnlargeMaxMp(_) => None,
        StateData::Origin(_) => None,
        StateData::MeteorArrow(_) => Some((1, false)),
        StateData::EnergyHolding(_) => Some((1, false)),
        StateData::SoulCollect(_) => Some((1, false)),
        StateData::Swordship(_) => None,
        StateData::WuXing(_) => None,
        StateData::Agility2(_) => Some((0, true)),
        StateData::Callosity(_) => Some((1, false)),
        StateData::Hearten(_) => Some((1, false)),
        StateData::RageBreak(_) => Some((1, false)),
        StateData::Pillar(_) => Some((1, false)),
        StateData::TianShenXiaFan(_) => Some((1, false)),
        StateData::Wangsheng(_) => Some((1, false)),
        StateData::Blind(_) => Some((1, true)),
        StateData::Rush(_) => Some((1, true)),
        StateData::Rush2(_) => Some((1, true)),
        StateData::KnockOut(_) => Some((1, false)),
        StateData::KnightCut(_) => Some((1, false)),
        StateData::SpiderWeb(_) => Some((1, false)),
        StateData::Seal(_) => Some((1, false)),
        StateData::Strike(_) => Some((1, false)),
        StateData::Heal(_) => Some((1, false)),
        StateData::PoisonArrow(_) => Some((1, false)),
        StateData::SpiderPoison(_) => Some((1, false)),
        StateData::SpriteBurn(_) => Some((1, false)),
        StateData::BloodLoss(_) => Some((1, false)),
        StateData::LeafCut(_) => Some((1, false)),
        StateData::LeafCut2(_) => Some((1, false)),
        StateData::LeafCut3(_) => Some((1, false)),
        StateData::Kerosene(_) => Some((1, false)),
        StateData::Cure(_) => Some((1, false)),
        StateData::BossBlueQuake(_) => Some((1, false)),
        StateData::BoaLock(_) => Some((1, false)),
        StateData::GodBless(state) => Some((if state.skill_id() == GOD_BLESS_STATE_2_ID { 0 } else { 1 }, false)),
        StateData::Roar(_) => Some((1, false)),
        StateData::Weak(_) => Some((1, false)),
        StateData::Fury(_) => Some((1, false)),
        StateData::BossBlueFury(_) => Some((1, false)),
        StateData::PoisonFog(_) => Some((1, false)),
        StateData::BattleFairyAttribute(_) => Some((1, false)),
        StateData::DefenseShield(state) => {
            let once = matches!(state, DefenseShieldState::Promotion(_));
            Some((if once { 0 } else { 1 }, once))
        }
        StateData::DaubPoison(_) => Some((1, false)),
        StateData::AutomaticRestore(_) => Some((1, false)),
        StateData::ConsumableRestore(_) => Some((1, false)),
        StateData::Particular(_) => Some((1, true)),
        StateData::Team(_) => Some((1, true)),
        StateData::Script(state) => Some((if state.is_auto_protect() { 1 } else { 0 }, false)),
        StateData::ChangeBody(_) => Some((1, false)),
        StateData::Extended(_) => Some((1, false)),
        StateData::Undead(_) => Some((1, false)),
        StateData::Ride(_) => Some((1, false)),
    }?;
    let mut visual = CVisualEffect::new();
    visual.begin_visual_effect(loop_value);
    if updated { visual.update_visual_effect(); }
    Some(visual)
}
