//! Тонкий путь к координатору семейства навыков боевого духа в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! семейство appserver/skills и базовый appserver/states/skill.cpp.
//! Зарегистрированный вход, общий End-контракт, visual-таблица 19 тел
//! и read-проекции BF-summon перенесены буквально в
//! `nebokrai_zone::skills::battlefairyskill` порцией №6b (основания и статусы
//! MATCH/INFERRED/UNKNOWN — в шапке Zone-файла). Здесь — объявленные швы
//! переноса: фасадные реализации трейтов Zone над прежними методами
//! `CGame`/`CPlayer`/`CMoveShape` и делегации с прежними сигнатурами;
//! потребители (tianhuo, thunder, poisonarrow и очередь Game) не меняются.

use super::kernel::{BattleFairyExecution, SkillTermination};
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::monster::{CMonster, MonsterSkillExecution};
use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapeSkill};
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::{CShape, ShapeFigure, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, GameMessageDomainOps};
use nebokrai_shared::values::CGuid;
use nebokrai_zone::combat::PlayerCombatProperties;
use nebokrai_zone::content::CSkillBaseProperties;
use nebokrai_zone::effects::{BattleFairyAttributeState, LifeShieldState};
use nebokrai_zone::skills::skillfactory::SkillEndEffect;
use nebokrai_zone::skills::state::{StateData, StateKey};
use nebokrai_zone::skills::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPkPermissions, BattleFairyPlayer,
    BattleFairySkillOutcome, SkillLifecycle,
};

/// Итог BF-исполнения Zone в прежнюю обёртку очереди (`state_skill_outcome`
/// не меняет `first_contact`, как и прежний `state_skill_outcome`/`terminal`).
pub(crate) fn battle_fairy_outcome(outcome: BattleFairySkillOutcome) -> QueuedSkillExecutionOutcome {
    state_skill_outcome(match outcome {
        BattleFairySkillOutcome::Begun => QueuedSkillExecutionState::Begun,
        BattleFairySkillOutcome::Pending => QueuedSkillExecutionState::Pending,
        BattleFairySkillOutcome::Completed => QueuedSkillExecutionState::Completed,
        BattleFairySkillOutcome::Rejected => QueuedSkillExecutionState::Rejected,
        BattleFairySkillOutcome::RejectedAfterUse => QueuedSkillExecutionState::RejectedAfterUse,
    })
}

impl BattleFairyPlayer for CPlayer {
    fn player_id(&self) -> i32 { self.player_id() }
    fn shape(&self) -> &CShape { self.shape() }
    fn figure(&self) -> ShapeFigure { self.figure() }
    fn health(&self) -> u32 { self.health() }
    fn set_health(&mut self, health: u32) { self.set_health(health); }
    fn mana(&self) -> u32 { self.mana() }
    fn set_mana(&mut self, mana: u32) { self.set_mana(mana); }
    fn war_soul_point(&self) -> (i32, i32) {
        let point = self.war_soul_point();
        (point.x, point.y)
    }
    fn team_id(&self) -> i32 { self.team_id() }
    fn faction_id(&self) -> i32 { self.faction_id() }
    fn union_id(&self) -> i32 { self.union_id() }
    fn country(&self) -> u8 { self.country() }
    fn combat_properties(&self) -> PlayerCombatProperties { self.combat_properties() }
    fn occupation(&self) -> u8 { self.occupation() }
    fn level(&self) -> u8 { self.level() }
    fn battle_fairy_pk_permissions(&self) -> BattleFairyPkPermissions {
        let permissions = self.pk_permissions();
        BattleFairyPkPermissions {
            player: permissions.player,
            teammate: permissions.teammate,
            guild_member: permissions.guild_member,
            criminal: permissions.criminal,
        }
    }
    fn selected_battle_fairy_skill_id(&self) -> u32 {
        self.player_ai().selected_battle_fairy_skill_id()
    }
}

impl BattleFairyMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }
    fn find_state_position(
        &self,
        matches: impl FnMut(&StateData) -> bool,
    ) -> Option<(usize, StateKey)> {
        self.find_state_position(matches)
    }
    fn state_at(&self, position: usize) -> Option<(StateKey, &StateData)> {
        self.state_at(position)
    }
}

