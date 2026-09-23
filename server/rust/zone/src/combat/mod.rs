//! Данные рассчитанной атаки и её источника; применение к живой цели остаётся у Zone.

mod attackpower;
mod masterinfo;
mod rounding;

pub use attackpower::{AttackInformation, AttackPower, AttackPowerType, FinalAttackDamage};
pub use masterinfo::MasterInfo;
pub use rounding::{truncate_original, truncate_original_i64_low};

/// Исходное значение ID навыка в конструкторе и Clear `tagAttackInformation`.
pub const UNKNOWN_SKILL_ID: u32 = 0x7fff_ffff;
