//! BFBaseAttack (0x224), gameserver.exe/GameServer.pdb,
//! appserver/skills/battlefairybasemagic.cpp.
//!
//! Общий зарегистрированный вход владеет Begin, visual и End; здесь остаются
//! Check, AI и создание самостоятельного снаряда. Check использует исходную
//! объектную S, а путь разрешает текущую базу. MP и WarSoul здесь не проверяются:
//! предмет требуется только позднему Summon. Конфликты состояний выбираются
//! в порядке массива, max0 не ограничивает дальность.
//!
//! Первый AI записывает CAN, проверяет смерть/self и публикует visual0.
//! Абсолютный wrapping-срок проверяется в том же проходе. Повторный GetS перед
//! выстрелом проверяет доступность цели, но время полёта и Summon используют
//! U/S начала AI. Игрок временно проходит реальный SetTileXY в POINT боевого
//! духа, затем возвращается в центр исходной клетки. Оба вызова сохраняют
//! block/area-поля и отмену захвата; дробная исходная позиция не восстанавливается.
//! ShapeView используется лишь как краткоживущая проекция текущей геометрии.
//!
//! Summon требует непустой путь, очищает S до проверки figure2 и только затем
//! читает Master/WarSoul и параметры конструктора. Clock предшествует ID,
//! SetCenter — позднему допуску региона. Country в Master остаётся нулевым.
//! Любая попытка Summon завершается End(1), независимо от создания снаряда;
//! ошибки AI дают End(0). Собственный attack-time End не обнуляет.

use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::battlefairybasemagicphalanx::CBattleFairyBaseMagicPhalanx;
use super::battlefairyskill::{
    check_battle_fairy_target_states, execute_registered_battle_fairy_skill,
};
use super::kernel::{BattleFairyExecution, SkillExecutionKernel, SkillStage, skill_is_restored};
use super::thunder::{fail_battle_fairy_summon, master_info, terminal};
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_BF_SPRITE;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::{ShapeFigure, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const BATTLE_FAIRY_BASE_MAGIC_SKILL_ID: u32 = 0x224;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyBaseMagicExecutionState {
    kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    attack_time: u32,
}

impl BattleFairyBaseMagicExecutionState {
    pub(crate) const fn begin(dispatch: BattleFairySkillDispatch, started_at_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), attack_time: 0 }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<BattleFairySkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn attack_time(&self) -> u32 { self.attack_time }
}

pub(crate) fn execute_battle_fairy_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != BATTLE_FAIRY_BASE_MAGIC_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, None,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime),
        |dispatch, started| BattleFairyExecution::BaseMagic(
            BattleFairyBaseMagicExecutionState::begin(dispatch, started),
        ),
        run_ai,
    )
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(target) = begin_target else { return false; };
    if target.1.object_type == 400 && target.1.id == player.player_id() {
        fail_battle_fairy_summon(game, instance, player_id, 10, b"ZHGS0045");
        return false;
    }
    if !check_battle_fairy_target_states(game, player_id, target) { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let last_used = skill.last_used_ms();
    if !skill_is_restored(last_used, reuse, runtime.now_milliseconds()) {
        fail_battle_fairy_summon(game, instance, player_id, 13, b"ZHGS0048");
        return false;
    }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && path.len() > properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize
    {
        fail_battle_fairy_summon(game, instance, player_id, 11, b"ZHGS0049");
        return false;
    }
    true
}

fn shape_view(game: &CGame, target: (i32, ShapeIdentity)) -> Option<ShapeView> {
    let shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
    let identity = shape.identity();
    let figure = match identity.object_type {
        400 => game.find_player(identity.id)?.figure(),
        500 => ShapeFigure::default(),
        600 => {
            let monster = game.find_region(shape.get_region_id())?.base().find_monster_by_id(identity.id)?;
            let properties = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            CMonster::figure(properties)
        }
        1100 | 1200 => game.find_region(shape.get_region_id())?.stationary_build(identity)?.shape_view().figure,
        _ => return None,
    };
    Some(ShapeView {
        identity,
        tile_x: shape.get_tile_x().unwrap_or(i32::MIN),
        tile_y: shape.get_tile_y().unwrap_or(i32::MIN),
        pos_x_bits: shape.get_pos_x().to_bits(),
        pos_y_bits: shape.get_pos_y().to_bits(),
        figure,
    })
}

