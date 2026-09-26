//! Состояние и тела расписаний игрового AI игрока `CPlayerAI` без hub-типов:
//! назначения клиента (destination FIFO), независимые очереди навыков игрока и
//! боевого духа, выбранные/текущие команды, object-цель проверки и хвост `Run`
//! (авто-прирост опыта/бодрости и регенерация энергии). Базовые FIFO, цель и
//! back-stage список — в `super::baseai::CBaseAI`; passive-реакции —
//! `super::reactions`. Исходный владелец PDB: `appserver/ai/playerai.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! Швы: игрок приходит узким фасадом `AutoIncPlayer` (двенадцать аксессорных
//! операций хвоста `Run` прежнего `CPlayer`, `appserver/player.rs`); часы
//! каждого события читаются отдельным вызовом делегата старого main loop;
//! dispatch/execution kernel навыков — уже Zone (`skills/dispatch.rs`,
//! `skills/lifecycle.rs`). Канонический `CPlayer` владеет очередями навыков и
//! публикует настоящий `CPlayerAI` на время обоих callback смерти.
//!
//! UNKNOWN: источник события `ChangeSkillWithWarSoul` в этой паре не найден;
//! полный registered `CSkill::End`; поведение слота `+0x8C CPlayer` на пути
//! MoveTo.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#ai-расписаний-и-поведение

use std::collections::VecDeque;

use crate::regions::ShapeIdentity;
use crate::skills::{
    BattleFairySkillDispatch, PlayerSkillDispatch, SkillExecutionKernel,
    BATTLE_FAIRY_BASE_MAGIC_SKILL_ID,
};

