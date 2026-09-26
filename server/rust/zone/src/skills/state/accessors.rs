//! Читающие проекции арены применённых состояний `CMoveShape` из
//! GameServer.exe/GameServer.pdb (пара gameserver.exe SHA-256
//! 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
//! ↔ GameServer.pdb RSDS 5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53 age 2, совпадают).
//! Перенесены из hub `appserver/moveshape.rs` (волна Z-M2a): queries семейств
//! ride/automatic restore, проекции particular/team/swordship/wuxing/taiji/
//! enlarge/origin, boss blue fury, promotion и защитные щиты, итераторы
//! energy holding/pillar, порядок curable/blind, battle fairy attribute,
//! script, tian/wangsheng и проекции undead/extended/change-body. Тела
//! буквальные: методы переходного CMoveShape преобразованы в свободные
//! функции над заимствованным `CanonicalStateStorage`; обход, поколенческие
//! ключи и typed-доступ остаются операциями арены соседнего `storage`.
//! Единственная не чисто читающая операция — `prepare_consumable_restore`:
//! RestoreHp/RestoreMp (0x00444C80/0x00444D50) проверяют cooldown по clock1
//! и записывают clock2 до создания состояния; здесь это таймер
//! `consumable_restore_intervals` владельца, а не запись арены или codec.
//! Проекции ChangeBody соответствуют GetCHBYState (0x004CEC40), который не
//! вызывает callbacks. Мутирующие операции этих же семейств — splice замены,
//! append/take/remove записей, span-сдвиги и tick с item-часами
//! undead/extended — остаются у hub moveshape до соседней волны Z-M2b;
//! у семейств spiderweb и boss blue quake читающей половины в hub нет.

use crate::effects::{
    AutomaticRestoreState, BattleFairyAttributeState, BossBlueFuryState, CTeamState,
    ChangeBodyState, ConsumableRestoreState, DefenseShieldState, EnergyHoldingState,
    EnlargeFullMissState, EnlargeMaxHpState, EnlargeMaxMpState, ExtendedState,
    ExtendedStateKind, OriginState, ParticularState, PillarState, RestoreHpState,
    RestoreMpState, RideState, ScriptMoveState, SwordshipState, TaiJiState,
    TianShenXiaFanState, UndeadState, WangshengState, WuXingState,
};

use super::storage::{AppliedState, CanonicalStateStorage, StateData, StateKey};

pub fn ride_state(storage: &CanonicalStateStorage) -> Option<&RideState> {
    storage.state_entries.first::<RideState>()
}

pub fn has_ride_state(storage: &CanonicalStateStorage) -> bool {
    storage.state_entries.first::<RideState>().is_some()
}

/// Cooldown и constructor до отдельного Begin: отказ читает только clock1,
/// допуск записывает clock2; timestamp нового payload пока остаётся нулём.
pub fn prepare_consumable_restore(
    storage: &mut CanonicalStateStorage,
    health: bool,
    amount: u32,
    time_to_keep_ms: u32,
    frequency_ms: u32,
    interval_ms: u32,
    now: &mut dyn FnMut() -> u32,
) -> Option<ConsumableRestoreState> {
    if !storage.consumable_restore_intervals.try_begin(health, interval_ms, &mut *now) {
        return None;
    }
    Some(if health {
        ConsumableRestoreState::Health(RestoreHpState::new(
            time_to_keep_ms, frequency_ms, amount,
        ))
    } else {
        ConsumableRestoreState::Mana(RestoreMpState::new(
            time_to_keep_ms, frequency_ms, amount,
        ))
    })
}

pub fn particular_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &ParticularState> {
    storage.state_entries.iter::<ParticularState>()
}

pub fn team_recruitment_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &CTeamState> {
    storage.state_entries.iter::<CTeamState>()
}

pub fn automatic_restore_state(
    storage: &CanonicalStateStorage,
    key: StateKey,
) -> Option<AutomaticRestoreState> {
    AutomaticRestoreState::as_data_ref(storage.state_entries.get(key)?).copied()
}

/// Точный `GetStateNumByStateID`: считает все живые экземпляры с данным
/// базовым `CState::m_lID`, независимо от concrete owner-а состояния.
pub fn state_count_by_state_id(storage: &CanonicalStateStorage, state_id: i32) -> u32 {
    (0..storage.state_entries.len())
        .filter(|index| state_id_at(storage, *index) == Some(state_id as u32))
        .count().min(u32::MAX as usize) as u32
}

/// `GetStateBySkillID` просматривает канонические типизированные состояния
/// по фактическому идентификатору навыка, а не по классу сетевой записи.
pub fn has_state_by_skill_id(storage: &CanonicalStateStorage, state_id: u32) -> bool {
    (0..storage.state_entries.len()).any(|index| state_id_at(storage, index) == Some(state_id))
}

pub fn swordship_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &SwordshipState> {
    storage.state_entries.iter::<SwordshipState>()
}

pub fn wuxing_states(storage: &CanonicalStateStorage) -> impl Iterator<Item = &WuXingState> {
    storage.state_entries.iter::<WuXingState>()
}

pub fn taiji_state(storage: &CanonicalStateStorage) -> Option<TaiJiState> {
    storage.state_entries.first::<TaiJiState>().copied()
}

