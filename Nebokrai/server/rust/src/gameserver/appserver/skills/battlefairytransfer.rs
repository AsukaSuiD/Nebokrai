//! Общий runtime двух навыков передачи ресурсов боевому духу.
//!
//! `CHuoxieshu` и `CLingzhishu` имеют одинаковые стадии, visual packet и
//! повтор при временном отсутствии equipment-owner-а. Раздельными остаются
//! source/target property, signed-проверки и тексты ошибок. Один
//! `SkillExecutionKernel` хранит только команду, стадии и исходные часы.

use super::baseattack::time_reached;
use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::huoxieshu::HUOXIESHU_SKILL_ID;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::lingzhishu::LINGZHISHU_SKILL_ID;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const VISUAL_OBJECT_TYPE: i32 = 700;
const SKILL_USAGE_USER_HP_LOSE: u32 = 1;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_HP_GAIN: u32 = 31;
const SKILL_USAGE_TARGET_MP_GAIN: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyTransferKind {
    Health,
    Mana,
}

impl BattleFairyTransferKind {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Health => HUOXIESHU_SKILL_ID,
            Self::Mana => LINGZHISHU_SKILL_ID,
        }
    }

    const fn cost_usage(self) -> u32 {
        match self {
            Self::Health => SKILL_USAGE_USER_HP_LOSE,
            Self::Mana => SKILL_USAGE_USER_MP_LOSE,
        }
    }

    const fn gain_usage(self) -> u32 {
        match self {
            Self::Health => SKILL_USAGE_TARGET_HP_GAIN,
            Self::Mana => SKILL_USAGE_TARGET_MP_GAIN,
        }
    }

    const fn failure_action(self) -> u8 {
        match self {
            Self::Health => 6,
            Self::Mana => 7,
        }
    }

    const fn source(self, player: &CPlayer) -> u32 {
        match self {
            Self::Health => player.health(),
            Self::Mana => player.mana(),
        }
    }

    const fn initial_cost_unavailable(self, current: u32, cost: u32) -> bool {
        if cost == 0 {
            return false;
        }
        match self {
            Self::Health => (current.wrapping_sub(cost).wrapping_sub(1) as i32) < 0,
            Self::Mana => (current.wrapping_sub(cost) as i32) < 0,
        }
    }

    const fn runtime_cost_unavailable(self, current: u32, cost: u32) -> bool {
        (current.wrapping_sub(cost).wrapping_sub(1) as i32) < 0
    }

    fn deduct(self, player: &mut CPlayer, current: u32, cost: u32) {
        match self {
            Self::Health => player.set_health(current.wrapping_sub(cost)),
            Self::Mana => player.set_mana(current.wrapping_sub(cost)),
        }
    }
}

fn send_failure(game: &CGame, player_id: i32, action: u8) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(4);
    message.add_byte(action);
    let _ = message.send_to_player(game.net_server(), player_id);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    kind: BattleFairyTransferKind,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(kind.skill_id() as i32);
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

pub(crate) fn execute_battle_fairy_transfer<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    kind: BattleFairyTransferKind,
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
        } if skill_id == kind.skill_id() => skill_level,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(properties) = game.skill_base_properties(kind.skill_id(), skill_level) else {
        send_cast(game, player_id, kind, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let cost = properties.query_property(kind.cost_usage());
    let gain = properties.query_property(kind.gain_usage());
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.battle_fairy_transfer().is_none() {
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.battle_fairy_transfer_last_used_ms(kind) != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.battle_fairy_transfer_last_used_ms(kind),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            send_cast(game, player_id, kind, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if kind.initial_cost_unavailable(kind.source(player), cost) {
            send_failure(game, player_id, kind.failure_action());
            let (string_id, value) = match kind {
                BattleFairyTransferKind::Health => (b"ZHGS0054".as_slice(), cost.wrapping_add(1)),
                BattleFairyTransferKind::Mana => (b"ZHGS0052".as_slice(), cost),
            };
            game.send_skill_system_info_with_unsigned(player_id, string_id, value);
            send_cast(game, player_id, kind, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        player_ai.begin_battle_fairy_transfer(SkillExecutionKernel::begin(
            dispatch,
            started_at_ms,
        ));
    } else if player_ai
        .battle_fairy_transfer()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game
        .find_player(player_id)
        .and_then(|player| player.server_region_id())
        .is_none()
    {
        send_cast(game, player_id, kind, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .battle_fairy_transfer()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if player.equipment().get_goods(10).is_none() {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        let current = kind.source(player);
        if kind.runtime_cost_unavailable(current, cost) {
            send_failure(game, player_id, kind.failure_action());
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", cost);
            send_cast(game, player_id, kind, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            kind.deduct(player, current, cost);
        }
        send_cast(game, player_id, kind, skill_level, 1);
        if let Some(state) = player_ai.battle_fairy_transfer_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .battle_fairy_transfer()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("исполнение передачи ресурса создано или восстановлено");
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
    send_cast(game, player_id, kind, skill_level, 2);
    let update = game.find_player_mut(player_id).and_then(|player| match kind {
        BattleFairyTransferKind::Health => {
            player.restore_war_soul_health(gain, &goods_factory, da_kong_key)
        }
        BattleFairyTransferKind::Mana => {
            player.restore_war_soul_mana(gain, &goods_factory, da_kong_key)
        }
    });
    if let Some(update) = update.as_ref() {
        send_goods_update(game, update);
    } else {
        tracing::warn!(player_id, skill_id = kind.skill_id(), "не удалось сериализовать боевой дух после передачи ресурса");
    }
    if let Some(state) = player_ai.battle_fairy_transfer_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_battle_fairy_transfer_used(kind, runtime.now_milliseconds());
    send_cast(game, player_id, kind, skill_level, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
