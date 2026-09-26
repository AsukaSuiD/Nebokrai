//! ИИ умного гладиатора `CSmartGladiator` (AI2): очередь шагов отхода,
//! выбор уязвимой цели (HP < 40%) среди игроков и питомцев, hurt-отход от
//! ближайшей угрозы и готовый-idle шлюз гибернации.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\smartgladiator.cpp`.
//! Машинная сверка: разобраны все пять функций класса:
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | ctor: `m_qTarget` — пустой `std::deque<tagCell>` (`[+0x7C..+0x8F]`, `_Mysize = [+0x8C] = 0`) | VA `0x00610850` | [`SmartGladiatorState`] (`Default`) | `MATCH` |
//! | `OnIdle`: базовый `CMonsterAI::OnIdle` (`0x005DCC60`) только при живом owner и `[+0x8C] == 0` — очередь шагов исчерпана; тот же признак `≡ smart_gladiator_ready_to_idle` гейта гибернации | VA `0x00610660` | [`SmartGladiatorState::has_queued_steps`], потребитель параметра — `ai/monsterai.rs::hibernates_without_nearby_players` | `MATCH` |
//! | `OnSchedule`: пустые очереди `[+0x14]`/`[+0x28]`; бой по общей схеме без `GetAtcInterval`-гейта; без цели и непустом `m_qTarget` — `front` (`tagCell{lX,lY}`) идёт в координатный `CBaseAI::MoveTo(run=0)` (`0x0061083D`), затем `pop` всегда, включая неуспешное движение | VA `0x006106E0` (входной hook `0x00485540`, безцелевая ветвь `0x006107EF`) | [`execute_smart_gladiator_retreat`] расписание — hub-оркестрация; ячейка — [`SmartGladiatorState`] | `MATCH` |
//! | `OnSearchEnemy`: игроки перед питомцами, живые внутри `GetGuardRange` (vt `+0x138`); ближайшая угроза (`<=` заменяет запись) и цель с HP vt `+0xD0`/`+0xD8` < 0.4 с минимальным абсолютным HP (замена при строго меньшем); vulnerable → virtual `SetTarget`, иначе от ближайшего — `GetLineDir(threat → owner)`, `GetDirPos` и `push_back` `tagCell{lX,lY}` | VA `0x00610AC0` | [`SmartGladiatorSelection`], [`select_smart_gladiator_enemy`], [`retreat_step_from`] | `MATCH` |
//! | `WhenBeenHurted`: базовый hurt всегда (`0x004C93E0`); тип 400 вне боя — существующий игрок при HP-части владельца < 0.75 (double-константа `0x3FE8000000000000`, vt `+0xD0`/`+0xD8`) принимается целью, иначе отход от этого игрока; игрок исчез — отход от ближайшего живого игрока, иначе **шаг к ближайшему монстру**, иначе шаг по текущему `GetDir`; тип 600 вне боя — приручённое существо или повозка принимается целью | VA `0x006103B0` | [`apply_player_hurt_response`], [`apply_monster_hurt_response`] | `MATCH` |
//! | hurt-отход/сближение: найденный ориентир пишет направление формы `CShape::SetDir` (owner vtable `+0x60`, `0x0044A1C0`) перед `GetDirPos` и общим координатным `MoveTo(run=0)` (AI vtable `+0x58`, `0x004C9020`, точка вызова `0x006105A9`) | vtable `CShape` `0x0064EA04`, RVA-место `0x00610561`/`0x006105A9` | [`apply_player_hurt_response`] | `MATCH`; две ветки прежнего hub расходились: шаг к ближайшему монстру прежний hub разворачивал отходом, а запись `SetDir` не выполнял (расхождения устранены) |
//!
//! `GetDirPos` (`0x0045B330`) безотказен для восьми направлений; входной
//! state-вопрос `0x0047B150` собственных `OnSearchEnemy` — `RET1`-эквивалент.
//!
//! Остаются hub-владением: общий monster tick hub, `Hibernate` (vtable
//! `+0x64`, `0x005DCC50`), реальный путь `monsterbaseattack` (в т.ч. его
//! hurt-вход `periodicattack`) и применение цели. Фактическое пространственное
//! перемещение и журналирование — через фасады `ai/monsterai.rs` и
//! [`SmartGladiatorDispatcherMonster`].
//!
//! Швы к hub-владельцам:
//!
//! - [`SmartGladiatorDispatcherMonster`] — очередь шагов на hub-владельце
//!   `CMonster`, общая hurt-ветвь, назначение цели и запись направления
//!   формы; состояние хранится hub-владельцем, тела перенесены сюда.
//! - [`SmartGladiatorDispatcherPlayer`] — текущее и максимальное HP игрока
//!   для уязвимого критерия; проходы кандидатов повторяют общий шов
//!   `ai/lord.rs` (`EnemySearchDispatcherRegion`).

