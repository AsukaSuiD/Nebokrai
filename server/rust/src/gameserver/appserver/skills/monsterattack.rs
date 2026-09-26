//! Тонкий путь к общей доставке удара боевых навыков монстров в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/monsterattack.cpp.
//! Тела resolver/delivery перенесены в `nebokrai_zone::skills::monsterattack`
//! (основание и статусы см. там): `CMonster::IsAttackAble` (RVA `0x0E7230`,
//! ветви player/monster до PK/tame), mapping `End(0)` без reuse и публикация
//! настоящего производного региона на время попадания. Здесь — объявленные
//! швы переноса: фасадные реализации hub-трейтов `MonsterCombatPlayer`/
//! `MonsterCombatGame`/`MonsterCombatContact` над прежними методами
//! `CPlayer`/`CGame`/`CServerRegion`/`ServerRegionOwner` и делегации с
//! прежними сигнатурами; внешние потребители (boss-навыки, machinerystomp,
//! game.rs, hub `monsterbaseattack.rs`) не меняются. Свёртка
//! `monster_combat_facts` объединяет find_monster, базовую таблицу и
//! приручённый attack interval: наблюдаемые чтения и их порядок сохраняются.

use crate::gameserver::appserver::ai::monsterai::{MonsterTraceTarget, approach_attack_range};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerCombatProperties, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::state::{
    resolve_coordinate_sufferer, resolve_identity_sufferer, resolve_owned_skill_begin_object,
    resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, RegionShapeResolver, ServerRegionOwner,
};
use crate::setup::monsterlist::MonsterProperties;
use nebokrai_shared::resources::GlobeSetupSnapshot;
use nebokrai_zone::skills::execution::PlayerMonsterThornExecutionState;
use nebokrai_zone::skills::{
    MonsterCombatCast, MonsterCombatContact, MonsterCombatFacts, MonsterCombatGame,
    MonsterCombatPlayer, MonsterShapeFacts, MonsterTamingTarget,
    SkillExecutionKernel, SkillStage, SkillTermination,
    monsterattack as zone,
};
pub(crate) use nebokrai_zone::skills::OwnedMonsterAttackTarget;

const VISUAL_MESSAGE: i32 = 0x000b_fe01;

impl MonsterCombatPlayer for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }

    fn shape_view(&self) -> Option<ShapeView> { self.shape_view() }

    fn face_cast_direction(&mut self, direction: i32) {
        self.movement_shape_mut().set_direction(direction);
    }

    fn combat_properties(&self) -> PlayerCombatProperties { self.combat_properties() }

    fn server_region_id(&self) -> Option<i32> { self.server_region_id() }

    fn is_dead(&self) -> bool { self.is_dead() }

    fn is_god_mode(&self) -> bool { self.is_god_mode() }

    fn city_war_died_state(&self) -> bool { self.city_war_died_state() }

    fn mana(&self) -> u32 { self.mana() }

    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }

    fn set_skill_moveable(&mut self, moveable: bool) { self.set_skill_moveable(moveable) }

    fn set_current_skill_id(&mut self, skill_id: Option<u32>) { self.set_current_skill_id(skill_id) }

    fn master_info(&self) -> MasterInfo { super::flash::master_info(self) }

    fn level(&self) -> u8 { self.level() }

    fn player_name(&self) -> &[u8] { self.player_name() }

    fn current_pets_mode(&self) -> i32 { self.current_pets_mode() }

    fn active_pets_count(&self) -> u32 { self.active_pets().len() as u32 }

    fn add_active_pet(&mut self, object_type: i32, id: i32, figure: i32) {
        self.add_active_pet(object_type, id, figure);
    }
}

impl MonsterCombatGame for CGame {
    type Player = CPlayer;
    type Region = CServerRegion;
    type RegionOwner = ServerRegionOwner;
    type PlayerAi = CPlayerAI;

    fn owner_base(owner: &ServerRegionOwner) -> &CServerRegion { owner.base() }

    fn owner_base_mut(owner: &mut ServerRegionOwner) -> &mut CServerRegion { owner.base_mut() }

    fn owner_region_id(owner: &ServerRegionOwner) -> i32 { owner.region_id() }

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

    fn monster_combat_player_skill_level(&self, player_id: i32, skill_id: u32) -> Option<i32> {
        Some(self.find_player(player_id)?.learned_skill_level(skill_id, self.skill_factory()))
    }

