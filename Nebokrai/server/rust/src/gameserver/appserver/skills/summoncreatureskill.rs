//! Общий достигнутый путь исполнения четырёх исходных владельцев `CSummonSkill`.
//!
//! `CSummonCorpseCandle`, `CSummonSkeleton` и `CSummonSpore` различаются
//! только идентификатором навыка. `CBossFiendSummon` дополнительно выбирает
//! одну из трёх разновидностей ровно одним исходным броском на всё применение.
//! Модуль сохраняет объектный, точечный и self-входы игрока и общий объектный
//! путь monster/pet, максимальную дистанцию цели, поворот владельца к
//! сохранённой точке эффекта, задержку повторного применения и исполнения,
//! пакеты `0xBFE01` и последовательность вызовов создания.
//! Поиск владельцев и around-доставка остаются у `CGame`; создаваемая сущность
//! сразу публикуется через `CServerRegion::add_summoned_creature`.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::bossfiendsummon::{BOSS_FIEND_SUMMON_SKILL_ID, summoned_creature_usage};
use super::flash::master_info;
use super::summoncorpsecandle::SUMMON_CORPSE_CANDLE_SKILL_ID;
use super::summonskeleton::SUMMON_SKELETON_SKILL_ID;
use super::summonspore::SUMMON_SPORE_SKILL_ID;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::ai::monsterai::{
    approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME: u32 = 30_001;
const SKILL_USAGE_SUMMONED_CREATURE_ID: u32 = 30_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SummonCreatureProgress {
    destination_x: i32,
    destination_y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSummonCreatureExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination: (i32, i32),
    variant_index: usize,
}

impl PlayerSummonCreatureExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), variant_index: usize, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination, variant_index }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn player_skill_id(dispatch: PlayerSkillDispatch) -> u32 {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    }
}

fn player_variant_index(skill_id: u32) -> Option<usize> {
    match skill_id {
        SUMMON_CORPSE_CANDLE_SKILL_ID => Some(0),
        SUMMON_SKELETON_SKILL_ID => Some(1),
        SUMMON_SPORE_SKILL_ID => Some(2),
        _ => None,
    }
}

pub(crate) fn is_player_summon_creature_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    player_variant_index(player_skill_id(dispatch)).is_some()
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn player_destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch, source: (i32, i32)) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => Some(source),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)),
    }
}

fn send_player_visual(game: &mut CGame, player_id: i32, skill_id: u32, skill_level: i32, action: u8, destination: (i32, i32)) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(direction);
    } else {
        message.add_long(0);
        message.add_long(0);
        message.add_long(destination.0);
        message.add_long(destination.1);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
}

fn finish_player_summon_creature<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, variant_index: usize, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_summon_creature_used(variant_index, now_ms));
}

fn abort_player_summon_creature(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
    abort_skill(game, player_id);
}

