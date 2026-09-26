//! Делегат городского мечевого охранника `CCityGuardWithSword` (AI10) и
//! унаследованной постовой семьи в Zone.
//!
//! Точка поста, городской selector (игроки/питомцы с фильтрами владельца
//! города и минимальной дистанцией навыка), хвост проверки дистанции до
//! поста и ветвь `Tracing` перенесены буквально в
//! `nebokrai_zone::ai::cityguardwithsword` — машинная база `MATCH` по точной
//! паре `4F5C98E0…` + GameServer.pdb (RSDS match), RVA-якоря (`0x0060E290`
//! OnSearchEnemy AI10, `0x0060DB10` AI11, `0x0060E350`/`0x0060E510`
//! selector-пара, vtable `0x00662BCC`) описаны в её шапке волной Z-AI вместе
//! с устранённым расхождением: третий виртуал `SearchEnemyGuildCarriage`
//! (`+0x98`) городские `OnSearchEnemy`/`WhenBeenHurted` не вызывают (его
//! caller-ы — только `CVilCouGuardWithBow` AI16), поэтому проход повозок
//! `603` из городского selector-а удалён. Здесь:
//!
//! - реализации hub-трейтов Zone над прежними `CGame`, `CPlayer`,
//!   `CMonster`, `CMoveShape` и `CServerRegion` — состояние поста остаётся
//!   полем переходного `CMonster`, `ForceMove` и RNG клеток — hub-швами;
//! - делегации с прежними сигнатурами и переэкспорт `CitySwordTraceOutcome`
//!   — потребители старого пакета (`monsterbaseattack`, шов
//!   `MonsterDispatcherRuntime` в `monsterai.rs`) не меняются.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeView;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};
use nebokrai_zone::ai::guardtarget::GuardDistanceTarget;

use nebokrai_zone::ai::cityguardwithsword::{
    CityGuardDispatcherMoveShape, CityGuardDispatcherPlayer, CityGuardDispatcherRegion,
    GuardStationDispatcherMonster,
};

pub(crate) use nebokrai_zone::ai::cityguardwithsword::CitySwordTraceOutcome;
pub(crate) use super::guardtarget::GuardStationState;

impl GuardStationDispatcherMonster for CMonster {
    fn guard_station_ai_mut(&mut self) -> Option<&mut GuardStationState> {
        CMonster::guard_station_ai_mut(self)
    }
}

impl CityGuardDispatcherMoveShape for CMoveShape {
    fn current_skill_id(&self) -> Option<u32> {
        CMoveShape::current_skill_id(self)
    }
}

impl CityGuardDispatcherPlayer for CPlayer {
    fn faction_id(&self) -> i32 {
        CPlayer::faction_id(self)
    }

    fn union_id(&self) -> i32 {
        CPlayer::union_id(self)
    }
}

impl CityGuardDispatcherRegion for CServerRegion {
    fn owned_city_faction(&self) -> i32 {
        CServerRegion::owned_city_faction(self)
    }

    fn owned_city_union(&self) -> i32 {
        CServerRegion::owned_city_union(self)
    }
}

/// Выбирает цель AI10/AI11 отдельными исходными проходами игроков и
/// питомцев (прежняя сигнатура). Минимальная дистанция сохраняется
/// caller-ом текущего навыка.
pub(crate) fn select_city_guard_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget> {
    nebokrai_zone::ai::cityguardwithsword::select_city_guard_enemy(
        game, region, owner, area_index, guard_range, minimum_skill_distance,
    )
}

/// Ветка `OnSearchEnemy` с уже имеющейся целью (прежняя сигнатура).
pub(crate) fn check_guard_station_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    chase_range: i32,
    runtime: &mut Runtime,
) {
    nebokrai_zone::ai::cityguardwithsword::check_guard_station_target(
        game, region, monster_id, chase_range, runtime,
    );
}

/// Выполняет виртуальный `OnLoseTarget` постовой семьи: очистка цели и
/// возврат к посту (прежняя сигнатура, шов `MonsterDispatcherRuntime`).
pub(crate) fn release_guard_sword_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) {
    nebokrai_zone::ai::cityguardwithsword::release_guard_sword_target(
        game, region, monster_id, runtime,
    );
}

/// Выполняет `OnLoseTarget` мечевого охранника перед внешним
/// `ASA_SEARCH_ENEMY` его schedule-owner-а (прежняя сигнатура).
#[allow(dead_code, reason = "прежняя hub-форма; schedule-owner вызывает release+search швами")]
pub(crate) fn lose_guard_sword_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) {
    nebokrai_zone::ai::cityguardwithsword::lose_guard_sword_target(
        game, region, monster_id, runtime, game_tick_milliseconds,
    );
}

/// Выполняет общую ветвь `Tracing` AI10 и производных AI15/AI19 (прежняя
/// сигнатура).
#[allow(clippy::too_many_arguments, reason = "граница сохраняет отдельные пределы навыка и преследования")]
pub(crate) fn trace_city_sword_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    owner: ShapeView,
    target: ShapeView,
    minimum_distance: i32,
    maximum_distance: i32,
    chase_range: i32,
    runtime: &mut Runtime,
) -> CitySwordTraceOutcome {
    nebokrai_zone::ai::cityguardwithsword::trace_city_sword_target(
        game,
        region,
        monster_id,
        owner,
        target,
        minimum_distance,
        maximum_distance,
        chase_range,
        runtime,
        game_tick_milliseconds,
    )
}
