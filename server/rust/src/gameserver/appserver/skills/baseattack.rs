//! Базовая атака CBaseAttack (1) и сохраняемые общие хвосты снарядов.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/baseattack.cpp.
//! Player Begin/AI/Attack/visual находятся в baseattackruntime; каждый terminal
//! явно вызывает зарегистрированный End, затем очередь освобождает команду.
//! End(1) изнашивает оружие только у игрока и фиксирует reuse; End(0) не делает ни того, ни другого.
//! Выбранный навык меняет AI, а не пустой callback OnEndSkill.
//! Monster-origin получает Begin из соседнего monsterbaseattack, а собственный
//! AI выполняется здесь с опубликованным регионом. Actual S перечитывается после
//! visual; Attack передаёт UNKNOWN/1 в общий приёмник без копии Defense/HP/AI.
//! Общие delayed/immediate helpers остаются у не переведённых на полный End
//! caller-ов; базовая атака игрока их больше не вызывает.
//! Достижимость отдельного Restart (0x00513E00) из текущего расписания
//! не подтверждена: повторный Begin не считается его реализацией.

use crate::gameserver::appserver::states::state::{
    resolve_owned_skill_begin_object, resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::real_distance_between_points;
use crate::gameserver::appserver::skills::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};

pub(crate) const BASE_ATTACK_SKILL_ID: u32 = 1;

pub(crate) fn begin_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) {
    let target_object = resolve_owned_skill_begin_object(game, region, target);
    let started = runtime.now_milliseconds();
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return; };
    if !monster.prepare_base_attack_cast(
        target, BASE_ATTACK_SKILL_ID, skill_level, started, target_object, game.skill_factory(),
    ) { return; }
    monster.move_shape_mut().replace_skill_visual_effect(
        BASE_ATTACK_SKILL_ID, game.skill_factory(), SkillVisualEffect::new(SkillVisualEffectKind::BaseAttack, 1),
    );
    let valid = monster.move_shape().skill(BASE_ATTACK_SKILL_ID, game.skill_factory())
        .is_some_and(|skill| game.skill_base_properties(BASE_ATTACK_SKILL_ID, skill.level()).is_some());
    if valid {
        monster.enqueue_base_attack_cast(started);
    } else {
        // CMonster::OnBeginSkill — RET1, AfterUse для него пуст. Отказ
        // CBaseAttack::Begin вызывает только End(0), без собственного visual 2.
        let _ = monster.finish_base_attack_cast_without_reuse(BASE_ATTACK_SKILL_ID, game.skill_factory());
    }
}

fn monster_sufferer(game: &CGame, instance: RegisteredSkill) -> Option<(i32, ShapeIdentity)> {
    let target = resolve_skill_sufferer(game, game.registered_skill(instance)?.lifecycle())?;
    let shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

fn monster_attack_bounds(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(u32, u32)> {
    let monster = game.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
    let properties = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
    Some(monster.state_attack_bounds(properties.minimum_attack, properties.maximum_attack))
}

fn calculate_monster_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.damage_modifier = 0;
    // CMonster наследует GetWeaponModifier = 1; GetCCH и GetAddElementAtk
    // возвращают ноль. Calculate не записывает skill ID/level в seed.
    attack.damage_factor = 1.0;
    attack.hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let Some((_, maximum)) = monster_attack_bounds(game, source) else { return; };
    let Some((minimum, _)) = monster_attack_bounds(game, source) else { return; };
    let span = (maximum as i32).wrapping_sub(minimum as i32).max(0);
    let Some((minimum, _)) = monster_attack_bounds(game, source) else { return; };
    let physical = (minimum as i32).wrapping_add(game.skill_random_below(span)).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 });
    let Some(monster) = game.find_region(source.0).and_then(|region| region.base().find_monster_by_id(source.1.id))
    else { return; };
    let Some(properties) = monster.base_property_key().and_then(|name| game.find_monster_property_by_origin_name(name))
    else { return; };
    let soul = monster.soul_attack(properties);
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(soul), mp_damage: 0 });
    // В отличие от MonsterBaseAttack, этот RNG вызывается и при CCH == 0.
    let _ = game.skill_random_below(100);
}

