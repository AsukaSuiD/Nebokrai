//! Типизированные payload-данные исполнений игрока и боевого духа Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/skill.cpp/.h
//! и конкретные appserver/skills/*.cpp/.h. Каталог ниже связывает concrete
//! payload с базой `skills/lifecycle.rs` и узкими End-hooks; его наличие
//! само по себе не запускает Begin, End или клиентский visual.
//! BF End(int) VA 0x00516FB0/0x0051A700/0x005222A0 очищает DWORD-фазу
//! перед visual и AfterUse; End(bool) VA 0x0051BE50/0x005246C0
//! очищает BYTE-флаги, но не заменяет End(int). Конкретный владелец
//! выбирает соответствующий пролог.
//!
//! Перенос из старого пакета сохраняет поля и тела
//! буквально; все поля объявлены `pub`, потому что оставшаяся логика cast
//! старого пакета читает и пишет их напрямую (`state.direction = …`,
//! `state.kernel.advance(…)`), а enum-варианты проецируются typed seam-ом
//! `PlayerSkillState` вместо повторных enum-обходов. Каждая секция помечена
//! файлом прежнего владельца.

use crate::regions::ShapeIdentity;
use crate::skills::{
    BattleFairySkillDispatch, PlayerSkillDispatch, SkillExecutionKernel, SkillStage,
};

const BLOCK_UNFLY: u8 = 2;

// ==== lightingarrowphalanx.rs: общий ключ дедупликации цели по региону ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArrowTargetIdentity {
    pub region_id: i32,
    pub identity: ShapeIdentity,
}

impl ArrowTargetIdentity {
    pub const fn new(region_id: i32, identity: ShapeIdentity) -> Self {
        Self { region_id: if identity.object_type == 400 { 0 } else { region_id }, identity }
    }
}

// ==== rainarrowphalanx.rs: тип ячейки пути прицельного ливня ====
pub type RainArrowCell = (i32, i32, u8);

// ==== rainarrowphalanx.rs: три полёта прицельного ливня с активным хвостом ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RainArrowPath {
    pub cells: Vec<RainArrowCell>,
    pub active_cells: u32,
}

// ==== energybolt.rs: область удара путевого снаряда по уровню ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathProjectileScope {
    Cell,
    Square3,
}

impl PathProjectileScope {
    pub const fn for_level(level: i32) -> Self {
        if matches!(level, 1 | 2) { Self::Cell } else { Self::Square3 }
    }

    pub const fn radius(self) -> i32 {
        match self {
            Self::Cell => 0,
            Self::Square3 => 1,
        }
    }
}

// ==== energybolt.rs: сохраняемая часть конкретного владельца путевого снаряда ====
/// Область заменяет выделение в исходной куче и не очищается End, остальные
/// поля принадлежат текущему полёту.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PathProjectileProgress {
    pub scope: Option<PathProjectileScope>,
    pub path: Vec<(i32, i32, u8)>,
    pub current_position: usize,
    pub end_x: i32,
    pub end_y: i32,
    pub visual_target: Option<ShapeIdentity>,
    pub missile_flying_time_ms: u32,
    pub fired: bool,
}

impl PathProjectileProgress {
    pub fn clear_end_paths(&mut self) {
        drop(std::mem::take(&mut self.path));
        self.current_position = 0;
        self.end_x = 0;
        self.end_y = 0;
        self.visual_target = None;
        self.missile_flying_time_ms = 0;
        self.fired = false;
    }

    pub fn ensure_scope(&mut self, level: i32) {
        if self.scope.is_none() {
            self.scope = Some(PathProjectileScope::for_level(level));
        }
    }

    pub fn scope(&self) -> Option<PathProjectileScope> {
        self.scope
    }

    pub fn fired(&self) -> bool {
        self.fired
    }

    pub fn prepare_flight(&mut self, path: Vec<(i32, i32, u8)>, flying_unit_ms: u32) {
        let unfly = path.iter().position(|cell| cell.2 == BLOCK_UNFLY);
        let end_index = unfly.unwrap_or(path.len());
        if let Some(&(x, y, _)) = unfly.and_then(|index| path.get(index)).or_else(|| path.last()) {
            self.end_x = x;
            self.end_y = y;
        }
        self.path = path;
        self.visual_target = None;
        self.missile_flying_time_ms = flying_unit_ms.wrapping_mul(end_index as u32);
    }

