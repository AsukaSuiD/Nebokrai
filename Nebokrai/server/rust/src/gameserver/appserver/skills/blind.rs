//! Ослепление `CBlind` (`0x76`) для игрока.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/blind.cpp`. Навык дважды проверяет MP и оружие категории
//! `2`, сохраняет расход MP перед поздней проверкой оружия и после задержки
//! повторно проверяет только дальность. Подтверждённая странность оригинала:
//! `AddBlindState` создаёт `CRushState2` (`0x7C`), а не `CBlindState`; поэтому
//! блокировка цели проходит через канонический `Rush2State`. `CGame` оставляет
//! разрешение владельцев, PK-уведомление и фактическую установку состояния.
//! Координатный и пустой `Begin` сохраняют приоритет проверки cooldown, затем
//! отказ `10 + GS0286` и `End(false)` без generic failure `2`.
//! `End(true)` фиксирует cooldown после установки, а `End(false)` не откатывает
//! уже установленный `Rush2State`.
//! Обе входные перегрузки проверяют reuse через exact `CSkill::IsRestored`.

use super::baseattack::{time_reached, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::poisonmoth::{master_info, MONSTER_TYPE, PLAYER_TYPE};
use super::rush::scaled_state_time;
use super::rushstate2::Rush2State;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::stateskill::finish_state_skill;
use crate::gameserver::appserver::states::summonskill::abort_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const BLIND_SKILL_ID: u32 = 0x76;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const STATE_PERSIST_TIME: u32 = 10_002;
const BLOCK_UNFLY: u8 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_blind_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: BLIND_SKILL_ID, .. }
            | PlayerSkillDispatch::Point { skill_id: BLIND_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: BLIND_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_blind<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_state_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(BLIND_SKILL_ID, now_ms));
}

fn abort_player_blind(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
    abort_skill(game, player_id);
}

pub(crate) fn complete_player_blind<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.blind().map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_blind(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_blind<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.blind().map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_blind(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn target_facts(
    game: &CGame,
    region_id: i32,
    target: ShapeIdentity,
) -> Option<(i32, i32, u8, bool, Vec<u8>)> {
    match target.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(target.id)?;
            if player.server_region_id() != Some(region_id) {
                return None;
            }
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.level(),
                player.is_dead(),
                player.player_name().to_vec(),
            ))
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(target.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let pet_level = monster.pet_progress().0;
            let level = if monster.is_tamed() && pet_level != 0 {
                pet_level as u8
            } else {
                property.level as u8
            };
            Some((
                monster.move_shape().shape().get_tile_x().ok()?,
                monster.move_shape().shape().get_tile_y().ok()?,
                level,
                monster.hit_points() == 0,
                monster.display_name().to_vec(),
            ))
        }
        _ => None,
    }
}

fn failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32, name: Option<&[u8]>, late_weapon: bool) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0286"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, if late_weapon { b"GS0287" } else { b"GS0292" }),
        0x0f => game.send_skill_system_info_with_text(player_id, b"GS0291", name.unwrap_or_default()),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(BLIND_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn execute_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_blind_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if matches!(dispatch, PlayerSkillDispatch::SelfTarget { .. } | PlayerSkillDispatch::Point { .. }) {
        let Some(level) = game.find_player(player_id).map(|player| player.learned_skill_level(BLIND_SKILL_ID)) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        if !skill_is_restored(
            ai.skill_last_used_ms(BLIND_SKILL_ID),
            reuse,
            runtime.now_milliseconds(),
        ) {
            failure(game, player_id, 0x0d, 0, None, false);
        } else {
            failure(game, player_id, 10, 0, None, false);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let PlayerSkillDispatch::Object { target, .. } = dispatch else { unreachable!() };
    let Some((region_id, source_x, source_y, source_level, level, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| Some((
            player.server_region_id()?,
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            player.level(),
            player.learned_skill_level(BLIND_SKILL_ID),
            player.mana(),
        )))
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some((target_x, target_y, _target_level, target_dead, target_name)) =
        target_facts(game, region_id, target)
    else {
        failure(game, player_id, 10, 0, None, false);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
        if ai.blind().is_some() {
            abort_player_blind(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let state_time = properties.query_property(STATE_PERSIST_TIME);
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.blind().is_none() {
        let now = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(BLIND_SKILL_ID), reuse, now) {
            failure(game, player_id, 0x0d, mp_loss, None, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum != 0 && path.len() as u32 > maximum {
            failure(game, player_id, 0x0b, mp_loss, None, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            failure(game, player_id, 0x0f, mp_loss, Some(&target_name), false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
            failure(game, player_id, 0x0e, mp_loss, None, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            failure(game, player_id, 7, mp_loss, None, false);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(BLIND_SKILL_ID));
        }
        ai.begin_blind(SkillExecutionKernel::begin(dispatch, now));
    } else if ai.blind().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if target_dead {
        game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_blind(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if ai.blind().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            failure(game, player_id, 7, mp_loss, None, false);
            abort_player_blind(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
            failure(game, player_id, 0x0e, mp_loss, None, true);
            abort_player_blind(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        send_visual(game, player_id, level, 1);
        if let Some(execution) = ai.blind_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started = ai.blind().map(SkillExecutionKernel::started_at_ms).unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started, delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some((live_x, live_y, live_level, live_dead, _)) = target_facts(game, region_id, target) else {
        abort_player_blind(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if live_dead {
        game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10);
        abort_player_blind(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let path = game.base_magic_path(region_id, source_x, source_y, live_x, live_y, None);
    if maximum != 0 && path.len() as u32 > maximum {
        failure(game, player_id, 0x0b, mp_loss, None, false);
        abort_player_blind(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    send_visual(game, player_id, level, 2);
    let Some(master) = game.find_player(player_id).map(master_info) else {
        abort_player_blind(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.owned_player_skill_target_attackable(master, target, region_id) {
        let owner_id = if target.object_type == PLAYER_TYPE {
            Some(target.id)
        } else {
            game.find_region(region_id)
                .and_then(|owner| owner.base().find_monster_by_id(target.id))
                .and_then(|monster| {
                    let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
                    let master = monster.master_info();
                    ((monster.is_tamed() || monster.is_carriage(property))
                        && master.master_type == PLAYER_TYPE
                        && master.master_id != 0)
                        .then_some(master.master_id)
                })
        };
        if let Some(owner_id) = owner_id.filter(|owner_id| *owner_id != player_id) {
            let _ = game.player_on_first_skill(player_id, owner_id, Some(region_id), runtime);
        }
        let keep = scaled_state_time(source_level, live_level, state_time);
        if keep != 0 {
            let now = runtime.now_milliseconds();
            let _ = game.install_rush_2_state(region_id, target, Rush2State::new(now, keep), now);
        }
    }
    if let Some(execution) = ai.blind_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_blind(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

// Ниже сохранены только недостигнутые конструктор и деструктор. Отдельный
// `CBlindState` остаётся RAW в своём owner-файле: рабочая цепочка выше его не
// вызывает и не подменяет.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blind.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blind.h

// ============================================================================
// FUNCTION: CBlind::CBlind
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blind.cpp:28
// RVA: 0x0016D9A0
// ADDRESS: 0056d9a0
// PROTOTYPE: undefined __thiscall CBlind(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlind::~CBlind
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blind.cpp:36
// RVA: 0x0016DA10
// ADDRESS: 0056da10
// PROTOTYPE: void __thiscall ~CBlind(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
