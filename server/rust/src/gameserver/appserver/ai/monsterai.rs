//! Делегат диспетчер-ядра `CMonsterAI` в Zone.
//!
//! Тела ядра (finite-решения расписаний, отдельный timestamp атаки,
//! MoveTo/idle/tracing, OnLoseTarget-семья и stiffen-переход) перенесены
//! буквально в `nebokrai_zone::ai::monsterai` — машинная база VERIFIED по
//! точной паре `4F5C98E0…` + GameServer.pdb (RSDS match), объявленные швы и
//! честные UNKNOWN см. в её шапке (`ai/monsterai.rs`). Здесь:
//!
//! - реализации hub-трейтов Zone над прежними `CGame`, `CPlayer`,
//!   `CServerRegion`, `ServerRegionOwner`, `CMoveShape` и `CMonster` — имена
//!   членов сохраняют исходные операции, state-машина AI, реестр навыков,
//!   FIFO и пространственный рантайм остаются hub-владением;
//! - делегации с прежними сигнатурами и переэкспорт типов/предикатов —
//!   потребители старого пакета не меняются;
//! - проекция активного AI (`MonsterActiveAiView`) вычисляется из canonical
//!   `ActiveMonsterAi`/`MonsterAiKind` (единая таблица `aifactory.rs`
//!   остаётся источником): guard-station покрывает CGuardWithSword и
//!   наследников 9/10/12/16, обе формы повозки складываются в `Carriage`.
//!
//! Проекция `uses_stationary_attack_schedule`/`schedule_attack_interval`/
//! `hibernates_without_nearby_players`/`has_owned_search_enemy` в Zone
//! записана числовыми наборами `ai_type`; эквивалентность таблице
//! `MonsterAiKind::from_ai_type` задокументирована в шапке Zone-файла.

use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::ai::baseai::PassiveStiffenAction;
use crate::gameserver::appserver::monster::{CMonster, MonsterSkillExecution};
use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapePet, MoveShapeSkill};
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeAreaCoordinates, ShapeFigure, ShapeIdentity, ShapeView,
};
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, ServerRegionOwner, game_tick_milliseconds,
};
use crate::setup::monsterlist::MonsterProperties;
use nebokrai_shared::resources::GlobeSetupSnapshot;
use nebokrai_zone::ai::monsterai::{
    MonsterActiveAiView, MonsterBaseAttackDispatch, MonsterDispatcherGame,
    MonsterDispatcherMonster, MonsterDispatcherMoveShape, MonsterDispatcherOwner,
    MonsterDispatcherPlayer, MonsterDispatcherRegion, MonsterDispatcherRuntime,
};
use nebokrai_zone::content::CSkillBaseProperties;
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::{SkillExecutionKernel, SkillTermination};

pub(crate) use nebokrai_zone::ai::monsterai::{
    MonsterAiScheduleState, MonsterSkillCallOutcome, MonsterTraceTarget, accepts_hurt_target,
    has_owned_search_enemy, hibernates_without_nearby_players, schedule_attack_interval,
    uses_stationary_attack_schedule,
};

/// Проекция активного AI из canonical таблицы владельцев. Primary Carriage и
/// auxiliary Carriage разделять не требуется: расписания повозки общие.
fn active_monster_ai_view(ai: ActiveMonsterAi) -> MonsterActiveAiView {
    match ai {
        ActiveMonsterAi::Pet => MonsterActiveAiView::Pet,
        ActiveMonsterAi::Carriage => MonsterActiveAiView::Carriage,
        ActiveMonsterAi::Primary(MonsterAiKind::Carriage) => MonsterActiveAiView::Carriage,
        ActiveMonsterAi::Primary(kind) if kind.has_guard_station() => MonsterActiveAiView::GuardStation,
        ActiveMonsterAi::Primary(MonsterAiKind::JiuMai) => MonsterActiveAiView::JiuMai,
        ActiveMonsterAi::Primary(MonsterAiKind::PuninessCreature) => MonsterActiveAiView::PuninessCreature,
        ActiveMonsterAi::Primary(_) => MonsterActiveAiView::OtherPrimary,
    }
}

impl MonsterDispatcherMoveShape for CMoveShape {
    type Execution = MonsterSkillExecution;

    fn is_moveable(&self) -> bool {
        self.is_moveable()
    }

    fn shape(&self) -> &CShape {
        self.shape()
    }

    fn current_skill(&self, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        self.current_skill(factory)
    }

