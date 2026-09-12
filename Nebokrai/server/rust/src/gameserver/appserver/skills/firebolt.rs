//! Огненная стрела `CFireBolt` (`0x132`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/firebolt.cpp`. Владелец сохраняет двойную проверку MP,
//! время восстановления, расстояние, направление, задержку и точный пакет
//! запуска с длительностью полёта. После применения создаётся принадлежащий
//! региону `CFireBoltPhalanx`; урон выполняется только его ИИ после строгой
//! временной границы. При создании допустимого снаряда канонический
//! `CSoulCollectState` целиком потребляется, его visual завершается, а оба
//! множителя передаются phalanx-owner-у. Обычное, отказное и клиентское
//! завершение после `Begin` используют подтверждённый хвост `End(1)`: возврат
//! движения, `AfterUseSkill`, очистку текущего навыка и фиксацию времени
//! восстановления.
//! Reuse использует exact `CSkill::IsRestored`; cast и phalanx flight — elapsed.
//! Временные данные прежнего исполнителя теперь принадлежат этому owner-у,
//! без зависимости от перенесённого зарегистрированного CBaseMagic.
//! Координатная перегрузка `Begin` разрешает первый `CMoveShape` клетки через
//! точный `CState::GetSufferer` без fallback к заклинателю.

use super::baseattack::{finish_delayed_base_attack, real_distance, time_reached};
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::fireboltphalanx::CFireBoltPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::soulcollectstate::send_soul_collect_state_visual;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::resolve_coordinate_sufferer;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FireBoltExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
    condition_checked: bool,
}

impl FireBoltExecutionState {
    pub(crate) const fn begin(
        dispatch: PlayerSkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
            condition_checked: false,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }

    pub(crate) const fn condition_checked(self) -> bool {
        self.condition_checked
    }

    pub(crate) fn mark_condition_checked(&mut self) {
        self.condition_checked = true;
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

pub(crate) const FIRE_BOLT_SKILL_ID: u32 = 0x132;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn target_dead(game: &CGame, region_id: i32, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => true,
    }
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(FIRE_BOLT_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(
    game: &mut CGame,
    player_id: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    level: i32,
    attack_time_ms: i32,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(FIRE_BOLT_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_long(attack_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_fire_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_delayed_base_attack(game, player_id, FIRE_BOLT_SKILL_ID, runtime);
}

pub(crate) fn cancel_player_fire_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_state::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID).copied().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    finish_player_fire_bolt(game, player_id, player_ai, runtime);
    game.finish_player_skill(player_id, player_ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) const fn is_fire_bolt_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point {
            skill_id: FIRE_BOLT_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: FIRE_BOLT_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
        }
    )
}

pub(crate) fn execute_player_fire_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some((region_id, source_x, source_y, level, initial_mana)) =
        game.find_player(player_id).and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(FIRE_BOLT_SKILL_ID, game.skill_factory()),
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let target = match dispatch {
        PlayerSkillDispatch::Object { skill_id: FIRE_BOLT_SKILL_ID, target } => target,
        PlayerSkillDispatch::Point { skill_id: FIRE_BOLT_SKILL_ID, x, y } => {
            let Some(target) = resolve_coordinate_sufferer(game, region_id, x, y) else {
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) {
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            target
        }
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(properties) = game.skill_base_properties(FIRE_BOLT_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let summoned_speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let summoned_lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if game.player_skill_state::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID).copied().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, FIRE_BOLT_SKILL_ID),
            reuse_delay_ms,
            cooldown_now_ms,
        ) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id, source_x, source_y, target_view.tile_x, target_view.tile_y, None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(FIRE_BOLT_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, FireBoltExecutionState::begin(
            dispatch, target, started_at_ms,
        ));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_state::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID).copied().is_none_or(|state| state.kernel().dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if target_dead(game, region_id, target) {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        finish_player_fire_bolt(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_state::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID).copied().is_some_and(|state| !state.condition_checked()) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish_player_fire_bolt(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 10);
            finish_player_fire_bolt(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, target_view.tile_x, target_view.tile_y,
            ));
            player.set_skill_moveable(false);
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_start(game, player_id, level);
        if let Some(state) = game.player_skill_state_mut::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID) {
            state.mark_condition_checked();
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = game.player_skill_state::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID).copied()
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение огненной стрелы создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        send_failure(game, player_id, 10);
        finish_player_fire_bolt(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target_dead(game, region_id, target) {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        send_failure(game, player_id, 10);
        finish_player_fire_bolt(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let attack_time_ms = real_distance(
        source_x, source_y, target_view.tile_x, target_view.tile_y,
    )
    .wrapping_mul(summoned_speed as i32);
    send_fire(
        game, player_id, target, target_view.tile_x, target_view.tile_y, level, attack_time_ms,
    );

    let forced_distance = real_distance(
        source_x, source_y, target_view.tile_x, target_view.tile_y,
    ) as u32;
    let path = game.base_magic_path(
        region_id,
        source_x,
        source_y,
        target_view.tile_x,
        target_view.tile_y,
        Some(forced_distance),
    );
    if !path.is_empty() && path.iter().all(|cell| cell.2 != 2) {
        let permissions = game.find_player(player_id).map(CPlayer::pk_permissions).unwrap_or_default();
        let master = game.find_player(player_id).map(|player| MasterInfo {
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
        }).unwrap_or_default();
        let soul = game
            .find_player_mut(player_id)
            .and_then(CPlayer::take_soul_collect_state);
        let (soul_count, soul_variable) = soul.map_or((0, 0), |state| {
            send_soul_collect_state_visual(
                game,
                region_id,
                ShapeIdentity {
                    object_type: PLAYER_TYPE,
                    id: player_id,
                    ex_id: Default::default(),
                },
                source_x,
                source_y,
                state,
                false,
            );
            (state.souls(), state.variable_percent() as i32)
        });
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CFireBoltPhalanx::new(
            summon_id,
            master,
            summon_started_at_ms,
            summoned_lifetime,
            level,
            minimum_attack,
            maximum_attack,
            element_modifier,
            target,
            attack_time_ms as u32,
            soul_count,
            soul_variable,
        );
        phalanx.shape_mut().set_region_id(region_id);
        let (tile_x, tile_y, _) = path[0];
        let (area_width, area_height) = game.area_dimensions();
        let result = if let Some(mut owner) = game.take_region_owner(region_id) {
            let result = owner.base_mut().add_fire_bolt_phalanx(
                phalanx,
                tile_x,
                tile_y,
                area_width,
                area_height,
                summon_started_at_ms,
                runtime,
            );
            game.restore_region_owner(owner);
            Some(result)
        } else {
            None
        };
        tracing::trace!(region_id, player_id, summon_id, ?result, "создан снаряд огненной стрелы");
    }

    if let Some(state) = game.player_skill_state_mut::<FireBoltExecutionState>(player_id, FIRE_BOLT_SKILL_ID) {
        let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_fire_bolt(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
