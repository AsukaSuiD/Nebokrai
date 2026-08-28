//! Область периодической атаки грома боевого духа `CThunderPhalanx`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderphalanx.cpp`. Владелец хранит подтверждённую маску
//! 7×7, строгие границы срока жизни и частоты, а также формулу элементального
//! урона. Три исходные таблицы уровней по адресам `0x006A4344/78/AC`
//! совпадают побайтно.
//! `Initialize` сохраняет исходные повторные пары RNG для каждой цели каждого
//! окна; одинаковая клетка может быть выбрана и обработана повторно. Снимок
//! намеренно передаёт только исходный префикс
//! `(m_dwLifeTime/m_dwFrequency)*m_dwNumTargets` из массива на 49 ячеек на
//! окно. Поиск сущностей и применение атаки остаются у исполняющего владельца.

use super::thunder::THUNDER_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

pub(crate) const THUNDER_SCOPE_SIDE: i32 = 7;
pub(crate) const THUNDER_SCOPE: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0,
    0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThunderPhalanxTick {
    Pending,
    Attack { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CThunderPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    frequency_ms: u32,
    minimum_attack: i32,
    maximum_attack: i32,
    _element_modifier: i32,
    _target_count: u32,
    _cch: i32,
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

impl CThunderPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        frequency_ms: u32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
        target_count: u32,
        cch: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE,
            id,
            ex_id: CGuid::GUID_INVALID,
        });
        Self {
            shape,
            master,
            started_at_ms,
            lifetime_ms,
            skill_level,
            frequency_ms: frequency_ms.max(1),
            minimum_attack,
            maximum_attack,
            _element_modifier: element_modifier,
            _target_count: target_count,
            _cch: cch,
            last_attack_ms: 0,
            attack_count: 0,
            cells: Vec::new(),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    pub(crate) fn tick(&mut self, now_ms: u32) -> ThunderPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ThunderPhalanxTick::Expired;
        }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now_ms {
            self.last_attack_ms = now_ms;
            self.attack_count = self.attack_count.wrapping_add(1);
            return ThunderPhalanxTick::Attack { sampled_at_ms: now_ms };
        }
        ThunderPhalanxTick::Pending
    }

    pub(crate) fn attack_cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let window = self.attack_count.wrapping_sub(1);
        let start = window.wrapping_mul(49) as usize;
        self.cells
            .get(start..start.saturating_add(49))
            .unwrap_or_default()
            .iter()
            .copied()
            .take_while(|cell| *cell != (0, 0))
    }

    pub(crate) fn initialize(
        &mut self,
        tile_x: i32,
        tile_y: i32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) {
        let attack_windows = self.lifetime_ms / self.frequency_ms;
        let total_cells = attack_windows.wrapping_mul(49) as usize;
        self.cells = vec![(0, 0); total_cells];
        let origin_x = tile_x.wrapping_sub(3);
        let origin_y = tile_y.wrapping_sub(3);
        for window in 0..attack_windows {
            for target in 0..self._target_count {
                let (x, y) = loop {
                    let x = random_below(THUNDER_SCOPE_SIDE);
                    let y = random_below(THUNDER_SCOPE_SIDE);
                    let index = x.wrapping_add(THUNDER_SCOPE_SIDE.wrapping_mul(y)) as usize;
                    if THUNDER_SCOPE.get(index).copied().unwrap_or_default() != 0 {
                        break (x, y);
                    }
                };
                let index = window
                    .wrapping_mul(49)
                    .wrapping_add(target) as usize;
                if let Some(cell) = self.cells.get_mut(index) {
                    *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
                }
            }
        }
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now {
            0
        } else {
            let second_now = now_milliseconds();
            self.lifetime_ms.wrapping_sub(second_now).wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(THUNDER_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.identity().object_type);
            writer.write_i32(self.shape.identity().id);
            writer.write_u32(remained);
            writer.write_u32(self.lifetime_ms);
            writer.write_u32(self.frequency_ms);
            let serialized_count =
                (self.lifetime_ms / self.frequency_ms).wrapping_mul(self._target_count);
            writer.write_u32(serialized_count);
            for &(x, y) in self.cells.iter().take(serialized_count as usize) {
                writer.write_i32(x);
                writer.write_i32(y);
            }
        }
        self.shape.encode_to_byte_array(&mut payload, true).then_some(payload)
    }

    #[allow(clippy::too_many_arguments, reason = "параметры сохраняют входы исходной формулы")]
    pub(crate) fn calculate_attack(
        &self,
        sprite: i32,
        combat: PlayerCombatProperties,
        occupation: u8,
        attacker_level: u8,
        target_damage_factor: u32,
        weapon_damage_factor: f32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) -> (AttackInformation, PlayerCombatProperties, u8, u8) {
        let constructor_delta = self.maximum_attack.wrapping_sub(self.minimum_attack);
        let constructor_width = constructor_delta.wrapping_abs().wrapping_add(1);
        let _discarded_constructor_roll = random_below(constructor_width);
        let base_damage = (f64::from(target_damage_factor) * f64::from(sprite) * 1.0e-6)
            .round_ties_even() as i32;
        let property_delta = self.maximum_attack.wrapping_sub(self.minimum_attack);
        let property_width = property_delta.wrapping_abs().wrapping_add(1);
        let damage = base_damage
            .wrapping_add(random_below(property_width))
            .wrapping_add(self.minimum_attack)
            .max(0);
        (
            AttackInformation {
                skill_id: THUNDER_SKILL_ID,
                skill_level: self.skill_level as u8,
                attacker_type: self.master.master_type,
                attacker_id: self.master.master_id,
                attacker_team_id: self.master.master_team_id,
                attacker_faction_id: self.master.master_guild_id,
                attacker_union_id: self.master.master_union_id,
                hit_modifier: 100,
                damage_factor: weapon_damage_factor,
                damage_modifier: 0,
                critical: false,
                blast_attack: false,
                full_miss: 0,
                damages: vec![AttackPower {
                    kind: AttackPowerType::Element,
                    hp_damage: damage,
                    mp_damage: 0,
                }],
            },
            combat,
            occupation,
            attacker_level,
        )
    }
}