    fn default_attack_skill_id(&self) -> u32 {
        self.default_attack_skill_id()
    }

    fn set_current_skill_id(&mut self, skill_id: Option<u32>) {
        self.set_current_skill_id(skill_id);
    }
}

impl MonsterDispatcherMonster for CMonster {
    type MoveShape = CMoveShape;

    fn move_shape(&self) -> &CMoveShape {
        self.move_shape()
    }

    fn move_shape_mut(&mut self) -> &mut CMoveShape {
        self.move_shape_mut()
    }

    fn base_property_key(&self) -> Option<&[u8]> {
        self.base_property_key()
    }

    fn figure_for(properties: &MonsterProperties) -> ShapeFigure {
        CMonster::figure(properties)
    }

    fn shape_view(&self, properties: &MonsterProperties) -> Option<ShapeView> {
        self.shape_view(properties)
    }

    fn stop_frame(&self, properties: &MonsterProperties) -> u32 {
        self.stop_frame(properties)
    }

    fn speed(&self) -> f32 {
        self.speed()
    }

    fn hit_points(&self) -> u32 {
        self.hit_points()
    }

    fn is_tamed(&self) -> bool {
        self.is_tamed()
    }

    fn master_info(&self) -> crate::gameserver::appserver::masterinfo::MasterInfo {
        self.master_info()
    }

    fn pet_mode(&self) -> i32 {
        self.pet_mode()
    }

    fn pet_action(&self) -> i32 {
        self.pet_action()
    }

    fn active_primary_ai_type(&self) -> Option<u32> {
        self.active_primary_ai_type()
    }

    fn active_ai_view(&self) -> Option<MonsterActiveAiView> {
        self.active_ai().map(active_monster_ai_view)
    }

    fn primary_ai_queues_idle(&self) -> bool {
        self.primary_ai_queues_idle()
    }

    fn ai_target(&self) -> Option<ShapeIdentity> {
        self.ai_target()
    }

    fn set_ai_target(&mut self, target: ShapeIdentity) {
        self.set_ai_target(target);
    }

    fn clear_ai_target(&mut self, factory: &CSkillFactory) {
        self.clear_ai_target(factory);
    }

    fn lose_ai_target_and_search(&mut self, now_ms: u32, factory: &CSkillFactory) {
        self.lose_ai_target_and_search(now_ms, factory);
    }

    fn release_ai_target_for_death(&mut self) {
        self.release_ai_target_for_death();
    }

    fn release_pet_ai_target(&mut self) {
        self.release_pet_ai_target();
    }

    fn begin_active_ai_move(&mut self, delay_ms: u32, now_ms: u32) {
        self.begin_active_ai_move(delay_ms, now_ms);
    }

    fn begin_active_ai_stand(&mut self, delay_ms: u32, now_ms: u32) {
        self.begin_active_ai_stand(delay_ms, now_ms);
    }

    fn begin_active_ai_search_enemy(&mut self, now_ms: u32) {
        self.begin_active_ai_search_enemy(now_ms);
    }

    fn begin_active_ai_change_skill(&mut self, now_ms: u32) {
        self.begin_active_ai_change_skill(now_ms);
    }

    fn current_active_attack_cast(
        &self,
        factory: &CSkillFactory,
    ) -> Option<SkillExecutionKernel<MonsterBaseAttackDispatch>> {
        self.current_active_attack_cast(factory)
    }

    fn skill_last_used_ms(&self, skill_id: u32, factory: &CSkillFactory) -> u32 {
        self.skill_last_used_ms(skill_id, factory)
    }

    fn begin_reached_stiffen_action(&mut self) -> PassiveStiffenAction {
        self.begin_reached_stiffen_action()
    }

    fn finish_reached_stiffen_action(
        &mut self,
        begun: PassiveStiffenAction,
        now: impl FnOnce() -> u32,
    ) -> PassiveStiffenAction {
        self.finish_reached_stiffen_action(begun, now)
    }

    fn stiffen_attack_needs_end(&self) -> Option<bool> {
        self.selected_base_ai().map(|ai| ai.stiffen_attack_needs_end())
    }

    fn prepare_stiffen_attack(
        &mut self,
        factory: &CSkillFactory,
    ) -> Option<(bool, Option<u32>)> {
        self.prepare_stiffen_attack(factory)
    }

    fn finish_stiffen_attack(
        &mut self,
        ended_skill: Option<u32>,
        factory: &CSkillFactory,
        now: impl FnOnce() -> u32,
    ) {
        self.finish_stiffen_attack(ended_skill, factory, now);
    }

