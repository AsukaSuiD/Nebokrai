//! Достигнутая event-queue часть `CBaseAI` исторического GameServer.
//! Run (0x004C7D10) допускает OnIdle только при AES_IDLE всех основных фаз.
//! Defense и первый OnStiffen возвращают EXEC даже после снятия последнего
//! события: monster caller сохраняет результат passive-фазы отдельно от
//! пустоты FIFO. EXEC разрешает active-фазу, но не последующий OnIdle.
//! Общая проверка ненулевого handling сохраняет три результата фазы:
//! истёкшая последняя запись даёт IDLE, оставшийся хвост — EXEC, ожидание
//! не-Move — HUNG_UP. Ни один результат не запускает следующий handler.
//! IDLE разрешает только хвостовой OnIdle без цели, а не второй OnSchedule.
//! Сам факт вызова OnSchedule не блокирует OnIdle: если он снял цель без
//! нового события, а основные фазы вернули IDLE, хвост исполняется в том же Run.
//! Monster-caller проверяет отсутствие цели/исполнения, а не флаг попытки
//! расписания; отдельного исключения для неудачного отхода AI2 не требуется.
//! Move/Stand читают часы непосредственно при проверке deadline после записи
//! handling. Время внешнего прохода до Begin/background/passive не используется;
//! пустая очередь и чужой action не вызывают лишнего чтения часов.
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
//! Элементы очередей (`AiEvent`/`AiShapeAction`) и семья действий-порядков
//! passive-реакций (`PassiveDeathAction`/`PassiveStiffenAction`, Defense-цикл,
//! Stiffen/Died-пары и `discard_active_prefix`) перенесены в Zone
//! (`ai/events.rs`, `ai/reactions.rs`) зелёной AI-порцией; ниже сохраняются
//! их сигнатуры как делегаты, чтобы caller-ы hub не менялись.
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
//! WhenAddBackStageSkill (0x004C94B0) хранит ordered ID в vector конкретного
//! CBaseAI, без дедупликации. CMoveShape владеет самими навыками, не этим
//! списком: смена GetAI не переносит фон между primary/pet/carriage.
//! OnExecuteBackStageSkills (0x004C88E0) сначала удаляет старые UNKNOWN,
//! затем разрешает каждый ID у CMoveShape; отсутствующий/завершённый навык
//! только помечается UNKNOWN до следующего прохода. Clear (0x004C7F70)
//! сохраняет список, destructor (0x004C8890) уничтожает его вместе с AI.
//! Vec заменяет native vector; технический begin_pending сохраняет место
//! отложенного подключения player-kernel без второй очереди или копии ID.
//! OnStiffen 0x004C87D9/0x004C87E0 сначала вызывает End(4), затем IsEnded:
//! Attack остаётся в FIFO, пока concrete владелец не подтвердит завершение.
//! Для handling != 0 или отсутствующего навыка End не вызывается. Очистка
//! префикса останавливается на Move и не снимает его.
//! Указатель владельца и
//! остальные обработчики ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Публичный `MoveTo`
//! сохраняет исходный промежуточный `float` времени шага и усекает сумму со
//! stop-frame к нулю перед постановкой действия в очередь.

use std::collections::VecDeque;

pub(crate) use nebokrai_zone::ai::{
    ai_event_deadline_reached, AiEvent, AiShapeAction, PassiveDeathAction, PassiveStiffenAction,
};

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, MoveCheckCellRegistry, ShapeAreaCoordinates, ShapeIdentity,
};

const SLIP_ORDER: [[usize; 8]; 8] = [
    [0, 7, 1, 6, 2, 5, 3, 4],
    [1, 0, 2, 7, 3, 6, 4, 5],
    [2, 1, 3, 0, 4, 7, 5, 6],
    [3, 2, 4, 1, 5, 0, 6, 7],
    [4, 3, 5, 2, 6, 1, 7, 0],
    [5, 4, 6, 3, 7, 2, 0, 1],
    [6, 5, 7, 4, 0, 3, 1, 2],
    [7, 6, 0, 5, 1, 4, 2, 3],
];

