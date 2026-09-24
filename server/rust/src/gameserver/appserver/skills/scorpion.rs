//! Три прицельных арбалетных удара Scorpion (0xD1).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/scorpion.cpp.
//!
//! Общий Begin сохраняет исходного U и объектного S; координатная команда
//! проверяет свежего S после loop1 visual. Check читает таблицу до проверки
//! NULL S, затем reuse, дальность, арбалет категории 4 и signed MP. MP0
//! отклоняется молча; источник не типа Player проходит без Move0. Самонаведение
//! разрешено Begin, но AI отвергает его после проверки смерти цели.
//!
//! Каждый AI удерживает свежие таблицу и U/S. Первый списывает MP до
//! OnChangeStates и повторной проверки арбалета, затем задаёт направление,
//! заново строит путь, проверяет дальность/BLOCK_UNFLY и только потом пишет
//! CAN, visual0 и condition. Удары ждут абсолютные unsigned сроки
//! start+delay+first[+second[+third]], не более одного удара за AI.
//! Первый публикует visual1; каждый заново рассчитывает обычный оружейный
//! урон, только третий применяет skill factor. Допуска цели и RP в Attack нет.
//! NULL таблица Calculate сохраняет исходный пустой UNKNOWN/1 контакт.
//!
//! Два флага ударов независимы: вложенный End может очистить их до записи
//! завершившегося Attack. End сбрасывает фазу и оба флага, включает CAN до
//! свежего U Move1; только End(0) отправляет visual3. Общая база сохраняет
//! AfterUse и cooldown. Kernel остаётся опубликованным через весь AI;
//! общие ranged/weapon helpers заменяют повторение ресурса, пути и формулы.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    ArrowCastPathRule, CastPathBlock, RangedWeaponKind, check_ranged_weapon_cast,
    check_skill_path, prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use super::skillfactory::SkillOwner;
use super::weaponattack::{
    PlayerWeaponRoll, calculate_player_weapon_attack, calculate_unscaled_weapon_attack,
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

pub(crate) const SCORPION_SKILL_ID: u32 = 0xD1;
const FIRST_TIME: u32 = 15_001;
const SECOND_TIME: u32 = 15_002;
const THIRD_TIME: u32 = 15_003;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScorpionExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    first_attack: bool,
    second_attack: bool,
}

impl ScorpionExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), first_attack: false, second_attack: false }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.first_attack = false;
        self.second_attack = false;
        true
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(user) = original_user else { return false; };
    if resolve_state_move_shape(game, user.0, user.1).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    if game.skill_base_properties(skill.id(), skill.level()).is_none() { return false; }
    if target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity)).is_none() {
        ranged_weapon_failure(game, instance, (user.1.object_type == 400).then_some(user.1.id),
            10, RangedWeaponKind::Crossbow);
        return false;
    }
    check_ranged_weapon_cast(game, instance, user, ArrowCastPathRule::DistanceOnly,
        RangedWeaponKind::Crossbow, runtime)
}

fn attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), final_strike: bool, runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let calculated = if final_strike {
        calculate_player_weapon_attack(game, instance, source, target, TARGET_DAMAGE_FACTOR,
            PlayerWeaponRoll::AbsoluteRange)
    } else {
        calculate_unscaled_weapon_attack(game, instance, source, target, PlayerWeaponRoll::AbsoluteRange)
    };
    if let Some((master, attack)) = calculated {
        game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    }
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
    let source = resolve_state_move_shape(game, region, identity);
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let (Some(source), Some(target)) = (source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let targets_self = std::ptr::eq(source, target);
    let source = (source.shape().get_region_id(), source.shape().identity());
    let target = (target.shape().get_region_id(), target.shape().identity());
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0285"); }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if targets_self {
        ranged_weapon_failure(game, instance, player, 10, RangedWeaponKind::Crossbow);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::Crossbow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path(skill.lifecycle());
        if !check_skill_path(game, instance, &properties, &path, player,
            CastPathBlock::Named { target, message: b"GS0296" })
        { return terminal(QueuedSkillExecutionState::Rejected); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(first_attack) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<ScorpionExecutionState>()).map(|state| state.first_attack)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !first_attack {
        let first = properties.query_property(FIRST_TIME);
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if started.wrapping_add(first).wrapping_add(delay) <= runtime.now_milliseconds() {
            game.update_registered_skill_visual(instance, 1);
            attack(game, instance, source, target, false, runtime);
            if let Some(state) = game.registered_skill_mut(instance)
                .and_then(|skill| skill.player_state_mut::<ScorpionExecutionState>())
            { state.first_attack = true; }
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if game.registered_skill(instance).and_then(|skill| skill.player_state::<ScorpionExecutionState>())
            .is_none_or(|state| !state.first_attack)
        { return terminal(QueuedSkillExecutionState::Pending); }
    }
    let Some(second_attack) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<ScorpionExecutionState>()).map(|state| state.second_attack)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !second_attack {
        let second = properties.query_property(SECOND_TIME);
        let first = properties.query_property(FIRST_TIME);
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if started.wrapping_add(second).wrapping_add(first).wrapping_add(delay) <= runtime.now_milliseconds() {
            attack(game, instance, source, target, false, runtime);
            if let Some(state) = game.registered_skill_mut(instance)
                .and_then(|skill| skill.player_state_mut::<ScorpionExecutionState>())
            { state.second_attack = true; }
            return terminal(QueuedSkillExecutionState::Pending);
        }
    }
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<ScorpionExecutionState>())
        .is_some_and(|state| state.first_attack && state.second_attack)
    {
        let third = properties.query_property(THIRD_TIME);
        let second = properties.query_property(SECOND_TIME);
        let first = properties.query_property(FIRST_TIME);
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if started.wrapping_add(third).wrapping_add(second).wrapping_add(first).wrapping_add(delay)
            <= runtime.now_milliseconds()
        {
            attack(game, instance, source, target, true, runtime);
            return terminal(QueuedSkillExecutionState::Completed);
        }
    }
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_scorpion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target| {
            original_user.and_then(|user| game.player_skill_begin_object(user.0, target))
        })
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Scorpion,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            if !check_cast(game, instance, original_user, target, runtime) { return false; }
            if let Some(state) = game.registered_skill_mut(instance)
                .and_then(|skill| skill.player_state_mut::<ScorpionExecutionState>())
            {
                state.first_attack = false;
                state.second_attack = false;
                state.kernel.lifecycle_mut().set_available(true);
            }
            true
        },
        |dispatch, started| ScorpionExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

pub(crate) fn publish_scorpion_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) -> bool {
    if skill.owner() != SkillOwner::CScorpion || skill.visual_effect().is_none_or(|effect|
        effect.kind() != SkillVisualEffectKind::Scorpion || effect.is_ended())
    { return true; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return true; };
    let source = user.shape();
    if matches!(mode, 2 | 4 | 7 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return true;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return true };
    let target = if mode == 1 {
        let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return false; };
        let Some(target) = resolve_state_move_shape(game, region, identity) else { return false; };
        Some(target.shape())
    } else { None };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
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
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
    true
}
