//! Тонкий путь к направленному громовому удару `CThunderBlow2` (`0x14D`) в
//! Zone. Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderblow2.cpp.
//! Тела Check/AI (отдельная S, reuse, длина пути, цена MP; списание MP,
//! OnChangeStates, CAN, направление, visual0; отбрасывание и raw-контакт по
//! impactattack-швам) перенесены буквально в `nebokrai_zone::skills::thunderblow2`
//! (основание и статусы см. там). Здесь — зарегистрированный вход общего
//! playercast с прежней сигнатурой, объявленные швы переноса (фасадные
//! реализации трейтов Zone над прежними методами `CGame`/`CPlayer`/`CMoveShape`)
//! и impactattack-швы формулы/отбрасывания; внешние потребители не меняются.

use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use crate::nets::netserver::message::GameMessageDomainOps;
use nebokrai_zone::skills::SkillLifecycle;
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::thunderblow2::{
    ThunderBlow2Contact, ThunderBlow2Game, ThunderBlow2MoveShape, ThunderBlow2Outcome,
    ThunderBlow2Player,
};
use nebokrai_zone::skills::thunderblow2 as zone;

pub(crate) use nebokrai_zone::skills::execution::ThunderBlow2Execution;
pub(crate) use nebokrai_zone::skills::thunderblow2::THUNDER_BLOW_2_SKILL_ID;

impl ThunderBlow2Player for CPlayer {
    fn shape(&self) -> &CShape { self.shape() }
    fn mana(&self) -> u32 { self.mana() }
    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }
}

impl ThunderBlow2MoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }
    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }
    fn has_state_by_skill_id(&self, skill_id: u32) -> bool { self.has_state_by_skill_id(skill_id) }
}

impl ThunderBlow2Game for CGame {
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

    fn move_shape_health(&self, region_id: i32, holder: ShapeIdentity) -> Option<u32> {
        self.move_shape_health(region_id, holder)
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

    fn thunder_blow_2_region_present(&self, region_id: i32) -> bool {
        self.find_region(region_id).is_some()
    }

    fn publish_player_states(&self, player_id: i32) {
        let _ = self.publish_player_states(player_id);
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount)
    }

    fn send_thunder_blow_2_visual_to_player(
        &self,
        player_id: i32,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_thunder_blow_2_visual_around(
        &self,
        region_id: i32,
        origin: &CShape,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        // Гейт существующего региона прежнего caller-а сохранён.
        if let Some(owner) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(owner.base(), origin, None, message);
        }
    }

    fn knock_back_impact_target(
        &mut self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        region_id: i32,
        properties: &CSkillBaseProperties,
    ) {
        super::impactattack::knock_back_impact_target(self, source, target, region_id, properties);
    }
}

impl<Runtime: GameMainLoopRuntime> ThunderBlow2Contact<Runtime> for CGame {
    fn apply_thunder_blow_2_attack(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) {
        super::impactattack::apply_thunder_blow_2_attack(self, address, source, target, runtime);
    }
}

fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let _ = runtime;
    zone::check_cast(game, instance, player_id, target, game_tick_milliseconds)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let state = match zone::run_ai(game, instance, runtime, game_tick_milliseconds) {
        ThunderBlow2Outcome::Pending => QueuedSkillExecutionState::Pending,
        ThunderBlow2Outcome::Rejected => QueuedSkillExecutionState::Rejected,
        ThunderBlow2Outcome::Completed => QueuedSkillExecutionState::Completed,
    };
    outcome(state)
}

pub(crate) fn execute_player_thunder_blow_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != THUNDER_BLOW_2_SKILL_ID { return outcome(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ThunderBlow2,
        |game, instance, player_id, runtime| {
            let target = game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
            check_cast(game, instance, player_id, target, runtime)
        },
        |dispatch, started| ThunderBlow2Execution::begin(dispatch, started).into(), run_ai,
    )
}
