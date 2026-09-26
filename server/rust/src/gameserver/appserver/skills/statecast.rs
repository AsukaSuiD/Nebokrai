//! Hub-реализации швов Zone state-кастов пятёрки (CCure, CHearten,
//! CPromotion, CGodBless/CGodBless2) и heal-квартета (CHeal/CHeal2/
//! CSuperHeal/CSuperHeal2) над прежними методами `CGame`/`CPlayer`/
//! `CMoveShape`, плюс общий outcome-шов делегатов. Источник:
//! gameserver.exe + GameServer.pdb; тела Check/AI и живые callbacks
//! состояний перенесены буквально в Zone `skills/{statecast,cure,
//! curestate,hearten,heartenstate,promotion,promotionstate,godbless,
//! godblessstate,heal,healstate}.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; основание и машинные статусы см. там и в записи аудита
//! «Zone skills: машинная разведка battlefairy-навыков (порция №6)»).
//! Потребители не меняются.
//!
//! Check-скелет `rangedweaponcast` (distant-ветка Ignore и MP-контракт) и
//! обвязка арены `states/state.rs` остаются в этом пакете: декларированные
//! швы делегируют им вызовы в прежних точках. Обход Ex-состояний первичной
//! установки GodBless принадлежит `CGame::install_god_bless_state`
//! (`game/godbless.rs`, не переносился этой порцией).

use super::rangedweaponcast::{CastPathBlock, check_cast_mana, check_skill_path, spend_cast_mana};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{end_state_skill, state_skill_outcome};
use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, end_and_destroy_state_at, end_move_shape_state,
    remove_applied_state_from, resolve_applied_state_sufferer, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut, update_applied_state_end_visual,
    update_player_state_properties, update_property_state_visual,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::{CMessage, GameMessageDomainOps};
use nebokrai_zone::combat::PlayerCombatProperties;
use nebokrai_zone::effects::CureState;
use nebokrai_zone::skills::SkillLifecycle;
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::state::{AppliedState, StateData, StateKey};
use nebokrai_zone::skills::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastMoveShape, StateCastPlayer,
    StateCastPropertyTarget,
};

impl StateCastPlayer for CPlayer {
    fn mana(&self) -> u32 { self.mana() }

    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }

    fn combat_properties(&self) -> PlayerCombatProperties { self.combat_properties() }

    fn update_state_combat_properties(
        &mut self,
        update: impl FnOnce(PlayerCombatProperties) -> PlayerCombatProperties,
    ) {
        self.update_state_combat_properties(update);
    }
}

impl StateCastMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }

    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }

    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }

    fn set_fightable(&mut self, fightable: bool) { self.set_fightable(fightable) }

    fn state_slot_count(&self) -> usize { self.state_slot_count() }

    fn state_at(&self, position: usize) -> Option<(StateKey, &StateData)> {
        self.state_at(position)
    }

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

    fn defense_shield_key(&self, skill_id: u32) -> Option<StateKey> {
        self.defense_shield_key(skill_id)
    }

    fn cure_state_key(&self) -> Option<StateKey> { self.cure_state_key() }

    fn cure_state_by_key(&self, key: StateKey) -> Option<CureState> {
        self.cure_state_by_key(key)
    }
}

/// Соответствие прежнего перечня адресатов property-visual общего обхода.
fn state_property_target(target: StateCastPropertyTarget) -> StatePropertyTarget {
    match target {
        StateCastPropertyTarget::User => StatePropertyTarget::User,
        StateCastPropertyTarget::Sufferer => StatePropertyTarget::Sufferer,
    }
}

