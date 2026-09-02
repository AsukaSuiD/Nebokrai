//! Базовая атака боевой феи GameServer (`SKILL_BATTLEFAIRY_BASE_ATTACK`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/battlefairybasemagic.cpp`. Отдельный FIFO боевой феи
//! проходит общий `SkillExecutionKernel`, но не блокирует движение игрока.
//! Начало, выстрел и обязательное завершение используют визуальный тип `700`;
//! ошибки используют отдельный префикс `4`. Проверки цели, cooldown, стадии,
//! формирование пакетов и создание `CBattleFairyBaseMagicPhalanx` принадлежат
//! этому owner-у. Общий `CState::GetSufferer` сохраняет player/NPC/monster/
//! build/gate; NPC отклоняется как мёртвый, а постройки проходят собственный
//! region-owned defence. `CGame` только предоставляет владельцев, регион и
//! доставку.

use super::baseattack::{real_distance, time_reached};
use super::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairybasemagicphalanx::CBattleFairyBaseMagicPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{BattleFairySkillDispatch, CPlayer};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

const PLAYER_TYPE: i32 = 400;

fn add_legacy_c_string(
    message: &mut crate::nets::basemessage::CBaseMessage,
    value: &[u8],
) {
    let length = value.iter().position(|byte| *byte == 0).unwrap_or(value.len());
    message.add(&value[..length]);
    message.add_byte(0);
}

fn send_end(game: &mut CGame, player_id: i32, skill_level: i32) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(3);
    message.add_long(BATTLE_FAIRY_BASE_MAGIC_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(BATTLE_FAIRY_VISUAL_OBJECT_TYPE);
    message.add_long(player_id);
    message.add_long(player.shape().get_direction());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) const BATTLE_FAIRY_BASE_MAGIC_SKILL_ID: u32 = 0x224;
pub(crate) const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;
pub(crate) const DENIED_STATE_A: u32 = 0x192;
pub(crate) const DENIED_STATE_B: u32 = 0x67;
pub(crate) const DENIED_STATE_C: u32 = 0xd2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyBaseMagicExecutionState {
    kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    target: ShapeIdentity,
}

impl BattleFairyBaseMagicExecutionState {
    pub(crate) const fn begin(
        dispatch: BattleFairySkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<BattleFairySkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(
        &mut self,
    ) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }
}

