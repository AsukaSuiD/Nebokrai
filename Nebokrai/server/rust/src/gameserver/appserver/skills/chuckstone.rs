//! Владелец метания камня `CChuckStone` (`0x19D`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/chuckstone.cpp`. Player-перегрузки входят в общий прямой
//! снаряд `directprojectile`: проверяют оружие, сохраняют точку исчезнувшей
//! цели, рассчитывают первую преграду и после полёта атакуют клетку в живом
//! порядке региона. Monster/pet-путь использует `monsterprojectile`. Формулы,
//! два RNG-вызова, пакеты и задержки принадлежат этим skill-owner-ам; `CGame`
//! только разрешает владельцев, применяет удар и выполняет доставку.
//! Не достигнуты вызов минимальной дальности, региональная отмена и режим
//! `m_bAutoRestart`; их точные фрагменты сохранены ниже.

use super::directprojectile::execute_player_direct_projectile;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome};

pub(crate) const CHUCK_STONE_SKILL_ID: u32 = 0x19d;

pub(crate) fn execute_player_chuck_stone<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_direct_projectile(game, player_id, dispatch, player_ai, runtime)
}

// FUNCTION: CChuckStone::GetAffectRangeMin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:187
// RVA: 0x001387A0
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// FUNCTION: CChuckStone::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\chuckstone.cpp:573
// RVA: 0x0013CF10
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
