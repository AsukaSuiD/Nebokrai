//! Тонкий путь к громовому удару `CThunderBlow` (`0x13F`) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderblow.cpp.
//! Тела pipeline Begin/Check/AI/Summon, wire-visual и данные области
//! перенесены буквально в `nebokrai_zone::skills::thunderblow` (основание и
//! статусы MATCH/UNKNOWN см. там); FIX порции T3 — порядок AI Begin приведён
//! к машинному якорю `0x17A520` (списание MP, state-update +0x164 и поворот
//! до повторной проверки дальности; её отказ теряет уже списанные MP и
//! поворот). Здесь — делегации с прежними сигнатурами и объявленные швы
//! переноса: фасадные реализации трейтов Zone над прежними методами
//! `CGame`/`CPlayer`; внешние потребители не меняются.

use super::baseattack::finish_immediate_base_attack;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState, game_tick_milliseconds,
};
use nebokrai_zone::skills::thunderblow::{
    ThunderBlowContact, ThunderBlowGame, ThunderBlowOutcome, ThunderBlowPlayer,
    ThunderBlowPkPermissions,
};
use nebokrai_zone::skills::thunderblow as zone;

pub(crate) use nebokrai_zone::skills::thunderblow::THUNDER_BLOW_SKILL_ID;

impl ThunderBlowPlayer for CPlayer {
    fn shape(&self) -> &crate::gameserver::appserver::shape::CShape { self.shape() }
    fn movement_shape_mut(&mut self) -> &mut crate::gameserver::appserver::shape::CShape { self.movement_shape_mut() }
    fn mana(&self) -> u32 { self.mana() }
    fn set_mana(&mut self, mana: u32) { self.set_mana(mana) }
    fn player_id(&self) -> i32 { self.player_id() }
    fn faction_id(&self) -> i32 { self.faction_id() }
    fn team_id(&self) -> i32 { self.team_id() }
    fn union_id(&self) -> i32 { self.union_id() }
    fn server_region_id(&self) -> Option<i32> { self.server_region_id() }
    fn is_dead(&self) -> bool { self.is_dead() }
    fn set_current_skill_id(&mut self, skill_id: Option<u32>) { self.set_current_skill_id(skill_id) }
    fn combat_properties(&self) -> nebokrai_zone::combat::PlayerCombatProperties { self.combat_properties() }
    fn occupation(&self) -> u8 { self.occupation() }
    fn level(&self) -> u8 { self.level() }
    fn pk_permissions(&self) -> ThunderBlowPkPermissions {
        let permissions = self.pk_permissions();
        // MasterInfo этого владельца хранит четвёрку допусков и нулевую страну.
        ThunderBlowPkPermissions {
            player: permissions.player,
            teammate: permissions.teammate,
            guild_member: permissions.guild_member,
            criminal: permissions.criminal,
        }
    }
}

impl ThunderBlowGame for CGame {
    type Player = CPlayer;
    type PlayerAi = CPlayerAI;

