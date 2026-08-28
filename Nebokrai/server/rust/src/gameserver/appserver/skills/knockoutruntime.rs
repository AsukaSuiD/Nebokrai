//! Рабочий владелец исполнения `CKnockOut` (`0x192`) с объектом-целью.
//!
//! Формулы, RNG, проверки, время восстановления и сетевой формат принадлежат
//! навыку; `CGame` остаётся координатором общей защиты, жизненного цикла цели
//! и доставки.

use super::baseattack::time_reached;
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::knockoutstate::{KnockOutState, send_knock_out_state_visual};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const KNOCK_OUT_SKILL_ID: u32 = 0x192;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const CURE_SKILL_ID: u32 = 0x131;
const UNKNOWN_ATTACK_SKILL_ID: u32 = 0x7fff_ffff;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

#[derive(Clone, Copy)]
struct Target { identity: ShapeIdentity, x: i32, y: i32, level: u8, dodge: u16, dead: bool, cured: bool }

fn result(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<Target> {
    match identity.object_type {
        PLAYER_TYPE => {
            let player = game.find_player(identity.id)?;
            (player.server_region_id() == Some(region_id)).then_some(Target {
                identity, x: player.shape().get_tile_x().ok()?, y: player.shape().get_tile_y().ok()?,
                level: player.level(), dodge: player.combat_properties().dodge,
                dead: player.is_dead(), cured: player.has_state_by_skill_id(CURE_SKILL_ID),
            })
        }
        MONSTER_TYPE => {
            let monster = game.find_region(region_id)?.base().find_monster_by_id(identity.id)?;
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(Target {
                identity, x: monster.move_shape().shape().get_tile_x().ok()?, y: monster.move_shape().shape().get_tile_y().ok()?,
                level: property.level as u8, dodge: property.dodge as u16, dead: monster.hit_points() == 0,
                cured: monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID),
            })
        }
        _ => None,
    }
}

fn failure(game: &CGame, player_id: i32, action: u8) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(0); message.add_byte(action);
    let _ = message.send_to_player(game.net_server(), player_id);
}

fn cast(game: &mut CGame, player_id: i32, target: Target, level: i32, fire: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let source = player.shape();
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if fire { 2 } else { 1 });
    message.add_long(KNOCK_OUT_SKILL_ID as i32); message.add_short(level as i16);
    message.add_long(source.identity().object_type); message.add_long(source.identity().id);
    if fire {
        message.add_long(target.identity.object_type); message.add_long(target.identity.id);
        message.add_long(target.x); message.add_long(target.y);
    } else { message.add_long(source.get_direction()); }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn master(player: &CPlayer) -> MasterInfo {
    let p = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE, master_id: player.player_id(), master_guild_id: player.faction_id(),
        master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()),
        permitted_to_kill_player: i32::from(p.player), permitted_to_kill_teammate: i32::from(p.teammate),
        permitted_to_kill_guild_member: i32::from(p.guild_member), permitted_to_kill_criminal: i32::from(p.criminal),
    }
}

fn attack(game: &mut CGame, player_id: i32, target_level: u8) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?;
    let combat = player.combat_properties();
    let master = master(player);
    let weapon = player.weapon_damage_level(game.goods_factory());
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let delta = weapon.wrapping_sub(i32::from(target_level)).max(0);
    let factor = (if divisor == 0.0 { 1.0 } else { delta as f32 / divisor }).min(1.0).max(minimum_factor);
    let difference = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32);
    let span = if difference < 0 { difference.wrapping_neg() } else { difference }.wrapping_add(1);
    let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(span)).max(0);
    let mut value = AttackInformation {
        // `CKnockOut::Attack` не переписывает значения конструктора
        // `tagAttackInformation`: ID остаётся `SKILL_UNKNOW`, а уровень — 1.
        skill_id: UNKNOWN_ATTACK_SKILL_ID, skill_level: 1, attacker_type: PLAYER_TYPE, attacker_id: player_id,
        attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier: 100, damage_factor: factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: combat.add_element_attack as i32, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 },
        ],
    };
    if game.skill_random_below(100) < i32::from(combat.cch) {
        value.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut value.damages { power.hp_damage = ((power.hp_damage as f32) * rate).round_ties_even() as i32; }
    }
    Some((master, value))
}

fn install(game: &mut CGame, region_id: i32, target: Target, state: KnockOutState, now_ms: u32) {
    let installed = match target.identity.object_type {
        PLAYER_TYPE => game.find_player_mut(target.identity.id).map(|player| {
            let old = player.replace_knock_out_state(state);
            if old.is_some() { player.set_skill_fightable(true); player.set_skill_moveable(true); }
            player.set_skill_moveable(false); player.set_skill_fightable(false); old
        }),
        MONSTER_TYPE => {
            let mut owner = game.take_region_owner(region_id);
            let old = owner.as_mut().and_then(|owner| owner.base_mut().find_monster_by_id_mut(target.identity.id)).map(|monster| {
                let old = monster.move_shape_mut().replace_knock_out_state(state);
                if old.is_some() { monster.move_shape_mut().set_fightable(true); monster.move_shape_mut().set_moveable(true); }
                monster.move_shape_mut().set_moveable(false); monster.move_shape_mut().set_fightable(false); old
            });
            if let Some(owner) = owner { game.restore_region_owner(owner); }
            old
        }
        _ => None,
    };
    let Some(old) = installed else { return };
    if let Some(old) = old { send_knock_out_state_visual(game, region_id, target.identity, target.x, target.y, old, false, now_ms); }
    send_knock_out_state_visual(game, region_id, target.identity, target.x, target.y, state, true, now_ms);
    if target.identity.object_type == PLAYER_TYPE { let _ = game.publish_player_states(target.identity.id); }
}

