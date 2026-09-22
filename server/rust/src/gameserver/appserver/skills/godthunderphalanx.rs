//! Периодические области божественного грома 0x140/0x143.
//! Источник: gameserver.exe/GameServer.pdb, godthunderphalanx{,2}.cpp.
//! Первый вариант использует полную маску 3×3, второй — округлую 7×7 и
//! предварительный обход боевых духов. Конструктор выделяет окна; Initialize
//! после SetTile сохраняет X/Y RNG с повторами клеток. AI пишет last-attack
//! до callbacks, а номер окна — после них. Массив имеет шаг полной площади,
//! но wire передаёт его непрерывный префикс (life/frequency)*target_count.
//! Нулевую частоту, выход числа целей за окно и невозможное выделение памяти
//! отклоняем явно вместо исходного деления на ноль/выхода за массив.
//! Для server decode 0x005F5D90 подтверждённого вызывающего пути нет;
//! отдельный runtime API для него не создаётся.

use super::elementphalanxattack::ElementPhalanxAttack;
use super::godthunder::GOD_THUNDER_SKILL_ID;
use super::godthunder2::GOD_THUNDER_2_SKILL_ID;
use super::godthunderphalanx2::{GOD_THUNDER_2_SCOPE, GOD_THUNDER_2_SCOPE_SIDE};
use nebokrai_shared::protocol::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodThunderParametersError {
    UnknownSkill,
    ZeroFrequency,
    TooManyTargets,
    CellArrayTooLarge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodThunderPhalanx {
    shape: CShape,
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    target_count: u32,
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

impl CGodThunderPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля конструктора исходной области")]
    pub(crate) fn new_for_skill(
        skill_id: u32, id: i32, master: MasterInfo, started_at_ms: u32,
        lifetime_ms: u32, skill_level: i32, frequency_ms: u32,
        minimum: i32, maximum: i32, element: i32, target_count: u32,
        critical_chance: i32,
    ) -> Result<Self, GodThunderParametersError> {
        let area = match skill_id {
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
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Ok(Self {
            shape,
            attack: ElementPhalanxAttack {
                master, skill_id, skill_level, minimum, maximum, element, critical_chance,
            },
            started_at_ms, lifetime_ms, frequency_ms, target_count,
            last_attack_ms: 0, attack_count: 0, cells,
        })
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub(crate) const fn has_war_soul_pass(&self) -> bool {
        self.attack.skill_id == GOD_THUNDER_2_SKILL_ID
    }
    pub(crate) const fn scope_area(&self) -> u32 {
        if self.has_war_soul_pass() { 49 } else { 9 }
    }

    pub(crate) fn initialize(&mut self, random: &mut dyn FnMut(i32) -> i32) {
        let side = if self.has_war_soul_pass() { GOD_THUNDER_2_SCOPE_SIDE } else { 3 };
        let rounded = self.has_war_soul_pass();
        let area = self.scope_area() as usize;
        let origin_x = self.shape.get_tile_x().unwrap_or(i32::MIN).wrapping_sub(side >> 1);
        let origin_y = self.shape.get_tile_y().unwrap_or(i32::MIN).wrapping_sub(side >> 1);
        self.cells.fill((0, 0));
        for window in self.cells.chunks_exact_mut(area) {
            for cell in window.iter_mut().take(self.target_count as usize) {
                let (x, y) = loop {
                    let x = random(side);
                    let y = random(side);
                    if !rounded || GOD_THUNDER_2_SCOPE[(y * side + x) as usize] != 0 {
                        break (x, y);
                    }
                };
                *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
            }
        }
    }

    pub(crate) const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub(crate) const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }
    pub(crate) fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub(crate) fn advance_attack_window(&mut self) {
        self.attack_count = self.attack_count.wrapping_add(1);
    }
    pub(crate) fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        let index = self.attack_count.wrapping_mul(self.scope_area()).wrapping_add(index);
        // Native может исчерпать массив до срока; не воспроизводим чтение
        // чужой памяти и не создаём вместо него дополнительные клетки.
        self.cells.get(index as usize).copied()
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        let mut writer = LegacyWriter::new(&mut payload);
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
            writer.write_i32(x); writer.write_i32(y);
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}
