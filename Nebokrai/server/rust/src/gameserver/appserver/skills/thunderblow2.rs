//! Второй громовой удар `CThunderBlow2` (`0x14D`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/thunderblow2.cpp`. Навык принимает только отдельную живую
//! цель, дважды проверяет дальность, после исходной задержки отбрасывает более
//! слабую цель без состояния `0x74`, публикует попадание и сразу применяет
//! элементальную атаку. Формула сохраняет ровно два вызова MSVCRT RNG. `CGame`
//! только координирует независимых владельцев региона и цели при `ForceMove`
//! и последующем применении рассчитанной атаки.
//! Подтверждённый `End` не возвращает движение: он очищает внутренний путь,
//! обновляет свойства через `CSummonSkill`, очищает текущий навык и фиксирует
//! время восстановления.
//! Element modifier вычисляется в расширенной точности x87 из целых свойств и
//! сохранённой `f32`-константы; он и критический множитель усекаются к нулю
//! перед `int`. Восстановление использует абсолютный срок
//! `CSkill::IsRestored`; отложенный удар сохраняет elapsed-семантику.

use super::baseattack::{SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
};
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const THUNDER_BLOW_2_SKILL_ID: u32 = 0x14d;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;
const TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;
const PILLAR_STATE_ID: u32 = 0x74;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn finish_player_thunder_blow_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(THUNDER_BLOW_2_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_thunder_blow_2<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(THUNDER_BLOW_2_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
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

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32, check_cast: bool) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 if check_cast => game.send_skill_system_info(player_id, b"GS0286"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_direction_visual(game: &mut CGame, player_id: i32, level: i32, action: u8) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(THUNDER_BLOW_2_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_attack_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(2);
    message.add_long(THUNDER_BLOW_2_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(CPlayer::level),
        MONSTER_TYPE => game.find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .and_then(|monster| monster.base_property_key())
            .and_then(|key| game.find_monster_property_by_origin_name(key))
            .map(|property| property.level as u8),
        _ => None,
    }
}

fn knockback_destination(
    game: &CGame,
    region_id: i32,
    source_x: i32,
    source_y: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    steps: u32,
) -> (i32, i32, u32) {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else {
        return (target_x, target_y, 0);
    };
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    let mut position = ShapeAreaCoordinates { x: target_x, y: target_y };
    let mut moved = 0_u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break };
        if region.block_at(next.x, next.y) != Some(0) { break }
        position = next;
        moved = moved.wrapping_add(1);
    }
    tracing::trace!(region_id, ?target, moved, "рассчитано отбрасывание второго громового удара");
    (position.x, position.y, moved)
}

#[allow(clippy::too_many_arguments, reason = "аргументы соответствуют полям исходной формулы")]
fn calculate_attack(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    minimum: i32,
    maximum: i32,
    element_modifier: u32,
    hit_modifier: i32,
    damage_modifier: i32,
) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let width_delta = maximum.wrapping_sub(minimum);
    let width = if width_delta < 0 { width_delta.wrapping_neg() } else { width_delta }.wrapping_add(1);
    let element_bonus = truncate_original(
        f64::from(element_modifier)
            * f64::from(0.01_f32)
            * f64::from(combat.element_modify),
    );
    let damage = (combat.add_element_attack as i32)
        .wrapping_add(game.skill_random_below(width))
        .wrapping_add(minimum)
        .wrapping_add(element_bonus)
        .max(0);
    let mut attack = AttackInformation {
        skill_id: THUNDER_BLOW_2_SKILL_ID,
        skill_level: level as u8,
        attacker_type: PLAYER_TYPE,
        attacker_id: player_id,
        attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id,
        hit_modifier,
        damage_factor: 1.0,
        damage_modifier,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage =
                truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
    Some((master, attack))
}

pub(crate) const fn is_thunder_blow_2_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch, PlayerSkillDispatch::Object {
        skill_id: THUNDER_BLOW_2_SKILL_ID,
        target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
    })
}

