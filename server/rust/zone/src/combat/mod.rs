//! Данные рассчитанной атаки и её источника; применение к живой цели остаётся у Zone.

mod attackpower; // типизированное описание рассчитанной атаки.
mod masterinfo; // tagMasterInfo: данные владельца атаки.
pub mod monsterformula; // формулы боевого опыта и property-пакеты монстра.
mod rounding; // числовые усечения боевых формул исходного GameServer.

pub use attackpower::{AttackInformation, AttackPower, AttackPowerType, FinalAttackDamage}; // контракт рассчитанной атаки.
pub use masterinfo::MasterInfo; // снимок владельца атаки.
pub use rounding::{truncate_original, truncate_original_i64_low}; // исходные правила усечения.

/// Исходное значение ID навыка в конструкторе и Clear `tagAttackInformation`.
pub const UNKNOWN_SKILL_ID: u32 = 0x7fff_ffff;
