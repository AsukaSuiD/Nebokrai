//! Божественное благословение `CGodBless` (`0x12F`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godbless.cpp`. Здесь находятся выбор цели, cooldown,
//! двустадийный расход MP, задержка, `SkillExecutionKernel`, визуальный пакет,
//! формулы трёх прибавок и replacement `GodBlessState`. Обычный неприручённый
//! и не транспортный монстр подтверждённо заменяется самим заклинателем.
//! `CGame` оставляет только доступ к владельцам, применение и доставку.
//! Каждая прибавка сохраняет обе исходные точки округления до `float`, после
//! чего x87 усекает итог к нулю перед созданием состояния.
//! Семейный reuse-gate использует exact `CSkill::IsRestored`, отдельно от
//! elapsed-задержки каста.
//! Begin заканчивается возвратом Begun после создания исполнения. Проверки
//! и эффекты первого AI остаются после этой границы; координатор вызывает AI
//! в том же Run после постановки Attack, не сдвигая исходное время Begin.
//! Первичная replacement-граница публикует настоящий AI источника. God1
//! сначала завершает Extended Original/type0x12F, затем обе версии заменяют
//! первый GodBless1/2 по runtime-позиции. Новый ctor и Begin выполняются
//! после старого End; clock начала срока принадлежит Begin, не формуле.
//! AI завершает skill через End(1) и при отказе нового state Begin
//! (0x005B0930/0x00550E70); такой отказ не превращается в отмену каста.

use super::baseattack::time_reached;
use super::godblessstate::GodBlessState;
use super::godbless2::GOD_BLESS_2_SKILL_ID;
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::stateskill::finish_state_skill;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const GOD_BLESS_SKILL_ID: u32 = 0x12f;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_ELEMENT_GAIN: u32 = 115;
const TARGET_MINIMUM_GAIN: u32 = 116;
const TARGET_MAXIMUM_GAIN: u32 = 117;
const TARGET_ELEMENT_COEFFICIENT: u32 = 120;
const TARGET_MINIMUM_COEFFICIENT: u32 = 121;
const TARGET_MAXIMUM_COEFFICIENT: u32 = 122;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const REUSE_DELAY_TIME: u32 = 10_005;

#[derive(Clone, Copy)]
struct Target { identity: ShapeIdentity, x: i32, y: i32 }

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false } }

fn requested_target(game: &CGame, region_id: i32, player_id: i32, skill_id: u32, dispatch: PlayerSkillDispatch) -> Option<Target> {
    let requested = match dispatch {
        PlayerSkillDispatch::Object { skill_id: requested_skill, target } if requested_skill == skill_id && matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => target,
        PlayerSkillDispatch::SelfTarget { skill_id: requested_skill, .. } | PlayerSkillDispatch::Point { skill_id: requested_skill, .. } if requested_skill == GOD_BLESS_SKILL_ID && requested_skill == skill_id => ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID },
        _ => return None,
    };
    let identity = if requested.object_type == MONSTER_TYPE {
        let ordinary = game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(requested.id)).and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(!monster.is_tamed() && !monster.is_carriage(property))
        }).unwrap_or(false);
        if ordinary { ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID } } else { requested }
    } else { requested };
    let (x, y) = game.move_shape_target_tile(Some(region_id), identity)?;
    Some(Target { identity, x, y })
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_cast(game: &mut CGame, player_id: i32, skill_id: u32, target: Target, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(target.identity.object_type); message.add_long(target.identity.id); message.add_long(target.x); message.add_long(target.y);
    } else { message.add_long(player.shape().get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_god_bless<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _player_ai: &mut CPlayerAI, skill_id: u32, runtime: &mut Runtime) { restore_player_movement(game, player_id); finish_state_skill(game, player_id, skill_id, runtime); }
fn abort_player_god_bless(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }
pub(crate) fn complete_player_god_bless<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, skill_id: u32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = game.player_skill_execution(player_id, skill_id).map(SkillExecutionKernel::dispatch) else { return false }; let skill_id = match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id }; finish_player_god_bless(game, player_id, player_ai, skill_id, runtime); game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Completed) }
pub(crate) fn cancel_player_god_bless<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, skill_id: u32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool { let Some(dispatch) = game.player_skill_execution(player_id, skill_id).map(SkillExecutionKernel::dispatch) else { return false }; abort_player_god_bless(game, player_id); game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled) }

