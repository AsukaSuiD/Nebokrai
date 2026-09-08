//! Общая вертикаль исполнения восьми атрибутных навыков боевого духа.
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! Конкретные идентификаторы и коды свойств принадлежат модулям навыков.
//! Этот владелец семейства сохраняет общий порядок: проверка экипировки,
//! задержки повторного применения и запаса MP, необратимое списание и
//! `0xBF918`, задержка, действия применения 2 и 3, замена состояния и
//! пересчёт свойств. Восстановление использует исходный абсолютный срок
//! `CSkill::IsRestored`; задержка AI также сравнивает unsigned now с
//! wrapping-суммой start + delay (CPojia 0x0052a89b, CYufa 0x00524266). `CGame`
//! используется только для разрешения владельцев и фактической доставки.
//! Источник: gameserver.exe + GameServer.pdb, CPojia..CYufa::AI
//! (0x00523fc0..0x0052aac0). Виртуальный End(+0x94) — 0x005246c0.
//! Отказы Po используют End(0); смерть GetSufferer у Yu вызывает End(1),
//! а отсутствие цели или недостаток MP — End(0). Получатель усиления Yu —
//! сам владелец, но GetSufferer остаётся объектом запроса: CYujia::Begin
//! (0x00526440) сохраняет оба аргумента через CAttackSkill::Begin.
//! После списания MP все восемь AI вызывают CPlayer::OnChangeStates (+0x164):
//! self BFE02 и командная публикация предшествуют visual и обновлению товара.
//! Pojia отправляет BF918 до start-visual (0x0052a827/0x0052a848), остальные
//! семь — после (например, Yujia 0x00526d66/0x00526e03). Сериализуется сам
//! equipment[10], без Wangsheng-проверки GetWarSoulGoods после списания.
//! Отказ Begin после собственного End(0) получает внешний 4,2 от
//! CPlayerAI::OnScheduleAboutWarSoul (0x00509861); отказ уже запущенного AI
//! этого ответа не получает, поскольку OnLoseTargetWarSoul видит IsEnded.

