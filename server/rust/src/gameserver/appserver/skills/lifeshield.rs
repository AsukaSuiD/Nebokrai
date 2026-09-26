//! Тонкий путь к Check/AI щита жизни CLifeShield (0x220) в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/lifeshield.cpp (Begin×3 `0x118190…`, Check `0x118850`,
//! AI `0x118A60`, собственный End(H) `0x11A700`). Тела перенесены буквально в
//! `nebokrai_zone::skills::lifeshield` порцией №6b. Ключи свойств остаются
//! здесь: hub-lifecycle `lifeshieldstate.rs` читает срок по прежнему пути.
//! Делегация исполнения сохраняет прежнюю сигнатуру; потребители не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::LIFE_SHIELD_SKILL_ID;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_STATE_HP: u32 = 10_010;
pub(crate) const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
pub(crate) const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

pub(crate) fn execute_battle_fairy_life_shield<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_life_shield(
        game, player_id, instance, dispatch, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
    ))
}
