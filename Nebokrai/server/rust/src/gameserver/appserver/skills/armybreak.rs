//! Армейский удар ArmyBreak/ArmyBreak2 (0x68/0x7B).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/armybreak*.cpp.
//!
//! Общий зарегистрированный вход сохраняет исходного U для Check и отдельное
//! направление каждого исполнения. Check игрока не использует S: после reuse
//! нужны меч категории 1 и signed MP/RP; нулевая цена не читает ресурс.
//! В первом AI живой U списывает MP до RP, сохраняя частичную трату при отказе,
//! затем OnChangeStates и повторная проверка меча. Свежий S либо базовая точка
//! определяет направление, после него записывается CAN, visual0 и condition.
//! Таблица свойств этого AI остаётся прежней через callbacks.
//!
//! Удар ожидает абсолютный unsigned срок start+delay без elapsed-нормализации.
//! Visual1 читает каноническое направление и живую позицию U; после него область
//! и локальный список обработанных целей принадлежат armybreakattack. Успешный
//! хвост возвращает End(1), в том числе при исчезнувшем регионе; ошибки дают
//! End(0). Общий End сбрасывает фазу и направление до возврата движения,
//! AfterUse и reuse. Ручной command-tail не завершает тот же навык повторно.

use super::armybreak2::ARMY_BREAK_2_SKILL_ID;
use super::armybreakattack::run_army_break_attack;
use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const ARMY_BREAK_SKILL_ID: u32 = 0x68;
const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArmyBreakExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    direction: i32,
}

impl ArmyBreakExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), direction: -1 }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.direction = -1;
        true
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

pub(crate) fn publish_army_break_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CArmyBreak | SkillOwner::CArmyBreak2)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::ArmyBreak || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let shape = user.shape();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 4 | 7 | 8 | 10 | 11 | 13 | 14 | 15) {
        if shape.identity().object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), shape.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(shape.identity().object_type);
    message.add_long(shape.identity().id);
    if mode == 0 {
        message.add_long(shape.get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
        let x = shape.get_tile_x().unwrap_or(i32::MIN);
        let y = shape.get_tile_y().unwrap_or(i32::MIN);
        let Some(state) = skill.player_state::<ArmyBreakExecutionState>() else { return; };
        // Native индексирует таблицу без проверки: ошибочное направление
        // не заменяется выдуманной клеткой эффекта.
        let Ok(front) = CShape::get_direction_position(state.direction, ShapeAreaCoordinates { x, y }) else { return; };
        message.add_long(front.x);
        message.add_long(front.y);
    }
    if shape.is_assigned_to_server_region()
        && let Some(owner) = game.find_region(shape.get_region_id())
    {
        let _ = game.send_game_shape_around(owner.base(), shape, None, &message);
    }
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0287", _ => return };
    game.send_skill_system_info(player_id, text);
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
    if properties.query_property(USER_MP_LOSE) != 0 {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let rp = u32::from(player.rp());
        let loss = properties.query_property(USER_RP_LOSE);
        if (rp.wrapping_sub(loss) as i32) < 0 {
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
                resource_failure(game, instance, user.1.id, &properties, USER_MP_LOSE);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
            let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let rp = u32::from(player.rp());
            let remaining = rp.wrapping_sub(properties.query_property(USER_RP_LOSE));
            if (remaining as i32) < 0 {
                resource_failure(game, instance, user.1.id, &properties, USER_RP_LOSE);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(user.1.id) { player.set_rp(remaining as u16); }
            game.publish_player_states(user.1.id);
            if game.find_player(user.1.id).is_none_or(|player| !weapon_is_sword(game, player)) {
                failure(game, instance, user.1.id, 14);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
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
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<ArmyBreakExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.direction = direction;
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.shape_mut().set_direction(direction); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    let Some(direction) = game.registered_skill(instance).and_then(|skill| skill.player_state::<ArmyBreakExecutionState>()).map(|state| state.direction) else { return terminal(QueuedSkillExecutionState::Completed); };
    run_army_break_attack(game, instance, user, direction, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_army_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !matches!(dispatch.skill_id(), ARMY_BREAK_SKILL_ID | ARMY_BREAK_2_SKILL_ID) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ArmyBreak,
        check_cast, |dispatch, started| ArmyBreakExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
