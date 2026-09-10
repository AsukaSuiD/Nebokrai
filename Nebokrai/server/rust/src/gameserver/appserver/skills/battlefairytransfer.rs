//! Общий runtime двух навыков передачи ресурсов боевому духу.
//! Успешный Begin возвращает Begun до первого AI; общий координатор
//! продолжает тот же owner без повторного допуска расписания.
//!
//! `CHuoxieshu` и `CLingzhishu` имеют одинаковые стадии, visual packet и
//! повтор при временном отсутствии equipment-owner-а. Раздельными остаются
//! source/target property, signed-проверки и тексты ошибок. Один
//! `SkillExecutionKernel` хранит только команду, стадии и исходные часы.
//! Источник: gameserver.exe + GameServer.pdb, CHuoxieshu::AI (0x0051d180)
//! и CLingzhishu::AI (0x0051c550). Ожидание сравнивает unsigned now с
//! wrapping(start + delay), cmp/jb 0x0051d2bf/0x0051c680. После расхода
//! ресурса вызывается OnChangeStates (+0x164), не UpdateProperty.
//! Отказ Begin завершает visual перед внешним 4,2 планировщика; AI-отказ
//! заканчивается End(0) без повторного общего ответа.
//! В Rust внешний 4,2 принадлежит только координатору после общего End(0),
//! а этот владелец передаёт собственные ошибки и режимы 0/1 общему visual-dispatch.
//! CHuoxieshuEffect/CLingzhishuEffect (Update 0x0051cb80/0x0051bf60) создаются
//! в Begin после базового вызова: 0x0c, BeginVisualEffect(1), затем CheckCondition.
//! Их mode 1 пишет реальный тип и ID GetUser дважды, затем его координаты.
//! Собственный End(bool,+0x94) 0x0051be50 вызывает Update(3) без повторного Begin,
//! затем наследуемый End(int). Этот хвост concrete Begin/AI выполняет общий
//! координатор; внешний int-слот +0x68 не превращается в вызов bool-перегрузки.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::huoxieshu::HUOXIESHU_SKILL_ID;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage};
use super::lingzhishu::LINGZHISHU_SKILL_ID;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const SKILL_USAGE_USER_HP_LOSE: u32 = 1;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_HP_GAIN: u32 = 31;
const SKILL_USAGE_TARGET_MP_GAIN: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyTransferKind {
    Health,
    Mana,
}

impl BattleFairyTransferKind {
    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Health => HUOXIESHU_SKILL_ID,
            Self::Mana => LINGZHISHU_SKILL_ID,
        }
    }

    const fn cost_usage(self) -> u32 {
        match self {
            Self::Health => SKILL_USAGE_USER_HP_LOSE,
            Self::Mana => SKILL_USAGE_USER_MP_LOSE,
        }
    }

    const fn gain_usage(self) -> u32 {
        match self {
            Self::Health => SKILL_USAGE_TARGET_HP_GAIN,
            Self::Mana => SKILL_USAGE_TARGET_MP_GAIN,
        }
    }

    const fn failure_action(self) -> u32 {
        match self {
            Self::Health => 6,
            Self::Mana => 7,
        }
    }

    const fn source(self, player: &CPlayer) -> u32 {
        match self {
            Self::Health => player.health(),
            Self::Mana => player.mana(),
        }
    }

    const fn initial_cost_unavailable(self, current: u32, cost: u32) -> bool {
        if cost == 0 {
            return false;
        }
        match self {
            Self::Health => (current.wrapping_sub(cost).wrapping_sub(1) as i32) < 0,
            Self::Mana => (current.wrapping_sub(cost) as i32) < 0,
        }
    }

    const fn runtime_cost_unavailable(self, current: u32, cost: u32) -> bool {
        (current.wrapping_sub(cost).wrapping_sub(1) as i32) < 0
    }

    fn deduct(self, player: &mut CPlayer, current: u32, cost: u32) {
        match self {
            Self::Health => player.set_health(current.wrapping_sub(cost)),
            Self::Mana => player.set_mana(current.wrapping_sub(cost)),
        }
    }
}

