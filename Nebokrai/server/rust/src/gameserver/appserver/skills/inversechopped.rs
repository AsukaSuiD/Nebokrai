//! Обратный рубящий удар `CInverseChopped` (`0x8A`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/inversechopped.cpp`. Навык атакует все допустимые фигуры
//! лицевой клетки в региональном порядке. Накопленная энергия потребляется
//! при расчёте первой атаки, усиливает только её, а при отсутствии цели всё
//! равно снимается после обхода. Формулы и RNG остаются у владельца навыка;
//! `CGame` связывает независимых владельцев цели, боя и доставки. Успех,
//! отказ после `Begin` и клиентская отмена проходят через подтверждённый
//! `CSummonSkill::End(1)`: возврат движения, обновление свойств, очистку и
//! фиксацию времени восстановления.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::energyholdingstate::take_player_energy_holding;
use super::flash::cell_views;
use super::frontcellsword::{FrontCellSwordDefinition, MONSTER_TYPE, PLAYER_TYPE, calculate_attack_with_multiplier, destination, master_info, send_failure, send_visual, target_level, weapon_is_compatible};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::public::tools::get_line_direction;

pub(crate) const INVERSE_CHOPPED_SKILL_ID: u32 = 0x8a;
const USER_MP_LOSE: u32 = 2;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const DEFINITION: FrontCellSwordDefinition = FrontCellSwordDefinition { skill_id: INVERSE_CHOPPED_SKILL_ID, weapon_category: 2, weapon_failure_string: b"GS0292" };

pub(crate) const fn is_inverse_chopped_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: INVERSE_CHOPPED_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: INVERSE_CHOPPED_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: INVERSE_CHOPPED_SKILL_ID, .. })
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }

fn finish_player_inverse_chopped<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_inverse_chopped_used(now_ms);
    });
}

pub(crate) fn cancel_player_inverse_chopped<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.inverse_chopped().map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_inverse_chopped(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn front_targets(game: &CGame, region_id: i32, player_id: i32) -> Vec<crate::gameserver::appserver::shape::ShapeIdentity> {
    let Some(face) = game.find_player(player_id).and_then(|player| player.shape().get_face_position().ok()) else { return Vec::new() };
    cell_views(game, region_id, face.x, face.y).into_iter().map(|view| view.identity).collect()
}

pub(crate) fn execute_player_inverse_chopped<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_inverse_chopped_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, level, source_x, source_y, mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(INVERSE_CHOPPED_SKILL_ID), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(INVERSE_CHOPPED_SKILL_ID, level) else { if ai.inverse_chopped().is_some() { finish_player_inverse_chopped(game, player_id, ai, runtime) } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let target_damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.inverse_chopped().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if ai.inverse_chopped_last_used_ms() != 0 && !time_reached(cooldown_now_ms, ai.inverse_chopped_last_used_ms(), reuse_delay_ms) { send_failure(game, player_id, DEFINITION, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_compatible(game, player, DEFINITION) { send_failure(game, player_id, DEFINITION, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, DEFINITION, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(INVERSE_CHOPPED_SKILL_ID)); }
        ai.begin_inverse_chopped(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if ai.inverse_chopped().is_none_or(|execution| execution.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    if game.find_player(player_id).is_some_and(CPlayer::is_dead) { finish_player_inverse_chopped(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    if ai.inverse_chopped().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, DEFINITION, 7, mp_loss); finish_player_inverse_chopped(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let target = destination(game, region_id, player_id, dispatch).map(|(_, x, y)| (x, y)).unwrap_or((source_x, source_y));
        if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target.0, target.1)); }
        send_visual(game, player_id, DEFINITION, level, dispatch, 1);
        if let Some(execution) = ai.inverse_chopped_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = ai.inverse_chopped().map(SkillExecutionKernel::started_at_ms).expect("выполнение обратного рубящего удара создано выше");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }
    send_visual(game, player_id, DEFINITION, level, dispatch, 2);
    if let Some(execution) = ai.inverse_chopped_mut() { let _ = execution.advance(SkillStage::Check, SkillStage::Calculate); let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack); }
    let owner = game.find_player(player_id).map(master_info).unwrap_or_default();
    for target in front_targets(game, region_id, player_id) {
        if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || (target.object_type == PLAYER_TYPE && target.id == player_id) || game.periodic_state_target_dead(region_id, target) || !game.owned_player_skill_target_attackable(owner, target, region_id) { continue }
        let Some(target_level) = target_level(game, region_id, target) else { continue };
        let energy = take_player_energy_holding(game, player_id);
        let multiplier = energy.map_or(1.0, |state| 1.0 + f64::from(state.energy_count()) * f64::from(state.parameter_percent()) * 0.01);
        let Some((master, attack)) = calculate_attack_with_multiplier(game, player_id, DEFINITION, target_level, level, hit_modifier, target_damage_factor, multiplier) else { continue };
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
            _ => unreachable!("тип цели проверен выше"),
        }
        game.damage_player_weapon(player_id, runtime);
    }
    let _ = take_player_energy_holding(game, player_id);
    if let Some(execution) = ai.inverse_chopped_mut() { let _ = execution.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_inverse_chopped(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
