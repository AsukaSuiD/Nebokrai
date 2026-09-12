//! Поклеточный арбалетный выстрел PoisonMoth (0xCF).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/poisonmoth.cpp.
//!
//! Общий Begin удерживает исходного U; Check получает исходного S для
//! объектной команды и свежего S для координатной. После запрета самонаведения
//! следуют reuse, свежий путь, дальность, BLOCK_UNFLY, арбалет категории 4 и
//! signed MP. MP0 тихо отклоняется; источник не типа Player проходит без Move0.
//! Отказ Check добавляет visual2 перед общим End(0).
//!
//! AI удерживает таблицу свойств и U без ранних проверок смерти или региона.
//! MP списывается до OnChangeStates и повторной проверки арбалета, затем идут
//! CAN, свежий S, направление и visual0. После абсолютного unsigned срока
//! Move1 предшествует новому пути без заданной длины и проверке MAX+1.
//! Первая непролётная клетка либо конец пути задаёт endpoint и полное время
//! полёта; S очищается до visual1, attacking включается после него. Prepared
//! здесь не записывается. Один AI обрабатывает не более одной клетки.
//!
//! Клеточный срок заново читает step/delay; текущий регион U проверяется после
//! часов, даже для завершённого пути. BLOCK_SHAPE останавливает выстрел после
//! всех допустимых живых Move-целей; последняя identity пишется перед ударом.
//! Дедупликации нет. Visual3 после попадания или стены повторяется в конечном
//! проходе перед End(1). End сбрасывает фазу, счётчики и цель, освобождает путь
//! до свежего U Move1 и общего Attack End с исходным аргументом.
//! Безопасный Vec и зарегистрированный kernel заменяют контейнеры/указатели;
//! состояние остаётся опубликованным на протяжении всех боевых callbacks.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::crossbowattack::run_poison_moth_cell;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast,
    prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const POISON_MOTH_SKILL_ID: u32 = 0xCF;
pub(super) const PLAYER_TYPE: i32 = 400;
pub(super) const MONSTER_TYPE: i32 = 600;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PoisonMothExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    attacking_started: bool,
    missile_flying_time: u32,
    path: Vec<(i32, i32, u8)>,
    current_position: u32,
    end_tile: (i32, i32),
    visual_target: (i32, i32),
}

impl PoisonMothExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            path: Vec::new(),
            current_position: 0,
            end_tile: (0, 0),
            visual_target: (0, 0),
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }
    pub(crate) const fn end_tile(&self) -> (i32, i32) { self.end_tile }
    pub(crate) const fn visual_target(&self) -> (i32, i32) { self.visual_target }

    pub(super) fn set_visual_target(&mut self, target: ShapeIdentity) {
        self.visual_target = (target.object_type, target.id);
    }

    pub(crate) fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.current_position = 0;
        self.end_tile = (0, 0);
        self.visual_target = (0, 0);
        drop(std::mem::take(&mut self.path));
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(user) = original_user else { return false; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return false; };
    let targets_self = target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .is_some_and(|target| std::ptr::eq(source, target));
    if targets_self {
        ranged_weapon_failure(game, instance, (user.1.object_type == PLAYER_TYPE).then_some(user.1.id),
            10, RangedWeaponKind::Crossbow);
        return false;
    }
    check_ranged_weapon_cast(game, instance, user, ArrowCastPathRule::DistanceAndBlocks,
        RangedWeaponKind::Crossbow, runtime)
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
    let Some(user) = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let player = (user.1.object_type == PLAYER_TYPE).then_some(user.1.id);
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::Crossbow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else {
                    return terminal(QueuedSkillExecutionState::Rejected);
                };
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<PoisonMothExecutionState>())
        .map(|state| state.attacking_started)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if runtime.now_milliseconds() < started.wrapping_add(delay) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path(skill.lifecycle());
        let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.path = path;
        if properties.query_property(TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(TARGET_MAX_DISTANCE);
            if maximum.wrapping_add(1) < state.path.len() as u32 {
                ranged_weapon_failure(game, instance, player, 11, RangedWeaponKind::Crossbow);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let stop = state.path.iter().position(|cell| cell.2 == 2).unwrap_or(state.path.len());
        let blocked_endpoint = state.path.get(stop).map(|cell| (cell.0, cell.1));
        let last_endpoint = state.path.last().map(|cell| (cell.0, cell.1));
        if let Some(endpoint) = blocked_endpoint {
            if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_destination(endpoint); }
        }
        let flight = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(stop as u32);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(state) = skill.player_state_mut::<PoisonMothExecutionState>() { state.missile_flying_time = flight; }
        if blocked_endpoint.is_none() {
            if let Some(endpoint) = last_endpoint { skill.lifecycle_mut().set_destination(endpoint); }
        }
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        {
            state.attacking_started = true;
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    let step = properties.query_property(MISSILE_FLYING_TIME);
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(state) = skill.player_state::<PoisonMothExecutionState>() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if !state.attacking_started { return terminal(QueuedSkillExecutionState::Pending); }
    let position = state.current_position;
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let deadline = step.wrapping_mul(position).wrapping_add(delay).wrapping_add(skill.lifecycle().started_at_ms());
    if runtime.now_milliseconds() < deadline { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(region) = resolve_state_move_shape(game, user.0, user.1)
        .filter(|source| source.shape().is_assigned_to_server_region())
        .and_then(|source| game.find_region(source.shape().get_region_id()))
    else { return terminal(QueuedSkillExecutionState::Pending); };
    let Some(cell) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<PoisonMothExecutionState>())
        .and_then(|state| state.path.get(state.current_position as usize).copied())
    else {
        game.update_registered_skill_visual(instance, 3);
        return terminal(QueuedSkillExecutionState::Completed);
    };
    let region_id = region.region_id();
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
    { state.end_tile = (cell.0, cell.1); }
    let block = game.find_region(region_id).map_or(2, |region| region.base().skill_cell_block(cell.0, cell.1));
    let stop = match block {
        3 => run_poison_moth_cell(game, instance, user, (cell.0, cell.1), runtime),
        2 => true,
        _ => false,
    };
    if stop {
        game.update_registered_skill_visual(instance, 3);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
        { state.current_position = state.path.len() as u32; }
    }
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<PoisonMothExecutionState>())
    { state.current_position = state.current_position.wrapping_add(1); }
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_poison_moth<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target| {
            let player = game.find_player(player_id)?;
            game.player_skill_begin_object(player.shape().get_region_id(), target)
        })
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::CrossbowCast,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            let accepted = check_cast(game, instance, original_user, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| PoisonMothExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

pub(super) fn weapon_is_crossbow(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 4) }
pub(super) fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch { PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y)), PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)) }
}
pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}
pub(super) fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type { PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }), _ => None }
}
pub(super) fn cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() }; let (area_width, area_height) = game.area_dimensions(); let mut shapes = Vec::new(); if region.get_shapes(x, y, area_width, area_height, game, &mut shapes).is_err() { return Vec::new() } shapes.into_iter().map(|shape| shape.identity).collect()
}