use super::baseai::CBaseAI;
use super::events::AiShapeAction;
use super::reactions::{PassiveDeathAction, PassiveStiffenAction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerAiDestination {
    pub direction: i32,
    pub is_run: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairySkillQueueOutcome {
    PendingUnchanged,
    Queued { replaced: usize },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CPlayerAI {
    base_ai: CBaseAI,
    destinations: VecDeque<PlayerAiDestination>,
    player_skills: VecDeque<PlayerSkillDispatch>,
    current_player_skill: Option<PlayerSkillDispatch>,
    selected_battle_fairy_skill_id: u32,
    current_battle_fairy_skill: Option<BattleFairySkillDispatch>,
    battle_fairy_skills: VecDeque<BattleFairySkillDispatch>,
    auto_inc_last_time_ms: u32,
    auto_inc_energy_last_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerAutoProgress {
    pub player_id: i32,
    pub sampled_at_ms: u32,
    pub experience_gain: u32,
    pub vigour_gain: u32,
    pub previous_experience: u32,
    pub current_experience: u32,
    pub previous_vigour: u32,
    pub current_vigour: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerEnergyRegeneration {
    pub player_id: i32,
    pub sampled_at_ms: u32,
    pub increment: u32,
    pub previous_energy: u32,
    pub current_energy: u32,
}

/// Игрок хвоста авто-прироста опыта/бодрости и регенерации энергии
/// `CPlayerAI::Run`: переходный фасад прежнего `CPlayer`
/// (`appserver/player.rs`). Имена членов сохраняют исходные аксессорные
/// операции; hub-реализация только делегирует им без изменения семантики.
pub trait AutoIncPlayer {
    fn player_id(&self) -> i32;

    fn is_dead(&self) -> bool;

    fn faction_id(&self) -> i32;

    fn faction_level(&self) -> u16;

    fn level(&self) -> u8;

    fn experience(&self) -> u32;

    fn set_experience(&mut self, value: u32);

    fn vigour(&self) -> u32;

    fn set_vigour(&mut self, value: u32);

    fn energy(&self) -> u32;

    fn maximum_energy(&self) -> u32;

    fn set_energy(&mut self, value: u32);
}

impl CPlayerAI {
    pub const fn base_ai(&self) -> &CBaseAI {
        &self.base_ai
    }

    pub const fn base_ai_mut(&mut self) -> &mut CBaseAI {
        &mut self.base_ai
    }

    pub fn when_been_hurted(&mut self, now_ms: u32) {
        self.base_ai.when_been_hurted(now_ms);
    }

    pub fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.when_been_stiffened(delay_ms, now_ms);
    }

    pub fn when_been_killed(&mut self, now_ms: u32) {
        self.base_ai.when_been_killed(now_ms);
    }

    pub fn process_reached_defense_actions(&mut self) -> usize {
        self.base_ai.process_reached_defense_actions(|_| {})
    }

    pub fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        self.base_ai.begin_reached_stiffen_action()
    }

    pub fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        self.base_ai.finish_reached_stiffen_action(begun, now)
    }

    pub fn begin_reached_death_action(&mut self) -> bool {
        self.base_ai.begin_reached_death_action()
    }

    pub fn reached_death_action_state(&self) -> PassiveDeathAction {
        self.base_ai.reached_death_action_state()
    }

    pub fn finish_reached_death_action(&mut self, now_ms: u32) {
        self.base_ai.finish_reached_death_action(now_ms);
    }

    pub fn stiffen_attack_needs_end(&self) -> bool {
        self.base_ai.stiffen_attack_needs_end()
    }

    pub fn stiffen_attack_pending(&self) -> bool {
        self.base_ai.stiffen_attack_pending()
    }

    pub fn current_active_action(&self) -> Option<AiShapeAction> {
        self.base_ai.current_active_action()
    }

    pub fn advance_handled_active_action(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_handled_active_action(now).is_some()
    }

    pub fn advance_handled_passive_action(&mut self, now: impl FnOnce() -> u32) -> Option<bool> {
        self.base_ai.advance_handled_passive_action(now)
    }

    pub fn advance_handled_war_soul_action(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_handled_war_soul_action(now).is_some()
    }

    pub fn finish_stiffen_attack(&mut self, release_target: bool) {
        self.base_ai.finish_stiffen_attack(release_target);
    }

    pub fn discard_active_prefix(&mut self) {
        self.base_ai.discard_active_prefix();
    }

    pub fn queue_client_destination(&mut self, direction: i32, is_run: bool) {
        while 3 < self.destinations.len() {
            self.destinations.pop_front();
        }
        self.destinations
            .push_back(PlayerAiDestination { direction, is_run });
    }

    /// Выполняет достигнутую `ASA_MOVE`-границу после текущего `OnSchedule`.
    /// Даже снятое в этом вызове событие удерживает расписание до следующего
    /// такта, как `CBaseAI::ProcessActiveAction`.
    pub fn advance_active_move(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_active_move(now)
    }

    pub fn active_move_unhandled(&self) -> bool {
        self.base_ai.active_move_unhandled()
    }

    pub fn advance_active_stand(&mut self, now: impl FnOnce() -> u32) -> bool {
        self.base_ai.advance_active_stand(now)
    }

    pub fn active_stand_pending(&self) -> bool {
        self.base_ai.active_stand_pending()
    }

    pub fn active_stand_unhandled(&self) -> bool {
        self.base_ai.active_stand_unhandled()
    }

    pub fn active_attack_pending(&self) -> bool {
        self.base_ai.active_attack_pending()
    }

    pub fn primary_queues_idle(&self) -> bool {
        self.base_ai.primary_queues_idle()
    }

    pub fn begin_player_fighting(&mut self, now_ms: u32) {
        if !self.base_ai.active_actions().iter().any(|event| event.action == AiShapeAction::Attack) {
            self.base_ai.add_ai_event(AiShapeAction::Attack, 0, 0, now_ms);
        }
    }

    /// OnFighting проверяет IsEnded и IsPrepared до AI. Перенос в фон
    /// выполняется до ChangeSkill, но после уже прошедшего фонового обхода.
    pub fn finish_player_attack(
        &mut self,
        selected_skill_id: Option<u32>,
        execution: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
        mut now: impl FnMut() -> u32,
    ) -> bool {
        if !self.base_ai.active_attack_pending() {
            return false;
        }
        if let Some(skill_id) = selected_skill_id
            && let Some(execution) = execution
        {
            if !execution.is_prepared() {
                return false;
            }
            self.base_ai.add_started_back_stage_skill(skill_id);
        }
        self.current_player_skill = None;
        if selected_skill_id.is_some() {
            self.base_ai.add_ai_event(AiShapeAction::ChangeSkill, 0, 0, now());
        }
        self.base_ai.finish_active_attack(now());
        true
    }

    pub fn active_change_skill_pending(&self) -> bool {
        self.base_ai.active_change_skill_pending()
    }

    pub fn finish_active_change_skill(&mut self, now_ms: u32) {
        self.base_ai.lose_target();
        self.base_ai.finish_active_change_skill(now_ms);
    }

    pub const fn is_hibernated(&self) -> bool {
        self.base_ai.is_hibernated()
    }

    pub fn next_destination(&self) -> Option<PlayerAiDestination> {
        self.destinations.front().copied()
    }

    /// `CPlayerAI::OnSchedule` удаляет назначение после попытки `MoveTo`,
    /// независимо от результата region cast и самого движения; поэтому
    /// изъятие принадлежит самому FIFO-owner-у, но его порядок относительно
    /// `OnMove`/`OnCannotMove` сохраняет caller.
    pub fn finish_destination(&mut self, expected: PlayerAiDestination) -> bool {
        if self.destinations.front().copied() != Some(expected) {
            return false;
        }
        self.destinations.pop_front();
        true
    }

    pub fn begin_destination_move(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.begin_active_move(delay_ms, now_ms);
    }

    /// Хвост `CMoveShape::ForceMove`: spatial mutation уже завершена, после
    /// чего concrete player AI получает ожидание `ASA_STAND` на длительность
    /// принудительного перемещения.
    pub fn begin_forced_stand(&mut self, delay_ms: u32, now_ms: u32) {
        self.base_ai.begin_active_stand(delay_ms, now_ms);
    }

    pub fn stop_destination_move(&mut self) {
        self.base_ai.cancel_active_move();
    }

    /// Native Attack заменяет только m_qTarget; выбранная OnSchedule команда
    /// уже извлечена из FIFO и сохраняется независимо от текущего ID навыка.
    pub fn queue_player_skill(&mut self, dispatch: PlayerSkillDispatch) -> usize {
        Self::replace_pending_skill(&mut self.player_skills, dispatch, |pending, requested| {
            pending.same_pending_request(*requested)
        }).unwrap_or(0)
    }

    pub fn queue_battle_fairy_skill(
        &mut self,
        dispatch: BattleFairySkillDispatch,
    ) -> BattleFairySkillQueueOutcome {
        match Self::replace_pending_skill(&mut self.battle_fairy_skills, dispatch, |pending, requested| {
            pending.same_pending_request(*requested)
        }) {
            None => BattleFairySkillQueueOutcome::PendingUnchanged,
            Some(replaced) => BattleFairySkillQueueOutcome::Queued { replaced },
        }
    }

    /// Общая механика двух независимых очередей: точный повтор головы ничего
    /// не меняет, другой запрос заменяет ожидающие. Текущее исполнение не входит
    /// в эту операцию; разницу wire-отказов применяет CGame по числу замен.
    fn replace_pending_skill<Dispatch>(
        queue: &mut VecDeque<Dispatch>,
        dispatch: Dispatch,
        same_request: impl FnOnce(&Dispatch, &Dispatch) -> bool,
    ) -> Option<usize> {
        if queue.front().is_some_and(|pending| same_request(pending, &dispatch)) {
            return None;
        }
        let replaced = queue.len();
        queue.clear();
        queue.push_back(dispatch);
        Some(replaced)
    }

    pub fn player_skills(&self) -> &VecDeque<PlayerSkillDispatch> {
        &self.player_skills
    }

    pub const fn current_player_skill(&self) -> Option<PlayerSkillDispatch> {
        self.current_player_skill
    }

    /// Pop не меняет текущую цель: riding-отказ выполняется до этой границы.
    pub fn take_pending_player_skill(&mut self) -> Option<PlayerSkillDispatch> {
        self.player_skills.pop_front()
    }

    pub fn select_player_skill(&mut self, dispatch: PlayerSkillDispatch) {
        self.current_player_skill = Some(dispatch);
    }

    pub fn has_current_object_target(&self, target: ShapeIdentity) -> bool {
        self.current_player_skill.and_then(PlayerSkillDispatch::object_target)
            .is_some_and(|current| current.object_type == target.object_type && current.id == target.id)
    }

    pub fn release_current_player_command(&mut self) {
        self.current_player_skill = None;
    }

    /// Свободная WarSoul-очередь очищает прежнюю цель до проверки нового FIFO;
    /// живой kernel не уничтожается этим выбором.
    /// Запрет расписания у мёртвого владельца не останавливает активный AI.
    pub fn begin_next_battle_fairy_skill(
        &mut self,
        can_schedule: bool,
        has_execution: impl Fn(u32) -> bool,
    ) -> Option<BattleFairySkillDispatch> {
        if self.base_ai.active_war_soul_actions().is_empty() {
            if !can_schedule {
                return None;
            }
            self.current_battle_fairy_skill = None;
            if let Some(dispatch) = self.battle_fairy_skills.pop_front() {
                self.current_battle_fairy_skill = Some(dispatch);
                self.selected_battle_fairy_skill_id = dispatch.skill_id();
                // OnScheduleAboutWarSoul извлёк запрос, но IsEnded запрещает
                // повторный Begin уже работающего фонового экземпляра;
                // выбранная цель при этом сохраняется до нового расписания.
                if has_execution(dispatch.skill_id()) {
                    return None;
                }
            }
        }
        self.current_battle_fairy_skill
    }

    pub fn begin_battle_fairy_fighting(&mut self, now_ms: u32) {
        self.base_ai.add_ai_event(AiShapeAction::Attack, 0, 1, now_ms);
    }

    /// OnFightingWithWarSoul (RVA `0x109230`) проверяет IsEnded до вызова AI.
    /// End внутри AI оставляет Attack до следующего Run, без ChangeSkill.
    pub fn finish_battle_fairy_attack(
        &mut self,
        execution: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
        now_ms: u32,
    ) -> bool {
        let Some(handling) = self.base_ai.active_war_soul_actions().front()
            .filter(|event| event.action == AiShapeAction::Attack && matches!(event.handling, 0 | 1))
            .map(|event| event.handling)
        else {
            return false;
        };
        let skill_id = self.selected_battle_fairy_skill_id();
        if handling == 0 && let Some(execution) = execution {
            if !execution.is_prepared() {
                return false;
            }
            self.base_ai.add_started_back_stage_skill(skill_id);
        }
        self.current_battle_fairy_skill = None;
        self.base_ai.finish_war_soul_attack(now_ms);
        true
    }

    /// Исходный игрок сохраняет выбранный навык после его `End`; нулевое
    /// начальное поле Rust кодирует установленную конструктором базовую атаку.
    pub const fn selected_battle_fairy_skill_id(&self) -> u32 {
        if self.selected_battle_fairy_skill_id == 0 {
            BATTLE_FAIRY_BASE_MAGIC_SKILL_ID
        } else {
            self.selected_battle_fairy_skill_id
        }
    }

    pub const fn current_battle_fairy_skill(&self) -> Option<BattleFairySkillDispatch> {
        self.current_battle_fairy_skill
    }

    pub fn release_current_battle_fairy_command(&mut self) {
        self.current_battle_fairy_skill = None;
    }

    /// Точный последний side effect `OnChangeSkillWithWarSoul` и
    /// `OnLoseTargetWarSoul`: ID `0x224` назначается только после полного
    /// concrete `End(1)`, включая оружейный эффект и cooldown.
    pub const fn restore_battle_fairy_base_attack_after_end(&mut self) {
        self.selected_battle_fairy_skill_id = 0;
    }

    pub const fn battle_fairy_skill_is_active(&self) -> bool {
        self.current_battle_fairy_skill.is_some()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn increment_player_progress<Player: AutoIncPlayer>(
        &mut self,
        player: &mut Player,
        interval_ms: u32,
        auto_exp_1: f32,
        auto_exp_2: f32,
        exp_to_vigour_x: u32,
        exp_to_vigour_y: u32,
        maximum_vigour_once: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerAutoProgress> {
        if player.is_dead() || player.faction_id() == 0 {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_last_time_ms = get_tick_ms();

        let level = f64::from(player.level());
        let experience_gain = (((f64::from(player.faction_level()) * 0.05 + 1.0)
            * level.powi(3)
            * f64::from(auto_exp_2)
            + f64::from(auto_exp_1))
            * f64::from(0.000_115_740_74_f32))
        .trunc() as u32;
        if experience_gain == 0 {
            return None;
        }
        let vigour_raw = f64::from(exp_to_vigour_x)
            * f64::from(experience_gain.wrapping_add(600)).log10()
            - f64::from(exp_to_vigour_y);
        let vigour_gain = (vigour_raw.trunc() as i32 as u32).min(maximum_vigour_once);
        let previous_experience = player.experience();
        let previous_vigour = player.vigour();
        player.set_experience(previous_experience.wrapping_add(experience_gain));
        player.set_vigour(previous_vigour.wrapping_add(vigour_gain));
        Some(PlayerAutoProgress {
            player_id: player.player_id(),
            sampled_at_ms,
            experience_gain,
            vigour_gain,
            previous_experience,
            current_experience: player.experience(),
            previous_vigour,
            current_vigour: player.vigour(),
        })
    }

    /// Exact energy tail `CPlayerAI::Run`: первый живой tick только заводит
    /// clock; full energy не двигает его дальше. Due comparison намеренно не
    /// wrap-safe (`last < now - interval`) — это наблюдаемая native-семантика.
    pub fn regenerate_player_energy<Player: AutoIncPlayer>(
        &mut self,
        player: &mut Player,
        interval_ms: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerEnergyRegeneration> {
        if player.is_dead() {
            return None;
        }
        if self.auto_inc_energy_last_time_ms == 0 {
            self.auto_inc_energy_last_time_ms = get_tick_ms();
        }
        let previous_energy = player.energy();
        if previous_energy == player.maximum_energy() {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_energy_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_energy_last_time_ms = get_tick_ms();

        let faction_bonus = if player.faction_id() == 0 {
            0.0
        } else {
            (f64::from(player.level()) * f64::from(0.01_f32))
                .min(1.0)
                .mul_add(f64::from(player.faction_level()) * 0.5, 0.0)
                .max(1.0)
        };
        // MSVC меняет x87 rounding mode на truncation перед `__ftol2`.
        let increment =
            ((f64::from(player.level()) * f64::from(0.1_f32) - 1.0) * 5.0 + faction_bonus + 10.0)
                .trunc() as u32;
        if increment == 0 {
            return None;
        }
        player.set_energy(previous_energy.wrapping_add(increment));
        let current_energy = player.energy();
        (current_energy != previous_energy).then_some(PlayerEnergyRegeneration {
            player_id: player.player_id(),
            sampled_at_ms,
            increment,
            previous_energy,
            current_energy,
        })
    }
}
