//! Пошаговые снаряды `CEnergyBolt` (`0x1A0`), `CSnakeBolt` (`0x1A5`) и
//! `CZombieClaw` (`0x1A2`): классы ICF-идентичны. Payload полёта и область
//! удара — `skills/execution/payload.rs` (`PathProjectileProgress`/
//! `PathProjectileScope`).
//!
//! Quirks: путь форсирован MAX и сохраняется после выпуска, обработка по
//! одной клетке через `start+delay+flying·position`; область 1×1 уровням 1/2,
//! иначе 3×3, сохраняется между End; позиция начинается с 0 в Begin и не
//! перезаписывается выпуском (в т.ч. после вложенного вызова); у EnergyBolt
//! нулевая MP-цена не запрещает движение, у SnakeBolt/ZombieClaw — запрещает.
//!
//! Швы: трейты — фасады `CGame`/`CMoveShape` старого пакета (делегат
//! `appserver/skills/energybolt.rs`); проверка/списание MP (`rangedweaponcast`),
//! стихийный удар (`directelementattack`) и полный End — runtime-швы main loop;
//! часы — fn-параметр делегата.
//!
//! Исходные владельцы PDB: `appserver/skills/{energybolt,snakebolt,zombieclaw}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#снаряды-монстров-direct-path-littlestar-yunshenglightning

use nebokrai_shared::runtime::get_line_direction;

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};
use crate::regions::shape::{CShape, ShapeView};

use super::baseattackruntime::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::execution::{
    ArrowTargetIdentity, MonsterSkillExecutionAccess, PathProjectileProgress, PathProjectileScope,
    RegisteredSkillRecord,
};
use super::lifecycle::SkillLifecycle;
use super::{SkillStage, skill_is_restored};

pub const ENERGY_BOLT_SKILL_ID: u32 = 0x1a0;
pub const SNAKE_BOLT_SKILL_ID: u32 = 0x1a5;
pub const ZOMBIE_CLAW_SKILL_ID: u32 = 0x1a2;

const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;

/// Стадии результата одного тика ИИ путевого снаряда; обёртка очереди и
/// сам полный `End` семейства остаются у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathProjectileOutcome {
    Pending,
    Rejected,
    Completed,
}

/// Живая фигура стороны путевого снаряда: переходный фасад старого `CMoveShape`.
pub trait PathProjectileMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);
}

/// Переходные фасады прежнего владельца `CGame`, открывающие путевому снаряду
/// только прежние обращения; имена сохраняют исходную операцию.
pub trait PathProjectileGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для исполнения.
    type SkillAddress: Copy;

    type MoveShape: PathProjectileMoveShape;

    fn registered_skill(
        &self,
        address: Self::SkillAddress,
    ) -> Option<&RegisteredSkillRecord<Self::MonsterExecution>>;

    fn registered_skill_mut(
        &mut self,
        address: Self::SkillAddress,
    ) -> Option<&mut RegisteredSkillRecord<Self::MonsterExecution>>;

    fn update_registered_skill_visual(&mut self, address: Self::SkillAddress, mode: u32);

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties>;

    /// Точный общий `GetSufferer`: сохранённая identity, затем клетка региона.
    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)>;

    /// Живой `GetUser` по сохранённым region/type/id.
    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&Self::MoveShape>;

    fn resolve_state_move_shape_mut(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&mut Self::MoveShape>;

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)>;

    fn skill_target_path_with_length(
        &self,
        lifecycle: &SkillLifecycle,
        maximum: u32,
    ) -> Vec<(i32, i32, u8)>;

    /// Живая мана игрока-источника (`CPlayer::mana`).
    fn player_mana(&self, player_id: i32) -> Option<u32>;

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]);

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32);

    /// Живое здоровье фигуры цели области.
    fn move_shape_health(&self, region_id: i32, identity: ShapeIdentity) -> Option<u32>;

    /// Допуск `IsAttackAble` между живыми фигурами источника и цели.
    fn live_skill_target_attackable_between(
        &self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
    ) -> bool;

    /// Все фигуры клетки региона (`GetShapes`), пустой список при отказе.
    fn path_projectile_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView>;

    /// Текущий server region жив после срока; его исчезновение не превращается
    /// в End и не подменяется прежним регионом Begin.
    fn path_projectile_region_exists(&self, region_id: i32) -> bool;

    /// Блок клетки области текущего региона (`skill_cell_block`).
    fn path_projectile_skill_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8>;
}

