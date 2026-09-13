//! Общее исполнение областных призывов Weak, PoisonFog, SnowStorm, YinYang и GodThunder.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/weak.cpp,
//! poisonfog.cpp, snowstorm.cpp, yinyang.cpp/yinyang2.cpp и
//! godthunder.cpp/godthunder2.cpp. Все формы Begin сохраняют базовую цель,
//! создают visual loop1 и проверяют исходного U; отказ даёт End0 без extra2.
//! Check использует абсолютный reuse и свежий путь. Weak не проверяет BLOCK2;
//! PoisonFog требует арбалет. Player MP0 — тихий отказ, signed-разность допускает
//! Move0; остальные CMoveShape проходят без запрета движения.
//!
//! PoisonFog превращает найденную S в точку ещё до таблицы/reuse в Check.
//! Его AI читает только U и базовую точку. Weak в каждом AI проверяет смерть S,
//! затем сохраняет её X/Y и очищает identity S; SnowStorm, YinYang и GodThunder
//! оставляют S неизменной. Все они удерживают начальные координаты через callbacks.
//! Одна таблица AI переживает MP→OnChangeStates→CAN→направление→visual0.
//! PoisonFog повторно проверяет оружие после расхода MP, без возврата расхода.
//! SnowStorm отправляет только visual-ошибки, без GS-текстов и лишнего MP-query.
//!
//! Задержка — unsigned start+delay; после visual0 нет нового active-gate в том же
//! AI. PoisonFog перед visual1 разрешает captured U Move1 и берёт базовую точку
//! после callback; остальные владельцы передают сохранённые координаты.
//! Любая попытка Summon завершается End1 независимо от её результата.
//! Общий End сбрасывает фазу, разрешает свежий U Move1 и сохраняет actual argument.
//! RegisteredSkill/SlotMap и общий kernel хранят единственное исполнение;
//! независимые формулы, конструкторы и регистрация областей остаются у владельцев.
//! Общий префикс YinYang/GodThunder сохраняет Master(country0) и Player EM,
//! затем читает свежую таблицу. Unsigned usage20015 умножается на расширенный
//! literal0.01f и signed EM; FISTP с усечением выполняется до CCH и остальных
//! запросов конструктора, без промежуточного округления к f32.

use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::godthunder::{GOD_THUNDER_SKILL_ID, summon_god_thunder};
use super::godthunder2::GOD_THUNDER_2_SKILL_ID;
use super::playercast::execute_registered_player_cast;
use super::poisonfog::{POISON_FOG_SKILL_ID, summon_poison_fog};
use super::rangedweaponcast::{
    CastManaRule, CastPathBlock, RangedWeaponKind, check_cast_mana,
    check_cast_mana_without_text, check_ranged_weapon_and_mana, check_skill_path,
    prepare_ranged_weapon_player, spend_cast_mana, spend_cast_mana_without_text, terminal,
};
use super::snowstorm::{SNOW_STORM_SKILL_ID, summon_snow_storm};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::weak::{WEAK_SKILL_ID, summon_weak};
use super::weaponattack::source_master;
use super::yinyang::{YIN_YANG_SKILL_ID, summon_yin_yang};
use super::yinyang2::YIN_YANG_2_SKILL_ID;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const DELAY: u32 = 10_001;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

pub(crate) const fn is_zonal_cast_skill(id: u32) -> bool {
    matches!(id, WEAK_SKILL_ID | POISON_FOG_SKILL_ID | SNOW_STORM_SKILL_ID
        | YIN_YANG_SKILL_ID | YIN_YANG_2_SKILL_ID | GOD_THUNDER_SKILL_ID | GOD_THUNDER_2_SKILL_ID)
}

pub(super) fn prepare_element_summon(
    game: &CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
) -> Option<(MasterInfo, CSkillBaseProperties, i32)> {
    let mut master = source_master(game, source)?;
    master.master_country_id = 0;
    let element = if source.1.object_type == 400 {
        game.find_player(source.1.id)?.combat_properties().element_modify
    } else { 0 };
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let modifier = properties.query_property(20_015);
    let scaled = truncate_original(f64::from(modifier) * f64::from(0.01_f32) * f64::from(element));
    Some((master, properties, scaled))
}

