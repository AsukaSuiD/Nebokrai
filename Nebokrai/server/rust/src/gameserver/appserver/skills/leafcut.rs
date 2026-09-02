//! Наложение периодического удара `CLeafCut` (`0x6B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcut.cpp`. Владелец дважды проверяет цель и путь,
//! запрещает ячейку с `lPetFigure == 2`, сохраняет частичное изменение
//! MP → RP → проверка оружия, фиксирует боевой снимок и заменяет
//! `CLeafCutState` в прежней позиции.
//! `CGame` оставляет только разрешение межвладельческой PK-ветви, доступ к
//! владельцу цели, применение состояния и доставку. Обе точные `AI` не
//! изнашивают оружие при наложении состояния: унаследованный `AfterUseSkill`
//! делает это один раз в подтверждённом хвосте `CSummonSkill::End(1)`, после
//! возврата движения.
//! Во всех трёх вариантах damage factor остаётся точным unsigned-значением x87
//! до умножения на `0.01f`. Оружейный модификатор отдельно сохраняется в
//! `f32`, а unsigned-уровень оружия умножается на него без предварительного
//! округления; оба результата сохраняются в `f32` перед конструктором состояния.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::leafcutstate::{send_leaf_cut_state_visual, LeafCutState};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const LEAF_CUT_SKILL_ID: u32 = 0x6b;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const WEAPON_DAMAGE_LEVEL_MODIFIER: u32 = 20_018;

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id } }
pub(crate) fn is_leaf_cut_dispatch(dispatch: PlayerSkillDispatch) -> bool { skill_id(dispatch) == LEAF_CUT_SKILL_ID }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn finish_player_leaf_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } game.damage_player_weapon(player_id, runtime); finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| { player_ai.mark_leaf_cut_used(now_ms); }); }
pub(crate) fn cancel_player_leaf_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = player_ai.leaf_cut().map(SkillExecutionKernel::dispatch) else { return false }; finish_player_leaf_cut(game, player_id, player_ai, runtime); player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }
fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2) }
fn dispatch_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> { match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None } }

fn target_facts(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<(i32, i32, bool, Vec<u8>)> {
    match target.object_type {
        PLAYER_TYPE => { let player = game.find_player(target.id)?; if player.server_region_id() != Some(region_id) { return None } Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.is_dead(), player.player_name().to_vec())) }
        MONSTER_TYPE => { let monster = game.find_region(region_id)?.base().find_monster_by_id(target.id)?; Some((monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?, monster.hit_points() == 0, monster.display_name().to_vec())) }
        _ => None,
    }
}

fn path_block(game: &CGame, region_id: i32, source_x: i32, source_y: i32, target_x: i32, target_y: i32, maximum: u32) -> Option<bool> {
    let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None); if maximum != 0 && maximum < path.len() as u32 { return None } Some(path.iter().any(|cell| cell.2 == 2))
}

fn send_failure(game: &CGame, player_id: i32, code: u8, amount: u32, text: Option<(&[u8], &[u8])>) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    if let Some((id, value)) = text { game.send_skill_system_info_with_text(player_id, id, value); return }
    match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount), 8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount), 10 => game.send_skill_system_info(player_id, b"GS0286"), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0292"), _ => {} }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8, target: Option<(ShapeIdentity, i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(action); message.add_long(LEAF_CUT_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); if action == 1 { message.add_long(player.shape().get_direction()); } else if let Some((target, x, y)) = target { message.add_long(target.object_type); message.add_long(target.id); message.add_long(x); message.add_long(y); } let _ = game.send_player_shape_around(player_id, None, &message);
}

fn master_info(player: &CPlayer) -> MasterInfo { let p = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate), permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) } }
pub(crate) fn leaf_cut_factors(factor: u32, weapon_level: u32, weapon_modifier: u32) -> (f32, f32) {
    let weapon_modifier = weapon_modifier as f32;
    let weapon_factor = (
        f64::from(weapon_level)
            * f64::from(weapon_modifier)
            * f64::from(0.01_f32)
    ) as f32;
    let damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    (damage_factor, weapon_factor)
}

