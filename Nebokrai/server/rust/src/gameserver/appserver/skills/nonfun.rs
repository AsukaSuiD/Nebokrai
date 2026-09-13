//! Зарегистрированное исполнение пятидесяти навыков CNonFun.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/nonfun.cpp и
//! производные CNonFun1..46, CNonFun60..62 с тем же виртуальным поведением.
//!
//! Все три Begin выполняют только общий Attack Begin и возвращают успех.
//! CheckCastCondition проверяет ненулевой U, но эти Begin его не вызывают.
//! Нет собственного visual, ресурсов, задержки или AutoStart. AI не меняет
//! цель; запись участников в Begin и боевой callback остаются в общем lifecycle.
//!
//! Каждый вызов AI без проверки фазы, U или свойств вызывает End(0).
//! Внешний End передаёт фактический аргумент общей базе: не сбрасывает CAN
//! или движение, а End(1) сохраняет оружейный AfterUse. Завершение команды
//! отдельно очищает исполнение, не меняя выбранный ID навыка.
//! Plain kernel связывает расписание с экземпляром SlotMap; собственных
//! полей исполнения у семейства нет.

use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast_without_visual;
use super::stateskill::{end_state_skill, state_skill_outcome};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};

pub(crate) const fn is_non_fun_skill(skill_id: u32) -> bool {
    matches!(skill_id, 0x384..=0x3b2 | 0x3c0..=0x3c2)
}

fn run_non_fun_ai<Runtime: GameMainLoopRuntime>(
    _game: &mut CGame, _instance: RegisteredSkill, _runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    // Общий playercast завершает этот owner с аргументом 0, не с обычным 1.
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_non_fun<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_non_fun_skill(dispatch.skill_id()) {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_player_cast_without_visual(
        game, player_id, instance, dispatch, runtime,
        |_, _, _, _| true,
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_non_fun_ai,
    )
}

pub(crate) fn execute_owned_monster_non_fun<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    let Some(region) = owner.as_mut().map(ServerRegionOwner::base_mut) else { return false; };
    let Some(monster) = region.find_monster_by_id(monster_id) else { return false; };
    let Some(skill_id) = monster.move_shape().current_skill(game.skill_factory()).map(|skill| skill.id()) else {
        return false;
    };
    if !is_non_fun_skill(skill_id) { return false; }
    let source = (monster.move_shape().shape().get_region_id(), monster.move_shape().shape().identity());
    if monster.current_active_attack_cast(game.skill_factory()).is_some() {
        return game.with_published_region(owner, |game| {
            let Some(instance) = game.registered_move_shape_skill(source.0, source.1, skill_id) else { return false; };
            let _ = run_non_fun_ai(game, instance, runtime);
            let _ = end_state_skill(game, instance, 0, runtime);
            true
        }).unwrap_or(false);
    }
    let target_object = resolve_owned_skill_begin_object(game, region, target);
    let started = runtime.now_milliseconds();
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false; };
    if !monster.prepare_base_attack_cast(target, skill_id, skill_level, started, target_object, game.skill_factory()) {
        return false;
    }
    monster.enqueue_base_attack_cast(runtime.now_milliseconds());
    true
}