impl BattleFairyGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type Player = CPlayer;
    type MoveShape = CMoveShape;

    fn registered_skill(&self, address: RegisteredSkill) -> Option<&MoveShapeSkill> {
        self.registered_skill(address)
    }
    fn registered_skill_mut(&mut self, address: RegisteredSkill) -> Option<&mut MoveShapeSkill> {
        self.registered_skill_mut(address)
    }
    fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<RegisteredSkill> {
        self.registered_player_skill(player_id, skill_id)
    }

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }
    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> {
        self.find_player_mut(player_id)
    }

    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&CMoveShape> {
        resolve_state_move_shape(self, region_id, identity)
    }
    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, lifecycle)
    }
    fn battle_fairy_shape_figure(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<ShapeFigure> {
        match identity.object_type {
            400 => Some(self.find_player(identity.id)?.figure()),
            500 => Some(ShapeFigure::default()),
            600 => {
                let monster = self.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
                let properties = self.find_monster_property_by_origin_name(monster.base_property_key()?)?;
                Some(CMonster::figure(properties))
            }
            1100 | 1200 => Some(self.find_region(region_id)?.stationary_build(identity)?.shape_view().figure),
            _ => None,
        }
    }
    fn battle_fairy_region_exists(&self, region_id: i32) -> bool {
        self.find_region(region_id).is_some()
    }
    fn set_player_tile_position(&mut self, player_id: i32, tile_x: i32, tile_y: i32) {
        let _ = self.set_player_tile_position(player_id, tile_x, tile_y);
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }
    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }
    fn base_combat_scales(&self) -> [f32; 5] { self.globe_setup().base_combat_scales() }

    fn base_magic_target_dead(&self, region_id: i32, target: ShapeIdentity) -> bool {
        self.base_magic_target_dead(region_id, target)
    }
    fn base_magic_target_name(&self, region_id: i32, target: ShapeIdentity) -> Option<&[u8]> {
        self.base_magic_target_name(region_id, target)
    }
    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }
    fn skill_target_path_with_length(
        &self,
        lifecycle: &SkillLifecycle,
        length: u32,
    ) -> Vec<(i32, i32, u8)> {
        self.skill_target_path_with_length(lifecycle, length)
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        self.update_registered_skill_visual(address, mode);
    }
    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text);
    }
    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount);
    }
    fn send_skill_system_info_with_text(&self, player_id: i32, text: &[u8], name: &[u8]) {
        self.send_skill_system_info_with_text(player_id, text, name);
    }
    fn send_battle_fairy_notice(&self, player_id: i32, first_color: u32, second_color: u32, text: &[u8]) {
        let _ = colored_player_notice_message(first_color, second_color, text)
            .send_to_player(self.net_server(), player_id);
    }
    fn send_battle_fairy_message_to_player(&self, player_id: i32, message: &CMessage) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }
    fn send_battle_fairy_visual_around(&self, region_id: i32, origin: &CShape, message: &CMessage) {
        if let Some(region) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(region.base(), origin, None, message);
        }
    }
    fn battle_fairy_string(&self, id: &[u8]) -> &[u8] { self.get_string_by_id(id) }
    fn publish_player_states(&self, player_id: i32) -> bool {
        self.publish_player_states(player_id).is_some()
    }
    fn update_move_shape_properties(&mut self, region_id: i32, holder: ShapeIdentity) -> bool {
        self.update_move_shape_properties(region_id, holder).is_some()
    }

    fn battle_fairy_equipment_present(&self, player_id: i32) -> bool {
        self.find_player(player_id)
            .is_some_and(|player| player.equipment().get_goods(10).is_some())
    }
    fn battle_fairy_equipment_addon(&self, player_id: i32, property: i32) -> Option<i32> {
        Some(self.find_player(player_id)?.equipment().get_goods(10)?
            .addon_property_value(self.goods_factory(), property, 1))
    }
    fn battle_fairy_equipment_identity_addon(
        &self,
        player_id: i32,
        property: i32,
    ) -> Option<(CGuid, i32)> {
        let goods = self.find_player(player_id)?.equipment().get_goods(10)?;
        Some((goods.identity().ex_id, goods.addon_property_value(self.goods_factory(), property, 1)))
    }
    fn battle_fairy_war_soul_addon(&self, player_id: i32, property: i32) -> Option<i32> {
        Some(self.find_player(player_id)?.war_soul_goods(self.goods_factory())?
            .addon_property_value(self.goods_factory(), property, 1))
    }
    fn battle_fairy_war_soul_goods_present(&self, player_id: i32) -> bool {
        self.find_player(player_id)
            .is_some_and(|player| player.war_soul_goods(self.goods_factory()).is_some())
    }
    fn set_battle_fairy_equipment_addon(
        &mut self,
        player_id: i32,
        property: i32,
        value: i32,
    ) -> Option<()> {
        self.set_player_equipment_addon_property(player_id, 10, property, 1, value).map(|_| ())
    }
    fn battle_fairy_equipment_payload(&self, player_id: i32) -> Option<(CGuid, Vec<u8>)> {
        let goods = self.find_player(player_id)?.equipment().get_goods(10)?;
        let mut old_client_payload = Vec::new();
        let _ = goods.serialize_for_old_client(
            &mut old_client_payload, self.goods_factory(), self.globe_setup().da_kong_key(),
        );
        Some((goods.identity().ex_id, old_client_payload))
    }
    fn battle_fairy_war_soul_payload(&self, player_id: i32) -> Option<(CGuid, Vec<u8>)> {
        let goods = self.find_player(player_id)?.war_soul_goods(self.goods_factory())?;
        let mut old_client_payload = Vec::new();
        let _ = goods.serialize_for_old_client(
            &mut old_client_payload, self.goods_factory(), self.globe_setup().da_kong_key(),
        );
        Some((goods.identity().ex_id, old_client_payload))
    }
    fn spend_battle_fairy_mana(&mut self, player_id: i32, amount: u32) -> Option<(CGuid, Vec<u8>)> {
        let (update, _encoded) = self.spend_war_soul_mana_record(player_id, amount)?;
        Some((update.goods.ex_id, update.old_client_payload))
    }

    fn battle_fairy_end_first_state(&mut self, region_id: i32, holder: ShapeIdentity, state_id: u32) -> bool {
        let Some((position, _)) = resolve_state_move_shape(self, region_id, holder)
            .and_then(|shape| shape.find_state_position(|state| state.state_id() == state_id))
        else { return false; };
        end_and_destroy_state_at(self, region_id, holder, position).is_some()
    }
    fn begin_battle_fairy_attribute_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        sufferer: ShapeIdentity,
        state: BattleFairyAttributeState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        super::battlefairyattributestate::begin_battle_fairy_attribute_state(
            self, region_id, holder, sufferer, state, now,
        )
    }
    fn begin_life_shield_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        user: Option<(i32, ShapeIdentity)>,
        sufferer: Option<(i32, ShapeIdentity)>,
        state: LifeShieldState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        super::shieldstate::begin_primary_self_shield_state(
            self, region_id, holder, user, sufferer,
            super::shieldstate::DefenseShieldState::Life(state), now,
        ).is_some()
    }

    fn prepare_registered_skill_end_effect(
        &mut self,
        address: RegisteredSkill,
        effect: SkillEndEffect,
        argument: i32,
    ) -> bool {
        self.prepare_registered_skill_end_effect(address, effect, argument).is_some()
    }
    fn end_registered_battle_fairy(
        &mut self,
        address: RegisteredSkill,
        argument: i32,
        termination: SkillTermination,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        self.end_registered_instance_with_clock(address, argument, termination, now).is_some()
    }
    fn end_registered_battle_fairy_without_after_use(
        &mut self,
        address: RegisteredSkill,
        termination: SkillTermination,
    ) -> bool {
        self.end_registered_instance_without_after_use(address, termination).is_some()
    }

    fn allocate_summon_shape_id(&mut self) -> i32 { self.allocate_summon_shape_id() }
}

