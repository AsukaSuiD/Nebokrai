//! Божественная кара CGodPunishment (0x13A).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godpunishment.cpp.
//! Общий зарегистрированный экземпляр игрока/монстра хранит единственную базу
//! Begin и фазу AI. Visual создаётся до Check; отказ вызывает End0, выпуск —
//! End1 независимо от результата Summon. End очищает фазу, читает GetUser
//! без изменения движения и выполняет общий SummonEnd с оружейным AfterUse.
//!
//! Check допускает NULL S и самоцель, проверяет абсолютный reuse, путь и MP.
//! Первый AI списывает MP и публикует OnChangeStates до CAN и поворота;
//! затем повторяет только проверку дальности. Отказ не возвращает потраченное.
//! Ни Check, ни AI не блокируют движение. Срок выпуска — unsigned start+delay.
//! После задержки ненулевые type/id требуют живой цели; её X/Y сохраняются,
//! а S очищается до visual1. Сообщение выпуска поэтому содержит нулевые type/id.
//!
//! Summon проверяет фактический регион U и BLOCK2 до свежих свойств и очистки S.
//! MasterInfo сохраняет принадлежность и PK игрока, но не страну. EM читается
//! один раз вхолостую, затем EM/MAX/MIN, уровень и lifetime образуют независимую
//! область. Региональный runtime выполняет замену прежних областей, Add,
//! кодирование и BF502, сохраняя частичные изменения при отказе.
//! SlotMap и Vec заменяют указатели/STL; геометрия и MP используют существующие
//! механизмы, а игровые порядок и сообщения остаются явными.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_SUMMONED_LIFETIME,
};
use super::baseprojectilecheck::{check_god_punishment_cast, check_god_punishment_distance};
use super::godpunishmentphalanx::CGodPunishmentPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{spend_cast_mana, terminal};
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::weaponattack::source_master;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

pub(crate) const GOD_PUNISHMENT_SKILL_ID: u32 = 0x13a;

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    x: i32, y: i32, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    let Some(region) = game.find_region(region_id) else { return; };
    if region.base().skill_cell_block(x, y) == 2 { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    let destination = skill.lifecycle().destination();
    skill.lifecycle_mut().set_point_target(destination);
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let element = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CGodPunishmentPhalanx::new(id, master, started, lifetime, level, minimum, maximum, element);
    let _ = game.spawn_god_punishment_phalanx(instance, region_id, phalanx, x, y, started, runtime);
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
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let source = (user.shape().get_region_id(), user.shape().identity());
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) { return terminal(QueuedSkillExecutionState::Rejected); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let (target_x, target_y) = match resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| resolve_state_move_shape(game, target.0, target.1))
        {
            Some(target) => (target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN)),
            None => skill.lifecycle().destination(),
        };
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        if !check_god_punishment_distance(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let started = skill.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let target = skill.lifecycle().sufferer().1;
    if target.object_type != 0 && target.id != 0 {
        let target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| resolve_state_move_shape(game, target.0, target.1))
            .map(|target| (target.shape().get_region_id(), target.shape().identity()));
        let Some(target) = target.filter(|target| game.move_shape_health(target.0, target.1) != Some(0)) else {
            game.update_registered_skill_visual(instance, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(target) = resolve_state_move_shape(game, target.0, target.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
        let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_point_target((x, y)); }
    }
    game.update_registered_skill_visual(instance, 1);
    if let Some((x, y)) = game.registered_skill(instance).map(|skill| skill.lifecycle().destination()) {
        summon(game, instance, source, x, y, runtime);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_god_punishment<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != GOD_PUNISHMENT_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::BaseProjectile,
        |game, instance, _, runtime| check_god_punishment_cast(game, instance, original_user, runtime),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}

struct GodPunishmentSkill;
impl RegisteredStateSkill for GodPunishmentSkill {
    const ID: u32 = GOD_PUNISHMENT_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::BaseProjectile;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;
    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let user = game.registered_skill(instance).map(|skill| skill.lifecycle().user());
        check_god_punishment_cast(game, instance, user, runtime)
    }
    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_god_punishment<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<GodPunishmentSkill, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
