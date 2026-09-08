//! Общий исполняемый конвейер навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/states/skill.cpp`, `attackskill.cpp`, `defenseskill.cpp` и
//! `stateskill.cpp`. Подтверждённый общий контракт —
//! последовательность `Begin → Check → Calculate → Attack → Apply`, хранение
//! времени начала и одно конечное состояние выполнения. Старую C++-иерархию
//! с виртуальными конструкторами и RTTI не воспроизводим: intrinsic-категория
//! берётся из фабричного owner-каталога, имя — из актуальных свойств ID/уровня.
//! Выбранный ID принадлежит CMoveShape, команды — CPlayerAI, а путь, поворот,
//! формулы, RNG и сетевые эффекты остаются у конкретного владельца навыка.
//! Kernel описывает уже начатое исполнение, но не подменяет зарегистрированный
//! экземпляр. Полное состояние CSkill до Begin и после End, как и общий
//! registered-skill End, ещё требует соединения с реестром CMoveShape.
//! Lifecycle визуальных state-эффектов сохраняет отдельный исходный owner.
//!
//! Отложенные межвладельческие действия формируются до постановки команды
//! через `GameEffectJournal`; уже выполняемые синхронно боевые действия в
//! журнал не копируются.
//! Общий текст недостатка маны боевого духа получает fixed-point стоимость
//! через `double 0.0001` и x87-усечение; это одинаковый контракт всех
//! достигнутых concrete skill owner-ов.
//! Все достигнутые monster-skill reuse-gate вызывают общий `IsRestored` ниже,
//! включая исходный нулевой timestamp до первого применения; stage, missile
//! и periodic duration продолжают использовать elapsed-часы.
//! CState::Begin (0x005DBD70/0x005DBDD0) читает часы до OnBeginSkill.
//! При установке нового kernel расписание может передать этот ранний отсчёт,
//! чтобы проверки ресурсов не сдвигали начало каста. Вызов ограничен
//! установщиками нового исполнения в AI и привязан к dispatch. Некоторые
//! владельцы до установки уже выполняют первый переход в Check; это не
//! основание терять исходный отсчёт. Активный AI сюда повторно не входит.
//! m_bSkillPrepared — независимый флаг CSkill (+0x44), не стадия Attack:
//! OnFighting (0x005092B0) переносит подготовленный экземпляр в фон до End.
//! Конкретный owner устанавливает его в подтверждённой точке выпуска.

pub(crate) fn battle_fairy_mana_text_cost(cost: u32) -> u32 {
    (f64::from(cost) * 0.0001_f64).trunc() as i64 as u32
}

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
    prepared: bool,
}

impl<Dispatch: Copy + Eq> SkillExecutionKernel<Dispatch> {
    pub(crate) const fn begin(dispatch: Dispatch, started_at_ms: u32) -> Self {
        Self {
            dispatch,
            started_at_ms,
            stage: SkillStage::Begin,
            termination: None,
            prepared: false,
        }
    }

    pub(crate) const fn dispatch(self) -> Dispatch {
        self.dispatch
    }

    pub(crate) const fn started_at_ms(self) -> u32 {
        self.started_at_ms
    }

    pub(crate) fn inherit_scheduled_begin(&mut self, begin: Option<(Dispatch, u32)>) {
        if self.termination.is_none()
            && let Some((dispatch, started_at_ms)) = begin
            && dispatch == self.dispatch
        {
            self.started_at_ms = started_at_ms;
        }
    }

    pub(crate) const fn stage(self) -> SkillStage {
        self.stage
    }

    pub(crate) const fn termination(self) -> Option<SkillTermination> {
        self.termination
    }

    pub(crate) const fn is_prepared(self) -> bool {
        self.prepared
    }

    pub(crate) fn mark_prepared(&mut self) {
        if self.termination.is_none() {
            self.prepared = true;
        }
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
        self.prepared = false;
        true
    }
}
