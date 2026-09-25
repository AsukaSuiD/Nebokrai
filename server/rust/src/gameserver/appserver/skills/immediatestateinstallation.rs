//! Установка постоянных состояний TaiJi, Origin, Swordship, WuXing и Enlarge.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/{taiji,origin,
//! swordship*,wuxing*,enlargefullmiss,enlargemaxhp,enlargemaxmp}.cpp и CState::Begin.
//! Выбор ветки состояния и табличное чтение прибавок — zone rules
//! `skills/immediate.rs`; здесь живой обход Game по прежнему контракту.
//! TaiJi/Origin/Swordship/WuXing создают и начинают новое состояние до поиска
//! первого старого ID, затем заменяют его в прежней позиции. Enlarge выполняют
//! End/destructor первого ID, лишь затем читают прибавку и добавляют новый
//! экземпляр в конец. Выбор не фильтрует RTTI или ended.
//! Первичный Begin читает часы и сохраняет фактические U/S, но до установки
//! новый payload не виден списку и callback-ам старого End. У этих Begin нет
//! visual; timestamp не читается их AI, свойствами или DB-записью.
//! SlotMap/Vec и существующий DB-cache заменяют указатели/STL. Формулы и
//! перезапуск остаются у раздельных payload. Внешний UpdateProperty у Enlarge
//! безусловен после Begin, у остальных требует его успеха; OnChangeStates
//! и завершение навыка не принадлежат этой операции.

use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::moveshape::AppliedState;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::{ImmediateStatePayload, ImmediateStatePlacement, immediate_state_placement};

fn replace_immediate_payload<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    payload: ImmediateStatePayload, runtime: &mut Runtime,
) -> bool {
    match payload {
        ImmediateStatePayload::TaiJi(state) => replace_immediate_state(game, source, skill_id, state, &state.encoded(), runtime),
        ImmediateStatePayload::Origin(state) => replace_immediate_state(game, source, skill_id, state, &state.encoded(), runtime),
        ImmediateStatePayload::Swordship(state) => replace_immediate_state(game, source, skill_id, state, &state.encoded(), runtime),
        _ => false,
    }
}

fn append_immediate_payload(
    game: &mut CGame, source: (i32, ShapeIdentity), payload: ImmediateStatePayload,
    participants: ((i32, ShapeIdentity), (i32, ShapeIdentity)),
) -> bool {
    match payload {
        ImmediateStatePayload::FullMiss(state) => publish(game, source, state, &state.encoded(), participants, None),
        ImmediateStatePayload::MaxHp(state) => publish(game, source, state, &state.encoded(), participants, None),
        ImmediateStatePayload::MaxMp(state) => publish(game, source, state, &state.encoded(), participants, None),
        _ => false,
    }
}

fn primary_begin<Runtime: GameMainLoopRuntime>(
    game: &CGame, source: (i32, ShapeIdentity), runtime: &mut Runtime,
) -> Option<((i32, ShapeIdentity), (i32, ShapeIdentity))> {
    let _ = runtime.now_milliseconds();
    let participant = || {
        let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    Some((participant()?, participant()?))
}

fn publish<T: AppliedState>(
    game: &mut CGame, source: (i32, ShapeIdentity), state: T, record: &[u8],
    participants: ((i32, ShapeIdentity), (i32, ShapeIdentity)), placement: Option<(usize, usize)>,
) -> bool {
    let Some(holder) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    let key = match placement {
        Some(location) => {
            let Some(key) = holder.insert_replacement_state_record(state, record, location) else { return false; };
            key
        }
        None => holder.append_applied_state_record(state, record),
    };
    holder.set_applied_state_user(key, Some(participants.0));
    holder.set_applied_state_sufferer(key, Some(participants.1));
    true
}

pub(super) fn replace_immediate_state<Runtime: GameMainLoopRuntime, T: AppliedState>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    state: T, record: &[u8], runtime: &mut Runtime,
) -> bool {
    let Some(participants) = primary_begin(game, source, runtime) else { return false; };
    let Some(holder) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
    let previous = holder.find_state_position(|state| state.state_id() == skill_id);
    let placement = match previous {
        Some((index, key)) => {
            let Some(location) = holder.applied_state_replacement_location(key) else { return false; };
            if end_and_destroy_state_at(game, source.0, source.1, index).is_none() { return false; }
            Some(location)
        }
        None => None,
    };
    if !publish(game, source, state, record, participants, placement) { return false; }
    let _ = game.update_move_shape_properties(source.0, source.1);
    true
}

pub(super) fn apply_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) -> bool {
    match immediate_state_placement(skill_id) {
        Some(ImmediateStatePlacement::ReplaceAtOldPosition) => {
            let Some(payload) = ImmediateStatePayload::new(
                skill_id, |usage| properties.query_property(usage),
            ) else { return false; };
            replace_immediate_payload(game, source, skill_id, payload, runtime)
        }
        Some(ImmediateStatePlacement::EndFirstThenAppend) => {
            let Some(holder) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
            if let Some((index, _)) = holder.find_state_position(|state| state.state_id() == skill_id)
                && end_and_destroy_state_at(game, source.0, source.1, index).is_none()
            { return false; }
            let Some(payload) = ImmediateStatePayload::new(
                skill_id, |usage| properties.query_property(usage),
            ) else { return false; };
            let begun = primary_begin(game, source, runtime);
            let installed = begun
                .is_some_and(|participants| append_immediate_payload(game, source, payload, participants));
            let _ = game.update_move_shape_properties(source.0, source.1);
            installed
        }
        None => false,
    }
}
