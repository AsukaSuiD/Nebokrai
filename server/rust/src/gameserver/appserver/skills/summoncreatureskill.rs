//! Тонкий путь к семье призыва `CSummonSkill` (0x19A/0x19B/0x19C/0x1F9) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/
//! {summoncreatureskill,summoncorpsecandle,summonskeleton,summonspore,
//! bossfiendsummon}.cpp. Тела Begin/AI/Summon перенесены буквально в
//! `nebokrai_zone::skills::summoncreatureskill` и `nebokrai_zone::skills::spidermist`
//! (основание и статусы MATCH см. там): прежний явный поворот в Begin —
//! расхождение реконструкции, удалён машинной сверкой (в телах навыка оригинала
//! поворота не существует; монстру поворот задаёт движение подхода, игроку —
//! клиент). Machine-адреса семьи и поправка карты полосы (Summon — НЕ
//! 4-классовый фолд; истинные VA `0x53E260/0x53E8D0/0x53F270`) перенесены в
//! шапку zone-владельца.
//! Здесь — объявленные швы переноса: фасадные реализации hub-трейтов
//! `SummonSkillPlayer`/`SummonSkillGame`/`SummonSkillContact` над прежними
//! методами `CPlayer`/`CGame`/`CServerRegion` и делегации с прежними
//! сигнатурами; внешние потребители (диспетчер `game.rs`, executors-реестр
//! `monsterbaseattack.rs`) не меняются. Свёртка `summon_monster_facts`
//! объединяет find_monster, базовую таблицу и приручённый attack interval:
//! наблюдаемые чтения и их порядок сохраняются.

use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::monsterattack::resolve_owned_monster_attack_target;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::state::{
    resolve_owned_skill_begin_object, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState, ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::execution::{
    PlayerSkillExecution, PlayerSpiderMistExecutionState, SpiderMistProgress,
};
use nebokrai_zone::skills::{
    SPIDER_MIST_SKILL_ID, SkillStage, SkillTermination, SummonMonsterCast, SummonMonsterFacts,
    SummonSkillContact, SummonSkillGame, SummonSkillOutcome, SummonSkillPlayer,
    summoncreatureskill as zone,
};

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SUMMON_VISUAL_MESSAGE: i32 = 0x000b_fe01;

impl SummonSkillPlayer for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }

    fn cast_face_direction(&mut self, direction: i32) {
        self.movement_shape_mut().set_direction(direction);
    }

    fn server_region_id(&self) -> Option<i32> { self.server_region_id() }

    fn set_skill_moveable(&mut self, moveable: bool) { self.set_skill_moveable(moveable) }

    fn set_current_skill_id(&mut self, skill_id: Option<u32>) { self.set_current_skill_id(skill_id) }

    fn master_info(&self) -> MasterInfo { super::flash::master_info(self) }
}

impl SummonSkillGame for CGame {
    type Player = CPlayer;
    type Region = CServerRegion;
    type RegionOwner = ServerRegionOwner;
    type PlayerAi = CPlayerAI;

    fn summon_region_base_mut(owner: &mut ServerRegionOwner) -> &mut CServerRegion {
        owner.base_mut()
    }

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.find_player_mut(player_id)
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32 {
        self.player_skill_last_used_ms(player_id, skill_id)
    }

    fn summon_player_skill_level(&self, player_id: i32, skill_id: u32) -> Option<i32> {
        Some(self.find_player(player_id)?.learned_skill_level(skill_id, self.skill_factory()))
    }

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView> {
        self.base_magic_target_view(region_id, target)
    }

