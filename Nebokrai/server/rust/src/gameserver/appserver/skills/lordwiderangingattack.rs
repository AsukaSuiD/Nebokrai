//! Широкая атака владыки `CLordWiderangingAttack` (`0x1f6`) для объектного пути игрока и монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lordwiderangingattack.cpp`. Точечная проверка EXE
//! подтвердила отдельные глобальные данные `0x006A1608..0x006A162C`: полную
//! маску 5×5 и две дуги по три клетки. Проверка, порядок стадий, формула,
//! два вызова RNG на допустимую цель и сетевой контракт совпадают с
//! `CMachineryStomp`, кроме ID и таблицы свойств, поэтому объектный путь игрока
//! и монстра использует один узкий семейный владелец. Координатный player-вход
//! разрешает цель через GetSufferer на каждом такте; отсутствие цели даёт
//! failure `13` в AI после обычной проверки условий Begin.
//! Объектная цель включает NPC/постройки/ворота, но область поражает только
//! `400/600` после IsAttackAble. Delay использует абсолютный DWORD-срок;
//! Attack не добавляет RP, отмена не изнашивает оружие и не ставит reuse.

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
        PlayerSkillDispatch::SelfTarget { skill_id: LORD_WIDERANGING_ATTACK_SKILL_ID, .. }
            | PlayerSkillDispatch::Point { skill_id: LORD_WIDERANGING_ATTACK_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
            skill_id: LORD_WIDERANGING_ATTACK_SKILL_ID,
            target: ShapeIdentity { object_type: 400 | 500 | 600 | 1100 | 1200, .. },
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
