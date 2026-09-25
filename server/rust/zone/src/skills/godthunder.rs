//! Окна целей, клиентские поля и живая форма областей
//! CGodThunderPhalanx/CGodThunderPhalanx2.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.
//! Конструкторы VA 0x005F5B50/0x005EF190, Initialize 0x005F59C0/0x005EEF30,
//! общий AddToByteArray 0x005EF0C0, AI 0x005F6150/0x005EF6B0,
//! Summon 0x00573840/0x00553A80 (appserver/skills/godthunder{,2}.cpp).
//! Композит `CGodThunderPhalanx` (CShape + область) перенесён из старого
//! адаптера буквально порцией замыкания; новых машинных оснований он не
//! добавляет. Для server decode VA 0x005F5D90 подтверждённого вызывающего
//! пути оригинала нет (UNKNOWN), decoder не переносится.

use nebokrai_shared::protocol::LegacyWriter;
use nebokrai_shared::values::CGuid;
use crate::combat::MasterInfo;
use crate::effects::timed_client_state_time;
use crate::regions::ShapeIdentity;
use crate::regions::shape::CShape;
use super::summonshape::SUMMON_SHAPE_TYPE;
use super::{ElementPhalanxAttack, ElementSummonLiveField};

pub const GOD_THUNDER_SKILL_ID: u32 = 0x140;
pub const GOD_THUNDER_2_SKILL_ID: u32 = 0x143;
pub const ROUNDED_THUNDER_SCOPE_SIDE: i32 = 7;
pub const ROUNDED_THUNDER_SCOPE: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0,
    0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
];

const TARGET_COUNT_PROPERTY: u32 = 20_010;
const MAXIMUM_ATTACK_PROPERTY: u32 = 20_009;
const MINIMUM_ATTACK_PROPERTY: u32 = 20_008;
const FREQUENCY_PROPERTY: u32 = 6_001;
const LIFETIME_PROPERTY: u32 = 30_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GodThunderSummonParameters {
    pub skill_id: u32,
    pub skill_level: i32,
    pub critical_chance: i32,
    pub target_count: u32,
    pub element_attack: i32,
    pub maximum_attack: i32,
    pub minimum_attack: i32,
    pub frequency_ms: u32,
    pub lifetime_ms: u32,
}

