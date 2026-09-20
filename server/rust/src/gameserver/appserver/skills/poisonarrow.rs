//! CPoisonArrow (0x21E), gameserver.exe/GameServer.pdb,
//! исходный владелец appserver/skills/poisonarrow.cpp.
//!
//! PoisonArrow/BloodLoss разделяют Check и AI: исходная S передаётся из Begin,
//! таблица читается до отказа NULL/self, конфликт выбирается по позиции.
//! MP0 разрешён в Check, но первый AI всё равно требует WarSoul и выполняет
//! signed wrapping-списание → Serialize/BF918, включая частичный payload,
//! затем CAN → visual0 → condition. Путь повторно разрешается через текущий
//! навык, тогда как смерть, имя цели и установка состояния используют S,
//! захваченную на входе AI. Ожидание сравнивает unsigned wrapping-срок.
//!
//! После visual1 снимается MasterInfo с country=0; PK читает source XY
//! в регионе цели. Сохранённая таблица используется и после callbacks:
//! CONST → frequency → keep → ctor → первый старый state End → destructor
//! свежего остатка → Begin(U,S) → прежний слот либо append. Нет RNG,
//! UpdateProperty или дополнительной публикации списка состояний.
//! Отказ state Begin не отменяет End(1). Общий зарегистрированный вход
//! сохраняет исходный экземпляр, базовые часы и дополнительный visual2
//! при отказе Check; единственный End(int) выполняется внешним координатором.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairyskill::{
    check_battle_fairy_target_states, execute_registered_battle_fairy_state,
};
use super::battlefairytransfer::send_goods_update;
use super::kernel::{SkillStage, battle_fairy_mana_text_cost, skill_is_restored};
use super::poisonarrowstate::{PoisonArrowState, begin_primary_poison_arrow_state};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome as terminal;
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_MP;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const POISON_ARROW_SKILL_ID: u32 = 0x21e;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;

pub(super) struct ArrowEffect {
    pub user: (i32, ShapeIdentity),
    pub target: (i32, ShapeIdentity),
    pub master: MasterInfo,
    pub properties: CSkillBaseProperties,
}

fn fail(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    mode: u32, string_id: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    game.send_skill_system_info(player_id, string_id);
}

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

fn path_allowed(
    game: &mut CGame, instance: RegisteredSkill, player_id: Option<i32>,
    target: (i32, ShapeIdentity), properties: &CSkillBaseProperties,
    obstacle_string: &[u8],
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        game.update_registered_skill_visual(instance, 11);
        if let Some(player_id) = player_id { game.send_skill_system_info(player_id, b"ZHGS0049"); }
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_registered_skill_visual(instance, 15);
        if let Some(player_id) = player_id {
            if let Some(shape) = resolve_state_move_shape(game, target.0, target.1) {
                game.send_skill_system_info_with_text(
                    player_id, obstacle_string, shape.shape().base_object().get_name(),
                );
            }
        }
        return false;
    }
    true
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let user = player.shape().identity();
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let Some(target) = begin_target.filter(|(_, target)| {
        target.object_type != user.object_type || target.id != user.id
    }) else {
        fail(game, instance, player_id, 10, b"ZHGS0045");
        return false;
    };
    if !check_battle_fairy_target_states(game, player_id, target) { return false; }
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let Some(last_used) = game.registered_skill(instance).map(|skill| skill.last_used_ms()) else {
        return false;
    };
    if !skill_is_restored(last_used, reuse, runtime.now_milliseconds()) {
        fail(game, instance, player_id, 13, b"ZHGS0048");
        return false;
    }
    if !path_allowed(game, instance, Some(player_id), target, &properties, b"ZHGS0051") {
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) == 0 { return true; }
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

pub(super) fn execute_periodic_battle_fairy_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime, obstacle_string: &[u8],
    apply: impl FnOnce(&mut CGame, ArrowEffect, &mut Runtime),
) -> QueuedSkillExecutionOutcome {
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, Some(2),
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime),
        |game, instance, runtime| run_ai(game, instance, runtime, obstacle_string, apply),
    )
}

