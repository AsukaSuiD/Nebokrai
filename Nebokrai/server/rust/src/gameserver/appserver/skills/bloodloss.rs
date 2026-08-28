//! Потеря крови боевого духа `CBloodLoss` (`0x21D`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bloodloss.cpp`. Модуль сохраняет проверки цели и пути,
//! расход MP боевого духа, задержку, пакеты и замену периодического состояния.
//! `CGame` предоставляет владельцев, регион и атомарную установку состояния
//! игрока или монстра; последующие периодические удары принадлежат
//! `bloodlossstate.rs`. Координатные перегрузки `Begin` не подключены к
//! исполняющему владельцу и сохранены ниже как RAW без параллельного пути.

use super::baseattack::time_reached;
use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairytransfer::send_goods_update;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::bloodlossstate::{BloodLossState, send_blood_loss_state_visual};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const BLOOD_LOSS_SKILL_ID: u32 = 0x21d;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const VISUAL_OBJECT_TYPE: i32 = 700;
const DENIED_STATE_A: u32 = 0x192;
const DENIED_STATE_B: u32 = 0x67;
const DENIED_STATE_C: u32 = 0xd2;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn send_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_battle_fairy_skill_failure(player_id, action);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(BLOOD_LOSS_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(VISUAL_OBJECT_TYPE);
    message.add_long(player_id);
    if action == 2 {
        let Some((identity, x, y)) = target else { return };
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

pub(crate) fn execute_battle_fairy_blood_loss<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_level, target) = match dispatch {
        BattleFairySkillDispatch::Object {
            skill_id: BLOOD_LOSS_SKILL_ID,
            skill_level,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => (skill_level, target),
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(region_id) = player.server_region_id() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(BLOOD_LOSS_SKILL_ID, skill_level) else {
        send_failure(game, player_id, 2);
        send_cast(game, player_id, skill_level, 3, None);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as u16;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as u16;
    let damage_factor =
        properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as f32 * 0.01;
    let skill_name = properties.skill_name().to_vec();
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.blood_loss().is_none() {
        if target.id == player_id && target.object_type == PLAYER_TYPE {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
            || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B)
        {
            game.send_skill_system_info(player_id, b"ZHGS0046");
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C) {
            game.send_skill_system_info(player_id, b"ZHGS0047");
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.blood_loss_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.blood_loss_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            source_view.tile_x,
            source_view.tile_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"ZHGS0049");
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info_with_text(
                player_id,
                b"ZHGS0051",
                game.periodic_state_target_name(region_id, target),
            );
            send_failure(game, player_id, 2);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 {
            let Some(current) = game
                .find_player(player_id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else {
                send_failure(game, player_id, 2);
                send_cast(game, player_id, skill_level, 3, None);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            if i64::from(current) - i64::from(mp_loss) < 0 {
                send_failure(game, player_id, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    (f64::from(mp_loss) * 0.0001).round() as u32,
                );
                send_failure(game, player_id, 2);
                send_cast(game, player_id, skill_level, 3, None);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        player_ai.begin_blood_loss(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .blood_loss()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .blood_loss()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let target_alive = game
            .base_magic_target_view(region_id, target)
            .is_some_and(|_| !game.periodic_state_target_dead(region_id, target));
        if !target_alive {
            send_failure(game, player_id, 10);
            send_cast(game, player_id, skill_level, 3, None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 {
            let goods_factory = game.goods_factory().clone();
            let da_kong_key = game.globe_setup().da_kong_key();
            let update = game.find_player_mut(player_id).and_then(|player| {
                player.spend_war_soul_mana(mp_loss, &goods_factory, da_kong_key)
            });
            let Some(update) = update else {
                send_failure(game, player_id, 7);
                game.send_skill_system_info_with_unsigned(
                    player_id,
                    b"ZHGS0052",
                    (f64::from(mp_loss) * 0.0001).round() as u32,
                );
                send_cast(game, player_id, skill_level, 3, None);
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            send_goods_update(game, &update);
        }
        send_cast(game, player_id, skill_level, 1, None);
        if let Some(state) = player_ai.blood_loss_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .blood_loss()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение потери крови создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        send_failure(game, player_id, 10);
        send_cast(game, player_id, skill_level, 3, None);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.periodic_state_target_dead(region_id, target) {
        send_failure(game, player_id, 10);
        send_cast(game, player_id, skill_level, 3, None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let source_view = game
        .find_player(player_id)
        .and_then(CPlayer::shape_view)
        .expect("владелец потери крови сохранён");
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
        None,
    );
    if (maximum_distance != 0 && path.len() > maximum_distance as usize)
        || path.iter().any(|cell| cell.2 == 2)
    {
        let action = if maximum_distance != 0 && path.len() > maximum_distance as usize {
            0x0b
        } else {
            0x0f
        };
        send_failure(game, player_id, action);
        if action == 0x0b {
            game.send_skill_system_info(player_id, b"ZHGS0049");
        } else {
            game.send_skill_system_info_with_text(player_id, b"ZHGS0053", &skill_name);
        }
        send_cast(game, player_id, skill_level, 3, None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    send_cast(
        game,
        player_id,
        skill_level,
        2,
        Some((target, target_view.tile_x, target_view.tile_y)),
    );
    if target.object_type == PLAYER_TYPE {
        let _ = game.player_on_first_skill(player_id, target.id, Some(region_id), runtime);
    }
    let master = game
        .find_player(player_id)
        .map(master_info)
        .expect("владелец потери крови сохранён");
    let state_now_ms = runtime.now_milliseconds();
    let damage_modifier = game
        .find_player(player_id)
        .and_then(|player| player.war_soul_attack(game.goods_factory()))
        .map(|value| (f64::from(value) * 0.0001).round_ties_even() as i32 as f32)
        .unwrap_or_default();
    let state = BloodLossState::new(
        master,
        state_now_ms,
        keep_time_ms,
        frequency_ms,
        damage_factor,
        damage_modifier,
        minimum_attack,
        maximum_attack,
    );
    if let Some((previous, identity, x, y)) = game.replace_blood_loss_state(region_id, target, state) {
        if let Some(previous) = previous {
            send_blood_loss_state_visual(game, region_id, identity, x, y, previous, false, state_now_ms);
        }
        send_blood_loss_state_visual(game, region_id, identity, x, y, state, true, state_now_ms);
        if target.object_type == PLAYER_TYPE {
            let _ = game.publish_player_states(target.id);
        }
    }
    if let Some(execution) = player_ai.blood_loss_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_blood_loss_used(runtime.now_milliseconds());
    send_cast(game, player_id, skill_level, 3, None);
    terminal(QueuedSkillExecutionState::Completed)
}

// Остаются недостигнутыми координатные перегрузки запуска навыка.
// ============================================================================
// FUNCTION: CBloodLoss::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodloss.cpp:197
// RVA: 0x0011A550
// ADDRESS: 0051a550
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLoss::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodloss.cpp:214
// RVA: 0x0011A620
// ADDRESS: 0051a620
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
