//! Hub-реализации швов Zone self/zone-кастов порции №6c (CBlind и общий
//! lifecycle 8-байт lock-состояний, CEnergyHolding, CRoar, CPillar,
//! CCallosity/CCallosity2, CSoulMirror) над прежними методами `CGame`/
//! `CPlayer`/`CMoveShape`/`CPlayerAI`, плюс общий outcome-шов делегатов.
//! Источник: gameserver.exe + GameServer.pdb; тела Check/AI, клеточного
//! обхода, visual и живые callbacks состояний перенесены буквально в Zone
//! `skills/{selfcast,blind,blindstate,energyholding,energyholdingstate,
//! roar,roarstate,pillar,pillarstate,callosity,callositystate,
//! soulmirror}.rs` (порция №6c «self/zone-касты»; основание и машинные
//! статусы см. там и в записи аудита «Zone skills: машинная разведка
//! battlefairy-навыков (порция №6)»). Потребители не меняются.
//!
//! Обвязка арены `states/state.rs`, machine накопления `accumulatedstate`
//! (владелец поделён с SoulCollect), прямой элементный контакт
//! `directelementattack`, мастер источника `weaponattack` и lifecycle
//! призванного существа (property → owner → `add_summoned_creature_owned` →
//! возврат owner) остаются в этом пакете: декларированные швы делегируют им
//! вызовы в прежних точках. Региональные `region_*` фасады перечитывают
//! owner-а региона на каждый вызов (первичный гейт find_region конкретных
//! zone-тел сохранён).

use super::accumulatedstate::{add_accumulated_state, update_accumulated_visual};
use super::directelementattack::apply_direct_element_attack;
use super::skillbaseproperties::CSkillBaseProperties;
use super::weaponattack::source_master;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    end_and_destroy_state_at, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
    update_applied_state_end_visual, update_applied_state_visual_base,
    update_property_state_visual,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState, RegionShapeResolver,
};
use crate::nets::netserver::message::{CMessage, GameMessageDomainOps};
use nebokrai_zone::combat::PlayerCombatProperties;
use nebokrai_zone::effects::{EnergyHoldingState, RoarState};
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::state::{AppliedState, StateData, StateKey};
use nebokrai_zone::skills::selfcast::{
    SelfCastContact, SelfCastExecutionOutcome, SelfCastGame, SelfCastMoveShape,
    SelfCastPlayer, SelfCastPropertyTarget,
};
use nebokrai_zone::skills::{
    PlayerSkillDispatch, SkillExecutionKernel, SkillLifecycle, SkillTermination, SkillVisualEffect,
};

impl SelfCastPlayer for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }

    fn movement_shape_mut(&mut self) -> &mut CShape { self.movement_shape_mut() }

    fn mana(&self) -> u32 { self.mana() }

    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }

    fn rp(&self) -> u16 { self.rp() }

    fn set_rp(&mut self, rp: u16) { self.set_rp(rp) }

    fn level(&self) -> u8 { self.level() }

    fn is_dead(&self) -> bool { self.is_dead() }

    fn set_skill_moveable(&mut self, moveable: bool) { self.set_skill_moveable(moveable) }

    fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    ) {
        self.update_state_combat_properties(update);
    }
}

fn selfcast_property_target(target: SelfCastPropertyTarget) -> StatePropertyTarget {
    match target {
        SelfCastPropertyTarget::User => StatePropertyTarget::User,
        SelfCastPropertyTarget::Sufferer => StatePropertyTarget::Sufferer,
    }
}

impl SelfCastMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }

    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }

    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }

    fn set_fightable(&mut self, fightable: bool) { self.set_fightable(fightable) }

    fn find_state_position(
        &self,
        matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)> {
        self.find_state_position(matches)
    }

    fn applied_state<T: AppliedState>(&self, key: StateKey) -> Option<&T> {
        self.applied_state(key)
    }

    fn applied_state_mut<T: AppliedState>(&mut self, key: StateKey) -> Option<&mut T> {
        self.applied_state_mut(key)
    }

    fn applied_state_data(&self, key: StateKey) -> Option<&StateData> {
        self.applied_state_data(key)
    }

    fn applied_state_replacement_location(&self, key: StateKey) -> Option<(usize, usize)> {
        self.applied_state_replacement_location(key)
    }

    fn append_applied_state_record<T: AppliedState>(&mut self, state: T, record: &[u8]) -> StateKey {
        self.append_applied_state_record(state, record)
    }

    fn insert_replacement_state_record<T: AppliedState>(
        &mut self,
        state: T,
        record: &[u8],
        location: (usize, usize),
    ) -> Option<StateKey> {
        self.insert_replacement_state_record(state, record, location)
    }

    fn begin_applied_state_visual(&mut self, key: StateKey, loop_value: i32) -> bool {
        self.begin_applied_state_visual(key, loop_value)
    }

    fn update_applied_state_visual_base(&mut self, key: StateKey) -> bool {
        self.update_applied_state_visual_base(key)
    }

    fn mark_applied_state_begun(&mut self, key: StateKey) -> bool {
        self.mark_applied_state_begun(key)
    }

    fn mark_applied_state_ended(&mut self, key: StateKey) -> bool {
        self.mark_applied_state_ended(key)
    }

    fn set_applied_state_user(
        &mut self,
        key: StateKey,
        user: Option<(i32, ShapeIdentity)>,
    ) -> bool {
        self.set_applied_state_user(key, user)
    }

    fn set_applied_state_sufferer(
        &mut self,
        key: StateKey,
        sufferer: Option<(i32, ShapeIdentity)>,
    ) -> bool {
        self.set_applied_state_sufferer(key, sufferer)
    }
}

