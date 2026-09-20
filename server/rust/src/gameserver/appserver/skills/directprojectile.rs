//! Общий прямой снаряд `CChuckStone` (`0x19D`) и `CSkeletonArchery`
//! (`0x1A1`). Источник: точная пара `gameserver.exe + GameServer.pdb`,
//! исходные владельцы `chuckstone.cpp` и `skeletonarchery.cpp`.
//!
//! Все Begin создают effect до Check. Check требует только U, таблицу,
//! исходный GetTargetPath/MAX и для игрока оружие категории 3 или 4;
//! MP, reuse, блоки пути и самонацеливание ему не принадлежат. Первый AI
//! читает живого S либо сохранённую точку, удерживает U до абсолютного
//! `start + delay`, затем повторяет путь, выбирает первую BLOCK_UNFLY или
//! BLOCK_SHAPE с `S.IsAttackAble(U)` и выпускает снаряд. ChuckStone строит
//! выпуск с принудительной длиной MAX, SkeletonArchery — обычный путь.
//! Преграда заменяет S точкой базы, после чего GetS закрепляет объект этой
//! клетки. Следующий AI читает живую S; координаты текущего вызова остаются
//! локальными. NULL S на входе AI даёт два End(1) после удара.
//! После полёта оба обходят все `CMoveShape` клетки без нового допуска,
//! фильтра типа, проверки смерти или дедупликации.
//!
//! `DirectProjectileProgress` заменяет поля C++-объекта и хранится в том же
//! зарегистрированном экземпляре игрока или монстра. End очищает доступность,
//! проверку условия, начало выстрела и время полёта до Move1;
//! `m_bAutoRestart` конструируется ложным, не меняется, а унаследованный
//! Restart пуст. Вектор и SlotMap реестра заменяют временные C++-контейнеры
//! и указатели без копии.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::cell_views;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
    state_skill_outcome,
};
use super::weaponattack::apply_direct_projectile_attack;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
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
    RegionShapeResolver, ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;
const SKILL_USAGE_TARGET_MIN_DISTANCE: u32 = 5_004;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectProjectileKind {
    ChuckStone,
    SkeletonArchery,
}

impl DirectProjectileKind {
    const fn from_skill_id(id: u32) -> Option<Self> {
        match id {
            super::chuckstone::CHUCK_STONE_SKILL_ID => Some(Self::ChuckStone),
            super::skeletonarchery::SKELETON_ARCHERY_SKILL_ID => Some(Self::SkeletonArchery),
            _ => None,
        }
    }

    const fn marks_prepared(self) -> bool {
        matches!(self, Self::ChuckStone)
    }
}

/// Поля конкретного C++-экземпляра между Begin, выпуском и End.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DirectProjectileProgress {
    available: bool,
    condition_checked: bool,
    attacking_started: bool,
    missile_flying_time_ms: u32,
    auto_restart: bool,
}

impl DirectProjectileProgress {
    fn begin_after_check(&mut self) {
        self.available = true;
        self.condition_checked = false;
        self.attacking_started = false;
        self.missile_flying_time_ms = 0;
    }

    fn is_available(self) -> bool {
        self.available
    }

    fn condition_checked(self) -> bool {
        self.condition_checked
    }

    fn attacking_started(self) -> bool {
        self.attacking_started
    }

    fn mark_condition_checked(&mut self) {
        self.condition_checked = true;
    }

    fn start_flight(&mut self, missile_flying_time_ms: u32) {
        self.missile_flying_time_ms = missile_flying_time_ms;
    }

    fn mark_attacking_started(&mut self) {
        self.attacking_started = true;
    }

    fn auto_restart(self) -> bool {
        self.auto_restart
    }

    pub(crate) const fn missile_flying_time_ms(&self) -> u32 {
        self.missile_flying_time_ms
    }

    /// `CChuckStone::End` / `CSkeletonArchery::End` сбрасывает эти четыре
    /// поля до `GetUser()->SetMoveable(true)` и базового End.
    pub(crate) fn prepare_derived_end(&mut self) {
        self.available = false;
        self.condition_checked = false;
        self.attacking_started = false;
        self.missile_flying_time_ms = 0;
    }
}

fn direct_projectile_path(
    game: &CGame,
    skill: &MoveShapeSkill,
    kind: DirectProjectileKind,
    maximum: u32,
) -> Vec<(i32, i32, u8)> {
    match kind {
        // CChuckStone::AI вызывает перегрузку GetTargetPath(vector, MAX).
        DirectProjectileKind::ChuckStone => {
            game.skill_target_path_with_length(skill.lifecycle(), maximum)
        }
        // CSkeletonArchery::AI вызывает перегрузку без длины.
        DirectProjectileKind::SkeletonArchery => game.skill_target_path(skill.lifecycle()),
    }
}

fn path_too_far(path: &[(i32, i32, u8)], maximum: u32) -> bool {
    maximum != 0 && maximum.wrapping_add(1) < path.len() as u32
}

