//! Клеточный полёт `CLightingArrow2` (`0xE7`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lightingarrow2.cpp`. Навык после общей задержки сохраняет
//! путь и обрабатывает не более одной клетки за проход `CPlayerAI`. Первая
//! непролётная клетка не атакуется; уже задетая фигура повторно не выбирается.
//! Урон использует текущие свойства стрелка и ровно два собственных RNG-вызова,
//! а активный яд оружия применяется после основного `OnBeenAttacked`. Общий
//! `End(true)` прекращает оставшиеся клетки и фиксирует cooldown, тогда как
//! внутренний `End(false)` сохраняет уже нанесённые необратимые эффекты, но не
//! запускает новые клетки и не фиксирует cooldown.
//! Беззнаковый коэффициент урона остаётся в расширенной точности x87 до
//! единственной записи в `float`; критический урон также вычисляется в x87 и
//! усекается к нулю при записи в `i32`. Cooldown использует абсолютный срок
//! `CSkill::IsRestored`; поклеточные сроки полёта остаются elapsed.
//! Выпуск устанавливает общий prepared-флаг после эффекта 1
//! (0x0054D4A9). Последующий AI продолжает тот же
//! экземпляр в фоне; повторный Begin и отдельное хранилище не создаются.
//! Успешный Begin возвращает Begun после инициализации исполнения. Первый
//! AI выполняет повторные проверки и эффекты отдельно, в том же Run после
//! постановки Attack; раннее время Begin сохраняется общим kernel.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::heartlessarrow::apply_daub_poison;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::poisonmoth::{cell_targets, master_info, target_level, target_position};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const LIGHTING_ARROW_2_SKILL_ID: u32 = 0xe7;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LightingArrow2ExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination: (i32, i32),
    condition_checked: bool,
    path: Vec<(i32, i32, u8)>,
    attack_cell_count: usize,
    current_cell: usize,
    attacked_creatures: Vec<ShapeIdentity>,
}

impl LightingArrow2ExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination, condition_checked: false, path: Vec::new(), attack_cell_count: 0, current_cell: 0, attacked_creatures: Vec::new() }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_lighting_arrow_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, LIGHTING_ARROW_2_SKILL_ID, runtime);
}
fn abort_player_lighting_arrow_2(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }
pub(crate) fn complete_player_lighting_arrow_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_lighting_arrow_2(game, player_id, ai, runtime);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_lighting_arrow_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    abort_player_lighting_arrow_2(game, player_id);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}
fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3) }
fn target_is_dead(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } if target.object_type == PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        PlayerSkillDispatch::Object { target, .. } if target.object_type == MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).is_none_or(|target: &CMonster| target.hit_points() == 0),
        PlayerSkillDispatch::Object { .. } => true,
        _ => false,
    }
}
fn failure(game: &CGame, player_id: i32, action: u8, mp_loss: u32) {
    game.send_base_magic_failure(player_id, action);
    match action { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0297"), 10 => game.send_skill_system_info(player_id, b"GS0285"), _ => {} }
}
fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE); message.add_byte(1); message.add_long(LIGHTING_ARROW_2_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_broken(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(3);
    message.add_long(LIGHTING_ARROW_2_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_fire(game: &mut CGame, player_id: i32, level: i32, dispatch: PlayerSkillDispatch, destination: (i32, i32), missile_time_ms: u32) {
    let target = match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None }; let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE); message.add_byte(2); message.add_long(LIGHTING_ARROW_2_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(target.map_or(0, |value| value.object_type)); message.add_long(target.map_or(0, |value| value.id)); message.add_long(destination.0); message.add_long(destination.1); message.add_ulong(missile_time_ms); let _ = game.send_player_shape_around(player_id, None, &message);
}
pub(crate) const fn is_lighting_arrow_2_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Point { skill_id: LIGHTING_ARROW_2_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: LIGHTING_ARROW_2_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }) }