impl GodThunderSummonParameters {
    /// После общего масштабирования элемента сохраняет порядок обоих Summon.
    pub fn read(
        skill_id: u32,
        mut query_property: impl FnMut(u32) -> u32,
        mut read_live: impl FnMut(ElementSummonLiveField) -> Option<i32>,
        current_level: impl FnOnce() -> Option<i32>,
        scaled_element: i32,
    ) -> Option<Self> {
        let critical_chance = (read_live(ElementSummonLiveField::CriticalChance)? as u16) as i32;
        let target_count = query_property(TARGET_COUNT_PROPERTY);
        let element_attack = read_live(ElementSummonLiveField::AddElementAttack)?
            .wrapping_add(scaled_element);
        let maximum_attack = query_property(MAXIMUM_ATTACK_PROPERTY) as i32;
        let minimum_attack = query_property(MINIMUM_ATTACK_PROPERTY) as i32;
        let frequency_ms = query_property(FREQUENCY_PROPERTY);
        let skill_level = current_level()?;
        let lifetime_ms = query_property(LIFETIME_PROPERTY);
        Some(Self { skill_id, skill_level, critical_chance, target_count, element_attack,
            maximum_attack, minimum_attack, frequency_ms, lifetime_ms })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GodThunderParametersError {
    UnknownSkill,
    ZeroFrequency,
    TooManyTargets,
    CellArrayTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GodThunderPhalanx {
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    target_count: u32,
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

impl GodThunderPhalanx {
    pub fn new(
        attack: ElementPhalanxAttack, started_at_ms: u32, lifetime_ms: u32,
        frequency_ms: u32, target_count: u32,
    ) -> Result<Self, GodThunderParametersError> {
        let area: u32 = match attack.skill_id {
            GOD_THUNDER_SKILL_ID => 9,
            GOD_THUNDER_2_SKILL_ID => 49,
            _ => return Err(GodThunderParametersError::UnknownSkill),
        };
        let windows = lifetime_ms.checked_div(frequency_ms)
            .ok_or(GodThunderParametersError::ZeroFrequency)?;
        if target_count > area { return Err(GodThunderParametersError::TooManyTargets); }
        let count = windows.checked_mul(area)
            .filter(|count| count.checked_mul(8).is_some())
            .ok_or(GodThunderParametersError::CellArrayTooLarge)?;
        let mut cells = Vec::new();
        cells.try_reserve_exact(count as usize)
            .map_err(|_| GodThunderParametersError::CellArrayTooLarge)?;
        cells.resize(count as usize, (0, 0));
        Ok(Self { attack, started_at_ms, lifetime_ms, frequency_ms, target_count,
            last_attack_ms: 0, attack_count: 0, cells })
    }

    pub const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub const fn has_war_soul_pass(&self) -> bool {
        self.attack.skill_id == GOD_THUNDER_2_SKILL_ID
    }
    pub const fn scope_area(&self) -> u32 {
        if self.has_war_soul_pass() { 49 } else { 9 }
    }

    pub fn initialize(&mut self, center: (i32, i32), random: &mut dyn FnMut(i32) -> i32) {
        let rounded = self.has_war_soul_pass();
        let side = if rounded { ROUNDED_THUNDER_SCOPE_SIDE } else { 3 };
        let area = self.scope_area() as usize;
        let origin_x = center.0.wrapping_sub(side >> 1);
        let origin_y = center.1.wrapping_sub(side >> 1);
        self.cells.fill((0, 0));
        for window in self.cells.chunks_exact_mut(area) {
            for cell in window.iter_mut().take(self.target_count as usize) {
                let (x, y) = loop {
                    let x = random(side);
                    let y = random(side);
                    let in_scope = !rounded || ROUNDED_THUNDER_SCOPE
                        .get(y.wrapping_mul(side).wrapping_add(x) as usize)
                        .copied().unwrap_or_default() != 0;
                    if in_scope { break (x, y); }
                };
                *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
            }
        }
    }

    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }
    pub fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub fn advance_attack_window(&mut self) {
        self.attack_count = self.attack_count.wrapping_add(1);
    }
    pub fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        let index = self.attack_count.wrapping_mul(self.scope_area()).wrapping_add(index);
        // После исчерпания окон native читает вне массива; не создаём фиктивную цель.
        self.cells.get(index as usize).copied()
    }

    /// Поля перед базовым CShape. Wire-count использует target_count,
    /// хотя шаг окна и длина хранимого массива используют полную площадь.
    pub fn write_client_snapshot_fields(
        &self, payload: &mut Vec<u8>, now: impl FnMut() -> u32,
    ) {
        let mut writer = LegacyWriter::new(payload);
        writer.write_u32(self.attack.skill_id);
        writer.write_i32(self.attack.skill_level);
        writer.write_i32(self.attack.master.master_type);
        writer.write_i32(self.attack.master.master_id);
        writer.write_u32(timed_client_state_time(self.started_at_ms, self.lifetime_ms, now));
        writer.write_u32(self.lifetime_ms);
        writer.write_u32(self.frequency_ms);
        let count = (self.lifetime_ms / self.frequency_ms).wrapping_mul(self.target_count);
        writer.write_u32(count);
        for &(x, y) in self.cells.iter().take(count as usize) {
            writer.write_i32(x);
            writer.write_i32(y);
        }
    }
}

/// Живая форма областей GodThunder/GodThunder2: связка CShape и области
/// окон. Состав и порядок соответствуют адаптеру старого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CGodThunderPhalanx {
    shape: CShape,
    area: GodThunderPhalanx,
}

impl CGodThunderPhalanx {
    pub fn new(
        id: i32, master: MasterInfo, started_at_ms: u32,
        parameters: GodThunderSummonParameters,
    ) -> Result<Self, GodThunderParametersError> {
        let area = GodThunderPhalanx::new(
            ElementPhalanxAttack {
                master, skill_id: parameters.skill_id, skill_level: parameters.skill_level,
                minimum: parameters.minimum_attack, maximum: parameters.maximum_attack,
                element: parameters.element_attack, critical_chance: parameters.critical_chance,
            },
            started_at_ms, parameters.lifetime_ms, parameters.frequency_ms,
            parameters.target_count,
        )?;
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Ok(Self { shape, area })
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.area.attack_snapshot().master }
    pub const fn attack_snapshot(&self) -> ElementPhalanxAttack {
        self.area.attack_snapshot()
    }
    pub const fn has_war_soul_pass(&self) -> bool { self.area.has_war_soul_pass() }
    pub const fn scope_area(&self) -> u32 { self.area.scope_area() }

    pub fn initialize(&mut self, random: &mut dyn FnMut(i32) -> i32) {
        let center = (self.shape.get_tile_x().unwrap_or(i32::MIN),
            self.shape.get_tile_y().unwrap_or(i32::MIN));
        self.area.initialize(center, random);
    }
    pub const fn expired_at(&self, now: u32) -> bool { self.area.expired_at(now) }
    pub const fn attack_due_at(&self, now: u32) -> bool { self.area.attack_due_at(now) }
    pub fn mark_attack_at(&mut self, now: u32) { self.area.mark_attack_at(now); }
    pub fn advance_attack_window(&mut self) { self.area.advance_attack_window(); }
    pub fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        self.area.current_cell(index)
    }

    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.area.write_client_snapshot_fields(&mut payload, now);
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}
