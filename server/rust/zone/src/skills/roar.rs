//! Боевой клич CRoar (0x83): Check/AI, обход клеток и границы окна.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/roar.cpp/.h`. Машинные якоря: Begin
//! `0x14A7D0`, AI `0x14B060`; окно обхода `roar_bounds` —
//! CRoar::AI VA 0x0054B1CC–0x0054B23E (подтверждено ранее); состояние
//! принадлежит `roarstate.cpp/.h` (Serialize 5-fold `0x1F65F0`).
//! Прежний переходный владелец — `src/gameserver/appserver/skills/roar.rs`;
//! тела Check/AI и клеточного обхода перенесены буквально порцией №6c
//! «self/zone-касты» (разведка — запись аудита «Zone skills: машинная
//! разведка battlefairy-навыков (порция №6)», 26 сентября 2026).
//!
//! Общий Attack Begin сохраняет раннее время и visual loop1. Check получает
//! исходного игрока, проверяет reuse, оружие категории 1 и signed MP;
//! нулевая цена разрешена. Успех запрещает движение, отказ дополнительно
//! отправляет visual2 перед End(0). AI каждый раз требует живого игрока,
//! но не повторяет оружейную проверку. MP списывается до OnChangeStates,
//! затем CAN, visual0 и condition; срок start+delay сравнивается unsigned.
//!
//! После visual1 NULL-регион оставляет исполнение ожидающим. Обход берёт
//! живые X/Y источника и ограничивает окно 5×5 размерами региона включительно.
//! X — внешний цикл, каждая клетка заново вызывает одиночный GetShape после
//! предыдущего наложения. Нет общего снимка целей, дедупликации и начисления RP.
//! Полный CMoveShape проходит self/death/live-admission до фильтра типа.
//! Для пары игроков SAFE читается по свежим координатам U и S; OnFirstSkill
//! ещё раз читает клетку U. Затем первый state ID83 заменяется с параметрами
//! замороженной таблицы AI; Begin, порядок append/Update и DB у roarstate.
//! End(1) вызывается после обхода даже без целей; отказы AI дают End(0).
//! Общий End сбрасывает фазу до freshU Move1 и Attack End(actualarg).
//! Kernel и арена сохраняют единственное каноническое исполнение.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::{SelfCastGame,
//! SelfCastContact, SelfCastPlayer}` реализован у прежнего владельца; общий
//! зарегистрированный вход и visual2 по отказу Check остаются у `playercast`
//! делегата; одиночный GetShape клетки, SAFE-гейт и размеры региона —
//! одноимённые швы hub (порядок старого тела сохранён, обе ветви пропуска
//! завершали клетку молча). Создание состояния — `skills/roarstate.rs`.

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::roarstate::replace_roar_state;
use super::selfcast::{
    SelfCastContact, SelfCastExecutionOutcome, SelfCastGame, SelfCastMoveShape, SelfCastPlayer,
};

pub const ROAR_SKILL_ID: u32 = 0x83;
const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoarBounds {
    pub minimum_x: i32,
    pub minimum_y: i32,
    pub maximum_x: i32,
    pub maximum_y: i32,
}

/// Максимумы ограничены размером региона и обходятся включительно.
pub fn roar_bounds(source_x: i32, source_y: i32, width: i32, height: i32) -> RoarBounds {
    RoarBounds {
        minimum_x: source_x.wrapping_sub(2).max(0),
        minimum_y: source_y.wrapping_sub(2).max(0),
        maximum_x: source_x.wrapping_add(2).min(width),
        maximum_y: source_y.wrapping_add(2).min(height),
    }
}

fn weapon_is_valid<Game: SelfCastGame>(game: &Game, player: &Game::Player) -> bool {
    game.player_weapon_addon_category(player).is_some_and(|category| category == 1)
}

fn failure<Game: SelfCastGame>(game: &mut Game, instance: Game::SkillAddress, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0287", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let loss = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", loss);
}

pub fn check_cast<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, now_milliseconds: fn() -> u32,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    let Some(player) = game.find_player(player_id) else { return false; };
    if !weapon_is_valid(game, player) {
        failure(game, instance, player_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            mana_failure(game, instance, player_id, &properties);
            return false;
        }
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn apply_cell<Game, Runtime>(
    game: &mut Game, source: (i32, ShapeIdentity), region_id: i32,
    cell: (i32, i32), properties: &CSkillBaseProperties, runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
)
where
    Game: SelfCastContact<Runtime>,
{
    let target = game.area_shape_view_at(region_id, cell.0, cell.1)
        .and_then(|view| game.resolve_state_move_shape(region_id, view.identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let Some(target) = target else { return; };
    if target.1 == source.1
        || game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
        || !game.live_skill_target_attackable(target.0, source.1, target.1)
    { return; }
    if source.1.object_type == PLAYER_TYPE && target.1.object_type == PLAYER_TYPE {
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(source_safe) = game.region_cell_safe(region_id, source_x, source_y) else { return; };
        if source_safe { return; }
        let Some(sufferer) = game.resolve_state_move_shape(target.0, target.1) else { return; };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(target_safe) = game.region_cell_safe(region_id, target_x, target_y) else { return; };
        if target_safe { return; }
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return; };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        game.skill_first_attack_at_position(
            source.1.id, target.1.id, region_id, (source_x, source_y), runtime,
        );
    }
    if matches!(target.1.object_type, 400 | 600 | 602) {
        let _ = replace_roar_state(game, source, target, properties, &mut || now_milliseconds());
    }
}

pub fn run_ai<Game, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> SelfCastExecutionOutcome
where
    Game: SelfCastContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return SelfCastExecutionOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return SelfCastExecutionOutcome::Pending;
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return SelfCastExecutionOutcome::Rejected; };
    let source = (user.shape().get_region_id(), user.shape().identity());
    if source.1.object_type != PLAYER_TYPE || game.find_player(source.1.id).is_none_or(|player| player.is_dead()) {
        return SelfCastExecutionOutcome::Rejected;
    }
    if stage == SkillStage::Begin {
        let Some(player) = game.find_player(source.1.id) else { return SelfCastExecutionOutcome::Rejected; };
        let mana = player.mana();
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, source.1.id, &properties);
            return SelfCastExecutionOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
        game.publish_player_states(source.1.id);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return SelfCastExecutionOutcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    if now_milliseconds() < started.wrapping_add(delay) { return SelfCastExecutionOutcome::Pending; }
    game.update_registered_skill_visual(instance, 1);
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else { return SelfCastExecutionOutcome::Rejected; };
    if !user.shape().is_assigned_to_server_region() { return SelfCastExecutionOutcome::Pending; }
    let region_id = user.shape().get_region_id();
    let Some((width, height)) = game.region_dimensions(region_id) else { return SelfCastExecutionOutcome::Pending; };
    let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
    let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
    let bounds = roar_bounds(source_x, source_y, width, height);
    for x in bounds.minimum_x..=bounds.maximum_x {
        for y in bounds.minimum_y..=bounds.maximum_y {
            apply_cell(game, source, region_id, (x, y), &properties, runtime, now_milliseconds);
        }
    }
    SelfCastExecutionOutcome::Completed
}

/// Дисциплина dispatch-а сохранена у зарегистрированного входа делегата.
pub const fn is_roar_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == ROAR_SKILL_ID
}
