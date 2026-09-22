//! Владелец трупного яда `CCorpsePtomaine` (`0x19F`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/corpseptomaine.cpp`. Player и monster/pet пути сохраняют
//! cooldown, повторную проверку MP игрока, задержку и visual fire на клетке
//! caster-а. После задержки полный квадрат 3×3 обходится сначала по X, затем
//! по Y и в живом порядке клетки; допустимым живым целям без `CureState`
//! централизованно заменяется канонический `SpiderPoisonState`. `CGame`
//! временно передаёт region-owner, а формирование состояния и пакетов остаётся
//! в skill-owner-е. Нулевой MP-loss сохраняет исходный отказ player-cast.
//! Обе ветви проверяют восстановление абсолютным сроком `CSkill::IsRestored`,
//! а общую задержку — отдельной elapsed-проверкой.
//! AddState0x00539FF0 создаёт payload до замены: CONST → frequency → keep,
//! первый ID191 получает End и destructor остатка, затем Begin(source,target)
//! и запись в прежний слот либо append при отсутствии совпадения. MasterInfo
//! берётся у живого source на каждом наложении; country остаётся нулём.
//! Callback публикует настоящий derived region, без копии base/состояния.
//! Общий AI (0x0053A230): RTTI CMoveShape → IsDied → живой IsAttackAble
//! → Cure → AddState. Дополнительного god-фильтра и ограничения 400/600 нет.

use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_owned_skill_begin_object, resolve_state_move_shape,
};
use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED};
use super::flash::{cell_views, master_info};
use super::monsterattack::{
    monster_attack_cell_candidates,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoison::SPIDER_POISON_SKILL_ID;
use super::spiderpoisonstate::{SpiderPoisonState, begin_primary_spider_poison_state};
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::{
    finish_summon_skill,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner};
use nebokrai_shared::values::CGuid;
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;
pub(crate) const CORPSE_PTOMAINE_SKILL_ID: u32 = 0x19f;

fn add_corpse_poison_state(
    game: &mut CGame, region_id: i32, source: ShapeIdentity, target: ShapeIdentity,
    properties: &CSkillBaseProperties, now: &mut dyn FnMut() -> u32,
) {
    let Some(source_region) = resolve_state_move_shape(game, region_id, source)
        .map(|shape| shape.shape().get_region_id()) else { return; };
    let Some(target_region) = resolve_state_move_shape(game, region_id, target)
        .map(|shape| shape.shape().get_region_id()) else { return; };
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
    let state = SpiderPoisonState::new(master, keep, frequency, hp_loss);
    let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return; };
    let selected = shape.find_state_position(|state| state.state_id() == SPIDER_POISON_SKILL_ID);
    let placement = if let Some((position, key)) = selected {
        let Some(location) = shape.applied_state_replacement_location(key) else { return; };
        let _ = end_and_destroy_state_at(game, region_id, target, position);
        Some(location)
    } else { None };
    let _ = begin_primary_spider_poison_state(
        game, region_id, target, Some((source_region, source)), Some((target_region, target)),
        state, placement, now,
    );
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32);
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
    message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32);
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
pub(crate) fn execute_owned_corpse_ptomaine<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: crate::gameserver::appserver::shape::ShapeIdentity,
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
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_last_used_ms(CORPSE_PTOMAINE_SKILL_ID, game.skill_factory()),
            ))
        })
    else {
        return false;
    };

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity)
        else {
            if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target(game.skill_factory());
            }
            return true;
        };
        if !approach_attack_range(
            game,
            region_owner.base_mut(),
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region_owner.base_mut()
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
            if !attack_started {
                return true;
            }
        }
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_ms,
            )
        {
            return true;
        }
        let target_object = resolve_owned_skill_begin_object(game, region_owner.base_mut(), target_identity);
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.begin_base_attack_cast(
                target_identity,
                CORPSE_PTOMAINE_SKILL_ID,
                skill_level,
                now_ms,
                target_object,
                game.skill_factory(),
            );
        }
        send_start(game, region_owner.base_mut(), &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение трупного яда проверено выше");
    if cast.dispatch().skill_id != CORPSE_PTOMAINE_SKILL_ID {
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
    send_fire(game, region_owner.base_mut(), &source, skill_level, center_x, center_y);
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let Some(region_owner) = owner.as_mut() else { return true; };
            let candidates = monster_attack_cell_candidates(game, region_owner,
                monster_id,
                center_x.wrapping_add(offset_x),
                center_y.wrapping_add(offset_y),
            );
            for identity in candidates {
                let Some(region_owner) = owner.as_mut() else { return true; };
                let Some(target) = resolve_owned_monster_attack_target(game, region_owner, identity)
                else {
                    continue;
                };
                if target.dead
                    || !game.live_skill_target_attackable_in(region_owner, source.identity(), identity)
                {
                    continue;
                }
                let _ = game.with_published_region(owner, |game| {
                    let Some(target) = resolve_state_move_shape(game, region_id, identity) else { return; };
                    if target.has_state_by_skill_id(super::cure::CURE_SKILL_ID) { return; }
                    add_corpse_poison_state(
                        game, region_id, source.identity(), identity, properties,
                        &mut || runtime.now_milliseconds(),
                    );
                });
            }
        }
    }
    let Some(region_owner) = owner.as_mut() else { return true; };
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(CORPSE_PTOMAINE_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(CORPSE_PTOMAINE_SKILL_ID, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
        let _ = monster.advance_base_attack_cast(CORPSE_PTOMAINE_SKILL_ID, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        let _ = monster.finish_base_attack_cast_with_clock(CORPSE_PTOMAINE_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false } }
pub(crate) const fn is_player_corpse_ptomaine_dispatch(dispatch: PlayerSkillDispatch) -> bool { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == CORPSE_PTOMAINE_SKILL_ID } }
fn send_player_failure(game: &CGame, player_id: i32, action: u8) { game.send_self_state_skill_failure(0x000b_fe01, player_id, action); }
fn send_player_start(game: &mut CGame, player_id: i32, level: i32) { let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return }; let mut message = CMessage::new(0x000b_fe01); message.add_byte(1); message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(direction); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_player_fire(game: &mut CGame, player_id: i32, level: i32, center: (i32, i32)) { let mut message = CMessage::new(0x000b_fe01); message.add_byte(2); message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(0); message.add_long(0); message.add_long(center.0); message.add_long(center.1); let _ = game.send_player_shape_around(player_id, None, &message); }
fn restore_player(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _ai: &mut CPlayerAI, runtime: &mut Runtime) { restore_player(game, player_id); finish_summon_skill(game, player_id, CORPSE_PTOMAINE_SKILL_ID, runtime); }
pub(crate) fn cancel_player_corpse_ptomaine<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = game.player_skill_execution(player_id, CORPSE_PTOMAINE_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; finish_player(game, player_id, ai, runtime); game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled) }

