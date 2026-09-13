//! Божественное благословение CGodBless/CGodBless2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godbless{,2}.cpp.
//! Оба Begin сохраняют исходную цель в общей базе, создают visual loop1
//! и проверяют только исходного U. Нет проверки S, пути или оружия.
//! После абсолютного reuse источник не типа Player проходит без Move0;
//! Player MP0 означает тихий отказ, иначе signed DWORD-разность допускает Move0.
//! Отказ Begin вызывает End0 без дополнительного visual2.
//!
//! AI удерживает одну таблицу и найденные U/S через callbacks. GodBless
//! при NULL S использует захваченного U; GodBless2 требует именно Monster
//! при каждом входе AI, иначе Player получает visual10/GS0305 и End0.
//! Обычный неприручённый монстр без Carriage AI заменяется свежим U,
//! с записью базовых type/id S. Поэтому GodBless2 после такой замены и
//! ожидания задержки может отвергнуть уже изменившуюся S на следующем AI.
//! Нет проверки смерти или смены направления. Первый AI выполняет
//! MP→OnChangeStates→CAN→visual0→condition; выпуск ждёт unsigned start+delay.
//!
//! После visual1 захваченный U даёт живой уровень оружия либо ноль.
//! Свойства читаются в порядке MIN_COEFF→MIN→MAX_COEFF→MAX→ELEMENT_COEFF→ELEMENT.
//! Каждое unsigned wrapping-произведение с 0.01f сохраняется в f32,
//! затем прибавление константы отдельно сохраняется в f32. Значения живут
//! через удаление прежнего состояния; только потом PERSIST→FISTP
//! ELEMENT/MAX/MIN→ctor→primary Begin→append→безусловный UpdateProperty.
//! Различие DelExStateByType у GodBless принадлежит общему установщику.
//! Выпуск заканчивается End1 даже при отказе state Begin; ранние отказы — End0.
//! Общий End сбрасывает фазу, разрешает свежий U либо S Move1 и сохраняет
//! исходный аргумент. Общий kernel и SlotMap исключают отдельную копию исполнения.

use super::fightdefense::truncate_original;
use super::godbless2::GOD_BLESS_2_SKILL_ID;
use super::godblessstate::GodBlessState;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{check_cast_mana, spend_cast_mana, terminal};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill, publish_state_skill_visual,
};
use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};

pub(crate) const GOD_BLESS_SKILL_ID: u32 = 0x12f;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const TARGET_ELEMENT_GAIN: u32 = 115;
const TARGET_MINIMUM_GAIN: u32 = 116;
const TARGET_MAXIMUM_GAIN: u32 = 117;
const TARGET_ELEMENT_COEFFICIENT: u32 = 120;
const TARGET_MINIMUM_COEFFICIENT: u32 = 121;
const TARGET_MAXIMUM_COEFFICIENT: u32 = 122;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;

pub(crate) const fn is_god_bless_skill(skill_id: u32) -> bool {
    matches!(skill_id, GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID)
}

fn resolved_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity)?.shape();
    Some((user.get_region_id(), user.identity()))
}

pub(crate) fn check_god_bless_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(user) = resolve_state_move_shape(game, region, identity).map(|user| user.shape()) else { return false; };
    let source = (user.get_region_id(), user.identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    check_cast_mana(game, instance, source, &properties)
}

fn gain(properties: &CSkillBaseProperties, coefficient: u32, constant: u32, weapon: u32) -> f32 {
    let coefficient = properties.query_property(coefficient);
    let scaled = (f64::from(coefficient.wrapping_mul(weapon)) * f64::from(0.01_f32)) as f32;
    let constant = properties.query_property(constant);
    (f64::from(constant) + f64::from(scaled)) as f32
}

pub(crate) fn run_god_bless_ai<Runtime: GameMainLoopRuntime>(
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
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let Some(source) = source else { return terminal(QueuedSkillExecutionState::Rejected); };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let monster = target.filter(|(_, identity)| identity.object_type == MONSTER_TYPE)
        .and_then(|(region, identity)| game.find_region(region)?.base().find_monster_by_id(identity.id));
    if skill_id == GOD_BLESS_2_SKILL_ID && monster.is_none() {
        if let Some(player) = player {
            game.update_registered_skill_visual(instance, 10);
            game.send_skill_system_info(player, b"GS0305");
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let ordinary_monster = monster.is_some_and(|monster| !monster.is_tamed() && !matches!(
        monster.active_ai(), Some(ActiveMonsterAi::Carriage | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)),
    ));
    let target = if ordinary_monster {
        let Some(target) = game.registered_skill(instance).and_then(|skill| resolved_user(game, skill)) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_sufferer_identity(target.1);
        }
        target
    } else { target.unwrap_or(source) };
    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
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
    let weapon = player.and_then(|player| game.find_player(player))
        .map_or(0, |player| player.weapon_damage_level(game.goods_factory()) as u32);
    let minimum = gain(&properties, TARGET_MINIMUM_COEFFICIENT, TARGET_MINIMUM_GAIN, weapon);
    let maximum = gain(&properties, TARGET_MAXIMUM_COEFFICIENT, TARGET_MAXIMUM_GAIN, weapon);
    let element = gain(&properties, TARGET_ELEMENT_COEFFICIENT, TARGET_ELEMENT_GAIN, weapon);
    let _ = game.install_god_bless_state(source, target, skill_id, || {
        let keep = properties.query_property(STATE_PERSIST_TIME);
        let element = truncate_original(f64::from(element)) as u32;
        let maximum = truncate_original(f64::from(maximum)) as u32;
        let minimum = truncate_original(f64::from(minimum)) as u32;
        GodBlessState::new(skill_id, keep, minimum, maximum, element)
    }, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_god_bless<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::GodBless,
        |game, instance, _, runtime| check_god_bless_cast(game, instance, original_user, runtime),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_god_bless_ai,
    )
}

struct GodBlessSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for GodBlessSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::GodBless;
    const VISUAL_FAILURES: &'static [u32] = if ID == GOD_BLESS_2_SKILL_ID { &[2, 7, 10, 13] } else { &[2, 7, 13] };
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let user = game.registered_skill(instance).and_then(|skill| resolved_user(game, skill));
        check_god_bless_cast(game, instance, user, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_god_bless_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_god_bless<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<GodBlessSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

pub(crate) fn publish_god_bless_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    match skill.id() {
        GOD_BLESS_SKILL_ID => publish_state_skill_visual::<GodBlessSkill<GOD_BLESS_SKILL_ID>>(game, skill, mode),
        GOD_BLESS_2_SKILL_ID => publish_state_skill_visual::<GodBlessSkill<GOD_BLESS_2_SKILL_ID>>(game, skill, mode),
        _ => {}
    }
}
