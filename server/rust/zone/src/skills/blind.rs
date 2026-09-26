//! Ослепление CBlind (0x76): kernel-вход, visual и наложение состояния.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/blind.cpp`; опорные адреса —
//! docs/reconstruction/gameserver-skills.md. AddBlindState создаёт
//! CRushState2 (VERIFIED разведкой — создаётся состояние второго рывка, а не
//! CBlindState); codec семейства 8-байтный ✓ `effects/blind.rs`; End(H)
//! 13-fold — общий CStateSkill tail, здесь не дублируется
//! (порядок clear+End исполняет kernel). Тела перенесены буквально.
//!
//! Подготовка и visual принадлежат зарегистрированному навыку; AddBlindState
//! создаёт CRushState2 (0x7C), а не CBlindState. Его запреты принадлежат цели.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::{SelfCastGame,
//! SelfCastContact, SelfCastPlayer, SelfCastMoveShape}` реализован у владельца
//! hub-делегата; `SkillExecutionKernel` и Begin состояния — общие
//! (`lifecycle`, `blindstate`). Тип `Rush2State` (алиас `BlindState<0x7C>`,
//! ctor `0x5F12E0` VERIFIED) и формула scaled keep (query 10002, линейный
//! scale с clamp — movzx-уровни, `FISTP`-усечение через `truncate_original`)
//! объявлены здесь самодостаточно по машинному свидетельству; состояние
//! второго рывка уже описано в `effects/blind.rs`, а полноценный владелец
//! Rush/Rush2 — `rush.rs` (co-located объявление остаётся здесь до снятия
//! дублирования). Вызовы visual с mode 10/11/15 существуют, но CBlindEffect
//! их не публикует: общий visual tail всё равно выполняется, а текст ошибки
//! отправляет caller.

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::combat::truncate_original;
use crate::effects::{BLIND_STATE_ID, BlindState};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::blindstate::begin_primary_blind_state;
use super::dispatch::PlayerSkillDispatch;
use super::execution::RegisteredSkillRecord;
use super::lifecycle::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::selfcast::{
    SelfCastContact, SelfCastExecutionOutcome, SelfCastGame, SelfCastMoveShape, SelfCastPlayer,
};
use super::skillfactory::SkillOwner;
use super::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};

pub const BLIND_SKILL_ID: u32 = BLIND_STATE_ID;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const DELAY: u32 = 10_001;
const STATE_TIME: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

/// Единственный результат AddBlindState: состояние второго рывка (0x7C),
/// разделяемый 8-байт payload семейства lock-состояний.
pub const RUSH_2_STATE_ID: u32 = 0x7c;
pub type Rush2State = BlindState<RUSH_2_STATE_ID>;

/// Scaled keep AddBlindState: movzx-уровни, линейный scale с clamp и
/// `FISTP`-усечение; владелец полного семейства Rush — `rush.rs`.
pub fn scaled_state_time(source_level: u8, target_level: u8, base_time: u32) -> u32 {
    if u32::from(source_level) + 5 >= u32::from(target_level) { return base_time; }
    let difference = i32::from(target_level) - i32::from(source_level) - 5;
    let factor = (1.0_f32 - difference as f32 * 0.25).max(0.0);
    truncate_original(f64::from(base_time) * f64::from(factor)) as u32
}

pub const fn is_blind_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == BLIND_SKILL_ID
}

