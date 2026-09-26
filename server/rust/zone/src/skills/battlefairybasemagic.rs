//! BFBaseAttack (0x224): Check/AI и Summon снаряда базовой атаки феи.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! `appserver/skills/battlefairybasemagic.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/battlefairybasemagic.rs`; тела перенесены
//! буквально порцией №6b «BF-ядро».
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
//!
//! Объявленные швы переноса (не расхождения): hub `battlefairyskill::
//! BattleFairyGame`; регистрация снаряда выполняется делегатом через callback
//! `complete_summon` (`BattleFairyBaseMagicSummon`) с прежним main-loop runtime;
//! figure-ветка проекции ShapeView — шов `battle_fairy_shape_figure`.

use crate::combat::MasterInfo;
use crate::content::goods::GAP_BF_SPRITE;
use crate::regions::ShapeIdentity;
use crate::regions::shape::ShapeView;

use super::battlefairybasemagicphalanx::{
    BATTLE_FAIRY_BASE_MAGIC_SKILL_ID, CBattleFairyBaseMagicPhalanx,
};
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairyPlayer, BattleFairySkillOutcome,
    battle_fairy_master_info, check_battle_fairy_target_states,
    execute_registered_battle_fairy_skill,
};
use super::dispatch::BattleFairySkillDispatch;
use super::execution::{BattleFairyBaseMagicExecutionState, BattleFairyExecution};
use super::lifecycle::{SkillStage, skill_is_restored};

const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;
const SKILL_USAGE_SUMMONED_SPEED: u32 = 30_002;

/// Построенный снаряд для делегата: регистрация региона выполняется прежним
/// `CGame` с его main-loop runtime (без входного сообщения — BF502 у этого
/// owner-а шлёт только сетевой путь региона).
pub struct BattleFairyBaseMagicSummon {
    pub region_id: i32,
    pub phalanx: CBattleFairyBaseMagicPhalanx,
    pub started_at_ms: u32,
}

fn fail_battle_fairy_summon<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, mode: u32, string_id: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    game.send_skill_system_info(player_id, string_id);
}

pub fn execute_battle_fairy_base_magic<Game: BattleFairyGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    mut complete_summon: impl FnMut(&mut Game, &mut Runtime, BattleFairyBaseMagicSummon),
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != BATTLE_FAIRY_BASE_MAGIC_SKILL_ID {
        return BattleFairySkillOutcome::Rejected;
    }
    execute_registered_battle_fairy_skill(
        game, player_id, instance, dispatch, runtime, None,
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, begin_target, runtime, now),
        |dispatch, started| BattleFairyExecution::BaseMagic(
            BattleFairyBaseMagicExecutionState::begin(dispatch, started),
        ),
        |game, instance, runtime| run_ai(game, instance, runtime, now, &mut complete_summon),
    )
}

fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let player_id_self = player.player_id();
    let Some(target) = begin_target else { return false; };
    if target.1.object_type == 400 && target.1.id == player_id_self {
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
    if !skill_is_restored(last_used, reuse, now(runtime)) {
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

fn shape_view<Game: BattleFairyGame>(game: &Game, target: (i32, ShapeIdentity)) -> Option<ShapeView> {
    let shape = game.resolve_state_move_shape(target.0, target.1)?.shape();
    let identity = shape.identity();
    let figure = game.battle_fairy_shape_figure(shape.get_region_id(), identity)?;
    Some(ShapeView {
        identity,
        tile_x: shape.get_tile_x().unwrap_or(i32::MIN),
        tile_y: shape.get_tile_y().unwrap_or(i32::MIN),
        pos_x_bits: shape.get_pos_x().to_bits(),
        pos_y_bits: shape.get_pos_y().to_bits(),
        figure,
    })
}

fn target_failure<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
    text: &[u8], second_visual: bool,
) -> BattleFairySkillOutcome {
    game.update_registered_skill_visual(instance, 10);
    if source.1.object_type == 400 { game.send_skill_system_info(source.1.id, text); }
    if second_visual { game.update_registered_skill_visual(instance, 10); }
    BattleFairySkillOutcome::Rejected
}

fn run_ai<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
    complete_summon: &mut dyn FnMut(&mut Game, &mut Runtime, BattleFairyBaseMagicSummon),
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let source = skill.lifecycle().user();
    let target = game.resolve_skill_sufferer(skill.lifecycle());
    if game.resolve_state_move_shape(source.0, source.1).is_none() || target.is_none() {
        game.update_registered_skill_visual(instance, 10);
        return BattleFairySkillOutcome::Rejected;
    }
    let target = target.expect("наличие S проверено до первой фазы");
    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
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
            return BattleFairySkillOutcome::Rejected;
        };
        let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else {
        return BattleFairySkillOutcome::Rejected;
    };
    let started = skill.lifecycle().started_at_ms();
    if now(runtime) < started.wrapping_add(delay) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(late_target) = game.resolve_skill_sufferer(skill.lifecycle()) else {
        game.update_registered_skill_visual(instance, 10);
        return BattleFairySkillOutcome::Rejected;
    };
    if game.base_magic_target_dead(late_target.0, late_target.1) {
        return target_failure(game, instance, source, b"ZHGS0050", true);
    }

    let restore = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else {
            return BattleFairySkillOutcome::Rejected;
        };
        let original = (
            player.shape().get_tile_x().unwrap_or(i32::MIN),
            player.shape().get_tile_y().unwrap_or(i32::MIN),
        );
        let point = player.war_soul_point();
        game.set_player_tile_position(source.1.id, point.0, point.1);
        Some(original)
    } else { None };
    let (Some(fire_source), Some(target_view)) = (shape_view(game, source), shape_view(game, target)) else {
        if let Some((x, y)) = restore {
            game.set_player_tile_position(source.1.id, x, y);
        }
        return BattleFairySkillOutcome::Rejected;
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
        game.set_player_tile_position(source.1.id, x, y);
    }
    if !stored { return BattleFairySkillOutcome::Rejected; }
    game.update_registered_skill_visual(instance, 1);
    if let Some(request) = summon(game, instance, source, target, runtime, now) {
        complete_summon(game, runtime, request);
    }
    BattleFairySkillOutcome::Completed
}

fn summon<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> Option<BattleFairyBaseMagicSummon> {
    if game.resolve_state_move_shape(source.0, source.1).is_none()
        || game.resolve_state_move_shape(target.0, target.1).is_none() { return None; }
    let skill = game.registered_skill(instance)?;
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return None; };
    let (Some(source_view), Some(target_view)) = (shape_view(game, source), shape_view(game, target)) else { return None; };
    let distance = source_view.real_distance(Some(target_view)) as u32;
    let path = game.skill_target_path_with_length(skill.lifecycle(), distance);
    let &(tile_x, tile_y, _) = path.first()?;
    if let Some(skill) = game.registered_skill_mut(instance) {
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
    }
    if path.iter().any(|cell| cell.2 == 2) { return None; }
    let mut master = MasterInfo {
        master_type: source.1.object_type, master_id: source.1.id, ..MasterInfo::default()
    };
    if source.1.object_type == 400 {
        let player = game.find_player(source.1.id)?;
        if !game.battle_fairy_war_soul_goods_present(source.1.id) { return None; }
        master = battle_fairy_master_info(player);
        master.master_country_id = 0;
        let _ = game.battle_fairy_war_soul_addon(source.1.id, GAP_BF_SPRITE);
    }
    let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let skill = game.registered_skill(instance)?;
    let Some(BattleFairyExecution::BaseMagic(state)) = skill.battle_fairy_execution_state() else { return None; };
    let attack_time = state.attack_time();
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let level = game.registered_skill(instance).map(|skill| skill.level())?;
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = now(runtime);
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CBattleFairyBaseMagicPhalanx::new(
        id, master, started, lifetime, level, minimum, maximum, element_modifier, attack_time, target.1,
    );
    phalanx.set_center(tile_x, tile_y);
    let region = game.resolve_state_move_shape(source.0, source.1)
        .map(|source| source.shape())
        .filter(|shape| shape.is_assigned_to_server_region())
        .map(|shape| shape.get_region_id())
        .filter(|region| game.battle_fairy_region_exists(*region))?;
    Some(BattleFairyBaseMagicSummon { region_id: region, phalanx, started_at_ms: started })
}
