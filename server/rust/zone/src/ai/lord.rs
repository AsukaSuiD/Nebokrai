//! ИИ владыки `CLord` (AI100): hurt-план бегового отвода от призванной формы,
//! общий enemy-проход ближайшего игрока/питомца и фазовый выбор боевого
//! навыка по доле HP.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\lord.cpp`.
//! Свидетельства машинной базы зафиксированы разведкой линий D/E и прежней
//! шапкой `appserver/ai/lord.rs`; тела перенесены буквально:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `SelectAttackSkill`: один RNG-бросок caller-а, HP сохраняется как `f32`, деление и сравнения на x87-подобной ширине; `[20%, 50%)` → `TRUNC(roll * float(0.6666667))`, `< 20%` → `roll / 2`; исключённые ID `1`/`2` продолжают копить `odds` | VA `0x0060AF60` | [`select_lord_attack_skill`] | `MATCH` |
//! | `WhenBeenHurted`: общая Defense-ветвь → скан квадрата младшего байта `figure` в порядке `x -> y` через `CServerRegion::GetShape`, первая `CSummonShape` → направленный беговой отход; атакующий принимается только при пустой прежней цели | VA `0x0060B0B0` | [`LordHurtPlan`], [`plan_lord_hurt_response`], [`apply_lord_hurt_response`] | `MATCH` |
//! | `OnSearchEnemy` AI100: игроки перед питомцами, равная дистанция заменяет предыдущую запись | подтверждён прежней шапкой владельца | [`select_lord_enemy`] через [`select_nearest_player_or_pet`] | `MATCH` |
//!
//! Вызов `0x0060B259` идёт через направленный MoveTo (`0x004C7CB0`) в общий
//! координатный MoveTo (`0x004C9020`): два Slip с исходным направлением,
//! Move(run=1), затем отдельное событие Move перед HasTarget/SetTarget. Отказ
//! любого Slip не публикует частичный шаг; общий обработчик сохраняет формулу
//! задержки и свежий timestamp после spatial-вызова. Этот общий MoveTo-контракт
//! — дом `ai/monsterai.rs`, здесь он только вызывается.
//!
//! Граница порции E1 (не расхождения): тела общего monster tick hub — `Run`
//! (VA `0x0060E250` → `CMonsterAI::Run` RVA `0x0C7D10`), `OnSchedule`
//! (VA `0x0060AF50` → thunk общего расписания), `OnIdle`, `OnMoving`
//! `CMonsterAI`/`CBossBlue`/`CBossFiend`/`CLord` и `CRage::End` (RVA
//! `0x59F790`) — остаются hub-владением и этой волной не затрагиваются. Там
//! же — реальный путь `monsterbaseattack`, назначающий выбранный навык и
//! ближайшую живую цель, и исполнители `lordfastattack`/
//! `lordwiderangingattack` конкретных стадий и эффектов.
//!
//! Объявленные швы (не расхождения):
//!
//! - [`LordDispatcherMonster`] — общая Defense-ветвь `when_been_hurted` и
//!   назначение цели hub-владельца `CMonster` (state-машина AI и FIFO
//!   остаются hub-владением).
//! - [`LordDispatcherGame`] — размеры области и `GetShape` одной клетки с
//!   resolver-ом hub-владельца; `None` соответствует исходному отказу
//!   `GetShape`, клетка пропускается без результата, как и прежде.
//! - [`EnemySearchDispatcherRegion`]/[`EnemySearchDispatcherPlayer`] — общий
//!   проход кандидатов (players → pets) в исходном девяти-area порядке живого
//!   региона с фильтром живых игроков этого региона и приручённых живых
//!   питомцев; теми же швами пользуются `ai/bossblue.rs` и `ai/bossfiend.rs`
//!   этой порции. Общий guard-target слой старого пакета (`guardtarget`)
//!   переходит своей волной вместе с охранниками; до неё
//!   [`select_nearest_player_or_pet`] — единый дом этого прохода для E1.
//! - Часы каждого события читаются отдельным вызовом `now` (closure/fn от
//!   делегата старого main loop); точное значение равно
//!   `game_tick_milliseconds` (`GameClockContext::now_milliseconds`).
//! - Пространственная мутация и wire-доставка MoveTo остаются у общего ядра
//!   `ai/monsterai.rs` и hub-владельца `CGame`.

