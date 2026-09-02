//! Ядовитая атака паука `CSpiderPoison` (`0x191`) для игрока и монстра.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/spiderpoison.cpp`. Объектный путь сохраняет проверку
//! длины пути, задержку повторного применения, блокировку движения, прямой удар
//! с двумя RNG-вызовами и отдельный бросок вероятности `CSpiderPoisonState`.
//! `Attack` создаёт `tagAttackInformation` со штатными skill-id `0x7fffffff`
//! и уровнем `1`; `CalculateAttackPower` заполняет урон, не меняя эти поля.
//! Player-критический множитель переводится в `int` с x87 усечением к нулю;
//! monster-вызов сохраняет RNG, но его виртуальный critical chance равен нулю.
//! Состояние заменяется после удара и только при отсутствии `Cure`;
//! `CGame` координирует владельцев и применение рассчитанных последствий.
//! Координатный `Begin` по точному EXE использует общий
//! `CState::GetSufferer`: выбирает первый `CMoveShape` клетки и продолжает
//! через тот же объектный pipeline.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillTermination};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoisonstate::{
    SpiderPoisonState, send_spider_poison_state_visual_in_region,
};
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::state::resolve_coordinate_sufferer;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const CURE_SKILL_ID: u32 = 0x131;
const DEFAULT_CONTACT_SKILL_ID: u32 = i32::MAX as u32;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_BASE_PROBABILITY: u32 = 40_001;
pub(crate) const SPIDER_POISON_SKILL_ID: u32 = 0x191;

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_player_spider_poison_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: SPIDER_POISON_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: SPIDER_POISON_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

fn player_target(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        PlayerSkillDispatch::Point { skill_id, x, y } if skill_id == SPIDER_POISON_SKILL_ID => {
            let target = resolve_coordinate_sufferer(game, region_id, x, y)?;
            matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE).then_some(target)
        }
        _ => None,
    }
}

fn player_master(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn send_player_visual(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(SPIDER_POISON_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(direction);
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        return;
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(0x000b_fe01, player_id, action);
}

fn finish_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    successful: bool,
) {
    if successful {
        game.damage_player_weapon(player_id, runtime);
    }
    let _ = game.update_player_properties(player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
    if successful {
        player_ai.mark_spider_poison_used(runtime.now_milliseconds());
    }
}

pub(crate) fn cancel_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.spider_poison().map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_spider_poison(game, player_id, player_ai, runtime, false);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn calculate_player_attack(game: &mut CGame, player_id: i32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = player_master(player);
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let mut attack = AttackInformation {
        skill_id: DEFAULT_CONTACT_SKILL_ID, skill_level: 1, attacker_type: PLAYER_TYPE,
        attacker_id: player_id, attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier: 0, damage_factor: 1.0, damage_modifier: 0, critical: false,
        blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    Some((master, attack))
}

pub(crate) fn execute_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_spider_poison_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, skill_level)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(SPIDER_POISON_SKILL_ID)))
    }) else { return player_terminal(QueuedSkillExecutionState::Rejected) };
    let Some(target) = player_target(game, region_id, dispatch) else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(SPIDER_POISON_SKILL_ID, skill_level).cloned() else {
        if player_ai.spider_poison().is_some() { finish_player_spider_poison(game, player_id, player_ai, runtime, false); }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let target_position = game.move_shape_target_tile(Some(region_id), target);
    let Some((target_x, target_y)) = target_position else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    if player_ai.spider_poison().is_none() {
        let now_ms = runtime.now_milliseconds();
        if player_ai.spider_poison_last_used_ms() != 0
            && !time_reached(now_ms, player_ai.spider_poison_last_used_ms(), reuse_delay)
        {
            send_player_failure(game, player_id, 0x0d);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_player_failure(game, player_id, 0x0b);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SPIDER_POISON_SKILL_ID));
        }
        player_ai.begin_spider_poison(SkillExecutionKernel::begin(dispatch, now_ms));
    } else if player_ai.spider_poison().is_none_or(|execution| execution.dispatch() != dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.periodic_state_target_dead(region_id, target) {
        send_player_failure(game, player_id, 10);
        finish_player_spider_poison(game, player_id, player_ai, runtime, false);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.spider_poison().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(direction);
        }
        send_player_visual(game, player_id, skill_level, 1, None);
        if let Some(execution) = player_ai.spider_poison_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = player_ai.spider_poison().map(SkillExecutionKernel::started_at_ms).unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
    if maximum_distance != 0 && path.len() > maximum_distance as usize {
        send_player_failure(game, player_id, 0x0b);
        finish_player_spider_poison(game, player_id, player_ai, runtime, false);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    send_player_visual(game, player_id, skill_level, 2, Some((target, target_x, target_y)));
    if let Some(execution) = player_ai.spider_poison_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let owner = game.find_player(player_id).map(player_master).unwrap_or_default();
    if let Some((master, attack)) = calculate_player_attack(game, player_id) {
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
            _ => {}
        }
    }
    let has_cure = game.find_region(region_id).is_some_and(|region| target_has_cure(game, region.base(), target));
    if !has_cure && game.skill_random_below(100) <= properties.query_property(SKILL_USAGE_BASE_PROBABILITY) as i32 {
        let state_now_ms = runtime.now_milliseconds();
        if let Some(mut region) = game.take_region_owner(region_id) {
            install_spider_poison_state(
                game,
                region.base_mut(),
                target,
                SpiderPoisonState::new(
                    owner,
                    state_now_ms,
                    properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
                    properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
                    properties.query_property(SKILL_USAGE_CONST),
                ),
                state_now_ms,
            );
            game.restore_region_owner(region);
        }
    }
    if let Some(execution) = player_ai.spider_poison_mut() {
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_spider_poison(game, player_id, player_ai, runtime, true);
    player_terminal(QueuedSkillExecutionState::Completed)
}

fn send_visual(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(SPIDER_POISON_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

pub(crate) fn target_has_cure(
    game: &CGame,
    region: &CServerRegion,
    target: ShapeIdentity,
) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_some_and(|player| player.has_state_by_skill_id(CURE_SKILL_ID)),
        MONSTER_TYPE => region.find_monster_by_id(target.id).is_some_and(|monster| monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID)),
        _ => false,
    }
}

