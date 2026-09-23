//! Данные живых навыков Zone, которыми временно управляет прежний Game.

mod visualeffect;
mod weak;
mod poisonfog;

pub use visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
pub use weak::{WEAK_SKILL_ID, WeakPhalanx, WeakPhalanxTick, weak_lifetime};
pub use poisonfog::{PoisonFogPhalanx, PoisonFogPhalanxTick};