pub fn complete_blind<Game: SelfCastGame>(
    game: &mut Game, player_id: i32, ai: &mut Game::PlayerAi,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub fn cancel_blind<Game: SelfCastGame>(
    game: &mut Game, player_id: i32, ai: &mut Game::PlayerAi,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn weapon_is_valid<Game: SelfCastGame>(game: &Game, player: &Game::Player) -> bool {
    game.player_weapon_addon_category(player).is_some_and(|category| category == 2)
}

fn target<Game: SelfCastGame>(game: &Game, player_id: i32) -> Option<(i32, ShapeIdentity)> {
    let lifecycle = game.player_skill_lifecycle(player_id, BLIND_SKILL_ID)?;
    let (region, identity) = game.resolve_skill_sufferer(lifecycle)?;
    let shape = game.resolve_state_move_shape(region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub fn publish_blind_visual<Game: SelfCastGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if skill.owner() != SkillOwner::CBlind
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Blind || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    // Вызовы с mode 10/11/15 существуют, но CBlindEffect их не публикует:
    // общий visual tail всё равно выполняется, а текст ошибки отправляет caller.
    if matches!(mode, 2 | 7 | 8 | 13 | 14) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_cast_visual_to_player(identity.id, &message);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
    }
    game.send_cast_visual_around(source.get_region_id(), source, &message);
}

fn failure<Game: SelfCastGame>(game: &mut Game, player_id: i32, mode: u32, text: &[u8], mp: Option<u32>) {
    game.update_player_skill_visual(player_id, BLIND_SKILL_ID, mode);
    if let Some(mp) = mp {
        game.send_skill_system_info_with_unsigned(player_id, text, mp);
    } else {
        game.send_skill_system_info(player_id, text);
    }
}

fn begin_blind<Game: SelfCastGame>(
    game: &mut Game, player_id: i32, dispatch: PlayerSkillDispatch, now_milliseconds: fn() -> u32,
) -> SelfCastExecutionOutcome {
    game.replace_player_skill_visual_effect(
        player_id, BLIND_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::Blind, 1),
    );
    let target = target(game, player_id);
    if game.find_player(player_id).is_none() { return SelfCastExecutionOutcome::Rejected; }
    let Some(skill) = game.registered_player_skill(player_id, BLIND_SKILL_ID)
        .and_then(|address| game.registered_skill(address))
    else { return SelfCastExecutionOutcome::Rejected; };
    let started = skill.lifecycle().started_at_ms();
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, skill.level()) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(
        game.player_skill_last_used_ms(player_id, BLIND_SKILL_ID), reuse, now_milliseconds(),
    ) {
        failure(game, player_id, 13, b"GS0278", None);
        return SelfCastExecutionOutcome::Rejected;
    }
    let Some(target) = target else {
        failure(game, player_id, 10, b"GS0286", None);
        return SelfCastExecutionOutcome::Rejected;
    };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, player_id, 11, b"GS0290", None);
        return SelfCastExecutionOutcome::Rejected;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 15);
        let name = game.base_magic_target_name(target.0, target.1);
        game.send_skill_system_info_with_text(player_id, b"GS0291", &name);
        return SelfCastExecutionOutcome::Rejected;
    }
    let Some(player) = game.find_player(player_id) else { return SelfCastExecutionOutcome::Rejected; };
    if !weapon_is_valid(game, player) {
        failure(game, player_id, 14, b"GS0292", None);
        return SelfCastExecutionOutcome::Rejected;
    }
    if properties.query_property(MP_LOSS) != 0
        && (player.mana().wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0
    {
        let amount = properties.query_property(MP_LOSS);
        failure(game, player_id, 7, b"GS0288", Some(amount));
        return SelfCastExecutionOutcome::Rejected;
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); }
    game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
    SelfCastExecutionOutcome::Begun
}

fn add_blind_state<Game: SelfCastGame>(
    game: &mut Game, player_id: i32, target: (i32, ShapeIdentity), keep: u32, now_milliseconds: fn() -> u32,
) {
    if keep == 0 { return; }
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return; };
    if !game.live_skill_target_attackable(target.0, user, target.1) { return; }
    let Some(level) = game.registered_player_skill(player_id, BLIND_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(|skill| skill.level())
    else { return; };
    if game.skill_base_properties(BLIND_SKILL_ID, level).is_none() { return; }
    let Some(region) = game.find_player(player_id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then_some(player.shape().get_region_id())
    }) else { return; };
    if game.region_dimensions(region).is_none() { return; }
    let state = Rush2State::new(keep);
    if let Some((position, _)) = game.resolve_state_move_shape(target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == RUSH_2_STATE_ID))
    {
        let _ = game.end_and_destroy_state_at(target.0, target.1, position);
    }
    let user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let sufferer = game.resolve_state_move_shape(target.0, target.1)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let _ = begin_primary_blind_state(
        game, target.0, target.1, user, sufferer, state, &mut || now_milliseconds(),
    );
}

fn apply_blind<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    target: (i32, ShapeIdentity),
    source_region: i32,
    level: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: SelfCastContact<Runtime>,
{
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return false; };
    if !game.live_skill_target_attackable(target.0, user, target.1) { return false; }
    if let Some(controller) = game.skill_target_controller(target.0, target.1)
        .filter(|controller| *controller != player_id)
    {
        let position = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        });
        if let Some(position) = position {
            game.skill_first_attack_at_position(
                player_id, controller, source_region, position, runtime,
            );
        }
    }
    let Some(source_level) = game.find_player(player_id).map(|player| player.level()) else { return true; };
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return true; };
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else { return true; };
    let keep = scaled_state_time(source_level, target_level, properties.query_property(STATE_TIME));
    if keep != 0 { add_blind_state(game, player_id, target, keep, now_milliseconds); }
    true
}

