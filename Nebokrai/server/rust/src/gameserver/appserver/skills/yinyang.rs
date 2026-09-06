//! Область инь-ян `CYinYang` (`0x139`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/yinyang.cpp`. Здесь находятся три достигнутых варианта
//! цели, проверка пути, задержка повторного использования, двухфазный расход
//! MP, направление, `SkillExecutionKernel`, точный визуальный пакет и
//! построение `CYinYangPhalanx`. `CGame` только разрешает владельцев,
//! регистрирует область и выполняет фактическую доставку.
//! Общий для двух вариантов `End(1)` возвращает движение и завершает
//! зарегистрированную область; отмена использует `End(0)` без cooldown.
//! Стихийная прибавка вычисляется в расширенной точности x87 из беззнакового
//! свойства и знакового modifier-а игрока, затем усекается к нулю. Cooldown
//! следует абсолютному сроку `CSkill::IsRestored`, а задержка стадии остаётся
//! elapsed-проверкой.

use super::baseattack::time_reached;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::yinyangphalanx::CYinYangPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const YIN_YANG_SKILL_ID: u32 = 0x139;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_001;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_002;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }

fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|shape| (shape.tile_x, shape.tile_y, None)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|shape| (shape.tile_x, shape.tile_y, Some(target))),
    }
}

fn send_error(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0f => game.send_skill_system_info(player_id, b"GS0282"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, skill_id: u32, skill_level: i32, action: u8, destination: Option<(i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(skill_id as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 { message.add_long(player.shape().get_direction()); } else {
        let Some((x, y)) = destination else { return };
        message.add_long(0); message.add_long(0); message.add_long(x); message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn execution(ai: &CPlayerAI, second: bool) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { if second { ai.player_skill_execution(crate::gameserver::appserver::skills::yinyang2::YIN_YANG_2_SKILL_ID) } else { ai.player_skill_execution(YIN_YANG_SKILL_ID) } }
fn execution_mut(ai: &mut CPlayerAI, second: bool) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { if second { ai.player_skill_execution_mut(crate::gameserver::appserver::skills::yinyang2::YIN_YANG_2_SKILL_ID) } else { ai.player_skill_execution_mut(YIN_YANG_SKILL_ID) } }

fn last_used(ai: &CPlayerAI, second: bool) -> u32 { if second { ai.skill_last_used_ms(crate::gameserver::appserver::skills::yinyang2::YIN_YANG_2_SKILL_ID) } else { ai.skill_last_used_ms(YIN_YANG_SKILL_ID) } }
fn mark_used(ai: &mut CPlayerAI, second: bool, now: u32) { if second { ai.mark_skill_used(crate::gameserver::appserver::skills::yinyang2::YIN_YANG_2_SKILL_ID, now); } else { ai.mark_skill_used(YIN_YANG_SKILL_ID, now); } }

fn finish_player_yin_yang<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    second: bool,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        mark_used(player_ai, second, now_ms);
    });
}

fn abort_player_yin_yang(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

pub(crate) fn complete_player_yin_yang_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    second: bool,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = execution(player_ai, second).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_yin_yang(game, player_id, player_ai, second, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_yin_yang_family<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    second: bool,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = execution(player_ai, second).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    abort_player_yin_yang(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_yin_yang<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    execute_player_yin_yang_family(game, player_id, dispatch, player_ai, runtime, YIN_YANG_SKILL_ID, false)
}

pub(super) fn execute_player_yin_yang_family<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime, skill_id: u32, second: bool) -> QueuedSkillExecutionOutcome {
    let requested = match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id };
    if requested != skill_id { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let skill_level = player.learned_skill_level(skill_id);
    let Some(region_id) = player.server_region_id() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        if execution(player_ai, second).is_some() {
            abort_player_yin_yang(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_scale = properties.query_property(SKILL_USAGE_EM_MODIFIER);

    if execution(player_ai, second).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(last_used(player_ai, second), cooldown_ms, cooldown_now_ms) {
            send_error(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { send_error(game, player_id, 10, mp_loss); return terminal(QueuedSkillExecutionState::Rejected); };
        if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) { send_error(game, player_id, 10, mp_loss); return terminal(QueuedSkillExecutionState::Rejected); }
        let Some(source) = game.find_player(player_id).and_then(CPlayer::shape_view) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.base_magic_path(region_id, source.tile_x, source.tile_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize { send_error(game, player_id, 0x0b, mp_loss); return terminal(QueuedSkillExecutionState::Rejected); }
        if path.iter().any(|cell| cell.2 == 2) { send_error(game, player_id, 0x0f, mp_loss); return terminal(QueuedSkillExecutionState::Rejected); }
        if mp_loss == 0 || player.mana() < mp_loss { if mp_loss != 0 { send_error(game, player_id, 7, mp_loss); } return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id)); }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if execution(player_ai, second).is_none_or(|execution| execution.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }

    if execution(player_ai, second).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { abort_player_yin_yang(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
        if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) { send_error(game, player_id, 10, mp_loss); abort_player_yin_yang(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss { send_error(game, player_id, 7, mp_loss); abort_player_yin_yang(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            if let Some(source) = player.shape_view() { player.movement_shape_mut().set_direction(get_line_direction(source.tile_x, source.tile_y, target_x, target_y)); }
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let _ = game.update_player_criminal_state(player_id, GamePlayerFightStatePhase::MoveShapeAi, runtime);
        send_visual(game, player_id, skill_id, skill_level, 1, None);
        if let Some(execution) = execution_mut(player_ai, second) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started_at_ms = execution(player_ai, second).map(SkillExecutionKernel::started_at_ms).expect("выполнение инь-ян создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { abort_player_yin_yang(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
    if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) { send_error(game, player_id, 10, mp_loss); abort_player_yin_yang(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
    send_visual(game, player_id, skill_id, skill_level, 2, Some((target_x, target_y)));
    let Some(player) = game.find_player(player_id) else { abort_player_yin_yang(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
    let master = MasterInfo { master_type: PLAYER_TYPE, master_id: player_id, master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(player.pk_permissions().player), permitted_to_kill_teammate: i32::from(player.pk_permissions().teammate), permitted_to_kill_guild_member: i32::from(player.pk_permissions().guild_member), permitted_to_kill_criminal: i32::from(player.pk_permissions().criminal) };
    let combat = player.combat_properties();
    let scaled_element = truncate_original(
        f64::from(element_scale)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let element_modifier = (combat.add_element_attack as i32).wrapping_add(scaled_element);
    let critical_chance = i32::from(combat.cch);
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CYinYangPhalanx::new_for_skill(skill_id, summon_id, master, summon_started_at_ms, lifetime_ms, skill_level, minimum_attack, maximum_attack, element_modifier, critical_chance);
    phalanx.shape_mut().set_region_id(region_id);
    let summoned = game.add_yin_yang_phalanx(region_id, phalanx, target_x, target_y, summon_started_at_ms, runtime).is_some_and(|result| result.is_ok());
    if summoned { let _ = game.send_yin_yang_phalanx_entry(region_id, summon_id); }
    if let Some(execution) = execution_mut(player_ai, second) { let _ = execution.advance(SkillStage::Check, SkillStage::Calculate); let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack); let _ = execution.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_yin_yang(game, player_id, player_ai, second, runtime);
    terminal(if summoned { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}

pub(crate) const fn is_yin_yang_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: YIN_YANG_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: YIN_YANG_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: YIN_YANG_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } })
}
