//! Усиление `CPromotion` (`0x142`) для игрока и монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/promotion.cpp`. Подключённые пути `SelfTarget` и `Object` сохраняют
//! двойную проверку MP, время восстановления, расстояние, задержку, направление, пакеты
//! `0xBFE01` и подтверждённую особенность повторного применения: прежнее
//! состояние завершается, но новое в этот проход не создаётся. `CGame`
//! предоставляет владельцев и доставку, а проверки, формулы и жизненный цикл
//! остаются в этом модуле. Координатная перегрузка сохранена ниже как RAW,
//! потому что её исходный выбор цели состояния ещё не достигнут цепочкой выполнения.

use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::promotionstate::{PromotionState, send_promotion_state_begin};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;

pub(crate) const PROMOTION_SKILL_ID: u32 = 0x142;
pub(crate) const PROMOTION_EFFECT_MESSAGE: i32 = 0x000b_fe01;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy)]
struct TargetSnapshot {
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    dead: bool,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn finish_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

fn requested_target(player_id: i32, dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. } if skill_id == PROMOTION_SKILL_ID => {
            Some(ShapeIdentity {
                object_type: PLAYER_TYPE,
                id: player_id,
                ex_id: CGuid::GUID_INVALID,
            })
        }
        PlayerSkillDispatch::Object { skill_id, target }
            if skill_id == PROMOTION_SKILL_ID
                && matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) =>
        {
            Some(target)
        }
        _ => None,
    }
}

fn target_snapshot(
    game: &CGame,
    region_id: i32,
    identity: ShapeIdentity,
) -> Option<TargetSnapshot> {
    let (tile_x, tile_y) = game.move_shape_target_tile(Some(region_id), identity)?;
    let dead = match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(identity.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => return None,
    };
    Some(TargetSnapshot {
        identity,
        tile_x,
        tile_y,
        dead,
    })
}

fn send_failure(game: &mut CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(PROMOTION_EFFECT_MESSAGE, player_id, action);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target: TargetSnapshot,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let source = player.shape().identity();
    let mut message = CMessage::new(PROMOTION_EFFECT_MESSAGE);
    match action {
        0 => {
            message.add_byte(1);
            message.add_long(PROMOTION_SKILL_ID as i32);
            message.add_short(skill_level as i16);
            message.add_long(source.object_type);
            message.add_long(source.id);
            message.add_long(player.shape().get_direction());
        }
        1 => {
            message.add_byte(2);
            message.add_long(PROMOTION_SKILL_ID as i32);
            message.add_short(skill_level as i16);
            message.add_long(source.object_type);
            message.add_long(source.id);
            message.add_long(target.identity.object_type);
            message.add_long(target.identity.id);
            message.add_long(target.tile_x);
            message.add_long(target.tile_y);
        }
        _ => return,
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn install_state(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    state: PromotionState,
) -> Option<bool> {
    match target.object_type {
        PLAYER_TYPE => {
            let player = game.find_player_mut(target.id)?;
            (player.server_region_id() == Some(region_id))
                .then(|| player.begin_promotion_state(state))
        }
        MONSTER_TYPE => {
            let mut owner = game.take_region_owner(region_id)?;
            let installed = owner
                .base_mut()
                .find_monster_by_id_mut(target.id)
                .map(|monster| monster.move_shape_mut().begin_promotion_state(state));
            game.restore_region_owner(owner);
            installed
        }
        _ => None,
    }
}

pub(crate) fn execute_player_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(target_identity) = requested_target(player_id, dispatch) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some((region_id, source_x, source_y, skill_level, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(PROMOTION_SKILL_ID),
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let target = target_snapshot(game, region_id, target_identity);
    let Some(properties) = game.skill_base_properties(PROMOTION_SKILL_ID, skill_level) else {
        send_failure(game, player_id, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let magic_attack_factor = properties.query_property(SKILL_USAGE_EM_MODIFIER) as u16;
    let heal_recover_factor =
        properties.query_property(SKILL_USAGE_HEAL_RECOVER_COEFFICIENT) as u16;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.promotion().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let Some(target) = target else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.promotion_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.promotion_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if player_id != target.identity.id || target.identity.object_type != PLAYER_TYPE {
            if maximum_distance != 0
                && game
                    .base_magic_path(
                        region_id,
                        source_x,
                        source_y,
                        target.tile_x,
                        target.tile_y,
                        None,
                    )
                    .len()
                    > maximum_distance as usize
            {
                send_failure(game, player_id, 0x0b);
                game.send_skill_system_info(player_id, b"GS0290");
                send_failure(game, player_id, 2);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        if mp_loss != 0 && initial_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(PROMOTION_SKILL_ID));
        }
        player_ai.begin_promotion(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .promotion()
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(target) = target_snapshot(game, region_id, target_identity) else {
        finish_movement(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.dead {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        finish_movement(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .promotion()
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish_movement(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                target.tile_x,
                target.tile_y,
            ));
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        send_cast(game, player_id, target, skill_level, 0);
        if let Some(execution) = player_ai.promotion_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .promotion()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение усиления создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_cast(game, player_id, target, skill_level, 1);
    let state_now_ms = runtime.now_milliseconds();
    let state = PromotionState::new(
        state_now_ms,
        keep_time_ms,
        magic_attack_factor,
        heal_recover_factor,
    );
    if install_state(game, region_id, target.identity, state) == Some(true) {
        send_promotion_state_begin(
            game,
            region_id,
            target.identity,
            target.tile_x,
            target.tile_y,
            state,
            state_now_ms,
        );
    }
    let _ = game.publish_player_states(player_id);
    if let Some(execution) = player_ai.promotion_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_promotion_used(runtime.now_milliseconds());
    finish_movement(game, player_id);
    terminal(QueuedSkillExecutionState::Completed)
}

// Остаётся недостигнутой координатная перегрузка выбора цели состояния.
// ============================================================================
// FUNCTION: CPromotion::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\promotion.cpp:172
// RVA: 0x001686A0
// ADDRESS: 005686a0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1,long param_2,long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
