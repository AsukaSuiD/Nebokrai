//! Исполнение зарегистрированного навыка Zone (порция 5 волны moveshape).
//! Обязанность компонента — lifecycle исполнения: typed payload игрока и
//! боевого духа (`payload`, `player`, `battlefairy`) и полная запись
//! реестра (`record`): скалярная база `SkillIdentity` + execution + retained
//! данные полёта с фасадами install/clear/progress/advance/prepare/clear_end.
//! Исполнение монстра остаётся hub-владением `CMonster`; запись сваривается с
//! ним только generic-трейтами `MonsterSkillExecutionAccess` и
//! `MonsterSkillProgressState<M>` по прецеденту `StateRecordTarget` /
//! `SkillIdentityAccess`.
//! Источник: gameserver.exe + GameServer.pdb, `appserver/moveshape.h/.cpp`,
//! `appserver/states/skill.cpp/.h` и конкретные `appserver/skills/*.cpp/.h`.

mod battlefairy;
mod payload;
mod player;
mod record;

pub use battlefairy::BattleFairyExecution;
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
