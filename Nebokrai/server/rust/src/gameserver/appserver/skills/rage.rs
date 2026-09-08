//! Канальный навык ярости `CRage` (`0x6D`).
//! End очищает своё исполнение, не выбранный навык игрока; m_pCurrentSkill
//! меняют OnChangeSkill/OnLoseTarget. Общий CSkill::End вызывает пустой
//! callback CPlayer +0x158 (0x00485540).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rage.cpp`. После задержки навык публикует начало канала,
//! а затем со строгим интервалом списывает MP и пополняет RP. Достижение
//! предельного RP проверяется после необратимого списания MP. Собственный
//! `RageExecutionState` хранит только жизненный цикл канала; `CGame`
//! разрешает player-owner-а, обновляет общий боевой режим и доставляет пакеты.
//! Замена команды и потеря цели вызывают тот же owner-`End` до очистки AI.
//! Восстановление использует абсолютный срок `CSkill::IsRestored`; интервал
//! канального списания сохраняет elapsed-семантику.
//! Объектный Begin (0x005A0830) проверяет CheckCastCondition до успешной
//! инициализации флагов (0x005A08C8), а при отказе вызывает End(0) и возвращает
//! false. Kernel создаётся только после допуска; возврат Begun отделяет
//! первый AI от Begin, сохраняя ранний отсчёт. Это позволяет расписанию
//! отличить отказ Begin от ошибки уже начатого канала.
//! Точечная проверка Begin: 0x005A0854 → 0x00601A50 → CSkill::Begin
//! (0x004D83E0), затем CRageEffect и только потом CheckCastCondition
//! (0x005A089E). База и visual существуют также при отказе без kernel.
//! Point Begin (0x005A05F0 → 0x005DFC00) и typed Begin
//! (0x005A06C0 → 0x005DFB00) сохраняют исходные аргументы, не заменяя target
//! на self; обе ветви также создают CRageEffect до проверки допуска.
//! End(0) возвращает движение живому GetUser, публикует его состояния,
//! обновляет visual действием 3 и очищает ту же базу без AfterUse/reuse.
//! CRageEffect::Update (0x005A0900) берёт ID/level у экземпляра, source из
//! сохранённого lifecycle, направление/координаты и frequency — из живых
//! владельцев. Mode 0/1/3 даёт wire action 1/2/3; mode 2/7/13 — отказ
//! только source-player. Завершающий base Update выполняется и без пакета.
//! Поздний combat канального тика не заменяет OnBeginSkill и сохранён отдельно.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, resolve_state_user,
};
use crate::gameserver::appserver::states::visualeffect::{CRageEffect, SkillVisualEffect};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const RAGE_SKILL_ID: u32 = 0x6d;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_GAIN: u32 = 23;
const TARGET_AFFECT_FREQUENCY: u32 = 6001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RageExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    last_using_time_ms: u32,
}

impl RageExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            last_using_time_ms: 0,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn last_using_time_ms(self) -> u32 {
        self.last_using_time_ms
    }

    pub(crate) const fn mark_used(&mut self, now_ms: u32) {
        self.last_using_time_ms = now_ms;
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) const fn is_rage_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == RAGE_SKILL_ID,
    }
}

fn rage_user(game: &CGame, player_id: i32) -> Option<(i32, ShapeIdentity)> {
    Some(game.player_skill_lifecycle(player_id, RAGE_SKILL_ID)?.user())
}

