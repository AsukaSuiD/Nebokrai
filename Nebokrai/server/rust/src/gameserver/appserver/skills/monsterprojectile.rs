//! Общий снарядный путь монстров для навыков с прямой траекторией.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные владельцы
//! `CSkeletonArchery` и `CChuckStone`, подтверждает общий
//! порядок подготовки, проверки преград, расчёта времени полёта и удара по
//! клетке. Конкретный владелец сохраняет идентификатор навыка, а начало полёта
//! отдельно занимает attack-speed timestamp ИИ; выбор цели, защита и
//! последствия смерти остаются у существующих владельцев боя и `CGame`.
//! Оба исходных `CalculateAttackPower` берут elemental damage из virtual
//! `CMonster::GetAddElementAtk == 0`, поэтому ресурсный element range здесь не
//! участвует и между physical и critical roll нет дополнительного RNG.
//! Защита и попадание читают часы внутри общего OnBeenAttacked. Reuse отдельно
//! читает runtime после очистки ресурсов и освобождения движения,
//! как `CSkill::End` (0x4d84c0).
//! Cell Skeleton/Chuck (0x005392B0/0x0053DA40) не вызывает IsAttackAble перед
//! Calculate; проверка первой BLOCK_SHAPE остаётся отдельной границей полёта.
//! Полный регион сохраняет исходный RTTI всех CMoveShape и порядок клеточного снимка.
//! End `0x0056A330` и `0x0057B810` обнуляет четыре derived DWORD
//! `+0x4C/+0x50/+0x54/+0x58` до возврата движения. Общий зарегистрированный
//! End снимает фазу kernel, а этот owner сбрасывает fired и время полёта.
//! Технический снимок detached-impact сохраняется: он не является отдельным
//! native-полётным полем и после ended не применяется повторно.
//! Цепочка попадания передаёт Option владельца региона до синхронной смерти.
//! Заимствование базы не переживает эту границу; продолжение заново получает
//! оставшегося владельца, не создавая замену исчезнувшему региону.

use crate::gameserver::gameserver::game::ServerRegionOwner;

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
use super::monsterattack::{
    apply_owned_monster_attack_hit,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::schedule_attack_interval;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::public::guid::CGuid;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_MIN_DISTANCE: u32 = 5_004;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;

#[derive(Clone, Debug)]
pub(crate) struct MonsterProjectileDispatch {
    pub(crate) monster_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) impact_x: i32,
    pub(crate) impact_y: i32,
    skill_level: u16,
    properties: CSkillBaseProperties,
    property: MonsterProperties,
}