/// Один одноклеточный `Slip` из `CBaseAI::MoveTo` (0x004C7FF0).
/// Точная пара `gameserver.exe`/`GameServer.pdb` задаёт порядок `_slip_order`,
/// проверку всех `s_listMoveCheckCell[figure][direction]` и выбор первого
/// свободного направления. `CRegion::get_block` уже возвращает исходные
/// младшие три бита клетки (`& 7`).
pub(crate) fn find_slip_step_in_direction(
    move_check_cells: &MoveCheckCellRegistry,
    region: &CServerRegion,
    origin: ShapeAreaCoordinates,
    desired_direction: i32,
    figure_index: usize,
) -> Option<(i32, ShapeAreaCoordinates)> {
    let slip_order = SLIP_ORDER.get(usize::try_from(desired_direction).ok()?)?;
    slip_order.iter().copied().find_map(|direction| {
        let destination = CShape::get_direction_position(direction as i32, origin).ok()?;
        let cells = move_check_cells.get(figure_index, direction)?;
        cells
            .iter()
            .all(|cell| {
                region
                    .region
                    .get_block(
                        origin.x.wrapping_add(cell.x),
                        origin.y.wrapping_add(cell.y),
                    )
                    .is_ok_and(|block| block == 0)
            })
            .then_some((direction as i32, destination))
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiPhaseState {
    Idle,
    Executing,
    HungUp,
}

impl AiPhaseState {
    pub(crate) const fn is_idle(self) -> bool {
        matches!(self, Self::Idle)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CBaseAI {
    active_actions: VecDeque<AiEvent>,
    passive_actions: VecDeque<AiEvent>,
    active_war_soul_actions: VecDeque<AiEvent>,
    back_stage_skill_ids: Vec<BackStageSkill>,
    is_dormant: bool,
    dormancy_time_ms: u32,
    dormancy_interval_ms: u32,
    target_id: i32,
    target_type: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BackStageSkill {
    skill_id: u32,
    begin_pending: bool,
}

/// Сварка Zone-порядка passive-реакций к двум FIFO этого hub-владельца.
impl nebokrai_zone::ai::PassiveReactionQueues for CBaseAI {
    fn passive_reaction_queue(&self) -> &VecDeque<AiEvent> {
        &self.passive_actions
    }

    fn active_reaction_queue(&self) -> &VecDeque<AiEvent> {
        &self.active_actions
    }

    fn passive_reaction_queue_mut(&mut self) -> &mut VecDeque<AiEvent> {
        &mut self.passive_actions
    }

    fn active_reaction_queue_mut(&mut self) -> &mut VecDeque<AiEvent> {
        &mut self.active_actions
    }
}

impl CBaseAI {
    fn add_back_stage_skill(&mut self, skill_id: u32, begin_pending: bool) {
        if skill_id != 0x7fff_ffff {
            self.back_stage_skill_ids.push(BackStageSkill { skill_id, begin_pending });
        }
    }

    /// Когда Begin уже исполнен, подготовленный навык добавляется в конец
    /// того же списка и не требует повторного Begin.
    pub(crate) fn add_started_back_stage_skill(&mut self, skill_id: u32) {
        self.add_back_stage_skill(skill_id, false);
    }

    pub(crate) fn add_pending_back_stage_skill(&mut self, skill_id: u32) {
        self.add_back_stage_skill(skill_id, true);
    }

    pub(crate) fn prepare_back_stage_skill_pass(&mut self) {
        self.back_stage_skill_ids.retain(|entry| entry.skill_id != 0x7fff_ffff);
    }

    pub(crate) fn back_stage_skill_id(&self, index: usize) -> Option<u32> {
        self.back_stage_skill_ids.get(index).map(|entry| entry.skill_id)
    }

    pub(crate) fn mark_ended_back_stage_skill(&mut self, index: usize, expected: u32) {
        if let Some(entry) = self.back_stage_skill_ids.get_mut(index)
            && entry.skill_id == expected
        {
            entry.skill_id = 0x7fff_ffff;
        }
    }

    pub(crate) fn begin_pending_back_stage_skill_ids(&mut self) -> Vec<u32> {
        self.back_stage_skill_ids.iter_mut().filter_map(|entry| {
            std::mem::take(&mut entry.begin_pending).then_some(entry.skill_id)
        }).collect()
    }

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
            ex_id: nebokrai_shared::values::CGuid::GUID_INVALID,
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
        Self::advance_handled_action(&mut self.passive_actions, now)
            .map(|state| state == AiPhaseState::HungUp)
    }

    /// Выполняет материализованный `Defense`-участок `ProcessPassiveAction`.
    /// Последовательные события снимаются FIFO; `OnBeenHurted` удаляет только
    /// префикс `active_actions` до первого `Attack` либо `Move`.
    pub(crate) fn process_reached_defense_actions(
        &mut self,
        after_base_handler: impl FnMut(&mut Self),
    ) -> usize {
        nebokrai_zone::ai::process_reached_defense_actions(self, after_base_handler)
    }

    /// Материализует exact `ASA_STIFFEN -> OnStiffen` и deadline-часть
    /// `ProcessPassiveAction`. До первой атаки/движения active-префикс
    /// отбрасывается; движение сохраняется, атака остаётся в FIFO до ответа
    /// concrete owner-а на `CSkill::End(4)` и проверку `IsEnded`.
    pub(crate) fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        nebokrai_zone::ai::begin_reached_stiffen_action(self)
    }

    /// ProcessPassiveAction фиксирует результат OnStiffen и срок после callback.
    pub(crate) fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        nebokrai_zone::ai::finish_reached_stiffen_action(self, begun, now)
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
        nebokrai_zone::ai::discard_active_prefix(self);
    }

    /// Выполняет точный `ASA_DIED -> OnBeenKilled` после того, как caller
    /// отдельным проходом снял предшествующий Defense-префикс. Исходник
    /// отбрасывает все active-события до первого Move, сохраняет только этот
    /// Move со всеми его часами/handling и повторяет обработчик, пока движение
    /// не завершится. Снятие Died после virtual `OnLoseTarget` выполняет
    /// парный `finish_reached_death_action` после обработчика смерти owner-а.
    pub(crate) fn begin_reached_death_action(&mut self) -> bool {
        nebokrai_zone::ai::begin_reached_death_action(self)
    }

    /// Проверяет готовность после OnLoseTarget, не снимая Died до owner OnDied.
    pub(crate) fn reached_death_action_state(&self) -> PassiveDeathAction {
        nebokrai_zone::ai::reached_death_action_state(self)
    }

    pub(crate) fn finish_reached_death_action(&mut self, now_ms: u32) {
        nebokrai_zone::ai::finish_reached_death_action(self, now_ms);
    }

    /// Выполняет достигнутую `Stand`-ветвь `ProcessActiveAction` и сообщает,
    /// разрешено ли в этом же такте переходить к `OnSchedule`.
    ///
    /// Первый вызов обработчика всегда считается исполнением, даже если
    /// нулевая либо уже истёкшая задержка позволяет сразу снять событие.
    /// Повторный вызов после истечения снимает событие и разрешает расписание
    /// только при пустом остатке FIFO. Другие активные действия пока не
    /// интерпретируются и поэтому продолжают блокировать расписание.
    pub(crate) fn advance_active_stand(&mut self, now: impl FnOnce() -> u32) -> bool {
        let Some(event) = self.active_actions.front_mut() else {
            return true;
        };
        if event.action != AiShapeAction::Stand {
            return false;
        }
        if event.handling == 1 {
            if ai_event_deadline_reached(event, now()) {
                self.active_actions.pop_front();
                return self.active_actions.is_empty();
            }
            return false;
        }
        if event.handling != 0 {
            return false;
        }
        event.handling = 1;
        if ai_event_deadline_reached(event, now()) {
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
    pub(crate) fn advance_active_move(&mut self, now: impl FnOnce() -> u32) -> bool {
        let Some(event) = self.active_actions.front_mut() else {
            return false;
        };
        if event.action != AiShapeAction::Move {
            return false;
        }
        if event.handling == 0 {
            event.handling = 1;
        }
        if event.handling == 1 && ai_event_deadline_reached(event, now()) {
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

    pub(crate) fn advance_handled_active_action(&mut self, now: impl FnOnce() -> u32) -> Option<AiPhaseState> {
        Self::advance_handled_action(&mut self.active_actions, now)
    }

    pub(crate) fn advance_handled_war_soul_action(&mut self, now: impl FnOnce() -> u32) -> Option<AiPhaseState> {
        Self::advance_handled_action(&mut self.active_war_soul_actions, now)
    }

    fn advance_handled_action(queue: &mut VecDeque<AiEvent>, now: impl FnOnce() -> u32) -> Option<AiPhaseState> {
        let event = queue.front().filter(|event| event.handling != 0)?;
        if event.handling == 1 && ai_event_deadline_reached(event, now()) {
            queue.pop_front();
            return Some(if queue.is_empty() { AiPhaseState::Idle } else { AiPhaseState::Executing });
        }
        Some(if event.action == AiShapeAction::Move { AiPhaseState::Executing } else { AiPhaseState::HungUp })
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
