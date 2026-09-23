//! Visual-ресурс зарегистрированного навыка Zone.
//! Категории SkillVisualEffectKind — внутренний диспетчер Rust, а не enum оригинала.
//! База восстановлена по GameServer/gameserver.exe + GameServer.pdb,
//! исходный владелец appserver/states/visualeffect.h/.cpp.
//! См. docs/gameplay/skills.md о порядке публикации и удаления.

use crate::effects::CVisualEffect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillVisualEffectKind {
    BaseAttack,
    Rage,
    KnightCut,
    Swallow,
    Flash,
    LittleFlash,
    BattleFairy,
    SpriteBurn,
    LeafCut,
    FrontCellSword,
    SelfCast,
    ThunderSlash,
    ArrowCast,
    CrossbowCast,
    Scorpion,
    BoaLock,
    Kerosene,
    Ignition,
    HeartlessArrow,
    BaseProjectile,
    Heal,
    GodBless,
    ZonalCast,
    Lightning,
    ChainLightning,
    Infernol,
    Blind,
    Rush,
    Rush2,
    ArmyBreak,
    GhostCut,
    Mosou,
    ThunderBlow2,
    TargetedProjectile,
    PathProjectile,
    DirectProjectile,
    KnockOut,
    SpiderWeb,
    SpiderPoison,
    Promotion,
    Cure,
    Hearten,
    Fury,
    RageBreak,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SkillVisualEffect {
    kind: SkillVisualEffectKind,
    base: CVisualEffect,
}

impl SkillVisualEffect {
    pub const fn new(kind: SkillVisualEffectKind, loop_value: i32) -> Self {
        let mut base = CVisualEffect::new();
        base.begin_visual_effect(loop_value);
        Self { kind, base }
    }

    pub const fn kind(&self) -> SkillVisualEffectKind {
        self.kind
    }

    pub const fn is_ended(&self) -> bool {
        self.base.is_ended()
    }

    pub const fn update_base_tail(&mut self) {
        self.base.update_visual_effect();
    }

    pub const fn begin_visual_effect(&mut self, loop_value: i32) {
        self.base.begin_visual_effect(loop_value);
    }
}
