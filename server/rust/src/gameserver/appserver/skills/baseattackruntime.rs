//! Тонкий путь к исполнению `CBaseAttack` игроком и монстром в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/baseattack.cpp.
//! Стадии Begin/AI/Attack, формула, visual и terminal перенесены буквально в
//! `nebokrai_zone::skills::baseattackruntime` (основание и статусы MATCH см.
//! там). Здесь — объявленные швы переноса: фасадные реализации трейтов Zone
//! над прежними методами `CGame` и делегации с прежними сигнатурами; потребители
//! не меняются. Свёртка `base_attack_weapon_modifier` объединяет find_player,
//! чтение факторов globe и вызов `weapon_modifier`: наблюдаемые записи
//! `damage_factor` и их порядок сохраняются. Around-отправка внутри шва
//! сохраняет гейт существующего региона прежнего caller-а.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapeSkill};
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::kernel::SkillExecutionKernel;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffect;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use crate::nets::netserver::message::GameMessageDomainOps;
use nebokrai_zone::combat::PlayerCombatProperties;
use nebokrai_zone::skills::{
    BaseAttackContact, BaseAttackExecutionOutcome, BaseAttackGame, BaseAttackMoveShape,
    BaseAttackPkPermissions, BaseAttackPlayer, SkillLifecycle, SkillTermination,
    execute_player_base_attack as zone_execute_player_base_attack,
    publish_base_attack_visual as zone_publish_base_attack_visual,
};
use nebokrai_zone::skills::execution::RegisteredSkillRecord;

impl BaseAttackPlayer for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }
    fn movement_shape_mut(&mut self) -> &mut CShape { self.movement_shape_mut() }
    fn shape_view(&self) -> Option<ShapeView> { self.shape_view() }
    fn set_skill_moveable(&mut self, moveable: bool) { self.set_skill_moveable(moveable) }
    fn combat_properties(&self) -> PlayerCombatProperties { self.combat_properties() }
    fn faction_id(&self) -> i32 { self.faction_id() }
    fn team_id(&self) -> i32 { self.team_id() }
    fn union_id(&self) -> i32 { self.union_id() }
    fn country(&self) -> u8 { self.country() }
    fn pk_permissions(&self) -> BaseAttackPkPermissions {
        let permissions = self.pk_permissions();
        // MasterInfo пользуется четырьмя допусками и отдельной страной.
        BaseAttackPkPermissions {
            player: permissions.player,
            teammate: permissions.teammate,
            guild_member: permissions.guild_member,
            criminal: permissions.criminal,
        }
    }
}

impl BaseAttackMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }
    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }
}

