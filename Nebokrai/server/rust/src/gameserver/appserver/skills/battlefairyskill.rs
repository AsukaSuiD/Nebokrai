//! Общий вход, visual и завершение семейства навыков боевого духа.
//! Источник: gameserver.exe/GameServer.pdb, семейство appserver/skills
//! и базовый appserver/states/skill.cpp.
//!
//! Очередь и background сохраняют поколенческий ключ до Begin/AI/contacts.
//! Материализация State/Fatal/BaseAttack переносит единственную базу без
//! второго Begin. Полётные скаляры принадлежат concrete payload, а созданные
//! снаряды и области живут независимо. Все 19 навыков 0x212..=0x224 обходятся
//! без prepared; после попытки Summon AI завершает навык через End(1).
//!
//! End читает живого GetUser, выполняет AfterUse/износ и фиксирует cooldown
//! последним; затем освобождает source/target/visual и payload. Player callback
//! не добавляет UpdateProperty. AI опубликован у игрока, новая команда из
//! callback сохраняется, а продолжение не переходит на удалённую регистрацию.
//! Собственный пролог End отключает concrete AI до visual, но оставляет базу
//! доступной AfterUse. Idle не является повторным Begin.
//!
//! Ошибки атак, LifeShield и transfer дают End(0); внешний OnLoseTargetWarSoul
//! может вызвать End(1) только у ещё активного навыка. Po/Yu отличают смерть S
//! от отсутствия S/MP: RejectedAfterUse сохраняет успешный хвост End(1).
//! Concrete End(bool) Po/Yu/transfer сохраняет собственный rearm/visual3 и
//! не подменяет внешний End(int), чью политику выбирает фабрика.
//! Пространственные SetWarSoulXY/DelWarSoul выбирают m_tgCurrentWarSoulSkill
//! без gates по current-команде/payload и не снимают команду; их End(0)
//! не требует часов. Set требует target area, Delete проверяет навык до area.
//! При отказе Begin сбрасывается и база без payload, затем расписание
//! отправляет единственный внешний 4,2 сверх внутренних отказов владельца.
//!
//! Все skill-owned эффекты — общий CVisualEffect без собственных полей.
//! Он создаётся между базовым Begin и concrete Check; visual читает ID,
//! уровень, U/S из живого экземпляра. Даже молчащий Update проходит base tail,
//! а mode3 сам по себе не завершает ресурс. Различия wire находятся в одном
//! статическом контракте, без копий lifecycle и отдельных контейнеров.
//! Fatal/Leiming требуют source.type=700 только в wire: выбор player-around
//! в оригинале основан на RTTI. Живая форма не меняет тип, сохраняются обычная
//! доставка и дальний team-tail. Формулы и проверки остаются у concrete owners.

use super::basemagic::BASE_MAGIC_EFFECT_MESSAGE;
use super::kernel::{BattleFairyExecution, SkillExecutionKernel, SkillTermination};
use super::skillfactory::{SkillEndEffect, SkillOwner};
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::moveshape::{MoveShapeSkill, RegisteredSkillDispatch};
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;

#[derive(Clone, Copy)]
enum BattleFairyFireTarget { Optional, Required, Point, Source, LifeShield }

/// Только различия wire конкретных классов. База ресурса и её lifecycle
/// общие; здесь нет копий source, target, уровня или исполнения навыка.
struct BattleFairyVisualContract {
    target: BattleFairyFireTarget,
    failure_modes: &'static [u32],
    fire_time: bool,
    source_type_on_fire: bool,
    fire_id: Option<u32>,
    zero_prefix_mode: Option<u32>,
    wide_failure_eight: bool,
    boolean_end_effect: Option<SkillEndEffect>,
}