fn gains(base: u32, coefficient: u32, weapon: u32) -> u32 {
    let scaled_bits = coefficient.wrapping_mul(weapon);
    let scaled = (f64::from(scaled_bits) * f64::from(0.01_f32)) as f32;
    let gain = (f64::from(base) + f64::from(scaled)) as f32;
    truncate_original(f64::from(gain)) as u32
}

pub(crate) fn execute_player_god_bless<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id };
    if !matches!(skill_id, GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID) { return terminal(QueuedSkillExecutionState::Rejected); }
    if skill_id == GOD_BLESS_2_SKILL_ID && !matches!(dispatch, PlayerSkillDispatch::Object { target: ShapeIdentity { object_type: MONSTER_TYPE, .. }, .. }) {
        game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0305");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(skill_id, game.skill_factory()), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay = properties.query_property(DELAY_TIME);
    let cooldown = properties.query_property(REUSE_DELAY_TIME);
    let keep_time = properties.query_property(STATE_PERSIST_TIME);
    let minimum_base = properties.query_property(TARGET_MINIMUM_GAIN);
    let minimum_coefficient = properties.query_property(TARGET_MINIMUM_COEFFICIENT);
    let maximum_base = properties.query_property(TARGET_MAXIMUM_GAIN);
    let maximum_coefficient = properties.query_property(TARGET_MAXIMUM_COEFFICIENT);
    let element_base = properties.query_property(TARGET_ELEMENT_GAIN);
    let element_coefficient = properties.query_property(TARGET_ELEMENT_COEFFICIENT);
    if game.player_skill_execution(player_id, skill_id).is_none() {
        let started = runtime.now_milliseconds();
        let cooldown_now = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, skill_id),
            cooldown,
            cooldown_now,
        ) {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || initial_mana < mp_loss { if mp_loss != 0 { send_failure(game, player_id, 7, mp_loss); } return terminal(QueuedSkillExecutionState::Rejected); }
        if requested_target(game, region_id, player_id, skill_id, dispatch).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(skill_id)); }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, skill_id).is_none_or(|execution| execution.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(target) = requested_target(game, region_id, player_id, skill_id, dispatch) else { abort_player_god_bless(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
    if game.player_skill_execution(player_id, skill_id).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if mana < mp_loss { send_failure(game, player_id, 7, mp_loss); abort_player_god_bless(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_cast(game, player_id, skill_id, target, level, false);
        if let Some(execution) = game.player_skill_execution_mut(player_id, skill_id) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = game.player_skill_execution(player_id, skill_id).map(SkillExecutionKernel::started_at_ms).expect("выполнение божественного благословения создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending); }
    send_cast(game, player_id, skill_id, target, level, true);
    let weapon = game.find_player(player_id).map(|player| player.weapon_damage_level(game.goods_factory()) as u32).unwrap_or(0);
    let minimum_gain = gains(minimum_base, minimum_coefficient, weapon);
    let maximum_gain = gains(maximum_base, maximum_coefficient, weapon);
    let element_gain = gains(element_base, element_coefficient, weapon);
    let user = ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID };
    let _ = game.with_published_player_ai(player_id, player_ai, |game| {
        game.install_god_bless_state(region_id, target.identity, user, skill_id,
            || GodBlessState::new(skill_id, 0, keep_time, minimum_gain, maximum_gain, element_gain), runtime)
    });
    if let Some(execution) = game.player_skill_execution_mut(player_id, skill_id) { let _ = execution.advance(SkillStage::Check, SkillStage::Calculate); let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack); let _ = execution.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_god_bless(game, player_id, player_ai, skill_id, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) const fn is_god_bless_skill(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } })
}