impl SelfCastGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type Player = CPlayer;
    type MoveShape = CMoveShape;
    type PlayerAi = CPlayerAI;

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

    fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<RegisteredSkill> {
        self.registered_player_skill(player_id, skill_id)
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

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32 {
        self.player_skill_last_used_ms(player_id, skill_id)
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
    ) {
        let _ = self.replace_player_skill_visual_effect(player_id, skill_id, effect);
    }

    fn update_player_skill_visual(&mut self, player_id: i32, skill_id: u32, mode: u32) {
        self.update_player_skill_visual(player_id, skill_id, mode)
    }

    fn update_player_current_state_move_shape_ai(&mut self, player_id: i32) {
        let _ = self.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    }

    fn finish_player_skill(
        &mut self,
        player_id: i32,
        ai: &mut CPlayerAI,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        self.finish_player_skill(player_id, ai, dispatch, termination)
    }

    fn with_published_player_ai<R>(
        &mut self,
        player_id: i32,
        ai: &mut CPlayerAI,
        body: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.with_published_player_ai(player_id, ai, body)
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount)
    }

    fn send_skill_system_info_with_text(&self, player_id: i32, text: &[u8], name: &[u8]) {
        self.send_skill_system_info_with_text(player_id, text, name)
    }

    fn publish_player_states(&self, player_id: i32) {
        let _ = self.publish_player_states(player_id);
    }

    fn move_shape_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32> {
        self.move_shape_health(region_id, target)
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

    fn live_skill_target_attackable_between(
        &self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
    ) -> bool {
        self.live_skill_target_attackable_between(source, target)
    }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }

    fn base_magic_target_name(&self, region_id: i32, target: ShapeIdentity) -> Vec<u8> {
        self.base_magic_target_name(region_id, target).unwrap_or_default().to_vec()
    }

    fn skill_target_controller(&self, region_id: i32, target: ShapeIdentity) -> Option<i32> {
        self.skill_target_controller(region_id, target)
    }

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }

    fn area_shape_view_at(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeView> {
        let (area_width, area_height) = self.area_dimensions();
        let owner = self.find_region(region_id)?;
        let resolver = RegionShapeResolver { game: self, owner };
        owner.base().get_shape(x, y, area_width, area_height, &resolver).ok().flatten()
    }

    fn area_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView> {
        let Some(owner) = self.find_region(region_id) else { return Vec::new(); };
        let resolver = RegionShapeResolver { game: self, owner };
        let (area_width, area_height) = self.area_dimensions();
        let mut views = Vec::new();
        if owner.base().get_shapes(x, y, area_width, area_height, &resolver, &mut views).is_err() { return Vec::new(); }
        views
    }

    fn region_dimensions(&self, region_id: i32) -> Option<(i32, i32)> {
        let region = self.find_region(region_id)?.base();
        Some((region.region.width, region.region.height))
    }

    fn region_cell_safe(&self, region_id: i32, x: i32, y: i32) -> Option<bool> {
        let region = self.find_region(region_id)?;
        Some(region.get_security(x, y).ok() == Some(RegionSecurity::SAFE))
    }

    fn region_cell_walkable(&self, region_id: i32, x: i32, y: i32) -> bool {
        self.find_region(region_id).is_some_and(|owner| {
            let region = owner.base();
            x >= 0 && x < region.region.width && y >= 0 && y < region.region.height
                && region.skill_cell_block(x, y) & 7 == 0
        })
    }

    fn update_move_shape_properties(&mut self, region_id: i32, target: ShapeIdentity) {
        let _ = self.update_move_shape_properties(region_id, target);
    }

    fn send_move_shape_around(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
        message: &CMessage,
    ) {
        let _ = self.send_move_shape_around(region_id, identity, message);
    }

    fn send_cast_visual_to_player(&self, player_id: i32, message: &CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_cast_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage) {
        // Гейт существующего региона прежнего caller-а сохранён.
        if let Some(owner) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(owner.base(), origin, None, message);
        }
    }

    fn player_weapon_addon_category(&self, player: &CPlayer) -> Option<i32> {
        player.equipment().get_goods(2)
            .map(|weapon| weapon.addon_property_value(self.goods_factory(), GAP_WEAPON_CATEGORY, 1))
    }

    fn resolve_applied_state_sufferer(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_applied_state_sufferer(self, region_id, holder, key)
    }

    fn begin_base_applied_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> bool {
        begin_base_applied_state(self, region_id, holder, key)
    }

    fn begin_applied_state_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        loop_value: i32,
    ) -> bool {
        begin_applied_state_visual(self, region_id, holder, key, loop_value)
    }

    fn update_applied_state_visual_base(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> bool {
        update_applied_state_visual_base(self, region_id, holder, key)
    }

    fn update_applied_state_end_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: SelfCastPropertyTarget,
    ) -> bool {
        update_applied_state_end_visual(self, region_id, holder, key, selfcast_property_target(target))
    }

    fn update_property_state_visual<S: AppliedState>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: SelfCastPropertyTarget,
        now: &mut dyn FnMut() -> u32,
        client_time: impl FnOnce(&S, &mut dyn FnMut() -> u32) -> u32,
    ) -> bool {
        update_property_state_visual(
            self, region_id, holder, key, selfcast_property_target(target), now, client_time,
        )
    }

    fn remove_applied_state_from(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: (i32, ShapeIdentity),
        bytes: usize,
    ) -> bool {
        remove_applied_state_from(self, region_id, holder, key, target, bytes)
    }

    fn end_and_destroy_state_at(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        index: usize,
    ) -> bool {
        end_and_destroy_state_at(self, region_id, holder, index).is_some()
    }

    fn add_energy_holding_state(
        &mut self,
        source: (i32, ShapeIdentity),
        create: impl FnOnce(&Self) -> Option<EnergyHoldingState>,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        add_accumulated_state(self, source, create, now)
    }

    fn update_energy_holding_accumulated_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        mode: u32,
    ) {
        update_accumulated_visual::<EnergyHoldingState>(self, (region_id, holder), key, mode);
    }

    fn roar_monster_apply_losses(&mut self, region_id: i32, monster_id: i32, state: RoarState) {
        let losses = self.find_region(region_id)
            .and_then(|region| region.base().find_monster_by_id(monster_id))
            .and_then(|monster| {
                let property = self.find_monster_property_by_origin_name(monster.original_name())?;
                let (minimum, maximum) = monster.state_attack_bounds(
                    property.minimum_attack, property.maximum_attack,
                );
                Some(state.monster_losses(minimum, maximum, monster.element_modifier()))
            });
        if let Some((minimum_loss, maximum_loss, element_loss)) = losses {
            if let Some(monster) = self.find_region_mut(region_id)
                .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id)) {
                let modifiers = monster.move_shape_mut().property_modifiers_mut();
                modifiers.minimum_attack = modifiers.minimum_attack.wrapping_sub(minimum_loss);
                modifiers.maximum_attack = modifiers.maximum_attack.wrapping_sub(maximum_loss);
                modifiers.element_modify = modifiers.element_modify.wrapping_sub(element_loss);
            }
        }
    }

    fn soul_mirror_source_master(&self, source: (i32, ShapeIdentity)) -> Option<MasterInfo> {
        source_master(self, source)
    }

    fn add_soul_mirror_summoned_creature(
        &mut self,
        region_id: i32,
        picture_id: u32,
        master: MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    ) {
        let Some(property) = self.find_monster_property_by_picture_id(picture_id).cloned() else {
            return;
        };
        let Some(mut owner) = self.take_region_owner(region_id) else { return; };
        let _ = self.add_summoned_creature_owned(
            owner.base_mut(), &property, master, tile_x, tile_y, direction, lifetime_ms,
        );
        self.restore_region_owner(owner);
    }
}

