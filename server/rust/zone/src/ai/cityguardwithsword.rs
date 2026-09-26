//! Городские охранники `CCityGuardWithSword` (AI10) и `CCityGuardWithBow`
//! (AI11) стационарной семьи: точка поста, фильтры владельца города по
//! фракции/союзу, правило минимальной дистанции навыка и унаследованная ветвь
//! `Tracing` с сбросом за `chase_range`. Исходные владельцы PDB:
//! `appserver/ai/cityguardwithsword.cpp` и соседний `cityguardwithbow.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! Установленное расхождение hub исправлено: selector-пути AI10/AI11 не вызывают
//! `SearchEnemyGuildCarriage` (vt `+0x98`) — проход повозок принадлежит
//! деревенской AI16-семье и здесь удалён. Часть строк семьи (`OnLoseTarget`
//! AI9/10/15/19, `Tracing`) держит статус по прежней шапке владельца без
//! построчной перечитки. Остаются hub-владением: общий monster tick, FIFO,
//! `Hibernate`, `OnMoving` с отдельным `ASA_SEARCH_ENEMY` и реальный путь
//! `monsterbaseattack`; min-distance текущего навыка вычисляет hub-caller через
//! `QueryProperty(5004)` — открытый gap G1: машина зовёт виртуальный
//! `GetAffectRangeMin` (=1 для всех классов, кроме ChuckStone-семьи),
//! hub-форма совпадает с ней только у ChuckStone;
//! docs/reconstruction/gameserver-npc-and-regions.md, строка
//! `CCityGuardWithBow::WhenBeenHurted`. Швы: [`CityGuardDispatcherPlayer`]/[`CityGuardDispatcherRegion`]
//! — фракция/союз и владелец города (проходы — общий шов `ai/lord.rs`);
//! [`GuardStationDispatcherMonster`]/[`CityGuardDispatcherMoveShape`] — состояние
//! поста и ID текущего навыка; `ForceMove` и RNG клеток — через
//! [`super::jiumai::JiuMaiDispatcherGame`] и `RegionRandomContext`.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#ai-расписаний-и-поведение

use nebokrai_shared::runtime::get_line_direction;

use crate::regions::region::RegionRandomContext;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, ShapeView};

use super::guardtarget::{
    GuardDistanceTarget, GuardStationState, consider_guard_distance_target,
    select_guard_target_groups,
};
use super::jiumai::{JiuMaiDispatcherGame, JiuMaiDispatcherMonster};
use super::lord::{EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherOwner, MonsterDispatcherPlayer, MonsterDispatcherRegion,
    move_owned_monster_to,
};

const PLAYER_TYPE: i32 = 400;

/// Игрок городской пары фильтров: переходный фасад прежнего `CPlayer`
/// (`m_lFactionID`/`m_lUnionID`).
pub trait CityGuardDispatcherPlayer: EnemySearchDispatcherPlayer {
    fn faction_id(&self) -> i32;

    fn union_id(&self) -> i32;
}

/// Регион-хозяин городской пары фильтров: переходный фасад прежнего
/// `CServerRegion` (владелец города по faction/union слотам).
pub trait CityGuardDispatcherRegion: EnemySearchDispatcherRegion {
    fn owned_city_faction(&self) -> i32;

    fn owned_city_union(&self) -> i32;
}

/// Монстр-охранник поста: переходный фасад прежнего `CMonster` с состоянием
/// точки поста и hurt/направление-фасадами общей семьи.
pub trait GuardStationDispatcherMonster: JiuMaiDispatcherMonster {
    fn guard_station_ai_mut(&mut self) -> Option<&mut GuardStationState>;
}

/// Подвижная форма городского охранника: переходный фасад прежнего
/// `CMoveShape` (ID текущего навыка для hurt-поиска лучника).
pub trait CityGuardDispatcherMoveShape: MonsterDispatcherMoveShape {
    fn current_skill_id(&self) -> Option<u32>;
}

fn belongs_to_city_owner(
    faction_id: i32,
    union_id: i32,
    owner_faction_id: i32,
    owner_union_id: i32,
) -> bool {
    (faction_id != 0 && faction_id == owner_faction_id)
        || (union_id != 0 && union_id == owner_union_id)
}

