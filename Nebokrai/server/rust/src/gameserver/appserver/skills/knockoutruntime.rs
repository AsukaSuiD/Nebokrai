//! Оглушающий удар CKnockOut (0x192), gameserver.exe + GameServer.pdb,
//! appserver/skills/knockout.cpp; payload состояния — knockoutstate.cpp.
//! Общий Begin сохраняет U/S и часы до создания visual и CheckCast. Первый
//! AI задаёт breakability, направление и visual(0); reuse и delay сравниваются
//! с абсолютными wrapping-сроками. Общий AI обслуживает игрока и монстра.
//! Удар сохраняет UNKNOWN/1, отдельный hit RNG и два RNG Calculate; при
//! промахе End(0), после попадания — поздний Cure и End(1), без лишнего RP.
//! Новый payload создаётся до End старого состояния и полного Begin, затем
//! занимает прежний слот; без старого состояния добавляется в конец.
//! Настоящие AI/регион опубликованы через callbacks. Завершение использует
//! захваченный поколенческий ключ, не замену навыка с тем же ID.

use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::blindstate::begin_primary_blind_state_at;
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::knockoutstate::KnockOutState;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterSkillCallOutcome, finish_monster_skill_call,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_owned_skill_begin_object, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

pub(crate) const KNOCK_OUT_SKILL_ID: u32 = 0x192;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const CURE_SKILL_ID: u32 = 0x131;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

fn knock_out_hit_chance(
    source_hit: u16,
    target_dodge: u16,
    source_level: u8,
    target_level: u8,
    base: i32,
    magnify: f32,
    level_rate: f32,
) -> i32 {
    let level_component = (f64::from(
        i32::from(source_level).wrapping_sub(i32::from(target_level)),
    ) * f64::from(level_rate)
        + f64::from(base)) as f32;
    truncate_original(
        f64::from(i32::from(source_hit).wrapping_sub(i32::from(target_dodge)))
            * f64::from(magnify)
            + f64::from(level_component),
    )
}

fn participant(game: &CGame, value: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

fn sufferer(game: &CGame, instance: RegisteredSkill) -> Option<(i32, ShapeIdentity)> {
    participant(game, resolve_skill_sufferer(game, game.registered_skill(instance)?.lifecycle())?)
}

fn stage(game: &CGame, instance: RegisteredSkill) -> Option<SkillStage> {
    let skill = game.registered_skill(instance)?;
    skill.monster_kernel().map(|kernel| kernel.stage()).or_else(|| {
        skill.player_kernel().map(|kernel| kernel.stage())
    })
}

fn advance(game: &mut CGame, instance: RegisteredSkill, from: SkillStage, to: SkillStage) {
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    if let Some(kernel) = skill.monster_kernel_mut() {
        let _ = kernel.advance(from, to);
    } else if let Some(kernel) = skill.player_kernel_mut() {
        let _ = kernel.advance(from, to);
    }
}

pub(crate) fn publish_knock_out_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CKnockOut
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::KnockOut || effect.is_ended()
        })
    { return; }
    let Some(user) = participant(game, skill.lifecycle().user()) else { return; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1).map(|shape| shape.shape()) else { return; };
    let mut message = CMessage::new(0x000b_fe01);
    match mode {
        2 | 7 | 10 | 11 | 13 | 15 => {
            if user.1.object_type == PLAYER_TYPE {
                message.add_byte(0);
                message.add_byte(mode as u8);
                let _ = message.send_to_player(game.net_server(), user.1.id);
            }
            return;
        }
        0 | 1 => {
            message.add_byte(if mode == 0 { 1 } else { 2 });
            message.add_long(skill.id() as i32);
            message.add_short(skill.level() as i16);
            message.add_long(user.1.object_type);
            message.add_long(user.1.id);
            if mode == 0 {
                message.add_long(source.get_direction());
            } else {
                let Some(target) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
                let Some(shape) = resolve_state_move_shape(game, target.0, target.1).map(|shape| shape.shape()) else { return; };
                let (x, y) = (
                    shape.get_tile_x().unwrap_or(i32::MIN),
                    shape.get_tile_y().unwrap_or(i32::MIN),
                );
                message.add_long(shape.identity().object_type);
                message.add_long(shape.identity().id);
                message.add_long(x);
                message.add_long(y);
            }
        }
        _ => return,
    }
    if let Some(region) = game.find_region(user.0) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let lifecycle = *skill.lifecycle();
    let Some(user) = participant(game, lifecycle.user()) else { return false; };
    if sufferer(game, instance).is_none() { return false; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return false; };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        return false;
    }
    let path = game.skill_target_path(&lifecycle);
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE).wrapping_add(1) < path.len() as u32
    {
        game.update_registered_skill_visual(instance, 11);
        return false;
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, user.0, user.1) {
        shape.set_moveable(false);
    }
    true
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

