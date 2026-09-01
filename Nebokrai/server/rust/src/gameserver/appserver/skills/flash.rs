//! Рывок сквозь строй `CFlash` (`0x69`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/flash.cpp`. Навык требует меч категории `2` и живое
//! состояние `CRageBreakState`, строит и подрезает прямой путь по снимку
//! блоков, переносит владельца в конечную свободную клетку, затем один раз
//! поражает цели на всех предшествующих клетках в региональном порядке.
//! Цели дедуплицируются на весь рывок; каждая формула выполняет ровно два
//! собственных RNG-вызова. `CGame` оставляет только spatial relocation,
//! применение урона и доставку. Совпадающая с `CLittleFlash` damage-формула
//! остаётся узким семейным helper-ом; path и lifecycle навыков различаются.
//! Ни `AI`, ни `Attack` не изнашивают оружие до завершения: унаследованный
//! `AfterUseSkill` делает это один раз в подтверждённом хвосте
//! `CSummonSkill::End(1)`, после освобождения обоих path-наборов и возврата
//! движения.

use super::baseattack::SKILL_USAGE_REUSE_DELAY_TIME;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::ragebreakstate::send_rage_break_state_visual;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{GAP_WEAPON_CATEGORY, GAP_WEAPON_DAMAGE_LEVEL};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const FLASH_SKILL_ID: u32 = 0x69;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const BATTLE_FAIRY_TYPE: i32 = 1200;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const ACTION_INTERVAL: u32 = 10_009;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const USER_HIT_MODIFIER: u32 = 20_001;
const PILLAR_SKILL_ID: u32 = 0x74;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FlashExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    attacked: bool,
    path: Vec<(i32, i32, u8)>,
    attacked_creatures: Vec<ShapeIdentity>,
}

impl FlashExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), condition_checked: false, attacked: false, path: Vec::new(), attacked_creatures: Vec::new() }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id } }
pub(crate) fn is_flash_dispatch(dispatch: PlayerSkillDispatch) -> bool { skill_id(dispatch) == FLASH_SKILL_ID }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn finish_player_flash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } game.damage_player_weapon(player_id, runtime); finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| { player_ai.mark_flash_used(now_ms); }); }
pub(crate) fn cancel_player_flash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = player_ai.flash().map(|state| state.kernel().dispatch()) else { return false }; finish_player_flash(game, player_id, player_ai, runtime); player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }
pub(super) fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool { player.equipment().get_goods(2).is_some_and(|weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2) }

fn failure(game: &CGame, player_id: i32, code: u8, amount: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        2 => game.send_skill_system_info(player_id, b"GS0303"),
        4 => game.send_skill_system_info(player_id, b"GS0304"),
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0301"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8, destination: Option<(i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(FLASH_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 2 {
        message.add_long(0); message.add_long(0);
        let (x, y) = destination.unwrap_or_default(); message.add_long(x); message.add_long(y);
    } else { message.add_long(player.shape().get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)),
    }
}

pub(super) fn cell_views(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<crate::gameserver::appserver::shape::ShapeView> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut views = Vec::new();
    if region.get_shapes(x, y, area_width, area_height, game, &mut views).is_err() { return Vec::new() }
    views
}

fn build_attack_path(
    game: &mut CGame,
    region_id: i32,
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
    maximum: u32,
) -> Vec<(i32, i32, u8)> {
    let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
    if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
    while maximum < path.len() as u32 { path.pop(); }
    let Some(last) = path.last().copied() else { return path };
    let direction = get_line_direction(source_x, source_y, last.0, last.1);
    let next = CShape::get_direction_position(direction, ShapeAreaCoordinates { x: last.0, y: last.1 }).ok();
    if let Some(next) = next {
        let block = game.find_region(region_id).map_or(2, |owner| owner.base().skill_cell_block(next.x, next.y));
        path.push((next.x, next.y, block));
    }

    let mut index = 0usize;
    let mut reached_shapes = false;
    while index < path.len() {
        let cell = path[index];
        let fairy = cell_views(game, region_id, cell.0, cell.1).iter().any(|shape| shape.identity.object_type == BATTLE_FAIRY_TYPE);
        if fairy || matches!(cell.2, 1 | 2) { if index != 0 { index -= 1; } break; }
        if !reached_shapes {
            if cell.2 != 3 { index += 1; continue; }
            index += 1; reached_shapes = true;
        } else {
            if cell.2 != 3 { break; }
            index += 1;
        }
    }
    if !reached_shapes { return Vec::new() }
    if index == path.len() { index = index.wrapping_sub(1); }
    path.truncate(index.wrapping_add(1));
    if path.last().is_some_and(|cell| cell.2 != 0) {
        let occupied = *path.last().expect("путь проверен выше");
        for direction in 0..8 {
            let Ok(candidate) = CShape::get_direction_position(direction, ShapeAreaCoordinates { x: occupied.0, y: occupied.1 }) else { continue };
            let open = game.find_region(region_id).is_some_and(|owner| candidate.x >= 0 && candidate.y >= 0 && candidate.x < owner.base().region.width && candidate.y < owner.base().region.height && owner.base().skill_cell_block(candidate.x, candidate.y) & 7 == 0);
            if open { path.push((candidate.x, candidate.y, 0)); return path; }
        }
        if let Some(owner) = game.take_region_owner(region_id) {
            if let Ok(candidate) = game.random_region_position_owned(owner.base(), occupied.0.wrapping_sub(2), occupied.1.wrapping_sub(2), 5, 5) {
                path.push((candidate.x, candidate.y, 0));
            }
            game.restore_region_owner(owner);
        }
    }
    path
}

