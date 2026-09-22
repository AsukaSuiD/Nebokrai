//! Данные рассчитанной атаки и её источника; применение к живой цели остаётся у Zone.

mod attackpower;
mod masterinfo;

pub use attackpower::{AttackInformation, AttackPower, AttackPowerType, FinalAttackDamage};
pub use masterinfo::MasterInfo;

/// Исходное значение ID навыка в конструкторе и Clear `tagAttackInformation`.
pub const UNKNOWN_SKILL_ID: u32 = 0x7fff_ffff;
