//! Базовая магическая атака GameServer (`SKILL_BASE_MAGIC == 3`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/basemagic.cpp`. `SkillExecutionKernel` сохраняет применение
//! между тактами: первый такт проверяет цель, поворачивает игрока, отправляет
//! начало эффекта и запрещает движение; по истечении задержки движение
//! разрешается до повторной проверки цели и отправки пакета выстрела. Сам урон
//! намеренно не выполняется здесь: выстрел создаёт принадлежащий региону
//! `CBaseMagicPhalanx`, который атакует в ИИ региона после отдельной задержки
//! полёта. Monster-source проходит тот же общий cast без player-only проверок
//! и создаёт ту же phalanx; её исходный расчёт затем ищет attacker ID только в
//! player-map, поэтому такой снаряд остаётся визуальным без выдуманного урона.
//! Это сохраняет исходные моменты действий, двух владельцев жизненного
//! цикла и порядок пакетов. Обычное, отказное и клиентское завершение после
//! `Begin` используют один хвост `End(1)` с износом оружия и временем
//! восстановления. Все
//! достигнутые перегрузки, проверки и визуальные пакеты реализованы этим
//! владельцем; `CGame` оставляет только доступ к региону, владельцам целей и
//! фактическую доставку. Общий `CState::GetSufferer` разрешает типы player
//! `400`, NPC `500`, monster `600`, build `1100` и gate `1200`; NPC сразу
//! отклоняется как мёртвый, а стационарные цели проходят тот же region-owned
//! combat lifecycle. Значение `100` относится к другой legacy enum и не
//! является `CShape::GetType`.

use super::baseattack::{finish_delayed_base_attack, real_distance, time_reached};
use super::basemagicphalanx::CBaseMagicPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::build::BUILD_OBJECT_TYPE;
use crate::gameserver::appserver::citygate::CITY_GATE_OBJECT_TYPE;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;

pub(crate) const BASE_MAGIC_SKILL_ID: u32 = 3;
pub(crate) const BASE_MAGIC_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
pub(crate) const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub(crate) const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;
pub(crate) const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;
pub(crate) const SKILL_USAGE_SUMMONED_SPEED: u32 = 30_002;

/// Типы, которые исходный `CState::GetSufferer` разрешает в object-target
/// перегрузках семейства базовой магии. NPC доходит до owner-а и уже там
/// отклоняется как мёртвый; постройки продолжают region-owned combat path.
pub(crate) const fn is_base_magic_object_target_type(object_type: i32) -> bool {
    matches!(object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE)
        || object_type == BUILD_OBJECT_TYPE as i32
        || object_type == CITY_GATE_OBJECT_TYPE as i32
}

impl CGame {
    /// Общий legacy failure-пакет базовой магии и стрельбы остаётся рядом с
    /// семейством навыков; `CGame` предоставляет только фактическую сетевую доставку.
    pub(crate) fn send_base_magic_failure(&self, player_id: i32, action: u8) {
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(0);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BaseMagicExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
    condition_checked: bool,
}

impl BaseMagicExecutionState {
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

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
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

fn finish_player_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_delayed_base_attack(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_base_magic_used(now_ms);
    });
}

