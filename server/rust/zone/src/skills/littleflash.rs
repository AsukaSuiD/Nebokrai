//! Малые рывки сквозь строй: LittleFlash и LittleFlash2.
//!
//! Quirks: только первый вариант требует S и занятую клетку на пути; второй
//! движется к сохранённой точке и отвергает одиночную заблокированную клетку
//! (`clear_single_blocked`); MP списывается до повторной проверки меча без
//! возврата при отказе; visual публикует последнюю клетку пути до настоящего
//! SetTileXY; пролог End освобождает путь перед списком целей.
//!
//! Швы: фасады `DashSkillGame`/`DashSkillContact` (`skills/dash.rs`);
//! проверка меча — общий предикат `flash::weapon_is_valid` (category 2).
//! `SKILL_USAGE_CAN_BE_BREAKED` — usage `10006`, локальная константа по
//! конвенции per-owner файлов старого пакета.
//!
//! UNKNOWN: потребители raw CAN `available`.
//!
//! Исходные владельцы PDB: `appserver/skills/{littleflash,littleflash2}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#dashflashlittleflash--рывки

use nebokrai_shared::runtime::get_line_direction;

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::baseattackruntime::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::dash::{
    DashSkillContact, DashSkillExecutionOutcome, DashSkillGame, DashSkillMoveShape,
    DashSkillPlayer, apply_dash_attack, check_dash_path, publish_dash_visual,
};
use super::execution::{LittleFlashExecutionState, RegisteredSkillRecord};
use super::flash::weapon_is_valid;
use super::pillar::PILLAR_SKILL_ID;
use super::skill_is_restored;
use super::skillfactory::SkillOwner;
use super::visualeffect::SkillVisualEffectKind;
use super::{PlayerSkillDispatch, SkillStage};

pub const LITTLE_FLASH_SKILL_ID: u32 = 0x71;
pub const LITTLE_FLASH_2_SKILL_ID: u32 = 0x7f;
pub const LITTLE_FLASH_2_EMPTY_PATH_MESSAGE_ID: &[u8] = b"GS0309";
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const ACTION_INTERVAL: u32 = 10_009;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LittleFlashVariant { Original, Second }

impl LittleFlashVariant {
    const fn from_id(id: u32) -> Option<Self> {
        match id {
            LITTLE_FLASH_SKILL_ID => Some(Self::Original),
            LITTLE_FLASH_2_SKILL_ID => Some(Self::Second),
            _ => None,
        }
    }
    const fn empty_path_message_id(self) -> &'static [u8] {
        match self {
            Self::Original => b"GS0290",
            Self::Second => LITTLE_FLASH_2_EMPTY_PATH_MESSAGE_ID,
        }
    }
}

pub const fn is_little_flash_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    LittleFlashVariant::from_id(dispatch.skill_id()).is_some()
}

fn source<Game: DashSkillGame>(game: &Game, instance: Game::SkillAddress) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = game.registered_skill(instance)?.lifecycle().user();
    let shape = game.resolve_state_move_shape(region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

fn state<Game: DashSkillGame>(game: &Game, instance: Game::SkillAddress) -> Option<&LittleFlashExecutionState> {
    game.registered_skill(instance)?.player_state()
}

fn state_mut<Game: DashSkillGame>(game: &mut Game, instance: Game::SkillAddress) -> Option<&mut LittleFlashExecutionState> {
    game.registered_skill_mut(instance)?.player_state_mut()
}

pub fn publish_little_flash_visual<Game: DashSkillGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if !matches!(skill.owner(), SkillOwner::CLittleFlash | SkillOwner::CLittleFlash2) { return; }
    let destination = skill.player_state::<LittleFlashExecutionState>()
        .and_then(|state| state.path.last()).map(|cell| (cell.0, cell.1));
    publish_dash_visual(game, skill, mode, SkillVisualEffectKind::LittleFlash, destination);
}

fn failure<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    user: ShapeIdentity,
    code: u32,
    text: Option<&[u8]>,
) {
    game.update_registered_skill_visual(instance, code);
    if user.object_type == PLAYER_TYPE {
        if let Some(text) = text { game.send_skill_system_info(user.id, text); }
    }
}

pub fn check_cast<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let user = player.shape().identity();
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(|skill| skill.last_used_ms()) else { return false; };
    if !skill_is_restored(last_used, reuse, now_milliseconds()) {
        failure(game, instance, user, 13, Some(b"GS0278"));
        return false;
    }
    if game.find_player(player_id).is_some_and(|player| player.has_state_by_skill_id(PILLAR_SKILL_ID)) {
        failure(game, instance, user, 2, Some(b"GS0302"));
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn attack_path<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    user: (i32, ShapeIdentity),
    region: i32,
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) where
    Game: DashSkillGame + DashSkillContact<Runtime>,
{
    let mut index = 0usize;
    loop {
        if state(game, instance).is_none_or(|state| index >= state.path.len().saturating_sub(1)) { break; }
        if properties.query_property(TARGET_MAX_DISTANCE) <= index as u32 { break; }
        let Some(cell) = state(game, instance).and_then(|state| state.path.get(index).copied()) else { break; };
        for view in game.dash_cell_views(region, cell.0, cell.1) {
            let target = view.identity;
            if target == user.1 || game.resolve_state_move_shape(region, target).is_none()
                || !game.live_skill_target_attackable(region, user.1, target)
            { continue; }
            let Some(state) = state_mut(game, instance) else { return; };
            if state.attacked_creatures.contains(&target) { continue; }
            state.attacked_creatures.push(target);
            apply_dash_attack(game, instance, user, (region, target), runtime);
        }
        index += 1;
    }
}