fn monster_properties(game: &CGame, source: (i32, ShapeIdentity)) -> Option<&MonsterProperties> {
    let monster = game.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
    game.find_monster_property_by_origin_name(monster.base_property_key()?)
}

fn bounds(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(u32, u32)> {
    if source.1.object_type == PLAYER_TYPE {
        let combat = game.find_player(source.1.id)?.combat_properties();
        Some((combat.minimum_attack, combat.maximum_attack))
    } else {
        let monster = game.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
        let properties = monster_properties(game, source)?;
        Some(monster.state_attack_bounds(properties.minimum_attack, properties.maximum_attack))
    }
}

fn calculate_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    if game.skill_base_properties(skill.id(), skill.level()).is_none() { return; }
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    attack.damage_factor = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
        player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor)
    } else { 1.0 };
    attack.hit_modifier = 100;
    let Some((_, maximum)) = bounds(game, source) else { return; };
    let Some((minimum, _)) = bounds(game, source) else { return; };
    let difference = (maximum as i32).wrapping_sub(minimum as i32);
    let span = if difference < 0 { difference.wrapping_neg() } else { difference }.wrapping_add(1);
    let random = game.skill_random_below(span);
    let Some((minimum, _)) = bounds(game, source) else { return; };
    let physical = (minimum as i32).wrapping_add(random).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    let (element, soul, critical_chance) = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        let combat = player.combat_properties();
        ((combat.add_element_attack as i32).max(0), i32::from(combat.add_soul_attack), combat.cch)
    } else {
        let Some(monster) = game.find_region(source.0).and_then(|region| region.base().find_monster_by_id(source.1.id)) else { return; };
        let Some(properties) = monster_properties(game, source) else { return; };
        (0, i32::from(monster.soul_attack(properties)), 0)
    };
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    // Даже CMonster::GetCCH == 0 не устраняет второй RNG Calculate.
    if game.skill_random_below(100) < i32::from(critical_chance) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

fn attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let master = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        master(player)
    } else {
        MasterInfo { master_type: source.1.object_type, master_id: source.1.id, ..MasterInfo::default() }
    };
    let mut attack = AttackInformation::for_master(master);
    calculate_attack(game, instance, source, target, &mut attack);
    // NULL таблица Calculate оставляет пустой UNKNOWN/1, но не отменяет +15C.
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

fn install<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    state: KnockOutState, runtime: &mut Runtime,
) {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let previous = shape.find_state_position(|state| state.state_id() == KNOCK_OUT_SKILL_ID);
    let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let _ = begin_primary_blind_state_at(
        game, target.0, target.1, Some(source), Some(target), state, placement,
        &mut || runtime.now_milliseconds(),
    );
}

fn hit(game: &CGame, source: (i32, ShapeIdentity)) -> Option<u16> {
    if source.1.object_type == PLAYER_TYPE {
        Some(game.find_player(source.1.id)?.combat_properties().hit)
    } else {
        let monster = game.find_region(source.0)?.base().find_monster_by_id(source.1.id)?;
        Some(monster.hit(monster_properties(game, source)?))
    }
}

fn dodge(game: &CGame, target: (i32, ShapeIdentity)) -> Option<u16> {
    match target.1.object_type {
        PLAYER_TYPE => Some(game.find_player(target.1.id)?.combat_properties().dodge),
        MONSTER_TYPE => {
            let monster = game.find_region(target.0)?.base().find_monster_by_id(target.1.id)?;
            Some(monster.dodge(monster_properties(game, target)?))
        }
        500 | 1_100 | 1_200 => Some(0),
        _ => None,
    }
}

