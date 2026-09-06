//! Паучий туман `CSpiderMist` (`0x198`) для игрока, монстра и питомца.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spidermist.cpp`. Владелец сохраняет координаты цели в
//! момент `Begin`, reuse, прямой путь и `BLOCK_UNFLY`, запрет движения на
//! задержке и точные пакеты `0xBFE01`. После задержки он создаёт
//! `CSpiderMistPhalanx` с исходными lifetime/frequency/poison-параметрами;
//! регистрация сначала завершает перекрывающийся `PoisonFog` в той же клетке.
//! Monster/pet дополнительно сохраняют независимое attack-speed расписание.
//! `CGame` только разрешает владельцев, выполняет dispatch и доставку. Player
//! и monster ветви используют абсолютный срок `CSkill::IsRestored`; задержка
//! и lifetime области остаются elapsed.
//! AI (0x00540A64) снимает запрет движения перед выпуском области, затем
//! End (0x00540890) снимает ещё один. Счётчик не нормализуется. Второй вызов,
//! очистку регистрации навыка и Stiffen-End выполняет общий CMonster;
//! область тумана остаётся независимой от завершённого навыка.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::flash::master_info;
use super::skillbaseproperties::CSkillBaseProperties;
use super::spidermistphalanx::CSpiderMistPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::monsterattack::resolve_owned_monster_attack_target;
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const BLOCK_UNFLY: u8 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME: u32 = 30_001;
pub(crate) const SPIDER_MIST_SKILL_ID: u32 = 0x198;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSpiderMistExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination: (i32, i32),
}

impl PlayerSpiderMistExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderMistProgress {
    pub(crate) destination_x: i32,
    pub(crate) destination_y: i32,
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPIDER_MIST_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    destination_x: i32,
    destination_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPIDER_MIST_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(destination_x);
    message.add_long(destination_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_phalanx_entry(game: &CGame, region: &CServerRegion, phalanx_id: i32) {
    let Some(crate::gameserver::appserver::summonshape::SummonedSkillShape::SpiderMist(phalanx)) =
        region.find_skill_phalanx(phalanx_id)
    else {
        return;
    };
    let Some(payload) = phalanx.encode_client_snapshot() else { return };
    let identity = phalanx.shape().identity();
    let mut message = CMessage::new(0x000b_f502);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.base_mut().add_guid(identity.ex_id);
    message.add_long(payload.len() as i32);
    message.base_mut().add(&payload);
    message.base_mut().add_char(0);
    let _ = game.send_game_shape_around(region, phalanx.shape(), None, &message);
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_player_spider_mist_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: SPIDER_MIST_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: SPIDER_MIST_SKILL_ID, .. }
    )
}

fn player_destination(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game
            .base_magic_target_view(region_id, target)
            .map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::SelfTarget { .. } => None,
    }
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(0x000b_fe01, player_id, action);
}

fn send_player_visual(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    destination: (i32, i32),
) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(SPIDER_MIST_SKILL_ID as i32);
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
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.finish_curable_skill_state(SPIDER_MIST_SKILL_ID);
    }
}

fn finish_player_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(SPIDER_MIST_SKILL_ID, now_ms);
    });
}

