//! Летающий рубящий удар якши `CYakshaSlash` (`0x196`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/yakshaslash.cpp`. Сохранены начальная проверка реального
//! расстояния, две проверки непроходимого для полёта пути, задержка, время
//! полёта по числу клеток, положительный hit modifier и исходные RNG-вызовы
//! конкретного `CMoveShape`. Объектная ветвь достигнута и для игрока, и для
//! монстра; `CGame` только разрешает владельцев, применяет готовый удар и
//! выполняет доставку.
//! Беззнаковый skill factor сохраняется в x87 до единственной записи в
//! `float`. Критический множитель переводится в `int` с x87 rounding-control
//! `11`, то есть усечением к нулю после умножения каждого боевого компонента.
//! `End(true)` фиксирует обновление свойств и cooldown после удара;
//! `End(false)` прекращает полёт без отката уже применённой атаки. Player и
//! monster ветви используют абсолютный срок `CSkill::IsRestored`; cast и полёт
//! остаются elapsed.
//! После эффекта выпуска player-ветвь ставит prepared (0x005432FE).
//! Этот флаг общего kernel сохраняет полёт при переходе из Attack в фон.
//! Успешный Begin возвращает Begun после инициализации исполнения. Первый
//! AI выполняет повторные проверки и эффекты отдельно, в том же Run после
//! постановки Attack; раннее время Begin сохраняется общим kernel.
//! End (0x0057B810, общий со SpiderWeb) очищает полёт и снимает один запрет
//! движения перед CAttackSkill::End. Monster-путь использует общую очистку
//! CMonster при успехе, отмене и Stiffen; отдельное снятие запрета перед
//! выпуском сохраняется, а End не откатывает удар и не отправляет эффект.

use super::baseattack::{time_reached, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::monsterattack::{MonsterAttackDeath, finish_owned_monster_attack_impact, owned_monster_attackable, resolve_owned_monster_attack_target};
use super::monsterprojectile::{MonsterProjectileDispatch, execute_owned_monster_projectile_target};
use super::poisonmoth::{master_info, MONSTER_TYPE, PLAYER_TYPE};
use crate::gameserver::appserver::ai::monsterai::schedule_attack_interval;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

pub(crate) const YAKSHA_SLASH_SKILL_ID: u32 = 0x196;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const BLOCK_UNFLY: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct YakshaSlashExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    missile_flying_time_ms: u32,
}

impl YakshaSlashExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), condition_checked: false, missile_flying_time_ms: 0 }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_yaksha_slash_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::Object { skill_id: YAKSHA_SLASH_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } })
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_yaksha_slash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(YAKSHA_SLASH_SKILL_ID, now_ms));
}

fn abort_player_yaksha_slash(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }

