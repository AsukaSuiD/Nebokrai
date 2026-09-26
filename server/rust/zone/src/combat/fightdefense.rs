//! Базовая защита GameServer (`SKILL_BASE_DEFENSE`) для обычной атаки.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fightdefense.cpp`. Сохранены проверки попадания и полного
//! промаха, физический, стихийный и духовный урон, критические и усиленные
//! удары, уклонение и коэффициент PvP. Полный промах обнуляет только physical,
//! element и soul: poison остаётся для последующего `ApplyFinalDamage`. Для
//! монстров сохраняются отдельные
//! ограничения попадания, защита и сопротивления без коэффициента PvP.
//! PvP full-miss со стихийным уроном сравнивает целый RNG непосредственно с
//! x87-произведением `FILD u16 * FMUL f32`: дробная часть порога не усекается
//! и не округляется промежуточной записью в `f32`.
//! Уклонение аналогично загружает исходную константу `0.01_f32` в x87 и
//! усекает только итоговое произведение с целым уроном.
//! Стихийный множитель `Promotion` сначала перемножает два целых операнда,
//! затем применяет исходную `0.001_f32` и также усекает лишь конечный результат.
//! Беззнаковые `defense` и `element resistance` сохраняются как `u32`: перед
//! x87 оригинал корректирует старший бит через `+2^32`, а обычную половину
//! вычисляет логическим `SHR` до преобразования в целое повреждение.
//! Функции вызываются на стадии `Calculate` общего конвейера и не меняют число
//! или порядок обращений к RNG. Типизированная ветвь щитов и `Promotion`
//! вызывается в исходной точке `PreDefense`, до обычной защиты и в порядке
//! добавления состояний. Коэффициент `PillarState` применяется в
//! точке `PostDefense`, после обычного расчёта, но до множителей урона и PvP;
//! произведение целого урона и сохранённого `f32`-коэффициента усекается к нулю
//! при записи обратно в целое поле. Прочие ещё не восстановленные состояния не
//! подменяются этой реализацией.
//! Общий FISTP DWORD-адаптер (Zone `combat/rounding`) сначала усекает к нулю
//! и лишь затем проверяет signed range: дробная часть выше INT_MAX ещё может
//! усечься в INT_MAX. NaN, бесконечность и переполнение дают исходный
//! indefinite INT_MIN.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Машинные статусы по дизассемблу тел точной пары:
//!
//! | функция | RVA | статус |
//! |---|---|---|
//! | `CFightDefense::CFightDefense` | `0x001B0970` | `VERIFIED_DISASSEMBLY` (id 0xA = `SKILL_BASE_DEFENSE`) |
//! | `CFightDefense::~CFightDefense` | `0x001B09D0` | `VERIFIED_DISASSEMBLY` (sentinel = `UNKNOWN_SKILL_ID`) |
//! | `CFightDefense::PreDefense` | `0x001B0A50` | `VERIFIED_DISASSEMBLY` skip-правила `530..=545 && != 544`; диспетчер щитов и само правило — `effects/defenseshield.rs` |
//! | `CFightDefense::Defense` | `0x001B10E0` | `VERIFIED_DISASSEMBLY` |
//! | `CFightDefense::PostDefense` | `0x001B1050` | `VERIFIED_DISASSEMBLY` (Pillar factor) |
//!
//! По `Defense` сверены: hit-таблицы по occupation, level-модификатор
//! `(Δlevel − 3) * 15` через `shl 4` + `sub`, full-miss x87-ветвь без
//! f32-промежутка, порядок RNG hit → blast, обнуление видов 1/3/4 при miss с
//! сохранением Poison (`full_miss = 2`), логический SHR половины defense до
//! float, критический множитель `−0.5f32`, один FISTP уклонения и PvP factor
//! после clamp без повторного clamp. Safe/city-war гейты — у caller
//! (`received_defense_allowed`, `game/periodicattack.rs` старого пакета),
//! маркер city-war совпадает. Установленное расхождение f32-округления
//! mp-фактора закрыто в Zone `effects/{shieldabsorption, lifeshield}`;
//! формулы этого файла оно не затрагивает.
//!
//! Швы к владельцам старого пакета: RNG-состояние и `random(int)` остаются у владельца (параметр
//! `&mut dyn FnMut(i32) -> i32` передаётся дословно); снимок
//! `PlayerCombatProperties` производит hub `CPlayer` (`combat_properties()`),
//! прежний путь типа в `appserver/player.rs` сохраняется re-export-переходником;
//! `MonsterCombatProperties` — соседний снимок `combat/monsterformula`;
//! setup — тот же `GlobeSetupSnapshot` ресурсов `nebokrai_shared`, что
//! переиздаёт `setup/globesetup.rs` старого пакета; ветвь щитов до обычной
//! защиты исполняет диспетчер Zone `effects::DefenseShieldState`, а живые
//! Begin/restart/AI/End цепочки щитов остаются у переходного Game
//! (`appserver/skills/shieldstate.rs`); допуск защиты, очистка attack и
//! проекция war-soul маны — у caller `game/periodicattack.rs` старого пакета.

