//! Область навыка SpriteBurn: допуск, визуальный эффект и наложение яда.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/spriteburn.cpp.
//! Визуальный ресурс принадлежит зарегистрированному навыку; наложенные
//! состояния хранятся независимо в общей арене цели. Синхронные callbacks
//! работают с опубликованными владельцами региона и ИИ, без их копирования.

use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_owned_skill_begin_object, resolve_state_move_shape,
};
use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::{cell_views, master_info};
use super::monsterattack::resolve_owned_monster_attack_target;
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoison::SPIDER_POISON_SKILL_ID;
use super::spiderpoisonstate::{SpiderPoisonState, begin_primary_spider_poison_state};
use crate::gameserver::appserver::ai::monsterai::{
    MonsterSkillCallOutcome, MonsterTraceTarget, approach_attack_range,
    finish_monster_skill_call, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillfactory::SkillOwner;
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::skills::kernel::{
    skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState, ServerRegionOwner,
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
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) const fn is_sprite_burn_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == SPRITE_BURN_SKILL_ID,
    }
}

pub(crate) fn publish_sprite_burn_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CSpriteBurn
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::SpriteBurn || effect.is_ended()
        })
    { return; }
    let (region_id, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region_id, identity) else { return; };
    let source = source.shape();
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
        let (Ok(x), Ok(y)) = (source.get_tile_x(), source.get_tile_y()) else { return; };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

pub(crate) fn cancel_player_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game
        .player_skill_state::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID)
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn add_sprite_burn_poison(
    game: &mut CGame,
    region_id: i32,
    address: RegisteredSkill,
    target: ShapeIdentity,
    now: &mut dyn FnMut() -> u32,
) {
    let Some(skill) = game.registered_skill(address) else { return; };
    let (user_region, source) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, user_region, source)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()))
    else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, region_id, target)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()))
    else { return; };
    let mut master = if source.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.id) else { return; };
        master_info(player)
    } else {
        MasterInfo { master_type: source.object_type, master_id: source.id, ..MasterInfo::default() }
    };
    master.master_country_id = 0;
    let hp_loss = properties.query_property(SKILL_USAGE_CONST);
    let frequency = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    // SpriteBurn накладывает паучий яд 0x191, а не одноимённое состояние
    // 0x1a6. Новый payload создаётся до callbacks завершения старого яда.
    let state = SpiderPoisonState::new(master, keep, frequency, hp_loss);
    let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return; };
    let selected = shape.find_state_position(|state| state.state_id() == SPIDER_POISON_SKILL_ID);
    let placement = if let Some((position, key)) = selected {
        let Some(location) = shape.applied_state_replacement_location(key) else { return; };
        let _ = end_and_destroy_state_at(game, region_id, target, position);
        Some(location)
    } else { None };
    let _ = begin_primary_spider_poison_state(
        game, region_id, target, Some(user), Some(sufferer), state, placement, now,
    );
}


