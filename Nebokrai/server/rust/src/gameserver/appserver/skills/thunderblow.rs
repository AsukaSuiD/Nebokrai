//! Громовой удар `CThunderBlow` (`0x13F`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderblow.cpp`. Владелец сохраняет две проверки MP и
//! дальности, время восстановления, задержку, направление и визуальные
//! пакеты. После задержки он создаёт в целевой проходимой клетке принадлежащую
//! региону форму; её срок жизни, поиск цели и формула остаются у владельца формы.
//! Собственный `End` не меняет движение, но вызывает оружейный
//! `CAttackSkill::End`; успех, отказ после `Begin` и клиентская отмена используют
//! один хвост с `AfterUseSkill` и временем восстановления. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; задержка формы остаётся elapsed.

use super::baseattack::{
    finish_immediate_base_attack, time_reached, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME,
};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::thunderblowphalanx::CThunderBlowPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const THUNDER_BLOW_SKILL_ID: u32 = 0x13f;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn finish_player_thunder_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_immediate_base_attack(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(THUNDER_BLOW_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_thunder_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(THUNDER_BLOW_SKILL_ID).map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_thunder_blow(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(),
        master_guild_id: player.faction_id(), master_team_id: player.team_id(),
        master_union_id: player.union_id(), master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
            let view = game.base_magic_target_view(region_id, target)?;
            Some((view.tile_x, view.tile_y, Some(target)))
        }
        _ => None,
    }
}

fn target_dead(game: &CGame, region_id: i32, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game.find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => true,
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, destination: Option<(ShapeIdentity, i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if destination.is_some() { 2 } else { 1 });
    message.add_long(THUNDER_BLOW_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if let Some((target, x, y)) = destination {
        message.add_long(target.object_type); message.add_long(target.id);
        message.add_long(x); message.add_long(y);
    } else { message.add_long(player.shape().get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) const fn is_thunder_blow_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: THUNDER_BLOW_SKILL_ID, .. }
        | PlayerSkillDispatch::Object {
            skill_id: THUNDER_BLOW_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
        }
    )
}

pub(crate) fn execute_player_thunder_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_thunder_blow_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some((region_id, level, initial_mana, source_x, source_y)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.learned_skill_level(THUNDER_BLOW_SKILL_ID), player.mana(),
        player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(THUNDER_BLOW_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_execution(THUNDER_BLOW_SKILL_ID).is_none() {
        if !skill_is_restored(
            player_ai.skill_last_used_ms(THUNDER_BLOW_SKILL_ID),
            cooldown_ms,
            runtime.now_milliseconds(),
        ) { send_failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected); }
        let Some((target_x, target_y, _)) = destination(game, region_id, dispatch) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            if mp_loss != 0 { send_failure(game, player_id, 7, mp_loss); }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_current_skill_id(Some(THUNDER_BLOW_SKILL_ID)); }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, runtime.now_milliseconds()));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_execution(THUNDER_BLOW_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y, target)) = destination(game, region_id, dispatch) else {
        finish_player_thunder_blow(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if player_ai.player_skill_execution(THUNDER_BLOW_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            finish_player_thunder_blow(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b, mp_loss);
            finish_player_thunder_blow(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, None);
        if let Some(state) = player_ai.player_skill_execution_mut(THUNDER_BLOW_SKILL_ID) { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = player_ai.player_skill_execution(THUNDER_BLOW_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение громового удара создано");
    if !time_reached(runtime.now_milliseconds(), started, delay_ms) { return terminal(QueuedSkillExecutionState::Pending); }
    if target.is_some_and(|identity| target_dead(game, region_id, identity)) {
        send_failure(game, player_id, 10, mp_loss);
        finish_player_thunder_blow(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    send_visual(game, player_id, level, Some((target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: Default::default() }), target_x, target_y)));
    let summon_id = game.allocate_summon_shape_id();
    let started_at_ms = runtime.now_milliseconds();
    let master = game.find_player(player_id).map(master_info).unwrap_or_default();
    let mut phalanx = CThunderBlowPhalanx::new(
        summon_id, master, started_at_ms, lifetime_ms, level, minimum, maximum, element_modifier,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let result = if game.find_region(region_id).is_some_and(|owner| owner.base().block_at(target_x, target_y) != Some(2)) {
        game.add_thunder_blow_phalanx(region_id, phalanx, target_x, target_y, started_at_ms, runtime)
    } else { None };
    if result.as_ref().is_some_and(|result| result.is_ok()) {
        let _ = game.send_thunder_blow_phalanx_entry(region_id, summon_id, runtime);
    }
    tracing::trace!(region_id, player_id, summon_id, ?result, "создана форма громового удара");
    if let Some(state) = player_ai.player_skill_execution_mut(THUNDER_BLOW_SKILL_ID) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_thunder_blow(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