use nebokrai_shared::resources::GlobeSetupSnapshot;

use super::monsterformula::MonsterCombatProperties;
use super::{AttackInformation, AttackPowerType, truncate_original};
use crate::effects::{DefenseShieldState, is_pre_defense_skipped_skill, promotion_element_attack};

/// Снимок боевых свойств `CPlayer` для формул базовой защиты и смежных
/// боевых семейств. Производитель — hub `CPlayer` (`combat_properties()`);
/// scale/rate поля хранят сырые f32-биты исходного property-пакета.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PlayerCombatProperties {
    pub maximum_hp: u32,
    pub maximum_mp: u32,
    pub maximum_yp: u16,
    pub maximum_rp: u16,
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub attack_speed: u16,
    pub hit: u16,
    pub dodge: u16,
    pub cch: u16,
    pub defense: u32,
    pub element_resistance: u32,
    pub add_element_attack: u32,
    pub hp_recovery: u16,
    pub mp_recovery: u16,
    pub burden: u16,
    pub reank: u16,
    pub attack_avoid: u16,
    pub element_avoid: u16,
    pub full_miss: u16,
    pub element_modify: i32,
    pub blast_attack: u16,
    pub blast_element_attack: u16,
    pub soul_resistance: u16,
    pub add_soul_attack: u16,
    pub blast_attack_scale_bits: u32,
    pub blast_defense_scale_bits: u32,
    pub element_blast_attack_scale_bits: u32,
    pub element_blast_defense_scale_bits: u32,
    pub full_miss_scale_bits: u32,
    pub critical_rate_bits: u32,
    pub resume_hp_peace: i32,
    pub resume_mp_peace: i32,
    pub resume_hp_fight: i32,
    pub resume_mp_fight: i32,
    pub restored_hp_peace: i32,
    pub restored_mp_peace: i32,
    pub restored_hp_fight: i32,
    pub restored_mp_fight: i32,
    pub battle_fairy_summoned: bool,
    pub battle_fairy_recall: bool,
    pub battle_fairy_died: bool,
}

impl PlayerCombatProperties {
    pub const fn blast_attack_scale(self) -> f32 {
        f32::from_bits(self.blast_attack_scale_bits)
    }

    pub const fn blast_defense_scale(self) -> f32 {
        f32::from_bits(self.blast_defense_scale_bits)
    }

    pub const fn full_miss_scale(self) -> f32 {
        f32::from_bits(self.full_miss_scale_bits)
    }

    pub const fn element_blast_attack_scale(self) -> f32 {
        f32::from_bits(self.element_blast_attack_scale_bits)
    }

    pub const fn element_blast_defense_scale(self) -> f32 {
        f32::from_bits(self.element_blast_defense_scale_bits)
    }

    pub const fn critical_rate(self) -> f32 {
        f32::from_bits(self.critical_rate_bits)
    }
}

fn avoid_damage(damage: i32, avoid: u16) -> i32 {
    let passed = 100i32.wrapping_sub(i32::from(avoid));
    if (1..100).contains(&passed) {
        truncate_original(f64::from(passed) * f64::from(0.01_f32) * f64::from(damage))
    } else {
        damage
    }
}

