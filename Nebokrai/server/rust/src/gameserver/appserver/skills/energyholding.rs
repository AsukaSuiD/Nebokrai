//! Накопление энергии `CEnergyHolding` (`0x89`).
//! Begin возвращает Begun после инициализации; повторные проверки и эффекты
//! первого AI исполняются после постановки Attack в том же Run.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/energyholding.cpp`. Здесь находятся проверка лука,
//! перезарядка, предел зарядов по уровню навыка, расход MP, задержка и
//! визуальная последовательность. `CGame` оставляет только доступ к владельцу
//! `CPlayer` и рассылку изменения канонического состояния вокруг него. Заряд
//! добавляется только после задержки; общий `CSummonSkill::End(1)` затем
//! возвращает движение, обновляет свойства и фиксирует cooldown. Клиентская
//! отмена проходит тот же хвост без добавления заряда. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; накопление остаётся elapsed.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::energyholdingstate::add_player_energy_holding;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;

pub(crate) const ENERGY_HOLDING_SKILL_ID: u32 = 0x89;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
pub(crate) const PARAMETER_PERCENT: u32 = 20_020;

pub(crate) const fn is_energy_holding_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: ENERGY_HOLDING_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: ENERGY_HOLDING_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: ENERGY_HOLDING_SKILL_ID, .. })
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2)
}

fn failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    if code != 0x0e {
        game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    }
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0292"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(ENERGY_HOLDING_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(PLAYER_TYPE);
        message.add_long(player_id);
        message.add_long(player.shape().get_tile_x().unwrap_or_default());
        message.add_long(player.shape().get_tile_y().unwrap_or_default());
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_energy_holding<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(ENERGY_HOLDING_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_energy_holding<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(ENERGY_HOLDING_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_energy_holding(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_energy_holding<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_energy_holding_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((level, mana, energy_count)) = game.find_player(player_id).map(|player| (player.learned_skill_level(ENERGY_HOLDING_SKILL_ID), player.mana(), player.energy_holding_state().map_or(0, |state| state.energy_count()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(ENERGY_HOLDING_SKILL_ID, level) else { if ai.player_skill_execution(ENERGY_HOLDING_SKILL_ID).is_some() { finish_player_energy_holding(game, player_id, ai, runtime) } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let parameter_percent = properties.query_property(PARAMETER_PERCENT);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.player_skill_execution(ENERGY_HOLDING_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(ENERGY_HOLDING_SKILL_ID), reuse_delay_ms, cooldown_now_ms) { failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if u32::try_from(level).is_ok_and(|level| level <= energy_count) { game.send_skill_system_info(player_id, b"GS0299"); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(ENERGY_HOLDING_SKILL_ID)); }
        ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_execution(ENERGY_HOLDING_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    if game.find_player(player_id).is_some_and(CPlayer::is_dead) {
        failure(game, player_id, 2, mp_loss);
        finish_player_energy_holding(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if ai.player_skill_execution(ENERGY_HOLDING_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); finish_player_energy_holding(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, false);
        if let Some(execution) = ai.player_skill_execution_mut(ENERGY_HOLDING_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = ai.player_skill_execution(ENERGY_HOLDING_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение накопления энергии создано выше");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }
    send_visual(game, player_id, level, true);
    let installed = u32::try_from(level).is_ok_and(|level| add_player_energy_holding(game, player_id, level, parameter_percent));
    if let Some(execution) = ai.player_skill_execution_mut(ENERGY_HOLDING_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_energy_holding(game, player_id, ai, runtime);
    terminal(if installed { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