fn attack_by_monster<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) {
    let Some(target) = target else { return; };
    if resolve_state_move_shape(game, source.0, source.1).is_none()
        || (source.1.object_type == target.1.object_type && source.1.id == target.1.id)
        || target.1.object_type == 500
        || !game.live_skill_target_attackable(source.0, source.1, target.1)
    { return; }
    let master = MasterInfo { master_type: source.1.object_type, master_id: source.1.id, ..MasterInfo::default() };
    let mut attack = AttackInformation::for_master(master);
    calculate_monster_attack(game, instance, source, &mut attack);
    // Исчезнувшая таблица оставляет пустой UNKNOWN/1 seed, но не отменяет
    // вызов +15C(..., false). CMonster::IncreaseRp после него — пустой RET8.
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

fn execute_monster_stage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> Option<i32> {
    let skill = game.registered_skill(instance)?;
    let kernel = *skill.monster_kernel()?;
    if kernel.lifecycle().is_ended() { return None; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return Some(0); };
    let lifecycle = *skill.lifecycle();
    let Some(source_shape) = resolve_state_move_shape(game, lifecycle.user().0, lifecycle.user().1) else { return Some(0); };
    let source = (source_shape.shape().get_region_id(), source_shape.shape().identity());
    let target = monster_sufferer(game, instance);
    if target.is_some_and(|target| game.base_magic_target_dead(target.0, target.1)) {
        game.update_registered_skill_visual(instance, 2);
        return Some(1);
    }
    if kernel.stage() == SkillStage::Begin {
        let Some(source_view) = game.base_magic_target_view(source.0, source.1) else { return Some(0); };
        let distance = if let Some(target) = target {
            let Some(target_view) = game.base_magic_target_view(target.0, target.1) else { return Some(0); };
            source_view.real_distance(Some(target_view))
        } else {
            source_shape.shape().real_distance_to_point(lifecycle.destination().0, lifecycle.destination().1)
        };
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
            && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < distance as u32
        {
            game.update_registered_skill_visual(instance, 11);
            return Some(0);
        }
        let can_break = properties.query_property(super::basemagic::SKILL_USAGE_CAN_BE_BREAKED);
        game.registered_skill_mut(instance)?.lifecycle_mut().set_available(can_break != 0);
        let (target_x, target_y) = if let Some(target) = target {
            let shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
            (shape.get_tile_x().ok()?, shape.get_tile_y().ok()?)
        } else {
            lifecycle.destination()
        };
        let source_shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        let source_y = source_shape.get_tile_y().ok()?;
        let source_x = source_shape.get_tile_x().ok()?;
        resolve_state_move_shape_mut(game, source.0, source.1)?.shape_mut().set_direction(
            crate::public::tools::get_line_direction(source_x, source_y, target_x, target_y),
        );
        game.update_registered_skill_visual(instance, 0);
        let _ = game.registered_skill_mut(instance)?.monster_kernel_mut()?.advance(SkillStage::Begin, SkillStage::Check);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let started = game.registered_skill(instance)?.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return None; }
    game.update_registered_skill_visual(instance, 1);
    let target = monster_sufferer(game, instance);
    attack_by_monster(game, instance, source, target, runtime);
    Some(1)
}

/// Публикуется исходный регион целиком: visual, приём удара и вложенный End
/// работают с тем же экземпляром. FIFO и выбор следующего навыка остаются у AI.
pub(crate) fn execute_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(region) = owner.as_ref().map(ServerRegionOwner::base) else { return false; };
    let Some(monster) = region.find_monster_by_id(monster_id) else { return false; };
    let holder = (region.id, monster.move_shape().shape().identity());
    game.with_published_region(owner, |game| {
        let Some(instance) = game.registered_move_shape_skill(holder.0, holder.1, BASE_ATTACK_SKILL_ID) else { return false; };
        if let Some(argument) = execute_monster_stage(game, instance, runtime)
            && game.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended())
        {
            let termination = if argument != 0 { SkillTermination::Completed } else { SkillTermination::Rejected };
            let _ = game.end_registered_instance(instance, argument, termination, runtime);
        }
        true
    }).unwrap_or(false)
}

pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5003;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_USER_HIT_MODIFIER: u32 = 20_001;

pub(crate) type BaseAttackExecutionState = SkillExecutionKernel<PlayerSkillDispatch>;

pub(crate) const fn time_reached(now_ms: u32, started_at_ms: u32, delay_ms: u32) -> bool {
    now_ms.wrapping_sub(started_at_ms) >= delay_ms
}

pub(crate) fn real_distance(source_x: i32, source_y: i32, target_x: i32, target_y: i32) -> i32 {
    real_distance_between_points(source_x, source_y, target_x, target_y)
}

/// Отказ ещё не переведённых projectile-caller-ов восстанавливает движение
/// без AfterUse. Сам kernel освобождает общий хвост очереди.
pub(crate) fn finish_failed_base_attack(game: &mut CGame, player_id: i32, restore_movement: bool) {
    if let Some(player) = game.find_player_mut(player_id) {
        if restore_movement {
            player.set_skill_moveable(true);
        }
    }
}

/// Частичный хвост ещё не переведённых caller-ов: восстановление движения
/// при необходимости, затем износ оружия и reuse. Это не полный End.
fn finish_base_attack_owner<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
    restore_movement: bool,
) {
    if restore_movement
        && let Some(player) = game.find_player_mut(player_id)
    {
        player.set_skill_moveable(true);
    }
    game.after_use_player_skill(player_id, skill_id, runtime);
}

pub(crate) fn finish_delayed_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    finish_base_attack_owner(game, player_id, skill_id, runtime, true);
}

pub(crate) fn finish_immediate_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    finish_base_attack_owner(game, player_id, skill_id, runtime, false);
}

pub(crate) fn cancel_player_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    nonzero_end: bool,
    runtime: &mut Runtime,
) -> bool {
    let Some(instance) = game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID) else {
        return false;
    };
    let Some(dispatch) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    if game.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended()) {
        game.with_published_player_ai(player_id, player_ai, |game| {
            let _ = game.end_registered_instance(instance, i32::from(nonzero_end), SkillTermination::Cancelled, runtime);
        });
    }
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn abort_player_base_attack_on_region_change(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, BASE_ATTACK_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    let instance = game.registered_player_skill(player_id, BASE_ATTACK_SKILL_ID);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    if let Some(instance) = instance {
        let _ = game.end_registered_instance_without_after_use(instance, SkillTermination::Cancelled);
    }
    game.finish_registered_player_command(instance, player_ai, dispatch, SkillTermination::Cancelled)
}

// ============================================================================
// FUNCTION: CBaseAttack::Restart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:110
// RVA: 0x00113E00
// ADDRESS: 00513e00
// PROTOTYPE: void __thiscall Restart(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