pub(crate) fn install_spider_poison_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    target: ShapeIdentity,
    state: SpiderPoisonState,
    now_ms: u32,
) {
    let replaced = match target.object_type {
        PLAYER_TYPE => game.find_player_mut(target.id).and_then(|player| {
            let identity = player.shape().identity();
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            Some((player.replace_spider_poison_state(state), identity, x, y))
        }),
        MONSTER_TYPE => region.find_monster_by_id_mut(target.id).and_then(|monster| {
            let identity = monster.move_shape().shape().identity();
            let x = monster.move_shape().shape().get_tile_x().ok()?;
            let y = monster.move_shape().shape().get_tile_y().ok()?;
            Some((monster.move_shape_mut().replace_spider_poison_state(state), identity, x, y))
        }),
        _ => None,
    };
    let Some((previous, identity, x, y)) = replaced else { return };
    if let Some(previous) = previous {
        send_spider_poison_state_visual_in_region(game, region, identity, x, y, previous, false, now_ms);
    }
    send_spider_poison_state_visual_in_region(game, region, identity, x, y, state, true, now_ms);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((source_shape, property, attacker_master, attacker_tamed, pet_attack, cast, last_used_ms)) =
        region.find_monster_by_id(monster_id).and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone();
            let pet_attack = monster.is_tamed().then(|| monster.pet_attack_properties(&property));
            Some((monster.move_shape().shape().clone(), property, monster.master_info(), monster.is_tamed(), pet_attack, monster.base_attack_cast(), monster.skill_last_used_ms(SPIDER_POISON_SKILL_ID)))
        })
    else { return false };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead || target.god || target.city_dead || !owned_monster_attackable(
        game, region.id, &property, attacker_tamed, attacker_master, target_identity, &target,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source_shape.get_tile_x(), source_shape.get_tile_y(), target.shape.get_tile_x(), target.shape.get_tile_y(),
    ) else { return true };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let path_too_long = maximum_distance != 0
        && region.straight_skill_path(source_x, source_y, target_x, target_y, None).len() > maximum_distance as usize;
    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        if path_too_long {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let attack_interval = pet_attack.map_or(property.attack_speed, |pet| pet.attack_interval);
        let schedule_ready = schedule_attack_interval(property.ai, attack_interval)
            .is_none_or(|interval| {
                region.find_monster_by_id_mut(monster_id).is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, interval)
                })
            });
        if !schedule_ready
            || (last_used_ms != 0 && !time_reached(now_ms, last_used_ms, reuse_delay))
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, SPIDER_POISON_SKILL_ID, skill_level, now_ms);
        }
        let source = region.find_monster_by_id(monster_id).map(|monster| monster.move_shape().shape()).unwrap_or(&source_shape);
        send_visual(game, region, source, monster_id, skill_level, 1, None);
        return true;
    }
    let cast = cast.expect("выполнение ядовитой атаки проверено выше");
    if cast.dispatch().skill_id != SPIDER_POISON_SKILL_ID || cast.dispatch().target != target_identity { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) { return true; }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.move_shape_mut().set_moveable(true); }
    if path_too_long {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(); }
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) { let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate); }
    send_visual(game, region, &source_shape, monster_id, skill_level, 2, Some((target_identity, target_x, target_y)));
    let (minimum, maximum, element) = region.find_monster_by_id(monster_id).map(|monster| {
        let (minimum, maximum) = monster.state_attack_bounds(property.minimum_attack, property.maximum_attack);
        let minimum = pet_attack.map_or(minimum, |pet| pet.minimum_attack);
        let maximum = pet_attack.map_or(maximum, |pet| pet.maximum_attack);
        (minimum as i32, maximum as i32, monster.element_modifier(0) as i32)
    }).unwrap_or((property.minimum_attack as i32, property.maximum_attack as i32, 0));
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span));
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: DEFAULT_CONTACT_SKILL_ID, skill_level: 1, attacker_type: MONSTER_TYPE,
        attacker_id: monster_id, attacker_team_id: 0, attacker_faction_id: 0,
        attacker_union_id: 0, hit_modifier: 0, damage_factor: 1.0, damage_modifier: 0,
        critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: element.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(CMonster::resource_soul_attack(&property)), mp_damage: 0 },
        ],
    };
    let attack = defend_owned_monster_attack(game, target_identity, target.mana, target.war_soul_mana, target.player_properties, target.monster_properties, attack);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    apply_owned_monster_attack_hit(game, region, runtime, now_ms, monster_id, attacker_master,
        target_identity, &target.shape, target.health, target.mana, target.master,
        target.monster_property, target.tamed, target.carriage, attack, deaths);
    if !target_has_cure(game, region, target_identity)
        && game.skill_random_below(100) <= properties.query_property(SKILL_USAGE_BASE_PROBABILITY) as i32 {
        install_spider_poison_state(game, region, target_identity, SpiderPoisonState::new(
            MasterInfo { master_type: MONSTER_TYPE, master_id: monster_id, ..MasterInfo::default() },
            now_ms, properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
            properties.query_property(SKILL_USAGE_CONST)), now_ms);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
