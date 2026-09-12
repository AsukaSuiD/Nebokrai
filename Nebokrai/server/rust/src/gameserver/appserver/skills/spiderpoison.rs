//! Ядовитый удар CSpiderPoison (0x191), gameserver.exe + GameServer.pdb,
//! исходный owner appserver/skills/spiderpoison.cpp; payload — spiderpoisonstate.cpp.
//! Общая обвязка stateskill обслуживает Begin/End игрока и монстра.
//! Здесь остаются собственные CheckCast, AI, прямой удар и позднее наложение яда;
//! состояния принадлежат аренам получателей, а не исполнению навыка.

use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::fightdefense::truncate_original;
use super::kernel::{SkillStage, SkillTermination, skill_is_restored};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoisonstate::{SpiderPoisonState, begin_primary_spider_poison_state};
use super::stateskill::{
    RegisteredStateSkill, end_state_skill, execute_owned_state_skill, execute_player_state_skill,
    finish_player_state_skill, publish_state_skill_visual, state_skill_outcome,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

pub(crate) const SPIDER_POISON_SKILL_ID: u32 = 0x191;
const PLAYER_TYPE: i32 = 400;
const CURE_SKILL_ID: u32 = 0x131;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;
const FREQUENCY: u32 = 6_001;
const HP_LOSS: u32 = 20_010;
const PROBABILITY: u32 = 40_001;

pub(crate) const fn is_player_spider_poison_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == SPIDER_POISON_SKILL_ID
}

fn participant(game: &CGame, value: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn publish_spider_poison_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<SpiderPoison>(game, skill, mode);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    begin_target: super::stateskill::StateSkillBeginTarget, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let lifecycle = *skill.lifecycle();
    let Some(user) = participant(game, lifecycle.user()) else { return false; };
    if begin_target.resolve(game, skill, false).is_none() { return false; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return false; };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        return false;
    }
    let path = game.skill_target_path(&lifecycle);
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
    {
        game.update_registered_skill_visual(instance, 11);
        return false;
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, user.0, user.1) {
        shape.set_moveable(false);
    }
    true
}

fn player_master(player: &CPlayer) -> MasterInfo {
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

fn master(game: &CGame, source: (i32, ShapeIdentity)) -> Option<MasterInfo> {
    if source.1.object_type == PLAYER_TYPE {
        Some(player_master(game.find_player(source.1.id)?))
    } else {
        Some(MasterInfo { master_type: source.1.object_type, master_id: source.1.id, ..MasterInfo::default() })
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
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    if game.skill_base_properties(skill.id(), skill.level()).is_none() { return; }
    attack.damage_modifier = 0;
    attack.damage_factor = 1.0;
    attack.hit_modifier = 0;
    let Some((_, maximum)) = bounds(game, source) else { return; };
    let Some((minimum, _)) = bounds(game, source) else { return; };
    let span = (maximum as i32).wrapping_sub(minimum as i32).unsigned_abs().wrapping_add(1) as i32;
    let random = game.skill_random_below(span);
    let Some((minimum, _)) = bounds(game, source) else { return; };
    attack.damages.push(AttackPower {
        kind: AttackPowerType::Physical, hp_damage: (minimum as i32).wrapping_add(random).max(0), mp_damage: 0,
    });
    let element = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().add_element_attack as i32
    } else {
        // GetAddElementAtk монстра, не его независимый GetElementModify.
        0
    };
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element.max(0), mp_damage: 0 });
    let soul = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().add_soul_attack
    } else {
        let Some(monster) = game.find_region(source.0).and_then(|region| region.base().find_monster_by_id(source.1.id)) else { return; };
        let Some(properties) = monster_properties(game, source) else { return; };
        monster.soul_attack(properties)
    };
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(soul), mp_damage: 0 });
    let critical_chance = if source.1.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().cch
    } else { 0 };
    // Нулевой CMonster::GetCCH не устраняет второй RNG Calculate.
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
    let Some(master) = master(game, source) else { return; };
    // NULL таблица Calculate не отменяет приём пустого UNKNOWN/1.
    let mut attack = AttackInformation::for_master(master);
    calculate_attack(game, instance, source, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

fn apply_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if shape.has_state_by_skill_id(CURE_SKILL_ID) { return; }
    // DWORD probability сравнивается знаково; равенство сохраняет успех.
    let probability = properties.query_property(PROBABILITY);
    if game.skill_random_below(100) > probability as i32 { return; }
    let Some(master) = master(game, source) else { return; };
    // В отличие от прямого удара, поздний MasterInfo не получает country.
    let master = MasterInfo { master_country_id: 0, ..master };
    let hp_loss = properties.query_property(HP_LOSS);
    let frequency = properties.query_property(FREQUENCY);
    let keep = properties.query_property(PERSIST);
    // Ctor предшествует старому End; после callback очищается свежий остаток
    // того же слота, затем полный Begin регистрируется в прежней позиции.
    let state = SpiderPoisonState::new(master, keep, frequency, hp_loss);
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let previous = shape.find_state_position(|state| state.state_id() == SPIDER_POISON_SKILL_ID);
    let placement = previous.and_then(|(_, key)| shape.applied_state_replacement_location(key));
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let _ = begin_primary_spider_poison_state(
        game, target.0, target.1, Some(source), Some(target), state, placement,
        &mut || runtime.now_milliseconds(),
    );
}

