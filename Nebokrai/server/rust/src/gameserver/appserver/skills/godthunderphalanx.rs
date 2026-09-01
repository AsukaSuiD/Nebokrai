//! Периодическая область божественного грома `CGodThunderPhalanx` (`0x140`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godthunderphalanx.cpp`. Три таблицы уровней по адресам
//! `0x006A509C/0x006A50A8/0x006A50B4` совпадают: это маска 3×3 из единиц.
//! `Initialize` заранее расходует два значения MSVCRT RNG на каждую цель каждого
//! окна и допускает повтор клетки. AI читает только текущее окно с шагом девять;
//! формула затем расходует RNG на урон и критический удар для каждой цели.

use super::godthunder::GOD_THUNDER_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const SCOPE_AREA: u32 = 9;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodThunderPhalanxTick { Pending, Attack { sampled_at_ms: u32 }, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodThunderPhalanx {
    shape: CShape,
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
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

pub(crate) fn god_thunder_targets(game: &CGame, region_id: i32, phalanx: &CGodThunderPhalanx) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut targets = Vec::new();
    for (x, y) in phalanx.attack_cells() {
        let mut shapes = Vec::new();
        if region.get_shapes(x, y, area_width, area_height, game, &mut shapes).is_err() { continue }
        for shape in shapes {
            let identity = shape.identity;
            if identity != phalanx.shape().identity()
                && !(identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
                && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                && game.owned_player_skill_target_attackable(phalanx.master(), identity, region_id)
            {
                targets.push(identity);
            }
        }
    }
    targets
}

pub(crate) fn calculate_owned_god_thunder_attack(game: &mut CGame, phalanx: &CGodThunderPhalanx, target_level: u8) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master().master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let level = player.level();
    let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
    let factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum);
    let critical_rate = game.globe_setup().critical_rate();
    Some(phalanx.calculate_attack(combat, occupation, level, factor, critical_rate, &mut |maximum| game.skill_random_below(maximum)))
}

impl CGodThunderPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, minimum_attack: i32,
        maximum_attack: i32, element_modifier: i32, target_count: u32, cch: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self {
            shape, master, started_at_ms, lifetime_ms, skill_level,
            frequency_ms: frequency_ms.max(1), minimum_attack, maximum_attack,
            element_modifier, target_count, cch, last_attack_ms: 0,
            attack_count: 0, cells: Vec::new(),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }

    pub(crate) fn initialize(
        &mut self, tile_x: i32, tile_y: i32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) {
        let windows = self.lifetime_ms / self.frequency_ms;
        self.cells = vec![(0, 0); windows.wrapping_mul(SCOPE_AREA) as usize];
        let origin_x = tile_x.wrapping_sub(1);
        let origin_y = tile_y.wrapping_sub(1);
        for window in 0..windows {
            for target in 0..self.target_count {
                let x = random_below(3);
                let y = random_below(3);
                let index = window.wrapping_mul(SCOPE_AREA).wrapping_add(target) as usize;
                if let Some(cell) = self.cells.get_mut(index) {
                    *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
                }
            }
        }
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> GodThunderPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.finish();
            return GodThunderPhalanxTick::Expired;
        }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now_ms {
            self.last_attack_ms = now_ms;
            self.attack_count = self.attack_count.wrapping_add(1);
            return GodThunderPhalanxTick::Attack { sampled_at_ms: now_ms };
        }
        GodThunderPhalanxTick::Pending
    }

    pub(crate) fn attack_cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let window = self.attack_count.wrapping_sub(1);
        let start = window.wrapping_mul(SCOPE_AREA) as usize;
        self.cells
            .get(start..start.saturating_add(SCOPE_AREA as usize))
            .unwrap_or_default().iter().copied()
            .take_while(|cell| *cell != (0, 0))
    }

    pub(crate) fn encode_client_snapshot(&self, mut now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first = now();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first { 0 } else {
            self.lifetime_ms.wrapping_sub(now()).wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(GOD_THUNDER_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.identity().object_type);
            writer.write_i32(self.shape.identity().id);
            writer.write_u32(remained);
            writer.write_u32(self.lifetime_ms);
            writer.write_u32(self.frequency_ms);
            let serialized_count = (self.lifetime_ms / self.frequency_ms).wrapping_mul(self.target_count);
            writer.write_u32(serialized_count);
            for &(x, y) in self.cells.iter().take(serialized_count as usize) {
                writer.write_i32(x); writer.write_i32(y);
            }
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }

    pub(crate) fn calculate_attack(
        &self, combat: PlayerCombatProperties, occupation: u8, attacker_level: u8,
        weapon_damage_factor: f32, critical_rate: f32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) -> (AttackInformation, PlayerCombatProperties, u8, u8) {
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let damage = self.minimum_attack.wrapping_add(random_below(width)).wrapping_add(self.element_modifier).max(0);
        let mut attack = AttackInformation {
            skill_id: GOD_THUNDER_SKILL_ID, skill_level: self.skill_level as u8,
            attacker_type: self.master.master_type, attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id,
            attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id,
            hit_modifier: 100, damage_factor: weapon_damage_factor,
            damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
            damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
        };
        if random_below(100) < self.cch {
            attack.critical = true;
            for power in &mut attack.damages {
                power.hp_damage = (power.hp_damage as f32 * critical_rate).round_ties_even() as i32;
            }
        }
        (attack, combat, occupation, attacker_level)
    }
}
