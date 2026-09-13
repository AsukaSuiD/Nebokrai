//! Огненный круг `CInfernol` (`0x135`).
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/infernol.cpp`. Маска 7×7 обходит клетки X→Y вокруг
//! исполнителя.
//!
//! Player и Monster используют один зарегистрированный Attack. Check сначала
//! читает таблицу и reuse для любого U; только Player затем требует ненулевой
//! MP и получает Move0. Первый AI повторно проверяет MP, списывает его, записывает CAN и
//! публикует visual0. По unsigned сроку start+delay visual1 предшествует чтению текущего
//! региона и центра U. Каждая клетка разрешается заново, поэтому callback
//! попадания меняет следующий обход; допуск идёт до дедупликации. Формула
//! directelementattack сохраняет Player-only EM, x87/FISTP, два RNG, weapon
//! factor как у ChainLightning, но без RP и без usage 20002. Общий End
//! сначала сбрасывает concrete phase, затем возвращает движение и передаёт
//! фактический аргумент базовому Attack End.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::directelementattack::apply_direct_element_attack;
use super::flash::cell_views;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::lightingarrowphalanx::ArrowTargetIdentity;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{check_cast_mana, spend_cast_mana, terminal};
use super::skillfactory::SkillOwner;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const INFERNOL_SKILL_ID: u32 = 0x135;

const PLAYER_TYPE: i32 = 400;
const SIDE: i32 = 7;
const SCOPE: [bool; 49] = [
    false, false, true, true, true, false, false,
    false, true, true, true, true, true, false,
    true, true, true, true, true, true, true,
    true, true, true, true, true, true, true,
    true, true, true, true, true, true, true,
    false, true, true, true, true, true, false,
    false, false, true, true, true, false, false,
];

fn resolved_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn check_infernol_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(source) = resolve_state_move_shape(game, region, identity) else {
        return false;
    };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player {
            game.send_skill_system_info(player, b"GS0278");
        }
        return false;
    }
    // У непользовательского U helper возвращает успех после таблицы/reuse;
    // MP и Move0 принадлежат только CPlayer::CheckCastCondition.
    check_cast_mana(game, instance, source, &properties)
}

fn attack_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) {
    // visual1 мог запустить callback: регион и центр читаются после него,
    // но source для контакта остаётся тем GetUser, который начал этот AI.
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
        return;
    };
    let user = user.shape();
    if !user.is_assigned_to_server_region() {
        return;
    }
    let region = user.get_region_id();
    let center_x = user.get_tile_x().unwrap_or(i32::MIN);
    let center_y = user.get_tile_y().unwrap_or(i32::MIN);
    let start_x = center_x.wrapping_sub(SIDE >> 1);
    let start_y = center_y.wrapping_sub(SIDE >> 1);
    let mut attacked = Vec::new();

    for x in 0..SIDE {
        for y in 0..SIDE {
            let index = y.wrapping_mul(SIDE).wrapping_add(x) as usize;
            if !SCOPE[index] {
                continue;
            }
            // Снимок существует только для одной клетки. Следующая клетка
            // строится после всех вложенных OnBeenAttacked/End этой клетки.
            for view in cell_views(game, region, start_x.wrapping_add(x), start_y.wrapping_add(y)) {
                let Some(target) = resolve_state_move_shape(game, region, view.identity) else {
                    continue;
                };
                let target = (target.shape().get_region_id(), target.shape().identity());
                if !game.live_skill_target_attackable_between(source, target) {
                    continue;
                }
                let target_key = ArrowTargetIdentity::new(target.0, target.1);
                if attacked.contains(&target_key) {
                    continue;
                }
                apply_direct_element_attack(game, instance, source, target, runtime);
                // Native добавляет после синхронного контакта: End внутри
                // callback не отменяет уже достигнутую запись списка.
                attacked.push(target_key);
            }
        }
    }
}

fn run_infernol_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(source) = resolved_user(game, skill) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);

    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }

    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game
        .registered_skill(instance)
        .map(|skill| skill.lifecycle().started_at_ms())
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    game.update_registered_skill_visual(instance, 1);
    attack_area(game, instance, source, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_infernol<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: INFERNOL_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: INFERNOL_SKILL_ID, .. }
    ) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let original_user = game
        .find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game,
        player_id,
        instance,
        dispatch,
        runtime,
        SkillVisualEffectKind::Infernol,
        |game, instance, _, runtime| check_infernol_cast(game, instance, original_user, runtime),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_infernol_ai,
    )
}

struct InfernolSkill;

impl RegisteredStateSkill for InfernolSkill {
    const ID: u32 = INFERNOL_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Infernol;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        _target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let source = game
            .registered_skill(instance)
            .and_then(|skill| resolved_user(game, skill));
        check_infernol_cast(game, instance, source, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_infernol_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
                end_state_skill(game, instance, 1, runtime)
            }
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_infernol<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<InfernolSkill, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}

pub(crate) fn publish_infernol_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CInfernol
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Infernol || effect.is_ended()
        })
    {
        return;
    }
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity).map(|source| source.shape()) else {
        return;
    };
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if source.identity().object_type == PLAYER_TYPE {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode {
        0 => 1,
        1 => 2,
        _ => return,
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if mode == 0 {
        message.add_long(source.get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
        message.add_long(source.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(source.get_tile_y().unwrap_or(i32::MIN));
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
