//! Защитная стойка CPillar (0x74).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/pillar.cpp.
//!
//! Зарегистрированный Attack Begin сохраняет исходного U, раннее время
//! и visual loop1. Check проверяет только reuse и запрещает движение;
//! MP и оружие здесь не проверяются. Отказ Begin получает End(0).
//! Каждый AI сохраняет таблицу свойств и найденного U через callbacks.
//! Смерть U публикует visual2 и даёт End(1); недостаток MP — visual7/End(0).
//! Первый AI списывает MP со signed wrapping-проверкой до OnChangeStates,
//! затем записывает CAN, публикует visual0 и включает condition.
//!
//! Абсолютный unsigned срок start+delay предшествует visual1. Pillar —
//! переключатель: первый слот ID74 снимается без создания нового состояния.
//! Только при отсутствии такого слота читаются factor, затем persist;
//! PillarState получает Begin(U,U), append и отдельный UpdateProperty.
//! Беззнаковый factor умножается на сохранённую f32-константу 0.001
//! в расширенной точности до записи f32. Состояние владеет своим запретом
//! движения; общий End сбрасывает фазу до возврата движения свежему U
//! и выполняет унаследованный State End с исходным аргументом.
//! Ресурсы CPlayer не читаются у чужого живого CMoveShape: исходная ветка
//! использует непроверенное приведение, которое безопасная модель отклоняет.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::pillarstate::{PillarState, toggle_pillar_state};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const PILLAR_SKILL_ID: u32 = 0x74;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if source.1.object_type == PLAYER_TYPE { game.send_skill_system_info(source.1.id, b"GS0278"); }
        return false;
    }
    let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    source.set_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
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
        let Some(player) = game.find_player(source.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let mana = player.mana();
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, source.1.id, &properties);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
        game.publish_player_states(source.1.id);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    let _ = toggle_pillar_state(game, source, |_game| {
        let factor = (f64::from(properties.query_property(TARGET_DAMAGE_FACTOR)) * f64::from(0.001_f32)) as f32;
        let keep = properties.query_property(STATE_PERSIST_TIME);
        Some(PillarState::new(keep, factor))
    }, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_pillar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != PILLAR_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_cast(game, instance, source, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
