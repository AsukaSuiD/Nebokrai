//! Общая вертикаль исполнения восьми атрибутных навыков боевого духа.
//!
//! Конкретные идентификаторы и коды свойств принадлежат модулям навыков.
//! Этот владелец семейства сохраняет общий порядок: проверка экипировки,
//! задержки повторного применения и запаса MP, необратимое списание и
//! `0xBF918`, задержка, действия применения 2 и 3, замена состояния и
//! пересчёт свойств. `CGame` используется только для разрешения владельцев
//! и фактической доставки.

use super::baseattack::time_reached;
use super::battlefairyattributestate::{
    ATTRIBUTE_STATE_BEGIN_MESSAGE, ATTRIBUTE_STATE_END_MESSAGE, BattleFairyAttributeKind,
    BattleFairyAttributeState,
};
use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{BattleFairyManaSpendOutcome, BattleFairySkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const VISUAL_OBJECT_TYPE: i32 = 700;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyAttributeSkill {
    pub(crate) value_usage: u32,
    pub(crate) kind: BattleFairyAttributeKind,
}

pub(crate) const fn definition(skill_id: u32) -> Option<BattleFairyAttributeSkill> {
    match skill_id {
        super::pojia::SKILL_ID => Some(super::pojia::DEFINITION),
        super::pobing::SKILL_ID => Some(super::pobing::DEFINITION),
        super::pomo::SKILL_ID => Some(super::pomo::DEFINITION),
        super::pofa::SKILL_ID => Some(super::pofa::DEFINITION),
        super::yujia::SKILL_ID => Some(super::yujia::DEFINITION),
        super::yubing::SKILL_ID => Some(super::yubing::DEFINITION),
        super::yumo::SKILL_ID => Some(super::yumo::DEFINITION),
        super::yufa::SKILL_ID => Some(super::yufa::DEFINITION),
        _ => None,
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn dispatch_fields(dispatch: BattleFairySkillDispatch) -> (u32, i32, Option<ShapeIdentity>) {
    match dispatch {
        BattleFairySkillDispatch::SelfTarget { skill_id, skill_level, .. }
        | BattleFairySkillDispatch::Point { skill_id, skill_level, .. } => {
            (skill_id, skill_level, None)
        }
        BattleFairySkillDispatch::Object { skill_id, skill_level, target } => {
            (skill_id, skill_level, Some(target))
        }
    }
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target: ShapeIdentity,
    skill_id: u32,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    let source = player.shape().identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    match action {
        1 => {
            message.add_byte(1);
            message.add_long(skill_id as i32);
            message.add_short(skill_level as i16);
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(player.shape().get_direction());
        }
        2 => {
            let (x, y) = game.move_shape_target_tile(player.server_region_id(), target)
                .unwrap_or_default();
            message.add_byte(2);
            message.add_long(skill_id as i32);
            message.add_short(skill_level as i16);
            message.add_long(source.object_type);
            message.add_long(source.id);
            message.add_long(target.object_type);
            message.add_long(target.id);
            message.add_long(x);
            message.add_long(y);
        }
        3 => {
            message.add_byte(3);
            message.add_long(skill_id as i32);
            message.add_short(skill_level as i16);
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(player.shape().get_direction());
        }
        _ => return,
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_goods_update(game: &mut CGame, update: &crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate) {
    let mut message = CMessage::new(update.message_type as i32);
    message.add_long(update.player_id);
    message.base_mut().add_guid(update.goods.ex_id);
    message.add_ulong(update.old_client_payload.len() as u32);
    message.base_mut().add(&update.old_client_payload);
    let _ = message.send_to_player(game.net_server(), update.player_id);
}

pub(crate) fn send_state_visual(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BattleFairyAttributeState,
    begin: bool,
) {
    let mut message = CMessage::new(if begin { ATTRIBUTE_STATE_BEGIN_MESSAGE } else { ATTRIBUTE_STATE_END_MESSAGE });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.keep_time_ms() as i32);
        message.add_long(state.started_at_ms() as i32);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn execute_battle_fairy_attribute<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_id, skill_level, requested_target) = dispatch_fields(dispatch);
    let Some(definition) = definition(skill_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some((region_id, source_identity)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity()))
    }) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let target = if definition.kind.targets_self() {
        source_identity
    } else {
        let Some(target) = requested_target.filter(|target| matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)) else {
            send_cast(game, player_id, source_identity, skill_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        target
    };
    if game.move_shape_target_tile(Some(region_id), target).is_none() {
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let value = properties.query_property(definition.value_usage) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.battle_fairy_attribute().is_none() {
        let now_ms = runtime.now_milliseconds();
        if player_ai.battle_fairy_attribute_last_used_ms(skill_id) != 0
            && !time_reached(now_ms, player_ai.battle_fairy_attribute_last_used_ms(skill_id), delay_ms)
        {
            game.send_battle_fairy_skill_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            send_cast(game, player_id, target, skill_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(current) = game.find_player(player_id).and_then(|player| player.war_soul_mana(game.goods_factory())) else {
            let text = game.get_string_by_id(b"ZHGS0011");
            let _ = colored_player_notice_message(0xffff_ffff, 0, text).send_to_player(game.net_server(), player_id);
            send_cast(game, player_id, target, skill_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if current.wrapping_sub(mp_loss as i32) < 0 {
            game.send_battle_fairy_skill_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", (f64::from(mp_loss) * 0.0001).round() as u32);
            send_cast(game, player_id, target, skill_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        player_ai.begin_battle_fairy_attribute(SkillExecutionKernel::begin(dispatch, now_ms));
    } else if player_ai.battle_fairy_attribute().is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.periodic_state_target_dead(region_id, target) {
        game.send_battle_fairy_skill_failure(player_id, 2);
        game.send_skill_system_info(player_id, b"ZHGS0050");
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.battle_fairy_attribute().is_some_and(|state| state.stage() == SkillStage::Begin) {
        let Some(current) = game.find_player(player_id).and_then(|player| player.equipped_battle_fairy_mana(game.goods_factory())) else {
            return terminal(QueuedSkillExecutionState::Pending);
        };
        if current.wrapping_sub(mp_loss as i32) < 0 {
            game.send_battle_fairy_skill_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", (f64::from(mp_loss) * 0.0001).round() as u32);
            send_cast(game, player_id, target, skill_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let factory = game.goods_factory().clone();
        let da_kong_key = game.globe_setup().da_kong_key();
        let spend = game.find_player_mut(player_id).map(|player| {
            player.spend_equipped_battle_fairy_mana(mp_loss, &factory, da_kong_key)
        }).unwrap_or(BattleFairyManaSpendOutcome::MissingEquipment);
        let update = match spend {
            BattleFairyManaSpendOutcome::Spent { update } => update,
            BattleFairyManaSpendOutcome::MissingEquipment | BattleFairyManaSpendOutcome::SpentWithoutWarSoul => {
                return terminal(QueuedSkillExecutionState::Pending);
            }
        };
        if let Some(update) = update.as_ref() { send_goods_update(game, update); }
        send_cast(game, player_id, target, skill_id, skill_level, 1);
        if let Some(state) = player_ai.battle_fairy_attribute_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai.battle_fairy_attribute().map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение атрибутного навыка создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_cast(game, player_id, target, skill_id, skill_level, 2);
    let state_started_at_ms = runtime.now_milliseconds();
    let state = BattleFairyAttributeState::new(skill_id, definition.kind, state_started_at_ms, keep_time_ms, value);
    let Some((previous, tile_x, tile_y)) = game.replace_battle_fairy_attribute_state(region_id, target, state) else {
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if let Some(previous) = previous {
        send_state_visual(game, region_id, target, tile_x, tile_y, previous, false);
    }
    send_state_visual(game, region_id, target, tile_x, tile_y, state, true);
    if target.object_type == PLAYER_TYPE {
        let _ = game.update_player_properties(target.id, runtime);
    }
    if let Some(execution) = player_ai.battle_fairy_attribute_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_battle_fairy_attribute_used(skill_id, runtime.now_milliseconds());
    send_cast(game, player_id, target, skill_id, skill_level, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
