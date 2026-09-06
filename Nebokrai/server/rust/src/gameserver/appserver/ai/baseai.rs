//! Достигнутая event-queue часть `CBaseAI` исторического GameServer.
//! Run (0x004C7D10) допускает OnIdle только при AES_IDLE всех основных фаз.
//! Defense и первый OnStiffen возвращают EXEC даже после снятия последнего
//! события: monster caller сохраняет результат passive-фазы отдельно от
//! пустоты FIFO. EXEC разрешает active-фазу, но не последующий OnIdle.
//! Базовый OnBeenHurted: EXE/PDB GameServer, appserver/ai/baseai.cpp:815,
//! RVA 0x000C8700; discard_active_prefix сохраняет первую Attack/Move-границу.
//! Defense в ProcessPassiveAction (0x004C84F0) вызывает производный
//! OnBeenHurted для каждого элемента до его pop. Callback находится внутри
//! FIFO-прохода: следующий Defense видит мутации active-очереди предыдущего.
//! ProcessPassiveAction различает ноль, единицу и прочие знаковые handling:
//! только единица допускает снятие по deadline. Иное ненулевое значение
//! не вызывает обработчик снова и не нормализуется в успешное завершение.
//! То же различие сохраняют ProcessActiveAction (0x004C81D0) и WarSoul
//! (0x004C8390): общий завершитель принимает только handling 0 или 1,
//! а не весь знаковый диапазон до единицы.
//! Уже обработанная голова active FIFO проверяет deadline до dispatch любого
//! action. Даже её снятие завершает этот проход, не исполняя следующий элемент.
//! Общая операция обслуживает раздельные обычную и WarSoul очереди.
//! Passive-фаза также сначала проверяет ненулевой handling: снятие головы
//! завершает только passive-проход, а ожидание любого action, кроме Move,
//! возвращает блокировку active-фазы. Следующий passive-handler не вызывается.
//! OnBeenKilled (0x004C9220) проверяет active FIFO после OnLoseTarget,
//! затем вызывает owner OnDied. Died остаётся в passive FIFO до возврата
//! owner-а; только после него ProcessPassiveAction фиксирует handling и deadline.
//! Проверка срока Stiffen получает часы лениво после прерывания навыка
//! и записи handling, а не использует время начала внешнего прохода монстра.
//!
//! `AddAIEvent` RVA `0x000C8F90` имеет статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `server/gameserver/appserver/ai/baseai.cpp`. PDB подтверждает numeric
//! `AI_SHAPE_ACTION` `0..8`, `ASA_FORCE_DWROD = 0xFF`, layout `AI_EVENT`
//! `action/begin/delay/handling +0/+4/+8/+C` и три очереди active/passive/
//! war-soul.
//! GetCurrentActiveAction (0x004C81B0) читает только action первого элемента,
//! без проверки handling; пустая очередь представлена None вместо native -1.
//!
//! `VecDeque` заменяет внутренности `std::queue<std::deque<...>>`, сохраняя
//! FIFO и `push_back`. Время среды выполнения передаётся точным `now_ms` в
//! момент вызова: так владелец Linux не копирует `timeGetTime`, а `u32`
//! сохраняет исходное переполнение. Deadline событий сравнивается после
//! wrapping DWORD-сложения `begin + delay`, включая раннее завершение рядом
//! с переполнением часов. `STIFFEN/DIED/OPEN/DEFENSE` всегда идут
//! в `passive_actions`, остальные — в `active_war_soul_actions` при любом
//! ненулевом флаге и иначе в `active_actions`.
//!
//! Состояние сна также принадлежит этому владельцу: `Hibernate` запоминает
//! оборачивающийся счётчик времени, а `WakeUp` один раз вычисляет интервал сна.
//! Достигнутый `Stand` из `ProcessActiveAction` удерживает расписание до
//! исходного срока и сохраняет отдельный первый такт обработки. `OnIdle` точно
//! ставит следующий `Stand` на 1000 мс только при пустом результате основного
//! прохода и отсутствии цели. Это базовый обработчик, не поведение игрока:
//! слот +0x48 CPlayerAI указывает на пустой RET 0x00485540. Общий runtime
//! не вызывает очереди и `OnSchedule`, пока этот
//! владелец спит. Достигнутая object-target часть `SetTarget`, `GetTarget`,
//! `HasTarget` и `OnLoseTarget` также принадлежит этому owner-у. City/country
//! guard refresh достигает `Clear` обычных active/passive очередей,
//! object-цели и dormancy-флага без затрагивания war-soul FIFO.
//! Пассивный `Died` также исполняет точный `OnBeenKilled`: из active FIFO
//! сохраняется только первый `Move`, цель отпускается на каждом
//! повторном проходе, а owner death разрешается лишь после завершения этого
//! движения. `WhenBeenHurted` сохраняет отдельные часы `Defense`/`Stiffen`, а
//! достигнутый monster runtime прерывает атаку, теряет цель, сохраняет движение
//! и удерживает passive FIFO до исходного stun deadline.
//! PassiveStiffenAction отдельно сообщает прерывание и AES_HUNG_UP: End(4)
//! при уже достигнутом сроке не задерживает active. Обработанный Defense
//! возвращает AES_EXEC, поэтому также не запрещает active игрока/монстра.
//! Фоновая фаза обоих владельцев предшествует passive и не зависит от stun.
//! OnStiffen 0x004C87D9/0x004C87E0 сначала вызывает End(4), затем IsEnded:
//! Attack остаётся в FIFO, пока concrete владелец не подтвердит завершение.
//! Для handling != 0 или отсутствующего навыка End не вызывается. Очистка
//! префикса останавливается на Move и не снимает его.
//! Указатель владельца и
//! остальные обработчики ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Публичный `MoveTo`
//! сохраняет исходный промежуточный `float` времени шага и усекает сумму со
//! stop-frame к нулю перед постановкой действия в очередь.

