//! Состояния Zone: арена экземпляров мобильного носителя (`storage`,
//! бывший `moveshape/state_storage.rs` переходного Game), читающие проекции
//! её семейств (`accessors` — бывшие query-методы hub
//! `appserver/moveshape.rs`), клиентские контракты (`catalog` — проекция
//! живых записей для `CMoveShape::AddToByteArray_ForClient` и runtime-план
//! visual после Begin) и DB Save/Load (`serialization` — кодек проекции
//! GameSave, бывшая Save/Load-семья hub `appserver/moveshape.rs`).

mod accessors; // читающие проекции арены состояний владельца.
mod catalog; // клиентская проекция живых состояний и runtime-план их visual.
mod serialization; // DB Save/Load-кодек состояний GameSave и сброс persisted snapshot.
mod storage; // арена экземпляров состояний мобильного носителя.

pub use accessors::{
    active_change_body_state, automatic_restore_state, battle_fairy_attribute_states,
    blind_state_instances, blind_state_order, boss_blue_fury_state, curable_state_ids,
    defense_shield, defense_shield_key, defense_shield_keys, defense_shields,
    energy_holding_states, enlarge_full_miss_state, enlarge_max_hp_state,
    enlarge_max_mp_state, extended_states, first_change_body_state_id, get_change_body_state,
    get_extended_state, get_undead_state, has_ride_state, has_state_by_skill_id, origin_state,
    particular_states, pillar_state, prepare_consumable_restore,
    promotion_magic_attack_factor, ride_state, script_states, state_count_by_state_id,
    swordship_states, taiji_state, team_recruitment_states, tian_shen_xia_fan_state,
    wangsheng_state, wuxing_states,
}; // читающие проекции семейств состояний арены `CMoveShape`.
pub use catalog::{StateClientRecord, registered_runtime_state_visual, state_client_record}; // клиентская запись состояния и runtime-план visual.
pub use serialization::{
    clear_persisted_runtime_state, read_i16, read_i32, read_u16, read_u32, replace_ex_states,
    serialize_ex_states_for_save, serialized_ex_states, update_known_state_record,
    update_nth_known_state_record, write_i16, write_i32, write_u16, write_u32,
}; // DB-кодек проекции GameSave и бинарные примитивы записей состояний.
pub use storage::{
    AppliedState, AppliedStateEntries, CanonicalStateStorage, LegacyStateCodec, StateBatch,
    StateData, StateKey, StateSerialization,
}; // хранилище состояний и его кодек сериализации.
