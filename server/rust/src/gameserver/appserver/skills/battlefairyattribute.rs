//! Атрибутные навыки CPojia..CYufa, gameserver.exe/GameServer.pdb,
//! appserver/skills/{pojia,pobing,pomo,pofa,yujia,yubing,yumo,yufa}.cpp.
//!
//! Координатор выполняет общий Begin с visual loop1 и завершает тот же
//! зарегистрированный экземпляр. Check принимает исходного CPlayer,
//! требует equipment[10] с маркером боевой феи и ненулевой GetSufferer; тип,
//! здоровье и дальность цели здесь не проверяются. Восстановление использует
//! delay, а не reuse. MP0 допускается без чтения MP. Только Yumo списывает
//! ненулевую стоимость уже в Check; первый AI затем списывает её повторно.
//!
//! AI сохраняет таблицу свойств и фактические U/S до побочных эффектов.
//! Отсутствие CPlayer или equipment[10] оставляет ожидание; смерть S у всех
//! восьми завершает с End(1), отсутствие S/региона U и нехватка MP — End(0).
//! OnChangeStates следует после записи MP. Pojia сериализует и отправляет
//! BF918 до CAN/visual0/condition, остальные семь — после. Отказ Serialize
//! не подавляет пакет. Дальше перечитывается condition и проверяется
//! абсолютный wrapping-срок start + delay, без нового начального clock.
//!
//! Первый прежний exact-ID проходит End и destructor свежего остатка слота.
//! Только затем из сохранённой таблицы читаются keep/value: primary Begin
//! с собственными часами и silent loop1 → append. Po держит состояние на S
//! с state-U=S/state-S=U; Yu — на U с обоими указателями U. Внешний Update
//! обязателен у Po и Yujia, у остальных Yu — лишь после успешного Begin.
//! Собственный End(bool) остаётся у координатора и отличается от внешнего
//! унаследованного End(int); ручного visual или второго завершения здесь нет.

use super::battlefairyattributestate::{
    BattleFairyAttributeKind, BattleFairyAttributeState, begin_battle_fairy_attribute_state,
};
use super::battlefairyskill::execute_registered_battle_fairy_state;
use super::kernel::{
    SkillStage, battle_fairy_mana_text_cost, skill_is_restored,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BF_BATTLE_FAIRY, GAP_BF_MP,
};
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyAttributeSkill {
    pub(crate) value_usage: u32,
    pub(crate) kind: BattleFairyAttributeKind,
}

pub(crate) const fn definition(skill_id: u32) -> Option<BattleFairyAttributeSkill> {
    match skill_id {
        super::pojia::SKILL_ID => Some(super::pojia::DEFINITION),
        super::pobing::SKILL_ID => Some(super::pobing::DEFINITION),
        super::pomo::SKILL_ID => Some(super::pomo::DEFINITION),
        super::pofa::SKILL_ID => Some(super::pofa::DEFINITION),
        super::yujia::SKILL_ID => Some(super::yujia::DEFINITION),
        super::yubing::SKILL_ID => Some(super::yubing::DEFINITION),
        super::yumo::SKILL_ID => Some(super::yumo::DEFINITION),
        super::yufa::SKILL_ID => Some(super::yufa::DEFINITION),
        _ => None,
    }
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

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(goods) = game.find_player(player_id)
        .and_then(|player| player.equipment().get_goods(10))
    else { return false; };
    if goods.addon_property_value(game.goods_factory(), GAP_BF_BATTLE_FAIRY, 1) != 1 {
        let text = game.get_string_by_id(b"ZHGS0011");
        let _ = colored_player_notice_message(0xffff_ffff, 0, text)
            .send_to_player(game.net_server(), player_id);
        return false;
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    if resolve_skill_sufferer(game, skill.lifecycle()).is_none() { return false; }
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return false;
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), delay, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(player_id, b"ZHGS0048");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) != 0 {
        let current = goods.addon_property_value(game.goods_factory(), GAP_BF_MP, 1);
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, player_id, &properties);
            return false;
        }
        if skill_id == super::yumo::SKILL_ID {
            let Some(_stored) = game.set_player_equipment_addon_property(
                player_id, 10, GAP_BF_MP, 1, remaining,
            )
            else { return false; };
        }
    }
    true
}

fn send_goods_update(game: &CGame, player_id: i32, identity: ShapeIdentity) {
    // Между захватом предмета и этим чтением только OnChangeStates/visual:
    // они не меняют inventory. Slot10 остаётся тем же native CGoods, без
    // повторного допуска WarSoul или поиска одноимённого товара по GUID.
    let Some(goods) = game.find_player(player_id)
        .and_then(|player| player.equipment().get_goods(10))
    else { return; };
    let mut payload = Vec::new();
    let _ = goods.serialize_for_old_client(
        &mut payload, game.goods_factory(), game.globe_setup().da_kong_key(),
    );
    let mut message = CMessage::new(0x0b_f918);
    message.add_long(player_id);
    message.base_mut().add_guid(identity.ex_id);
    message.add_ulong(payload.len() as u32);
    message.base_mut().add(&payload);
    let _ = message.send_to_player(game.net_server(), player_id);
}

pub(crate) fn execute_battle_fairy_attribute<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(definition) = definition(dispatch.skill_id()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None, check_cast,
        |game, instance, runtime| run_ai(game, instance, definition, runtime),
    )
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, definition: BattleFairyAttributeSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let (user_region, user) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, user_region, user)
        .map(|shape| shape.shape().identity()).filter(|user| user.object_type == 400)
    else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
    let Some(sufferer) = resolve_skill_sufferer(game, skill.lifecycle()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(sufferer.0, sufferer.1) {
        game.update_registered_skill_visual(instance, 2);
        game.send_skill_system_info(user.id, b"ZHGS0050");
        return state_skill_outcome(QueuedSkillExecutionState::RejectedAfterUse);
    }
    let Some(source_region) = game.find_player(user.id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then(|| player.shape().get_region_id())
    }) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage() == Some(SkillStage::Begin) {
        let Some((goods_identity, current)) = game.find_player(user.id)
            .and_then(|player| player.equipment().get_goods(10))
            .map(|goods| (goods.identity(), goods.addon_property_value(game.goods_factory(), GAP_BF_MP, 1)))
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, user.id, &properties);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(_stored) = game.set_player_equipment_addon_property(
            user.id, 10, GAP_BF_MP, 1, remaining,
        )
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
        let _ = game.publish_player_states(user.id);
        let goods_before_visual = skill_id == super::pojia::SKILL_ID;
        if goods_before_visual { send_goods_update(game, user.id, goods_identity); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
        if !goods_before_visual { send_goods_update(game, user.id, goods_identity); }
    }
    if game.registered_skill(instance).is_none_or(|skill| skill.execution_stage() != Some(SkillStage::Check)) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    let (region, holder) = if definition.kind.targets_self() {
        (source_region, user)
    } else {
        sufferer
    };
    if let Some((position, _)) = resolve_state_move_shape(game, region, holder)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == skill_id))
    {
        let _ = end_and_destroy_state_at(game, region, holder, position);
    }
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let value = properties.query_property(definition.value_usage) as i32;
    let state = BattleFairyAttributeState::new(skill_id, definition.kind, keep, value);
    let begun = begin_battle_fairy_attribute_state(
        game, region, holder, user, state, &mut || runtime.now_milliseconds(),
    );
    if begun || !definition.kind.targets_self() || skill_id == super::yujia::SKILL_ID {
        let _ = game.update_move_shape_properties(region, holder);
    }
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}