use super::battlefairyattributestate::{
    send_battle_fairy_attribute_state_visual, BattleFairyAttributeKind,
    BattleFairyAttributeState,
};
use super::kernel::{
    battle_fairy_mana_text_cost, skill_is_restored, SkillExecutionKernel, SkillStage,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{BattleFairyManaSpendOutcome, BattleFairySkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const VISUAL_OBJECT_TYPE: i32 = 700;

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

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn dispatch_fields(dispatch: BattleFairySkillDispatch) -> (u32, i32, Option<ShapeIdentity>) {
    match dispatch {
        BattleFairySkillDispatch::SelfTarget { skill_id, skill_level, .. }
        | BattleFairySkillDispatch::Point { skill_id, skill_level, .. } => {
            (skill_id, skill_level, None)
        }
        BattleFairySkillDispatch::Object { skill_id, skill_level, target } => {
            (skill_id, skill_level, Some(target))
        }
    }
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target: ShapeIdentity,
    skill_id: u32,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    let source = player.shape().identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    match action {
        1 => {
            message.add_byte(1);
            message.add_long(skill_id as i32);
            message.add_short(skill_level as i16);
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(player.shape().get_direction());
        }
        2 => {
            let (x, y) = game.move_shape_target_tile(player.server_region_id(), target)
                .unwrap_or_default();
            message.add_byte(2);
            message.add_long(skill_id as i32);
            message.add_short(skill_level as i16);
            message.add_long(source.object_type);
            message.add_long(source.id);
            message.add_long(target.object_type);
            message.add_long(target.id);
            message.add_long(x);
            message.add_long(y);
        }
        3 => {
            message.add_byte(3);
            message.add_long(skill_id as i32);
            message.add_short(skill_level as i16);
            message.add_long(VISUAL_OBJECT_TYPE);
            message.add_long(player_id);
            message.add_long(player.shape().get_direction());
        }
        _ => return,
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_goods_update(game: &mut CGame, update: &crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate) {
    let mut message = CMessage::new(update.message_type as i32);
    message.add_long(update.player_id);
    message.base_mut().add_guid(update.goods.ex_id);
    message.add_ulong(update.old_client_payload.len() as u32);
    message.base_mut().add(&update.old_client_payload);
    let _ = message.send_to_player(game.net_server(), update.player_id);
}

pub(crate) fn execute_battle_fairy_attribute<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (skill_id, skill_level, requested_target) = dispatch_fields(dispatch);
    let Some(definition) = definition(skill_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some((region_id, source_identity)) = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity()))
    }) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let starting = game.battle_fairy_execution(player_id, skill_id).is_none();
    let reject_before_ai = |game: &mut CGame, target: ShapeIdentity| {
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        if starting {
            game.send_battle_fairy_skill_failure(player_id, 2);
        }
        terminal(QueuedSkillExecutionState::Rejected)
    };
    let target = if definition.kind.targets_self() {
        source_identity
    } else {
        let Some(target) = requested_target.filter(|target| matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)) else {
            return reject_before_ai(game, source_identity);
        };
        target
    };
    if game.move_shape_target_tile(Some(region_id), target).is_none() {
        return reject_before_ai(game, target);
    }
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return reject_before_ai(game, target);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let value = properties.query_property(definition.value_usage) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.battle_fairy_execution(player_id, skill_id).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.battle_fairy_skill_last_used_ms(player_id, skill_id),
            delay_ms,
            now_ms,
        ) {
            game.send_battle_fairy_skill_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            return reject_before_ai(game, target);
        }
        let Some(current) = game.find_player(player_id).and_then(|player| player.war_soul_mana(game.goods_factory())) else {
            let text = game.get_string_by_id(b"ZHGS0011");
            let _ = colored_player_notice_message(0xffff_ffff, 0, text).send_to_player(game.net_server(), player_id);
            return reject_before_ai(game, target);
        };
        if current.wrapping_sub(mp_loss as i32) < 0 {
            game.send_battle_fairy_skill_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(
                player_id,
                b"ZHGS0052",
                battle_fairy_mana_text_cost(mp_loss),
            );
            return reject_before_ai(game, target);
        }
        game.begin_battle_fairy_state(player_id, player_ai, SkillExecutionKernel::begin(dispatch, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.battle_fairy_execution(player_id, skill_id).is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(sufferer) = requested_target
        .filter(|target| game.base_magic_target_view(region_id, *target).is_some())
    else {
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(region_id, sufferer) {
        game.send_battle_fairy_skill_failure(player_id, 2);
        game.send_skill_system_info(player_id, b"ZHGS0050");
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(if definition.kind.targets_self() {
            QueuedSkillExecutionState::RejectedAfterUse
        } else {
            QueuedSkillExecutionState::Rejected
        });
    }

    if game.battle_fairy_execution(player_id, skill_id).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let Some(current) = game.find_player(player_id).and_then(|player| player.equipped_battle_fairy_mana(game.goods_factory())) else {
            return terminal(QueuedSkillExecutionState::Pending);
        };
        if current.wrapping_sub(mp_loss as i32) < 0 {
            game.send_battle_fairy_skill_failure(player_id, 7);
            game.send_skill_system_info_with_unsigned(
                player_id,
                b"ZHGS0052",
                battle_fairy_mana_text_cost(mp_loss),
            );
            send_cast(game, player_id, target, skill_id, skill_level, 3);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let factory = game.goods_factory().clone();
        let da_kong_key = game.globe_setup().da_kong_key();
        let spend = game.find_player_mut(player_id).map(|player| {
            player.spend_attribute_skill_mana(mp_loss, &factory, da_kong_key)
        }).unwrap_or(BattleFairyManaSpendOutcome::MissingEquipment);
        let update = match spend {
            BattleFairyManaSpendOutcome::Spent { update } => update,
            BattleFairyManaSpendOutcome::MissingEquipment | BattleFairyManaSpendOutcome::SpentWithoutWarSoul => {
                return terminal(QueuedSkillExecutionState::Pending);
            }
        };
        let _ = game.publish_player_states(player_id);
        let goods_before_visual = skill_id == super::pojia::SKILL_ID;
        if goods_before_visual {
            if let Some(update) = update.as_ref() { send_goods_update(game, update); }
        }
        send_cast(game, player_id, target, skill_id, skill_level, 1);
        if let Some(state) = game.battle_fairy_execution_mut(player_id, skill_id) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
        if !goods_before_visual {
            if let Some(update) = update.as_ref() { send_goods_update(game, update); }
        }
    }

    let started_at_ms = game.battle_fairy_execution(player_id, skill_id).map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение атрибутного навыка создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_cast(game, player_id, target, skill_id, skill_level, 2);
    let state_started_at_ms = runtime.now_milliseconds();
    let state = BattleFairyAttributeState::new(skill_id, definition.kind, state_started_at_ms, keep_time_ms, value);
    let Some((previous, tile_x, tile_y)) = game.replace_battle_fairy_attribute_state(region_id, target, state) else {
        send_cast(game, player_id, target, skill_id, skill_level, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if let Some(previous) = previous {
        send_battle_fairy_attribute_state_visual(
            game, region_id, target, tile_x, tile_y, previous, false,
        );
    }
    send_battle_fairy_attribute_state_visual(
        game, region_id, target, tile_x, tile_y, state, true,
    );
    if target.object_type == PLAYER_TYPE {
        let _ = game.update_player_properties(target.id);
    }
    if let Some(execution) = game.battle_fairy_execution_mut(player_id, skill_id) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    send_cast(game, player_id, target, skill_id, skill_level, 3);
    terminal(QueuedSkillExecutionState::Completed)
}
