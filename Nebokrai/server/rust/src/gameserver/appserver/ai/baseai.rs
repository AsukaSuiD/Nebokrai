//! Достигнутая event-queue часть `CBaseAI` исторического GameServer.
//!
//! `AddAIEvent` RVA `0x000C8F90` имеет статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `server/gameserver/appserver/ai/baseai.cpp`. PDB подтверждает numeric
//! `AI_SHAPE_ACTION` `0..8`, `ASA_FORCE_DWROD = 0xFF`, layout `AI_EVENT`
//! `action/begin/delay/handling +0/+4/+8/+C` и три очереди active/passive/
//! war-soul.
//!
//! `VecDeque` заменяет внутренности `std::queue<std::deque<...>>`, сохраняя
//! FIFO и `push_back`. Время среды выполнения передаётся точным `now_ms` в
//! момент вызова: так владелец Linux не копирует `timeGetTime`, а `u32`
//! сохраняет исходное переполнение. `STIFFEN/DIED/OPEN/DEFENSE` всегда идут
//! в `passive_actions`, остальные — в `active_war_soul_actions` при любом
//! ненулевом флаге и иначе в `active_actions`.
//!
//! Состояние сна также принадлежит этому владельцу: `Hibernate` запоминает
//! оборачивающийся счётчик времени, а `WakeUp` один раз вычисляет интервал сна.
//! Достигнутый `Stand` из `ProcessActiveAction` удерживает расписание до
//! исходного срока и сохраняет отдельный первый такт обработки. `OnIdle` точно
//! ставит следующий `Stand` на 1000 мс только при пустом результате основного
//! прохода и отсутствии цели; `CPlayerAI` применяет эту границу после своих
//! typed очередей. Общий runtime не вызывает очереди и `OnSchedule`, пока этот
//! владелец спит. City/country guard refresh также достигает точный `Clear`
//! обычных active/passive очередей и dormancy-флага без затрагивания war-soul
//! FIFO.
//! Пассивный `Died` также исполняет точный `OnBeenKilled`: из active FIFO
//! сохраняется только первый `Move`, цель отпускается на каждом
//! повторном проходе, а owner death разрешается лишь после завершения этого
//! движения. Указатель владельца и остальные обработчики ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально).

