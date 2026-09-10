//! Общее исполнение пустых навыков `CNonFun`.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/nonfun.cpp`. Семейство `900..946` и `960..962` принимает
//! все три формы цели при существующем игроке после общего допуска OnSchedule,
//! устанавливает текущий навык и
//! на том же вызове AI завершает его с признаком `false`. Оно не расходует
//! ресурсы, не вызывает RNG и не изменяет цель. Собственных пакетов нет,
//! но унаследованный Begin переводит игрока в бой через общий вход расписания
//! (включая снятие AutoProtect и сообщение о смене боевого состояния).
//! AI вызывает End(false); End (0x0050E9B0) передаёт флаг CStateSkill::End:
//! при таком завершении нет ни AfterUseSkill,
//! ни cooldown, ни UpdateProperty; виртуальный callback игрока +0x158 пуст.

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
    }
}

pub(crate) fn execute_player_non_fun<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
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
    if game.player_skill_execution(player_id, skill_id).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game
        .player_skill_execution(player_id, skill_id)
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(state) = game.player_skill_execution_mut(player_id, skill_id) {
        let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
