//! Воспламенение: подготовка арбалетного удара и расход горючей смеси на цели.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/ignition.cpp.
//! Навык сохраняет расход MP до поздней проверки оружия, абсолютный срок
//! применения и выбор коэффициента по состояниям цели в момент расчёта.
//! Общая арена выполняет End и удаление смеси после OnBeenAttacked;
//! зарегистрированный CAttackSkill владеет завершением и износом оружия.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::kerosene::{
    kerosene_path_block, kerosene_skill_target, kerosene_source_position,
    kerosene_target_position,
};
use super::kerosenestate::KEROSENE_STATE_ID;
use super::poisonmoth::{master_info, weapon_is_crossbow};
use super::skillfactory::{SkillOwner, UNKNOWN_SKILL_ID};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const IGNITION_SKILL_ID: u32 = 0xf2;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const SECOND_TIME: u32 = 15_002;
const THIRD_TIME: u32 = 15_003;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const TARGET_DAMAGE_FACTOR_2: u32 = 20_021;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) fn is_ignition_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == IGNITION_SKILL_ID
}

/// Возвращает необходимость общего visual-хвоста. Только исчезнувшая цель
/// режима попадания завершает исходный UpdateVisualEffect до этого хвоста.
pub(crate) fn publish_ignition_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) -> bool {
    if skill.owner() != SkillOwner::CIgnition
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Ignition || effect.is_ended()
        })
    { return true; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return true; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 4 | 7 | 10 | 11 | 13 | 14 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return true;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return true };
    let target = if action == 2 {
        let Some((region, target)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return false; };
        let Some(target) = resolve_state_move_shape(game, region, target) else { return false; };
        Some(target.shape())
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if let Some(target) = target {
        let (Ok(x), Ok(y)) = (target.get_tile_x(), target.get_tile_y()) else { return true; };
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(x);
        message.add_long(y);
        if let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) {
            message.add_long(0);
            message.add_ulong(properties.query_property(SECOND_TIME));
            let third = properties.query_property(THIRD_TIME);
            let second = properties.query_property(SECOND_TIME);
            message.add_ulong(third.wrapping_add(second));
        }
    } else {
        message.add_long(source.get_direction());
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
    true
}

pub(crate) fn complete_player_ignition<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, IGNITION_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_ignition<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, IGNITION_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn send_failure(game: &mut CGame, player_id: i32, code: u32, amount: u32) {
    game.update_player_skill_visual(player_id, IGNITION_SKILL_ID, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        10 => game.send_skill_system_info(player_id, b"GS0286"),
        11 => game.send_skill_system_info(player_id, b"GS0290"),
        13 => game.send_skill_system_info(player_id, b"GS0278"),
        14 => game.send_skill_system_info(player_id, b"GS0293"),
        _ => {}
    }
}

fn send_path_failure(game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), id: &[u8]) {
    game.update_player_skill_visual(player_id, IGNITION_SKILL_ID, 15);
    let name = game.base_magic_target_name(target.0, target.1).unwrap_or_default();
    game.send_skill_system_info_with_text(player_id, id, name);
}

fn reject_begin(game: &mut CGame, player_id: i32) -> QueuedSkillExecutionOutcome {
    game.update_player_skill_visual(player_id, IGNITION_SKILL_ID, 2);
    terminal(QueuedSkillExecutionState::Rejected)
}

