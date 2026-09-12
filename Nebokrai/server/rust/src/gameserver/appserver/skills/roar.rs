//! Боевой клич CRoar (0x83).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/roar.cpp.
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

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::roarstate::replace_roar_state;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    RegionShapeResolver,
};

pub(crate) const ROAR_SKILL_ID: u32 = 0x83;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0287", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let loss = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", loss);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
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

fn apply_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), region_id: i32,
    cell: (i32, i32), properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let (area_width, area_height) = game.area_dimensions();
    let target = game.find_region(region_id).and_then(|region| {
        let resolver = RegionShapeResolver { game, owner: region };
        region.base().get_shape(cell.0, cell.1, area_width, area_height, &resolver).ok().flatten()
    }).and_then(|view| resolve_state_move_shape(game, region_id, view.identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let Some(target) = target else { return; };
    if target.1 == source.1
        || game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
        || !game.live_skill_target_attackable(target.0, source.1, target.1)
    { return; }
    if source.1.object_type == PLAYER_TYPE && target.1.object_type == PLAYER_TYPE {
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(region) = game.find_region(region_id) else { return; };
        if region.get_security(source_x, source_y).ok() == Some(RegionSecurity::SAFE) { return; }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        if region.get_security(target_x, target_y).ok() == Some(RegionSecurity::SAFE) { return; }
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let _ = game.player_on_first_skill_at_position(
            source.1.id, target.1.id, region_id, source_x, source_y, runtime,
        );
    }
    if matches!(target.1.object_type, 400 | 600 | 602) {
        let _ = replace_roar_state(game, source, target, properties, &mut || runtime.now_milliseconds());
    }
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let source = (user.shape().get_region_id(), user.shape().identity());
    if source.1.object_type != PLAYER_TYPE || game.find_player(source.1.id).is_none_or(CPlayer::is_dead) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        let Some(player) = game.find_player(source.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let mana = player.mana();
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, source.1.id, &properties);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
        game.publish_player_states(source.1.id);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !user.shape().is_assigned_to_server_region() { return terminal(QueuedSkillExecutionState::Pending); }
    let region_id = user.shape().get_region_id();
    let Some(region) = game.find_region(region_id) else { return terminal(QueuedSkillExecutionState::Pending); };
    let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
    let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
    let minimum_x = source_x.wrapping_sub(2).max(0);
    let minimum_y = source_y.wrapping_sub(2).max(0);
    let maximum_x = source_x.wrapping_add(2).min(region.base().region.width);
    let maximum_y = source_y.wrapping_add(2).min(region.base().region.height);
    for x in minimum_x..=maximum_x {
        for y in minimum_y..=maximum_y {
            apply_cell(game, source, region_id, (x, y), &properties, runtime);
        }
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_roar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != ROAR_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, player_id, runtime| {
            let accepted = check_cast(game, instance, player_id, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