pub fn enlarge_full_miss_state(storage: &CanonicalStateStorage) -> Option<EnlargeFullMissState> {
    storage.state_entries.first::<EnlargeFullMissState>().copied()
}

pub fn enlarge_max_hp_state(storage: &CanonicalStateStorage) -> Option<EnlargeMaxHpState> {
    storage.state_entries.first::<EnlargeMaxHpState>().copied()
}

pub fn enlarge_max_mp_state(storage: &CanonicalStateStorage) -> Option<EnlargeMaxMpState> {
    storage.state_entries.first::<EnlargeMaxMpState>().copied()
}

pub fn origin_state(storage: &CanonicalStateStorage) -> Option<OriginState> {
    storage.state_entries.first::<OriginState>().copied()
}

pub fn boss_blue_fury_state(storage: &CanonicalStateStorage) -> Option<BossBlueFuryState> {
    storage.state_entries.first::<BossBlueFuryState>().copied()
}

pub fn promotion_magic_attack_factor(storage: &CanonicalStateStorage) -> Option<u16> {
    storage.state_entries.iter::<DefenseShieldState>().find_map(|state| match state {
        DefenseShieldState::Promotion(state) => Some(state.magic_attack_factor()),
        _ => None,
    })
}

pub fn defense_shields(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &DefenseShieldState> {
    storage.state_entries.iter::<DefenseShieldState>()
}

pub fn defense_shield_keys(storage: &CanonicalStateStorage) -> Vec<StateKey> {
    storage.state_entries.keys::<DefenseShieldState>()
}

pub fn defense_shield_key(storage: &CanonicalStateStorage, skill_id: u32) -> Option<StateKey> {
    defense_shield_keys(storage).into_iter().find(|key| {
        defense_shield(storage, *key).is_some_and(|state| state.skill_id() == skill_id)
    })
}

pub fn defense_shield(
    storage: &CanonicalStateStorage,
    key: StateKey,
) -> Option<&DefenseShieldState> {
    match storage.state_entries.get(key)? {
        StateData::DefenseShield(state) => Some(state),
        _ => None,
    }
}

pub fn energy_holding_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &EnergyHoldingState> {
    storage.state_entries.iter::<EnergyHoldingState>()
}

pub fn pillar_state(storage: &CanonicalStateStorage) -> Option<PillarState> {
    storage.state_entries.first::<PillarState>().copied()
}

pub fn curable_state_ids(storage: &CanonicalStateStorage) -> Vec<u32> {
    storage.state_entries.iter_data().filter(|state| state.is_curable())
        .map(StateData::state_id).collect()
}

pub fn blind_state_order(storage: &CanonicalStateStorage) -> Vec<u32> {
    storage.state_entries.iter_data().filter(|state| state.is_blind())
        .map(StateData::state_id).collect()
}

pub fn blind_state_instances(storage: &CanonicalStateStorage) -> Vec<(StateKey, u32)> {
    storage.state_entries.entries().filter(|(_, state)| state.is_blind())
        .map(|(key, state)| (key, state.state_id())).collect()
}

pub fn battle_fairy_attribute_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &BattleFairyAttributeState> {
    storage.state_entries.iter::<BattleFairyAttributeState>()
}

pub fn script_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &ScriptMoveState> {
    storage.state_entries.iter::<ScriptMoveState>()
}

pub fn tian_shen_xia_fan_state(storage: &CanonicalStateStorage) -> Option<TianShenXiaFanState> {
    storage.state_entries.first::<TianShenXiaFanState>().copied()
}

pub fn wangsheng_state(storage: &CanonicalStateStorage) -> Option<WangshengState> {
    storage.state_entries.first::<WangshengState>().copied()
}

pub fn get_undead_state(storage: &CanonicalStateStorage, state_id: u32) -> u32 {
    storage.state_entries.iter::<UndeadState>()
        .any(|state| state.state_id() == state_id)
        .then_some(state_id)
        .unwrap_or(0)
}

pub fn get_extended_state(
    storage: &CanonicalStateStorage,
    kind: ExtendedStateKind,
    state_id: u32,
) -> u32 {
    storage.state_entries.iter::<ExtendedState>()
        .any(|state| state.kind == kind && state.level == state_id)
        .then_some(state_id)
        .unwrap_or(0)
}

pub fn extended_states(
    storage: &CanonicalStateStorage,
) -> impl Iterator<Item = &ExtendedState> {
    storage.state_entries.iter::<ExtendedState>()
}

pub fn get_change_body_state(storage: &CanonicalStateStorage, state_id: u32) -> u32 {
    storage.state_entries.iter::<ChangeBodyState>()
        .any(|state| state.level == state_id)
        .then_some(state_id)
        .unwrap_or_default()
}

pub fn active_change_body_state(storage: &CanonicalStateStorage) -> Option<&ChangeBodyState> {
    storage.state_entries.iter::<ChangeBodyState>().last()
}

pub fn first_change_body_state_id(storage: &CanonicalStateStorage) -> Option<u32> {
    storage.state_entries.first::<ChangeBodyState>().map(|state| state.level)
}

fn state_id_at(storage: &CanonicalStateStorage, index: usize) -> Option<u32> {
    Some(storage.state_entries.get(storage.state_entries.address(index)?)?.state_id())
}
