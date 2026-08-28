//! Призыв ядовитого тумана `CPoisonFog` (`0xC9`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonfog.cpp`. Владелец сохраняет диспетчеризацию цели
//! по точке и объекту,
//! cooldown, путь, оружие, двойную проверку MP и необратимое списание MP до
//! повторной проверки оружия. `CGame` только разрешает владельцев, регистрирует
//! `CPoisonFogPhalanx` и выполняет фактическую доставку вокруг объекта.

use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::poisonfogphalanx::CPoisonFogPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{GAP_WEAPON_CATEGORY, GAP_WEAPON_DAMAGE_LEVEL};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const POISON_FOG_SKILL_ID: u32 = 0xc9;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const MP_LOSS: u32 = 2; const MAX_DISTANCE: u32 = 5_003; const DELAY: u32 = 10_001;
const STATE_TIME: u32 = 10_002; const REUSE: u32 = 10_005; const CAN_BREAK: u32 = 10_006;
const DEF_LOSS: u32 = 209; const DODGE_LOSS: u32 = 210; const ELEMENT_LOSS: u32 = 212;
const DEF_COEFFICIENT: u32 = 223; const ER_COEFFICIENT: u32 = 224; const LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn weapon_valid(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 4) }
fn destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> { match dispatch { PlayerSkillDispatch::Point { skill_id: POISON_FOG_SKILL_ID, x, y } => Some((x, y)), PlayerSkillDispatch::Object { skill_id: POISON_FOG_SKILL_ID, target } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => { let target = game.base_magic_target_view(region_id, target)?; Some((target.tile_x, target.tile_y)) }, _ => None } }
fn fail(game: &mut CGame, player_id: i32, code: u8, text: &[u8], mp: Option<u32>) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code); if let Some(mp) = mp { game.send_skill_system_info_with_unsigned(player_id, text, mp); } else { game.send_skill_system_info(player_id, text); } }
fn visual(game: &mut CGame, player_id: i32, level: i32, action: u8, target: Option<(i32, i32)>) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(action); message.add_long(POISON_FOG_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); if action == 1 { message.add_long(player.shape().get_direction()); } else { let Some((x, y)) = target else { return }; message.add_long(0); message.add_long(0); message.add_long(x); message.add_long(y); } let _ = game.send_player_shape_around(player_id, None, &message); }
fn finish(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); player.set_current_skill_id(None); } }
pub(crate) const fn is_poison_fog_target(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Point { skill_id: POISON_FOG_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: POISON_FOG_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }) }

pub(crate) fn execute_player_poison_fog<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let Some((region_id, source_x, source_y, level, mana, master, weapon_level)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(POISON_FOG_SKILL_ID), player.mana(), MasterInfo { master_type: PLAYER_TYPE, master_id: player_id, master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: 0, permitted_to_kill_player: i32::from(player.pk_permissions().player), permitted_to_kill_teammate: i32::from(player.pk_permissions().teammate), permitted_to_kill_guild_member: i32::from(player.pk_permissions().guild_member), permitted_to_kill_criminal: i32::from(player.pk_permissions().criminal) }, player.equipment().get_goods(2).map_or(0, |weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1) as u32)))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some((mp_loss, max_distance, delay, reuse, can_break, lifetime, state_time, defense_loss, defense_coefficient, dodge_loss, element_loss, er_coefficient)) = game.skill_base_properties(POISON_FOG_SKILL_ID, level).map(|properties| (properties.query_property(MP_LOSS), properties.query_property(MAX_DISTANCE), properties.query_property(DELAY), properties.query_property(REUSE), properties.query_property(CAN_BREAK), properties.query_property(LIFETIME), properties.query_property(STATE_TIME), properties.query_property(DEF_LOSS), properties.query_property(DEF_COEFFICIENT), properties.query_property(DODGE_LOSS), properties.query_property(ELEMENT_LOSS), properties.query_property(ER_COEFFICIENT))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    if ai.poison_fog().is_none() { let started = runtime.now_milliseconds(); if ai.poison_fog_last_used_ms() != 0 && !time_reached(runtime.now_milliseconds(), ai.poison_fog_last_used_ms(), reuse) { fail(game, player_id, 0x0d, b"GS0278", None); return terminal(QueuedSkillExecutionState::Rejected) } let Some((x, y)) = destination(game, region_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, x, y, None); if max_distance != 0 && path.len() > max_distance as usize { fail(game, player_id, 0x0b, b"GS0290", None); return terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == 2) { fail(game, player_id, 0x0f, b"GS0282", None); return terminal(QueuedSkillExecutionState::Rejected) } let weapon_ok = game.find_player(player_id).is_some_and(|player| weapon_valid(game, player)); if !weapon_ok { fail(game, player_id, 0x0e, b"GS0293", None); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) } if (mana.wrapping_sub(mp_loss) as i32) < 0 { fail(game, player_id, 7, b"GS0288", Some(mp_loss)); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(POISON_FOG_SKILL_ID)); } ai.begin_poison_fog(SkillExecutionKernel::begin(dispatch, started), (x, y)); } else if ai.poison_fog().is_none_or(|state| state.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((x, y)) = ai.poison_fog_destination() else { finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
    if ai.poison_fog().is_some_and(|state| state.stage() == SkillStage::Begin) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { fail(game, player_id, 7, b"GS0288", Some(mp_loss)); finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if !game.find_player(player_id).is_some_and(|player| weapon_valid(game, player)) { fail(game, player_id, 0x0e, b"GS0293", None); finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } let _can_break = can_break; if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, x, y)); } visual(game, player_id, level, 1, None); if let Some(state) = ai.poison_fog_mut() { let _ = state.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.poison_fog().map(SkillExecutionKernel::started_at_ms).expect("выполнение ядовитого тумана создано"); if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) }
    finish(game, player_id); visual(game, player_id, level, 2, Some((x, y)));
    let id = game.allocate_summon_shape_id(); let now = runtime.now_milliseconds(); let mut phalanx = CPoisonFogPhalanx::new(id, master, now, lifetime, level, state_time, defense_loss, defense_coefficient, dodge_loss, element_loss, er_coefficient, weapon_level); phalanx.shape_mut().set_region_id(region_id);
    let summoned = game.add_poison_fog_phalanx(region_id, phalanx, x, y, now, runtime).is_some_and(|result| result.is_ok()); if summoned { let _ = game.send_poison_fog_phalanx_entry(region_id, id); }
    if let Some(state) = ai.poison_fog_mut() { let _ = state.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.advance(SkillStage::Attack, SkillStage::Apply); } ai.mark_poison_fog_used(runtime.now_milliseconds()); terminal(if summoned { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