pub(crate) fn publish_battle_fairy_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::publish_battle_fairy_visual(game, skill, mode);
}

/// Делегация прежней сигнатуры для multiclass owner-ов (thunder/tianhuo и
/// далее): rejected/installed отображаются прежним `state_skill_outcome`.
pub(crate) fn check_battle_fairy_target_states(
    game: &CGame, player_id: i32, target: (i32, ShapeIdentity),
) -> bool {
    nebokrai_zone::skills::check_battle_fairy_target_states(game, player_id, target)
}

/// Делегация прежней сигнатуры: runtime остаётся прежним main-loop швом.
pub(crate) fn execute_registered_battle_fairy_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    begin_failure_visual: Option<u32>,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    nebokrai_zone::skills::execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, begin_failure_visual,
        || state_skill_outcome(QueuedSkillExecutionState::Rejected),
        || state_skill_outcome(QueuedSkillExecutionState::Begun),
        check, run_ai,
    )
}

/// Делегация прежней сигнатуры с materialize веткой concrete owner-а.
pub(crate) fn execute_registered_battle_fairy_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    begin_failure_visual: Option<u32>,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(BattleFairySkillDispatch, u32) -> BattleFairyExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    nebokrai_zone::skills::execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, begin_failure_visual,
        || state_skill_outcome(QueuedSkillExecutionState::Rejected),
        || state_skill_outcome(QueuedSkillExecutionState::Begun),
        check, materialize, run_ai,
    )
}

impl CGame {
    /// SetWarSoulXY (0x0042DF50) и DelWarSoul (0x0042E0A0) — делегация в Zone.
    pub(crate) fn cancel_active_battle_fairy_skill(
        &mut self,
        player_id: i32,
    ) -> bool {
        nebokrai_zone::skills::cancel_active_battle_fairy_skill(self, player_id)
    }

    /// Терминальная граница после concrete owner и contacts — делегация в Zone.
    pub(crate) fn finish_registered_battle_fairy_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        instance: RegisteredSkill,
        dispatch: BattleFairySkillDispatch,
        argument: i32,
        termination: SkillTermination,
        begin_attempted: bool,
        runtime: &mut Runtime,
    ) -> bool {
        nebokrai_zone::skills::finish_registered_battle_fairy_skill(
            self, instance, dispatch, argument, termination, begin_attempted,
            &mut || runtime.now_milliseconds(),
        )
    }

    pub(crate) fn send_battle_fairy_skill_failure(&self, player_id: i32, action: u8) {
        nebokrai_zone::skills::send_battle_fairy_skill_failure(self, player_id, action);
    }
}
