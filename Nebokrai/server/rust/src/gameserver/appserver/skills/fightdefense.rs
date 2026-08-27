//! Базовая защита GameServer (`SKILL_BASE_DEFENSE`) для обычной атаки.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fightdefense.cpp`. Сохранены проверки попадания и полного
//! промаха, физический, стихийный и духовный урон, критические и усиленные
//! удары, уклонение и коэффициент PvP. Для монстров сохраняются отдельные
//! ограничения попадания, защита и сопротивления без коэффициента PvP.
//! Функции вызываются на стадии `Calculate` общего конвейера и не меняют число
//! или порядок обращений к RNG. Канонический мана-щит вызывается в исходной
//! точке `PreDefense`, до обычной защиты; прочие ещё не восстановленные щиты
//! не подменяются этой реализацией.

use crate::gameserver::appserver::monster::MonsterCombatProperties;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::skills::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPowerType};
use crate::setup::globesetup::GlobeSetupSnapshot;

fn truncate_original(value: f64) -> i32 {
    if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
        i32::MIN
    } else {
        value as i32
    }
}

fn avoid_damage(damage: i32, avoid: u16) -> i32 {
    let passed = 100i32.wrapping_sub(i32::from(avoid));
    if (1..100).contains(&passed) {
        truncate_original(f64::from(passed) * 0.01 * f64::from(damage))
    } else {
        damage
    }
}

pub(crate) fn defend_monster_base_attack(
    attack: &mut AttackInformation,
    attacker: PlayerCombatProperties,
    attacker_occupation: u8,
    attacker_level: u8,
    target: MonsterCombatProperties,
    setup: &GlobeSetupSnapshot,
    random: &mut dyn FnMut(i32) -> i32,
) {
    let (minimum_hit, maximum_hit) = setup.player_hit_limits(attacker_occupation);
    let level_modifier = (i32::from(target.level)
        .wrapping_sub(i32::from(attacker_level))
        .wrapping_sub(3))
    .wrapping_mul(15);
    let hit = maximum_hit
        .wrapping_add(level_modifier)
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    if hit <= random(100) {
        for power in &mut attack.damages {
            power.hp_damage = 0;
        }
        attack.damage_modifier = 0;
        attack.full_miss = 2;
        return;
    }

    for power in &mut attack.damages {
        match power.kind {
            AttackPowerType::Physical => {
                let defense = target.defense as i32;
                if random(100) < i32::from(attacker.blast_attack) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage) * f64::from(attacker.blast_attack_scale()),
                    );
                    power.hp_damage = if attack.critical {
                        power.hp_damage.wrapping_add(truncate_original(
                            f64::from(defense)
                                * f64::from(attacker.blast_defense_scale())
                                * f64::from(attacker.critical_rate())
                                * -0.5,
                        ))
                    } else {
                        power.hp_damage.wrapping_sub(truncate_original(
                            f64::from(defense / 2) * f64::from(attacker.blast_defense_scale()),
                        ))
                    };
                    attack.blast_attack = true;
                } else if attack.critical {
                    power.hp_damage = power.hp_damage.wrapping_add(truncate_original(
                        f64::from(defense) * f64::from(attacker.critical_rate()) * -0.5,
                    ));
                } else {
                    power.hp_damage = power.hp_damage.wrapping_sub(defense / 2);
                }
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(1);
            }
            AttackPowerType::Element => {
                let resistance = target.element_resistance as i32;
                if random(100) < i32::from(attacker.blast_element_attack) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage)
                            * f64::from(attacker.element_blast_attack_scale()),
                    );
                    power.hp_damage = if attack.critical {
                        power.hp_damage.wrapping_add(truncate_original(
                            f64::from(resistance)
                                * f64::from(attacker.element_blast_defense_scale())
                                * f64::from(attacker.critical_rate())
                                * -0.5,
                        ))
                    } else {
                        power.hp_damage.wrapping_sub(truncate_original(
                            f64::from(resistance / 2)
                                * f64::from(attacker.element_blast_defense_scale()),
                        ))
                    };
                    attack.blast_attack = true;
                } else if attack.critical {
                    power.hp_damage = power.hp_damage.wrapping_add(truncate_original(
                        f64::from(resistance) * f64::from(attacker.critical_rate()) * -0.5,
                    ));
                } else {
                    power.hp_damage = power.hp_damage.wrapping_sub(resistance / 2);
                }
                power.hp_damage = avoid_damage(power.hp_damage, target.element_avoid).max(1);
            }
            AttackPowerType::Soul => {
                let resistance = i32::from(target.soul_resistance);
                power.hp_damage = if attack.critical {
                    power.hp_damage.wrapping_sub(truncate_original(
                        f64::from(resistance) * f64::from(attacker.critical_rate()),
                    ))
                } else {
                    power.hp_damage.wrapping_sub(resistance)
                }
                .max(0);
            }
            AttackPowerType::Poison => {}
        }
        if power.hp_damage > 0 {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(attack.damage_factor))
                    .max(1);
        }
    }
}

