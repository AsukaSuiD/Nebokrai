//! Контакт и последовательное наложение контроля CBoaLock.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/boalock.cpp.
//!
//! PK и пустая UNKNOWN/1-атака сохраняются до свежего запроса таблицы.
//! NULL таблица отменяет весь контакт. Уровни читаются U→S, а при превышении
//! U+5 — повторно; коэффициент сохраняется в f32, unsigned PERSIST затем
//! умножается без промежуточного округления в f32 и усекается как x87 FISTP.
//! При ненулевом сроке AddLock и AddKnockOut независимо повторяют допуск и
//! проверяют фактический регион U. Уровень ограничивает только AddLock.
//!
//! Новый payload создаётся до поиска старого. AddLock снимает первый ID 0xD2,
//! AddKnockOut — именно ID 0x73, хотя создаёт KnockOut с ID 0x192. После полного
//! End/destructor новый объект добавляется в конец, не в освобождённую позицию.
//! Пустой OnBeenAttacked(..., false) следует после обеих попыток и действует
//! также при нулевом сроке. Здесь нет Cure, RP, ForceMove или отдельного RNG.

use super::blindstate::begin_primary_blind_state;
use super::boalockstate::{BoaLockState, replace_boa_lock_state};
use super::fightdefense::truncate_original;
use super::knockoutstate::KnockOutState;
use super::weaponattack::source_master;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const PERSIST: u32 = 10_002;
const PREVIOUS_KNOCK_OUT_ID: u32 = 0x73;

fn source_has_region(game: &CGame, source: (i32, ShapeIdentity)) -> bool {
    resolve_state_move_shape(game, source.0, source.1).is_some_and(|shape| {
        shape.shape().is_assigned_to_server_region()
            && game.find_region(shape.shape().get_region_id()).is_some()
    })
}

fn add_lock_state(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    keep: u32, now: &mut dyn FnMut() -> u32,
) {
    let Some(source_level) = game.move_shape_level(source.0, source.1) else { return; };
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    if target_level >= source_level
        || !game.live_skill_target_attackable_between(source, target)
        || !source_has_region(game, source)
    { return; }
    let state = BoaLockState::new(keep);
    let _ = replace_boa_lock_state(game, source, target, state, now);
}

fn add_knock_out_state(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    keep: u32, now: &mut dyn FnMut() -> u32,
) {
    if !game.live_skill_target_attackable_between(source, target)
        || !source_has_region(game, source)
    { return; }
    let state = KnockOutState::new(keep);
    let previous = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == PREVIOUS_KNOCK_OUT_ID));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let _ = begin_primary_blind_state(
        game, target.0, target.1, Some(source), Some(target), state, now,
    );
}

pub(super) fn apply_boa_lock_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if std::ptr::eq(user, sufferer) { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let attack = AttackInformation::for_master(master);
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    let Some(source_level) = game.move_shape_level(source.0, source.1) else { return; };
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let keep = if u32::from(source_level) + 5 < u32::from(target_level) {
        let Some(source_level) = game.move_shape_level(source.0, source.1) else { return; };
        let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
        let delta = (i32::from(target_level) - i32::from(source_level) - 5).max(0);
        let factor = (1.0_f32 - delta as f32 * 0.25_f32).max(0.0);
        let persist = properties.query_property(PERSIST);
        truncate_original(f64::from(persist) * f64::from(factor)) as u32
    } else {
        properties.query_property(PERSIST)
    };
    if keep != 0 {
        add_lock_state(game, source, target, keep, &mut || runtime.now_milliseconds());
        add_knock_out_state(game, source, target, keep, &mut || runtime.now_milliseconds());
    }
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}
