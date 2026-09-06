//! Двойной направленный удар `CSwallow` (`0x6A`).
//! Успешный Begin возвращает Begun до первого AI. Расход ресурсов,
//! перемещение и атака остаются у AI после постановки Attack в том же Run;
//! раннее время Begin сохраняется общим kernel.
//! Reuse проверяется exact `CSkill::IsRestored`; оба attack interval — elapsed.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/swallow.cpp`. Владелец сохраняет повторную проверку меча,
//! необратимое списание MP до этой проверки, две атаки через `delay` и
//! `delay + action interval`, региональный порядок целей и отдельную
//! дедупликацию каждого прохода. Восемь матриц `3×3` восстановлены из `.data`;
//! уровни `1`, `2` и остальные используют одинаковые байты. `CGame` только
//! разрешает цели, применяет рассчитанный урон и выполняет доставку. `Attack`
//! и `AI` не изнашивают оружие на каждой цели двух проходов: унаследованный
//! `AfterUseSkill` делает это один раз в подтверждённом `End(1)`, после сброса
//! сохранённого направления и возврата движения.
//! Беззнаковый коэффициент урона проходит исходную x87-цепочку до единственной
//! записи в `float`, а критический множитель переводится в `int` с усечением
//! к нулю отдельно для каждого боевого компонента.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
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

pub(crate) const SWALLOW_SKILL_ID: u32 = 0x6a;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const ACTION_INTERVAL: u32 = 10_009;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

// Оригинал обходит X снаружи, Y внутри, но индексирует `x + 3 * y`.
const DIRECTIONAL_SCOPE: [[u8; 9]; 8] = [
    [1, 1, 1, 0, 0, 0, 0, 0, 0], [0, 1, 1, 0, 0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0, 1, 0, 0, 1], [0, 0, 0, 0, 0, 1, 0, 1, 1],
    [0, 0, 0, 0, 0, 0, 1, 1, 1], [0, 0, 0, 1, 0, 0, 1, 1, 0],
    [1, 0, 0, 1, 0, 0, 1, 0, 0], [1, 1, 0, 1, 0, 0, 0, 0, 0],
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwallowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    first_attack_done: bool,
    direction: i32,
}
impl SwallowExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now), condition_checked: false, first_attack_done: false, direction: -1 } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id } }
pub(crate) fn is_swallow_dispatch(dispatch: PlayerSkillDispatch) -> bool { skill_id(dispatch) == SWALLOW_SKILL_ID }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn finish_player_swallow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| { player_ai.mark_skill_used(SWALLOW_SKILL_ID, now_ms); }); }
pub(crate) fn cancel_player_swallow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = player_ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false }; finish_player_swallow(game, player_id, player_ai, runtime); player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }
fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2) }
fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0292"), _ => {} }
}
fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8, direction: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(action); message.add_long(SWALLOW_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id);
    if action == 1 { message.add_long(direction); } else { let origin = ShapeAreaCoordinates { x: player.shape().get_tile_x().unwrap_or_default(), y: player.shape().get_tile_y().unwrap_or_default() }; let destination = CShape::get_direction_position(direction, origin).unwrap_or(origin); message.add_long(0); message.add_long(0); message.add_long(destination.x); message.add_long(destination.y); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}
fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch { PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y)), PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)) }
}
fn master_info(player: &CPlayer) -> MasterInfo { let p = player.pk_permissions(); MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate), permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal) } }
fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> { match target.object_type { PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }), _ => None } }
fn calculate_attack(game: &mut CGame, player_id: i32, target_level: u8, level: i32, hit: i32, factor: u32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player); let (divisor, floor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor); let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_abs().wrapping_add(1); let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let damage_factor =
        (f64::from(factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id: SWALLOW_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } } Some((master, attack))
}
fn cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> { let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() }; let (width, height) = game.area_dimensions(); let mut shapes = Vec::new(); if region.get_shapes(x, y, width, height, game, &mut shapes).is_err() { return Vec::new() } shapes.into_iter().map(|shape| shape.identity).collect() }
fn attack_scope<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, direction: i32, level: i32, hit: i32, factor: u32, runtime: &mut Runtime) {
    let Some((source_x, source_y, master)) = game.find_player(player_id).and_then(|player| Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, master_info(player)))) else { return }; let Some(scope) = usize::try_from(direction).ok().and_then(|index| DIRECTIONAL_SCOPE.get(index)) else { return }; let mut attacked = Vec::new();
    for x_offset in 0..3usize { for y_offset in 0..3usize { if scope[x_offset + 3 * y_offset] == 0 { continue } let x = source_x.wrapping_sub(1).wrapping_add(x_offset as i32); let y = source_y.wrapping_sub(1).wrapping_add(y_offset as i32); for target in cell_targets(game, region_id, x, y) { if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || attacked.contains(&target) { continue } attacked.push(target); if !game.owned_player_skill_target_attackable(master, target, region_id) { continue } let Some(target_level) = target_level(game, region_id, target) else { continue }; let Some((master, attack)) = calculate_attack(game, player_id, target_level, level, hit, factor) else { continue }; match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => continue } } } }
}

pub(crate) fn execute_player_swallow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_swallow_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(SWALLOW_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(SWALLOW_SKILL_ID, level) else { if ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).is_some() { finish_player_swallow(game, player_id, ai, runtime); } return terminal(QueuedSkillExecutionState::Rejected) }; let mp_loss = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let interval = properties.query_property(ACTION_INTERVAL); let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).is_none() { let now = runtime.now_milliseconds(); if !skill_is_restored(ai.skill_last_used_ms(SWALLOW_SKILL_ID), reuse, now) { send_failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !weapon_is_sword(game, player) { send_failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) } if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(SWALLOW_SKILL_ID)); } ai.begin_player_skill_execution(SwallowExecutionState::begin(dispatch, now)); return terminal(QueuedSkillExecutionState::Begun); } else if ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).is_some_and(|state| !state.condition_checked) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); finish_player_swallow(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if game.find_player(player_id).is_none_or(|player| !weapon_is_sword(game, player)) { send_failure(game, player_id, 0x0e, mp_loss); finish_player_swallow(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) } let Some((target_x, target_y)) = target_position(game, region_id, player_id, dispatch) else { finish_player_swallow(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }; let direction = get_line_direction(source_x, source_y, target_x, target_y); if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(direction); } send_visual(game, player_id, level, 1, direction); if let Some(state) = ai.player_skill_state_mut::<SwallowExecutionState>(SWALLOW_SKILL_ID) { state.condition_checked = true; state.direction = direction; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).map(|state| state.kernel.started_at_ms()).unwrap_or_default(); if !ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).is_some_and(|state| state.first_attack_done) { if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending) } let direction = ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).map_or(-1, |state| state.direction); send_visual(game, player_id, level, 2, direction); attack_scope(game, player_id, region_id, direction, level, hit, factor, runtime); if let Some(state) = ai.player_skill_state_mut::<SwallowExecutionState>(SWALLOW_SKILL_ID) { state.first_attack_done = true; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); } }
    if runtime.now_milliseconds() < started.wrapping_add(delay).wrapping_add(interval) { return terminal(QueuedSkillExecutionState::Pending) } let direction = ai.player_skill_state::<SwallowExecutionState>(SWALLOW_SKILL_ID).map_or(-1, |state| state.direction); attack_scope(game, player_id, region_id, direction, level, hit, factor, runtime); if let Some(state) = ai.player_skill_state_mut::<SwallowExecutionState>(SWALLOW_SKILL_ID) { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_swallow(game, player_id, ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
