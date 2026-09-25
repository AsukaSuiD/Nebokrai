//! CFatalBlow (0x21C), gameserver.exe/GameServer.pdb, appserver/skills/fatalblow.cpp.
//!
//! Общий вход сохраняет зарегистрированный экземпляр, исходные аргументы
//! объектного Begin и visual loop1. Check читает таблицу до проверки S;
//! отсутствие S даёт 10/ZHGS0045, собственная цель проверяется только в AI.
//! Первый конфликт состояния выбирается по позиции. MP0 и отсутствие WarSoul
//! дают тихий отказ; стоимость вычитается как signed wrapping DWORD.
//!
//! AI сохраняет U/S и таблицу, проверяет смерть S, затем равенство U/S.
//! Исчезнувший предмет оставляет ожидание; запись MP и BF918 (даже при false
//! Serialize) предшествуют CAN/visual0/condition и абсолютной задержке.
//! Поздний GetTargetPath разрешает свежую S, но диагностика и Summon используют
//! прежнюю цель. Flying-time — поле единственного BF-payload, не копия в effect.
//!
//! Summon повторно проверяет регион U и таблицу, очищает S навыка, сохраняет
//! Master без country и читает параметры конструктора в исходном порядке.
//! Clock предшествует ID; центр берётся у прежней S после конструктора.
//! Отказ AddShape не подавляет сериализацию; после любой попытки Summon
//! координатор выполняет End(1). Попадание и lifetime принадлежат отдельному
//! снаряду: этот owner не подменяет его фоновой AI синхронной атакой.
//! End обнуляет flying-time до visual3; его единственный общий хвост внешний.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairyskill::{
    check_battle_fairy_target_states, execute_registered_battle_fairy_skill,
};
use super::battlefairytransfer::send_goods_update;
use super::fatalblowphalanx::CFatalBlowPhalanx;
use super::kernel::{
    BattleFairyExecution, SkillStage,
    battle_fairy_mana_text_cost, skill_is_restored,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome;
use super::thunder::{summon_user_cch, summon_user_region};
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{GAP_BF_MP, GAP_BF_SPRITE};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
pub(crate) use nebokrai_zone::skills::execution::{FatalBlowExecutionState};

pub(crate) const FATAL_BLOW_SKILL_ID: u32 = 0x21c;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;

fn fail(
    game: &mut CGame, instance: RegisteredSkill, player_id: Option<i32>, mode: u32, text: &[u8],
) -> QueuedSkillExecutionOutcome {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player_id) = player_id { game.send_skill_system_info(player_id, text); }
    state_skill_outcome(QueuedSkillExecutionState::Rejected)
}

fn fail_mana(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost));
}

