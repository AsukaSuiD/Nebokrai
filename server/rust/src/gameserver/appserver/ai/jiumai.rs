//! Делегат ИИ близнецов `CJiuMai` (AI101) в Zone.
//!
//! Состояние пары, создание/сближение близнецов, min-HP выбор цели и
//! hurt-поведение перенесены буквально в `nebokrai_zone::ai::jiumai` —
//! машинная база `MATCH` по точной паре `4F5C98E0…` + GameServer.pdb (RSDS
//! match), RVA-якоря (`0x0060A5F0` OnIdle, `0x0060A750` WhenBeenHurted,
//! `0x0060A990` OnLoseTarget, `0x0060AA50` SetTarget, `0x0060AB10`
//! OnSchedule, `0x0060AD10` OnSearchEnemy), PARTIAL-оговорка спавна при
//! отказе позиции и исправленная этой волной запись `CShape::SetDir`
//! hurt-отхода описаны в её шапке. Здесь:
//!
//! - реализации hub-трейтов Zone над прежними `CGame`, `CPlayer` и
//!   `CMonster` — состояние пары остаётся полем переходного `CMonster`,
//!   фактический спавн, RNG позиции, `ForceMove` и пространственный
//!   рантайм — hub-владением;
//! - делегации с прежними сигнатурами и переэкспорт `JiuMaiAiState` —
//!   потребители старого пакета (runtime-входы `game.rs`,
//!   `monsterbaseattack`, виртуальный шов `monsterai`) не меняются.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};
use crate::setup::monsterlist::MonsterProperties;

use nebokrai_zone::ai::jiumai::{
    JiuMaiDispatcherGame, JiuMaiDispatcherMonster, JiuMaiDispatcherPlayer,
};

pub(crate) use nebokrai_zone::ai::jiumai::JiuMaiAiState;

impl JiuMaiDispatcherMonster for CMonster {
    fn jiu_mai_ai(&self) -> Option<&JiuMaiAiState> {
        CMonster::jiu_mai_ai(self)
    }

    fn jiu_mai_ai_mut(&mut self) -> Option<&mut JiuMaiAiState> {
        CMonster::jiu_mai_ai_mut(self)
    }

    fn is_summoned_creature(&self) -> bool {
        CMonster::is_summoned_creature(self)
    }

    fn is_carriage(&self, property: &MonsterProperties) -> bool {
        CMonster::is_carriage(self, property)
    }

    fn when_been_hurted(&mut self, now_ms: u32) {
        CMonster::when_been_hurted(self, now_ms);
    }

    fn set_shape_direction(&mut self, direction: i32) {
        CMoveShape::shape_mut(self.move_shape_mut()).set_direction(direction);
    }
}

impl JiuMaiDispatcherPlayer for CPlayer {
    fn hit_points(&self) -> u32 {
        CPlayer::health(self)
    }
}

impl JiuMaiDispatcherGame for CGame {
    fn random_region_position(
        &mut self,
        region: &CServerRegion,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Option<ShapeAreaCoordinates> {
        self.random_region_position_owned(&region.region, left, top, width, height)
            .ok()
            .map(|position| ShapeAreaCoordinates {
                x: position.x,
                y: position.y,
            })
    }

    fn add_summoned_creature(
        &mut self,
        region: &mut CServerRegion,
        property: &MonsterProperties,
        master: MasterInfo,
        tile_x: i32,
        tile_y: i32,
        direction: i32,
        lifetime_ms: u32,
    ) -> Option<i32> {
        self.add_summoned_creature_owned(
            region, property, master, tile_x, tile_y, direction, lifetime_ms,
        )
        .ok()
    }

    fn force_move_owned_monster(
        &mut self,
        region: &mut CServerRegion,
        monster_id: i32,
        tile_x: i32,
        tile_y: i32,
        run: i32,
    ) -> bool {
        self.force_move_owned_monster(region, monster_id, tile_x, tile_y, run as u32)
            .and_then(Result::ok)
            .unwrap_or(false)
    }
}

/// Выполняет достигнутый префикс `OnIdle` AI101: призванный близнец берёт
/// обратный ID, обычный монстр один раз создаёт бессрочного близнеца
/// (прежняя сигнатура). Базовый общий `OnIdle` ставится caller-ом.
pub(crate) fn ensure_jiumai_twin(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
) -> bool {
    nebokrai_zone::ai::jiumai::ensure_jiumai_twin(game, region, monster_id, property)
}

/// Сохраняет достигнутый префикс `OnSchedule`: сближение живого близнеца
/// через точный случайный пункт и исходный `ForceMove(run=0)` (прежняя
/// сигнатура). Боевой хвост расписания — общий диспетчер навыка.
pub(crate) fn maintain_jiumai_twin<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::jiumai::maintain_jiumai_twin(game, region, monster_id, runtime)
}

/// Выбирает цель `OnSearchEnemy` AI101: первая живая запись с минимальным
/// текущим HP среди игроков и питомцев (прежняя сигнатура).
pub(crate) fn select_jiumai_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    nebokrai_zone::ai::jiumai::select_jiumai_enemy(game, region, owner, area_index, guard_range)
}

/// Повторяет `CJiuMai::SetTarget`: владелец — всегда, живой близнец — только
/// вне собственного боя (прежняя сигнатура).
pub(crate) fn assign_jiumai_target(
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
) -> bool {
    nebokrai_zone::ai::jiumai::assign_jiumai_target(region, monster_id, target)
}

/// Завершает достигнутый `OnLoseTarget` после общего боевого такта (прежняя
/// сигнатура): стирается только ранее связанная цель.
pub(crate) fn synchronize_jiumai_target_loss(
    region: &mut CServerRegion,
    monster_id: i32,
) -> bool {
    nebokrai_zone::ai::jiumai::synchronize_jiumai_target_loss(region, monster_id)
}

/// Материализует виртуальный `CJiuMai::OnLoseTarget` для расписания и death
/// FIFO (прежняя сигнатура, шов трейта `MonsterDispatcherGame`).
pub(crate) fn release_jiumai_target(
    region: &mut CServerRegion,
    monster_id: i32,
) -> bool {
    nebokrai_zone::ai::jiumai::release_jiumai_target(region, monster_id)
}

/// Выполняет достигнутые ветви `WhenBeenHurted` AI101 (прежняя сигнатура):
/// защита, принятие допустимого атакующего или один отход/сближение общим
/// `MoveTo(run=0)` с записью направления формы.
pub(crate) fn retarget_jiumai_after_hurt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    attacker: ShapeIdentity,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::jiumai::retarget_jiumai_after_hurt(
        game,
        region,
        monster_id,
        attacker,
        game_tick_milliseconds,
    )
}
