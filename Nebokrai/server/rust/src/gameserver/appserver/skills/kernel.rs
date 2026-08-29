//! Общий исполняемый конвейер навыков GameServer.
//!
//! Конвейер хранит только порядок подтверждённых стадий
//! `Begin → Check → Calculate → Attack → Apply`, исходную команду и часы
//! начала. Формулы, RNG и особые сетевые действия остаются у конкретного
//! владельца навыка. Отложенные межвладельческие действия формируются до
//! постановки команды через `GameEffectJournal`; уже выполняемые синхронно
//! боевые действия в журнал не копируются.

/// Точная беззнаковая проверка `CSkill::IsRestored`: сложение выполняется в
/// `u32`, после чего результат сравнивается с текущими миллисекундами. Это не
/// устойчивый к переполнению срок и потому намеренно отличается от
/// периодических часов.
pub(crate) const fn skill_is_restored(
    last_used_ms: u32,
    reuse_delay_ms: u32,
    now_ms: u32,
) -> bool {
    last_used_ms.wrapping_add(reuse_delay_ms) <= now_ms
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SkillStage {
    Begin,
    Check,
    Calculate,
    Attack,
    Apply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillTermination {
    Completed,
    Rejected,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillExecutionKernel<Dispatch> {
    dispatch: Dispatch,
    started_at_ms: u32,
    stage: SkillStage,
    termination: Option<SkillTermination>,
}

impl<Dispatch: Copy + Eq> SkillExecutionKernel<Dispatch> {
    pub(crate) const fn begin(dispatch: Dispatch, started_at_ms: u32) -> Self {
        Self {
            dispatch,
            started_at_ms,
            stage: SkillStage::Begin,
            termination: None,
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

    pub(crate) const fn termination(self) -> Option<SkillTermination> {
        self.termination
    }

    pub(crate) fn advance(&mut self, expected: SkillStage, next: SkillStage) -> bool {
        if self.termination.is_some() || self.stage != expected || next <= expected {
            return false;
        }
        self.stage = next;
        true
    }

    pub(crate) fn terminate(&mut self, termination: SkillTermination) -> bool {
        if self.termination.is_some() {
            return false;
        }
        self.termination = Some(termination);
        true
    }
}
