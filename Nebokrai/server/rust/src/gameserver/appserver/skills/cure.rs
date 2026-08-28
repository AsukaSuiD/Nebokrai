//! Очищение `CCure` (`0x131`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/cure.cpp`. Модуль сохраняет двойную проверку MP,
//! время восстановления, путь и препятствия, задержку, направление, точную
//! вероятность и один вызов генератора MSVCRT на каждое подходящее состояние
//! в порядке исходного вектора состояний. Из уже типизированных состояний
//! достигнуты `0x191`, `0x192`, `0x199` и `0x1F8`; неизвестные старые записи
//! остаются нетронутыми. `CGame` только разрешает владельцев и выполняет
//! доставку. Координатная перегрузка
//! `Begin` остаётся ниже как `UNKNOWN` (исследовательский декомпилят хранится локально).

use super::baseattack::time_reached;
use super::bossbluequakestate::{
    BOSS_BLUE_QUAKE_STATE_ID, BossBlueQuakeState,
    finish_player_boss_blue_quake_state_on_cure, send_boss_blue_quake_state_visual,
};
use super::curestate::{CureState, send_cure_state_visual, send_cure_state_visual_at};
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::knockoutstate::{
    KNOCK_OUT_STATE_ID, KnockOutState, finish_player_knock_out_state_on_defense,
    send_knock_out_state_visual,
};
use super::spiderpoison::SPIDER_POISON_SKILL_ID;
use super::spiderpoisonstate::{
    SpiderPoisonState, finish_player_spider_poison_state_on_cure,
    send_spider_poison_state_visual,
};
use super::spiderweb::SPIDER_WEB_SKILL_ID;
use super::spiderwebstate::{
    SpiderWebState, finish_player_spider_web_state_on_defense,
    send_spider_web_state_visual,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;

pub(crate) const CURE_SKILL_ID: u32 = 0x131;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;
const CONST: u32 = 20_010;
const EM_MODIFIER: u32 = 20_015;
const BASE_PROBABILITY: u32 = 40_001;

#[derive(Clone)]
struct CureTarget {
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    dead: bool,
    ordinary_monster: bool,
    unsupported_tamed_monster: bool,
    display_name: Vec<u8>,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn caster_identity(player_id: i32) -> ShapeIdentity {
    ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<CureTarget> {
    let (tile_x, tile_y) = game.move_shape_target_tile(Some(region_id), identity)?;
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            Some(CureTarget {
                identity,
                tile_x,
                tile_y,
                dead: player.is_dead(),
                ordinary_monster: false,
                unsupported_tamed_monster: false,
                display_name: player.player_name().to_vec(),
            })
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let carriage = monster.is_carriage(property);
            Some(CureTarget {
                identity,
                tile_x,
                tile_y,
                dead: monster.hit_points() == 0,
                ordinary_monster: !monster.is_tamed() && !carriage,
                unsupported_tamed_monster: monster.is_tamed() && !carriage,
                display_name: monster.display_name().to_vec(),
            })
        }
        _ => None,
    }
}

fn requested_target(dispatch: PlayerSkillDispatch, player_id: i32) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id: CURE_SKILL_ID, .. } => Some(caster_identity(player_id)),
        PlayerSkillDispatch::Object { skill_id: CURE_SKILL_ID, target: target @ ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } } => Some(target),
        _ => None,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_cast(game: &mut CGame, player_id: i32, target: &CureTarget, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(CURE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(target.identity.object_type);
        message.add_long(target.identity.id);
        message.add_long(target.tile_x);
        message.add_long(target.tile_y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

fn cure_threshold(element_modify: i32, base_probability: u32, constant: u32, em_modifier: u32) -> i32 {
    let scaled = ((em_modifier as f32) * 0.01 * (element_modify as f32)).round_ties_even() as i32;
    (scaled as u32).wrapping_mul(constant).wrapping_add(base_probability) as i32
}

fn curable_state_ids(game: &CGame, region_id: i32, target: ShapeIdentity) -> Vec<u32> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::curable_state_ids).unwrap_or_default(),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(target.id)).map(|monster| monster.move_shape().curable_state_ids()).unwrap_or_default(),
        _ => Vec::new(),
    }
}

enum RemovedMonsterCurableState {
    SpiderPoison(SpiderPoisonState),
    SpiderWeb(SpiderWebState),
    KnockOut(KnockOutState),
    BossBlueQuake(BossBlueQuakeState),
}

fn finish_monster_curable_state(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    state_id: u32,
    now_ms: u32,
) -> bool {
    let Some(mut owner) = game.take_region_owner(region_id) else { return false };
    let removed = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
        let removed = match state_id {
            SPIDER_POISON_SKILL_ID => RemovedMonsterCurableState::SpiderPoison(monster.move_shape_mut().take_spider_poison_state()?),
            SPIDER_WEB_SKILL_ID => {
                let state = monster.move_shape_mut().take_spider_web_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::SpiderWeb(state)
            }
            KNOCK_OUT_STATE_ID => {
                let state = monster.move_shape_mut().take_knock_out_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::KnockOut(state)
            }
            BOSS_BLUE_QUAKE_STATE_ID => {
                let state = monster.move_shape_mut().take_boss_blue_quake_state()?;
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
                RemovedMonsterCurableState::BossBlueQuake(state)
            }
            _ => return None,
        };
        Some((removed, monster.move_shape().shape().identity(), monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?))
    });
    game.restore_region_owner(owner);
    let Some((removed, identity, tile_x, tile_y)) = removed else { return false };
    match removed {
        RemovedMonsterCurableState::SpiderPoison(state) => send_spider_poison_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
        RemovedMonsterCurableState::SpiderWeb(state) => send_spider_web_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
        RemovedMonsterCurableState::KnockOut(state) => send_knock_out_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
        RemovedMonsterCurableState::BossBlueQuake(state) => send_boss_blue_quake_state_visual(game, region_id, identity, tile_x, tile_y, state, false, now_ms),
    }
    true
}