pub(crate) fn complete_player_yaksha_slash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_yaksha_slash(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_yaksha_slash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_yaksha_slash(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn fail(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn target_view(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<ShapeView> {
    game.base_magic_target_view(region_id, target)
}

fn send_cast(game: &mut CGame, player_id: i32, level: i32, target: ShapeIdentity, position: (i32, i32), flying_time: Option<u32>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if flying_time.is_some() { 2 } else { 1 });
    message.add_long(YAKSHA_SLASH_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if let Some(flying_time) = flying_time {
        message.add_long(target.object_type); message.add_long(target.id);
        message.add_long(position.0); message.add_long(position.1);
        message.add_ulong(flying_time);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_monster_cast(game: &CGame, region: &CServerRegion, monster_id: i32, level: u16, target: ShapeIdentity, position: (i32, i32), flying_time: Option<u32>) {
    let Some(monster) = region.find_monster_by_id(monster_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if flying_time.is_some() { 2 } else { 1 });
    message.add_long(YAKSHA_SLASH_SKILL_ID as i32); message.add_short(level as i16);
    message.add_long(MONSTER_TYPE); message.add_long(monster_id);
    if let Some(flying_time) = flying_time {
        message.add_long(target.object_type); message.add_long(target.id);
        message.add_long(position.0); message.add_long(position.1); message.add_ulong(flying_time);
    } else { message.add_long(monster.move_shape().shape().get_direction()); }
    let _ = game.send_game_shape_around(region, monster.move_shape().shape(), None, &message);
}

/// Объектный `CYakshaSlash::AI` для владельца-монстра: reuse/range и две
/// проверки непроходимого полёта, блокировка движения до delay, время полёта
/// по числу клеток и общий monster defence/death tail.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_owned_monster_yaksha_slash<Runtime: GameMainLoopRuntime>(game: &mut CGame, region: &mut CServerRegion, monster_id: i32, target_identity: ShapeIdentity, skill_level: u16, properties: &CSkillBaseProperties, property: &MonsterProperties, now_ms: u32, runtime: &mut Runtime, deaths: &mut Vec<MonsterAttackDeath>) -> bool {
    let Some((source, source_view, master, tamed, cast, progress)) = region.find_monster_by_id(monster_id).and_then(|monster| Some((monster.move_shape().shape().clone(), monster.shape_view(property)?, monster.master_info(), monster.is_tamed(), monster.current_active_attack_cast(), monster.monster_projectile_progress()))) else { return false };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_none_or(|execution| execution.termination().is_some()) {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead || target.god || target.city_dead || !owned_monster_attackable(game, region.id, property, tamed, master, target_identity, &target) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_none_or(|execution| execution.termination().is_some()) {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (source.get_tile_x(), source.get_tile_y(), target.shape.get_tile_x(), target.shape.get_tile_y()) else { return true };
    let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
    if cast.is_none() {
        let attack_interval = if tamed { region.find_monster_by_id(monster_id).map(|monster| monster.pet_attack_properties(property).attack_interval).unwrap_or(property.attack_speed) } else { property.attack_speed };
        if schedule_attack_interval(property.ai, attack_interval).is_some_and(|interval| region.find_monster_by_id_mut(monster_id).is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))) { return true; }
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let last_used = region.find_monster_by_id(monster_id).map(|monster| monster.skill_last_used_ms(YAKSHA_SLASH_SKILL_ID)).unwrap_or_default();
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used, reuse, now_ms,
            )
        {
            return true;
        }
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        if (maximum != 0 && source_view.real_distance(Some(target.view)) > maximum as i32) || path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(); }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction); monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, YAKSHA_SLASH_SKILL_ID, skill_level, now_ms); monster.begin_monster_projectile_progress();
        }
        send_monster_cast(game, region, monster_id, skill_level, target_identity, (target_x, target_y), None);
        return true;
    }
    let cast = cast.expect("ветвь активного полёта проверена выше");
    if cast.dispatch().skill_id != YAKSHA_SLASH_SKILL_ID || cast.dispatch().target != target_identity { return false; }
    let Some(mut progress) = progress else { return true };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay) { return true; }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(); }
            return true;
        }
        let flying_time = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(path.len() as u32);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true); progress.fire(flying_time, None);
            *monster.monster_projectile_progress_mut().expect("состояние полёта принадлежит текущему навыку") = progress;
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        send_monster_cast(game, region, monster_id, skill_level, target_identity, (target_x, target_y), Some(flying_time));
    }
    if !time_reached(now_ms, cast.started_at_ms(), delay.wrapping_add(progress.missile_flying_time_ms())) { return true; }
    let dispatch = MonsterProjectileDispatch::object_target(monster_id, YAKSHA_SLASH_SKILL_ID, target_x, target_y, skill_level, properties.clone(), property.clone(), master, tamed, now_ms);
    let _ = execute_owned_monster_projectile_target(game, region, &dispatch, target_identity, runtime, deaths);
    finish_owned_monster_attack_impact(region, dispatch.monster_id, runtime);
    true
}

