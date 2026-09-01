//! Движущийся огненный шар `CFireBallPhalanx` (`0x13D`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fireballphalanx.cpp`. Снаряд принадлежит региону,
//! проходит путь по одной клетке через заданный интервал и в каждой достигнутой
//! клетке с блоком `3` обходит подтверждённую маску 3×3 в порядке X→Y.
//! Боевой дух клетки обрабатывается перед обычными фигурами. Формула хранится
//! здесь и сохраняет два вызова генератора MSVCRT на каждую рассчитанную атаку;
//! усиление душами и критический множитель усекаются к нулю.

use super::fireball::FIRE_BALL_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireBallPhalanxTick {
    Pending,
    Active { force_move: Option<(i32, i32, u32)>, scan: Option<(i32, i32, u32)> },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireBallPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    soul_count: i32,
    soul_variable: u32,
    current_position: usize,
    force_moved: bool,
}

pub(crate) fn fire_ball_targets(game: &CGame, region_id: i32, phalanx: &CFireBallPhalanx, center_x: i32, center_y: i32) -> Vec<(ShapeIdentity, bool)> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut targets = Vec::new();
    let mut ordinary = Vec::new();
    for (tile_x, tile_y) in CFireBallPhalanx::scope_cells(center_x, center_y) {
        for (&player_id, _) in &region.war_souls_at(tile_x, tile_y) {
            let player_id = player_id as i32;
            if player_id != phalanx.master().master_id && game.find_player(player_id).is_some_and(|player| !player.is_dead()) && game.player_base_attackable(phalanx.master().master_id, player_id) {
                targets.push((ShapeIdentity { object_type: 400, id: player_id, ex_id: CGuid::GUID_INVALID }, true));
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

impl CFireBallPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, path: Vec<(i32, i32)>, speed_ms: u32,
        soul_count: i32, soul_variable: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self {
            shape, master, started_at_ms, lifetime_ms, skill_level, minimum_attack,
            maximum_attack, element_modifier, path, speed_ms, soul_count,
            soul_variable, current_position: 0, force_moved: false,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }

    pub(crate) fn tick(&mut self, now_ms: u32) -> FireBallPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms || self.path.is_empty() {
            self.finish();
            return FireBallPhalanxTick::Expired;
        }
        if self.current_position >= self.path.len() {
            self.finish();
            return FireBallPhalanxTick::Expired;
        }
        let force_move = if self.force_moved {
            None
        } else {
            self.force_moved = true;
            let &(x, y) = self.path.last().expect("непустой путь проверен выше");
            Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
        };
        let due = self.started_at_ms
            .wrapping_add((self.current_position as u32).wrapping_mul(self.speed_ms)) <= now_ms;
        let scan = due.then(|| {
            let (x, y) = self.path[self.current_position];
            self.current_position = self.current_position.wrapping_add(1);
            (x, y, now_ms)
        });
        if force_move.is_some() || scan.is_some() {
            FireBallPhalanxTick::Active { force_move, scan }
        } else {
            FireBallPhalanxTick::Pending
        }
    }

    /// `g_dwLength/g_dwHeight == 3`, а все девять байтов `g_bScope` равны
    /// единице. Порядок циклов исходного owner-а: сначала X, затем Y.
    pub(crate) fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        (0..3).flat_map(move |x| (0..3).map(move |y| {
            (center_x.wrapping_add(x - 1), center_y.wrapping_add(y - 1))
        }))
    }

    pub(crate) fn encode_client_snapshot(&self, now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            FIRE_BALL_SKILL_ID as i32,
            self.skill_level,
            self.master.master_type,
            self.master.master_id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }
}

pub(crate) fn calculate_owned_fire_ball_attack(
    game: &mut CGame,
    phalanx: &CFireBallPhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master.master_id)?;
    let mut combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let (weapon_divisor, weapon_minimum) = game.globe_setup().weapon_damage_factors();
    let damage_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), weapon_divisor, weapon_minimum);
    let width_delta = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let mut damage = phalanx.element_modifier
        .wrapping_mul(combat.element_modify).wrapping_div(100)
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(phalanx.minimum_attack);
    if phalanx.soul_count != 0 && phalanx.soul_variable != 0 {
        damage = ((phalanx.soul_variable as f32 * phalanx.soul_count as f32 * 0.01 + 1.0)
            * damage as f32) as i32;
    }
    damage = damage.max(0);
    let mut attack = AttackInformation {
        skill_id: FIRE_BALL_SKILL_ID,
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
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * critical_rate) as i32;
        }
    }
    let [blast_attack, blast_defense, element_blast_attack, element_blast_defense, full_miss] = game.globe_setup().base_combat_scales();
    if combat.blast_attack_scale() < 1.0 { combat.blast_attack_scale_bits = blast_attack.max(1.0).to_bits(); }
    if combat.blast_defense_scale() < 0.01 { combat.blast_defense_scale_bits = blast_defense.max(0.01).to_bits(); }
    if combat.element_blast_attack_scale() < 1.0 { combat.element_blast_attack_scale_bits = element_blast_attack.max(1.0).to_bits(); }
    if combat.element_blast_defense_scale() < 0.01 { combat.element_blast_defense_scale_bits = element_blast_defense.max(0.01).to_bits(); }
    if combat.full_miss_scale() < 0.01 { combat.full_miss_scale_bits = full_miss.max(0.01).to_bits(); }
    if combat.critical_rate() < 1.0 { combat.critical_rate_bits = game.globe_setup().critical_rate().max(1.0).to_bits(); }
    Some((attack, combat, occupation, attacker_level))
}
