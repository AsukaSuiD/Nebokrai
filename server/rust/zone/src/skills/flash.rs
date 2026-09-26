//! Рывок CFlash (0x69). Источник: точная пара `gameserver.exe` `4F5C98E0…` +
//! `GameServer.pdb` (RSDS match), appserver/skills/flash.cpp; тела Check/AI,
//! helpers и visual перенесены буквально. Машинная сверка подтверждает:
//! AI-стадии Flash — порядок
//! GetTargetPath → direction → path → player-only weapon addon==2 → RageBreak
//! id `0x6E` → MP → RP → OnChangeStates → teleport back → VE(1) → condition=1
//! → attack-фаза `condition && !attacked` (region else — End(0) без VE) →
//! tick ≤ started+10009 → pending; иначе SetMoveable(1) → VE(3) → End(1);
//! visual — прямой 16-switch с `{0→1, 1→2, 3→3}` и personal-множеством
//! `{2,4,7,8,10,11,13,14,15}`; GS-строки 0278/0288/0289/0301-0304 байт-сверены.
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
//!
//! Объявленные швы — фасады `DashSkillGame`/`DashSkillContact`
//! (`skills/dash.rs`, реализация у владельца старого пакета); find_state_position
//! RageBreak свёрнут в `move_shape_state_position`, развёрнутый результат
//! End состояния отбрасывается, как и раньше. Оставшийся UNKNOWN — место
//! push в Attack-списке CFlash (низкий риск, зафиксирован разведкой).

use nebokrai_shared::runtime::get_line_direction;

use crate::combat::MasterInfo;
use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::CShape;

use super::baseattackruntime::SKILL_USAGE_REUSE_DELAY_TIME;
use super::dash::{
    DashSkillContact, DashSkillExecutionOutcome, DashSkillGame, DashSkillMoveShape,
    DashSkillPlayer, apply_dash_attack, check_dash_path, publish_dash_visual,
};
use super::execution::{FlashExecutionState, RegisteredSkillRecord};
use super::pillar::PILLAR_SKILL_ID;
use super::skill_is_restored;
use super::skillfactory::SkillOwner;
use super::visualeffect::SkillVisualEffectKind;
use super::SkillStage;

pub const FLASH_SKILL_ID: u32 = 0x69;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const ACTION_INTERVAL: u32 = 10_009;
const RAGE_BREAK_STATE_ID: u32 = 0x6e;

/// Общий предикат меча семейства (category 2): живой addon слота 2 оружия
/// по `GAP_WEAPON_CATEGORY` (резолв фабрики и отсутствия — у шва владельца).
pub fn weapon_is_valid<Game: DashSkillGame>(game: &Game, player: &Game::Player) -> bool {
    game.player_weapon_addon_category(player).is_some_and(|category| category == 2)
}

fn fail<Game: DashSkillGame>(game: &mut Game, instance: Game::SkillAddress, player_id: Option<i32>, mode: u32, text: &[u8]) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id { game.send_skill_system_info(player_id, text); }
}

fn fail_resource<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    properties: &CSkillBaseProperties,
    usage: u32,
) {
    let (mode, text): (u32, &[u8]) = if usage == USER_MP_LOSE { (7, b"GS0288") } else { (8, b"GS0289") };
    game.update_registered_skill_visual(instance, mode);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

pub fn publish_flash_visual<Game: DashSkillGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if skill.owner() != SkillOwner::CFlash { return; }
    let destination = skill.player_state::<FlashExecutionState>()
        .and_then(|state| state.path.last()).map(|cell| (cell.0, cell.1));
    publish_dash_visual(game, skill, mode, SkillVisualEffectKind::Flash, destination);
}

pub fn master_info<Player: DashSkillPlayer>(player: &Player) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo { master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(permissions.player), permitted_to_kill_teammate: i32::from(permissions.teammate), permitted_to_kill_guild_member: i32::from(permissions.guild_member), permitted_to_kill_criminal: i32::from(permissions.criminal) }
}

pub fn target_level<Game: DashSkillGame>(game: &Game, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(DashSkillPlayer::level),
        MONSTER_TYPE => game.dash_monster_level(region_id, target.id),
        _ => None,
    }
}