/// Контактные операции с runtime игрового хода: списание MP первого AI и
/// стихийный удар области. Отделены, потому что тип хода принадлежит старому
/// main loop.
pub trait PathProjectileContact<Runtime>: PathProjectileGame {
    /// Общее списание MP без системной строки (`spend_cast_mana_without_text`
    /// семейства `rangedweaponcast`; visual7 при отказе остаётся внутри него).
    fn spend_path_projectile_cast_mana(
        &mut self,
        address: Self::SkillAddress,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool;

    /// Прямой стихийный удар (`apply_direct_element_attack` владельца):
    /// формула прямой стихии, SoulCollect, PK и OnBeenAttacked — hub-шов.
    fn apply_direct_element_attack(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathProjectileProfile {
    EnergyBolt,
    SnakeBolt,
    ZombieClaw,
}

impl PathProjectileProfile {
    fn from_skill_id(skill_id: u32) -> Option<Self> {
        match skill_id {
            ENERGY_BOLT_SKILL_ID => Some(Self::EnergyBolt),
            SNAKE_BOLT_SKILL_ID => Some(Self::SnakeBolt),
            ZOMBIE_CLAW_SKILL_ID => Some(Self::ZombieClaw),
            _ => None,
        }
    }

    const fn locks_free_mana(self) -> bool {
        !matches!(self, Self::EnergyBolt)
    }
}

fn resolved_user<Game: PathProjectileGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = game.resolve_state_move_shape(region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn fail<Game: PathProjectileGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player: Option<i32>,
    mode: u32,
    text: &[u8],
) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player) = player {
        game.send_skill_system_info(player, text);
    }
}

fn check_path_projectile_mana<Game: PathProjectileGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    profile: PathProjectileProfile,
) -> bool {
    if source.1.object_type != PLAYER_TYPE {
        return true;
    }
    let player = source.1.id;
    let cost = properties.query_property(USER_MP_LOSE);
    if cost != 0 {
        let Some(mana) = game.player_mana(player) else {
            return false;
        };
        if (mana.wrapping_sub(cost) as i32) < 0 {
            game.update_registered_skill_visual(instance, 7);
            game.send_skill_system_info_with_unsigned(player, b"GS0288", cost);
            return false;
        }
    }
    if cost != 0 || profile.locks_free_mana() {
        let Some(source) = game.resolve_state_move_shape_mut(source.0, source.1) else {
            return false;
        };
        source.set_moveable(false);
    }
    true
}

/// Аргумент конкретного Begin живёт только на стеке вызова: цель резолвит
/// обвязка старого пакета (`StateSkillBeginTarget`), сюда приходит живой снимок.
pub fn check_cast<Game: PathProjectileGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>,
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(original_user) = original_user else {
        return false;
    };
    let Some(source) = game.resolve_state_move_shape(original_user.0, original_user.1) else {
        return false;
    };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    if let Some(target) = original_target {
        let self_target = match (
            game.resolve_state_move_shape(source.0, source.1),
            game.resolve_state_move_shape(target.0, target.1),
        ) {
            (Some(source), Some(target)) => std::ptr::eq(source, target),
            _ => false,
        };
        if self_target {
            fail(game, instance, player, 10, b"GS0286");
            return false;
        }
    }
    let Some(skill) = game.registered_skill(instance) else {
        return false;
    };
    let Some(profile) = PathProjectileProfile::from_skill_id(skill.id()) else {
        return false;
    };
    let level = skill.level();
    let Some(properties) = game.skill_base_properties(skill.id(), level).cloned() else {
        return false;
    };
    if !skill_is_restored(
        skill.last_used_ms(),
        properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
        now_milliseconds(),
    ) {
        fail(game, instance, player, 13, b"GS0278");
        return false;
    }

    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let Some(path) = game.registered_skill(instance).map(|skill| game.skill_target_path(skill.lifecycle())) else {
        return false;
    };
    if maximum != 0 && path.len() > maximum as usize {
        fail(game, instance, player, 11, b"GS0290");
        return false;
    }
    if !check_path_projectile_mana(game, instance, source, &properties, profile) {
        return false;
    }
    let Some(progress) = game.registered_skill_mut(instance)
        .and_then(RegisteredSkillRecord::path_projectile_progress_mut)
    else {
        return false;
    };
    progress.clear_end_paths();
    progress.ensure_scope(level);
    true
}