pub(crate) fn cancel_player_summon_creature<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some((dispatch, _)) = player_ai.summon_creature().map(|state| (state.kernel().dispatch(), state.variant_index)) else { return false };
    abort_player_summon_creature(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_summon_creature<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let skill_id = player_skill_id(dispatch);
    let Some(variant_index) = player_variant_index(skill_id) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some((region_id, source_x, source_y, skill_level, master)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(skill_id), master_info(player)))) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(skill_id, skill_level).cloned() else { if player_ai.summon_creature().is_some() { abort_player_summon_creature(game, player_id); } return player_terminal(QueuedSkillExecutionState::Rejected) };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let amount = properties.query_property(SKILL_USAGE_CONST);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
    let picture_id = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_ID);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();
    if player_ai.summon_creature().is_none() {
        if player_ai.summon_creature_last_used_ms(variant_index) != 0 && !time_reached(now_ms, player_ai.summon_creature_last_used_ms(variant_index), reuse_delay_ms) {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 0x0d);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(destination) = player_destination(game, region_id, dispatch, (source_x, source_y)) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
        if maximum_distance != 0
            && game
                .base_magic_path(
                    region_id,
                    source_x,
                    source_y,
                    destination.0,
                    destination.1,
                    None,
                )
                .len()
                > maximum_distance as usize
        {
            game.send_self_state_skill_failure(0x000b_fe01, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id)); }
        player_ai.begin_summon_creature(PlayerSummonCreatureExecutionState::begin(dispatch, destination, variant_index, now_ms));
    } else if player_ai.summon_creature().is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let destination = player_ai.summon_creature().map(|state| state.destination).expect("выполнение призыва хранит координаты эффекта");
    if player_ai.summon_creature().is_some_and(|state| state.kernel().stage() == SkillStage::Begin) {
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                destination.0,
                destination.1,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_player_visual(game, player_id, skill_id, skill_level, 1, destination);
        if let Some(state) = player_ai.summon_creature_mut() { let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = player_ai.summon_creature().map(|state| state.kernel().started_at_ms()).expect("выполнение призыва хранит время начала");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return player_terminal(QueuedSkillExecutionState::Pending) }
    restore_player_movement(game, player_id);
    send_player_visual(game, player_id, skill_id, skill_level, 2, destination);
    let property = game.find_monster_property_by_picture_id(picture_id).cloned();
    if let Some(mut owner) = game.take_region_owner(region_id) {
        for _ in 0..amount {
            let mut tile_x = 0;
            let mut tile_y = 0;
            if let Ok(position) = game.random_region_position_owned(owner.base(), source_x.wrapping_sub(4), source_y.wrapping_sub(4), 8, 8) && position.found { tile_x = position.x; tile_y = position.y; }
            if let Some(property) = property.as_ref() {
                let _ = game.add_summoned_creature_owned(owner.base_mut(), property, master, tile_x, tile_y, -1, lifetime_ms);
            }
        }
        game.restore_region_owner(owner);
    }
    if let Some(state) = player_ai.summon_creature_mut() { let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_summon_creature(game, player_id, player_ai, variant_index, runtime);
    player_terminal(QueuedSkillExecutionState::Completed)
}

fn target_coordinates(
    game: &CGame,
    region: &CServerRegion,
    target: ShapeIdentity,
) -> Option<(i32, i32)> {
    let shape = match target.object_type {
        400 => game.find_player(target.id).and_then(|player| {
            (player.server_region_id() == Some(region.id)).then(|| player.shape())
        }),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .map(|monster| monster.move_shape().shape()),
        _ => None,
    }?;
    Some((shape.get_tile_x().ok()?, shape.get_tile_y().ok()?))
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_id: u32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "пакет буквально сохраняет поля исходного сетевого эффекта")]
fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_id: u32,
    skill_level: u16,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет идентификатор, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_summon_creature(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
) -> bool {
    let Some((source, property, attack_interval_ms, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let attack_interval_ms = monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().clone(),
                property,
                attack_interval_ms,
                monster.base_attack_cast(),
                monster.skill_last_used_ms(skill_id),
            ))
        })
    else {
        return false;
    };

    if let Some(cast) = cast {
        if cast.dispatch().skill_id != skill_id {
            return false;
        }
        if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) {
            return true;
        }
        let target = target_coordinates(game, region, cast.dispatch().target).or_else(|| {
            region
                .find_monster_by_id(monster_id)
                .and_then(|monster| monster.summon_creature_progress())
                .map(|progress| (progress.destination_x, progress.destination_y))
        });
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        if let Some((target_x, target_y)) = target {
            send_fire(game, region, &source, monster_id, skill_id, skill_level, target_x, target_y);
        }

        let amount = properties.query_property(SKILL_USAGE_CONST);
        let summoned_creature_usage = if skill_id == BOSS_FIEND_SUMMON_SKILL_ID {
            summoned_creature_usage(game.skill_random_below(3))
        } else {
            SKILL_USAGE_SUMMONED_CREATURE_ID
        };
        let source_x = source.get_tile_x().unwrap_or_default();
        let source_y = source.get_tile_y().unwrap_or_default();
        let master = MasterInfo {
            master_type: MONSTER_TYPE,
            master_id: monster_id,
            ..MasterInfo::default()
        };
        for _ in 0..amount {
            let mut tile_x = 0;
            let mut tile_y = 0;
            if let Ok(position) = game.random_region_position_owned(
                region, source_x.wrapping_sub(4), source_y.wrapping_sub(4), 8, 8,
            ) && position.found {
                tile_x = position.x;
                tile_y = position.y;
            }
            let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
            let picture_id = properties.query_property(summoned_creature_usage);
            let property = game.find_monster_property_by_picture_id(picture_id).cloned();
            if let Some(property) = property {
                let _ = game.add_summoned_creature_owned(
                    region, &property, master, tile_x, tile_y, -1, lifetime_ms,
                );
            }
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }

    let Some((destination_x, destination_y)) = target_coordinates(game, region, target) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    if !approach_attack_range(
        game,
        region,
        monster_id,
        destination_x,
        destination_y,
        properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
        now_ms,
    ) {
        return true;
    }
    if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms) {
        let attack_started = region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, attack_interval_ms));
        if !attack_started {
            return true;
        }
    }
    if last_used_ms != 0
        && !time_reached(
            now_ms,
            last_used_ms,
            properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
        )
    {
        return true;
    }
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let direction = get_line_direction(
        source.get_tile_x().unwrap_or_default(),
        source.get_tile_y().unwrap_or_default(),
        destination_x,
        destination_y,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_direction(direction);
        monster.move_shape_mut().set_moveable(false);
        monster.begin_base_attack_cast(target, skill_id, skill_level, now_ms);
        monster.set_summon_creature_progress(SummonCreatureProgress {
            destination_x,
            destination_y,
        });
    }
    let source = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.move_shape().shape())
        .unwrap_or(&source);
    send_start(game, region, source, monster_id, skill_id, skill_level);
    true
}