fn send_projectile_visual(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_id: u32,
    skill_level: u16,
    action: u8,
    target: Option<(i32, i32, i32, i32)>,
    timing_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    if action == 1 {
        message.add_long(source.get_direction());
        message.add_ulong(timing_ms);
    } else if let Some((target_type, target_id, x, y)) = target {
        message.add_long(target_type);
        message.add_long(target_id);
        message.add_long(x);
        message.add_long(y);
        message.add_ulong(timing_ms);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт полёта")]
pub(crate) fn prepare_owned_monster_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_owner: &mut ServerRegionOwner,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    dispatch: &mut Option<MonsterProjectileDispatch>,
    runtime: &mut Runtime,
) -> bool {
    let region = region_owner.base_mut();
    let Some((source, property, tamed, cast, progress)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                monster.move_shape().shape().clone(),
                property,
                monster.is_tamed(),
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_progress::<MonsterProjectileProgress>(skill_id, game.skill_factory()).copied(),
            ))
        })
    else {
        return false;
    };
    let detached_impact = progress.and_then(MonsterProjectileProgress::detached_impact);
    let target = resolve_owned_monster_attack_target(game, region_owner, target_identity);
    let region = region_owner.base_mut();
    if target.is_none() && detached_impact.is_none() {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }
    if detached_impact.is_none()
        && target
            .as_ref()
            .is_some_and(|target| target.dead)
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                let _ = monster.finish_base_attack_cast_with_clock(skill_id, game.skill_factory(), || runtime.now_milliseconds());
            }
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    let (target_x, target_y) = if let Some(impact) = detached_impact {
        impact
    } else {
        let Some(target) = target.as_ref() else { return true };
        let (Ok(x), Ok(y)) = (target.shape.get_tile_x(), target.shape.get_tile_y()) else {
            return true;
        };
        (x, y)
    };
    let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none()
        && maximum_distance != 0
        && path.len() > maximum_distance.wrapping_add(1) as usize
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }

    if cast.is_none() {
        let attack_interval = if tamed {
            region
                .find_monster_by_id(monster_id)
                .map(|monster| monster.pet_attack_properties(&property).attack_interval)
                .unwrap_or(property.attack_speed)
        } else {
            property.attack_speed
        };
        let schedule_ready = schedule_attack_interval(property.ai, attack_interval)
            .is_none_or(|interval| {
                region
                    .find_monster_by_id_mut(monster_id)
                    .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval))
            });
        if !schedule_ready {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        let target_object = resolve_owned_skill_begin_object(game, region, target_identity);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                skill_id,
                skill_level,
                now_ms,
                target_object,
                game.skill_factory(),
            );
            monster.set_skill_progress(skill_id, MonsterProjectileProgress::default(), game.skill_factory());
        }
        let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_projectile_visual(game, region, source, skill_id, skill_level, 1, None, delay_ms);
        return true;
    }

    let cast = cast.expect("выполнение прямого снаряда проверено выше");
    if cast.dispatch().skill_id != skill_id
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    let Some(mut progress) = progress else {
        return true;
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
        }
        let minimum_distance = properties.query_property(SKILL_USAGE_TARGET_MIN_DISTANCE);
        if (maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize)
            || (minimum_distance != 0 && path.len() < minimum_distance as usize)
        {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target(game.skill_factory());
            }
            return true;
        }
        let mut path_index = path.len();
        let mut impact_x = target_x;
        let mut impact_y = target_y;
        let mut detached = false;
        for (index, &(x, y, block)) in path.iter().enumerate() {
            let blocked = if block == BLOCK_UNFLY {
                true
            } else if block == BLOCK_SHAPE {
                let (area_width, area_height) = game.area_dimensions();
                let resolver = crate::gameserver::gameserver::game::RegionShapeResolver { game, owner: region_owner };
                region_owner.base().get_shape(x, y, area_width, area_height, &resolver)
                    .ok().flatten()
                    .filter(|target| matches!(target.identity.object_type, 400 | 500 | 600 | 1100 | 1200))
                    .is_some_and(|target| game.live_skill_target_attackable_in(
                        region_owner,
                        ShapeIdentity { object_type: MONSTER_TYPE, id: monster_id, ex_id: CGuid::GUID_INVALID },
                        target.identity,
                    ))
            } else {
                false
            };
            if blocked {
                path_index = index;
                impact_x = x;
                impact_y = y;
                detached = true;
                break;
            }
        }
        let region = region_owner.base_mut();
        let missile_flying_time_ms = properties
            .query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path_index as u32);
        send_projectile_visual(
            game,
            region,
            &source,
            skill_id,
            skill_level,
            2,
            Some((
                if detached { 0 } else { target_identity.object_type },
                if detached { 0 } else { target_identity.id },
                impact_x,
                impact_y,
            )),
            missile_flying_time_ms,
        );
        progress.fire(missile_flying_time_ms, detached.then_some((impact_x, impact_y)));
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            *monster
                .skill_progress_mut::<MonsterProjectileProgress>(skill_id, game.skill_factory())
                .expect("состояние полёта принадлежит текущему навыку") = progress;
            let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        }
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        delay_ms.wrapping_add(progress.missile_flying_time_ms()),
    ) {
        return true;
    }
    *dispatch = Some(MonsterProjectileDispatch {
        monster_id,
        skill_id,
        impact_x: target_x,
        impact_y: target_y,
        skill_level,
        properties: properties.clone(),
        property,
    });
    true
}

/// Состояние полёта прямого снаряда между тактами исходного навыка.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MonsterProjectileProgress {
    missile_flying_time_ms: u32,
    fired: bool,
    detached_impact: Option<(i32, i32)>,
}

impl MonsterProjectileProgress {
    pub(crate) fn prepare_derived_end(&mut self) {
        self.missile_flying_time_ms = 0;
        self.fired = false;
    }

    pub(crate) const fn fired(self) -> bool {
        self.fired
    }

    pub(crate) const fn missile_flying_time_ms(self) -> u32 {
        self.missile_flying_time_ms
    }

    pub(crate) const fn detached_impact(self) -> Option<(i32, i32)> {
        self.detached_impact
    }

    pub(crate) fn fire(
        &mut self,
        missile_flying_time_ms: u32,
        detached_impact: Option<(i32, i32)>,
    ) {
        self.missile_flying_time_ms = missile_flying_time_ms;
        self.fired = true;
        self.detached_impact = detached_impact;
    }
}

pub(crate) fn execute_owned_monster_projectile_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    dispatch: &MonsterProjectileDispatch,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_ref() else { return false; };
    if resolve_owned_monster_attack_target(game, region_owner, identity).is_none() {
        return false;
    }
    let region = region_owner.base();
    let Some(monster) = region.find_monster_by_id(dispatch.monster_id) else { return false };
    let bounds = monster.state_attack_bounds(
        dispatch.property.minimum_attack,
        dispatch.property.maximum_attack,
    );
    let soul_attack = monster.soul_attack(&dispatch.property);
    let physical_minimum = bounds.0 as i32;
    let physical_span = (bounds.1 as i32)
        .wrapping_sub(physical_minimum)
        .max(0)
        .wrapping_add(1);
    let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
    // `CMonster::GetAddElementAtk` возвращает ноль. Нулевая вероятность
    // критического удара всё равно оставляет второй вызов генератора.
    let element = 0;
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: dispatch.skill_id,
        skill_level: dispatch.skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: dispatch.monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: dispatch
            .properties
            .query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical.max(0),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: element.max(0),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(soul_attack),
                mp_damage: 0,
            },
        ],
    };
    apply_owned_monster_attack_hit(game, owner, runtime, identity, attack);
    true
}
