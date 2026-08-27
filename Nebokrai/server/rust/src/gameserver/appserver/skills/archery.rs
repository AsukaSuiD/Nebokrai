//! Базовая стрельба GameServer (`SKILL_BASE_ARCHERY`, ID `2`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/archery.cpp`. Навык исполняется из обычной очереди
//! `CPlayerAI`: проверяет дальность, непролётные клетки и оружие категории
//! лука либо арбалета, блокирует движение на задержку и передаёт попадание
//! региональному `CArcheryPhalanx`. Формулы и порядок RNG применяются только
//! при достижении цели снарядом.

use super::archeryphalanx::CArcheryPhalanx;
use super::baseattack::{real_distance, time_reached};
use super::kernel::{SkillExecutionKernel, SkillStage};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::basemagic::{
    BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 100;

pub(crate) const ARCHERY_SKILL_ID: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcheryExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
}

impl ArcheryExecutionState {
    pub(crate) const fn begin(
        dispatch: PlayerSkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }
}

pub(crate) fn execute_player_archery<Runtime: GameMainLoopRuntime>(
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
    let skill_level = player.learned_skill_level(ARCHERY_SKILL_ID);
    let Some(properties) = game.skill_base_properties(ARCHERY_SKILL_ID, skill_level)
    else {
        return rejected();
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let summoned_speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let summoned_lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();
    let target = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => target,
        _ => {
            game.send_base_magic_failure(player_id, 10);
            return rejected();
        }
    };

    if player_ai.archery().is_none() {
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.archery_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.archery_last_used_ms(),
                reuse_delay_ms,
            )
        {
            game.send_base_magic_failure(player_id, 0x0d);
            return rejected();
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
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
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize + 1 {
            game.send_base_magic_failure(player_id, 0x0b);
            let target_name = match target.object_type {
                PLAYER_TYPE => game
                    .find_player(target.id)
                    .map(CPlayer::player_name)
                    .unwrap_or_default(),
                MONSTER_TYPE => game
                    .find_region(region_id)
                    .and_then(|owner| owner.base().find_monster_by_id(target.id))
                    .map(CMonster::display_name)
                    .unwrap_or_default(),
                _ => &[],
            };
            game.send_skill_system_info_with_text(player_id, b"GS0280", target_name);
            return rejected();
        }
        if path.iter().any(|cell| cell.2 == 2) {
            game.send_base_magic_failure(player_id, 0x0f);
            game.send_skill_system_info(player_id, b"GS0282");
            return rejected();
        }
        let weapon_category = player
            .equipment()
            .get_goods(2)
            .map(|weapon| {
                weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1)
            });
        match weapon_category {
            None => {
                game.send_base_magic_failure(player_id, 0x0e);
                game.send_skill_system_info(player_id, b"GS0283");
                return rejected();
            }
            Some(3 | 4) => {}
            Some(_) => {
                game.send_base_magic_failure(player_id, 0x0e);
                game.send_skill_system_info(player_id, b"GS0284");
                return rejected();
            }
        }
        let target_dead = match target.object_type {
            PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
            MONSTER_TYPE => game
                .find_region(region_id)
                .and_then(|owner| owner.base().find_monster_by_id(target.id))
                .is_none_or(|monster| monster.hit_points() == 0),
            _ => true,
        };
        if target_dead {
            game.send_base_magic_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0285");
            return rejected();
        }
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            game.send_base_magic_failure(player_id, 10);
            game.send_base_magic_failure(player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return rejected();
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                target_view.tile_x,
                target_view.tile_y,
            ));
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(ARCHERY_SKILL_ID));
        }
        let direction = game
            .find_player(player_id)
            .map(|player| player.shape().get_direction())
            .unwrap_or_default();
        let mut start = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        start.add_byte(1);
        start.add_long(ARCHERY_SKILL_ID as i32);
        start.base_mut().add_short(skill_level as i16);
        start.add_long(PLAYER_TYPE);
        start.add_long(player_id);
        start.add_long(direction);
        let _ = game.send_player_shape_around(player_id, None, &start);
        let mut execution = ArcheryExecutionState::begin(dispatch, target, now_ms);
        let _ = execution
            .kernel_mut()
            .advance(SkillStage::Begin, SkillStage::Check);
        player_ai.begin_archery(execution);
        let first_ai_now_ms = runtime.now_milliseconds();
        if !time_reached(first_ai_now_ms, now_ms, delay_ms) {
            return pending();
        }
    } else if player_ai
        .archery()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return rejected();
    } else if !time_reached(
        now_ms,
        player_ai
            .archery()
            .map_or(now_ms, |state| state.kernel().started_at_ms()),
        delay_ms,
    ) {
        return pending();
    }

    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        game.send_base_magic_failure(player_id, 10);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(None);
        }
        return rejected();
    };
    let target_dead = match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => true,
    };
    if target_dead {
        game.send_base_magic_failure(player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        game.send_base_magic_failure(player_id, 10);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(None);
        }
        return rejected();
    }
    let Some(source_view) = game.find_player(player_id).and_then(CPlayer::shape_view) else {
        return rejected();
    };
    let attack_time = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
    )
    .wrapping_mul(summoned_speed as i32);
    let mut fire = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    fire.add_byte(2);
    fire.add_long(ARCHERY_SKILL_ID as i32);
    fire.base_mut().add_short(skill_level as i16);
    fire.add_long(PLAYER_TYPE);
    fire.add_long(player_id);
    fire.add_long(target.object_type);
    fire.add_long(target.id);
    fire.add_long(target_view.tile_x);
    fire.add_long(target_view.tile_y);
    fire.add_long(attack_time);
    let _ = game.send_player_shape_around(player_id, None, &fire);

    let forced_distance = real_distance(
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
    ) as u32;
    let path = game.base_magic_path(
        region_id,
        source_view.tile_x,
        source_view.tile_y,
        target_view.tile_x,
        target_view.tile_y,
        Some(forced_distance),
    );
    if !path.is_empty() && path.iter().all(|cell| cell.2 != 2) {
        let player = game.find_player(player_id).expect("стрелок сохранён");
        let permissions = player.pk_permissions();
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
        let summon_id = game.allocate_summon_shape_id();
        let summon_started_at_ms = runtime.now_milliseconds();
        let mut phalanx = CArcheryPhalanx::new(
            summon_id,
            master,
            summon_started_at_ms,
            summoned_lifetime,
            skill_level,
            attack_time as u32,
            target,
        );
        phalanx.shape_mut().set_region_id(region_id);
        let (tile_x, tile_y, _) = path[0];
        let result = game.add_archery_phalanx(
            region_id,
            phalanx,
            tile_x,
            tile_y,
            summon_started_at_ms,
            runtime,
        );
        tracing::trace!(region_id, player_id, summon_id, ?result, "создан снаряд базовой стрельбы");
    }
    if let Some(state) = player_ai.archery_mut() {
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
    player_ai.mark_archery_used(runtime.now_milliseconds());
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
    QueuedSkillExecutionOutcome {
        state: QueuedSkillExecutionState::Completed,
        first_contact: false,
        killing_blow: None,
    }
    }

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранена только недостигнутая внутренняя функция `FUN_005b27a3`; skill-путь материализован полностью.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.h

// ============================================================================
// FUNCTION: FUN_005b27a3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:41
// RVA: 0x001B27A3
// ADDRESS: 005b27a3
// PROTOTYPE: undefined FUN_005b27a3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
