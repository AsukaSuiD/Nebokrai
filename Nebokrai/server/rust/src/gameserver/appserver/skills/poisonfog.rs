//! Призыв ядовитого тумана CPoisonFog (0xC9).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/poisonfog.cpp.
//! Зарегистрированный навык владеет подготовкой, точкой назначения и visual;
//! самостоятельная область тумана продолжает жить после завершения навыка.

use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::poisonfogphalanx::CPoisonFogPhalanx;
use super::poisonmoth::{master_info, weapon_is_crossbow};
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const POISON_FOG_SKILL_ID: u32 = 0xc9;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const DELAY: u32 = 10_001;
const STATE_TIME: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;
const DEF_LOSS: u32 = 209;
const DODGE_LOSS: u32 = 210;
const ELEMENT_LOSS: u32 = 212;
const DEF_COEFFICIENT: u32 = 223;
const ER_COEFFICIENT: u32 = 224;
const LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) fn is_poison_fog_target(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == POISON_FOG_SKILL_ID
}

pub(crate) fn complete_player_poison_fog<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, POISON_FOG_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_poison_fog<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, POISON_FOG_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn publish_poison_fog_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CPoisonFog
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::PoisonFog || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        let (x, y) = skill.lifecycle().destination();
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn fail(game: &mut CGame, player_id: i32, code: u32, text: &[u8], mp: Option<u32>) {
    game.update_player_skill_visual(player_id, POISON_FOG_SKILL_ID, code);
    if let Some(mp) = mp {
        game.send_skill_system_info_with_unsigned(player_id, text, mp);
    } else {
        game.send_skill_system_info(player_id, text);
    }
}

fn begin_poison_fog<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    game.replace_player_skill_visual_effect(
        player_id, POISON_FOG_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::PoisonFog, 1),
    );
    if game.find_player(player_id).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(address) = game.registered_player_skill(player_id, POISON_FOG_SKILL_ID) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let target_position = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, target)| resolve_state_move_shape(game, region, target))
        .and_then(|target| Some((target.shape().get_tile_x().ok()?, target.shape().get_tile_y().ok()?)));
    if let Some(destination) = target_position {
        // CheckCast превращает найденную S в координатную цель ещё до reuse:
        // последующее движение исходного объекта уже не перемещает туман.
        if let Some(skill) = game.registered_skill_mut(address) {
            skill.lifecycle_mut().set_point_target(destination);
        }
    }
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let level = skill.level();
    let started = skill.lifecycle().started_at_ms();
    let Some(properties) = game.skill_base_properties(POISON_FOG_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(
        game.player_skill_last_used_ms(player_id, POISON_FOG_SKILL_ID), reuse, runtime.now_milliseconds(),
    ) {
        fail(game, player_id, 13, b"GS0278", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        fail(game, player_id, 11, b"GS0290", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if path.iter().any(|cell| cell.2 == 2) {
        fail(game, player_id, 15, b"GS0282", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !weapon_is_crossbow(game, player) {
        fail(game, player_id, 14, b"GS0293", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    // В этом навыке нулевая стоимость не означает бесплатное применение:
    // CheckCast возвращает отказ без дополнительного пакета или сообщения.
    if properties.query_property(MP_LOSS) == 0 { return terminal(QueuedSkillExecutionState::Rejected); }
    let mana = player.mana();
    if (mana.wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0 {
        let amount = properties.query_property(MP_LOSS);
        fail(game, player_id, 7, b"GS0288", Some(amount));
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); }
    game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
    terminal(QueuedSkillExecutionState::Begun)
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, destination: (i32, i32), runtime: &mut Runtime,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    let mut master = master_info(player);
    master.master_country_id = 0;
    let weapon_level = player.weapon_damage_level(game.goods_factory()) as u32;
    let Some(level) = game.registered_player_skill(player_id, POISON_FOG_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return; };
    let Some(properties) = game.skill_base_properties(POISON_FOG_SKILL_ID, level) else { return; };
    // Summon выбирает свойства и оружие после подготовки/visual, а не из
    // снимка начала AI. Часы конструктора идут после всех его аргументов.
    let er_coefficient = properties.query_property(ER_COEFFICIENT);
    let element_loss = properties.query_property(ELEMENT_LOSS);
    let dodge_loss = properties.query_property(DODGE_LOSS);
    let defense_coefficient = properties.query_property(DEF_COEFFICIENT);
    let defense_loss = properties.query_property(DEF_LOSS);
    let state_time = properties.query_property(STATE_TIME);
    let lifetime = properties.query_property(LIFETIME);
    let now = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CPoisonFogPhalanx::new(
        id, master, now, lifetime, level, state_time, defense_loss,
        defense_coefficient, dodge_loss, element_loss, er_coefficient, weapon_level,
    );
    phalanx.set_center(destination.0, destination.1);
    let Some(region) = game.find_player(player_id).map(|player| player.shape().get_region_id()) else { return; };
    if game.add_poison_fog_phalanx(region, phalanx, destination.0, destination.1, now, runtime)
        .is_some_and(|result| result.is_ok())
    {
        let _ = game.send_poison_fog_phalanx_entry(region, id, runtime);
    }
}

pub(crate) fn execute_player_poison_fog<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_poison_fog_target(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    if game.player_skill_execution(player_id, POISON_FOG_SKILL_ID).is_none() {
        return begin_poison_fog(game, player_id, dispatch, runtime);
    }
    let Some(level) = game.registered_player_skill(player_id, POISON_FOG_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(POISON_FOG_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.find_player(player_id).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
    if game.player_skill_execution(player_id, POISON_FOG_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let loss = properties.query_property(MP_LOSS);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(MP_LOSS);
            fail(game, player_id, 7, b"GS0288", Some(amount));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_crossbow(game, player)) {
            fail(game, player_id, 14, b"GS0293", None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(properties) = game.skill_base_properties(POISON_FOG_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let can_break = properties.query_property(CAN_BREAK);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, POISON_FOG_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        let Some(destination) = game.player_skill_lifecycle(player_id, POISON_FOG_SKILL_ID)
            .map(|lifecycle| lifecycle.destination())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, destination.0, destination.1,
            ));
        }
        game.update_player_skill_visual(player_id, POISON_FOG_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, POISON_FOG_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(POISON_FOG_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(DELAY);
    let Some(started) = game.player_skill_execution(player_id, POISON_FOG_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    game.update_player_skill_visual(player_id, POISON_FOG_SKILL_ID, 1);
    if let Some(destination) = game.player_skill_lifecycle(player_id, POISON_FOG_SKILL_ID)
        .map(|lifecycle| lifecycle.destination())
    {
        let _ = game.with_published_player_ai(player_id, ai, |game| {
            summon(game, player_id, destination, runtime);
        });
    }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, POISON_FOG_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    // Результат Summon не определяет End: попытка призыва завершает навык
    // успешно даже при отсутствии региона или невозможности регистрации.
    terminal(QueuedSkillExecutionState::Completed)
}
