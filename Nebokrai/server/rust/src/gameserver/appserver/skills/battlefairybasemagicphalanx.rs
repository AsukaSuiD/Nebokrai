//! Снаряд базовой атаки боевой феи GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/battlefairybasemagicphalanx.cpp`. Жизненный цикл и два
//! раздельных чтения часов совпадают с другими `CSummonShape`, но формула
//! использует `GAP_BF_SPRITE`, единичный коэффициент оружия и реальную
//! критическую ставку игрока. Формула, wrapping и два исходных вызова RNG
//! принадлежат этому owner-у; `CGame` передаёт снимок владельца и применяет
//! рассчитанную атаку к независимому владельцу цели.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyPhalanxTick {
    Pending,
    Attack {
        target: ShapeIdentity,
        sampled_at_ms: u32,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBattleFairyBaseMagicPhalanx {
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

impl CBattleFairyBaseMagicPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору CBFBaseAttackPhalanx")]
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

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn minimum_attack(&self) -> i32 { self.minimum_attack }
    pub(crate) const fn maximum_attack(&self) -> i32 { self.maximum_attack }
    pub(crate) const fn element_modifier(&self) -> i32 { self.element_modifier }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        get_attack_now_ms: impl FnOnce() -> u32,
    ) -> BattleFairyPhalanxTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return BattleFairyPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if attack_now_ms.wrapping_sub(self.started_at_ms) > self.attack_delay_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return BattleFairyPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        BattleFairyPhalanxTick::Pending
    }
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют входы исходной формулы")]
pub(crate) fn calculate_battle_fairy_base_magic_attack(
    phalanx: &CBattleFairyBaseMagicPhalanx,
    mut combat: PlayerCombatProperties,
    occupation: u8,
    attacker_level: u8,
    sprite: i32,
    combat_scales: [f32; 5],
    mut random_below: impl FnMut(i32) -> i32,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let master = phalanx.master();
    if master.master_type != 400 || master.master_id == 0 {
        return None;
    }
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
        .wrapping_mul(sprite)
        .wrapping_div(100)
        .wrapping_add(random_damage)
        .wrapping_add(phalanx.minimum_attack())
        .max(0);
    let mut attack = AttackInformation {
        skill_id: super::battlefairybasemagic::BATTLE_FAIRY_BASE_MAGIC_SKILL_ID,
        skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type,
        attacker_id: master.master_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier: 100,
        damage_factor: 1.0,
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
        let critical_rate = combat.critical_rate();
        for power in &mut attack.damages {
            power.hp_damage =
                ((power.hp_damage as f32) * critical_rate).round_ties_even() as i32;
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
    Some((attack, combat, occupation, attacker_level))
}
