//! Сбор душ `CSoulCollect` (`0x13B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulcollect.cpp`. Здесь находятся проверка владельца-
//! игрока, двухфазный расход MP, задержка повторного применения, обычная
//! задержка, `SkillExecutionKernel` и точный порядок визуальных пакетов
//! накопления. Состоянием владеет
//! `CanonicalStateStorage`; `CGame` используется только для разрешения игрока,
//! обновления общего fight-state и фактической around-доставки. Три исходные
//! перегрузки `Begin` имели одинаковую семантику состояния владельца и сведены
//! к одному типизированному `PlayerSkillDispatch` без параллельного пути.

use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::soulcollectstate::{SoulCollectState, send_soul_collect_state_visual};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const SOUL_COLLECT_SKILL_ID: u32 = 0x13b;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const PARAMETER_PERCENT: u32 = 20_020;
const DELAY_TIME: u32 = 10_001;
const REUSE_DELAY_TIME: u32 = 10_005;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_cast_visual(game: &mut CGame, player_id: i32, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let Some(shape) = player.shape_view() else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(SOUL_COLLECT_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(PLAYER_TYPE);
        message.add_long(player_id);
        message.add_long(shape.tile_x);
        message.add_long(shape.tile_y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

fn add_soul(game: &mut CGame, player_id: i32, variable_percent: u32, level: i32) -> bool {
    let Some((region_id, shape)) = game
        .find_player(player_id)
        .and_then(|player| Some((player.server_region_id()?, player.shape_view()?)))
    else { return false };

    if let Some(previous) = game.find_player(player_id).and_then(CPlayer::soul_collect_state) {
        let changed = game
            .find_player_mut(player_id)
            .and_then(CPlayer::soul_collect_state_mut)
            .is_some_and(SoulCollectState::add_soul);
        if changed {
            let current = game
                .find_player(player_id)
                .and_then(CPlayer::soul_collect_state)
                .expect("состояние сбора душ изменено на месте");
            send_soul_collect_state_visual(game, region_id, shape.identity, shape.tile_x, shape.tile_y, previous, false);
            send_soul_collect_state_visual(game, region_id, shape.identity, shape.tile_x, shape.tile_y, current, true);
        }
        return true;
    }

    let empty = SoulCollectState::new(level, variable_percent);
    send_soul_collect_state_visual(game, region_id, shape.identity, shape.tile_x, shape.tile_y, empty, true);
    let mut state = empty;
    if state.add_soul() {
        send_soul_collect_state_visual(game, region_id, shape.identity, shape.tile_x, shape.tile_y, empty, false);
        send_soul_collect_state_visual(game, region_id, shape.identity, shape.tile_x, shape.tile_y, state, true);
    }
    let Some(player) = game.find_player_mut(player_id) else { return false };
    player.begin_soul_collect_state(state);
    true
}

pub(crate) fn execute_player_soul_collect<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_soul_collect_skill(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let level = player.learned_skill_level(SOUL_COLLECT_SKILL_ID);
    let Some(properties) = game.skill_base_properties(SOUL_COLLECT_SKILL_ID, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay = properties.query_property(DELAY_TIME);
    let cooldown = properties.query_property(REUSE_DELAY_TIME);
    let variable_percent = properties.query_property(PARAMETER_PERCENT);

    if player_ai.soul_collect().is_none() {
        let started = runtime.now_milliseconds();
        let cooldown_now = runtime.now_milliseconds();
        if player_ai.soul_collect_last_used_ms() != 0
            && !time_reached(cooldown_now, player_ai.soul_collect_last_used_ms(), cooldown)
        {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || player.mana() < mp_loss {
            if mp_loss != 0 { send_failure(game, player_id, 7, mp_loss); }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SOUL_COLLECT_SKILL_ID));
        }
        player_ai.begin_soul_collect(SkillExecutionKernel::begin(dispatch, started));
    } else if player_ai.soul_collect().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.soul_collect().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if mana < mp_loss {
            send_failure(game, player_id, 7, mp_loss);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_cast_visual(game, player_id, level, false);
        if let Some(execution) = player_ai.soul_collect_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started = player_ai.soul_collect().map(SkillExecutionKernel::started_at_ms).expect("выполнение сбора душ создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending); }
    send_cast_visual(game, player_id, level, true);
    let applied = add_soul(game, player_id, variable_percent, level);
    if let Some(execution) = player_ai.soul_collect_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_soul_collect_used(runtime.now_milliseconds());
    finish(game, player_id);
    terminal(if applied { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}

pub(crate) const fn is_soul_collect_skill(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: SOUL_COLLECT_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: SOUL_COLLECT_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: SOUL_COLLECT_SKILL_ID, .. }
    )
}
