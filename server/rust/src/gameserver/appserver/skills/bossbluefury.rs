//! Тонкий путь ярости синего босса `CBossBlueFury` (`0x1F7`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, исходный владелец
//! `appserver/skills/bossbluefury.cpp/.h`. Тела Check/AI (`0x52E8E0`/
//! `0x52EAC0`), порядок состояний (продув каждого прежнего `0x1F7` → новый
//! `CBossBlueFuryState` → UpdateProperty `0x52EDAC` → End(1)) и монстровый
//! owned-вход перенесены буквально в `nebokrai_zone::skills::bossbluefury`
//! (статусы и швы — там и в `.local/recon-de/notes/E4-bossbluefury.md`;
//! **FIX F3** — машинный полный продув вместо первого типизированного ключа;
//! состояние — соседний `bossbluefurystate.rs`). Здесь — делегации с прежними
//! сигнатурами: зарегистрированный вход игрока идёт общим stateskill,
//! монстровый — hub `monsterattack` над `CGame`; потребители (`game.rs`,
//! `monsterbaseattack.rs`, `monster.rs`) не меняются.

use super::kernel::SkillTermination;
use super::statecast::finish_state_cast;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, execute_player_state_skill,
    finish_player_state_skill,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, ServerRegionOwner,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::bossbluefury as zone;

pub(crate) use nebokrai_zone::skills::bossbluefury::BOSS_BLUE_FURY_SKILL_ID;

pub(crate) const fn is_player_boss_blue_fury_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_boss_blue_fury_dispatch(dispatch)
}

struct BossBlueFury;

impl RegisteredStateSkill for BossBlueFury {
    const ID: u32 = BOSS_BLUE_FURY_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::BossBlueFury;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, _target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        zone::check_boss_blue_fury_cast(game, address, &mut || runtime.now_milliseconds())
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = zone::run_boss_blue_fury_ai(game, address, &mut || runtime.now_milliseconds());
        finish_state_cast(game, address, outcome, runtime)
    }
}

pub(crate) fn publish_boss_blue_fury_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    zone::publish_boss_blue_fury_visual(game, skill, mode);
}

pub(crate) fn cancel_player_boss_blue_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<BossBlueFury, Runtime>(
        game, player_id, player_ai, 0, SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_boss_blue_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<BossBlueFury, Runtime>(game, player_id, dispatch, player_ai, runtime)
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт исходного навыка")]
pub(crate) fn execute_owned_boss_blue_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    zone::execute_owned_boss_blue_fury(
        game, owner, monster_id, target_identity, skill_level, properties, now_ms, runtime,
        game_tick_milliseconds,
    )
}