use std::collections::VecDeque;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiShapeAction {
    Stand = 0,
    Move = 1,
    Attack = 2,
    Defense = 3,
    Stiffen = 4,
    SearchEnemy = 5,
    ChangeSkill = 6,
    Died = 7,
    Open = 8,
    ForceDwrod = 0xff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AiEvent {
    pub(crate) action: AiShapeAction,
    pub(crate) beginning_time_ms: u32,
    pub(crate) delay_ms: u32,
    pub(crate) handling: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PassiveDeathAction {
    None,
    WaitingForMove,
    Ready,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CBaseAI {
    active_actions: VecDeque<AiEvent>,
    passive_actions: VecDeque<AiEvent>,
    active_war_soul_actions: VecDeque<AiEvent>,
    is_dormant: bool,
    dormancy_time_ms: u32,
    dormancy_interval_ms: u32,
}

impl CBaseAI {
    pub(crate) fn add_ai_event(
        &mut self,
        action: AiShapeAction,
        delay_ms: u32,
        war_soul_ai: i32,
        now_ms: u32,
    ) {
        let event = AiEvent {
            action,
            beginning_time_ms: now_ms,
            delay_ms,
            handling: 0,
        };
        if matches!(
            action,
            AiShapeAction::Stiffen
                | AiShapeAction::Died
                | AiShapeAction::Open
                | AiShapeAction::Defense
        ) {
            self.passive_actions.push_back(event);
        } else if war_soul_ai != 0 {
            self.active_war_soul_actions.push_back(event);
        } else {
            self.active_actions.push_back(event);
        }
    }

    /// Точный достигнутый префикс `WhenBeenHurted`: `Defense` всегда
    /// становится в очередь `passive_actions`. Ненулевая длительность
    /// оглушения потребует второго отдельного замера часов; материализованные
    /// боевые вызовы передают ноль и не создают `Stiffen`.
    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Defense, 0, 0, now_ms);
    }

    /// Точный достигнутый префикс `WhenBeenKilled`: событие смерти сохраняет
    /// место относительно уже поставленных пассивных действий.
    pub(crate) fn when_been_killed(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Died, 0, 0, now_ms);
    }

    /// Точный наблюдаемый участок `CBaseAI::Clear`, вызываемый при обновлении
    /// city/country guard: очищает обычные active/passive FIFO и снимает сон.
    /// Исходный owner не очищает отдельную war-soul очередь и не обнуляет
    /// сохранённые времена сна, поэтому Rust сохраняет это различие.
    pub(crate) fn clear_guard_refresh_state(&mut self) {
        self.active_actions.clear();
        self.passive_actions.clear();
        self.is_dormant = false;
    }

    /// Выполняет материализованный `Defense`-участок `ProcessPassiveAction`.
    /// Последовательные события снимаются FIFO; `OnBeenHurted` удаляет только
    /// префикс `active_actions` до первого `Attack` либо `Move`.
    pub(crate) fn process_reached_defense_actions(&mut self) -> usize {
        let mut processed = 0usize;
        while self
            .passive_actions
            .front()
            .is_some_and(|event| event.action == AiShapeAction::Defense && event.handling == 0)
        {
            while self.active_actions.front().is_some_and(|event| {
                !matches!(event.action, AiShapeAction::Attack | AiShapeAction::Move)
            }) {
                self.active_actions.pop_front();
            }
            self.passive_actions.pop_front();
            processed = processed.wrapping_add(1);
        }
        processed
    }

    /// Выполняет точный `ASA_DIED -> OnBeenKilled` после того, как caller
    /// отдельным проходом снял предшествующий Defense-префикс. Исходник
    /// отбрасывает все active-события до первого Move, сохраняет только этот
    /// Move со всеми его часами/handling и повторяет обработчик, пока движение
    /// не завершится. Нулевая задержка Died снимает событие в том же проходе,
    /// в котором owner death становится разрешён.
    pub(crate) fn process_reached_death_action(&mut self) -> PassiveDeathAction {
        if !self.passive_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Died && event.handling == 0
        }) {
            return PassiveDeathAction::None;
        }

        let retained_move = self
            .active_actions
            .iter()
            .find(|event| event.action == AiShapeAction::Move)
            .copied();
        self.active_actions.clear();
        if let Some(event) = retained_move {
            self.active_actions.push_back(event);
            PassiveDeathAction::WaitingForMove
        } else {
            self.passive_actions.pop_front();
            PassiveDeathAction::Ready
        }
    }

    /// Выполняет достигнутую `Stand`-ветвь `ProcessActiveAction` и сообщает,
    /// разрешено ли в этом же такте переходить к `OnSchedule`.
    ///
    /// Первый вызов обработчика всегда считается исполнением, даже если
    /// нулевая либо уже истёкшая задержка позволяет сразу снять событие.
    /// Повторный вызов после истечения снимает событие и разрешает расписание
    /// только при пустом остатке FIFO. Другие активные действия пока не
    /// интерпретируются и поэтому продолжают блокировать расписание.
    pub(crate) fn advance_active_stand(&mut self, now_ms: u32) -> bool {
        let Some(event) = self.active_actions.front_mut() else {
            return true;
        };
        if event.action != AiShapeAction::Stand {
            return false;
        }
        if event.handling == 1 {
            if now_ms.wrapping_sub(event.beginning_time_ms) >= event.delay_ms {
                self.active_actions.pop_front();
                return self.active_actions.is_empty();
            }
            return false;
        }
        if event.handling != 0 {
            return false;
        }
        event.handling = 1;
        if now_ms.wrapping_sub(event.beginning_time_ms) >= event.delay_ms {
            self.active_actions.pop_front();
        }
        false
    }

    /// Сообщает, что следующий FIFO-элемент должен выполнить подтверждённый
    /// `OnChangeSkill`. Сам выбор остаётся у конкретного monster AI owner-а.
    pub(crate) fn active_change_skill_pending(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::ChangeSkill && event.handling == 0
        })
    }

    pub(crate) fn active_search_enemy_pending(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::SearchEnemy && event.handling == 0
        })
    }

    pub(crate) fn active_move_unhandled(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Move && event.handling == 0
        })
    }

    pub(crate) fn active_stand_pending(&self) -> bool {
        self.active_actions
            .front()
            .is_some_and(|event| event.action == AiShapeAction::Stand)
    }

    pub(crate) fn active_stand_unhandled(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Stand && event.handling == 0
        })
    }

    /// Сообщает, что следующий FIFO-элемент должен продолжить текущий
    /// `CSkill::AI` через подтверждённый `OnFighting`.
    pub(crate) fn active_attack_pending(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Attack && event.handling == 0
        })
    }

    /// Ставит завершённый пространственный шаг в ту же FIFO-очередь, через
    /// которую исходный `MoveTo` удерживал дальнейшее расписание.
    pub(crate) fn begin_active_move(&mut self, delay_ms: u32, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Move, delay_ms, 0, now_ms);
    }

    pub(crate) fn begin_active_stand(&mut self, delay_ms: u32, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Stand, delay_ms, 0, now_ms);
    }

    /// Материализует точный `CBaseAI::OnIdle`: пустой основной проход без
    /// target-а ставит `ASA_STAND` на 1000 мс. Наличие Rust-owner-а заменяет
    /// исходную проверку `m_pOwner != nullptr`; dormant и непустые очереди
    /// проверяются здесь, чтобы caller не мог обойти начало `CBaseAI::Run`.
    pub(crate) fn begin_idle_stand(&mut self, now_ms: u32) -> bool {
        if self.is_dormant || !self.primary_queues_idle() {
            return false;
        }
        self.begin_active_stand(1_000, now_ms);
        true
    }

    pub(crate) fn begin_active_search_enemy(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::SearchEnemy, 0, 0, now_ms);
    }

    pub(crate) fn cancel_active_move(&mut self) {
        self.active_actions
            .retain(|event| event.action != AiShapeAction::Move);
    }

    /// Выполняет общий `OnMoving`, который возвращает единицу, и сохраняет
    /// исходную границу задержки уже совершённого шага. Производные реакции
    /// лучника, охранника и питомца остаются отдельными проходами владельцев.
    pub(crate) fn advance_active_move(&mut self, now_ms: u32) -> bool {
        let Some(event) = self.active_actions.front_mut() else {
            return false;
        };
        if event.action != AiShapeAction::Move {
            return false;
        }
        if event.handling == 0 {
            event.handling = 1;
        }
        if event.handling == 1
            && now_ms.wrapping_sub(event.beginning_time_ms) >= event.delay_ms
        {
            self.active_actions.pop_front();
        }
        true
    }

    /// Снимает завершённый `ASA_ATTACK` после того, как владелец навыка уже
    /// поставил следующий `ASA_CHANGE_SKILL`. Нулевая исходная задержка
    /// сохраняет их относительный FIFO-порядок.
    pub(crate) fn finish_active_attack(&mut self, now_ms: u32) {
        let Some(event) = self.active_actions.front_mut() else {
            return;
        };
        if event.action != AiShapeAction::Attack || event.handling != 0 {
            return;
        }
        event.handling = 1;
        if now_ms.wrapping_sub(event.beginning_time_ms) >= event.delay_ms {
            self.active_actions.pop_front();
        }
    }

    /// Завершает один вызов `OnChangeSkill`: исходный обработчик возвращает
    /// единицу, поэтому событие с нулевой задержкой снимается в том же проходе,
    /// но `Run` всё равно не вызывает `OnSchedule` до следующего такта.
    pub(crate) fn finish_active_change_skill(&mut self, now_ms: u32) {
        let Some(event) = self.active_actions.front_mut() else {
            return;
        };
        if event.action != AiShapeAction::ChangeSkill || event.handling != 0 {
            return;
        }
        event.handling = 1;
        if now_ms.wrapping_sub(event.beginning_time_ms) >= event.delay_ms {
            self.active_actions.pop_front();
        }
    }

    pub(crate) fn finish_active_search_enemy(&mut self, now_ms: u32) {
        let Some(event) = self.active_actions.front_mut() else {
            return;
        };
        if event.action != AiShapeAction::SearchEnemy || event.handling != 0 {
            return;
        }
        event.handling = 1;
        if now_ms.wrapping_sub(event.beginning_time_ms) >= event.delay_ms {
            self.active_actions.pop_front();
        }
    }

    pub(crate) fn active_actions(&self) -> &VecDeque<AiEvent> {
        &self.active_actions
    }

    pub(crate) fn passive_actions(&self) -> &VecDeque<AiEvent> {
        &self.passive_actions
    }

    pub(crate) fn active_war_soul_actions(&self) -> &VecDeque<AiEvent> {
        &self.active_war_soul_actions
    }

    /// Соответствует проверке размеров первых двух исходных очередей в
    /// `OnSchedule`; очередь боевого духа в это условие не входит.
    pub(crate) fn primary_queues_idle(&self) -> bool {
        self.active_actions.is_empty() && self.passive_actions.is_empty()
    }

    pub(crate) fn hibernate(&mut self, now_ms: u32) {
        self.is_dormant = true;
        self.dormancy_time_ms = now_ms;
        self.dormancy_interval_ms = 0;
    }

    pub(crate) fn wake_up(&mut self, now_ms: u32) -> u32 {
        self.dormancy_interval_ms = now_ms.wrapping_sub(self.dormancy_time_ms);
        self.dormancy_time_ms = 0;
        self.is_dormant = false;
        self.dormancy_interval_ms
    }

    pub(crate) const fn is_hibernated(&self) -> bool {
        self.is_dormant
    }
}

