//! Немедленные навыки TaiJi, Origin и EnlargeFullMiss/MaxHp/MaxMp.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/taiji.cpp,
//! origin.cpp и enlargefullmiss.cpp/enlargemaxhp.cpp/enlargemaxmp.cpp.
//!
//! Begin записывает общую базу, проверяет только исходный U и свежие свойства,
//! затем включает фазу. Здесь нет reuse-допуска, MP, visual или Move.
//! Игрок, активный и фоновый монстр исполняют один зарегистрированный экземпляр.
//! AI читает свойства до GetU; при NULL U использует GetS, не держателя навыка.
//! Отсутствие обоих участников или свойств вызывает End0.
//!
//! Установку независимых состояний выполняет immediatestateinstallation:
//! TaiJi/Origin сохраняют прежнюю позицию после первичного Begin нового объекта,
//! Enlarge завершают прежний объект до создания нового и добавляют его в конец.
//! UpdateProperty не заменяется OnChangeStates. End1 сбрасывает фазу и выполняет
//! общий StateEnd с AfterUse и свежими часами, не меняя движение и CAN.
//! SlotMap сохраняет идентичность навыка через callbacks без копии исполнения.
//! Swordship и WuXing используют ниже только прежний monster-dispatch;
//! их отдельные AI и порядок завершения не входят в это семейство.

use super::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use super::enlargemaxhp::ENLARGE_MAX_HP_SKILL_ID;
use super::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use super::immediatestateinstallation::apply_immediate_state;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::origin::ORIGIN_SKILL_ID;
use super::playercast::execute_registered_player_cast_without_visual;
use super::rangedweaponcast::terminal;
use super::stateskill::end_state_skill;
use super::taiji::TAIJI_SKILL_ID;
use super::wuxing::is_wuxing_skill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};

pub(crate) const fn is_property_state_skill(skill_id: u32) -> bool {
    matches!(skill_id, TAIJI_SKILL_ID | ORIGIN_SKILL_ID | ENLARGE_FULL_MISS_SKILL_ID
        | ENLARGE_MAX_HP_SKILL_ID | ENLARGE_MAX_MP_SKILL_ID)
}

pub(crate) const fn is_immediate_state_skill(skill_id: u32) -> bool {
    is_property_state_skill(skill_id) || is_wuxing_skill(skill_id)
}

pub(crate) fn check_immediate_state_cast(
    game: &CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
) -> bool {
    original_user.is_some() && game.registered_skill(instance)
        .is_some_and(|skill| game.skill_base_properties(skill.id(), skill.level()).is_some())
}

pub(crate) fn run_immediate_state_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity).or_else(|| {
        let (region, identity) = resolve_skill_sufferer(game, skill.lifecycle())?;
        resolve_state_move_shape(game, region, identity)
    });
    let Some(source) = source.map(|shape| (shape.shape().get_region_id(), shape.shape().identity())) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let _ = apply_immediate_state(game, source, skill_id, &properties, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_property_state_skill(dispatch.skill_id()) { return terminal(QueuedSkillExecutionState::Rejected); }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast_without_visual(
        game, player_id, instance, dispatch, runtime,
        |game, instance, _, _| check_immediate_state_cast(game, instance, original_user),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_immediate_state_ai,
    )
}

pub(crate) enum MonsterImmediateSkill { State, Swordship, PlayerOnly }

impl MonsterImmediateSkill {
    pub(crate) fn from_skill_id(skill_id: u32) -> Option<Self> {
        if super::swordship::is_swordship_skill(skill_id) { Some(Self::Swordship) }
        else if is_wuxing_skill(skill_id) { Some(Self::PlayerOnly) }
        else if is_property_state_skill(skill_id) { Some(Self::State) }
        else { None }
    }

    pub(crate) const fn has_effect(&self) -> bool { !matches!(self, Self::PlayerOnly) }

    pub(crate) fn execute<Runtime: GameMainLoopRuntime>(
        self, game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
        skill_id: u32, skill_level: i32, runtime: &mut Runtime,
    ) -> bool {
        if matches!(self, Self::State) {
            return execute_monster_immediate_state(game, owner, monster_id, skill_id, skill_level, runtime);
        }
        let Some(region) = owner.as_mut().map(ServerRegionOwner::base_mut) else { return false; };
        if !region.find_monster_by_id(monster_id).is_some_and(|monster|
            monster.move_shape().immediate_skill_started(skill_id, game.skill_factory())) { return false; }
        match self {
            Self::State => unreachable!(),
            Self::Swordship => super::swordship::execute_monster_auto_start_swordship(
                game, owner, monster_id, skill_id, skill_level,
            ),
            Self::PlayerOnly => {
                let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false; };
                monster.move_shape_mut().finish_immediate_skill(skill_id, game.skill_factory());
                true
            }
        }
    }
}

pub(crate) fn execute_monster_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    skill_id: u32, _skill_level: i32, runtime: &mut Runtime,
) -> bool {
    let Some(monster) = owner.as_ref().and_then(|region| region.base().find_monster_by_id(monster_id)) else { return false; };
    let source = (monster.move_shape().shape().get_region_id(), monster.move_shape().shape().identity());
    game.with_published_region(owner, |game| {
        let Some(instance) = game.registered_move_shape_skill(source.0, source.1, skill_id) else { return false; };
        let outcome = run_immediate_state_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => { end_state_skill(game, instance, 0, runtime); }
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => { end_state_skill(game, instance, 1, runtime); }
            _ => {}
        }
        true
    }).unwrap_or(false)
}
