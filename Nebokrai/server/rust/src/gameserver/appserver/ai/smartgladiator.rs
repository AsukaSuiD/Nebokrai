//! Умный гладиатор (AI2).
//!
//! Источник: точная пара gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/ai/smartgladiator.cpp. Владелец AI хранит очередь шагов,
//! выбирает уязвимую цель среди игроков и питомцев, отступает от ближайшей угрозы
//! и обрабатывает реакцию на урон. CGame участвует только в разрешении
//! владельцев и фактическом пространственном перемещении; наблюдаемый порядок
//! обхода, пороги здоровья и момент потребления очереди сохранены здесь.
//! OnSchedule без цели (0x006107EF) вызывает MoveTo до pop сохранённого шага;
//! даже неуспешное движение потребляет запись. CGame проводит это до background
//! и passive. OnIdle (0x00610660) при непустой очереди не вызывает базовый idle.

use std::collections::VecDeque;

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use super::baseai::one_step_move_delay_ms;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SmartGladiatorState {
    queued_steps: VecDeque<ShapeAreaCoordinates>,
}

impl SmartGladiatorState {
    pub(crate) fn queue_step(&mut self, destination: ShapeAreaCoordinates) {
        self.queued_steps.push_back(destination);
    }

    pub(crate) fn take_step(&mut self) -> Option<ShapeAreaCoordinates> {
        self.queued_steps.pop_front()
    }

    pub(crate) fn first_step(&self) -> Option<ShapeAreaCoordinates> {
        self.queued_steps.front().copied()
    }

    pub(crate) fn has_queued_steps(&self) -> bool {
        !self.queued_steps.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.queued_steps.clear();
    }
}

/// Один шаг schedule-фазы; очередь доступна callback-ам до завершения движения.
pub(crate) fn execute_smart_gladiator_retreat<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let Some((property, origin, speed, stop_frame, destination)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            if property.ai != 2 || monster.is_tamed() || monster.ai_target().is_some()
                || !monster.primary_ai_queues_idle()
            {
                return None;
            }
            Some((property.clone(), monster.shape_view(property)?,
                monster.move_shape().shape().get_speed(), monster.stop_frame(property),
                monster.smart_gladiator_ai()?.first_step()?))
        })
    else {
        return false;
    };
    if game.move_owned_monster_step(region, monster_id, destination.x, destination.y,
        CMonster::figure(&property))
        && let Some(monster) = region.find_monster_by_id_mut(monster_id)
    {
        let direction = get_line_direction(origin.tile_x, origin.tile_y, destination.x, destination.y);
        monster.begin_active_ai_move(one_step_move_delay_ms(direction, speed, stop_frame),
            runtime.now_milliseconds());
    }
    if let Some(state) = region.find_monster_by_id_mut(monster_id)
        .and_then(CMonster::smart_gladiator_ai_mut)
    {
        let _ = state.take_step();
    }
    true
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SmartGladiatorCandidate {
    pub(crate) view: ShapeView,
    pub(crate) hit_points: u32,
    pub(crate) maximum_hit_points: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SmartGladiatorSelection {
    nearest: Option<(ShapeView, i32)>,
    vulnerable: Option<(ShapeIdentity, u32)>,
}

impl SmartGladiatorSelection {
    pub(crate) fn consider(
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

    pub(crate) const fn vulnerable_target(self) -> Option<ShapeIdentity> {
        match self.vulnerable {
            Some((identity, _)) => Some(identity),
            None => None,
        }
    }

    pub(crate) fn retreat_step(self, owner: ShapeView) -> Option<ShapeAreaCoordinates> {
        let (nearest, _) = self.nearest?;
        retreat_step_from(owner, nearest)
    }
}

/// Собирает достигнутый выбор AI2 по упорядоченным индексам игроков, затем
/// питомцев. Владелец хранит одновременно ближайшую угрозу и наиболее слабую
/// цель с уровнем здоровья ниже сорока процентов.
pub(crate) fn select_smart_gladiator_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> SmartGladiatorSelection {
    let mut selection = SmartGladiatorSelection::default();
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id) || player.is_dead() {
            continue;
        }
        let Some(view) = player.shape_view() else {
            continue;
        };
        selection = selection.consider(
            owner,
            SmartGladiatorCandidate {
                view,
                hit_points: player.health(),
                maximum_hit_points: player.combat_properties().maximum_hp,
            },
            guard_range,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((view, hit_points, maximum_hit_points)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((
                    pet.shape_view(property)?,
                    pet.hit_points(),
                    pet.maximum_hp(property),
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

/// Один исходный шаг от угрозы: направление строится от цели к владельцу.
pub(crate) fn retreat_step_from(
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

fn nearest_player(
    game: &CGame,
    region: &CServerRegion,
    area_index: usize,
    owner: ShapeView,
) -> Option<ShapeView> {
    region
        .player_ids_around_area(area_index)
        .into_iter()
        .filter_map(|player_id| {
            let player = game.find_player(player_id)?;
            (player.server_region_id() == Some(region.id) && !player.is_dead())
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

fn nearest_monster(
    game: &CGame,
    region: &CServerRegion,
    area_index: usize,
    owner: ShapeView,
    owner_id: i32,
) -> Option<ShapeView> {
    region
        .monster_ids_around_area(area_index)
        .into_iter()
        .filter(|candidate_id| *candidate_id != owner_id)
        .filter_map(|candidate_id| {
            let candidate = region.find_monster_by_id(candidate_id)?;
            if CMoveShape::is_died(candidate.hit_points()) {
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
/// когда гладиатор ещё не ведёт бой.
pub(crate) fn apply_player_hurt_response(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    player_id: i32,
    now_ms: u32,
) {
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
        monster.when_been_hurted(now_ms);
    }
    if was_fighting {
        return;
    }

    let player = game.find_player(player_id).and_then(|player| {
        (player.server_region_id() == Some(region.id) && !player.is_dead())
            .then(|| player.shape_view())
            .flatten()
    });
    if player.is_some() && health as f32 / (property.maximum_hp as f32) < 0.75 {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_ai_target(ShapeIdentity {
                object_type: PLAYER_TYPE,
                id: player_id,
                ex_id: crate::public::guid::CGuid::GUID_INVALID,
            });
        }
        return;
    }

    let threat = player.or_else(|| {
        let area_index = area_index?;
        nearest_player(game, region, area_index, owner)
            .or_else(|| nearest_monster(game, region, area_index, owner, monster_id))
    });
    if let Some(destination) = threat.and_then(|threat| retreat_step_from(owner, threat)) {
        let _ = game.move_owned_monster_step(
            region,
            monster_id,
            destination.x,
            destination.y,
            CMonster::figure(property),
        );
    }
}

/// AI2 принимает в цель только приручённого монстра или повозку и только если
/// до удара ещё не вёл бой.
pub(crate) fn apply_monster_hurt_response(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    attacker_id: i32,
    now_ms: u32,
) {
    let attacker_is_owned_creature = region
        .find_monster_by_id(attacker_id)
        .and_then(|attacker| {
            let property =
                game.find_monster_property_by_origin_name(attacker.base_property_key()?)?;
            Some(attacker.is_tamed() || attacker.is_carriage(property))
        })
        .unwrap_or(false);
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return;
    };
    let was_fighting = monster.ai_target().is_some();
    monster.when_been_hurted(now_ms);
    if !was_fighting && attacker_is_owned_creature {
        monster.set_ai_target(ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: attacker_id,
            ex_id: crate::public::guid::CGuid::GUID_INVALID,
        });
    }
}