pub(crate) fn defend_player_base_attack(
    attack: &mut AttackInformation,
    attacker: PlayerCombatProperties,
    attacker_occupation: u8,
    target: PlayerCombatProperties,
    target_mana: u32,
    target_war_soul_mana: Option<i32>,
    setup: &GlobeSetupSnapshot,
    random: &mut dyn FnMut(i32) -> i32,
    defense_shields: &mut [DefenseShieldState],
) {
    let (minimum_hit, maximum_hit) = setup.player_hit_limits(attacker_occupation);
    let hit = maximum_hit
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    let has_element = attack
        .damages
        .iter()
        .any(|power| power.kind == AttackPowerType::Element && power.hp_damage > 0);
    let full_miss = if target.full_miss == 0 {
        false
    } else {
        let chance = if has_element {
            truncate_original(f64::from(target.full_miss) * f64::from(attacker.full_miss_scale()))
        } else {
            i32::from(target.full_miss)
        };
        random(100) < chance
    };
    if hit <= random(100) || full_miss {
        for power in &mut attack.damages {
            power.hp_damage = 0;
        }
        attack.damage_modifier = 0;
        attack.full_miss = if full_miss { 1 } else { 2 };
        return;
    }

    for power in &mut attack.damages {
        for state in defense_shields.iter_mut() {
            state.absorb_damage(
                attack.skill_id,
                attack.damage_factor,
                target_mana,
                target_war_soul_mana,
                power,
            );
        }
        match power.kind {
            AttackPowerType::Physical => {
                let defense = target.defense as i32;
                if random(100) < i32::from(attacker.blast_attack) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage) * f64::from(attacker.blast_attack_scale()),
                    );
                    let scale = f64::from(attacker.blast_defense_scale());
                    power.hp_damage = if attack.critical {
                        power.hp_damage.wrapping_add(truncate_original(
                            f64::from(defense) * scale * f64::from(attacker.critical_rate()) * -0.5,
                        ))
                    } else {
                        power
                            .hp_damage
                            .wrapping_sub(truncate_original(f64::from(defense / 2) * scale))
                    };
                    attack.blast_attack = true;
                } else if attack.critical {
                    power.hp_damage = power.hp_damage.wrapping_add(truncate_original(
                        f64::from(defense) * f64::from(attacker.critical_rate()) * -0.5,
                    ));
                } else {
                    power.hp_damage = power.hp_damage.wrapping_sub(defense / 2);
                }
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(1);
            }
            AttackPowerType::Element => {
                let resistance = target.element_resistance as i32;
                if random(100) < i32::from(attacker.blast_element_attack) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage)
                            * f64::from(attacker.element_blast_attack_scale()),
                    );
                    let scale = f64::from(attacker.element_blast_defense_scale());
                    power.hp_damage = if attack.critical {
                        power.hp_damage.wrapping_add(truncate_original(
                            f64::from(resistance)
                                * scale
                                * f64::from(attacker.critical_rate())
                                * -0.5,
                        ))
                    } else {
                        power
                            .hp_damage
                            .wrapping_sub(truncate_original(f64::from(resistance / 2) * scale))
                    };
                    attack.blast_attack = true;
                } else if attack.critical {
                    power.hp_damage = power.hp_damage.wrapping_add(truncate_original(
                        f64::from(resistance) * f64::from(attacker.critical_rate()) * -0.5,
                    ));
                } else {
                    power.hp_damage = power.hp_damage.wrapping_sub(resistance / 2);
                }
                power.hp_damage = avoid_damage(power.hp_damage, target.element_avoid).max(1);
            }
            AttackPowerType::Soul => {
                let resistance = i32::from(target.soul_resistance);
                power.hp_damage = if attack.critical {
                    power.hp_damage.wrapping_sub(truncate_original(
                        f64::from(resistance) * f64::from(attacker.critical_rate()),
                    ))
                } else {
                    power.hp_damage.wrapping_sub(resistance)
                }
                .max(0);
            }
            AttackPowerType::Poison => {}
        }
        if power.hp_damage > 0 {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(attack.damage_factor))
                    .max(1);
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(setup.pvp_damage_factor()),
            );
        }
    }
}

