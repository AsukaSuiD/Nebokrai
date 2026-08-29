//! Однократная область инь-ян `CYinYangPhalanx` (`0x139`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/yinyangphalanx.cpp`. Три маски по адресам
//! `0x006A554C/0x006A5558/0x006A5564` равны полному квадрату 3×3. Второй
//! вариант использует собственную маску 1×1. Новая область вырезает пересечение только из
//! ранее созданных областей. После строгого истечения lifetime активные клетки
//! обходятся X→Y, цели обрабатываются один раз в порядке первого появления,
//! затем область удаляется. Формула сохраняет два вызова legacy RNG: диапазон
//! урона и критический удар.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum YinYangPhalanxTick { Pending, AttackAndExpire { sampled_at_ms: u32 } }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CYinYangPhalanx {
    skill_id: u32,
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    critical_chance: i32,
    length: i32,
    height: i32,
    scope: Vec<bool>,
}

pub(crate) fn yin_yang_targets(game: &CGame, region_id: i32, phalanx: &CYinYangPhalanx) -> Vec<ShapeIdentity> {
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
                || !matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                || targets.contains(&identity)
                || !game.owned_player_skill_target_attackable(phalanx.master(), identity, region_id)
            { continue }
            targets.push(identity);
        }
    }
    targets
}

pub(crate) fn calculate_owned_yin_yang_attack(game: &mut CGame, phalanx: &CYinYangPhalanx, target_level: u8) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master().master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let weapon_level = player.equipment().get_goods(2).map_or(0, |goods| goods.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1));
    let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
    let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let damage_factor = if divisor == 0.0 { 1.0 } else { (delta as f32 / divisor).min(1.0).max(minimum) };
    let critical_rate = game.globe_setup().critical_rate();
    Some(phalanx.calculate_attack(damage_factor, combat, occupation, attacker_level, critical_rate, &mut |maximum| game.skill_random_below(maximum)))
}

impl CYinYangPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new_for_skill(skill_id: u32, id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32, critical_chance: i32) -> Self {
        let (length, height, scope): (i32, i32, &[bool]) = Self::level_scope(skill_id);
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { skill_id, shape, master, started_at_ms, lifetime_ms, skill_level, minimum_attack, maximum_attack, element_modifier, critical_chance, length, height, scope: scope.to_vec() }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn skill_id(&self) -> u32 { self.skill_id }

    fn level_scope(skill_id: u32) -> (i32, i32, &'static [bool]) {
        if skill_id == super::yinyang2::YIN_YANG_2_SKILL_ID {
            super::yinyangphalanx2::YIN_YANG_2_SCOPE
        } else {
            (3, 3, &[true; 9])
        }
    }

    pub(crate) fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) {
        let Ok(current_x) = self.shape.get_tile_x() else { return };
        let Ok(current_y) = self.shape.get_tile_y() else { return };
        let current_left = current_x.wrapping_sub(self.length >> 1);
        let current_top = current_y.wrapping_sub(self.height >> 1);
        let (new_length, new_height, new_scope) = Self::level_scope(self.skill_id);
        let new_left = tile_x.wrapping_sub(new_length >> 1);
        let new_top = tile_y.wrapping_sub(new_height >> 1);
        for x in 0..self.length {
            for y in 0..self.height {
                let new_x = current_left.wrapping_add(x).wrapping_sub(new_left);
                let new_y = current_top.wrapping_add(y).wrapping_sub(new_top);
                if new_x < 0 || new_y < 0 || new_x >= new_length || new_y >= new_height { continue; }
                let new_index = new_y.wrapping_mul(new_length).wrapping_add(new_x) as usize;
                if new_scope.get(new_index).copied().unwrap_or(false) {
                    let current_index = y.wrapping_mul(self.length).wrapping_add(x) as usize;
                    if let Some(cell) = self.scope.get_mut(current_index) { *cell = false; }
                }
            }
        }
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> YinYangPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            YinYangPhalanxTick::AttackAndExpire { sampled_at_ms: now_ms }
        } else { YinYangPhalanxTick::Pending }
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
                if self.scope.get(index).copied().unwrap_or(false) { cells.push((start_x.wrapping_add(x), start_y.wrapping_add(y))); }
            }
        }
        cells
    }

    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape.encode_to_byte_array(&mut payload, true).then_some(payload)
    }

    pub(crate) fn calculate_attack(&self, damage_factor: f32, combat: PlayerCombatProperties, occupation: u8, attacker_level: u8, critical_rate: f32, random_below: &mut dyn FnMut(i32) -> i32) -> (AttackInformation, PlayerCombatProperties, u8, u8) {
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let damage = self.minimum_attack.wrapping_add(random_below(width)).wrapping_add(self.element_modifier).max(0);
        let mut attack = AttackInformation { skill_id: self.skill_id, skill_level: self.skill_level as u8, attacker_type: self.master.master_type, attacker_id: self.master.master_id, attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id, attacker_union_id: self.master.master_union_id, hit_modifier: 100, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }] };
        if random_below(100) < self.critical_chance {
            attack.critical = true;
            for power in &mut attack.damages {
                if matches!(power.kind, AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul) { power.hp_damage = (power.hp_damage as f32 * critical_rate).round_ties_even() as i32; }
            }
        }
        (attack, combat, occupation, attacker_level)
    }
}
