//! Перенос здоровья игрока боевому духу (`CHuoxieshu`, навык `0x222`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/huoxieshu.cpp`. Навык оставляет игроку минимум одно HP,
//! списывает здоровье до задержки и только после неё восстанавливает
//! `GAP_BF_HP` с ограничением `GAP_BF_MAX_HP`. Полный old-client payload
//! `0xBF918` рассылается вокруг игрока после изменения боевого духа.
//! `SkillExecutionKernel` хранит стадии и исходные часы; `CGame` предоставляет
//! владельца игрока и фактическую доставку.

use super::baseattack::time_reached;
use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const HUOXIESHU_SKILL_ID: u32 = 0x222;
const VISUAL_OBJECT_TYPE: i32 = 700;
const SKILL_USAGE_USER_HP_LOSE: u32 = 1;
const SKILL_USAGE_TARGET_HP_GAIN: u32 = 5_001;

const fn health_cost_unavailable(current: u32, cost: u32) -> bool {
    (current.wrapping_sub(cost).wrapping_sub(1) as i32) < 0
}

fn send_failure(game: &CGame, player_id: i32, action: u8) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(4);
    message.add_byte(action);
    let _ = message.send_to_player(game.net_server(), player_id);
}

fn send_cast(game: &mut CGame, player_id: i32, skill_level: i32, action: u8) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(HUOXIESHU_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    match action {
        1 | 3 => {
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(player.shape().get_direction());
        }
        2 => {
            message.add_long(400);
            message.add_long(player_id);
            message.add_long(400);
            message.add_long(player_id);
            message.add_long(player.shape().get_tile_x().unwrap_or(0));
            message.add_long(player.shape().get_tile_y().unwrap_or(0));
        }
        _ => return,
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_goods_update(
    game: &mut CGame,
    update: &crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate,
) {
    let mut message = CMessage::new(update.message_type as i32);
    message.add_long(update.player_id);
    message.base_mut().add_guid(update.goods.ex_id);
    message.add_ulong(update.old_client_payload.len() as u32);
    message.base_mut().add(&update.old_client_payload);
    let _ = game.send_player_shape_around(update.player_id, None, &message);
}

pub(crate) fn execute_battle_fairy_huoxieshu<Runtime: GameMainLoopRuntime>(
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
    let skill_level = match dispatch {
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
        } if skill_id == HUOXIESHU_SKILL_ID => skill_level,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(properties) = game.skill_base_properties(HUOXIESHU_SKILL_ID, skill_level) else {
        send_cast(game, player_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let hp_loss = properties.query_property(SKILL_USAGE_USER_HP_LOSE);
    let hp_gain = properties.query_property(SKILL_USAGE_TARGET_HP_GAIN);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.huoxieshu().is_none() {
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.huoxieshu_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.huoxieshu_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if hp_loss != 0 && health_cost_unavailable(player.health(), hp_loss) {
            send_failure(game, player_id, 6);
            game.send_skill_system_info_with_unsigned(
                player_id,
                b"ZHGS0054",
                hp_loss.wrapping_add(1),
            );
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        player_ai.begin_huoxieshu(SkillExecutionKernel::begin(
            dispatch,
            started_at_ms,
        ));
    } else if player_ai
        .huoxieshu()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game
        .find_player(player_id)
        .and_then(|player| player.server_region_id())
        .is_none()
    {
        send_cast(game, player_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .huoxieshu()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let Some(player) = game.find_player(player_id) else {
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if player.equipment().get_goods(10).is_none() {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        let current_health = player.health();
        if health_cost_unavailable(current_health, hp_loss) {
            send_failure(game, player_id, 6);
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", hp_loss);
            send_cast(game, player_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_health(current_health.wrapping_sub(hp_loss));
        }
        send_cast(game, player_id, skill_level, 1);
        if let Some(state) = player_ai.huoxieshu_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .huoxieshu()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("исполнение переноса здоровья создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    let goods_factory = game.goods_factory().clone();
    let da_kong_key = game.globe_setup().da_kong_key();
    if game
        .find_player(player_id)
        .and_then(|player| player.war_soul_goods(&goods_factory))
        .is_none()
    {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_cast(game, player_id, skill_level, 2);
    let update = game.find_player_mut(player_id).and_then(|player| {
        player.restore_war_soul_health(hp_gain, &goods_factory, da_kong_key)
    });
    if let Some(update) = update.as_ref() {
        send_goods_update(game, update);
    } else {
        tracing::warn!(player_id, "не удалось сериализовать боевой дух после переноса здоровья");
    }
    if let Some(state) = player_ai.huoxieshu_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_huoxieshu_used(runtime.now_milliseconds());
    send_cast(game, player_id, skill_level, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