    pub fn start_flight(&mut self) {
        self.fired = true;
    }

    pub fn current_position(&self) -> usize {
        self.current_position
    }

    pub fn path_len(&self) -> usize {
        self.path.len()
    }

    pub fn current_cell(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_position).map(|&(x, y, _)| (x, y))
    }

    pub fn set_end_position(&mut self, x: i32, y: i32) {
        self.end_x = x;
        self.end_y = y;
    }

    pub fn select_visual_target_if_empty(&mut self, target: ShapeIdentity) {
        if self.visual_target.is_none() {
            self.visual_target = Some(target);
        }
    }

    pub fn finish_after_collision(&mut self) {
        self.current_position = self.path.len().wrapping_add(1);
    }

    pub fn finish_after_unfly(&mut self) {
        self.current_position = self.path.len();
    }

    pub fn advance(&mut self) {
        self.current_position = self.current_position.wrapping_add(1);
    }

    pub const fn end_position(&self) -> (i32, i32) {
        (self.end_x, self.end_y)
    }

    pub const fn visual_target(&self) -> Option<ShapeIdentity> {
        self.visual_target
    }

    pub const fn missile_flying_time_ms(&self) -> u32 {
        self.missile_flying_time_ms
    }
}

// ==== directprojectile.rs: поля конкретного C++-экземпляра между Begin, выпуском и End ====
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectProjectileProgress {
    pub available: bool,
    pub condition_checked: bool,
    pub attacking_started: bool,
    pub missile_flying_time_ms: u32,
    pub auto_restart: bool,
}

impl DirectProjectileProgress {
    pub fn begin_after_check(&mut self) {
        self.available = true;
        self.condition_checked = false;
        self.attacking_started = false;
        self.missile_flying_time_ms = 0;
    }

    pub fn is_available(self) -> bool {
        self.available
    }

    pub fn condition_checked(self) -> bool {
        self.condition_checked
    }

    pub fn attacking_started(self) -> bool {
        self.attacking_started
    }

    pub fn mark_condition_checked(&mut self) {
        self.condition_checked = true;
    }

    pub fn start_flight(&mut self, missile_flying_time_ms: u32) {
        self.missile_flying_time_ms = missile_flying_time_ms;
    }

    pub fn mark_attacking_started(&mut self) {
        self.attacking_started = true;
    }

    pub fn auto_restart(self) -> bool {
        self.auto_restart
    }

    pub const fn missile_flying_time_ms(&self) -> u32 {
        self.missile_flying_time_ms
    }

    /// `CChuckStone::End` / `CSkeletonArchery::End` сбрасывает эти четыре
    /// поля до `GetUser()->SetMoveable(true)` и базового End: в машинном коде
    /// `?End@CChuckStone@@UAEXH@Z` (VA 0x0056A330, общий адрес обоих
    /// владельцев) — четыре DWORD-зануления `+0x4C/0x50/0x54/0x58` перед
    /// `SetMoveable(true)` и базовым End.
    pub fn prepare_derived_end(&mut self) {
        self.available = false;
        self.condition_checked = false;
        self.attacking_started = false;
        self.missile_flying_time_ms = 0;
    }
}

// ==== baseprojectilecast.rs: часы атаки постоянного снаряда экземпляра ====
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BaseProjectileProgress {
    pub attack_time_ms: i32,
}

impl BaseProjectileProgress {
    pub const fn attack_time_ms(&self) -> i32 { self.attack_time_ms }
}

// ==== baseprojectilecast.rs: kernel обычного снаряда (CArchery/CBaseMagic/CFireBolt) ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseProjectileExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}

impl BaseProjectileExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started) }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

// ==== lightning.rs: флаг удара молнии и её kernel ====
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LightningProgress {
    pub attacking: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightningExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub progress: LightningProgress,
}

impl LightningExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            progress: LightningProgress::default(),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn progress(&self) -> &LightningProgress { &self.progress }
    pub fn progress_mut(&mut self) -> &mut LightningProgress { &mut self.progress }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.progress = LightningProgress::default();
        true
    }
}

