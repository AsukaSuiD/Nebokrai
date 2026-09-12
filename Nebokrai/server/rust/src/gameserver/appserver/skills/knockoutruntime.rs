//! Рабочий владелец исполнения `CKnockOut` (`0x192`) с движущейся целью.
//! На время применения удара настоящий AI источника опубликован в CPlayer;
//! изменения синхронных callback возвращаются в тот же проход навыка.
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

//! Цепочка попадания передаёт Option владельца региона до синхронной смерти.
//! Заимствование базы не переживает эту границу; продолжение заново получает
//! оставшегося владельца, не создавая замену исчезнувшему региону.
//! Координатный вход выбирает первый CMoveShape, включая NPC и постройку;
//! путь идёт к GetBeAttackedPoint. После удара Cure читается заново. Новый
//! payload создаётся до End старого состояния и занимает тот же слот после
//! полного Begin; при отсутствии старого состояния добавляется в конец.

use crate::gameserver::gameserver::game::ServerRegionOwner;

use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_coordinate_sufferer, resolve_identity_sufferer,
    resolve_owned_skill_begin_object, resolve_state_move_shape,
};
use super::baseattack::time_reached;
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::blindstate::begin_primary_blind_state_at;
use super::knockoutstate::KnockOutState;
use super::monsterattack::{
    apply_owned_monster_attack_hit,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::finish_state_skill;
use crate::gameserver::appserver::ai::monsterai::schedule_attack_interval;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
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
struct Target { identity: ShapeIdentity, x: i32, y: i32, level: u8, dodge: u16, dead: bool }

fn result(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<Target> {
    let shape = game.shape_view_in_owner(game.find_region(region_id)?, identity)?;
    let dodge = match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id)?.combat_properties().dodge,
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.dodge(property)
        }
        500 | 1_100 | 1_200 => 0,
        _ => return None,
    };
    Some(Target {
        identity, x: shape.tile_x, y: shape.tile_y,
        level: game.move_shape_level(region_id, identity)?, dodge,
        dead: game.base_magic_target_dead(region_id, identity),
    })
}

fn player_target_path_len(game: &CGame, player_id: i32, region_id: i32, x: i32, y: i32, target: ShapeIdentity) -> u32 {
    if target.object_type == PLAYER_TYPE && target.id == player_id { return 0; }
    game.base_magic_target_point(region_id, x, y, target)
        .map_or(0, |point| game.base_magic_path(region_id, x, y, point.0, point.1, None).len() as u32)
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
) -> Option<AttackInformation> {
    let monster = region.find_monster_by_id(monster_id)?;
    let (minimum, maximum) = monster.state_attack_bounds(property.minimum_attack, property.maximum_attack);
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
            // положительной суммы `dwYaoAtk` и modifier.
            AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(monster.soul_attack(property)), mp_damage: 0 },
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
    Some(attack)
}