    fn player_summon_creature_state(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&PlayerSummonCreatureExecutionState> {
        self.player_skill_state::<PlayerSummonCreatureExecutionState>(player_id, skill_id)
    }

    fn player_summon_creature_state_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut PlayerSummonCreatureExecutionState> {
        self.player_skill_state_mut::<PlayerSummonCreatureExecutionState>(player_id, skill_id)
    }

    fn player_spider_mist_state(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&PlayerSpiderMistExecutionState> {
        self.player_skill_state::<PlayerSpiderMistExecutionState>(player_id, skill_id)
    }

    fn player_spider_mist_state_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut PlayerSpiderMistExecutionState> {
        self.player_skill_state_mut::<PlayerSpiderMistExecutionState>(player_id, skill_id)
    }

    fn begin_player_execution(&mut self, player_id: i32, execution: PlayerSkillExecution) -> bool {
        self.begin_player_skill_execution(player_id, execution)
    }

    fn update_player_fight_state_move_shape(&mut self, player_id: i32) -> bool {
        self.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi).is_some()
    }

    fn send_summon_cast_failure(&self, player_id: i32, action: u8) {
        self.send_self_state_skill_failure(SUMMON_VISUAL_MESSAGE, player_id, action);
    }

    fn send_player_summon_visual(&mut self, player_id: i32, message: &nebokrai_zone::app::game_message::CMessage) {
        let _ = self.send_player_shape_around(player_id, None, message);
    }

    fn send_summon_visual_around(
        &self,
        region: &CServerRegion,
        origin: &CShape,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        let _ = self.send_game_shape_around(region, origin, None, message);
    }

    fn finish_summon_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut CPlayerAI,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        self.finish_player_skill(player_id, player_ai, expected, termination)
    }

    fn with_summon_region<Output>(
        &mut self,
        region_id: i32,
        callback: impl FnOnce(&mut Self, &mut CServerRegion) -> Output,
    ) -> Option<Output> {
        let mut owner = self.take_region_owner(region_id)?;
        let output = callback(self, owner.base_mut());
        self.restore_region_owner(owner);
        Some(output)
    }

    fn summon_random_region_position(
        &mut self,
        region: &CServerRegion,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Option<(i32, i32)> {
        self.random_region_position_owned(&region.region, left, top, width, height)
            .ok()
            .filter(|position| position.found)
            .map(|position| (position.x, position.y))
    }

    fn summon_add_creature_by_picture(
        &mut self,
        region: &mut CServerRegion,
        picture_id: u32,
        master: MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    ) -> bool {
        let Some(property) = self.find_monster_property_by_picture_id(picture_id).cloned()
        else { return false };
        self.add_summoned_creature_owned(region, &property, master, tile_x, tile_y, direction, lifetime_ms)
            .is_ok()
    }

    fn summon_allocate_shape_id(&mut self) -> i32 { self.allocate_summon_shape_id() }

    fn summon_monster_facts(
        &self,
        region: &CServerRegion,
        monster_id: i32,
        skill_id: u32,
    ) -> Option<SummonMonsterFacts> {
        let monster = region.find_monster_by_id(monster_id)?;
        let property = self
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let attack_interval_ms = monster
            .is_tamed()
            .then(|| monster.pet_attack_properties(&property))
            .map_or(property.attack_speed, |pet| pet.attack_interval);
        Some(SummonMonsterFacts {
            source: monster.move_shape().shape().clone(),
            attack_interval_ms,
            ai_kind: property.ai,
            cast: monster
                .current_active_attack_cast(self.skill_factory())
                .map(|cast| SummonMonsterCast {
                    skill_id: cast.dispatch().skill_id,
                    started_at_ms: cast.started_at_ms(),
                }),
            last_used_ms: monster.skill_last_used_ms(skill_id, self.skill_factory()),
        })
    }

    fn summon_monster_shape(&self, region: &CServerRegion, monster_id: i32) -> Option<CShape> {
        region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape().clone())
    }

