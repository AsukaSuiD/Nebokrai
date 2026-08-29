//! Владелец метания камня `CChuckStone` (`0x19D`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/chuckstone.cpp`. Player-перегрузки входят в общий прямой
//! снаряд `directprojectile`: проверяют оружие, сохраняют точку исчезнувшей
//! цели, рассчитывают первую преграду и после полёта атакуют клетку в живом
//! порядке региона. Monster/pet-путь использует `monsterprojectile`. Формулы,
//! два RNG-вызова, пакеты и задержки принадлежат этим skill-owner-ам; `CGame`
//! только разрешает владельцев, применяет удар и выполняет доставку.
//! Минимальная дальность сохраняет нижнюю границу `1`, а вход в другой регион
//! завершает активный снаряд через `End(false)`, не фиксируя время
//! восстановления. Не достигнут только режим `m_bAutoRestart`; его точный
//! фрагмент сохранён ниже.

use super::directprojectile::{
    abort_player_direct_projectile_on_region_change, execute_player_direct_projectile,
};
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

pub(crate) fn on_player_chuck_stone_change_region(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
) -> bool {
    abort_player_direct_projectile_on_region_change(
        game,
        player_id,
        player_ai,
        CHUCK_STONE_SKILL_ID,
    )
}

// Недостигнутая ветвь `CChuckStone::AI` после удара:
// При ненулевом m_bAutoRestart вызывается виртуальный метод со смещением
// 0x20, затем функция завершается. Основание: локальный декомпилят; INFERRED.
