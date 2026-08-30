//! Ярость `CFury` (`0x1a3`) для игрока и монстра.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает задержку и
//! повторное использование, пакеты `0xBFE01`, накопление `CFuryState`, порядок
//! снятия конфликтующих состояний и последующее краткоживущее `CCureState`.
//! Модуль владеет проверками, расходом RP, стадиями, состояниями и визуальными
//! пакетами. `CGame` предоставляет владельцев, доставку и общий пересчёт
//! свойств. Особая ветвь игрока снимает `CRageBreakState` и завершается без
//! создания `CFuryState`; обычная ветвь сохраняет накопление состояний.
//! Игровая ветвь игрока завершается общим `CSummonSkill::End(1)`; монстровый
//! lifecycle остаётся отдельным и не использует этот хвост.
use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::cure::finish_curable_state;
use super::curestate::{CureState, send_cure_state_visual_at};
use super::furystate::{FuryState, send_fury_state_visual};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::monsterattack::resolve_owned_monster_attack_target;
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoisonstate::send_spider_poison_state_visual;
use super::spiderwebstate::send_spider_web_state_visual;
use super::sealstate::send_seal_state_visual;
use crate::gameserver::appserver::ai::monsterai::{
    approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::appserver::skills::ragebreakstate::send_rage_break_state_visual;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const SKILL_USAGE_USER_RP_LOSE: u32 = 3;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_ATK_GAIN: u32 = 105;
const CONFLICTING_STATES: [u32; 9] = [
    0x138, 0xd2, 0xc9, 0x67, 0x192, 0x191, 0x198, 0x199, 0x1a6,
];
pub(crate) const FURY_SKILL_ID: u32 = 0x1a3;

fn self_identity(monster_id: i32) -> ShapeIdentity {
    ShapeIdentity {
        object_type: MONSTER_TYPE,
        id: monster_id,
        ex_id: CGuid::GUID_INVALID,
    }
}

fn send_cast_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_cast_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let Ok(tile_x) = source.get_tile_x() else {
        return;
    };
    let Ok(tile_y) = source.get_tile_y() else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn remove_reached_conflict_states(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) {
    let order = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.move_shape().curable_state_ids())
        .unwrap_or_default();
    for state_id in order {
        if state_id == super::sealstate::SEAL_STATE_ID {
            let removed = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
                let state = monster.move_shape_mut().take_seal_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                Some((state, monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().unwrap_or_default(), monster.move_shape().shape().get_tile_y().unwrap_or_default()))
            });
            if let Some((state, identity, tile_x, tile_y)) = removed {
                send_seal_state_visual(game, region.id, identity, tile_x, tile_y, state, false, now_ms);
            }
        } else if state_id == super::spiderpoison::SPIDER_POISON_SKILL_ID {
            let removed = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
                let state = monster.move_shape_mut().take_spider_poison_state()?;
                Some((state, monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().unwrap_or_default(), monster.move_shape().shape().get_tile_y().unwrap_or_default()))
            });
            if let Some((state, identity, tile_x, tile_y)) = removed {
                send_spider_poison_state_visual(game, region.id, identity, tile_x, tile_y, state, false, now_ms);
            }
        } else if state_id == super::spiderweb::SPIDER_WEB_SKILL_ID {
            let removed = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
                let state = monster.move_shape_mut().take_spider_web_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                Some((state, monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().unwrap_or_default(), monster.move_shape().shape().get_tile_y().unwrap_or_default()))
            });
            if let Some((state, identity, tile_x, tile_y)) = removed {
                send_spider_web_state_visual(
                    game, region.id, identity, tile_x, tile_y, state, false, || now_ms,
                );
            }
        }
    }
}

pub(crate) fn execute_owned_fury(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
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
                monster.skill_last_used_ms(FURY_SKILL_ID),
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
                self_identity(monster_id),
                FURY_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_cast_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение ярости проверено выше");
    if cast.dispatch().skill_id != FURY_SKILL_ID {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }

    send_cast_fire(game, region, &source, skill_level);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
    }

    let identity = source.identity();
    let tile_x = source.get_tile_x().unwrap_or_default();
    let tile_y = source.get_tile_y().unwrap_or_default();
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let fury = FuryState::new(
        now_ms,
        keep_time_ms,
        properties.query_property(SKILL_USAGE_TARGET_ATK_GAIN) as i32,
    );
    send_fury_state_visual(
        game, region.id, identity, tile_x, tile_y, fury, true, now_ms,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().push_fury_state(fury);
    }

    remove_reached_conflict_states(game, region, monster_id, now_ms);

    let cure = CureState::new(identity, identity);
    let previous_cure = region
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| monster.move_shape_mut().replace_cure_state(cure));
    if let Some(previous) = previous_cure {
        send_cure_state_visual_at(
            game, region.id, identity, tile_x, tile_y, previous, false,
        );
    }
    send_cure_state_visual_at(game, region.id, identity, tile_x, tile_y, cure, true);

    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}

