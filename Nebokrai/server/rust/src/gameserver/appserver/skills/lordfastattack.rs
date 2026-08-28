//! Быстрая атака владыки `CLordFastAttack` (`0x1f5`) для объектного пути игрока и монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lordfastattack.cpp`. Объектный путь сохраняет проверку
//! перезарядки, расстояния и непроходимых клеток, затем начало, задержку и два
//! последовательных удара на границах `15001/15002`. Каждый удар игрока
//! выполняет исходные вызовы RNG для физического разброса и критического удара;
//! путь монстра использует тот же узкий двухударный механизм с отдельной формулой.
//! `CGame` только разрешает владельцев, применяет рассчитанную атаку и доставляет
//! пакеты. Координатные перегрузки `Begin` остаются ниже недостигнутыми.

// ============================================================================
// FUNCTION: CLordFastAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:125
// RVA: 0x00130410
// ADDRESS: 00530410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordFastAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordfastattack.cpp:144
// RVA: 0x001304E0
// ADDRESS: 005304e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER, real_distance, time_reached,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::monsterfastattack::{SKILL_USAGE_FIRST_TIME, SKILL_USAGE_SECOND_TIME};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const LORD_FAST_ATTACK_SKILL_ID: u32 = 0x1f5;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LordFastAttackExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    fire_started: bool,
    first_attack_done: bool,
}

impl LordFastAttackExecutionState {
    const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            condition_checked: false,
            fire_started: false,
            first_attack_done: false,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

fn send_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
}

fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(direction) = game
        .find_player(player_id)
        .map(|player| player.shape().get_direction())
    else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(1);
    message.add_long(LORD_FAST_ATTACK_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_fire(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(LORD_FAST_ATTACK_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
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

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    hit_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let (combat, master) = game
        .find_player(player_id)
        .map(|player| (player.combat_properties(), master_info(player)))?;
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let span = maximum
        .wrapping_sub(minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let physical = minimum
        .wrapping_add(game.skill_random_below(span))
        .max(0);
    let mut attack = AttackInformation {
        skill_id: LORD_FAST_ATTACK_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: (combat.add_element_attack as i32).max(0),
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(combat.add_soul_attack),
                mp_damage: 0,
            },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage =
                (power.hp_damage as f32 * critical_rate).round_ties_even() as i32;
        }
    }
    Some((master, attack))
}

fn apply_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    hit_modifier: i32,
    runtime: &mut Runtime,
) {
    let Some((master, attack)) = calculate_attack(game, player_id, level, hit_modifier) else {
        return;
    };
    match target.object_type {
        PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
            master,
            target.id,
            region_id,
            attack,
            runtime,
        ),
        MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
            master,
            target.id,
            region_id,
            attack,
            runtime,
        ),
        _ => return,
    }
    game.damage_player_weapon(player_id, runtime);
}

pub(crate) const fn is_lord_fast_attack_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Object {
            skill_id: LORD_FAST_ATTACK_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | MONSTER_TYPE,
                ..
            },
        }
    )
}

pub(crate) fn execute_player_lord_fast_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let target = match dispatch {
        PlayerSkillDispatch::Object {
            skill_id: LORD_FAST_ATTACK_SKILL_ID,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => target,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, level, source_view)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(LORD_FAST_ATTACK_SKILL_ID),
            player.shape_view()?,
        ))
    }) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(LORD_FAST_ATTACK_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let first_time_ms = properties.query_property(SKILL_USAGE_FIRST_TIME);
    let second_time_ms = properties.query_property(SKILL_USAGE_SECOND_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.lord_fast_attack().is_none() {
        let now_ms = runtime.now_milliseconds();
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if player_ai.lord_fast_attack_last_used_ms() != 0
            && !time_reached(
                now_ms,
                player_ai.lord_fast_attack_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if maximum_distance != 0
            && real_distance(
                source_view.tile_x,
                source_view.tile_y,
                target_view.tile_x,
                target_view.tile_y,
            ) > maximum_distance as i32
        {
            send_failure(game, player_id, 0x0b);
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(
            region_id,
            source_view.tile_x,
            source_view.tile_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_failure(game, player_id, 0x0f);
            send_failure(game, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(LORD_FAST_ATTACK_SKILL_ID));
        }
        player_ai.begin_lord_fast_attack(LordFastAttackExecutionState::begin(dispatch, now_ms));
    } else if player_ai
        .lord_fast_attack()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        send_failure(game, player_id, 10);
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target_dead(game, region_id, target)
        || (target.object_type == PLAYER_TYPE && target.id == player_id)
    {
        send_failure(game, player_id, 10);
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai
        .lord_fast_attack()
        .is_some_and(|state| !state.condition_checked)
    {
        let Some(source_view) = game
            .find_player(player_id)
            .and_then(|player| player.shape_view())
        else {
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_view.tile_x,
                source_view.tile_y,
                target_view.tile_x,
                target_view.tile_y,
            ));
        }
        send_start(game, player_id, level);
        if let Some(state) = player_ai.lord_fast_attack_mut() {
            state.condition_checked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .lord_fast_attack()
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение быстрой атаки владыки хранит время начала");
    if player_ai
        .lord_fast_attack()
        .is_some_and(|state| !state.fire_started)
    {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        send_fire(
            game,
            player_id,
            level,
            target_view.tile_x,
            target_view.tile_y,
        );
        if let Some(state) = player_ai.lord_fast_attack_mut() {
            state.fire_started = true;
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    if player_ai
        .lord_fast_attack()
        .is_some_and(|state| !state.first_attack_done)
    {
        if !time_reached(
            runtime.now_milliseconds(),
            started_at_ms,
            delay_ms.wrapping_add(first_time_ms),
        ) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        apply_attack(
            game,
            player_id,
            region_id,
            target,
            level,
            hit_modifier,
            runtime,
        );
        if let Some(state) = player_ai.lord_fast_attack_mut() {
            state.first_attack_done = true;
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }

    if !time_reached(
        runtime.now_milliseconds(),
        started_at_ms,
        delay_ms
            .wrapping_add(first_time_ms)
            .wrapping_add(second_time_ms),
    ) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    apply_attack(
        game,
        player_id,
        region_id,
        target,
        level,
        hit_modifier,
        runtime,
    );
    if let Some(state) = player_ai.lord_fast_attack_mut() {
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_lord_fast_attack_used(runtime.now_milliseconds());
    finish(game, player_id);
    terminal(QueuedSkillExecutionState::Completed)
}
