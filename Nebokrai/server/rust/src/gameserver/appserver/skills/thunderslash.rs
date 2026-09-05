//! Громовое рассечение `CThunderSlash` (`0x72`).
//! Reuse проверяется exact `CSkill::IsRestored`. Каст сравнивает unsigned
//! now >= wrapping(start + delay), как cmp/jb в AI по `0x0057B1BE`;
//! срок и частота формы принадлежат отдельному `CThunderSlashPhalanx::AI`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderslash.cpp`. Навык требует топор категории `1` и
//! живое `RageBreakState`; при первом проходе он необратимо снимает состояние,
//! затем отдельно списывает MP и RP, сохраняя границы частичных эффектов, и
//! после задержки создаёт форму в клетке перед игроком. Формулы, пакеты и
//! жизненный цикл принадлежат этому модулю и `thunderslashphalanx`; `CGame`
//! координирует только владельцев региона, применение атаки и доставку.
//! После `Begin` успех, отказ и клиентская отмена используют точный общий
//! хвост `End(1)` с возвратом движения, `AfterUseSkill` и временем восстановления.
//! End RageBreak (0x0057B004) пересчитывает свойства до проверок MP/RP;
//! последующие ненулевые расходы проверяются по знаку DWORD-разности.
//! После обоих списаний вызывается OnChangeStates (0x0057B08E), а не
//! повторный UpdateProperty. Отказ после MP сохраняет это частичное списание.

use super::baseattack::{
    finish_delayed_base_attack, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_SUMMONED_LIFETIME};
use super::flash::master_info;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::ragebreakstate::end_player_rage_break_state;
use super::thunderslashphalanx::CThunderSlashPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const THUNDER_SLASH_SKILL_ID: u32 = 0x72;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch {
    PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. }
    | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
} }
pub(crate) fn is_thunder_slash_dispatch(dispatch: PlayerSkillDispatch) -> bool { skill_id(dispatch) == THUNDER_SLASH_SKILL_ID }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn finish_player_thunder_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_delayed_base_attack(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(THUNDER_SLASH_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_thunder_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.thunder_slash().map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_thunder_slash(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1) }

fn failure(game: &CGame, player_id: i32, code: u8, amount: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code { 4 => game.send_skill_system_info(player_id, b"GS0304"),
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0287"), _ => {} }
}

fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y, None)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y, Some(target))),
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, fire: Option<(Option<ShapeIdentity>, i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if fire.is_some() { 2 } else { 1 }); message.add_long(THUNDER_SLASH_SKILL_ID as i32);
    message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if let Some((target, x, y)) = fire { let target = target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: Default::default() });
        message.add_long(target.object_type); message.add_long(target.id); message.add_long(x); message.add_long(y);
    } else { message.add_long(player.shape().get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn execute_player_thunder_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_thunder_slash_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, mana, rp)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
        player.learned_skill_level(THUNDER_SLASH_SKILL_ID), player.mana(), player.rp(),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(THUNDER_SLASH_SKILL_ID, level) else {
        if ai.thunder_slash().is_some() {
            finish_player_thunder_slash(game, player_id, ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE); let rp_loss = properties.query_property(USER_RP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME); let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.thunder_slash().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(THUNDER_SLASH_SKILL_ID), reuse, cooldown_now_ms) {
            failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if mp_loss != 0 && mana < mp_loss { failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if rp_loss != 0 && u32::from(rp) < rp_loss { failure(game, player_id, 8, rp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(THUNDER_SLASH_SKILL_ID)); }
        ai.begin_thunder_slash(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if ai.thunder_slash().is_none_or(|state| state.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else {
        finish_player_thunder_slash(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if ai.thunder_slash().is_some_and(|state| state.stage() == SkillStage::Begin) {
        let Some(player) = game.find_player(player_id) else {
            finish_player_thunder_slash(game, player_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if !weapon_is_valid(game, player) {
            failure(game, player_id, 0x0e, mp_loss);
            finish_player_thunder_slash(game, player_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if !end_player_rage_break_state(game, player_id, runtime.now_milliseconds()) {
            failure(game, player_id, 4, mp_loss);
            finish_player_thunder_slash(game, player_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if mp_loss != 0 && (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            failure(game, player_id, 7, mp_loss);
            finish_player_thunder_slash(game, player_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
        let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if rp_loss != 0 && (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 {
            failure(game, player_id, 8, rp_loss);
            finish_player_thunder_slash(game, player_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { if rp_loss != 0 { player.set_rp(u32::from(current_rp).wrapping_sub(rp_loss) as u16); } player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); }
        let _ = game.publish_player_states(player_id); send_visual(game, player_id, level, None);
        if let Some(state) = ai.thunder_slash_mut() { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = ai.thunder_slash().map(SkillExecutionKernel::started_at_ms).unwrap_or_default();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending) }
    send_visual(game, player_id, level, Some((target, target_x, target_y)));
    let direction = game.find_player(player_id).map(|player| player.shape().get_direction()).unwrap_or_default();
    if let Ok(front) = CShape::get_direction_position(direction, ShapeAreaCoordinates { x: source_x, y: source_y }) {
        let summon_id = game.allocate_summon_shape_id(); let started_at_ms = runtime.now_milliseconds();
        if let Some((master, combat)) = game.find_player(player_id).map(|player| (master_info(player), player.combat_properties())) {
            let mut phalanx = CThunderSlashPhalanx::new(summon_id, master, started_at_ms, lifetime, level, frequency,
                combat.maximum_attack as i32, combat.minimum_attack as i32, combat.add_element_attack as i32,
                combat.dexterity as i32, i32::from(combat.cch), i32::from(combat.add_soul_attack), front.x, front.y);
            phalanx.shape_mut().set_region_id(region_id);
            let result = game.add_thunder_slash_phalanx(region_id, phalanx, front.x, front.y, started_at_ms, runtime);
            if result.as_ref().is_some_and(|result| result.is_ok()) { let _ = game.send_thunder_slash_phalanx_entry(region_id, summon_id, runtime); }
            tracing::trace!(region_id, player_id, summon_id, ?result, "создана форма громового рассечения");
        }
    }
    if let Some(state) = ai.thunder_slash_mut() { let _ = state.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_thunder_slash(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