// ==== chainlightning.rs: путь цепной молнии, атакованные цели и её kernel ====
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ChainLightningProgress {
    pub path: Vec<(i32, i32, u8)>,
    pub attacked_targets: Vec<ArrowTargetIdentity>,
    pub attacked: bool,
}

impl ChainLightningProgress {
    pub fn clear_end_paths(&mut self) {
        self.attacked = false;
        self.path.clear();
        self.attacked_targets.clear();
    }

    pub fn has_attacked(&self, target: (i32, ShapeIdentity)) -> bool {
        self.attacked_targets.contains(&ArrowTargetIdentity::new(target.0, target.1))
    }

    pub fn append_target(&mut self, target: (i32, ShapeIdentity)) {
        self.attacked_targets.push(ArrowTargetIdentity::new(target.0, target.1));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainLightningExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub progress: ChainLightningProgress,
}

impl ChainLightningExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), progress: ChainLightningProgress::default() }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn progress(&self) -> &ChainLightningProgress { &self.progress }
    pub fn progress_mut(&mut self) -> &mut ChainLightningProgress { &mut self.progress }
    pub fn clear_end_paths(&mut self) { self.progress.clear_end_paths(); }
}

// ==== targetedprojectile.rs: флаг удара и полёт прицельного снаряда с его kernel ====
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TargetedProjectileProgress {
    pub attacking: bool,
    pub flight: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetedProjectileExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub progress: TargetedProjectileProgress,
}

impl TargetedProjectileExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), progress: TargetedProjectileProgress::default() }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn progress(&self) -> &TargetedProjectileProgress { &self.progress }
    pub fn progress_mut(&mut self) -> &mut TargetedProjectileProgress { &mut self.progress }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.progress = TargetedProjectileProgress::default();
        true
    }
}

// ==== heartlessarrow.rs: состояние безжалостной стрелы с её kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeartlessArrowExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
    pub skill_casted: bool,
    pub hold_time_ms: u32,
    pub missile_flying_time_ms: u32,
}

impl HeartlessArrowExecutionState {
    pub fn prepare_derived_end(&mut self, argument: i32) -> bool {
        if argument != 0 && self.kernel.stage() == SkillStage::Check && !self.attacking_started {
            self.attacking_started = true;
            return false;
        }
        self.kernel.clear_phase_for_end();
        self.attacking_started = false;
        self.skill_casted = false;
        self.missile_flying_time_ms = 0;
        self.hold_time_ms = 0;
        true
    }

    pub fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            attacking_started: false, skill_casted: false, hold_time_ms: 0, missile_flying_time_ms: 0,
        }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time_ms(&self) -> u32 { self.missile_flying_time_ms }
}

// ==== heartlessarrow2.rs: площадная стрела с её kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeartlessArrowAreaExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub missile_flying_time_ms: u32,
}

impl HeartlessArrowAreaExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), missile_flying_time_ms: 0 }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.missile_flying_time_ms = 0;
        true
    }
}

// ==== scopedarrowcast.rs: состояние прицельной стрелы с её kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedArrowExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
    pub missile_flying_time: u32,
    pub current_position: u32,
    pub end_tile: (i32, i32),
    pub visual_target: (i32, i32),
    pub path: Vec<(i32, i32, u8)>,
    pub attacked: Vec<ArrowTargetIdentity>,
}

impl ScopedArrowExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            current_position: 0,
            end_tile: (0, 0),
            visual_target: (0, 0),
            path: Vec::new(),
            attacked: Vec::new(),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }
    pub const fn end_tile(&self) -> (i32, i32) { self.end_tile }
    pub const fn visual_target(&self) -> (i32, i32) { self.visual_target }

    pub fn is_target_attacked(&self, target: (i32, ShapeIdentity)) -> bool {
        self.attacked.contains(&ArrowTargetIdentity::new(target.0, target.1))
    }

    pub fn mark_target_attacked(&mut self, target: (i32, ShapeIdentity)) -> bool {
        let key = ArrowTargetIdentity::new(target.0, target.1);
        if self.attacked.contains(&key) { return false; }
        self.attacked.push(key);
        true
    }

    pub fn select_visual_target_if_empty(&mut self, identity: ShapeIdentity) {
        if self.visual_target == (0, 0) {
            self.visual_target = (identity.object_type, identity.id);
        }
    }

    pub fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.current_position = 0;
        self.end_tile = (0, 0);
        self.visual_target = (0, 0);
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked));
    }
}