pub fn run_blind<Game, Runtime>(
    game: &mut Game,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut Game::PlayerAi,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> SelfCastExecutionOutcome
where
    Game: SelfCastContact<Runtime>,
{
    if !is_blind_dispatch(dispatch) { return SelfCastExecutionOutcome::Rejected; }
    if game.player_skill_execution(player_id, BLIND_SKILL_ID).is_none() {
        return begin_blind(game, player_id, dispatch, now_milliseconds);
    }
    let Some(level) = game.registered_player_skill(player_id, BLIND_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(|skill| skill.level())
    else { return SelfCastExecutionOutcome::Rejected; };
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let source_region = game.find_player(player_id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then_some(player.shape().get_region_id())
    });
    let Some(target) = target(game, player_id) else { return SelfCastExecutionOutcome::Rejected; };
    let Some(source_region) = source_region else { return SelfCastExecutionOutcome::Rejected; };
    if game.base_magic_target_dead(target.0, target.1) {
        failure(game, player_id, 10, b"GS0285", None);
        return SelfCastExecutionOutcome::Rejected;
    }
    if game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(|player| player.mana()) else {
            return SelfCastExecutionOutcome::Rejected;
        };
        let loss = properties.query_property(MP_LOSS);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(MP_LOSS);
            failure(game, player_id, 7, b"GS0288", Some(amount));
            return SelfCastExecutionOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(loss)); }
        game.update_player_current_state_move_shape_ai(player_id);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
            failure(game, player_id, 14, b"GS0287", None);
            return SelfCastExecutionOutcome::Rejected;
        }
        let Some((target_x, target_y)) = game.resolve_state_move_shape(target.0, target.1).and_then(|target| {
            let y = target.shape().get_tile_y().ok()?;
            let x = target.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return SelfCastExecutionOutcome::Rejected; };
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return SelfCastExecutionOutcome::Rejected; };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
            return SelfCastExecutionOutcome::Rejected;
        };
        let can_break = properties.query_property(CAN_BREAK);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BLIND_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BLIND_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let delay = properties.query_property(DELAY);
    let Some(started) = game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
    else { return SelfCastExecutionOutcome::Rejected; };
    if now_milliseconds() < started.wrapping_add(delay) {
        return SelfCastExecutionOutcome::Pending;
    }
    let Some(address) = game.registered_player_skill(player_id, BLIND_SKILL_ID) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let Some(skill) = game.registered_skill(address) else { return SelfCastExecutionOutcome::Rejected; };
    let (_, saved_target) = skill.lifecycle().sufferer();
    if saved_target.object_type != 0 && saved_target.id != 0 {
        let live_target = self::target(game, player_id);
        if live_target.is_none_or(|target| game.base_magic_target_dead(target.0, target.1)) {
            game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 10);
            return SelfCastExecutionOutcome::Rejected;
        }
        let Some((region, identity)) = live_target else { return SelfCastExecutionOutcome::Rejected; };
        let Some(shape) = game.resolve_state_move_shape(region, identity) else { return SelfCastExecutionOutcome::Rejected; };
        let (Ok(x), Ok(y)) = (shape.shape().get_tile_x(), shape.shape().get_tile_y()) else {
            return SelfCastExecutionOutcome::Rejected;
        };
        if let Some(skill) = game.registered_skill_mut(address) { skill.lifecycle_mut().set_point_target((x, y)); }
    }
    let Some(skill) = game.registered_skill(address) else { return SelfCastExecutionOutcome::Rejected; };
    let path = game.skill_target_path(skill.lifecycle());
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, player_id, 11, b"GS0290", None);
        return SelfCastExecutionOutcome::Rejected;
    }
    // Повторный GetS выше нужен только для conversion/path. Наложение и PK
    // сохраняют S, регион U и таблицу свойств, выбранные в начале этого AI.
    game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 1);
    let applied = game.with_published_player_ai(player_id, ai, |game| {
        apply_blind(game, player_id, target, source_region, level, runtime, now_milliseconds)
    });
    if !applied { return SelfCastExecutionOutcome::Rejected; }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, BLIND_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    SelfCastExecutionOutcome::Completed
}
