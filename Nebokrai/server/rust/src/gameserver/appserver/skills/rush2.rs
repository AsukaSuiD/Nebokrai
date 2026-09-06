//! Второй прямой рывок `CRush2` (`0x7c`).
//! Успешный Begin возвращает Begun до первого AI. Расход ресурсов,
//! перемещение и атака остаются у AI после постановки Attack в том же Run;
//! раннее время Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rush2.cpp`. Геометрия пути и wire-форма совпадают с
//! `CRush`, поэтому переиспользуются узкие семейные функции. Отличия сохранены
//! здесь: строгая проверка дальности, собственное `Rush2State` и вызов
//! боевого контакта после состояния и отбрасывания. Контакт сохраняет
//! канонический default skill-id `0x7fffffff` конструктора
//! `tagAttackInformation`. `CGame` только связывает каноническое состояние,
//! пространство, боевой owner и доставку. Завершение
//! использует подтверждённый семейный хвост `CRush::End`, но сохраняет
//! отдельный cooldown второго навыка. Он проверяется абсолютным сроком
//! `CSkill::IsRestored`; перемещение и состояние сохраняют elapsed-сроки.

use super::baseattack::{SKILL_USAGE_REUSE_DELAY_TIME, real_distance};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::{cell_views, master_info};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::rush::{
    build_path, destination, failure, finish_rush_owner, knockback_destination,
    scaled_state_time, send_visual, skill_id, target_identity, terminal, weapon_is_valid,
};
use super::rushstate2::Rush2State;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const RUSH_2_SKILL_ID: u32 = 0x7c;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;
const DEFAULT_CONTACT_SKILL_ID: u32 = 0x7fff_ffff;

pub(crate) fn is_rush_2_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    skill_id(dispatch) == RUSH_2_SKILL_ID
}

fn finish_player_rush_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_rush_owner(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(RUSH_2_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_rush_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(RUSH_2_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_rush_2(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn contact_attack(player: &CPlayer) -> AttackInformation {
    let master = master_info(player);
    AttackInformation {
        skill_id: DEFAULT_CONTACT_SKILL_ID,
        skill_level: 1,
        attacker_type: PLAYER_TYPE,
        attacker_id: player.player_id(),
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: 0,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют формулу состояния и отбрасывания")]
fn apply_targets<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    impact_x: i32,
    impact_y: i32,
    source_level: u8,
    state_time: u32,
    back_steps: u32,
    move_speed: u32,
    runtime: &mut Runtime,
) {
    let Some((source_x, source_y, master, contact)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, master_info(player), contact_attack(player)))
    }) else { return };
    for target in cell_views(game, region_id, impact_x, impact_y) {
        let identity = target.identity;
        if (identity.object_type == PLAYER_TYPE && identity.id == player_id)
            || !matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
            || !game.owned_player_skill_target_attackable(master, identity, region_id)
        {
            continue;
        }
        let target_level = match identity.object_type {
            PLAYER_TYPE => game.find_player(identity.id).map(CPlayer::level),
            MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
                let monster = owner.base().find_monster_by_id(identity.id)?;
                game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8)
            }),
            _ => None,
        };
        let Some(keep_time_ms) = target_level
            .map(|level| scaled_state_time(source_level, level, state_time))
            .filter(|time| *time != 0)
        else { continue };
        let (destination_x, destination_y, moved) = knockback_destination(
            game, region_id, source_x, source_y, target, back_steps,
        );
        let now_ms = runtime.now_milliseconds();
        let installed = game.apply_rush_2_control(
            region_id,
            identity,
            Rush2State::new(now_ms, keep_time_ms),
            destination_x,
            destination_y,
            move_speed.wrapping_mul(moved),
            now_ms,
        );
        if !installed { continue }
        game.apply_owned_skill_contact(master, identity, region_id, contact.clone(), runtime);
    }
}

pub(crate) fn execute_player_rush_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rush_2_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, level, source_level, source_x, source_y, mana, rp)) = game
        .find_player(player_id)
        .and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(RUSH_2_SKILL_ID), player.level(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana(), player.rp())))
    else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(RUSH_2_SKILL_ID, level) else {
        if ai.player_skill_execution(RUSH_2_SKILL_ID).is_some() { finish_player_rush_2(game, player_id, ai, runtime) }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let rp_loss = properties.query_property(USER_RP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let state_time = properties.query_property(STATE_PERSIST_TIME);
    let back_steps = properties.query_property(TARGET_BACK_STEP);
    let move_speed = properties.query_property(TARGET_MOVE_SPEED);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.player_skill_execution(RUSH_2_SKILL_ID).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(RUSH_2_SKILL_ID), reuse, now_ms) {
            failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if rp_loss != 0 && (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 { failure(game, player_id, 8, rp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if player.has_state_by_skill_id(0x74) { failure(game, player_id, 2, 0); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(RUSH_2_SKILL_ID));
        }
        ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_execution(RUSH_2_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
    if (current_mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); finish_player_rush_2(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
    let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
    if (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 { failure(game, player_id, 8, rp_loss); finish_player_rush_2(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    if let Some(player) = game.find_player_mut(player_id) { player.set_rp(u32::from(current_rp).wrapping_sub(rp_loss) as u16); }
    let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) { failure(game, player_id, 0x0e, 0); finish_player_rush_2(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((target_x, target_y)) = destination(game, region_id, player_id, dispatch) else { finish_player_rush_2(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(direction); }
    let Some((destination_cell, impact)) = build_path(game, region_id, source_x, source_y, target_x, target_y, maximum) else { finish_player_rush_2(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
    let _ = game.relocate_player_shape(player_id, region_id, destination_cell.0, destination_cell.1);
    send_visual(game, player_id, RUSH_2_SKILL_ID, level, false, target_identity(dispatch));
    if let Some(execution) = ai.player_skill_execution_mut(RUSH_2_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }

    send_visual(game, player_id, RUSH_2_SKILL_ID, level, true, target_identity(dispatch));
    if game.find_region(region_id).is_none()
        || real_distance(destination_cell.0, destination_cell.1, impact.0, impact.1) >= maximum as i32
    {
        game.send_self_state_skill_failure(0x000b_fe01, player_id, 2);
        finish_player_rush_2(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    apply_targets(game, player_id, region_id, impact.0, impact.1, source_level, state_time, back_steps, move_speed, runtime);
    if let Some(execution) = ai.player_skill_execution_mut(RUSH_2_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_rush_2(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