    fn summon_monster_target_view(
        &self,
        region: &CServerRegion,
        target: ShapeIdentity,
    ) -> Option<ShapeView> {
        match target.object_type {
            PLAYER_TYPE => self.find_player(target.id).and_then(|player| {
                (player.server_region_id() == Some(region.id))
                    .then(|| player.shape_view())
                    .flatten()
            }),
            MONSTER_TYPE => region.find_monster_by_id(target.id).and_then(|monster| {
                let property = self
                    .find_monster_property_by_origin_name(monster.base_property_key()?)?;
                monster.shape_view(property)
            }),
            _ => None,
        }
    }

    fn summon_monster_attack_target(
        &self,
        owner: &ServerRegionOwner,
        target: ShapeIdentity,
    ) -> Option<(CShape, ShapeView)> {
        resolve_owned_monster_attack_target(self, owner, target)
            .map(|target| (target.shape, target.view))
    }

    fn summon_monster_begin_attack_attempt(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        now_ms: u32,
        interval_ms: u32,
    ) -> bool {
        region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval_ms))
    }

    fn summon_schedule_attack_interval(&self, ai_kind: u32, interval_ms: u32) -> Option<u32> {
        schedule_attack_interval(ai_kind, interval_ms)
    }

    fn summon_skill_begin_object(
        &self,
        region: &CServerRegion,
        target: ShapeIdentity,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_owned_skill_begin_object(self, region, target)
    }

    fn summon_monster_begin_cast(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        target: ShapeIdentity,
        skill_id: u32,
        skill_level: u16,
        now_ms: u32,
        target_object: Option<(i32, ShapeIdentity)>,
    ) {
        let factory = self.skill_factory();
        let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return };
        monster.begin_base_attack_cast(target, skill_id, skill_level, now_ms, target_object, factory);
    }

    fn summon_monster_advance_cast(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        skill_id: u32,
        from: SkillStage,
        to: SkillStage,
    ) {
        let factory = self.skill_factory();
        let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return };
        let _ = monster.advance_base_attack_cast(skill_id, from, to, factory);
    }

    fn summon_monster_cancel_cast(&mut self, region: &mut CServerRegion, monster_id: i32) {
        let factory = self.skill_factory();
        let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return };
        monster.cancel_base_attack_cast(factory);
    }

    fn summon_monster_clear_ai_target(&mut self, region: &mut CServerRegion, monster_id: i32) {
        let factory = self.skill_factory();
        let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return };
        monster.clear_ai_target(factory);
    }

    fn summon_monster_set_moveable(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        moveable: bool,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(moveable);
        }
    }

    fn summon_monster_face_direction(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        direction: i32,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
        }
    }

    fn summon_straight_skill_path(
        &self,
        region: &CServerRegion,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)> {
        region.straight_skill_path(source_x, source_y, target_x, target_y, None)
    }

    fn summon_base_magic_path(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)> {
        self.base_magic_path(region_id, source_x, source_y, target_x, target_y, None)
    }

    fn summon_spider_mist_progress(
        &self,
        region: &CServerRegion,
        monster_id: i32,
    ) -> Option<SpiderMistProgress> {
        region
            .find_monster_by_id(monster_id)
            .and_then(|monster| {
                monster.skill_progress::<SpiderMistProgress>(SPIDER_MIST_SKILL_ID, self.skill_factory())
            })
            .copied()
    }

    fn summon_set_spider_mist_progress(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        progress: SpiderMistProgress,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_skill_progress(SPIDER_MIST_SKILL_ID, progress, self.skill_factory());
        }
    }

    fn summon_region_cell_identities(
        &self,
        region_id: i32,
        tile_x: i32,
        tile_y: i32,
    ) -> Option<Vec<ShapeIdentity>> {
        let region = self.find_region(region_id).map(|owner| owner.base())?;
        let (width, height) = self.area_dimensions();
        let mut shapes = Vec::new();
        region
            .get_shapes(tile_x, tile_y, width, height, self, &mut shapes)
            .ok()?;
        Some(shapes.into_iter().map(|shape| shape.identity).collect())
    }

    fn summon_shape_present_in_region(&self, region_id: i32, identity: ShapeIdentity) -> bool {
        self.find_shape_in_region(region_id, identity).is_some()
    }

    fn summon_shape_region_of(&self, region_id: i32, identity: ShapeIdentity) -> Option<i32> {
        resolve_state_move_shape(self, region_id, identity)
            .map(|shape| shape.shape().get_region_id())
    }

    fn summon_move_shape_health(&self, region_id: i32, identity: ShapeIdentity) -> Option<u32> {
        self.move_shape_health(region_id, identity)
    }

    fn summon_shape_has_state(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        skill_id: u32,
    ) -> Option<bool> {
        resolve_state_move_shape(self, region_id, identity)
            .map(|shape| shape.has_state_by_skill_id(skill_id))
    }

    fn summon_skill_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        self.live_skill_target_attackable(region_id, source, target)
    }

    fn summon_begin_spider_poison_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        user: (i32, ShapeIdentity),
        state: nebokrai_zone::skills::statefactory::SpiderPoisonState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        super::spiderpoisonstate::begin_primary_spider_poison_state(
            self,
            region_id,
            holder,
            Some(user),
            Some((region_id, holder)),
            state,
            None,
            now,
        )
        .is_some()
    }

    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }
}

