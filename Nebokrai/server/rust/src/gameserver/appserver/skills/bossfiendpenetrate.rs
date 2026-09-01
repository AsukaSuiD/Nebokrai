//! Проникающая атака демона-босса `CBossFiendPenetrate` (`0x1FA`) для игрока и монстра.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/bossfiendpenetrate.cpp`. Навык проверяет задержку повторного применения,
//! дальность, оружие категории `3` и MP, затем необратимо списывает MP и
//! повторно проверяет оружие. После общей задержки он строит прямой путь,
//! прекращает поражение перед первой клеткой `BLOCK_UNFLY` и обрабатывает не
//! более одной клетки за проход ИИ. Каждая фигура поражается не более одного
//! раза; формула игрока сохраняет два RNG-вызова и поправку уровня оружия,
//! а критический множитель усекает каждый компонент к нулю. Формула монстра —
//! физический, стихийный и холистический RNG-порядок.
//! `SkillExecutionKernel` хранит стадии игрока, а `CGame` только разрешает
//! владельцев, применяет рассчитанную атаку и доставляет пакеты.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
    time_reached,
};
use super::kernel::{SkillExecutionKernel, SkillTermination};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::poisonmoth::{cell_targets, master_info, target_level, target_position};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::approach_attack_range;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_WEAPON_CATEGORY, GAP_WEAPON_DAMAGE_LEVEL,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const BLOCK_UNFLY: u8 = 2;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;

pub(crate) const BOSS_FIEND_PENETRATE_SKILL_ID: u32 = 0x1fa;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerBossFiendPenetrateExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    destination: (i32, i32),
    condition_checked: bool,
    attacking_started: bool,
    path: Vec<(i32, i32, u8)>,
    attack_cell_count: usize,
    current_cell: usize,
    attacked: Vec<ShapeIdentity>,
}
impl PlayerBossFiendPenetrateExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, now_ms),
            destination,
            condition_checked: false,
            attacking_started: false,
            path: Vec::new(),
            attack_cell_count: 0,
            current_cell: 0,
            attacked: Vec::new(),
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) const fn is_player_boss_fiend_penetrate_dispatch(
    dispatch: PlayerSkillDispatch,
) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point {
            skill_id: BOSS_FIEND_PENETRATE_SKILL_ID,
            ..
        } | PlayerSkillDispatch::Object {
            skill_id: BOSS_FIEND_PENETRATE_SKILL_ID,
            target: ShapeIdentity {
                object_type: PLAYER_TYPE | MONSTER_TYPE,
                ..
            },
        }
    )
}

fn player_weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 3
    })
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(0x000b_fe01, player_id, action);
    match action {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0297"),
        _ => {}
    }
}

fn send_player_start(game: &mut CGame, player_id: i32, skill_level: i32) {
    let Some(direction) = game
        .find_player(player_id)
        .map(|player| player.shape().get_direction())
    else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_fire(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    dispatch: PlayerSkillDispatch,
    destination: (i32, i32),
    missile_flying_time_ms: u32,
) {
    let target = match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        _ => None,
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(destination.0);
    message.add_long(destination.1);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_empty_path(game: &mut CGame, player_id: i32, skill_level: i32) {
    let Some(direction) = game
        .find_player(player_id)
        .map(|player| player.shape().get_direction())
    else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_boss_fiend_penetrate<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    successful: bool,
) {
    restore_player_movement(game, player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
    if successful {
        player_ai.mark_boss_fiend_penetrate_used(runtime.now_milliseconds());
    }
}

pub(crate) fn complete_player_boss_fiend_penetrate<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .boss_fiend_penetrate()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, true);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_boss_fiend_penetrate<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai
        .boss_fiend_penetrate()
        .map(|state| state.kernel().dispatch())
    else {
        return false;
    };
    finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn player_target_is_dead(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> bool {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } if target.object_type == PLAYER_TYPE => {
            game.find_player(target.id).is_none_or(CPlayer::is_dead)
        }
        PlayerSkillDispatch::Object { target, .. } if target.object_type == MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .is_none_or(|monster: &CMonster| monster.hit_points() == 0),
        PlayerSkillDispatch::Object { .. } => true,
        _ => false,
    }
}

fn calculate_player_attack(
    game: &mut CGame,
    player_id: i32,
    target_level: u8,
    skill_level: i32,
    damage_factor_percent: u32,
    hit_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let weapon_level = player.equipment().get_goods(2).map_or(0, |weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1)
    });
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let level_delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let weapon_factor = (if divisor == 0.0 {
        1.0
    } else {
        level_delta as f32 / divisor
    })
    .min(1.0)
    .max(floor);
    let minimum = combat.minimum_attack as i32;
    let maximum = combat.maximum_attack as i32;
    let width = maximum.wrapping_sub(minimum).wrapping_add(1);
    let physical = minimum
        .wrapping_add(game.skill_random_below(width))
        .max(0);
    let mut attack = AttackInformation {
        skill_id: BOSS_FIEND_PENETRATE_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: damage_factor_percent as f32 * weapon_factor * 0.01,
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
            power.hp_damage = (power.hp_damage as f32 * critical_rate) as i32;
        }
    }
    Some((master, attack))
}

