//! Зарегистрированный вход CAgility, CAgility2, CNatural и CRapture.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые appserver/skills owners.
//! Attack Begin сохраняет исходного U, ранний отсчёт и loop1 visual; отказ
//! Check получает End(0) без дополнительного visual2. После reuse чужой
//! CMoveShape допускается без Move0, но игроку требуется ненулевая цена MP:
//! MP0 даёт тихий отказ. Ненулевой расход проверяется по знаку DWORD-разности
//! перед Move0. Natural использует GS0288, остальные варианты — GS0279.
//!
//! Каждый AI сохраняет свежую таблицу свойств и найденного U через callbacks.
//! Смерть U публикует visual2 и завершает End(1); нехватка MP — visual7/End(0).
//! Первый AI всегда списывает MP, затем вызывает OnChangeStates, записывает
//! CAN, публикует visual0 и включает condition. Непроверенный native доступ
//! к ресурсам CPlayer безопасно отклоняется для чужого живого CMoveShape.
//! Абсолютный unsigned срок start+delay предшествует visual1, без раннего
//! region/S-gate; S и координаты команды не определяют получателя состояния.
//!
//! Постоянные варианты удаляют все встреченные DA/DB/DC живым индексным
//! обходом; Agility2 заменяет только первый ID81. Frozen-таблица отдаёт бонус
//! после завершения старых состояний; только Agility2 затем читает persist.
//! Begin(U,U), visual, append и безусловный UpdateProperty принадлежат владельцу
//! состояния. Отдельных публикаций после замены нет. Общий End сбрасывает фазу,
//! возвращает движение свежему U и завершает State с настоящим аргументом.

pub(crate) use super::agility2::AGILITY_2_SKILL_ID;
use super::agilitystate::{PersistentAgilityFamilyState, replace_persistent_agility_state};
use super::agilitystate2::{AgilityState2, replace_agility_state_2};
use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::natural::{NATURAL_SKILL_ID, SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN};
use super::playercast::execute_registered_player_cast;
use super::rapture::{RAPTURE_SKILL_ID, SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const AGILITY_SKILL_ID: u32 = 0xda;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const TARGET_FULL_MISS_GAIN: u32 = 127;
const STATE_PERSIST_TIME: u32 = 10_002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AgilityProfile { Agility, Agility2, Natural, Rapture }

impl AgilityProfile {
    fn from_skill_id(skill_id: u32) -> Option<Self> {
        match skill_id {
            AGILITY_SKILL_ID => Some(Self::Agility),
            AGILITY_2_SKILL_ID => Some(Self::Agility2),
            NATURAL_SKILL_ID => Some(Self::Natural),
            RAPTURE_SKILL_ID => Some(Self::Rapture),
            _ => None,
        }
    }

    fn mana_failure(self) -> &'static [u8] {
        if self == Self::Natural { b"GS0288" } else { b"GS0279" }
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn mana_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, profile: AgilityProfile,
) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, profile.mana_failure(), amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    profile: AgilityProfile, runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player_id = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player_id) = player_id { game.send_skill_system_info(player_id, b"GS0278"); }
        return false;
    }
    let Some(player_id) = player_id else { return true; };
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else { return false; };
    let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
    if (remaining as i32) < 0 {
        mana_failure(game, instance, player_id, &properties, profile);
        return false;
    }
    let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    source.set_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, profile: AgilityProfile, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    }
    if stage == SkillStage::Begin {
        if source.1.object_type != PLAYER_TYPE { return terminal(QueuedSkillExecutionState::Rejected); }
        let Some(mana) = game.find_player(source.1.id).map(CPlayer::mana) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, source.1.id, &properties, profile);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player_mut(source.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
        player.set_mana(remaining);
        game.publish_player_states(source.1.id);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    if profile == AgilityProfile::Agility2 {
        let _ = replace_agility_state_2(game, source, || {
            let full_miss = properties.query_property(TARGET_FULL_MISS_GAIN) as u16;
            let keep = properties.query_property(STATE_PERSIST_TIME) as i32;
            AgilityState2::new(full_miss, 0, keep)
        }, &mut || runtime.now_milliseconds());
    } else {
        let _ = replace_persistent_agility_state(game, source, || match profile {
            AgilityProfile::Natural => PersistentAgilityFamilyState::Natural {
                element_resistance_gain: properties.query_property(SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN) as u16,
            },
            AgilityProfile::Rapture => PersistentAgilityFamilyState::Rapture {
                blast_attack_gain: properties.query_property(SKILL_USAGE_TARGET_BLAST_COEFFICIENT_GAIN) as u16,
            },
            AgilityProfile::Agility | AgilityProfile::Agility2 => PersistentAgilityFamilyState::Agility {
                full_miss: properties.query_property(TARGET_FULL_MISS_GAIN) as u16,
            },
        }, &mut || runtime.now_milliseconds());
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_agility_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(profile) = AgilityProfile::from_skill_id(dispatch.skill_id()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_cast(game, instance, source, profile, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        |game, instance, runtime| run_ai(game, instance, profile, runtime),
    )
}
