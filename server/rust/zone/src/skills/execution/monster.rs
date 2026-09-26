//! Исполнение боевого навыка монстра в Zone; сформировано растворением
//! hub-сварки. Раньше enum-каталог `MonsterSkillProgress` и обёртка
//! `MonsterSkillExecution` (kernel + typed progress) жили в hub
//! `appserver/monster.rs`, а `impl MonsterSkillExecutionAccess` и каталог
//! impl-ов `MonsterSkillProgressState<M>` — в `appserver/moveshape.rs`; запись
//! Zone `RegisteredSkillRecord<M>` связывалась с этими hub-данными только
//! generic-трейтами. Теперь payload исполнения монстра — данные этого
//! компонента, alias `MoveShapeSkill` (в `skills/execution/mod.rs`)
//! специализирует запись монстра-владельцем, а hub `appserver` сохраняет
//! re-export прежних имён без правок потребителей. Тела каталога, обоих
//! макросов и сварки перенесены буквально; hub-пути payload заменены прямыми
//! Zone-типами `execution/payload`. Источник: gameserver.exe + GameServer.pdb,
//! `appserver/monster.h/.cpp`, `appserver/moveshape.h/.cpp` и конкретные
//! `appserver/skills/*.cpp/.h`.
//!
//! Kernel и типизированный ресурс cast принадлежат `MonsterSkillExecution`
//! конкретного `MoveShapeSkill`; reuse хранится в том же зарегистрированном
//! экземпляре. Каталог вариантов прогресса задаёт и доступ, и End-hooks:
//! `prepare_derived_end` снимает фазу и подтверждённые флаги полёта без
//! удаления payload, `clear_end_paths` затем очищает owned пути в порядке
//! `SkillOwner::end_policy`. Destination у SpiderMist и YunShengLightning не
//! обнуляется вместе с derived-полётными флагами: End 0x0057B810 пишет только
//! +0x4C/+0x50/+0x54/+0x58 перед GetUser, а координаты базового CState
//! очищаются уже общим хвостом. Тип dispatch активного cast-а монстра — Zone
//! `ai/monsterai.rs`; полная запись реестра
//! навыков фигуры и скалярная база `SkillIdentity` — Zone
//! `regions/skillregistry.rs`.

use super::payload::{
    BossFiendPenetrateProgress, ChainLightningProgress, LightningProgress, LittleStarProgress,
    MonsterFastAttackProgress, SpiderMistProgress, SpiderWebProgress, TargetedProjectileProgress,
    YunShengLightningProgress,
};
use super::record::{MonsterSkillExecutionAccess, MonsterSkillProgressState};
use crate::ai::monsterai::MonsterBaseAttackDispatch;
use crate::skills::SkillExecutionKernel;

/// Ядро исполнения атаки/навыка монстра: kernel над Zone-dispatch активного
/// cast-а (`ai/monsterai`); lifecycle-фасады
/// hub-владельца сохраняют прежний контракт через re-export.
pub type MonsterBaseAttackCast = SkillExecutionKernel<MonsterBaseAttackDispatch>;

macro_rules! monster_skill_progress {
    ($($variant:ident($state:ty) $(prepare($prepare:expr))? $(paths($paths:ident))?),+ $(,)?) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum MonsterSkillProgress {
            $($variant($state)),+
        }

        /// Typed извлечение конкретного progress-состояния из enum-каталога
        /// (read/mut), выделенное из hub `appserver/monster.rs`.
        /// Форма установки — `From<$state> for MonsterSkillProgress` ниже;
        /// геттеры записи `monster_progress`/`set_monster_progress` обслуживает
        /// generic-трейт `MonsterSkillProgressState<M>` (record).
        pub trait MonsterSkillProgressAccess: Sized {
            fn from_progress(progress: &MonsterSkillProgress) -> Option<&Self>;
            fn from_progress_mut(progress: &mut MonsterSkillProgress) -> Option<&mut Self>;
        }

        impl MonsterSkillExecution {
            pub fn prepare_derived_end(&mut self) {
                self.kernel.clear_phase_for_end();
                if let Some(progress) = self.progress.as_mut() {
                    match progress {
                        $(MonsterSkillProgress::$variant(_state) =>
                            monster_skill_progress!(@prepare _state $(, $prepare)?),)+
                    }
                }
            }

            pub fn clear_end_paths(&mut self) {
                if let Some(progress) = self.progress.as_mut() {
                    match progress {
                        $(MonsterSkillProgress::$variant(_state) =>
                            monster_skill_progress!(@paths _state $(, $paths)?),)+
                    }
                }
            }
        }

        $(
            impl From<$state> for MonsterSkillProgress {
                fn from(state: $state) -> Self {
                    Self::$variant(state)
                }
            }

            impl MonsterSkillProgressAccess for $state {
                fn from_progress(progress: &MonsterSkillProgress) -> Option<&Self> {
                    if let MonsterSkillProgress::$variant(state) = progress {
                        Some(state)
                    } else {
                        None
                    }
                }

                fn from_progress_mut(progress: &mut MonsterSkillProgress) -> Option<&mut Self> {
                    if let MonsterSkillProgress::$variant(state) = progress {
                        Some(state)
                    } else {
                        None
                    }
                }
            }
        )+
    };
    (@prepare $state:ident) => { {} };
    (@prepare $state:ident, $prepare:expr) => { ($prepare)($state) };
    (@paths $state:ident) => { {} };
    (@paths $state:ident, $method:ident) => { $state.$method() };
}