pub(crate) fn defend_player_from_monster_base_attack(
    attack: &mut AttackInformation,
    target: PlayerCombatProperties,
    target_mana: u32,
    target_war_soul_mana: Option<i32>,
    setup: &GlobeSetupSnapshot,
    random: &mut dyn FnMut(i32) -> i32,
    defense_shields: &mut [DefenseShieldState],
) {
    let (minimum_hit, maximum_hit) = setup.monster_hit_limits();
    let hit = maximum_hit
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    let full_miss = target.full_miss != 0 && random(100) < i32::from(target.full_miss);
    if hit <= random(100) || full_miss {
        for power in &mut attack.damages {
            power.hp_damage = 0;
        }
        attack.damage_modifier = 0;
        attack.full_miss = if full_miss { 1 } else { 2 };
        return;
    }

    for power in &mut attack.damages {
        for state in defense_shields.iter_mut() {
            state.absorb_damage(
                attack.skill_id,
                attack.damage_factor,
                target_mana,
                target_war_soul_mana,
                power,
            );
        }
        match power.kind {
            AttackPowerType::Physical => {
                power.hp_damage = power.hp_damage.wrapping_sub(target.defense as i32 / 2);
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(0);
            }
            AttackPowerType::Element => {
                power.hp_damage = power
                    .hp_damage
                    .wrapping_sub(target.element_resistance as i32 / 2);
                power.hp_damage = avoid_damage(power.hp_damage, target.element_avoid).max(0);
            }
            AttackPowerType::Soul => {
                power.hp_damage = power
                    .hp_damage
                    .wrapping_sub(i32::from(target.soul_resistance))
                    .max(0);
            }
            AttackPowerType::Poison => {}
        }
        if power.hp_damage > 0 {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(attack.damage_factor))
                    .max(1);
        }
    }
}

pub(crate) fn defend_monster_from_monster_base_attack(
    attack: &mut AttackInformation,
    target: MonsterCombatProperties,
    setup: &GlobeSetupSnapshot,
    random: &mut dyn FnMut(i32) -> i32,
) {
    let (minimum_hit, maximum_hit) = setup.monster_hit_limits();
    let hit = maximum_hit
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    if hit <= random(100) {
        for power in &mut attack.damages {
            power.hp_damage = 0;
        }
        attack.damage_modifier = 0;
        attack.full_miss = 2;
        return;
    }

    for power in &mut attack.damages {
        match power.kind {
            AttackPowerType::Physical => {
                power.hp_damage = power.hp_damage.wrapping_sub(target.defense as i32 / 2);
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(0);
            }
            AttackPowerType::Element => {
                power.hp_damage = power
                    .hp_damage
                    .wrapping_sub(target.element_resistance as i32 / 2);
                power.hp_damage = avoid_damage(power.hp_damage, target.element_avoid).max(0);
            }
            AttackPowerType::Soul => {
                power.hp_damage = power
                    .hp_damage
                    .wrapping_sub(i32::from(target.soul_resistance))
                    .max(0);
            }
            AttackPowerType::Poison => {}
        }
        if power.hp_damage > 0 {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(attack.damage_factor))
                    .max(1);
        }
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.h

// ============================================================================
// FUNCTION: CFightDefense::CFightDefense
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.cpp:22
// RVA: 0x001B0970
// ADDRESS: 005b0970
// PROTOTYPE: undefined __thiscall CFightDefense(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFightDefense::~CFightDefense
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.cpp:28
// RVA: 0x001B09D0
// ADDRESS: 005b09d0
// PROTOTYPE: void __thiscall ~CFightDefense(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFightDefense::PreDefense
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.cpp:506
// RVA: 0x001B0A50
// ADDRESS: 005b0a50
// PROTOTYPE: void __thiscall PreDefense(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3, tagAttackPower * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFightDefense::PostDefense
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.cpp:835
// RVA: 0x001B1050
// ADDRESS: 005b1050
// PROTOTYPE: void __thiscall PostDefense(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3, tagAttackPower * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFightDefense::Defense
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fightdefense.cpp:34
// RVA: 0x001B10E0
// ADDRESS: 005b10e0
// PROTOTYPE: void __thiscall Defense(tagAttackInformation * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
