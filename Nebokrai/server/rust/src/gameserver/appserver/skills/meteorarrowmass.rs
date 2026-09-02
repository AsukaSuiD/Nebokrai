//! Накопление метеорных стрел `CMeteorArrowMass` (`0xCC`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/meteorarrowmass.cpp`. Навык проверяет лук категории `3`,
//! дважды проверяет MP, списывает его до поздней проверки оружия и после
//! задержки пополняет единственный `MeteorArrowState`. `CGame` доставляет
//! уже построенные owner-ом пакеты и обновляет внешние свойства игрока.
//! Унаследованный `CStateSkill::End(true)` фиксирует cooldown без повторного
//! добавления состояния; `End(false)` только прерывает выполнение. Cooldown
//! следует абсолютному сроку `CSkill::IsRestored`; задержка стадии остаётся elapsed.

use super::baseattack::time_reached;
use super::basemagic::{BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::meteorarrowstate::MeteorArrowState;
pub(crate) use super::meteorarrowstate::METEOR_ARROW_MASS_SKILL_ID;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::abort_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;

const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const AMOUNT: u32 = 20_016;
const AMOUNT_LIMIT: u32 = 20_017;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MeteorArrowMassExecutionState { kernel: SkillExecutionKernel<PlayerSkillDispatch>, condition_checked: bool }
impl MeteorArrowMassExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), condition_checked: false } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}
fn result(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn weapon_valid(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3) }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_meteor_arrow_mass<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    abort_skill(game, player_id);
    ai.mark_meteor_arrow_mass_used(runtime.now_milliseconds());
}
fn abort_player_meteor_arrow_mass(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); abort_skill(game, player_id); }
pub(crate) fn complete_player_meteor_arrow_mass<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.meteor_arrow_mass().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_meteor_arrow_mass(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_meteor_arrow_mass<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.meteor_arrow_mass().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_meteor_arrow_mass(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn send_cast(game: &mut CGame, player_id: i32, level: i32, action: u8) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action); message.add_long(METEOR_ARROW_MASS_SKILL_ID as i32); message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if action == 1 { message.add_long(player.shape().get_direction()); }
    else { message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_tile_x().unwrap_or_default()); message.add_long(player.shape().get_tile_y().unwrap_or_default()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}
pub(crate) fn send_meteor_arrow_state_add(game: &mut CGame, player_id: i32, state: MeteorArrowState) {
    let mut message = CMessage::new(0x000b_fe03); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    message.add_long(state.skill_id() as i32); message.add_ulong(0); message.add_long(state.additional_data());
    let _ = game.send_player_shape_around(player_id, None, &message);
}
pub(crate) fn send_meteor_arrow_state_remove(game: &mut CGame, player_id: i32) {
    let mut message = CMessage::new(0x000b_fe04); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(METEOR_ARROW_MASS_SKILL_ID as i32);
    let _ = game.send_player_shape_around(player_id, None, &message);
}
pub(crate) const fn is_meteor_arrow_mass_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: METEOR_ARROW_MASS_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: METEOR_ARROW_MASS_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: METEOR_ARROW_MASS_SKILL_ID, .. })
}
pub(crate) fn execute_player_meteor_arrow_mass<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_meteor_arrow_mass_dispatch(dispatch) { return result(QueuedSkillExecutionState::Rejected) }
    let Some(level) = game.find_player(player_id).map(|p| p.learned_skill_level(METEOR_ARROW_MASS_SKILL_ID)) else { return result(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(METEOR_ARROW_MASS_SKILL_ID, level) else { if ai.meteor_arrow_mass().is_some() { abort_player_meteor_arrow_mass(game, player_id); } return result(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let amount = properties.query_property(AMOUNT); let limit = properties.query_property(AMOUNT_LIMIT);
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.meteor_arrow_mass().is_none() {
        if !skill_is_restored(ai.meteor_arrow_mass_last_used_ms(), reuse, runtime.now_milliseconds()) {
            game.send_base_magic_failure(player_id, 0x0d); game.send_skill_system_info(player_id, b"GS0278"); return result(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else { return result(QueuedSkillExecutionState::Rejected) };
        if !weapon_valid(game, player) { game.send_base_magic_failure(player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0297"); return result(QueuedSkillExecutionState::Rejected); }
        if mp_loss == 0 { return result(QueuedSkillExecutionState::Rejected) }
        if (player.mana().wrapping_sub(mp_loss) as i32) < 0 { game.send_base_magic_failure(player_id, 7); game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss); return result(QueuedSkillExecutionState::Rejected); }
        let now = runtime.now_milliseconds(); if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(METEOR_ARROW_MASS_SKILL_ID)); }
        ai.begin_meteor_arrow_mass(MeteorArrowMassExecutionState::begin(dispatch, now));
    } else if ai.meteor_arrow_mass().is_none_or(|state| state.kernel().dispatch() != dispatch) { return result(QueuedSkillExecutionState::Rejected) }
    if ai.meteor_arrow_mass().is_some_and(|state| !state.condition_checked) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 { game.send_base_magic_failure(player_id, 7); game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss); abort_player_meteor_arrow_mass(game, player_id); return result(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_valid(game, player)) { game.send_base_magic_failure(player_id, 0x0e); game.send_skill_system_info(player_id, b"GS0297"); abort_player_meteor_arrow_mass(game, player_id); return result(QueuedSkillExecutionState::Rejected); }
        send_cast(game, player_id, level, 1);
        if let Some(state) = ai.meteor_arrow_mass_mut() { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = ai.meteor_arrow_mass().expect("накопление создано").kernel().started_at_ms();
    if !time_reached(runtime.now_milliseconds(), started, delay) { return result(QueuedSkillExecutionState::Pending) }
    send_cast(game, player_id, level, 2);
    let was_missing = game.find_player(player_id).is_some_and(|player| player.meteor_arrow_state().is_none());
    let state = game.find_player_mut(player_id).and_then(|player| player.add_meteor_arrows(limit, amount));
    if was_missing { send_meteor_arrow_state_add(game, player_id, MeteorArrowState::new(limit)); }
    if let Some(state) = state { send_meteor_arrow_state_add(game, player_id, state); }
    if was_missing || state.is_some() { let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); }
    if let Some(state) = ai.meteor_arrow_mass_mut() { let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_meteor_arrow_mass(game, player_id, ai, runtime); result(QueuedSkillExecutionState::Completed)
}
