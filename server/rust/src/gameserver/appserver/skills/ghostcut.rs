//! Бегущий удар GhostCut/GhostCut2/GhostCut3 (0x66/0x79/0x7A).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/ghostcut*.cpp.
//!
//! Общий зарегистрированный вход сохраняет раннее время Begin и исходного U
//! для Check. Первые два варианта требуют оружие категории 1, третий — 2.
//! Нулевая цена MP отклоняет Check молча; отказ Begin дополнительно вызывает
//! visual2 перед End(0). В первом AI повторная проверка оружия предшествует
//! списанию MP и OnChangeStates; затем CAN, живой S либо базовая точка,
//! направление U и visual0. Таблица свойств сохраняется до конца этого AI.
//!
//! Выпуск и каждая клетка ждут абсолютный unsigned срок без elapsed-поправки.
//! Endpoint — первый BLOCK_UNFLY либо последняя клетка полученного пути;
//! время полёта равно шагу, умноженному на индекс преграды или длину.
//! До visual1 цель становится точкой; attacking и prepared включаются после.
//! AI обрабатывает не более одной живой клетки, начиная с позиции 0. При
//! исчезнувшем регионе он ждёт, а не завершает навык. Попадания и их общий
//! оружейный расчёт принадлежат ghostcutattack; постоянный список хранится
//! здесь и не извлекается через callbacks. End сбрасывает фазу, attacking,
//! время и позицию, освобождает путь и список до возврата движения/AfterUse.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::ghostcut2::GHOST_CUT_2_SKILL_ID;
use super::ghostcut3::GHOST_CUT_3_SKILL_ID;
use super::ghostcutattack::attack_ghost_cut_cell;
use super::kernel::{SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{GhostCutExecutionState};

pub(crate) const GHOST_CUT_SKILL_ID: u32 = 0x66;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_matches(game: &CGame, player: &CPlayer, skill_id: u32) -> bool {
    let category = if skill_id == GHOST_CUT_3_SKILL_ID { 2 } else { 1 };
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == category
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, skill_id: u32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code {
        11 => b"GS0290",
        13 => b"GS0278",
        14 if skill_id == GHOST_CUT_3_SKILL_ID => b"GS0292",
        14 => b"GS0287",
        _ => return,
    };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, skill_id, 13);
        return false;
    }
    if !weapon_matches(game, player, skill_id) {
        failure(game, instance, player_id, skill_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let mana = player.mana();
    let loss = properties.query_property(USER_MP_LOSE);
    if (mana.wrapping_sub(loss) as i32) < 0 {
        mana_failure(game, instance, player_id, &properties);
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            if !weapon_matches(game, player, skill_id) {
                failure(game, instance, user.1.id, skill_id, 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                mana_failure(game, instance, user.1.id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            game.publish_player_states(user.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let Some(attacking) = game.registered_skill(instance).and_then(|skill| skill.player_state::<GhostCutExecutionState>()).map(|state| state.attacking_started) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<GhostCutExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.path = path;
        if properties.query_property(TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(TARGET_MAX_DISTANCE);
            if maximum.wrapping_add(1) < state.path.len() as u32 {
                if user.1.object_type == PLAYER_TYPE { failure(game, instance, user.1.id, skill_id, 11); }
                else { game.update_registered_skill_visual(instance, 11); }
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let index = state.path.iter().position(|cell| cell.2 == 2).unwrap_or(state.path.len());
        let endpoint = state.path.get(index).or_else(|| state.path.last()).map(|cell| (cell.0, cell.1));
        state.missile_flying_time = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(index as u32);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = endpoint.unwrap_or_else(|| skill.lifecycle().destination());
        skill.lifecycle_mut().set_point_target(destination);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<GhostCutExecutionState>()) {
            state.attacking_started = true;
            state.kernel.lifecycle_mut().mark_prepared();
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    let step = properties.query_property(MISSILE_FLYING_TIME);
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(state) = skill.player_state::<GhostCutExecutionState>() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let position = state.current_position;
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let deadline = step.wrapping_mul(position).wrapping_add(delay).wrapping_add(skill.lifecycle().started_at_ms());
    if runtime.now_milliseconds() < deadline { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Pending); };
    if !source.shape().is_assigned_to_server_region() { return terminal(QueuedSkillExecutionState::Pending); }
    let region_id = source.shape().get_region_id();
    let Some(owner) = game.find_region(region_id) else { return terminal(QueuedSkillExecutionState::Pending); };
    let Some((x, y, _)) = state.path.get(position as usize).copied() else {
        game.update_registered_skill_visual(instance, 3);
        return terminal(QueuedSkillExecutionState::Completed);
    };
    let block = owner.base().skill_cell_block(x, y);
    if block == 3 {
        attack_ghost_cut_cell(game, instance, user, x, y, runtime);
    } else if block == 2 {
        game.update_registered_skill_visual(instance, 3);
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<GhostCutExecutionState>()) {
            state.current_position = state.path.len() as u32;
        }
    }
    if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<GhostCutExecutionState>()) {
        state.current_position = state.current_position.wrapping_add(1);
    }
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_ghost_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !matches!(dispatch.skill_id(), GHOST_CUT_SKILL_ID | GHOST_CUT_2_SKILL_ID | GHOST_CUT_3_SKILL_ID) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::GhostCut,
        |game, instance, player_id, runtime| {
            let accepted = check_cast(game, instance, player_id, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| GhostCutExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