fn subtract_non_player_defense(
    damage: i32,
    defense: u32,
    critical: bool,
    critical_rate: f32,
) -> i32 {
    if critical {
        // Для источника-монстра используется общий критический множитель;
        // беззнаковая защита умножается без промежуточного округления к f32.
        damage.wrapping_add(truncate_original(
            f64::from(defense) * f64::from(critical_rate) * -0.5,
        ))
    } else {
        damage.wrapping_sub((defense / 2) as i32)
    }
}

fn clear_miss_sensitive_damage(attack: &mut AttackInformation) {
    for power in &mut attack.damages {
        if matches!(
            power.kind,
            AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul
        ) {
            power.hp_damage = 0;
        }
    }
    attack.damage_modifier = 0;
}

fn apply_monster_promotion(
    skill_id: u32,
    factor: Option<u16>,
    kind: AttackPowerType,
    damage: i32,
) -> i32 {
    if is_pre_defense_skipped_skill(skill_id) {
        return damage;
    }
    if kind != AttackPowerType::Element {
        return damage;
    }
    factor.map_or(damage, |factor| promotion_element_attack(factor, damage))
}

fn apply_pillar_post_defense(
    skill_id: u32,
    pillar_damage_factor: Option<f32>,
    damage: i32,
) -> i32 {
    if damage == 0 || (0x212..=0x224).contains(&skill_id) {
        return damage;
    }
    pillar_damage_factor.map_or(damage, |factor| {
        truncate_original(f64::from(damage) * f64::from(factor))
    })
}

pub fn defend_monster_base_attack(
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
        clear_miss_sensitive_damage(attack);
        attack.full_miss = 2;
        return;
    }

    for power in &mut attack.damages {
        power.hp_damage = apply_monster_promotion(
            attack.skill_id,
            target.promotion_magic_attack_factor,
            power.kind,
            power.hp_damage,
        );
        match power.kind {
            AttackPowerType::Physical => {
                let defense = target.defense;
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
                    power.hp_damage = power.hp_damage.wrapping_sub((defense / 2) as i32);
                }
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(1);
            }
            AttackPowerType::Element => {
                let resistance = target.element_resistance;
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
                    power.hp_damage = power.hp_damage.wrapping_sub((resistance / 2) as i32);
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

/// Ветка общего `CFightDefense::Defense` для `CBuild`: target не является
/// player/monster, поэтому level modifier, PvP factor и avoid не применяются.
/// PreDefense сохраняет все Promotion по порядку состояний; ресурсные щиты
/// требуют игрока и пропускаются. Player-attacker сохраняет blast/critical scales
/// и исходный порядок RNG: hit, затем по одному blast-броску для physical и
/// element damage.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет источник, защиту и упорядоченные Promotion цели")]
pub fn defend_build_base_attack(
    attack: &mut AttackInformation,
    attacker: PlayerCombatProperties,
    attacker_occupation: u8,
    defense: u32,
    element_resistance: u32,
    setup: &GlobeSetupSnapshot,
    promotion_factors: &[u16],
    random: &mut dyn FnMut(i32) -> i32,
) {
    let (minimum_hit, maximum_hit) = setup.player_hit_limits(attacker_occupation);
    let hit = maximum_hit
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    if hit <= random(100) {
        clear_miss_sensitive_damage(attack);
        attack.full_miss = 2;
        return;
    }

    for power in &mut attack.damages {
        for &factor in promotion_factors {
            power.hp_damage = apply_monster_promotion(
                attack.skill_id, Some(factor), power.kind, power.hp_damage,
            );
        }
        match power.kind {
            AttackPowerType::Physical => {
                if random(100) < i32::from(attacker.blast_attack) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage) * f64::from(attacker.blast_attack_scale()),
                    );
                    let scale = f64::from(attacker.blast_defense_scale());
                    power.hp_damage = if attack.critical {
                        power.hp_damage.wrapping_add(truncate_original(
                            f64::from(defense)
                                * scale
                                * f64::from(attacker.critical_rate())
                                * -0.5,
                        ))
                    } else {
                        power.hp_damage.wrapping_sub(truncate_original(
                            f64::from(defense / 2) * scale,
                        ))
                    };
                    attack.blast_attack = true;
                } else if attack.critical {
                    power.hp_damage = power.hp_damage.wrapping_add(truncate_original(
                        f64::from(defense) * f64::from(attacker.critical_rate()) * -0.5,
                    ));
                } else {
                    power.hp_damage = power.hp_damage.wrapping_sub((defense / 2) as i32);
                }
                power.hp_damage = power.hp_damage.max(1);
            }
            AttackPowerType::Element => {
                if random(100) < i32::from(attacker.blast_element_attack) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage)
                            * f64::from(attacker.element_blast_attack_scale()),
                    );
                    let scale = f64::from(attacker.element_blast_defense_scale());
                    power.hp_damage = if attack.critical {
                        power.hp_damage.wrapping_add(truncate_original(
                            f64::from(element_resistance)
                                * scale
                                * f64::from(attacker.critical_rate())
                                * -0.5,
                        ))
                    } else {
                        power.hp_damage.wrapping_sub(truncate_original(
                            f64::from(element_resistance / 2) * scale,
                        ))
                    };
                    attack.blast_attack = true;
                } else if attack.critical {
                    power.hp_damage = power.hp_damage.wrapping_add(truncate_original(
                        f64::from(element_resistance)
                            * f64::from(attacker.critical_rate())
                            * -0.5,
                    ));
                } else {
                    power.hp_damage = power
                        .hp_damage
                        .wrapping_sub((element_resistance / 2) as i32);
                }
                power.hp_damage = power.hp_damage.max(1);
            }
            AttackPowerType::Soul => {
                // `CBuild` наследует нулевую реализацию soul resistance.
                power.hp_damage = power.hp_damage.max(0);
            }
            AttackPowerType::Poison => {}
        }
        if power.hp_damage > 0 {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(attack.damage_factor),
            )
            .max(1);
        }
    }
}