impl BaseAttackGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type Player = CPlayer;
    type MoveShape = CMoveShape;
    type PlayerAi = CPlayerAI;
    type RegionOwner = ServerRegionOwner;

    fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<RegisteredSkill> {
        self.registered_player_skill(player_id, skill_id)
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

    fn registered_skill_mut(
        &mut self,
        address: RegisteredSkill,
    ) -> Option<&mut RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill_mut(address)
    }

    fn player_skill_execution(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.player_skill_execution(player_id, skill_id)
    }

    fn player_skill_execution_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.player_skill_execution_mut(player_id, skill_id)
    }

    fn player_skill_lifecycle(&self, player_id: i32, skill_id: u32) -> Option<&SkillLifecycle> {
        self.player_skill_lifecycle(player_id, skill_id)
    }

    fn begin_player_skill_execution(
        &mut self,
        player_id: i32,
        kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool {
        self.begin_player_skill_execution(player_id, kernel)
    }

    fn replace_player_skill_visual_effect(
        &mut self,
        player_id: i32,
        skill_id: u32,
        effect: SkillVisualEffect,
    ) -> bool {
        self.replace_player_skill_visual_effect(player_id, skill_id, effect)
    }

    fn update_player_skill_visual(&mut self, player_id: i32, skill_id: u32, mode: u32) {
        self.update_player_skill_visual(player_id, skill_id, mode)
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        self.update_registered_skill_visual(address, mode)
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, lifecycle)
    }

    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&CMoveShape> {
        resolve_state_move_shape(self, region_id, identity)
    }

    fn resolve_state_move_shape_mut(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&mut CMoveShape> {
        resolve_state_move_shape_mut(self, region_id, identity)
    }

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> {
        self.find_player(player_id)
    }

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.find_player_mut(player_id)
    }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }

    fn base_magic_target_view(&self, region_id: i32, target: ShapeIdentity) -> Option<ShapeView> {
        self.base_magic_target_view(region_id, target)
    }

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8> {
        self.move_shape_level(region_id, target)
    }

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        user: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        self.live_skill_target_attackable(region_id, user, target)
    }

    fn weapon_damage_factors(&self) -> (f32, f32) {
        self.globe_setup().weapon_damage_factors()
    }

    fn base_attack_weapon_modifier(
        &self,
        player_id: i32,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> Option<f32> {
        let player = self.find_player(player_id)?;
        Some(player.weapon_modifier(self.goods_factory(), target_level, divisor, minimum_factor))
    }

    fn critical_rate(&self) -> f32 {
        self.globe_setup().critical_rate()
    }

    fn skill_random_below(&mut self, maximum: i32) -> i32 {
        self.skill_random_below(maximum)
    }

    fn base_attack_monster_holder(
        &self,
        owner: &ServerRegionOwner,
        monster_id: i32,
    ) -> Option<(i32, ShapeIdentity)> {
        let region = owner.base();
        let monster = region.find_monster_by_id(monster_id)?;
        Some((region.id, monster.move_shape().shape().identity()))
    }

    fn monster_base_attack_bounds(&self, source: (i32, ShapeIdentity)) -> Option<(u32, u32)> {
        let monster = self.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
        let properties = self.find_monster_property_by_origin_name(monster.base_property_key()?)?;
        Some(monster.state_attack_bounds(properties.minimum_attack, properties.maximum_attack))
    }

    fn monster_base_attack_soul_attack(&self, source: (i32, ShapeIdentity)) -> Option<u16> {
        let monster = self.find_region(source.0)
            .and_then(|region| region.base().find_monster_by_id(source.1.id))?;
        let properties = monster.base_property_key()
            .and_then(|name| self.find_monster_property_by_origin_name(name))?;
        Some(monster.soul_attack(properties))
    }

    fn send_base_attack_visual_to_player(&self, player_id: i32, message: &crate::nets::netserver::message::CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_base_attack_visual_around(
        &self,
        region_id: i32,
        origin: &CShape,
        message: &crate::nets::netserver::message::CMessage,
    ) {
        // Гейт существующего региона прежнего caller-а сохранён.
        if let Some(region) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(region.base(), origin, None, message);
        }
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

    fn end_registered_instance<Runtime>(
        &mut self,
        address: RegisteredSkill,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
    ) {
        let _ = self.end_registered_instance(address, argument, termination, runtime);
    }

    fn end_registered_instance_without_after_use(
        &mut self,
        address: RegisteredSkill,
        termination: SkillTermination,
    ) {
        let _ = self.end_registered_instance_without_after_use(address, termination);
    }

    fn finish_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut CPlayerAI,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        self.finish_player_skill(player_id, player_ai, dispatch, termination)
    }

    fn finish_registered_player_command(
        &mut self,
        address: Option<RegisteredSkill>,
        player_ai: &mut CPlayerAI,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        self.finish_registered_player_command(address, player_ai, dispatch, termination)
    }

    fn increase_owned_player_rp(&mut self, player_id: i32, attacking: bool, damage: u16) {
        self.increase_owned_player_rp(player_id, attacking, damage)
    }
}

impl<Runtime: GameMainLoopRuntime> BaseAttackContact<Runtime> for CGame {
    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_contact(master, target, region_id, attack, runtime)
    }
}

pub(crate) fn publish_base_attack_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    zone_publish_base_attack_visual(game, skill, mode)
}

pub(super) fn execute_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let state = match zone_execute_player_base_attack(
        game, player_id, dispatch, ai, runtime, game_tick_milliseconds,
    ) {
        BaseAttackExecutionOutcome::Begun => QueuedSkillExecutionState::Begun,
        BaseAttackExecutionOutcome::Pending => QueuedSkillExecutionState::Pending,
        BaseAttackExecutionOutcome::Completed => QueuedSkillExecutionState::Completed,
        BaseAttackExecutionOutcome::Rejected => QueuedSkillExecutionState::Rejected,
        BaseAttackExecutionOutcome::RejectedAfterUse => QueuedSkillExecutionState::RejectedAfterUse,
    };
    // Runtime выставляет event только в tick фактического первого контакта;
    // у базовой атаки его носит приёмник.
    QueuedSkillExecutionOutcome { state, first_contact: false }
}