fn fail_obstacle(
    game: &mut CGame, instance: RegisteredSkill, player_id: Option<i32>,
    target: (i32, ShapeIdentity), text: &[u8],
) {
    game.update_registered_skill_visual(instance, 15);
    if let Some(player_id) = player_id
        && let Some(name) = game.base_magic_target_name(target.0, target.1).map(<[u8]>::to_vec)
    {
        game.send_skill_system_info_with_text(player_id, text, &name);
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let Some(target) = begin_target else {
        let _ = fail(game, instance, Some(player_id), 10, b"ZHGS0045");
        return false;
    };
    if !check_battle_fairy_target_states(game, player_id, target) { return false; }
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(|skill| skill.last_used_ms()) else { return false; };
    if !skill_is_restored(last_used, reuse, runtime.now_milliseconds()) {
        let _ = fail(game, instance, Some(player_id), 13, b"ZHGS0048");
        return false;
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        let _ = fail(game, instance, Some(player_id), 11, b"ZHGS0049");
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        fail_obstacle(game, instance, Some(player_id), target, b"ZHGS0051");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) == 0 { return false; }
    let Some(current) = game.find_player(player_id)
        .and_then(|player| player.war_soul_mana(game.goods_factory()))
    else { return false; };
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    if current.wrapping_sub(cost as i32) < 0 {
        fail_mana(game, instance, player_id, &properties);
        return false;
    }
    true
}

pub(crate) fn execute_battle_fairy_fatal_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != FATAL_BLOW_SKILL_ID { return state_skill_outcome(QueuedSkillExecutionState::Rejected); }
    execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, None,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime),
        |dispatch, started| FatalBlowExecutionState::begin(dispatch, started).into(),
        run_ai,
    )
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let user = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, user.0, user.1)
        .map(|shape| (user.0, shape.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    let (Some(source), Some(target)) = (source, target) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let player_id = (source.1.object_type == 400).then_some(source.1.id);
    if game.base_magic_target_dead(target.0, target.1) {
        return fail(game, instance, player_id, 10, b"ZHGS0050");
    }
    if resolve_state_move_shape(game, source.0, source.1)
        .zip(resolve_state_move_shape(game, target.0, target.1))
        .is_some_and(|(source, target)| std::ptr::eq(source, target))
    {
        return fail(game, instance, player_id, 10, b"ZHGS0045");
    }
    if skill.execution_stage() == Some(SkillStage::Begin) {
        if let Some(player_id) = player_id {
            let Some(current) = game.find_player(player_id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
            let remaining = current.wrapping_sub(properties.query_property(SKILL_USAGE_USER_MP_LOSE) as i32);
            if remaining < 0 {
                fail_mana(game, instance, player_id, &properties);
                return state_skill_outcome(QueuedSkillExecutionState::Rejected);
            }
            let Some(_stored) = game.set_player_equipment_addon_property(player_id, 10, GAP_BF_MP, 1, remaining)
            else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
            let Some(goods) = game.find_player(player_id).and_then(|player| player.equipment().get_goods(10))
            else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
            let mut old_client_payload = Vec::new();
            let _ = goods.serialize_for_old_client(
                &mut old_client_payload, game.goods_factory(), game.globe_setup().da_kong_key(),
            );
            let update = BattleFairyDefaultGoodsUpdate {
                message_type: 0x0b_f918, player_id, goods: goods.identity(), old_client_payload,
            };
            send_goods_update(game, &update);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };
    let started = skill.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return state_skill_outcome(QueuedSkillExecutionState::Pending); }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        return fail(game, instance, player_id, 11, b"ZHGS0049");
    }
    if path.iter().any(|cell| cell.2 == 2) {
        fail_obstacle(game, instance, player_id, target, b"ZHGS0053");
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    let flight = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME).wrapping_mul(path.len() as u32);
    let Some(BattleFairyExecution::FatalBlow(state)) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.battle_fairy_execution_state_mut())
    else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };
    state.set_missile_flying_time(flight);
    game.update_registered_skill_visual(instance, 1);
    summon(game, instance, source, target, runtime);
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1).map(|shape| shape.shape().identity()) else { return; };
    if resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let Some(region) = summon_user_region(game, source) else { return; };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    if let Some(skill) = game.registered_skill_mut(instance) {
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
    }
    let mut master = MasterInfo { master_type: user.object_type, master_id: user.id, ..MasterInfo::default() };
    if user.object_type == 400 {
        let Some(player) = game.find_player(user.id) else { return; };
        let Some(goods) = player.war_soul_goods(game.goods_factory()) else { return; };
        let permissions = player.pk_permissions();
        master.master_team_id = player.team_id();
        master.master_guild_id = player.faction_id();
        master.master_union_id = player.union_id();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
        let _ = goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1);
    }
    let _ = properties.query_property(SKILL_USAGE_EM_MODIFIER);
    let Some(target_identity) = resolve_state_move_shape(game, target.0, target.1)
        .map(|shape| shape.shape().identity())
    else { return; };
    let cch = summon_user_cch(game, user);
    let factor = properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32;
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CFatalBlowPhalanx::new(id, master, started, lifetime, level, factor, cch, target_identity);
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    phalanx.set_center(x, y);
    if game.add_fatal_blow_phalanx(region, phalanx, started, runtime).is_some_and(|result| result.is_ok()) {
        let _ = game.send_fatal_blow_phalanx_entry(region, id, runtime);
    }
}
