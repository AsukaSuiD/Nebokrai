//! Периодическая область огненной стены `CFireWallPhalanx` (`0x134`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/firewallphalanx.cpp`. Три подтверждённые маски имеют вид
//! 1×1, креста 3×3 и полного квадрата 3×3. Новая стена вырезает пересечение
//! только из ранее созданных стен. Обход активных клеток идёт X→Y, цели
//! обрабатываются один раз за окно, а строгие lifetime/frequency границы
//! используют три последовательных чтения часов. Формула сохраняет ровно два
//! вызова legacy RNG: диапазон урона и критический удар.

use super::firewall::FIRE_WALL_SKILL_ID;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireWallPhalanxTick {
    Pending,
    Scan { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireWallPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    frequency_ms: u32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    critical_chance: i32,
    last_attack_ms: u32,
    length: i32,
    height: i32,
    scope: Vec<bool>,
}

pub(crate) fn fire_wall_targets(game: &CGame, region_id: i32, phalanx: &CFireWallPhalanx) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut targets = Vec::new();
    for (tile_x, tile_y) in phalanx.active_cells() {
        let mut shapes = Vec::new();
        if region.get_shapes(tile_x, tile_y, area_width, area_height, game, &mut shapes).is_err() { continue }
        for shape in shapes {
            let identity = shape.identity;
            if identity == phalanx.shape().identity()
                || (identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
                || !matches!(identity.object_type, 400 | 600)
                || targets.contains(&identity)
            { continue }
            targets.push(identity);
        }
    }
    targets
}

impl CFireWallPhalanx {
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
        critical_chance: i32,
    ) -> Self {
        let (length, height, scope): (i32, i32, &[bool]) = match skill_level {
            2 => (3, 3, &[false, true, false, true, true, true, false, true, false]),
            3 => (3, 3, &[true; 9]),
            _ => (1, 1, &[true]),
        };
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
            frequency_ms,
            minimum_attack,
            maximum_attack,
            element_modifier,
            critical_chance,
            last_attack_ms: 0,
            length,
            height,
            scope: scope.to_vec(),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    fn level_scope(level: i32) -> (i32, i32, &'static [bool]) {
        match level {
            2 => (3, 3, &[false, true, false, true, true, true, false, true, false]),
            3 => (3, 3, &[true; 9]),
            _ => (1, 1, &[true]),
        }
    }

    pub(crate) fn replace_affect_region(&mut self, level: i32, tile_x: i32, tile_y: i32) {
        let Ok(current_x) = self.shape.get_tile_x() else { return };
        let Ok(current_y) = self.shape.get_tile_y() else { return };
        let current_left = current_x.wrapping_sub(self.length >> 1);
        let current_top = current_y.wrapping_sub(self.height >> 1);
        let (new_length, new_height, new_scope) = Self::level_scope(level);
        let new_left = tile_x.wrapping_sub(new_length >> 1);
        let new_top = tile_y.wrapping_sub(new_height >> 1);
        for x in 0..self.length {
            for y in 0..self.height {
                let world_x = current_left.wrapping_add(x);
                let world_y = current_top.wrapping_add(y);
                let new_x = world_x.wrapping_sub(new_left);
                let new_y = world_y.wrapping_sub(new_top);
                if new_x < 0 || new_y < 0 || new_x >= new_length || new_y >= new_height {
                    continue;
                }
                let new_index = new_y.wrapping_mul(new_length).wrapping_add(new_x) as usize;
                if new_scope.get(new_index).copied().unwrap_or(false) {
                    let current_index = y.wrapping_mul(self.length).wrapping_add(x) as usize;
                    if let Some(cell) = self.scope.get_mut(current_index) {
                        *cell = false;
                    }
                }
            }
        }
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        mut now_milliseconds: impl FnMut() -> u32,
    ) -> FireWallPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms
            || !self.scope.iter().any(|cell| *cell)
        {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return FireWallPhalanxTick::Expired;
        }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now_milliseconds() {
            let attack_now_ms = now_milliseconds();
            self.last_attack_ms = attack_now_ms;
            return FireWallPhalanxTick::Scan { sampled_at_ms: attack_now_ms };
        }
        FireWallPhalanxTick::Pending
    }

    pub(crate) fn active_cells(&self) -> Vec<(i32, i32)> {
        let Ok(center_x) = self.shape.get_tile_x() else { return Vec::new() };
        let Ok(center_y) = self.shape.get_tile_y() else { return Vec::new() };
        let start_x = center_x.wrapping_sub(self.length >> 1);
        let start_y = center_y.wrapping_sub(self.height >> 1);
        let mut cells = Vec::new();
        for x in 0..self.length {
            for y in 0..self.height {
                let index = y.wrapping_mul(self.length).wrapping_add(x) as usize;
                if self.scope.get(index).copied().unwrap_or(false) {
                    cells.push((start_x.wrapping_add(x), start_y.wrapping_add(y)));
                }
            }
        }
        cells
    }

    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_owned_fire_wall_attack(
    game: &mut CGame,
    phalanx: &CFireWallPhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master.master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let weapon_level = player.equipment().get_goods(2).map_or(0, |goods| {
        goods.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1)
    });
    let (weapon_divisor, weapon_minimum) = game.globe_setup().weapon_damage_factors();
    let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let damage_factor = if weapon_divisor == 0.0 {
        1.0
    } else {
        (delta as f32 / weapon_divisor).min(1.0).max(weapon_minimum)
    };
    let width = phalanx
        .maximum_attack
        .wrapping_sub(phalanx.minimum_attack)
        .wrapping_abs()
        .wrapping_add(1);
    let damage = phalanx
        .minimum_attack
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(phalanx.element_modifier)
        .max(0);
    let mut attack = AttackInformation {
        skill_id: FIRE_WALL_SKILL_ID,
        skill_level: phalanx.skill_level as u8,
        attacker_type: phalanx.master.master_type,
        attacker_id: phalanx.master.master_id,
        attacker_team_id: phalanx.master.master_team_id,
        attacker_faction_id: phalanx.master.master_guild_id,
        attacker_union_id: phalanx.master.master_union_id,
        hit_modifier: 100,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: damage,
            mp_damage: 0,
        }],
    };
    if game.skill_random_below(100) < phalanx.critical_chance {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * critical_rate).round_ties_even() as i32;
        }
    }
    Some((attack, combat, occupation, attacker_level))
}