impl BattleFairyVisualContract {
    fn for_owner(owner: SkillOwner) -> Option<Self> {
        use BattleFairyFireTarget as Target;
        use SkillOwner::*;
        let mut result = Self {
            target: Target::Required, failure_modes: &[2, 7, 10, 11, 13, 15],
            fire_time: false, source_type_on_fire: false, fire_id: None,
            zero_prefix_mode: None, wide_failure_eight: false,
            boolean_end_effect: None,
        };
        match owner {
            BFBaseAttack => { result.target = Target::Optional; result.fire_time = true; }
            CFatalBlow => {
                result.fire_time = true; result.failure_modes = &[2, 7, 10, 11, 13, 14, 15];
            }
            CThunder | CLeiming2 => {
                result.target = Target::Point;
                if owner == CLeiming2 { result.zero_prefix_mode = Some(7); }
            }
            CTianhuo => { result.target = Target::Optional; result.fire_id = Some(0x13a); }
            CBloodLoss | CPoisonArrow => {
                result.failure_modes = &[2, 7, 8, 10, 11, 13, 14, 15];
                if owner == CBloodLoss { result.zero_prefix_mode = Some(13); }
            }
            CLifeShield => { result.target = Target::LifeShield; result.failure_modes = &[2, 7, 8, 13, 14]; }
            CPojia | CPobing | CPomo | CPofa | CYujia | CYubing | CYumo | CYufa
            | CHuoxieshu | CLingzhishu | CWangsheng => {
                if !matches!(owner, CPojia | CPobing | CPomo | CPofa) { result.target = Target::Source; }
                result.source_type_on_fire = true; result.wide_failure_eight = true;
                result.failure_modes = if owner == CHuoxieshu { &[2, 6, 8, 13] } else { &[2, 7, 8, 13] };
                result.boolean_end_effect = Some(if matches!(owner, CHuoxieshu | CLingzhishu | CWangsheng) {
                    SkillEndEffect::BattleFairySummon
                } else {
                    SkillEndEffect::BattleFairyState
                });
            }
            _ => return None,
        }
        Some(result)
    }
}

pub(crate) fn publish_battle_fairy_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    let Some(contract) = BattleFairyVisualContract::for_owner(skill.owner()) else { return };
    if skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::BattleFairy || effect.is_ended()) {
        return;
    }
    let (region_id, source_identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region_id, source_identity) else { return };
    let source = source.shape();
    let identity = source.identity();
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    if contract.failure_modes.contains(&mode) {
        if identity.object_type != 400 { return; }
        if mode == 8 && contract.wide_failure_eight { message.add_long(4); }
        else { message.add_byte(if contract.zero_prefix_mode == Some(mode) { 0 } else { 4 }); }
        message.add_byte(mode as u8);
        let _ = message.send_to_player(game.net_server(), identity.id);
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    message.add_byte(action);
    message.add_long(if action == 2 { contract.fire_id.unwrap_or(skill.id()) } else { skill.id() } as i32);
    message.add_short(skill.level() as i16);
    message.add_long(if action == 2 && contract.source_type_on_fire { identity.object_type } else { BATTLE_FAIRY_VISUAL_OBJECT_TYPE });
    message.add_long(identity.id);
    if action == 2 {
        let sufferer = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, target)| resolve_state_move_shape(game, region, target))
            .map(|target| target.shape());
        if matches!(contract.target, BattleFairyFireTarget::Required | BattleFairyFireTarget::LifeShield)
            && sufferer.is_none() { return; }
        let target = if matches!(contract.target, BattleFairyFireTarget::Source) { Some(source) } else { sufferer };
        let target_identity = target.map(|target| target.identity());
        let zero_identity = matches!(contract.target, BattleFairyFireTarget::Point | BattleFairyFireTarget::LifeShield);
        message.add_long(if zero_identity { 0 } else { target_identity.map_or(0, |identity| identity.object_type) });
        message.add_long(if zero_identity { 0 } else { target_identity.map_or(0, |identity| identity.id) });
        if !matches!(contract.target, BattleFairyFireTarget::LifeShield) {
            let (x, y) = match target {
                Some(target) => {
                    let (Ok(x), Ok(y)) = (target.get_tile_x(), target.get_tile_y()) else { return };
                    (x, y)
                }
                None => skill.lifecycle().destination(),
            };
            message.add_long(x); message.add_long(y);
        }
        if contract.fire_time {
            let time = match skill.battle_fairy_execution_state() {
                Some(BattleFairyExecution::BaseMagic(state)) => state.attack_time(),
                Some(BattleFairyExecution::FatalBlow(state)) => state.missile_flying_time(),
                _ => 0,
            };
            message.add_ulong(time);
        }
    } else { message.add_long(source.get_direction()); }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

/// Конфликт выбирается по первой позиции исходного массива состояний, а не
/// по приоритету ID. Цвет SystemInfo здесь белый, без дополнительного visual.
pub(super) fn check_battle_fairy_target_states(
    game: &CGame, player_id: i32, target: (i32, ShapeIdentity),
) -> bool {
    let Some(holder) = resolve_state_move_shape(game, target.0, target.1) else { return false; };
    let conflict = holder.find_state_position(|state| matches!(state.state_id(), 0x192 | 0xd2 | 0x67))
        .and_then(|(position, _)| holder.state_at(position))
        .map(|(_, state)| state.state_id());
    let Some(state_id) = conflict else { return true; };
    let text = game.get_string_by_id(if state_id == 0xd2 { b"ZHGS0047" } else { b"ZHGS0046" });
    let mut message = CMessage::new(0x0b_f807);
    message.add_ulong(0xffff_ffff);
    let length = text.iter().position(|byte| *byte == 0).unwrap_or(text.len());
    message.base_mut().add(&text[..length]);
    message.base_mut().add_byte(0);
    let _ = message.send_to_player(game.net_server(), player_id);
    false
}