pub(super) fn send_failure(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    mode: u32,
) {
    game.update_player_skill_visual(player_id, skill_id, mode);
}

pub(super) fn send_goods_update(
    game: &mut CGame,
    update: &crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate,
) {
    let mut message = CMessage::new(update.message_type as i32);
    message.add_long(update.player_id);
    message.base_mut().add_guid(update.goods.ex_id);
    message.add_ulong(update.old_client_payload.len() as u32);
    message.base_mut().add(&update.old_client_payload);
    let _ = game.send_player_shape_around(update.player_id, None, &message);
}

pub(crate) fn execute_battle_fairy_transfer<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    kind: BattleFairyTransferKind,
    _player_ai: &mut CPlayerAI,
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
        } if skill_id == kind.skill_id() => skill_level,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(properties) = game.skill_base_properties(kind.skill_id(), skill_level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let cost = properties.query_property(kind.cost_usage());
    let gain = properties.query_property(kind.gain_usage());
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.battle_fairy_execution(player_id, kind.skill_id()).is_none() {
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.battle_fairy_skill_last_used_ms(player_id, kind.skill_id()),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, kind.skill_id(), 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if kind.initial_cost_unavailable(kind.source(player), cost) {
            send_failure(game, player_id, kind.skill_id(), kind.failure_action());
            let (string_id, value) = match kind {
                BattleFairyTransferKind::Health => (b"ZHGS0054".as_slice(), cost.wrapping_add(1)),
                BattleFairyTransferKind::Mana => (b"ZHGS0052".as_slice(), cost),
            };
            game.send_skill_system_info_with_unsigned(player_id, string_id, value);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        game.begin_battle_fairy_state(player_id, SkillExecutionKernel::begin(
            dispatch,
            started_at_ms,
        ));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.battle_fairy_execution(player_id, kind.skill_id())
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game
        .find_player(player_id)
        .and_then(|player| player.server_region_id())
        .is_none()
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.battle_fairy_execution(player_id, kind.skill_id())
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if player.equipment().get_goods(10).is_none() {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        let current = kind.source(player);
        if kind.runtime_cost_unavailable(current, cost) {
            send_failure(game, player_id, kind.skill_id(), kind.failure_action());
            game.send_skill_system_info_with_unsigned(player_id, b"ZHGS0052", cost);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            kind.deduct(player, current, cost);
        }
        let _ = game.publish_player_states(player_id);
        game.update_player_skill_visual(player_id, kind.skill_id(), 0);
        if let Some(state) = game.battle_fairy_execution_mut(player_id, kind.skill_id()) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.battle_fairy_execution(player_id, kind.skill_id())
        .map(SkillExecutionKernel::started_at_ms)
        .expect("исполнение передачи ресурса создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let goods_factory = game.goods_factory().clone();
    let da_kong_key = game.globe_setup().da_kong_key();
    if game
        .find_player(player_id)
        .and_then(|player| player.war_soul_goods(&goods_factory))
        .is_none()
    {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_player_skill_visual(player_id, kind.skill_id(), 1);
    let update = game.find_player_mut(player_id).and_then(|player| match kind {
        BattleFairyTransferKind::Health => {
            player.restore_war_soul_health(gain, &goods_factory, da_kong_key)
        }
        BattleFairyTransferKind::Mana => {
            player.restore_war_soul_mana(gain, &goods_factory, da_kong_key)
        }
    });
    if let Some(update) = update.as_ref() {
        send_goods_update(game, update);
    } else {
        tracing::warn!(player_id, skill_id = kind.skill_id(), "не удалось сериализовать боевой дух после передачи ресурса");
    }
    if let Some(state) = game.battle_fairy_execution_mut(player_id, kind.skill_id()) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
