//! ИИ слабого существа `CPuninessCreature` (AI7): поиск ближайшего живого
//! игрока или питомца и собственное расписание с пошаговым отходом от цели
//! без интервального гейта.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\puninesscreature.cpp`.
//! Машинная сверка: разобраны все три метода класса, кроме ctor `0x0060F370`:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `OnSchedule`: пустой общий hook (`0x00485540`), owner ≠ 0, `HasTarget == 1` и пустые очереди `[+0x14]`/`[+0x28]` → tail-call виртуального `Tracing` (vt `+0x4C`); без цели расписание ничего не ставит (следующий вход без цели обслуживает общий `OnIdle`) | VA `0x0060F4B0` | [`execute_owned_puniness_creature`] (гейты очередей и tamed-исключение), runtime-вход — hub | `MATCH` |
//! | `Tracing`: цель отсутствует или `IsDied == 1` → `OnLoseTarget + AddAIEvent(5)`, возврат 0; внутри `GetGuardRange` (vt `+0x138`) — направление `GetLineDir(target → owner)`, `GetDirPos` и общий координатный `MoveTo(run=0)` (AI vtable `+0x58`, `0x004C9020`, `0x0060F45B`), **без** записи `SetDir`; за дальностью охраны — проверка `GetChaseRange` (vt `+0x13C`) и возможная потеря цели | VA `0x0060F390` | [`execute_owned_puniness_creature`] | `MATCH` |
//! | `OnSearchEnemy`: игроки перед питомцами в девяти соседних областях, живые внутри `GetGuardRange`, ближайший с заменой записи при `≤` (равная `RealDistance` побеждает более позднюю запись), назначение через virtual `SetTarget` | VA `0x0060F4E0` | [`search_puniness_enemy`] | `MATCH` |
//!
//! Питомцы попадают в проход через общий `FindAroundPets`-список региона;
//! tamed-фильтр hub-формы сохранён. Проверка принадлежности исключает
//! приручённого монстра (его текущий владелец `CPet`, даже при setup AI7).
//! `GetDirPos` (`0x0045B330`) безотказен для восьми направлений.
//!
//! Остаются hub-владением: общий monster tick hub, runtime-вход `CGame`
//! (собственный `OnSchedule` вызывается до background/passive) и применение
//! цели; пространственная мутация — общий `MoveTo` `ai/monsterai.rs`.

use nebokrai_shared::runtime::get_line_direction;

use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, ShapeView};

use super::lord::{EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherOwner, MonsterDispatcherPlayer, MonsterDispatcherRegion,
    move_owned_monster_to, queue_monster_idle,
};

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PuninessTarget {
    identity: ShapeIdentity,
    distance: i32,
}

/// Ближайший с заменой записи при равной дистанции (`≤` машинного сравнения).
fn consider_target(
    selected: Option<PuninessTarget>,
    candidate: PuninessTarget,
    guard_range: i32,
) -> Option<PuninessTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    match selected {
        Some(current) if current.distance < candidate.distance => Some(current),
        _ => Some(candidate),
    }
}

fn live_target_view<Game, Region>(
    game: &Game,
    region: &Region,
    identity: ShapeIdentity,
) -> Option<ShapeView>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: MonsterDispatcherRegion,
{
    match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).and_then(|player| {
            (player.server_region_id() == Some(region.region_id()) && !player.is_dead())
                .then(|| player.shape_view())
                .flatten()
        }),
        MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
            if is_died(monster.hit_points()) {
                return None;
            }
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.shape_view(property)
        }),
        _ => None,
    }
}

/// Исполняет достигнутую вертикаль `OnSchedule → Tracing` AI7: при
/// существующей цели и пустых основных очередях — один шаг от цели внутри
/// дальности охраны и сброс за дальностью преследования; без цели — общий
/// `CMonsterAI::OnIdle` его schedule-владельца.
pub fn execute_owned_puniness_creature<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    now_milliseconds: fn() -> u32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    if region
        .find_monster_by_id(monster_id)
        .is_some_and(MonsterDispatcherMonster::is_tamed)
    {
        return false;
    }
    let Some((property, source_view, target, schedule_idle)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                property.clone(),
                monster.shape_view(&property)?,
                monster.ai_target(),
                monster.primary_ai_queues_idle(),
            ))
        })
    else {
        return false;
    };
    if property.ai != 7 {
        return false;
    }
    if let Some(target) = target {
        if !schedule_idle {
            return true;
        }
        let Some(target_view) = live_target_view(game, region, target) else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.lose_ai_target_and_search(now_milliseconds(), game.skill_factory());
            }
            return true;
        };
        let distance = source_view.real_distance(Some(target_view));
        if distance > property.guard_range as i32 {
            if distance > property.chase_range as i32
                && let Some(monster) = region.find_monster_by_id_mut(monster_id)
            {
                monster.lose_ai_target_and_search(now_milliseconds(), game.skill_factory());
            }
            return true;
        }
        let direction = get_line_direction(
            target_view.tile_x,
            target_view.tile_y,
            source_view.tile_x,
            source_view.tile_y,
        );
        let origin = ShapeAreaCoordinates {
            x: source_view.tile_x,
            y: source_view.tile_y,
        };
        if let Ok(destination) = CShape::get_direction_position(direction, origin) {
            move_owned_monster_to(game, region, monster_id, destination, 0, now_milliseconds);
        }
        return true;
    }

    if !schedule_idle {
        return true;
    }
    queue_monster_idle(game, region, monster_id, &property, now_milliseconds)
}

/// Выполняет отдельный `OnSearchEnemy` AI7 без движения и без запуска навыка.
pub fn search_puniness_enemy<Game, Region>(
    game: &Game,
    region: &mut Region,
    monster_id: i32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    let Some((property, source_view, area_index)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let source_view = monster.shape_view(&property)?;
            let area_index = monster.move_shape().shape().area_index()?;
            Some((property, source_view, area_index))
        })
    else {
        return false;
    };
    if property.ai != 7 {
        return false;
    }
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let identity = ShapeIdentity {
            object_type: PLAYER_TYPE,
            id: player_id,
            ex_id: nebokrai_shared::values::CGuid::GUID_INVALID,
        };
        let Some(candidate) = live_target_view(game, region, identity) else {
            continue;
        };
        selected = consider_target(
            selected,
            PuninessTarget {
                identity,
                distance: source_view.real_distance(Some(candidate)),
            },
            property.guard_range as i32,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(pet) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !is_died(pet.hit_points()))
        else {
            continue;
        };
        let Some(property_key) = pet.base_property_key() else {
            continue;
        };
        let Some(pet_property) = game.find_monster_property_by_origin_name(property_key) else {
            continue;
        };
        let Some(candidate) = pet.shape_view(pet_property) else {
            continue;
        };
        selected = consider_target(
            selected,
            PuninessTarget {
                identity: candidate.identity,
                distance: source_view.real_distance(Some(candidate)),
            },
            property.guard_range as i32,
        );
    }
    if let Some(selected) = selected
        && let Some(monster) = region.find_monster_by_id_mut(monster_id)
    {
        monster.set_ai_target(selected.identity);
    }
    true
}