fn abort_player_spider_mist(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn cancel_player_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    abort_player_spider_mist(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_spider_mist_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, skill_level, master)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(SPIDER_MIST_SKILL_ID),
                master_info(player),
            ))
        })
    else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(SPIDER_MIST_SKILL_ID, skill_level).cloned() else {
        if player_ai.player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID).is_some() {
            abort_player_spider_mist(game, player_id);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
    let state_lifetime_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let hp_loss = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();

    if player_ai.player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID).is_none() {
        if !skill_is_restored(player_ai.skill_last_used_ms(SPIDER_MIST_SKILL_ID), reuse_delay_ms, now_ms) {
            send_player_failure(game, player_id, 0x0d);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(destination) = player_destination(game, region_id, dispatch) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            destination.0,
            destination.1,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_player_failure(game, player_id, 0x0b);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_player_failure(game, player_id, 0x0f);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SPIDER_MIST_SKILL_ID));
            // `CSpiderMist` является `CStateSkill`: native owner помещает
            // активный cast в общий ordered `m_vStates`, откуда его может
            // снять `CCure` до создания phalanx.
            player.register_curable_skill_state(SPIDER_MIST_SKILL_ID);
        }
        player_ai.begin_player_skill_execution(PlayerSpiderMistExecutionState::begin(
            dispatch,
            destination,
            now_ms,
        ));
        return player_terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai
        .player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID)
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    let destination = player_ai
        .player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID)
        .map(|state| state.destination)
        .expect("выполнение паучьего тумана хранит координаты призыва");
    if player_ai
        .player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                destination.0,
                destination.1,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_player_visual(game, player_id, skill_level, 1, destination);
        if let Some(state) = player_ai.player_skill_state_mut::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = player_ai
        .player_skill_state::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID)
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение паучьего тумана хранит время начала");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }

    restore_player_movement(game, player_id);
    send_player_visual(game, player_id, skill_level, 2, destination);
    let phalanx_id = game.allocate_summon_shape_id();
    let phalanx_started_at_ms = runtime.now_milliseconds();
    let phalanx = CSpiderMistPhalanx::new(
        phalanx_id,
        master,
        phalanx_started_at_ms,
        lifetime_ms,
        skill_level,
        state_lifetime_ms,
        frequency_ms,
        hp_loss,
    );
    let (area_width, area_height) = game.area_dimensions();
    let summoned = if let Some(mut owner) = game.take_region_owner(region_id) {
        let result = owner.base_mut().add_spider_mist_phalanx(
            phalanx,
            destination.0,
            destination.1,
            area_width,
            area_height,
            phalanx_started_at_ms,
            runtime,
        );
        if result.is_ok() {
            send_phalanx_entry(game, owner.base(), phalanx_id);
        }
        game.restore_region_owner(owner);
        result.is_ok()
    } else {
        false
    };
    if let Some(state) = player_ai.player_skill_state_mut::<PlayerSpiderMistExecutionState>(SPIDER_MIST_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_spider_mist(game, player_id, player_ai, runtime);
    player_terminal(if summoned {
        QueuedSkillExecutionState::Completed
    } else {
        QueuedSkillExecutionState::Rejected
    })
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source_shape, cast, progress, last_used_ms, ai_type, attack_interval)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let attack_interval = if monster.is_tamed() {
                monster.pet_attack_properties(property).attack_interval
            } else {
                property.attack_speed
            };
            Some((
                monster.move_shape().shape().clone(),
                monster.base_attack_cast(),
                monster.spider_mist_progress(),
                monster.skill_last_used_ms(SPIDER_MIST_SKILL_ID),
                property.ai,
                attack_interval,
            ))
        })
    else {
        return false;
    };
    if let Some(cast) = cast {
        if cast.dispatch().skill_id != SPIDER_MIST_SKILL_ID {
            return false;
        }
        let Some(progress) = progress else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.cancel_base_attack_cast();
            }
            return true;
        };
        if !time_reached(
            now_ms,
            cast.started_at_ms(),
            properties.query_property(SKILL_USAGE_DELAY_TIME),
        ) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        send_fire(
            game,
            region,
            &source_shape,
            monster_id,
            skill_level,
            progress.destination_x,
            progress.destination_y,
        );
        let phalanx_started_at_ms = runtime.now_milliseconds();
        let phalanx = CSpiderMistPhalanx::new(
            game.allocate_summon_shape_id(),
            MasterInfo {
                master_type: MONSTER_TYPE,
                master_id: monster_id,
                ..MasterInfo::default()
            },
            phalanx_started_at_ms,
            properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME),
            i32::from(skill_level),
            properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
            properties.query_property(SKILL_USAGE_CONST),
        );
        let (area_width, area_height) = game.area_dimensions();
        if let Ok(phalanx_id) = region.add_spider_mist_phalanx(
            phalanx,
            progress.destination_x,
            progress.destination_y,
            area_width,
            area_height,
            phalanx_started_at_ms,
            runtime,
        ) {
            send_phalanx_entry(game, region, phalanx_id);
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast_with_clock(|| runtime.now_milliseconds());
        }
        return true;
    }

    let Some(target_owner) = resolve_owned_monster_attack_target(game, region, target) else {
        return true;
    };
    let target_shape = &target_owner.shape;
    let (Ok(source_x), Ok(source_y), Ok(destination_x), Ok(destination_y)) = (
        source_shape.get_tile_x(),
        source_shape.get_tile_y(),
        target_shape.get_tile_x(),
        target_shape.get_tile_y(),
    ) else { return true };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if !approach_attack_range(
        game,
        region,
        monster_id,
        MonsterTraceTarget::Shape(target_owner.view),
        maximum_distance,
        now_ms,
    ) {
        return true;
    }
    let path = region.straight_skill_path(source_x, source_y, destination_x, destination_y, None);
    if (maximum_distance != 0 && path.len() > maximum_distance as usize)
        || path.iter().any(|cell| cell.2 == BLOCK_UNFLY)
    {
        return true;
    }
    let schedule_ready = schedule_attack_interval(ai_type, attack_interval).is_none_or(|interval| {
        region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval))
    });
    if !schedule_ready {
        return true;
    }
    let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
            last_used_ms,
            reuse_delay,
            now_ms,
        )
    {
        return true;
    }
    let direction = get_line_direction(source_x, source_y, destination_x, destination_y);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_direction(direction);
        monster.move_shape_mut().set_moveable(false);
        monster.begin_base_attack_cast(target, SPIDER_MIST_SKILL_ID, skill_level, now_ms);
        monster.set_spider_mist_progress(SpiderMistProgress { destination_x, destination_y });
    }
    let source = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.move_shape().shape())
        .unwrap_or(&source_shape);
    send_start(game, region, source, monster_id, skill_level);
    true
}
