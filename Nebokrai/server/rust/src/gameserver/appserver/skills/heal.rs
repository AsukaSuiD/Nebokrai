//! Семейство периодического лечения `CHeal/CHeal2/CSuperHeal/CSuperHeal2`
//! (`0xD3/0xE3/0xD9/0xE4`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/heal*.cpp` и `superheal*.cpp`. Совпадающий конвейер объединён:
//! двойная проверка MP, время восстановления, расстояние, задержка, направление,
//! точная формула усиления оружием с плавающей точкой и установка
//! периодического состояния. Начальная прибавка HP во всех четырёх `AI`
//! сначала сохраняет произведение с `0.01f` в `f32`, затем отдельно сохраняет
//! в `f32` сумму с unsigned-константой и только при создании состояния усекает
//! её через `FISTP dword`; ручного округления дробной части здесь нет.
//! Обычный нетранспортный монстр после начальной проверки заменяется самим
//! заклинателем. `CGame` координирует владельцев и доставку; формула и пакеты
//! остаются здесь. У `CSuperHeal2` состояние подтверждённо хранится у выбранной
//! цели, но лечит и визуализирует заклинателя; каноническое состояние поэтому
//! отдельно хранит владельца эффекта. Координатный `Begin` после базовой записи
//! точки получает нулевую object-target и тем же virtual fallback выбирает
//! заклинателя, поэтому точечный клиентский dispatch проходит как self-target.
//! Семейный reuse-gate использует exact `CSkill::IsRestored`; cast delay и
//! частота периодического лечения сохраняют elapsed-семантику.
//! Begin заканчивается возвратом Begun после создания исполнения. Проверки
//! и эффекты первого AI остаются после этой границы; координатор вызывает AI
//! в том же Run после постановки Attack, не сдвигая исходное время Begin.

use super::baseattack::time_reached;
use super::heal2::HEAL_2_SKILL_ID;
use super::fightdefense::truncate_original;
use super::healstate::{HealState, send_heal_state_visual};
use super::healstate2::HealState2;
use super::superheal::SUPER_HEAL_SKILL_ID;
use super::superheal2::SUPER_HEAL_2_SKILL_ID;
use super::superhealstate::SuperHealState;
use super::superhealstate2::SuperHealState2;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::stateskill::finish_state_skill;
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

pub(crate) const HEAL_SKILL_ID: u32 = 0xd3;
pub(crate) const HEAL_EFFECT_MESSAGE: i32 = 0x000b_fe01;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 5_002;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy)]
struct HealTarget {
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    dead: bool,
    ordinary_monster: bool,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) const fn is_heal_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID
    )
}



fn requested_target(player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(u32, ShapeIdentity)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
            if is_heal_skill(skill_id) =>
        {
            Some((
                skill_id,
                ShapeIdentity {
                    object_type: PLAYER_TYPE,
                    id: player_id,
                    ex_id: CGuid::GUID_INVALID,
                },
            ))
        }
        PlayerSkillDispatch::Object { skill_id, target }
            if is_heal_skill(skill_id)
                && matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) =>
        {
            Some((skill_id, target))
        }
        _ => None,
    }
}