use std::collections::VecDeque;

use nebokrai_shared::resources::MonsterProperties;
use nebokrai_shared::runtime::get_line_direction;

use crate::regions::ShapeIdentity;
use crate::regions::moveshape::is_died;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, ShapeView};

use super::jiumai::JiuMaiDispatcherMonster;
use super::lord::{EnemySearchDispatcherPlayer, EnemySearchDispatcherRegion};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherOwner, MonsterDispatcherPlayer, MonsterDispatcherRegion,
    move_owned_monster_to,
};

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

/// Каноническое состояние `CSmartGladiator::m_qTarget`: очередь координатных
/// шагов отхода (`tagCell{lX, lY}`). Хранилище перенесено целиком; владелец
/// экземпляра — переходный `CMonster` старого пакета.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmartGladiatorState {
    queued_steps: VecDeque<ShapeAreaCoordinates>,
}

impl SmartGladiatorState {
    pub fn queue_step(&mut self, destination: ShapeAreaCoordinates) {
        self.queued_steps.push_back(destination);
    }

    pub fn take_step(&mut self) -> Option<ShapeAreaCoordinates> {
        self.queued_steps.pop_front()
    }

    pub fn first_step(&self) -> Option<ShapeAreaCoordinates> {
        self.queued_steps.front().copied()
    }

    /// Пустая очередь ≡ машинное `[+0x8C] == 0`: готовый-idle шлюз.
    pub fn has_queued_steps(&self) -> bool {
        !self.queued_steps.is_empty()
    }

    pub fn clear(&mut self) {
        self.queued_steps.clear();
    }
}

/// Монстр-гладиатор: переходный фасад прежнего `CMonster`. Очередь шагов,
/// hurt-ветвь и направление формы — через общий фасад пары
/// [`super::jiumai::JiuMaiDispatcherMonster`]; здесь добавлен только доступ
/// к состоянию AI2.
pub trait SmartGladiatorDispatcherMonster: JiuMaiDispatcherMonster {
    fn smart_gladiator_ai(&self) -> Option<&SmartGladiatorState>;

    fn smart_gladiator_ai_mut(&mut self) -> Option<&mut SmartGladiatorState>;

    /// Максимальное HP по свойству (vt `+0xD8`-эквивалент питомца).
    fn maximum_hit_points(&self, property: &MonsterProperties) -> u32;
}

/// Игрок-кандидат уязвимого критерия AI2: переходный фасад прежнего `CPlayer`.
pub trait SmartGladiatorDispatcherPlayer: EnemySearchDispatcherPlayer {
    /// Текущее HP (`GetHitPoint`, vt `+0xD0`).
    fn hit_points(&self) -> u32;

    /// Максимальное HP (`GetMaxHitPoint`, vt `+0xD8`).
    fn maximum_hit_points(&self) -> u32;
}

/// Один шаг schedule-фазы AI2: обрабатываемая запись очереди всегда
/// потребляется, даже когда пространственная попытка неуспешна.
pub fn execute_smart_gladiator_retreat<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    now: impl FnOnce() -> u32,
) -> bool
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: SmartGladiatorDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some(destination) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            if property.ai != 2
                || monster.active_primary_ai_type() != Some(2)
                || monster.ai_target().is_some()
                || !monster.primary_ai_queues_idle()
            {
                return None;
            }
            SmartGladiatorDispatcherMonster::smart_gladiator_ai(monster)?.first_step()
        })
    else {
        return false;
    };
    move_owned_monster_to(game, region, monster_id, destination, 0, now);
    if let Some(state) = region
        .find_monster_by_id_mut(monster_id)
        .and_then(SmartGladiatorDispatcherMonster::smart_gladiator_ai_mut)
    {
        let _ = state.take_step();
    }
    true
}

#[derive(Clone, Copy, Debug)]
pub struct SmartGladiatorCandidate {
    pub view: ShapeView,
    pub hit_points: u32,
    pub maximum_hit_points: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SmartGladiatorSelection {
    nearest: Option<(ShapeView, i32)>,
    vulnerable: Option<(ShapeIdentity, u32)>,
}

impl SmartGladiatorSelection {
    pub fn consider(
        mut self,
        owner: ShapeView,
        candidate: SmartGladiatorCandidate,
        guard_range: i32,
    ) -> Self {
        let distance = owner.real_distance(Some(candidate.view));
        if guard_range < distance {
            return self;
        }
        if self.nearest.is_none_or(|(_, current)| distance <= current) {
            self.nearest = Some((candidate.view, distance));
        }
        let health_ratio = candidate.hit_points as f32 / candidate.maximum_hit_points as f32;
        if health_ratio < 0.4
            && self
                .vulnerable
                .is_none_or(|(_, current)| candidate.hit_points < current)
        {
            self.vulnerable = Some((candidate.view.identity, candidate.hit_points));
        }
        self
    }

