//! Исполнение зарегистрированного навыка Zone. Обязанность компонента —
//! lifecycle исполнения: typed payload игрока и боевого духа (`payload`,
//! `player`, `battlefairy`), payload исполнения монстра с его
//! progress-каталогом (`monster`) и
//! полная запись реестра (`record`): скалярная база `SkillIdentity` +
//! execution + retained данные полёта с фасадами install/clear/progress/
//! advance/prepare/clear_end. Alias `MoveShapeSkill` специализирует запись
//! монстровым payload прямо здесь: оба операнда (`RegisteredSkillRecord` и
//! `MonsterSkillExecution`) — типы этого компонента, тогда как
//! `regions::skillregistry` связан с записью только generic-швом
//! `SkillIdentityAccess` и конкретную специализацию не называет.
//! Переходным hub-владельцам записи generic-сварки `MonsterSkillExecutionAccess`
//! и `MonsterSkillProgressState<M>` остаются открытой формой доступа.
//! Источник: gameserver.exe + GameServer.pdb, `appserver/moveshape.h/.cpp`,
//! `appserver/states/skill.cpp/.h` и конкретные `appserver/skills/*.cpp/.h`.

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

/// Полная запись зарегистрированного навыка фигуры: скалярная база
/// `SkillIdentity` (Zone `regions/skillregistry`), execution kernel и retained
/// данные полёта. Раньше alias жил в hub `appserver/moveshape.rs` из-за
/// hub-владения payload монстра; теперь оба операнда специализации принадлежат
/// этому компоненту, и hub сохраняет только re-export прежнего имени.
pub type MoveShapeSkill = RegisteredSkillRecord<MonsterSkillExecution>;
