//! Зарегистрированное применение Agility, Agility2, Natural, Rapture, DaubPoison
//! и двух щитов ManaShield/MachineShield.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые appserver/skills owners.
//! Общий Begin сохраняет исходную команду и часы до loop1 visual; Check
//! получает исходного U и при отказе вызывает End(0) без второго visual2.
//! У пяти усилений после reuse источник не типа Player проходит без Move0,
//! а у игрока MP0 означает тихий отказ. Щиты требуют Player; MP0 не читает
//! ману, но допускает Move0. Положительная цена проверяется по знаку DWORD
//! разности перед Move0. Natural, DaubPoison и щиты используют GS0288,
//! остальные усиления — GS0279.
//!
//! Каждый AI удерживает одну таблицу и найденного U через callbacks. Пять
//! усилений при смерти U вызывают visual2/End(1); щиты смерть не проверяют.
//! Первый AI читает MP, заново спрашивает цену, выполняет signed-проверку
//! и SetMP→OnChangeStates→CAN→visual0→condition. Небезопасный native доступ
//! к ресурсам отклоняется для источника не типа Player. Выпуск ждёт
//! unsigned start+delay и публикует visual1 без
//! раннего S/region-gate. Замена состояния использует захваченного U и таблицу;
//! её результат не отменяет End(1). Только владельцы состояний определяют
//! порядок удаления, свойства, часы Begin, append и UpdateProperty.
//!
//! Playercast публикует единственное исполнение на весь AI. Обычный kernel
//! хранит U/S, phase, CAN и время; отдельного payload или набора флагов нет.
//! End сбрасывает фазу до свежего U Move1 и общего State End с исходным аргументом.

use super::agility::apply_agility_state;
use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::daubpoison::{DAUB_POISON_SKILL_ID, apply_daub_poison};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::natural::NATURAL_SKILL_ID;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::terminal;
use super::selfshield::{apply_self_shield_state, is_self_shield_skill};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;

fn mana_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    skill_id: u32, properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    let message = if matches!(skill_id, NATURAL_SKILL_ID | DAUB_POISON_SKILL_ID) || is_self_shield_skill(skill_id) {
        b"GS0288"
    } else { b"GS0279" };
    game.send_skill_system_info_with_unsigned(player_id, message, amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player_id = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let skill_id = skill.id();
    let shield = is_self_shield_skill(skill_id);
    if shield && player_id.is_none() { return false; }
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player_id) = player_id { game.send_skill_system_info(player_id, b"GS0278"); }
        return false;
    }
    let Some(player_id) = player_id else { return true; };
    if properties.query_property(USER_MP_LOSE) == 0 {
        if !shield { return false; }
    } else {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else { return false; };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, player_id, skill_id, &properties);
            return false;
        }
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
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !is_self_shield_skill(skill_id) && game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    }
    if stage == SkillStage::Begin {
        if source.1.object_type != PLAYER_TYPE { return terminal(QueuedSkillExecutionState::Rejected); }
        let Some(mana) = game.find_player(source.1.id).map(CPlayer::mana) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, source.1.id, skill_id, &properties);
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
    if skill_id == DAUB_POISON_SKILL_ID { apply_daub_poison(game, source, &properties, runtime); }
    else if is_self_shield_skill(skill_id) { apply_self_shield_state(game, source, skill_id, &properties, runtime); }
    else { apply_agility_state(game, source, skill_id, &properties, runtime); }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_self_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_cast(game, instance, source, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_ai,
    )
}
