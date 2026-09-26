//! Делегат ИИ демона-босса `CBossFiend` (AI104) в Zone.
//!
//! Восемь одноразовых HP-порогов призыва, строгий таймер повторного призыва,
//! пороговый выбор навыка и enemy-проход с минимальной дистанцией текущего
//! навыка перенесены буквально в `nebokrai_zone::ai::bossfiend` — машинная
//! база `MATCH` по точной паре `4F5C98E0…` + GameServer.pdb (RSDS match),
//! RVA-якоря (ctor `0x00609310`, Select `0x00609670`; `WakeUp` слинкован ICF
//! тем же RVA, что у `CBossBlue`) и граница tick hub описаны в её шапке.
//! Здесь:
//!
//! - реализация hub-трейта Zone над прежним `CMonster` — состояние порогов и
//!   таймер остаются полем переходного владельца;
//! - делегации с прежними сигнатурами и переэкспорт `BossFiendAiState` —
//!   потребители старого пакета (`monster.rs`, `monsterbaseattack`) не
//!   меняются. Часы идут тем же швом соседей: точное значение blanket
//!   `GameClockContext::now_milliseconds` = `game_tick_milliseconds`.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};
use crate::setup::monsterlist::MonsterProperties;

use nebokrai_zone::ai::bossfiend::BossFiendDispatcherMonster;

pub(crate) use nebokrai_zone::ai::bossfiend::BossFiendAiState;

impl BossFiendDispatcherMonster for CMonster {
    fn boss_fiend_ai(&self) -> Option<&BossFiendAiState> {
        self.boss_fiend_ai()
    }

    fn boss_fiend_ai_mut(&mut self) -> Option<&mut BossFiendAiState> {
        self.boss_fiend_ai_mut()
    }
}

/// Выполняет подтверждённый `OnSearchEnemy` демона-босса: один выбор проходит
/// игроков, затем питомцев с предпочтением целей не ближе минимальной
/// дистанции текущего навыка (прежняя сигнатура).
pub(crate) fn select_boss_fiend_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity> {
    nebokrai_zone::ai::bossfiend::select_boss_fiend_enemy(
        game, region, owner, area_index, guard_range, minimum_skill_distance,
    )
}

/// Выполняет полную AI-specific фиксацию выбранного навыка демона-босса
/// (прежняя сигнатура). Проверка таймера и запись момента призыва используют
/// два отдельных чтения часов в исходных местах.
pub(crate) fn choose_boss_fiend_attack_skill<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    hit_points: u32,
    roll: i32,
    _runtime: &mut Runtime,
) -> Option<u16> {
    nebokrai_zone::ai::bossfiend::choose_boss_fiend_attack_skill(
        game, region, monster_id, property, hit_points, roll, game_tick_milliseconds,
    )
}