use nebokrai_shared::resources::{MonsterProperties, MonsterSkill};
use nebokrai_shared::runtime::get_line_direction;

use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, ShapeView};
use crate::skills::SUMMON_SHAPE_TYPE;

use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherOwner,
    MonsterDispatcherPlayer, MonsterDispatcherRegion, move_owned_monster_to,
};

const EXCLUDED_BASE_ATTACK_SKILL_ID: u16 = 1;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

/// Игрок в общем проходе поиска цели производных AI: переходный фасад
/// прежнего `CPlayer` (признак смерти остаётся чтением базовых свойств
/// hub-владельца).
pub trait EnemySearchDispatcherPlayer: MonsterDispatcherPlayer {
    fn is_dead(&self) -> bool;
}

/// Регион общего прохода поиска цели: перечисление членов hub-региона
/// вокруг области в исходном девяти-area порядке обхода.
pub trait EnemySearchDispatcherRegion: MonsterDispatcherRegion {
    /// `FindAroundObject(owner, 400)`: живой девяти-area порядок,
    /// player ID — в порядке внутреннего vector-а области.
    fn player_ids_around_area(&self, area_index: usize) -> Vec<i32>;

    /// Pet-категория того же девяти-area порядка hub-региона.
    fn pet_ids_around_area(&self, area_index: usize) -> Vec<i32>;
}

/// Монстр-владыка: переходный фасад прежнего `CMonster`. Общая Defense-ветвь
/// и цель остаются state-машиной hub-владельца.
pub trait LordDispatcherMonster: MonsterDispatcherMonster {
    /// Общая Defense-ветвь `CMonsterAI::WhenBeenHurted` hub-владельца.
    fn when_been_hurted(&mut self, now_ms: u32);
}

/// Владелец пространственного скана владыки: переходный фасад прежнего
/// `CGame`. Сам figure-скан и выбор первой призванной формы — тело порции
/// выше; фасад открывает только `GetShape` одной клетки и размеры области.
pub trait LordDispatcherGame: MonsterDispatcherGame {
    /// Общие размеры области `CGame` для figure-скана.
    fn area_dimensions(&self) -> (i32, i32);

    /// `CServerRegion::GetShape` с resolver-ом hub-владельца: identities живых
    /// форм одной клетки в исходном порядке; `None` — отказ `GetShape`
    /// (искомая клетка пропускается без результата, как и прежде).
    fn shape_identities_at_cell(
        &self,
        region: &<Self::RegionOwner as MonsterDispatcherOwner>::Region,
        tile_x: i32,
        tile_y: i32,
        area_width: i32,
        area_height: i32,
    ) -> Option<Vec<ShapeIdentity>>;
}

/// Неизменяющая пространственная часть `WhenBeenHurted` владыки, вычисленная
/// до временного изъятия region-owner-а из `CGame`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LordHurtPlan {
    avoidance_step: Option<ShapeAreaCoordinates>,
}

/// Вычисляет пространственную часть `WhenBeenHurted` (VA `0x0060B0B0`):
/// в квадрате младшего байта `figure` в порядке `x -> y` ищется первая
/// `CSummonShape`, от её клетки задаётся направленный отход; caller владеет
/// регионом и разрешает его до вызова.
pub fn plan_lord_hurt_response<Game, Region>(
    game: &Game,
    region: &Region,
    monster_id: i32,
    property: &MonsterProperties,
) -> LordHurtPlan
where
    Game: LordDispatcherGame,
    Region: MonsterDispatcherRegion,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some(owner) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.shape_view(property))
    else {
        return LordHurtPlan::default();
    };
    let radius = i32::from(property.figure as u8);
    let end_x = owner.tile_x.wrapping_add(radius);
    let end_y = owner.tile_y.wrapping_add(radius);
    let (area_width, area_height) = game.area_dimensions();

    let mut x = owner.tile_x.wrapping_sub(radius);
    while x < end_x {
        let mut y = owner.tile_y.wrapping_sub(radius);
        while y < end_y {
            if let Some(shapes) =
                game.shape_identities_at_cell(region, x, y, area_width, area_height)
                && shapes
                    .iter()
                    .any(|identity| identity.object_type == SUMMON_SHAPE_TYPE)
            {
                let direction = get_line_direction(x, y, owner.tile_x, owner.tile_y);
                let avoidance_step = CShape::get_direction_position(
                    direction,
                    ShapeAreaCoordinates {
                        x: owner.tile_x,
                        y: owner.tile_y,
                    },
                )
                .ok();
                return LordHurtPlan { avoidance_step };
            }
            y = y.wrapping_add(1);
        }
        x = x.wrapping_add(1);
    }
    LordHurtPlan::default()
}

