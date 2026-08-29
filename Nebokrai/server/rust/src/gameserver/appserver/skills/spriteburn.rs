//! Огненная область `CSpriteBurn` (`0x1a6`) для игрока и монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spriteburn.cpp`. Владелец сохраняет повторную проверку и
//! расход MP игрока, восстановление навыка, задержку и точные пакеты начала и
//! срабатывания. После задержки навык читает текущую клетку владельца, обходит
//! подтверждённую маску 7×7 в порядке X → Y и живой порядок объектов каждой
//! клетки, исключает мёртвые и недоступные цели с `CureState`, затем атомарно
//! заменяет их канонический `SpiderPoisonState` (`0x191`). Состояние создаётся
//! с отдельным чтением часов для каждой цели. `CGame` только разрешает
//! независимых владельцев региона и цели и выполняет фактическую доставку.
//! Player `End` возвращает движение и выполняет `CSummonSkill::End(1)`;
//! установленное состояние цели живёт независимо от завершённого cast.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::{cell_views, master_info};
use super::monsterattack::{
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::monsterrangeattack::range_attack_scope_cells;
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoison::{install_spider_poison_state, target_has_cure};
use super::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::ai::monsterai::{
    approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{
    SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_CONST: u32 = 20_010;

pub(crate) const SPRITE_BURN_SKILL_ID: u32 = 0x1a6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpriteBurnExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}

impl SpriteBurnExecutionState {
    const fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms) }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_sprite_burn_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == SPRITE_BURN_SKILL_ID,
    }
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
}

fn send_player_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(SPRITE_BURN_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_fire(game: &mut CGame, player_id: i32, level: i32, tile_x: i32, tile_y: i32) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(SPRITE_BURN_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_sprite_burn_used(now_ms);
    });
}

pub(crate) fn cancel_player_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .sprite_burn()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish_player_sprite_burn(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn apply_player_scope<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    center_x: i32,
    center_y: i32,
    lifetime_ms: u32,
    frequency_ms: u32,
    constant: u32,
    runtime: &mut Runtime,
) {
    for (offset_x, offset_y) in range_attack_scope_cells() {
        let tile_x = center_x.wrapping_add(offset_x);
        let tile_y = center_y.wrapping_add(offset_y);
        for view in cell_views(game, region_id, tile_x, tile_y) {
            let target = view.identity;
            if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) {
                continue;
            }
            let Some(master) = game.find_player(player_id).map(master_info) else { return };
            if !game.owned_player_skill_target_attackable(master, target, region_id) {
                continue;
            }
            let Some(mut owner) = game.take_region_owner(region_id) else { return };
            if !target_has_cure(game, owner.base(), target) {
                let now_ms = runtime.now_milliseconds();
                install_spider_poison_state(
                    game,
                    owner.base_mut(),
                    target,
                    SpiderPoisonState::new(master, now_ms, lifetime_ms, frequency_ms, constant),
                    now_ms,
                );
            }
            game.restore_region_owner(owner);
        }
    }
}

pub(crate) fn execute_player_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_sprite_burn_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, initial_mana)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(SPRITE_BURN_SKILL_ID),
            player.mana(),
        ))
    }) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(SPRITE_BURN_SKILL_ID, level) else {
        if player_ai.sprite_burn().is_some() {
            finish_player_sprite_burn(game, player_id, player_ai, runtime);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let constant = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.sprite_burn().is_none() {
        let now_ms = runtime.now_milliseconds();
        if player_ai.sprite_burn_last_used_ms() != 0
            && !time_reached(now_ms, player_ai.sprite_burn_last_used_ms(), reuse_delay_ms)
        {
            send_player_failure(game, player_id, 0x0d);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SPRITE_BURN_SKILL_ID));
        }
        player_ai.begin_sprite_burn(SpriteBurnExecutionState::begin(dispatch, now_ms));
    } else if player_ai.sprite_burn().is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.sprite_burn().is_some_and(|state| state.kernel().stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7);
            finish_player_sprite_burn(game, player_id, player_ai, runtime);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_player_start(game, player_id, level);
        if let Some(state) = player_ai.sprite_burn_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai.sprite_burn()
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение огненной области хранит время начала");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    let Some((center_x, center_y)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
    }) else {
        finish_player_sprite_burn(game, player_id, player_ai, runtime);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    send_player_fire(game, player_id, level, center_x, center_y);
    if let Some(state) = player_ai.sprite_burn_mut() {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
    }
    apply_player_scope(
        game, player_id, region_id, center_x, center_y, lifetime_ms, frequency_ms, constant, runtime,
    );
    if let Some(state) = player_ai.sprite_burn_mut() {
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_sprite_burn(game, player_id, player_ai, runtime);
    player_terminal(QueuedSkillExecutionState::Completed)
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPRITE_BURN_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
    tile_x: i32,
    tile_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPRITE_BURN_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт")]
pub(crate) fn execute_owned_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source, property, master, tamed, attack_interval_ms, cast, last_used_ms)) = region
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
                monster.master_info(),
                monster.is_tamed(),
                attack_interval_ms,
                monster.base_attack_cast(),
                monster.skill_last_used_ms(SPRITE_BURN_SKILL_ID),
            ))
        })
    else {
        return false;
    };

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity)
        else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        };
        let (Ok(target_x), Ok(target_y)) =
            (target.shape.get_tile_x(), target.shape.get_tile_y())
        else {
            return true;
        };
        if !approach_attack_range(
            game,
            region,
            monster_id,
            target_x,
            target_y,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            now_ms,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
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
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_base_attack_cast(
                target_identity,
                SPRITE_BURN_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение огненной области проверено выше");
    if cast.dispatch().skill_id != SPRITE_BURN_SKILL_ID {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }
    let (Ok(center_x), Ok(center_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    send_fire(game, region, &source, skill_level, center_x, center_y);

    let state_master = MasterInfo {
        master_type: MONSTER_TYPE,
        master_id: monster_id,
        ..MasterInfo::default()
    };
    for (offset_x, offset_y) in range_attack_scope_cells() {
        let candidates = monster_attack_cell_candidates(
            game,
            region,
            monster_id,
            center_x.wrapping_add(offset_x),
            center_y.wrapping_add(offset_y),
        );
        for identity in candidates {
            let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
                continue;
            };
            if target.dead
                || target.god
                || target.city_dead
                || !owned_monster_attackable(
                    game,
                    region.id,
                    &property,
                    tamed,
                    master,
                    identity,
                    &target,
                )
                || target_has_cure(game, region, identity)
            {
                continue;
            }
            let state_now_ms = runtime.now_milliseconds();
            install_spider_poison_state(
                game,
                region,
                identity,
                SpiderPoisonState::new(
                    state_master,
                    state_now_ms,
                    properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
                    properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
                    properties.query_property(SKILL_USAGE_CONST),
                ),
                state_now_ms,
            );
        }
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
