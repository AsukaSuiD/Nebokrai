//! Общее исполнение пустых навыков `CNonFun`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/nonfun.cpp`. Семейство `900..946` и `960..962` принимает
//! все три формы цели при существующем игроке, устанавливает текущий навык и
//! на том же вызове AI завершает его с признаком `false`. Оно не расходует
//! ресурсы, не вызывает RNG, не формирует пакеты и не изменяет цель.

use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const fn is_non_fun_skill(skill_id: u32) -> bool {
    matches!(skill_id, 0x384..=0x3b2 | 0x3c0..=0x3c2)
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) fn execute_player_non_fun<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if is_non_fun_skill(skill_id) => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    if game.find_player(player_id).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.non_fun().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_non_fun(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .non_fun()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(state) = player_ai.non_fun_mut() {
        let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
