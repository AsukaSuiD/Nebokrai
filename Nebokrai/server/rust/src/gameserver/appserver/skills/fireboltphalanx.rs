//! Снаряд огненной стрелы `CFireBoltPhalanx` (`0x132`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fireboltphalanx.cpp`. Снаряд хранит снимок владельца,
//! цель, задержку полёта и параметры урона. Первое чтение часов проверяет срок
//! жизни, второе — строгую границу атаки; после единственной попытки объект
//! удаляется. Формула сохраняет два вызова генератора MSVCRT: диапазон урона,
//! затем критический удар. Снимок `CSoulCollectState`, потреблённый владельцем
//! навыка при создании снаряда, применяется здесь без обращения к live state;
//! результаты усиления душами и критического множителя усекаются к нулю.

use super::firebolt::FIRE_BOLT_SKILL_ID;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FireBoltPhalanxTick {
    Pending,
    Attack { target: ShapeIdentity, sampled_at_ms: u32 },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireBoltPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    target: ShapeIdentity,
    attack_delay_ms: u32,
    soul_count: i32,
    soul_variable: i32,
}

impl CFireBoltPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
        target: ShapeIdentity,
        attack_delay_ms: u32,
        soul_count: i32,
        soul_variable: i32,
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
            minimum_attack,
            maximum_attack,
            element_modifier,
            target,
            attack_delay_ms,
            soul_count,
            soul_variable,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }

    /// Точный клиентский `AddToByteArray` сведён в EXE по адресу `0x005fbd20`
    /// с тем же машинным телом, что у огненного шара; пара `long` здесь
    /// остаётся идентичностью цели, а параметры душ в снимок не входят.
    pub(crate) fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            FIRE_BOLT_SKILL_ID as i32,
            self.skill_level,
            self.target.object_type,
            self.target.id,
            self.started_at_ms,
            self.lifetime_ms,
            now_milliseconds,
        )
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        get_attack_now_ms: impl FnOnce() -> u32,
    ) -> FireBoltPhalanxTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return FireBoltPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if attack_now_ms.wrapping_sub(self.started_at_ms) > self.attack_delay_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return FireBoltPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        FireBoltPhalanxTick::Pending
    }
}

pub(crate) fn calculate_owned_fire_bolt_attack(
    game: &mut CGame,
    phalanx: &CFireBoltPhalanx,
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
    let mut damage_factor = if weapon_divisor == 0.0 {
        1.0
    } else {
        delta as f32 / weapon_divisor
    };
    damage_factor = damage_factor.min(1.0).max(weapon_minimum);

    let width_delta = phalanx.maximum_attack.wrapping_sub(phalanx.minimum_attack);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }
        .wrapping_add(1);
    let random_damage = game.skill_random_below(width);
    let mut damage = phalanx
        .element_modifier
        .wrapping_mul(combat.element_modify)
        .wrapping_div(100)
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(random_damage)
        .wrapping_add(phalanx.minimum_attack);
    if phalanx.soul_count != 0 && phalanx.soul_variable != 0 {
        damage = ((phalanx.soul_variable as f32 * phalanx.soul_count as f32 * 0.01 + 1.0)
            * damage as f32)
            as i32;
    }
    damage = damage.max(0);
    let mut attack = AttackInformation {
        skill_id: FIRE_BOLT_SKILL_ID,
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
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * critical_rate) as i32;
        }
    }

    let [blast_attack, blast_defense, element_blast_attack, element_blast_defense, full_miss] =
        game.globe_setup().base_combat_scales();
    let mut combat = combat;
    if combat.blast_attack_scale() < 1.0 { combat.blast_attack_scale_bits = blast_attack.max(1.0).to_bits(); }
    if combat.blast_defense_scale() < 0.01 { combat.blast_defense_scale_bits = blast_defense.max(0.01).to_bits(); }
    if combat.element_blast_attack_scale() < 1.0 { combat.element_blast_attack_scale_bits = element_blast_attack.max(1.0).to_bits(); }
    if combat.element_blast_defense_scale() < 0.01 { combat.element_blast_defense_scale_bits = element_blast_defense.max(0.01).to_bits(); }
    if combat.full_miss_scale() < 0.01 { combat.full_miss_scale_bits = full_miss.max(0.01).to_bits(); }
    if combat.critical_rate() < 1.0 { combat.critical_rate_bits = game.globe_setup().critical_rate().max(1.0).to_bits(); }
    Some((attack, combat, occupation, attacker_level))
}
