//! Владелец трупного яда `CCorpsePtomaine` (`0x19F`).
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

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED};
use super::flash::{cell_views, master_info};
use super::monsterattack::{
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoison::{install_spider_poison_state, target_has_cure};
use super::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::CShape;
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::{
    finish_summon_skill_without_weapon_wear,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
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
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: crate::gameserver::appserver::shape::ShapeIdentity,
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
                monster.skill_last_used_ms(CORPSE_PTOMAINE_SKILL_ID),
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
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
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
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_ms,
            )
        {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_base_attack_cast(
                target_identity,
                CORPSE_PTOMAINE_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_start(game, region, &source, skill_level);
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
    send_fire(game, region, &source, skill_level, center_x, center_y);
    let state_master = MasterInfo {
        master_type: MONSTER_TYPE,
        master_id: monster_id,
        ..MasterInfo::default()
    };
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let candidates = monster_attack_cell_candidates(
                game,
                region,
                monster_id,
                center_x.wrapping_add(offset_x),
                center_y.wrapping_add(offset_y),
            );
            for identity in candidates {
                let Some(target) = resolve_owned_monster_attack_target(game, region, identity)
                else {
                    continue;
                };
                if target.dead
                    || target.god
                    || target.city_dead
                    || target_has_cure(game, region, identity)
                    || !owned_monster_attackable(
                        game,
                        region.id,
                        &property,
                        tamed,
                        master,
                        identity,
                        &target,
                    )
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
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) const fn is_player_corpse_ptomaine_dispatch(dispatch: PlayerSkillDispatch) -> bool { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == CORPSE_PTOMAINE_SKILL_ID } }
fn send_player_failure(game: &CGame, player_id: i32, action: u8) { game.send_self_state_skill_failure(0x000b_fe01, player_id, action); }
fn send_player_start(game: &mut CGame, player_id: i32, level: i32) { let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return }; let mut message = CMessage::new(0x000b_fe01); message.add_byte(1); message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(direction); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_player_fire(game: &mut CGame, player_id: i32, level: i32, center: (i32, i32)) { let mut message = CMessage::new(0x000b_fe01); message.add_byte(2); message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(0); message.add_long(0); message.add_long(center.0); message.add_long(center.1); let _ = game.send_player_shape_around(player_id, None, &message); }
fn restore_player(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) { restore_player(game, player_id); finish_summon_skill_without_weapon_wear(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(CORPSE_PTOMAINE_SKILL_ID, now_ms)); }
pub(crate) fn cancel_player_corpse_ptomaine<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = ai.player_skill_execution(CORPSE_PTOMAINE_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; finish_player(game, player_id, ai, runtime); ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }

pub(crate) fn execute_player_corpse_ptomaine<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_player_corpse_ptomaine_dispatch(dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, center_x, center_y, level, mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(CORPSE_PTOMAINE_SKILL_ID), player.mana()))) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CORPSE_PTOMAINE_SKILL_ID, level).cloned() else { if ai.player_skill_execution(CORPSE_PTOMAINE_SKILL_ID).is_some() { restore_player(game, player_id); } return player_terminal(QueuedSkillExecutionState::Rejected) };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED); let now = runtime.now_milliseconds();
    if ai.player_skill_execution(CORPSE_PTOMAINE_SKILL_ID).is_none() { if !skill_is_restored(ai.skill_last_used_ms(CORPSE_PTOMAINE_SKILL_ID), reuse, now) { send_player_failure(game, player_id, 0x0d); return player_terminal(QueuedSkillExecutionState::Rejected) } if mp_loss == 0 { return player_terminal(QueuedSkillExecutionState::Rejected) } if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_player_failure(game, player_id, 7); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(CORPSE_PTOMAINE_SKILL_ID)); } ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, now)); }
    if ai.player_skill_execution(CORPSE_PTOMAINE_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    if ai.player_skill_execution(CORPSE_PTOMAINE_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) { let current = game.find_player(player_id).map_or(0, CPlayer::mana); if (current.wrapping_sub(mp_loss) as i32) < 0 { send_player_failure(game, player_id, 7); restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_player_start(game, player_id, level); if let Some(state) = ai.player_skill_execution_mut(CORPSE_PTOMAINE_SKILL_ID) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.player_skill_execution(CORPSE_PTOMAINE_SKILL_ID).map(SkillExecutionKernel::started_at_ms).unwrap_or_default(); if !time_reached(now, started, delay) { return player_terminal(QueuedSkillExecutionState::Pending) }
    send_player_fire(game, player_id, level, (center_x, center_y));
    let Some(master) = game.find_player(player_id).map(master_info) else { restore_player(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) };
    let mut targets = Vec::new(); for offset_x in -1..=1 { for offset_y in -1..=1 { for view in cell_views(game, region_id, center_x.wrapping_add(offset_x), center_y.wrapping_add(offset_y)) { let identity = view.identity; if matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE) && game.owned_player_skill_target_attackable(master, identity, region_id) && !game.periodic_state_target_dead(region_id, identity) { targets.push(identity); } } } }
    if let Some(mut region) = game.take_region_owner(region_id) { for identity in targets { if target_has_cure(game, region.base(), identity) { continue } let state_now = runtime.now_milliseconds(); install_spider_poison_state(game, region.base_mut(), identity, SpiderPoisonState::new(master, state_now, properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME), properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY), properties.query_property(SKILL_USAGE_CONST)), state_now); } game.restore_region_owner(region); }
    if let Some(state) = ai.player_skill_execution_mut(CORPSE_PTOMAINE_SKILL_ID) { let _ = state.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.advance(SkillStage::Attack, SkillStage::Apply); } finish_player(game, player_id, ai, runtime); player_terminal(QueuedSkillExecutionState::Completed)
}