fn calculate_attack(game: &mut CGame, player_id: i32, level: i32, factor: u32, hit: i32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let span = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_abs().wrapping_add(1);
    let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(span)).max(0);
    let damage_factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    let mut attack = AttackInformation { skill_id: YAKSHA_SLASH_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } }
    Some((master, attack))
}

pub(crate) fn execute_player_yaksha_slash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_yaksha_slash_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let PlayerSkillDispatch::Object { target, .. } = dispatch else { unreachable!() };
    let Some((region_id, source_view, level)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape_view()?, player.learned_skill_level(YAKSHA_SLASH_SKILL_ID)))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let (source_x, source_y) = (source_view.tile_x, source_view.tile_y);
    let Some(properties) = game.skill_base_properties(YAKSHA_SLASH_SKILL_ID, level) else { fail(game, player_id, 2); if ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().is_some() { abort_player_yaksha_slash(game, player_id); } return terminal(QueuedSkillExecutionState::Rejected) };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum = properties.query_property(TARGET_MAX_DISTANCE); let missile_step = properties.query_property(MISSILE_FLYING_TIME); let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().is_none() {
        let now = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(YAKSHA_SLASH_SKILL_ID), reuse, now) { fail(game, player_id, 0x0d); fail(game, player_id, 2); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(target_view) = target_view(game, region_id, target) else { fail(game, player_id, 2); return terminal(QueuedSkillExecutionState::Rejected) };
        if maximum != 0 && source_view.real_distance(Some(target_view)) > maximum as i32 { fail(game, player_id, 0x0b); fail(game, player_id, 2); return terminal(QueuedSkillExecutionState::Rejected) }
        if game.base_magic_path(region_id, source_x, source_y, target_view.tile_x, target_view.tile_y, None).iter().any(|cell| cell.2 == BLOCK_UNFLY) { fail(game, player_id, 0x0f); fail(game, player_id, 2); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(YAKSHA_SLASH_SKILL_ID)); }
        ai.begin_player_skill_execution(YakshaSlashExecutionState::begin(dispatch, now));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if game.periodic_state_target_dead(region_id, target) || (target.object_type == PLAYER_TYPE && target.id == player_id) { fail(game, player_id, 10); abort_player_yaksha_slash(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
    if ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().is_some_and(|state| !state.condition_checked) {
        let Some(target_view) = target_view(game, region_id, target) else { abort_player_yaksha_slash(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
        let position = (target_view.tile_x, target_view.tile_y);
        if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, position.0, position.1)); }
        send_cast(game, player_id, level, target, position, None);
        if let Some(state) = ai.player_skill_state_mut::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID) { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().map(|state| state.kernel.started_at_ms()).unwrap_or_default();
    if !ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().is_some_and(|state| state.kernel().is_prepared()) {
        if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
        let Some(target_view) = target_view(game, region_id, target) else { abort_player_yaksha_slash(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
        let position = (target_view.tile_x, target_view.tile_y);
        let path = game.base_magic_path(region_id, source_x, source_y, position.0, position.1, None);
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) { fail(game, player_id, 0x0f); abort_player_yaksha_slash(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
        let flying_time = missile_step.wrapping_mul(path.len() as u32);
        send_cast(game, player_id, level, target, position, Some(flying_time));
        if let Some(state) = ai.player_skill_state_mut::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID) { state.missile_flying_time_ms = flying_time; state.kernel_mut().mark_prepared(); let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); }
    }
    let flying_time = ai.player_skill_state::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID).copied().map_or(0, |state| state.missile_flying_time_ms);
    if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(flying_time)) { return terminal(QueuedSkillExecutionState::Pending) }
    if let Some((master, attack)) = calculate_attack(game, player_id, level, factor, hit) { match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} } }
    if let Some(state) = ai.player_skill_state_mut::<YakshaSlashExecutionState>(YAKSHA_SLASH_SKILL_ID) { let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_yaksha_slash(game, player_id, ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