    fn resume_stiffen_after_target_release(&mut self, released: bool) {
        self.resume_stiffen_after_target_release(released);
    }
}

impl MonsterDispatcherRegion for CServerRegion {
    type Monster = CMonster;

    fn find_monster_by_id(&self, id: i32) -> Option<&CMonster> {
        self.find_monster_by_id(id)
    }

    fn find_monster_by_id_mut(&mut self, id: i32) -> Option<&mut CMonster> {
        self.find_monster_by_id_mut(id)
    }

    fn monster_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        self.monster_ids_around_area(area_index)
    }

    fn straight_skill_path(
        &self,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
        forced_length: Option<u32>,
    ) -> Vec<(i32, i32, u8)> {
        self.straight_skill_path(source_x, source_y, target_x, target_y, forced_length)
    }

    fn base_region(&self) -> &nebokrai_zone::regions::region::CRegion {
        &self.region
    }

    fn region_id(&self) -> i32 {
        self.id
    }
}

impl MonsterDispatcherOwner for ServerRegionOwner {
    type Region = CServerRegion;

    fn base(&self) -> &CServerRegion {
        self.base()
    }

    fn base_mut(&mut self) -> &mut CServerRegion {
        self.base_mut()
    }

    fn region_id(&self) -> i32 {
        self.region_id()
    }
}

impl MonsterDispatcherPlayer for CPlayer {
    fn shape(&self) -> &CShape {
        self.shape()
    }

    fn shape_view(&self) -> Option<ShapeView> {
        self.shape_view()
    }

    fn server_region_id(&self) -> Option<i32> {
        self.server_region_id()
    }

    fn is_badman(&self, pk_count_per_kill: u32) -> bool {
        self.is_badman(pk_count_per_kill)
    }

    fn active_pets(&self) -> &[MoveShapePet] {
        self.active_pets()
    }
}

impl MonsterDispatcherGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type Player = CPlayer;
    type RegionOwner = ServerRegionOwner;

    fn skill_factory(&self) -> &CSkillFactory {
        self.skill_factory()
    }

    fn find_monster_property_by_origin_name(
        &self,
        origin_name: &[u8],
    ) -> Option<&MonsterProperties> {
        self.find_monster_property_by_origin_name(origin_name)
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn skill_random_below(&mut self, maximum: i32) -> i32 {
        self.skill_random_below(maximum)
    }

    fn globe_setup(&self) -> &GlobeSetupSnapshot {
        self.globe_setup()
    }

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> {
        self.find_player(player_id)
    }

    fn shape_view_in_owner(
        &self,
        owner: &ServerRegionOwner,
        identity: ShapeIdentity,
    ) -> Option<ShapeView> {
        self.shape_view_in_owner(owner, identity)
    }

    fn monster_attack_target_view(
        &self,
        owner: &ServerRegionOwner,
        identity: ShapeIdentity,
    ) -> Option<ShapeView> {
        crate::gameserver::appserver::skills::monsterattack::resolve_owned_monster_attack_target(
            self, owner, identity,
        )
        .filter(|target| !target.dead)
        .map(|target| target.view)
    }

    fn find_slip_step_in_direction(
        &self,
        region: &CServerRegion,
        origin: ShapeAreaCoordinates,
        desired_direction: i32,
        figure_index: usize,
    ) -> Option<(i32, ShapeAreaCoordinates)> {
        super::baseai::find_slip_step_in_direction(
            self.move_check_cells(), region, origin, desired_direction, figure_index,
        )
    }

    fn one_step_move_delay_ms(direction: i32, speed: f32, stop_frame: u32) -> u32 {
        super::baseai::one_step_move_delay_ms(direction, speed, stop_frame)
    }

    fn move_owned_monster_step_with_run(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        x: i32,
        y: i32,
        figure: ShapeFigure,
        run: i32,
    ) -> bool {
        self.move_owned_monster_step_with_run(region, monster_id, x, y, figure, run)
    }

    fn spatial_delivery_ready(&self) -> bool {
        self.spatial_delivery_ready()
    }

    fn set_owned_pet_position(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        x: i32,
        y: i32,
        figure: ShapeFigure,
    ) -> bool {
        self.set_owned_pet_position(region, monster_id, x, y, figure)
    }

    fn release_jiumai_target(&mut self, region: &mut CServerRegion, monster_id: i32) -> bool {
        super::jiumai::release_jiumai_target(region, monster_id)
    }

    fn is_non_fun_skill_id(skill_id: u32) -> bool {
        crate::gameserver::appserver::skills::nonfun::is_non_fun_skill(skill_id)
    }

    fn send_little_star_end(
        &self,
        region: &CServerRegion,
        source: &CShape,
        skill_level: u16,
    ) {
        crate::gameserver::appserver::skills::littlestar::send_end(self, region, source, skill_level);
    }

    fn with_published_region<Output>(
        &mut self,
        owner: &mut Option<ServerRegionOwner>,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Option<Output> {
        self.with_published_region(owner, callback)
    }

    fn registered_move_shape_skill(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        skill_id: u32,
    ) -> Option<RegisteredSkill> {
        self.registered_move_shape_skill(region_id, holder, skill_id)
    }

    fn registered_skill(
        &self,
        address: RegisteredSkill,
    ) -> Option<&RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill(address)
    }

    fn end_registered_instance<Runtime>(
        &mut self,
        address: RegisteredSkill,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
    ) {
        let _ = self.end_registered_instance(address, argument, termination, runtime);
    }
}

