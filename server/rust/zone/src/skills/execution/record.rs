//! Запись зарегистрированного навыка `CMoveShape` в Zone: скалярная база
//! `SkillIdentity`, исполнение и retained данные полёта; раньше эти поля
//! были hub-владением записи `appserver/moveshape.rs`, тела перенесены
//! буквально.
//! Источник: gameserver.exe + GameServer.pdb, `appserver/moveshape.h/.cpp`,
//! `appserver/states/skill.cpp/.h` и конкретные `appserver/skills/*.cpp/.h`.
//!
//! Исполнение игрока и боевого духа — конкретные Zone-каталоги
//! (`execution/player.rs`, `execution/battlefairy.rs`); их dispatch-типы уже
//! Zone (`skills/dispatch.rs`). Исполнение монстра — соседний
//! `execution/monster.rs` волны Z-M4 (Zone-владение `MonsterSkillExecution`
//! вместе с его progress-каталогом); запись связана с ним generic-сварками
//! `MonsterSkillExecutionAccess`
//! (kernel/стадии/End-hooks/три общих progress-типа) и
//! `MonsterSkillProgressState<M>` (typed извлечение и установка progress),
//! по прецеденту трейтов `StateRecordTarget` и `SkillIdentityAccess`.
//!
//! Исполнение и принадлежащие навыку ресурсы хранятся в единственной
//! типизированной ячейке экземпляра вместе с общим reuse timestamp. База
//! lifecycle хранится в Inactive до concrete Begin, затем перемещается внутрь
//! kernel игрока, боевого духа либо монстра. Установка concrete-данных
//! сохраняет эту базу, включая уже записанные общим Begin source/target и
//! время. Неуспешный Begin сам по себе не удаляет прежние concrete-данные.
//! Удаление только исполнения возвращает ту же базу в Inactive без Begin, End
//! и callback; завершение базы вызывается владельцем отдельно до удаления.
//! При неназначенной execution retained данные также очищаются
//! (`RegisteredSkillRecord::retained_data` всегда сбрасывается свежей
//! регистрацией через `SkillRetainedData::for_owner`).