fn release(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); player.set_current_skill_id(None); }
}

pub(crate) fn execute_player_knock_out<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let identity = match dispatch { PlayerSkillDispatch::Object { skill_id: KNOCK_OUT_SKILL_ID, target } if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => target, _ => return result(QueuedSkillExecutionState::Rejected) };
    let Some((region_id, sx, sy, level)) = game.find_player(player_id).and_then(|p| Some((p.server_region_id()?, p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.learned_skill_level(KNOCK_OUT_SKILL_ID)))) else { return result(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(KNOCK_OUT_SKILL_ID, level) else { failure(game, player_id, 2); return result(QueuedSkillExecutionState::Rejected) };
    let delay = properties.query_property(DELAY); let persist = properties.query_property(PERSIST);
    let reuse = properties.query_property(REUSE); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let _can_break = properties.query_property(CAN_BREAK);
    let Some(mut target) = target_snapshot(game, region_id, identity) else { failure(game, player_id, 10); return result(QueuedSkillExecutionState::Rejected) };
    if ai.knock_out().is_none() {
        let now = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if ai.knock_out_last_used_ms() != 0 && !time_reached(now, ai.knock_out_last_used_ms(), reuse) {
            failure(game, player_id, 0x0d); failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        if maximum != 0 && maximum.wrapping_add(1) < game.base_magic_path(region_id, sx, sy, target.x, target.y, None).len() as u32 {
            failure(game, player_id, 0x0b); failure(game, player_id, 2);
            return result(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false); player.set_current_skill_id(Some(KNOCK_OUT_SKILL_ID));
            player.movement_shape_mut().set_direction(get_line_direction(sx, sy, target.x, target.y));
        }
        ai.begin_knock_out(SkillExecutionKernel::begin(dispatch, now));
    } else if ai.knock_out().is_none_or(|execution| execution.dispatch() != dispatch) { return result(QueuedSkillExecutionState::Rejected); }
    if target.dead { failure(game, player_id, 10); release(game, player_id); return result(QueuedSkillExecutionState::Rejected); }
    if ai.knock_out().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        cast(game, player_id, target, level, false);
        if let Some(execution) = ai.knock_out_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }
    let started = ai.knock_out().map(SkillExecutionKernel::started_at_ms).expect("выполнение оглушения создано");
    if !time_reached(runtime.now_milliseconds(), started, delay) { return result(QueuedSkillExecutionState::Pending); }
    release(game, player_id);
    target = match target_snapshot(game, region_id, identity) { Some(target) => target, None => return result(QueuedSkillExecutionState::Rejected) };
    let Some((sx, sy, combat, source_level)) = game.find_player(player_id).and_then(|p| Some((p.shape().get_tile_x().ok()?, p.shape().get_tile_y().ok()?, p.combat_properties(), p.level()))) else { return result(QueuedSkillExecutionState::Rejected) };
    if maximum != 0 && maximum.wrapping_add(1) < game.base_magic_path(region_id, sx, sy, target.x, target.y, None).len() as u32 { failure(game, player_id, 0x0b); return result(QueuedSkillExecutionState::Rejected); }
    cast(game, player_id, target, level, true);
    let (base, magnify, level_rate) = game.globe_setup().base_attack_hit_formula();
    let chance = ((i32::from(combat.hit).wrapping_sub(i32::from(target.dodge))) as f32 * magnify + base as f32 + i32::from(source_level).wrapping_sub(i32::from(target.level)) as f32 * level_rate).round_ties_even() as i32;
    if chance <= game.skill_random_below(100) { return result(QueuedSkillExecutionState::Completed); }
    if let Some((master, attack)) = attack(game, player_id, target.level) {
        match target.identity.object_type {
            PLAYER_TYPE => game.apply_periodic_state_attack_to_player(master, target.identity.id, region_id, attack, runtime),
            MONSTER_TYPE => game.apply_periodic_state_attack_to_monster(master, target.identity.id, region_id, attack, runtime), _ => {}
        }
    }
    if !target.cured {
        let state_now = runtime.now_milliseconds();
        install(game, region_id, target, KnockOutState::new(state_now, persist), runtime.now_milliseconds());
    }
    if let Some(execution) = ai.knock_out_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    ai.mark_knock_out_used(runtime.now_milliseconds());
    result(QueuedSkillExecutionState::Completed)
}