/// Применяет упорядоченную часть `WhenBeenHurted`: общая Defense-ветвь, затем
/// беговой MoveTo и назначение атакующего только при всё ещё пустой текущей
/// цели.
pub fn apply_lord_hurt_response<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    attacker: ShapeIdentity,
    mut now: impl FnMut() -> u32,
    plan: LordHurtPlan,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: LordDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    monster.when_been_hurted(now());

    if let Some(destination) = plan.avoidance_step {
        move_owned_monster_to(game, region, monster_id, destination, 1, &mut now);
    }

    if let Some(monster) = region.find_monster_by_id_mut(monster_id)
        && monster.ai_target().is_none()
    {
        monster.set_ai_target(attacker);
    }
    true
}

/// Выбирает ближайшую живую цель общим проходом игроков, затем питомцев.
/// Равная дистанция заменяет предыдущую запись, поэтому порядок индексов и
/// категорий остаётся частью результата. Игрок допускается живым и членом
/// этого региона, питомец — приручённым и живым.
pub fn select_nearest_player_or_pet<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    let mut selected: Option<(ShapeIdentity, i32)> = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.region_id()) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        let distance = owner.real_distance(Some(candidate));
        if distance <= guard_range
            && selected.is_none_or(|(_, current)| distance <= current)
        {
            selected = Some((candidate.identity, distance));
        }
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(candidate) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                pet.shape_view(property)
            })
        else {
            continue;
        };
        let distance = owner.real_distance(Some(candidate));
        if distance <= guard_range
            && selected.is_none_or(|(_, current)| distance <= current)
        {
            selected = Some((candidate.identity, distance));
        }
    }
    selected.map(|(identity, _)| identity)
}

/// Выполняет подтверждённый `OnSearchEnemy` AI100 через общий nearest-проход,
/// сохраняя игроков перед питомцами и замену при равной дистанции.
pub fn select_lord_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    select_nearest_player_or_pet(game, region, owner, area_index, guard_range)
}

/// `SelectAttackSkill` (VA `0x0060AF60`): единственный исходный бросок и
/// зависимое от HP сжатие его шкалы: в диапазоне `[20%, 50%)` применяется
/// `TRUNC(roll * float(0.6666667))`, ниже 20% — целочисленное деление на два.
/// Исключённые ID продолжают накапливать `odds`.
pub fn select_lord_attack_skill(
    hit_points: u32,
    maximum_hit_points: u32,
    skills: &[MonsterSkill],
    roll: i32,
    default_skill_id: u16,
) -> u16 {
    let stored_hit_points = hit_points as f32;
    let health_rate = f64::from(stored_hit_points) / f64::from(maximum_hit_points);
    let adjusted_roll = if health_rate >= f64::from(0.2_f32) {
        if health_rate < f64::from(0.5_f32) {
            (f64::from(roll) * f64::from(0.666_666_7_f32)).trunc() as i32
        } else {
            roll
        }
    } else {
        roll / 2
    };

    let mut cumulative_odds = 0_i32;
    for skill in skills {
        cumulative_odds = cumulative_odds.wrapping_add(i32::from(skill.odds));
        if !matches!(
            skill.id,
            EXCLUDED_BASE_ATTACK_SKILL_ID | EXCLUDED_ARCHERY_SKILL_ID
        ) && adjusted_roll <= cumulative_odds
        {
            return skill.id;
        }
    }
    default_skill_id
}