pub(crate) fn cancel_player_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.base_magic().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    finish_player_base_magic(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let rejected = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Rejected,
        first_contact: false,
        killing_blow: None,
    };
    let pending = || QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Pending,
        first_contact: false,
        killing_blow: None,
    };
    let Some(player) = game.find_player(player_id) else {
        return rejected();
    };
    let Some(region_id) = player.server_region_id() else {
        return rejected();
    };
    let skill_level = player.learned_skill_level(BASE_MAGIC_SKILL_ID);
    let Some(properties) = game.skill_base_properties(BASE_MAGIC_SKILL_ID, skill_level)
    else {
        return rejected();
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let summoned_speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let summoned_lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();
    let target = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => target,
        _ => {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        }
    };

    if player_ai.base_magic().is_none() {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            game.send_base_magic_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return rejected();
        }
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.base_magic_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.base_magic_last_used_ms(),
                reuse_delay_ms,
            )
        {
            game.send_base_magic_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return rejected();
        }
        let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        };
        let (source_x, source_y) = match (
            player.shape().get_tile_x(),
            player.shape().get_tile_y(),
        ) {
            (Ok(x), Ok(y)) => (x, y),
            _ => return rejected(),
        };
        let Some((target_x, target_y)) =
            game.base_magic_target_point(region_id, source_x, source_y, target)
        else {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        };
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            target_x,
            target_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            game.send_base_magic_failure(player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return rejected();
        }
        let target_dead = game.base_magic_target_dead(region_id, target);
        if target_dead {
            game.send_base_magic_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0285");
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                target_x,
                target_y,
            ));
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(BASE_MAGIC_SKILL_ID));
        }
        let direction = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction())
            .unwrap_or_default();
        let mut start = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        start.add_byte(1);
        start.add_long(BASE_MAGIC_SKILL_ID as i32);
        start.base_mut().add_short(skill_level as i16);
        start.add_long(PLAYER_TYPE);
        start.add_long(player_id);
        start.add_long(direction);
        let _ = game.send_player_shape_around(player_id, None, &start);
        let mut execution = BaseMagicExecutionState::begin(dispatch, target, now_ms);
        let _ = execution
            .kernel_mut()
            .advance(SkillStage::Begin, SkillStage::Check);
        execution.mark_condition_checked();
        player_ai.begin_base_magic(execution);
        let first_ai_now_ms = runtime.now_milliseconds();
        if !time_reached(first_ai_now_ms, now_ms, delay_ms) {
            return pending();
        }
    } else if player_ai
        .base_magic()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return rejected();
    } else if !time_reached(
        now_ms,
        player_ai
            .base_magic()
            .map_or(now_ms, |state| state.kernel().started_at_ms()),
        delay_ms,
    ) {
        return pending();
    }

    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
        game.send_base_magic_failure(player_id, 10);
        finish_player_base_magic(game, player_id, player_ai, runtime);
        return rejected();
    };
    let target_dead = game.base_magic_target_dead(region_id, target);
    if target_dead {
        game.send_base_magic_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        game.send_base_magic_failure(player_id, 10);
        finish_player_base_magic(game, player_id, player_ai, runtime);
        return rejected();
    }
    let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
        finish_player_base_magic(game, player_id, player_ai, runtime);
        return rejected();
    };
    let Some((target_x, target_y)) = game.base_magic_target_point(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target,
    ) else {
        finish_player_base_magic(game, player_id, player_ai, runtime);
        return rejected();
    };
    let attack_time = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
    )
    .wrapping_mul(summoned_speed as i32);
    let mut fire = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    fire.add_byte(2);
    fire.add_long(BASE_MAGIC_SKILL_ID as i32);
    fire.base_mut().add_short(skill_level as i16);
    fire.add_long(PLAYER_TYPE);
    fire.add_long(player_id);
    fire.add_long(target.object_type);
    fire.add_long(target.id);
    fire.add_long(target_x);
    fire.add_long(target_y);
    fire.add_long(attack_time);
    let _ = game.send_player_shape_around(player_id, None, &fire);

    let forced_distance = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
    ) as u32;
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_x,
        target_y,
        Some(forced_distance),
    );
    if !path.is_empty() && path.iter().all(|cell| cell.2 != 2) {
        let permissions = game
            .find_player(player_id)
            .map(CPlayer::pk_permissions)
            .unwrap_or_default();
        let master = game
            .find_player(player_id)
            .map(|player| MasterInfo {
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
            })
            .unwrap_or_default();
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CBaseMagicPhalanx::new(
            summon_id,
            master,
            summon_started_at_ms,
            summoned_lifetime,
            skill_level,
            minimum_attack,
            maximum_attack,
            element_modifier,
            attack_time as u32,
            target,
        );
        phalanx.shape_mut().set_region_id(region_id);
        let (tile_x, tile_y, _) = path[0];
        let result = game.add_base_magic_phalanx(
            region_id,
            phalanx,
            tile_x,
            tile_y,
            summon_started_at_ms,
            runtime,
        );
        tracing::trace!(region_id, player_id, summon_id, ?result, "создан снаряд базовой магии");
    }

    if let Some(state) = player_ai.base_magic_mut() {
        let _ = state
            .kernel_mut()
            .advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state
            .kernel_mut()
            .advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state
            .kernel_mut()
            .advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_base_magic(game, player_id, player_ai, runtime);
    QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Completed,
        first_contact: false,
        killing_blow: None,
    }
    }
