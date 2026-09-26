//! Исполнение зарегистрированного навыка Zone: typed payload игрока и боевого
//! духа, payload исполнения монстра с progress-каталогом и полная запись
//! реестра со скалярной базой `SkillIdentity`; generic-сварки записи открыты
//! переходным hub-владельцам.

mod battlefairy;
mod monster;
mod payload;
mod player;
mod record;

pub use battlefairy::BattleFairyExecution;
pub use monster::{
    MonsterBaseAttackCast, MonsterSkillExecution, MonsterSkillProgress, MonsterSkillProgressAccess,
};
pub use payload::{
    ArmyBreakExecutionState, ArrowTargetIdentity, BaseProjectileExecutionState,
    BaseProjectileProgress, BattleFairyBaseMagicExecutionState, BoaLockExecutionState,
    BossFiendPenetrateProgress, ChainLightningExecutionState, ChainLightningProgress,
    DirectProjectileProgress, FatalBlowExecutionState, FlashExecutionState,
    GhostCutExecutionState, HeartlessArrowAreaExecutionState, HeartlessArrowExecutionState,
    KnightCutExecutionState, LightingArrow2ExecutionState, LightingArrowExecutionState,
    LightningExecutionState, LightningProgress, LittleFlashExecutionState, LittleStarProgress,
    LordFastAttackExecutionState, MonsterFastAttackProgress, PathProjectileProgress,
    PathProjectileScope, PlayerBossBlueQuakeExecutionState, PlayerBossFiendPenetrateExecutionState,
    PlayerLittleStarExecutionState, PlayerMonsterThornExecutionState, PlayerSpiderMistExecutionState,
    PlayerSpiderWebExecutionState, PlayerSummonCreatureExecutionState,
    PlayerYunShengLightningExecutionState, PoisonMothExecutionState, RageExecutionState,
    RainArrowCell, RainArrowExecutionState, RainArrowPath, RushExecutionState,
    ScorpionExecutionState, ScopedArrowExecutionState, SevenShootingStarExecutionState,
    SpiderMistProgress, SpiderWebProgress, SpriteBurnExecutionState, SwallowExecutionState,
    TargetedProjectileExecutionState, TargetedProjectileProgress, ThunderBlow2Execution,
    YunShengLightningProgress,
};
pub use player::{PlayerSkillExecution, PlayerSkillState};
pub use record::{
    MonsterSkillExecutionAccess, MonsterSkillProgressState, RegisteredSkillDispatch,
    RegisteredSkillExecution, RegisteredSkillRecord, SkillRetainedData,
};

/// Полная запись зарегистрированного навыка фигуры: скалярная база `SkillIdentity`
/// (`regions/skillregistry`), execution kernel и retained данные полёта; оба операнда
/// специализации — типы этого компонента, hub сохраняет только re-export прежнего имени.
pub type MoveShapeSkill = RegisteredSkillRecord<MonsterSkillExecution>;
