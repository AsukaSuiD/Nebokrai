//! Рабочий владелец исполнения `CKnockOut` (`0x192`) с объектом-целью.
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/knockout.cpp` и `knockoutstate.cpp`. Контактные атаки
//! player- и monster-ветвей сохраняют default skill-id `0x7fffffff` из
//! конструктора `tagAttackInformation` и уровень `1`.
//! Формулы, последовательность случайных чисел, проверки, время восстановления
//! и сетевой формат принадлежат навыку. Player- и monster-owner-ы подключены
//! к своим реальным AI/runtime путям; `CGame` остаётся координатором общей
//! защиты, жизненного цикла цели и доставки.
//! Шанс попадания отдельно сохраняет level-компонент в `f32`, затем складывает
//! его с расширенным x87 hit-компонентом и усекает результат к нулю.
//! Критический урон также усекается при исходной записи в `i32`. Player и
//! monster ветви используют абсолютный срок `CSkill::IsRestored`; задержка
//! контакта и длительность состояния остаются elapsed.

use super::baseattack::time_reached;
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::knockoutstate::{
    KnockOutState, replace_monster_knock_out_state, replace_player_knock_out_state,
};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::finish_state_skill;
use crate::gameserver::appserver::ai::monsterai::schedule_attack_interval;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

pub(crate) const KNOCK_OUT_SKILL_ID: u32 = 0x192;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const CURE_SKILL_ID: u32 = 0x131;
const DEFAULT_CONTACT_SKILL_ID: u32 = 0x7fff_ffff;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;

fn knock_out_hit_chance(
    source_hit: u16,
    target_dodge: u16,
    source_level: u8,
    target_level: u8,
    base: i32,
    magnify: f32,
    level_rate: f32,
) -> i32 {
    let level_component = (f64::from(
        i32::from(source_level).wrapping_sub(i32::from(target_level)),
    ) * f64::from(level_rate)
        + f64::from(base)) as f32;
    truncate_original(
        f64::from(i32::from(source_hit).wrapping_sub(i32::from(target_dodge)))
            * f64::from(magnify)
            + f64::from(level_component),
    )
}
const CAN_BREAK: u32 = 10_006;

#[derive(Clone, Copy)]
struct Target { identity: ShapeIdentity, x: i32, y: i32, level: u8, dodge: u16, dead: bool, cured: bool }

fn result(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<Target> {
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            (player.server_region_id() == Some(region_id)).then_some(Target {
                identity, x: player.shape().get_tile_x().ok()?, y: player.shape().get_tile_y().ok()?,
                level: player.level(), dodge: player.combat_properties().dodge,
                dead: player.is_dead(), cured: player.has_state_by_skill_id(CURE_SKILL_ID),
            })
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(Target {
                identity, x: monster.move_shape().shape().get_tile_x().ok()?, y: monster.move_shape().shape().get_tile_y().ok()?,
                level: property.level as u8, dodge: monster.dodge(property), dead: monster.hit_points() == 0,
                cured: monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID),
            })
        }
        _ => None,
    }
}

fn failure(game: &CGame, player_id: i32, action: u8) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(0); message.add_byte(action);
    let _ = message.send_to_player(game.net_server(), player_id);
}

fn cast(game: &mut CGame, player_id: i32, target: Target, level: i32, fire: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let source = player.shape();
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if fire { 2 } else { 1 });
    message.add_long(KNOCK_OUT_SKILL_ID as i32); message.add_short(level as i16);
    message.add_long(source.identity().object_type); message.add_long(source.identity().id);
    if fire {
        message.add_long(target.identity.object_type); message.add_long(target.identity.id);
        message.add_long(target.x); message.add_long(target.y);
    } else { message.add_long(source.get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn monster_cast(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    level: u16,
    fire: bool,
) {
    let Some(monster) = region.find_monster_by_id(monster_id) else { return };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if fire { 2 } else { 1 });
    message.add_long(KNOCK_OUT_SKILL_ID as i32); message.add_short(level as i16);
    message.add_long(MONSTER_TYPE); message.add_long(monster_id);
    if fire {
        message.add_long(target.object_type); message.add_long(target.id);
        message.add_long(target_x); message.add_long(target_y);
    } else { message.add_long(monster.move_shape().shape().get_direction()); }
    let _ = game.send_game_shape_around(region, monster.move_shape().shape(), None, &message);
}

fn master(player: &CPlayer) -> MasterInfo {
    let p = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(),
        master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate),
        permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal),
    }
}

fn attack(game: &mut CGame, player_id: i32, target_level: u8) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master(player);
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let factor = player.weapon_modifier(
        game.goods_factory(),
        i32::from(target_level),
        divisor,
        minimum_factor,
    );
    let difference = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32);
    let span = if difference < 0 { difference.wrapping_neg() } else { difference }.wrapping_add(1);
    let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(span)).max(0);
    let mut value = AttackInformation {
        // `CKnockOut::Attack` не переписывает значения конструктора
        // `tagAttackInformation`: ID остаётся `0x7fffffff`, а уровень — 1.
        skill_id: DEFAULT_CONTACT_SKILL_ID, skill_level: 1, attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier: 100, damage_factor: factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: combat.add_element_attack as i32, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        value.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut value.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); }
    }
    Some((master, value))
}

fn monster_attack(
    game: &mut CGame,
    region: &CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
) -> Option<(MasterInfo, AttackInformation)> {
    let monster = region.find_monster_by_id(monster_id)?;
    let master = monster.master_info();
    let (minimum, maximum) = if monster.is_tamed() {
        let attack = monster.pet_attack_properties(property);
        (attack.minimum_attack, attack.maximum_attack)
    } else {
        let clamp = |value: u32| value.clamp(1, i32::MAX as u32);
        monster.state_attack_bounds(clamp(property.minimum_attack), clamp(property.maximum_attack))
    };
    let minimum = minimum as i32;
    let difference = (maximum as i32).wrapping_sub(minimum);
    let span = if difference < 0 { difference.wrapping_neg() } else { difference }.wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let mut attack = AttackInformation {
        // Конструкторное значение сохраняется и для non-player owner-а.
        skill_id: DEFAULT_CONTACT_SKILL_ID, skill_level: 1,
        attacker_type: MONSTER_TYPE, attacker_id: monster_id,
        attacker_team_id: 0, attacker_faction_id: 0, attacker_union_id: 0,
        hit_modifier: 100, damage_factor: 1.0, damage_modifier: 0,
        critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            // Виртуальный `CMonster::GetAddElementAtk` обычного monster-owner-а
            // возвращает ноль; GetAddSoulAtk оставляет младшие 16 бит только
            // положительного `dwYaoAtk`.
            AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(CMonster::resource_soul_attack(property)), mp_damage: 0 },
        ],
    };
    // `CMoveShape::GetCCH` для монстра равен нулю, но исходный owner всё
    // равно расходует отдельный critical RNG после physical RNG.
    let critical_roll = game.skill_random_below(100);
    let critical_chance = 0;
    if critical_roll < critical_chance {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); }
    }
    Some((master, attack))
}

fn owned_target_has_cure(game: &CGame, region: &CServerRegion, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_some_and(|player| player.has_state_by_skill_id(CURE_SKILL_ID)),
        MONSTER_TYPE => region.find_monster_by_id(target.id).is_some_and(|monster| monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID)),
        _ => true,
    }
}

/// Generic object-target ветвь `CKnockOut::AI` для monster-owner-а: общий
/// reuse/range lifecycle, блокировка движения до delay, отдельный hit RNG,
/// затем concrete attack, replacement состояния и monster death tail.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_owned_monster_knock_out<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    property: &MonsterProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((source, master, tamed, cast)) = region.find_monster_by_id(monster_id).map(|monster| (monster.move_shape().shape().clone(), monster.master_info(), monster.is_tamed(), monster.current_active_attack_cast(game.skill_factory()))) else { return false };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    };
    if target.dead || target.god || target.city_dead || !owned_monster_attackable(game, region.id, property, tamed, master, target_identity, &target) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (source.get_tile_x(), source.get_tile_y(), target.shape.get_tile_x(), target.shape.get_tile_y()) else { return true };
    let path_len = region.straight_skill_path(source_x, source_y, target_x, target_y, None).len() as u32;
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none() {
        let interval = if tamed { region.find_monster_by_id(monster_id).map(|monster| monster.pet_attack_properties(property).attack_interval).unwrap_or(property.attack_speed) } else { property.attack_speed };
        if schedule_attack_interval(property.ai, interval).is_some_and(|interval| region.find_monster_by_id_mut(monster_id).is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))) { return true; }
        let reuse = properties.query_property(REUSE);
        let last_used = region.find_monster_by_id(monster_id).map(|monster| monster.skill_last_used_ms(KNOCK_OUT_SKILL_ID, game.skill_factory())).unwrap_or_default();
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used, reuse, now_ms,
            )
        {
            return true;
        }
        if maximum != 0 && maximum.wrapping_add(1) < path_len {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, KNOCK_OUT_SKILL_ID, skill_level, now_ms, game.skill_factory());
        }
        monster_cast(game, region, monster_id, target_identity, target_x, target_y, skill_level, false);
        return true;
    }
    let cast = cast.expect("активное оглушение проверено выше");
    if cast.dispatch().skill_id != KNOCK_OUT_SKILL_ID || cast.dispatch().target != target_identity { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(DELAY)) { return true; }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.move_shape_mut().set_moveable(true); }
    if maximum != 0 && maximum.wrapping_add(1) < path_len {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
        return true;
    }
    monster_cast(game, region, monster_id, target_identity, target_x, target_y, skill_level, true);
    let target_level = if target_identity.object_type == PLAYER_TYPE {
        game.find_player(target_identity.id).map_or(0, CPlayer::level)
    } else {
        target.monster_property.as_ref().map_or(0, |property| property.level as u8)
    };
    let target_dodge = if target_identity.object_type == PLAYER_TYPE {
        target.player_properties.map_or(0, |combat| combat.dodge)
    } else {
        region.find_monster_by_id(target_identity.id).zip(target.monster_property.as_ref()).map_or(0, |(monster, property)| monster.dodge(property))
    };
    let source_hit = region.find_monster_by_id(monster_id).map_or(1, |monster| monster.hit(property));
    let source_level = property.level as u8;
    let (base, magnify, level_rate) = game.globe_setup().base_attack_hit_formula();
    let chance = knock_out_hit_chance(
        source_hit,
        target_dodge,
        source_level,
        target_level,
        base,
        magnify,
        level_rate,
    );
    if chance <= game.skill_random_below(100) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { let _ = monster.finish_base_attack_cast_without_reuse(KNOCK_OUT_SKILL_ID, game.skill_factory()); }
        return true;
    }
    let Some((attacker_master, attack)) = monster_attack(game, region, monster_id, property) else { return true };
    let attack = defend_owned_monster_attack(game, target_identity, target.mana, target.war_soul_mana, target.player_properties, target.monster_properties, attack);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(KNOCK_OUT_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(KNOCK_OUT_SKILL_ID, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
    }
    apply_owned_monster_attack_hit(game, region, runtime, now_ms, monster_id, attacker_master, target_identity, &target.shape, target.health, target.mana, target.master, target.monster_property, target.tamed, target.carriage, attack, deaths);
    if !owned_target_has_cure(game, region, target_identity) {
        let state_now = runtime.now_milliseconds();
        let state = KnockOutState::new(state_now, properties.query_property(PERSIST));
        match target_identity.object_type {
            PLAYER_TYPE => { let _ = replace_player_knock_out_state(game, target_identity.id, state, state_now); }
            MONSTER_TYPE => { let _ = replace_monster_knock_out_state(game, region, target_identity.id, state, state_now); }
            _ => {}
        }
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(KNOCK_OUT_SKILL_ID, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        let _ = monster.finish_base_attack_cast_with_clock(KNOCK_OUT_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}

fn install(game: &mut CGame, region_id: i32, target: Target, state: KnockOutState, now_ms: u32) {
    match target.identity.object_type {
        PLAYER_TYPE => {
            let _ = replace_player_knock_out_state(game, target.identity.id, state, now_ms);
        }
        MONSTER_TYPE => {
            let mut owner = game.take_region_owner(region_id);
            if let Some(owner) = owner.as_mut() {
                let _ = replace_monster_knock_out_state(
                    game,
                    owner.base_mut(),
                    target.identity.id,
                    state,
                    now_ms,
                );
            }
            if let Some(owner) = owner {
                game.restore_region_owner(owner);
            }
        }
        _ => {}
    }
}

fn release(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
}

fn abort_player_knock_out(game: &mut CGame, player_id: i32) { release(game, player_id); }
fn finish_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) { finish_state_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(KNOCK_OUT_SKILL_ID, now_ms)); }
pub(crate) fn complete_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = ai.player_skill_execution(KNOCK_OUT_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; release(game, player_id); finish_player_knock_out(game, player_id, ai, runtime); ai.finish_player_skill(dispatch, SkillTermination::Completed) }
pub(crate) fn cancel_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool { let Some(dispatch) = ai.player_skill_execution(KNOCK_OUT_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; abort_player_knock_out(game, player_id); ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }

pub(crate) fn execute_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let identity = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id: KNOCK_OUT_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: KNOCK_OUT_SKILL_ID, .. } => {
            failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        PlayerSkillDispatch::Object { skill_id: KNOCK_OUT_SKILL_ID, target } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => target,
        _ => return result(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, sx, sy, level)) = game.find_player(player_id).and_then(|p| Some((p.server_region_id()?, p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.learned_skill_level(KNOCK_OUT_SKILL_ID, game.skill_factory())))) else { return result(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(KNOCK_OUT_SKILL_ID, level) else { failure(game, player_id, 2); return result(QueuedSkillExecutionState::Rejected) };
    let delay = properties.query_property(DELAY); let persist = properties.query_property(PERSIST);
    let reuse = properties.query_property(REUSE); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let _can_break = properties.query_property(CAN_BREAK);
    let Some(mut target) = target_snapshot(game, region_id, identity) else { failure(game, player_id, 10); return result(QueuedSkillExecutionState::Rejected) };
    if ai.player_skill_execution(KNOCK_OUT_SKILL_ID).is_none() {
        let now = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if !skill_is_restored(ai.skill_last_used_ms(KNOCK_OUT_SKILL_ID), reuse, now) {
            failure(game, player_id, 0x0d); failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        if maximum != 0 && maximum.wrapping_add(1) < game.base_magic_path(region_id, sx, sy, target.x, target.y, None).len() as u32 {
            failure(game, player_id, 0x0b); failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false); player.set_current_skill_id(Some(KNOCK_OUT_SKILL_ID));
            player.movement_shape_mut().set_direction(get_line_direction(sx, sy, target.x, target.y));
        }
        ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, now));
        return result(QueuedSkillExecutionState::Begun);
    } else if ai.player_skill_execution(KNOCK_OUT_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) { return result(QueuedSkillExecutionState::Rejected); }
    if target.dead { failure(game, player_id, 10); abort_player_knock_out(game, player_id); return result(QueuedSkillExecutionState::Rejected); }
    if ai.player_skill_execution(KNOCK_OUT_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        cast(game, player_id, target, level, false);
        if let Some(execution) = ai.player_skill_execution_mut(KNOCK_OUT_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = ai.player_skill_execution(KNOCK_OUT_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение оглушения создано");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return result(QueuedSkillExecutionState::Pending); }
    abort_player_knock_out(game, player_id);
    target = match target_snapshot(game, region_id, identity) { Some(target) => target, None => return result(QueuedSkillExecutionState::Rejected) };
    let Some((sx, sy, combat, source_level)) = game.find_player(player_id).and_then(|p| Some((p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.combat_properties(), p.level()))) else { return result(QueuedSkillExecutionState::Rejected) };
    if maximum != 0 && maximum.wrapping_add(1) < game.base_magic_path(region_id, sx, sy, target.x, target.y, None).len() as u32 { failure(game, player_id, 0x0b); return result(QueuedSkillExecutionState::Rejected); }
    cast(game, player_id, target, level, true);
    let (base, magnify, level_rate) = game.globe_setup().base_attack_hit_formula();
    let chance = knock_out_hit_chance(
        combat.hit,
        target.dodge,
        source_level,
        target.level,
        base,
        magnify,
        level_rate,
    );
    if chance <= game.skill_random_below(100) { return result(QueuedSkillExecutionState::Completed); }
    if let Some((master, attack)) = attack(game, player_id, target.level) {
        match target.identity.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.identity.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.identity.id, region_id, attack, runtime), _ => {}
        }
    }
    if !target.cured {
        let state_now = runtime.now_milliseconds();
        install(game, region_id, target, KnockOutState::new(state_now, persist), runtime.now_milliseconds());
    }
    if let Some(execution) = ai.player_skill_execution_mut(KNOCK_OUT_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_knock_out(game, player_id, ai, runtime);
    result(QueuedSkillExecutionState::Completed)
}
