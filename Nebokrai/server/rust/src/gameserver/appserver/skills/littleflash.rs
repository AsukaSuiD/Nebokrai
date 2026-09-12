//! Малые рывки сквозь строй: LittleFlash и LittleFlash2.
//!
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/littleflash.cpp
//! и littleflash2.cpp. Независимые регистрации используют общий lifecycle,
//! но только первый вариант требует S и занятую клетку на пути. Второй может
//! двигаться к сохранённой точке и отвергает одиночную заблокированную клетку.
//! MP списывается до повторной проверки меча; отказ не возвращает ресурс.
//! Visual публикует последнюю клетку пути до настоящего SetTileXY. Во время
//! удара поклеточный снимок сохраняется, а путь и список поражённых остаются
//! у зарегистрированного экземпляра и видны синхронному End. Его пролог
//! освобождает путь перед списком целей; общий Attack-End владеет AfterUse.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::dash::{apply_dash_attack, check_dash_path, execute_registered_dash, publish_dash_visual};
use super::flash::{cell_views, weapon_is_valid};
use super::kernel::{PlayerSkillExecution, SkillExecutionKernel, SkillStage, skill_is_restored};
use super::littleflash2::{EMPTY_PATH_MESSAGE_ID as LITTLE_FLASH_2_EMPTY_PATH_MESSAGE_ID, LITTLE_FLASH_2_SKILL_ID};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::public::tools::get_line_direction;

pub(crate) const LITTLE_FLASH_SKILL_ID: u32 = 0x71;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const ACTION_INTERVAL: u32 = 10_009;
const PILLAR_SKILL_ID: u32 = 0x74;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LittleFlashExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    path: Vec<(i32, i32, u8)>,
    attacked_creatures: Vec<ShapeIdentity>,
    attacking_started: bool,
    attacked: bool,
}

impl LittleFlashExecutionState {
    pub(crate) fn clear_end_paths(&mut self) {
        self.attacked = false;
        self.attacking_started = false;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked_creatures));
    }
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            path: Vec::new(), attacked_creatures: Vec::new(),
            attacking_started: false, attacked: false,
        }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}
pub(crate) const fn is_little_flash_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    LittleFlashVariant::from_id(dispatch.skill_id()).is_some()
}
fn source(game: &CGame, instance: RegisteredSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = game.registered_skill(instance)?.lifecycle().user();
    let shape = resolve_state_move_shape(game, region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}
fn state(game: &CGame, instance: RegisteredSkill) -> Option<&LittleFlashExecutionState> {
    game.registered_skill(instance)?.player_state()
}
fn state_mut(game: &mut CGame, instance: RegisteredSkill) -> Option<&mut LittleFlashExecutionState> {
    game.registered_skill_mut(instance)?.player_state_mut()
}

pub(crate) fn publish_little_flash_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CLittleFlash | SkillOwner::CLittleFlash2) { return; }
    let destination = skill.player_state::<LittleFlashExecutionState>()
        .and_then(|state| state.path.last()).map(|cell| (cell.0, cell.1));
    publish_dash_visual(game, skill, mode, SkillVisualEffectKind::LittleFlash, destination);
}

