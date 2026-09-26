//! Общая геометрия, визуальный формат и контактная атака рывков
//! Flash/LittleFlash и hub-швы семейства melee-рывков (dash/flash/littleflash/
//! rush) — тела перенесены буквально в Zone `skills/{dash,flash,littleflash,
//! rush}.rs` (порция №5 «player melee», запись аудита «Zone skills: машинная
//! разведка melee dash/flash/littleflash/rush (порция №5)»; основание и
//! машинные статусы см. там). Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/flash.cpp, littleflash.cpp и littleflash2.cpp.
//! Здесь — объявленные швы переноса: фасадные реализации трейтов Zone над
//! прежними методами `CGame`/`CPlayer`/`CMoveShape` и общий outcome-шов
//! делегатов; потребители не меняются.
//!
//! Региональные `dash_*` фасады перечитывают owner-а региона на каждый вызов
//! (первичный гейт find_region конкретных zone-тел сохранён); random-поиск
//! свободной клетки идёт через тот же `GetRandomPosInRange` от переданного
//! runtime и потому сохраняет исходный поток случайных чисел. Контакт семьи
//! проходит прежний общий `apply_player_weapon_attack`; Begin состояний
//! рывков — прежний общий `begin_primary_blind_state` семейства Blind.

use super::blindstate::begin_primary_blind_state;
use super::skillbaseproperties::CSkillBaseProperties;
use super::weaponattack::apply_player_weapon_attack;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    RegionShapeResolver,
};
use crate::nets::netserver::message::{CMessage, GameMessageDomainOps};
use nebokrai_zone::combat::MasterInfo;
use nebokrai_zone::skills::dash::{
    DashSkillContact, DashSkillExecutionOutcome, DashSkillGame, DashSkillMoveShape,
    DashSkillPkPermissions, DashSkillPlayer,
};
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use super::rushstate::RushState;
use super::rushstate2::Rush2State;
use nebokrai_zone::skills::state::StateKey;
use nebokrai_zone::skills::SkillLifecycle;

impl DashSkillPlayer for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }
    fn mana(&self) -> u32 { self.mana() }
    fn rp(&self) -> u16 { self.rp() }
    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }
    fn set_rp(&mut self, rp: u16) { self.set_rp(rp) }
    fn level(&self) -> u8 { self.level() }
    fn player_id(&self) -> i32 { self.player_id() }
    fn faction_id(&self) -> i32 { self.faction_id() }
    fn team_id(&self) -> i32 { self.team_id() }
    fn union_id(&self) -> i32 { self.union_id() }
    fn country(&self) -> u8 { self.country() }
    fn pk_permissions(&self) -> DashSkillPkPermissions {
        let permissions = self.pk_permissions();
        // MasterInfo пользуется четырьмя допусками и отдельной страной.
        DashSkillPkPermissions {
            player: permissions.player,
            teammate: permissions.teammate,
            guild_member: permissions.guild_member,
            criminal: permissions.criminal,
        }
    }
    fn has_state_by_skill_id(&self, skill_id: u32) -> bool { self.has_state_by_skill_id(skill_id) }
    fn set_skill_moveable(&mut self, moveable: bool) { self.set_skill_moveable(moveable) }
}

impl DashSkillMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }
    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }
    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }
}

