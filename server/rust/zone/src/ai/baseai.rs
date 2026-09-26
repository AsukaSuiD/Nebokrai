//! Базовый `CBaseAI` исторического GameServer без hub-типов: три FIFO-очереди
//! объявленных действий (`active`/`passive`/`war-soul`), object-цель,
//! dormancy, фоновый список back-stage навыков, active-фаза
//! `ProcessActiveAction` и пространственная механика `MoveTo`
//! (`Slip` + задержка одного шага). Порядковые реакции
//! Defense/Stiffen/Died живут в соседнем `ai/reactions.rs` (кластер A1)
//! и переставляют эти очереди через узкую сварку `PassiveReactionQueues`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные
//! `off pub + 0x1000`). Исходный владелец PDB:
//! `server/gameserver/appserver/ai/baseai.cpp`. Тела перенесены буквально из
//! старого hub-файла `appserver/ai/baseai.rs` волной Z-AI-player; до переноса
//! свидетельства были зафиксированы в его шапке, дополнительные точечные
//! сверки этой волны отмечены ниже отдельно.
//!
//! Машинная база (VERIFIED по этой паре):
//!
//! - Постановка и очереди: `AddAIEvent` (RVA `0x0C8F90`) кладёт
//!   `{action, beginning, delay, handling=0} = {+0,+4,+8,+C}` с замером часов
//!   в момент вызова; `STIFFEN`/`DIED`/`OPEN`/`DEFENSE` всегда идут в
//!   `passive_actions`, остальные — в `active_war_soul_actions` при любом
//!   ненулевом флаге и иначе в `active_actions`. PDB подтверждает numeric
//!   `AI_SHAPE_ACTION 0..8`, `ASA_FORCE_DWROD = 0xFF` и три очереди
//!   (`ai/events.rs`). `GetCurrentActiveAction` (RVA `0x0C81B0`) читает
//!   только action первого элемента без проверки handling; пролог начинается
//!   с `or eax,-1` + `cmp [ecx+0x14],0`, поэтому пустая очередь — это
//!   native `-1`, в Rust представлена `None`.
//! - `Run` (RVA `0x0C7D10`): guard owner `[+0x68]` и hibernate `[+0x6c]` →
//!   OnSchedule → background → passive → active → хвостовой OnIdle только
//!   при AES_IDLE всех основных фаз и отсутствии цели, затем WarSoul-хвост.
//!   Сам факт вызова OnSchedule не блокирует OnIdle.
//! - Общий обработанный-элемент любой из трёх очередей (`ProcessActiveAction`
//!   RVA `0x0C81D0`, `ProcessPassiveAction` RVA `0x0C84F0`, WarSoul-вариант
//!   RVA `0x0C8390`): различает ноль, единицу и прочие знаковые `handling`;
//!   снятие по deadline только при `handling == 1`, ожидание любого action
//!   кроме `Move` даёт HUNG_UP и блокирует следующую фазу; пробы deadline
//!   читают часы лениво, после записи handling. Снятие головы завершает
//!   текущий проход, не исполняя следующий элемент.
//! - Достигнутый `Stand` из `ProcessActiveAction`: первый вызов обработчика
//!   всегда считается исполнением и удерживает расписание до исходного срока;
//!   повторный после истечения снимает событие. `OnIdle` ставит следующий
//!   `Stand` на 1000 мс только при пустом результате основного прохода и
//!   отсутствии цели (базовый обработчик; слот +0x48 `CPlayerAI` — пустой
//!   RET `0x00485540`, с игроком это не срабатывает).
//! - Object-части `SetTarget`/`GetTarget`/`HasTarget`/`OnLoseTarget`:
//!   identity `{type, id}`, нулевые type/id не образуют lookup, положительные
//!   обязательны для HasTarget. `Clear` (RVA `0x0C7F70`) при guard-refresh
//!   очищает обычные active/passive очереди, object-цель и флаг сна;
//!   war-soul FIFO и сохранённые времена сна не затрагиваются. Спящий
//!   владелец (`Hibernate`/`WakeUp`) запоминает оборачивающийся счётчик
//!   времени; интервал сна вычисляется один раз при пробуждении.
//! - Back-stage список: `WhenAddBackStageSkill` (RVA `0x0C94B0`) требует
//!   `owner != null` (`cmp [rcx/rcx+0x68],0`) и `ID != SKILL_UNKNOW`
//!   (`cmp [esp+4], 0x7FFFFFFF` — наблюдается в прологе) и добавляет ID в
//!   `m_vBackStageSkills` конкретного AI без дедупликации.
//!   `OnExecuteBackStageSkills` (RVA `0x0C88E0`) сначала удаляет старые
//!   UNKNOWN, затем разрешает каждый ID у `CMoveShape`;
//!   отсутствующий/завершённый навык только помечается UNKNOWN до
//!   следующего прохода. `Clear` список сохраняет, destructor (RVA
//!   `0x0C8890`, запись vtable `0x651D24` в прологе) уничтожает вместе с AI.
//! - Пространство `MoveTo` (RVA `0x0C9020`, пролог `83 ec 18`; сверено этой
//!   волной): один `Slip` (RVA `0x0C7FF0`) для ходьбы, два для ненулевого
//!   run с исходным желаемым направлением; отказ Slip не меняет очередь.
//!   `Slip` пробует статическую таблицу `_slip_order` — dword-набор 8×8
//!   (`.data` raw `0x29EDE0`, VA `0x69EDE0`), побайтово совпадающую с
//!   `SLIP_ORDER` ниже (сверено этой волной), и построчно
//!   `s_listMoveCheckCell[figure][direction]` (база VA `0xEF670C`,
//!   индексация `[n*8+direction]`, `.data`-BSS); `CRegion::get_block`
//!   возвращает исходные младшие три бита клетки (`& 7`).
//! - Задержка шага (сверено этой волной по телу `0x0C9020`): MSVC считает в
//!   x87 `fsubr 1000.0` (raw `0x251DB8`) от `fild(4*g_ms)`, затем
//!   `fmul 0.001` (raw `0x24E9FC`, VA `0x64E9FC`), `fdivr` по скорости
//!   (vt `+0x6C`), `fmul` по `1_000_000.0` (чётное направление, raw
//!   `0x251DB4`) либо `1_414_000.0` (нечётное, raw `0x251DB0`),
//!   `fadd fild` stop-frame (vt `+0xB8`) и `fistp` под forcibly-truncating
//!   control word (`fstcw`/`or ah,0x0C`); промежуточные значения округляются
//!   до `float` через `fstp dword`. Глобальный `g_ms` (raw `0x29E1F4`,
//!   VA `0x69E1F4`) равен 80, поэтому коэффициент кадра
//!   `(1000 - 4*80)*0.001 = 0.68`. Те же три константы и фактор грузит
//!   трёхаргументный `CPlayerAI::MoveTo` (ссылки raw `0x109031`/`0x109092`
//!   и `0x109046`/`0x1090A7`).
//!
//! Швы переноса (не расхождения):
//!
//! - Единственная hub-связь старого файла — обёртка `CServerRegion` вокруг
//!   `CRegion` в `Slip`; Zone-версия принимает `&CRegion` напрямую
//!   (`zone/src/regions/region.rs`), а переупаковка `&CServerRegion →
//!   &region.region` остаётся в делегате старого пакета. Региональный
//!   реестр, around-доставка шагов, игровые часы owner-а и оркестрация монстра
//!   остаются hub-владением через существующие узкие фасады
//!   `ai/monsterai.rs` (`MonsterDispatcherGame::find_slip_step_in_direction`,
//!   `one_step_move_delay_ms`) — здесь не дублируются.
//! - `VecDeque` заменяет внутренности `std::queue<std::deque<...>>`, сохраняя
//!   FIFO и `push_back`; `Vec` заменяет vector back-stage. Время среды
//!   передаётся точным `now_ms` в момент вызова (`u32` сохраняет исходное
//!   переполнение); deadline сравнивается после wrapping-сложения
//!   `begin + delay` (`ai_event_deadline_reached`, см. `ai/events.rs`).
//!
//! Честные UNKNOWN этой порции (не достраиваются догадкой): указатель
//! владельца (`m_pOwner` `[+0x68]`) и обработчики вне достигнутых участков;
//! точная семантика virtual-вызовов (`OnLoseTarget` из `OnStiffen`,
//! `End(4)` ответ concrete owner-а) — политики разрешения остаются у
//! hub-владельцев (`appserver/monster.rs`, `appserver/player.rs`).

