//! Каталог конкретных исполнений боевого духа в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/skill.cpp/.h
//! и конкретные appserver/skills/*.cpp/.h (адаптер перенесён из hub
//! `appserver/skills/kernel.rs` порцией 5 волны moveshape без смены тел).
//! End(int) перед конкретной очисткой сначала очищает DWORD-фазу kernel.

use super::payload::{BattleFairyBaseMagicExecutionState, FatalBlowExecutionState};
use crate::skills::{BattleFairySkillDispatch, SkillExecutionKernel, SkillLifecycle};

macro_rules! battle_fairy_skill_states {
    ($($variant:ident($state:ty) $(prepare($prepare:ident))?),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum BattleFairyExecution {
            State(SkillExecutionKernel<BattleFairySkillDispatch>),
            $($variant($state),)+
        }

        impl BattleFairyExecution {
            pub fn kernel(&self) -> SkillExecutionKernel<BattleFairySkillDispatch> {
                match self {
                    Self::State(state) => *state,
                    $(Self::$variant(state) => *state.kernel(),)+
                }
            }

            pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
                match self {
                    Self::State(state) => state,
                    $(Self::$variant(state) => state.kernel_mut(),)+
                }
            }

            pub fn lifecycle(&self) -> &SkillLifecycle {
                match self {
                    Self::State(state) => state.lifecycle(),
                    $(Self::$variant(state) => state.kernel().lifecycle(),)+
                }
            }

            pub fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
                self.kernel_mut().lifecycle_mut()
            }

            pub fn prepare_derived_end(&mut self) {
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