    fn player_skill_execution(
        &self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<nebokrai_zone::skills::SkillExecutionKernel<PlayerSkillDispatch>> {
        self.player_skill_execution(player_id, skill_id)
    }

    fn player_skill_execution_mut(
        &mut self,
        player_id: i32,
        skill_id: u32,
    ) -> Option<&mut nebokrai_zone::skills::SkillExecutionKernel<PlayerSkillDispatch>> {
        self.player_skill_execution_mut(player_id, skill_id)
    }

    fn begin_player_skill_execution(
        &mut self,
        player_id: i32,
        kernel: nebokrai_zone::skills::SkillExecutionKernel<PlayerSkillDispatch>,
    ) -> bool {
        self.begin_player_skill_execution(player_id, kernel)
    }

    fn player_skill_last_used_ms(&self, player_id: i32, skill_id: u32) -> u32 {
        self.player_skill_last_used_ms(player_id, skill_id)
    }

    fn finish_player_skill(
        &mut self,
        player_id: i32,
        player_ai: &mut Self::PlayerAi,
        dispatch: PlayerSkillDispatch,
        termination: nebokrai_zone::skills::SkillTermination,
    ) -> bool {
        self.finish_player_skill(player_id, player_ai, dispatch, termination)
    }

    fn find_player(&self, player_id: i32) -> Option<&CPlayer> { self.find_player(player_id) }

    fn find_player_mut(&mut self, player_id: i32) -> Option<&mut CPlayer> { self.find_player_mut(player_id) }

    fn thunder_blow_skill_level(&self, player_id: i32) -> Option<i32> {
        Some(self.find_player(player_id)?.learned_skill_level(THUNDER_BLOW_SKILL_ID, self.skill_factory()))
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn base_magic_target_view(
        &self,
        region_id: i32,
        target: ShapeIdentity,
    ) -> Option<crate::gameserver::appserver::shape::ShapeView> {
        self.base_magic_target_view(region_id, target)
    }

    fn thunder_blow_monster_hit_points(&self, region_id: i32, monster_id: i32) -> Option<u32> {
        Some(self.find_region(region_id)?.base().find_monster_by_id(monster_id)?.hit_points())
    }

    fn base_magic_path(
        &self,
        region_id: i32,
        source_x: i32,
        source_y: i32,
        target_x: i32,
        target_y: i32,
        forced_length: Option<u32>,
    ) -> Vec<(i32, i32, u8)> {
        self.base_magic_path(region_id, source_x, source_y, target_x, target_y, forced_length)
    }

    fn thunder_blow_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8> {
        self.find_region(region_id)?.base().block_at(x, y)
    }

    fn send_self_state_skill_failure(&self, message_type: i32, player_id: i32, action: u8) {
        self.send_self_state_skill_failure(message_type, player_id, action);
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount)
    }

    fn update_player_fight_state_move_shape(&mut self, player_id: i32) -> bool {
        self.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi).is_some()
    }

    fn send_player_shape_around(
        &mut self,
        player_id: i32,
        excluded_player_id: Option<i32>,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        let _ = self.send_player_shape_around(player_id, excluded_player_id, message);
    }

    fn allocate_summon_shape_id(&mut self) -> i32 { self.allocate_summon_shape_id() }

    fn weapon_damage_factors(&self) -> (f32, f32) { self.globe_setup().weapon_damage_factors() }

    fn thunder_blow_weapon_modifier(
        &self,
        player: &CPlayer,
        target_level: i32,
        divisor: f32,
        minimum_factor: f32,
    ) -> f32 {
        player.weapon_modifier(self.goods_factory(), target_level, divisor, minimum_factor)
    }

    fn critical_rate(&self) -> f32 { self.globe_setup().critical_rate() }

    fn base_combat_scales(&self) -> [f32; 5] { self.globe_setup().base_combat_scales() }

    fn skill_random_below(&mut self, maximum: i32) -> i32 { self.skill_random_below(maximum) }
}

impl<Runtime: GameMainLoopRuntime> ThunderBlowContact<Runtime> for CGame {
    fn finish_immediate_base_attack(&mut self, player_id: i32, skill_id: u32, runtime: &mut Runtime) {
        finish_immediate_base_attack(self, player_id, skill_id, runtime);
    }

    fn add_thunder_blow_phalanx(
        &mut self,
        region_id: i32,
        phalanx: zone::CThunderBlowPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<bool> {
        self.add_thunder_blow_phalanx(region_id, phalanx, tile_x, tile_y, started_at_ms, runtime)
            .map(|result| result.is_ok())
    }

    fn send_thunder_blow_phalanx_entry(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.send_thunder_blow_phalanx_entry(region_id, phalanx_id, runtime)
    }
}

pub(crate) const fn is_thunder_blow_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_thunder_blow_dispatch(dispatch)
}

pub(crate) fn cancel_player_thunder_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    zone::cancel_player_thunder_blow(game, player_id, player_ai, runtime)
}

pub(crate) fn execute_player_thunder_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let state = match zone::execute_player_thunder_blow(
        game, player_id, dispatch, runtime, game_tick_milliseconds,
    ) {
        ThunderBlowOutcome::Begun => QueuedSkillExecutionState::Begun,
        ThunderBlowOutcome::Pending => QueuedSkillExecutionState::Pending,
        ThunderBlowOutcome::Completed => QueuedSkillExecutionState::Completed,
        ThunderBlowOutcome::Rejected => QueuedSkillExecutionState::Rejected,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}
