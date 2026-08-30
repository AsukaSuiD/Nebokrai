//! Движущаяся область сферы хаоса `CChaosSpherePhalanx` (`0x137`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/chaosspherephalanx.cpp`. Владелец хранит путь, строгие
//! границы lifetime/frequency/speed и квадратную маску 3×3. Видимая фигура
//! одним `ForceMove` направляется в конец пути, а серверный центр поражения
//! продвигается по одной клетке за speed-интервал. Обход клеток идёт X→Y;
//! цели боевого духа предшествуют обычным фигурам каждой клетки. Формула сохраняет
//! ровно два вызова legacy RNG на каждую рассчитанную атаку.

use super::chaossphere::CHAOS_SPHERE_SKILL_ID;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChaosSpherePhalanxTick {
    Pending,
    Active {
        force_move: Option<(i32, i32, u32)>,
        scan: Option<(i32, i32, u32)>,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CChaosSpherePhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    frequency_ms: u32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    critical_chance: i32,
    last_attack_ms: u32,
    last_move_ms: u32,
    current_cell: usize,
    force_moved: bool,
}

pub(crate) fn chaos_sphere_targets(game: &CGame, region_id: i32, phalanx: &CChaosSpherePhalanx, center_x: i32, center_y: i32) -> Vec<(ShapeIdentity, bool)> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut targets = Vec::new();
    let mut ordinary = Vec::new();
    for (tile_x, tile_y) in CChaosSpherePhalanx::scope_cells(center_x, center_y) {
        if region.block_at(tile_x, tile_y) != Some(2) {
            for (&player_id, _) in &region.war_souls_at(tile_x, tile_y) {
                let player_id = player_id as i32;
                if player_id != phalanx.master().master_id && game.find_player(player_id).is_some_and(|player| !player.is_dead()) && game.player_base_attackable(phalanx.master().master_id, player_id) {
                    targets.push((ShapeIdentity { object_type: 400, id: player_id, ex_id: CGuid::GUID_INVALID }, true));
                }
            }
        }
        let mut shapes = Vec::new();
        if region.get_shapes(tile_x, tile_y, area_width, area_height, game, &mut shapes).is_err() { continue }
        for shape in shapes {
            let identity = shape.identity;
            if identity == phalanx.shape().identity() || (identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id) || !matches!(identity.object_type, 400 | 600) || ordinary.contains(&identity) { continue }
            if phalanx.master().master_type == 400 && identity.object_type == 400 && !game.player_base_attackable(phalanx.master().master_id, identity.id) { continue }
            ordinary.push(identity);
            targets.push((identity, false));
        }
    }
    targets
}

impl CChaosSpherePhalanx {
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
        path: Vec<(i32, i32)>,
        speed_ms: u32,
        critical_chance: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self {
            shape, master, started_at_ms, lifetime_ms, skill_level, frequency_ms,
            minimum_attack, maximum_attack, element_modifier, path, speed_ms,
            critical_chance, last_attack_ms: 0, last_move_ms: 0, current_cell: 0,
            force_moved: false,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, mut now_milliseconds: impl FnMut() -> u32) -> ChaosSpherePhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ChaosSpherePhalanxTick::Expired;
        }
        let Some(&(destination_x, destination_y)) = self.path.last() else { return ChaosSpherePhalanxTick::Pending };
        let force_move = if !self.force_moved {
            self.force_moved = true;
            self.last_move_ms = now_milliseconds();
            Some((destination_x, destination_y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
        } else { None };
        if self.speed_ms.wrapping_add(self.last_move_ms) < now_milliseconds() {
            self.last_move_ms = now_milliseconds();
            if self.current_cell < self.path.len().saturating_sub(1) { self.current_cell += 1; }
        }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now_milliseconds() {
            self.last_attack_ms = now_milliseconds();
            let (center_x, center_y) = self.path[self.current_cell];
            return ChaosSpherePhalanxTick::Active {
                force_move,
                scan: Some((center_x, center_y, self.last_attack_ms)),
            };
        }
        if force_move.is_some() {
            ChaosSpherePhalanxTick::Active { force_move, scan: None }
        } else {
            ChaosSpherePhalanxTick::Pending
        }
    }

    pub(crate) fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        (0..3).flat_map(move |x| (0..3).map(move |y| (center_x.wrapping_add(x - 1), center_y.wrapping_add(y - 1))))
    }

    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 } else {
            self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(CHAOS_SPHERE_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.identity().object_type);
            writer.write_i32(self.shape.identity().id);
            writer.write_u32(remained);
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_owned_chaos_sphere_attack(
    game: &mut CGame,
    phalanx: &CChaosSpherePhalanx,
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
    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack).wrapping_abs().wrapping_add(1);
    let damage = phalanx.minimum_attack.wrapping_add(game.skill_random_below(width)).wrapping_add(phalanx.element_modifier).max(0);
    let mut attack = AttackInformation {
        skill_id: CHAOS_SPHERE_SKILL_ID,
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
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
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
