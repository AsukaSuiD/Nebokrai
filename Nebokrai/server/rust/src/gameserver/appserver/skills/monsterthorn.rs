//! Владелец шипастой атаки `CMonsterThorn` (`0x197`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/monsterthorn.cpp`. Player и monster/pet пути сохраняют
//! cooldown, повторную проверку длины и `BLOCK_UNFLY` после задержки, запрет
//! движения и одиночный удар по живой объектной цели. Player-формула выполняет
//! RNG физического урона и critical-roll, затем в исходном порядке передаёт
//! physical, element и soul записи; monster-формула сохраняет нулевые element
//! и soul записи и обязательный второй RNG. `CGame` только разрешает владельцев,
//! применяет защиту/смерть и доставляет готовые visual packets.
//! Player-критический множитель вычисляется в расширенной точности x87 и
//! усекается к нулю перед записью `int`. Player и monster ветви используют
//! абсолютный срок `CSkill::IsRestored`; cast-delay остаётся elapsed.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    time_reached,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::master_info;
use super::fightdefense::truncate_original;
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
pub(crate) const MONSTER_THORN_SKILL_ID: u32 = 0x197;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMonsterThornExecutionState { kernel: SkillExecutionKernel<PlayerSkillDispatch>, destination: (i32, i32), condition_checked: bool }
impl PlayerMonsterThornExecutionState { fn begin(dispatch: PlayerSkillDispatch, destination: (i32, i32), now: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now), destination, condition_checked: false } } pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel } pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel } }

fn send_thorn_visual(
    game: &CGame,
    region: &CServerRegion,
    source_shape: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(MONSTER_THORN_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    if action == 1 {
        message.add_long(source_shape.get_direction());
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source_shape, None, &message);
}
#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_monster_thorn<Runtime: GameMainLoopRuntime>(
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
    let Some((
        source_shape,
        property,
        attacker_master,
        attacker_tamed,
        pet_attack,
        cast,
        last_used_ms,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        Some((
            monster.move_shape().shape().clone(),
            property.clone(),
            monster.master_info(),
            monster.is_tamed(),
            monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property)),
            monster.base_attack_cast(),
            monster.skill_last_used_ms(MONSTER_THORN_SKILL_ID),
        ))
    }) else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some_and(|execution| {
                execution.dispatch().skill_id == MONSTER_THORN_SKILL_ID
            }) {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            &property,
            attacker_tamed,
            attacker_master,
            target_identity,
            &target,
        )
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some_and(|execution| {
                execution.dispatch().skill_id == MONSTER_THORN_SKILL_ID
            }) {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source_shape.get_tile_x(),
        source_shape.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return true;
    };

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            now_ms,
        ) {
            return true;
        }
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let attack_interval = pet_attack.map_or(property.attack_speed, |pet| pet.attack_interval);
        let schedule_ready = schedule_attack_interval(property.ai, attack_interval)
            .is_none_or(|interval| {
                region
                    .find_monster_by_id_mut(monster_id)
                    .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval))
            });
        if !schedule_ready
            || !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                    last_used_ms,
                    reuse_delay_ms,
                    now_ms,
                )
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                MONSTER_THORN_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        let source_shape = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source_shape);
        send_thorn_visual(
            game,
            region,
            source_shape,
            monster_id,
            skill_level,
            1,
            None,
        );
        return true;
    }

    let cast = cast.expect("выполнение шипастой атаки проверено выше");
    if cast.dispatch().skill_id != MONSTER_THORN_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    send_thorn_visual(
        game,
        region,
        &source_shape,
        monster_id,
        skill_level,
        2,
        Some((target_identity, target_x, target_y)),
    );

    let ordinary_attack = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            monster.state_attack_bounds(property.minimum_attack, property.maximum_attack)
        })
        .unwrap_or((property.minimum_attack, property.maximum_attack));
    let physical_minimum = pet_attack.map_or(ordinary_attack.0, |pet| pet.minimum_attack) as i32;
    let physical_maximum = pet_attack.map_or(ordinary_attack.1, |pet| pet.maximum_attack) as i32;
    let physical_span = physical_maximum
        .wrapping_sub(physical_minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
    // `CMonster::GetCriticalChance` равен нулю, но исходный код всё равно
    // выполняет второй RNG-вызов `random(100)`.
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: MONSTER_THORN_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties
            .query_property(super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER)
            as i32,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical.max(0),
                mp_damage: 0,
            },
            // Виртуальные `GetAddElementAtk/GetAddSoulAtk` монстра возвращают
            // ноль, но обе записи остаются частью исходного порядка защиты.
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: 0,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: 0,
                mp_damage: 0,
            },
        ],
    };
    let attack = defend_owned_monster_attack(
        game,
        target_identity,
        target.mana,
        target.war_soul_mana,
        target.player_properties,
        target.monster_properties,
        attack,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    apply_owned_monster_attack_hit(
        game,
        region,
        runtime,
        now_ms,
        monster_id,
        attacker_master,
        target_identity,
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
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        monster.move_shape_mut().set_moveable(true);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) const fn is_player_monster_thorn_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Point { skill_id: MONSTER_THORN_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: MONSTER_THORN_SKILL_ID, .. }) }
fn player_target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> { match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None } }
fn player_destination(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch, fallback: Option<(i32, i32)>) -> Option<(i32, i32)> { match dispatch { PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)), PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)).or(fallback), PlayerSkillDispatch::SelfTarget { .. } => None } }
fn send_player_failure(game: &CGame, player_id: i32, action: u8) { game.send_self_state_skill_failure(0x000b_fe01, player_id, action); }
fn send_player_visual(game: &mut CGame, player_id: i32, level: i32, action: u8, target: Option<ShapeIdentity>, destination: (i32, i32)) { let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return }; let mut message = CMessage::new(0x000b_fe01); message.add_byte(action); message.add_long(MONSTER_THORN_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); if action == 1 { message.add_long(direction); } else { message.add_long(target.map_or(0, |target| target.object_type)); message.add_long(target.map_or(0, |target| target.id)); message.add_long(destination.0); message.add_long(destination.1); } let _ = game.send_player_shape_around(player_id, None, &message); }
fn restore_player(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) { restore_player(game, player_id); finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(MONSTER_THORN_SKILL_ID, now_ms)); }
pub(crate) fn cancel_player_monster_thorn<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false }; finish_player(game, player_id, ai, runtime); ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }
fn calculate_player_attack(game: &mut CGame, player_id: i32, level: i32, hit: i32) -> Option<(MasterInfo, AttackInformation)> { let (combat, master) = game.find_player(player_id).map(|player| (player.combat_properties(), master_info(player)))?; let minimum = combat.minimum_attack as i32; let span = (combat.maximum_attack as i32).wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32; let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0); let mut attack = AttackInformation { skill_id: MONSTER_THORN_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit, damage_factor: 1.0, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] }; if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } } Some((master, attack)) }

