//! Передача ресурсов CHuoxieshu/CLingzhishu: gameserver.exe/GameServer.pdb,
//! appserver/skills/{huoxieshu,lingzhishu}.cpp.
//!
//! Общий registered-вход сохраняет base Begin, visual loop1 и исходные часы;
//! Check требует CPlayer и ненулевой GetS, но не предмет или регион. Нулевая
//! цена допускается без чтения ресурса. Health оставляет одно HP, Mana в
//! Check допускает точную цену, а в AI также требует остаток не меньше одного.
//! Разность проверяется как signed DWORD, с исходным wrapping.
//!
//! AI больше не читает S и не проверяет смерть. Отсутствие CPlayer либо
//! equipment[10] оставляет ожидание; NULL региона U и нехватка ресурса дают
//! End(0). Расход → OnChangeStates → CAN → visual0 → condition; затем живой
//! condition и абсолютный start+delay. Отдельного начального clock нет.
//! Оба AI при нехватке используют ZHGS0052 с ценой без масштабирования,
//! даже Health; его Check использует ZHGS0054 с ценой плюс один.
//!
//! По сроку visual1 предшествует свежему GetWarSoulGoods. При отсутствии
//! предмета AI повторит этот visual в следующем такте без повторного расхода.
//! Восстановление читает current → поздний gain из таблицы начала AI → max,
//! при превышении максимума повторяет GetMax, затем пишет один раз.
//! Общий setter перезагружает уже существующие fairy-проекции до Serialize.
//! BF918 around отправляется и при отказе Serialize. Собственный End(bool)
//! с visual3 принадлежит координатору, не дублируется внешним End(int).

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::battlefairyskill::execute_registered_battle_fairy_state;
use super::huoxieshu::HUOXIESHU_SKILL_ID;
use super::kernel::{SkillStage, skill_is_restored};
use super::lingzhishu::LINGZHISHU_SKILL_ID;
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::container::cbattlefairycontainer::BattleFairyDefaultGoodsUpdate;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_BF_HP, GAP_BF_MAX_HP, GAP_BF_MAX_MP, GAP_BF_MP,
};
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
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

    const fn target_properties(self) -> (i32, i32) {
        match self {
            Self::Health => (GAP_BF_HP, GAP_BF_MAX_HP),
            Self::Mana => (GAP_BF_MP, GAP_BF_MAX_MP),
        }
    }

    const fn failure_action(self) -> u32 {
        match self {
            Self::Health => 6,
            Self::Mana => 7,
        }
    }

    fn cost_and_current(
        self, game: &CGame, player_id: i32, properties: &CSkillBaseProperties,
    ) -> Option<(u32, u32)> {
        // Health вызывает QueryProperty до виртуального GetHP; Mana читает
        // поле MP до QueryProperty. Этот порядок одинаков в Check и AI.
        Some(match self {
            Self::Health => {
                let cost = properties.query_property(self.cost_usage());
                (cost, game.find_player(player_id)?.health())
            }
            Self::Mana => {
                let current = game.find_player(player_id)?.mana();
                (properties.query_property(self.cost_usage()), current)
            }
        })
    }

    fn deduct(self, player: &mut CPlayer, remaining: u32) {
        match self {
            Self::Health => player.set_health(remaining),
            Self::Mana => player.set_mana(remaining),
        }
    }
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

fn fail_resource(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, kind: BattleFairyTransferKind, checking: bool,
) {
    game.update_registered_skill_visual(instance, kind.failure_action());
    let cost = properties.query_property(kind.cost_usage());
    let (text, amount) = if checking && kind == BattleFairyTransferKind::Health {
        (b"ZHGS0054".as_slice(), cost.wrapping_add(1))
    } else {
        (b"ZHGS0052".as_slice(), cost)
    };
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    kind: BattleFairyTransferKind, runtime: &mut Runtime,
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
    if properties.query_property(kind.cost_usage()) != 0 {
        let Some((cost, current)) = kind.cost_and_current(game, player_id, &properties) else {
            return false;
        };
        let reserve = u32::from(kind == BattleFairyTransferKind::Health);
        if (current.wrapping_sub(cost).wrapping_sub(reserve) as i32) < 0 {
            fail_resource(game, instance, player_id, &properties, kind, true);
            return false;
        }
    }
    true
}

fn restore_goods_resource(
    game: &mut CGame, player_id: i32, kind: BattleFairyTransferKind,
    properties: &CSkillBaseProperties,
) -> Option<BattleFairyDefaultGoodsUpdate> {
    let factory = game.goods_factory().clone();
    let (property, maximum_property) = kind.target_properties();
    let goods = game.find_player(player_id)?.war_soul_goods(&factory)?;
    let current = goods.addon_property_value(&factory, property, 1) as u32;
    let gain = properties.query_property(kind.gain_usage());
    let mut restored = current.wrapping_add(gain);
    let maximum = goods.addon_property_value(&factory, maximum_property, 1) as u32;
    if maximum < restored {
        restored = goods.addon_property_value(&factory, maximum_property, 1) as u32;
    }
    let da_kong_key = game.globe_setup().da_kong_key();
    let _ = game.set_player_equipment_addon_property(player_id, 10, property, 1, restored as i32)?;
    let goods = game.find_player(player_id)?.equipment().get_goods(10)?;
    let mut old_client_payload = Vec::new();
    let _ = goods.serialize_for_old_client(&mut old_client_payload, &factory, da_kong_key);
    Some(BattleFairyDefaultGoodsUpdate {
        message_type: 0x0b_f918, player_id, goods: goods.identity(), old_client_payload,
    })
}

pub(crate) fn execute_battle_fairy_transfer<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, kind: BattleFairyTransferKind, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != kind.skill_id() {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, None,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, kind, runtime),
        |game, instance, runtime| run_ai(game, instance, kind, runtime),
    )
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, kind: BattleFairyTransferKind,
    runtime: &mut Runtime,
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
    let Some(user) = resolve_state_move_shape(game, region, user)
        .map(|shape| shape.shape().identity()).filter(|user| user.object_type == 400)
    else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
    let Some(player) = game.find_player(user.id) else {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    };
    if !player.shape().is_assigned_to_server_region() {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    if skill.execution_stage() == Some(SkillStage::Begin) {
        if player.equipment().get_goods(10).is_none() {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        let Some((cost, current)) = kind.cost_and_current(game, user.id, &properties) else {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        };
        let remaining = current.wrapping_sub(cost);
        if (remaining.wrapping_sub(1) as i32) < 0 {
            fail_resource(game, instance, user.id, &properties, kind, false);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(user.id) { kind.deduct(player, remaining); }
        let _ = game.publish_player_states(user.id);
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
    let Some(update) = restore_goods_resource(game, user.id, kind, &properties) else {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    };
    send_goods_update(game, &update);
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}