fn apply_scope(
    game: &mut CGame,
    region_id: i32,
    source: ShapeIdentity,
    address: RegisteredSkill,
    center: (i32, i32),
    now: &mut dyn FnMut() -> u32,
) {
    // Источник не исключается из квадрата: решение оставлено правам цели,
    // которые, в частности, допускают некоторые случаи осиротевших питомцев.
    for offset_x in -1_i32..=1 {
        for offset_y in -1_i32..=1 {
            let tile_x = center.0.wrapping_add(offset_x);
            let tile_y = center.1.wrapping_add(offset_y);
            for view in cell_views(game, region_id, tile_x, tile_y) {
                let target = view.identity;
                if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || game.periodic_state_target_dead(region_id, target)
                    || !game.live_skill_target_attackable(region_id, source, target)
                {
                    continue;
                }
                let Some(shape) = resolve_state_move_shape(game, region_id, target) else { continue; };
                if shape.has_state_by_skill_id(0x131) { continue; }
                add_sprite_burn_poison(game, region_id, address, target, now);
            }
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
    let Some(level) = game.find_player(player_id)
        .map(|player| player.learned_skill_level(SPRITE_BURN_SKILL_ID, game.skill_factory()))
    else { return player_terminal(QueuedSkillExecutionState::Rejected); };
    let beginning = game.player_skill_state::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID).is_none();
    if beginning {
        game.replace_player_skill_visual_effect(
            player_id, SPRITE_BURN_SKILL_ID,
            SkillVisualEffect::new(SkillVisualEffectKind::SpriteBurn, 1),
        );
    }
    let Some(properties) = game.skill_base_properties(SPRITE_BURN_SKILL_ID, level) else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    if beginning {
        let started_at_ms = game.player_skill_lifecycle(player_id, SPRITE_BURN_SKILL_ID)
            .expect("общий Begin расписания сохранил базу SpriteBurn").started_at_ms();
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, SPRITE_BURN_SKILL_ID), reuse_delay_ms,
            cooldown_now_ms,
        ) {
            game.update_player_skill_visual(player_id, SPRITE_BURN_SKILL_ID, 0x0d);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.update_player_skill_visual(player_id, SPRITE_BURN_SKILL_ID, 7);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
        }
        game.begin_player_skill_execution(
            player_id, SpriteBurnExecutionState::begin(dispatch, started_at_ms),
        );
        return player_terminal(QueuedSkillExecutionState::Begun);
    }
    if game.player_skill_state::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID)
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_state::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.update_player_skill_visual(player_id, SPRITE_BURN_SKILL_ID, 7);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let Some(can_be_breaked) = game.skill_base_properties(SPRITE_BURN_SKILL_ID, level)
            .map(|properties| properties.query_property(SKILL_USAGE_CAN_BE_BREAKED))
        else { return player_terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(state) = game.player_skill_state_mut::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID) {
            state.kernel_mut().lifecycle_mut().set_available(can_be_breaked != 0);
        }
        game.update_player_skill_visual(player_id, SPRITE_BURN_SKILL_ID, 0);
        if let Some(state) = game.player_skill_state_mut::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(delay_ms) = game.skill_base_properties(SPRITE_BURN_SKILL_ID, level)
        .map(|properties| properties.query_property(SKILL_USAGE_DELAY_TIME))
    else { return player_terminal(QueuedSkillExecutionState::Rejected); };
    let Some(started_at_ms) = game.player_skill_state::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID)
        .map(|state| state.kernel().started_at_ms())
    else { return player_terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_player_skill_visual(player_id, SPRITE_BURN_SKILL_ID, 1);
    if let Some(state) = game.player_skill_state_mut::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let _ = game.with_published_player_ai(player_id, player_ai, |game| {
        let Some(address) = game.registered_player_skill(player_id, SPRITE_BURN_SKILL_ID) else { return; };
        let Some((region_id, source, center)) = game.find_player(player_id).and_then(|player| {
            Some((
                player.server_region_id()?, player.shape().identity(),
                (player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?),
            ))
        }) else { return; };
        apply_scope(game, region_id, source, address, center, &mut || runtime.now_milliseconds());
    });
    if let Some(state) = game.player_skill_state_mut::<SpriteBurnExecutionState>(player_id, SPRITE_BURN_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_terminal(QueuedSkillExecutionState::Completed)
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт")]
pub(crate) fn execute_owned_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_mut() else { return false; };
    let region_id = region_owner.base().id;
    let Some((source, property, attack_interval_ms, cast, last_used_ms)) = region_owner.base_mut()
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone();
            let attack_interval_ms = monster.is_tamed()
                .then(|| monster.pet_attack_properties(&property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().identity(), property, attack_interval_ms,
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_last_used_ms(SPRITE_BURN_SKILL_ID, game.skill_factory()),
            ))
        })
    else { return false; };
    if cast.is_some_and(|cast| cast.dispatch().skill_id != SPRITE_BURN_SKILL_ID) {
        return false;
    }
    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity) else {
            if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target(game.skill_factory());
            }
            return true;
        };
        if !approach_attack_range(
            game, region_owner.base_mut(), monster_id, MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE), runtime,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
            && !region_owner.base_mut().find_monster_by_id_mut(monster_id).is_some_and(|monster| {
                monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
            })
        {
            return true;
        }
        let target_object = resolve_owned_skill_begin_object(game, region_owner.base_mut(), target_identity);
        let started_at_ms = runtime.now_milliseconds();
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            if !monster.prepare_base_attack_cast(
                target_identity, SPRITE_BURN_SKILL_ID, skill_level, started_at_ms,
                target_object, game.skill_factory(),
            ) { return true; }
            monster.move_shape_mut().replace_skill_visual_effect(
                SPRITE_BURN_SKILL_ID, game.skill_factory(),
                SkillVisualEffect::new(SkillVisualEffectKind::SpriteBurn, 1),
            );
        }
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(last_used_ms, reuse, cooldown_now_ms) {
            let _ = game.with_published_region(owner, |game| {
                if let Some(address) = game.registered_move_shape_skill(region_id, source, SPRITE_BURN_SKILL_ID) {
                    game.update_registered_skill_visual(address, 0x0d);
                    let _ = game.end_registered_instance(
                        address, 0, SkillTermination::Rejected, runtime,
                    );
                }
            });
            let Some(region_owner) = owner.as_mut() else { return true; };
            return finish_monster_skill_call(
                game, region_owner.base_mut(), monster_id, MonsterSkillCallOutcome::BeginRejected, runtime,
            );
        }
        let queued_at_ms = runtime.now_milliseconds();
        let Some(region_owner) = owner.as_mut() else { return true; };
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.enqueue_base_attack_cast(queued_at_ms);
        }
        let can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(region_owner) = owner.as_mut() else { return true; };
        if let Some(lifecycle) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
            .and_then(|monster| monster.move_shape_mut().skill_lifecycle_mut(SPRITE_BURN_SKILL_ID, game.skill_factory()))
        {
            lifecycle.set_available(can_be_breaked != 0);
        }
        let _ = game.with_published_region(owner, |game| {
            if let Some(address) = game.registered_move_shape_skill(region_id, source, SPRITE_BURN_SKILL_ID) {
                game.update_registered_skill_visual(address, 0);
            }
        });
        if let Some(monster) = owner.as_mut().and_then(|owner| owner.base_mut().find_monster_by_id_mut(monster_id)) {
            let _ = monster.advance_base_attack_cast(
                SPRITE_BURN_SKILL_ID, SkillStage::Begin, SkillStage::Check, game.skill_factory(),
            );
        }
    }
    let Some(cast) = owner.as_ref().and_then(|owner| owner.base().find_monster_by_id(monster_id))
        .and_then(|monster| monster.base_attack_cast(SPRITE_BURN_SKILL_ID, game.skill_factory()))
    else { return true; };
    if cast.dispatch().skill_id != SPRITE_BURN_SKILL_ID { return false; }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if runtime.now_milliseconds() < cast.started_at_ms().wrapping_add(delay) { return true; }
    let _ = game.with_published_region(owner, |game| {
        let Some(address) = game.registered_move_shape_skill(region_id, source, SPRITE_BURN_SKILL_ID) else { return; };
        game.update_registered_skill_visual(address, 1);
        if let Some((source_region, center)) = resolve_state_move_shape(game, region_id, source)
            .and_then(|shape| {
                let shape = shape.shape();
                Some((shape.get_region_id(), (shape.get_tile_x().ok()?, shape.get_tile_y().ok()?)))
            })
        {
            apply_scope(
                game, source_region, source, address, center,
                &mut || runtime.now_milliseconds(),
            );
        }
        let _ = game.end_registered_instance(address, 1, SkillTermination::Completed, runtime);
    });
    true
}