impl<Runtime: GameMainLoopRuntime> SelfCastContact<Runtime> for CGame {
    fn skill_first_attack_at_position(
        &mut self,
        attacker_id: i32,
        victim_id: i32,
        region_id: i32,
        position: (i32, i32),
        runtime: &mut Runtime,
    ) {
        let _ = self.player_on_first_skill_at_position(
            attacker_id, victim_id, region_id, position.0, position.1, runtime,
        );
    }

    fn apply_direct_element_contact(
        &mut self,
        instance: RegisteredSkill,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) {
        apply_direct_element_attack(self, instance, source, target, runtime);
    }
}

/// Обёртка очереди прежнего планировщика: терминал семьи без первого
/// контакта; варианты приходят из Zone `selfcast::SelfCastExecutionOutcome`.
pub(super) fn selfcast_outcome(outcome: SelfCastExecutionOutcome) -> QueuedSkillExecutionOutcome {
    let state = match outcome {
        SelfCastExecutionOutcome::Pending => QueuedSkillExecutionState::Pending,
        SelfCastExecutionOutcome::Rejected => QueuedSkillExecutionState::Rejected,
        SelfCastExecutionOutcome::RejectedAfterUse => QueuedSkillExecutionState::RejectedAfterUse,
        SelfCastExecutionOutcome::Begun => QueuedSkillExecutionState::Begun,
        SelfCastExecutionOutcome::Completed => QueuedSkillExecutionState::Completed,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}
