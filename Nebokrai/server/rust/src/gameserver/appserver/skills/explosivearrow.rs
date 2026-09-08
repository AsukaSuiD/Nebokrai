//! Семейство проникающих стрел `CExplosiveArrow` (`0x32`, `0x3D`, `0x3E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/explosivearrow{,2,3}.cpp`. Все три варианта сохраняют
//! общий отложенный полёт по прямому пути, маску `3×3`, уникальность обычных
//! целей и отбрасывание стрелка после завершения пути. Второй вариант внутри
//! каждой из девяти клеток маски повторно обходит вынесенные боевые души и не
//! включает их в список обычных целей — это исходное наблюдаемое отличие.
//! Формулы, два RNG-вызова на каждое фактическое попадание и пакеты остаются у
//! владельца семейства; `CGame` только разрешает владельцев, применяет готовую
//! атаку, перемещает стрелка и выполняет доставку.
//! Ни одна из шести перегрузок `Attack` не изнашивает оружие на отдельных
//! целях: общий унаследованный `AfterUseSkill` делает это один раз из
//! `End(true)`, после чего обновляются свойства и cooldown. `End(false)` только
//! освобождает runtime-состояние и не откатывает попадания.
//! Все три варианта сохраняют x87-порядок damage factor: точный `u32 factor`,
//! `f32 weapon` и `0.01_f32` округляются только итоговой записью в `f32`.
//! Critical не округляет исходный `i32` в `f32` и усекается лишь перед `int`.
//! Cooldown использует абсолютный срок `CSkill::IsRestored`; полёт остаётся elapsed.
//! Физический RNG получает исходную DWORD-ширину `maximum - minimum + 1`
//! без нормализации перевёрнутых границ; результат затем складывается с
//! минимумом с wrapping-семантикой и ограничивается снизу нулём.
//! Выпуск устанавливает общий prepared-флаг после эффекта 1
//! (варианты 1/2/3: 0x00578B18 / 0x00558578 / 0x00556778). Последующий AI продолжает тот же
//! экземпляр в фоне; повторный Begin и отдельное хранилище не создаются.
//! Успешный Begin возвращает Begun после инициализации исполнения. Первый
//! AI выполняет повторные проверки и эффекты отдельно, в том же Run после
//! постановки Attack; раннее время Begin сохраняется общим kernel.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::poisonmoth::{MONSTER_TYPE, PLAYER_TYPE, cell_targets, master_info, target_level, target_position};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const EXPLOSIVE_ARROW_SKILL_ID: u32 = 0x32;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const ADDITION_ELEMENT_ATTACK: u32 = 20_013;
const TARGET_BACK_STEP: u32 = 30_002;
const TARGET_MOVE_SPEED: u32 = 30_003;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplosiveArrowVariant { Bow, WarSoul, Crossbow }