pub(crate) fn execute_player_leaf_cut<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_leaf_cut_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) } let Some(target) = dispatch_target(dispatch) else { send_failure(game, player_id, 10, 0, None); return terminal(QueuedSkillExecutionState::Rejected) }; if target.object_type == PLAYER_TYPE && target.id == player_id { send_failure(game, player_id, 10, 0, None); return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, initial_mana, initial_rp)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(LEAF_CUT_SKILL_ID), player.mana(), player.rp()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some((target_x, target_y, target_dead, target_name)) = target_facts(game, region_id, target) else { send_failure(game, player_id, 10, 0, None); return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(LEAF_CUT_SKILL_ID, level) else { if ai.leaf_cut().is_some() { finish_player_leaf_cut(game, player_id, ai, runtime); } return terminal(QueuedSkillExecutionState::Rejected) }; let mp_loss = properties.query_property(USER_MP_LOSE); let rp_loss = properties.query_property(USER_RP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE); let keep = properties.query_property(STATE_PERSIST_TIME); let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY); let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let weapon_modifier = properties.query_property(WEAPON_DAMAGE_LEVEL_MODIFIER); let _hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.leaf_cut().is_none() { let now = runtime.now_milliseconds(); if ai.leaf_cut_last_used_ms() != 0 && now < ai.leaf_cut_last_used_ms().wrapping_add(reuse) { send_failure(game, player_id, 0x0d, 0, None); return terminal(QueuedSkillExecutionState::Rejected) } let Some(blocked) = path_block(game, region_id, source_x, source_y, target_x, target_y, maximum) else { send_failure(game, player_id, 0x0b, 0, None); return terminal(QueuedSkillExecutionState::Rejected) }; if blocked { send_failure(game, player_id, 0x0f, 0, Some((b"GS0291", &target_name))); return terminal(QueuedSkillExecutionState::Rejected) } let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !weapon_is_sword(game, player) { send_failure(game, player_id, 0x0e, 0, None); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss, None); return terminal(QueuedSkillExecutionState::Rejected) } if rp_loss != 0 && (u32::from(initial_rp).wrapping_sub(rp_loss) as i32) < 0 { send_failure(game, player_id, 8, rp_loss, None); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(LEAF_CUT_SKILL_ID)); } ai.begin_leaf_cut(SkillExecutionKernel::begin(dispatch, now)); }
    if target_dead { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    if ai.leaf_cut().is_some_and(|state| state.stage() == SkillStage::Begin) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss, None); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let rp = game.find_player(player_id).map_or(0, CPlayer::rp); if (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 { send_failure(game, player_id, 8, rp_loss, None); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_rp(u32::from(rp).wrapping_sub(rp_loss) as u16); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if game.find_player(player_id).is_none_or(|player| !weapon_is_sword(game, player)) { send_failure(game, player_id, 0x0e, 0, None); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) } let direction = get_line_direction(source_x, source_y, target_x, target_y); if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(direction); } send_visual(game, player_id, level, 1, None); if let Some(state) = ai.leaf_cut_mut() { let _ = state.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.leaf_cut().map(SkillExecutionKernel::started_at_ms).unwrap_or_default(); if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    let Some((live_x, live_y, live_dead, live_name)) = target_facts(game, region_id, target) else { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }; if live_dead { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) } let Some(blocked) = path_block(game, region_id, source_x, source_y, live_x, live_y, maximum) else { send_failure(game, player_id, 0x0b, 0, None); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }; if blocked { send_failure(game, player_id, 0x0f, 0, Some((b"GS0307", &live_name))); finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
    send_visual(game, player_id, level, 2, Some((target, live_x, live_y))); if target.object_type == PLAYER_TYPE { let _ = game.player_on_first_skill(player_id, target.id, Some(region_id), runtime); }
    let Some((master, combat, weapon_level)) = game.find_player(player_id).map(|player| (master_info(player), player.combat_properties(), player.weapon_damage_level(game.goods_factory()))) else { finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }; let now = runtime.now_milliseconds(); let (damage_factor, weapon_factor) = leaf_cut_factors(factor, weapon_level as u32, weapon_modifier); let state = LeafCutState::new(master, now, keep, frequency, damage_factor, weapon_factor, combat.minimum_attack as u16, combat.maximum_attack as u16, combat.add_element_attack as u16, combat.add_soul_attack, );
    let Some((previous, identity, x, y)) = game.replace_leaf_cut_state(region_id, target, state, now) else { finish_player_leaf_cut(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }; if let Some(previous) = previous { send_leaf_cut_state_visual(game, region_id, identity, x, y, previous, false, now); } send_leaf_cut_state_visual(game, region_id, identity, x, y, state, true, now); if let Some(kernel) = ai.leaf_cut_mut() { let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack); let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_leaf_cut(game, player_id, ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
