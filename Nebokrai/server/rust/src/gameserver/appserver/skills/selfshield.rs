//! Общий execution-owner самонакладываемых `CManaShield` и `CMachineShield`.
//!
//! Здесь объединён только подтверждённый одинаковый контракт двух навыков:
//! двойная проверка MP, время восстановления, стадии каста, клиентская отмена
//! и точный порядок полей пакета. Конкретный набор параметров и создание
//! канонического состояния остаются у владельца навыка. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`. Источник: gameserver.exe
//! + GameServer.pdb, `CMachineShield::AI` (0x005671C0) и `CManaShield::AI`
//! (0x00569DF0). Каст ждёт unsigned now >= wrapping(start + delay).
//! После расхода MP вызывается OnChangeStates; прежний щит полностью
//! завершается до Begin нового, а после установки вызывается UpdateProperty.
//! Проверка MP внутри AI использует знак 32-битной разности (`sub/js`),
//! а не беззнаковое сравнение исходных значений.
//! Begin заканчивается возвратом Begun после создания исполнения. Проверки
//! и эффекты первого AI остаются после этой границы; координатор вызывает AI
//! в том же Run после постановки Attack, не сдвигая исходное время Begin.

use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::machineshield::{MACHINE_SHIELD_SKILL_ID, MachineShieldOwner};
use super::manashield::{MANA_SHIELD_SKILL_ID, ManaShieldOwner};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_STATE_HP: u32 = 10_010;
const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

pub(crate) trait SelfShieldOwner {
    type State: Copy;
    type Extra: Copy;

    const SKILL_ID: u32;
    const EFFECT_MESSAGE: i32;

    fn read_extra(properties: &CSkillBaseProperties) -> Self::Extra;
    fn create_state(
        started_at_ms: u32,
        keep_time_ms: u32,
        life: i32,
        hp_factor: u16,
        mp_factor: u16,
        extra: Self::Extra,
    ) -> Self::State;
    fn replace_state(player: &mut CPlayer, state: Self::State) -> Option<Self::State>;
    fn send_state_visual(
        game: &mut CGame,
        player_id: i32,
        state: Self::State,
        begin: bool,
        now_milliseconds: impl FnMut() -> u32,
    );

}

pub(crate) const fn is_self_shield_skill(skill_id: u32) -> bool {
    matches!(skill_id, MANA_SHIELD_SKILL_ID | MACHINE_SHIELD_SKILL_ID)
}

fn finish_player_self_shield<Owner: SelfShieldOwner, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) {
    game.finish_self_shield_movement(player_id);
    game.mark_player_skill_used(player_id, Owner::SKILL_ID, runtime.now_milliseconds());
}

fn cancel_player_self_shield<Owner: SelfShieldOwner, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, Owner::SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_self_shield::<Owner, Runtime>(game, player_id, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn cancel_player_self_shield_dispatch<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    match skill_id {
        MANA_SHIELD_SKILL_ID => cancel_player_self_shield::<ManaShieldOwner, Runtime>(
            game, player_id, player_ai, runtime,
        ),
        MACHINE_SHIELD_SKILL_ID => cancel_player_self_shield::<MachineShieldOwner, Runtime>(
            game, player_id, player_ai, runtime,
        ),
        _ => false,
    }
}

pub(crate) fn execute_player_self_shield_dispatch<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    match skill_id {
        MANA_SHIELD_SKILL_ID => execute_player_self_shield::<ManaShieldOwner, Runtime>(
            game, player_id, dispatch, player_ai, runtime,
        ),
        MACHINE_SHIELD_SKILL_ID => execute_player_self_shield::<MachineShieldOwner, Runtime>(
            game, player_id, dispatch, player_ai, runtime,
        ),
        _ => QueuedSkillExecutionOutcome {
            state: QueuedSkillExecutionState::Rejected,
            first_contact: false,
            killing_blow: None,
        },
    }
}

fn send_cast<Owner: SelfShieldOwner>(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(Owner::EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(Owner::SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn execute_player_self_shield<Owner, Runtime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome
where
    Owner: SelfShieldOwner,
    Runtime: GameMainLoopRuntime,
{
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if skill_id == Owner::SKILL_ID => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(player) = game.find_player(player_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let skill_level = player.learned_skill_level(skill_id, game.skill_factory());
    let initial_mana = player.mana();
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state_life = properties.query_property(SKILL_USAGE_STATE_HP) as i32;
    let hp_factor = properties.query_property(SKILL_USAGE_TARGET_HP_DECREASE_FACTOR) as u16;
    let mp_factor = properties.query_property(SKILL_USAGE_TARGET_MP_DECREASE_FACTOR) as u16;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let extra = Owner::read_extra(properties);

    if game.player_skill_execution(player_id, Owner::SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, Owner::SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            game.send_self_state_skill_failure(Owner::EFFECT_MESSAGE, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if initial_mana < mp_loss {
            game.send_self_state_skill_failure(Owner::EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, Owner::SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.player_skill_execution(player_id, Owner::SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        let remaining_mana = current_mana.wrapping_sub(mp_loss);
        if (remaining_mana as i32) < 0 {
            game.send_self_state_skill_failure(Owner::EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            game.finish_self_shield_movement(player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(remaining_mana);
        }
        let _ = game.publish_player_states(player_id);
        send_cast::<Owner>(game, player_id, skill_level, 1);
        if let Some(state) = game.player_skill_execution_mut(player_id, Owner::SKILL_ID) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_execution(player_id, Owner::SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение щита создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_cast::<Owner>(game, player_id, skill_level, 2);
    let _ = super::shieldstate::end_player_defense_shield(
        game, player_id, Owner::SKILL_ID, runtime.now_milliseconds(),
    );
    let state = Owner::create_state(
        runtime.now_milliseconds(),
        keep_time_ms,
        state_life,
        hp_factor,
        mp_factor,
        extra,
    );
    Owner::send_state_visual(game, player_id, state, true, || runtime.now_milliseconds());
    let _ = game
        .find_player_mut(player_id)
        .and_then(|player| Owner::replace_state(player, state));
    let _ = game.update_player_properties(player_id);
    if let Some(state) = game.player_skill_execution_mut(player_id, Owner::SKILL_ID) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_self_shield::<Owner, Runtime>(game, player_id, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
