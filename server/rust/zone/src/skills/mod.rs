//! Данные живых навыков Zone, которыми временно управляет прежний Game.

mod visualeffect;
mod weak;
mod poisonfog;
mod spidermist;

pub use visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
pub use weak::{WEAK_SKILL_ID, WeakPhalanx, WeakPhalanxTick, weak_lifetime};
pub use poisonfog::{PoisonFogPhalanx, PoisonFogPhalanxTick};
pub use spidermist::{SPIDER_MIST_SKILL_ID, SpiderMistPhalanx, SpiderMistPhalanxTick};
