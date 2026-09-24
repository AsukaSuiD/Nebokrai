//! Небесный огонь CTianhuo (0x21A), gameserver.exe/GameServer.pdb,
//! appserver/skills/tianhuo.cpp.
//!
//! Общий зарегистрированный вход сохраняет исходную объектную цель Check,
//! часы base Begin и единственный внешний End(int). Проверки состояний,
//! reuse и пути общие с громовыми призывами, но Tianhuo не проверяет
//! препятствия, читает часы reuse раньше свойства и требует equipment[10]
//! даже при нулевой цене. CAN этим навыком не изменяется.
//!
//! Первый AI необратимо списывает MP и отправляет BF918 только владельцу,
//! затем поворачивает U и повторяет проверку длины пути. После абсолютной
//! задержки сохранённая объектная цель проверяется и превращается в точку
//! до visual1. Summon получает прежнего U, заново читает свойства и создаёт
//! самостоятельную область; его отказ не меняет завершающий End(1).
//! Gameplay ID области — 0x21A, legacy ID режима применения эффекта — 0x13A;
//! wire и lifetime области принадлежат CTianhuoPhalanx.

use super::basemagic::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
};
use super::battlefairyskill::execute_registered_battle_fairy_state;
use super::kernel::{SkillStage, battle_fairy_mana_text_cost};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome;
use super::thunder::{
    check_battle_fairy_summon_prefix, fail_battle_fairy_summon, master_info,
};
use super::tianhuophalanx::CTianhuoPhalanx;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BF_MP, GAP_BF_SPRITE_BASE,
};
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::public::tools::get_line_direction;

pub(crate) const TIANHUO_SKILL_ID: u32 = 0x21a;
pub(crate) const TIANHUO_TARGET_DAMAGE_FACTOR_PROPERTY: u32 = 20_003;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

fn fail_mana(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(
        player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost),
    );
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(properties) = check_battle_fairy_summon_prefix(
        game, instance, player_id, begin_target, runtime, true, false,
    ) else { return false; };
    let _ = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let Some(current) = game.find_player(player_id)
        .and_then(|player| player.equipment().get_goods(10))
        .map(|goods| goods.addon_property_value(game.goods_factory(), GAP_BF_MP, 1))
    else { return false; };
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    if current.wrapping_sub(cost as i32) < 0 {
        fail_mana(game, instance, player_id, &properties);
        return false;
    }
    true
}

pub(crate) fn execute_battle_fairy_tianhuo<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != TIANHUO_SKILL_ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        |game, instance, player_id, runtime| {
            check_cast(game, instance, player_id, begin_target, runtime)
        },
        run_ai,
    )
}

fn send_equipment_update_to_player(game: &CGame, player_id: i32) {
    let Some(goods) = game.find_player(player_id)
        .and_then(|player| player.equipment().get_goods(10))
    else { return; };
    let mut payload = Vec::new();
    let _ = goods.serialize_for_old_client(
        &mut payload, game.goods_factory(), game.globe_setup().da_kong_key(),
    );
    let mut message = CMessage::new(0x0b_f918);
    message.add_long(player_id);
    message.base_mut().add_guid(goods.identity().ex_id);
    message.add_ulong(payload.len() as u32);
    message.base_mut().add(&payload);
    let _ = message.send_to_player(game.net_server(), player_id);
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let (source_region, source_identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, source_region, source_identity)
        .map(|shape| shape.shape().identity())
    else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };

    if skill.execution_stage() == Some(SkillStage::Begin) {
        // В этой ветви оригинал разыменовывает CPlayer после RTTI без проверки.
        if source.object_type != 400 {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(current) = game.find_player(source.id)
            .and_then(|player| player.equipment().get_goods(10))
            .map(|goods| goods.addon_property_value(game.goods_factory(), GAP_BF_MP, 1))
        else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, source.id, &properties);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(_stored) = game.set_player_equipment_addon_property(
            source.id, 10, GAP_BF_MP, 1, remaining,
        ) else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };
        send_equipment_update_to_player(game, source.id);

        let Some(skill) = game.registered_skill(instance) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        let destination = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, target)| resolve_state_move_shape(game, region, target))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ))
            .unwrap_or_else(|| skill.lifecycle().destination());
        if let Some(player) = game.find_player_mut(source.id) {
            let source_y = player.shape().get_tile_y().unwrap_or(i32::MIN);
            let source_x = player.shape().get_tile_x().unwrap_or(i32::MIN);
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, destination.0, destination.1,
            ));
        }
        let Some(skill) = game.registered_skill(instance) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
            if path.len() > maximum as usize {
                fail_battle_fairy_summon(game, instance, source.id, 11, b"ZHGS0049");
                return state_skill_outcome(QueuedSkillExecutionState::Rejected);
            }
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| {
        skill.execution_stage() != Some(SkillStage::Check)
    }) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let started = skill.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let (_, saved_target) = skill.lifecycle().sufferer();
    if saved_target.object_type != 0 && saved_target.id != 0 {
        let target = resolve_skill_sufferer(game, skill.lifecycle());
        let Some((region, target)) = target.filter(|(region, target)| {
            !game.base_magic_target_dead(*region, *target)
        }) else {
            game.update_registered_skill_visual(instance, 10);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        let Some(destination) = resolve_state_move_shape(game, region, target).map(|target| (
            target.shape().get_tile_x().unwrap_or(i32::MIN),
            target.shape().get_tile_y().unwrap_or(i32::MIN),
        )) else {
            game.update_registered_skill_visual(instance, 10);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_point_target(destination);
        }
    }
    game.update_registered_skill_visual(instance, 1);
    if let Some(destination) = game.registered_skill(instance)
        .map(|skill| skill.lifecycle().destination())
    {
        summon(game, instance, (source_region, source), destination, runtime);
    }
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some(region_id) = resolve_state_move_shape(game, source.0, source.1)
        .filter(|source| source.shape().is_assigned_to_server_region())
        .map(|source| source.shape().get_region_id())
        .filter(|region| game.find_region(*region).is_some())
    else { return; };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return;
    };
    if let Some(skill) = game.registered_skill_mut(instance) {
        let point = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(point);
    }
    if source.1.object_type != 400 { return; }
    let Some(player) = game.find_player(source.1.id) else { return; };
    let Some(goods) = player.equipment().get_goods(10) else { return; };
    let master = master_info(player);
    let _ = goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE_BASE, 1);
    let _ = properties.query_property(SKILL_USAGE_EM_MODIFIER);
    let element_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let summon_id = game.allocate_summon_shape_id();
    let mut phalanx = CTianhuoPhalanx::new(
        summon_id, master, started, lifetime, level, minimum_attack,
        maximum_attack, element_modifier,
    );
    phalanx.set_center(destination.0, destination.1);
    if game.add_tianhuo_phalanx(
        region_id, phalanx, destination.0, destination.1, started, runtime,
    ).is_some_and(|result| result.is_ok()) {
        let _ = game.send_tianhuo_phalanx_entry(region_id, summon_id, runtime);
    }
}