impl DashSkillGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type Player = CPlayer;
    type MoveShape = CMoveShape;

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

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> { self.find_player_mut(player_id) }

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }

    fn skill_target_path_with_length(
        &self,
        lifecycle: &SkillLifecycle,
        maximum: u32,
    ) -> Vec<(i32, i32, u8)> {
        self.skill_target_path_with_length(lifecycle, maximum)
    }

    fn dash_skill_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8> {
        Some(self.find_region(region_id)?.base().skill_cell_block(x, y))
    }

    fn dash_region_size(&self, region_id: i32) -> Option<(i32, i32)> {
        let region = self.find_region(region_id)?.base();
        Some((region.region.width, region.region.height))
    }

    fn dash_shape_view_at(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeView> {
        let (area_width, area_height) = self.area_dimensions();
        let owner = self.find_region(region_id)?;
        let resolver = RegionShapeResolver { game: self, owner };
        owner.base().get_shape(x, y, area_width, area_height, &resolver).ok().flatten()
    }

    fn dash_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView> {
        let Some(owner) = self.find_region(region_id) else { return Vec::new(); };
        let resolver = RegionShapeResolver { game: self, owner };
        let (area_width, area_height) = self.area_dimensions();
        let mut views = Vec::new();
        if owner.base().get_shapes(x, y, area_width, area_height, &resolver, &mut views).is_err() { return Vec::new(); }
        views
    }

    fn dash_monster_level(&self, region_id: i32, monster_id: i32) -> Option<u8> {
        let monster = self.find_region(region_id)?.base().find_monster_by_id(monster_id)?;
        Some(self.find_monster_property_by_origin_name(monster.base_property_key()?)?.level as u8)
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount)
    }

    fn publish_player_states(&self, player_id: i32) {
        let _ = self.publish_player_states(player_id);
    }

    fn player_weapon_addon_category(&self, player: &CPlayer) -> Option<i32> {
        player.equipment().get_goods(2)
            .map(|weapon| weapon.addon_property_value(self.goods_factory(), GAP_WEAPON_CATEGORY, 1))
    }

    fn set_player_tile_position(&mut self, player_id: i32, tile_x: i32, tile_y: i32) {
        let _ = self.set_player_tile_position(player_id, tile_x, tile_y);
    }

    fn relocate_region_shape(&mut self, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32) {
        let _ = self.relocate_region_shape(region_id, identity, tile_x, tile_y);
    }

    fn live_skill_target_attackable(
        &self,
        region_id: i32,
        user: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        self.live_skill_target_attackable(region_id, user, target)
    }

    fn move_shape_level(&self, region_id: i32, target: ShapeIdentity) -> Option<u8> {
        self.move_shape_level(region_id, target)
    }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }

    fn skill_target_controller(&self, region_id: i32, target: ShapeIdentity) -> Option<i32> {
        self.skill_target_controller(region_id, target)
    }

    fn move_shape_state_position(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        state_id: u32,
    ) -> Option<usize> {
        resolve_state_move_shape(self, region_id, identity)?
            .find_state_position(|state| state.state_id() == state_id)
            .map(|(position, _)| position)
    }

    fn end_move_shape_state_at(&mut self, region_id: i32, identity: ShapeIdentity, index: usize) {
        let _ = end_and_destroy_state_at(self, region_id, identity, index);
    }

    fn send_dash_visual_to_player(&self, player_id: i32, message: &CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_dash_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage) {
        // Гейт существующего региона прежнего caller-а сохранён.
        if let Some(owner) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(owner.base(), origin, None, message);
        }
    }

    fn begin_rush_state(
        &mut self,
        target: (i32, ShapeIdentity),
        user: Option<(i32, ShapeIdentity)>,
        state: RushState,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<StateKey> {
        begin_primary_blind_state(self, target.0, target.1, user, Some(target), state, now)
    }

    fn begin_rush_2_state(
        &mut self,
        target: (i32, ShapeIdentity),
        user: Option<(i32, ShapeIdentity)>,
        state: Rush2State,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<StateKey> {
        begin_primary_blind_state(self, target.0, target.1, user, Some(target), state, now)
    }

    fn force_move_skill_target(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        tile_x: i32,
        tile_y: i32,
        duration_ms: u32,
    ) {
        // Развёрнутый результат у всех caller-ов семьи всегда отбрасывался.
        let _ = self.force_move_skill_target(region_id, target, tile_x, tile_y, duration_ms);
    }
}

impl<Runtime: GameMainLoopRuntime> DashSkillContact<Runtime> for CGame {
    fn apply_dash_weapon_attack(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        damage_factor_usage: u32,
        runtime: &mut Runtime,
    ) {
        apply_player_weapon_attack(self, address, source, target, damage_factor_usage, runtime);
    }

    fn rush_first_attack_at_position(
        &mut self,
        player_id: i32,
        controller: i32,
        region_id: Option<i32>,
        position: (i32, i32),
        runtime: &mut Runtime,
    ) {
        let _ = self.player_on_first_attack_at_position(player_id, controller, region_id, position, runtime);
    }

    fn apply_owned_skill_contact(
        &mut self,
        master: MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        self.apply_owned_skill_contact(master, target, region_id, attack, runtime);
    }

    fn dash_random_pos_in_range(
        &mut self,
        region_id: i32,
        left: i32,
        top: i32,
        range_width: i32,
        range_height: i32,
        runtime: &mut Runtime,
    ) -> Option<(i32, i32)> {
        let position = self.find_region(region_id)?.base().region
            .get_random_pos_in_range(left, top, range_width, range_height, runtime).ok()?;
        Some((position.x, position.y))
    }
}

/// Обёртка очереди прежнего планировщика: общий терминал семейства без
/// первого контакта; стадии приходят из Zone `dash::DashSkillExecutionOutcome`.
pub(super) fn dash_skill_outcome(outcome: DashSkillExecutionOutcome) -> QueuedSkillExecutionOutcome {
    let state = match outcome {
        DashSkillExecutionOutcome::Pending => QueuedSkillExecutionState::Pending,
        DashSkillExecutionOutcome::Rejected => QueuedSkillExecutionState::Rejected,
        DashSkillExecutionOutcome::Completed => QueuedSkillExecutionState::Completed,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}