fn player_has_direct_projectile_weapon(
    game: &mut CGame,
    instance: RegisteredSkill,
    player_id: i32,
) -> bool {
    let accepted = game.find_player(player_id)
        .and_then(|player| player.equipment().get_goods(2))
        .is_some_and(|weapon| {
            matches!(
                weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1),
                3 | 4
            )
        });
    if !accepted {
        // В конкретной проверке нет системной строки, а effect mode14 намеренно
        // не отправляет BFE01 для этого семейства.
        game.update_registered_skill_visual(instance, 14);
    }
    accepted
}

fn check_direct_projectile_cast(
    game: &mut CGame,
    instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>,
) -> bool {
    let Some((original_region, original_identity)) = original_user else {
        return false;
    };
    let Some(source) = resolve_state_move_shape(game, original_region, original_identity) else {
        return false;
    };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else {
        return false;
    };
    if DirectProjectileKind::from_skill_id(skill.id()).is_none() {
        return false;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let path = game.skill_target_path(skill.lifecycle());
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if path_too_far(&path, maximum) {
        game.update_registered_skill_visual(instance, 11);
        return false;
    }
    if source.1.object_type == PLAYER_TYPE
        && !player_has_direct_projectile_weapon(game, instance, source.1.id)
    {
        return false;
    }
    let Some(progress) = game.registered_skill_mut(instance)
        .and_then(MoveShapeSkill::direct_projectile_progress_mut)
    else {
        return false;
    };
    progress.begin_after_check();
    true
}

fn live_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn live_sufferer(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = resolve_skill_sufferer(game, skill.lifecycle())?;
    let target = resolve_state_move_shape(game, region, identity)?.shape();
    Some((target.get_region_id(), target.identity()))
}

fn target_position(
    game: &CGame,
    skill: &MoveShapeSkill,
    target: Option<(i32, ShapeIdentity)>,
) -> (i32, i32) {
    target
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .map(|target| {
            (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            )
        })
        .unwrap_or_else(|| skill.lifecycle().destination())
}

fn first_blocking_cell(
    game: &CGame,
    source: (i32, ShapeIdentity),
    base_region: i32,
    path: &[(i32, i32, u8)],
) -> Option<(usize, (i32, i32))> {
    let (area_width, area_height) = game.area_dimensions();
    for (index, &(x, y, block)) in path.iter().enumerate() {
        if block == BLOCK_UNFLY {
            return Some((index, (x, y)));
        }
        if block != BLOCK_SHAPE {
            continue;
        }
        // CServerRegion::GetShape возвращает первый объект клетки. Если он
        // неподвижен, native не перебирает следующие CMoveShape этой клетки.
        let target = game.find_region(base_region).and_then(|owner| {
            let resolver = RegionShapeResolver { game, owner };
            owner.base()
                .get_shape(x, y, area_width, area_height, &resolver)
                .ok()
                .flatten()
        }).and_then(|view| resolve_state_move_shape(game, base_region, view.identity))
            .map(|target| (target.shape().get_region_id(), target.shape().identity()));
        if target.is_some_and(|target| game.live_skill_target_attackable_between(source, target)) {
            return Some((index, (x, y)));
        }
    }
    None
}

fn attack_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    end: (i32, i32),
    runtime: &mut Runtime,
) {
    if end == (0, 0) {
        return;
    }
    let Some(source) = resolve_state_move_shape(game, source.0, source.1)
        .filter(|source| source.shape().is_assigned_to_server_region())
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else {
        return;
    };
    // GetShape даёт снимок контейнера клетки; до каждого Attack Rust заново
    // разрешает объект, чтобы обработчик конца или смерти не оставил ссылку.
    for view in cell_views(game, source.0, end.0, end.1) {
        let Some(target) = resolve_state_move_shape(game, source.0, view.identity)
            .map(|target| (target.shape().get_region_id(), target.shape().identity()))
        else {
            continue;
        };
        // Здесь остаются только RTTI CMoveShape и защита от U == S. Общий
        // helper сохраняет эту границу, Calculate и исходный OnBeenAttacked.
        apply_direct_projectile_attack(game, instance, source, target, runtime);
    }
}

