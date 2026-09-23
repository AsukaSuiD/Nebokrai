//! База и стадии живого навыка Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/state.cpp/.h
//! и appserver/states/skill.cpp/.h; CState ctor/Begin VA 0x005DBCA0,
//! 0x005DBD70/0x005DBDD0; CSkill Begin/End VA 0x004D83E0/0x004D84C0.
//! SkillStage и причина завершения — внутренняя модель Rust, не native-поля.
//! Native Begin сохраняет сторону при NULL и обновляет часы только при U;
//! point-ветвь очищает S. Отказ callback не откатывает базу. End очищает
//! участников до удаления visual, затем отмечает завершение; полный
//! concrete End и ресурс visual выполняет вызывающий владелец.

use crate::regions::ShapeIdentity;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SkillStage {
    Idle,
    Begin,
    Check,
    Calculate,
    Attack,
    Apply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillTermination {
    Completed,
    Rejected,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillLifecycle {
    user: (i32, ShapeIdentity),
    sufferer: (i32, ShapeIdentity),
    destination: (i32, i32),
    started_at_ms: u32,
    ended: bool,
    available: bool,
    prepared: bool,
    termination: Option<SkillTermination>,
}

impl Default for SkillLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillLifecycle {
    const EMPTY_IDENTITY: ShapeIdentity = ShapeIdentity {
        object_type: 0,
        id: 0,
        ex_id: CGuid::GUID_INVALID,
    };

    const fn new() -> Self {
        Self {
            user: (0, Self::EMPTY_IDENTITY),
            sufferer: (0, Self::EMPTY_IDENTITY),
            destination: (0, 0),
            started_at_ms: 0,
            ended: true,
            available: true,
            prepared: false,
            termination: None,
        }
    }

    const fn for_execution(started_at_ms: u32) -> Self {
        Self {
            started_at_ms,
            ended: false,
            ..Self::new()
        }
    }

    const fn native_identity(identity: ShapeIdentity) -> ShapeIdentity {
        ShapeIdentity {
            ex_id: CGuid::GUID_INVALID,
            ..identity
        }
    }

    pub const fn user(&self) -> (i32, ShapeIdentity) {
        self.user
    }

    pub const fn sufferer(&self) -> (i32, ShapeIdentity) {
        self.sufferer
    }

    /// Переназначение цели внутри AI меняет только её type/id, без нового
    /// Begin, сброса координат или замены региона и времени исходной базы.
    pub fn set_sufferer_identity(&mut self, identity: ShapeIdentity) {
        self.sufferer.1 = Self::native_identity(identity);
    }

    /// Прямой снаряд после выбора клетки вновь закрепляет найденного GetS,
    /// сохраняя записанную точку и время исходного Begin.
    pub fn set_sufferer(&mut self, region_id: i32, identity: ShapeIdentity) {
        self.sufferer = (region_id, Self::native_identity(identity));
    }

    pub const fn destination(&self) -> (i32, i32) {
        self.destination
    }

    pub fn set_destination(&mut self, destination: (i32, i32)) {
        self.destination = destination;
    }

    /// Фиксирует точку вместо объекта без нового Begin и повторного чтения часов.
    pub fn set_point_target(&mut self, destination: (i32, i32)) {
        self.sufferer = (0, Self::EMPTY_IDENTITY);
        self.destination = destination;
    }

    pub const fn started_at_ms(&self) -> u32 {
        self.started_at_ms
    }

    pub const fn is_ended(&self) -> bool {
        self.ended
    }

    pub const fn is_available(&self) -> bool {
        self.available
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }

    pub const fn is_prepared(&self) -> bool {
        self.prepared
    }

    pub fn mark_prepared(&mut self) {
        self.prepared = true;
    }

    pub const fn termination(&self) -> Option<SkillTermination> {
        self.termination
    }

    /// Записи CState предшествуют OnBeginSkill. Нулевой объектный аргумент
    /// сохраняет соответствующую прежнюю сторону, а не очищает её.
    pub fn begin_objects(
        &mut self,
        source: Option<(i32, ShapeIdentity)>,
        target: Option<(i32, ShapeIdentity)>,
        now: impl FnOnce() -> u32,
    ) {
        if let Some((region_id, identity)) = source {
            self.started_at_ms = now();
            self.user = (region_id, Self::native_identity(identity));
        }
        if let Some((region_id, identity)) = target {
            self.sufferer = (region_id, Self::native_identity(identity));
            self.destination = (0, 0);
        }
        self.ended = false;
        self.termination = None;
    }

    pub fn begin_point(
        &mut self,
        source: (i32, ShapeIdentity),
        destination: (i32, i32),
        now: impl FnOnce() -> u32,
    ) {
        self.started_at_ms = now();
        self.user = (source.0, Self::native_identity(source.1));
        self.set_point_target(destination);
        self.ended = false;
        self.termination = None;
    }

    pub fn finish_begin(&mut self, success: bool) -> bool {
        if success {
            self.prepared = false;
        }
        success
    }

    /// Сброс полей и удаление visual после внешних OnEndSkill/concrete cleanup.
    /// Повторный вызов допустим; этот метод сам не исполняет полный native End.
    pub fn reset_after_end(&mut self, termination: SkillTermination, clear_visual: impl FnOnce()) {
        self.clear_end_context();
        self.finish_end(termination, clear_visual);
    }

    /// CSkill::End очищает участников и prepared до чтения часов reuse.
    pub fn clear_end_context(&mut self) {
        self.user = (0, Self::EMPTY_IDENTITY);
        self.sufferer = (0, Self::EMPTY_IDENTITY);
        self.destination = (0, 0);
        self.started_at_ms = 0;
        self.prepared = false;
    }

    pub fn finish_end(&mut self, termination: SkillTermination, clear_visual: impl FnOnce()) {
        clear_visual();
        self.ended = true;
        self.termination = Some(termination);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillExecutionKernel<Dispatch> {
    dispatch: Dispatch,
    stage: SkillStage,
    lifecycle: SkillLifecycle,
}

impl<Dispatch: Copy + Eq> SkillExecutionKernel<Dispatch> {
    pub const fn begin(dispatch: Dispatch, started_at_ms: u32) -> Self {
        Self {
            dispatch,
            stage: SkillStage::Begin,
            lifecycle: SkillLifecycle::for_execution(started_at_ms),
        }
    }

    pub const fn dispatch(self) -> Dispatch {
        self.dispatch
    }

    pub const fn started_at_ms(self) -> u32 {
        self.lifecycle.started_at_ms()
    }

    pub const fn stage(self) -> SkillStage {
        self.stage
    }

    pub fn clear_phase_for_end(&mut self) {
        self.stage = SkillStage::Idle;
    }

    pub const fn termination(self) -> Option<SkillTermination> {
        self.lifecycle.termination()
    }

    pub const fn is_prepared(self) -> bool {
        self.lifecycle.is_prepared()
    }

    pub fn mark_prepared(&mut self) {
        if self.lifecycle.termination().is_none() {
            self.lifecycle.mark_prepared();
        }
    }

    pub fn advance(&mut self, expected: SkillStage, next: SkillStage) -> bool {
        if self.lifecycle.termination().is_some() || self.stage != expected || next <= expected {
            return false;
        }
        self.stage = next;
        true
    }

    pub const fn lifecycle(&self) -> &SkillLifecycle {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut SkillLifecycle {
        &mut self.lifecycle
    }

    pub fn replace_lifecycle(&mut self, lifecycle: SkillLifecycle) -> SkillLifecycle {
        std::mem::replace(&mut self.lifecycle, lifecycle)
    }
}
