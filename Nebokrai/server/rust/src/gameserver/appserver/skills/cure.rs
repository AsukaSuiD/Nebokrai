//! Очищение `CCure` (`0x131`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/cure.cpp`. Модуль сохраняет двойную проверку MP,
//! время восстановления, путь и препятствия, задержку, направление, точную
//! вероятность и один вызов генератора MSVCRT на каждое подходящее состояние
//! в порядке исходного вектора состояний. Из уже типизированных состояний
//! достигнуты `0x67`, `0x73`, `0x7C`, `0xC9`, `0xD2`, `0x138`, `0x191`,
//! `0x192`, эффекты `0x199`, `0x1A6` и
//! `0x1F8`; неизвестные старые записи
//! остаются нетронутыми. `CGame` только разрешает владельцев и выполняет
//! доставку. Координатная перегрузка `Begin` использует точный базовый
//! `CState::GetSufferer` и fallback к `GetUser`, когда цель не найдена;
//! `DoesTargetEffective` допускает игрока либо только carriage-монстра, не
//! подменяя обычного или приручённого монстра заклинателем.
//! Порог очищения сохраняет расширенное вычисление x87 и усечение к нулю
//! перед исходным целочисленным умножением. Восстановление использует
//! абсолютные сроки `CSkill::IsRestored` и cast-delay (cmp/jb по 0x005AE373).
//! После списания MP вызывается OnChangeStates (0x005AE2F1) до поворота
//! и визуализации каста, а не обновление общего боевого статуса.
//! AI (`0x005AE110`) сначала выполняет Begin нового Cure (`0x005AE49F`),
//! затем завершает только первый прежний (`0x005AE50A`) и устанавливает
//! новый в освободившийся слот (`0x005AE53A`). End пересчитывает свойства
//! без обоих экземпляров; остальные Cure сохраняются. При отсутствии
//! прежней записи новый экземпляр добавляется в конец без UpdateProperty.
//! CastCure (0x005ADB10) допускает 0x198 (cmp в 0x005ADC0A) без
//! исключения для самого заклинателя и вызывает End состояния через +0x1C
//! в 0x005ADC58. Этот фильтр ID не доказывает наличие активного SpiderMist
//! в m_vStates: exact RTTI 0x0066F16C задаёт базу CSummonSkill, а его
//! Begin/CheckCastCondition не вызывают AddState (см. spidermist.rs).
//! Поэтому недостигнутый producer записи 0x198 не заменяется выдуманной
//! регистрацией cast и его отменой. На время owning callbacks извлечённый
//! AI заклинателя публикуется в CPlayer и затем возвращается тому же владельцу.
//! CastCure перечитывает живую длину (0x005ADBA0) и выполняет один RNG на
//! достигнутую подходящую позицию. После общего direct End (0x005ADC58)
//! перечитывается та же позиция; оставшийся экземпляр удаляется без второго
//! End (0x005ADC5B..0x005ADC7F). Дополнительного UpdateProperty после
//! обхода нет. Установка нового Cure также выполняется с опубликованными
//! player AI и настоящим регионом, сохраняя прежнюю позицию замены.

use super::fightdefense::truncate_original;
use super::curestate::{CureState, end_cure_state_key, send_cure_state_visual_for_holder};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::stateskill::finish_state_skill;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_coordinate_sufferer, resolve_identity_sufferer, resolve_state_user,
    resolve_state_move_shape, resolve_state_move_shape_mut, end_move_shape_state,
    end_and_destroy_state_at,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
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
    display_name: Vec<u8>,
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn caster_identity(player_id: i32) -> ShapeIdentity {
    ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<CureTarget> {
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            let tile_x = player.shape().get_tile_x().ok()?;
            let tile_y = player.shape().get_tile_y().ok()?;
            Some(CureTarget {
                identity,
                tile_x,
                tile_y,
                dead: player.is_dead(),
                display_name: player.player_name().to_vec(),
            })
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let tile_x = monster.move_shape().shape().get_tile_x().ok()?;
            let tile_y = monster.move_shape().shape().get_tile_y().ok()?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            if !monster.is_carriage(property) {
                return None;
            }
            Some(CureTarget {
                identity,
                tile_x,
                tile_y,
                dead: monster.hit_points() == 0,
                display_name: monster.display_name().to_vec(),
            })
        }
        _ => None,
    }
}

fn requested_target(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
    player_id: i32,
) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id: CURE_SKILL_ID, .. } => Some(caster_identity(player_id)),
        PlayerSkillDispatch::Point { skill_id: CURE_SKILL_ID, x, y } => {
            resolve_coordinate_sufferer(game, region_id, x, y)
                .or_else(|| resolve_state_user(game, region_id, caster_identity(player_id)))
        }
        PlayerSkillDispatch::Object { skill_id: CURE_SKILL_ID, target: target @ ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } } => {
            resolve_identity_sufferer(game, region_id, target)
                .or_else(|| resolve_state_user(game, region_id, caster_identity(player_id)))
        }
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

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_cure<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_state_skill(game, player_id, CURE_SKILL_ID, runtime);
}

