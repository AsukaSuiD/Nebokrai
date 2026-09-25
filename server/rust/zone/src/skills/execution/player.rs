//! Каталог конкретных исполнений игрока в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/skill.cpp/.h
//! и конкретные appserver/skills/*.cpp/.h (адаптеры перенесены из hub
//! `appserver/skills/kernel.rs` порцией 5 волны moveshape без смены тел).
//! Из игровых данных ниже выводятся хранение, доступ к kernel и безопасное
//! извлечение конкретного состояния; сами payload — в `execution/payload.rs`.

use super::payload::{
    ArmyBreakExecutionState, BaseProjectileExecutionState, BoaLockExecutionState,
    ChainLightningExecutionState, FlashExecutionState, GhostCutExecutionState,
    HeartlessArrowAreaExecutionState, HeartlessArrowExecutionState,
    KnightCutExecutionState, LightingArrow2ExecutionState, LightingArrowExecutionState,
    LightningExecutionState, LittleFlashExecutionState, LordFastAttackExecutionState,
    PlayerBossBlueQuakeExecutionState, PlayerBossFiendPenetrateExecutionState,
    PlayerLittleStarExecutionState, PlayerMonsterThornExecutionState,
    PlayerSpiderMistExecutionState, PlayerSpiderWebExecutionState,
    PlayerSummonCreatureExecutionState, PlayerYunShengLightningExecutionState,
    PoisonMothExecutionState, RageExecutionState, RainArrowExecutionState,
    RushExecutionState, ScorpionExecutionState, ScopedArrowExecutionState,
    SevenShootingStarExecutionState, SpriteBurnExecutionState, SwallowExecutionState,
    TargetedProjectileExecutionState, ThunderBlow2Execution,
};
use crate::skills::{PlayerSkillDispatch, SkillExecutionKernel, SkillLifecycle};

// Типы игровых данных перечислены один раз: из них выводятся хранение,
// доступ к kernel и безопасное извлечение конкретного состояния.
macro_rules! player_skill_states {
    ($($variant:ident($state:ty) $(prepare($prepare:ident))? $(paths($paths:ident))?),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum PlayerSkillExecution {
            State(SkillExecutionKernel<PlayerSkillDispatch>),
            $($variant($state),)+
        }

        impl PlayerSkillExecution {
            pub fn kernel(&self) -> SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) => *kernel,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) => kernel,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(kernel) => kernel.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }

            pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
                match self {
                    Self::State(_) => true,
                    $(Self::$variant(_state) => player_skill_states!(@prepare _state, _argument $(, $prepare)?),)+
                }
            }

            pub fn clear_end_paths(&mut self) {
                match self {
                    Self::State(_) => {},
                    $(Self::$variant(_state) => player_skill_states!(@paths _state $(, $paths)?),)+
                }
            }
        }

        $(
            impl From<$state> for PlayerSkillExecution {
                fn from(state: $state) -> Self { Self::$variant(state) }
            }

            impl PlayerSkillState for $state {
                fn from_execution(execution: &PlayerSkillExecution) -> Option<&Self> {
                    match execution {
                        PlayerSkillExecution::$variant(state) => Some(state),
                        _ => None,
                    }
                }

                fn from_execution_mut(execution: &mut PlayerSkillExecution) -> Option<&mut Self> {
                    match execution {
                        PlayerSkillExecution::$variant(state) => Some(state),
                        _ => None,
                    }
                }
            }
        )+
    };
    (@prepare $state:ident, $argument:ident) => { true };
    (@prepare $state:ident, $argument:ident, $method:ident) => { $state.$method($argument) };
    (@paths $state:ident) => { {} };
    (@paths $state:ident, $method:ident) => { $state.$method() };
}

pub trait PlayerSkillState {
    fn from_execution(execution: &PlayerSkillExecution) -> Option<&Self>;
    fn from_execution_mut(execution: &mut PlayerSkillExecution) -> Option<&mut Self>;
}

player_skill_states! {
    HeartlessArrowArea(HeartlessArrowAreaExecutionState) prepare(prepare_derived_end),
    ScopedArrow(ScopedArrowExecutionState) paths(clear_end_paths),
    GhostCut(GhostCutExecutionState) paths(clear_end_paths),
    ThunderBlow2(ThunderBlow2Execution) prepare(prepare_derived_end),
    ArmyBreak(ArmyBreakExecutionState) prepare(prepare_derived_end),
    LittleFlash(LittleFlashExecutionState) paths(clear_end_paths),
    SummonCreature(PlayerSummonCreatureExecutionState),
    LordFastAttack(LordFastAttackExecutionState),
    BaseProjectile(BaseProjectileExecutionState),
    HeartlessArrow(HeartlessArrowExecutionState) prepare(prepare_derived_end),
    LightingArrow(LightingArrowExecutionState) paths(clear_end_paths),
    LightingArrow2(LightingArrow2ExecutionState) paths(clear_end_paths),
    RainArrow(RainArrowExecutionState) paths(clear_end_paths),
    PoisonMoth(PoisonMothExecutionState) paths(clear_end_paths),
    Scorpion(ScorpionExecutionState) prepare(prepare_derived_end),
    BoaLock(BoaLockExecutionState) prepare(prepare_derived_end),
    TargetedProjectile(TargetedProjectileExecutionState) prepare(prepare_derived_end),
    ChainLightning(ChainLightningExecutionState) paths(clear_end_paths),
    KnightCut(KnightCutExecutionState),
    Rage(RageExecutionState),
    Flash(FlashExecutionState) paths(clear_end_paths),
    Rush(RushExecutionState) paths(clear_end_paths),
    Swallow(SwallowExecutionState) prepare(prepare_derived_end),
    SevenShootingStar(SevenShootingStarExecutionState) paths(clear_end_paths),
    LittleStar(PlayerLittleStarExecutionState) paths(clear_end_paths),
    YunshengLightning(PlayerYunShengLightningExecutionState),
    MonsterThorn(PlayerMonsterThornExecutionState),
    SpiderMist(PlayerSpiderMistExecutionState),
    SpiderWeb(PlayerSpiderWebExecutionState) prepare(prepare_derived_end),
    BossBlueQuake(PlayerBossBlueQuakeExecutionState),
    BossFiendPenetrate(PlayerBossFiendPenetrateExecutionState) paths(clear_end_paths),
    SpriteBurn(SpriteBurnExecutionState),
    Lightning(LightningExecutionState) prepare(prepare_derived_end),
}

impl From<SkillExecutionKernel<PlayerSkillDispatch>> for PlayerSkillExecution {
    fn from(state: SkillExecutionKernel<PlayerSkillDispatch>) -> Self { Self::State(state) }
}
