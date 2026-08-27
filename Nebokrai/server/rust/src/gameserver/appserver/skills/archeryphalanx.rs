//! Региональный снаряд базовой стрельбы GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/archeryphalanx.cpp`. Два раздельных чтения часов,
//! строгие границы срока жизни и задержки атаки сохранены. В отличие от
//! базовой магии `End` лишь ставит `CS_DELETE`: отдельный немедленный пакет
//! выхода здесь не отправляется. Расчёт трёх типов урона, wrapping и два
//! исходных вызова RNG принадлежат этому owner-у; `CGame` передаёт снимок
//! живого игрока и применяет рассчитанную атаку к независимому владельцу цели.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArcheryPhalanxTick {
    Pending,
    Attack {
        target: ShapeIdentity,
        sampled_at_ms: u32,
    },
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CArcheryPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    attack_delay_ms: u32,
    target: ShapeIdentity,
}

impl CArcheryPhalanx {
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
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
            attack_delay_ms,
            target,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        get_attack_now_ms: impl FnOnce() -> u32,
    ) -> ArcheryPhalanxTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.lifetime_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ArcheryPhalanxTick::Expired;
        }
        let attack_now_ms = get_attack_now_ms();
        if attack_now_ms.wrapping_sub(self.started_at_ms) > self.attack_delay_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            return ArcheryPhalanxTick::Attack {
                target: self.target,
                sampled_at_ms: attack_now_ms,
            };
        }
        ArcheryPhalanxTick::Pending
    }
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют входы исходной формулы")]
pub(crate) fn calculate_archery_attack(
    phalanx: &CArcheryPhalanx,
    target_level: u8,
    mut combat: PlayerCombatProperties,
    occupation: u8,
    attacker_level: u8,
    weapon_level: i32,
    hit_modifier: i32,
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
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let width_delta = maximum.wrapping_sub(minimum);
    let width = if width_delta < 0 {
        width_delta.wrapping_neg()
    } else {
        width_delta
    }
    .wrapping_add(1);
    let physical = minimum.wrapping_add(random_below(width));
    let mut attack = AttackInformation {
        skill_id: super::archery::ARCHERY_SKILL_ID,
        skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type,
        attacker_id: master.master_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical.max(0),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: combat.add_element_attack as i32,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(combat.add_soul_attack),
                mp_damage: 0,
            },
        ],
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

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый legacy byte codec снаряда.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp

// ============================================================================
// FUNCTION: CArcheryPhalanx::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:270
// RVA: 0x001E47A0
// ADDRESS: 005e47a0
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CArcheryPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archeryphalanx.cpp:281
// RVA: 0x001EB070
// ADDRESS: 005eb070
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