impl ExplosiveArrowVariant {
    pub(crate) const fn skill_id(self) -> u32 { match self { Self::Bow => EXPLOSIVE_ARROW_SKILL_ID, Self::WarSoul => super::explosivearrow2::EXPLOSIVE_ARROW_2_SKILL_ID, Self::Crossbow => super::explosivearrow3::EXPLOSIVE_ARROW_3_SKILL_ID } }
    const fn weapon_category(self) -> i32 { match self { Self::Bow => 4, Self::WarSoul | Self::Crossbow => 3 } }
    const fn wrong_weapon_message(self) -> &'static [u8] { match self { Self::Bow => b"GS0293", Self::WarSoul | Self::Crossbow => b"GS0297" } }
    const fn attacks_war_soul(self) -> bool { matches!(self, Self::WarSoul) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplosiveArrowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>, variant: ExplosiveArrowVariant,
    condition_checked: bool, path: Vec<(i32, i32, u8)>,
    current_position: usize, destination: (i32, i32), end_tile: (i32, i32),
    visual_target: Option<ShapeIdentity>, attacked: Vec<ShapeIdentity>, end_sent: bool,
}

impl ExplosiveArrowExecutionState {
    fn begin(variant: ExplosiveArrowVariant, dispatch: PlayerSkillDispatch, started_at_ms: u32, destination: (i32, i32)) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), variant, condition_checked: false, path: Vec::new(), current_position: 0, destination, end_tile: (0, 0), visual_target: None, attacked: Vec::new(), end_sent: false } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) const fn explosive_arrow_variant(dispatch: PlayerSkillDispatch) -> Option<ExplosiveArrowVariant> { let skill_id = match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id }; match skill_id { EXPLOSIVE_ARROW_SKILL_ID => Some(ExplosiveArrowVariant::Bow), super::explosivearrow2::EXPLOSIVE_ARROW_2_SKILL_ID => Some(ExplosiveArrowVariant::WarSoul), super::explosivearrow3::EXPLOSIVE_ARROW_3_SKILL_ID => Some(ExplosiveArrowVariant::Crossbow), _ => None } }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_explosive_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _ai: &mut CPlayerAI, variant: ExplosiveArrowVariant, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, variant.skill_id(), runtime);
}
fn abort_player_explosive_arrow(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }
pub(crate) fn complete_player_explosive_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, execution_skill_id: u32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some((dispatch, variant)) = game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, execution_skill_id).map(|state| (state.kernel().dispatch(), state.variant)) else { return false };
    finish_player_explosive_arrow(game, player_id, ai, variant, runtime);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_explosive_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, execution_skill_id: u32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, execution_skill_id).map(|state| state.kernel().dispatch()) else { return false };
    abort_player_explosive_arrow(game, player_id);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}
fn required_weapon_failure(game: &CGame, player: &CPlayer, variant: ExplosiveArrowVariant) -> Option<&'static [u8]> { let Some(weapon) = player.equipment().get_goods(2) else { return Some(b"GS0293") }; (weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) != variant.weapon_category()).then(|| variant.wrong_weapon_message()) }
fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32, text: Option<&[u8]>) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code); if let Some(text) = text { if code == 7 { game.send_skill_system_info_with_unsigned(player_id, text, mp_loss); } else { game.send_skill_system_info(player_id, text); } } }
fn send_start(game: &mut CGame, player_id: i32, skill_id: u32, level: i32) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(1); message.add_long(skill_id as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_fire(game: &mut CGame, region_id: i32, player_id: i32, skill_id: u32, level: i32, dispatch: PlayerSkillDispatch, destination: (i32, i32), flying_time: u32) { let live = match dispatch { PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (target, view.tile_x, view.tile_y)), _ => None }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(2); message.add_long(skill_id as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(live.map_or(0, |value| value.0.object_type)); message.add_long(live.map_or(0, |value| value.0.id)); message.add_long(live.map_or(destination.0, |value| value.1)); message.add_long(live.map_or(destination.1, |value| value.2)); message.add_ulong(flying_time); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_end(game: &mut CGame, player_id: i32, skill_id: u32, level: i32, end_tile: (i32, i32), target: Option<ShapeIdentity>) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(3); message.add_long(skill_id as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); message.add_long(end_tile.0); message.add_long(end_tile.1); message.add_long(target.map_or(0, |value| value.object_type)); message.add_long(target.map_or(0, |value| value.id)); let _ = game.send_player_shape_around(player_id, None, &message); }

fn calculate_attack(game: &mut CGame, player_id: i32, target_level: u8, skill_id: u32, level: i32, factor: u32, hit: i32, element_addition: u32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player); let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor); let minimum = combat.minimum_attack as i32; let width = (combat.maximum_attack as i32).wrapping_sub(minimum).wrapping_add(1); let physical = minimum.wrapping_add(game.skill_random_below(width)).max(0);
    let damage_factor = (f64::from(factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).wrapping_add(element_addition as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } } Some((master, attack))
}