/// Generic object-target ветвь `CKnockOut::AI` для monster-owner-а: общий
/// reuse/range lifecycle, блокировка движения до delay, отдельный hit RNG,
/// затем concrete attack, replacement состояния и monster death tail.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_owned_monster_knock_out<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    property: &MonsterProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_mut() else { return false; };
    let Some((source, tamed, cast)) = region_owner.base().find_monster_by_id(monster_id).map(|monster| (monster.move_shape().shape().clone(), monster.is_tamed(), monster.current_active_attack_cast(game.skill_factory()))) else { return false };
    let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity) else {
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    };
    if target.dead {
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (source.get_tile_x(), source.get_tile_y(), target.shape.get_tile_x(), target.shape.get_tile_y()) else { return true };
    let path_len = if source.identity() == target_identity { 0 } else {
        game.base_magic_target_point_in(region_owner, source_x, source_y, target_identity)
            .map_or(0, |point| region_owner.base().straight_skill_path(source_x, source_y, point.0, point.1, None).len() as u32)
    };
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none() {
        let interval = if tamed { region_owner.base().find_monster_by_id(monster_id).map(|monster| monster.pet_attack_properties(property).attack_interval).unwrap_or(property.attack_speed) } else { property.attack_speed };
        if schedule_attack_interval(property.ai, interval).is_some_and(|interval| region_owner.base_mut().find_monster_by_id_mut(monster_id).is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))) { return true; }
        let reuse = properties.query_property(REUSE);
        let last_used = region_owner.base().find_monster_by_id(monster_id).map(|monster| monster.skill_last_used_ms(KNOCK_OUT_SKILL_ID, game.skill_factory())).unwrap_or_default();
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used, reuse, now_ms,
            )
        {
            return true;
        }
        if maximum != 0 && maximum.wrapping_add(1) < path_len {
            if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        let target_object = resolve_owned_skill_begin_object(game, region_owner.base_mut(), target_identity);
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, KNOCK_OUT_SKILL_ID, skill_level, now_ms, target_object, game.skill_factory());
        }
        monster_cast(game, region_owner.base_mut(), monster_id, target_identity, target_x, target_y, skill_level, false);
        return true;
    }
    let cast = cast.expect("активное оглушение проверено выше");
    if cast.dispatch().skill_id != KNOCK_OUT_SKILL_ID || cast.dispatch().target != target_identity { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(DELAY)) { return true; }
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().set_moveable(true); }
    if maximum != 0 && maximum.wrapping_add(1) < path_len {
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
        return true;
    }
    monster_cast(game, region_owner.base_mut(), monster_id, target_identity, target_x, target_y, skill_level, true);
    let target_level = match target_identity.object_type {
        PLAYER_TYPE => game.find_player(target_identity.id).map_or(1, CPlayer::level),
        MONSTER_TYPE => target.monster_property.as_ref().map_or(1, |property| property.level as u8),
        _ => 1,
    };
    let target_dodge = if target_identity.object_type == PLAYER_TYPE {
        target.player_properties.map_or(0, |combat| combat.dodge)
    } else {
        region_owner.base().find_monster_by_id(target_identity.id).zip(target.monster_property.as_ref()).map_or(0, |(monster, property)| monster.dodge(property))
    };
    let source_hit = region_owner.base().find_monster_by_id(monster_id).map_or(1, |monster| monster.hit(property));
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
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) { let _ = monster.finish_base_attack_cast_without_reuse(KNOCK_OUT_SKILL_ID, game.skill_factory()); }
        return true;
    }
    let Some(attack) = monster_attack(game, region_owner.base_mut(), monster_id, property) else { return true };
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(KNOCK_OUT_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(KNOCK_OUT_SKILL_ID, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
    }
    let region_id = region_owner.region_id();
    apply_owned_monster_attack_hit(game, owner, runtime, target_identity, attack);
    let _ = game.with_published_region(owner, |game| {
        if resolve_state_move_shape(game, region_id, target_identity)
            .is_some_and(|shape| !shape.has_state_by_skill_id(CURE_SKILL_ID))
        {
            let state = KnockOutState::new(0, properties.query_property(PERSIST));
            install(game, region_id, source.identity(), target_identity, state, runtime);
        }
    });
    let Some(region_owner) = owner.as_mut() else { return true; };
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(KNOCK_OUT_SKILL_ID, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        let _ = monster.finish_base_attack_cast_with_clock(KNOCK_OUT_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}

fn install<Runtime: GameMainLoopRuntime>(game: &mut CGame, region_id: i32, user: ShapeIdentity, target: ShapeIdentity, state: KnockOutState, runtime: &mut Runtime) {
    let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return; };
    let target_region = shape.shape().get_region_id();
    let previous = shape.find_state_position(|state| state.state_id() == KNOCK_OUT_SKILL_ID);
    let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target_region, target, position);
    }
    let _ = begin_primary_blind_state_at(
        game, target_region, target, Some((region_id, user)), Some((target_region, target)),
        state, placement, &mut || runtime.now_milliseconds(),
    );
}

fn release(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
}

fn abort_player_knock_out(game: &mut CGame, player_id: i32) { release(game, player_id); }
fn finish_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _ai: &mut CPlayerAI, runtime: &mut Runtime) { finish_state_skill(game, player_id, KNOCK_OUT_SKILL_ID, runtime); }
pub(crate) fn complete_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = game.player_skill_execution(player_id, KNOCK_OUT_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; release(game, player_id); finish_player_knock_out(game, player_id, ai, runtime); game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed) }
pub(crate) fn cancel_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool { let Some(dispatch) = game.player_skill_execution(player_id, KNOCK_OUT_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; abort_player_knock_out(game, player_id); game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled) }