/// Навыки без собственных скалярных полей используют тот же общий вход.
pub(crate) fn execute_registered_battle_fairy_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    begin_failure_visual: Option<u32>,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, begin_failure_visual, check,
        |dispatch, started| BattleFairyExecution::State(SkillExecutionKernel::begin(dispatch, started)),
        run_ai,
    )
}

/// Общая материализация сохраняет единственную базу зарегистрированного
/// экземпляра. Координатор уже выполнил base Begin и опубликовал AI; здесь
/// нет вторых часов или End. Фабрика создаёт лишь собственные данные owner-а.
pub(crate) fn execute_registered_battle_fairy_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    begin_failure_visual: Option<u32>,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(BattleFairySkillDispatch, u32) -> BattleFairyExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.id() != dispatch.skill_id() {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    if let Some(previous) = skill.battle_fairy_dispatch() {
        if previous != dispatch {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        return run_ai(game, instance, runtime);
    }
    if !check(game, instance, player_id, runtime) {
        if let Some(mode) = begin_failure_visual {
            game.update_registered_skill_visual(instance, mode);
        }
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    let Some(skill) = game.registered_skill_mut(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let execution = materialize(dispatch, skill.lifecycle().started_at_ms());
    if !skill.install_battle_fairy_execution(execution) {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    state_skill_outcome(QueuedSkillExecutionState::Begun)
}

impl CGame {
    /// SetWarSoulXY (0x0042DF50) и DelWarSoul (0x0042E0A0) вызывают End(int,0)
    /// выбранного зарегистрированного навыка, если его база ещё не ended.
    /// Payload и текущая команда не являются gates; очередь и target не снимаются.
    /// Собственный End(bool) Po/Yu/transfer сюда не подставляется.
    pub(crate) fn cancel_active_battle_fairy_skill(
        &mut self,
        player_id: i32,
    ) -> bool {
        let Some(skill_id) = self.find_player(player_id).map(|player| player.player_ai().selected_battle_fairy_skill_id()) else { return false };
        let Some(instance) = self.registered_player_skill(player_id, skill_id) else { return false };
        let Some(skill) = self.registered_skill(instance) else { return false };
        if skill.lifecycle().is_ended() { return false; }
        let dispatch = skill.battle_fairy_dispatch();
        let _ = self.end_registered_instance_without_after_use(instance, SkillTermination::Cancelled);
        if let Some(dispatch) = dispatch && let Some(skill) = self.registered_skill_mut(instance) {
            skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
        }
        true
    }

    /// Терминальная граница после concrete owner и contacts. Выполняет точный
    /// concrete End(bool/int); общий End сохраняет источник до AfterUse и
    /// только затем сбрасывает базу. Callback не может перенести reuse/cleanup
    /// на новую регистрацию того же ID. Неуспешный Begin не требует payload.
    pub(crate) fn finish_registered_battle_fairy_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        instance: RegisteredSkill,
        dispatch: BattleFairySkillDispatch,
        argument: i32,
        termination: SkillTermination,
        begin_attempted: bool,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = self.registered_skill(instance) else { return false };
        let materialized = skill.battle_fairy_dispatch() == Some(dispatch);
        let failed_begin = begin_attempted && !skill.lifecycle().is_ended()
            && skill.is_execution_inactive();
        if !materialized && !failed_begin {
            return false;
        }
        let argument = if materialized { argument } else { 0 };
        self.prepare_battle_fairy_boolean_end(instance);
        let _ = self.end_registered_instance(instance, argument, termination, runtime);
        if let Some(skill) = self.registered_skill_mut(instance) {
            skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
        }
        true
    }

    fn prepare_battle_fairy_boolean_end(&mut self, instance: RegisteredSkill) {
        if let Some(effect) = self.registered_skill(instance)
            .and_then(|skill| BattleFairyVisualContract::for_owner(skill.owner()))
            .and_then(|contract| contract.boolean_end_effect)
        {
            let _ = self.prepare_registered_skill_end_effect(instance, effect, 0);
        }
    }

    pub(crate) fn send_battle_fairy_skill_failure(&self, player_id: i32, action: u8) {
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(4);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

}