#[allow(clippy::too_many_arguments, reason = "граница буквально сохраняет параметры одного попадания EXE")]
fn apply_target<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, target: ShapeIdentity, war_soul: bool, skill_id: u32, level: i32, factor: u32, hit: i32, element_addition: u32, runtime: &mut Runtime) {
    let Some(target_level) = target_level(game, region_id, target) else { return }; let Some((master, attack)) = calculate_attack(game, player_id, target_level, skill_id, level, factor, hit, element_addition) else { return }; match target.object_type { PLAYER_TYPE if war_soul => game.apply_owned_skill_attack_to_war_soul(master, target.id, region_id, attack, runtime), PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => return }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок маски, прохода боевых душ и обычных целей")]
fn attack_scope<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, variant: ExplosiveArrowVariant, level: i32, factor: u32, hit: i32, element_addition: u32, center: (i32, i32), attacked: &mut Vec<ShapeIdentity>, runtime: &mut Runtime) -> (bool, Option<ShapeIdentity>) {
    let Some(master) = game.find_player(player_id).map(master_info) else { return (false, None) }; let mut any = false; let mut visual = None;
    for offset_x in -1..=1 { for offset_y in -1..=1 {
        if variant.attacks_war_soul() { let war_souls = game.find_region(region_id).map(|owner| owner.base().war_souls_at(center.0, center.1).into_keys().collect::<Vec<_>>()).unwrap_or_default(); for target_id in war_souls { let target = ShapeIdentity { object_type: PLAYER_TYPE, id: target_id as i32, ex_id: Default::default() }; if target.id == player_id || attacked.contains(&target) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue } apply_target(game, player_id, region_id, target, true, variant.skill_id(), level, factor, hit, element_addition, runtime); any = true; } }
        let x = center.0.wrapping_add(offset_x); let y = center.1.wrapping_add(offset_y); for target in cell_targets(game, region_id, x, y) { if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || attacked.contains(&target) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue } if x == center.0 && y != 0 && visual.is_none() { visual = Some(target); } attacked.push(target); apply_target(game, player_id, region_id, target, false, variant.skill_id(), level, factor, hit, element_addition, runtime); any = true; }
    } }
    (any, visual)
}

fn knock_back_owner(game: &mut CGame, player_id: i32, region_id: i32, back_steps: u32, move_speed: u32) {
    let Some((direction, mut position)) = game.find_player(player_id).and_then(|player| Some(((player.shape().get_direction().wrapping_add(4)) & 7, ShapeAreaCoordinates { x: player.shape().get_tile_x().ok()?, y: player.shape().get_tile_y().ok()? }))) else { return }; let original = position; let mut moved = 0; while moved < back_steps { let Ok(next) = CShape::get_direction_position(direction, position) else { break }; let blocked = game.find_region(region_id).is_none_or(|owner| owner.base().region.get_block(next.x, next.y).map_or(true, |block| block != 0)); if blocked { break } position = next; moved = moved.wrapping_add(1); } if position != original { let _ = game.force_move_player(player_id, position.x, position.y, move_speed.wrapping_mul(moved)); }
}