pub fn check_cast<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    now_milliseconds: fn() -> u32,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(|skill| skill.last_used_ms()) else { return false; };
    if !skill_is_restored(last_used, reuse, now_milliseconds()) {
        fail(game, instance, Some(player_id), 13, b"GS0278");
        return false;
    }
    if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
        fail(game, instance, Some(player_id), 14, b"GS0301");
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let Some(mana) = game.find_player(player_id).map(DashSkillPlayer::mana) else { return false; };
        let cost = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(cost) as i32) < 0 {
            fail_resource(game, instance, player_id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let Some(rp) = game.find_player(player_id).map(DashSkillPlayer::rp) else { return false; };
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
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return DashSkillExecutionOutcome::Rejected; };
    let (region, identity) = skill.lifecycle().user();
    let user = game.resolve_state_move_shape(region, identity).map(|user| (user.shape().get_region_id(), user.shape().identity()));
    let target = game.resolve_skill_sufferer(skill.lifecycle());
    let (Some(user), Some(_target)) = (user, target) else {
        game.update_registered_skill_visual(instance, 10);
        return DashSkillExecutionOutcome::Rejected;
    };
    let player_id = (user.1.object_type == PLAYER_TYPE).then_some(user.1.id);
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).is_some_and(|state| !state.condition_checked) {
        let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
        let destination = game.resolve_skill_sufferer(skill.lifecycle())
            .and_then(|target| game.resolve_state_move_shape(target.0, target.1))
            .map(|target| (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN)))
            .unwrap_or_else(|| skill.lifecycle().destination());
        let Some(source) = game.resolve_state_move_shape(user.0, user.1).map(|source| source.shape()) else { return DashSkillExecutionOutcome::Rejected; };
        let source_y = source.get_tile_y().unwrap_or(i32::MIN);
        let source_x = source.get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(source) = game.resolve_state_move_shape_mut(user.0, user.1) { source.shape_mut().set_direction(direction); }
        let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
        let path = game.skill_target_path(skill.lifecycle());
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let path = check_dash_path(game, user, path, maximum, true, false, runtime);
        let empty = path.is_empty();
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<FlashExecutionState>()) else { return DashSkillExecutionOutcome::Rejected; };
        state.path = path;
        if empty {
            fail(game, instance, player_id, 2, b"GS0303");
            return DashSkillExecutionOutcome::Rejected;
        }
        if let Some(player_id) = player_id {
            if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
                fail(game, instance, Some(player_id), 14, b"GS0301");
                return DashSkillExecutionOutcome::Rejected;
            }
            let previous = game.move_shape_state_position(user.0, user.1, RAGE_BREAK_STATE_ID);
            let Some(position) = previous else {
                fail(game, instance, Some(player_id), 4, b"GS0304");
                return DashSkillExecutionOutcome::Rejected;
            };
            game.end_move_shape_state_at(user.0, user.1, position);
            let Some(mana) = game.find_player(player_id).map(DashSkillPlayer::mana) else { return DashSkillExecutionOutcome::Rejected; };
            let mp_loss = properties.query_property(USER_MP_LOSE);
            let mana = mana.wrapping_sub(mp_loss);
            if (mana as i32) < 0 {
                fail_resource(game, instance, player_id, &properties, USER_MP_LOSE);
                return DashSkillExecutionOutcome::Rejected;
            }
            if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana); }
            let Some(rp) = game.find_player(player_id).map(DashSkillPlayer::rp) else { return DashSkillExecutionOutcome::Rejected; };
            let rp_loss = properties.query_property(USER_RP_LOSE);
            let rp = u32::from(rp).wrapping_sub(rp_loss);
            if (rp as i32) < 0 {
                fail_resource(game, instance, player_id, &properties, USER_RP_LOSE);
                return DashSkillExecutionOutcome::Rejected;
            }
            if let Some(player) = game.find_player_mut(player_id) { player.set_rp(rp as u16); }
            game.publish_player_states(player_id);
        }
        let Some(destination) = game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).and_then(|state| state.path.last()).copied() else { return DashSkillExecutionOutcome::Rejected; };
        game.relocate_region_shape(user.0, user.1, destination.0, destination.1);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<FlashExecutionState>()) {
            state.condition_checked = true;
            let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).is_some_and(|state| state.condition_checked && !state.attacked) {
        let Some(region_id) = game.resolve_state_move_shape(user.0, user.1)
            .map(|source| source.shape()).filter(|source| source.is_assigned_to_server_region())
            .map(CShape::get_region_id).filter(|region| game.dash_region_size(*region).is_some())
        else { return DashSkillExecutionOutcome::Rejected; };
        let mut index = 0usize;
        loop {
            let Some(state) = game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()) else { break; };
            if index >= state.path.len().saturating_sub(1) { break; }
            if properties.query_property(TARGET_MAX_DISTANCE) as usize <= index { break; }
            let (x, y, _) = state.path[index];
            for view in game.dash_cell_views(region_id, x, y) {
                let target = view.identity;
                let Some(target_shape) = game.resolve_state_move_shape(region_id, target) else { continue; };
                let Some(source_shape) = game.resolve_state_move_shape(user.0, user.1) else { continue; };
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
    if game.registered_skill(instance).and_then(|skill| skill.player_state::<FlashExecutionState>()).is_none_or(|state| !state.attacked) { return DashSkillExecutionOutcome::Pending; }
    let interval = properties.query_property(ACTION_INTERVAL);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return DashSkillExecutionOutcome::Rejected; };
    if now_milliseconds() <= started.wrapping_add(interval) { return DashSkillExecutionOutcome::Pending; }
    if let Some(source) = game.resolve_state_move_shape_mut(user.0, user.1) { source.set_moveable(true); }
    game.update_registered_skill_visual(instance, 3);
    DashSkillExecutionOutcome::Completed
}