fn resolved_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn check_zonal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return false; };
    let source = (user.shape().get_region_id(), user.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let id = skill.id();
    if id == POISON_FOG_SKILL_ID {
        let destination = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ));
        if let Some(destination) = destination
            && let Some(skill) = game.registered_skill_mut(instance)
        { skill.lifecycle_mut().set_point_target(destination); }
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(id, skill.level()).cloned() else { return false; };
    let player = (source.1.object_type == 400).then_some(source.1.id);
    let text_player = player.filter(|_| id != SNOW_STORM_SKILL_ID);
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = text_player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    let block = if id == WEAK_SKILL_ID { CastPathBlock::Ignore } else { CastPathBlock::Generic };
    if !check_skill_path(game, instance, &properties, &path, text_player, block) { return false; }
    match id {
        POISON_FOG_SKILL_ID => check_ranged_weapon_and_mana(
            game, instance, source, &properties, RangedWeaponKind::Crossbow, CastManaRule::RequireCost,
        ),
        SNOW_STORM_SKILL_ID => check_cast_mana_without_text(game, instance, source, &properties),
        _ => check_cast_mana(game, instance, source, &properties),
    }
}

fn run_zonal_cast_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let id = skill.id();
    let Some(properties) = game.skill_base_properties(id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source = resolved_user(game, skill);
    let destination = if id == POISON_FOG_SKILL_ID {
        skill.lifecycle().destination()
    } else {
        let target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
        if let Some(target) = target {
            let target_identity = (target.shape().get_region_id(), target.shape().identity());
            if game.move_shape_health(target_identity.0, target_identity.1) == Some(0) {
                game.update_registered_skill_visual(instance, 10);
                if id != SNOW_STORM_SKILL_ID
                    && let Some((_, user)) = source.filter(|(_, user)| user.object_type == 400)
                { game.send_skill_system_info(user.id, b"GS0285"); }
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let destination = (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            );
            if id == WEAK_SKILL_ID {
                let point = (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                );
                if let Some(skill) = game.registered_skill_mut(instance) {
                    skill.lifecycle_mut().set_point_target(point);
                }
            }
            destination
        } else { skill.lifecycle().destination() }
    };
    let Some(source) = source else { return terminal(QueuedSkillExecutionState::Rejected); };
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if stage == SkillStage::Begin {
        let paid = match id {
            POISON_FOG_SKILL_ID => prepare_ranged_weapon_player(
                game, instance, player, &properties, RangedWeaponKind::Crossbow,
            ),
            SNOW_STORM_SKILL_ID => spend_cast_mana_without_text(game, instance, player, &properties),
            _ => spend_cast_mana(game, instance, player, &properties),
        };
        if !paid { return terminal(QueuedSkillExecutionState::Rejected); }
        let can_break = properties.query_property(CAN_BREAK);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let destination = if id == POISON_FOG_SKILL_ID {
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            skill.lifecycle().destination()
        } else { destination };
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(DELAY);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if id == POISON_FOG_SKILL_ID
        && let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1)
    { user.set_moveable(true); }
    game.update_registered_skill_visual(instance, 1);
    match id {
        WEAK_SKILL_ID => summon_weak(game, instance, source, destination, runtime),
        POISON_FOG_SKILL_ID => {
            if let Some(destination) = game.registered_skill(instance).map(|skill| skill.lifecycle().destination()) {
                summon_poison_fog(game, instance, source, destination, runtime);
            }
        }
        SNOW_STORM_SKILL_ID => summon_snow_storm(game, instance, source, destination, runtime),
        YIN_YANG_SKILL_ID | YIN_YANG_2_SKILL_ID => summon_yin_yang(game, instance, source, destination, runtime),
        GOD_THUNDER_SKILL_ID | GOD_THUNDER_2_SKILL_ID => summon_god_thunder(game, instance, source, destination, runtime),
        _ => {}
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_zonal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ZonalCast,
        |game, instance, _, runtime| check_zonal_cast(game, instance, original_user, runtime),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_zonal_cast_ai,
    )
}

struct ZonalCastSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for ZonalCastSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::ZonalCast;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let user = game.registered_skill(instance).and_then(|skill| resolved_user(game, skill));
        check_zonal_cast(game, instance, user, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_zonal_cast_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_zonal_cast<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<ZonalCastSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

pub(crate) fn publish_zonal_cast_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !is_zonal_cast_skill(skill.id()) || skill.visual_effect().is_none_or(|effect|
        effect.kind() != SkillVisualEffectKind::ZonalCast || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) || (skill.id() == POISON_FOG_SKILL_ID && mode == 14) {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let destination = match mode {
        0 => None,
        1 => Some(if skill.id() == POISON_FOG_SKILL_ID { skill.lifecycle().destination() } else {
            resolve_skill_sufferer(game, skill.lifecycle())
                .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
                .map_or_else(|| skill.lifecycle().destination(), |target| (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                ))
        }),
        _ => return,
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if mode == 0 { 1 } else { 2 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some((x, y)) = destination {
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    { let _ = game.send_game_shape_around(region.base(), source, None, &message); }
}