pub(crate) fn execute_player_explosive_arrow<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let Some(variant) = explosive_arrow_variant(dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) }; let skill_id = variant.skill_id(); let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(skill_id, game.skill_factory()), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(skill_id, level) else { if game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).is_some() { abort_player_explosive_arrow(game, player_id); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE); let missile_step = properties.query_property(MISSILE_FLYING_TIME); let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let element_addition = properties.query_property(ADDITION_ELEMENT_ATTACK); let back_steps = properties.query_property(TARGET_BACK_STEP); let move_speed = properties.query_property(TARGET_MOVE_SPEED); let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).is_none() { let now = runtime.now_milliseconds(); let self_target = matches!(dispatch, PlayerSkillDispatch::SelfTarget { .. }) || matches!(dispatch, PlayerSkillDispatch::Object { target: ShapeIdentity { object_type: PLAYER_TYPE, id, .. }, .. } if id == player_id); if self_target { send_failure(game, player_id, 10, mp_loss, Some(b"GS0286")); return terminal(QueuedSkillExecutionState::Rejected) } if !skill_is_restored(game.player_skill_last_used_ms(player_id, variant.skill_id()), reuse, now) { send_failure(game, player_id, 0x0d, mp_loss, Some(b"GS0278")); return terminal(QueuedSkillExecutionState::Rejected) } let Some(destination) = target_position(game, region_id, player_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if maximum_distance != 0 && path.len() as u32 > maximum_distance { send_failure(game, player_id, 0x0b, mp_loss, Some(b"GS0290")); return terminal(QueuedSkillExecutionState::Rejected) } let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if let Some(text) = required_weapon_failure(game, player, variant) { send_failure(game, player_id, 0x0e, mp_loss, Some(text)); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss, Some(b"GS0288")); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id)); } game.begin_player_skill_execution(player_id, ExplosiveArrowExecutionState::begin(variant, dispatch, now, destination)); return terminal(QueuedSkillExecutionState::Begun); }
    else if game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).is_none_or(|state| state.kernel.dispatch() != dispatch || state.variant != variant) { return terminal(QueuedSkillExecutionState::Rejected) }
    if game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).is_some_and(|state| !state.condition_checked) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss, Some(b"GS0288")); abort_player_explosive_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); let late_failure = game.find_player(player_id).and_then(|player| required_weapon_failure(game, player, variant)); if let Some(text) = late_failure { send_failure(game, player_id, 0x0e, mp_loss, Some(text)); abort_player_explosive_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } let destination = game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).map(|state| state.destination).unwrap_or_default(); if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } send_start(game, player_id, skill_id, level); if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).map(|state| state.kernel.started_at_ms()).unwrap_or_default(); if !game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).is_some_and(|state| state.kernel().is_prepared()) { if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } let Some(destination) = target_position(game, region_id, player_id, dispatch) else { abort_player_explosive_arrow(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, (maximum_distance != 0).then_some(maximum_distance)); let index = path.iter().position(|cell| cell.2 == BLOCK_UNFLY).unwrap_or(path.len()); let endpoint = path.get(index).or_else(|| path.last()).copied().unwrap_or((destination.0, destination.1, BLOCK_UNFLY)); send_fire(game, region_id, player_id, skill_id, level, dispatch, destination, missile_step.wrapping_mul(index as u32)); if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.path = path; state.current_position = 1; state.end_tile = (endpoint.0, endpoint.1); state.kernel_mut().mark_prepared(); let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); } }
    let Some((position, path_len, cell, end_sent)) = game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).map(|state| (state.current_position, state.path.len(), state.path.get(state.current_position).copied(), state.end_sent)) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(missile_step.wrapping_mul(position as u32))) { return terminal(QueuedSkillExecutionState::Pending) }
    let Some((x, y, _)) = cell else { knock_back_owner(game, player_id, region_id, back_steps, move_speed); if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_explosive_arrow(game, player_id, ai, variant, runtime); return terminal(QueuedSkillExecutionState::Completed) }; let live_block = game.find_region(region_id).map_or(BLOCK_UNFLY, |owner| owner.base().skill_cell_block(x, y)); if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.end_tile = (x, y) }
    if live_block == BLOCK_SHAPE { let mut attacked = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).map(|state| std::mem::take(&mut state.attacked)).unwrap_or_default(); let (any, visual) = attack_scope(game, player_id, region_id, variant, level, factor, hit, element_addition, (x, y), &mut attacked, runtime); if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.attacked = attacked; if state.visual_target.is_none() { state.visual_target = visual; } } if any { let target = game.player_skill_state::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()).and_then(|state| state.visual_target); send_end(game, player_id, skill_id, level, (x, y), target); if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.end_sent = true; state.current_position = path_len.wrapping_add(1); } return terminal(QueuedSkillExecutionState::Pending) } }
    else if live_block == BLOCK_UNFLY { if !end_sent { send_end(game, player_id, skill_id, level, (x, y), None); } if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.end_sent = true; state.current_position = path_len; } }
    if let Some(state) = game.player_skill_state_mut::<ExplosiveArrowExecutionState>(player_id, dispatch.skill_id()) { state.current_position = state.current_position.wrapping_add(1); } terminal(QueuedSkillExecutionState::Pending)
}