// ==== ghostcut.rs: состояние призрачного разреза с его kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GhostCutExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
    pub missile_flying_time: u32,
    pub path: Vec<(i32, i32, u8)>,
    pub current_position: u32,
    pub attacked: Vec<ShapeIdentity>,
}

impl GhostCutExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            path: Vec::new(),
            current_position: 0,
            attacked: Vec::new(),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }

    pub fn mark_target_attacked(&mut self, identity: ShapeIdentity) -> bool {
        if self.attacked.contains(&identity) { return false; }
        self.attacked.push(identity);
        true
    }

    pub fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.current_position = 0;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked));
    }
}

// ==== thunderblow2.rs: состояние второго удара грома с его kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThunderBlow2Execution {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
}

impl ThunderBlow2Execution {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), attacking_started: false }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.attacking_started = false;
        true
    }
}

// ==== armybreak.rs: направление прорыва строя с его kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArmyBreakExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub direction: i32,
}

impl ArmyBreakExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), direction: -1 }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.direction = -1;
        true
    }
}

// ==== littleflash.rs: короткая вспышка с путём, целями и её kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LittleFlashExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub path: Vec<(i32, i32, u8)>,
    pub attacked_creatures: Vec<ShapeIdentity>,
    pub attacking_started: bool,
    pub attacked: bool,
}

impl LittleFlashExecutionState {
    pub fn clear_end_paths(&mut self) {
        self.attacked = false;
        self.attacking_started = false;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked_creatures));
    }
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            path: Vec::new(), attacked_creatures: Vec::new(),
            attacking_started: false, attacked: false,
        }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

// ==== summoncreatureskill.rs: точка призыва с kernel игрока ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerSummonCreatureExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub destination: (i32, i32),
}

impl PlayerSummonCreatureExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

// ==== lordfastattack.rs: два шага господского быстрого удара с его kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LordFastAttackExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub condition_checked: bool,
    pub fire_started: bool,
    pub first_attack_done: bool,
}

impl LordFastAttackExecutionState {
    pub const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            condition_checked: false,
            fire_started: false,
            first_attack_done: false,
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

// ==== lightingarrow.rs: путь стрелы-молнии с её kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightingArrowExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub path: Vec<(i32, i32, u8)>,
}

impl LightingArrowExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), path: Vec::new() }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub fn clear_end_paths(&mut self) { self.path.clear(); }
}

// ==== lightingarrow2.rs: клеточный веер второй стрелы-молнии с её kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightingArrow2ExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
    pub missile_flying_time: u32,
    pub path: Vec<(i32, i32, u8)>,
    pub attack_cell_count: u32,
    pub current_cell: u32,
    pub attacked_creatures: Vec<ArrowTargetIdentity>,
}

impl LightingArrow2ExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            path: Vec::new(),
            attack_cell_count: 0,
            current_cell: 0,
            attacked_creatures: Vec::new(),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }

    pub fn mark_target_attacked(&mut self, target: (i32, ShapeIdentity)) -> bool {
        let key = ArrowTargetIdentity::new(target.0, target.1);
        if self.attacked_creatures.contains(&key) { return false; }
        self.attacked_creatures.push(key);
        true
    }

    pub fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.attack_cell_count = 0;
        self.current_cell = 0;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked_creatures));
    }
}

// ==== rainarrow.rs: три полёта прицельного ливня с kernel игрока ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RainArrowExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub angle_bits: u32,
    pub missile_flying_time: u32,
    pub right: RainArrowPath,
    pub center: RainArrowPath,
    pub left: RainArrowPath,
}

impl RainArrowExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started), angle_bits: 0,
            missile_flying_time: 0,
            right: RainArrowPath { cells: Vec::new(), active_cells: 0 },
            center: RainArrowPath { cells: Vec::new(), active_cells: 0 },
            left: RainArrowPath { cells: Vec::new(), active_cells: 0 },
        }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }
    pub fn right_impact(&self) -> Option<(i32, i32)> { impact(&self.right) }
    pub fn left_impact(&self) -> Option<(i32, i32)> { impact(&self.left) }

    pub fn clear_end_paths(&mut self) {
        self.missile_flying_time = 0;
        self.right.active_cells = 0;
        self.center.active_cells = 0;
        self.left.active_cells = 0;
        self.angle_bits = 0;
        self.right.cells.clear();
        self.center.cells.clear();
        self.left.cells.clear();
    }
}