use std::collections::VecDeque;

use crate::gameserver::appserver::shape::ShapeIdentity;

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

/// Exact unsigned deadline из `ProcessActiveAction/ProcessPassiveAction`:
/// исходник сначала складывает два DWORD, затем сравнивает результат с
/// текущими часами. Это намеренно не эквивалентно elapsed-сравнению в момент
/// переполнения `timeGetTime`.
const fn ai_event_deadline_reached(event: &AiEvent, now_ms: u32) -> bool {
    event.beginning_time_ms.wrapping_add(event.delay_ms) <= now_ms
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PassiveDeathAction {
    None,
    WaitingForMove,
    Ready,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PassiveStiffenAction {
    None,
    InterruptAttack,
    InterruptAttackFinished,
    StartedWaiting,
    StartedFinished,
    Waiting,
    Finished,
}

impl PassiveStiffenAction {
    /// ProcessPassiveAction (0x004C84F0): незавершённый Stiffen возвращает
    /// AES_HUNG_UP, снятый по сроку — AES_IDLE/AES_EXEC. Сам факт вызова
    /// OnStiffen или End(4) не запрещает последующий ProcessActiveAction.
    pub(crate) const fn blocks_active(self) -> bool {
        matches!(self, Self::InterruptAttack | Self::StartedWaiting | Self::Waiting)
    }

    pub(crate) const fn interrupts_attack(self) -> bool {
        matches!(self, Self::InterruptAttack | Self::InterruptAttackFinished)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CBaseAI {
    active_actions: VecDeque<AiEvent>,
    passive_actions: VecDeque<AiEvent>,
    active_war_soul_actions: VecDeque<AiEvent>,
    is_dormant: bool,
    dormancy_time_ms: u32,
    dormancy_interval_ms: u32,
    target_id: i32,
    target_type: i32,
}

impl CBaseAI {
    /// Точный достигнутый `SetTarget(long, long)` object identity.
    pub(crate) const fn set_object_target(&mut self, target: ShapeIdentity) {
        self.target_type = target.object_type;
        self.target_id = target.id;
    }

    /// Материализованная identity-часть `GetTarget`. Разрешение объекта
    /// остаётся у region owner-а, а нулевые legacy-поля не образуют lookup.
    pub(crate) const fn object_target(&self) -> Option<ShapeIdentity> {
        if self.target_type == 0 || self.target_id == 0 {
            return None;
        }
        Some(ShapeIdentity {
            object_type: self.target_type,
            id: self.target_id,
            ex_id: crate::public::guid::CGuid::GUID_INVALID,
        })
    }

    /// Достигнутая object-ветвь `HasTarget` требует строго положительные
    /// type/ID. Point-цель пока остаётся у недостигнутого coordinate caller-а.
    pub(crate) const fn has_object_target(&self) -> bool {
        self.target_id > 0 && self.target_type > 0
    }

    /// Достигнутая object-часть `OnLoseTarget`.
    pub(crate) const fn lose_target(&mut self) {
        self.target_id = 0;
        self.target_type = 0;
    }

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

    /// Точный первый event `WhenBeenHurted`: `Defense` всегда становится в
    /// `passive_actions`; caller отдельно добавляет Stiffen после собственного
    /// второго замера часов.
    pub(crate) fn when_been_hurted(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Defense, 0, 0, now_ms);
    }

    pub(crate) fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        if delay_ms != 0 {
            self.add_ai_event(AiShapeAction::Stiffen, delay_ms, 0, now_ms);
        }
    }

    /// Точный достигнутый префикс `WhenBeenKilled`: событие смерти сохраняет
    /// место относительно уже поставленных пассивных действий.
    pub(crate) fn when_been_killed(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Died, 0, 0, now_ms);
    }

    /// Точный наблюдаемый участок `CBaseAI::Clear`, вызываемый при обновлении
    /// city/country guard: очищает обычные active/passive FIFO, object-цель и
    /// снимает сон. Исходный owner не очищает отдельную war-soul очередь и
    /// сохранённые времена сна, поэтому Rust сохраняет различие.
    pub(crate) fn clear_guard_refresh_state(&mut self) {
        self.active_actions.clear();
        self.passive_actions.clear();
        self.target_id = 0;
        self.target_type = 0;
        self.is_dormant = false;
    }

    /// None — требуется dispatch; Some — проход занят, значение блокирует active.
    pub(crate) fn advance_handled_passive_action(&mut self, now: impl FnOnce() -> u32) -> Option<bool> {
        let event = self.passive_actions.front().filter(|event| event.handling != 0)?;
        if event.handling == 1 && ai_event_deadline_reached(event, now()) {
            self.passive_actions.pop_front();
            return Some(false);
        }
        Some(event.action != AiShapeAction::Move)
    }

    /// Выполняет материализованный `Defense`-участок `ProcessPassiveAction`.
    /// Последовательные события снимаются FIFO; `OnBeenHurted` удаляет только
    /// префикс `active_actions` до первого `Attack` либо `Move`.
    pub(crate) fn process_reached_defense_actions(
        &mut self,
        mut after_base_handler: impl FnMut(&mut Self),
    ) -> usize {
        let mut processed = 0usize;
        while self
            .passive_actions
            .front()
            .is_some_and(|event| event.action == AiShapeAction::Defense && event.handling == 0)
        {
            self.discard_active_prefix();
            after_base_handler(self);
            self.passive_actions.pop_front();
            processed = processed.wrapping_add(1);
        }
        processed
    }

    /// Материализует exact `ASA_STIFFEN -> OnStiffen` и deadline-часть
    /// `ProcessPassiveAction`. До первой атаки/движения active-префикс
    /// отбрасывается; движение сохраняется, атака остаётся в FIFO до ответа
    /// concrete owner-а на `CSkill::End(4)` и проверку `IsEnded`.
    pub(crate) fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        let Some(event) = self.passive_actions.front() else {
            return PassiveStiffenAction::None;
        };
        if event.action != AiShapeAction::Stiffen {
            return PassiveStiffenAction::None;
        }
        if event.handling != 0 {
            return PassiveStiffenAction::Waiting;
        }
        self.discard_active_prefix();
        if self.stiffen_attack_pending() {
            PassiveStiffenAction::InterruptAttack
        } else {
            PassiveStiffenAction::StartedWaiting
        }
    }

    /// ProcessPassiveAction фиксирует результат OnStiffen и срок после callback.
    pub(crate) fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        if begun == PassiveStiffenAction::None {
            return PassiveStiffenAction::None;
        }
        let Some(event) = self.passive_actions.front_mut()
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
        self.passive_actions.pop_front();
        if begun.interrupts_attack() {
            PassiveStiffenAction::InterruptAttackFinished
        } else if begun == PassiveStiffenAction::StartedWaiting {
            PassiveStiffenAction::StartedFinished
        } else {
            PassiveStiffenAction::Finished
        }
    }

    pub(crate) fn current_active_action(&self) -> Option<AiShapeAction> {
        self.active_actions.front().map(|event| event.action)
    }

    /// OnStiffen (0x004C87C0) вызывает End(4) только для Attack с handling=0.
    pub(crate) fn stiffen_attack_pending(&self) -> bool {
        self.current_active_action() == Some(AiShapeAction::Attack)
    }

    pub(crate) fn stiffen_attack_needs_end(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Attack && event.handling == 0
        })
    }

    /// Вызывается после подтверждённого IsEnded либо при отсутствии навыка.
    /// Нельзя снимать Attack до End: производный навык может остаться активным.
    pub(crate) fn finish_stiffen_attack(&mut self, release_target: bool) {
        if self.active_actions.front().is_some_and(|event| event.action == AiShapeAction::Attack) {
            self.active_actions.pop_front();
            if release_target {
                self.lose_target();
            }
        }
    }

    /// Общий префикс Defense/Stiffen до первой границы Attack/Move.
    /// OnStiffen продолжает обход после виртуального OnLoseTarget/default.
    /// Производный callback должен увидеть ещё не удалённый хвост очереди.
    pub(crate) fn discard_active_prefix(&mut self) {
        while self.active_actions.front().is_some_and(|event| {
            !matches!(event.action, AiShapeAction::Attack | AiShapeAction::Move)
        }) {
            self.active_actions.pop_front();
        }
    }

    /// Выполняет точный `ASA_DIED -> OnBeenKilled` после того, как caller
    /// отдельным проходом снял предшествующий Defense-префикс. Исходник
    /// отбрасывает все active-события до первого Move, сохраняет только этот
    /// Move со всеми его часами/handling и повторяет обработчик, пока движение
    /// не завершится. Снятие Died после virtual `OnLoseTarget` выполняет
    /// парный `finish_reached_death_action` после обработчика смерти owner-а.
    pub(crate) fn begin_reached_death_action(&mut self) -> bool {
        if !self.passive_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Died && event.handling == 0
        }) {
            return false;
        }

        let retained_move = self
            .active_actions
            .iter()
            .find(|event| event.action == AiShapeAction::Move)
            .copied();
        self.active_actions.clear();
        if let Some(event) = retained_move {
            self.active_actions.push_back(event);
        }
        true
    }

    /// Проверяет готовность после OnLoseTarget, не снимая Died до owner OnDied.
    pub(crate) fn reached_death_action_state(&self) -> PassiveDeathAction {
        if !self.passive_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Died && event.handling == 0
        }) {
            return PassiveDeathAction::None;
        }
        if self.active_actions.is_empty() {
            PassiveDeathAction::Ready
        } else {
            PassiveDeathAction::WaitingForMove
        }
    }

    pub(crate) fn finish_reached_death_action(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.passive_actions, AiShapeAction::Died, now_ms);
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
            if ai_event_deadline_reached(event, now_ms) {
                self.active_actions.pop_front();
                return self.active_actions.is_empty();
            }
            return false;
        }
        if event.handling != 0 {
            return false;
        }
        event.handling = 1;
        if ai_event_deadline_reached(event, now_ms) {
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
        if event.handling == 1 && ai_event_deadline_reached(event, now_ms) {
            self.active_actions.pop_front();
        }
        true
    }

    /// Снимает завершённый `ASA_ATTACK` после того, как владелец навыка уже
    /// поставил следующий `ASA_CHANGE_SKILL`. Нулевая исходная задержка
    /// сохраняет их относительный FIFO-порядок.
    pub(crate) fn finish_active_attack(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.active_actions, AiShapeAction::Attack, now_ms);
    }

    /// Завершает один вызов `OnChangeSkill`: исходный обработчик возвращает
    /// единицу, поэтому событие с нулевой задержкой снимается в том же проходе,
    /// но `Run` всё равно не вызывает `OnSchedule` до следующего такта.
    pub(crate) fn finish_active_change_skill(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.active_actions, AiShapeAction::ChangeSkill, now_ms);
    }

    pub(crate) fn finish_war_soul_attack(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.active_war_soul_actions, AiShapeAction::Attack, now_ms);
    }

    pub(crate) fn advance_handled_active_action(&mut self, now: impl FnOnce() -> u32) -> bool {
        Self::advance_handled_action(&mut self.active_actions, now)
    }

    pub(crate) fn advance_handled_war_soul_action(&mut self, now: impl FnOnce() -> u32) -> bool {
        Self::advance_handled_action(&mut self.active_war_soul_actions, now)
    }

    fn advance_handled_action(queue: &mut VecDeque<AiEvent>, now: impl FnOnce() -> u32) -> bool {
        let Some(event) = queue.front().filter(|event| event.handling == 1) else {
            return false;
        };
        if ai_event_deadline_reached(event, now()) {
            queue.pop_front();
        }
        true
    }

    fn finish_action(queue: &mut VecDeque<AiEvent>, action: AiShapeAction, now_ms: u32) {
        let Some(event) = queue.front_mut() else {
            return;
        };
        if event.action != action || !matches!(event.handling, 0 | 1) {
            return;
        }
        event.handling = 1;
        if ai_event_deadline_reached(event, now_ms) {
            queue.pop_front();
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
        if ai_event_deadline_reached(event, now_ms) {
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

/// Общая длительность одного шага `CBaseAI::MoveTo`: при исходном `g_ms = 80`
/// доступная доля кадра равна `0.68`; путь сохраняется в `float`, после чего
/// к нему прибавляется целый остановочный кадр владельца.
pub(crate) fn one_step_move_delay_ms(direction: i32, speed: f32, stop_frame: u32) -> u32 {
    let distance_units = if direction % 2 == 0 {
        1_000_000.0_f32
    } else {
        1_414_000.0_f32
    };
    if speed > 0.0 {
        let travel_ms = (0.68_f32 / speed) * distance_units;
        (f64::from(stop_frame) + f64::from(travel_ms)).trunc() as i32 as u32
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
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// IMPLEMENTED: object identity материализована в `CBaseAI::set_object_target`.
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
// через `CBaseAI::advance_active_stand`. Пустой OnIdle игрока не добавляет
// Stand. Слоты +0x40/+8/+0/+4 задают OnSchedule/background/passive/active;
// +0x44/+0xC — отдельный хвост WarSoul даже после AES_HUNG_UP passive.
// Player Begin выполняется в OnSchedule до background, первый AI — только
// в active после passive. WarSoul Begin ставит собственный Attack до первого
// AI; End внутри AI сохраняет событие до следующего вызова OnFighting.
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
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CBaseAI::lose_target` материализует достигнутые object-поля;
// point-координаты остаются у недостигнутого coordinate caller-а.
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
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CBaseAI::has_object_target` материализует достигнутую object-
// ветвь; coordinate-ветвь пока не имеет caller-а.
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
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `CBaseAI::object_target` владеет сохранённой identity;
// canonical region callers разрешают её в живую форму.
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
// STATUS: IMPLEMENTED
// IMPLEMENTED: discard_active_prefix и process_reached_defense_actions;
// EXE/PDB GameServer, appserver/ai/baseai.cpp:815, RVA 0x000C8700.
// Базовый обработчик сохраняет первую Attack/Move-границу и возвращает 1.
// Nullable owner заменён владением CPlayer/CMonster; отдельного вызова без owner нет.

// ============================================================================
// FUNCTION: CBaseAI::OnStiffen
// STATUS: IMPLEMENTED
// IMPLEMENTED: reached monster/player caller-ы сохраняют active-префикс,
// Attack/Move-границу, passive handling/deadline и concrete interruption.
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
// STATUS: IMPLEMENTED
// IMPLEMENTED: `when_been_hurted` и `when_been_stiffened` сохраняют оба
// отдельных замера времени и FIFO-порядок Defense -> Stiffen.
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
