//! Восстановление здоровья игрока за ману боевого духа (`CWangsheng`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/wangsheng.cpp`. Мана owned боевого духа списывается и
//! рассылается до задержки; после задержки здоровье игрока увеличивается через
//! ограничивающий `SetHP`. Повтор при исчезнувшем equipment-owner-е, порядок
//! visual packets и отдельные часы восстановления сохранены. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; AI также сравнивает
//! unsigned now с wrapping(start + delay), cmp/jb 0x0051e00a.
//! После SetHP AI вызывает OnChangeStates (+0x164, 0x0051e043), без
//! UpdateProperty. Отказ Begin выдаёт action 3 перед внешним 4,2
//! планировщика; отказ уже начатого AI не получает повторного ответа.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::battlefairytransfer::{send_failure, send_goods_update, send_transfer_cast};
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, SkillExecutionKernel, SkillStage,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{
    BattleFairyManaSpendOutcome, BattleFairySkillDispatch,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const WANGSHENG_SKILL_ID: u32 = 0x221;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_HP_GAIN: u32 = 31;

const fn mana_cost_unavailable(current: i32, cost: u32) -> bool {
    (current.wrapping_sub(cost as i32)) < 0
}

pub(crate) fn execute_battle_fairy_wangsheng<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let skill_level = match dispatch {
        BattleFairySkillDispatch::SelfTarget {
            skill_id,
            skill_level,
            ..
        }
        | BattleFairySkillDispatch::Point {
            skill_id,
            skill_level,
            ..
        }
        | BattleFairySkillDispatch::Object {
            skill_id,
            skill_level,
            ..
        } if skill_id == WANGSHENG_SKILL_ID => skill_level,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let starting = player_ai.wangsheng().is_none();
    let reject_before_ai = |game: &mut CGame| {
        send_transfer_cast(game, player_id, WANGSHENG_SKILL_ID, skill_level, 3);
        if starting { send_failure(game, player_id, 2); }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let Some(properties) = game.skill_base_properties(WANGSHENG_SKILL_ID, skill_level) else {
        return reject_before_ai(game);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let hp_gain = properties.query_property(SKILL_USAGE_TARGET_HP_GAIN);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.wangsheng().is_none() {
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            player_ai.wangsheng_last_used_ms(),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            return reject_before_ai(game);
        }
        if mp_loss != 0
            && player
                .war_soul_mana(game.goods_factory())
                .is_some_and(|current| mana_cost_unavailable(current, mp_loss))
        {
            send_failure(game, player_id, 7);
            let text_cost = battle_fairy_mana_text_cost(mp_loss);
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", text_cost);
            return reject_before_ai(game);
        }
        player_ai.begin_wangsheng(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .wangsheng()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game
        .find_player(player_id)
        .and_then(|player| player.server_region_id())
        .is_none()
    {
        send_transfer_cast(game, player_id, WANGSHENG_SKILL_ID, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .wangsheng()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let Some(current_mana) = game
            .find_player(player_id)
            .and_then(|player| player.equipped_battle_fairy_mana(game.goods_factory()))
        else {
            return terminal(QueuedSkillExecutionState::Pending);
        };
        if mana_cost_unavailable(current_mana, mp_loss) {
            send_failure(game, player_id, 7);
            let text_cost = battle_fairy_mana_text_cost(mp_loss);
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", text_cost);
            send_transfer_cast(game, player_id, WANGSHENG_SKILL_ID, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let goods_factory = game.goods_factory().clone();
        let da_kong_key = game.globe_setup().da_kong_key();
        let spend = game
            .find_player_mut(player_id)
            .map(|player| {
                player.spend_equipped_battle_fairy_mana(
                    mp_loss,
                    &goods_factory,
                    da_kong_key,
                )
            })
            .unwrap_or(BattleFairyManaSpendOutcome::MissingEquipment);
        let update = match spend {
            BattleFairyManaSpendOutcome::MissingEquipment
            | BattleFairyManaSpendOutcome::SpentWithoutWarSoul => {
                return terminal(QueuedSkillExecutionState::Pending);
            }
            BattleFairyManaSpendOutcome::Spent { update } => update,
        };
        if let Some(update) = update.as_ref() {
            send_goods_update(game, update);
        } else {
            tracing::warn!(player_id, "не удалось сериализовать боевой дух после расхода маны");
        }
        send_transfer_cast(game, player_id, WANGSHENG_SKILL_ID, skill_level, 1);
        if let Some(state) = player_ai.wangsheng_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .wangsheng()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("исполнение восстановления здоровья создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_transfer_cast(game, player_id, WANGSHENG_SKILL_ID, skill_level, 2);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_health(player.health().wrapping_add(hp_gain));
    }
    let _ = game.publish_player_states(player_id);
    if let Some(state) = player_ai.wangsheng_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    send_transfer_cast(game, player_id, WANGSHENG_SKILL_ID, skill_level, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