fn impact(path: &RainArrowPath) -> Option<(i32, i32)> {
    path.active_cells.checked_sub(1).and_then(|index| path.cells.get(index as usize)).map(|cell| (cell.0, cell.1))
}

// ==== poisonmoth.rs: ядовитый мотылёк с путём, конечной клеткой и kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoisonMothExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
    pub missile_flying_time: u32,
    pub path: Vec<(i32, i32, u8)>,
    pub current_position: u32,
    pub end_tile: (i32, i32),
    pub visual_target: (i32, i32),
}

impl PoisonMothExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            path: Vec::new(),
            current_position: 0,
            end_tile: (0, 0),
            visual_target: (0, 0),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }
    pub const fn end_tile(&self) -> (i32, i32) { self.end_tile }
    pub const fn visual_target(&self) -> (i32, i32) { self.visual_target }

    pub fn set_visual_target(&mut self, target: ShapeIdentity) {
        self.visual_target = (target.object_type, target.id);
    }

    pub fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.current_position = 0;
        self.end_tile = (0, 0);
        self.visual_target = (0, 0);
        drop(std::mem::take(&mut self.path));
    }
}

// ==== scorpion.rs: два удара скорпиона с их kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScorpionExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub first_attack: bool,
    pub second_attack: bool,
}

impl ScorpionExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), first_attack: false, second_attack: false }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.first_attack = false;
        self.second_attack = false;
        true
    }
}

// ==== boalock.rs: удавка с её kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoaLockExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub attacking_started: bool,
    pub missile_flying_time: u32,
}

impl BoaLockExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        true
    }
}

// ==== knightcut.rs: направление рыцарского удара с его kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KnightCutExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub direction: i32,
}

impl KnightCutExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), direction: -1 }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn direction(&self) -> i32 { self.direction }
}

// ==== rage.rs: жизненный цикл канала ярости с его kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RageExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub last_using_time_ms: u32,
}

impl RageExecutionState {
    pub const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            last_using_time_ms: 0,
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub const fn last_using_time_ms(self) -> u32 {
        self.last_using_time_ms
    }

    pub const fn mark_used(&mut self, now_ms: u32) {
        self.last_using_time_ms = now_ms;
    }
}

// ==== flash.rs: вспышка с путём, целями и её kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlashExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub condition_checked: bool,
    pub attacked: bool,
    pub path: Vec<(i32, i32, u8)>,
    pub attacked_creatures: Vec<ShapeIdentity>,
}

impl FlashExecutionState {
    pub fn clear_end_paths(&mut self) {
        self.condition_checked = false;
        self.attacked = false;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked_creatures));
    }

    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            condition_checked: false, attacked: false,
            path: Vec::new(), attacked_creatures: Vec::new(),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

// ==== rush.rs: путь рывка с его kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RushExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub path: Vec<(i32, i32, u8)>,
}

impl RushExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), path: Vec::new() }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub fn clear_end_paths(&mut self) { drop(std::mem::take(&mut self.path)); }
}

// ==== swallow.rs: направление ласточки с флагом первого удара и kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwallowExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub first_attack_done: bool,
    pub direction: i32,
}

impl SwallowExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            first_attack_done: false,
            direction: -1,
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub const fn direction(&self) -> i32 { self.direction }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.first_attack_done = false;
        self.kernel.lifecycle_mut().set_available(true);
        self.direction = -1;
        true
    }
}

// ==== sevenshootingstar.rs: семь падающих звёзд с путём и kernel ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SevenShootingStarExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub path: Option<Vec<(i32, i32, u8)>>,
    pub destination: Option<(i32, i32)>,
    pub last_attack_ms: u32,
}

impl SevenShootingStarExecutionState {
    pub fn clear_end_paths(&mut self) {
        self.last_attack_ms = 0;
        drop(self.path.take());
    }

