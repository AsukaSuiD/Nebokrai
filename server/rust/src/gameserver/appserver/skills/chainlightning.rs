//! Цепная молния CChainLightning (0x13E).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/chainlightning.cpp.
//! Player и Monster исполняют один зарегистрированный навык. Vec заменяет
//! два исходных STL-вектора, сохраняя их жизнь внутри callbacks и очистку End.
//! Begin создаёт visual перед Check; Check требует только U, таблицу и reuse,
//! затем запрещает движение. MP, путь и контакт принадлежат первому AI.
//!
//! AI захватывает U и таблицу, превращает свежую S в точку, поворачивает U,
//! строит forced-MAX путь, удаляет начальную клетку U и ограничивает длину.
//! Пустой путь даёт End0; MP→OnChangeStates предшествуют visual0/1.
//! Следующая клетка читается из живого пути; перед BLOCK2 обход прекращается.
//! Список целей текущей клетки дочитывается даже после вложенного End.
//! FullMove/admission предшествуют dedup, без отдельного IsDied. Attack исключает
//! самоцель и добавляет указатель до Calculate/receipt/RP; AI добавляет его ещё
//! раз после возврата, в том числе после вложенного End. Идентичность игрока
//! глобальна, остальных объектов — региональная. Формула общая с Lightning.
//!
//! Завершение ждёт строгий unsigned start+interval < now, возвращает движение,
//! публикует visual3 и вызывает End1. End сбрасывает флаги, путь и dedup до
//! двух свежих GetU и Move1; базовый Attack End получает исходный аргумент.
//! Здесь нет CAN, prepared, самостоятельного снаряда или снимка всего боя.
//! Visual1 передаёт последнюю клетку обрезанного пути, не исходную точку.

use super::basemagic::{SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::directelementattack::apply_direct_element_attack;
use super::flash::cell_views;
use super::kernel::{SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{spend_cast_mana, terminal};
use super::skillfactory::SkillOwner;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{ChainLightningProgress, ChainLightningExecutionState};

pub(crate) const CHAIN_LIGHTNING_SKILL_ID: u32 = 0x13e;
const ACTION_INTERVAL: u32 = 10_009;

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(source) = original_user else { return false; };
    if resolve_state_move_shape(game, source.0, source.1).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if source.1.object_type == 400 { game.send_skill_system_info(source.1.id, b"GS0278"); }
        return false;
    }
    let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    source.set_moveable(false);
    if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::chain_lightning_progress_mut) {
        progress.clear_end_paths();
    }
    true
}

fn check_attack_path(game: &mut CGame, instance: RegisteredSkill, maximum: u32) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(first) = skill.chain_lightning_progress().and_then(|progress| progress.path.first()).copied() else { return; };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else {
        if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::chain_lightning_progress_mut) {
            progress.path.clear();
        }
        return;
    };
    let remove_first = first.0 == source.shape().get_tile_x().unwrap_or(i32::MIN)
        && first.1 == source.shape().get_tile_y().unwrap_or(i32::MIN);
    if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::chain_lightning_progress_mut) {
        if remove_first { progress.path.remove(0); }
        progress.path.truncate(maximum as usize);
    }
}

fn attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    source: (i32, ShapeIdentity), target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if std::ptr::eq(user, sufferer) { return; }
    let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::chain_lightning_progress_mut) else { return; };
    if progress.has_attacked(target) { return; }
    progress.append_target(target);
    apply_direct_element_attack(game, instance, source, target, runtime);
}

