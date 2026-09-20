//! Снежная буря: заранее выбранные клетки и элементальная атака.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstormphalanx.cpp.
//! Все уровни используют полную маску 5×5. Конструктор выделяет окна, а
//! Initialize после SetTile расходует X/Y RNG для каждой цели каждого окна.
//! AI сохраняет last-attack до обхода; счётчик окна увеличивает после callbacks.
//! Клетки не дедуплицируются, нулевая пара завершает текущее окно. Пакет
//! содержит skill/level/master type/id, срок, частоту и весь массив клеток.
//! Нулевая частота, число целей >25 и невозможный размер массива отклоняются
//! явно: исходные деление на ноль и выход за массив не воспроизводятся.

use super::snowstorm::SNOW_STORM_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;

const SCOPE_SIDE: i32 = 5;
pub(crate) const SNOW_STORM_SCOPE_AREA: u32 = 25;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SnowStormParametersError {
    ZeroFrequency,
    TooManyTargets,
    CellArrayTooLarge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SnowStormAttack {
    master: MasterInfo,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSnowStormPhalanx {
    shape: CShape,
    attack: SnowStormAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    target_count: u32,
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

impl CSnowStormPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля конструктора исходной области")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, minimum_attack: i32,
        maximum_attack: i32, element_modifier: i32, target_count: u32,
    ) -> Result<Self, SnowStormParametersError> {
        let windows = lifetime_ms.checked_div(frequency_ms)
            .ok_or(SnowStormParametersError::ZeroFrequency)?;
        if target_count > SNOW_STORM_SCOPE_AREA {
            return Err(SnowStormParametersError::TooManyTargets);
        }
        let count = windows.checked_mul(SNOW_STORM_SCOPE_AREA)
            .filter(|count| count.checked_mul(8).is_some())
            .ok_or(SnowStormParametersError::CellArrayTooLarge)?;
        let mut cells = Vec::new();
        cells.try_reserve_exact(count as usize)
            .map_err(|_| SnowStormParametersError::CellArrayTooLarge)?;
        cells.resize(count as usize, (0, 0));
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Ok(Self {
            shape,
            attack: SnowStormAttack { master, skill_level, minimum_attack, maximum_attack, element_modifier },
            started_at_ms, lifetime_ms, frequency_ms, target_count,
            last_attack_ms: 0, attack_count: 0, cells,
        })
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn attack_snapshot(&self) -> SnowStormAttack { self.attack }

    pub(crate) fn initialize(&mut self, random_below: &mut dyn FnMut(i32) -> i32) {
        let origin_x = self.shape.get_tile_x().unwrap_or(i32::MIN).wrapping_sub(SCOPE_SIDE >> 1);
        let origin_y = self.shape.get_tile_y().unwrap_or(i32::MIN).wrapping_sub(SCOPE_SIDE >> 1);
        self.cells.fill((0, 0));
        for window in self.cells.chunks_exact_mut(SNOW_STORM_SCOPE_AREA as usize) {
            for cell in window.iter_mut().take(self.target_count as usize) {
                let x = random_below(SCOPE_SIDE);
                let y = random_below(SCOPE_SIDE);
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
        let index = self.attack_count.wrapping_mul(SNOW_STORM_SCOPE_AREA).wrapping_add(index);
        // После исчерпания массива native читает за его концом. Не создаём
        // лишние клетки и не подменяем такой выход успешной атакой.
        self.cells.get(index as usize).copied().filter(|cell| *cell != (0, 0))
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        let mut writer = LegacyWriter::new(&mut payload);
        writer.write_u32(SNOW_STORM_SKILL_ID);
        writer.write_i32(self.attack.skill_level);
        writer.write_i32(self.attack.master.master_type);
        writer.write_i32(self.attack.master.master_id);
        writer.write_u32(timed_client_state_time(self.started_at_ms, self.lifetime_ms, now));
        writer.write_u32(self.lifetime_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.cells.len() as u32);
        for &(x, y) in &self.cells { writer.write_i32(x); writer.write_i32(y); }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

impl SnowStormAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }
}

pub(crate) fn apply_snow_storm_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: SnowStormAttack, region: i32,
    target: ShapeIdentity, runtime: &mut Runtime,
) {
    if game.move_shape_health(region, target).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    attack.skill_id = SNOW_STORM_SKILL_ID;
    attack.skill_level = snapshot.skill_level as u8;
    attack.damage_modifier = 0;
    attack.damage_factor = 1.0;
    attack.hit_modifier = 100;
    let width = snapshot.maximum_attack.wrapping_sub(snapshot.minimum_attack)
        .wrapping_abs().wrapping_add(1);
    let damage = game.skill_random_below(width).wrapping_add(snapshot.minimum_attack)
        .wrapping_add(snapshot.element_modifier).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
    game.apply_owned_skill_contact(master, target, region, attack, runtime);
}
