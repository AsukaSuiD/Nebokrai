//! Семейство армейского удара `CArmyBreak` (`0x68/0x7B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/armybreak*.cpp`. Оба варианта требуют меч категории 1,
//! сохраняют частичную трату MP перед RP и повторной проверкой оружия, затем
//! единожды поражают каждую допустимую цель полного scope 5×5. Клетка перед
//! игроком использует major factor, остальные — minor; обход остаётся
//! `x → y → региональный порядок`, а формула делает ровно два RNG-вызова.
//! Оба варианта завершаются одинаково: сбрасывают сохранённое направление,
//! возвращают движение и выполняют `CSummonSkill::End(1)` с единичным
//! оружейным `AfterUseSkill`; обход целей сам оружие не изнашивает.
//! Damage factor сохраняет x87-порядок `u32 factor × f32 weapon × 0.01_f32`
//! и округляется в `f32` только при записи. Критический множитель не округляет
//! исходный урон в `f32` и усекается к нулю лишь при итоговой записи `int`.

use super::armybreak2::ARMY_BREAK_2_SKILL_ID;
use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const ARMY_BREAK_SKILL_ID: u32 = 0x68;
const PLAYER_TYPE: i32 = 400; const MONSTER_TYPE: i32 = 600; const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2; const USER_RP_LOSE: u32 = 3;
const MINOR_DAMAGE_FACTOR: u32 = 20_011; const MAJOR_DAMAGE_FACTOR: u32 = 20_012;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmyBreakExecutionState { kernel: SkillExecutionKernel<PlayerSkillDispatch>, direction: i32 }
impl ArmyBreakExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), direction: -1 } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id } }
pub(crate) fn is_army_break_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(skill_id(dispatch), ARMY_BREAK_SKILL_ID | ARMY_BREAK_2_SKILL_ID) }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn finish_player_army_break<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, skill_id: u32, runtime: &mut Runtime) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| { player_ai.mark_army_break_used(skill_id, now_ms); }); }
pub(crate) fn cancel_player_army_break<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = player_ai.army_break().map(|state| state.kernel().dispatch()) else { return false }; finish_player_army_break(game, player_id, player_ai, skill_id(dispatch), runtime); player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }

fn master(player: &CPlayer) -> MasterInfo { let p = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate), permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) } }
fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1) }
fn send_failure(game: &CGame, player_id: i32, code: u8, amount: u32) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code); match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount), 8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0287"), _ => {} } }
fn destination(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> { match dispatch { PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)), PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(|player| { let face = player.shape().get_face_position().ok()?; Some((face.x, face.y)) }) } }

