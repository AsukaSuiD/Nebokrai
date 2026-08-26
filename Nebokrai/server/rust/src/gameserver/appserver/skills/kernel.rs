//! Общий исполняемый конвейер навыков GameServer.
//!
//! Конвейер хранит только порядок подтверждённых стадий
//! `Begin → Check → Calculate → Attack → Apply`, исходную команду и часы
//! начала. Формулы, RNG и особые сетевые действия остаются у конкретного
//! владельца навыка. Отложенные межвладельческие действия формируются до
//! постановки команды через `GameEffectJournal`; уже выполняемые синхронно
//! боевые действия в журнал не копируются.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SkillStage {
    Begin,
    Check,
    Calculate,
    Attack,
    Apply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillExecutionKernel<Dispatch> {
    dispatch: Dispatch,
    started_at_ms: u32,
    stage: SkillStage,
}

impl<Dispatch: Copy + Eq> SkillExecutionKernel<Dispatch> {
    pub(crate) const fn begin(dispatch: Dispatch, started_at_ms: u32) -> Self {
        Self {
            dispatch,
            started_at_ms,
            stage: SkillStage::Begin,
        }
    }

    pub(crate) const fn dispatch(self) -> Dispatch {
        self.dispatch
    }

    pub(crate) const fn started_at_ms(self) -> u32 {
        self.started_at_ms
    }

    pub(crate) const fn stage(self) -> SkillStage {
        self.stage
    }

    pub(crate) fn advance(&mut self, expected: SkillStage, next: SkillStage) -> bool {
        if self.stage != expected || next <= expected {
            return false;
        }
        self.stage = next;
        true
    }
}