fn attack_path<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    source: (i32, ShapeIdentity), region: i32, runtime: &mut Runtime,
) {
    let mut index = 0;
    loop {
        let cell = game.registered_skill(instance).and_then(MoveShapeSkill::chain_lightning_progress)
            .and_then(|progress| progress.path.get(index)).copied();
        let Some((x, y, block)) = cell else { break; };
        if block == 2 { break; }
        for view in cell_views(game, region, x, y) {
            let Some(target) = resolve_state_move_shape(game, region, view.identity) else { continue; };
            let target = (target.shape().get_region_id(), target.shape().identity());
            if !game.live_skill_target_attackable_between(source, target) { continue; }
            let Some(progress) = game.registered_skill(instance).and_then(MoveShapeSkill::chain_lightning_progress) else { return; };
            if progress.has_attacked(target) { continue; }
            attack(game, instance, source, target, runtime);
            // Это второй native push_back, не лишний элемент set: callbacks
            // между двумя записями могут очистить список через End.
            if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::chain_lightning_progress_mut) {
                progress.append_target(target);
            }
        }
        index += 1;
    }
    if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::chain_lightning_progress_mut) {
        progress.attacked = true;
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
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let source = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        let player = (source.1.object_type == 400).then_some(source.1.id);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| resolve_state_move_shape(game, target.0, target.1))
            .map(|target| (target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN)));
        let destination = target.unwrap_or_else(|| skill.lifecycle().destination());
        if target.is_some()
            && let Some(skill) = game.registered_skill_mut(instance)
        {
            let previous = skill.lifecycle().destination();
            skill.lifecycle_mut().set_point_target(previous);
        }
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_destination(destination);
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let Some(progress) = skill.chain_lightning_progress_mut() else { return terminal(QueuedSkillExecutionState::Rejected); };
        progress.path = path;
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
        check_attack_path(game, instance, maximum);
        if game.registered_skill(instance).and_then(MoveShapeSkill::chain_lightning_progress)
            .is_none_or(|progress| progress.path.is_empty())
        {
            game.update_registered_skill_visual(instance, 2);
            if let Some(player) = player { game.send_skill_system_info(player, b"GS0298"); }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if !spend_cast_mana(game, instance, player, &properties) { return terminal(QueuedSkillExecutionState::Rejected); }
        game.update_registered_skill_visual(instance, 0);
        game.update_registered_skill_visual(instance, 1);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    if game.registered_skill(instance).and_then(MoveShapeSkill::chain_lightning_progress)
        .is_some_and(|progress| !progress.attacked)
    {
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if !user.shape().is_assigned_to_server_region() { return terminal(QueuedSkillExecutionState::Rejected); }
        let region = user.shape().get_region_id();
        if game.find_region(region).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
        attack_path(game, instance, source, region, runtime);
    }
    if game.registered_skill(instance).and_then(MoveShapeSkill::chain_lightning_progress)
        .is_none_or(|progress| !progress.attacked)
    { return terminal(QueuedSkillExecutionState::Pending); }
    let interval = properties.query_property(ACTION_INTERVAL);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() <= started.wrapping_add(interval) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    game.update_registered_skill_visual(instance, 3);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_chain_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ChainLightning,
        |game, instance, _, runtime| check_cast(game, instance, original_user, runtime),
        |dispatch, started| ChainLightningExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

struct ChainLightningSkill;
impl RegisteredStateSkill for ChainLightningSkill {
    const ID: u32 = CHAIN_LIGHTNING_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::ChainLightning;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn prepare_monster(skill: &mut MoveShapeSkill) {
        skill.set_monster_progress(ChainLightningProgress::default());
    }

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(instance) else { return false; };
        let (region, identity) = skill.lifecycle().user();
        let user = resolve_state_move_shape(game, region, identity)
            .map(|user| (user.shape().get_region_id(), user.shape().identity()));
        check_cast(game, instance, user, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_chain_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<ChainLightningSkill, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}

pub(crate) fn publish_chain_lightning_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CChainLightning
        || skill.visual_effect().is_none_or(|effect|
            effect.kind() != SkillVisualEffectKind::ChainLightning || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity).map(|source| source.shape()) else { return; };
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 4 | 7 | 8 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if mode == 1 {
        // При пустом пути native читает память перед vector.end(); этот
        // недостижимый в штатном AI случай не превращается в фиктивную точку.
        let Some(&(x, y, _)) = skill.chain_lightning_progress().and_then(|progress| progress.path.last()) else { return; };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