use super::battlefairy::BattleFairyExecution;
use super::payload::{
    BaseProjectileProgress, ChainLightningProgress, DirectProjectileProgress,
    LightningProgress, PathProjectileProgress, TargetedProjectileProgress,
};
use super::player::{PlayerSkillExecution, PlayerSkillState};
use crate::regions::skillregistry::{SkillIdentity, SkillIdentityAccess};
use crate::skills::skillfactory::{CSkillFactory, SkillOwner};
use crate::skills::{
    BattleFairySkillDispatch, PlayerSkillDispatch, SkillExecutionKernel, SkillLifecycle,
    SkillStage, SkillTermination, SkillVisualEffect,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisteredSkillExecution<M> {
    Inactive(SkillLifecycle),
    Player(PlayerSkillExecution),
    BattleFairy(BattleFairyExecution),
    Monster(M),
}

/// Сварка hub-данных исполнения монстра к записи Zone: kernel этого execution
/// и его End-hooks; три общих progress-типа (прицельный снаряд, молния и
/// цепная молния) извлекаются владельцем `M` из своего progress-каталога.
pub trait MonsterSkillExecutionAccess: Sized {
    type Dispatch: Copy + Eq;

    fn kernel(&self) -> &SkillExecutionKernel<Self::Dispatch>;
    fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<Self::Dispatch>;
    fn prepare_derived_end(&mut self);
    fn clear_end_paths(&mut self);

    fn targeted_projectile_progress(&self) -> Option<&TargetedProjectileProgress>;
    fn targeted_projectile_progress_mut(&mut self) -> Option<&mut TargetedProjectileProgress>;
    fn lightning_progress(&self) -> Option<&LightningProgress>;
    fn lightning_progress_mut(&mut self) -> Option<&mut LightningProgress>;
    fn chain_lightning_progress(&self) -> Option<&ChainLightningProgress>;
    fn chain_lightning_progress_mut(&mut self) -> Option<&mut ChainLightningProgress>;
}

/// Typed извлечение и установка конкретного progress-состояния внутри
/// hub-исполнения монстра `M` (форма соответствует hub-каталогу
/// `MonsterSkillProgress`; сами варианты остаются данными `CMonster`).
pub trait MonsterSkillProgressState<M>: Sized {
    fn from_execution(execution: &M) -> Option<&Self>;
    fn install(execution: &mut M, progress: Self);
}

impl<M: MonsterSkillExecutionAccess> RegisteredSkillExecution<M> {
    pub fn lifecycle(&self) -> &SkillLifecycle {
        match self {
            Self::Inactive(lifecycle) => lifecycle,
            Self::Player(execution) => execution.lifecycle(),
            Self::BattleFairy(execution) => execution.lifecycle(),
            Self::Monster(execution) => execution.kernel().lifecycle(),
        }
    }

    pub fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        match self {
            Self::Inactive(lifecycle) => lifecycle,
            Self::Player(execution) => execution.lifecycle_mut(),
            Self::BattleFairy(execution) => execution.lifecycle_mut(),
            Self::Monster(execution) => execution.kernel_mut().lifecycle_mut(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisteredSkillDispatch {
    Player(PlayerSkillDispatch),
    BattleFairy(BattleFairySkillDispatch),
}

#[derive(Debug, Eq, PartialEq)]
pub enum SkillRetainedData {
    None,
    BaseProjectile(BaseProjectileProgress),
    PathProjectile(PathProjectileProgress),
    DirectProjectile(DirectProjectileProgress),
}

impl SkillRetainedData {
    pub fn for_owner(owner: SkillOwner) -> Self {
        match owner {
            SkillOwner::CArchery | SkillOwner::CBaseMagic | SkillOwner::CFireBolt =>
                Self::BaseProjectile(Default::default()),
            SkillOwner::CEnergyBolt | SkillOwner::CSnakeBolt | SkillOwner::CZombieClaw =>
                Self::PathProjectile(Default::default()),
            SkillOwner::CChuckStone | SkillOwner::CSkeletonArchery =>
                Self::DirectProjectile(Default::default()),
            _ => Self::None,
        }
    }
}

/// Достигнутая common-проекция `CSkill`: скалярная база (identity, level,
/// concrete owner, item position, reuse, owned visual) живёт в Zone
/// `regions/skillregistry` типом `SkillIdentity`, а исполнение и retained
/// данные полёта — в этой записи. Алгоритмы concrete attack/defense/state/
/// summon остаются у skill owners; исполнение, его ресурсы и reuse
/// принадлежат каждому экземпляру.
#[derive(Debug, Eq, PartialEq)]
pub struct RegisteredSkillRecord<M> {
    identity: SkillIdentity,
    execution: RegisteredSkillExecution<M>,
    retained_data: SkillRetainedData,
}

/// Связка записи с реестром Zone: правила реестра смотрят только в скалярную
/// identity, execution/retained принадлежат этой записи (прежний impl — у
/// владельца записи в старом пакете, до переноса записи в Zone).
impl<M: MonsterSkillExecutionAccess> SkillIdentityAccess for RegisteredSkillRecord<M> {
    fn identity(&self) -> &SkillIdentity {
        &self.identity
    }

    fn identity_mut(&mut self) -> &mut SkillIdentity {
        &mut self.identity
    }
}

impl<M: MonsterSkillExecutionAccess> RegisteredSkillRecord<M> {
    /// Свежая регистрация: Inactive с общей базой lifecycle и retained
    /// выбором concrete owner-а (конструктор `CSkill` обнуляет timestamp
    /// +0x40; новая запись не наследует его от удалённого того же ID).
    pub fn registered(id: u32, level: i32, owner: SkillOwner) -> Self {
        Self {
            identity: SkillIdentity::new_registered(id, level, owner),
            execution: RegisteredSkillExecution::Inactive(SkillLifecycle::default()),
            retained_data: SkillRetainedData::for_owner(owner),
        }
    }

    pub const fn owner(&self) -> SkillOwner {
        self.identity.owner()
    }

    pub fn lifecycle(&self) -> &SkillLifecycle {
        self.execution.lifecycle()
    }

    pub fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        self.execution.lifecycle_mut()
    }

    pub fn player_state<State: PlayerSkillState>(&self) -> Option<&State> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) => State::from_execution(execution),
            _ => None,
        }
    }

    pub fn player_state_mut<State: PlayerSkillState>(&mut self) -> Option<&mut State> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => State::from_execution_mut(execution),
            _ => None,
        }
    }

    /// Один payload полёта находится в активной ветви исполнения владельца.
    /// Player и Monster не копируют его друг у друга при callbacks.
    pub fn targeted_projectile_progress(&self) -> Option<&TargetedProjectileProgress> {
        match &self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::TargetedProjectile(state)) => Some(state.progress()),
            RegisteredSkillExecution::Monster(execution) => execution.targeted_projectile_progress(),
            _ => None,
        }
    }

    pub fn targeted_projectile_progress_mut(&mut self) -> Option<&mut TargetedProjectileProgress> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::TargetedProjectile(state)) => Some(state.progress_mut()),
            RegisteredSkillExecution::Monster(execution) => execution.targeted_projectile_progress_mut(),
            _ => None,
        }
    }

    pub fn path_projectile_progress(&self) -> Option<&PathProjectileProgress> {
        match &self.retained_data {
            SkillRetainedData::PathProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub fn direct_projectile_progress(&self) -> Option<&DirectProjectileProgress> {
        match &self.retained_data {
            SkillRetainedData::DirectProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub fn direct_projectile_progress_mut(&mut self) -> Option<&mut DirectProjectileProgress> {
        match &mut self.retained_data {
            SkillRetainedData::DirectProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub fn path_projectile_progress_mut(&mut self) -> Option<&mut PathProjectileProgress> {
        match &mut self.retained_data {
            SkillRetainedData::PathProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub fn lightning_progress(&self) -> Option<&LightningProgress> {
        match &self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::Lightning(state)) => Some(state.progress()),
            RegisteredSkillExecution::Monster(execution) => execution.lightning_progress(),
            _ => None,
        }
    }

    pub fn lightning_progress_mut(&mut self) -> Option<&mut LightningProgress> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::Lightning(state)) => Some(state.progress_mut()),
            RegisteredSkillExecution::Monster(execution) => execution.lightning_progress_mut(),
            _ => None,
        }
    }

    pub fn chain_lightning_progress(&self) -> Option<&ChainLightningProgress> {
        match &self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::ChainLightning(state)) => Some(state.progress()),
            RegisteredSkillExecution::Monster(execution) => execution.chain_lightning_progress(),
            _ => None,
        }
    }

    pub fn chain_lightning_progress_mut(&mut self) -> Option<&mut ChainLightningProgress> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(PlayerSkillExecution::ChainLightning(state)) => Some(state.progress_mut()),
            RegisteredSkillExecution::Monster(execution) => execution.chain_lightning_progress_mut(),
            _ => None,
        }
    }

    pub fn execution_stage(&self) -> Option<SkillStage> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) => Some(execution.kernel().stage()),
            RegisteredSkillExecution::Monster(execution) => Some(execution.kernel().stage()),
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution.kernel().stage()),
            RegisteredSkillExecution::Inactive(_) => None,
        }
    }

    pub fn base_projectile_progress(&self) -> Option<&BaseProjectileProgress> {
        match &self.retained_data {
            SkillRetainedData::BaseProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub fn base_projectile_progress_mut(&mut self) -> Option<&mut BaseProjectileProgress> {
        match &mut self.retained_data {
            SkillRetainedData::BaseProjectile(progress) => Some(progress),
            _ => None,
        }
    }

    pub fn advance_execution(
        &mut self,
        from: SkillStage,
        to: SkillStage,
    ) -> bool {
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => execution.kernel_mut().advance(from, to),
            RegisteredSkillExecution::Monster(execution) => execution.kernel_mut().advance(from, to),
            RegisteredSkillExecution::BattleFairy(execution) => execution.kernel_mut().advance(from, to),
            RegisteredSkillExecution::Inactive(_) => false,
        }
    }

    pub fn monster_kernel(&self) -> Option<&SkillExecutionKernel<M::Dispatch>> {
        match &self.execution {
            RegisteredSkillExecution::Monster(execution) => Some(execution.kernel()),
            _ => None,
        }
    }

    pub fn monster_kernel_mut(&mut self) -> Option<&mut SkillExecutionKernel<M::Dispatch>> {
        match &mut self.execution {
            RegisteredSkillExecution::Monster(execution) => Some(execution.kernel_mut()),
            _ => None,
        }
    }

    pub fn monster_progress<State: MonsterSkillProgressState<M>>(&self) -> Option<&State> {
        match &self.execution {
            RegisteredSkillExecution::Monster(execution) => State::from_execution(execution),
            _ => None,
        }
    }

    pub fn set_monster_progress<State: MonsterSkillProgressState<M>>(&mut self, progress: State) {
        if let RegisteredSkillExecution::Monster(execution) = &mut self.execution {
            State::install(execution, progress);
        }
    }

    pub fn execution_dispatch(&self) -> Option<RegisteredSkillDispatch> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) =>
                Some(RegisteredSkillDispatch::Player(execution.kernel().dispatch())),
            RegisteredSkillExecution::BattleFairy(execution) =>
                Some(RegisteredSkillDispatch::BattleFairy(execution.kernel().dispatch())),
            RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Monster(_) => None,
        }
    }

    pub fn player_dispatch(&self) -> Option<PlayerSkillDispatch> {
        match self.execution_dispatch() {
            Some(RegisteredSkillDispatch::Player(dispatch)) => Some(dispatch),
            _ => None,
        }
    }

    pub fn battle_fairy_dispatch(&self) -> Option<BattleFairySkillDispatch> {
        match self.execution_dispatch() {
            Some(RegisteredSkillDispatch::BattleFairy(dispatch)) => Some(dispatch),
            _ => None,
        }
    }

    /// Доступ к payload исполнения игрока для typed фасадов его владельца.
    pub fn player_execution(&self) -> Option<&PlayerSkillExecution> {
        match &self.execution {
            RegisteredSkillExecution::Player(execution) => Some(execution),
            _ => None,
        }
    }

    pub fn player_execution_mut(&mut self) -> Option<&mut PlayerSkillExecution> {
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => Some(execution),
            _ => None,
        }
    }

    pub fn battle_fairy_execution_state(&self) -> Option<&BattleFairyExecution> {
        match &self.execution {
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution),
            _ => None,
        }
    }

    pub fn battle_fairy_execution_state_mut(&mut self) -> Option<&mut BattleFairyExecution> {
        match &mut self.execution {
            RegisteredSkillExecution::BattleFairy(execution) => Some(execution),
            _ => None,
        }
    }

    /// Материализация сохраняет уже начатую базу именно этого экземпляра.
    pub fn install_player_execution(&mut self, mut execution: PlayerSkillExecution) -> bool {
        if execution.kernel().dispatch().skill_id() != self.id()
            || !matches!(self.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Player(_))
        {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        self.execution = RegisteredSkillExecution::Player(execution);
        true
    }

    /// Материализация сохраняет уже начатую базу именно этого экземпляра.
    pub fn install_battle_fairy_execution(&mut self, mut execution: BattleFairyExecution) -> bool {
        if execution.kernel().dispatch().skill_id() != self.id()
            || !matches!(self.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_))
        {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        self.execution = RegisteredSkillExecution::BattleFairy(execution);
        true
    }

    /// Материализация сохраняет уже начатую базу именно этого экземпляра;
    /// ID исполнения монстра уже разрешил этот экземпляр внешний поиск.
    pub fn install_monster_execution(&mut self, mut execution: M) -> bool {
        if !matches!(self.execution, RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::Monster(_)) {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        execution.kernel_mut().replace_lifecycle(lifecycle);
        self.execution = RegisteredSkillExecution::Monster(execution);
        true
    }

    /// Убирает только payload этого экземпляра, без повторного поиска по ID.
    /// Общая база, visual и reuse не получают дополнительных End-переходов.
    pub fn clear_monster_execution(&mut self) -> bool {
        if !matches!(self.execution, RegisteredSkillExecution::Monster(_)) {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        self.execution = RegisteredSkillExecution::Inactive(lifecycle);
        true
    }

    /// Убирает только payload этого экземпляра, без повторного поиска по ID.
    /// Общая база, visual и reuse не получают дополнительных End-переходов.
    pub fn clear_execution(&mut self, expected: RegisteredSkillDispatch) -> bool {
        if self.execution_dispatch() != Some(expected) {
            return false;
        }
        let lifecycle = std::mem::take(self.execution.lifecycle_mut());
        self.execution = RegisteredSkillExecution::Inactive(lifecycle);
        true
    }

    pub fn is_execution_inactive(&self) -> bool {
        matches!(self.execution, RegisteredSkillExecution::Inactive(_))
    }

    /// Доступ к payload исполнения монстра для typed фасадов его владельца.
    pub fn monster_payload(&self) -> Option<&M> {
        match &self.execution {
            RegisteredSkillExecution::Monster(execution) => Some(execution),
            _ => None,
        }
    }

    pub fn monster_payload_mut(&mut self) -> Option<&mut M> {
        match &mut self.execution {
            RegisteredSkillExecution::Monster(execution) => Some(execution),
            _ => None,
        }
    }

    pub fn prepare_derived_end(&mut self, argument: i32) -> bool {
        let owner = self.owner();
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => {
                if owner.end_policy().reset_phase {
                    execution.kernel_mut().clear_phase_for_end();
                }
                if !execution.prepare_derived_end(argument) {
                    return false;
                }
            }
            RegisteredSkillExecution::Monster(execution) => execution.prepare_derived_end(),
            RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_) => {}
        }
        if let SkillRetainedData::DirectProjectile(progress) = &mut self.retained_data {
            progress.prepare_derived_end();
        }
        true
    }

    pub fn clear_end_paths(&mut self) {
        if let SkillRetainedData::PathProjectile(progress) = &mut self.retained_data {
            progress.clear_end_paths();
        }
        match &mut self.execution {
            RegisteredSkillExecution::Player(execution) => execution.clear_end_paths(),
            RegisteredSkillExecution::Monster(execution) => execution.clear_end_paths(),
            RegisteredSkillExecution::Inactive(_) | RegisteredSkillExecution::BattleFairy(_) => {}
        }
    }

    pub fn mark_used(&mut self, now_ms: u32) {
        self.identity.mark_used(now_ms);
    }

    pub const fn last_used_ms(&self) -> u32 {
        self.identity.last_used_ms()
    }

    pub fn replace_visual_effect(&mut self, effect: SkillVisualEffect) {
        self.identity.replace_visual_effect(effect);
    }

    pub fn visual_effect_mut(&mut self) -> Option<&mut SkillVisualEffect> {
        self.identity.visual_effect_mut()
    }

    pub fn visual_effect(&self) -> Option<&SkillVisualEffect> {
        self.identity.visual_effect()
    }

    /// Общий хвост CSkill::End после concrete cleanup и OnEndSkill.
    /// Владеющий visual не входит в копируемый снимок скалярного lifecycle.
    pub fn finish_base(&mut self, termination: SkillTermination) {
        self.clear_base_end_context();
        self.finish_cleared_base_end(termination);
    }

    pub fn clear_base_end_context(&mut self) {
        self.execution.lifecycle_mut().clear_end_context();
    }

    /// Visual и IsEnded следуют после отдельного native reuse-clock; общая
    /// очистка принадлежит Zone-identity этой записи.
    pub fn finish_cleared_base_end(&mut self, termination: SkillTermination) {
        self.identity
            .finish_cleared_base_end(self.execution.lifecycle_mut(), termination);
    }

    pub const fn id(&self) -> u32 {
        self.identity.id()
    }

    pub const fn level(&self) -> i32 {
        self.identity.level()
    }

    pub fn minimum_range(&self, factory: &CSkillFactory) -> u32 {
        self.identity.minimum_range(factory)
    }

    pub const fn skill_type(&self) -> u32 {
        self.identity.skill_type()
    }

    pub fn name<'a>(&self, factory: &'a CSkillFactory) -> Option<&'a [u8]> {
        self.identity.name(factory)
    }

    pub const fn item_position(&self) -> i32 {
        self.identity.item_position()
    }

    pub const fn set_item_position(&mut self, position: i32) {
        self.identity.set_item_position(position);
    }
}
