//! Региональные стрелы CHeartLessArrow2/3 и общий visual семейства CA/E5/E6.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/heartlessarrow.cpp,
//! heartlessarrow2.cpp и heartlessarrow3.cpp. Общий playercast хранит одно
//! зарегистрированное исполнение; исходные U/S сохраняются до базового Begin.
//! Check читает таблицу до NULL S, затем reuse, свежий путь с MAX0 без ограничения,
//! BLOCK2 и оружие. Self здесь допускается; MP0 тихо отклоняет игрока, а другой
//! CMoveShape проходит без Move0. Отказ сразу вызывает End0.
//!
//! Каждый AI удерживает свою таблицу и полные U/S: смерть S проверяется раньше
//! указательного self. Первый AI: MP→OnChangeStates→арбалет→CAN→S.Y/X→U.Y/X,
//! направление и visual0. Выпуск использует абсолютный unsigned start+delay,
//! свежий путь и DWORD времени полёта; visual1 предшествует prepared/Summon.
//! E5 возвращает движение перед повторным путём, E6 — только в End. Время
//! полёта относится к пакету, а не задерживает создание формы и End1.
//! End очищает фазу и время полёта до свежего U/Move1; CAN не сбрасывается.
//!
//! Summon получает новую таблицу после visual, очищает S, сохраняет MasterInfo
//! с country0 и читает EM, CCH WORD, factor, уровень, lifetime. Базовые часы
//! предшествуют выделению ID. Региональный адаптер сохраняет SetTileXY→GetShapes
//! и пустой callback существующих E5, затем Add и пакет даже при отказе Add.
//! Нет замены существующей формы или дополнительного cooldown. Общий visual
//! сохраняет базовый хвост даже при NULL S режима 1; нечисловые координаты
//! передаются с native FISTP sentinel i32::MIN.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME,
};
use super::heartlessarrow::HeartlessArrowExecutionState;
use super::heartlessarrow3::HEARTLESS_ARROW_3_SKILL_ID;
use super::heartlessarrowphalanx2::CHeartlessArrowPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastManaRule, CastPathBlock, RangedWeaponKind, check_ranged_weapon_and_mana,
    check_skill_path, prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use super::skillfactory::SkillOwner;
use super::weaponattack::{SourceProperty, source_master, source_property};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const HEARTLESS_ARROW_2_SKILL_ID: u32 = 0xe5;
const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeartlessArrowAreaExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    missile_flying_time_ms: u32,
}

impl HeartlessArrowAreaExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), missile_flying_time_ms: 0 }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.missile_flying_time_ms = 0;
        true
    }
}

pub(super) fn check_heartless_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, weapon: RangedWeaponKind, runtime: &mut Runtime,
) -> bool {
    let Some(source) = original_user.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
    else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let target = original_target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let Some(target) = target else {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0286"); }
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        ranged_weapon_failure(game, instance, player, 13, weapon);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path(game, instance, &properties, &path, player,
        CastPathBlock::Named { target, message: b"GS0291" })
    { return false; }
    check_ranged_weapon_and_mana(game, instance, source, &properties, weapon, CastManaRule::RequireCost)
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    x: i32, y: i32, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    if game.find_region(region).is_none() { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let destination = skill.lifecycle().destination();
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    skill.lifecycle_mut().set_point_target(destination);
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let Some(cch) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
    let factor = properties.query_property(TARGET_DAMAGE_FACTOR) as i32;
    let Some(skill) = game.registered_skill(instance) else { return; };
    let level = skill.level();
    let skill_id = skill.id();
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CHeartlessArrowPhalanx::new(id, master, started, lifetime, skill_id, level, factor, i32::from(cch as u16));
    let _ = game.spawn_heartless_arrow_phalanx(region, phalanx, x, y, started, runtime);
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity);
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let (Some(source), Some(target)) = (source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let is_self = std::ptr::eq(source, target);
    let source = (source.shape().get_region_id(), source.shape().identity());
    let target = (target.shape().get_region_id(), target.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0285"); }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if is_self {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0286"); }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::HeartlessCrossbow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if skill_id == HEARTLESS_ARROW_2_SKILL_ID
        && let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1)
    { user.set_moveable(true); }
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path(game, instance, &properties, &path, player,
        CastPathBlock::Named { target, message: b"GS0307" })
    { return terminal(QueuedSkillExecutionState::Rejected); }
    let flight = properties.query_property(MISSILE_FLYING_TIME).wrapping_mul(path.len() as u32);
    let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<HeartlessArrowAreaExecutionState>())
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    state.missile_flying_time_ms = flight;
    game.update_registered_skill_visual(instance, 1);
    if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().mark_prepared(); }
    if let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) {
        let y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        summon(game, instance, source, x, y, runtime);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn execute_player_heartless_arrow_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target|
            original_user.and_then(|source| game.player_skill_begin_object(source.0, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::HeartlessArrow,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            check_heartless_cast(game, instance, original_user, target, RangedWeaponKind::HeartlessCrossbow, runtime)
        },
        |dispatch, started| HeartlessArrowAreaExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

pub(crate) fn publish_heartless_arrow_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CHeartLessArrow | SkillOwner::CHeartLessArrow2 | SkillOwner::CHeartLessArrow3)
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::HeartlessArrow || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if action == 2 {
        let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
        let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
        Some(target.shape())
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
        let flight = if matches!(skill.id(), HEARTLESS_ARROW_2_SKILL_ID | HEARTLESS_ARROW_3_SKILL_ID) {
            skill.player_state::<HeartlessArrowAreaExecutionState>().map(|state| state.missile_flying_time_ms)
        } else {
            skill.player_state::<HeartlessArrowExecutionState>().map(HeartlessArrowExecutionState::missile_flying_time_ms)
        };
        let Some(flight) = flight else { return; };
        message.add_ulong(flight);
    } else {
        message.add_long(source.get_direction());
    }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
