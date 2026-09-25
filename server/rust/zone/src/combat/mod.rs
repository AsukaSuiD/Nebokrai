//! Данные рассчитанной атаки и её источника; применение к живой цели остаётся у Zone.

mod attackpower; // типизированное описание рассчитанной атаки.
mod fightdefense; // базовая защита CFightDefense: hit/полный промах, защиты и сопротивления, blast/critical, уклонение, PvP/Pillar; снимок свойств игрока.
mod masterinfo; // tagMasterInfo: данные владельца атаки.
pub mod monsterformula; // формулы боевого опыта и property-пакеты монстра.
mod rounding; // числовые усечения боевых формул исходного GameServer.
mod weaponattack; // общая оружейная формула CalculateAttackPower: виды roll-ширины, компоненты, критический хвост и property источника по типу владельца.

pub use attackpower::{AttackInformation, AttackPower, AttackPowerType, FinalAttackDamage}; // контракт рассчитанной атаки.
pub use fightdefense::PlayerCombatProperties; // снимок боевых свойств игрока.
pub use fightdefense::{
    defend_build_base_attack, defend_build_from_monster_base_attack, defend_monster_base_attack,
    defend_monster_from_monster_base_attack, defend_player_base_attack,
    defend_player_from_monster_base_attack,
}; // тела базовой защиты по парам источник–цель.
pub use masterinfo::MasterInfo; // снимок владельца атаки.
pub use rounding::{truncate_original, truncate_original_i64_low}; // исходные правила усечения.
pub use weaponattack::{PlayerWeaponRoll, WeaponDamageLiveField, WeaponPowerBoost, WeaponSourceCombat, WeaponSourceProperty}; // типы оружейной формулы.
pub use weaponattack::{apply_weapon_critical, fill_weapon_damage, weapon_source_property}; // тела оружейной формулы.

/// Исходное значение ID навыка в конструкторе и Clear `tagAttackInformation`.
pub const UNKNOWN_SKILL_ID: u32 = 0x7fff_ffff;