pub(super) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}

pub(super) fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(target.id)?; game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8) }),
        _ => None,
    }
}

pub(super) fn calculate_dash_attack(game: &mut CGame, player_id: i32, skill_id: u32, target_level: u8, level: i32, hit_modifier: i32, damage_factor: u32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player);
    let weapon_level = player.equipment().get_goods(2).map_or(0, |weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1));
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors(); let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let weapon_factor = if divisor == 0.0 { 1.0 } else { (delta as f32 / divisor).min(1.0).max(minimum_factor) };
    let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_abs().wrapping_add(1);
    let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let mut attack = AttackInformation { skill_id, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier, damage_factor: damage_factor as f32 * weapon_factor * 0.01, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * rate).round_ties_even() as i32; } }
    Some((master, attack))
}

fn attack_path<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, level: i32, hit: i32, factor: u32, path: &[(i32, i32, u8)], attacked: &mut Vec<ShapeIdentity>, runtime: &mut Runtime) {
    let Some(master) = game.find_player(player_id).map(master_info) else { return };
    for &(x, y, _) in path.iter().take(path.len().saturating_sub(1)) {
        for view in cell_views(game, region_id, x, y) {
            let target = view.identity;
            if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || attacked.contains(&target) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue }
            attacked.push(target);
            let Some(target_level) = target_level(game, region_id, target) else { continue };
            let Some((master, attack)) = calculate_dash_attack(game, player_id, FLASH_SKILL_ID, target_level, level, hit, factor) else { continue };
            match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} }
        }
    }
}

pub(crate) fn execute_player_flash<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_flash_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, mana, rp)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(FLASH_SKILL_ID), player.mana(), player.rp()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(FLASH_SKILL_ID, level) else { if ai.flash().is_some() { finish_player_flash(game, player_id, ai, runtime); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let rp_loss = properties.query_property(USER_RP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let maximum = properties.query_property(TARGET_MAX_DISTANCE); let interval = properties.query_property(ACTION_INTERVAL); let hit = properties.query_property(USER_HIT_MODIFIER) as i32; let factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    if ai.flash().is_none() {
        let now = runtime.now_milliseconds();
        if ai.flash_last_used_ms() != 0 && now < ai.flash_last_used_ms().wrapping_add(reuse) { failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if (mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 { failure(game, player_id, 8, rp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        if player.has_state_by_skill_id(PILLAR_SKILL_ID) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 2); game.send_skill_system_info(player_id, b"GS0302"); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(FLASH_SKILL_ID)); }
        ai.begin_flash(FlashExecutionState::begin(dispatch, now));
    } else if ai.flash().is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }

    if ai.flash().is_some_and(|state| !state.condition_checked) {
        let Some(player) = game.find_player(player_id) else { finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_valid(game, player) { failure(game, player_id, 0x0e, mp_loss); finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some((target_x, target_y)) = target_position(game, region_id, player_id, dispatch) else { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10); finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
        let path = build_attack_path(game, region_id, source_x, source_y, target_x, target_y, maximum);
        if path.is_empty() { failure(game, player_id, 2, mp_loss); finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(rage_state) = game.find_player_mut(player_id).and_then(CPlayer::take_rage_break_state) else { failure(game, player_id, 4, mp_loss); finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) };
        send_rage_break_state_visual(game, region_id, ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: Default::default() }, source_x, source_y, rage_state, false, runtime.now_milliseconds());
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 { failure(game, player_id, 7, mp_loss); let _ = game.update_player_properties(player_id); finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current_mana.wrapping_sub(mp_loss)); }
        let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 { failure(game, player_id, 8, rp_loss); let _ = game.update_player_properties(player_id); finish_player_flash(game, player_id, ai, runtime); return terminal(QueuedSkillExecutionState::Rejected) }
        if let Some(player) = game.find_player_mut(player_id) { player.set_rp(u32::from(current_rp).wrapping_sub(rp_loss) as u16); player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); }
        let destination = *path.last().expect("непустой путь проверен выше");
        let _ = game.relocate_player_shape(player_id, region_id, destination.0, destination.1);
        send_visual(game, player_id, level, 2, Some((destination.0, destination.1)));
        if let Some(state) = ai.flash_mut() { state.path = path; state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); }
        let _ = game.update_player_properties(player_id);
    }

    if ai.flash().is_some_and(|state| state.condition_checked && !state.attacked) {
        let path = ai.flash().map(|state| state.path.clone()).unwrap_or_default();
        let mut attacked = ai.flash_mut().map(|state| std::mem::take(&mut state.attacked_creatures)).unwrap_or_default();
        attack_path(game, player_id, region_id, level, hit, factor, &path, &mut attacked, runtime);
        if let Some(state) = ai.flash_mut() { state.attacked_creatures = attacked; state.attacked = true; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); }
    }
    let started = ai.flash().map(|state| state.kernel.started_at_ms()).unwrap_or_default();
    if runtime.now_milliseconds() <= started.wrapping_add(interval) { return terminal(QueuedSkillExecutionState::Pending) }
    send_visual(game, player_id, level, 3, None);
    if let Some(state) = ai.flash_mut() { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_flash(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