    pub fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            path: None,
            destination: None,
            last_attack_ms: 0,
        }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub fn path(&self) -> Option<&[(i32, i32, u8)]> { self.path.as_deref() }
    pub fn destination(&self) -> Option<(i32, i32)> { self.destination }
    pub fn needs_path_refresh(&self) -> bool { self.last_attack_ms == 0 }
    pub fn set_path(&mut self, path: Vec<(i32, i32, u8)>, destination: (i32, i32)) {
        self.path = Some(path);
        self.destination = Some(destination);
    }
    pub fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }
    pub fn record_attack(&mut self, now_ms: u32) { self.last_attack_ms = now_ms; }
}

// ==== littlestar.rs: малая звезда игрока с путём и kernel; монстровый ход той же звезды ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerLittleStarExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub path: Option<Vec<(i32, i32, u8)>>,
    pub last_attack_ms: u32,
}

impl PlayerLittleStarExecutionState {
    pub fn clear_end_paths(&mut self) {
        self.last_attack_ms = 0;
        drop(self.path.take());
    }

    pub fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), path: None, last_attack_ms: 0 }
    }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LittleStarProgress {
    pub path: Vec<(i32, i32, u8)>,
    pub last_attack_ms: u32,
}

impl LittleStarProgress {
    pub fn clear_end_paths(&mut self) {
        self.last_attack_ms = 0;
        drop(std::mem::take(&mut self.path));
    }

    pub fn new(path: Vec<(i32, i32, u8)>) -> Self {
        Self {
            path,
            last_attack_ms: 0,
        }
    }

    pub fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }

    pub fn record_attack(&mut self, now_ms: u32) {
        self.last_attack_ms = now_ms;
    }
}

// ==== yunshenglightning.rs: точка облачной молнии игрока с kernel; монстровый её ход ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerYunShengLightningExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub destination: (i32, i32),
    pub condition_checked: bool,
}

impl PlayerYunShengLightningExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination, condition_checked: false } }
    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct YunShengLightningProgress {
    pub destination_x: i32,
    pub destination_y: i32,
    pub flying_time_ms: u32,
    pub fired: bool,
}

impl YunShengLightningProgress {
    pub const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            flying_time_ms: 0,
            fired: false,
        }
    }

    pub const fn fired(self) -> bool {
        self.fired
    }

    pub const fn destination(self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    pub const fn flying_time_ms(self) -> u32 {
        self.flying_time_ms
    }

    pub fn fire(
        &mut self,
        destination_x: i32,
        destination_y: i32,
        flying_time_ms: u32,
    ) {
        self.destination_x = destination_x;
        self.destination_y = destination_y;
        self.flying_time_ms = flying_time_ms;
        self.fired = true;
    }
}

// ==== monsterthorn.rs: точка монстрового шипа игрока с kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerMonsterThornExecutionState { pub kernel: SkillExecutionKernel<PlayerSkillDispatch>, pub destination: (i32, i32) }
impl PlayerMonsterThornExecutionState { pub fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now), destination: match dispatch { PlayerSkillDispatch::Object { .. } => (0, 0), PlayerSkillDispatch::Point { x, y, .. } => (x, y), _ => destination } } } pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel } pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel } }

// ==== spidermist.rs: точка паутинного тумана игрока с kernel; монстровый его ход ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerSpiderMistExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub destination: (i32, i32),
}

impl PlayerSpiderMistExecutionState {
    pub fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), destination }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpiderMistProgress {
    pub destination_x: i32,
    pub destination_y: i32,
}

// ==== spiderweb.rs: полёт паутины игрока с kernel; монстровые часы паутины ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerSpiderWebExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub flight: SpiderWebProgress,
}

impl PlayerSpiderWebExecutionState {
    pub fn before_check(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        let mut kernel = SkillExecutionKernel::begin(dispatch, started);
        kernel.clear_phase_for_end();
        Self { kernel, flight: SpiderWebProgress::new(0) }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.flight = SpiderWebProgress::new(0);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpiderWebProgress {
    pub missile_flying_time_ms: u32,
}

impl SpiderWebProgress {
    pub const fn new(missile_flying_time_ms: u32) -> Self {
        Self { missile_flying_time_ms }
    }

    pub const fn missile_flying_time_ms(self) -> u32 {
        self.missile_flying_time_ms
    }
}

// ==== bossbluequake.rs: направление дрожи синего босса игрока с kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerBossBlueQuakeExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub direction: i32,
}

impl PlayerBossBlueQuakeExecutionState {
    pub const fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            direction: -1,
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub const fn direction(&self) -> i32 {
        self.direction
    }

