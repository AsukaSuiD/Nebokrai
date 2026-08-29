//! Периодическая область снежной бури `CSnowStormPhalanx`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/snowstormphalanx.cpp`. Все три подтверждённые таблицы
//! по адресам `0x006A5270/8C/A8` имеют размер 5×5 и целиком заполнены единицами.
//! Владелец заранее расходует по два вызова legacy RNG на каждую цель каждого
//! окна, хранит FIFO окон и формирует точный клиентский снимок. Поиск сущностей
//! и применение атаки к независимым владельцам остаются у `CGame`.

use super::snowstorm::SNOW_STORM_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const SCOPE_SIDE: i32 = 5;
const SCOPE_AREA: u32 = 25;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SnowStormPhalanxTick { Pending, Attack { sampled_at_ms: u32 }, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSnowStormPhalanx {
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
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

pub(crate) fn snow_storm_targets(
    game: &CGame,
    region_id: i32,
    phalanx: &CSnowStormPhalanx,
) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else {
        return Vec::new();
    };
    let (area_width, area_height) = game.area_dimensions();
    let mut targets = Vec::new();
    for (x, y) in phalanx.current_cells() {
        let mut shapes = Vec::new();
        if region
            .get_shapes(x, y, area_width, area_height, game, &mut shapes)
            .is_err()
        {
            continue;
        }
        for shape in shapes {
            if shape.identity == phalanx.shape().identity()
                || (shape.identity.object_type == phalanx.master().master_type
                    && shape.identity.id == phalanx.master().master_id)
                || !matches!(shape.identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
            {
                continue;
            }
            targets.push(shape.identity);
        }
    }
    targets
}

pub(crate) fn calculate_owned_snow_storm_attack(
    game: &mut CGame,
    phalanx: &CSnowStormPhalanx,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    let (combat, occupation, level) = game.find_player(master.master_id).map(|player| {
        (
            player.combat_properties(),
            player.occupation(),
            player.level(),
        )
    })?;
    Some(phalanx.calculate_attack(combat, occupation, level, &mut |maximum| {
        game.skill_random_below(maximum)
    }))
}

impl CSnowStormPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, frequency_ms: u32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32, target_count: u32) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, skill_level, frequency_ms, minimum_attack, maximum_attack, element_modifier, target_count, last_attack_ms: 0, attack_count: 0, cells: Vec::new() }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    pub(crate) fn initialize(&mut self, tile_x: i32, tile_y: i32, random_below: &mut dyn FnMut(i32) -> i32) -> bool {
        if self.frequency_ms == 0 { return false; }
        let windows = self.lifetime_ms / self.frequency_ms;
        self.cells = vec![(0, 0); windows.wrapping_mul(SCOPE_AREA) as usize];
        let origin_x = tile_x.wrapping_sub(SCOPE_SIDE >> 1);
        let origin_y = tile_y.wrapping_sub(SCOPE_SIDE >> 1);
        for window in 0..windows {
            for target in 0..self.target_count {
                let x = random_below(SCOPE_SIDE);
                let y = random_below(SCOPE_SIDE);
                let index = window.wrapping_mul(SCOPE_AREA).wrapping_add(target) as usize;
                if let Some(cell) = self.cells.get_mut(index) {
                    *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
                }
            }
        }
        true
    }

    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, mut now_milliseconds: impl FnMut() -> u32) -> SnowStormPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return SnowStormPhalanxTick::Expired;
        }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now_milliseconds() {
            let sampled_at_ms = now_milliseconds();
            self.last_attack_ms = sampled_at_ms;
            self.attack_count = self.attack_count.wrapping_add(1);
            return SnowStormPhalanxTick::Attack { sampled_at_ms };
        }
        SnowStormPhalanxTick::Pending
    }

    pub(crate) fn current_cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let start = self.attack_count.wrapping_sub(1).wrapping_mul(SCOPE_AREA) as usize;
        self.cells.get(start..start.saturating_add(SCOPE_AREA as usize)).unwrap_or_default().iter().copied().take_while(|cell| *cell != (0, 0))
    }

    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 } else { let second_now = now_milliseconds(); self.lifetime_ms.wrapping_sub(second_now).wrapping_add(self.started_at_ms) };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(SNOW_STORM_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.get_tile_x().ok()?);
            writer.write_i32(self.shape.get_tile_y().ok()?);
            writer.write_u32(remained);
            writer.write_u32(self.lifetime_ms);
            writer.write_u32(self.frequency_ms);
            writer.write_u32(self.cells.len() as u32);
            for &(x, y) in &self.cells { writer.write_i32(x); writer.write_i32(y); }
        }
        self.shape.encode_to_byte_array(&mut payload, true).then_some(payload)
    }

    pub(crate) fn calculate_attack(&self, combat: PlayerCombatProperties, occupation: u8, attacker_level: u8, random_below: &mut dyn FnMut(i32) -> i32) -> (AttackInformation, PlayerCombatProperties, u8, u8) {
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let damage = self.minimum_attack.wrapping_add(random_below(width)).wrapping_add(self.element_modifier).max(0);
        (AttackInformation { skill_id: SNOW_STORM_SKILL_ID, skill_level: self.skill_level as u8, attacker_type: self.master.master_type, attacker_id: self.master.master_id, attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id, attacker_union_id: self.master.master_union_id, hit_modifier: 100, damage_factor: 1.0, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }] }, combat, occupation, attacker_level)
    }
}