#[allow(clippy::too_many_arguments, reason = "аргументы сохраняют исходную клеточную атаку игрока")]
fn attack_player_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    skill_level: i32,
    damage_factor_percent: u32,
    hit_modifier: i32,
    cell_x: i32,
    cell_y: i32,
    attacked: &mut Vec<ShapeIdentity>,
    runtime: &mut Runtime,
) {
    if cell_x == 0 && cell_y == 0 {
        return;
    }
    let Some(master) = game.find_player(player_id).map(master_info) else {
        return;
    };
    for target in cell_targets(game, region_id, cell_x, cell_y) {
        if (target.object_type == PLAYER_TYPE && target.id == player_id)
            || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE)
            || attacked.contains(&target)
            || !game.owned_player_skill_target_attackable(master, target, region_id)
        {
            continue;
        }
        attacked.push(target);
        let Some(level) = target_level(game, region_id, target) else {
            continue;
        };
        let Some((master, attack)) = calculate_player_attack(
            game,
            player_id,
            level,
            skill_level,
            damage_factor_percent,
            hit_modifier,
        ) else {
            continue;
        };
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(
                master, target.id, region_id, attack, runtime,
            ),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(
                master, target.id, region_id, attack, runtime,
            ),
            _ => {}
        }
    }
}