/// Выбирает цель AI10/AI11 отдельными исходными проходами игроков и
/// питомцев (`SearchEnemyGuildMember`/`SearchEnemyGuildPet`). Совпадение
/// ненулевой фракции либо союза с владельцем города исключает живого игрока
/// и принадлежащих ему существ; owner без разрешимого локального игрока
/// остаётся допустимой целью.
pub fn select_city_guard_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget>
where
    Game: MonsterDispatcherGame,
    Game::Player: CityGuardDispatcherPlayer,
    Region: CityGuardDispatcherRegion,
{
    let owner_faction_id = region.owned_city_faction();
    let owner_union_id = region.owned_city_union();
    let mut selected_player = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.region_id())
            || player.is_dead()
            || belongs_to_city_owner(
                player.faction_id(),
                player.union_id(),
                owner_faction_id,
                owner_union_id,
            )
        {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected_player = consider_guard_distance_target(
            selected_player,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
        );
    }

    let mut selected_pet = None;
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((candidate, master)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !crate::regions::moveshape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, pet.master_info()))
            })
        else {
            continue;
        };
        let protected_by_owner = (master.master_type == PLAYER_TYPE)
            .then(|| game.find_player(master.master_id))
            .flatten()
            .is_some_and(|player| {
                belongs_to_city_owner(
                    player.faction_id(),
                    player.union_id(),
                    owner_faction_id,
                    owner_union_id,
                )
            });
        if protected_by_owner {
            continue;
        }
        selected_pet = consider_guard_distance_target(
            selected_pet,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    select_guard_target_groups(selected_player, selected_pet)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CitySwordTraceOutcome {
    Ready,
    Handled,
}

/// Ветка `OnSearchEnemy` с уже имеющейся целью; завершение FIFO остаётся
/// caller-у. Свободный пост (`m_lX/m_lY == −1`) отключает проверку.
pub fn check_guard_station_target<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    chase_range: i32,
    runtime: &mut Runtime,
) where
    Game: JiuMaiDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: GuardStationDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
    Runtime: RegionRandomContext,
{
    let outside = region.find_monster_by_id_mut(monster_id).is_some_and(|monster| {
        let station = GuardStationDispatcherMonster::guard_station_ai_mut(monster)
            .and_then(|state| state.station());
        station.is_some_and(|station| {
            monster.move_shape().shape().real_distance_to_point(station.x, station.y) > chase_range
        })
    });
    if outside {
        release_guard_sword_target(game, region, monster_id, runtime);
    }
}

/// Выполняет виртуальный `OnLoseTarget` AI9/AI10/AI15/AI19: цель очищается,
/// владелец возвращается к сохранённому посту, а заблокированная клетка
/// заменяется одним `GetRandomPosInRange` на квадрате 3×3. Следующее событие
/// расписания намеренно остаётся вызывающей стороне.
pub fn release_guard_sword_target<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    runtime: &mut Runtime,
) where
    Game: JiuMaiDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: GuardStationDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
    Runtime: RegionRandomContext,
{
    let station = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        monster.release_ai_target_for_death();
        GuardStationDispatcherMonster::guard_station_ai_mut(monster)?.station()
    });
    if let Some(station) = station {
        let (destination_x, destination_y) = if region
            .base_region()
            .get_block(station.x, station.y)
            .unwrap_or(2)
            != 0
        {
            region
                .base_region()
                .get_random_pos_in_range(
                    station.x.wrapping_sub(1),
                    station.y.wrapping_sub(1),
                    3,
                    3,
                    runtime,
                )
                .map(|position| (position.x, position.y))
                .unwrap_or((station.x, station.y))
        } else {
            (station.x, station.y)
        };
        let _ = game.force_move_owned_monster(
            region,
            monster_id,
            destination_x,
            destination_y,
            0,
        );
    }
}

/// Выполняет `OnLoseTarget` мечевого охранника перед внешним
/// `ASA_SEARCH_ENEMY` из его schedule-owner-а.
pub fn lose_guard_sword_target<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) where
    Game: JiuMaiDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: GuardStationDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
    Runtime: RegionRandomContext,
{
    release_guard_sword_target(game, region, monster_id, runtime);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_search_enemy(now_milliseconds());
    }
}

/// Выполняет общую ветвь `Tracing` AI10 и производных AI15/AI19. Слишком
/// близкая цель вызывает один шаг назад, далёкая в пределах преследования —
/// исходный `ForceMove` в случайную клетку вокруг неё, а выход за
/// `chase_range` сбрасывает цель.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет отдельные пределы навыка и преследования")]
pub fn trace_city_sword_target<Game, Region, Runtime>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    owner: ShapeView,
    target: ShapeView,
    minimum_distance: i32,
    maximum_distance: i32,
    chase_range: i32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> CitySwordTraceOutcome
where
    Game: JiuMaiDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: GuardStationDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
    Runtime: RegionRandomContext,
{
    let distance = owner.real_distance(Some(target));
    let (target_x, target_y) = (target.tile_x, target.tile_y);
    if minimum_distance <= distance && distance <= maximum_distance {
        return CitySwordTraceOutcome::Ready;
    }
    if distance > chase_range {
        lose_guard_sword_target(game, region, monster_id, runtime, now_milliseconds);
        return CitySwordTraceOutcome::Handled;
    }
    if distance > maximum_distance {
        if let Ok(destination) = region.base_region().get_random_pos_in_range(
            target_x.wrapping_sub(1),
            target_y.wrapping_sub(1),
            3,
            3,
            runtime,
        ) {
            let _ = game.force_move_owned_monster(
                region,
                monster_id,
                destination.x,
                destination.y,
                0,
            );
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_active_ai_move(0, now_milliseconds());
        }
        return CitySwordTraceOutcome::Handled;
    }

    let direction = get_line_direction(target_x, target_y, owner.tile_x, owner.tile_y);
    let origin = ShapeAreaCoordinates {
        x: owner.tile_x,
        y: owner.tile_y,
    };
    if let Ok(destination) =
        CShape::get_direction_position(direction, origin)
    {
        move_owned_monster_to(game, region, monster_id, destination, 0, now_milliseconds);
    }
    CitySwordTraceOutcome::Handled
}
