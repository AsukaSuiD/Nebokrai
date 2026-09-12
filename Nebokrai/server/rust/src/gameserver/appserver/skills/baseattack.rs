//! Базовая атака CBaseAttack (1) и сохраняемые общие хвосты снарядов.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/baseattack.cpp.
//! Player Begin/AI/Attack/visual находятся в baseattackruntime; каждый terminal
//! явно вызывает зарегистрированный End, затем очередь освобождает команду.
//! End(1) единожды изнашивает оружие и фиксирует reuse; End(0) этого не делает.
//! Выбранный навык меняет AI, а не пустой callback OnEndSkill.
//! Monster-origin использует соседний monsterbaseattack и приведённые ниже
//! адаптеры. Его ещё не сверенный общий Calculate/Attack/AI сохранён в RAW.
//! Общие delayed/immediate helpers остаются у не переведённых на полный End
//! caller-ов; базовая атака игрока их больше не вызывает.
//! Достижимость отдельного Restart (0x00513E00) из текущего расписания
//! не подтверждена: повторный Begin не считается его реализацией.

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::real_distance_between_points;
use crate::gameserver::appserver::skills::kernel::{SkillExecutionKernel, SkillTermination};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const BASE_ATTACK_SKILL_ID: u32 = 1;

#[allow(clippy::too_many_arguments, reason = "контекст исходного Attack и его IsAttackAble")]
pub(crate) fn owned_monster_base_attack_allowed(
    game: &CGame,
    region: &crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    property: &crate::setup::monsterlist::MonsterProperties,
    tamed: bool,
    master: crate::gameserver::appserver::masterinfo::MasterInfo,
    target: crate::gameserver::appserver::shape::ShapeIdentity,
) -> bool {
    if target.object_type == 600 && target.id == monster_id {
        return false;
    }
    super::monsterattack::resolve_owned_monster_attack_target(game, region, target)
        .is_some_and(|snapshot| {
            !snapshot.god && !snapshot.city_dead
                && super::monsterattack::owned_monster_attackable(
                    game, region.id, property, tamed, master, target, &snapshot,
                )
        })
}

pub(crate) fn begin_owned_monster_base_attack(
    game: &CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    target: crate::gameserver::appserver::shape::ShapeIdentity,
    skill_level: u16,
    started_at_ms: u32,
    factory: &super::skillfactory::CSkillFactory,
) {
    use crate::gameserver::appserver::monster::{MonsterBaseAttackCast, MonsterBaseAttackDispatch};
    let target_object = resolve_owned_skill_begin_object(game, region, target);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.install_base_attack_cast(MonsterBaseAttackCast::begin(MonsterBaseAttackDispatch {
            target, skill_id: BASE_ATTACK_SKILL_ID, skill_level,
        }, started_at_ms), target_object, factory);
    }
}

pub(crate) fn start_owned_monster_base_attack_ai(
    game: &CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    source: crate::gameserver::appserver::shape::ShapeView,
    target: Option<crate::gameserver::appserver::shape::ShapeView>,
    maximum_distance: u32,
) -> bool {
    use super::kernel::SkillStage;
    let distance = if let Some(target) = target {
        source.real_distance(Some(target))
    } else {
        let Some(monster) = region.find_monster_by_id(monster_id) else { return false };
        monster.move_shape().shape().real_distance_to_point(0, 0)
    };
    if maximum_distance != 0 && distance as u32 > maximum_distance {
        let _ = super::monsterattack::end_owned_monster_skill_without_reuse(region, monster_id, BASE_ATTACK_SKILL_ID, game.skill_factory());
        return false;
    }
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else { return false };
    let Some(cast) = monster.current_active_attack_cast(game.skill_factory()) else { return false };
    let (target_x, target_y) = target.map_or((0, 0), |target| (target.tile_x, target.tile_y));
    monster.move_shape_mut().shape_mut().set_direction(crate::public::tools::get_line_direction(
        source.tile_x, source.tile_y, target_x, target_y,
    ));
    let shape = monster.move_shape().shape().clone();
    let mut start = crate::nets::netserver::message::CMessage::new(0x000b_fe01);
    start.add_byte(1);
    start.add_long(BASE_ATTACK_SKILL_ID as i32);
    start.add_short(cast.dispatch().skill_level as i16);
    start.add_long(600);
    start.add_long(monster_id);
    start.add_long(shape.get_direction());
    let _ = game.send_game_shape_around(region, &shape, None, &start);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(BASE_ATTACK_SKILL_ID, SkillStage::Begin, SkillStage::Check, game.skill_factory());
    }
    true
}

pub(crate) fn handle_owned_monster_base_target_loss<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut crate::gameserver::appserver::serverregion::CServerRegion,
    monster_id: i32,
    source: crate::gameserver::appserver::shape::ShapeView,
    properties: &super::skillbaseproperties::CSkillBaseProperties,
    runtime: &mut Runtime,
) -> bool {
    use super::kernel::SkillStage;
    let Some(cast) = region.find_monster_by_id(monster_id).and_then(|monster| monster.current_active_attack_cast(game.skill_factory())) else { return false };
    if cast.dispatch().skill_id != BASE_ATTACK_SKILL_ID { return false }
    let target = super::monsterattack::resolve_owned_monster_attack_target(game, region, cast.dispatch().target);
    if let Some(target) = target {
        if !target.dead { return false }
    } else {
        if cast.stage() == SkillStage::Begin && !start_owned_monster_base_attack_ai(
            game, region, monster_id, source, None,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
        ) {
            return true;
        }
        if !super::kernel::skill_is_restored(
            cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME), runtime.now_milliseconds(),
        ) {
            return true;
        }
        let Some(monster) = region.find_monster_by_id(monster_id) else { return true };
        let mut fire = crate::nets::netserver::message::CMessage::new(0x000b_fe01);
        fire.add_byte(2);
        fire.add_long(BASE_ATTACK_SKILL_ID as i32);
        fire.add_short(cast.dispatch().skill_level as i16);
        fire.add_long(600);
        fire.add_long(monster_id);
        for _ in 0..4 { fire.add_long(0); }
        let _ = game.send_game_shape_around(region, monster.move_shape().shape(), None, &fire);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.finish_base_attack_cast_with_clock(BASE_ATTACK_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
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

// Сохраняется для ещё не сверенного исполнения исходным monster-owner.
// ============================================================================
// FUNCTION: CBaseAttack::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:279
// RVA: 0x001B3600
// ADDRESS: 005b3600
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:248
// RVA: 0x001B3860
// ADDRESS: 005b3860
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:123
// RVA: 0x001B39B0
// ADDRESS: 005b39b0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