    pub const fn set_direction(&mut self, direction: i32) {
        self.direction = direction;
    }
}

// ==== bossfiendpenetrate.rs: пронзание демона игрока с kernel; монстровый его ход ====
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerBossFiendPenetrateExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    pub destination: (i32, i32),
    pub condition_checked: bool,
    pub path: Vec<(i32, i32, u8)>,
    pub attack_cell_count: usize,
    pub current_cell: usize,
    pub attacked: Vec<ShapeIdentity>,
}

impl PlayerBossFiendPenetrateExecutionState {
    pub fn clear_end_paths(&mut self) {
        self.condition_checked = false;
        self.attack_cell_count = 0;
        self.current_cell = 0;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked));
    }

    pub fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            destination,
            condition_checked: false,
            path: Vec::new(),
            attack_cell_count: 0,
            current_cell: 0,
            attacked: Vec::new(),
        }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BossFiendPenetrateProgress {
    pub destination_x: i32,
    pub destination_y: i32,
    pub path: Vec<(i32, i32, u8)>,
    pub current_cell: usize,
    pub attack_cell_count: usize,
    pub attacked: Vec<ShapeIdentity>,
    pub fired: bool,
}

impl BossFiendPenetrateProgress {
    pub fn clear_end_paths(&mut self) {
        self.fired = false;
        self.attack_cell_count = 0;
        self.current_cell = 0;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked));
    }

    pub const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            path: Vec::new(),
            current_cell: 0,
            attack_cell_count: 0,
            attacked: Vec::new(),
            fired: false,
        }
    }

    pub const fn destination(&self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    pub fn fire(&mut self, path: Vec<(i32, i32, u8)>) {
        self.attack_cell_count = path
            .iter()
            .position(|cell| cell.2 == BLOCK_UNFLY)
            .unwrap_or(path.len());
        self.path = path;
        self.current_cell = 0;
        self.fired = true;
    }
}

// ==== spriteburn.rs: kernel огненного духа ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpriteBurnExecutionState {
    pub kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}

impl SpriteBurnExecutionState {
    pub const fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms) }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

// ==== fatalblow.rs: время полёта рокового удара боевого духа с kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FatalBlowExecutionState {
    pub kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    pub missile_flying_time: u32,
}

impl FatalBlowExecutionState {
    pub const fn begin(dispatch: BattleFairySkillDispatch, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), missile_flying_time: 0 }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<BattleFairySkillDispatch> { &self.kernel }
    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> { &mut self.kernel }
    pub const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }

    pub const fn set_missile_flying_time(&mut self, missile_flying_time: u32) {
        self.missile_flying_time = missile_flying_time;
    }

    pub fn prepare_derived_end(&mut self) { self.missile_flying_time = 0; }
}

// ==== battlefairybasemagic.rs: часы базовой магии боевого духа с kernel ====
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyBaseMagicExecutionState {
    pub kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    pub attack_time: u32,
}

impl BattleFairyBaseMagicExecutionState {
    pub const fn begin(dispatch: BattleFairySkillDispatch, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), attack_time: 0 }
    }

    pub const fn kernel(&self) -> &SkillExecutionKernel<BattleFairySkillDispatch> {
        &self.kernel
    }

    pub fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        &mut self.kernel
    }

    pub const fn attack_time(&self) -> u32 { self.attack_time }
}

// ==== monsterfastattack.rs: состояние двух последовательных ударов навыка; общий
// `SkillExecutionKernel` по-прежнему хранит начало и основные стадии ====
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MonsterFastAttackProgress {
    pub visual_started: bool,
    pub first_attack_done: bool,
}

impl MonsterFastAttackProgress {
    pub const fn visual_started(self) -> bool {
        self.visual_started
    }

    pub const fn first_attack_done(self) -> bool {
        self.first_attack_done
    }

    pub const fn mark_visual_started(&mut self) {
        self.visual_started = true;
    }

    pub const fn mark_first_attack_done(&mut self) {
        self.first_attack_done = true;
    }
}