/// Some(0/1) — буквальный End; None оставляет экземпляр следующему AI.
fn execute_stage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> Option<i32> {
    let skill = game.registered_skill(instance)?;
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) { return None; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return Some(0); };
    let Some(source) = participant(game, skill.lifecycle().user()) else { return Some(0); };
    let Some(target) = resolve_skill_sufferer(game, skill.lifecycle()).and_then(|value| participant(game, value)) else { return Some(0); };
    if game.base_magic_target_dead(target.0, target.1) {
        game.update_registered_skill_visual(instance, 10);
        return Some(0);
    }
    if game.registered_skill(instance)?.execution_stage() == Some(SkillStage::Begin) {
        game.registered_skill_mut(instance)?.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
        let target_shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
        let target_y = target_shape.get_tile_y().unwrap_or(i32::MIN);
        let target_x = target_shape.get_tile_x().unwrap_or(i32::MIN);
        let source_shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
        let source_y = source_shape.get_tile_y().unwrap_or(i32::MIN);
        let source_x = source_shape.get_tile_x().unwrap_or(i32::MIN);
        resolve_state_move_shape_mut(game, source.0, source.1)?.shape_mut()
            .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(DELAY);
    let started = game.registered_skill(instance)?.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return None; }
    resolve_state_move_shape_mut(game, source.0, source.1)?.set_moveable(true);
    let path = game.skill_target_path(game.registered_skill(instance)?.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
    {
        game.update_registered_skill_visual(instance, 11);
        return Some(0);
    }
    // Путь и visual разрешают S заново; gameplay использует S начала AI.
    game.update_registered_skill_visual(instance, 1);
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
        let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
    }
    attack(game, instance, source, target, runtime);
    // После удара нет IsDied/IsAttackAble: только живой Cure и бросок яда.
    apply_poison(game, source, target, &properties, runtime);
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Attack, SkillStage::Apply);
    }
    Some(1)
}

struct SpiderPoison;

impl RegisteredStateSkill for SpiderPoison {
    const ID: u32 = SPIDER_POISON_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::SpiderPoison;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 8, 10, 11, 13, 14, 15];

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, begin_target: super::stateskill::StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        check_cast(game, instance, begin_target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        match execute_stage(game, instance, runtime) {
            Some(argument) => end_state_skill(game, instance, argument, runtime),
            None => state_skill_outcome(QueuedSkillExecutionState::Pending),
        }
    }
}

pub(crate) fn execute_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<SpiderPoison, Runtime>(game, player_id, dispatch, ai, runtime)
}

pub(crate) fn cancel_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<SpiderPoison, Runtime>(
        game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_owned_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<SpiderPoison, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