use std::collections::VecDeque;

use nebokrai_shared::values::CGuid;

use crate::regions::ShapeIdentity;
use crate::regions::region::CRegion;
use crate::regions::shape::{CShape, MoveCheckCellRegistry, ShapeAreaCoordinates};

use super::events::{ai_event_deadline_reached, AiEvent, AiShapeAction};
use super::reactions::{
    self, PassiveDeathAction, PassiveReactionQueues, PassiveStiffenAction,
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

/// Один одноклеточный `Slip` из `CBaseAI::Slip` (RVA `0x0C7FF0`).
/// Точная пара `gameserver.exe`/`GameServer.pdb` задаёт порядок `_slip_order`
/// (dword-таблица 8×8, raw `0x29EDE0`), проверку всех
/// `s_listMoveCheckCell[figure][direction]` и выбор первого свободного
/// направления. `CRegion::get_block` уже возвращает исходные младшие три бита
/// клетки (`& 7`). Hub-обёртка `CServerRegion` переводится в `&CRegion`
/// делегатом старого пакета и сюда не поднимается.
pub fn find_slip_step_in_direction(
    move_check_cells: &MoveCheckCellRegistry,
    region: &CRegion,
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
pub enum AiPhaseState {
    Idle,
    Executing,
    HungUp,
}

impl AiPhaseState {
    pub const fn is_idle(self) -> bool {
        matches!(self, Self::Idle)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CBaseAI {
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

/// Сварка Zone-порядка passive-реакций к двум FIFO этого владельца.
impl PassiveReactionQueues for CBaseAI {
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
    pub fn add_started_back_stage_skill(&mut self, skill_id: u32) {
        self.add_back_stage_skill(skill_id, false);
    }

    pub fn add_pending_back_stage_skill(&mut self, skill_id: u32) {
        self.add_back_stage_skill(skill_id, true);
    }

    pub fn prepare_back_stage_skill_pass(&mut self) {
        self.back_stage_skill_ids.retain(|entry| entry.skill_id != 0x7fff_ffff);
    }

    pub fn back_stage_skill_id(&self, index: usize) -> Option<u32> {
        self.back_stage_skill_ids.get(index).map(|entry| entry.skill_id)
    }

    pub fn mark_ended_back_stage_skill(&mut self, index: usize, expected: u32) {
        if let Some(entry) = self.back_stage_skill_ids.get_mut(index)
            && entry.skill_id == expected
        {
            entry.skill_id = 0x7fff_ffff;
        }
    }

    pub fn begin_pending_back_stage_skill_ids(&mut self) -> Vec<u32> {
        self.back_stage_skill_ids.iter_mut().filter_map(|entry| {
            std::mem::take(&mut entry.begin_pending).then_some(entry.skill_id)
        }).collect()
    }

    /// Точный достигнутый `SetTarget(long, long)` object identity.
    pub const fn set_object_target(&mut self, target: ShapeIdentity) {
        self.target_type = target.object_type;
        self.target_id = target.id;
    }

    /// Материализованная identity-часть `GetTarget`. Разрешение объекта
    /// остаётся у region owner-а, а нулевые legacy-поля не образуют lookup.
    pub const fn object_target(&self) -> Option<ShapeIdentity> {
        if self.target_type == 0 || self.target_id == 0 {
            return None;
        }
        Some(ShapeIdentity {
            object_type: self.target_type,
            id: self.target_id,
            ex_id: CGuid::GUID_INVALID,
        })
    }

    /// Достигнутая object-ветвь `HasTarget` требует строго положительные
    /// type/ID. Point-цель пока остаётся у недостигнутого coordinate caller-а.
    pub const fn has_object_target(&self) -> bool {
        self.target_id > 0 && self.target_type > 0
    }

    /// Достигнутая object-часть `OnLoseTarget`.
    pub const fn lose_target(&mut self) {
        self.target_id = 0;
        self.target_type = 0;
    }

    pub fn add_ai_event(
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
    pub fn when_been_hurted(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Defense, 0, 0, now_ms);
    }

    pub fn when_been_stiffened(&mut self, delay_ms: u32, now_ms: u32) {
        if delay_ms != 0 {
            self.add_ai_event(AiShapeAction::Stiffen, delay_ms, 0, now_ms);
        }
    }

    /// Точный достигнутый префикс `WhenBeenKilled`: событие смерти сохраняет
    /// место относительно уже поставленных пассивных действий.
    pub fn when_been_killed(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Died, 0, 0, now_ms);
    }

    /// Точный наблюдаемый участок `CBaseAI::Clear` (RVA `0x0C7F70`),
    /// вызываемый при обновлении city/country guard: очищает обычные
    /// active/passive FIFO, object-цель и снимает сон. Исходный owner не
    /// очищает отдельную war-soul очередь и сохранённые времена сна, поэтому
    /// Rust сохраняет различие.
    pub fn clear_guard_refresh_state(&mut self) {
        self.active_actions.clear();
        self.passive_actions.clear();
        self.target_id = 0;
        self.target_type = 0;
        self.is_dormant = false;
    }

    /// None — требуется dispatch; Some — проход занят, значение блокирует active.
    pub fn advance_handled_passive_action(&mut self, now: impl FnOnce() -> u32) -> Option<bool> {
        Self::advance_handled_action(&mut self.passive_actions, now)
            .map(|state| state == AiPhaseState::HungUp)
    }

    /// Выполняет материализованный `Defense`-участок `ProcessPassiveAction`.
    /// Последовательные события снимаются FIFO; `OnBeenHurted` удаляет только
    /// префикс `active_actions` до первого `Attack` либо `Move`.
    pub fn process_reached_defense_actions(
        &mut self,
        after_base_handler: impl FnMut(&mut Self),
    ) -> usize {
        reactions::process_reached_defense_actions(self, after_base_handler)
    }

    /// Материализует exact `ASA_STIFFEN -> OnStiffen` и deadline-часть
    /// `ProcessPassiveAction`. До первой атаки/движения active-префикс
    /// отбрасывается; движение сохраняется, атака остаётся в FIFO до ответа
    /// concrete owner-а на `CSkill::End(4)` и проверку `IsEnded`.
    pub fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        reactions::begin_reached_stiffen_action(self)
    }

    /// ProcessPassiveAction фиксирует результат OnStiffen и срок после callback.
    pub fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        reactions::finish_reached_stiffen_action(self, begun, now)
    }

    pub fn current_active_action(&self) -> Option<AiShapeAction> {
        self.active_actions.front().map(|event| event.action)
    }

    /// OnStiffen (RVA `0x0C87C0`) вызывает End(4) только для Attack с handling=0.
    pub fn stiffen_attack_pending(&self) -> bool {
        self.current_active_action() == Some(AiShapeAction::Attack)
    }

    pub fn stiffen_attack_needs_end(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Attack && event.handling == 0
        })
    }

    /// Вызывается после подтверждённого IsEnded либо при отсутствии навыка.
    /// Нельзя снимать Attack до End: производный навык может остаться активным.
    pub fn finish_stiffen_attack(&mut self, release_target: bool) {
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
    pub fn discard_active_prefix(&mut self) {
        reactions::discard_active_prefix(self);
    }

    /// Выполняет точный `ASA_DIED -> OnBeenKilled` после того, как caller
    /// отдельным проходом снял предшествующий Defense-префикс. Исходник
    /// отбрасывает все active-события до первого Move, сохраняет только этот
    /// Move со всеми его часами/handling и повторяет обработчик, пока движение
    /// не завершится. Снятие Died после virtual `OnLoseTarget` выполняет
    /// парный `finish_reached_death_action` после обработчика смерти owner-а.
    pub fn begin_reached_death_action(&mut self) -> bool {
        reactions::begin_reached_death_action(self)
    }

    /// Проверяет готовность после OnLoseTarget, не снимая Died до owner OnDied.
    pub fn reached_death_action_state(&self) -> PassiveDeathAction {
        reactions::reached_death_action_state(self)
    }

    pub fn finish_reached_death_action(&mut self, now_ms: u32) {
        reactions::finish_reached_death_action(self, now_ms);
    }

    /// Выполняет достигнутую `Stand`-ветвь `ProcessActiveAction` и сообщает,
    /// разрешено ли в этом же такте переходить к `OnSchedule`.
    ///
    /// Первый вызов обработчика всегда считается исполнением, даже если
    /// нулевая либо уже истёкшая задержка позволяет сразу снять событие.
    /// Повторный вызов после истечения снимает событие и разрешает расписание
    /// только при пустом остатке FIFO. Другие активные действия пока не
    /// интерпретируются и поэтому продолжают блокировать расписание.
    pub fn advance_active_stand(&mut self, now: impl FnOnce() -> u32) -> bool {
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
    pub fn active_change_skill_pending(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::ChangeSkill && event.handling == 0
        })
    }

    pub fn active_search_enemy_pending(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::SearchEnemy && event.handling == 0
        })
    }

    pub fn active_move_unhandled(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Move && event.handling == 0
        })
    }

    pub fn active_stand_pending(&self) -> bool {
        self.active_actions
            .front()
            .is_some_and(|event| event.action == AiShapeAction::Stand)
    }

    pub fn active_stand_unhandled(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Stand && event.handling == 0
        })
    }

    /// Сообщает, что следующий FIFO-элемент должен продолжить текущий
    /// `CSkill::AI` через подтверждённый `OnFighting`.
    pub fn active_attack_pending(&self) -> bool {
        self.active_actions.front().is_some_and(|event| {
            event.action == AiShapeAction::Attack && event.handling == 0
        })
    }

    /// Ставит завершённый пространственный шаг в ту же FIFO-очередь, через
    /// которую исходный `MoveTo` удерживал дальнейшее расписание.
    pub fn begin_active_move(&mut self, delay_ms: u32, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Move, delay_ms, 0, now_ms);
    }

    pub fn begin_active_stand(&mut self, delay_ms: u32, now_ms: u32) {
        self.add_ai_event(AiShapeAction::Stand, delay_ms, 0, now_ms);
    }

    /// Материализует точный `CBaseAI::OnIdle`: пустой основной проход без
    /// target-а ставит `ASA_STAND` на 1000 мс. Наличие Rust-owner-а заменяет
    /// исходную проверку `m_pOwner != nullptr`; dormant и непустые очереди
    /// проверяются здесь, чтобы caller не мог обойти начало `CBaseAI::Run`.
    pub fn begin_idle_stand(&mut self, now_ms: u32) -> bool {
        if self.is_dormant || !self.primary_queues_idle() {
            return false;
        }
        self.begin_active_stand(1_000, now_ms);
        true
    }

    pub fn begin_active_search_enemy(&mut self, now_ms: u32) {
        self.add_ai_event(AiShapeAction::SearchEnemy, 0, 0, now_ms);
    }

    pub fn cancel_active_move(&mut self) {
        self.active_actions
            .retain(|event| event.action != AiShapeAction::Move);
    }

    /// Выполняет общий `OnMoving`, который возвращает единицу, и сохраняет
    /// исходную границу задержки уже совершённого шага. Производные реакции
    /// лучника, охранника и питомца остаются отдельными проходами владельцев.
    pub fn advance_active_move(&mut self, now: impl FnOnce() -> u32) -> bool {
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
    pub fn finish_active_attack(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.active_actions, AiShapeAction::Attack, now_ms);
    }

    /// Завершает один вызов `OnChangeSkill`: исходный обработчик возвращает
    /// единицу, поэтому событие с нулевой задержкой снимается в том же проходе,
    /// но `Run` всё равно не вызывает `OnSchedule` до следующего такта.
    pub fn finish_active_change_skill(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.active_actions, AiShapeAction::ChangeSkill, now_ms);
    }

    pub fn finish_war_soul_attack(&mut self, now_ms: u32) {
        Self::finish_action(&mut self.active_war_soul_actions, AiShapeAction::Attack, now_ms);
    }

    pub fn advance_handled_active_action(&mut self, now: impl FnOnce() -> u32) -> Option<AiPhaseState> {
        Self::advance_handled_action(&mut self.active_actions, now)
    }

    pub fn advance_handled_war_soul_action(&mut self, now: impl FnOnce() -> u32) -> Option<AiPhaseState> {
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

    pub fn finish_active_search_enemy(&mut self, now_ms: u32) {
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

    pub fn active_actions(&self) -> &VecDeque<AiEvent> {
        &self.active_actions
    }

    pub fn passive_actions(&self) -> &VecDeque<AiEvent> {
        &self.passive_actions
    }

    pub fn active_war_soul_actions(&self) -> &VecDeque<AiEvent> {
        &self.active_war_soul_actions
    }

    /// Соответствует проверке размеров первых двух исходных очередей в
    /// `OnSchedule`; очередь боевого духа в это условие не входит.
    pub fn primary_queues_idle(&self) -> bool {
        self.active_actions.is_empty() && self.passive_actions.is_empty()
    }

    pub fn hibernate(&mut self, now_ms: u32) {
        self.is_dormant = true;
        self.dormancy_time_ms = now_ms;
        self.dormancy_interval_ms = 0;
    }

    pub fn wake_up(&mut self, now_ms: u32) -> u32 {
        self.dormancy_interval_ms = now_ms.wrapping_sub(self.dormancy_time_ms);
        self.dormancy_time_ms = 0;
        self.is_dormant = false;
        self.dormancy_interval_ms
    }

    pub const fn is_hibernated(&self) -> bool {
        self.is_dormant
    }
}

/// Общая длительность одного шага `CBaseAI::MoveTo` (RVA `0x0C9020`):
/// доступная доля кадра — `f32((1000 - 4*g_ms)*0.001)` = `0.68` при
/// `g_ms = 80`; путь сохраняется в `float`, после чего к нему прибавляется
/// целый остановочный кадр владельца, а сумма усекается (MSVC x87
/// truncation idiom, см. шапку).
pub fn one_step_move_delay_ms(direction: i32, speed: f32, stop_frame: u32) -> u32 {
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
