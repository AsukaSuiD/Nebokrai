//! Периодическое лечение CHeal/CHeal2/CSuperHeal/CSuperHeal2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/heal{,2}.cpp
//! и superheal{,2}.cpp. Зарегистрированный экземпляр игрока или монстра хранит
//! единственную базу и фазу; отдельного payload и снимка цели нет.
//! Объектный Begin проверяет исходные U/S, координатный — свежую S либо U
//! после loop1 visual. Check требует обе фигуры, абсолютный reuse и свежий
//! GetTargetPath даже при самоцели. Дальность проверяется только для разных
//! U/S; препятствия не запрещают лечение. Player MP0 — тихий отказ,
//! положительная цена проверяется по знаку DWORD-разности перед Move0.
//! Источник другого типа проходит без MP и запрета движения.
//!
//! Каждый AI удерживает таблицу свойств и найденные U/S через callbacks.
//! Отсутствующая S заменяется свежим U; обычный неприручённый монстр без
//! Carriage AI дополнительно переназначает базовые type/id цели на U.
//! Только смерть выбранной S отменяет лечение. Первый AI выполняет
//! MP→OnChangeStates→CAN→S.Y/X→U.Y/X→направление→visual0→condition.
//! После unsigned start+delay visual1 заново разрешает S либо U, тогда как
//! состояние устанавливается на захваченную этим AI цель.
//!
//! Прибавка Player считается после visual1: COEFF→живой уровень оружия,
//! unsigned wrapping-произведение с 0.01f сохраняется в f32; CONST прибавляется
//! с отдельным сохранением f32. У источника другого типа прибавка равна нулю.
//! После End/destructor первого прежнего состояния значение усекается FISTP,
//! затем читаются FREQ→PERSIST. Новый primary Begin предшествует append;
//! отдельного UpdateProperty или пакета состояния здесь нет.
//! SuperHeal заменяет только Heal; SuperHeal2 хранится у выбранной S,
//! но primary Begin получает U/U. Эти различия принадлежат healstate.
//! Выпуск всегда завершает навык End1, ранние отказы — End0. Общий End
//! сбрасывает фазу, разрешает свежий U Move1 и сохраняет исходный аргумент.
//! Vec, общий kernel и каноническое хранилище заменяют native STL/указатели.

use super::fightdefense::truncate_original;
use super::heal2::HEAL_2_SKILL_ID;
use super::healstate::{HealState, replace_heal_state};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastPathBlock, check_cast_mana, check_skill_path, spend_cast_mana, terminal,
};
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill, publish_state_skill_visual,
};
use super::superheal::SUPER_HEAL_SKILL_ID;
use super::superheal2::SUPER_HEAL_2_SKILL_ID;
use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

pub(crate) const HEAL_SKILL_ID: u32 = 0xd3;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const CONST: u32 = 20_010;
const HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_AFFECT_FREQUENCY: u32 = 5_002;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;

pub(crate) const fn is_heal_skill(skill_id: u32) -> bool {
    matches!(skill_id, HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID)
}

fn resolved_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity)?.shape();
    Some((user.get_region_id(), user.identity()))
}

pub(crate) fn check_heal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let (Some(user), Some(target)) = (original_user, original_target) else { return false; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return false; };
    let Some(target) = resolve_state_move_shape(game, target.0, target.1) else { return false; };
    let same_target = std::ptr::eq(source, target);
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !same_target && !check_skill_path(
        game, instance, &properties, &path, player, CastPathBlock::Ignore,
    ) { return false; }
    check_cast_mana(game, instance, source, &properties)
}

pub(crate) fn run_heal_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source = resolved_user(game, skill);
    let mut target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    if target.is_none() {
        target = resolved_user(game, skill);
    } else if target.is_some_and(|(region, identity)| {
        identity.object_type == MONSTER_TYPE && game.find_region(region)
            .and_then(|region| region.base().find_monster_by_id(identity.id))
            .is_some_and(|monster| !monster.is_tamed() && !matches!(
                monster.active_ai(), Some(ActiveMonsterAi::Carriage | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)),
            ))
    }) {
        target = resolved_user(game, skill);
        if let Some(target) = target
            && let Some(skill) = game.registered_skill_mut(instance)
        { skill.lifecycle_mut().set_sufferer_identity(target.1); }
    }
    let (Some(source), Some(target)) = (source, target) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        if source != target {
            let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else {
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
            let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
            let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
            let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
            let direction = get_line_direction(source_x, source_y, target_x, target_y);
            if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
                user.shape_mut().set_direction(direction);
            }
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    let hp_gain = if let Some(user) = player.and_then(|player| game.find_player(player)) {
        let coefficient = properties.query_property(HEAL_RECOVER_COEFFICIENT);
        let weapon = user.weapon_damage_level(game.goods_factory()) as u32;
        let scaled = (f64::from(coefficient.wrapping_mul(weapon)) * f64::from(0.01_f32)) as f32;
        let constant = properties.query_property(CONST);
        (f64::from(constant) + f64::from(scaled)) as f32
    } else { 0.0 };
    let _ = replace_heal_state(game, source, target, skill_id, || {
        let hp_gain = truncate_original(f64::from(hp_gain)) as u32;
        let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
        let keep = properties.query_property(STATE_PERSIST_TIME);
        HealState::new(skill_id, keep, frequency, hp_gain)
    }, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_heal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target|
            original_user.and_then(|source| game.player_skill_begin_object(source.0, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Heal,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill|
                    resolve_skill_sufferer(game, skill.lifecycle()).or_else(|| resolved_user(game, skill)))
            } else { original_target };
            let accepted = check_heal_cast(game, instance, original_user, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_heal_ai,
    )
}

struct HealSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for HealSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Heal;
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(instance) else { return false; };
        let user = resolved_user(game, skill);
        let target = begin_target.resolve(game, skill, true);
        check_heal_cast(game, instance, user, target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_heal_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_heal<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<HealSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

pub(crate) fn publish_heal_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    match skill.id() {
        HEAL_SKILL_ID => publish_state_skill_visual::<HealSkill<HEAL_SKILL_ID>>(game, skill, mode),
        HEAL_2_SKILL_ID => publish_state_skill_visual::<HealSkill<HEAL_2_SKILL_ID>>(game, skill, mode),
        SUPER_HEAL_SKILL_ID => publish_state_skill_visual::<HealSkill<SUPER_HEAL_SKILL_ID>>(game, skill, mode),
        SUPER_HEAL_2_SKILL_ID => publish_state_skill_visual::<HealSkill<SUPER_HEAL_2_SKILL_ID>>(game, skill, mode),
        _ => {}
    }
}
