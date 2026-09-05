//! Божественная кара `CGodPunishment` (`0x13A`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godpunishment.cpp`. Здесь находятся достигнутые варианты
//! цели, две проверки расстояния, двухфазный расход MP, задержка, визуальные
//! пакеты, `SkillExecutionKernel` и построение phalanx. `CGame` оставляет за
//! собой только разрешение независимых владельцев, регистрацию и доставку.
//! `End(1)` сбрасывает execution-состояние после регистрации phalanx;
//! отмена использует `End(0)` без обновления свойств и cooldown. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; задержка phalanx остаётся elapsed.

use super::baseattack::time_reached;
use super::godpunishmentphalanx::CGodPunishmentPhalanx;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const GOD_PUNISHMENT_SKILL_ID: u32 = 0x13a;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const MP_LOSE: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const DELAY_TIME: u32 = 10_001;
const REUSE_TIME: u32 = 10_005;
const MIN_ATTACK: u32 = 20_001;
const MAX_ATTACK: u32 = 20_002;
const ELEMENT_MODIFIER: u32 = 20_015;
const SUMMONED_LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
fn position(game: &CGame, region: i32, player: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player).and_then(CPlayer::shape_view).map(|s| (s.tile_x, s.tile_y, None)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region, target).map(|s| (s.tile_x, s.tile_y, Some(target))),
    }
}
fn fail(game: &CGame, player: i32, code: u8, mp: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player, code);
    match code { 7 => game.send_skill_system_info_with_unsigned(player, b"GS0288", mp), 10 => game.send_skill_system_info(player, b"GS0285"), 0x0b => game.send_skill_system_info(player, b"GS0290"), 0x0d => game.send_skill_system_info(player, b"GS0278"), _ => {} }
}
fn visual(game: &mut CGame, player: i32, level: i32, action: u8, target: Option<(ShapeIdentity, i32, i32)>) {
    let Some(source) = game.find_player(player) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action); message.add_long(GOD_PUNISHMENT_SKILL_ID as i32); message.base_mut().add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player);
    if action == 1 { message.add_long(source.shape().get_direction()); } else {
        let (identity, x, y) = target.unwrap_or((ShapeIdentity { object_type: 0, id: 0, ex_id: crate::public::guid::CGuid::GUID_INVALID }, 0, 0));
        message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(x); message.add_long(y);
    }
    let _ = game.send_player_shape_around(player, None, &message);
}
fn finish_player_god_punishment<Runtime: GameMainLoopRuntime>(game: &mut CGame, player: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    if let Some(owner) = game.find_player_mut(player) { owner.set_skill_moveable(true); }
    finish_summon_skill(game, player, ai, runtime, |ai, now_ms| ai.mark_skill_used(GOD_PUNISHMENT_SKILL_ID, now_ms));
}

fn abort_player_god_punishment(game: &mut CGame, player: i32) {
    if let Some(owner) = game.find_player_mut(player) { owner.set_skill_moveable(true); }
    abort_skill(game, player);
}

pub(crate) fn complete_player_god_punishment<Runtime: GameMainLoopRuntime>(game: &mut CGame, player: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_god_punishment(game, player, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_god_punishment<Runtime: GameMainLoopRuntime>(game: &mut CGame, player: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_god_punishment(game, player);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_god_punishment<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let id = match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id };
    if id != GOD_PUNISHMENT_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let level = player.learned_skill_level(id); let Some(region) = player.server_region_id() else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(props) = game.skill_base_properties(id, level) else {
        if ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).is_some() { abort_player_god_punishment(game, player_id); }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse = props.query_property(REUSE_TIME); let maximum = props.query_property(MAX_DISTANCE); let mp = props.query_property(MP_LOSE); let delay = props.query_property(DELAY_TIME);
    let lifetime = props.query_property(SUMMONED_LIFETIME); let minimum = props.query_property(MIN_ATTACK) as i32; let maximum_attack = props.query_property(MAX_ATTACK) as i32; let element = props.query_property(ELEMENT_MODIFIER) as i32;
    if ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).is_none() {
        let started = runtime.now_milliseconds(); let now = runtime.now_milliseconds();
        if !skill_is_restored(ai.skill_last_used_ms(GOD_PUNISHMENT_SKILL_ID), reuse, now) { fail(game, player_id, 0x0d, mp); return terminal(QueuedSkillExecutionState::Rejected); }
        let Some((x, y, _)) = position(game, region, player_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) };
        let Some(source) = player.shape_view() else { return terminal(QueuedSkillExecutionState::Rejected) };
        if maximum != 0 && game.base_magic_path(region, source.tile_x, source.tile_y, x, y, None).len() > maximum as usize { fail(game, player_id, 0x0b, mp); return terminal(QueuedSkillExecutionState::Rejected); }
        if mp == 0 || player.mana() < mp { if mp != 0 { fail(game, player_id, 7, mp); } return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_current_skill_id(Some(id)); }
        ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started));
    } else if ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    if ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let Some((x, y, _)) = position(game, region, player_id, dispatch) else { abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if mana < mp { fail(game, player_id, 7, mp); abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp)); if let Some(source) = player.shape_view() { player.movement_shape_mut().set_direction(get_line_direction(source.tile_x, source.tile_y, x, y)); } }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); let _ = game.update_player_criminal_state(player_id, GamePlayerFightStatePhase::MoveShapeAi, runtime);
        let Some(source) = game.find_player(player_id).and_then(CPlayer::shape_view) else { abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
        if maximum != 0 && game.base_magic_path(region, source.tile_x, source.tile_y, x, y, None).len() > maximum as usize { fail(game, player_id, 0x0b, mp); abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
        visual(game, player_id, level, 1, None); if let Some(state) = ai.player_skill_execution_mut(GOD_PUNISHMENT_SKILL_ID) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = ai.player_skill_execution(GOD_PUNISHMENT_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("божественная кара начата");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some((x, y, target)) = position(game, region, player_id, dispatch) else { abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
    if target.is_some_and(|identity| game.periodic_state_target_dead(region, identity)) { fail(game, player_id, 10, mp); abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); }
    visual(game, player_id, level, 2, Some((target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: crate::public::guid::CGuid::GUID_INVALID }), x, y)));
    let Some(player) = game.find_player(player_id) else { abort_player_god_punishment(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) };
    let master = MasterInfo { master_type: PLAYER_TYPE, master_id: player_id, master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(player.pk_permissions().player), permitted_to_kill_teammate: i32::from(player.pk_permissions().teammate), permitted_to_kill_guild_member: i32::from(player.pk_permissions().guild_member), permitted_to_kill_criminal: i32::from(player.pk_permissions().criminal) };
    let summon_id = game.allocate_summon_shape_id(); let summon_time = runtime.now_milliseconds(); let mut phalanx = CGodPunishmentPhalanx::new(summon_id, master, summon_time, lifetime, level, minimum, maximum_attack, element); phalanx.shape_mut().set_region_id(region);
    let summoned = game.add_god_punishment_phalanx(region, phalanx, x, y, summon_time, runtime).is_some_and(|r| r.is_ok()); if summoned { let _ = game.send_god_punishment_phalanx_entry(region, summon_id, runtime); }
    if let Some(state) = ai.player_skill_execution_mut(GOD_PUNISHMENT_SKILL_ID) { let _ = state.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_god_punishment(game, player_id, ai, runtime); terminal(if summoned { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
pub(crate) const fn is_god_punishment_target(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: GOD_PUNISHMENT_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: GOD_PUNISHMENT_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: GOD_PUNISHMENT_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }) }
