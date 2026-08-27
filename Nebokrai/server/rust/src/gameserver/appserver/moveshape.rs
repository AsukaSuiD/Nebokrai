//! Реализованная часть `CMoveShape` исторического GameServer.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! исходные владельцы `appserver/moveshape.h/.cpp`. Сохранены точный порядок
//! смены пространственной принадлежности, двоичные форматы `0xBF603/604/605`,
//! счётчики запрета движения и боя, а также подтверждённая странность
//! `ForceMove`, где верхняя граница Y записывает `width - 1`.
//!
//! Известные состояния смены тела, расширенные состояния, бессмертие,
//! сценарные состояния и езда принадлежат одному `CanonicalStateStorage`.
//! Сырой `ex_states` скрыт внутри `LegacyStateCodec` и служит только для
//! сохранения точного порядка, неизвестных записей и обратного wire-кодека;
//! игровое поведение читает типизированные состояния. Добавление, замена,
//! таймеры и удаление обновляют типизированную модель и её кодек в одной
//! операции с прежними смещениями и порядком.
//! Little-endian доступ к legacy codec выполняют общие `LegacyReader` и
//! `LegacyWriter`; размещение записей и их смещения остаются у этого owner-а.
//!
//! Реализованные `AddSkill`, `DelSkill`, `AddState`, `GetStatesNum` и
//! `UpdateAbnormality` используют это же хранилище. Ещё не восстановленные
//! классы навыков и AI остаются в сохранённом `UNKNOWN` (исследовательский декомпилят хранится локально) ниже.

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use indexmap::IndexSet;

use super::ai::baseai::{AiShapeAction, CBaseAI};
use super::chbystate::{CHANGE_BODY_STATE_ID, ChangeBodyMutation, ChangeBodyState};
use super::exstate::{
    EX_STATE_ID, EX_STATE_NEW_ID, ExtendedState, ExtendedStateKind, ExtendedStateMutation,
};
use super::legacycodec::{LegacyReader, LegacyWriter};
use super::region::{CRegion, RegionCellAccessBlock};
use super::ridestate::{RIDE_STATE_ID, RideState};
use super::serverregion::{CServerRegion, RegionMembershipBlock};
use super::shape::{
    CShape, SHAPE_CHANGE_AREA, SHAPE_CHANGE_NONE, ShapeAreaCoordinates, ShapeBlockError,
    ShapeCoordinateBlock, ShapeFigure, ShapeIdentity, ShapePositionDispatch, ShapeResolver,
};
use crate::gameserver::appserver::skills::agilitystate::{
    AgilityState, PersistentAgilityFamilyState,
};
use crate::gameserver::appserver::skills::callositystate::CallosityState;
use crate::gameserver::appserver::skills::curestate::CureState;
use crate::gameserver::appserver::skills::enlargefullmissstate::EnlargeFullMissState;
use crate::gameserver::appserver::skills::enlargemaxhpstate::EnlargeMaxHpState;
use crate::gameserver::appserver::skills::enlargemaxmpstate::EnlargeMaxMpState;
use crate::gameserver::appserver::skills::heartenstate::HeartenState;
use crate::gameserver::appserver::skills::lifeshieldstate::LifeShieldState;
use crate::gameserver::appserver::skills::machineshieldstate::MachineShieldState;
use crate::gameserver::appserver::skills::manashieldstate::ManaShieldState;
use crate::gameserver::appserver::skills::originstate::OriginState;
use crate::gameserver::appserver::skills::poisonarrowstate::PoisonArrowState;
use crate::gameserver::appserver::skills::bloodlossstate::BloodLossState;
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::appserver::skills::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::skills::taijistate::TaiJiState;
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
const UNDEAD_STATE_ID: u32 = 0x38;
const UNDEAD_STATE_PARAMETER_BYTES: usize = 72;
pub(crate) const STATE_USER_GOODS_ENLARGE_MAX_HP: i32 = 100_007;
pub(crate) const STATE_USER_GOODS_ENLARGE_MAX_MP: i32 = 100_008;
pub(crate) const STATE_IMPROVE_EXP: i32 = 100_009;
pub(crate) const STATE_USER_GOODS_ENLARGE_DEF: i32 = 100_010;
pub(crate) const STATE_USER_GOODS_ENLARGE_ELM_DEF: i32 = 100_011;
pub(crate) const STATE_USER_GOODS_ENLARGE_FULL_MISS: i32 = 100_012;
pub(crate) const STATE_AUTO_PROTECT: i32 = 110_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMoveState {
    state_id: i32,
    keep_time: u32,
    coefficient: u32,
    visual_pending: bool,
}

impl ScriptMoveState {
    pub(crate) const fn state_id(self) -> i32 {
        self.state_id
    }

    pub(crate) const fn keep_time(self) -> u32 {
        self.keep_time
    }

    pub(crate) const fn coefficient(self) -> u32 {
        self.coefficient
    }

    pub(crate) const fn is_auto_protect(self) -> bool {
        self.state_id == STATE_AUTO_PROTECT
    }
}

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