    pub const fn vulnerable_target(self) -> Option<ShapeIdentity> {
        match self.vulnerable {
            Some((identity, _)) => Some(identity),
            None => None,
        }
    }

    pub fn retreat_step(self, owner: ShapeView) -> Option<ShapeAreaCoordinates> {
        let (nearest, _) = self.nearest?;
        retreat_step_from(owner, nearest)
    }
}

/// Собирает достигнутый выбор AI2 по упорядоченным индексам игроков, затем
/// питомцев. Владелец хранит одновременно ближайшую угрозу и наиболее слабую
/// цель с уровнем здоровья ниже сорока процентов.
pub fn select_smart_gladiator_enemy<Game, Region>(
    game: &Game,
    region: &Region,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> SmartGladiatorSelection
where
    Game: MonsterDispatcherGame,
    Game::Player: SmartGladiatorDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
    Region::Monster: SmartGladiatorDispatcherMonster,
{
    let mut selection = SmartGladiatorSelection::default();
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.region_id()) || player.is_dead() {
            continue;
        }
        let Some(view) = player.shape_view() else {
            continue;
        };
        selection = selection.consider(
            owner,
            SmartGladiatorCandidate {
                view,
                hit_points: player.hit_points(),
                maximum_hit_points: player.maximum_hit_points(),
            },
            guard_range,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((view, hit_points, maximum_hit_points)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((
                    pet.shape_view(property)?,
                    pet.hit_points(),
                    SmartGladiatorDispatcherMonster::maximum_hit_points(pet, property),
                ))
            })
        else {
            continue;
        };
        selection = selection.consider(
            owner,
            SmartGladiatorCandidate {
                view,
                hit_points,
                maximum_hit_points,
            },
            guard_range,
        );
    }
    selection
}

/// Один исходный шаг от угрозы: направление строится от цели к владельцу
/// (`GetLineDir(threat → owner)` + `GetDirPos`).
pub fn retreat_step_from(
    owner: ShapeView,
    threat: ShapeView,
) -> Option<ShapeAreaCoordinates> {
    let direction = get_line_direction(
        threat.tile_x,
        threat.tile_y,
        owner.tile_x,
        owner.tile_y,
    );
    CShape::get_direction_position(
        direction,
        ShapeAreaCoordinates {
            x: owner.tile_x,
            y: owner.tile_y,
        },
    )
    .ok()
}

fn nearest_player<Game, Region>(
    game: &Game,
    region: &Region,
    area_index: usize,
    owner: ShapeView,
) -> Option<ShapeView>
where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
{
    region
        .player_ids_around_area(area_index)
        .into_iter()
        .filter_map(|player_id| {
            let player = game.find_player(player_id)?;
            (player.server_region_id() == Some(region.region_id()) && !player.is_dead())
                .then(|| player.shape_view())
                .flatten()
        })
        .fold(None, |nearest, candidate| match nearest {
            Some((_, distance))
                if distance < owner.real_distance(Some(candidate)) =>
            {
                nearest
            }
            _ => Some((candidate, owner.real_distance(Some(candidate)))),
        })
        .map(|(candidate, _)| candidate)
}

fn nearest_monster<Game, Region>(
    game: &Game,
    region: &Region,
    area_index: usize,
    owner: ShapeView,
    owner_id: i32,
) -> Option<ShapeView>
where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
{
    region
        .monster_ids_around_area(area_index)
        .into_iter()
        .filter(|candidate_id| *candidate_id != owner_id)
        .filter_map(|candidate_id| {
            let candidate = region.find_monster_by_id(candidate_id)?;
            if is_died(candidate.hit_points()) {
                return None;
            }
            let property =
                game.find_monster_property_by_origin_name(candidate.base_property_key()?)?;
            candidate.shape_view(property)
        })
        .fold(None, |nearest, candidate| match nearest {
            Some((_, distance))
                if distance < owner.real_distance(Some(candidate)) =>
            {
                nearest
            }
            _ => Some((candidate, owner.real_distance(Some(candidate)))),
        })
        .map(|(candidate, _)| candidate)
}