/// Источник-монстр против постройки использует общий monster hit range.
/// У CBuild нет full-miss, avoid и soul resistance. Promotion действует до
/// обычной защиты; физический и стихийный урон может уменьшиться до нуля.
pub fn defend_build_from_monster_base_attack(
    attack: &mut AttackInformation,
    defense: u32,
    element_resistance: u32,
    setup: &GlobeSetupSnapshot,
    promotion_factors: &[u16],
    random: &mut dyn FnMut(i32) -> i32,
) {
    let (minimum_hit, maximum_hit) = setup.monster_hit_limits();
    let hit = maximum_hit
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    if hit <= random(100) {
        clear_miss_sensitive_damage(attack);
        attack.full_miss = 2;
        return;
    }

    for power in &mut attack.damages {
        for &factor in promotion_factors {
            power.hp_damage = apply_monster_promotion(
                attack.skill_id, Some(factor), power.kind, power.hp_damage,
            );
        }
        match power.kind {
            AttackPowerType::Physical => {
                power.hp_damage = subtract_non_player_defense(
                    power.hp_damage, defense, attack.critical, setup.critical_rate(),
                ).max(0);
            }
            AttackPowerType::Element => {
                power.hp_damage = subtract_non_player_defense(
                    power.hp_damage, element_resistance, attack.critical, setup.critical_rate(),
                ).max(0);
            }
            AttackPowerType::Soul => power.hp_damage = power.hp_damage.max(0),
            AttackPowerType::Poison => {}
        }
        if power.hp_damage > 0 {
            power.hp_damage = truncate_original(
                f64::from(power.hp_damage) * f64::from(attack.damage_factor),
            ).max(1);
        }
    }
}

