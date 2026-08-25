//! Достигнутая movement/spatial-transition часть `CMoveShape` GameServer.
//!
//! `GetDestDir` RVA `0x000CCF60`, `IsDied` `0x000CCF20` и virtual
//! `SetPosXY` `0x000CD050`, а также `ForceMove/OnMove/OnSetPosition`
//! `0x000CD1A0/0x000CD490/0x000CD5C0` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `server/gameserver/appserver/moveshape.h/.cpp`.
//!
//! PDB подтверждает наследование `CShape`, `m_pFather +0x40`, area-link
//! `+0x60`, next-area X/Y `+0x68/+0x6C` и change-state `+0x80`. Exact EXE
//! фиксирует порядок: при живом region-link сначала virtual `SetBlock` снимает
//! старую footprint, затем HP virtual `+0xD0` (либо NPC type `500`) разрешает
//! новую footprint, после чего бит-в-бит пишутся X/Y и только затем через
//! x87 truncation и `AREA_WIDTH/AREA_HEIGHT` записывается `CS_CHANGEAREA`.
//! Invalid float и area span останавливаются typed-границей в достигнутой
//! точке, не откатывая уже доказанные предшествующие эффекты.
//!
//! Movement commands сохраняют разные wire layouts `0xBF603/604/605`, send до
//! virtual `SetTileXY`, direction до `0xBF605` и `ASA_STAND` после force
//! position. Подтверждённая странность `ForceMove`: верхняя граница Y пишет
//! `width - 1`, хотя сравнивает с height; это наблюдаемое поведение сохранено.
//! Nullable father/AI выражены `Option`; deep around-send теперь подключён
//! concrete owner-цепочкой `CMessage -> CServerRegion/CArea -> CGame/player ->
//! CMyNetServer`, а runtime context сохраняет только ещё не материализованные
//! concrete derived AI lookup и realtime clock.
//!
//! Combat, pets, общий AI tick и остальные поля/методы ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Для battle-fairy combine/reset материализованы `AddSkill`
//! и оба name/ID overload-а `DelSkill/AddSkill`: factory подтверждает
//! level/type/name, а `BTreeMap`
//! хранит identity вместо четырёх raw pointer-vector-ов. При удалении current
//! skill ID очищается до category lookup, как в EXE. Concrete skill execution,
//! его virtual параметры остаются у отдельных skill owners. Derived HP/figure
//! передаются как факты, а не копируются из ещё сырых
//! player/monster owners.
//! `GetCurrentSkill` получил только безопасный ID-view для caller-а
//! `SummonBF`; virtual lifecycle skill остаётся у будущего skill owner-а.
//! `SetMoveable` RVA `0x000CCEE0` хранит exact nesting counter и derived bool;
//! goods-session `0x8FC25` снимает один запрет строго между session End и plug Exit.
//! Reached appellation scripts материализуют `Add/Del/GetUndeadState` RVA
//! `0x000D1780/0x000CDE80/0x000CDEF0`: свойства берутся из skill `(56, ID)`,
//! активные state заменяются по usage type либо ID, а полный usage snapshot
//! остаётся у state для общего property owner-а.

use std::collections::BTreeMap;

use super::ai::baseai::{AiShapeAction, CBaseAI};
use super::chbystate::{ChangeBodyMutation, ChangeBodyState};
use super::exstate::{ExtendedState, ExtendedStateKind, ExtendedStateMutation};
use super::region::{CRegion, RegionCellAccessBlock};
use super::serverregion::{CServerRegion, RegionMembershipBlock};
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapePositionDispatch, ShapeResolver,
};
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::nets::netserver::message::{CMessage, GameServerAroundRuntime};
use crate::public::tools::get_line_direction;

const NPC_TYPE: i32 = 500;
const SET_POSITION_MESSAGE: i32 = 0xBF603;
const FORCE_MOVE_MESSAGE: i32 = 0xBF604;
const MOVE_MESSAGE: i32 = 0xBF605;
const SKILL_TYPE_ATTACK: u32 = 0;
const SKILL_TYPE_DEFENSE: u32 = 1;
const SKILL_TYPE_STATE: u32 = 2;
const SKILL_TYPE_SUMMON: u32 = 3;
const SKILL_NOT_DISAPPEAR_AFTER_DEAD: u32 = 56;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;

/// Достигнутая common-проекция `CSkill`: identity, level, category и name.
/// Исполнение concrete attack/defense/state/summon owners остаётся у самих
/// skill owners; здесь хранится точный результат `CMoveShape::AddSkill`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapeSkill {
    id: u32,
    level: i32,
    skill_type: u32,
    name: Vec<u8>,
}

/// Достигнутая identity/lifecycle-проекция `CNotDisappearAfterDead`.
/// Полная usage-map сохраняется вместе со state, чтобы последующий общий
/// property owner мог применить те же модификаторы без повторного lookup-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UndeadState {
    state_id: u32,
    state_type: u16,
    keep_time_ms: u32,
    started_ms: u32,
    properties: BTreeMap<u32, u32>,
}

impl UndeadState {
    pub(crate) const fn state_id(&self) -> u32 {
        self.state_id
    }

    pub(crate) const fn keep_time_ms(&self) -> u32 {
        self.keep_time_ms
    }

    pub(crate) const fn started_ms(&self) -> u32 {
        self.started_ms
    }

