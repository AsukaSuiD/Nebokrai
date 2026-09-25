//! Рывок CFlash (0x69). Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/flash.cpp.
//!
//! Check не использует S: sword, signed MP/RP и отсутствие Pillar проверяются
//! после reuse. AI требует разрешимую S, но не проверяет её здоровье; направление
//! и отдельный path строятся до повторной проверки оружия. Первый RageBreak
//! завершается с удалением свежего остатка до поздних MP/RP queries. Списание
//! MP сохраняется и при последующем отказе RP. OnChangeStates предшествует
//! переносу, visual1 и condition; CAN этим навыком не изменяется.
//!
//! Path и список уже атакованных принадлежат конкретному зарегистрированному
//! навыку и остаются доступны callbacks. AI читает живой path на каждой клетке,
//! ограничивает его дополнительным max и проверяет каждый объект через IsAttackAble.
//! Общий удар семейства выполняет fresh Calculate, raw OnBeenAttacked и RP.
//! После strict unsigned срока start+interval идут Moveable1 и visual3;
//! единственный End, очистка двух списков и AfterUse принадлежат общему входу.

use super::baseattack::SKILL_USAGE_REUSE_DELAY_TIME;
use super::dash::{apply_dash_attack, check_dash_path, publish_dash_visual};
use super::playercast::execute_registered_player_cast;
use super::kernel::{SkillStage, skill_is_restored};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{FlashExecutionState};

pub(crate) const FLASH_SKILL_ID: u32 = 0x69;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const ACTION_INTERVAL: u32 = 10_009;
const PILLAR_SKILL_ID: u32 = 0x74;
const RAGE_BREAK_STATE_ID: u32 = 0x6e;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(super) fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn fail(game: &mut CGame, instance: RegisteredSkill, player_id: Option<i32>, mode: u32, text: &[u8]) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id { game.send_skill_system_info(player_id, text); }
}

fn fail_resource(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, usage: u32,
) {
    let (mode, text): (u32, &[u8]) = if usage == USER_MP_LOSE { (7, b"GS0288") } else { (8, b"GS0289") };
    game.update_registered_skill_visual(instance, mode);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

pub(crate) fn publish_flash_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CFlash { return; }
    let destination = skill.player_state::<FlashExecutionState>()
        .and_then(|state| state.path.last()).map(|cell| (cell.0, cell.1));
    publish_dash_visual(game, skill, mode, SkillVisualEffectKind::Flash, destination);
}

pub(super) fn cell_views(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<crate::gameserver::appserver::shape::ShapeView> {
    let Some(owner) = game.find_region(region_id) else { return Vec::new(); };
    let resolver = crate::gameserver::gameserver::game::RegionShapeResolver { game, owner };
    let (area_width, area_height) = game.area_dimensions();
    let mut views = Vec::new();
    if owner.base().get_shapes(x, y, area_width, area_height, &resolver, &mut views).is_err() { return Vec::new(); }
    views
}

pub(crate) fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}