pub(crate) const fn is_fury_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == FURY_SKILL_ID,
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn finish_player_fury<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| { player_ai.mark_fury_used(now_ms); });
}

pub(crate) fn cancel_player_fury<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.fury().map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_fury(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8, rp_loss: u32) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if action == 8 {
        message.add_long(0);
    } else {
        message.add_byte(0);
    }
    message.add_byte(action);
    let _ = message.send_to_player(game.net_server(), player_id);
    match action {
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", rp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_player_cast(game: &mut CGame, player_id: i32, level: i32, fired: bool) {
    game.send_self_state_skill_cast(
        EFFECT_MESSAGE,
        player_id,
        FURY_SKILL_ID,
        level,
        if fired { 2 } else { 1 },
    );
}

pub(crate) fn execute_player_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_fury_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((level, initial_rp)) = game
        .find_player(player_id)
        .map(|player| (player.learned_skill_level(FURY_SKILL_ID), player.rp()))
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(FURY_SKILL_ID, level) else {
        if player_ai.fury().is_some() {
            finish_player_fury(game, player_id, player_ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let rp_loss = properties.query_property(SKILL_USAGE_USER_RP_LOSE);
    let reuse_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let attack_gain = properties.query_property(SKILL_USAGE_TARGET_ATK_GAIN) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.fury().is_none() {
        let now_ms = runtime.now_milliseconds();
        if player_ai.fury_last_used_ms() != 0
            && now_ms < player_ai.fury_last_used_ms().wrapping_add(reuse_ms)
        {
            send_player_failure(game, player_id, 0x0d, rp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if rp_loss != 0 && (u32::from(initial_rp).wrapping_sub(rp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 8, rp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(FURY_SKILL_ID));
        }
        player_ai.begin_fury(SkillExecutionKernel::begin(dispatch, now_ms));
    } else if player_ai
        .fury()
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.find_player(player_id).is_none_or(CPlayer::is_dead) {
        send_player_cast(game, player_id, level, false);
        finish_player_fury(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .fury()
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 8, rp_loss);
            finish_player_fury(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_rp(u32::from(current_rp).wrapping_sub(rp_loss) as u16);
        }
        let _ = game.publish_player_states(player_id);
        send_player_cast(game, player_id, level, false);
        if let Some(execution) = player_ai.fury_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .fury()
        .map(SkillExecutionKernel::started_at_ms)
        .unwrap_or_default();
    if started_at_ms.wrapping_add(delay_ms) > runtime.now_milliseconds() {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_player_cast(game, player_id, level, true);
    if let Some(execution) = player_ai.fury_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let now_ms = runtime.now_milliseconds();
    let Some((region_id, identity, tile_x, tile_y)) =
        game.find_player(player_id).and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().identity(),
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
            ))
        })
    else {
        finish_player_fury(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };

    if let Some(rage_break) = game
        .find_player_mut(player_id)
        .and_then(CPlayer::take_rage_break_state)
    {
        send_rage_break_state_visual(
            game, region_id, identity, tile_x, tile_y, rage_break, false, now_ms,
        );
        let _ = game.update_player_properties(player_id, runtime);
        if let Some(execution) = player_ai.fury_mut() {
            let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
        }
        finish_player_fury(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Completed);
    }

    let fury = FuryState::new(now_ms, keep_time_ms, attack_gain);
    if let Some(player) = game.find_player_mut(player_id) {
        player.push_fury_state(fury);
    }
    send_fury_state_visual(game, region_id, identity, tile_x, tile_y, fury, true, now_ms);

    let state_order = game
        .find_player(player_id)
        .map(CPlayer::curable_state_ids)
        .unwrap_or_default();
    for state_id in state_order {
        if CONFLICTING_STATES.contains(&state_id) {
            let _ = finish_curable_state(game, region_id, identity, state_id, now_ms);
        }
    }

    let cure = CureState::new(identity, identity);
    let previous_cure = game
        .find_player_mut(player_id)
        .and_then(|player| player.replace_cure_state(cure));
    if let Some(previous) = previous_cure {
        super::curestate::send_cure_state_visual(game, player_id, previous, false);
    }
    super::curestate::send_cure_state_visual(game, player_id, cure, true);
    let _ = game.update_player_properties(player_id, runtime);
    let _ = game.publish_player_states(player_id);

    if let Some(execution) = player_ai.fury_mut() {
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_fury(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