fn calculate_attack(game: &mut CGame, player_id: i32, target_level: u8, level: i32, factor: u32, hit: i32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player); let (divisor, floor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor); let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_add(1); let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let damage_factor =
        (f64::from(factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id: LIGHTING_ARROW_2_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } }
    Some((master, attack))
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют входы исходной клеточной атаки")]
fn attack_cell<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, level: i32, factor: u32, hit: i32, x: i32, y: i32, attacked: &mut Vec<ShapeIdentity>, runtime: &mut Runtime) {
    if x == 0 && y == 0 { return } let Some(master) = game.find_player(player_id).map(master_info) else { return };
    for target in cell_targets(game, region_id, x, y) {
        if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || attacked.contains(&target) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue }
        attacked.push(target); let Some(target_level) = target_level(game, region_id, target) else { continue }; let Some((master, attack)) = calculate_attack(game, player_id, target_level, level, factor, hit) else { continue };
        match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} }
        apply_daub_poison(game, player_id, region_id, target, runtime.now_milliseconds());
    }
}

pub(crate) fn execute_player_lighting_arrow_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_lighting_arrow_2_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(LIGHTING_ARROW_2_SKILL_ID, game.skill_factory()), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(LIGHTING_ARROW_2_SKILL_ID, level) else { if game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).is_some() { abort_player_lighting_arrow_2(game, player_id) } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum = properties.query_property(TARGET_MAX_DISTANCE); let missile_time = properties.query_property(MISSILE_FLYING_TIME); let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).is_none() {
        let Some(destination) = target_position(game, region_id, player_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) }; let now = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, LIGHTING_ARROW_2_SKILL_ID), reuse, now) { failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if maximum != 0 && path.len() > maximum as usize { failure(game, player_id, 0x0b, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { if mp_loss != 0 { player.set_skill_moveable(false) } player.set_current_skill_id(Some(LIGHTING_ARROW_2_SKILL_ID)); }
        game.begin_player_skill_execution(player_id, LightingArrow2ExecutionState::begin(dispatch, destination, now));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).is_none_or(|state| state.kernel().dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let destination = target_position(game, region_id, player_id, dispatch).unwrap_or_else(|| game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).map(|state| state.destination).unwrap_or_default()); if target_is_dead(game, region_id, dispatch) { failure(game, player_id, 10, mp_loss); abort_player_lighting_arrow_2(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
    if game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).is_some_and(|state| !state.condition_checked) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); abort_player_lighting_arrow_2(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) { failure(game, player_id, 0x0e, mp_loss); abort_player_lighting_arrow_2(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } send_start(game, player_id, level); if let Some(state) = game.player_skill_state_mut::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID) { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).map(|state| state.kernel().started_at_ms()).unwrap_or_default();
    if game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).is_some_and(|state| !state.kernel().is_prepared()) {
        if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) } restore_player_movement(game, player_id); let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, Some(maximum)); if path.is_empty() { send_broken(game, player_id, level); abort_player_lighting_arrow_2(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if maximum != 0 && path.len() > maximum.wrapping_add(1) as usize { failure(game, player_id, 0x0b, mp_loss); abort_player_lighting_arrow_2(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } let count = path.iter().position(|cell| cell.2 == 2).unwrap_or(path.len()); let endpoint = path.get(count).or_else(|| path.last()).copied().unwrap_or((destination.0, destination.1, 2)); let visual_destination = if matches!(dispatch, PlayerSkillDispatch::Object { .. }) { destination } else { (endpoint.0, endpoint.1) }; send_fire(game, player_id, level, dispatch, visual_destination, missile_time); if let Some(state) = game.player_skill_state_mut::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID) { state.path = path; state.attack_cell_count = count; state.current_cell = 0; state.kernel_mut().mark_prepared(); let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); }
    }
    let Some((current, count, cell)) = game.player_skill_state::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).map(|state| (state.current_cell, state.attack_cell_count, state.path.get(state.current_cell).copied())) else { return terminal(QueuedSkillExecutionState::Rejected) }; if current >= count { if let Some(state) = game.player_skill_state_mut::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); } finish_player_lighting_arrow_2(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Completed) } if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(missile_time.wrapping_mul(current as u32))) { return terminal(QueuedSkillExecutionState::Pending) }
    if let Some((x, y, _)) = cell { let mut attacked = game.player_skill_state_mut::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID).map(|state| std::mem::take(&mut state.attacked_creatures)).unwrap_or_default(); attack_cell(game, player_id, region_id, level, factor, hit, x, y, &mut attacked, runtime); if let Some(state) = game.player_skill_state_mut::<LightingArrow2ExecutionState>(player_id, LIGHTING_ARROW_2_SKILL_ID) { state.attacked_creatures = attacked; state.current_cell = state.current_cell.wrapping_add(1); } }
    terminal(QueuedSkillExecutionState::Pending)
}
