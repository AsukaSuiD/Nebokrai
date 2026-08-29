//! Владелец снаряда базовой магической атаки GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/basemagicphalanx.cpp`. Объект типа `1000` принадлежит
//! `CServerRegion`: первое чтение часов проверяет срок жизни строго через `>`,
//! второе отдельное чтение проверяет задержку атаки тем же строгим правилом.
//! После единственной попытки атаки объект отправляет exit и становится
//! `SHAPE_CHANGE_DELETE`; цель может исчезнуть без побочного эффекта. Формула,
//! wrapping и два исходных вызова RNG принадлежат этому owner-у; `CGame`
//! передаёт только снимки владельцев и применяет рассчитанную атаку к цели.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{
    CShape, SHAPE_CHANGE_DELETE, ShapeIdentity,
};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::{
    SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot,
};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseMagicPhalanxTick {
    Pending,
    Attack {
        target: ShapeIdentity,
        sampled_at_ms: u32,
    },
    Expired,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBaseMagicPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl CBaseMagicPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют constructor BaseMagicPhalanx")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        minimum_attack: i32,
        maximum_attack: i32,
        element_modifier: i32,
        attack_delay_ms: u32,
        target: ShapeIdentity,
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
            attack_delay_ms,
            target,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub(crate) const fn master(&self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn skill_level(&self) -> i32 {
        self.skill_level
    }

    pub(crate) const fn minimum_attack(&self) -> i32 {
        self.minimum_attack
    }

    pub(crate) const fn maximum_attack(&self) -> i32 {
        self.maximum_attack
    }

    pub(crate) const fn element_modifier(&self) -> i32 {
        self.element_modifier
    }

    pub(crate) const fn target(&self) -> ShapeIdentity {
        self.target
    }

    /// Точный клиентский `AddToByteArray` использует ту же сведённую линкером
    /// машинную функцию, что снаряд базовой стрельбы и атака боевой феи.
    pub(crate) fn encode_client_snapshot(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape,
            super::basemagic::BASE_MAGIC_SKILL_ID as i32,
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
    ) -> BaseMagicPhalanxTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return BaseMagicPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if attack_now_ms.wrapping_sub(self.started_at_ms) > self.attack_delay_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return BaseMagicPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        BaseMagicPhalanxTick::Pending
    }
}

pub(crate) fn calculate_owned_base_magic_attack(
    game: &mut CGame,
    phalanx: &CBaseMagicPhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master().master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let attacker_level = player.level();
    let weapon_level = player.equipment().get_goods(2).map_or(0, |goods| {
        goods.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1)
    });
    let weapon_damage_factors = game.globe_setup().weapon_damage_factors();
    let critical_rate = game.globe_setup().critical_rate();
    let combat_scales = game.globe_setup().base_combat_scales();
    calculate_base_magic_attack(
        phalanx,
        target_level,
        combat,
        occupation,
        attacker_level,
        weapon_level,
        weapon_damage_factors,
        critical_rate,
        combat_scales,
        |maximum| game.skill_random_below(maximum),
    )
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют входы исходной формулы")]
pub(crate) fn calculate_base_magic_attack(
    phalanx: &CBaseMagicPhalanx,
    target_level: u8,
    mut combat: PlayerCombatProperties,
    occupation: u8,
    attacker_level: u8,
    weapon_level: i32,
    weapon_damage_factors: (f32, f32),
    critical_rate: f32,
    combat_scales: [f32; 5],
    mut random_below: impl FnMut(i32) -> i32,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    if master.master_type != 400 || master.master_id == 0 {
        return None;
    }
    let (weapon_divisor, weapon_minimum) = weapon_damage_factors;
    let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let mut damage_factor = if weapon_divisor == 0.0 {
        1.0
    } else {
        delta as f32 / weapon_divisor
    };
    damage_factor = damage_factor.min(1.0).max(weapon_minimum);
    let width_delta = phalanx
        .maximum_attack()
        .wrapping_sub(phalanx.minimum_attack());
    let width = if width_delta < 0 {
        width_delta.wrapping_neg()
    } else {
        width_delta
    }
    .wrapping_add(1);
    let random_damage = random_below(width);
    let element_damage = phalanx
        .element_modifier()
        .wrapping_mul(combat.element_modify)
        .wrapping_div(100)
        .wrapping_add(combat.add_element_attack as i32)
        .wrapping_add(random_damage)
        .wrapping_add(phalanx.minimum_attack())
        .max(0);
    let mut attack = AttackInformation {
        skill_id: super::basemagic::BASE_MAGIC_SKILL_ID,
        skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type,
        attacker_id: master.master_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: 100,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: element_damage,
            mp_damage: 0,
        }],
    };
    if random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        for power in &mut attack.damages {
            power.hp_damage = ((power.hp_damage as f32) * critical_rate).round_ties_even() as i32;
        }
    }
    let [blast_attack, blast_defense, element_blast_attack, element_blast_defense, full_miss] =
        combat_scales;
    if combat.blast_attack_scale() < 1.0 {
        combat.blast_attack_scale_bits = blast_attack.max(1.0).to_bits();
    }
    if combat.blast_defense_scale() < 0.01 {
        combat.blast_defense_scale_bits = blast_defense.max(0.01).to_bits();
    }
    if combat.element_blast_attack_scale() < 1.0 {
        combat.element_blast_attack_scale_bits = element_blast_attack.max(1.0).to_bits();
    }
    if combat.element_blast_defense_scale() < 0.01 {
        combat.element_blast_defense_scale_bits = element_blast_defense.max(0.01).to_bits();
    }
    if combat.full_miss_scale() < 0.01 {
        combat.full_miss_scale_bits = full_miss.max(0.01).to_bits();
    }
    if combat.critical_rate() < 1.0 {
        combat.critical_rate_bits = critical_rate.max(1.0).to_bits();
    }
    Some((attack, combat, occupation, attacker_level))
}
