//! Общий owned visual-ресурс зарегистрированного навыка.
//! Источник: gameserver.exe/GameServer.pdb, appserver/states/visualeffect.h/.cpp;
//! производные эффекты принадлежат соответствующим owners appserver/skills.
//!
//! Signed loop не нормализуется: любой ненулевой loop подавляет базовый End.
//! Базовый Update выполняется и для завершённого эффекта; производный publisher
//! сам проверяет ended и наличие участников. Общий хвост пропускается только
//! при явном раннем выходе производного эффекта.
//! Одноаргументный Update не вызывает производную публикацию.
//!
//! Ресурс принадлежит зарегистрированному экземпляру, а не concrete payload:
//! отказавший Begin сохраняет его до End. Источники, цели и свойства не
//! кешируются внутри visual. Повторный Begin безопасно освобождает прежний
//! ресурс; Drop сбрасывает базу и не отправляет пакетов. Остальные производные
//! эффекты не подменяются подключёнными здесь вариантами.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillVisualEffectKind {
    BaseAttack,
    Rage,
    KnightCut,
    Flash,
    LittleFlash,
    BattleFairy,
    SpriteBurn,
    LeafCut,
    Kerosene,
    Ignition,
    PoisonFog,
    Weak,
    Blind,
    Rush,
    Rush2,
    ArmyBreak,
    Strike,
    KnockOut,
    SpiderWeb,
    SpiderPoison,
    Promotion,
    Cure,
    Hearten,
    ManaShield,
    MachineShield,
    Fury,
    RageBreak,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SkillVisualEffect {
    kind: SkillVisualEffectKind,
    base: CVisualEffect,
}

impl SkillVisualEffect {
    pub(crate) const fn new(kind: SkillVisualEffectKind, loop_value: i32) -> Self {
        let mut base = CVisualEffect::new();
        base.begin_visual_effect(loop_value);
        Self { kind, base }
    }

    pub(crate) const fn kind(&self) -> SkillVisualEffectKind {
        self.kind
    }

    pub(crate) const fn is_ended(&self) -> bool {
        self.base.is_ended()
    }

    pub(crate) const fn update_base_tail(&mut self) {
        self.base.update_visual_effect();
    }

    pub(crate) const fn begin_visual_effect(&mut self, loop_value: i32) {
        self.base.begin_visual_effect(loop_value);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CVisualEffect {
    ended: bool,
    loop_value: i32,
}

impl CVisualEffect {
    pub(crate) const fn new() -> Self {
        Self {
            ended: false,
            loop_value: 0,
        }
    }

    pub(crate) const fn is_ended(&self) -> bool {
        self.ended
    }

    pub(crate) const fn loop_value(&self) -> i32 {
        self.loop_value
    }

    pub(crate) const fn begin_visual_effect(&mut self, loop_value: i32) {
        self.ended = false;
        self.loop_value = loop_value;
    }

    pub(crate) const fn end_visual_effect(&mut self) {
        self.ended = true;
    }

    pub(crate) const fn update_visual_effect(&mut self) {
        if self.loop_value == 0 {
            self.end_visual_effect();
        }
    }
}

impl Default for CVisualEffect {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CVisualEffect {
    fn drop(&mut self) {
        self.ended = true;
        self.loop_value = 0;
    }
}