    fn globe_setup(&self) -> &GlobeSetupSnapshot { self.globe_setup() }

    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }

    fn player_kernel(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.player_skill_execution(player_id, skill_id)
    }

    fn player_kernel_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.player_skill_execution_mut(player_id, skill_id)
    }

    fn begin_player_kernel(
        &mut self,
        player_id: i32,
        kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool {
        self.begin_player_skill_execution(player_id, kernel)
    }

    fn begin_player_combat_command(
        &mut self,
        player_id: i32,
        dispatch: PlayerSkillDispatch,
        started_at_ms: u32,
    ) {
        let _ = self.begin_player_skill_with_combat(player_id, dispatch, started_at_ms);
    }

    fn player_monster_thorn_state(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&PlayerMonsterThornExecutionState> {
        self.player_skill_state::<PlayerMonsterThornExecutionState>(player_id, skill_id)
    }

    fn player_monster_thorn_state_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut PlayerMonsterThornExecutionState> {
        self.player_skill_state_mut::<PlayerMonsterThornExecutionState>(player_id, skill_id)
    }

    fn begin_player_monster_thorn_execution(
        &mut self,
        player_id: i32,
        state: PlayerMonsterThornExecutionState,
    ) -> bool {
        self.begin_player_skill_execution(player_id, state)
    }

    fn update_player_fight_state_move_shape(&mut self, player_id: i32) -> bool {
        self.update_player_current_state(
            player_id,
            crate::gameserver::gameserver::game::GamePlayerFightStatePhase::MoveShapeAi,
        )
        .is_some()
    }

    fn publish_player_states(&self, player_id: i32) {
        let _ = self.publish_player_states(player_id);
    }

    fn send_cast_failure(&self, player_id: i32, reason: u8) {
        self.send_self_state_skill_failure(VISUAL_MESSAGE, player_id, reason);
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn send_skill_system_info_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount)
    }

    fn send_skill_system_info_text(&self, player_id: i32, text: &[u8], argument: &[u8]) {
        self.send_skill_system_info_with_text(player_id, text, argument)
    }

    fn send_player_visual(&mut self, player_id: i32, message: &nebokrai_zone::app::game_message::CMessage) {
        let _ = self.send_player_shape_around(player_id, None, message);
    }

    fn send_visual_around(
        &self,
        region: &CServerRegion,
        origin: &CShape,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        let _ = self.send_game_shape_around(region, origin, None, message);
    }

    fn with_published_player_ai<Output>(
        &mut self,
        player_id: i32,
        player_ai: &mut CPlayerAI,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Output {
        self.with_published_player_ai(player_id, player_ai, callback)
    }

    fn with_published_region<Output>(
        &mut self,
        owner: &mut Option<ServerRegionOwner>,
        callback: impl FnOnce(&mut Self) -> Output,
    ) -> Option<Output> {
        self.with_published_region(owner, callback)
    }

    fn finish_monster_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut CPlayerAI,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        self.finish_player_skill(player_id, player_ai, expected, termination)
    }

    fn resolve_identity_sufferer(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeIdentity> {
        resolve_identity_sufferer(self, region_id, target)
    }

    fn resolve_coordinate_sufferer(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeIdentity> {
        resolve_coordinate_sufferer(self, region_id, x, y)
    }

    fn resolve_owned_skill_begin_object(
        &self,
        region: &CServerRegion,
        target: ShapeIdentity,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_owned_skill_begin_object(self, region, target)
    }

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView> {
        self.base_magic_target_view(region_id, target)
    }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }

    fn base_magic_path(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)> {
        self.base_magic_path(region_id, source_x, source_y, target_x, target_y, None)
    }

    fn base_magic_target_point_in(
        &self,
        owner: &ServerRegionOwner,
        source_x: i32,
        source_y: i32,
        identity: ShapeIdentity,
    ) -> Option<(i32, i32)> {
        self.base_magic_target_point_in(owner, source_x, source_y, identity)
    }

    fn base_magic_target_point(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        identity: ShapeIdentity,
    ) -> Option<(i32, i32)> {
        self.base_magic_target_point(region_id, source_x, source_y, identity)
    }

    fn live_skill_target_attackable_in(
        &self,
        owner: &ServerRegionOwner,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        self.live_skill_target_attackable_in(owner, source, target)
    }

    fn stationary_build_attackable_by_player(
        &self,
        player_id: i32,
        region_id: i32,
        target: ShapeIdentity,
    ) -> bool {
        self.stationary_build_attackable_by_player(player_id, region_id, target)
    }

    fn owned_player_skill_target_attackable(
        &self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
    ) -> bool {
        self.owned_player_skill_target_attackable(master, target, region_id)
    }

    fn monster_combat_target_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8> {
        super::flash::target_level(self, region_id, target)
    }

    fn monster_combat_cell_views(&self, region_id: i32, tile_x: i32, tile_y: i32) -> Vec<ShapeView> {
        super::flash::cell_views(self, region_id, tile_x, tile_y)
    }

    fn state_move_shape_region_id(&self, region_id: i32, target: ShapeIdentity) -> Option<i32> {
        resolve_state_move_shape(self, region_id, target).map(|shape| shape.shape().get_region_id())
    }

    fn monster_combat_weapon_modifier(&self, player_id: i32, target_level: i32) -> f32 {
        let Some(player) = self.find_player(player_id) else { return 1.0; };
        let (divisor, floor) = self.globe_setup().weapon_damage_factors();
        player.weapon_modifier(self.goods_factory(), target_level, divisor, floor)
    }

    fn monster_combat_weapon_damage_level(&self, player_id: i32) -> Option<i32> {
        Some(self.find_player(player_id)?.weapon_damage_level(self.goods_factory()))
    }

    fn monster_combat_calculate_attack(
        &mut self,
        player_id: i32,
        skill_id: u32,
        level: i32,
        hit_modifier: i32,
    ) -> Option<(MasterInfo, AttackInformation)> {
        super::lordfastattack::calculate_attack(self, player_id, skill_id, level, hit_modifier)
    }

    fn monster_combat_facts(
        &self,
        region: &CServerRegion,
        monster_id: i32,
        skill_id: u32,
    ) -> Option<MonsterCombatFacts> {
        let monster = region.find_monster_by_id(monster_id)?;
        let property = self
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let attack_interval_ms = monster
            .is_tamed()
            .then(|| monster.pet_attack_properties(&property))
            .map_or(property.attack_speed, |pet| pet.attack_interval);
        Some(MonsterCombatFacts {
            source: monster.move_shape().shape().clone(),
            attack_interval_ms,
            ai_kind: property.ai,
            property,
            cast: monster
                .current_active_attack_cast(self.skill_factory())
                .map(|cast| MonsterCombatCast {
                    target: cast.dispatch().target,
                    skill_id: cast.dispatch().skill_id,
                    skill_level: cast.dispatch().skill_level,
                    started_at_ms: cast.started_at_ms(),
                    stage: cast.stage(),
                }),
            last_used_ms: monster.skill_last_used_ms(skill_id, self.skill_factory()),
        })
    }

    fn monster_shape_facts(&self, region: &CServerRegion, monster_id: i32) -> Option<MonsterShapeFacts> {
        let monster = region.find_monster_by_id(monster_id)?;
        let property = self
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let view = monster.shape_view(&property)?;
        Some(MonsterShapeFacts {
            shape: monster.move_shape().shape().clone(),
            property,
            view,
            hit_points: monster.hit_points(),
            god: monster.move_shape().is_god(),
        })
    }

    fn monster_shape(&self, region: &CServerRegion, monster_id: i32) -> Option<CShape> {
        region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape().clone())
    }

    fn monster_begin_attack_attempt(
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

    fn monster_set_moveable(&mut self, region: &mut CServerRegion, monster_id: i32, moveable: bool) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(moveable);
        }
    }

    fn monster_set_direction(&mut self, region: &mut CServerRegion, monster_id: i32, direction: i32) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
        }
    }

    fn monster_set_action(&mut self, region: &mut CServerRegion, monster_id: i32, action: i32) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            // CShape::set_action хранит u16 (действие 1 семьи ударов влезает).
            monster.move_shape_mut().shape_mut().set_action(action as u16);
        }
    }

    fn monster_cast_present(&self, region: &CServerRegion, monster_id: i32, skill_id: u32) -> bool {
        region
            .find_monster_by_id(monster_id)
            .is_some_and(|monster| monster.base_attack_cast(skill_id, self.skill_factory()).is_some())
    }

    fn monster_finish_cast_without_reuse(&mut self, region: &mut CServerRegion, monster_id: i32, skill_id: u32) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.finish_base_attack_cast_without_reuse(skill_id, self.skill_factory());
        }
    }

    fn monster_advance_cast(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        skill_id: u32,
        from: SkillStage,
        to: SkillStage,
    ) {
        let factory = self.skill_factory();
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(skill_id, from, to, factory);
        }
    }

    fn monster_install_cast(
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
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            // Точный `install_base_attack_cast` прежних тел thorn/range:
            // экземпляр ставится в stage Begin без разового advance
            // (Begin-визуал/поворот достаются следующему AI-тику).
            monster.install_base_attack_cast(
                crate::gameserver::appserver::monster::MonsterBaseAttackCast::begin(
                    crate::gameserver::appserver::monster::MonsterBaseAttackDispatch {
                        target,
                        skill_id,
                        skill_level,
                    },
                    now_ms,
                ),
                target_object,
                factory,
            );
        }
    }

    fn monster_clear_ai_target(&mut self, region: &mut CServerRegion, monster_id: i32) {
        let factory = self.skill_factory();
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(factory);
        }
    }

    fn monster_state_attack_bounds(
        &self,
        region: &CServerRegion,
        monster_id: i32,
        minimum: u32,
        maximum: u32,
    ) -> Option<(u32, u32)> {
        Some(region.find_monster_by_id(monster_id)?.state_attack_bounds(minimum, maximum))
    }

    fn monster_region_cell_identities(
        &self,
        owner: &ServerRegionOwner,
        tile_x: i32,
        tile_y: i32,
    ) -> Vec<ShapeIdentity> {
        let (area_width, area_height) = self.area_dimensions();
        let resolver = RegionShapeResolver { game: self, owner };
        let region = owner.base();
        let mut shapes = Vec::new();
        if region
            .get_shapes(tile_x, tile_y, area_width, area_height, &resolver, &mut shapes)
            .is_err()
        {
            return Vec::new();
        }
        shapes.into_iter().map(|shape| shape.identity).collect()
    }

    fn npc_move_shape_facts(&self, region: &CServerRegion, npc_id: i32) -> Option<(CShape, bool)> {
        let npc = region.find_npc_by_id(npc_id)?;
        Some((npc.move_shape().shape().clone(), npc.move_shape().is_god()))
    }

    fn stationary_build_facts(
        &self,
        owner: &ServerRegionOwner,
        identity: ShapeIdentity,
    ) -> Option<(CShape, bool, u32)> {
        let build = owner.stationary_build(identity)?;
        Some((build.move_shape().shape().clone(), build.move_shape().is_god(), build.hp()))
    }

    fn shape_view_in_owner(&self, owner: &ServerRegionOwner, identity: ShapeIdentity) -> Option<ShapeView> {
        self.shape_view_in_owner(owner, identity)
    }

    fn monster_straight_skill_path(
        &self,
        region: &CServerRegion,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> Vec<(i32, i32, u8)> {
        region.straight_skill_path(source_x, source_y, target_x, target_y, None)
    }

    fn monster_taming_target_snapshot(&self, region_id: i32, monster_id: i32) -> Option<MonsterTamingTarget> {
        let monster = self.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
        let property = self
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        Some(MonsterTamingTarget {
            tile_x: monster.move_shape().shape().get_tile_x().ok()?,
            tile_y: monster.move_shape().shape().get_tile_y().ok()?,
            display_name: monster.display_name().to_vec(),
            hit_points: monster.hit_points(),
            tamable: monster.is_tamable(&property),
            property,
        })
    }

    fn monster_combat_cell_blocked(&self, region_id: i32, tile_x: i32, tile_y: i32) -> bool {
        self.find_region(region_id).is_none_or(|region| {
            region.base().region.get_block(tile_x, tile_y).unwrap_or(2) == 2
        })
    }

    fn with_monster_combat_region<Output>(
        &mut self,
        region_id: i32,
        callback: impl FnOnce(&mut Self, &mut CServerRegion) -> Output,
    ) -> Option<Output> {
        let mut owner = self.take_region_owner(region_id)?;
        let output = callback(self, owner.base_mut());
        self.restore_region_owner(owner);
        Some(output)
    }

    fn monster_combat_stop_all_skills(&mut self, region_id: i32, holder: ShapeIdentity) {
        self.stop_all_move_shape_skills(region_id, holder);
    }

    fn monster_increase_tame_attempt(&mut self, region: &mut CServerRegion, monster_id: i32) -> bool {
        region.find_monster_by_id_mut(monster_id).is_some_and(|monster| {
            monster.increase_tame_attempt_count();
            true
        })
    }

    fn monster_try_become_tamed(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        master: MasterInfo,
        pet_mode: i32,
    ) -> bool {
        region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.try_become_tamed(master, pet_mode))
    }

    fn monster_region_shape(&self, region: &CServerRegion, monster_id: i32) -> Option<CShape> {
        region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape().clone())
    }

    fn monster_pet_progress(&self, region: &CServerRegion, monster_id: i32) -> Option<(u32, u32)> {
        Some(region.find_monster_by_id(monster_id)?.pet_progress())
    }

    fn monster_upgrade_pet_level(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        property: &MonsterProperties,
        experience_factor: f32,
        current_factors: Option<[f32; 10]>,
        next_factors: Option<[f32; 10]>,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.increase_pet_experience(0, property, experience_factor, current_factors, next_factors);
        }
    }

    fn monster_hp_snapshot(
        &self,
        region: &CServerRegion,
        monster_id: i32,
        property: &MonsterProperties,
    ) -> Option<(u32, u32)> {
        let monster = region.find_monster_by_id(monster_id)?;
        Some((monster.hit_points(), monster.maximum_hp(property)))
    }

    fn monster_taming_decrease_refresh(&mut self, region: &mut CServerRegion, monster_id: i32) {
        let _ = region.finish_owned_monster_taming(monster_id);
    }
}