pub(super) fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
            let monster = owner.base().find_monster_by_id(target.id)?;
            game.find_monster_property_by_origin_name(monster.base_property_key()?).map(|property| property.level as u8)
        }),
        _ => None,
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(MoveShapeSkill::last_used_ms) else { return false; };
    if !skill_is_restored(last_used, reuse, runtime.now_milliseconds()) {
        fail(game, instance, Some(player_id), 13, b"GS0278");
        return false;
    }
    if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
        fail(game, instance, Some(player_id), 14, b"GS0301");
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else { return false; };
        let cost = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(cost) as i32) < 0 {
            fail_resource(game, instance, player_id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let Some(rp) = game.find_player(player_id).map(CPlayer::rp) else { return false; };
        let cost = properties.query_property(USER_RP_LOSE);
        if (u32::from(rp).wrapping_sub(cost) as i32) < 0 {
            fail_resource(game, instance, player_id, &properties, USER_RP_LOSE);
            return false;
        }
    }
    if game.find_player(player_id).is_some_and(|player| player.has_state_by_skill_id(PILLAR_SKILL_ID)) {
        fail(game, instance, Some(player_id), 2, b"GS0302");
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

pub(crate) fn execute_player_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != FLASH_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Flash,
        check_cast, |dispatch, started| FlashExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

fn run_ai<Runtime: GameMainLoopRuntime>(game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity).map(|user| (user.shape().get_region_id(), user.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    let (Some(user), Some(_target)) = (user, target) else {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let player_id = (user.1.object_type == PLAYER_TYPE).then_some(user.1.id);
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).is_some_and(|state| !state.condition_checked) {
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| resolve_state_move_shape(game, target.0, target.1))
            .map(|target| (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN)))
            .unwrap_or_else(|| skill.lifecycle().destination());
        let Some(source) = resolve_state_move_shape(game, user.0, user.1).map(|source| source.shape()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = source.get_tile_y().unwrap_or(i32::MIN);
        let source_x = source.get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path(skill.lifecycle());
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let path = check_dash_path(game, user, path, maximum, true, false, runtime);
        let empty = path.is_empty();
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<FlashExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.path = path;
        if empty {
            fail(game, instance, player_id, 2, b"GS0303");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player_id) = player_id {
            if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
                fail(game, instance, Some(player_id), 14, b"GS0301");
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let previous = resolve_state_move_shape(game, user.0, user.1)
                .and_then(|source| source.find_state_position(|state| state.state_id() == RAGE_BREAK_STATE_ID))
                .map(|(position, _)| position);
            let Some(position) = previous else {
                fail(game, instance, Some(player_id), 4, b"GS0304");
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            let _ = end_and_destroy_state_at(game, user.0, user.1, position);
            let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let mp_loss = properties.query_property(USER_MP_LOSE);
            let mana = mana.wrapping_sub(mp_loss);
            if (mana as i32) < 0 {
                fail_resource(game, instance, player_id, &properties, USER_MP_LOSE);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana); }
            let Some(rp) = game.find_player(player_id).map(CPlayer::rp) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let rp_loss = properties.query_property(USER_RP_LOSE);
            let rp = u32::from(rp).wrapping_sub(rp_loss);
            if (rp as i32) < 0 {
                fail_resource(game, instance, player_id, &properties, USER_RP_LOSE);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(player_id) { player.set_rp(rp as u16); }
            let _ = game.publish_player_states(player_id);
        }
        let Some(destination) = game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).and_then(|state| state.path.last()).copied() else { return terminal(QueuedSkillExecutionState::Rejected); };
        let _ = game.relocate_region_shape(user.0, user.1, destination.0, destination.1);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<FlashExecutionState>()) {
            state.condition_checked = true;
            let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).is_some_and(|state| state.condition_checked && !state.attacked) {
        let Some(region_id) = resolve_state_move_shape(game, user.0, user.1)
            .map(|source| source.shape()).filter(|source| source.is_assigned_to_server_region())
            .map(CShape::get_region_id).filter(|region| game.find_region(*region).is_some())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        let mut index = 0usize;
        loop {
            let Some(state) = game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()) else { break; };
            if index >= state.path.len().saturating_sub(1) { break; }
            if properties.query_property(TARGET_MAX_DISTANCE) as usize <= index { break; }
            let (x, y, _) = state.path[index];
            for view in cell_views(game, region_id, x, y) {
                let target = view.identity;
                let Some(target_shape) = resolve_state_move_shape(game, region_id, target) else { continue; };
                let Some(source_shape) = resolve_state_move_shape(game, user.0, user.1) else { continue; };
                if std::ptr::eq(source_shape, target_shape)
                    || !game.live_skill_target_attackable(region_id, user.1, target) { continue; }
                let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<FlashExecutionState>()) else { break; };
                if state.attacked_creatures.contains(&target) { continue; }
                state.attacked_creatures.push(target);
                apply_dash_attack(game, instance, user, (region_id, target), runtime);
            }
            index += 1;
        }
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<FlashExecutionState>()) {
            state.attacked = true;
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).is_none_or(|state| !state.attacked) { return terminal(QueuedSkillExecutionState::Pending); }
    let interval = properties.query_property(ACTION_INTERVAL);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() <= started.wrapping_add(interval) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
    game.update_registered_skill_visual(instance, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