fn attack_scope<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    region: i32,
    center_x: i32,
    center_y: i32,
    runtime: &mut Runtime,
) -> bool
where
    Game: PathProjectileGame + PathProjectileContact<Runtime>,
{
    if game.resolve_state_move_shape(source.0, source.1).is_none()
        || (center_x == 0 && center_y == 0)
    {
        return false;
    }
    let Some(radius) = game.registered_skill(instance)
        .and_then(RegisteredSkillRecord::path_projectile_progress)
        .and_then(|progress| progress.scope())
        .map(PathProjectileScope::radius)
    else {
        return false;
    };
    let source_key = ArrowTargetIdentity::new(source.0, source.1);
    let mut attacked = Vec::<ArrowTargetIdentity>::new();
    let mut did_attack = false;

    for offset_x in -radius..=radius {
        for offset_y in -radius..=radius {
            let cell_x = center_x.wrapping_add(offset_x);
            let cell_y = center_y.wrapping_add(offset_y);
            for view in game.path_projectile_cell_views(region, cell_x, cell_y) {
                let Some(target) = game.resolve_state_move_shape(region, view.identity) else {
                    continue;
                };
                let target = (target.shape().get_region_id(), target.shape().identity());
                let target_key = ArrowTargetIdentity::new(target.0, target.1);
                if target_key == source_key
                    || game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
                    || attacked.contains(&target_key)
                {
                    continue;
                }
                // Цель эффекта записывается до проверок типа и CanAttack.
                if cell_x == center_x && cell_y != 0 {
                    if let Some(progress) = game.registered_skill_mut(instance)
                        .and_then(RegisteredSkillRecord::path_projectile_progress_mut)
                    {
                        progress.select_visual_target_if_empty(target.1);
                    }
                }
                if !matches!(target.1.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || !game.live_skill_target_attackable_between(source, target)
                {
                    continue;
                }
                game.apply_direct_element_attack(instance, source, target, runtime);
                // Добавление идёт после непосредственного контакта: End или
                // повторный callback не отменяют запись временного списка.
                attacked.push(target_key);
                did_attack = true;
            }
        }
    }
    did_attack
}

pub fn run_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> PathProjectileOutcome
where
    Game: PathProjectileGame + PathProjectileContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else {
        return PathProjectileOutcome::Rejected;
    };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return PathProjectileOutcome::Pending;
    };
    if PathProjectileProfile::from_skill_id(skill.id()).is_none() {
        return PathProjectileOutcome::Rejected;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return PathProjectileOutcome::Rejected;
    };
    let Some(source) = resolved_user(game, skill) else {
        return PathProjectileOutcome::Rejected;
    };
    if game.registered_skill(instance)
        .and_then(RegisteredSkillRecord::path_projectile_progress)
        .and_then(PathProjectileProgress::scope)
        .is_none()
    {
        return PathProjectileOutcome::Rejected;
    }
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);

    if stage == SkillStage::Begin {
        if !game.spend_path_projectile_cast_mana(instance, player, &properties) {
            return PathProjectileOutcome::Rejected;
        }
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(
                properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0,
            );
        }
        let destination = game.registered_skill(instance)
            .and_then(|skill| game.resolve_skill_sufferer(skill.lifecycle()))
            .and_then(|target| game.resolve_state_move_shape(target.0, target.1))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ))
            .or_else(|| game.registered_skill(instance).map(|skill| skill.lifecycle().destination()));
        let Some(destination) = destination else {
            return PathProjectileOutcome::Rejected;
        };
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else {
            return PathProjectileOutcome::Rejected;
        };
        let direction = get_line_direction(
            user.shape().get_tile_x().unwrap_or(i32::MIN),
            user.shape().get_tile_y().unwrap_or(i32::MIN),
            destination.0,
            destination.1,
        );
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }

    let Some(release_started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return PathProjectileOutcome::Rejected;
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let fired = game.registered_skill(instance)
        .and_then(RegisteredSkillRecord::path_projectile_progress)
        .map(PathProjectileProgress::fired);
    let Some(fired) = fired else {
        return PathProjectileOutcome::Rejected;
    };

    if !fired {
        if now_milliseconds() < release_started.wrapping_add(delay) {
            return PathProjectileOutcome::Pending;
        }
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.set_moveable(true);
        }
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let Some(path) = game.registered_skill(instance)
            .map(|skill| game.skill_target_path_with_length(skill.lifecycle(), maximum))
        else {
            return PathProjectileOutcome::Rejected;
        };
        if maximum != 0 && path.len() > maximum.wrapping_add(1) as usize {
            game.update_registered_skill_visual(instance, 11);
            return PathProjectileOutcome::Rejected;
        }
        let flying_unit_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
        let Some(progress) = game.registered_skill_mut(instance)
            .and_then(RegisteredSkillRecord::path_projectile_progress_mut)
        else {
            return PathProjectileOutcome::Rejected;
        };
        progress.prepare_flight(path, flying_unit_ms);
        game.update_registered_skill_visual(instance, 1);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return PathProjectileOutcome::Pending;
        };
        let Some(progress) = skill.path_projectile_progress_mut() else {
            return PathProjectileOutcome::Pending;
        };
        progress.start_flight();
        skill.lifecycle_mut().mark_prepared();
        let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
    }

    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return PathProjectileOutcome::Pending;
    };
    let Some((current_position, path_len, cell)) = game.registered_skill(instance)
        .and_then(RegisteredSkillRecord::path_projectile_progress)
        .map(|progress| (
            progress.current_position(),
            progress.path_len(),
            progress.current_cell(),
        ))
    else {
        return PathProjectileOutcome::Pending;
    };
    let flying_unit_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let due = started
        .wrapping_add(delay)
        .wrapping_add(flying_unit_ms.wrapping_mul(current_position as u32));
    if now_milliseconds() < due {
        return PathProjectileOutcome::Pending;
    }

    // После срока оригинал читает текущий server region; его исчезновение не
    // превращается в End и не подменяется прежним регионом Begin.
    let Some(user) = game.resolve_state_move_shape(source.0, source.1) else {
        return PathProjectileOutcome::Pending;
    };
    let user = user.shape();
    if !user.is_assigned_to_server_region() {
        return PathProjectileOutcome::Pending;
    }
    let region = user.get_region_id();
    if !game.path_projectile_region_exists(region) {
        return PathProjectileOutcome::Pending;
    }

    if current_position >= path_len {
        game.update_registered_skill_visual(instance, 3);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
            let _ = skill.advance_execution(SkillStage::Attack, SkillStage::Apply);
        }
        return PathProjectileOutcome::Completed;
    }
    let Some((cell_x, cell_y)) = cell else {
        return PathProjectileOutcome::Pending;
    };
    let Some(progress) = game.registered_skill_mut(instance)
        .and_then(RegisteredSkillRecord::path_projectile_progress_mut)
    else {
        return PathProjectileOutcome::Pending;
    };
    progress.set_end_position(cell_x, cell_y);

    let Some(block) = game.path_projectile_skill_cell_block(region, cell_x, cell_y) else {
        return PathProjectileOutcome::Pending;
    };
    match block {
        BLOCK_SHAPE => {
            if attack_scope(game, instance, source, region, cell_x, cell_y, runtime) {
                game.update_registered_skill_visual(instance, 3);
                let Some(progress) = game.registered_skill_mut(instance)
                    .and_then(RegisteredSkillRecord::path_projectile_progress_mut)
                else {
                    return PathProjectileOutcome::Pending;
                };
                progress.finish_after_collision();
                return PathProjectileOutcome::Pending;
            }
        }
        BLOCK_UNFLY => {
            game.update_registered_skill_visual(instance, 3);
            let Some(progress) = game.registered_skill_mut(instance)
                .and_then(RegisteredSkillRecord::path_projectile_progress_mut)
            else {
                return PathProjectileOutcome::Pending;
            };
            progress.finish_after_unfly();
        }
        _ => {}
    }
    let Some(skill) = game.registered_skill_mut(instance) else {
        return PathProjectileOutcome::Pending;
    };
    let Some(progress) = skill.path_projectile_progress_mut() else {
        return PathProjectileOutcome::Pending;
    };
    progress.advance();
    let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
    PathProjectileOutcome::Pending
}
