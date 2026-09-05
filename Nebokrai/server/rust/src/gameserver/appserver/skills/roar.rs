//! Боевой клич `CRoar` (`0x83`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/roar.cpp`. Владелец сохраняет проверку меча, перезарядку,
//! единственный расход MP, обход окна 5×5 сначала по X через одиночный
//! `CServerRegion::GetShape`, PK-контакт от клетки заклинателя и замену
//! `RoarState`. `CGame` только связывает владельцев, свойства и доставку.
//! Успех, отказ после `Begin` и клиентская отмена используют подтверждённый
//! `CSummonSkill::End(1)`: возврат движения, обновление свойств, очистку и cooldown.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; установка
//! состояния сохраняет elapsed-задержку.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::roarstate::RoarState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;

pub(crate) const ROAR_SKILL_ID: u32 = 0x83;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_ATTACK_LOSE: u32 = 205;
const TARGET_ELEMENT_MODIFY_LOSE: u32 = 215;
const STATE_PERSIST_TIME: u32 = 10_002;

pub(crate) const fn is_roar_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: ROAR_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: ROAR_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: ROAR_SKILL_ID, .. })
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1)
}

fn failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0287"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(ROAR_SKILL_ID as i32);
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

fn finish_player_roar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(ROAR_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_roar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.roar().map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_roar(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn reached_targets(game: &CGame, region_id: i32, source_x: i32, source_y: i32) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let minimum_x = source_x.wrapping_sub(2).max(0);
    let minimum_y = source_y.wrapping_sub(2).max(0);
    let maximum_x = source_x.wrapping_add(2).min(region.region.width);
    let maximum_y = source_y.wrapping_add(2).min(region.region.height);
    let mut targets = Vec::new();
    let mut x = minimum_x;
    while x <= maximum_x {
        let mut y = minimum_y;
        while y <= maximum_y {
            let Ok(shape) = region.get_shape(x, y, area_width, area_height, game) else { return targets };
            if let Some(shape) = shape { targets.push(shape.identity); }
            y = y.wrapping_add(1);
        }
        x = x.wrapping_add(1);
    }
    targets
}

#[allow(clippy::too_many_arguments, reason = "аргументы сохраняют единый снимок свойств навыка")]
fn apply_targets<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, source_x: i32, source_y: i32, keep_time_ms: u32, attack_loss: i32, element_loss: i32, runtime: &mut Runtime) {
    let Some(master) = game.find_player(player_id).map(super::flash::master_info) else { return };
    for target in reached_targets(game, region_id, source_x, source_y) {
        if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue }
        if target.object_type == PLAYER_TYPE {
            let safe = game.find_region(region_id).is_none_or(|owner| {
                let source_safe = owner.get_security(source_x, source_y).ok() == Some(RegionSecurity::SAFE);
                let target_safe = game.find_player(target.id).and_then(|player| Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))).is_none_or(|(x, y)| owner.get_security(x, y).ok() == Some(RegionSecurity::SAFE));
                source_safe || target_safe
            });
            if safe { continue }
            let _ = game.player_on_first_skill_at_position(player_id, target.id, region_id, source_x, source_y, runtime);
        }
        let now_ms = runtime.now_milliseconds();
        let _ = game.install_roar_state(region_id, target, RoarState::new(now_ms, keep_time_ms, attack_loss, element_loss), runtime);
    }
}

pub(crate) fn execute_player_roar<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_roar_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, level, source_x, source_y, mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(ROAR_SKILL_ID), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(ROAR_SKILL_ID, level) else { if ai.roar().is_some() { finish_player_roar(game, player_id, ai, runtime) } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let keep_time_ms = properties.query_property(STATE_PERSIST_TIME);
    let attack_loss = properties.query_property(TARGET_ATTACK_LOSE) as i32;
    let element_loss = properties.query_property(TARGET_ELEMENT_MODIFY_LOSE) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.roar().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(ROAR_SKILL_ID), reuse_delay_ms, cooldown_now_ms) { failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(ROAR_SKILL_ID)); }
        ai.begin_roar(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if ai.roar().is_none_or(|execution| execution.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    if game.find_player(player_id).is_some_and(CPlayer::is_dead) {
        failure(game, player_id, 2, mp_loss);
        finish_player_roar(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if ai.roar().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); finish_player_roar(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, false);
        if let Some(execution) = ai.roar_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = ai.roar().map(SkillExecutionKernel::started_at_ms).expect("выполнение боевого клича создано выше");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }
    send_visual(game, player_id, level, true);
    apply_targets(game, player_id, region_id, source_x, source_y, keep_time_ms, attack_loss, element_loss, runtime);
    if let Some(execution) = ai.roar_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_roar(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