fn finish_curable_state(game: &mut CGame, region_id: i32, target: ShapeIdentity, state_id: u32, now_ms: u32) -> bool {
    match (target.object_type, state_id) {
        (PLAYER_TYPE, SPIDER_POISON_SKILL_ID) => finish_player_spider_poison_state_on_cure(game, target.id, now_ms),
        (PLAYER_TYPE, SPIDER_WEB_SKILL_ID) => finish_player_spider_web_state_on_defense(game, target.id, now_ms),
        (PLAYER_TYPE, KNOCK_OUT_STATE_ID) => finish_player_knock_out_state_on_defense(game, target.id, now_ms),
        (PLAYER_TYPE, BOSS_BLUE_QUAKE_STATE_ID) => finish_player_boss_blue_quake_state_on_cure(game, target.id, now_ms),
        (MONSTER_TYPE, _) => finish_monster_curable_state(game, region_id, target.id, state_id, now_ms),
        _ => false,
    }
}

fn install_cure_state(game: &mut CGame, region_id: i32, target: &CureTarget, state: CureState) -> bool {
    match target.identity.object_type {
        PLAYER_TYPE => {
            let installed = game.find_player_mut(target.identity.id).and_then(|player| {
                (player.server_region_id() == Some(region_id)).then(|| player.replace_cure_state(state))
            });
            let Some(old) = installed else { return false };
            if let Some(old) = old { send_cure_state_visual(game, target.identity.id, old, false); }
            send_cure_state_visual(game, target.identity.id, state, true);
            true
        }
        MONSTER_TYPE => {
            let Some(mut owner) = game.take_region_owner(region_id) else { return false };
            let old = owner.base_mut().find_monster_by_id_mut(target.identity.id).map(|monster| monster.move_shape_mut().replace_cure_state(state));
            game.restore_region_owner(owner);
            let Some(old) = old else { return false };
            if let Some(old) = old { send_cure_state_visual_at(game, region_id, target.identity, target.tile_x, target.tile_y, old, false); }
            send_cure_state_visual_at(game, region_id, target.identity, target.tile_x, target.tile_y, state, true);
            true
        }
        _ => false,
    }
}

pub(crate) const fn is_cure_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: CURE_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: CURE_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }
    )
}

pub(crate) fn execute_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(requested_identity) = requested_target(dispatch, player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(CURE_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CURE_SKILL_ID, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(DELAY_TIME);
    let keep_time_ms = properties.query_property(STATE_PERSIST_TIME);
    let reuse_delay_ms = properties.query_property(REUSE_DELAY_TIME);
    let constant = properties.query_property(CONST);
    let em_modifier = properties.query_property(EM_MODIFIER);
    let base_probability = properties.query_property(BASE_PROBABILITY);
    let _can_be_breaked = properties.query_property(CAN_BE_BREAKED);

    if player_ai.cure().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let Some(initial_target) = target_snapshot(game, region_id, requested_identity) else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if initial_target.unsupported_tamed_monster {
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.cure_last_used_ms() != 0 && !time_reached(cooldown_now_ms, player_ai.cure_last_used_ms(), reuse_delay_ms) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = if requested_identity == caster_identity(player_id) { Vec::new() } else { game.base_magic_path(region_id, source_x, source_y, initial_target.tile_x, initial_target.tile_y, None) };
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            send_failure(game, player_id, 0x0f);
            game.send_skill_system_info_with_text(player_id, b"GS0295", &initial_target.display_name);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(CURE_SKILL_ID));
        }
        player_ai.begin_cure(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai.cure().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let mut target = match target_snapshot(game, region_id, requested_identity) {
        Some(target) => target,
        None => { finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
    };
    if target.ordinary_monster {
        target = target_snapshot(game, region_id, caster_identity(player_id)).expect("заклинатель очищения сохранён");
    }
    if target.dead {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.cure().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target.tile_x, target.tile_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_cast(game, player_id, &target, level, false);
        if let Some(execution) = player_ai.cure_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = player_ai.cure().map(SkillExecutionKernel::started_at_ms).expect("выполнение очищения создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }

    send_cast(game, player_id, &target, level, true);
    let element_modify = game.find_player(player_id).map(|player| player.combat_properties().element_modify).unwrap_or_default();
    let threshold = cure_threshold(element_modify, base_probability, constant, em_modifier);
    for state_id in curable_state_ids(game, region_id, target.identity) {
        if game.skill_random_below(100) < threshold {
            // Пакеты завершения этих состояний не содержат время; дополнительное
            // чтение часов между вызовами генератора MSVCRT исходный `CastCure` не делал.
            let _ = finish_curable_state(game, region_id, target.identity, state_id, 0);
        }
    }
    let installed = install_cure_state(game, region_id, &target, CureState::new(keep_time_ms));
    if let Some(execution) = player_ai.cure_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_cure_used(runtime.now_milliseconds());
    finish(game, player_id);
    terminal(if installed { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\cure.cpp

// ============================================================================
// FUNCTION: CCure::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\cure.cpp:199
// RVA: 0x001AD3A0
// ADDRESS: 005ad3a0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