pub(crate) fn execute_player_corpse_ptomaine<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_player_corpse_ptomaine_dispatch(dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, center_x, center_y, level, mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(CORPSE_PTOMAINE_SKILL_ID, game.skill_factory()), player.mana()))) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CORPSE_PTOMAINE_SKILL_ID, level).cloned() else { if game.player_skill_execution(player_id, CORPSE_PTOMAINE_SKILL_ID).is_some() { restore_player(game, player_id); } return player_terminal(QueuedSkillExecutionState::Rejected) };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED); let now = runtime.now_milliseconds();
    if game.player_skill_execution(player_id, CORPSE_PTOMAINE_SKILL_ID).is_none() { if !skill_is_restored(game.player_skill_last_used_ms(player_id, CORPSE_PTOMAINE_SKILL_ID), reuse, now) { send_player_failure(game, player_id, 0x0d); return player_terminal(QueuedSkillExecutionState::Rejected) } if mp_loss == 0 { return player_terminal(QueuedSkillExecutionState::Rejected) } if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_player_failure(game, player_id, 7); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(CORPSE_PTOMAINE_SKILL_ID)); } game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now)); return player_terminal(QueuedSkillExecutionState::Begun); }
    if game.player_skill_execution(player_id, CORPSE_PTOMAINE_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    if game.player_skill_execution(player_id, CORPSE_PTOMAINE_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) { let current = game.find_player(player_id).map_or(0, CPlayer::mana); if (current.wrapping_sub(mp_loss) as i32) < 0 { send_player_failure(game, player_id, 7); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_player_start(game, player_id, level); if let Some(state) = game.player_skill_execution_mut(player_id, CORPSE_PTOMAINE_SKILL_ID) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = game.player_skill_execution(player_id, CORPSE_PTOMAINE_SKILL_ID).map(SkillExecutionKernel::started_at_ms).unwrap_or_default(); if !time_reached(now, started, delay) { return player_terminal(QueuedSkillExecutionState::Pending) }
    send_player_fire(game, player_id, level, (center_x, center_y));
    let source = ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID };
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            for view in cell_views(game, region_id, center_x.wrapping_add(offset_x), center_y.wrapping_add(offset_y)) {
                let identity = view.identity;
                if !matches!(identity.object_type, PLAYER_TYPE | 500 | MONSTER_TYPE | 1100 | 1200)
                    || game.base_magic_target_dead(region_id, identity)
                    || !game.live_skill_target_attackable(region_id, source, identity)
                { continue; }
                let Some(target) = resolve_state_move_shape(game, region_id, identity) else { continue; };
                if target.has_state_by_skill_id(super::cure::CURE_SKILL_ID) { continue; }
                game.with_published_player_ai(player_id, ai, |game| {
                    add_corpse_poison_state(game, region_id, source, identity, &properties, &mut || runtime.now_milliseconds());
                });
            }
        }
    }
    if let Some(state) = game.player_skill_execution_mut(player_id, CORPSE_PTOMAINE_SKILL_ID) { let _ = state.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.advance(SkillStage::Attack, SkillStage::Apply); } finish_player(game, player_id, ai, runtime); player_terminal(QueuedSkillExecutionState::Completed)
}