fn run_direct_projectile_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some((kind, available)) = game.registered_skill(instance).and_then(|skill| {
        let kind = DirectProjectileKind::from_skill_id(skill.id())?;
        let available = skill.direct_projectile_progress()?.is_available();
        Some((kind, available))
    }) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if !available {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.registered_skill(instance)
        .and_then(|skill| game.skill_base_properties(skill.id(), skill.level())).cloned()
    else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };

    let target = game.registered_skill(instance).and_then(|skill| live_sufferer(game, skill));
    if target.is_some_and(|target| game.move_shape_health(target.0, target.1) == Some(0)) {
        game.update_registered_skill_visual(instance, 10);
        // Умерший S завершает End(1) ещё до обязательного получения U.
        return state_skill_outcome(QueuedSkillExecutionState::Completed);
    }
    let Some((mut destination, source)) = game.registered_skill(instance).and_then(|skill| {
        live_user(game, skill).map(|source| (target_position(game, skill, target), source))
    }) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };

    let Some(condition_checked) = game.registered_skill(instance)
        .and_then(|skill| skill.direct_projectile_progress())
        .map(|progress| progress.condition_checked())
    else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if !condition_checked {
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(
                properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0,
            );
        }
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        let direction = get_line_direction(
            user.shape().get_tile_x().unwrap_or(i32::MIN),
            user.shape().get_tile_y().unwrap_or(i32::MIN),
            destination.0,
            destination.1,
        );
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        // Удалённая регистрация не подменяется новым экземпляром того же ID.
        if game.registered_skill(instance).is_none() {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.set_moveable(false);
        }
        let Some(skill) = game.registered_skill_mut(instance) else {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        };
        let Some(progress) = skill.direct_projectile_progress_mut() else {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        };
        progress.mark_condition_checked();
        let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
    }

    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some((started, attacking_started)) = game.registered_skill(instance).and_then(|skill| {
        skill.direct_projectile_progress()
            .map(|progress| (skill.lifecycle().started_at_ms(), progress.attacking_started()))
    }) else {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    };
    if !attacking_started {
        if runtime.now_milliseconds() < started.wrapping_add(delay) {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.set_moveable(true);
        }
        let Some(skill) = game.registered_skill(instance) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let path = direct_projectile_path(game, skill, kind, maximum);
        let base_region = skill.lifecycle().user().0;
        let minimum = properties.query_property(SKILL_USAGE_TARGET_MIN_DISTANCE);
        if path_too_far(&path, maximum) || (minimum != 0 && path.len() < minimum as usize) {
            game.update_registered_skill_visual(instance, 11);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let blocking = first_blocking_cell(game, source, base_region, &path);
        let path_index = blocking.map_or(path.len(), |(index, _)| index);
        if let Some((_, endpoint)) = blocking {
            // Сначала S заменяется точкой, затем GetS ищет объект уже в ней.
            // Найденная цель закрепляется в регионе исходного U; координаты
            // этого вызова AI остаются самой клеткой препятствия.
            destination = endpoint;
            let Some(skill) = game.registered_skill_mut(instance) else {
                return state_skill_outcome(QueuedSkillExecutionState::Pending);
            };
            skill.lifecycle_mut().set_point_target(endpoint);
            let selected = game.registered_skill(instance).and_then(|skill| live_sufferer(game, skill));
            if let Some((_, identity)) = selected
                && let Some(skill) = game.registered_skill_mut(instance)
            {
                skill.lifecycle_mut().set_sufferer(base_region, identity);
            }
        }
        let flight = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path_index as u32);
        {
            let Some(skill) = game.registered_skill_mut(instance) else {
                return state_skill_outcome(QueuedSkillExecutionState::Pending);
            };
            let Some(progress) = skill.direct_projectile_progress_mut() else {
                return state_skill_outcome(QueuedSkillExecutionState::Pending);
            };
            progress.start_flight(flight);
        }
        game.update_registered_skill_visual(instance, 1);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        };
        {
            let Some(progress) = skill.direct_projectile_progress_mut() else {
                return state_skill_outcome(QueuedSkillExecutionState::Pending);
            };
            progress.mark_attacking_started();
        }
        if kind.marks_prepared() {
            skill.lifecycle_mut().mark_prepared();
        }
        let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
    }

    let Some((started, progress)) = game.registered_skill(instance).and_then(|skill| {
        skill.direct_projectile_progress().copied().map(|progress| {
            (skill.lifecycle().started_at_ms(), progress)
        })
    }) else {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    };
    if runtime.now_milliseconds()
        < started.wrapping_add(delay).wrapping_add(progress.missile_flying_time_ms())
    {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    // Координаты и NULL-проверка относятся к стеку текущего AI, не к новому
    // GetS после visual/Attack. Следующий AI сам вновь прочитает живую цель.
    attack_cell(game, instance, source, destination, runtime);
    if target.is_none() {
        let _ = end_state_skill(game, instance, 1, runtime);
    }
    let Some(auto_restart) = game.registered_skill(instance)
        .and_then(|skill| skill.direct_projectile_progress())
        .map(|progress| progress.auto_restart())
    else {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    };
    if auto_restart {
        // Унаследованный CState::Restart пуст; после него AI возвращается.
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
        let _ = skill.advance_execution(SkillStage::Attack, SkillStage::Apply);
    }
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_direct_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game,
        player_id,
        instance,
        dispatch,
        runtime,
        SkillVisualEffectKind::DirectProjectile,
        |game, instance, _, _| check_direct_projectile_cast(game, instance, original_user),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_direct_projectile_ai,
    )
}

struct DirectProjectileSkill<const ID: u32>;

impl<const ID: u32> RegisteredStateSkill for DirectProjectileSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::DirectProjectile;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        _target: StateSkillBeginTarget,
        _runtime: &mut Runtime,
    ) -> bool {
        let source = game.registered_skill(instance)
            .and_then(|skill| live_user(game, skill));
        check_direct_projectile_cast(game, instance, source)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_direct_projectile_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
                end_state_skill(game, instance, 1, runtime)
            }
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_direct_projectile<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<DirectProjectileSkill<ID>, Runtime>(
        game,
        owner,
        monster_id,
        target,
        skill_level,
        runtime,
    )
}