fn calculate_attack(game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), attack: &mut AttackInformation) {
    let Some(level) = game.registered_player_skill(player_id, IGNITION_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return; };
    let Some(properties) = game.skill_base_properties(IGNITION_SKILL_ID, level) else { return; };
    attack.skill_id = IGNITION_SKILL_ID;
    attack.skill_level = level as u8;
    attack.damage_modifier = 0;
    let soaked = resolve_state_move_shape(game, target.0, target.1)
        .is_some_and(|shape| shape.has_state_by_skill_id(KEROSENE_STATE_ID));
    let factor = properties.query_property(if soaked { TARGET_DAMAGE_FACTOR_2 } else { TARGET_DAMAGE_FACTOR });
    // В x87 исходный unsigned factor сохраняет точность до записи float.
    attack.damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;

    let Some(player) = game.find_player(player_id) else { return; };
    let maximum = player.combat_properties().maximum_attack as i32;
    let minimum = player.combat_properties().minimum_attack as i32;
    let span = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random = game.skill_random_below(span);
    let Some(player) = game.find_player(player_id) else { return; };
    let physical = (player.combat_properties().minimum_attack as i32).wrapping_add(random).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    let element = (player.combat_properties().add_element_attack as i32).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    let soul = i32::from(player.combat_properties().add_soul_attack);
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    let cch = i32::from(player.combat_properties().cch);
    if game.skill_random_below(100) < cch {
        attack.critical = true;
        for power in &mut attack.damages {
            let rate = game.globe_setup().critical_rate();
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

fn apply_ignition<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) {
    let Some(source) = game.find_player(player_id).map(|player| player.shape().identity())
    else { return; };
    if source == target.1 { return; }
    let attackable = match target.1.object_type {
        400 | 600 => game.live_skill_target_attackable(target.0, source, target.1),
        1100 | 1200 => game.stationary_build_attackable_by_player(player_id, target.0, target.1),
        _ => false,
    };
    if !attackable { return; }
    let Some(player) = game.find_player(player_id) else { return; };
    let master = master_info(player);
    let mut attack = AttackInformation {
        skill_id: UNKNOWN_SKILL_ID, skill_level: 1,
        attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: 0, damage_factor: 1.0, damage_modifier: 0,
        critical: false, blast_attack: false, full_miss: 0, damages: Vec::new(),
    };
    calculate_attack(game, player_id, target, &mut attack);
    match target.1.object_type {
        400 => game.apply_owned_skill_attack_to_player(master, target.1.id, target.0, attack, runtime),
        600 => game.apply_owned_skill_attack_to_monster(master, target.1.id, target.0, attack, runtime),
        1100 | 1200 => game.receive_stationary_build_skill_attack(target.0, target.1, attack, runtime),
        _ => return,
    }
    // Защита и callbacks попадания могут изменить арену. Ищется первая живая
    // смесь уже после удара; её End может снова заменить тот же слот.
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == KEROSENE_STATE_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
}

pub(crate) fn execute_player_ignition<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_ignition_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let beginning = game.player_skill_execution(player_id, IGNITION_SKILL_ID).is_none();
    if beginning {
        game.replace_player_skill_visual_effect(
            player_id, IGNITION_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::Ignition, 1),
        );
    }
    let Some(level) = game.registered_player_skill(player_id, IGNITION_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    // CheckCast проверяет U/S до таблицы свойств; AI начинает с таблицы.
    if !beginning && game.skill_base_properties(IGNITION_SKILL_ID, level).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.find_player(player_id).is_none() {
        return if beginning { reject_begin(game, player_id) }
            else { terminal(QueuedSkillExecutionState::Rejected) };
    }
    let Some(target) = kerosene_skill_target(game, player_id, IGNITION_SKILL_ID) else {
        if beginning {
            send_failure(game, player_id, 10, 0);
            return reject_begin(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if beginning {
        if target.1.object_type == PLAYER_TYPE && target.1.id == player_id {
            send_failure(game, player_id, 10, 0);
            return reject_begin(game, player_id);
        }
        let Some(properties) = game.skill_base_properties(IGNITION_SKILL_ID, level) else {
            return reject_begin(game, player_id);
        };
        let started = game.player_skill_lifecycle(player_id, IGNITION_SKILL_ID)
            .expect("общий Begin сохранил базу воспламенения").started_at_ms();
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, IGNITION_SKILL_ID), reuse, runtime.now_milliseconds()) {
            send_failure(game, player_id, 13, 0);
            return reject_begin(game, player_id);
        }
        let Some(blocked) = kerosene_path_block(game, player_id, IGNITION_SKILL_ID, properties) else {
            send_failure(game, player_id, 11, 0);
            return reject_begin(game, player_id);
        };
        if blocked {
            send_path_failure(game, player_id, target, b"GS0291");
            return reject_begin(game, player_id);
        }
        let Some(player) = game.find_player(player_id) else { return reject_begin(game, player_id); };
        if !weapon_is_crossbow(game, player) {
            send_failure(game, player_id, 14, 0);
            return reject_begin(game, player_id);
        }
        if properties.query_property(USER_MP_LOSE) != 0
            && (player.mana().wrapping_sub(properties.query_property(USER_MP_LOSE)) as i32) < 0
        {
            let amount = properties.query_property(USER_MP_LOSE);
            send_failure(game, player_id, 7, amount);
            return reject_begin(game, player_id);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
        return terminal(QueuedSkillExecutionState::Begun);
    }

    if game.base_magic_target_dead(target.0, target.1) {
        game.update_player_skill_visual(player_id, IGNITION_SKILL_ID, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, IGNITION_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let properties = game.skill_base_properties(IGNITION_SKILL_ID, level)
            .expect("таблица свойств активного навыка сохраняется");
        let mp_loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            let amount = properties.query_property(USER_MP_LOSE);
            send_failure(game, player_id, 7, amount);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_crossbow(game, player)) {
            send_failure(game, player_id, 14, 0);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(properties) = game.skill_base_properties(IGNITION_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, IGNITION_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_be_breaked != 0);
        }
        let Some((target_x, target_y)) = kerosene_target_position(game, target) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some((_, source_x, source_y)) = kerosene_source_position(game, player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        game.update_player_skill_visual(player_id, IGNITION_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, IGNITION_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(IGNITION_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.player_skill_execution(player_id, IGNITION_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    let Some(properties) = game.skill_base_properties(IGNITION_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(blocked) = kerosene_path_block(game, player_id, IGNITION_SKILL_ID, properties) else {
        send_failure(game, player_id, 11, 0);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if blocked {
        send_path_failure(game, player_id, target, b"GS0307");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let has_visual = game.registered_player_skill(player_id, IGNITION_SKILL_ID)
        .and_then(|address| game.registered_skill(address))
        .is_some_and(|skill| skill.visual_effect().is_some());
    if !has_visual { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_player_skill_visual(player_id, IGNITION_SKILL_ID, 1);
    if let Some(target) = kerosene_skill_target(game, player_id, IGNITION_SKILL_ID) {
        game.with_published_player_ai(player_id, ai, |game| apply_ignition(game, player_id, target, runtime));
    }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, IGNITION_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