pub(crate) fn execute_player_monster_thorn<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_player_monster_thorn_dispatch(dispatch) { return player_terminal(QueuedSkillExecutionState::Rejected) }
    let Some((region_id, source_x, source_y, level)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(MONSTER_THORN_SKILL_ID)))) else { return player_terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(MONSTER_THORN_SKILL_ID, level).cloned() else { if ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).is_some() { restore_player(game, player_id); abort_skill(game, player_id); } return player_terminal(QueuedSkillExecutionState::Rejected) }; let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE); let hit = properties.query_property(super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER) as i32; let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED); let now = runtime.now_milliseconds();
    if ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).is_none() { if !skill_is_restored(ai.skill_last_used_ms(MONSTER_THORN_SKILL_ID), reuse, now) { send_player_failure(game, player_id, 0x0d); return player_terminal(QueuedSkillExecutionState::Rejected) } let Some(destination) = player_destination(game, region_id, dispatch, None) else { return player_terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if maximum != 0 && path.len() > maximum as usize { send_player_failure(game, player_id, 0x0b); return player_terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == 2) { send_player_failure(game, player_id, 0x0f); return player_terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(MONSTER_THORN_SKILL_ID)); } ai.begin_player_skill_execution(PlayerMonsterThornExecutionState::begin(dispatch, destination, now)); }
    let fallback = ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).map(|state| state.destination); let Some(destination) = player_destination(game, region_id, dispatch, fallback) else { restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }; if player_target(dispatch).is_some_and(|target| game.base_magic_target_view(region_id, target).is_some() && game.periodic_state_target_dead(region_id, target)) { send_player_failure(game, player_id, 10); restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }
    if ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).is_some_and(|state| !state.condition_checked) { if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_player_visual(game, player_id, level, 1, None, destination); if let Some(state) = ai.player_skill_state_mut::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID) { state.condition_checked = true; let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.player_skill_state::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID).map(|state| state.kernel().started_at_ms()).unwrap_or_default(); if !time_reached(now, started, delay) { return player_terminal(QueuedSkillExecutionState::Pending) } let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if maximum != 0 && path.len() > maximum as usize { send_player_failure(game, player_id, 0x0b); restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == 2) { send_player_failure(game, player_id, 0x0f); restore_player(game, player_id); abort_skill(game, player_id); return player_terminal(QueuedSkillExecutionState::Rejected) }
    let target = player_target(dispatch).filter(|target| game.base_magic_target_view(region_id, *target).is_some()); send_player_visual(game, player_id, level, 2, target, destination); if let Some(target) = target && game.find_player(player_id).map(master_info).is_some_and(|master| game.owned_player_skill_target_attackable(master, target, region_id)) { if let Some((master, attack)) = calculate_player_attack(game, player_id, level, hit) { match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} } } } if let Some(state) = ai.player_skill_state_mut::<PlayerMonsterThornExecutionState>(MONSTER_THORN_SKILL_ID) { let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply); } finish_player(game, player_id, ai, runtime); player_terminal(QueuedSkillExecutionState::Completed)
}
