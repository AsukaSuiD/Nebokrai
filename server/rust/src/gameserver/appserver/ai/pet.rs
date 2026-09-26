//! Делегат семьи `CPet` в Zone.
//!
//! Состояние lifecycle и поведения питомца, `pet_master_ref`, active-поиск,
//! follow/idle и OnLoseTarget-семья перенесены буквально в
//! `nebokrai_zone::ai::pet` — машинная база VERIFIED по точной паре
//! `4F5C98E0…` + GameServer.pdb (RSDS match), оставшиеся UNKNOWN
//! (BSS-глобалы `0xEF44E8`/`0xEF44EC` watchdog/преследования, нить вызова
//! Evanish/отзыва) перечислены в её шапке. Здесь — прежние сигнатуры и
//! переэкспорт типов: state-машина `CMonster`, слот следования `CPlayer` и
//! пространственный рантайм остаются hub-владением через фасады
//! `MonsterDispatcher*` (`appserver/ai/monsterai.rs`). Watch-hz
//! `PetLifecycleState::tick` исполняет конкретный Evanish/watchdog caller в
//! hub (старый животный тик), эта делегация его не подменяет.

use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};

pub(crate) use nebokrai_zone::ai::pet::{
    PetBehaviorState, PetLifecycleFacts, PetLifecycleNotice, PetLifecycleOutcome, PetMasterRef,
    pet_master_ref,
};

/// Выбирает ближайшего дикого монстра для активного питомца (прежняя
/// сигнатура); равная дальность побеждает более позднюю запись обхода.
pub(crate) fn execute_owned_pet_active_search(
    game: &CGame,
    region: &mut CServerRegion,
    region_id: i32,
    monster_id: i32,
) -> bool {
    nebokrai_zone::ai::pet::execute_owned_pet_active_search(game, region, region_id, monster_id)
}

/// Ставит точный `CPet::OnIdle` без случайного движения (прежняя сигнатура).
pub(crate) fn queue_pet_idle<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    stop_frame: u32,
    factory: &CSkillFactory,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::pet::queue_pet_idle(
        region, monster_id, stop_frame, factory, game_tick_milliseconds,
    )
}

/// Выполняет `CPet::OnLoseTarget` и отдельный SearchEnemy его schedule-caller-а
/// (прежняя сигнатура).
pub(crate) fn lose_pet_target_and_search<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    _runtime: &mut Runtime,
) {
    nebokrai_zone::ai::pet::lose_pet_target_and_search(region, monster_id, game_tick_milliseconds)
}

/// Чистый virtual CPet::OnLoseTarget, общий для расписания и death FIFO.
pub(crate) fn release_pet_target(
    region: &mut CServerRegion,
    monster_id: i32,
) {
    nebokrai_zone::ai::pet::release_pet_target(region, monster_id)
}

/// Исполняет достигнутое следование `CPet` (прежняя сигнатура): слот позади
/// хозяина, ближний шаг или перенос в свободную клетку `7x7`.
pub(crate) fn execute_owned_pet_follow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    region_id: i32,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::pet::execute_owned_pet_follow(
        game, region, region_id, monster_id, runtime, game_tick_milliseconds,
    )
}
