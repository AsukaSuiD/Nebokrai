//! Семейство периодических ударов листвы LeafCut/LeafCut2/LeafCut3 (0x6B/0x80/0x8F).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/leafcut*.cpp.
//!
//! Общий зарегистрированный Begin сохраняет раннее время и loop1 visual;
//! отказ Check дополнительно посылает visual2 перед End(0). Объектный Check
//! использует исходный S, координатный разрешает S после начала visual.
//! Допуск проверяет цель, reuse, свежий путь, меч категории 2 и signed MP;
//! только LeafCut требует ещё RP. Нулевая цена в Check не читает ресурс.
//!
//! AI сохраняет исходные U/S и таблицу свойств через callbacks. Смерть S
//! проверяется перед расходом; MP, затем RP первого варианта списываются до
//! OnChangeStates и повторной проверки меча. После CAN идут живые координаты
//! исходных S/U, направление, visual0 и condition. Удар ждёт абсолютный
//! unsigned срок start+delay, возвращает движение и заново строит путь по
//! текущей базе навыка. Его карта берётся из региона Begin, не текущего U.
//! Visual1 разрешает свежую S, а наложение получает исходную S этого AI.
//! Leafcutapply сохраняет различия замены состояний и округление параметров.
//! После наложения IncreaseRp(true,0) предшествует End(1), независимо от
//! успеха Begin состояния. Общий End сбрасывает фазу до движения/AfterUse;
//! весь AI опубликован у игрока и не извлекает каноническое исполнение.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::leafcut2::LEAF_CUT_2_SKILL_ID;
use super::leafcut3::LEAF_CUT_3_SKILL_ID;
use super::leafcutapply::apply_leaf_cut_family;
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) use nebokrai_zone::effects::LEAF_CUT_STATE_ID as LEAF_CUT_SKILL_ID;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, source: ShapeIdentity, code: u32) {
    game.update_registered_skill_visual(instance, code);
    if source.object_type != PLAYER_TYPE { return; }
    let text: &[u8] = match code {
        10 => b"GS0286", 11 => b"GS0290", 13 => b"GS0278", 14 => b"GS0292", _ => return,
    };
    game.send_skill_system_info(source.id, text);
}

fn resource_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, usage: u32,
) {
    let (code, text): (u32, &[u8]) = if usage == USER_MP_LOSE { (7, b"GS0288") } else { (8, b"GS0289") };
    game.update_registered_skill_visual(instance, code);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

fn path_is_clear(
    game: &mut CGame, instance: RegisteredSkill, source: ShapeIdentity,
    target: (i32, ShapeIdentity), properties: &CSkillBaseProperties, blocked_message: &[u8],
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
    {
        failure(game, instance, source, 11);
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_registered_skill_visual(instance, 15);
        if source.object_type == PLAYER_TYPE {
            let name = game.base_magic_target_name(target.0, target.1).unwrap_or_default();
            game.send_skill_system_info_with_text(source.id, blocked_message, name);
        }
        return false;
    }
    true
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let source = player.shape().identity();
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return false; };
    let Some(target) = target.filter(|target| target.1 != source) else {
        failure(game, instance, source, 10);
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, source, 13);
        return false;
    }
    if !path_is_clear(game, instance, source, target, &properties, b"GS0291") { return false; }
    let Some(player) = game.find_player(player_id) else { return false; };
    if !weapon_is_sword(game, player) {
        failure(game, instance, source, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if skill_id == LEAF_CUT_SKILL_ID && properties.query_property(USER_RP_LOSE) != 0 {
        let rp = player.rp();
        let loss = properties.query_property(USER_RP_LOSE);
        if (u32::from(rp).wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_RP_LOSE);
            return false;
        }
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
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = (source.shape().get_region_id(), source.shape().identity());
    let Some(target) = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if user.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                resource_failure(game, instance, user.1.id, &properties, USER_MP_LOSE);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            if skill_id == LEAF_CUT_SKILL_ID {
                let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
                let rp = player.rp();
                let remaining = u32::from(rp).wrapping_sub(properties.query_property(USER_RP_LOSE));
                if (remaining as i32) < 0 {
                    resource_failure(game, instance, user.1.id, &properties, USER_RP_LOSE);
                    return terminal(QueuedSkillExecutionState::Rejected);
                }
                if let Some(player) = game.find_player_mut(user.1.id) { player.set_rp(remaining as u16); }
            }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_sword(game, player)) {
                failure(game, instance, user.1, 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
    if !path_is_clear(game, instance, user.1, target, &properties, b"GS0307") {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.update_registered_skill_visual(instance, 1);
    apply_leaf_cut_family(game, instance, user, target, &properties, runtime);
    if user.1.object_type == PLAYER_TYPE { game.increase_owned_player_rp(user.1.id, true, 0); }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_leaf_cut<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !matches!(dispatch.skill_id(), LEAF_CUT_SKILL_ID | LEAF_CUT_2_SKILL_ID | LEAF_CUT_3_SKILL_ID) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target| {
            let player = game.find_player(player_id)?;
            game.player_skill_begin_object(player.shape().get_region_id(), target)
        })
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::LeafCut,
        |game, instance, player_id, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            let accepted = check_cast(game, instance, player_id, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
