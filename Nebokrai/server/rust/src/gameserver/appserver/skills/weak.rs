//! Призыв области ослабления CWeak (0x12E).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/weak.cpp.
//! Зарегистрированный навык владеет подготовкой, целью и visual; область
//! живёт после End навыка, а её состояния принадлежат получателям.

use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::poisonmoth::master_info;
use super::skillfactory::SkillOwner;
use super::weakphalanx::CWeakPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const WEAK_SKILL_ID: u32 = 0x12e;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const ATTACK_LOSS: u32 = 205;
const MAX_DISTANCE: u32 = 5_003;
const DELAY: u32 = 10_001;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;
const LIFETIME_FACTOR: u32 = 20_010;
const LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) const fn is_weak_target(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == WEAK_SKILL_ID
}

pub(crate) fn complete_player_weak<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, WEAK_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_weak<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, WEAK_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn publish_weak_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CWeak
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Weak || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
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
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, target)) => {
                let Some(target) = resolve_state_move_shape(game, region, target) else { return; };
                let (Ok(x), Ok(y)) = (target.shape().get_tile_x(), target.shape().get_tile_y()) else { return; };
                (x, y)
            }
            None => skill.lifecycle().destination(),
        };
        message.add_long(0);
        message.add_long(0);
        message.add_long(destination.0);
        message.add_long(destination.1);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn fail(game: &mut CGame, player_id: i32, code: u32, text: &[u8], mp: Option<u32>) {
    game.update_player_skill_visual(player_id, WEAK_SKILL_ID, code);
    if game.find_player(player_id).is_some() {
        if let Some(mp) = mp {
            game.send_skill_system_info_with_unsigned(player_id, text, mp);
        } else {
            game.send_skill_system_info(player_id, text);
        }
    }
}

fn begin_weak<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    game.replace_player_skill_visual_effect(
        player_id, WEAK_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::Weak, 1),
    );
    if game.find_player(player_id).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(skill) = game.registered_player_skill(player_id, WEAK_SKILL_ID)
        .and_then(|address| game.registered_skill(address))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let started = skill.lifecycle().started_at_ms();
    let Some(properties) = game.skill_base_properties(WEAK_SKILL_ID, skill.level()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(
        game.player_skill_last_used_ms(player_id, WEAK_SKILL_ID), reuse, runtime.now_milliseconds(),
    ) {
        fail(game, player_id, 13, b"GS0278", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        fail(game, player_id, 11, b"GS0290", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    // Нулевая стоимость здесь означает отказ CheckCast, а нулевая дальность
    // снимает ограничение. Ни препятствия, ни смерть S эта проверка не отвергает.
    if properties.query_property(MP_LOSS) == 0 { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if (mana.wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0 {
        let amount = properties.query_property(MP_LOSS);
        fail(game, player_id, 7, b"GS0288", Some(amount));
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); }
    game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
    terminal(QueuedSkillExecutionState::Begun)
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    let region_id = player.shape().get_region_id();
    let Some(region) = game.find_region(region_id) else { return; };
    if region.get_security(destination.0, destination.1).ok() == Some(RegionSecurity::SAFE) { return; }
    let mut master = master_info(player);
    master.master_country_id = 0;
    let element_modify = player.combat_properties().element_modify as u32;
    let Some(level) = game.registered_player_skill(player_id, WEAK_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return; };
    let Some(properties) = game.skill_base_properties(WEAK_SKILL_ID, level) else { return; };
    let scale = properties.query_property(LIFETIME_FACTOR)
        .wrapping_mul(element_modify).wrapping_add(100);
    // Между умножениями исходный коэффициент сохраняется как float;
    // конечный x87 fistp усекает к signed DWORD, включая indefinite при overflow.
    let scale = (f64::from(scale) * f64::from(0.01_f32)) as f32;
    let lifetime = truncate_original(
        f64::from(properties.query_property(LIFETIME)) * f64::from(scale),
    ) as u32;
    let attack_loss = properties.query_property(ATTACK_LOSS);
    let Some(level) = game.registered_player_skill(player_id, WEAK_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return; };
    let now = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CWeakPhalanx::new(id, master, now, lifetime, level, 1, 1, attack_loss);
    phalanx.set_center(destination.0, destination.1);
    if game.add_weak_phalanx(region_id, phalanx, destination.0, destination.1, now, runtime)
        .is_some_and(|result| result.is_ok())
    {
        let _ = game.send_weak_phalanx_entry(region_id, id, runtime);
    }
}

pub(crate) fn execute_player_weak<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_weak_target(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    if game.player_skill_execution(player_id, WEAK_SKILL_ID).is_none() {
        return begin_weak(game, player_id, dispatch, runtime);
    }
    let Some(address) = game.registered_player_skill(player_id, WEAK_SKILL_ID) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(skill) = game.registered_skill(address) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let level = skill.level();
    if game.skill_base_properties(WEAK_SKILL_ID, level).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let (region, user) = skill.lifecycle().user();
    let user_present = resolve_state_move_shape(game, region, user).is_some();
    let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
        Some((region, target)) => {
            if game.base_magic_target_dead(region, target) {
                fail(game, player_id, 10, b"GS0285", None);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let Some(target) = resolve_state_move_shape(game, region, target) else {
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            let (Ok(x), Ok(y)) = (target.shape().get_tile_x(), target.shape().get_tile_y()) else {
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            if let Some(skill) = game.registered_skill_mut(address) {
                skill.lifecycle_mut().set_point_target((x, y));
            }
            (x, y)
        }
        None => skill.lifecycle().destination(),
    };
    if !user_present { return terminal(QueuedSkillExecutionState::Rejected); }
    // В каждом AI S преобразуется в точку. Координаты и выбранная таблица
    // свойств этого вызова переживают OnChangeStates; Summon выбирает свою таблицу.
    if game.player_skill_execution(player_id, WEAK_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(properties) = game.skill_base_properties(WEAK_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let loss = properties.query_property(MP_LOSS);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(MP_LOSS);
            fail(game, player_id, 7, b"GS0288", Some(amount));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let Some(properties) = game.skill_base_properties(WEAK_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let can_break = properties.query_property(CAN_BREAK);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, WEAK_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, destination.0, destination.1,
            ));
        }
        game.update_player_skill_visual(player_id, WEAK_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, WEAK_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(WEAK_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(DELAY);
    let Some(started) = game.player_skill_execution(player_id, WEAK_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_player_skill_visual(player_id, WEAK_SKILL_ID, 1);
    let _ = game.with_published_player_ai(player_id, ai, |game| {
        summon(game, player_id, destination, runtime);
    });
    if let Some(kernel) = game.player_skill_execution_mut(player_id, WEAK_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    // Summon может отказать, но его результат не меняет успешный End навыка.
    // Движение, AfterUse и освобождение visual принадлежат зарегистрированному End.
    terminal(QueuedSkillExecutionState::Completed)
}