impl<Runtime: GameMainLoopRuntime> MonsterDispatcherRuntime<Runtime> for CGame {
    fn release_guard_sword_target(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        runtime: &mut Runtime,
    ) {
        super::cityguardwithsword::release_guard_sword_target(self, region, monster_id, runtime);
    }

    fn execute_puniness_creature(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        runtime: &mut Runtime,
    ) -> bool {
        super::puninesscreature::execute_owned_puniness_creature(self, region, monster_id, runtime)
    }

    fn trace_guard_sword_target_ready(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        owner: ShapeView,
        target: ShapeView,
        minimum_distance: i32,
        maximum_distance: i32,
        chase_range: i32,
        runtime: &mut Runtime,
    ) -> bool {
        super::cityguardwithsword::trace_city_sword_target(
            self, region, monster_id, owner, target,
            minimum_distance, maximum_distance, chase_range, runtime,
        ) == super::cityguardwithsword::CitySwordTraceOutcome::Ready
    }
}

/// Общий приёмник результата диспетчерского вызова (прежняя сигнатура).
pub(crate) fn finish_monster_skill_call<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    outcome: MonsterSkillCallOutcome,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::monsterai::finish_monster_skill_call(
        game, region, monster_id, outcome, runtime, game_tick_milliseconds,
    )
}

/// Stiffen-переход пассивной фазы `ProcessPassiveAction` (прежняя сигнатура).
pub(crate) fn process_owned_monster_stiffen<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    runtime: &mut Runtime,
) -> PassiveStiffenAction {
    nebokrai_zone::ai::monsterai::process_owned_monster_stiffen(
        game, owner, monster_id, runtime, game_tick_milliseconds,
    )
}

/// Обычный virtual OnLoseTarget с разрешением через `CMonster::GetAI`
/// (прежняя сигнатура).
pub(crate) fn release_owned_monster_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) {
    nebokrai_zone::ai::monsterai::release_owned_monster_target(
        game, region, monster_id, runtime,
    )
}

/// MoveTo (0x004C9020): один Slip для ходьбы, два для ненулевого run,
/// затем Move и FIFO (прежняя сигнатура).
pub(crate) fn move_owned_monster_to(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeAreaCoordinates,
    run: i32,
    now: impl FnOnce() -> u32,
) {
    nebokrai_zone::ai::monsterai::move_owned_monster_to(
        game, region, monster_id, target, run, now,
    )
}

/// Ставит достигнутый общий `CMonsterAI::OnIdle` (прежняя сигнатура).
pub(crate) fn queue_monster_idle<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::monsterai::queue_monster_idle(
        game, region, monster_id, property, game_tick_milliseconds,
    )
}

/// Общий адаптер подхода оставшихся владельцев навыков (прежняя сигнатура).
pub(crate) fn approach_attack_range<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: MonsterTraceTarget,
    maximum_distance: u32,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::monsterai::approach_attack_range(
        game, region, monster_id, target, maximum_distance, game_tick_milliseconds,
    )
}

/// Virtual Tracing перед новым Begin с диапазоном зарегистрированного навыка
/// (прежняя сигнатура).
pub(crate) fn trace_owned_target_state_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut ServerRegionOwner,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::monsterai::trace_owned_target_state_skill(
        game, owner, monster_id, runtime, game_tick_milliseconds,
    )
}
