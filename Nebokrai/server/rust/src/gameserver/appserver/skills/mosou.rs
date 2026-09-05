//! Фронтальный удар Мо-шоу `CMosou` (`0x65`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/mosou.cpp`. После проверки меча и двукратной проверки MP
//! навык атакует все допустимые фигуры лицевой клетки в порядке регионального
//! индекса. На каждую цель сохраняются два RNG-вызова формулы, затем отдельный
//! бросок оглушения, проверка уровня и `Cure`, replacement состояния `0x192`
//! и отбрасывание. `CGame` оставляет только межвладельческую доставку, защиту
//! и пространственное применение уже рассчитанного результата. `Attack` и
//! `AI` не изнашивают оружие на отдельных целях: унаследованный
//! `AfterUseSkill` делает это один раз через подтверждённый общий `End(1)`.
//! Критический множитель вычисляется в расширенной точности x87 и усекается к
//! нулю перед записью `int`. Восстановление использует абсолютный срок
//! `CSkill::IsRestored`; задержка и длительность оглушения остаются elapsed.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::cure::CURE_SKILL_ID;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::knockoutstate::KnockOutState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
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

pub(crate) const MOSOU_SKILL_ID: u32 = 0x65;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const BASE_PROBABILITY: u32 = 40_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn finish_player_mosou<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_skill_used(MOSOU_SKILL_ID, now_ms);
    });
}

pub(crate) fn cancel_player_mosou<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(MOSOU_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_mosou(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(),
        master_guild_id: player.faction_id(), master_team_id: player.team_id(),
        master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
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
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0287"),
        _ => {}
    }
}

fn send_visual(game: &mut CGame, player_id: i32, level: i32, action: u8) {
    let Some((direction, face)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_direction(), player.shape().get_face_position().ok()?))
    }) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(MOSOU_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(direction);
    } else {
        message.add_long(0); message.add_long(0); message.add_long(face.x); message.add_long(face.y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn destination(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(|player| {
            let face = player.shape().get_face_position().ok()?;
            Some((face.x, face.y))
        }),
    }
}

fn weapon_is_sword(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn target_level_avoid(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<(u8, u16)> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(|player| (player.level(), player.combat_properties().attack_avoid)),
        MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
            let monster = owner.base().find_monster_by_id(target.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let combat = monster.combat_properties(property);
            Some((combat.level, combat.attack_avoid))
        }),
        _ => None,
    }
}

fn target_dead(game: &CGame, region_id: i32, target: ShapeIdentity) -> bool {
    game.periodic_state_target_dead(region_id, target)
}

fn front_targets(game: &CGame, region_id: i32, player_id: i32) -> Vec<ShapeIdentity> {
    let Some((face, source)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_face_position().ok()?, player.shape().identity()))
    }) else { return Vec::new() };
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (area_width, area_height) = game.area_dimensions();
    let mut shapes = Vec::new();
    if region.get_shapes(face.x, face.y, area_width, area_height, game, &mut shapes).is_err() {
        return Vec::new();
    }
    shapes.into_iter().map(|shape| shape.identity).filter(|identity| {
        *identity != source && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
    }).collect()
}

fn knockback_destination(
    game: &CGame, region_id: i32, source_x: i32, source_y: i32,
    target: ShapeIdentity, steps: u32,
) -> Option<(i32, i32, u32)> {
    let view = game.base_magic_target_view(region_id, target)?;
    let region = game.find_region(region_id)?.base();
    let direction = get_line_direction(source_x, source_y, view.tile_x, view.tile_y);
    let mut position = ShapeAreaCoordinates { x: view.tile_x, y: view.tile_y };
    let mut moved = 0_u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break };
        if region.block_at(next.x, next.y) != Some(0) { break }
        position = next;
        moved = moved.wrapping_add(1);
    }
    Some((position.x, position.y, moved))
}