/// Some(0/1) — буквальный End навыка; None оставляет его до следующего AI.
fn execute_stage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> Option<i32> {
    let skill = game.registered_skill(instance)?;
    if skill.lifecycle().is_ended()
        || stage(game, instance).is_none_or(|stage| stage == SkillStage::Idle)
    { return None; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return Some(0); };
    let Some(source) = participant(game, skill.lifecycle().user()) else { return Some(0); };
    let Some(target) = sufferer(game, instance) else { return Some(0); };
    if game.base_magic_target_dead(target.0, target.1) {
        game.update_registered_skill_visual(instance, 10);
        return Some(0);
    }
    if stage(game, instance) == Some(SkillStage::Begin) {
        game.registered_skill_mut(instance)?.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
        let target_shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
        let (target_x, target_y) = (
            target_shape.get_tile_x().unwrap_or(i32::MIN),
            target_shape.get_tile_y().unwrap_or(i32::MIN),
        );
        let source_shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        let (source_x, source_y) = (
            source_shape.get_tile_x().unwrap_or(i32::MIN),
            source_shape.get_tile_y().unwrap_or(i32::MIN),
        );
        resolve_state_move_shape_mut(game, source.0, source.1)?.shape_mut()
            .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        game.update_registered_skill_visual(instance, 0);
        advance(game, instance, SkillStage::Begin, SkillStage::Check);
    }
    let delay = properties.query_property(DELAY);
    let started = game.registered_skill(instance)?.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return None; }
    resolve_state_move_shape_mut(game, source.0, source.1)?.set_moveable(true);
    let path = game.skill_target_path(game.registered_skill(instance)?.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE).wrapping_add(1) < path.len() as u32
    {
        game.update_registered_skill_visual(instance, 11);
        return Some(0);
    }
    // Gameplay S захвачена в начале AI; только путь и visual разрешают S заново.
    game.update_registered_skill_visual(instance, 1);
    let target_dodge = dodge(game, target)?;
    let source_hit = hit(game, source)?;
    let target_level = game.move_shape_level(target.0, target.1)?;
    let source_level = game.move_shape_level(source.0, source.1)?;
    let (base, magnify, level_rate) = game.globe_setup().base_attack_hit_formula();
    let chance = knock_out_hit_chance(source_hit, target_dodge, source_level, target_level, base, magnify, level_rate);
    if chance <= game.skill_random_below(100) { return Some(0); }
    advance(game, instance, SkillStage::Check, SkillStage::Calculate);
    advance(game, instance, SkillStage::Calculate, SkillStage::Attack);
    attack(game, instance, source, target, runtime);
    if resolve_state_move_shape(game, target.0, target.1)
        .is_some_and(|shape| !shape.has_state_by_skill_id(CURE_SKILL_ID))
    {
        let state = KnockOutState::new(0, properties.query_property(PERSIST));
        install(game, source, target, state, runtime);
    }
    advance(game, instance, SkillStage::Attack, SkillStage::Apply);
    Some(1)
}

fn end<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, argument: i32,
    termination: SkillTermination, runtime: &mut Runtime,
) {
    // End не имеет IsEnded-gate: внешний AI завершает тот же живой экземпляр
    // и после вложенного End. Поколение ключа защищает только его замену.
    let _ = game.end_registered_instance(instance, argument, termination, runtime);
}