fn caster_identity(player_id: i32) -> ShapeIdentity {
    ShapeIdentity {
        object_type: PLAYER_TYPE,
        id: player_id,
        ex_id: CGuid::GUID_INVALID,
    }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<HealTarget> {
    let (tile_x, tile_y) = game.move_shape_target_tile(Some(region_id), identity)?;
    match identity.object_type {
        PLAYER_TYPE => Some(HealTarget {
            identity,
            tile_x,
            tile_y,
            dead: game.find_player(identity.id).is_none_or(CPlayer::is_dead),
            ordinary_monster: false,
        }),
        MONSTER_TYPE => {
            let monster = game
                .find_region(region_id)?
                .base()
                .find_monster_by_id(identity.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(HealTarget {
                identity,
                tile_x,
                tile_y,
                dead: monster.hit_points() == 0,
                ordinary_monster: !monster.is_tamed() && !monster.is_carriage(property),
            })
        }
        _ => None,
    }
}

fn send_failure(game: &mut CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(HEAL_EFFECT_MESSAGE, player_id, action);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target: HealTarget,
    skill_id: u32,
    skill_level: i32,
    begin: bool,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let source = player.shape().identity();
    let mut message = CMessage::new(HEAL_EFFECT_MESSAGE);
    message.add_byte(if begin { 1 } else { 2 });
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(source.object_type);
    message.add_long(source.id);
    if begin {
        message.add_long(player.shape().get_direction());
    } else {
        message.add_long(target.identity.object_type);
        message.add_long(target.identity.id);
        message.add_long(target.tile_x);
        message.add_long(target.tile_y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_heal<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _player_ai: &mut CPlayerAI, skill_id: u32, runtime: &mut Runtime) {
    finish_movement(game, player_id);
    finish_state_skill(game, player_id, skill_id, runtime);
}

fn abort_player_heal(game: &mut CGame, player_id: i32) {
    finish_movement(game, player_id);
}

pub(crate) fn complete_player_heal<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, skill_id: u32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, skill_id).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_heal(game, player_id, player_ai, dispatch.skill_id(), runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_heal<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, skill_id: u32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, skill_id).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_heal(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn replace_state(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    removed_skill_id: u32,
    state: HealState,
) -> Option<Option<HealState>> {
    match target.object_type {
        PLAYER_TYPE => {
            let player = game.find_player_mut(target.id)?;
            (player.server_region_id() == Some(region_id))
                .then(|| player.replace_heal_state(removed_skill_id, state))
        }
        MONSTER_TYPE => {
            let mut owner = game.take_region_owner(region_id)?;
            let previous = owner
                .base_mut()
                .find_monster_by_id_mut(target.id)
                .map(|monster| {
                    monster
                        .move_shape_mut()
                        .replace_heal_state(removed_skill_id, state)
                });
            game.restore_region_owner(owner);
            previous
        }
        _ => None,
    }
}

pub(crate) fn execute_player_heal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some((skill_id, requested_identity)) = requested_target(player_id, dispatch) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some((region_id, source_x, source_y, skill_level, initial_mana, weapon_level)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(skill_id, game.skill_factory()),
                player.mana(),
                player.weapon_damage_level(game.goods_factory()) as u32,
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    // `CAttackSkill::Begin` подставляет заклинателя, если цель-объект уже
    // исчез. Это происходит до проверки условий и сохраняется на всех стадиях.
    let execution_started = game.player_skill_execution(player_id, skill_id).is_some();
    let effective_identity = if execution_started {
        requested_identity
    } else {
        target_snapshot(game, region_id, requested_identity)
            .map_or_else(|| caster_identity(player_id), |target| target.identity)
    };
    let requested = target_snapshot(game, region_id, effective_identity);
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        send_failure(game, player_id, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let coefficient = properties.query_property(SKILL_USAGE_HEAL_RECOVER_COEFFICIENT);
    let constant = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let scaled = coefficient.wrapping_mul(weapon_level);
    let scaled_factor = (f64::from(scaled) * f64::from(0.01_f32)) as f32;
    let hp_gain_float = (f64::from(constant) + f64::from(scaled_factor)) as f32;
    let hp_gain = truncate_original(f64::from(hp_gain_float)) as u32;

    if game.player_skill_execution(player_id, skill_id).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.begin_player_skill_with_combat(player_id, dispatch, started_at_ms);
        let Some(target) = requested else {
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, skill_id),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if !(target.identity.object_type == PLAYER_TYPE && target.identity.id == player_id)
            && maximum_distance != 0
            && game
                .base_magic_path(
                    region_id,
                    source_x,
                    source_y,
                    target.tile_x,
                    target.tile_y,
                    None,
                )
                .len()
                > maximum_distance as usize
        {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && initial_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(skill_id));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, skill_id)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(mut target) = target_snapshot(game, region_id, effective_identity) else {
        abort_player_heal(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.ordinary_monster {
        let self_identity = caster_identity(player_id);
        target = target_snapshot(game, region_id, self_identity)
            .expect("заклинатель лечения сохранён");
    }
    if target.dead {
        send_failure(game, player_id, 10);
        abort_player_heal(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.player_skill_execution(player_id, skill_id)
        .is_some_and(|execution| execution.stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_heal(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            if target.identity.object_type != PLAYER_TYPE || target.identity.id != player_id {
                player.movement_shape_mut().set_direction(get_line_direction(
                    source_x,
                    source_y,
                    target.tile_x,
                    target.tile_y,
                ));
            }
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        send_cast(game, player_id, target, skill_id, skill_level, true);
        if let Some(execution) = game.player_skill_execution_mut(player_id, skill_id) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_execution(player_id, skill_id)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение лечения создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_cast(game, player_id, target, skill_id, skill_level, false);
    let now_ms = runtime.now_milliseconds();
    let effect_target = if skill_id == SUPER_HEAL_2_SKILL_ID {
        caster_identity(player_id)
    } else {
        target.identity
    };
    let removed_skill_id = if skill_id == SUPER_HEAL_SKILL_ID {
        HEAL_SKILL_ID
    } else {
        skill_id
    };
    let state: HealState = match skill_id {
        HEAL_2_SKILL_ID => HealState2::new(
            skill_id, effect_target, now_ms, keep_time_ms, frequency_ms, hp_gain,
        ),
        SUPER_HEAL_SKILL_ID => SuperHealState::new(
            skill_id, effect_target, now_ms, keep_time_ms, frequency_ms, hp_gain,
        ),
        SUPER_HEAL_2_SKILL_ID => SuperHealState2::new(
            skill_id, effect_target, now_ms, keep_time_ms, frequency_ms, hp_gain,
        ),
        _ => HealState::new(
            skill_id, effect_target, now_ms, keep_time_ms, frequency_ms, hp_gain,
        ),
    };
    if let Some(previous) = replace_state(
        game,
        region_id,
        target.identity,
        removed_skill_id,
        state,
    ) {
        if let Some(previous) = previous {
            if let Some(previous_target) =
                target_snapshot(game, region_id, previous.effect_target())
            {
                send_heal_state_visual(
                    game,
                    region_id,
                    previous.effect_target(),
                    previous_target.tile_x,
                    previous_target.tile_y,
                    previous,
                    false,
                    || now_ms,
                );
            }
        }
        send_heal_state_visual(
            game,
            region_id,
            effect_target,
            if effect_target == target.identity { target.tile_x } else { source_x },
            if effect_target == target.identity { target.tile_y } else { source_y },
            state,
            true,
            || runtime.now_milliseconds(),
        );
    }
    if let Some(execution) = game.player_skill_execution_mut(player_id, skill_id) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_heal(game, player_id, player_ai, dispatch.skill_id(), runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
