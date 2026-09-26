//! Атрибутные навыки CPojia..CYufa (октет Po/Yu, 0x212..0x219): Check/AI.
//!
//! Quirks: восстановление использует delay, а не reuse; MP0 допускается без
//! чтения MP; только Yumo списывает ненулевую стоимость уже в Check (первый
//! AI затем списывает её повторно — машинная особенность); Po держит
//! состояние на S (state-U=S/state-S=U), Yu — на U. Pojia сериализует BF918
//! до CAN/visual0/condition, остальные семь — после; отказ Serialize не
//! подавляет пакет (точечная доставка — `send_battle_fairy_goods_update`).
//!
//! Скелет и константы октета общие для всех восьми классов; собственный
//! `End(bool)` — у координатора `battlefairyskill`; state, данные и формулы —
//! `effects/battlefairy.rs`, живой hub-lifecycle — `battlefairyattributestate.rs`
//! старого пакета.
//!
//! Швы: hub `battlefairyskill::BattleFairyGame`; арена и Begin — hub-швы
//! `battle_fairy_end_first_state` и `begin_battle_fairy_attribute_state`.
//!
//! Исходные владельцы PDB: `appserver/skills/{pojia,pobing,pomo,pofa,yujia,
//! yubing,yumo,yufa}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#навыки-семейства-атрибутный-октет-bfbaseattack-transfer-fatalblow-lifeshield

use nebokrai_shared::values::CGuid;

use crate::content::CSkillBaseProperties;
use crate::content::goods::{GAP_BF_BATTLE_FAIRY, GAP_BF_MP};
use crate::effects::{BattleFairyAttributeKind, BattleFairyAttributeState};

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    execute_registered_battle_fairy_state, send_battle_fairy_goods_update,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};

pub const POJIA_SKILL_ID: u32 = 0x212;
pub const YUJIA_SKILL_ID: u32 = 0x216;
pub const YUMO_SKILL_ID: u32 = 0x218;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyAttributeSkill {
    pub value_usage: u32,
    pub kind: BattleFairyAttributeKind,
}

/// Таблица октета: ID навыка → код свойства формулы и вид (буквальные записи
/// `pojia.cpp`..`yufa.cpp`; вид совпадает с машинным отображением
/// `effects::battle_fairy_attribute_kind`).
pub const fn definition(skill_id: u32) -> Option<BattleFairyAttributeSkill> {
    let (value_usage, kind) = match skill_id {
        0x212 => (0xe4, BattleFairyAttributeKind::AttackAvoidLoss),
        0x213 => (0xcd, BattleFairyAttributeKind::AttackLoss),
        0x214 => (0xd7, BattleFairyAttributeKind::ElementModifyLoss),
        0x215 => (0xe5, BattleFairyAttributeKind::ElementAvoidLoss),
        0x216 => (0x80, BattleFairyAttributeKind::AttackAvoidGain),
        0x217 => (0x69, BattleFairyAttributeKind::AttackGain),
        0x218 => (0x73, BattleFairyAttributeKind::ElementModifyGain),
        0x219 => (0x81, BattleFairyAttributeKind::ElementAvoidGain),
        _ => return None,
    };
    Some(BattleFairyAttributeSkill { value_usage, kind })
}

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(
        player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost),
    );
}

fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    // Исходный CPlayer и его equipment[10] — единый захват шва
    // (`find_player` + slot 10 внутри `battle_fairy_equipment_addon`).
    let Some(marker) = game.battle_fairy_equipment_addon(player_id, GAP_BF_BATTLE_FAIRY)
    else { return false; };
    if marker != 1 {
        let text = game.battle_fairy_string(b"ZHGS0011");
        game.send_battle_fairy_notice(player_id, 0xffff_ffff, 0, text);
        return false;
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    if game.resolve_skill_sufferer(skill.lifecycle()).is_none() { return false; }
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return false;
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), delay, now(runtime)) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(player_id, b"ZHGS0048");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) != 0 {
        let Some(current) = game.battle_fairy_equipment_addon(player_id, GAP_BF_MP)
        else { return false; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, player_id, &properties);
            return false;
        }
        if skill_id == YUMO_SKILL_ID {
            let Some(_stored) = game.set_battle_fairy_equipment_addon(
                player_id, GAP_BF_MP, remaining,
            )
            else { return false; };
        }
    }
    true
}

/// Расход → OnChangeStates → BF918 (сериализация в момент отправки, отказ не
/// подавляет пакет; Pojia — до CAN/visual0, остальные — после advance).
fn send_goods_update<Game: BattleFairyGame>(game: &Game, player_id: i32, ex_id: CGuid) {
    // Между захватом предмета и этим чтением только OnChangeStates/visual:
    // они не меняют inventory. Slot10 остаётся тем же native CGoods, без
    // повторного допуска WarSoul или поиска одноимённого товара по GUID.
    if let Some((_, payload)) = game.battle_fairy_equipment_payload(player_id) {
        send_battle_fairy_goods_update(game, player_id, ex_id, &payload);
    }
}

pub fn execute_battle_fairy_attribute<Game: BattleFairyGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    let Some(definition) = definition(dispatch.skill_id()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, runtime, now),
        |game, instance, runtime| run_ai(game, instance, definition, runtime, now),
    )
}

fn run_ai<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, definition: BattleFairyAttributeSkill,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let (user_region, user) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(user_region, user)
        .map(|shape| shape.shape().identity()).filter(|user| user.object_type == 400)
    else { return BattleFairySkillOutcome::Pending; };
    let Some(sufferer) = game.resolve_skill_sufferer(skill.lifecycle()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if game.base_magic_target_dead(sufferer.0, sufferer.1) {
        game.update_registered_skill_visual(instance, 2);
        game.send_skill_system_info(user.id, b"ZHGS0050");
        return BattleFairySkillOutcome::RejectedAfterUse;
    }
    let Some(source_region) = game.find_player(user.id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then(|| player.shape().get_region_id())
    }) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        let Some((goods_guid, current)) = game
            .battle_fairy_equipment_identity_addon(user.id, GAP_BF_MP)
        else { return BattleFairySkillOutcome::Pending; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, user.id, &properties);
            return BattleFairySkillOutcome::Rejected;
        }
        let Some(_stored) = game.set_battle_fairy_equipment_addon(user.id, GAP_BF_MP, remaining)
        else { return BattleFairySkillOutcome::Pending; };
        let _ = game.publish_player_states(user.id);
        let goods_before_visual = skill_id == POJIA_SKILL_ID;
        if goods_before_visual { send_goods_update(game, user.id, goods_guid); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
        if !goods_before_visual { send_goods_update(game, user.id, goods_guid); }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return BattleFairySkillOutcome::Pending;
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if now(runtime) < started.wrapping_add(delay) {
        return BattleFairySkillOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    let (region, holder) = if definition.kind.targets_self() {
        (source_region, user)
    } else {
        sufferer
    };
    let _ = game.battle_fairy_end_first_state(region, holder, skill_id);
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let value = properties.query_property(definition.value_usage) as i32;
    let state = BattleFairyAttributeState::new(skill_id, definition.kind, keep, value);
    let begun = game.begin_battle_fairy_attribute_state(
        region, holder, user, state, &mut || now(runtime),
    );
    if begun || !definition.kind.targets_self() || skill_id == YUJIA_SKILL_ID {
        let _ = game.update_move_shape_properties(region, holder);
    }
    BattleFairySkillOutcome::Completed
}