fn result(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) fn execute_player_knock_out<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != KNOCK_OUT_SKILL_ID { return result(QueuedSkillExecutionState::Rejected); }
    let Some(instance) = game.registered_player_skill(player_id, KNOCK_OUT_SKILL_ID) else {
        return result(QueuedSkillExecutionState::Rejected);
    };
    game.with_published_player_ai(player_id, ai, |game| {
        if game.registered_skill(instance).and_then(MoveShapeSkill::player_dispatch).is_none() {
            game.replace_player_skill_visual_effect(
                player_id, KNOCK_OUT_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::KnockOut, 1),
            );
            if !check_cast(game, instance, runtime) {
                game.update_registered_skill_visual(instance, 2);
                end(game, instance, 0, SkillTermination::Rejected, runtime);
                return result(QueuedSkillExecutionState::Rejected);
            }
            let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
                return result(QueuedSkillExecutionState::Rejected);
            };
            game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
            return result(QueuedSkillExecutionState::Begun);
        }
        if game.registered_skill(instance).and_then(MoveShapeSkill::player_dispatch) != Some(dispatch) {
            return result(QueuedSkillExecutionState::Rejected);
        }
        match execute_stage(game, instance, runtime) {
            Some(argument) => {
                let (termination, outcome) = if argument == 0 {
                    (SkillTermination::Rejected, QueuedSkillExecutionState::Rejected)
                } else {
                    (SkillTermination::Completed, QueuedSkillExecutionState::Completed)
                };
                end(game, instance, argument, termination, runtime);
                result(outcome)
            }
            None => result(QueuedSkillExecutionState::Pending),
        }
    })
}

fn finish_player<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, argument: i32,
    termination: SkillTermination, runtime: &mut Runtime,
) -> bool {
    let Some(instance) = game.registered_player_skill(player_id, KNOCK_OUT_SKILL_ID) else { return false; };
    let Some(dispatch) = game.registered_skill(instance).and_then(MoveShapeSkill::player_dispatch) else { return false; };
    game.with_published_player_ai(player_id, ai, |game| end(game, instance, argument, termination, runtime));
    game.finish_registered_player_command(Some(instance), ai, dispatch, termination)
}

pub(crate) fn complete_player_knock_out<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> bool {
    finish_player(game, player_id, ai, 1, SkillTermination::Completed, runtime)
}

pub(crate) fn cancel_player_knock_out<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player(game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime)
}

pub(crate) fn execute_owned_monster_knock_out<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    let Some(region) = owner.as_mut() else { return false; };
    let Some(monster) = region.base().find_monster_by_id(monster_id) else { return false; };
    let source = (region.region_id(), monster.move_shape().shape().identity());
    let beginning = monster.current_active_attack_cast(game.skill_factory()).is_none();
    if beginning {
        let target_object = resolve_owned_skill_begin_object(game, region.base(), target);
        let started = runtime.now_milliseconds();
        let Some(monster) = region.base_mut().find_monster_by_id_mut(monster_id) else { return false; };
        if !monster.prepare_base_attack_cast(target, KNOCK_OUT_SKILL_ID, skill_level, started, target_object, game.skill_factory()) {
            return false;
        }
        monster.move_shape_mut().replace_skill_visual_effect(
            KNOCK_OUT_SKILL_ID, game.skill_factory(), SkillVisualEffect::new(SkillVisualEffectKind::KnockOut, 1),
        );
    }
    let outcome = game.with_published_region(owner, |game| {
        let Some(instance) = game.registered_move_shape_skill(source.0, source.1, KNOCK_OUT_SKILL_ID) else {
            return MonsterSkillCallOutcome::NotHandled;
        };
        if beginning {
            if let Some(kernel) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::monster_kernel_mut) {
                kernel.clear_phase_for_end();
            }
            if !check_cast(game, instance, runtime) {
                game.update_registered_skill_visual(instance, 2);
                end(game, instance, 0, SkillTermination::Rejected, runtime);
                return MonsterSkillCallOutcome::BeginRejected;
            }
            advance(game, instance, SkillStage::Idle, SkillStage::Begin);
            if let Some(monster) = game.find_region_mut(source.0).and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id)) {
                monster.enqueue_base_attack_cast(runtime.now_milliseconds());
            }
            return MonsterSkillCallOutcome::Handled;
        }
        if let Some(argument) = execute_stage(game, instance, runtime) {
            let termination = if argument == 0 { SkillTermination::Rejected } else { SkillTermination::Completed };
            end(game, instance, argument, termination, runtime);
        }
        MonsterSkillCallOutcome::Handled
    }).unwrap_or(MonsterSkillCallOutcome::Handled);
    let Some(region) = owner.as_mut() else { return true; };
    finish_monster_skill_call(game, region.base_mut(), monster_id, outcome, runtime)
}
