//! Фронтальный удар Мо-шоу (0x65).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/mosou.cpp.
//!
//! Зарегистрированный вход сохраняет исходного U, раннее время Begin и loop1
//! visual. Check требует reuse, меч категории 1 и ненулевую цену MP; нулевая
//! цена отклоняется молча. В первом AI MP списывается до OnChangeStates и
//! повторной проверки меча: отказ не возвращает уже потраченный ресурс.
//! Затем записывается CAN, живой S либо базовая точка определяет направление,
//! visual0 предшествует condition. Таблица свойств остаётся прежней через
//! callbacks этого AI, а каждый Calculate отдельно разрешает свою таблицу.
//!
//! Удар ждёт абсолютный unsigned срок start+delay, публикует visual1 и передаёт
//! живую лицевую клетку общему impactattack. Отсутствующий регион не отменяет
//! успешный End(1); внутренние отказы дают End(0). Общий End сбрасывает фазу
//! до возврата движения, AfterUse, reuse и удаления visual. Ручного повторного
//! завершения из command-tail нет, состояние принадлежит точному экземпляру.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::impactattack::run_mosou_attack;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const MOSOU_SKILL_ID: u32 = 0x65;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0287", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    if !weapon_is_sword(game, player) {
        failure(game, instance, player_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let mana = player.mana();
    let loss = properties.query_property(USER_MP_LOSE);
    if (mana.wrapping_sub(loss) as i32) < 0 {
        mana_failure(game, instance, player_id, &properties);
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                mana_failure(game, instance, user.1.id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_sword(game, player)) {
                failure(game, instance, user.1.id, 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            }
            None => skill.lifecycle().destination(),
        };
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    run_mosou_attack(game, instance, user, &properties, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_mosou<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != MOSOU_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Mosou,
        check_cast, |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