pub(crate) fn execute_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let Some((region_id, sx, sy, level)) = game.find_player(player_id).and_then(|p| Some((p.server_region_id()?, p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.learned_skill_level(KNOCK_OUT_SKILL_ID, game.skill_factory())))) else { return result(QueuedSkillExecutionState::Rejected) };
    let identity = match dispatch {
        PlayerSkillDispatch::Object { skill_id: KNOCK_OUT_SKILL_ID, target } => resolve_identity_sufferer(game, region_id, target),
        PlayerSkillDispatch::Point { skill_id: KNOCK_OUT_SKILL_ID, x, y } => resolve_coordinate_sufferer(game, region_id, x, y),
        _ => None,
    };
    let Some(identity) = identity else {
        failure(game, player_id, 2);
        return result(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(KNOCK_OUT_SKILL_ID, level) else { failure(game, player_id, 2); return result(QueuedSkillExecutionState::Rejected) };
    let delay = properties.query_property(DELAY); let persist = properties.query_property(PERSIST);
    let reuse = properties.query_property(REUSE); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let _can_break = properties.query_property(CAN_BREAK);
    let Some(mut target) = target_snapshot(game, region_id, identity) else { failure(game, player_id, 10); return result(QueuedSkillExecutionState::Rejected) };
    if game.player_skill_execution(player_id, KNOCK_OUT_SKILL_ID).is_none() {
        let now = runtime.now_milliseconds();
        game.begin_player_skill_with_combat(player_id, dispatch, now);
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, KNOCK_OUT_SKILL_ID), reuse, now) {
            failure(game, player_id, 0x0d); failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        if maximum != 0 && maximum.wrapping_add(1) < player_target_path_len(game, player_id, region_id, sx, sy, identity) {
            failure(game, player_id, 0x0b); failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false); player.set_current_skill_id(Some(KNOCK_OUT_SKILL_ID));
            player.movement_shape_mut().set_direction(get_line_direction(sx, sy, target.x, target.y));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now));
        return result(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, KNOCK_OUT_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) { return result(QueuedSkillExecutionState::Rejected); }
    if target.dead { failure(game, player_id, 10); abort_player_knock_out(game, player_id); return result(QueuedSkillExecutionState::Rejected); }
    if game.player_skill_execution(player_id, KNOCK_OUT_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        cast(game, player_id, target, level, false);
        if let Some(execution) = game.player_skill_execution_mut(player_id, KNOCK_OUT_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = game.player_skill_execution(player_id, KNOCK_OUT_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение оглушения создано");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return result(QueuedSkillExecutionState::Pending); }
    abort_player_knock_out(game, player_id);
    target = match target_snapshot(game, region_id, identity) { Some(target) => target, None => return result(QueuedSkillExecutionState::Rejected) };
    let Some((sx, sy, combat, source_level)) = game.find_player(player_id).and_then(|p| Some((p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.combat_properties(), p.level()))) else { return result(QueuedSkillExecutionState::Rejected) };
    if maximum != 0 && maximum.wrapping_add(1) < player_target_path_len(game, player_id, region_id, sx, sy, identity) { failure(game, player_id, 0x0b); return result(QueuedSkillExecutionState::Rejected); }
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
        game.with_published_player_ai(player_id, ai, |game| {
            game.apply_owned_skill_contact(master, target.identity, region_id, attack, runtime);
        });
    }
    game.with_published_player_ai(player_id, ai, |game| {
        if resolve_state_move_shape(game, region_id, target.identity)
            .is_some_and(|shape| !shape.has_state_by_skill_id(CURE_SKILL_ID))
        {
            let state = KnockOutState::new(0, persist);
            if let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) {
                install(game, region_id, user, target.identity, state, runtime);
            }
        }
    });
    if let Some(execution) = game.player_skill_execution_mut(player_id, KNOCK_OUT_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_knock_out(game, player_id, ai, runtime);
    result(QueuedSkillExecutionState::Completed)
}