pub(crate) fn execute_player_thunder_blow_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let target = match dispatch {
        PlayerSkillDispatch::Object { skill_id: THUNDER_BLOW_2_SKILL_ID, target }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => target,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, level, source_level, source_x, source_y, initial_mana)) = game.find_player(player_id)
        .and_then(|player| Some((
            player.server_region_id()?, player.learned_skill_level(THUNDER_BLOW_2_SKILL_ID, game.skill_factory()),
            player.level(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana(),
        )))
    else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(THUNDER_BLOW_2_SKILL_ID, level) else {
        if player_ai.player_skill_execution(THUNDER_BLOW_2_SKILL_ID).is_some() {
            finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let back_steps = properties.query_property(TARGET_BACK_STEP);
    let move_speed = properties.query_property(TARGET_MOVE_SPEED);
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let damage_modifier = properties.query_property(TARGET_FINAL_DAMAGE_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_execution(THUNDER_BLOW_2_SKILL_ID).is_none() {
        if target.object_type == PLAYER_TYPE && target.id == player_id {
            send_failure(game, player_id, 10, mp_loss, true);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if !skill_is_restored(
            player_ai.skill_last_used_ms(THUNDER_BLOW_2_SKILL_ID),
            cooldown_ms,
            runtime.now_milliseconds(),
        ) {
            send_failure(game, player_id, 0x0d, mp_loss, true);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 10, mp_loss, true);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(region_id, source_x, source_y, target_view.tile_x, target_view.tile_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b, mp_loss, true);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss, true);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(THUNDER_BLOW_2_SKILL_ID));
            player.set_skill_moveable(false);
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, runtime.now_milliseconds()));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_execution(THUNDER_BLOW_2_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.player_skill_execution(THUNDER_BLOW_2_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss, false);
            finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            send_failure(game, player_id, 10, mp_loss, false);
            finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_view.tile_x, target_view.tile_y));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_direction_visual(game, player_id, level, 1);
        if let Some(execution) = player_ai.player_skill_execution_mut(THUNDER_BLOW_2_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai.player_skill_execution(THUNDER_BLOW_2_SKILL_ID).map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение второго громового удара создано");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if game.periodic_state_target_dead(region_id, target) {
        send_failure(game, player_id, 10, mp_loss, false);
        finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(target_view) = game.base_magic_target_view(region_id, target) else {
        send_failure(game, player_id, 10, mp_loss, false);
        finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let path = game.base_magic_path(region_id, source_x, source_y, target_view.tile_x, target_view.tile_y, None);
    if maximum_distance != 0 && path.len() > maximum_distance as usize {
        send_failure(game, player_id, 0x0b, mp_loss, false);
        finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if target_level(game, region_id, target).is_some_and(|target_level| target_level <= source_level)
        && !game.target_has_state_by_skill_id(region_id, target, PILLAR_STATE_ID)
    {
        let (destination_x, destination_y, moved) = knockback_destination(
            game, region_id, source_x, source_y, target, target_view.tile_x, target_view.tile_y, back_steps,
        );
        let _ = game.force_move_thunder_blow_2_target(
            region_id, target, destination_x, destination_y, move_speed.wrapping_mul(moved),
        );
    }
    let visual_target = game.base_magic_target_view(region_id, target).unwrap_or(target_view);
    send_attack_visual(game, player_id, level, target, visual_target.tile_x, visual_target.tile_y);
    if let Some(execution) = player_ai.player_skill_execution_mut(THUNDER_BLOW_2_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
    }

    let master = game.find_player(player_id).map(master_info).unwrap_or_default();
    if !game.owned_player_skill_target_attackable(master, target, region_id) {
        send_direction_visual(game, player_id, level, 3);
        finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((master, attack)) = calculate_attack(
        game, player_id, level, minimum, maximum, element_modifier, hit_modifier, damage_modifier,
    ) else {
        finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    match target.object_type {
        PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
        MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
        _ => unreachable!("тип цели проверен dispatcher-ом"),
    }
    if let Some(execution) = player_ai.player_skill_execution_mut(THUNDER_BLOW_2_SKILL_ID) {
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_thunder_blow_2(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
