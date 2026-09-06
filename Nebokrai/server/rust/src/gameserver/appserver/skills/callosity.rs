//! Исполнение взаимно исключающих навыков `CCallosity/CCallosity2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/callosity.cpp` и `callosity2.cpp`. Оба навыка сохраняют
//! один порядок: две проверки ресурсов, запрет движения, повторная проверка
//! с необратимым расходом MP до проверки RP, задержка, удаление первого
//! конфликтующего состояния, наложение нового состояния и `OnChangeStates`.
//! Новый коэффициент немедленно проходит через полный `UpdateProperty`.
//! Общий `SkillExecutionKernel` хранит только стадии и часы команды; форматы
//! сообщений, частичная мутация и два независимых времени восстановления
//! остаются здесь. Клиентская отмена сохраняет уже списанные ресурсы и
//! завершает тот же активный экземпляр до фиксации времени восстановления.
//! Reuse каждого варианта использует exact `CSkill::IsRestored`; stage delay
//! остаётся elapsed-интервалом.
//!
//! Сохранённый ниже псевдокод относится к `CCallosity`; `CCallosity2` имеет
//! тот же контракт с идентификатором `0x7d` и собственным временем
//! восстановления.

use super::baseattack::time_reached;
use super::callosity2::create_callosity_2_state;
pub(crate) use super::callosity2::CALLOSITY_2_SKILL_ID;
use super::callositystate::{
    end_player_callosity_state, send_callosity_state_begin, CallosityFamilyState, CallosityState,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::skills::kernel::{
    SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored,
};
use crate::gameserver::appserver::skills::stateskill::finish_state_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const CALLOSITY_SKILL_ID: u32 = 0x75;
pub(crate) const CALLOSITY_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_USER_RP_LOSE: u32 = 3;
pub(crate) const SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}
impl CallosityExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

fn restore_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn abort_player_callosity(game: &mut CGame, player_id: i32) {
    restore_movement(game, player_id);
}

fn finish_player_callosity<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    restore_movement(game, player_id);
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(skill_id, now_ms);
    });
}

pub(crate) fn cancel_player_callosity<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_state::<CallosityExecutionState>(execution_skill_id).copied().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    let skill_id = dispatch.skill_id();
    finish_player_callosity(game, player_id, skill_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_callosity<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let rejected = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Rejected,
        first_contact: false,
        killing_blow: None,
    };
    let pending = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Pending,
        first_contact: false,
        killing_blow: None,
    };
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if matches!(skill_id, CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID) => skill_id,
        _ => return rejected(),
    };
    let Some(player) = game.find_player(player_id) else {
        return rejected();
    };
    if player.server_region_id().is_none() {
        return rejected();
    }
    let skill_level = player.learned_skill_level(skill_id);
    let initial_mana = player.mana();
    let initial_rp = player.rp();
    let Some(properties) = game.skill_base_properties(skill_id, skill_level)
    else {
        if player_ai.player_skill_state::<CallosityExecutionState>(dispatch.skill_id()).copied().is_some() {
            abort_player_callosity(game, player_id);
        }
        return rejected();
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let rp_loss = properties.query_property(SKILL_USAGE_USER_RP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let blast_factor = properties.query_property(SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN) as u16;
    let state_persist_time = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_state::<CallosityExecutionState>(dispatch.skill_id()).copied().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let cooldown_now_ms = runtime.now_milliseconds();
        let last_used_ms = player_ai.skill_last_used_ms(skill_id);
        if !skill_is_restored(last_used_ms, reuse_delay_ms, cooldown_now_ms) {
            game.send_self_state_skill_failure(CALLOSITY_EFFECT_MESSAGE, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return rejected();
        }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_self_state_skill_failure(CALLOSITY_EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return rejected();
        }
        if rp_loss != 0 && (u32::from(initial_rp).wrapping_sub(rp_loss) as i32) < 0 {
            game.send_self_state_skill_failure(CALLOSITY_EFFECT_MESSAGE, player_id, 8);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0289", rp_loss);
            return rejected();
        }
        let Some(player) = game.find_player_mut(player_id) else {
            return rejected();
        };
        player.set_skill_moveable(false);
        player.set_current_skill_id(Some(skill_id));
        player_ai.begin_player_skill_execution(CallosityExecutionState::begin(dispatch, started_at_ms));
    } else if player_ai
        .player_skill_state::<CallosityExecutionState>(dispatch.skill_id()).copied()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return rejected();
    }

    if game.find_player(player_id).is_some_and(CPlayer::is_dead) {
        game.send_self_state_skill_failure(CALLOSITY_EFFECT_MESSAGE, player_id, 2);
        finish_player_callosity(game, player_id, skill_id, player_ai, runtime);
        return rejected();
    }

    if player_ai
        .player_skill_state::<CallosityExecutionState>(dispatch.skill_id()).copied()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        let current_mp = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mp.wrapping_sub(mp_loss) as i32) < 0 {
            game.send_self_state_skill_failure(CALLOSITY_EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_callosity(game, player_id);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mp.wrapping_sub(mp_loss));
        }
        let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 {
            game.send_self_state_skill_failure(CALLOSITY_EFFECT_MESSAGE, player_id, 8);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0289", rp_loss);
            abort_player_callosity(game, player_id);
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_rp(current_rp.wrapping_sub(rp_loss as u16));
        }
        game.send_self_state_skill_cast(CALLOSITY_EFFECT_MESSAGE, player_id, skill_id, skill_level, 1);
        if let Some(state) = player_ai.player_skill_state_mut::<CallosityExecutionState>(dispatch.skill_id()) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .player_skill_state::<CallosityExecutionState>(dispatch.skill_id()).copied()
        .map(|state| state.kernel().started_at_ms())
        .expect("исполнение закалки создано или восстановлено");
    let delay_now_ms = runtime.now_milliseconds();
    if !time_reached(delay_now_ms, started_at_ms, delay_ms) {
        return pending();
    }

    game.send_self_state_skill_cast(CALLOSITY_EFFECT_MESSAGE, player_id, skill_id, skill_level, 2);
    let _ = end_player_callosity_state(game, player_id);
    let state_started_at_ms = runtime.now_milliseconds();
    let state = if skill_id == CALLOSITY_2_SKILL_ID {
        CallosityFamilyState::Callosity2(create_callosity_2_state(
            blast_factor,
            state_started_at_ms,
            state_persist_time,
        ))
    } else {
        CallosityFamilyState::Callosity(CallosityState::new(
            blast_factor,
            state_started_at_ms,
            state_persist_time,
        ))
    };
    if let Some(player) = game.find_player_mut(player_id) {
        player.begin_callosity_state(state);
    }
    send_callosity_state_begin(game, player_id, state);
    let _ = game.publish_player_states(player_id);
    let _ = game.update_player_properties(player_id);
    if let Some(state) = player_ai.player_skill_state_mut::<CallosityExecutionState>(dispatch.skill_id()) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_callosity(game, player_id, skill_id, player_ai, runtime);
    QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Completed,
        first_contact: false,
        killing_blow: None,
    }
    }