fn send_visual(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, direction: i32, apply: bool) {
    let Some(shape) = game.find_player(player_id).map(|player| player.shape().clone()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(if apply { 2 } else { 1 }); message.add_long(skill_id as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if apply { let position = ShapeAreaCoordinates { x: shape.get_tile_x().unwrap_or_default(), y: shape.get_tile_y().unwrap_or_default() }; let front = CShape::get_direction_position(direction, position).unwrap_or(position); message.add_long(0); message.add_long(0); message.add_long(front.x); message.add_long(front.y); } else { message.add_long(direction); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn targets(game: &CGame, region_id: i32, player_id: i32, direction: i32) -> Vec<(ShapeIdentity, bool)> {
    let Some(shape) = game.find_player(player_id).map(|player| player.shape().clone()) else { return Vec::new() }; let position = ShapeAreaCoordinates { x: shape.get_tile_x().unwrap_or_default(), y: shape.get_tile_y().unwrap_or_default() }; let Ok(front) = CShape::get_direction_position(direction, position) else { return Vec::new() };
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() }; let (aw, ah) = game.area_dimensions(); let mut seen = Vec::new(); let mut result = Vec::new();
    for ox in 0..5 { for oy in 0..5 { let x = front.x.wrapping_sub(2).wrapping_add(ox); let y = front.y.wrapping_sub(2).wrapping_add(oy); let mut shapes = Vec::new(); if region.get_shapes(x, y, aw, ah, game, &mut shapes).is_err() { continue } for view in shapes { let identity = view.identity; if identity.object_type != PLAYER_TYPE && identity.object_type != MONSTER_TYPE || identity.object_type == PLAYER_TYPE && identity.id == player_id || seen.contains(&identity) { continue } seen.push(identity); result.push((identity, view.tile_x == front.x && view.tile_y == front.y)); } } }
    result
}

fn target_level(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<u8> { match identity.object_type { PLAYER_TYPE => game.find_player(identity.id).map(CPlayer::level), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(identity.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }), _ => None } }
fn calculate(game: &mut CGame, player_id: i32, target_level: u8, skill_id: u32, level: i32, factor: u32, hit_modifier: i32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let owner = master(player); let (divisor, floor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor);
    let difference = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32); let width = (if difference < 0 { difference.wrapping_neg() } else { difference }).wrapping_add(1); let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let damage_factor = (f64::from(factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: owner.master_team_id, attacker_faction_id: owner.master_guild_id, attacker_union_id: owner.master_union_id, hit_modifier, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for damage in &mut attack.damages { damage.hp_damage = truncate_original(f64::from(damage.hp_damage) * f64::from(rate)); } } Some((owner, attack))
}

pub(crate) fn execute_player_army_break<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_army_break_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) } let skill_id = skill_id(dispatch);
    let Some((region_id, level, sx, sy, mana, rp)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(skill_id), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana(), player.rp()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(skill_id, level) else { if ai.army_break().is_some() { finish_player_army_break(game, player_id, ai, skill_id, runtime); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let rp_loss = properties.query_property(USER_RP_LOSE); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let minor = properties.query_property(MINOR_DAMAGE_FACTOR); let major = properties.query_property(MAJOR_DAMAGE_FACTOR); let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.army_break().is_none() { let now = runtime.now_milliseconds(); if ai.army_break_last_used_ms(skill_id) != 0 && !time_reached(now, ai.army_break_last_used_ms(skill_id), reuse) { send_failure(game, player_id, 0x0d, 0); return terminal(QueuedSkillExecutionState::Rejected) } let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !weapon_is_sword(game, player) { send_failure(game, player_id, 0x0e, 0); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if rp_loss != 0 && (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 { send_failure(game, player_id, 8, rp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id)); } ai.begin_army_break(ArmyBreakExecutionState::begin(dispatch, now)); } else if ai.army_break().is_none_or(|state| state.kernel().dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if ai.army_break().is_some_and(|state| state.kernel().stage() == SkillStage::Begin) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); finish_player_army_break(game, player_id, ai, skill_id, runtime); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let rp = game.find_player(player_id).map_or(0, CPlayer::rp); if (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 { send_failure(game, player_id, 8, rp_loss); finish_player_army_break(game, player_id, ai, skill_id, runtime); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_rp(rp.wrapping_sub(rp_loss as u16)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if game.find_player(player_id).is_none_or(|player| !weapon_is_sword(game, player)) { send_failure(game, player_id, 0x0e, 0); finish_player_army_break(game, player_id, ai, skill_id, runtime); return terminal(QueuedSkillExecutionState::Rejected) } let (tx, ty) = destination(game, region_id, player_id, dispatch).unwrap_or((sx, sy)); let direction = get_line_direction(sx, sy, tx, ty); if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(direction); } if let Some(state) = ai.army_break_mut() { state.direction = direction; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); } send_visual(game, player_id, skill_id, level, direction, false); }
    let Some(state) = ai.army_break() else { return terminal(QueuedSkillExecutionState::Rejected) }; if !time_reached(runtime.now_milliseconds(), state.kernel().started_at_ms(), delay) { return terminal(QueuedSkillExecutionState::Pending) } let direction = state.direction; send_visual(game, player_id, skill_id, level, direction, true); if let Some(state) = ai.army_break_mut() { let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); }
    let owner = game.find_player(player_id).map(master).unwrap_or_default(); for (identity, central) in targets(game, region_id, player_id, direction) { if !game.owned_player_skill_target_attackable(owner, identity, region_id) { continue } let Some(target_level) = target_level(game, region_id, identity) else { continue }; let Some((owner, attack)) = calculate(game, player_id, target_level, skill_id, level, if central { major } else { minor }, hit) else { continue }; match identity.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(owner, identity.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(owner, identity.id, region_id, attack, runtime), _ => continue } }
    if let Some(state) = ai.army_break_mut() { let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); } finish_player_army_break(game, player_id, ai, skill_id, runtime); terminal(QueuedSkillExecutionState::Completed)
}