pub(crate) fn execute_player_boss_fiend_penetrate<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_boss_fiend_penetrate_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, skill_level, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(BOSS_FIEND_PENETRATE_SKILL_ID),
                player.mana(),
            ))
        })
    else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game
        .skill_base_properties(BOSS_FIEND_PENETRATE_SKILL_ID, skill_level)
        .cloned()
    else {
        if player_ai.boss_fiend_penetrate().is_some() {
            finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let missile_flying_time = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let damage_factor = properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.boss_fiend_penetrate().is_none() {
        let Some(destination) = target_position(game, region_id, player_id, dispatch) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let now_ms = runtime.now_milliseconds();
        if player_ai.boss_fiend_penetrate_last_used_ms() != 0
            && !time_reached(
                now_ms,
                player_ai.boss_fiend_penetrate_last_used_ms(),
                reuse_delay,
            )
        {
            send_player_failure(game, player_id, 0x0d, mp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            destination.0,
            destination.1,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_player_failure(game, player_id, 0x0b, mp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else {
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        if !player_weapon_is_valid(game, player) {
            send_player_failure(game, player_id, 0x0e, mp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7, mp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(BOSS_FIEND_PENETRATE_SKILL_ID));
        }
        player_ai.begin_boss_fiend_penetrate(PlayerBossFiendPenetrateExecutionState::begin(
            dispatch,
            destination,
            now_ms,
        ));
    } else if player_ai
        .boss_fiend_penetrate()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    let destination = target_position(game, region_id, player_id, dispatch).unwrap_or_else(|| {
        player_ai
            .boss_fiend_penetrate()
            .map(|state| state.destination)
            .unwrap_or((source_x, source_y))
    });
    if player_target_is_dead(game, region_id, dispatch) {
        send_player_failure(game, player_id, 10, mp_loss);
        finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .boss_fiend_penetrate()
        .is_some_and(|state| !state.condition_checked)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 7, mp_loss);
            finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        if game
            .find_player(player_id)
            .is_none_or(|player| !player_weapon_is_valid(game, player))
        {
            send_player_failure(game, player_id, 0x0e, mp_loss);
            finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(direction);
        }
        send_player_start(game, player_id, skill_level);
        if let Some(state) = player_ai.boss_fiend_penetrate_mut() {
            state.condition_checked = true;
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .boss_fiend_penetrate()
        .map(|state| state.kernel().started_at_ms())
        .unwrap_or_default();
    if player_ai
        .boss_fiend_penetrate()
        .is_some_and(|state| !state.attacking_started)
    {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay) {
            return player_terminal(QueuedSkillExecutionState::Pending);
        }
        restore_player_movement(game, player_id);
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            destination.0,
            destination.1,
            (maximum_distance != 0).then_some(maximum_distance),
        );
        if path.is_empty() {
            send_player_empty_path(game, player_id, skill_level);
            finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize
        {
            send_player_failure(game, player_id, 0x0b, mp_loss);
            finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, false);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let attack_cell_count = path
            .iter()
            .position(|cell| cell.2 == BLOCK_UNFLY)
            .unwrap_or(path.len());
        let endpoint = path
            .get(attack_cell_count)
            .or_else(|| path.last())
            .copied()
            .unwrap_or((destination.0, destination.1, BLOCK_UNFLY));
        let visual_destination = if matches!(dispatch, PlayerSkillDispatch::Object { .. }) {
            destination
        } else {
            (endpoint.0, endpoint.1)
        };
        send_player_fire(
            game,
            player_id,
            skill_level,
            dispatch,
            visual_destination,
            missile_flying_time,
        );
        if let Some(state) = player_ai.boss_fiend_penetrate_mut() {
            state.path = path;
            state.attack_cell_count = attack_cell_count;
            state.current_cell = 0;
            state.attacking_started = true;
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }

    let Some((current_cell, attack_cell_count, cell)) = player_ai
        .boss_fiend_penetrate()
        .map(|state| {
            (
                state.current_cell,
                state.attack_cell_count,
                state.path.get(state.current_cell).copied(),
            )
        })
    else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    if current_cell >= attack_cell_count {
        if let Some(state) = player_ai.boss_fiend_penetrate_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
        }
        finish_player_boss_fiend_penetrate(game, player_id, player_ai, runtime, true);
        return player_terminal(QueuedSkillExecutionState::Completed);
    }
    let cell_due_ms = delay.wrapping_add(
        missile_flying_time.wrapping_mul(current_cell as u32),
    );
    if !time_reached(runtime.now_milliseconds(), started_at_ms, cell_due_ms) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some((cell_x, cell_y, _)) = cell {
        let mut attacked = player_ai
            .boss_fiend_penetrate_mut()
            .map(|state| std::mem::take(&mut state.attacked))
            .unwrap_or_default();
        attack_player_cell(
            game,
            player_id,
            region_id,
            skill_level,
            damage_factor,
            hit_modifier,
            cell_x,
            cell_y,
            &mut attacked,
            runtime,
        );
        if let Some(state) = player_ai.boss_fiend_penetrate_mut() {
            state.attacked = attacked;
            state.current_cell = state.current_cell.wrapping_add(1);
        }
    }
    player_terminal(QueuedSkillExecutionState::Pending)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BossFiendPenetrateProgress {
    destination_x: i32,
    destination_y: i32,
    path: Vec<(i32, i32, u8)>,
    current_cell: usize,
    attack_cell_count: usize,
    attacked: Vec<ShapeIdentity>,
    fired: bool,
}

impl BossFiendPenetrateProgress {
    pub(crate) const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            path: Vec::new(),
            current_cell: 0,
            attack_cell_count: 0,
            attacked: Vec::new(),
            fired: false,
        }
    }

    const fn destination(&self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    fn fire(&mut self, path: Vec<(i32, i32, u8)>) {
        self.attack_cell_count = path
            .iter()
            .position(|cell| cell.2 == BLOCK_UNFLY)
            .unwrap_or(path.len());
        self.path = path;
        self.current_cell = 0;
        self.fired = true;
    }
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
    target: Option<ShapeIdentity>,
    target_x: i32,
    target_y: i32,
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_empty_path(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет формулу и порядок последствий одного поражения")]
fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &crate::setup::monsterlist::MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    identity: ShapeIdentity,
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return;
    };
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            attacker_property,
            attacker_tamed,
            attacker_master,
            identity,
            &target,
        )
    {
        return;
    }

    let (minimum, maximum) = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            monster.state_attack_bounds(
                attacker_property.minimum_attack,
                attacker_property.maximum_attack,
            )
        })
        .unwrap_or((attacker_property.minimum_attack, attacker_property.maximum_attack));
    let minimum = minimum as i32;
    let maximum = maximum as i32;
    let span = 1_i32.wrapping_sub(minimum).wrapping_add(maximum);
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let element_minimum = attacker_property.minimum_element as i32;
    let element_maximum = attacker_property.maximum_element as i32;
    let element_span = 1_i32
        .wrapping_sub(element_minimum)
        .wrapping_add(element_maximum);
    let element = element_minimum
        .wrapping_add(game.skill_random_below(element_span))
        .max(0);
    // `CMonster::GetCriticalChance` возвращает ноль, но исходный вызов
    // `random(100)` всё равно продвигает общий генератор.
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: BOSS_FIEND_PENETRATE_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as f32 * 0.01,
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
                hp_damage: element,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: i32::from(CMonster::resource_soul_attack(attacker_property)),
                mp_damage: 0,
            },
        ],
    };
    let attack = defend_owned_monster_attack(
        game,
        identity,
        target.mana,
        target.war_soul_mana,
        target.player_properties,
        target.monster_properties,
        attack,
    );
    apply_owned_monster_attack_hit(
        game,
        region,
        runtime,
        now_ms,
        monster_id,
        attacker_master,
        identity,
        &target.shape,
        target.health,
        target.mana,
        target.master,
        target.monster_property,
        target.tamed,
        target.carriage,
        attack,
        deaths,
    );
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, путь и текущий такт навыка")]
pub(crate) fn execute_owned_boss_fiend_penetrate<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((source, property, master, tamed, cast, progress, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.boss_fiend_penetrate_progress().cloned(),
                monster.skill_last_used_ms(BOSS_FIEND_PENETRATE_SKILL_ID),
            ))
        })
    else {
        return false;
    };
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    let live_destination = target.as_ref().and_then(|target| {
        (!target.dead).then_some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let Some(destination) = live_destination
        .or_else(|| progress.as_ref().map(BossFiendPenetrateProgress::destination))
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    };

    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            destination.0,
            destination.1,
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        if last_used_ms != 0
            && !time_reached(
                now_ms,
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
            )
        {
            return true;
        }
        let path = region.straight_skill_path(
            source_x,
            source_y,
            destination.0,
            destination.1,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                BOSS_FIEND_PENETRATE_SKILL_ID,
                skill_level,
                now_ms,
            );
            monster.set_boss_fiend_penetrate_progress(BossFiendPenetrateProgress::new(
                destination.0,
                destination.1,
            ));
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение проникающего удара проверено выше");
    if cast.dispatch().skill_id != BOSS_FIEND_PENETRATE_SKILL_ID {
        return false;
    }
    let Some(mut progress) = progress else { return true };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let missile_flying_time_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    if !progress.fired {
        if target.as_ref().is_some_and(|target| target.dead) {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.move_shape_mut().set_moveable(true);
                monster.clear_ai_target();
            }
            return true;
        }
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
        }
        let forced_length = (maximum_distance != 0).then_some(maximum_distance);
        let path = region.straight_skill_path(
            source_x,
            source_y,
            destination.0,
            destination.1,
            forced_length,
        );
        if path.is_empty() {
            send_empty_path(game, region, &source, skill_level);
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                let _ = monster.finish_base_attack_cast(now_ms);
            }
            return true;
        }
        if maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize
        {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        progress.fire(path);
        send_fire(
            game,
            region,
            &source,
            skill_level,
            target.as_ref().map(|_| target_identity),
            destination.0,
            destination.1,
            missile_flying_time_ms,
        );
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_boss_fiend_penetrate_progress(progress.clone());
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }

    if progress.current_cell >= progress.attack_cell_count {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }
    let due_ms = delay_ms.wrapping_add(
        missile_flying_time_ms.wrapping_mul(progress.current_cell as u32),
    );
    if !time_reached(now_ms, cast.started_at_ms(), due_ms) {
        return true;
    }
    let Some(&(cell_x, cell_y, _)) = progress.path.get(progress.current_cell) else {
        return true;
    };
    for identity in monster_attack_cell_candidates(game, region, monster_id, cell_x, cell_y) {
        if progress.attacked.contains(&identity) {
            continue;
        }
        let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
            continue;
        };
        if target.dead
            || target.god
            || target.city_dead
            || !owned_monster_attackable(
                game,
                region.id,
                &property,
                tamed,
                master,
                identity,
                &target,
            )
        {
            continue;
        }
        progress.attacked.push(identity);
        attack_target(
            game,
            region,
            runtime,
            now_ms,
            monster_id,
            skill_level,
            properties,
            &property,
            master,
            tamed,
            identity,
            deaths,
        );
    }
    progress.current_cell = progress.current_cell.wrapping_add(1);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_boss_fiend_penetrate_progress(progress);
    }
    true
}
