//! Владелец стрельбы скелета `CSkeletonArchery` (`0x1A1`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/skeletonarchery.cpp`. Player-перегрузки входят в общий
//! прямой снаряд `directprojectile`, а monster/pet-путь использует
//! `monsterprojectile`. Оба пути сохраняют повторный расчёт траектории перед
//! выстрелом, первую преграду, время полёта и живой порядок целей клетки.
//! Формулы, RNG и packet payload принадлежат skill-owner-ам; `CGame` только
//! разрешает владельцев, применяет удар и выполняет доставку. Общий с
//! `CChuckStone` `AfterUseSkill` один раз изнашивает оружие при `End(true)`.
//! Конструктор
//! обнуляет `m_bAutoRestart`, `Begin` его не меняет, а виртуальный `Restart`
//! наследует пустой `CState::Restart`, поэтому отдельного runtime-пути нет.

use super::directprojectile::execute_player_direct_projectile;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome};

pub(crate) const SKELETON_ARCHERY_SKILL_ID: u32 = 0x1a1;

pub(crate) fn execute_player_skeleton_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_direct_projectile(game, player_id, dispatch, player_ai, runtime)
}