/// Применяет подтверждённую реакцию AI2 на удар игрока. Базовая реакция на
/// урон выполняется всегда; выбор цели и немедленный шаг выполняются только
/// когда гладиатор ещё не ведёт бой. Исчезнувший игрок уступает ближайшему
/// живому игроку, затем ближайшему монстру: машинный шаг к монстру идёт по
/// направлению `owner → monster`, а не от него. Каждый найденный ориентир
/// пишет направление формы `SetDir` до `GetDirPos` и ходового `MoveTo`.
pub fn apply_player_hurt_response<Game, Region>(
    game: &mut Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    player_id: i32,
    mut now: impl FnMut() -> u32,
) where
    Game: MonsterDispatcherGame,
    Game::Player: EnemySearchDispatcherPlayer,
    Region: EnemySearchDispatcherRegion,
    Region::Monster: SmartGladiatorDispatcherMonster,
    Game::RegionOwner: MonsterDispatcherOwner<Region = Region>,
{
    let Some((owner, health, area_index, was_fighting)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.shape_view(property)?,
                monster.hit_points(),
                monster.move_shape().shape().area_index(),
                monster.ai_target().is_some(),
            ))
        })
    else {
        return;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        JiuMaiDispatcherMonster::when_been_hurted(monster, now());
    }
    if was_fighting {
        return;
    }

    let player = game.find_player(player_id).and_then(|player| {
        (player.server_region_id() == Some(region.region_id()) && !player.is_dead())
            .then(|| player.shape_view())
            .flatten()
    });
    if player.is_some() && health as f32 / (property.maximum_hp as f32) < 0.75 {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_ai_target(ShapeIdentity {
                object_type: PLAYER_TYPE,
                id: player_id,
                ex_id: nebokrai_shared::values::CGuid::GUID_INVALID,
            });
        }
        return;
    }

    let mut step: Option<(i32, ShapeAreaCoordinates)> = player.and_then(|threat| {
        let direction = get_line_direction(
            threat.tile_x,
            threat.tile_y,
            owner.tile_x,
            owner.tile_y,
        );
        CShape::get_direction_position(
            direction,
            ShapeAreaCoordinates {
                x: owner.tile_x,
                y: owner.tile_y,
            },
        )
        .ok()
        .map(|destination| (direction, destination))
    });
    if step.is_none()
        && let Some(area_index) = area_index
    {
        if let Some(threat) = nearest_player(game, region, area_index, owner) {
            let direction = get_line_direction(
                threat.tile_x,
                threat.tile_y,
                owner.tile_x,
                owner.tile_y,
            );
            step = CShape::get_direction_position(
                direction,
                ShapeAreaCoordinates {
                    x: owner.tile_x,
                    y: owner.tile_y,
                },
            )
            .ok()
            .map(|destination| (direction, destination));
        } else if let Some(companion) = nearest_monster(game, region, area_index, owner, monster_id) {
            let direction = get_line_direction(
                owner.tile_x,
                owner.tile_y,
                companion.tile_x,
                companion.tile_y,
            );
            step = CShape::get_direction_position(
                direction,
                ShapeAreaCoordinates {
                    x: owner.tile_x,
                    y: owner.tile_y,
                },
            )
            .ok()
            .map(|destination| (direction, destination));
        }
    }
    if let Some((direction, destination)) = step {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            JiuMaiDispatcherMonster::set_shape_direction(monster, direction);
        }
        move_owned_monster_to(game, region, monster_id, destination, 0, &mut now);
    }
}

/// AI2 принимает в цель только приручённого монстра или повозку и только если
/// до удара ещё не вёл бой (`DoesCreatureBeenTamed`/`IsCarriage` атакующего).
pub fn apply_monster_hurt_response<Game, Region>(
    game: &Game,
    region: &mut Region,
    monster_id: i32,
    attacker_id: i32,
    now_ms: u32,
) where
    Game: MonsterDispatcherGame,
    Region: MonsterDispatcherRegion,
    Region::Monster: SmartGladiatorDispatcherMonster,
{
    let attacker_is_owned_creature = region
        .find_monster_by_id(attacker_id)
        .and_then(|attacker| {
            let property =
                game.find_monster_property_by_origin_name(attacker.base_property_key()?)?;
            Some(
                attacker.is_tamed()
                    || JiuMaiDispatcherMonster::is_carriage(attacker, property),
            )
        })
        .unwrap_or(false);
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return;
    };
    let was_fighting = monster.ai_target().is_some();
    JiuMaiDispatcherMonster::when_been_hurted(monster, now_ms);
    if !was_fighting && attacker_is_owned_creature {
        monster.set_ai_target(ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: attacker_id,
            ex_id: nebokrai_shared::values::CGuid::GUID_INVALID,
        });
    }
}
