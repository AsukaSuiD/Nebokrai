//! Делегат ИИ слабого существа `CPuninessCreature` (AI7) в Zone.
//!
//! Ближайший выбор игрока/питомца, собственное расписание с пошаговым
//! отходом и отдельный `OnSearchEnemy` перенесены буквально в
//! `nebokrai_zone::ai::puninesscreature` — машинная база `MATCH` по точной
//! паре `4F5C98E0…` + GameServer.pdb (RSDS match), RVA-якоря (`0x0060F390`
//! Tracing, `0x0060F4B0` OnSchedule, `0x0060F4E0` OnSearchEnemy) и
//! tamed-исключение описаны в её шапке волной Z-AI. Здесь — прежние
//! сигнатуры: общий `MoveTo`/idle, применение цели и runtime-вход `CGame`
//! остаются hub-владением через фасады `MonsterDispatcher*`. Потребители
//! (`game.rs`, `monsterai.rs`, `monsterbaseattack`) не меняются.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};

/// Исполняет достигнутую вертикаль `OnSchedule → Tracing` AI7 (прежняя
/// сигнатура): шаг от цели внутри дальности охраны, сброс за дальностью
/// преследования, без цели — общий idle.
pub(crate) fn execute_owned_puniness_creature<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::puninesscreature::execute_owned_puniness_creature(
        game, region, monster_id, game_tick_milliseconds,
    )
}

/// Выполняет отдельный `OnSearchEnemy` AI7 без движения и без запуска навыка
/// (прежняя сигнатура).
pub(crate) fn search_puniness_enemy(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
) -> bool {
    nebokrai_zone::ai::puninesscreature::search_puniness_enemy(game, region, monster_id)
}