fn calculate_attack(game: &mut CGame, player_id: i32, level: i32, hit_modifier: i32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master_info(player);
    let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_abs().wrapping_add(1);
    let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let mut attack = AttackInformation {
        skill_id: MOSOU_SKILL_ID, skill_level: level as u8,
        attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id,
        attacker_union_id: master.master_union_id, hit_modifier, damage_factor: 1.0,
        damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
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

pub(crate) const fn is_mosou_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == MOSOU_SKILL_ID,
    }
}

pub(crate) fn execute_player_mosou<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_mosou_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, level, source_level, source_x, source_y, initial_mana)) = game.find_player(player_id).and_then(|player| Some((
        player.server_region_id()?, player.learned_skill_level(MOSOU_SKILL_ID), player.level(),
        player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.mana(),
    ))) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(MOSOU_SKILL_ID, level) else {
        if player_ai.player_skill_execution(MOSOU_SKILL_ID).is_some() {
            finish_player_mosou(game, player_id, player_ai, runtime);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let probability = properties.query_property(BASE_PROBABILITY) as i32;
    let persist_ms = properties.query_property(STATE_PERSIST_TIME);
    let back_steps = properties.query_property(TARGET_BACK_STEP);
    let move_speed = properties.query_property(TARGET_MOVE_SPEED);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_execution(MOSOU_SKILL_ID).is_none() {
        if !skill_is_restored(
            player_ai.skill_last_used_ms(MOSOU_SKILL_ID),
            cooldown_ms,
            runtime.now_milliseconds(),
        ) { send_failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) }
        let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) };
        if !weapon_is_sword(game, player) {
            send_failure(game, player_id, 0x0e, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) }
        if (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false); player.set_current_skill_id(Some(MOSOU_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, runtime.now_milliseconds()));
    } else if player_ai.player_skill_execution(MOSOU_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.player_skill_execution(MOSOU_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss); finish_player_mosou(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_sword(game, player)) {
            send_failure(game, player_id, 0x0e, mp_loss); finish_player_mosou(game, player_id, player_ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some((target_x, target_y)) = destination(game, region_id, player_id, dispatch)
            && let Some(player) = game.find_player_mut(player_id)
        { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); }
        send_visual(game, player_id, level, 1);
        if let Some(execution) = player_ai.player_skill_execution_mut(MOSOU_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = player_ai.player_skill_execution(MOSOU_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение Мо-шоу создано");
    if !time_reached(runtime.now_milliseconds(), started, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_visual(game, player_id, level, 2);
    if let Some(execution) = player_ai.player_skill_execution_mut(MOSOU_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let master = game.find_player(player_id).map(master_info).unwrap_or_default();
    for target in front_targets(game, region_id, player_id) {
        if !game.owned_player_skill_target_attackable(master, target, region_id) { continue }
        let Some((master, attack)) = calculate_attack(game, player_id, level, hit_modifier) else { continue };
        match target.object_type {
            PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime),
            _ => continue,
        }
        let Some((target_level, attack_avoid)) = target_level_avoid(game, region_id, target) else { continue };
        if game.skill_random_below(100) >= probability.wrapping_sub(i32::from(attack_avoid))
            || target_level > source_level || target_dead(game, region_id, target)
            || game.target_has_state_by_skill_id(region_id, target, CURE_SKILL_ID)
        { continue }
        let Some((destination_x, destination_y, moved)) = knockback_destination(
            game, region_id, source_x, source_y, target, back_steps,
        ) else { continue };
        let state_now_ms = runtime.now_milliseconds();
        let state = KnockOutState::new(state_now_ms, persist_ms);
        let _ = game.apply_mosou_control(
            region_id, target, state, destination_x, destination_y,
            move_speed.wrapping_mul(moved), state_now_ms,
        );
    }
    if let Some(execution) = player_ai.player_skill_execution_mut(MOSOU_SKILL_ID) { let _ = execution.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_mosou(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