/// Достигнутый wire/lifecycle owner `CNotDisappearAfterDead`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UndeadState {
    state_id: u32,
    state_type: u16,
    keep_time_ms: u32,
    started_ms: u32,
    last_item_tick_ms: u32,
    pub(crate) disappear_after_dead: bool,
    pub(crate) percentage: bool,
    pub(crate) maximum_hp: i16,
    pub(crate) maximum_mp: i16,
    pub(crate) minimum_attack: i16,
    pub(crate) maximum_attack: i16,
    pub(crate) element_modify: i16,
    pub(crate) defense: i16,
    pub(crate) element_resistance: i16,
    pub(crate) blast_attack: i16,
    pub(crate) blast_element_attack: i16,
    pub(crate) strength: i32,
    pub(crate) dexterity: i32,
    pub(crate) constitution: i32,
    pub(crate) intelligence: i32,
    pub(crate) cch: i16,
    pub(crate) full_miss: i16,
    pub(crate) attack_avoid: i16,
    pub(crate) element_avoid: i16,
    pub(crate) hit: i16,
    pub(crate) dodge: i16,
    item_index: u32,
    item_amount: u32,
    frequency_ms: u32,
    serialized_offset: Option<usize>,
}

impl UndeadState {
    pub(crate) const fn state_id(&self) -> u32 {
        self.state_id
    }

    pub(crate) const fn state_type(&self) -> u16 {
        self.state_type
    }

    pub(crate) const fn keep_time_ms(&self) -> u32 {
        self.keep_time_ms
    }

    pub(crate) const fn started_ms(&self) -> u32 {
        self.started_ms
    }

    pub(crate) fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        if self.keep_time_ms == 0 {
            0
        } else {
            self.keep_time_ms
                .saturating_sub(now_ms.wrapping_sub(self.started_ms))
        }
    }

    fn from_factory(state_id: u32, factory: &CSkillFactory, now_ms: u32) -> Option<Self> {
        let properties =
            factory.query_skill_base_properties(SKILL_NOT_DISAPPEAR_AFTER_DEAD, state_id as i32)?;
        let p = |usage| properties.query_property(usage);
        Some(Self {
            state_id,
            state_type: p(SKILL_USAGE_CONST) as u16,
            keep_time_ms: p(SKILL_USAGE_STATE_PERSIST_TIME),
            started_ms: now_ms,
            last_item_tick_ms: now_ms,
            disappear_after_dead: p(80_001) != 0,
            percentage: p(80_002) != 0,
            maximum_hp: p(118) as i16,
            maximum_mp: p(119) as i16,
            minimum_attack: p(116) as i16,
            maximum_attack: p(117) as i16,
            element_modify: p(115) as i16,
            defense: p(109) as i16,
            element_resistance: p(112) as i16,
            blast_attack: p(125) as i16,
            blast_element_attack: p(126) as i16,
            strength: p(101) as i32,
            dexterity: p(102) as i32,
            constitution: p(103) as i32,
            intelligence: p(104) as i32,
            cch: p(108) as i16,
            full_miss: p(127) as i16,
            attack_avoid: p(128) as i16,
            element_avoid: p(129) as i16,
            hit: p(20_001) as i16,
            dodge: p(110) as i16,
            item_index: p(50_001),
            item_amount: p(50_002),
            frequency_ms: p(6_001),
            serialized_offset: None,
        })
    }

    fn decode_all(payload: &[u8], now_ms: u32) -> Vec<Self> {
        let mut states = Vec::new();
        for offset in 4..payload.len().saturating_sub(3) {
            if read_u32(payload, offset) != Some(UNDEAD_STATE_ID) {
                continue;
            }
            let base = offset + 4;
            if base + UNDEAD_STATE_PARAMETER_BYTES > payload.len() {
                continue;
            }
            let state_id = read_u32(payload, base + 4).unwrap_or_default();
            if state_id == 0 {
                continue;
            }
            states.push(Self {
                state_id,
                state_type: read_u16(payload, base).unwrap_or_default(),
                keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
                started_ms: now_ms,
                last_item_tick_ms: now_ms,
                disappear_after_dead: payload[base + 12] != 0,
                percentage: payload[base + 13] != 0,
                maximum_hp: read_i16(payload, base + 14).unwrap_or_default(),
                maximum_mp: read_i16(payload, base + 16).unwrap_or_default(),
                minimum_attack: read_i16(payload, base + 18).unwrap_or_default(),
                maximum_attack: read_i16(payload, base + 20).unwrap_or_default(),
                element_modify: read_i16(payload, base + 22).unwrap_or_default(),
                defense: read_i16(payload, base + 24).unwrap_or_default(),
                element_resistance: read_i16(payload, base + 26).unwrap_or_default(),
                blast_attack: read_i16(payload, base + 28).unwrap_or_default(),
                blast_element_attack: read_i16(payload, base + 30).unwrap_or_default(),
                strength: read_i32(payload, base + 32).unwrap_or_default(),
                dexterity: read_i32(payload, base + 36).unwrap_or_default(),
                constitution: read_i32(payload, base + 40).unwrap_or_default(),
                intelligence: read_i32(payload, base + 44).unwrap_or_default(),
                cch: read_i16(payload, base + 48).unwrap_or_default(),
                full_miss: read_i16(payload, base + 50).unwrap_or_default(),
                attack_avoid: read_i16(payload, base + 52).unwrap_or_default(),
                element_avoid: read_i16(payload, base + 54).unwrap_or_default(),
                hit: read_i16(payload, base + 56).unwrap_or_default(),
                dodge: read_i16(payload, base + 58).unwrap_or_default(),
                item_index: read_u32(payload, base + 60).unwrap_or_default(),
                item_amount: read_u32(payload, base + 64).unwrap_or_default(),
                frequency_ms: read_u32(payload, base + 68).unwrap_or_default(),
                serialized_offset: Some(offset),
            });
        }
        states
    }

    fn write_serialized(&mut self, payload: &mut [u8], offset: usize) {
        let base = offset + 4;
        write_u32(payload, offset, UNDEAD_STATE_ID);
        write_u16(payload, base, self.state_type);
        write_u32(payload, base + 4, self.state_id);
        write_u32(payload, base + 8, self.keep_time_ms);
        payload[base + 12] = u8::from(self.disappear_after_dead);
        payload[base + 13] = u8::from(self.percentage);
        for (position, value) in [
            (14, self.maximum_hp),
            (16, self.maximum_mp),
            (18, self.minimum_attack),
            (20, self.maximum_attack),
            (22, self.element_modify),
            (24, self.defense),
            (26, self.element_resistance),
            (28, self.blast_attack),
            (30, self.blast_element_attack),
            (48, self.cch),
            (50, self.full_miss),
            (52, self.attack_avoid),
            (54, self.element_avoid),
            (56, self.hit),
            (58, self.dodge),
        ] {
            write_i16(payload, base + position, value);
        }
        for (position, value) in [
            (32, self.strength),
            (36, self.dexterity),
            (40, self.constitution),
            (44, self.intelligence),
        ] {
            write_i32(payload, base + position, value);
        }
        write_u32(payload, base + 60, self.item_index);
        write_u32(payload, base + 64, self.item_amount);
        write_u32(payload, base + 68, self.frequency_ms);
        self.serialized_offset = Some(offset);
    }

    fn update_serialized_runtime(&self, payload: &mut [u8], now_ms: u32) {
        if let Some(offset) = self.serialized_offset
            && offset + 4 + UNDEAD_STATE_PARAMETER_BYTES <= payload.len()
        {
            write_u32(payload, offset + 12, self.remaining_time_ms(now_ms));
        }
    }

    fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + UNDEAD_STATE_PARAMETER_BYTES))
    }

    fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }

    fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.keep_time_ms < now_ms.wrapping_sub(self.started_ms)
    }

    fn item_due(&self, now_ms: u32) -> bool {
        self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.frequency_ms < now_ms.wrapping_sub(self.last_item_tick_ms)
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
pub(crate) struct MoveShapePet {
    pub(crate) object_type: i32,
    pub(crate) id: i32,
    pub(crate) figure: i32,
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
    state_storage: CanonicalStateStorage,
    moveable_count: i32,
    moveable: bool,
    can_fight_count: i32,
    can_fight: bool,
    is_god: bool,
    pets: Vec<MoveShapePet>,
    current_pets_mode: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LegacyStateCodec {
    payload: Vec<u8>,
}

impl LegacyStateCodec {
    fn replace(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }
}

impl Deref for LegacyStateCodec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl DerefMut for LegacyStateCodec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CanonicalStateStorage {
    persistent_agility_family_state: Option<PersistentAgilityFamilyState>,
    agility_state_2: Option<AgilityState>,
    callosity_state: Option<CallosityState>,
    taiji_state: Option<TaiJiState>,
    enlarge_full_miss_state: Option<EnlargeFullMissState>,
    enlarge_max_hp_state: Option<EnlargeMaxHpState>,
    enlarge_max_mp_state: Option<EnlargeMaxMpState>,
    origin_state: Option<OriginState>,
    hearten_state: Option<HeartenState>,
    cure_state: Option<CureState>,
    poison_arrow_state: Option<PoisonArrowState>,
    blood_loss_state: Option<BloodLossState>,
    periodic_attack_order: IndexSet<u32>,
    defense_shields: Vec<DefenseShieldState>,
    ex_states: LegacyStateCodec,
    change_body_states: Vec<ChangeBodyState>,
    extended_states: Vec<ExtendedState>,
    undead_states: Vec<UndeadState>,
    script_states: Vec<ScriptMoveState>,
    ride_state: Option<RideState>,
}

impl Deref for CMoveShape {
    type Target = CanonicalStateStorage;

    fn deref(&self) -> &Self::Target {
        &self.state_storage
    }
}

impl DerefMut for CMoveShape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_storage
    }
}