fn target_failure(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    text: &[u8], second_visual: bool,
) -> QueuedSkillExecutionOutcome {
    game.update_registered_skill_visual(instance, 10);
    if source.1.object_type == 400 { game.send_skill_system_info(source.1.id, text); }
    if second_visual { game.update_registered_skill_visual(instance, 10); }
    terminal(QueuedSkillExecutionState::Rejected)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source = skill.lifecycle().user();
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    if resolve_state_move_shape(game, source.0, source.1).is_none() || target.is_none() {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let target = target.expect("наличие S проверено до первой фазы");
    if skill.execution_stage() == Some(SkillStage::Begin) {
        let can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_be_breaked != 0);
        }
        if game.base_magic_target_dead(target.0, target.1) {
            return target_failure(game, instance, source, b"ZHGS0050", false);
        }
        if source.1.object_type == target.1.object_type && source.1.id == target.1.id {
            game.update_registered_skill_visual(instance, 10);
            return target_failure(game, instance, source, b"ZHGS0045", false);
        }
        game.update_registered_skill_visual(instance, 0);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let started = skill.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(late_target) = resolve_skill_sufferer(game, skill.lifecycle()) else {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(late_target.0, late_target.1) {
        return target_failure(game, instance, source, b"ZHGS0050", true);
    }

    let restore = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let original = (
            player.shape().get_tile_x().unwrap_or(i32::MIN),
            player.shape().get_tile_y().unwrap_or(i32::MIN),
        );
        let point = player.war_soul_point();
        let _ = game.set_player_tile_position(source.1.id, point.x, point.y);
        Some(original)
    } else { None };
    let (Some(fire_source), Some(target_view)) = (shape_view(game, source), shape_view(game, target)) else {
        if let Some((x, y)) = restore {
            let _ = game.set_player_tile_position(source.1.id, x, y);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let distance = fire_source.real_distance(Some(target_view));
    let speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let attack_time = (distance as u32).wrapping_mul(speed);
    let stored = if let Some(BattleFairyExecution::BaseMagic(state)) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.battle_fairy_execution_state_mut())
    {
        state.attack_time = attack_time;
        true
    } else { false };
    if let Some((x, y)) = restore {
        let _ = game.set_player_tile_position(source.1.id, x, y);
    }
    if !stored { return terminal(QueuedSkillExecutionState::Rejected); }
    game.update_registered_skill_visual(instance, 1);
    summon(game, instance, source, target, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, source.0, source.1).is_none()
        || resolve_state_move_shape(game, target.0, target.1).is_none() { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let (Some(source_view), Some(target_view)) = (shape_view(game, source), shape_view(game, target)) else { return; };
    let distance = source_view.real_distance(Some(target_view)) as u32;
    let path = game.skill_target_path_with_length(skill.lifecycle(), distance);
    let Some(&(tile_x, tile_y, _)) = path.first() else { return; };
    if let Some(skill) = game.registered_skill_mut(instance) {
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
    }
    if path.iter().any(|cell| cell.2 == 2) { return; }
    let mut master = MasterInfo {
        master_type: source.1.object_type, master_id: source.1.id, ..MasterInfo::default()
    };
    if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        let Some(goods) = player.war_soul_goods(game.goods_factory()) else { return; };
        master = master_info(player);
        master.master_country_id = 0;
        let _ = goods.addon_property_value(game.goods_factory(), GAP_BF_SPRITE, 1);
    }
    let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(BattleFairyExecution::BaseMagic(state)) = skill.battle_fairy_execution_state() else { return; };
    let attack_time = state.attack_time();
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CBattleFairyBaseMagicPhalanx::new(
        id, master, started, lifetime, level, minimum, maximum, element_modifier, attack_time, target.1,
    );
    phalanx.set_center(tile_x, tile_y);
    let Some(region) = resolve_state_move_shape(game, source.0, source.1)
        .map(|source| source.shape())
        .filter(|shape| shape.is_assigned_to_server_region())
        .map(|shape| shape.get_region_id())
        .filter(|region| game.find_region(*region).is_some())
    else { return; };
    let _ = game.add_battle_fairy_base_magic_phalanx(
        region, phalanx, started, runtime,
    );
}
