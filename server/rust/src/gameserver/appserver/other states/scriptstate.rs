//! Живые Begin/restart/AI/End семи состояний `CMoveShape::AddState` в
//! переходном Game. Данные, enum и записи перенесены в Zone
//! `effects/scriptstate.rs` (там же адреса vtable и кодека).
//! Источник: gameserver.exe + GameServer.pdb, moveshape.cpp и
//! other states/{usegoodsenlarge*,improveexp,autoprotect}state.cpp.
//!
//! Пять UseGoods и ImproveExp требуют non-NULL User до base Begin. Первичное
//! self/self применение читает clock до U/S getters, создаёт visual loop0,
//! затем caller добавляет запись и вызывает UpdateProperty. Begin не шлёт
//! пакет; первый property visual шлёт BFE03 и завершает one-shot ресурс.
//! Begin(NULL, holder) этих шести возвращает false без мутаций и часов.
//! AutoProtect Begin0x005D4290 требует non-NULL S и отклоняет Player-GM:
//! base → visual loop1, без флага/пакета. Его NULL-user restart не читает часы.
//! Общая арена хранит остаточный visual без повторного Begin/публикации.
//! AI0x005D5BA0/EXP0x005D60B0: один clock → unsigned start+keep<now → End,
//! без ended/death/owner-gates и без особого случая keep0. End шести
//! 0x005D5B80: ended у записи → actual S → RemoveState(pointer), без visual.
//! AutoProtect End0x005D44E0 сохраняет первого Player-S, проверяет non-GM,
//! вызывает optional visual1 со свежим S, сбрасывает флаг первому S и удаляет
//! через него; state.ended не меняется. Чужая арена не получает local StateKey.
//! Property пяти UseGoods сохраняет первого S: NULL→false, optional visual0,
//! затем тип/формула на первом S с живым коэффициентом после visual. ImproveExp
//! property0x005D5F10 вызывает только visual0, формула опыта читается отдельно.
//! AutoProtect property0x005D4240: первый Player-S/non-GM → flag=true → visual0.
//! Все visuals читают actual S; ended/NULL подавляют пакет, но не base tail.
//! Шесть visuals всегда строят BFE03; AutoProtect visual0x005D4530 различает
//! BFE03/BFE04. Время — два чтения при положительном остатке, additional=0.
//! SetRegion пяти UseGoods и AutoProtect меняет User; ImproveExp — Sufferer.
//! SlotMap/заимствования заменяют CState* и ручной lifetime; отказ native
//! allocator не эмулируется. Неизвестные перегрузки Begin сохранены у owners.

use super::improveexpstate;
use super::player::PlayerCombatProperties;
use super::shape::ShapeIdentity;
use super::usegoodsenlargedefstate;
use super::usegoodsenlargeelmdefstate;
use super::usegoodsenlargefullmissstate;
use super::usegoodsenlargemaxhpstate;
use super::usegoodsenlargemaxmpstate;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
    update_applied_state_end_visual, update_property_state_visual,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_shared::values::CGuid;

pub(crate) use nebokrai_zone::effects::{
    AUTO_PROTECT_STATE_BYTES, SCRIPT_STATE_TIMED_BYTES, ScriptMoveState, ScriptStateKind,
};

/// Формулы пяти UseGoods сохраняют владельцев в своих файлах; здесь только
/// диспетчер по виду payload.
pub(crate) fn apply_script_combat_properties(
    state: &ScriptMoveState,
    properties: &mut PlayerCombatProperties,
) {
    match state.kind() {
        ScriptStateKind::EnlargeMaxHp(value) => {
            usegoodsenlargemaxhpstate::apply(*value, properties)
        }
        ScriptStateKind::EnlargeMaxMp(value) => {
            usegoodsenlargemaxmpstate::apply(*value, properties)
        }
        ScriptStateKind::EnlargeDefense(value) => {
            usegoodsenlargedefstate::apply(*value, properties)
        }
        ScriptStateKind::EnlargeElementDefense(value) => {
            usegoodsenlargeelmdefstate::apply(*value, properties)
        }
        ScriptStateKind::EnlargeFullMiss(value) => {
            usegoodsenlargefullmissstate::apply(*value, properties)
        }
        ScriptStateKind::ImproveExp(_) | ScriptStateKind::AutoProtect => {}
    }
}

pub(crate) fn script_experience_multiplier_delta(state: &ScriptMoveState) -> f64 {
    match state.kind() {
        ScriptStateKind::ImproveExp(value) => improveexpstate::multiplier_delta(*value),
        _ => 0.0,
    }
}

pub(crate) fn begin_primary_script_state(
    game: &mut CGame,
    player_id: i32,
    state_id: i32,
    value1: i32,
    value2: i32,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let mut state = ScriptMoveState::from_factory(state_id, value1, value2)?;
    game.find_player(player_id)?;
    if state.is_auto_protect() && game.script_player_gm_level(player_id).unwrap_or(0) != 0 {
        return None;
    }
    state.begin_at(now());
    let player = game.find_player(player_id)?;
    let participant = (
        player.shape().get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..player.shape().identity() },
    );
    let record = state.encoded_for_install();
    let shape = game.find_player_mut(player_id)?.move_shape_mut();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    Some(key)
}

pub(crate) fn update_script_move_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false; };
    if state.is_auto_protect() {
        let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return true; };
        if target.object_type != 400 || game.script_player_gm_level(target.id).unwrap_or(0) != 0 {
            return true;
        }
        let Some(player) = game.find_player_mut(target.id) else { return true; };
        player.set_auto_protected(true);
    } else if !state.is_improve_exp() {
        let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false; };
        update_script_state_begin_visual(game, region_id, holder, key, now);
        if target.object_type == 400 {
            let Some(player) = game.find_player(target.id) else { return false; };
            let mut properties = player.combat_properties();
            let Some(state) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
            else { return false; };
            apply_script_combat_properties(state, &mut properties);
            let Some(player) = game.find_player_mut(target.id) else { return false; };
            player.update_state_combat_properties(|_| properties);
        }
        return true;
    }
    update_script_state_begin_visual(game, region_id, holder, key, now);
    true
}

fn update_script_state_begin_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: &mut dyn FnMut() -> u32,
) {
    update_property_state_visual::<ScriptMoveState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now) as u32,
    );
}

pub(crate) fn restart_script_move_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _changing_region: bool, _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false; };
    if !state.is_auto_protect()
        || (holder.object_type == 400 && game.script_player_gm_level(holder.id).unwrap_or(0) != 0)
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false; }
    begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_script_move_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let sampled_at_ms = runtime.now_milliseconds();
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
        .is_some_and(|state| state.expired(sampled_at_ms));
    expired && end_script_move_state(game, region_id, holder, key)
}

pub(crate) fn end_script_move_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false; };
    if state.is_auto_protect() {
        let Some(target @ (_, identity)) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false; };
        if identity.object_type != 400 || game.script_player_gm_level(identity.id).unwrap_or(0) != 0 {
            return false;
        }
        update_applied_state_end_visual(game, region_id, holder, key, StatePropertyTarget::Sufferer);
        let Some(player) = game.find_player_mut(identity.id) else { return false; };
        player.set_auto_protected(false);
        remove_applied_state_from(game, region_id, holder, key, target, AUTO_PROTECT_STATE_BYTES)
    } else {
        if !resolve_state_move_shape_mut(game, region_id, holder)
            .is_some_and(|shape| shape.mark_applied_state_ended(key))
        {
            return false;
        }
        let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
        else { return false; };
        remove_applied_state_from(game, region_id, holder, key, target, SCRIPT_STATE_TIMED_BYTES)
    }
}