fn send_failure(game: &mut CGame, player_id: i32, action: u8, mp_loss: u32) {
    update_rage_visual(game, player_id, u32::from(action));
    let Some((region_id, identity)) = rage_user(game, player_id) else { return };
    let Some(source) = resolve_state_user(game, region_id, identity) else { return };
    if source.object_type != PLAYER_TYPE { return; }
    let player_id = source.id;
    match action {
        7 if mp_loss != 0 => {
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
        }
        7 => game.send_skill_system_info(player_id, b"GS0316"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn publish_rage_visual(
    game: &CGame,
    player_id: i32,
    mode: u32,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let Some(skill) = player.move_shape().skill(RAGE_SKILL_ID, game.skill_factory()) else { return };
    let Some(SkillVisualEffect::Rage(effect)) = skill.visual_effect() else { return };
    if effect.is_ended() { return; }
    let Some((region_id, identity)) = rage_user(game, player_id) else { return };
    let Some(source) = resolve_state_move_shape(game, region_id, identity) else { return };
    let source = source.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 13) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode {
        0 => 1,
        1 => 2,
        3 => 3,
        _ => return,
    };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 2 {
        let (Ok(tile_x), Ok(tile_y)) = (source.get_tile_x(), source.get_tile_y()) else { return };
        let frequency_ms = game.skill_base_properties(skill.id(), skill.level())
            .map_or(0, |properties| properties.query_property(TARGET_AFFECT_FREQUENCY));
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(tile_x);
        message.add_long(tile_y);
        message.add_ulong(frequency_ms);
    } else {
        message.add_long(source.get_direction());
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn update_rage_visual(game: &mut CGame, player_id: i32, mode: u32) {
    publish_rage_visual(game, player_id, mode);
    if let Some(effect) = game.player_skill_visual_effect_mut(player_id, RAGE_SKILL_ID) {
        effect.update_base_tail();
    }
}

/// CRage::End (0x005A0790): движение, OnChangeStates (+0x164), затем visual 3.
/// Ненулевой End после этой части выполняет AfterUse/reuse, нулевой — нет.
/// Обе ветви затем очищают общую базу и удаляют visual.
fn cleanup_player_rage(game: &mut CGame, player_id: i32) {
    if let Some((region_id, identity)) = rage_user(game, player_id) {
        let resolved = if let Some(source) = resolve_state_move_shape_mut(game, region_id, identity) {
            source.set_moveable(true);
            true
        } else {
            false
        };
        if resolved && identity.object_type == PLAYER_TYPE {
            let _ = game.publish_player_states(identity.id);
        }
    }
    update_rage_visual(game, player_id, 3);
}

fn end_player_rage(game: &mut CGame, player_id: i32) {
    cleanup_player_rage(game, player_id);
    game.finish_player_skill_base(player_id, RAGE_SKILL_ID, SkillTermination::Rejected);
}

fn finish_player_rage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    runtime: &mut Runtime,
) {
    cleanup_player_rage(game, player_id);
    if let Some((region_id, identity)) = rage_user(game, player_id)
        && let Some(source) = resolve_state_user(game, region_id, identity)
        && source.object_type == PLAYER_TYPE
    {
        game.damage_player_weapon(source.id, runtime);
    }
    game.mark_player_skill_used(player_id, RAGE_SKILL_ID, runtime.now_milliseconds());
    game.finish_player_skill_base(player_id, RAGE_SKILL_ID, SkillTermination::Completed);
}

pub(crate) fn cancel_player_rage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<RageExecutionState>(player_id, RAGE_SKILL_ID).map(|state| state.kernel().dispatch()) else {
        return false;
    };
    finish_player_rage(game, player_id, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_rage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rage_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(level) = game
        .find_player(player_id)
        .map(|player| player.learned_skill_level(RAGE_SKILL_ID, game.skill_factory()))
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };

    let beginning_at_ms = game.player_skill_state::<RageExecutionState>(player_id, RAGE_SKILL_ID)
        .is_none().then(|| runtime.now_milliseconds());
    if let Some(started_at_ms) = beginning_at_ms {
        game.begin_player_skill_with_combat(player_id, dispatch, started_at_ms);
        game.replace_player_skill_visual_effect(
            player_id, RAGE_SKILL_ID, SkillVisualEffect::Rage(CRageEffect::new()),
        );
    }
    if game
        .player_skill_state::<RageExecutionState>(player_id, RAGE_SKILL_ID)
        .is_some_and(|state| state.kernel().dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(properties) = game.skill_base_properties(RAGE_SKILL_ID, level) else {
        end_player_rage(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let rp_gain = properties.query_property(USER_RP_GAIN);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let reuse_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if let Some(started_at_ms) = beginning_at_ms
    {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, RAGE_SKILL_ID), reuse_ms, now_ms) {
            send_failure(game, player_id, 0x0d, 0);
            end_player_rage(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            end_player_rage(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(RAGE_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, RageExecutionState::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    }

    if game.find_player(player_id).is_none_or(|player| player.health() == 0) {
        update_rage_visual(game, player_id, 2);
        finish_player_rage(game, player_id, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game
        .player_skill_state::<RageExecutionState>(player_id, RAGE_SKILL_ID)
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        update_rage_visual(game, player_id, 0);
        if let Some(state) = game.player_skill_state_mut::<RageExecutionState>(player_id, RAGE_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game
        .player_skill_state::<RageExecutionState>(player_id, RAGE_SKILL_ID)
        .map(|state| state.kernel().started_at_ms())
        .unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    let last_using_time_ms = game
        .player_skill_state::<RageExecutionState>(player_id, RAGE_SKILL_ID)
        .map_or(0, |state| state.last_using_time_ms());
    if last_using_time_ms == 0 {
        update_rage_visual(game, player_id, 1);
        if let Some(state) = game.player_skill_state_mut::<RageExecutionState>(player_id, RAGE_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    if runtime
        .now_milliseconds()
        .wrapping_sub(last_using_time_ms)
        <= frequency_ms
    {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    game.enter_player_combat_state(player_id);
    let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
    if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
        send_failure(game, player_id, 7, 0);
        finish_player_rage(game, player_id, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_mana(current_mana.wrapping_sub(mp_loss));
    }

    let reached_maximum = game
        .find_player(player_id)
        .is_none_or(|player| player.maximum_rp() <= player.rp());
    if reached_maximum {
        if let Some(state) = game.player_skill_state_mut::<RageExecutionState>(player_id, RAGE_SKILL_ID) {
            let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
        }
        finish_player_rage(game, player_id, runtime);
        return terminal(QueuedSkillExecutionState::Completed);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_rp(player.rp().wrapping_add(rp_gain as u16));
    }
    let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    if let Some(state) = game.player_skill_state_mut::<RageExecutionState>(player_id, RAGE_SKILL_ID) {
        state.mark_used(runtime.now_milliseconds());
    }
    terminal(QueuedSkillExecutionState::Pending)
}
