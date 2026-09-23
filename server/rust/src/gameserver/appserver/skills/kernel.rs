//! Адаптеры конкретных исполнений игрока и боевого духа в прежнем Game.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/skill.cpp/.h
//! и конкретные appserver/skills/*.cpp/.h.
//! База, стадии и единственный kernel принадлежат `zone/skills/lifecycle.rs`.
//! Каталоги ниже связывают concrete payload с этой базой и с узкими End-hooks;
//! их наличие само по себе не запускает Begin, End или клиентский visual.
//! BF End(int) VA 0x00516FB0/0x0051A700/0x005222A0 очищает DWORD-фазу
//! перед visual и AfterUse; End(bool) VA 0x0051BE50/0x005246C0
//! очищает BYTE-флаги, но не заменяет End(int). Конкретный владелец
//! выбирает соответствующий пролог. Числовые правила текста стоимости MP
//! и повторного применения теперь находятся в Zone.

use crate::gameserver::appserver::player::{BattleFairySkillDispatch, PlayerSkillDispatch};

use super::baseprojectilecast::BaseProjectileExecutionState;
use super::armybreak::ArmyBreakExecutionState;
use super::battlefairybasemagic::BattleFairyBaseMagicExecutionState;
use super::scopedarrowcast::ScopedArrowExecutionState;
use super::boalock::BoaLockExecutionState;
use super::bossbluequake::PlayerBossBlueQuakeExecutionState;
use super::bossfiendpenetrate::PlayerBossFiendPenetrateExecutionState;
use super::chainlightning::ChainLightningExecutionState;
use super::fatalblow::FatalBlowExecutionState;
use super::flash::FlashExecutionState;
use super::ghostcut::GhostCutExecutionState;
use super::heartlessarrow::HeartlessArrowExecutionState;
use super::heartlessarrow2::HeartlessArrowAreaExecutionState;
use super::knightcut::KnightCutExecutionState;
use super::lightingarrow::LightingArrowExecutionState;
use super::lightingarrow2::LightingArrow2ExecutionState;
use super::lightning::LightningExecutionState;
use super::littleflash::LittleFlashExecutionState;
use super::littlestar::PlayerLittleStarExecutionState;
use super::lordfastattack::LordFastAttackExecutionState;
use super::monsterthorn::PlayerMonsterThornExecutionState;
use super::poisonmoth::PoisonMothExecutionState;
use super::rage::RageExecutionState;
use super::rainarrow::RainArrowExecutionState;
use super::rush::RushExecutionState;
use super::scorpion::ScorpionExecutionState;
use super::sevenshootingstar::SevenShootingStarExecutionState;
use super::spidermist::PlayerSpiderMistExecutionState;
use super::spiderweb::PlayerSpiderWebExecutionState;
use super::spriteburn::SpriteBurnExecutionState;
use super::targetedprojectile::TargetedProjectileExecutionState;
use super::summoncreatureskill::PlayerSummonCreatureExecutionState;
use super::swallow::SwallowExecutionState;
use super::thunderblow2::ThunderBlow2Execution;
use super::yunshenglightning::PlayerYunShengLightningExecutionState;

// Типы игровых данных перечислены один раз: из них выводятся хранение,
// доступ к kernel и безопасное извлечение конкретного состояния.
macro_rules! player_skill_states {
    ($($variant:ident($state:ty) $(prepare($prepare:ident))? $(paths($paths:ident))?),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) enum PlayerSkillExecution {
            State(SkillExecutionKernel<PlayerSkillDispatch>),
            $($variant($state),)+
        }

        impl PlayerSkillExecution {
            pub(crate) fn kernel(&self) -> SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) => *kernel,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
                match self {
                    Self::State(kernel) => kernel,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(kernel) => kernel.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }

            pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
                match self {
                    Self::State(_) => true,
                    $(Self::$variant(_state) => player_skill_states!(@prepare _state, _argument $(, $prepare)?),)+
                }
            }

            pub(crate) fn clear_end_paths(&mut self) {
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

pub(crate) trait PlayerSkillState {
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

macro_rules! battle_fairy_skill_states {
    ($($variant:ident($state:ty) $(prepare($prepare:ident))?),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum BattleFairyExecution {
            State(SkillExecutionKernel<BattleFairySkillDispatch>),
            $($variant($state),)+
        }

        impl BattleFairyExecution {
            pub(crate) fn kernel(&self) -> SkillExecutionKernel<BattleFairySkillDispatch> {
                match self {
                    Self::State(state) => *state,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
                match self {
                    Self::State(state) => state,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub(crate) fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(state) => state.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub(crate) fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }

            pub(crate) fn prepare_derived_end(&mut self) {
                self.kernel_mut().clear_phase_for_end();
                match self {
                    Self::State(_) => {},
                    $(Self::$variant(_state) => battle_fairy_skill_states!(@prepare _state $(, $prepare)?),)+
                }
            }
        }

        $(
            impl From<$state> for BattleFairyExecution {
                fn from(state: $state) -> Self { Self::$variant(state) }
            }
        )+
    };
    (@prepare $state:ident) => { {} };
    (@prepare $state:ident, $method:ident) => { $state.$method() };
}

battle_fairy_skill_states! {
    BaseMagic(BattleFairyBaseMagicExecutionState),
    FatalBlow(FatalBlowExecutionState) prepare(prepare_derived_end),
}

pub(crate) use nebokrai_zone::skills::{
    SkillExecutionKernel, SkillLifecycle, SkillStage, SkillTermination,
    battle_fairy_mana_text_cost, skill_is_restored,
};