pub(crate) fn execute_battle_fairy_poison_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != POISON_ARROW_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    execute_periodic_battle_fairy_arrow(
        game, player_id, instance, dispatch, begin_target, runtime, b"ZHGS0051", apply_poison_arrow,
    )
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    obstacle_string: &[u8], apply: impl FnOnce(&mut CGame, ArrowEffect, &mut Runtime),
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    let (Some(user), Some(target)) = (user, target) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(target.0, target.1) {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let player_id = (user.1.object_type == 400).then_some(user.1.id);
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if let Some(player_id) = player_id {
            let Some(current) = game.find_player(player_id)
                .and_then(|player| player.war_soul_mana(game.goods_factory()))
            else { return terminal(QueuedSkillExecutionState::Pending); };
            let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
            let remaining = current.wrapping_sub(cost as i32);
            if remaining < 0 {
                fail_mana(game, instance, player_id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let Some(_stored) = game.set_player_equipment_addon_property(player_id, 10, GAP_BF_MP, 1, remaining)
            else { return terminal(QueuedSkillExecutionState::Pending); };
            // Setter сохраняет предмет в equipment. Повторный WarSoul-gate
            // здесь отсутствует; false сериализации не подавляет BF918.
            let Some(goods) = game.find_player(player_id).and_then(|player| player.equipment().get_goods(10))
            else { return terminal(QueuedSkillExecutionState::Pending); };
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
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if !path_allowed(game, instance, player_id, target, &properties, obstacle_string) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.update_registered_skill_visual(instance, 1);
    let mut master = MasterInfo {
        master_type: user.1.object_type, master_id: user.1.id, ..MasterInfo::default()
    };
    if let Some(player) = player_id.and_then(|id| game.find_player(id)) {
        let permissions = player.pk_permissions();
        master.master_guild_id = player.faction_id();
        master.master_team_id = player.team_id();
        master.master_union_id = player.union_id();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
    }
    let target_region = resolve_state_move_shape(game, target.0, target.1).and_then(|shape| {
        let shape = shape.shape();
        shape.is_assigned_to_server_region().then_some(shape.get_region_id())
    }).filter(|region| game.find_region(*region).is_some());
    if let (Some(player_id), Some(target_region)) = (player_id, target_region) {
        if target.1.object_type == 400 {
            if let Some(player) = game.find_player(player_id) {
                let y = player.shape().get_tile_y().unwrap_or(i32::MIN);
                let x = player.shape().get_tile_x().unwrap_or(i32::MIN);
                let _ = game.player_on_first_skill_at_position(
                    player_id, target.1.id, target_region, x, y, runtime,
                );
            }
        }
    }
    apply(game, ArrowEffect { user, target, master, properties }, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn arrow_replacement_slot(
    game: &mut CGame, target: (i32, ShapeIdentity), state_id: u32,
) -> Option<Option<(usize, usize)>> {
    let shape = resolve_state_move_shape(game, target.0, target.1)?;
    if let Some((position, key)) = shape.find_state_position(|state| state.state_id() == state_id) {
        let location = shape.applied_state_replacement_location(key)?;
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
        Some(Some(location))
    } else { Some(None) }
}

fn apply_poison_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, effect: ArrowEffect, runtime: &mut Runtime,
) {
    let hp_loss = effect.properties.query_property(SKILL_USAGE_CONST);
    let frequency = effect.properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let keep = effect.properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state = PoisonArrowState::new(effect.master, keep, frequency, hp_loss);
    let Some(placement) = arrow_replacement_slot(game, effect.target, POISON_ARROW_SKILL_ID) else { return; };
    let _ = begin_primary_poison_arrow_state(
        game, effect.target.0, effect.target.1, Some(effect.user), Some(effect.target), state, placement,
        &mut || runtime.now_milliseconds(),
    );
}
