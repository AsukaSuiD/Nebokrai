//! Сфера хаоса `CChaosSphere` (`0x137`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/chaossphere.cpp`. Владелец сохраняет координатную и объектную
//! перегрузки, двойную проверку MP, cooldown, delay, направление, точные
//! визуальные пакеты и построение движущегося `CChaosSpherePhalanx`.
//! `CGame` предоставляет только разрешение владельцев, регион, регистрацию и
//! фактическую доставку.
//! `End(1)` возвращает движение и завершает общий `CSummonSkill::End` после
//! построения сферы; отмена использует `End(0)` без cooldown.
//! Стихийная прибавка вычисляется в расширенной точности x87 из целых свойств
//! и сохранённой `f32`-константы, затем усекается к нулю. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; delay и движение сферы
//! остаются elapsed.

use super::baseattack::time_reached;
use super::fightdefense::truncate_original;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED,
};
use super::chaosspherephalanx::CChaosSpherePhalanx;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const CHAOS_SPHERE_SKILL_ID: u32 = 0x137;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChaosSphereExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
}

impl ChaosSphereExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), condition_checked: false }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn condition_checked(&self) -> bool { self.condition_checked }
    pub(crate) fn mark_condition_checked(&mut self) { self.condition_checked = true; }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn finish_player_chaos_sphere<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(CHAOS_SPHERE_SKILL_ID, now_ms);
    });
}

fn abort_player_chaos_sphere(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    abort_skill(game, player_id);
}

pub(crate) fn complete_player_chaos_sphere<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_chaos_sphere(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_chaos_sphere<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID)
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    abort_player_chaos_sphere(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn target_position(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: CHAOS_SPHERE_SKILL_ID, x, y } => Some((x, y)),
        PlayerSkillDispatch::Object { skill_id: CHAOS_SPHERE_SKILL_ID, target }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
                let alive = match target.object_type {
                    PLAYER_TYPE => game.find_player(target.id).is_some_and(|player| !player.is_dead()),
                    MONSTER_TYPE => game.find_region(region_id)
                        .and_then(|owner| owner.base().find_monster_by_id(target.id))
                        .is_some_and(|monster| monster.hit_points() != 0),
                    _ => false,
                };
                alive.then(|| game.base_magic_target_view(region_id, target))
                    .flatten()
                    .map(|view| (view.tile_x, view.tile_y))
            }
        _ => None,
    }
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(CHAOS_SPHERE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_apply(game: &mut CGame, player_id: i32, level: i32, x: i32, y: i32) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(CHAOS_SPHERE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(x);
    message.add_long(y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) const fn is_chaos_sphere_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: CHAOS_SPHERE_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: CHAOS_SPHERE_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

pub(crate) fn execute_player_chaos_sphere<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_chaos_sphere_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?,
        player.learned_skill_level(CHAOS_SPHERE_SKILL_ID), player.mana(),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(CHAOS_SPHERE_SKILL_ID, level) else {
        if player_ai.player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID).is_some() {
            abort_player_chaos_sphere(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected)
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let speed_ms = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);

    if player_ai.player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            player_ai.skill_last_used_ms(CHAOS_SPHERE_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, 13);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if target_position(game, region_id, dispatch).is_none() {
            send_failure(game, player_id, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(CHAOS_SPHERE_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(ChaosSphereExecutionState::begin(dispatch, started_at_ms));
    } else if player_ai.player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID).is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y)) = target_position(game, region_id, dispatch) else {
        send_failure(game, player_id, 10);
        if matches!(dispatch, PlayerSkillDispatch::Object { .. }) { game.send_skill_system_info(player_id, b"GS0285"); }
        abort_player_chaos_sphere(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if player_ai.player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID).is_some_and(|state| !state.condition_checked()) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_chaos_sphere(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_start(game, player_id, level);
        if let Some(state) = player_ai.player_skill_state_mut::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID) {
            state.mark_condition_checked();
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = player_ai.player_skill_state::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID).map(|state| state.kernel().started_at_ms()).unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_apply(game, player_id, level, target_x, target_y);

    let forced_length = if speed_ms == 0 { 0 } else { lifetime_ms / speed_ms };
    let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, Some(forced_length));
    if let Some(blocked) = path.iter().position(|cell| cell.2 == 2) { path.truncate(blocked); }
    if !path.is_empty() {
        let player = game.find_player(player_id).expect("владелец сферы существует во время применения");
        let permissions = player.pk_permissions();
        let combat = player.combat_properties();
        let master = MasterInfo {
            master_type: PLAYER_TYPE,
            master_id: player_id,
            master_guild_id: player.faction_id(),
            master_team_id: player.team_id(),
            master_union_id: player.union_id(),
            master_country_id: 0,
            permitted_to_kill_player: i32::from(permissions.player),
            permitted_to_kill_teammate: i32::from(permissions.teammate),
            permitted_to_kill_guild_member: i32::from(permissions.guild_member),
            permitted_to_kill_criminal: i32::from(permissions.criminal),
        };
        let element_bonus = truncate_original(
            f64::from(element_modifier)
                * f64::from(0.01_f32)
                * f64::from(combat.element_modify),
        );
        let path_xy: Vec<_> = path.iter().map(|&(x, y, _)| (x, y)).collect();
        let (tile_x, tile_y) = path_xy[0];
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CChaosSpherePhalanx::new(
            summon_id, master, summon_started_at_ms, lifetime_ms, level,
            frequency_ms,
            minimum_attack,
            maximum_attack,
            (combat.add_element_attack as i32).wrapping_add(element_bonus),
            path_xy, speed_ms, i32::from(combat.cch),
        );
        phalanx.shape_mut().set_region_id(region_id);
        if game.add_chaos_sphere_phalanx(region_id, phalanx, tile_x, tile_y, summon_started_at_ms, runtime).is_some_and(|result| result.is_ok()) {
            let _ = game.send_chaos_sphere_phalanx_entry(region_id, summon_id, runtime);
        }
    }
    if let Some(state) = player_ai.player_skill_state_mut::<ChaosSphereExecutionState>(CHAOS_SPHERE_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_chaos_sphere(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
