//! Восстановление здоровья CWangsheng (0x221), gameserver.exe/GameServer.pdb,
//! appserver/skills/wangsheng.cpp.
//!
//! Общий вход сохраняет зарегистрированный экземпляр и часы base Begin.
//! Check принимает исходного игрока, требует GetS и проверяет reuse; MP0
//! допускается без предмета, но ненулевая цена требует GetWarSoulGoods.
//! Первый AI разрешает свежего U, требует его region-link, но не проверяет S
//! или смерть. Подтверждённый State owner 0x221 этим навыком не создаётся.
//!
//! AI сначала списывает MP у equipment[10] через общий setter с reload
//! существующих fairy-проекций, затем повторно проверяет
//! GetWarSoulGoods. Отказ сохраняет списание и ожидание без отката.
//! Serialize не подавляет BF918 при false; после отправки CAN/visual0/condition
//! предшествуют абсолютной задержке. После visual1 читаются HP и величина
//! лечения: wrapping-сумма проходит SetHP и OnChangeStates. Единственный
//! собственный End(bool), отличный от внешнего End(int), выполняет координатор.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::battlefairyskill::execute_registered_battle_fairy_state;
use super::battlefairytransfer::send_goods_update;
use super::kernel::{SkillStage, battle_fairy_mana_text_cost, skill_is_restored};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_MP;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use nebokrai_zone::skills::wangsheng_restored_health;

pub(crate) use nebokrai_zone::skills::WANGSHENG_SKILL_ID;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;

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
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    if resolve_skill_sufferer(game, skill.lifecycle()).is_none() { return false; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(player_id, b"ZHGS0048");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) != 0 {
        let Some(current) = game.find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else { return false; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(cost as i32) < 0 {
            fail_mana(game, instance, player_id, &properties);
            return false;
        }
    }
    true
}

pub(crate) fn execute_battle_fairy_wangsheng<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != WANGSHENG_SKILL_ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None, check_cast, run_ai,
    )
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
    let (region, user) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, user)
        .map(|shape| shape.shape().identity()).filter(|source| source.object_type == 400)
    else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
    if game.find_player(source.id).is_none_or(|player| {
        !player.move_shape().shape().is_assigned_to_server_region()
    }) {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    if skill.execution_stage() == Some(SkillStage::Begin) {
        let Some(current) = game.find_player(source.id)
            .and_then(|player| player.equipment().get_goods(10))
            .map(|goods| goods.addon_property_value(game.goods_factory(), GAP_BF_MP, 1))
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        let remaining = current.wrapping_sub(cost as i32);
        if remaining < 0 {
            fail_mana(game, instance, source.id, &properties);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(_stored) = game.set_player_equipment_addon_property(
            source.id, 10, GAP_BF_MP, 1, remaining,
        )
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };

        // Маркер проверяется после записи: неподходящий предмет теряет MP,
        // но condition не устанавливается, и следующий AI повторяет попытку.
        let Some(goods) = game.find_player(source.id)
            .and_then(|player| player.war_soul_goods(game.goods_factory()))
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
        let mut old_client_payload = Vec::new();
        let _ = goods.serialize_for_old_client(
            &mut old_client_payload, game.goods_factory(), game.globe_setup().da_kong_key(),
        );
        let update = BattleFairyDefaultGoodsUpdate {
            message_type: 0x0b_f918,
            player_id: source.id,
            goods: goods.identity(),
            old_client_payload,
        };
        send_goods_update(game, &update);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
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
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    let Some(current) = game.find_player(source.id).map(|player| player.health()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let restored = wangsheng_restored_health(current, |key| properties.query_property(key));
    if let Some(player) = game.find_player_mut(source.id) {
        player.set_health(restored);
    }
    let _ = game.publish_player_states(source.id);
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}
