//! Делегат ИИ владыки `CLord` (AI100) в Zone.
//!
//! Hurt-план бегового отвода от призванной формы, общий enemy-проход и
//! фазовый выбор навыка перенесены буквально в `nebokrai_zone::ai::lord` —
//! машинная база `MATCH` по точной паре `4F5C98E0…` + GameServer.pdb (RSDS
//! match), RVA-якоря (`0x0060AF60` Select, `0x0060B0B0` WhenBeenHurted),
//! граница tick hub (`Run` `0x0060E250`, `OnSchedule` `0x0060AF50`) и швы
//! описаны в её шапке. Здесь:
//!
//! - реализации hub-трейтов Zone над прежними `CGame`, `CPlayer`,
//!   `CServerRegion` и `CMonster` — identities клетки собираются тем же
//!   `GetShape` с resolver-ом владельца, перечисление players/pets — прежними
//!   девяти-area проходами региона;
//! - делегации с прежними сигнатурами и переэкспорт `LordHurtPlan`/
//!   `select_lord_attack_skill` — потребители старого пакета (урон игрока и
//!   периодики в `game.rs`/`periodicattack.rs`, `monsterbaseattack`) не
//!   меняются. Разрешение region owner-а по ID до tick-изъятия остаётся
//!   hub-композицией этого файла.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

use nebokrai_zone::ai::lord::{
    EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion, LordDispatcherGame,
    LordDispatcherMonster,
};

pub(crate) use nebokrai_zone::ai::lord::{LordHurtPlan, select_lord_attack_skill};

impl LordDispatcherMonster for CMonster {
    fn when_been_hurted(&mut self, now_ms: u32) {
        self.when_been_hurted(now_ms);
    }
}

impl LordDispatcherGame for CGame {
    fn area_dimensions(&self) -> (i32, i32) {
        self.area_dimensions()
    }

    fn shape_identities_at_cell(
        &self,
        region: &CServerRegion,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> Option<Vec<ShapeIdentity>> {
        let mut shapes = Vec::new();
        region
            .get_shapes(tile_x, tile_y, area_width, area_height, self, &mut shapes)
            .ok()?;
        Some(shapes.iter().map(|shape| shape.identity).collect())
    }
}

impl EnemySearchDispatcherRegion for CServerRegion {
    fn player_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        self.player_ids_around_area(area_index)
    }

    fn pet_ids_around_area(&self, area_index: usize) -> Vec<i32> {
        self.pet_ids_around_area(area_index)
    }
}

impl EnemySearchDispatcherPlayer for CPlayer {
    fn is_dead(&self) -> bool {
        self.is_dead()
    }
}

/// Вычисляет неизменяющую пространственную часть `WhenBeenHurted` до
/// временного изъятия region-owner-а из `CGame` (прежняя сигнатура).
pub(crate) fn plan_lord_hurt_response(
    game: &CGame,
    region_id: i32,
    monster_id: i32,
    property: &MonsterProperties,
) -> LordHurtPlan {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else {
        return LordHurtPlan::default();
    };
    nebokrai_zone::ai::lord::plan_lord_hurt_response(game, region, monster_id, property)
}

/// Применяет упорядоченную часть `WhenBeenHurted`: Defense, беговой MoveTo и
/// назначение атакующего только при всё ещё пустой текущей цели (прежняя
/// сигнатура).
pub(crate) fn apply_lord_hurt_response(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    attacker: ShapeIdentity,
    now: impl FnMut() -> u32,
    plan: LordHurtPlan,
) -> bool {
    nebokrai_zone::ai::lord::apply_lord_hurt_response(
        game, region, monster_id, attacker, now, plan,
    )
}

/// Выполняет подтверждённый `OnSearchEnemy` AI100 через общий nearest-проход
/// (прежняя сигнатура).
pub(crate) fn select_lord_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    nebokrai_zone::ai::lord::select_lord_enemy(game, region, owner, area_index, guard_range)
}
