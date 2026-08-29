//! Общий достигнутый префикс `OnIdle` синего босса и демона-босса.
//!
//! Оба исходных владельца после выбора текущего навыка выполняют один
//! `random(10000)`: при успехе — один `random(8)` и соседний шаг, иначе —
//! ожидание `stop_frame`. Поиск противника начинается только после этой
//! последовательной задержки. Игровые формулы и выбор навыка остаются у
//! конкретных владельцев боссов, а `CGame` координирует только пространственный шаг.

use super::monsterai::queue_monster_idle;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

/// Ставит подтверждённую последовательность `MOVE/STAND → SEARCH_ENEMY` в
/// общую FIFO-очередь `CBaseAI`. Поэтому задержка уже совершённого шага не
/// допускает повторного RNG до отдельного такта поиска противника.
pub(crate) fn queue_boss_idle<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    runtime: &mut Runtime,
) -> bool {
    queue_monster_idle(game, region, monster_id, property, runtime)
}