impl<Runtime: GameMainLoopRuntime> SummonSkillContact<Runtime> for CGame {
    fn summon_approach_attack_range(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        target_view: ShapeView,
        maximum_distance: u32,
        runtime: &mut Runtime,
    ) -> bool {
        approach_attack_range(
            self,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target_view),
            maximum_distance,
            runtime,
        )
    }

    fn summon_monster_finish_cast_clock(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    ) {
        let factory = self.skill_factory();
        let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return };
        let _ = monster.finish_base_attack_cast_with_clock(skill_id, factory, || {
            runtime.now_milliseconds()
        });
    }

    fn summon_add_spider_mist_phalanx(
        &mut self,
        region: &mut CServerRegion,
        phalanx_id: i32,
        rule: &nebokrai_zone::skills::spidermist::SpiderMistPhalanx,
        destination_x: i32,
        destination_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<i32> {
        let (area_width, area_height) = self.area_dimensions();
        let phalanx = super::spidermistphalanx::CSpiderMistPhalanx::from_rule(phalanx_id, rule.clone());
        let phalanx_id = region
            .add_spider_mist_phalanx(
                phalanx, destination_x, destination_y,
                area_width, area_height, started_at_ms, runtime,
            )
            .ok()?;
        super::spidermist::send_phalanx_entry(self, region, phalanx_id);
        Some(phalanx_id)
    }

    fn summon_after_use_player_skill(
        &mut self,
        player_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    ) {
        self.after_use_player_skill(player_id, skill_id, runtime);
    }
}

pub(crate) fn is_player_summon_creature_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_player_summon_creature_dispatch(dispatch)
}

pub(crate) fn cancel_player_summon_creature<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    execution_skill_id: u32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    zone::cancel_player_summon_creature(game, player_id, execution_skill_id, player_ai)
}

pub(crate) fn execute_player_summon_creature<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let state = match zone::execute_player_summon_creature(
        game, player_id, dispatch, runtime, game_tick_milliseconds,
    ) {
        SummonSkillOutcome::Begun => QueuedSkillExecutionState::Begun,
        SummonSkillOutcome::Pending => QueuedSkillExecutionState::Pending,
        SummonSkillOutcome::Completed => QueuedSkillExecutionState::Completed,
        SummonSkillOutcome::Rejected => QueuedSkillExecutionState::Rejected,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет идентификатор, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_summon_creature<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    zone::execute_owned_summon_creature(
        game, region, monster_id, target, skill_id, skill_level, properties, now_ms, runtime,
    )
}

pub(crate) use nebokrai_zone::skills::execution::PlayerSummonCreatureExecutionState;