fn abort_player_cure(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn complete_player_cure<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, CURE_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_cure(game, player_id, player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_cure<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, CURE_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_cure(game, player_id);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

fn cure_threshold(element_modify: i32, base_probability: u32, constant: u32, em_modifier: u32) -> i32 {
    let scaled = truncate_original(
        f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    (scaled as u32).wrapping_mul(constant).wrapping_add(base_probability) as i32
}

fn cast_cure_states(game: &mut CGame, region_id: i32, target: ShapeIdentity, threshold: i32) {
    let mut index = 0;
    loop {
        let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return };
        if index >= shape.state_slot_count() { break }
        let selected = shape.state_at(index).is_some_and(|(_, state)| state.is_curable());
        if selected && game.skill_random_below(100) < threshold {
            let _ = end_and_destroy_state_at(game, region_id, target, index);
        }
        index += 1;
    }
}

pub(crate) fn finish_curable_state(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    state_id: u32,
    _now_ms: u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return false };
    let key = (0..shape.state_slot_count()).find_map(|index| {
        let (key, state) = shape.state_at(index)?;
        (state.is_curable() && state.state_id() == state_id).then_some(key)
    });
    let Some(key) = key else { return false };
    end_move_shape_state(game, region_id, target, key)
}

fn install_cure_state(game: &mut CGame, region_id: i32, target: &CureTarget, state: CureState) -> bool {
    if !matches!(target.identity.object_type, PLAYER_TYPE | MONSTER_TYPE) {
        return false;
    }
    if target.identity.object_type == PLAYER_TYPE
        && !game.find_player(target.identity.id)
            .is_some_and(|player| player.server_region_id() == Some(region_id))
    {
        return false;
    }
    let Some(shape) = resolve_state_move_shape(game, region_id, target.identity) else { return false };
    let old = shape.cure_state_key();
    send_cure_state_visual_for_holder(game, region_id, target.identity, state, true);
    if let Some(key) = old {
        let Some(location) = resolve_state_move_shape(game, region_id, target.identity)
            .and_then(|shape| shape.applied_state_replacement_location(key)) else { return false };
        if !end_cure_state_key(game, region_id, target.identity, key) {
            return false;
        }
        let Some(shape) = resolve_state_move_shape_mut(game, region_id, target.identity) else { return false };
        let record = state.encoded_for_install();
        if shape.insert_replacement_state_record(state, &record, location).is_none() { return false; }
    } else if let Some(shape) = resolve_state_move_shape_mut(game, region_id, target.identity) {
        shape.push_cure_state(state);
    } else { return false }
    true
}

pub(crate) const fn is_cure_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: CURE_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: CURE_SKILL_ID, .. }
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
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(CURE_SKILL_ID, game.skill_factory()), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(requested_identity) = requested_target(game, region_id, dispatch, player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CURE_SKILL_ID, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(DELAY_TIME);
    let _keep_time_ms = properties.query_property(STATE_PERSIST_TIME);
    let reuse_delay_ms = properties.query_property(REUSE_DELAY_TIME);
    let constant = properties.query_property(CONST);
    let em_modifier = properties.query_property(EM_MODIFIER);
    let base_probability = properties.query_property(BASE_PROBABILITY);
    let _can_be_breaked = properties.query_property(CAN_BE_BREAKED);

    if game.player_skill_execution(player_id, CURE_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.begin_player_skill_with_combat(player_id, dispatch, started_at_ms);
        let Some(initial_target) = target_snapshot(game, region_id, requested_identity) else {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, CURE_SKILL_ID), reuse_delay_ms, cooldown_now_ms) {
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
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, CURE_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let target = match target_snapshot(game, region_id, requested_identity) {
        Some(target) => target,
        None => { abort_player_cure(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
    };
    if target.dead {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_cure(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, CURE_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_cure(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.publish_player_states(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target.tile_x, target.tile_y));
        }
        send_cast(game, player_id, &target, level, false);
        if let Some(execution) = game.player_skill_execution_mut(player_id, CURE_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started_at_ms = game.player_skill_execution(player_id, CURE_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение очищения создано или восстановлено");
    if runtime.now_milliseconds() < started_at_ms.wrapping_add(delay_ms) { return terminal(QueuedSkillExecutionState::Pending) }

    send_cast(game, player_id, &target, level, true);
    let element_modify = game.find_player(player_id).map(|player| player.combat_properties().element_modify).unwrap_or_default();
    let threshold = cure_threshold(element_modify, base_probability, constant, em_modifier);
    let installed = game.with_published_player_ai(player_id, player_ai, |game| {
        cast_cure_states(game, region_id, target.identity, threshold);
        install_cure_state(
            game,
            region_id,
            &target,
            CureState::new(caster_identity(player_id), target.identity).begin_now(),
        )
    });
    if let Some(execution) = game.player_skill_execution_mut(player_id, CURE_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_cure(game, player_id, player_ai, runtime);
    terminal(if installed { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
