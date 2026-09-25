//! Управляющий выстрел BoaLock (0xD2).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/boalock.cpp.
//! Общий Begin сохраняет исходные U/S для объектной проверки; координатная
//! перегрузка разрешает S после loop1-visual. Check отклоняет self до свойств,
//! затем требует S, reuse, допустимый путь и ненулевую MP-цену. При отказе
//! Begin добавляет visual2 перед End(0). Требования к оружию здесь нет.
//!
//! Каждый AI сохраняет таблицу свойств и разрешает U/S до callbacks. Смерть S
//! отклоняет выстрел; первый AI списывает MP→OnChangeStates→CAN и читает
//! координаты в порядке S.Y/X→U.Y/X перед направлением/visual0. Путь выпуска
//! строится заново по живой базе, но имя в ошибке относится к захваченному S.
//!
//! Две проверки абсолютного unsigned start+delay независимы. Срок контакта
//! не включает missile-time: последний служит только пакету выпуска. Если
//! первые часы ещё не допускают выпуск, вторые всё равно могут допустить
//! контакт. Финальная проверка зависит от condition, а не attacking/prepared.
//! Путь локален; единственный payload хранит лишь kernel, attacking и duration.
//!
//! Перед контактом проверяются текущий регион захваченного S и его допуск.
//! boalockattack сохраняет свежие свойства/уровни, порядок Lock→KnockOut и
//! пустой raw OnBeenAttacked; при NULL properties отменяет также контакт.
//! End сбрасывает фазу, attacking/time, затем свежий U Move1 и общий хвост.
//! Категория конструктора State не меняет совместимый базовый Begin/End.
//! Vec и зарегистрированный kernel заменяют native контейнеры и указатели.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::boalockattack::apply_boa_lock_attack;
use super::kernel::{SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastPathBlock, check_cast_mana, check_skill_path, spend_cast_mana, terminal,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{BoaLockExecutionState};

pub(crate) const BOA_LOCK_SKILL_ID: u32 = 0xD2;
const PLAYER_TYPE: i32 = 400;
const MISSILE_FLYING_TIME: u32 = 10_008;

fn failure(
    game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, mode: u32, text: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player) = player { game.send_skill_system_info(player, text); }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(user) = original_user else { return false; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return false; };
    let target = original_target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let player = (source.shape().identity().object_type == PLAYER_TYPE).then_some(source.shape().identity().id);
    if target.is_some_and(|target| std::ptr::eq(source, target)) {
        failure(game, instance, player, 10, b"GS0286");
        return false;
    }
    let target = target.map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let Some(target) = target else {
        failure(game, instance, player, 10, b"GS0294");
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player, 13, b"GS0278");
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path(
        game, instance, &properties, &path, player,
        CastPathBlock::Named { target, message: b"GS0295" },
    ) { return false; }
    check_cast_mana(game, instance, user, &properties)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    let (Some(user), Some(target)) = (user, target) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let player = (user.1.object_type == PLAYER_TYPE).then_some(user.1.id);
    if game.move_shape_health(target.0, target.1) == Some(0) {
        failure(game, instance, player, 10, b"GS0285");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).and_then(|skill| skill.execution_stage())
        .is_none_or(|stage| matches!(stage, SkillStage::Idle | SkillStage::Begin))
    { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<BoaLockExecutionState>())
        .map(|state| state.attacking_started)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if runtime.now_milliseconds() >= started.wrapping_add(delay) {
            if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let path = game.skill_target_path(skill.lifecycle());
            if !check_skill_path(
                game, instance, &properties, &path, player,
                CastPathBlock::Named { target, message: b"GS0296" },
            ) { return terminal(QueuedSkillExecutionState::Rejected); }
            let duration = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(path.len() as u32);
            if let Some(state) = game.registered_skill_mut(instance)
                .and_then(|skill| skill.player_state_mut::<BoaLockExecutionState>())
            { state.missile_flying_time = duration; }
            game.update_registered_skill_visual(instance, 1);
            if let Some(state) = game.registered_skill_mut(instance)
                .and_then(|skill| skill.player_state_mut::<BoaLockExecutionState>())
            {
                state.attacking_started = true;
                state.kernel.lifecycle_mut().mark_prepared();
                let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
                let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
            }
        }
    }
    // Native повторяет срок даже без выпуска: missile-time не входит в него.
    if game.registered_skill(instance).and_then(|skill| skill.execution_stage())
        .is_none_or(|stage| matches!(stage, SkillStage::Idle | SkillStage::Begin))
    { return terminal(QueuedSkillExecutionState::Pending); }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1)
        && sufferer.shape().is_assigned_to_server_region()
    {
        let region = sufferer.shape().get_region_id();
        if game.find_region(region).is_some()
            && game.live_skill_target_attackable_between(user, target)
        {
            apply_boa_lock_attack(game, instance, user, target, runtime);
        }
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_boa_lock<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target|
            original_user.and_then(|source| game.player_skill_begin_object(source.0, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::BoaLock,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            let accepted = check_cast(game, instance, original_user, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| BoaLockExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

pub(crate) fn publish_boa_lock_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.id() != BOA_LOCK_SKILL_ID || skill.visual_effect().is_none_or(|effect|
        effect.kind() != SkillVisualEffectKind::BoaLock || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let target = match mode {
        0 => None,
        1 => {
            let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
            Some(target.shape())
        }
        _ => return,
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if mode == 0 { 1 } else { 2 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
        let Some(state) = skill.player_state::<BoaLockExecutionState>() else { return; };
        message.add_ulong(state.missile_flying_time);
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