impl StateCastGame for CGame {
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
        self.update_registered_skill_visual(address, mode);
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

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.find_player_mut(player_id)
    }

    fn player_cure_state_key(&self, player_id: i32) -> Option<StateKey> {
        self.find_player(player_id)?.move_shape().cure_state_key()
    }

    fn player_shape_participant(&self, player_id: i32) -> Option<(i32, ShapeIdentity)> {
        let player = self.find_player(player_id)?;
        Some((player.shape().get_region_id(), player.shape().identity()))
    }

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
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

    fn resolve_applied_state_sufferer(
        &self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> Option<(i32, ShapeIdentity)> {
        resolve_applied_state_sufferer(self, region_id, holder, key)
    }

    fn update_player_state_properties<S: AppliedState + Copy>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        update: impl FnOnce(S, &mut CPlayer),
    ) -> bool {
        update_player_state_properties(self, region_id, holder, key, |state, player| update(state, player))
    }

    fn update_property_state_visual<S: AppliedState>(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: StateCastPropertyTarget,
        now: &mut dyn FnMut() -> u32,
        client_time: impl FnOnce(&S, &mut dyn FnMut() -> u32) -> u32,
    ) -> bool {
        update_property_state_visual(
            self, region_id, holder, key, state_property_target(target), now, client_time,
        )
    }

    fn update_applied_state_end_visual(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
        target: StateCastPropertyTarget,
    ) -> bool {
        update_applied_state_end_visual(self, region_id, holder, key, state_property_target(target))
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

    fn end_move_shape_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        key: StateKey,
    ) -> bool {
        end_move_shape_state(self, region_id, holder, key)
    }

    fn end_and_destroy_state_at(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        index: usize,
    ) -> bool {
        end_and_destroy_state_at(self, region_id, holder, index).is_some()
    }

    fn send_move_shape_around(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
        message: &CMessage,
    ) {
        let _ = self.send_move_shape_around(region_id, identity, message);
    }

    fn check_skill_path(
        &mut self,
        instance: RegisteredSkill,
        properties: &CSkillBaseProperties,
        path: &[(i32, i32, u8)],
        player: Option<i32>,
    ) -> bool {
        check_skill_path(self, instance, properties, path, player, CastPathBlock::Ignore)
    }

    fn check_cast_mana(
        &mut self,
        instance: RegisteredSkill,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool {
        check_cast_mana(self, instance, source, properties)
    }

    fn spend_cast_mana(
        &mut self,
        instance: RegisteredSkill,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool {
        spend_cast_mana(self, instance, player, properties)
    }

    fn monster_present(&self, region_id: i32, monster_id: i32) -> bool {
        self.find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(monster_id)).is_some()
    }

    fn wild_untamed_non_carriage_monster(&self, region_id: i32, monster_id: i32) -> bool {
        self.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(monster_id))
            .is_some_and(|monster| !monster.is_tamed() && !matches!(
                monster.active_ai(),
                Some(ActiveMonsterAi::Carriage | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)),
            ))
    }

    fn player_weapon_damage_level(&self, player: &CPlayer) -> u32 {
        player.weapon_damage_level(self.goods_factory()) as u32
    }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }

    fn base_magic_target_name(&self, region_id: i32, target: ShapeIdentity) -> Vec<u8> {
        self.base_magic_target_name(region_id, target).unwrap_or_default().to_vec()
    }

    fn move_shape_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32> {
        self.move_shape_health(region_id, target)
    }

    fn move_shape_maximum_health(&self, region_id: i32, target: ShapeIdentity) -> Option<u32> {
        self.move_shape_maximum_health(region_id, target)
    }

    fn set_move_shape_health(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        health: u32,
    ) -> bool {
        self.set_move_shape_health(region_id, target, health).is_some()
    }

    fn publish_move_shape_states(&mut self, region_id: i32, target: ShapeIdentity) {
        let _ = self.publish_move_shape_states(region_id, target);
    }

    fn update_move_shape_properties(&mut self, region_id: i32, target: ShapeIdentity) {
        let _ = self.update_move_shape_properties(region_id, target);
    }

    fn god_bless_monster_gains(
        &mut self,
        region_id: i32,
        monster_id: i32,
        gains: (i32, i32, i32),
    ) {
        // Живой `monster.move_shape_mut().property_modifiers_mut()` прежнего тела.
        let (minimum_gain, maximum_gain, element_gain) = gains;
        if let Some(monster) = self.find_region_mut(region_id)
            .and_then(|owner| owner.base_mut().find_monster_by_id_mut(monster_id))
        {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers.minimum_attack.wrapping_add(minimum_gain);
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(maximum_gain);
            modifiers.element_modify = modifiers.element_modify.wrapping_add(element_gain);
        }
    }

    fn skill_random_below(&mut self, maximum: i32) -> i32 {
        self.skill_random_below(maximum)
    }

    fn send_state_cast_visual_to_player(&self, player_id: i32, message: &CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_state_cast_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage) {
        // Гейт существующего региона прежнего caller-а сохранён.
        if let Some(owner) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(owner.base(), origin, None, message);
        }
    }
}

/// Терминал AI делегата пятёрки: Pending/Rejected возвращаются очереди без
/// End, а EndRejected/EndCompleted требуют прежний полный End того же ключа —
/// точка прежнего `end_state_skill(..., 0/1)` внутри перенесённого тела.
pub(super) fn finish_state_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    address: RegisteredSkill,
    outcome: StateCastExecutionOutcome,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    match outcome {
        StateCastExecutionOutcome::Pending => {
            state_skill_outcome(QueuedSkillExecutionState::Pending)
        }
        StateCastExecutionOutcome::Rejected => {
            state_skill_outcome(QueuedSkillExecutionState::Rejected)
        }
        StateCastExecutionOutcome::EndRejected => end_state_skill(game, address, 0, runtime),
        StateCastExecutionOutcome::EndCompleted => end_state_skill(game, address, 1, runtime),
    }
}

/// Сырой outcome для входа общего playercast: End обрабатывает его
/// `finish_outcome`, как делал прежний делегат терминального результата.
pub(super) fn state_cast_outcome(outcome: StateCastExecutionOutcome) -> QueuedSkillExecutionOutcome {
    let state = match outcome {
        StateCastExecutionOutcome::Pending => QueuedSkillExecutionState::Pending,
        StateCastExecutionOutcome::Rejected | StateCastExecutionOutcome::EndRejected => {
            QueuedSkillExecutionState::Rejected
        }
        StateCastExecutionOutcome::EndCompleted => QueuedSkillExecutionState::Completed,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}