impl Default for CMoveShape {
    fn default() -> Self {
        Self {
            shape: CShape::default(),
            skills: BTreeMap::new(),
            current_skill_id: None,
            item_skill_ids: Vec::new(),
            state_storage: CanonicalStateStorage::default(),
            moveable_count: 0,
            moveable: true,
            can_fight_count: 0,
            can_fight: true,
            is_god: false,
            pets: Vec::new(),
            current_pets_mode: 1,
        }
    }
}

impl CMoveShape {
    pub(crate) const fn current_pets_mode(&self) -> i32 {
        self.current_pets_mode
    }

    pub(crate) fn set_current_pets_mode(&mut self, mode: i32) -> bool {
        if self.current_pets_mode == mode {
            return false;
        }
        self.current_pets_mode = mode;
        true
    }

    pub(crate) fn add_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        self.pets.push(MoveShapePet {
            object_type,
            id,
            figure,
        });
    }

    pub(crate) fn remove_pet(&mut self, object_type: i32, id: i32) -> bool {
        let Some(index) = self
            .pets
            .iter()
            .position(|pet| pet.object_type == object_type && pet.id == id)
        else {
            return false;
        };
        self.pets.remove(index);
        true
    }

    pub(crate) fn pets(&self) -> &[MoveShapePet] {
        &self.pets
    }

    pub(crate) fn skill_level(&self, skill_id: u32) -> i32 {
        self.skills.get(&skill_id).map_or(0, MoveShapeSkill::level)
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    /// Exact inline `CMoveShape::God`: runtime-only invulnerability flag не
    /// сериализуется и проверяется ordinary `OnBeenAttacked` owner-ом.
    pub(crate) const fn set_god(&mut self, enabled: bool) {
        self.is_god = enabled;
    }

    pub(crate) const fn is_god(&self) -> bool {
        self.is_god
    }

    /// Exact nesting contract `SetFightable`: false добавляет запрет, true
    /// снимает один; отрицательный legacy count нормализуется только перед
    /// добавлением нового запрета.
    pub(crate) const fn set_fightable(&mut self, fightable: bool) {
        if !fightable {
            if self.can_fight_count < 0 {
                self.can_fight_count = 0;
            }
            self.can_fight_count = self.can_fight_count.wrapping_add(1);
        } else {
            self.can_fight_count = self.can_fight_count.wrapping_sub(1);
        }
        self.can_fight = self.can_fight_count < 1;
    }

    pub(crate) const fn can_fight(&self) -> bool {
        self.can_fight
    }

    pub(crate) const fn skills(&self) -> &BTreeMap<u32, MoveShapeSkill> {
        &self.skills
    }

    pub(crate) fn undead_states(&self) -> &[UndeadState] {
        &self.undead_states
    }

    pub(crate) fn serialized_ex_states(&self, now_ms: u32) -> Vec<u8> {
        let mut payload = self.ex_states.to_vec();
        for state in &self.change_body_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        for state in &self.extended_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        for state in &self.undead_states {
            state.update_serialized_runtime(&mut payload, now_ms);
        }
        payload
    }

    pub(crate) fn replace_ex_states(&mut self, states: Vec<u8>) {
        let known_offsets = known_state_record_offsets(&states);
        self.change_body_states = ChangeBodyState::decode_all(&states, 0);
        self.change_body_states.retain(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.extended_states = ExtendedState::decode_all(&states, 0);
        self.extended_states.retain(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.undead_states = UndeadState::decode_all(&states, 0);
        self.undead_states.retain(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.ride_state = RideState::decode(&states).filter(|state| {
            state
                .serialized_span()
                .is_some_and(|(offset, _)| known_offsets.contains(&offset))
        });
        self.ex_states.replace(states);
    }

    pub(crate) fn clear_persisted_runtime_state(&mut self) {
        self.skills.clear();
        self.current_skill_id = None;
        self.item_skill_ids.clear();
        self.ex_states.clear();
        self.persistent_agility_family_state = None;
        self.agility_state_2 = None;
        self.callosity_state = None;
        self.taiji_state = None;
        self.enlarge_full_miss_state = None;
        self.enlarge_max_hp_state = None;
        self.enlarge_max_mp_state = None;
        self.origin_state = None;
        self.hearten_state = None;
        self.cure_state = None;
        self.poison_arrow_state = None;
        self.blood_loss_state = None;
        self.periodic_attack_order.clear();
        self.defense_shields.clear();
        self.change_body_states.clear();
        self.extended_states.clear();
        self.undead_states.clear();
        self.script_states.clear();
        self.ride_state = None;
        self.can_fight_count = 0;
        self.can_fight = true;
    }

    pub(crate) const fn ride_state(&self) -> Option<&RideState> {
        self.state_storage.ride_state.as_ref()
    }

    pub(crate) const fn ride_state_mut(&mut self) -> Option<&mut RideState> {
        self.state_storage.ride_state.as_mut()
    }

    pub(crate) const fn has_ride_state(&self) -> bool {
        self.state_storage.ride_state.is_some()
    }

    pub(crate) const fn has_materialized_abnormality(&self) -> bool {
        self.state_storage.agility_state_2.is_some()
            || self.state_storage.hearten_state.is_some()
            || self.state_storage.cure_state.is_some()
            || self.state_storage.poison_arrow_state.is_some()
            || self.state_storage.blood_loss_state.is_some()
            || !self.state_storage.defense_shields.is_empty()
            || !self.state_storage.change_body_states.is_empty()
            || !self.state_storage.extended_states.is_empty()
            || !self.state_storage.undead_states.is_empty()
            || self.state_storage.ride_state.is_some()
    }

    /// Точный фабричный диапазон `CMoveShape::AddState`: остальные ID не
    /// создают состояние. Значения принимают исходное знаковое представление
    /// сценария и сохраняются как поля `DWORD` конкретных классов.
    pub(crate) fn add_script_state(
        &mut self,
        state_id: i32,
        value1: i32,
        value2: i32,
        sufferer_is_gm: bool,
    ) -> Option<ScriptMoveState> {
        let supported = matches!(
            state_id,
            STATE_USER_GOODS_ENLARGE_MAX_HP
                | STATE_USER_GOODS_ENLARGE_MAX_MP
                | STATE_IMPROVE_EXP
                | STATE_USER_GOODS_ENLARGE_DEF
                | STATE_USER_GOODS_ENLARGE_ELM_DEF
                | STATE_USER_GOODS_ENLARGE_FULL_MISS
                | STATE_AUTO_PROTECT
        );
        if !supported || (state_id == STATE_AUTO_PROTECT && sufferer_is_gm) {
            return None;
        }
        let state = ScriptMoveState {
            state_id,
            keep_time: value1 as u32,
            coefficient: if state_id == STATE_AUTO_PROTECT {
                0
            } else {
                value2 as u32
            },
            // У `AutoProtect` зацикленный визуальный эффект публикует начало
            // прямо из `Begin`; остальные эффекты ждут `UpdateProperty`.
            visual_pending: state_id != STATE_AUTO_PROTECT,
        };
        self.script_states.push(state);
        Some(state)
    }

    pub(crate) fn script_state_count(&self, state_id: i32) -> u32 {
        let scripted = self
            .script_states
            .iter()
            .filter(|state| state.state_id == state_id)
            .count();
        let change_body = (state_id == 0x37)
            .then_some(self.change_body_states.len())
            .unwrap_or(0);
        let extended = self
            .extended_states
            .iter()
            .filter(|state| state.kind.state_id() as i32 == state_id)
            .count();
        let undead = (state_id == UNDEAD_STATE_ID as i32)
            .then_some(self.undead_states.len())
            .unwrap_or(0);
        let ride = usize::from(state_id == RIDE_STATE_ID as i32 && self.ride_state.is_some());
        let callosity = usize::from(
            self.callosity_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let agility = usize::from(
            self.persistent_agility_family_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        ) + usize::from(
            self.agility_state_2
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let taiji = usize::from(
            self.taiji_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let enlarge_full_miss = usize::from(
            self.enlarge_full_miss_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let enlarge_max_hp = usize::from(
            self.enlarge_max_hp_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let enlarge_max_mp = usize::from(
            self.enlarge_max_mp_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let origin = usize::from(
            self.origin_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let hearten = usize::from(
            self.hearten_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let cure = usize::from(
            self.cure_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let poison_arrow = usize::from(
            self.poison_arrow_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let blood_loss = usize::from(
            self.blood_loss_state
                .is_some_and(|state| state.skill_id() as i32 == state_id),
        );
        let shields = self
            .defense_shields
            .iter()
            .filter(|state| state.skill_id() as i32 == state_id)
            .count();
        scripted
            .saturating_add(agility)
            .saturating_add(callosity)
            .saturating_add(taiji)
            .saturating_add(enlarge_full_miss)
            .saturating_add(enlarge_max_hp)
            .saturating_add(enlarge_max_mp)
            .saturating_add(origin)
            .saturating_add(hearten)
            .saturating_add(cure)
            .saturating_add(poison_arrow)
            .saturating_add(blood_loss)
            .saturating_add(shields)
            .saturating_add(change_body)
            .saturating_add(extended)
            .saturating_add(undead)
            .saturating_add(ride)
            .min(u32::MAX as usize) as u32
    }

    /// `GetStateBySkillID` просматривает канонические типизированные состояния
    /// по фактическому идентификатору навыка, а не по классу сетевой записи.
    pub(crate) fn has_state_by_skill_id(&self, state_id: u32) -> bool {
        self.persistent_agility_family_state
            .is_some_and(|state| state.skill_id() == state_id)
            || self
                .agility_state_2
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .callosity_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .taiji_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .enlarge_full_miss_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .enlarge_max_hp_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .enlarge_max_mp_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .origin_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .hearten_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .cure_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .poison_arrow_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .blood_loss_state
                .is_some_and(|state| state.skill_id() == state_id)
            || self
                .defense_shields
                .iter()
                .any(|state| state.skill_id() == state_id)
            || self
                .script_states
                .iter()
                .any(|state| state.state_id as u32 == state_id)
            || self
                .change_body_states
                .iter()
                .any(|state| state.level == state_id)
            || self
                .extended_states
                .iter()
                .any(|state| state.level == state_id)
            || self
                .undead_states
                .iter()
                .any(|state| state.state_id() == state_id)
    }

    pub(crate) fn callosity_state(&self) -> Option<CallosityState> {
        self.state_storage.callosity_state
    }

    pub(crate) fn take_callosity_state(&mut self, skill_id: u32) -> Option<CallosityState> {
        self.callosity_state
            .filter(|state| state.skill_id() == skill_id)?;
        self.callosity_state.take()
    }

    pub(crate) fn begin_callosity_state(&mut self, state: CallosityState) {
        debug_assert!(self.callosity_state.is_none());
        self.callosity_state = Some(state);
    }

    pub(crate) const fn taiji_state(&self) -> Option<TaiJiState> {
        self.state_storage.taiji_state
    }

    pub(crate) fn replace_taiji_state(&mut self, state: TaiJiState) -> Option<TaiJiState> {
        self.taiji_state.replace(state)
    }

    pub(crate) fn replace_enlarge_max_hp_state(
        &mut self,
        state: EnlargeMaxHpState,
    ) -> Option<EnlargeMaxHpState> {
        self.enlarge_max_hp_state.replace(state)
    }

    pub(crate) fn replace_enlarge_full_miss_state(
        &mut self,
        state: EnlargeFullMissState,
    ) -> Option<EnlargeFullMissState> {
        self.enlarge_full_miss_state.replace(state)
    }

    pub(crate) const fn enlarge_full_miss_state(&self) -> Option<EnlargeFullMissState> {
        self.state_storage.enlarge_full_miss_state
    }

    pub(crate) fn replace_enlarge_max_mp_state(
        &mut self,
        state: EnlargeMaxMpState,
    ) -> Option<EnlargeMaxMpState> {
        self.enlarge_max_mp_state.replace(state)
    }

    pub(crate) const fn enlarge_max_hp_state(&self) -> Option<EnlargeMaxHpState> {
        self.state_storage.enlarge_max_hp_state
    }

    pub(crate) const fn enlarge_max_mp_state(&self) -> Option<EnlargeMaxMpState> {
        self.state_storage.enlarge_max_mp_state
    }

    pub(crate) fn replace_origin_state(&mut self, state: OriginState) -> Option<OriginState> {
        self.origin_state.replace(state)
    }

    pub(crate) const fn origin_state(&self) -> Option<OriginState> {
        self.state_storage.origin_state
    }

    pub(crate) fn replace_hearten_state(&mut self, state: HeartenState) -> Option<HeartenState> {
        self.hearten_state.replace(state)
    }

    pub(crate) const fn hearten_state(&self) -> Option<HeartenState> {
        self.state_storage.hearten_state
    }

    pub(crate) fn take_expired_hearten_state(&mut self, now_ms: u32) -> Option<HeartenState> {
        self.hearten_state.filter(|state| state.expired(now_ms))?;
        self.hearten_state.take()
    }

    pub(crate) fn replace_mana_shield_state(
        &mut self,
        state: ManaShieldState,
    ) -> Option<ManaShieldState> {
        let previous = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.defense_shields.remove(position))
            .and_then(|candidate| match candidate {
                DefenseShieldState::Life(_) => None,
                DefenseShieldState::Mana(previous) => Some(previous),
                DefenseShieldState::Machine(_) => None,
            });
        self.defense_shields.push(DefenseShieldState::Mana(state));
        previous
    }

    pub(crate) fn replace_machine_shield_state(
        &mut self,
        state: MachineShieldState,
    ) -> Option<MachineShieldState> {
        let previous = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.defense_shields.remove(position))
            .and_then(|candidate| match candidate {
                DefenseShieldState::Life(_) => None,
                DefenseShieldState::Machine(previous) => Some(previous),
                DefenseShieldState::Mana(_) => None,
            });
        self.defense_shields
            .push(DefenseShieldState::Machine(state));
        previous
    }

    pub(crate) fn replace_life_shield_state(
        &mut self,
        state: LifeShieldState,
    ) -> Option<LifeShieldState> {
        let previous = self
            .defense_shields
            .iter()
            .position(|candidate| candidate.skill_id() == state.skill_id())
            .map(|position| self.defense_shields.remove(position))
            .and_then(|candidate| match candidate {
                DefenseShieldState::Life(previous) => Some(previous),
                DefenseShieldState::Machine(_) | DefenseShieldState::Mana(_) => None,
            });
        self.defense_shields.push(DefenseShieldState::Life(state));
        previous
    }

    pub(crate) fn take_expired_defense_shields(
        &mut self,
        now_ms: u32,
        player_mana: u32,
        player_dead: bool,
        war_soul_mana: Option<i32>,
    ) -> Vec<DefenseShieldState> {
        let mut expired = Vec::new();
        let mut position = 0;
        while position < self.defense_shields.len() {
            if self.defense_shields[position].expired(
                now_ms,
                player_mana,
                player_dead,
                war_soul_mana,
            ) {
                expired.push(self.defense_shields.remove(position));
            } else {
                position += 1;
            }
        }
        expired
    }

    pub(crate) fn take_defense_shields(&mut self) -> Vec<DefenseShieldState> {
        std::mem::take(&mut self.defense_shields)
    }

    pub(crate) fn restore_defense_shields(&mut self, states: Vec<DefenseShieldState>) {
        self.defense_shields = states;
    }

    pub(crate) fn replace_cure_state(&mut self, state: CureState) -> Option<CureState> {
        self.cure_state.replace(state)
    }

    pub(crate) fn take_cure_state_for_ai(&mut self) -> Option<CureState> {
        self.cure_state.take()
    }

    pub(crate) fn replace_poison_arrow_state(
        &mut self,
        state: PoisonArrowState,
    ) -> Option<PoisonArrowState> {
        self.periodic_attack_order.insert(state.skill_id());
        self.poison_arrow_state.replace(state)
    }

    pub(crate) fn take_poison_arrow_state_for_ai(&mut self) -> Option<PoisonArrowState> {
        self.poison_arrow_state.take()
    }

    pub(crate) fn replace_blood_loss_state(
        &mut self,
        state: BloodLossState,
    ) -> Option<BloodLossState> {
        self.periodic_attack_order.insert(state.skill_id());
        self.blood_loss_state.replace(state)
    }

    pub(crate) fn take_blood_loss_state_for_ai(&mut self) -> Option<BloodLossState> {
        self.blood_loss_state.take()
    }

    pub(crate) fn periodic_attack_state_ids(&self) -> Vec<u32> {
        self.periodic_attack_order.iter().copied().collect()
    }

    pub(crate) fn finish_periodic_attack_state(&mut self, skill_id: u32) {
        self.periodic_attack_order.shift_remove(&skill_id);
    }

    pub(crate) fn agility_state(&self, skill_id: u32) -> Option<AgilityState> {
        if skill_id == crate::gameserver::appserver::skills::agility::AGILITY_2_SKILL_ID {
            return self.agility_state_2;
        }
        match self.persistent_agility_family_state {
            Some(PersistentAgilityFamilyState::Agility(state)) if state.skill_id() == skill_id => {
                Some(state)
            }
            _ => None,
        }
    }

    pub(crate) fn take_agility_state(&mut self, skill_id: u32) -> Option<AgilityState> {
        if skill_id == crate::gameserver::appserver::skills::agility::AGILITY_2_SKILL_ID {
            return self.agility_state_2.take();
        }
        match self.persistent_agility_family_state {
            Some(PersistentAgilityFamilyState::Agility(state)) if state.skill_id() == skill_id => {
                self.persistent_agility_family_state.take();
                Some(state)
            }
            _ => None,
        }
    }

    pub(crate) fn begin_agility_state(&mut self, state: AgilityState) {
        match state.skill_id() {
            crate::gameserver::appserver::skills::agility::AGILITY_SKILL_ID => {
                debug_assert!(self.persistent_agility_family_state.is_none());
                self.persistent_agility_family_state =
                    Some(PersistentAgilityFamilyState::Agility(state));
            }
            crate::gameserver::appserver::skills::agility::AGILITY_2_SKILL_ID => {
                debug_assert!(self.agility_state_2.is_none());
                self.agility_state_2 = Some(state);
            }
            _ => unreachable!("каноническое состояние ловкости имеет известный ID"),
        }
    }

    pub(crate) fn persistent_agility_family_state(
        &self,
    ) -> Option<PersistentAgilityFamilyState> {
        self.persistent_agility_family_state
    }

    pub(crate) fn take_persistent_agility_family_state(
        &mut self,
    ) -> Option<PersistentAgilityFamilyState> {
        self.persistent_agility_family_state.take()
    }

    pub(crate) fn begin_persistent_agility_family_state(
        &mut self,
        state: PersistentAgilityFamilyState,
    ) {
        debug_assert!(PersistentAgilityFamilyState::is_known_skill(
            state.skill_id()
        ));
        debug_assert!(self.persistent_agility_family_state.is_none());
        self.persistent_agility_family_state = Some(state);
    }

    pub(crate) fn take_expired_agility_state_2(&mut self, now_ms: u32) -> Option<AgilityState> {
        self.agility_state_2.filter(|state| state.expired(now_ms))?;
        self.agility_state_2.take()
    }

    pub(crate) fn take_first_script_state(&mut self, state_id: i32) -> Option<ScriptMoveState> {
        let index = self
            .script_states
            .iter()
            .position(|state| state.state_id == state_id)?;
        Some(self.script_states.remove(index))
    }

    pub(crate) fn script_states(&self) -> &[ScriptMoveState] {
        &self.script_states
    }

    pub(crate) fn take_pending_script_state_visuals(&mut self) -> Vec<ScriptMoveState> {
        let mut pending = Vec::new();
        for state in &mut self.script_states {
            if state.visual_pending {
                state.visual_pending = false;
                pending.push(*state);
            }
        }
        pending
    }

    pub(crate) fn begin_ride_state(&mut self, mut state: RideState) -> Option<RideState> {
        if self.ride_state.is_some() {
            return None;
        }
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        state.append_serialized(&mut self.ex_states);
        self.set_fightable(false);
        self.ride_state = Some(state.clone());
        Some(state)
    }

    pub(crate) fn end_ride_state(&mut self) -> Option<RideState> {
        let state = self.ride_state.take()?;
        if let Some((offset, amount)) = state.serialized_span()
            && offset + amount <= self.ex_states.len()
        {
            self.ex_states.drain(offset..offset + amount);
            if self.ex_states.len() >= 4 {
                let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
                write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
            }
            for state in &mut self.extended_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.change_body_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            for state in &mut self.undead_states {
                state.shift_serialized_offset_after(offset, amount);
            }
        }
        self.set_fightable(true);
        Some(state)
    }

    pub(crate) fn activate_loaded_ride_state(&mut self) -> Option<RideState> {
        self.ride_state.as_ref()?;
        self.set_fightable(false);
        self.ride_state.clone()
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
        let now_ms = now_ms();
        let Some(mut state) = UndeadState::from_factory(state_id, factory, now_ms) else {
            return UndeadStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
                state_list_changed: false,
            };
        };
        let state_type = state.state_type;
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.undead_states.len() {
            if self.undead_states[index].state_type == state_type
                || self.undead_states[index].state_id == state_id
            {
                let removed_state = self.undead_states.remove(index);
                self.remove_undead_state_serialized(&removed_state);
                removed.push(removed_state);
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
        if self.ex_states.len() < 4 {
            self.ex_states.clear();
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        let offset = self.ex_states.len();
        self.ex_states
            .resize(offset + 4 + UNDEAD_STATE_PARAMETER_BYTES, 0);
        state.write_serialized(&mut self.ex_states, offset);
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
        self.remove_undead_state_serialized(&removed);
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

    fn remove_undead_state_serialized(&mut self, state: &UndeadState) {
        let Some((offset, amount)) = state.serialized_span() else {
            return;
        };
        if offset + amount > self.ex_states.len() {
            return;
        }
        self.ex_states.drain(offset..offset + amount);
        if self.ex_states.len() >= 4 {
            let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
            write_u32(&mut self.ex_states, 0, count.saturating_sub(1));
        }
        for state in &mut self.extended_states {
            state.shift_serialized_offset_after(offset, amount);
        }
        for state in &mut self.change_body_states {
            state.shift_serialized_offset_after(offset, amount);
        }
        for state in &mut self.undead_states {
            state.shift_serialized_offset_after(offset, amount);
        }
        if let Some(state) = &mut self.ride_state {
            state.shift_serialized_offset_after(offset, amount);
        }
    }

    pub(crate) fn activate_loaded_undead_states(&mut self, now_ms: u32) -> Vec<UndeadState> {
        let storage = &mut self.state_storage;
        for state in &mut storage.undead_states {
            state.started_ms = now_ms;
            state.last_item_tick_ms = now_ms;
            state.update_serialized_runtime(&mut storage.ex_states, now_ms);
        }
        storage.undead_states.clone()
    }

    pub(crate) fn undead_state_tick(
        &mut self,
        now_ms: u32,
        dead: bool,
    ) -> (Vec<u32>, Vec<(u32, u32, u32)>) {
        let mut ended = Vec::new();
        let mut item_due = Vec::new();
        for state in &mut self.undead_states {
            if state.expired(now_ms) || (dead && state.disappear_after_dead) {
                ended.push(state.state_id);
                continue;
            }
            if state.item_due(now_ms) {
                state.last_item_tick_ms = now_ms;
                item_due.push((state.state_id, state.item_index, state.item_amount));
            }
        }
        (ended, item_due)
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
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
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

    pub(crate) fn delete_extended_state_by_type(
        &mut self,
        state_type: u16,
    ) -> ExtendedStateMutation {
        let Some(index) = self.extended_states.iter().position(|state| {
            state.kind == ExtendedStateKind::Original && state.state_type == state_type
        }) else {
            return ExtendedStateMutation {
                removed: Vec::new(),
                added: None,
                legacy_return: 0,
            };
        };
        let removed = self.extended_states.remove(index);
        let legacy_return = removed.level;
        self.remove_extended_state_serialized(&removed);
        ExtendedStateMutation {
            removed: vec![removed],
            added: None,
            legacy_return,
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
            for state in &mut self.undead_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.ride_state {
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

    pub(crate) fn activate_loaded_extended_states(&mut self, now_ms: u32) -> Vec<ExtendedState> {
        let storage = &mut self.state_storage;
        for state in &mut storage.extended_states {
            state.activate_loaded(now_ms);
            state.update_serialized_runtime(&mut storage.ex_states, now_ms);
        }
        storage.extended_states.clone()
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
            LegacyWriter::new(&mut self.ex_states).write_u32(0);
        }
        let count = read_u32(&self.ex_states, 0).expect("счётчик состояний");
        write_u32(&mut self.ex_states, 0, count.wrapping_add(1));
        let offset = self.ex_states.len();
        LegacyWriter::new(&mut self.ex_states).write_u32(super::chbystate::CHANGE_BODY_STATE_ID);
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
            for state in &mut self.undead_states {
                state.shift_serialized_offset_after(offset, amount);
            }
            if let Some(state) = &mut self.ride_state {
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
        let storage = &mut self.state_storage;
        for state in &mut storage.change_body_states {
            state.activate_loaded(now_ms);
            state.update_serialized_runtime(&mut storage.ex_states, now_ms);
        }
        storage.change_body_states.clone()
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
        let storage = &mut self.state_storage;
        for state in &mut storage.change_body_states {
            if state.on_change_region() {
                ended.push(state.level);
            } else {
                state.update_serialized_runtime(&mut storage.ex_states, state.started_ms);
            }
        }
        ended
    }

    pub(crate) fn change_body_player_lost_end_ids(&mut self) -> Vec<u32> {
        let mut ended = Vec::new();
        let storage = &mut self.state_storage;
        for state in &mut storage.change_body_states {
            if state.on_player_lost() {
                ended.push(state.level);
            } else {
                state.update_serialized_runtime(&mut storage.ex_states, state.started_ms);
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

/// Возвращает только подтверждённые начала известных записей. Размер
/// неизвестного класса из wire не выводится, поэтому после него типизация
/// прекращается, а исходный хвост остаётся в `LegacyStateCodec` без изменений.
fn known_state_record_offsets(payload: &[u8]) -> Vec<usize> {
    let Some(declared_count) = read_u32(payload, 0) else {
        return Vec::new();
    };
    let mut offsets = Vec::new();
    let mut cursor = 4usize;
    for _ in 0..declared_count {
        let Some(state_id) = read_u32(payload, cursor) else {
            break;
        };
        let size = match state_id {
            CHANGE_BODY_STATE_ID => 124,
            EX_STATE_ID => 44,
            EX_STATE_NEW_ID => 56,
            UNDEAD_STATE_ID => 76,
            RIDE_STATE_ID => {
                let name_start = cursor.saturating_add(16);
                let Some(name) = payload.get(name_start..) else {
                    break;
                };
                let Some(length) = name.iter().take(256).position(|byte| *byte == 0) else {
                    break;
                };
                16 + length + 1
            }
            _ => break,
        };
        let Some(end) = cursor.checked_add(size).filter(|end| *end <= payload.len()) else {
            break;
        };
        offsets.push(cursor);
        cursor = end;
    }
    offsets
}

fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    LegacyReader::at(source, offset).ok()?.read_u16().ok()
}

fn read_i16(source: &[u8], offset: usize) -> Option<i16> {
    LegacyReader::at(source, offset).ok()?.read_i16().ok()
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

fn read_i32(source: &[u8], offset: usize) -> Option<i32> {
    LegacyReader::at(source, offset).ok()?.read_i32().ok()
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(destination, offset, value).expect("проверенное поле состояния");
}

fn write_i16(destination: &mut [u8], offset: usize, value: i16) {
    LegacyWriter::write_i16_at(destination, offset, value).expect("проверенное поле состояния");
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле состояния");
}

fn write_i32(destination: &mut [u8], offset: usize, value: i32) {
    LegacyWriter::write_i32_at(destination, offset, value).expect("проверенное поле состояния");
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
// STATUS: PARTIALLY_MATERIALIZED_KNOWN_STATE_OWNERS
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
