//! Порядковая механика passive-реакций `CBaseAI` исторического GameServer
//! над FIFO-очередями hub-владельца поведения.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match). Публичные
//! символы семейства: `CBaseAI::ProcessPassiveAction` (`1:0x0c74f0` → RVA
//! `0x0c84f0`), `CBaseAI::OnBeenHurted` (`1:0x0c7700` → RVA `0x0c8700`),
//! `CBaseAI::OnStiffen` (`1:0x0c7770` → RVA `0x0c8770`),
//! `CBaseAI::OnBeenKilled` (`1:0x0c8220` → RVA `0x0c9220`).
//! Исходный владелец PDB: `server/gameserver/appserver/ai/baseai.cpp`.
//!
//! Базовый `OnBeenHurted` удаляет только префикс active-очереди до первого
//! `Attack` либо `Move` и возвращает 1. `OnStiffen` вызывает `CSkill::End(4)`
//! только для `Attack` с `handling == 0`, а `Attack` остаётся в FIFO, пока
//! concrete владелец не подтвердит завершение навыка через `IsEnded` — этот
//! ответ лежит вне хранилищ очередей и остаётся hub-владением вместе с самими
//! очередями. Очистка префикса останавливается на `Move` и не снимает его.
//! `OnBeenKilled` отбрасывает все active-события до первого `Move`, сохраняет
//! только этот `Move` со всеми его часами/`handling` и повторяет обработчик,
//! пока движение не завершится; смерть owner-а разрешается лишь после него.
//! `ProcessPassiveAction` вызывает производный обработчик для каждого
//! `Defense` до его pop, поэтому следующий `Defense` видит мутации
//! active-очереди предыдущего, и различает ноль, единицу и прочие знаковые
//! `handling`: только единица допускает снятие по deadline. War-soul FIFO
//! реакциями не затрагивается никогда.

use std::collections::VecDeque;

use super::events::{ai_event_deadline_reached, AiEvent, AiShapeAction};

/// Стадия passive-реакции смерти владельца поведения (`OnBeenKilled`):
/// наличие достигнутого `Died` и готовность единственного сохранённого Move.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassiveDeathAction {
    None,
    WaitingForMove,
    Ready,
}

/// Стадия passive-реакции оглушения (`OnStiffen`): различает начатое
/// прерывание атаки, ожидание исходного stun deadline и их завершения вместе
/// со снятием `Stiffen` из passive FIFO.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassiveStiffenAction {
    None,
    InterruptAttack,
    InterruptAttackFinished,
    StartedWaiting,
    StartedFinished,
    Waiting,
    Finished,
}

impl PassiveStiffenAction {
    /// `ProcessPassiveAction` (RVA `0x0c84f0`): незавершённый Stiffen
    /// возвращает AES_HUNG_UP, снятый по сроку — AES_IDLE/AES_EXEC. Сам факт
    /// вызова OnStiffen или End(4) не запрещает последующий ProcessActiveAction.
    pub const fn blocks_active(self) -> bool {
        matches!(self, Self::InterruptAttack | Self::StartedWaiting | Self::Waiting)
    }

    pub const fn interrupts_attack(self) -> bool {
        matches!(self, Self::InterruptAttack | Self::InterruptAttackFinished)
    }
}

/// Узкая generic-сварка hub-владельца очередей к порядковой механике
/// реакций Zone. Поля, постановка событий и active-фаза остаются
/// hub-владением; сварка открывает только две FIFO, которые реакции
/// переставляют и снимают в исходном порядке.
pub trait PassiveReactionQueues {
    /// Passive FIFO владельца (`STIFFEN`/`DIED`/`OPEN`/`DEFENSE`).
    fn passive_reaction_queue(&self) -> &VecDeque<AiEvent>;

    /// Обычная active FIFO владельца; war-soul очередь сюда не входит.
    fn active_reaction_queue(&self) -> &VecDeque<AiEvent>;

    fn passive_reaction_queue_mut(&mut self) -> &mut VecDeque<AiEvent>;

    fn active_reaction_queue_mut(&mut self) -> &mut VecDeque<AiEvent>;
}

/// Общий префикс Defense/Stiffen до первой границы Attack/Move.
/// OnStiffen продолжает обход после виртуального OnLoseTarget/default.
/// Производный callback должен увидеть ещё не удалённый хвост очереди.
pub fn discard_active_prefix<Owner: PassiveReactionQueues>(owner: &mut Owner) {
    while owner.active_reaction_queue().front().is_some_and(|event| {
        !matches!(event.action, AiShapeAction::Attack | AiShapeAction::Move)
    }) {
        owner.active_reaction_queue_mut().pop_front();
    }
}