    pub(crate) const fn properties(&self) -> &BTreeMap<u32, u32> {
        &self.properties
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UndeadStateMutation {
    pub(crate) removed: Vec<UndeadState>,
    pub(crate) added: Option<UndeadState>,
    pub(crate) legacy_return: u32,
    pub(crate) state_list_changed: bool,
}

impl MoveShapeSkill {
    pub(crate) const fn id(&self) -> u32 {
        self.id
    }

    pub(crate) const fn level(&self) -> i32 {
        self.level
    }

    pub(crate) const fn skill_type(&self) -> u32 {
        self.skill_type
    }

    pub(crate) fn name(&self) -> &[u8] {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MoveShapePositionBlock {
    Coordinate(ShapeCoordinateBlock),
    ShapeBlock(ShapeBlockError),
    InvalidAreaSpan { width: i32, height: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapePositionFacts {
    pub(crate) current_hit_points: u32,
    pub(crate) figure: ShapeFigure,
    pub(crate) current_area: Option<ShapeAreaCoordinates>,
    pub(crate) area_width: i32,
    pub(crate) area_height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MoveShapeCommandBlock {
    Coordinate(ShapeCoordinateBlock),
    RegionCell(RegionCellAccessBlock),
    Position(RegionMembershipBlock),
    DetachedPosition(MoveShapePositionBlock),
}

pub(crate) trait MoveShapeCommandContext {
    /// Возвращает concrete AI и соответствующий realtime timestamp одним
    /// snapshot либо `None`, если AI у shape отсутствует.
    fn ai_with_realtime_ms(&mut self, identity: ShapeIdentity) -> Option<(&mut CBaseAI, u32)>;
}

pub(crate) trait MoveShapeResolver: ShapeResolver {
    /// `Some` означает успешный RTTI `CShape -> CMoveShape`; значение хранит
    /// exact `!IsDied`, полученный у concrete derived owner-а.
    fn move_shape_is_alive(&self, identity: ShapeIdentity) -> Option<bool>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMoveShape {
    shape: CShape,
    skills: BTreeMap<u32, MoveShapeSkill>,
    current_skill_id: Option<u32>,
    item_skill_ids: Vec<u32>,
    ex_states: Vec<u8>,
    change_body_states: Vec<ChangeBodyState>,
    extended_states: Vec<ExtendedState>,
    undead_states: Vec<UndeadState>,
    moveable_count: i32,
    moveable: bool,
}

impl Default for CMoveShape {
    fn default() -> Self {
        Self {
            shape: CShape::default(),
            skills: BTreeMap::new(),
            current_skill_id: None,
            item_skill_ids: Vec::new(),
            ex_states: Vec::new(),
            change_body_states: Vec::new(),
            extended_states: Vec::new(),
            undead_states: Vec::new(),
            moveable_count: 0,
            moveable: true,
        }
    }
}

impl CMoveShape {
    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub(crate) const fn skills(&self) -> &BTreeMap<u32, MoveShapeSkill> {
        &self.skills
    }

    pub(crate) fn undead_states(&self) -> &[UndeadState] {
        &self.undead_states
    }

    pub(crate) fn ex_states(&self) -> &[u8] {
        &self.ex_states
    }

    pub(crate) fn serialized_ex_states(&self, now_ms: u32) -> Vec<u8> {
        let mut payload = self.ex_states.clone();
        for state in &self.change_body_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        for state in &self.extended_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        payload
    }

    pub(crate) fn replace_ex_states(&mut self, states: Vec<u8>) {
        self.change_body_states = ChangeBodyState::decode_all(&states, 0);
        self.extended_states = ExtendedState::decode_all(&states, 0);
        self.ex_states = states;
    }

    pub(crate) fn clear_persisted_runtime_state(&mut self) {
        self.skills.clear();
        self.current_skill_id = None;
        self.item_skill_ids.clear();
        self.ex_states.clear();
        self.change_body_states.clear();
        self.extended_states.clear();
        self.undead_states.clear();
    }

    /// Exact `AddUndeadState`: registry key `(56, stateID)`, затем удаление
    /// всех state того же type либо ID, после чего ID `0` оставляет только
    /// removal tail. Успешный Begin хранит state и возвращает `1`.
    pub(crate) fn add_undead_state<Now>(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: Now,
    ) -> UndeadStateMutation
    where
        Now: FnOnce() -> u32,
    {
        let Some(properties) =
            factory.query_skill_base_properties(SKILL_NOT_DISAPPEAR_AFTER_DEAD, state_id as i32)
        else {
            return UndeadStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
                state_list_changed: false,
            };
        };
        let state_type = properties.query_property(SKILL_USAGE_CONST) as u16;
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.undead_states.len() {
            if self.undead_states[index].state_type == state_type
                || self.undead_states[index].state_id == state_id
            {
                removed.push(self.undead_states.remove(index));
            } else {
                index += 1;
            }
        }
        if state_id == 0 {
            return UndeadStateMutation {
                state_list_changed: !removed.is_empty(),
                removed,
                added: None,
                legacy_return: 0,
            };
        }
        let state = UndeadState {
            state_id,
            state_type,
            keep_time_ms: properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            started_ms: now_ms(),
            properties: properties.properties().clone(),
        };
        self.undead_states.push(state.clone());
        UndeadStateMutation {
            removed,
            added: Some(state),
            legacy_return: 1,
            state_list_changed: true,
        }
    }

    /// Exact first-match `DelUndeadState`; native `End` удаляет найденный
    /// state и возвращает его ID, отсутствующий state возвращает ноль.
    pub(crate) fn delete_undead_state(&mut self, state_id: u32) -> UndeadStateMutation {
        let Some(index) = self
            .undead_states
            .iter()
            .position(|state| state.state_id == state_id)
        else {
            return UndeadStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
                state_list_changed: false,
            };
        };
        let removed = self.undead_states.remove(index);
        UndeadStateMutation {
            removed: vec![removed],
            added: None,
            legacy_return: state_id,
            state_list_changed: true,
        }
    }

    pub(crate) fn get_undead_state(&self, state_id: u32) -> u32 {
        self.undead_states
            .iter()
            .any(|state| state.state_id == state_id)
            .then_some(state_id)
            .unwrap_or(0)
    }

    pub(crate) fn add_extended_state(
        &mut self,
        kind: ExtendedStateKind,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: u32,
    ) -> ExtendedStateMutation {
        let Some(mut added) = ExtendedState::from_factory(kind, state_id, factory, now_ms) else {
            return ExtendedStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
            };
        };
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.extended_states.len() {
            if self.extended_states[index].kind == kind
                && (self.extended_states[index].state_type == added.state_type
                    || self.extended_states[index].level == state_id)
            {
                let state = self.extended_states.remove(index);
                self.remove_extended_state_serialized(&state);
                removed.push(state);
            } else {
                index += 1;
            }
        }
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            self.ex_states.extend_from_slice(&0_u32.to_le_bytes());
        }
        let count = u32::from_le_bytes(self.ex_states[..4].try_into().expect("state count"));
        self.ex_states[..4].copy_from_slice(&count.wrapping_add(1).to_le_bytes());
        let offset = self.ex_states.len();
        let size = match kind {
            ExtendedStateKind::Original => 44,
            ExtendedStateKind::New => 56,
        };
        self.ex_states.resize(offset + size, 0);
        added.write_serialized(&mut self.ex_states, offset);
        self.extended_states.push(added.clone());
        ExtendedStateMutation {
            removed,
            added: Some(added),
            legacy_return: 1,
        }
    }

    pub(crate) fn delete_extended_state(
        &mut self,
        kind: ExtendedStateKind,
        state_id: u32,
    ) -> ExtendedStateMutation {
        let Some(index) = self
            .extended_states
            .iter()
            .position(|state| state.kind == kind && state.level == state_id)
        else {
            return ExtendedStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
            };
        };
        let removed = self.extended_states.remove(index);
        self.remove_extended_state_serialized(&removed);
        ExtendedStateMutation {
            removed: vec![removed],
            added: None,
            legacy_return: state_id,
        }
    }

    fn remove_extended_state_serialized(&mut self, state: &ExtendedState) {
        let span = state.serialized_span();
        state.remove_serialized(&mut self.ex_states);
        if let Some((offset, amount)) = span {
            for state in &mut self.extended_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.change_body_states {
                state.shift_serialized_offset_after(offset, amount);
            }
        }
    }

    pub(crate) fn get_extended_state(&self, kind: ExtendedStateKind, state_id: u32) -> u32 {
        self.extended_states
            .iter()
            .any(|state| state.kind == kind && state.level == state_id)
            .then_some(state_id)
            .unwrap_or(0)
    }

    pub(crate) fn extended_states(&self) -> &[ExtendedState] {
        &self.extended_states
    }

    pub(crate) fn extended_states_mut(&mut self) -> &mut [ExtendedState] {
        &mut self.extended_states
    }

    pub(crate) fn activate_loaded_extended_states(&mut self, now_ms: u32) -> Vec<ExtendedState> {
        for state in &mut self.extended_states {
            state.activate_loaded(now_ms);
            state.update_serialized_runtime(&mut self.ex_states, now_ms);
        }
        self.extended_states.clone()
    }

    pub(crate) fn extended_state_tick(
        &mut self,
        now_ms: u32,
    ) -> (
        Vec<(ExtendedStateKind, u32)>,
        Vec<(ExtendedStateKind, u32, u32, u32)>,
    ) {
        let mut expired = Vec::new();
        let mut item_due = Vec::new();
        for state in &mut self.extended_states {
            if state.expired(now_ms) {
                expired.push((state.kind, state.level));
                continue;
            }
            if state.item_due(now_ms) {
                state.restart_item_clock(now_ms);
                item_due.push((state.kind, state.level, state.item_index, state.item_amount));
            }
        }
        (expired, item_due)
    }

    pub(crate) fn add_change_body_state(
        &mut self,
        state_id: u32,
        factory: &CSkillFactory,
        now_ms: u32,
        old_hotkeys: [u32; 12],
    ) -> ChangeBodyMutation {
        let Some(mut added) = ChangeBodyState::from_factory(state_id, factory, now_ms) else {
            return ChangeBodyMutation {
                removed: None,
                added: None,
                legacy_return: 0,
            };
        };
        added.old_hotkeys = old_hotkeys;
        let removed = self
            .change_body_states
            .iter()
            .position(|state| state.level == state_id)
            .map(|index| {
                let removed = self.change_body_states.remove(index);
                self.remove_change_body_state_serialized(&removed);
                removed
            });
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            self.ex_states.extend_from_slice(&0_u32.to_le_bytes());
        }
        let count = u32::from_le_bytes(self.ex_states[..4].try_into().expect("state count"));
        self.ex_states[..4].copy_from_slice(&count.wrapping_add(1).to_le_bytes());
        let offset = self.ex_states.len();
        self.ex_states
            .extend_from_slice(&super::chbystate::CHANGE_BODY_STATE_ID.to_le_bytes());
        self.ex_states.resize(offset + 124, 0);
        added.write_serialized(&mut self.ex_states, offset);
        self.change_body_states.push(added.clone());
        ChangeBodyMutation {
            removed,
            added: Some(added),
            legacy_return: 1,
        }
    }

    pub(crate) fn delete_change_body_state(&mut self, state_id: u32) -> ChangeBodyMutation {
        let Some(index) = self
            .change_body_states
            .iter()
            .position(|state| state.level == state_id)
        else {
            return ChangeBodyMutation {
                removed: None,
                added: None,
                legacy_return: 0,
            };
        };
        let removed = self.change_body_states.remove(index);
        self.remove_change_body_state_serialized(&removed);
        ChangeBodyMutation {
            removed: Some(removed),
            added: None,
            legacy_return: state_id,
        }
    }

    pub(crate) fn get_change_body_state(&self, state_id: u32) -> u32 {
        self.change_body_states
            .iter()
            .any(|state| state.level == state_id)
            .then_some(state_id)
            .unwrap_or_default()
    }

    fn remove_change_body_state_serialized(&mut self, state: &ChangeBodyState) {
        let span = state.serialized_span();
        state.remove_serialized(&mut self.ex_states);
        if let Some((offset, amount)) = span {
            for state in &mut self.extended_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.change_body_states {
                state.shift_serialized_offset_after(offset, amount);
            }
        }
    }

    pub(crate) fn active_change_body_state(&self) -> Option<&ChangeBodyState> {
        self.change_body_states.last()
    }

    pub(crate) fn activate_loaded_change_body_states(
        &mut self,
        now_ms: u32,
    ) -> Vec<ChangeBodyState> {
        for state in &mut self.change_body_states {
            state.activate_loaded(now_ms);
            state.update_serialized_runtime(&mut self.ex_states, now_ms);
        }
        self.change_body_states.clone()
    }

    pub(crate) fn expired_change_body_state_ids(&self, now_ms: u32) -> Vec<u32> {
        self.change_body_states
            .iter()
            .filter(|state| state.expired(now_ms))
            .map(|state| state.level)
            .collect()
    }

    pub(crate) fn change_body_region_transition_end_ids(&mut self) -> Vec<u32> {
        let mut ended = Vec::new();
        for state in &mut self.change_body_states {
            if state.on_change_region() {
                ended.push(state.level);
            } else {
                state.update_serialized_runtime(&mut self.ex_states, state.started_ms);
            }
        }
        ended
    }

    pub(crate) fn change_body_player_lost_end_ids(&mut self) -> Vec<u32> {
        let mut ended = Vec::new();
        for state in &mut self.change_body_states {
            if state.on_player_lost() {
                ended.push(state.level);
            } else {
                state.update_serialized_runtime(&mut self.ex_states, state.started_ms);
            }
        }
        ended
    }

    pub(crate) fn change_body_death_end_ids(&self) -> Vec<u32> {
        self.change_body_states
            .iter()
            .filter(|state| !state.continue_after_death)
            .map(|state| state.level)
            .collect()
    }

    pub(crate) fn skill(&self, skill_id: u32) -> Option<&MoveShapeSkill> {
        self.skills.get(&skill_id)
    }

    /// Достигнутый ID-view `GetCurrentSkill`: concrete `CSkill` execution и
    /// его `End` остаются у ещё не перенесённого skill owner-а, но caller-ы
    /// могут точно отличить запретный active skill `0xD4`.
    pub(crate) const fn current_skill_id(&self) -> Option<u32> {
        self.current_skill_id
    }

    /// Typed boundary для snapshot/skill caller-а. Полное semantic действие
    /// `SetCurrentSkill` (завершение прежнего concrete skill) не подменяется
    /// записью ID и остаётся у соответствующего owner-а.
    pub(crate) const fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.current_skill_id = skill_id;
    }

    /// Exact `SetItemSkill`: native owner только добавляет ID в ordered vector
    /// непосредственно перед передачей item-skill в `CPlayerAI`.
    pub(crate) fn set_item_skill(&mut self, skill_id: u32) {
        self.item_skill_ids.push(skill_id);
    }

    pub(crate) const fn is_moveable(&self) -> bool {
        self.moveable
    }

    pub(crate) const fn moveable_count(&self) -> i32 {
        self.moveable_count
    }

    /// Exact counter semantics `SetMoveable`: `false` ставит новый запрет,
    /// `true` снимает один; отрицательный счётчик не нормализуется в ветви
    /// снятия и потому сохраняется как наблюдаемая legacy-семантика.
    pub(crate) const fn set_moveable(&mut self, moveable: bool) {
        if !moveable {
            if self.moveable_count < 0 {
                self.moveable_count = 0;
            }
            self.moveable_count = self.moveable_count.wrapping_add(1);
        } else {
            self.moveable_count = self.moveable_count.wrapping_sub(1);
        }
        self.moveable = self.moveable_count < 1;
    }

    /// Exact `AddSkill(tagSkillID, long)` для already decoded factory registry:
    /// прежний ненулевой уровень не понижается; иначе entry заменяется только
    /// для одной из четырёх canonical категорий.
    pub(crate) fn add_skill(&mut self, skill_id: u32, level: i32, factory: &CSkillFactory) -> bool {
        if let Some(existing) = self.skills.get(&skill_id) {
            if existing.level != 0 && level <= existing.level {
                return true;
            }
        }
        self.skills.remove(&skill_id);
        let Some(properties) = factory.query_skill_base_properties(skill_id, level) else {
            return false;
        };
        let skill_type = properties.skill_type();
        if !matches!(
            skill_type,
            SKILL_TYPE_ATTACK | SKILL_TYPE_DEFENSE | SKILL_TYPE_STATE | SKILL_TYPE_SUMMON
        ) {
            return false;
        }
        self.skills.insert(
            skill_id,
            MoveShapeSkill {
                id: skill_id,
                level,
                skill_type,
                name: properties.skill_name().to_vec(),
            },
        );
        true
    }

    /// Exact reached state-transition `DelSkill(tagSkillID)`. Исходник всегда
    /// завершает текущий skill до category lookup, даже когда удаляется другой
    /// ID или искомой записи нет. Concrete `End/Delete` не имеют отдельного
    /// наблюдаемого state в достигнутой common-проекции.
    pub(crate) fn delete_skill(&mut self, skill_id: u32, factory: &CSkillFactory) -> bool {
        if skill_id == 0 {
            return false;
        }
        if self
            .current_skill_id
            .is_some_and(|current| self.skills.contains_key(&current))
        {
            self.current_skill_id = None;
        }
        if !matches!(
            factory.query_skill_type(skill_id, 1),
            SKILL_TYPE_ATTACK | SKILL_TYPE_DEFENSE | SKILL_TYPE_STATE | SKILL_TYPE_SUMMON
        ) {
            return false;
        }
        self.skills.remove(&skill_id);
        true
    }

    pub(crate) fn set_pos_xy(
        &mut self,
        region: &mut CRegion,
        x: f32,
        y: f32,
        facts: MoveShapePositionFacts,
    ) -> Result<(), MoveShapePositionBlock> {
        set_pos_xy_core(Some(region), &mut self.shape, x, y, facts)
    }

    pub(crate) const fn is_died(current_hit_points: u32) -> bool {
        current_hit_points == 0
    }

    pub(crate) fn get_dest_direction(
        source_x: i32,
        source_y: i32,
        destination_x: i32,
        destination_y: i32,
    ) -> i32 {
        let delta_x = source_x.wrapping_sub(destination_x);
        let delta_y = source_y.wrapping_sub(destination_y);
        // Подтверждённая странность GameServer RVA 0x000CCF60: совпавшие
        // точки возвращают DIR_DOWN `4`, а не отдельный sentinel.
        match (delta_x.signum(), delta_y.signum()) {
            (1, 1) => 7,
            (1, 0) => 6,
            (1, -1) => 5,
            (-1, 1) => 1,
            (-1, 0) => 2,
            (-1, -1) => 3,
            (0, 1) => 0,
            (0, 0 | -1) => 4,
            _ => unreachable!("signum возвращает только -1/0/1"),
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "literal ForceMove сохраняет исходные аргументы и две достигнутые owner-границы"
    )]
    pub(crate) fn force_move<Context: MoveShapeCommandContext>(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
        context: &mut Context,
    ) -> Result<bool, MoveShapeCommandBlock> {
        let Some(server_region) = server_region else {
            return Ok(false);
        };
        let width = server_region.region.width;
        let height = server_region.region.height;
        let clamped_x = clamp_force_x(destination_x, width);
        let clamped_y = clamp_force_y(destination_y, width, height);
        let old_x = self
            .shape
            .get_tile_x()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let old_y = self
            .shape
            .get_tile_y()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let identity = self.shape.identity();

        let mut message = CMessage::new(FORCE_MOVE_MESSAGE);
        message.add_long(identity.id);
        message.add_long(identity.object_type);
        message.add_long(old_x);
        message.add_long(old_y);
        message.add_long(clamped_x);
        message.add_long(clamped_y);
        message.add_ulong(duration_ms);
        message.add_long(0);
        let _ = message
            .send_to_around(Some(&*server_region), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        server_region
            .set_move_shape_tile_position(&mut self.shape, clamped_x, clamped_y, facts)
            .map_err(MoveShapeCommandBlock::Position)?;
        if let Some((ai, now_ms)) = context.ai_with_realtime_ms(identity) {
            ai.add_ai_event(AiShapeAction::Stand, duration_ms, 0, now_ms);
        }
        Ok(true)
    }

    pub(crate) fn on_move(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        run: i32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<(), MoveShapeCommandBlock> {
        let old_x = self
            .shape
            .get_tile_x()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        let old_y = self
            .shape
            .get_tile_y()
            .map_err(MoveShapeCommandBlock::Coordinate)?;
        self.shape.set_direction(get_line_direction(
            old_x,
            old_y,
            destination_x,
            destination_y,
        ));
        let identity = self.shape.identity();

        let mut message = CMessage::new(MOVE_MESSAGE);
        message.add_long(identity.id);
        message.add_long(identity.object_type);
        message.add_long(old_x);
        message.add_long(old_y);
        message.add_byte(1);
        message.add_byte(2 + u8::from(run != 0));
        message.add_long(destination_x);
        message.add_long(destination_y);
        message.add_long(destination_x);
        message.add_long(destination_y);
        let _ = message
            .send_to_around(server_region.as_deref(), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        if let Some(server_region) = server_region {
            server_region
                .set_move_shape_tile_position(&mut self.shape, destination_x, destination_y, facts)
                .map_err(MoveShapeCommandBlock::Position)
        } else {
            set_pos_xy_core(
                None,
                &mut self.shape,
                (destination_x as f32) + 0.5,
                (destination_y as f32) + 0.5,
                facts,
            )
            .map_err(MoveShapeCommandBlock::DetachedPosition)
        }
    }

    pub(crate) fn on_set_position(
        &mut self,
        server_region: Option<&mut CServerRegion>,
        destination_x: i32,
        destination_y: i32,
        facts: MoveShapePositionFacts,
        around: &GameServerAroundRuntime<'_>,
    ) -> Result<bool, MoveShapeCommandBlock> {
        let Some(server_region) = server_region else {
            return Ok(false);
        };
        let region = &server_region.region;
        if destination_x < 0
            || destination_x >= region.width
            || destination_y < 0
            || destination_y >= region.height
        {
            return Ok(false);
        }
        if region
            .get_block(destination_x, destination_y)
            .map_err(MoveShapeCommandBlock::RegionCell)?
            != 0
        {
            return Ok(false);
        }

        let identity = self.shape.identity();
        let mut message = CMessage::new(SET_POSITION_MESSAGE);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(destination_x);
        message.add_long(destination_y);
        message.add_long(0);
        let _ = message
            .send_to_around(Some(&*server_region), &self.shape, None, around)
            .map_err(MoveShapeCommandBlock::Coordinate)?;

        server_region
            .set_move_shape_tile_position(&mut self.shape, destination_x, destination_y, facts)
            .map_err(MoveShapeCommandBlock::Position)?;
        Ok(true)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MoveShapePositionDispatch {
    pub(crate) facts: MoveShapePositionFacts,
}

impl ShapePositionDispatch for MoveShapePositionDispatch {
    type Error = MoveShapePositionBlock;

    fn set_pos_xy(
        &mut self,
        region: &mut CRegion,
        shape: &mut CShape,
        x: f32,
        y: f32,
    ) -> Result<(), Self::Error> {
        set_pos_xy_core(Some(region), shape, x, y, self.facts)
    }
}

fn set_pos_xy_core(
    region: Option<&mut CRegion>,
    shape: &mut CShape,
    x: f32,
    y: f32,
    facts: MoveShapePositionFacts,
) -> Result<(), MoveShapePositionBlock> {
    if let Some(region) =
        region.filter(|region| shape.is_assigned_to_server_region() && region.width != 0)
    {
        let old_y = shape
            .get_tile_y()
            .map_err(MoveShapePositionBlock::Coordinate)?;
        let old_x = shape
            .get_tile_x()
            .map_err(MoveShapePositionBlock::Coordinate)?;
        shape
            .set_block(region, old_x, old_y, 0, facts.figure)
            .map_err(MoveShapePositionBlock::ShapeBlock)?;

        if facts.current_hit_points != 0 || shape.identity().object_type == NPC_TYPE {
            let new_y = CShape::tile_from_value(y).map_err(MoveShapePositionBlock::Coordinate)?;
            let new_x = CShape::tile_from_value(x).map_err(MoveShapePositionBlock::Coordinate)?;
            shape
                .set_block(region, new_x, new_y, 3, facts.figure)
                .map_err(MoveShapePositionBlock::ShapeBlock)?;
        }
    }

    shape.set_pos_xy_move_order(x, y);
    let tile_y = CShape::tile_from_value(y).map_err(MoveShapePositionBlock::Coordinate)?;
    let tile_x = CShape::tile_from_value(x).map_err(MoveShapePositionBlock::Coordinate)?;
    if facts.area_width <= 0 || facts.area_height <= 0 {
        return Err(MoveShapePositionBlock::InvalidAreaSpan {
            width: facts.area_width,
            height: facts.area_height,
        });
    }

    let next_area = ShapeAreaCoordinates {
        x: tile_x / facts.area_width,
        y: tile_y / facts.area_height,
    };
    if facts
        .current_area
        .is_some_and(|current| current != next_area)
    {
        shape.set_next_area_coordinates(next_area);
        shape.set_change_state(SHAPE_CHANGE_AREA);
    } else {
        shape.set_change_state(SHAPE_CHANGE_NONE);
    }
    Ok(())
}

fn clamp_force_x(destination: i32, width: i32) -> i32 {
    if destination < 0 {
        0
    } else if destination >= width {
        width.wrapping_sub(1)
    } else {
        destination
    }
}

fn clamp_force_y(destination: i32, width: i32, height: i32) -> i32 {
    if destination < 0 {
        0
    } else if destination >= height {
        // Подтверждённая странность GameServer RVA 0x000CD1A0:
        // `if (height <= lDestY) lDestY = width - 1;`.
        width.wrapping_sub(1)
    } else {
        destination
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h

// ============================================================================
// FUNCTION: CMoveShape::God
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:151
// RVA: 0x0002ACB0
// ADDRESS: 0042acb0
// PROTOTYPE: void __thiscall God(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::CanMove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:155
// RVA: 0x0002ACC0
// ADDRESS: 0042acc0
// PROTOTYPE: int __thiscall CanMove(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetBeAttackedPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:76
// RVA: 0x0004A250
// ADDRESS: 0044a250
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAttackerDir
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:80
// RVA: 0x0004A270
// ADDRESS: 0044a270
// PROTOTYPE: long __thiscall GetAttackerDir(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:90
// RVA: 0x0004A280
// ADDRESS: 0044a280
// PROTOTYPE: CBaseAI * __thiscall GetAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAlertRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:142
// RVA: 0x0004A290
// ADDRESS: 0044a290
// PROTOTYPE: long __thiscall GetAlertRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetTrackRange
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:143
// RVA: 0x0004A2A0
// ADDRESS: 0044a2a0
// PROTOTYPE: long __thiscall GetTrackRange(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:332
// RVA: 0x0004A2B0
// ADDRESS: 0044a2b0
// PROTOTYPE: void __thiscall SetAttackAble(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:333
// RVA: 0x0004A2C0
// ADDRESS: 0044a2c0
// PROTOTYPE: bool __thiscall GetAttackAble(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetFightable
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:119
// RVA: 0x000CCE10
// ADDRESS: 004cce10
// PROTOTYPE: void __thiscall SetFightable(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetKilledMeAttackInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:139
// RVA: 0x000CCE50
// ADDRESS: 004cce50
// PROTOTYPE: void __thiscall SetKilledMeAttackInfo(tagAttackInformation * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAtcInterval
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:367
// RVA: 0x000CCF40
// ADDRESS: 004ccf40
// PROTOTYPE: ushort __thiscall GetAtcInterval(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetStrikeOutTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:373
// RVA: 0x000CCF50
// ADDRESS: 004ccf50
// PROTOTYPE: ulong __thiscall GetStrikeOutTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CMoveShape::GetDestDir` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CMoveShape::GetCurrentPetsMode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2947
// RVA: 0x000CCFC0
// ADDRESS: 004ccfc0
// PROTOTYPE: PET_SEARCH_ENEMY_MODE __thiscall GetCurrentPetsMode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::FindPositionForCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3015
// RVA: 0x000CCFD0
// ADDRESS: 004ccfd0
// PROTOTYPE: bool __thiscall FindPositionForCarriage(CMoveShape * param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CMoveShape::SetPosXY` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CMoveShape::ForceMove` материализован выше; покрытый raw-блок
// удалён.

// ============================================================================
// FUNCTION: CMoveShape::Stiffen
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2316
// RVA: 0x000CD2F0
// ADDRESS: 004cd2f0
// PROTOTYPE: ulong __thiscall Stiffen(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnChangeStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2359
// RVA: 0x000CD3E0
// ADDRESS: 004cd3e0
// PROTOTYPE: void __thiscall OnChangeStates(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CMoveShape::OnMove` материализован выше; покрытый raw-блок
// удалён.

// IMPLEMENTED: `CMoveShape::OnSetPosition` материализован выше; покрытый
// raw-блок удалён.

// ============================================================================
// FUNCTION: CMoveShape::GetPetsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2823
// RVA: 0x000CD6D0
// ADDRESS: 004cd6d0
// PROTOTYPE: ulong __thiscall GetPetsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::Evanish
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2977
// RVA: 0x000CD700
// ADDRESS: 004cd700
// PROTOTYPE: void __thiscall Evanish(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3109
// RVA: 0x000CD7C0
// ADDRESS: 004cd7c0
// PROTOTYPE: void __thiscall DelCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DoesStateExist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:634
// RVA: 0x000CD9C0
// ADDRESS: 004cd9c0
// PROTOTYPE: int __thiscall DoesStateExist(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetStateBySkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:648
// RVA: 0x000CDA10
// ADDRESS: 004cda10
// PROTOTYPE: CState * __thiscall GetStateBySkillID(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetStateNumByStateID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:662
// RVA: 0x000CDA60
// ADDRESS: 004cda60
// PROTOTYPE: uint __thiscall GetStateNumByStateID(tagStateID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::RemoveState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:725
// RVA: 0x000CDAB0
// ADDRESS: 004cdab0
// PROTOTYPE: void __thiscall RemoveState(CState * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::RemoveState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:744
// RVA: 0x000CDB20
// ADDRESS: 004cdb20
// PROTOTYPE: void __thiscall RemoveState(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AutoStartPassiveSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1687
// RVA: 0x000CDBB0
// ADDRESS: 004cdbb0
// PROTOTYPE: void __thiscall AutoStartPassiveSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetCurrentSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1720
// RVA: 0x000CDC10
// ADDRESS: 004cdc10
// PROTOTYPE: CSkill * __thiscall GetCurrentSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddToByteArray_ForClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1779
// RVA: 0x000CDD30
// ADDRESS: 004cdd30
// PROTOTYPE: bool __thiscall AddToByteArray_ForClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelUndeadState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1950
// RVA: 0x000CDE80
// ADDRESS: 004cde80
// PROTOTYPE: uint __thiscall DelUndeadState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetUndeadState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1971
// RVA: 0x000CDEF0
// ADDRESS: 004cdef0
// PROTOTYPE: uint __thiscall GetUndeadState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::StopAllSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2096
// RVA: 0x000CDF50
// ADDRESS: 004cdf50
// PROTOTYPE: void __thiscall StopAllSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::StartAllStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2234
// RVA: 0x000CE050
// ADDRESS: 004ce050
// PROTOTYPE: void __thiscall StartAllStates(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2303
// RVA: 0x000CE1E0
// ADDRESS: 004ce1e0
// PROTOTYPE: void __thiscall OnAction(tagAction param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetDefaultAttackSkillID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2464
// RVA: 0x000CE240
// ADDRESS: 004ce240
// PROTOTYPE: tagSkillID __thiscall GetDefaultAttackSkillID(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2677
// RVA: 0x000CE2D0
// ADDRESS: 004ce2d0
// PROTOTYPE: CSkill * __thiscall GetSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::FindPositionForPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2771
// RVA: 0x000CE450
// ADDRESS: 004ce450
// PROTOTYPE: int __thiscall FindPositionForPet(CMoveShape * param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::IncreaseExperienceForAllFallowers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2854
// RVA: 0x000CE5A0
// ADDRESS: 004ce5a0
// PROTOTYPE: void __thiscall IncreaseExperienceForAllFallowers(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetCurrentPetAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2891
// RVA: 0x000CE6B0
// ADDRESS: 004ce6b0
// PROTOTYPE: void __thiscall SetCurrentPetAction(PET_ACTION param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetTargetForAllPets
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2911
// RVA: 0x000CE790
// ADDRESS: 004ce790
// PROTOTYPE: void __thiscall SetTargetForAllPets(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetCurrentPetsMode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2952
// RVA: 0x000CE8C0
// ADDRESS: 004ce8c0
// PROTOTYPE: void __thiscall SetCurrentPetsMode(PET_SEARCH_ENEMY_MODE param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3299
// RVA: 0x000CE9C0
// ADDRESS: 004ce9c0
// PROTOTYPE: uint __thiscall DelExState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelExStateByType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3321
// RVA: 0x000CEA30
// ADDRESS: 004cea30
// PROTOTYPE: uint __thiscall DelExStateByType(ushort param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelExStateNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3343
// RVA: 0x000CEAA0
// ADDRESS: 004ceaa0
// PROTOTYPE: uint __thiscall DelExStateNew(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3365
// RVA: 0x000CEB10
// ADDRESS: 004ceb10
// PROTOTYPE: uint __thiscall GetExState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetExStateNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3384
// RVA: 0x000CEB70
// ADDRESS: 004ceb70
// PROTOTYPE: uint __thiscall GetExStateNew(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DelCHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3603
// RVA: 0x000CEBD0
// ADDRESS: 004cebd0
// PROTOTYPE: uint __thiscall DelCHBYState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetCHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3625
// RVA: 0x000CEC40
// ADDRESS: 004cec40
// PROTOTYPE: uint __thiscall GetCHBYState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::ClearSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:192
// RVA: 0x000CECE0
// ADDRESS: 004cece0
// PROTOTYPE: void __thiscall ClearSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::SetCurrentSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1704
// RVA: 0x000CEEE0
// ADDRESS: 004ceee0
// PROTOTYPE: void __thiscall SetCurrentSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnEnterRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2043
// RVA: 0x000CEF40
// ADDRESS: 004cef40
// PROTOTYPE: void __thiscall OnEnterRegion(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::ClearAllStates
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2132
// RVA: 0x000CF090
// ADDRESS: 004cf090
// PROTOTYPE: void __thiscall ClearAllStates(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::CheckSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2664
// RVA: 0x000CF540
// ADDRESS: 004cf540
// PROTOTYPE: long __thiscall CheckSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CMoveShape::CheckSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2652
// RVA: 0x000CF590
// ADDRESS: 004cf590
// PROTOTYPE: long __thiscall CheckSkill(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::RemovePet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2753
// RVA: 0x000CF5D0
// ADDRESS: 004cf5d0
// PROTOTYPE: int __thiscall RemovePet(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetValidPetsAmount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2828
// RVA: 0x000CF650
// ADDRESS: 004cf650
// PROTOTYPE: ulong __thiscall GetValidPetsAmount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::~CMoveShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:82
// RVA: 0x000CF950
// ADDRESS: 004cf950
// PROTOTYPE: void __thiscall ~CMoveShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnBeginSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:283
// RVA: 0x000CFB40
// ADDRESS: 004cfb40
// PROTOTYPE: int __thiscall OnBeginSkill(tagSkillID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetWeaponModifier
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.h:358
// RVA: 0x000CFB50
// ADDRESS: 004cfb50
// PROTOTYPE: float __thiscall GetWeaponModifier(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::UpdateProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:93
// RVA: 0x000CFB60
// ADDRESS: 004cfb60
// PROTOTYPE: void __thiscall UpdateProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004cfbff
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:107
// RVA: 0x000CFBFF
// ADDRESS: 004cfbff
// PROTOTYPE: undefined Catch@004cfbff()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::UpdateAbnormality
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:247
// RVA: 0x000CFD00
// ADDRESS: 004cfd00
// PROTOTYPE: void __thiscall UpdateAbnormality(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d002e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:301
// RVA: 0x000D002E
// ADDRESS: 004d002e
// PROTOTYPE: undefined Catch@004d002e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3036
// RVA: 0x000D0140
// ADDRESS: 004d0140
// PROTOTYPE: bool __thiscall AddCarriage(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:230
// RVA: 0x000D0530
// ADDRESS: 004d0530
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::CMoveShape
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:50
// RVA: 0x000D0C60
// ADDRESS: 004d0c60
// PROTOTYPE: undefined __thiscall CMoveShape(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::ApplyFinalDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1607
// RVA: 0x000D0DA0
// ADDRESS: 004d0da0
// PROTOTYPE: void __thiscall ApplyFinalDamage(tagAttackInformation * param_1, vector<CMoveShape::tagDamage*,std::allocator<CMoveShape::tagDamage*>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddExStatesToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1814
// RVA: 0x000D10F0
// ADDRESS: 004d10f0
// PROTOTYPE: bool __thiscall AddExStatesToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::GetAllPets
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2931
// RVA: 0x000D1270
// ADDRESS: 004d1270
// PROTOTYPE: void __thiscall GetAllPets(vector<CMonster*,std::allocator<CMonster*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::InitSkills
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:182
// RVA: 0x000D1460
// ADDRESS: 004d1460
// PROTOTYPE: void __thiscall InitSkills(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:674
// RVA: 0x000D1560
// ADDRESS: 004d1560
// PROTOTYPE: int __thiscall AddState(tagStateID param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddUndeadState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1862
// RVA: 0x000D1780
// ADDRESS: 004d1780
// PROTOTYPE: uint __thiscall AddUndeadState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::DecodeExStatesFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:1993
// RVA: 0x000D1A80
// ADDRESS: 004d1a80
// PROTOTYPE: void __thiscall DecodeExStatesFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CMoveShape::AddPet
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:2742
// RVA: 0x000D1E00
// ADDRESS: 004d1e00
// PROTOTYPE: int __thiscall AddPet(long param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3155
// RVA: 0x000D1E40
// ADDRESS: 004d1e40
// PROTOTYPE: uint __thiscall AddExState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddExStateNew
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3228
// RVA: 0x000D20D0
// ADDRESS: 004d20d0
// PROTOTYPE: uint __thiscall AddExStateNew(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::prison_check
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3404
// RVA: 0x000D2360
// ADDRESS: 004d2360
// PROTOTYPE: void __thiscall prison_check(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::AddCHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:3523
// RVA: 0x000D2590
// ADDRESS: 004d2590
// PROTOTYPE: uint __thiscall AddCHBYState(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMoveShape::OnBeenAttacked
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\moveshape.cpp:762
// RVA: 0x000D2890
// ADDRESS: 004d2890
// PROTOTYPE: void __thiscall OnBeenAttacked(tagAttackInformation * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