/// Общая длительность одного шага `CBaseAI::MoveTo`: диагональ длиннее
/// осевого шага, после чего прибавляется остановочный кадр владельца.
pub(crate) fn one_step_move_delay_ms(direction: i32, speed: f32, stop_frame: u32) -> u32 {
    let distance_units = if direction % 2 == 0 {
        1_000_000.0
    } else {
        1_414_000.0
    };
    if speed > 0.0 {
        (distance_units * 0.68 / speed + stop_frame as f32)
            .round()
            .max(0.0) as u32
    } else {
        0
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp

// ============================================================================
// FUNCTION: CBaseAI::Hibernate
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CBaseAI::hibernate`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:86
// RVA: 0x000C7C10
// ADDRESS: 004c7c10
// PROTOTYPE: void __thiscall Hibernate(void)
//
// ============================================================================
// FUNCTION: CBaseAI::WakeUp
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CBaseAI::wake_up`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:100
// RVA: 0x000C7C30
// ADDRESS: 004c7c30
// PROTOTYPE: void __thiscall WakeUp(void)
//
// ============================================================================
// FUNCTION: CBaseAI::IsHibernated
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: `CBaseAI::is_hibernated`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:113
// RVA: 0x000C7C50
// ADDRESS: 004c7c50
// PROTOTYPE: int __thiscall IsHibernated(void)
//
// ============================================================================
// FUNCTION: CBaseAI::SetTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:212
// RVA: 0x000C7C60
// ADDRESS: 004c7c60
// PROTOTYPE: void __thiscall SetTarget(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::GetOwner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:225
// RVA: 0x000C7C80
// ADDRESS: 004c7c80
// PROTOTYPE: CMoveShape * __thiscall GetOwner(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::SetOwner
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:237
// RVA: 0x000C7C90
// ADDRESS: 004c7c90
// PROTOTYPE: void __thiscall SetOwner(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CBaseAI::Run
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CGame` проверяет `CBaseAI::is_hibernated` до обработки
// очередей и конкретного `OnSchedule`; достигнутая `Stand`-ветвь исполняется
// через `CBaseAI::advance_active_stand`, а пустой player-проход завершает
// точный `CBaseAI::begin_idle_stand`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:472
// RVA: 0x000C7D10
// ADDRESS: 004c7d10
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::ProcessBackStageAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:799
// RVA: 0x000C7D90
// ADDRESS: 004c7d90
// PROTOTYPE: AI_EXEC_STATE __thiscall ProcessBackStageAction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1110
// RVA: 0x000C7DA0
// ADDRESS: 004c7da0
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::WhenLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1199
// RVA: 0x000C7DC0
// ADDRESS: 004c7dc0
// PROTOTYPE: void __thiscall WhenLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::HasTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1350
// RVA: 0x000C7DD0
// ADDRESS: 004c7dd0
// PROTOTYPE: int __thiscall HasTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::CheckSkillIsWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1364
// RVA: 0x000C7E00
// ADDRESS: 004c7e00
// PROTOTYPE: bool __thiscall CheckSkillIsWarSoul(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::GetTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:183
// RVA: 0x000C7E10
// ADDRESS: 004c7e10
// PROTOTYPE: CMoveShape * __thiscall GetTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::DoesBackStageSkillExist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1230
// RVA: 0x000C7E80
// ADDRESS: 004c7e80
// PROTOTYPE: int __thiscall DoesBackStageSkillExist(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:59
// RVA: 0x000C7F70
// ADDRESS: 004c7f70
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::MoveTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:304
// RVA: 0x000C7FF0
// ADDRESS: 004c7ff0
// PROTOTYPE: int __thiscall MoveTo(long param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::GetCurrentActiveAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:252
// RVA: 0x000C81B0
// ADDRESS: 004c81b0
// PROTOTYPE: AI_SHAPE_ACTION __thiscall GetCurrentActiveAction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::ProcessActiveAction
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CBaseAI::advance_active_stand` сохраняет достигнутую ветвь
// `ASA_STAND`, `advance_active_move` — задержку `ASA_MOVE`,
// `active_attack_pending` — продолжение `ASA_ATTACK`, отдельный такт
// `ASA_SEARCH_ENEMY` вызывает достигнутые функции выбора целей, а
// `finish_active_change_skill` — отдельный такт `ASA_CHANGE_SKILL`, их
// FIFO-позицию, handling и границу задержки.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:531
// RVA: 0x000C81D0
// ADDRESS: 004c81d0
// PROTOTYPE: AI_EXEC_STATE __thiscall ProcessActiveAction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::ProcessActiveActionWarSoul
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: общий `CMonsterAI::Tracing` выполняет пространственный шаг и
// ставит `ASA_MOVE` с подтверждённой длительностью в каноническую FIFO-очередь.
// Остались другие виртуальные варианты движения и реакции производных `OnMoving`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:612
// RVA: 0x000C8390
// ADDRESS: 004c8390
// PROTOTYPE: AI_EXEC_STATE __thiscall ProcessActiveActionWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::ProcessPassiveAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:693
// RVA: 0x000C84F0
// ADDRESS: 004c84f0
// PROTOTYPE: AI_EXEC_STATE __thiscall ProcessPassiveAction(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:815
// RVA: 0x000C8700
// ADDRESS: 004c8700
// PROTOTYPE: int __thiscall OnBeenHurted(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnStiffen
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:847
// RVA: 0x000C8770
// ADDRESS: 004c8770
// PROTOTYPE: int __thiscall OnStiffen(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::~CBaseAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:43
// RVA: 0x000C8890
// ADDRESS: 004c8890
// PROTOTYPE: void __thiscall ~CBaseAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnExecuteBackStageSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1003
// RVA: 0x000C88E0
// ADDRESS: 004c88e0
// PROTOTYPE: int __thiscall OnExecuteBackStageSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004c8998
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1039
// RVA: 0x000C8998
// ADDRESS: 004c8998
// PROTOTYPE: undefined Catch@004c8998()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::CBaseAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:20
// RVA: 0x000C8EA0
// ADDRESS: 004c8ea0
// PROTOTYPE: undefined __thiscall CBaseAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CBaseAI::AddAIEvent` материализован выше; покрытый raw-блок
// удалён.

// ============================================================================
// FUNCTION: CBaseAI::MoveTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:397
// RVA: 0x000C9020
// ADDRESS: 004c9020
// PROTOTYPE: void __thiscall MoveTo(CRegion * param_1, long param_2, long param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnBeenKilled
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:916
// RVA: 0x000C9220
// ADDRESS: 004c9220
// PROTOTYPE: int __thiscall OnBeenKilled(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnFighting
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: активный `ASA_ATTACK` продолжает материализованный навык, а
// после его завершения ставит `ASA_CHANGE_SKILL`; недостигнутые варианты
// `CSkill::AI` остаются RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:964
// RVA: 0x000C9320
// ADDRESS: 004c9320
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::OnIdle
// STATUS: IMPLEMENTED
// IMPLEMENTED: `CBaseAI::begin_idle_stand`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1134
// RVA: 0x000C93A0
// ADDRESS: 004c93a0
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::WhenBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1165
// RVA: 0x000C93E0
// ADDRESS: 004c93e0
// PROTOTYPE: void __thiscall WhenBeenHurted(long param_1, long param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::WhenBeenKilled
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1183
// RVA: 0x000C9460
// ADDRESS: 004c9460
// PROTOTYPE: void __thiscall WhenBeenKilled(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::WhenAddBackStageSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1213
// RVA: 0x000C94B0
// ADDRESS: 004c94b0
// PROTOTYPE: void __thiscall WhenAddBackStageSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::Tracing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:1277
// RVA: 0x000C94D0
// ADDRESS: 004c94d0
// PROTOTYPE: int __thiscall Tracing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::SetAIType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:147
// RVA: 0x000D7040
// ADDRESS: 004d7040
// PROTOTYPE: void __thiscall SetAIType(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTeamate::GetOwnerRegionID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:171
// RVA: 0x000EA290
// ADDRESS: 004ea290
// PROTOTYPE: long __thiscall GetOwnerRegionID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetGoodsType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:159
// RVA: 0x00104620
// ADDRESS: 00504620
// PROTOTYPE: GOODS_TYPE __thiscall GetGoodsType(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAI::GetDormancyInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\baseai.cpp:124
// RVA: 0x00106F10
// ADDRESS: 00506f10
// PROTOTYPE: ulong __thiscall GetDormancyInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