monster_skill_progress! {
    FastAttack(MonsterFastAttackProgress)
        prepare(|state: &mut MonsterFastAttackProgress| *state = MonsterFastAttackProgress::default()),
    TargetedProjectile(TargetedProjectileProgress)
        prepare(|state: &mut TargetedProjectileProgress| *state = TargetedProjectileProgress::default()),
    Lightning(LightningProgress)
        prepare(|state: &mut LightningProgress| *state = LightningProgress::default()),
    ChainLightning(ChainLightningProgress) paths(clear_end_paths),
    BossFiendPenetrate(BossFiendPenetrateProgress) paths(clear_end_paths),
    LittleStar(LittleStarProgress) paths(clear_end_paths),
    SpiderWeb(SpiderWebProgress)
        prepare(|state: &mut SpiderWebProgress| *state = SpiderWebProgress::new(0)),
    SpiderMist(SpiderMistProgress),
    YunShengLightning(YunShengLightningProgress)
        prepare(|state: &mut YunShengLightningProgress| {
            let (x, y) = state.destination();
            *state = YunShengLightningProgress::new(x, y);
        }),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonsterSkillExecution {
    pub kernel: MonsterBaseAttackCast,
    pub progress: Option<MonsterSkillProgress>,
}

/// Typed извлечение и установка конкретного progress-состояния внутри Zone
/// `MonsterSkillExecution`: перечислены все девять вариантов каталога,
/// чтобы обобщённые фасады записи (`monster_progress`/`set_monster_progress`)
/// покрывали его полностью, не теряя ни одного владельца.
macro_rules! monster_skill_progress_states {
    ($($variant:ident($state:ty)),+ $(,)?) => {$ (
        impl MonsterSkillProgressState<MonsterSkillExecution> for $state {
            fn from_execution(execution: &MonsterSkillExecution) -> Option<&Self> {
                match execution.progress.as_ref()? {
                    MonsterSkillProgress::$variant(state) => Some(state),
                    _ => None,
                }
            }

            fn install(execution: &mut MonsterSkillExecution, progress: Self) {
                execution.progress = Some(MonsterSkillProgress::$variant(progress));
            }
        }
    )+};
}

monster_skill_progress_states! {
    FastAttack(MonsterFastAttackProgress),
    TargetedProjectile(TargetedProjectileProgress),
    Lightning(LightningProgress),
    ChainLightning(ChainLightningProgress),
    BossFiendPenetrate(BossFiendPenetrateProgress),
    LittleStar(LittleStarProgress),
    SpiderWeb(SpiderWebProgress),
    SpiderMist(SpiderMistProgress),
    YunShengLightning(YunShengLightningProgress),
}

/// Сварка записи Zone `RegisteredSkillRecord<M>` с payload исполнения монстра:
/// kernel, End-hooks и три общих progress-типа извлекаются из живого каталога
/// ровно теми ветвями, которые раньше проходил enum каталог
/// `MonsterSkillProgress`; игровые ветви не дублируются (прецедент —
/// `SkillIdentityAccess`).
impl MonsterSkillExecutionAccess for MonsterSkillExecution {
    type Dispatch = MonsterBaseAttackDispatch;

    fn kernel(&self) -> &MonsterBaseAttackCast {
        &self.kernel
    }

    fn kernel_mut(&mut self) -> &mut MonsterBaseAttackCast {
        &mut self.kernel
    }

    fn prepare_derived_end(&mut self) {
        MonsterSkillExecution::prepare_derived_end(self);
    }

    fn clear_end_paths(&mut self) {
        MonsterSkillExecution::clear_end_paths(self);
    }

    fn targeted_projectile_progress(&self) -> Option<&TargetedProjectileProgress> {
        match self.progress.as_ref()? {
            MonsterSkillProgress::TargetedProjectile(state) => Some(state),
            _ => None,
        }
    }

    fn targeted_projectile_progress_mut(&mut self) -> Option<&mut TargetedProjectileProgress> {
        match self.progress.as_mut()? {
            MonsterSkillProgress::TargetedProjectile(state) => Some(state),
            _ => None,
        }
    }

    fn lightning_progress(&self) -> Option<&LightningProgress> {
        match self.progress.as_ref()? {
            MonsterSkillProgress::Lightning(state) => Some(state),
            _ => None,
        }
    }

    fn lightning_progress_mut(&mut self) -> Option<&mut LightningProgress> {
        match self.progress.as_mut()? {
            MonsterSkillProgress::Lightning(state) => Some(state),
            _ => None,
        }
    }

    fn chain_lightning_progress(&self) -> Option<&ChainLightningProgress> {
        match self.progress.as_ref()? {
            MonsterSkillProgress::ChainLightning(state) => Some(state),
            _ => None,
        }
    }

    fn chain_lightning_progress_mut(&mut self) -> Option<&mut ChainLightningProgress> {
        match self.progress.as_mut()? {
            MonsterSkillProgress::ChainLightning(state) => Some(state),
            _ => None,
        }
    }
}