pub fn run_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> DashSkillExecutionOutcome
where
    Game: DashSkillGame + DashSkillContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) { return DashSkillExecutionOutcome::Pending; }
    let Some(variant) = LittleFlashVariant::from_id(skill.id()) else { return DashSkillExecutionOutcome::Rejected; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return DashSkillExecutionOutcome::Rejected; };
    let user = source(game, instance);
    if variant == LittleFlashVariant::Original {
        let target = game.registered_skill(instance).and_then(|skill| game.resolve_skill_sufferer(skill.lifecycle()));
        if user.is_none() || target.is_none() {
            game.update_registered_skill_visual(instance, 10);
            return DashSkillExecutionOutcome::Rejected;
        }
    }
    let Some(user) = user else { return DashSkillExecutionOutcome::Rejected; };
    if variant == LittleFlashVariant::Second && game.resolve_state_move_shape(user.0, user.1)
        .is_none_or(|shape| !shape.shape().is_assigned_to_server_region())
    { return DashSkillExecutionOutcome::Rejected; }

    if state(game, instance).is_some_and(|state| state.kernel.stage() == SkillStage::Begin) {
        let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
        let destination = game.resolve_skill_sufferer(skill.lifecycle()).and_then(|target| {
            let shape = game.resolve_state_move_shape(target.0, target.1)?.shape();
            Some((shape.get_tile_x().unwrap_or(i32::MIN), shape.get_tile_y().unwrap_or(i32::MIN)))
        }).unwrap_or_else(|| skill.lifecycle().destination());
        let Some(shape) = game.resolve_state_move_shape_mut(user.0, user.1) else { return DashSkillExecutionOutcome::Rejected; };
        let y = shape.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = shape.shape().get_tile_x().unwrap_or(i32::MIN);
        shape.shape_mut().set_direction(get_line_direction(x, y, destination.0, destination.1));
        let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
        let path = game.skill_target_path(skill.lifecycle());
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let Some(path_source) = source(game, instance) else { return DashSkillExecutionOutcome::Rejected; };
        let path = check_dash_path(
            game, path_source, path, maximum,
            variant == LittleFlashVariant::Original, variant == LittleFlashVariant::Second,
            runtime,
        );
        let empty = path.is_empty();
        let Some(state) = state_mut(game, instance) else { return DashSkillExecutionOutcome::Rejected; };
        state.path = path;
        if empty {
            failure(game, instance, user.1, 2, Some(variant.empty_path_message_id()));
            return DashSkillExecutionOutcome::Rejected;
        }
        if user.1.object_type == PLAYER_TYPE {
            let Some(mana) = game.find_player(user.1.id).map(|player| player.mana()) else { return DashSkillExecutionOutcome::Rejected; };
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                game.update_registered_skill_visual(instance, 7);
                let amount = properties.query_property(USER_MP_LOSE);
                game.send_skill_system_info_with_unsigned(user.1.id, b"GS0288", amount);
                return DashSkillExecutionOutcome::Rejected;
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_valid(game, player)) {
                failure(game, instance, user.1, 14, Some(b"GS0292"));
                return DashSkillExecutionOutcome::Rejected;
            }
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return DashSkillExecutionOutcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
    }

    if state(game, instance).is_some_and(|state| !state.attacking_started) {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return DashSkillExecutionOutcome::Rejected; };
        if now_milliseconds() >= started.wrapping_add(delay) {
            game.update_registered_skill_visual(instance, 1);
            let Some(cell) = state(game, instance).and_then(|state| state.path.last().copied()) else { return DashSkillExecutionOutcome::Rejected; };
            game.set_player_tile_position(user.1.id, cell.0, cell.1);
            let Some(state) = state_mut(game, instance) else { return DashSkillExecutionOutcome::Rejected; };
            state.attacking_started = true;
        }
    }
    if state(game, instance).is_some_and(|state| state.attacking_started && !state.attacked) {
        let Some(region) = game.resolve_state_move_shape(user.0, user.1)
            .filter(|shape| shape.shape().is_assigned_to_server_region())
            .map(|shape| shape.shape().get_region_id())
            .filter(|region| game.dash_region_size(*region).is_some())
        else { return DashSkillExecutionOutcome::Rejected; };
        attack_path(game, instance, user, region, &properties, runtime);
        let Some(state) = state_mut(game, instance) else { return DashSkillExecutionOutcome::Rejected; };
        state.attacked = true;
    }
    if state(game, instance).is_none_or(|state| !state.attacked) { return DashSkillExecutionOutcome::Pending; }
    let interval = properties.query_property(ACTION_INTERVAL);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return DashSkillExecutionOutcome::Rejected; };
    if now_milliseconds() <= started.wrapping_add(delay).wrapping_add(interval) { return DashSkillExecutionOutcome::Pending; }
    if let Some(shape) = game.resolve_state_move_shape_mut(user.0, user.1) { shape.set_moveable(true); }
    game.update_registered_skill_visual(instance, 3);
    DashSkillExecutionOutcome::Completed
}
