//! Достигнутая часть ИИ владыки `CLord`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/lord.cpp` подтверждают один RNG-бросок и
//! зависимое от доли HP сжатие его шкалы перед упорядоченным выбором навыка.
//! HP сначала сохраняется как `f32`, деление и сравнения остаются на
//! x87-подобной ширине, а масштабированный бросок усекается к нулю.
//! Этот владелец выбирает навык и ближайшую живую цель, реальный путь
//! `monsterbaseattack` назначает результат, а `lordfastattack` и
//! `lordwiderangingattack` исполняют конкретные стадии и эффекты.
//! Реакция на урон сначала сохраняет общую Defense-ветвь, затем в порядке
//! `x -> y -> CServerRegion::GetShape` ищет первую призванную форму в квадрате
//! младшего байта `figure`, задаёт беговой отход от её клетки и только при
//! отсутствии прежней цели принимает атакующего. Пространственная мутация и
//! wire-доставка остаются у `CGame`.
//! Вызов 0x0060B259 идёт через направленный MoveTo (0x004C7CB0) в общий
//! координатный MoveTo (0x004C9020): два Slip с исходным направлением,
//! Move(run=1), затем отдельное событие Move перед HasTarget/SetTarget.
//! Отказ любого Slip не публикует частичный шаг; общий обработчик сохраняет
//! формулу задержки и свежий timestamp после spatial-вызова.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\lord.cpp

// CLord::WhenBeenHurted, RVA 0x0020B0B0, материализован ниже.

// COMPONENT_VARIANT_END: GameServer

use super::guardtarget::select_nearest_player_or_pet;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView,
};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::{MonsterProperties, MonsterSkill};

const EXCLUDED_BASE_ATTACK_SKILL_ID: u16 = 1;
const EXCLUDED_ARCHERY_SKILL_ID: u16 = 2;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LordHurtPlan {
    avoidance_step: Option<ShapeAreaCoordinates>,
}

/// Вычисляет неизменяющую пространственную часть `WhenBeenHurted` до
/// временного изъятия region-owner-а из `CGame`.
pub(crate) fn plan_lord_hurt_response(
    game: &CGame,
    region_id: i32,
    monster_id: i32,
    property: &MonsterProperties,
) -> LordHurtPlan {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else {
        return LordHurtPlan::default();
    };
    plan_lord_hurt_response_in_region(game, region, monster_id, property)
}

/// Тот же неизменяющий проход для caller-а, который уже временно владеет
/// регионом и поэтому не может повторно найти его в `CGame`.
pub(crate) fn plan_lord_hurt_response_in_region(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
) -> LordHurtPlan {
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
            let mut shapes = Vec::new();
            if region
                .get_shapes(x, y, area_width, area_height, game, &mut shapes)
                .is_ok()
                && shapes
                    .iter()
                    .any(|shape| shape.identity.object_type == SUMMON_SHAPE_TYPE)
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

/// Применяет упорядоченную часть `WhenBeenHurted`: Defense, беговой MoveTo и
/// назначение атакующего только при всё ещё пустой текущей цели.
pub(crate) fn apply_lord_hurt_response(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    attacker: ShapeIdentity,
    mut now: impl FnMut() -> u32,
    plan: LordHurtPlan,
) -> bool {
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    monster.when_been_hurted(now());

    if let Some(destination) = plan.avoidance_step {
        super::monsterai::move_owned_monster_to(game, region, monster_id, destination, 1,
            &mut now);
    }

    if let Some(monster) = region.find_monster_by_id_mut(monster_id)
        && monster.ai_target().is_none()
    {
        monster.set_ai_target(attacker);
    }
    true
}

/// Выполняет подтверждённый `OnSearchEnemy` AI100 через общий nearest-проход,
/// сохраняя игроков перед питомцами и замену при равной дистанции.
pub(crate) fn select_lord_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    select_nearest_player_or_pet(game, region, owner, area_index, guard_range)
        .map(|selected| selected.identity)
}

/// Сохраняет единственный исходный бросок и зависимое от HP сжатие его шкалы:
/// в диапазоне `[20%, 50%)` применяется `TRUNC(roll * float(0.6666667))`, ниже 20% —
/// целочисленное деление на два. Исключённые ID продолжают накапливать `odds`.
pub(crate) fn select_lord_attack_skill(
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