pub(crate) fn execute_battle_fairy_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
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
    let skill_level = match dispatch {
        BattleFairySkillDispatch::SelfTarget { skill_level, .. }
        | BattleFairySkillDispatch::Point { skill_level, .. }
        | BattleFairySkillDispatch::Object { skill_level, .. } => skill_level,
    };
    let Some(properties) = game.skill_base_properties(
        BATTLE_FAIRY_BASE_MAGIC_SKILL_ID,
        skill_level,
    ) else {
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
    let started_at_ms = runtime.now_milliseconds();
    let target = match dispatch {
        BattleFairySkillDispatch::Object { target, .. } => target,
        _ => {
            game.send_battle_fairy_skill_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            send_end(game, player_id, skill_level);
            return rejected();
        }
    };

    if player_ai.battle_fairy_base_magic().is_none() {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            game.send_battle_fairy_skill_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0045");
            send_end(game, player_id, skill_level);
            return rejected();
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_A)
            || game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_B)
        {
            let mut message = CMessage::new(0x000b_f807);
            message.add_ulong(0xffff_ffff);
            add_legacy_c_string(message.base_mut(), game.get_string_by_id(b"ZHGS0046"));
            let _ = message.send_to_player(game.net_server(), player_id);
            send_end(game, player_id, skill_level);
            return rejected();
        }
        if game.target_has_state_by_skill_id(region_id, target, DENIED_STATE_C) {
            let mut message = CMessage::new(0x000b_f807);
            message.add_ulong(0xffff_ffff);
            add_legacy_c_string(message.base_mut(), game.get_string_by_id(b"ZHGS0047"));
            let _ = message.send_to_player(game.net_server(), player_id);
            send_end(game, player_id, skill_level);
            return rejected();
        }
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.battle_fairy_base_magic_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.battle_fairy_base_magic_last_used_ms(),
                reuse_delay_ms,
            )
        {
            game.send_battle_fairy_skill_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"ZHGS0048");
            send_end(game, player_id, skill_level);
            return rejected();
        }
        let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
            game.send_battle_fairy_skill_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            send_end(game, player_id, skill_level);
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
            game.send_battle_fairy_skill_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            send_end(game, player_id, skill_level);
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
            game.send_battle_fairy_skill_failure(player_id, 0x0b);
            game.send_skill_system_info(player_id, b"ZHGS0049");
            send_end(game, player_id, skill_level);
            return rejected();
        }
        let target_dead = game.base_magic_target_dead(region_id, target);
        if target_dead {
            game.send_battle_fairy_skill_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"ZHGS0050");
            send_end(game, player_id, skill_level);
            return rejected();
        }
        let direction = player.shape().get_direction();
        let mut start = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        start.add_byte(1);
        start.add_long(BATTLE_FAIRY_BASE_MAGIC_SKILL_ID as i32);
        start.base_mut().add_short(skill_level as i16);
        start.add_long(BATTLE_FAIRY_VISUAL_OBJECT_TYPE);
        start.add_long(player_id);
        start.add_long(direction);
        let _ = game.send_player_shape_around(player_id, None, &start);
        let mut execution = BattleFairyBaseMagicExecutionState::begin(
            dispatch,
            target,
            started_at_ms,
        );
        let _ = execution
            .kernel_mut()
            .advance(SkillStage::Begin, SkillStage::Check);
        player_ai.begin_battle_fairy_base_magic(execution);
        let first_ai_now_ms = runtime.now_milliseconds();
        if !time_reached(first_ai_now_ms, started_at_ms, delay_ms) {
            return pending();
        }
    } else if player_ai
        .battle_fairy_base_magic()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return rejected();
    } else if !time_reached(
        started_at_ms,
        player_ai
            .battle_fairy_base_magic()
            .map_or(started_at_ms, |state| state.kernel().started_at_ms()),
        delay_ms,
    ) {
        return pending();
    }

    let Some(_target_view) = game.base_magic_target_view(region_id, target) else {
        game.send_battle_fairy_skill_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"ZHGS0050");
        send_end(game, player_id, skill_level);
        return rejected();
    };
    let target_dead = game.base_magic_target_dead(region_id, target);
    if target_dead {
        game.send_battle_fairy_skill_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"ZHGS0050");
        send_end(game, player_id, skill_level);
        return rejected();
    }
    let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
        return rejected();
    };
    let Some((target_x, target_y)) = game.base_magic_target_point(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target,
    ) else {
        send_end(game, player_id, skill_level);
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
    fire.add_long(BATTLE_FAIRY_BASE_MAGIC_SKILL_ID as i32);
    fire.base_mut().add_short(skill_level as i16);
    fire.add_long(BATTLE_FAIRY_VISUAL_OBJECT_TYPE);
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
    let has_war_soul = game
        .find_player(player_id)
        .and_then(|player| player.war_soul_goods(game.goods_factory()))
        .is_some();
    let mut summoned = false;
    if has_war_soul && path.iter().all(|cell| cell.2 != 2) {
        let player = game
            .find_player(player_id)
            .expect("владелец боевой феи сохранён");
        let permissions = player.pk_permissions();
        let master = MasterInfo {
            master_type: PLAYER_TYPE,
            master_id: player_id,
            master_guild_id: player.faction_id(),
            master_team_id: player.team_id(),
            master_union_id: player.union_id(),
            master_country_id: i32::from(player.country()),
            permitted_to_kill_player: i32::from(permissions.player),
            permitted_to_kill_teammate: i32::from(permissions.teammate),
            permitted_to_kill_guild_member: i32::from(permissions.guild_member),
            permitted_to_kill_criminal: i32::from(permissions.criminal),
        };
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CBattleFairyBaseMagicPhalanx::new(
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
        let (tile_x, tile_y) = path
            .first()
            .map(|cell| (cell.0, cell.1))
            .unwrap_or((source_view.tile_x, source_view.tile_y));
        let result = game.add_battle_fairy_base_magic_phalanx(
            region_id,
            phalanx,
            tile_x,
            tile_y,
            summon_started_at_ms,
            runtime,
        );
        summoned = result.is_some_and(|result| result.is_ok());
        tracing::trace!(region_id, player_id, summon_id, ?result, "создан снаряд базовой атаки боевой феи");
    }

    if let Some(state) = player_ai.battle_fairy_base_magic_mut() {
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
    send_end(game, player_id, skill_level);
    QueuedSkillExecutionOutcome {
        state: if summoned {
            QueuedSkillExecutionState::Completed
        } else {
            QueuedSkillExecutionState::Rejected
        },
        first_contact: false,
        killing_blow: None,
    }
}