/// Выполняет материализованный `Defense`-участок `ProcessPassiveAction`
/// (RVA `0x0c84f0`). Последовательные события снимаются FIFO; `OnBeenHurted`
/// (RVA `0x0c8700`) удаляет только префикс active-очереди до первого `Attack`
/// либо `Move`, а производный обработчик вызывается для каждого события до
/// его pop. Callback находится внутри FIFO-прохода: следующий `Defense` видит
/// мутации active-очереди предыдущего.
pub fn process_reached_defense_actions<Owner: PassiveReactionQueues>(
    owner: &mut Owner,
    mut after_base_handler: impl FnMut(&mut Owner),
) -> usize {
    let mut processed = 0usize;
    while owner
        .passive_reaction_queue()
        .front()
        .is_some_and(|event| event.action == AiShapeAction::Defense && event.handling == 0)
    {
        discard_active_prefix(owner);
        after_base_handler(owner);
        owner.passive_reaction_queue_mut().pop_front();
        processed = processed.wrapping_add(1);
    }
    processed
}

/// Материализует exact `ASA_STIFFEN -> OnStiffen` (RVA `0x0c8770`) и
/// deadline-часть `ProcessPassiveAction`. До первой атаки/движения
/// active-префикс отбрасывается; движение сохраняется, атака остаётся в FIFO
/// до ответа concrete owner-а на `CSkill::End(4)` и проверку `IsEnded`.
pub fn begin_reached_stiffen_action<Owner: PassiveReactionQueues>(
    owner: &mut Owner,
) -> PassiveStiffenAction {
    let Some(event) = owner.passive_reaction_queue().front() else {
        return PassiveStiffenAction::None;
    };
    if event.action != AiShapeAction::Stiffen {
        return PassiveStiffenAction::None;
    }
    if event.handling != 0 {
        return PassiveStiffenAction::Waiting;
    }
    discard_active_prefix(owner);
    if owner
        .active_reaction_queue()
        .front()
        .is_some_and(|event| event.action == AiShapeAction::Attack)
    {
        PassiveStiffenAction::InterruptAttack
    } else {
        PassiveStiffenAction::StartedWaiting
    }
}

/// `ProcessPassiveAction` фиксирует результат OnStiffen и срок после callback.
pub fn finish_reached_stiffen_action<Owner: PassiveReactionQueues>(
    owner: &mut Owner,
    begun: PassiveStiffenAction,
    now: impl FnOnce() -> u32,
) -> PassiveStiffenAction {
    if begun == PassiveStiffenAction::None {
        return PassiveStiffenAction::None;
    }
    let Some(event) = owner
        .passive_reaction_queue_mut()
        .front_mut()
        .filter(|event| event.action == AiShapeAction::Stiffen)
    else {
        return PassiveStiffenAction::None;
    };
    if begun == PassiveStiffenAction::Waiting {
        if event.handling != 1 {
            return PassiveStiffenAction::Waiting;
        }
    } else {
        event.handling = 1;
    }
    if !ai_event_deadline_reached(event, now()) {
        return begun;
    }
    owner.passive_reaction_queue_mut().pop_front();
    if begun.interrupts_attack() {
        PassiveStiffenAction::InterruptAttackFinished
    } else if begun == PassiveStiffenAction::StartedWaiting {
        PassiveStiffenAction::StartedFinished
    } else {
        PassiveStiffenAction::Finished
    }
}

/// Выполняет точный `ASA_DIED -> OnBeenKilled` (RVA `0x0c9220`) после того,
/// как caller отдельным проходом снял предшествующий Defense-префикс. Исходник
/// отбрасывает все active-события до первого Move, сохраняет только этот Move
/// со всеми его часами/handling и повторяет обработчик, пока движение не
/// завершится. Снятие Died после virtual `OnLoseTarget` выполняет парный
/// `finish_reached_death_action` после обработчика смерти owner-а.
pub fn begin_reached_death_action<Owner: PassiveReactionQueues>(owner: &mut Owner) -> bool {
    if !owner.passive_reaction_queue().front().is_some_and(|event| {
        event.action == AiShapeAction::Died && event.handling == 0
    }) {
        return false;
    }

    let retained_move = owner
        .active_reaction_queue()
        .iter()
        .find(|event| event.action == AiShapeAction::Move)
        .copied();
    owner.active_reaction_queue_mut().clear();
    if let Some(event) = retained_move {
        owner.active_reaction_queue_mut().push_back(event);
    }
    true
}

/// Проверяет готовность после OnLoseTarget, не снимая Died до owner OnDied.
pub fn reached_death_action_state<Owner: PassiveReactionQueues>(
    owner: &Owner,
) -> PassiveDeathAction {
    if !owner.passive_reaction_queue().front().is_some_and(|event| {
        event.action == AiShapeAction::Died && event.handling == 0
    }) {
        return PassiveDeathAction::None;
    }
    if owner.active_reaction_queue().is_empty() {
        PassiveDeathAction::Ready
    } else {
        PassiveDeathAction::WaitingForMove
    }
}

pub fn finish_reached_death_action<Owner: PassiveReactionQueues>(owner: &mut Owner, now_ms: u32) {
    let Some(event) = owner.passive_reaction_queue_mut().front_mut() else {
        return;
    };
    if event.action != AiShapeAction::Died || !matches!(event.handling, 0 | 1) {
        return;
    }
    event.handling = 1;
    if ai_event_deadline_reached(event, now_ms) {
        owner.passive_reaction_queue_mut().pop_front();
    }
}
