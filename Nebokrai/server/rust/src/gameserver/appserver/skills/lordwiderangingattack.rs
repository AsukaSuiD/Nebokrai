//! Широкая атака владыки `CLordWiderangingAttack` (`0x1f6`) для объектного пути игрока и монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lordwiderangingattack.cpp`. Точечная проверка EXE
//! подтвердила отдельные глобальные данные `0x006A1608..0x006A162C`: полную
//! маску 5×5 и две дуги по три клетки. Проверка, порядок стадий, формула,
//! два вызова RNG на допустимую цель и сетевой контракт совпадают с
//! `CMachineryStomp`, кроме ID и таблицы свойств, поэтому объектный путь игрока
//! и монстра использует один узкий семейный владелец. Координатные перегрузки
//! `Begin` остаются ниже недостигнутыми.

// ============================================================================
// FUNCTION: CLordWiderangingAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:141
// RVA: 0x0012EE70
// ADDRESS: 0052ee70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:161
// RVA: 0x0012EF50
// ADDRESS: 0052ef50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

use super::machinerystomp::{
    WideArcAttackDispatch, execute_player_wide_arc_attack, prepare_owned_wide_arc_attack,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) const LORD_WIDERANGING_ATTACK_SKILL_ID: u32 = 0x1f6;

pub(crate) const fn is_lord_wideranging_attack_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Object {
            skill_id: LORD_WIDERANGING_ATTACK_SKILL_ID,
            target: ShapeIdentity { object_type: 400 | 600, .. },
        }
    )
}
pub(crate) fn execute_player_lord_wideranging_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_wide_arc_attack(
        game,
        player_id,
        dispatch,
        LORD_WIDERANGING_ATTACK_SKILL_ID,
        player_ai,
        runtime,
    )
}

#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn prepare_owned_lord_wideranging_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    dispatch: &mut Option<WideArcAttackDispatch>,
) -> bool {
    prepare_owned_wide_arc_attack(
        game,
        region,
        monster_id,
        target_identity,
        LORD_WIDERANGING_ATTACK_SKILL_ID,
        skill_level,
        properties,
        now_ms,
        runtime,
        dispatch,
    )
}
