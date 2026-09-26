//! Городские охранники `CCityGuardWithSword` (AI10) и `CCityGuardWithBow`
//! (AI11) стационарной семьи: точка поста, фильтры владельца города по
//! фракции/союзу, правило минимальной дистанции навыка и унаследованная
//! ветвь `Tracing` с сбросом за `chase_range`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithsword.cpp`
//! и соседний `cityguardwithbow.cpp`. Машинная сверка: свидетельства прежней
//! шапки владельца плюс построчно дочитанные тела этой сборки:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `OnSearchEnemy` AI10: при имеющейся цели (`HasTarget ≠ 0`) и заданном посте (`m_lX/m_lY ≠ −1`) сравнивается только `RealDistance(long,long)` до поста с `GetChaseRange` (owners vt `+0x13C`) — при превышении virtual `OnLoseTarget` (`+0x2C`); новый selector и добавочный поиск в этой ветви не исполняются | VA `0x0060E290` | [`check_guard_station_target`] | `MATCH` |
//! | `OnSearchEnemy` AI10 (без цели) и AI11 (`0x0060DB10`, с прежним virtual `OnLoseTarget` у лучника): selector — `SearchEnemyGuildMember` (vt `+0x90`) и `SearchEnemyGuildPet` (vt `+0x94`), ближайший из двух, игрок при равной дистанции. **`SearchEnemyGuildCarriage` (vt `+0x98`) эти тела не вызывают**; по статическому скану `CALL [reg+0x98]` её единственные caller-ы — `CVilCouGuardWithBow::WhenBeenHurted` (`0x0060C8F6`) и `::OnSearchEnemy` (`0x0060C972`) деревенского AI16 | VA `0x0060E290`, VA `0x0060DB10`, vtable `0x00662BCC`, скан call-сайтов `+0x98` | [`select_city_guard_enemy`] | `MATCH`; прежний hub дополнительно комбинировал проход повозок `603` — установленное расхождение hub, здесь проход удалён (он принадлежит деревенской AI16-семье) |
//! | фильтры selector-а: ненулевая `m_lFactionID` игрока == `GetFactionID` региона (region vt `+0xB0`) либо ненулевая `m_lUnionID` == `GetUnionID` (`+0xB4`) исключает игрока; хозяин-игрок с тем же совпадением защищает питомца; минимальная дистанция навыка vt `+0x70` внутри каждого прохода | VA `0x0060E350`, VA `0x0060E510` | [`select_city_guard_enemy`] | `MATCH` |
//! | живость кандидатов гарантирована фильтром `CServerRegion::FindAroundObject` (`IsDied` внутри `0x00480E70`), региональное членство — списком областей | RVA `0x00480E70` | [`select_city_guard_enemy`] (hub-критерии `is_dead`/регион) | `MATCH` |
//! | `OnLoseTarget` AI9/AI10/AI15/AI19: базовый `CMonsterAI::OnLoseTarget` (`0x005DCC30`, только цель), возврат к посту, заблокированная клетка заменяется одним `GetRandomPosInRange` на квадрате 3×3 | VA `0x0060D020` (прежний владелец) | [`release_guard_sword_target`], [`lose_guard_sword_target`] | `MATCH` (по прежней шапке владельца; тело построчно не перечитано) |
//! | `Tracing`: шаг назад от слишком близкой цели, `ForceMove` в случайную клетку 3×3 около далёкой (затем отдельный `Move(0)`), сброс за `chase_range` через virtual `OnLoseTarget` и внешний `SearchEnemy` | VA `0x0060D0E0` (прежний владелец) | [`trace_city_sword_target`] | `MATCH` (по прежней шапке владельца; тело построчно не перечитано) |
//! | стационарное `OnSchedule` `0x0020B890` — общий dispatcher семьи до Begin; `OnIdle` один раз фиксирует пост (`m_lX/m_lY`) и продолжает через общий idle FIFO | зафиксированный факт `ai/monsterai.rs`; запись поста — прежний владелец | [`GuardStationState`] (`ai/guardtarget.rs`), [`check_guard_station_target`] | `MATCH` (по зафиксированному факту) |
//!
//! Остаются hub-владением: общий monster tick hub, материализация FIFO,
//! `Hibernate`, `OnMoving` с отдельным `ASA_SEARCH_ENEMY` и реальный путь
//! `monsterbaseattack`. Минимальная дистанция текущего навыка вычисляется
//! hub-caller-ом через свойства навыка (`QueryProperty(5004)` — эквивалент
//! skill vt `+0x70`-запроса).
//!
//! Швы к hub-владельцам:
//!
//! - [`CityGuardDispatcherPlayer`]/[`CityGuardDispatcherRegion`] — фракция и
//!   союз игрока, владелец города региона; общие проходы кандидатов
//!   повторяют шов `ai/lord.rs` (`EnemySearchDispatcher*`).
//! - [`GuardStationDispatcherMonster`]/[`CityGuardDispatcherMoveShape`] —
//!   состояние поста на hub-владельце `CMonster` и ID текущего навыка формы.
//! - Мгновенный `ForceMove` и RNG случайных клеток приходят через
//!   [`super::jiumai::JiuMaiDispatcherGame`] и `RegionRandomContext` —
//!   прежние контракты hub-владельца.

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
