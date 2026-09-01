//! Общий снарядный путь монстров для навыков с прямой траекторией.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные владельцы
//! `CSkeletonArchery` и `CChuckStone`, подтверждает общий порядок подготовки,
//! проверки преград, расчёта времени полёта и удара по клетке. Конкретный
//! владелец сохраняет идентификатор навыка, а начало полёта отдельно занимает
//! attack-speed timestamp ИИ; выбор цели, защита и последствия
//! смерти остаются у существующих владельцев боя и `CGame`.
use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::schedule_attack_interval;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
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
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug)]
pub(crate) struct MonsterProjectileDispatch {
    pub(crate) monster_id: i32,
    pub(crate) skill_id: u32,
    pub(crate) impact_x: i32,
    pub(crate) impact_y: i32,
    skill_level: u16,
    properties: CSkillBaseProperties,
    property: MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    damage_factor: f32,
    now_ms: u32,
}

impl MonsterProjectileDispatch {
    /// Собирает object-target удар для навыка, который сам ведёт полёт и
    /// визуальную фазу, но использует общий monster attack tail.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn object_target(
        monster_id: i32,
        skill_id: u32,
        target_x: i32,
        target_y: i32,
        skill_level: u16,
        properties: CSkillBaseProperties,
        property: MonsterProperties,
        attacker_master: MasterInfo,
        attacker_tamed: bool,
        now_ms: u32,
    ) -> Self {
        let damage_factor = match properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) {
            0 => 1.0,
            factor => factor as f32 * 0.01,
        };
        Self {
            monster_id, skill_id, impact_x: target_x, impact_y: target_y,
            skill_level, properties, property, attacker_master, attacker_tamed,
            damage_factor, now_ms,
        }
    }
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
pub(crate) fn prepare_owned_monster_projectile(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    dispatch: &mut Option<MonsterProjectileDispatch>,
) -> bool {
    let Some((source, property, master, tamed, cast, progress)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                monster.move_shape().shape().clone(),
                property,
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.monster_projectile_progress(),
            ))
        })
    else {
        return false;
    };
    let detached_impact = progress.and_then(MonsterProjectileProgress::detached_impact);
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    if target.is_none() && detached_impact.is_none() {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    if detached_impact.is_none()
        && target
            .as_ref()
            .is_some_and(|target| target.dead || target.god || target.city_dead)
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
                let _ = monster.finish_base_attack_cast(now_ms);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    if detached_impact.is_none() && target.as_ref().is_some_and(|target| {
        !owned_monster_attackable(
            game, region.id, &property, tamed, master, target_identity, target,
        )
    }) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
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
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
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
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                skill_id,
                skill_level,
                now_ms,
            );
            monster.begin_monster_projectile_progress();
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
                monster.clear_ai_target();
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
                monster_attack_cell_candidates(game, region, monster_id, x, y)
                    .into_iter()
                    .next()
                    .and_then(|identity| {
                        let target = resolve_owned_monster_attack_target(game, region, identity)?;
                        owned_monster_attackable(
                            game, region.id, &property, tamed, master, identity, &target,
                        )
                        .then_some(())
                    })
                    .is_some()
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
                .monster_projectile_progress_mut()
                .expect("состояние полёта принадлежит текущему навыку") = progress;
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
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
        attacker_master: master,
        attacker_tamed: tamed,
        damage_factor: 1.0,
        now_ms,
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
    region: &mut CServerRegion,
    dispatch: &MonsterProjectileDispatch,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return false;
    };
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            &dispatch.property,
            dispatch.attacker_tamed,
            dispatch.attacker_master,
            identity,
            &target,
        )
    {
        return false;
    }
    let bounds = region
        .find_monster_by_id(dispatch.monster_id)
        .map(|monster| {
            if dispatch.attacker_tamed {
                let pet = monster.pet_attack_properties(&dispatch.property);
                (pet.minimum_attack, pet.maximum_attack)
            } else {
                monster.state_attack_bounds(
                    dispatch.property.minimum_attack,
                    dispatch.property.maximum_attack,
                )
            }
        })
        .unwrap_or((
            dispatch.property.minimum_attack,
            dispatch.property.maximum_attack,
        ));
    let physical_minimum = bounds.0 as i32;
    let physical_span = (bounds.1 as i32)
        .wrapping_sub(physical_minimum)
        .max(0)
        .wrapping_add(1);
    let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
    let element_minimum = dispatch.property.minimum_element as i32;
    let element_span = (dispatch.property.maximum_element as i32)
        .wrapping_sub(element_minimum)
        .max(0)
        .wrapping_add(1);
    let element = element_minimum.wrapping_add(game.skill_random_below(element_span));
    // Нулевая вероятность критического удара монстра не устраняет исходный
    // третий вызов генератора.
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
        damage_factor: dispatch.damage_factor,
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
                hp_damage: i32::from(CMonster::resource_soul_attack(&dispatch.property)),
                mp_damage: 0,
            },
        ],
    };
    let attack = defend_owned_monster_attack(
        game,
        identity,
        target.mana,
        target.war_soul_mana,
        target.player_properties,
        target.monster_properties,
        attack,
    );
    apply_owned_monster_attack_hit(
        game,
        region,
        runtime,
        dispatch.now_ms,
        dispatch.monster_id,
        dispatch.attacker_master,
        identity,
        &target.shape,
        target.health,
        target.mana,
        target.master,
        target.monster_property,
        target.tamed,
        target.carriage,
        attack,
        deaths,
    );
    true
}

pub(crate) fn finish_owned_monster_projectile(
    region: &mut CServerRegion,
    dispatch: &MonsterProjectileDispatch,
) {
    if let Some(monster) = region.find_monster_by_id_mut(dispatch.monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(dispatch.now_ms);
    }
}
