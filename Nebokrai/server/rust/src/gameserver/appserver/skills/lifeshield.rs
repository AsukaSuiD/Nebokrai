//! Достигнутый контракт навыка боевого духа `CLifeShield`.
//!
//! Навык `544` расходует `GAP_BF_MP` экипированного боевого духа, немедленно
//! рассылает изменённый товар, затем создаёт упорядоченное защитное состояние.
//! Любое завершение состояния добавляет краткоживущий `CCureState`.

pub(crate) const LIFE_SHIELD_SKILL_ID: u32 = 544;
pub(crate) const LIFE_SHIELD_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const LIFE_SHIELD_VISUAL_OBJECT_TYPE: i32 = 700;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_STATE_HP: u32 = 10_010;
pub(crate) const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
pub(crate) const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::lifeshieldstate::{
    finish_life_shield_state, send_life_shield_state_visual, LifeShieldState,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

fn send_cast(game: &mut CGame, player_id: i32, skill_level: i32, action: u8) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(LIFE_SHIELD_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(LIFE_SHIELD_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(LIFE_SHIELD_VISUAL_OBJECT_TYPE);
    message.add_long(player_id);
    if action == 2 {
        message.add_long(0);
        message.add_long(0);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_goods_update(game: &mut CGame, update: &BattleFairyDefaultGoodsUpdate) {
    let mut message = CMessage::new(update.message_type as i32);
    message.add_long(update.player_id);
    message.base_mut().add_guid(update.goods.ex_id);
    message.add_ulong(update.old_client_payload.len() as u32);
    message.base_mut().add(&update.old_client_payload);
    let _ = game.send_player_shape_around(update.player_id, None, &message);
}

pub(crate) fn execute_battle_fairy_life_shield<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let (skill_id, skill_level) = match dispatch {
        BattleFairySkillDispatch::SelfTarget {
            skill_id,
            skill_level,
            ..
        }
        | BattleFairySkillDispatch::Point {
            skill_id,
            skill_level,
            ..
        }
        | BattleFairySkillDispatch::Object {
            skill_id,
            skill_level,
            ..
        } if skill_id == LIFE_SHIELD_SKILL_ID => (skill_id, skill_level),
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    if game.find_player(player_id).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        game.send_battle_fairy_skill_failure(player_id, 2);
        send_cast(game, player_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state_life = properties.query_property(SKILL_USAGE_STATE_HP) as i32;
    let hp_factor = properties.query_property(SKILL_USAGE_TARGET_HP_DECREASE_FACTOR) as u16;
    let mp_factor = properties.query_property(SKILL_USAGE_TARGET_MP_DECREASE_FACTOR) as u16;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.life_shield().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.life_shield_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.life_shield_last_used_ms(),
                reuse_delay_ms,
            )
        {
            game.send_battle_fairy_skill_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            game.send_battle_fairy_skill_failure(player_id, 2);
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 {
            let Some(current) = game
                .find_player(player_id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else {
                game.send_battle_fairy_skill_failure(player_id, 2);
                send_cast(game, player_id, skill_level, 3);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            if i64::from(current) - i64::from(mp_loss) < 0 {
                game.send_battle_fairy_skill_failure(player_id, 7);
                let text_cost = (f64::from(mp_loss) * 0.0001).round() as i32 as u32;
                game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", text_cost);
                game.send_battle_fairy_skill_failure(player_id, 2);
                send_cast(game, player_id, skill_level, 3);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        player_ai.begin_life_shield(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .life_shield()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .life_shield()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        if game
            .find_player(player_id)
            .and_then(CPlayer::server_region_id)
            .is_none()
        {
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(current) = game
            .find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else {
            return terminal(QueuedSkillExecutionState::Pending);
        };
        if i64::from(current) - i64::from(mp_loss) < 0 {
            game.send_battle_fairy_skill_failure(player_id, 7);
            let text_cost = (f64::from(mp_loss) * 0.0001).round() as i32 as u32;
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", text_cost);
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let goods_factory = game.goods_factory().clone();
        let da_kong_key = game.globe_setup().da_kong_key();
        let update = game.find_player_mut(player_id).and_then(|player| {
            player.spend_war_soul_mana(mp_loss, &goods_factory, da_kong_key)
        });
        if let Some(update) = update.as_ref() {
            send_goods_update(game, update);
        }
        send_cast(game, player_id, skill_level, 1);
        if let Some(state) = player_ai.life_shield_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .life_shield()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение щита жизни создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_cast(game, player_id, skill_level, 2);
    let state = LifeShieldState::new(
        runtime.now_milliseconds(),
        keep_time_ms,
        state_life,
        hp_factor,
        mp_factor,
        skill_level,
    );
    let removed = game
        .find_player_mut(player_id)
        .and_then(|player| player.replace_life_shield_state(state));
    if let Some(removed) = removed {
        finish_life_shield_state(game, player_id, removed, runtime.now_milliseconds());
    }
    send_life_shield_state_visual(game, player_id, state, true, || runtime.now_milliseconds());
    if let Some(state) = player_ai.life_shield_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_life_shield_used(runtime.now_milliseconds());
    send_cast(game, player_id, skill_level, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