impl<Runtime: GameMainLoopRuntime> MonsterCombatContact<Runtime> for CGame {
    fn monster_combat_approach_attack_range(
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

    fn monster_finish_cast_clock(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    ) {
        let factory = self.skill_factory();
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.finish_base_attack_cast_with_clock(skill_id, factory, || {
                runtime.now_milliseconds()
            });
        }
    }

    fn monster_combat_after_use_player_skill(
        &mut self,
        player_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    ) {
        self.after_use_player_skill(player_id, skill_id, runtime);
    }

    fn monster_combat_increase_rp(&mut self, player_id: i32, attacking: bool, damage: u16) {
        self.increase_owned_player_rp(player_id, attacking, damage);
    }

    fn monster_combat_apply_attack_to_player(
        &mut self,
        master: MasterInfo,
        player_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime);
    }

    fn monster_combat_apply_attack_to_monster(
        &mut self,
        master: MasterInfo,
        monster_id: i32,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime);
    }

    fn monster_combat_apply_attack_to_build(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.receive_stationary_build_skill_attack(region_id, target, attack, runtime);
    }
}

/// Owner `End(0)` живого cast без reuse-штампа; прежняя форма вызова
/// сохранена, аргумент фабрики замещён владельцем с тем же доступом
/// (`CGame::skill_factory`, объявленный шов переноса).
pub(crate) fn end_owned_monster_skill_without_reuse(
    region: &mut CServerRegion,
    monster_id: i32,
    skill_id: u32,
    game: &mut CGame,
) -> bool {
    zone::end_owned_monster_skill_without_reuse(game, region, monster_id, skill_id)
}

pub(crate) fn finish_owned_monster_attack_impact<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    skill_id: u32,
    game: &mut CGame,
    runtime: &mut Runtime,
) {
    zone::finish_owned_monster_attack_impact(game, region, monster_id, skill_id, runtime)
}

/// Живой упорядоченный снимок одной клетки для конкретного удара.
pub(crate) fn monster_attack_cell_candidates(
    game: &CGame,
    owner: &ServerRegionOwner,
    source_monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    zone::monster_attack_cell_candidates(game, owner, source_monster_id, tile_x, tile_y)
}

pub(crate) fn resolve_owned_monster_attack_target(
    game: &CGame,
    owner: &ServerRegionOwner,
    identity: ShapeIdentity,
) -> Option<OwnedMonsterAttackTarget> {
    zone::resolve_owned_monster_attack_target(game, owner, identity)
}

pub(crate) fn apply_owned_monster_attack_hit<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    runtime: &mut Runtime,
    target: ShapeIdentity,
    attack: AttackInformation,
) {
    zone::apply_owned_monster_attack_hit(game, owner, runtime, target, attack)
}
