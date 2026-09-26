//! Тонкий путь громового рассечения `CThunderSlash` (0x72) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/thunderslash.cpp.
//! Тела Check/AI/Summon и слитый visual `CThunderSlashEffect` перенесены
//! буквально в `nebokrai_zone::skills::thunderslash` (машинные якоря, статусы
//! MATCH/UNKNOWN и швы — там); phalanx — в
//! `nebokrai_zone::skills::thunderslashphalanx`. Здесь — делегации с прежними
//! сигнатурами и фасадные реализации hub-трейтов Zone над прежними методами
//! `CGame`/`CPlayer`; registered-вход (`playercast.rs`) и потребители не
//! меняются.

use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use crate::nets::netserver::message::GameMessageDomainOps;
use nebokrai_zone::skills::thunderslash::{
    ThunderSlashContact, ThunderSlashGame, ThunderSlashOutcome, ThunderSlashPkPermissions,
    ThunderSlashPlayer,
};

pub(crate) use nebokrai_zone::skills::thunderslash::THUNDER_SLASH_SKILL_ID;

impl ThunderSlashPlayer for CPlayer {
    fn shape(&self) -> &crate::gameserver::appserver::shape::CShape { self.shape() }

    fn set_skill_moveable(&mut self, moveable: bool) { self.set_skill_moveable(moveable) }

    fn player_id(&self) -> i32 { self.player_id() }

    fn faction_id(&self) -> i32 { self.faction_id() }

    fn team_id(&self) -> i32 { self.team_id() }

    fn union_id(&self) -> i32 { self.union_id() }

    fn thunder_slash_pk_permissions(&self) -> ThunderSlashPkPermissions {
        let permissions = self.pk_permissions();
        // MasterInfo этого владельца хранит четвёрку допусков и нулевую страну.
        ThunderSlashPkPermissions {
            player: permissions.player,
            teammate: permissions.teammate,
            guild_member: permissions.guild_member,
            criminal: permissions.criminal,
        }
    }
}

impl ThunderSlashGame for CGame {
    fn thunder_slash_player_weapon_valid(&self, player: &CPlayer) -> bool {
        player.equipment().get_goods(2).is_some_and(|weapon| {
            weapon.addon_property_value(self.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
        })
    }

    fn thunder_slash_region_present(&self, region_id: i32) -> bool {
        self.find_region(region_id).is_some()
    }

    fn allocate_summon_shape_id(&mut self) -> i32 { self.allocate_summon_shape_id() }

    fn send_thunder_slash_visual_to_player(
        &self,
        player_id: i32,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    fn send_thunder_slash_visual_around(
        &self,
        region_id: i32,
        origin: &crate::gameserver::appserver::shape::CShape,
        message: &nebokrai_zone::app::game_message::CMessage,
    ) {
        if let Some(owner) = self.find_region(region_id) {
            let _ = self.send_game_shape_around(owner.base(), origin, None, message);
        }
    }
}

impl<Runtime: GameMainLoopRuntime> ThunderSlashContact<Runtime> for CGame {
    fn spawn_thunder_slash_phalanx(
        &mut self,
        region_id: i32,
        phalanx: nebokrai_zone::skills::thunderslashphalanx::CThunderSlashPhalanx,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        self.spawn_thunder_slash_phalanx(region_id, phalanx, started_at_ms, runtime)
    }
}

pub(crate) fn execute_player_thunder_slash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    use nebokrai_zone::skills::thunderslash as zone;
    if dispatch.skill_id() != THUNDER_SLASH_SKILL_ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ThunderSlash,
        |game, instance, _player_id, _runtime| original_user
            .is_some_and(|source| zone::check_thunder_slash_cast(game, instance, source, game_tick_milliseconds)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        |game, instance, runtime| {
            let state = match zone::run_thunder_slash_ai(game, instance, runtime, game_tick_milliseconds) {
                ThunderSlashOutcome::Pending => QueuedSkillExecutionState::Pending,
                ThunderSlashOutcome::Rejected => QueuedSkillExecutionState::Rejected,
                ThunderSlashOutcome::Completed => QueuedSkillExecutionState::Completed,
            };
            state_skill_outcome(state)
        },
    )
}
