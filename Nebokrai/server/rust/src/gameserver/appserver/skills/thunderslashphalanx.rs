//! Стационарная форма громового рассечения `CThunderSlashPhalanx` (`0x72`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderslashphalanx.cpp`. Форма хранит снимок боевых
//! свойств владельца, соблюдает строгие границы частоты и срока жизни и
//! атакует только первый объект собственной клетки. Ошибка конструктора EXE
//! сохранена: оба края физического урона получают прежний максимум, но RNG
//! диапазона всё равно вызывается перед проверкой критического удара.
//! Критический множитель применяется в расширенной точности x87 и усекается к
//! нулю при записи результата в `i32`.

use super::fightdefense::truncate_original;
use super::thunderslash::THUNDER_SLASH_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThunderSlashPhalanxTick { Pending, Scan { sampled_at_ms: u32 }, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CThunderSlashPhalanx {
    shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
    tile_x: i32, tile_y: i32, frequency_ms: u32, last_attack_ms: u32,
    skill_level: i32, minimum_attack: i32, maximum_attack: i32,
    element_attack: i32, dexterity: i32, critical_chance: i32, soul_attack: i32,
}

pub(crate) fn thunder_slash_target(game: &CGame, region_id: i32, phalanx: &CThunderSlashPhalanx) -> Option<ShapeIdentity> {
    let region = game.find_region(region_id)?.base();
    let (x, y) = phalanx.tile();
    let (width, height) = game.area_dimensions();
    let first = region.get_shape(x, y, width, height, game).ok()??;
    matches!(first.identity.object_type, 400 | 600)
        .then_some(first.identity)
        .filter(|target| game.find_player(phalanx.master().master_id).is_none()
            || game.owned_player_skill_target_attackable(phalanx.master(), *target, region_id))
}

impl CThunderSlashPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, maximum_attack: i32,
        _minimum_attack: i32, element_attack: i32, dexterity: i32,
        critical_chance: i32, soul_attack: i32, tile_x: i32, tile_y: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, tile_x, tile_y, frequency_ms,
            last_attack_ms: 0, skill_level, minimum_attack: maximum_attack, maximum_attack,
            element_attack, dexterity, critical_chance, soul_attack }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn tile(&self) -> (i32, i32) { (self.tile_x, self.tile_y) }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }
    pub(crate) fn tick(&mut self, now_ms: u32, mut now_milliseconds: impl FnMut() -> u32) -> ThunderSlashPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms || self.tile_x == 0 || self.tile_y == 0 {
            self.finish(); return ThunderSlashPhalanxTick::Expired;
        }
        let frequency_now_ms = now_milliseconds();
        if self.last_attack_ms.wrapping_add(self.frequency_ms) >= frequency_now_ms { return ThunderSlashPhalanxTick::Pending; }
        let sampled_at_ms = now_milliseconds(); self.last_attack_ms = sampled_at_ms;
        ThunderSlashPhalanxTick::Scan { sampled_at_ms }
    }
    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 } else {
            self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        { let mut writer = LegacyWriter::new(&mut payload); writer.write_i32(self.tile_x);
          writer.write_i32(self.skill_level); writer.write_i32(self.tile_y);
          writer.write_i32(THUNDER_SLASH_SKILL_ID as i32); writer.write_u32(remained); }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_owned_thunder_slash_attack(
    game: &mut CGame, phalanx: &CThunderSlashPhalanx,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let properties = game.skill_base_properties(THUNDER_SLASH_SKILL_ID, phalanx.skill_level)?;
    let hit_modifier = properties.query_property(20_001) as i32;
    let damage_factor = properties.query_property(20_003) as f32 * 0.01;
    let mut combat = game.find_player(phalanx.master.master_id)
        .map_or_else(PlayerCombatProperties::default, |player| player.combat_properties());
    let occupation = game.find_player(phalanx.master.master_id).map_or(0, |player| player.occupation());
    let attacker_level = game.find_player(phalanx.master.master_id).map_or(0, |player| player.level());
    let width = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack).wrapping_abs().wrapping_add(1);
    let physical = phalanx.minimum_attack.wrapping_add(game.skill_random_below(width))
        .wrapping_add(if phalanx.master.master_type == 400 { phalanx.dexterity } else { 0 }).max(0);
    let mut attack = AttackInformation {
        skill_id: THUNDER_SLASH_SKILL_ID, skill_level: phalanx.skill_level as u8,
        attacker_type: phalanx.master.master_type, attacker_id: phalanx.master.master_id,
        attacker_team_id: phalanx.master.master_team_id, attacker_faction_id: phalanx.master.master_guild_id,
        attacker_union_id: phalanx.master.master_union_id,
        hit_modifier,
        damage_factor,
        damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: phalanx.element_attack.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: phalanx.soul_attack.max(0), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < phalanx.critical_chance {
        attack.critical = true; let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); }
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
