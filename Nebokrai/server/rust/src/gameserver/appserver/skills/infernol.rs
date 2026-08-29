//! Огненный круг `CInfernol` (`0x135`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/infernol.cpp`. PDB-глобали `0x006A36F4..0x006A372C`
//! подтверждают маску 7×7 и обход клеток X→Y вокруг самого исполнителя.
//! Владелец сохраняет двойную проверку MP, строгие границы восстановления и
//! задержки, точные пакеты визуального эффекта, фильтрацию и устранение
//! повторных целей. Для каждой допустимой цели формула выполняет ровно два
//! вызова legacy RNG: разброс элементального урона и критический удар.
//! `CGame` только разрешает независимых владельцев и применяет рассчитанные
//! атаки через общую защиту. Успех, отказ после `Begin` и клиентская отмена
//! проходят через подтверждённый `CSummonSkill::End(1)` с возвратом движения,
//! обновлением свойств, очисткой и фиксацией времени восстановления.

use super::baseattack::{SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const INFERNOL_SKILL_ID: u32 = 0x135;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const SIDE: i32 = 7;
const SCOPE: [bool; 49] = [
    false, false, true, true, true, false, false,
    false, true, true, true, true, true, false,
    true, true, true, true, true, true, true,
    true, true, true, true, true, true, true,
    true, true, true, true, true, true, true,
    false, true, true, true, true, true, false,
    false, false, true, true, true, false, false,
];

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn send_failure(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(INFERNOL_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else {
        let (Ok(x), Ok(y)) = (player.shape().get_tile_x(), player.shape().get_tile_y()) else {
            return;
        };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_player_infernol<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_infernol_used(now_ms);
    });
}

pub(crate) fn cancel_player_infernol<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.infernol().map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_infernol(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
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

fn targets(game: &CGame, region_id: i32, player_id: i32) -> Vec<ShapeIdentity> {
    let Some(player) = game.find_player(player_id) else { return Vec::new() };
    let (Ok(center_x), Ok(center_y)) =
        (player.shape().get_tile_x(), player.shape().get_tile_y())
    else {
        return Vec::new();
    };
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else {
        return Vec::new();
    };
    let (area_width, area_height) = game.area_dimensions();
    let start_x = center_x.wrapping_sub(SIDE >> 1);
    let start_y = center_y.wrapping_sub(SIDE >> 1);
    let mut result = Vec::new();
    for x in 0..SIDE {
        for y in 0..SIDE {
            let index = y.wrapping_mul(SIDE).wrapping_add(x) as usize;
            if !SCOPE[index] {
                continue;
            }
            let mut shapes = Vec::new();
            if region
                .get_shapes(
                    start_x.wrapping_add(x),
                    start_y.wrapping_add(y),
                    area_width,
                    area_height,
                    game,
                    &mut shapes,
                )
                .is_err()
            {
                continue;
            }
            for shape in shapes {
                if matches!(shape.identity.object_type, PLAYER_TYPE | MONSTER_TYPE) {
                    result.push(shape.identity);
                }
            }
        }
    }
    result
}

fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game
            .find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .and_then(|monster| monster.base_property_key())
            .and_then(|key| game.find_monster_property_by_origin_name(key))
            .map(|property| property.level as u8),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments, reason = "параметры соответствуют свойствам навыка EXE")]
fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    target: ShapeIdentity,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let target_level = target_level(game, region_id, target)?;
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let weapon_level = player.equipment().get_goods(2).map_or(0, |goods| {
        goods.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1)
    });
    let (weapon_divisor, weapon_minimum) = game.globe_setup().weapon_damage_factors();
    let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
    let damage_factor = if weapon_divisor == 0.0 {
        1.0
    } else {
        (delta as f32 / weapon_divisor).min(1.0).max(weapon_minimum)
    };
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    let random_damage = game.skill_random_below(width);
    let element_bonus =
        (element_modifier as f32 * 0.01 * combat.element_modify as f32).round_ties_even() as i32;
    let damage = (combat.add_element_attack as i32)
        .wrapping_add(random_damage)
        .wrapping_add(minimum)
        .wrapping_add(element_bonus)
        .max(0);
    let mut attack = AttackInformation {
        skill_id: INFERNOL_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: damage,
            mp_damage: 0,
        }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let critical_rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = (power.hp_damage as f32 * critical_rate).round_ties_even() as i32;
        }
    }
    Some((master, attack))
}

pub(crate) const fn is_infernol_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: INFERNOL_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: INFERNOL_SKILL_ID, .. }
    )
}

pub(crate) fn execute_player_infernol<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_infernol_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, initial_mana)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.learned_skill_level(INFERNOL_SKILL_ID),
            player.mana(),
        ))
    }) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(INFERNOL_SKILL_ID, level) else {
        if player_ai.infernol().is_some() {
            finish_player_infernol(game, player_id, player_ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.infernol().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if player_ai.infernol_last_used_ms() != 0
            && !time_reached(
                runtime.now_milliseconds(),
                player_ai.infernol_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
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
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(INFERNOL_SKILL_ID));
        }
        player_ai.begin_infernol(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai.infernol().is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.infernol().is_some_and(|state| state.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish_player_infernol(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, 1);
        if let Some(state) = player_ai.infernol_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .infernol()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение огненного круга создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_visual(game, player_id, level, 2);
    let mut attacked = Vec::new();
    for target in targets(game, region_id, player_id) {
        let Some(master) = game.find_player(player_id).map(master_info) else { break };
        let attackable = if target.object_type == PLAYER_TYPE {
            game.player_base_attackable(player_id, target.id)
        } else {
            game.owned_player_skill_target_attackable(master, target, region_id)
        };
        if !attackable || attacked.contains(&target) {
            continue;
        }
        let Some((master, attack)) = calculate_attack(
            game,
            player_id,
            region_id,
            target,
            level,
            minimum,
            maximum,
            element_modifier,
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
        attacked.push(target);
    }
    if let Some(state) = player_ai.infernol_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_infernol(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
