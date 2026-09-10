//! Защитная стойка `CPillar` (`0x74`).
//! Begin возвращает Begun после инициализации; повторные проверки и эффекты
//! первого AI исполняются после постановки Attack в том же Run.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/pillar.cpp`. Навык проверяет только восстановление при
//! постановке, списывает MP на первом проходе, после задержки заменяет
//! канонический `PillarState` и обновляет свойства. Состояние и поздний
//! коэффициент защиты принадлежат `pillarstate` и `fightdefense`; `CGame`
//! выполняет только доставку и координацию владельца игрока. Подтверждённый
//! `CSummonSkill::End(1)` после replacement возвращает движение, обновляет
//! свойства игрока и фиксирует cooldown; отказ после `Begin` и клиентская
//! отмена используют тот же хвост без создания нового состояния.
//! Беззнаковый коэффициент состояния умножается на сохранённую `f32`-константу
//! `0.001` в расширенной точности x87 и только затем записывается в `float`.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; задержка
//! установки состояния остаётся elapsed.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::pillarstate::{PillarState, replace_player_pillar_state};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;

pub(crate) const PILLAR_SKILL_ID: u32 = 0x74;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch {
    PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. }
    | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
} }
pub(crate) fn is_pillar_dispatch(dispatch: PlayerSkillDispatch) -> bool { skill_id(dispatch) == PILLAR_SKILL_ID }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false } }
fn finish_player_pillar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, PILLAR_SKILL_ID, runtime);
}

pub(crate) fn cancel_player_pillar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, PILLAR_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_pillar(game, player_id, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn failure(game: &CGame, player_id: i32, code: u8, amount: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"), _ => {} }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, fire: bool) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if fire { 2 } else { 1 }); message.add_long(PILLAR_SKILL_ID as i32);
    message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if fire { message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_tile_x().unwrap_or_default()); message.add_long(player.shape().get_tile_y().unwrap_or_default()); }
    else { message.add_long(player.shape().get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn execute_player_pillar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    _ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_pillar_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some(level) = game.find_player(player_id).map(|player| player.learned_skill_level(PILLAR_SKILL_ID, game.skill_factory())) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(PILLAR_SKILL_ID, level) else { if game.player_skill_execution(player_id, PILLAR_SKILL_ID).is_some() { finish_player_pillar(game, player_id, runtime); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let keep = properties.query_property(STATE_PERSIST_TIME);
    let damage_factor = (f64::from(properties.query_property(TARGET_DAMAGE_FACTOR))
        * f64::from(0.001_f32)) as f32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if game.player_skill_execution(player_id, PILLAR_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds(); let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, PILLAR_SKILL_ID), reuse, cooldown_now_ms) { failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(PILLAR_SKILL_ID)); }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, PILLAR_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if game.find_player(player_id).is_some_and(CPlayer::is_dead) { failure(game, player_id, 2, mp_loss); finish_player_pillar(game, player_id, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    if game.player_skill_execution(player_id, PILLAR_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if u64::from(mana) < u64::from(mp_loss) { failure(game, player_id, 7, mp_loss); finish_player_pillar(game, player_id, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, false);
        if let Some(state) = game.player_skill_execution_mut(player_id, PILLAR_SKILL_ID) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = game.player_skill_execution(player_id, PILLAR_SKILL_ID).map(SkillExecutionKernel::started_at_ms).unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay) { return terminal(QueuedSkillExecutionState::Pending) }
    send_visual(game, player_id, level, true); let now_ms = runtime.now_milliseconds();
    let state = PillarState::new(now_ms, keep, damage_factor); let _ = replace_player_pillar_state(game, player_id, state, now_ms);
    if let Some(state) = game.player_skill_execution_mut(player_id, PILLAR_SKILL_ID) { let _ = state.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_pillar(game, player_id, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
