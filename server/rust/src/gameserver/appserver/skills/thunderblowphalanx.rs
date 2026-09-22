//! Стационарная форма громового удара `CThunderBlowPhalanx` (`0x13F`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderblowphalanx.cpp`. Форма до истечения срока жизни
//! проверяет собственную клетку в порядке регионального индекса, атакует все
//! допустимые цели текущего прохода и после него завершается. Формула сохраняет
//! два вызова генератора MSVCRT: диапазон урона, затем критический удар.
//! Критический множитель применяется в расширенной точности x87 и усекается к
//! нулю при записи результата в `i32`.

use super::fightdefense::truncate_original;
use super::thunderblow::THUNDER_BLOW_SKILL_ID;
use nebokrai_shared::protocol::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThunderBlowPhalanxTick {
    Scan { sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CThunderBlowPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
}


impl CThunderBlowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, skill_level, minimum_attack, maximum_attack, element_modifier }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }

    pub(crate) fn tick(&mut self, now_ms: u32) -> ThunderBlowPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.finish();
            ThunderBlowPhalanxTick::Expired
        } else {
            ThunderBlowPhalanxTick::Scan { sampled_at_ms: now_ms }
        }
    }

    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 } else {
            self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms)
        };
        let mut payload = Vec::new();
        {
            let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(THUNDER_BLOW_SKILL_ID as i32);
            writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.identity().object_type);
            writer.write_i32(self.shape.identity().id);
            writer.write_u32(remained);
        }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_owned_thunder_blow_attack(
    game: &mut CGame,
    phalanx: &CThunderBlowPhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master.master_id)?;
    let mut combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let damage_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor);
    let width_delta = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let damage = phalanx.element_modifier
        .wrapping_mul(combat.element_modify).wrapping_div(100)
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(phalanx.minimum_attack).max(0);
    let mut attack = AttackInformation {
        skill_id: THUNDER_BLOW_SKILL_ID,
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
        let rate = game.globe_setup().critical_rate();
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
