//! Живые Begin/restart/AI/End периодического лечения в переходном Game:
//! CHealState/CHeal2State и CSuperHealState/CSuperHeal2State.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/{healstate,healstate2,superhealstate,
//! superhealstate2}.cpp` и первичные `heal*.cpp`. Прежний переходный
//! владелец — `src/gameserver/appserver/skills/healstate.rs`; тела
//! перенесены буквально порцией №6a. Данные, кодек и правило тика
//! принадлежат Zone `effects/heal.rs` (там же адреса vtable).
//!
//! Machine-факты state-квартета: AI/End/Unserialize fold
//! `0x1EEDF0/0x1EEBA0/0x1EEC70`, Serialize 5-fold `0x1F65F0` (с CRoarState).
//!
//! Четыре варианта имеют один payload и отличаются ID. Сначала завершается
//! первый прежний ID без RTTI/ended-фильтра, затем создаётся новый
//! экземпляр. SuperHeal удаляет D3, остальные — собственный ID. Begin
//! читает часы, сохраняет U/S и запускает loop=1 visual до append;
//! счётчик сбрасывается после BeginVisual. Begin и End не отправляют
//! BFE03/BFE04. Loop=1 не вызывает внешний callback; арена публикует
//! готовый visual вместе с записью.
//!
//! Запись SuperHeal2 остаётся у выбранной цели, но её U/S указывают на
//! заклинателя. Эти привязки живут только в общей арене, не в копии
//! payload. End удаляет собственный указатель через свежего S без записи
//! ended; при чужом или отсутствующем S запись остаётся у держателя.
//! SetRegion меняет только регион S. Restart сохраняет U/старт,
//! назначает S=holder, заново начинает visual и обнуляет счётчик.
//!
//! AI разрешает S и проверяет смерть до часов. Promotion читается перед
//! первым clock; строгий unsigned wrapping срок даёт не более одного тика.
//! Count++ предшествует свежим HP/MAX, setter и OnChangeStates; MAX
//! читается повторно при ограничении. После callbacks отдельно читаются
//! часы срока. Полный unsigned gain умножается на сохранённый f32-множитель
//! и усекается FISTP без промежуточного f32. HP складывается с
//! DWORD-переполнением. OnUpdateProperties возвращает 1 без побочных
//! эффектов. Native читает WORD первого ID без RTTI; чужой layout здесь
//! не имитируется.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::*`
//! реализован у прежнего владельца; `end_and_destroy_state_at` арены
//! остаётся у `states/state.rs` и объявлен швом.

use crate::combat::truncate_original;
use crate::effects::{DefenseShieldState, HEAL_STATE_BYTES, HealState};
use crate::regions::ShapeIdentity;

use super::heal::{HEAL_SKILL_ID, SUPER_HEAL_2_SKILL_ID, SUPER_HEAL_SKILL_ID};
use super::state::StateKey;
use super::statecast::{
    StateCastGame, StateCastMoveShape, state_cast_storage_participant,
};

/// Замена первого состояния своего ID: End/destructor прежнего в той же
/// позиции, затем ctor(J,J) и Begin(U,S) до append. `create` вызывается
/// после удаления прежнего состояния (FISTP прибавки, затем FREQ→PERSIST).
pub fn replace_heal_state<Game: StateCastGame>(
    game: &mut Game,
    source: (i32, ShapeIdentity),
    storage_target: (i32, ShapeIdentity),
    skill_id: u32,
    create: impl FnOnce() -> HealState,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let removed_id = if skill_id == SUPER_HEAL_SKILL_ID { HEAL_SKILL_ID } else { skill_id };
    let Some(storage) = game.resolve_state_move_shape(storage_target.0, storage_target.1) else { return false; };
    if let Some((index, _)) = storage.find_state_position(|state| state.state_id() == removed_id)
        && !game.end_and_destroy_state_at(storage_target.0, storage_target.1, index)
    { return false; }
    let mut state = create();
    state.begin_at(now());
    let Some(user) = state_cast_storage_participant(game, source) else { return false; };
    let effect_target = if skill_id == SUPER_HEAL_2_SKILL_ID { source } else { storage_target };
    let Some(sufferer) = state_cast_storage_participant(game, effect_target) else { return false; };

    state.reset_ticks();
    let record = state.encoded_for_install();
    let Some(storage) = game.resolve_state_move_shape_mut(storage_target.0, storage_target.1) else { return false; };
    let key = storage.append_applied_state_record(state, &record);
    storage.set_applied_state_user(key, Some(user));
    storage.set_applied_state_sufferer(key, Some(sufferer));
    true
}

/// Повторный вход: базовый Begin(NULL, holder), visual loop1 и обнуление
/// счётчика тиков без смены старта.
pub fn restart_heal_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return false; }
    if !game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.mark_applied_state_begun(key))
    { return false; }
    let _ = game.resolve_state_move_shape_mut(region_id, holder)
        .is_some_and(|shape| shape.begin_applied_state_visual(key, 1));
    if let Some(state) = game.resolve_state_move_shape_mut(region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key))
    {
        state.reset_ticks();
    }
    true
}

/// Тик AI владельца: смерть S → End; множитель Promotion читается перед
/// первым clock; строгий срок — не более одного тика, иначе истечение → End.
pub fn update_stored_heal_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    mut now: impl FnMut() -> u32,
) {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return; }
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key) else {
        let _ = end_heal_state(game, region_id, holder, key);
        return;
    };
    let Some(health) = game.move_shape_health(target.0, target.1) else { return; };
    if health == 0 {
        let _ = end_heal_state(game, region_id, holder, key);
        return;
    }
    let Some(target_shape) = game.resolve_state_move_shape(target.0, target.1) else { return; };
    let promotion = target_shape.find_state_position(|state| state.state_id() == 0x142);
    let multiplier = match promotion {
        None => 1.0,
        Some((_, promotion_key)) => {
            // Native читает WORD первого ID без RTTI; чужой layout не имитируем.
            let Some(DefenseShieldState::Promotion(promotion)) =
                target_shape.applied_state::<DefenseShieldState>(promotion_key)
            else { return; };
            f32::from(promotion.heal_recover_factor()) * 0.001
        }
    };
    let checked_at_ms = now();
    let gain = game.resolve_state_move_shape_mut(region_id, holder)
        .and_then(|shape| shape.applied_state_mut::<HealState>(key))
        .and_then(|state| state.take_due_gain(checked_at_ms));
    if let Some(gain) = gain {
        let Some(health) = game.move_shape_health(target.0, target.1) else { return; };
        let mut next = health.wrapping_add(truncate_original(f64::from(gain) * f64::from(multiplier)) as u32);
        let Some(maximum) = game.move_shape_maximum_health(target.0, target.1) else { return; };
        if maximum < next {
            let Some(maximum) = game.move_shape_maximum_health(target.0, target.1) else { return; };
            next = maximum;
        }
        if !game.set_move_shape_health(target.0, target.1, next) { return; }
        game.publish_move_shape_states(target.0, target.1);
    }
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return; }
    let checked_at_ms = now();
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key))
        .is_some_and(|state| state.expired(checked_at_ms))
    {
        let _ = end_heal_state(game, region_id, holder, key);
    }
}

/// Полный End по ключу: страдальцу сохранённого identity, не записывая ended.
pub fn end_heal_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<HealState>(key)).is_none()
    { return false; }
    let Some(target) = game.resolve_applied_state_sufferer(region_id, holder, key)
    else { return false; };
    game.remove_applied_state_from(region_id, holder, key, target, HEAL_STATE_BYTES)
}