fn failure(game: &mut CGame, instance: RegisteredSkill, user: ShapeIdentity, code: u32, text: Option<&[u8]>) {
    game.update_registered_skill_visual(instance, code);
    if user.object_type == PLAYER_TYPE {
        if let Some(text) = text { game.send_skill_system_info(user.id, text); }
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let user = player.shape().identity();
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(MoveShapeSkill::last_used_ms) else { return false; };
    if !skill_is_restored(last_used, reuse, runtime.now_milliseconds()) {
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

fn attack_path<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, user: (i32, ShapeIdentity),
    region: i32, properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let mut index = 0usize;
    loop {
        if state(game, instance).is_none_or(|state| index >= state.path.len().saturating_sub(1)) { break; }
        if properties.query_property(TARGET_MAX_DISTANCE) <= index as u32 { break; }
        let Some(cell) = state(game, instance).and_then(|state| state.path.get(index).copied()) else { break; };
        for view in cell_views(game, region, cell.0, cell.1) {
            let target = view.identity;
            if target == user.1 || resolve_state_move_shape(game, region, target).is_none()
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

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(variant) = LittleFlashVariant::from_id(skill.id()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = source(game, instance);
    if variant == LittleFlashVariant::Original {
        let target = game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
        if user.is_none() || target.is_none() {
            game.update_registered_skill_visual(instance, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
    }
    let Some(user) = user else { return terminal(QueuedSkillExecutionState::Rejected); };
    if variant == LittleFlashVariant::Second && resolve_state_move_shape(game, user.0, user.1)
        .is_none_or(|shape| !shape.shape().is_assigned_to_server_region())
    { return terminal(QueuedSkillExecutionState::Rejected); }

    if state(game, instance).is_some_and(|state| state.kernel.stage() == SkillStage::Begin) {
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = resolve_skill_sufferer(game, skill.lifecycle()).and_then(|target| {
            let shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
            Some((shape.get_tile_x().unwrap_or(i32::MIN), shape.get_tile_y().unwrap_or(i32::MIN)))
        }).unwrap_or_else(|| skill.lifecycle().destination());
        let Some(shape) = resolve_state_move_shape_mut(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = shape.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = shape.shape().get_tile_x().unwrap_or(i32::MIN);
        shape.shape_mut().set_direction(get_line_direction(x, y, destination.0, destination.1));
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path(skill.lifecycle());
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let Some(path_source) = source(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = check_dash_path(
            game, path_source, path, maximum,
            variant == LittleFlashVariant::Original, variant == LittleFlashVariant::Second,
            runtime,
        );
        let empty = path.is_empty();
        let Some(state) = state_mut(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.path = path;
        if empty {
            failure(game, instance, user.1, 2, Some(variant.empty_path_message_id()));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if user.1.object_type == PLAYER_TYPE {
            let Some(mana) = game.find_player(user.1.id).map(|player| player.mana()) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                game.update_registered_skill_visual(instance, 7);
                let amount = properties.query_property(USER_MP_LOSE);
                game.send_skill_system_info_with_unsigned(user.1.id, b"GS0288", amount);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_valid(game, player)) {
                failure(game, instance, user.1, 14, Some(b"GS0292"));
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
    }

    if state(game, instance).is_some_and(|state| !state.attacking_started) {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if runtime.now_milliseconds() >= started.wrapping_add(delay) {
            game.update_registered_skill_visual(instance, 1);
            let Some(cell) = state(game, instance).and_then(|state| state.path.last().copied()) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let _ = game.set_player_tile_position(user.1.id, cell.0, cell.1);
            let Some(state) = state_mut(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            state.attacking_started = true;
        }
    }
    if state(game, instance).is_some_and(|state| state.attacking_started && !state.attacked) {
        let Some(region) = resolve_state_move_shape(game, user.0, user.1)
            .filter(|shape| shape.shape().is_assigned_to_server_region())
            .map(|shape| shape.shape().get_region_id())
            .filter(|region| game.find_region(*region).is_some())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        attack_path(game, instance, user, region, &properties, runtime);
        let Some(state) = state_mut(game, instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.attacked = true;
    }
    if state(game, instance).is_none_or(|state| !state.attacked) { return terminal(QueuedSkillExecutionState::Pending); }
    let interval = properties.query_property(ACTION_INTERVAL);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() <= started.wrapping_add(delay).wrapping_add(interval) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(shape) = resolve_state_move_shape_mut(game, user.0, user.1) { shape.set_moveable(true); }
    game.update_registered_skill_visual(instance, 3);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_little_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_little_flash_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_registered_dash(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::LittleFlash,
        check_cast, |dispatch, started| PlayerSkillExecution::LittleFlash(LittleFlashExecutionState::begin(dispatch, started)), run_ai,
    )
}
