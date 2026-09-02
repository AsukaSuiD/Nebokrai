//! Зеркало душ `CSoulMirror` (`0x13C`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulmirror.cpp`. Направленные маски `3×3`, `5×5` и `7×7`
//! подтверждены глобалями EXE `0x006A33D0..0x006A367C` и выражены линией
//! ширины `2 * level - 1` на одну клетку впереди владельца. Модуль сохраняет
//! двухфазный расход MP, задержки, порядок клеток, устранение повторных целей,
//! один вызов legacy RNG на каждую атаку и призыв только в пустой проходимой
//! клетке. `CGame` разрешает независимых владельцев, применяет рассчитанную
//! атаку и временно извлекает регион только на время создания существа.
//! `End(1)` фиксирует завершённую область и cooldown; `End(0)` только очищает
//! отказ или отмену после восстановления движения.
//! Стихийная прибавка сохраняет расширенное вычисление x87 из целых свойств и
//! сохранённой `f32`-константы, затем усекается к нулю.

use super::baseattack::{SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::fightdefense::truncate_original;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const SOUL_MIRROR_SKILL_ID: u32 = 0x13c;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const SUMMONED_LIFETIME: u32 = 30_001;
const SUMMONED_CREATURE_ID: u32 = 30_003;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

const fn has_mana(mana: u32, loss: u32) -> bool {
    mana.wrapping_sub(loss) as i32 >= 0
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let Some(shape) = player.shape_view() else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(SOUL_MIRROR_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(0);
        message.add_long(0);
        message.add_long(shape.tile_x);
        message.add_long(shape.tile_y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_soul_mirror<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_soul_mirror_used(now_ms));
}

fn abort_player_soul_mirror(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
    abort_skill(game, player_id);
}

pub(crate) fn complete_player_soul_mirror<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.soul_mirror().map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_soul_mirror(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_soul_mirror<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.soul_mirror().map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_soul_mirror(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn destination(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
                let view = game.base_magic_target_view(region_id, target)?;
                Some((view.tile_x, view.tile_y, Some(target)))
            }
        _ => None,
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

fn scope_cells(level: i32, direction: i32, center_x: i32, center_y: i32) -> Vec<(i32, i32)> {
    if !(1..=3).contains(&level) || !(0..8).contains(&direction) { return Vec::new(); }
    let forward = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)][direction as usize];
    let perpendicular = [(1, 0), (1, 1), (0, 1), (-1, 1), (1, 0), (1, 1), (0, 1), (-1, 1)][direction as usize];
    let radius = level - 1;
    (-radius..=radius)
        .map(|offset| (
            center_x.wrapping_add(forward.0).wrapping_add(perpendicular.0 * offset),
            center_y.wrapping_add(forward.1).wrapping_add(perpendicular.1 * offset),
        ))
        .collect()
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

#[allow(clippy::too_many_arguments, reason = "аргументы соответствуют полям исходной формулы")]
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
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let damage_factor = player.weapon_modifier(
        game.goods_factory(),
        i32::from(target_level),
        divisor,
        floor,
    );
    let span_delta = maximum.wrapping_sub(minimum);
    let span = (if span_delta < 0 { span_delta.wrapping_neg() } else { span_delta }).wrapping_add(1);
    let random_damage = game.skill_random_below(span);
    let element_bonus = truncate_original(
        f64::from(element_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let damage = (combat.add_element_attack as i32)
        .wrapping_add(random_damage)
        .wrapping_add(minimum)
        .wrapping_add(element_bonus)
        .max(0);
    Some((master, AttackInformation {
        skill_id: SOUL_MIRROR_SKILL_ID,
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
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    }))
}

fn summon(
    game: &mut CGame,
    region_id: i32,
    master: MasterInfo,
    x: i32,
    y: i32,
    direction: i32,
    picture_id: u32,
    lifetime: u32,
) {
    let Some(property) = game.find_monster_property_by_picture_id(picture_id).cloned() else { return };
    let Some(mut owner) = game.take_region_owner(region_id) else { return };
    let _ = game.add_summoned_creature_owned(
        owner.base_mut(), &property, master, x, y, direction, lifetime,
    );
    game.restore_region_owner(owner);
}

pub(crate) const fn is_soul_mirror_skill(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: SOUL_MIRROR_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: SOUL_MIRROR_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }
    )
}

pub(crate) fn execute_player_soul_mirror<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_soul_mirror_skill(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some((region_id, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.learned_skill_level(SOUL_MIRROR_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(SOUL_MIRROR_SKILL_ID, level) else {
        if player_ai.soul_mirror().is_some() { abort_player_soul_mirror(game, player_id); }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let picture_id = properties.query_property(SUMMONED_CREATURE_ID);
    let lifetime = properties.query_property(SUMMONED_LIFETIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.soul_mirror().is_none() {
        let started = runtime.now_milliseconds();
        if player_ai.soul_mirror_last_used_ms() != 0 && !time_reached(runtime.now_milliseconds(), player_ai.soul_mirror_last_used_ms(), cooldown) {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || !has_mana(initial_mana, mp_loss) {
            if mp_loss != 0 { send_failure(game, player_id, 7, mp_loss); }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if destination(game, region_id, dispatch).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SOUL_MIRROR_SKILL_ID));
        }
        player_ai.begin_soul_mirror(SkillExecutionKernel::begin(dispatch, started));
    } else if player_ai.soul_mirror().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y, target)) = destination(game, region_id, dispatch) else {
        abort_player_soul_mirror(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.is_some_and(|identity| target_dead(game, region_id, identity)) {
        send_failure(game, player_id, 10, mp_loss);
        abort_player_soul_mirror(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.soul_mirror().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if !has_mana(mana, mp_loss) {
            send_failure(game, player_id, 7, mp_loss);
            abort_player_soul_mirror(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            if let Some(source) = player.shape_view() {
                player.movement_shape_mut().set_direction(get_line_direction(source.tile_x, source.tile_y, target_x, target_y));
            }
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, false);
        if let Some(execution) = player_ai.soul_mirror_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started = player_ai.soul_mirror().map(SkillExecutionKernel::started_at_ms).expect("выполнение зеркала душ создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending); }
    send_visual(game, player_id, level, true);
    let Some((direction, source_x, source_y, master)) = game.find_player(player_id).and_then(|player| Some((
        player.shape().get_direction(),
        player.shape().get_tile_x().ok()?,
        player.shape().get_tile_y().ok()?,
        master_info(player),
    ))) else {
        abort_player_soul_mirror(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (area_width, area_height) = game.area_dimensions();
    let mut attacked = Vec::new();
    for (x, y) in scope_cells(level, direction, source_x, source_y) {
        let Some((move_shapes, block)) = game.find_region(region_id).map(|owner| {
            let region = owner.base();
            let mut shapes = Vec::new();
            let _ = region.get_shapes(x, y, area_width, area_height, game, &mut shapes);
            let move_shapes = shapes
                .into_iter()
                .map(|shape| shape.identity)
                .filter(|identity| matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE))
                .collect::<Vec<_>>();
            (move_shapes, region.skill_cell_block(x, y))
        }) else { continue };
        if move_shapes.is_empty() {
            if block == 0 { summon(game, region_id, master, x, y, direction, picture_id, lifetime); }
            continue;
        }
        for target in move_shapes {
            if (target.id == player_id && target.object_type == PLAYER_TYPE)
                || attacked.contains(&target)
                || !game.owned_player_skill_target_attackable(master, target, region_id)
            {
                continue;
            }
            let Some((master, attack)) = calculate_attack(game, player_id, region_id, target, level, minimum, maximum, element_modifier, hit_modifier) else { continue };
            match target.object_type {
                PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
                MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
                _ => {}
            }
            attacked.push(target);
        }
    }
    if let Some(execution) = player_ai.soul_mirror_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_soul_mirror(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