pub fn defend_player_base_attack(
    attack: &mut AttackInformation,
    attacker: PlayerCombatProperties,
    attacker_occupation: u8,
    target: PlayerCombatProperties,
    target_mana: u32,
    target_war_soul_mana: Option<i32>,
    setup: &GlobeSetupSnapshot,
    random: &mut dyn FnMut(i32) -> i32,
    defense_shields: &mut [DefenseShieldState],
    pillar_damage_factor: Option<f32>,
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
        if has_element {
            f64::from(random(100))
                < f64::from(target.full_miss) * f64::from(attacker.full_miss_scale())
        } else {
            random(100) < i32::from(target.full_miss)
        }
    };
    if hit <= random(100) || full_miss {
        clear_miss_sensitive_damage(attack);
        attack.full_miss = if full_miss { 1 } else { 2 };
        return;
    }

    for power in &mut attack.damages {
        for state in defense_shields.iter_mut() {
            state.apply_pre_defense(
                attack.skill_id,
                attack.damage_factor,
                Some((target_mana, target_war_soul_mana)),
                power,
            );
        }
        match power.kind {
            AttackPowerType::Physical => {
                let defense = target.defense;
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
                    power.hp_damage = power.hp_damage.wrapping_sub((defense / 2) as i32);
                }
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(1);
            }
            AttackPowerType::Element => {
                let resistance = target.element_resistance;
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
                    power.hp_damage = power.hp_damage.wrapping_sub((resistance / 2) as i32);
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
        power.hp_damage = apply_pillar_post_defense(
            attack.skill_id,
            pillar_damage_factor,
            power.hp_damage,
        );
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

pub fn defend_player_from_monster_base_attack(
    attack: &mut AttackInformation,
    target: PlayerCombatProperties,
    target_mana: u32,
    target_war_soul_mana: Option<i32>,
    setup: &GlobeSetupSnapshot,
    random: &mut dyn FnMut(i32) -> i32,
    defense_shields: &mut [DefenseShieldState],
    pillar_damage_factor: Option<f32>,
) {
    let (minimum_hit, maximum_hit) = setup.monster_hit_limits();
    let hit = maximum_hit
        .wrapping_add(attack.hit_modifier)
        .clamp(minimum_hit, maximum_hit);
    let full_miss = target.full_miss != 0 && random(100) < i32::from(target.full_miss);
    if hit <= random(100) || full_miss {
        clear_miss_sensitive_damage(attack);
        attack.full_miss = if full_miss { 1 } else { 2 };
        return;
    }

    for power in &mut attack.damages {
        for state in defense_shields.iter_mut() {
            state.apply_pre_defense(
                attack.skill_id,
                attack.damage_factor,
                Some((target_mana, target_war_soul_mana)),
                power,
            );
        }
        match power.kind {
            AttackPowerType::Physical => {
                power.hp_damage = subtract_non_player_defense(
                    power.hp_damage,
                    target.defense,
                    attack.critical,
                    setup.critical_rate(),
                );
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(0);
            }
            AttackPowerType::Element => {
                power.hp_damage = subtract_non_player_defense(
                    power.hp_damage,
                    target.element_resistance,
                    attack.critical,
                    setup.critical_rate(),
                );
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
        power.hp_damage = apply_pillar_post_defense(
            attack.skill_id,
            pillar_damage_factor,
            power.hp_damage,
        );
        if power.hp_damage > 0 {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(attack.damage_factor))
                    .max(1);
        }
    }
}

pub fn defend_monster_from_monster_base_attack(
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
        clear_miss_sensitive_damage(attack);
        attack.full_miss = 2;
        return;
    }

    for power in &mut attack.damages {
        power.hp_damage = apply_monster_promotion(
            attack.skill_id,
            target.promotion_magic_attack_factor,
            power.kind,
            power.hp_damage,
        );
        match power.kind {
            AttackPowerType::Physical => {
                power.hp_damage = subtract_non_player_defense(
                    power.hp_damage,
                    target.defense,
                    attack.critical,
                    setup.critical_rate(),
                );
                power.hp_damage = avoid_damage(power.hp_damage, target.attack_avoid).max(0);
            }
            AttackPowerType::Element => {
                power.hp_damage = subtract_non_player_defense(
                    power.hp_damage,
                    target.element_resistance,
                    attack.critical,
                    setup.critical_rate(),
                );
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
