//! Общий прямой снаряд `CChuckStone` (`0x19D`) и `CSkeletonArchery`
//! (`0x1A1`). Источник: точная пара `gameserver.exe` (SHA-256 `4F5C98E0…`) +
//! `GameServer.pdb` (RSDS match), исходные владельцы `chuckstone.cpp` и
//! `skeletonarchery.cpp`. Прежний переходный владелец —
//! `src/gameserver/appserver/skills/directprojectile.rs`; тела Check/AI и
//! helpers перенесены буквально (кластер B полосы Monster 0x19x, карта
//! полосы — запись аудита «Zone skills: карта полосы Monster 0x19x — 5
//! кластеров волн», 26 сентября 2026). ICF-свёртка классов доказана по RVA:
//! ChuckStone ≡ SkeletonArchery 4 тела `0x5377A0/0x53BF10/0x53BF30/0x569330`;
//! базовые Begin — CAttackSkill `0x5DEB00…`; общий End обоих владельцев
//! `0x0056A330` зафиксирован у payload `DirectProjectileProgress`
//! (`skills/execution/payload.rs`, порция 5 волны moveshape).
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
//! Restart пуст.
//!
//! Объявленные швы переноса (не расхождения): трейты ниже — переходные
//! фасады прежнего владельца `CGame`/`CMoveShape`, реализация остаётся у
//! делегата старого пакета (`appserver/skills/directprojectile.rs`); имена
//! членов сохраняют исходную операцию, швы потребляются статически (generic),
//! dyn-совместимость и `Send`-контракт не вводятся (ADR-0013). Первичный гейт
//! существования региона первого объекта клетки сохранён у шва
//! (`direct_projectile_shape_view_at` перечитывает owner-а на каждый вызов,
//! как `dash_*` фасады `skills/dash.rs`); резолв фабрики предметов и
//! отсутствующего оружия — у владельца. Контактный `OnBeenAttacked` и общий
//! End семейства остаются runtime-швами старого main loop; часы приходят от
//! делегата (fn-параметр), как в `skills/flash.rs`.

use nebokrai_shared::runtime::get_line_direction;

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::{CShape, ShapeView};

use super::baseattackruntime::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::execution::{MonsterSkillExecutionAccess, RegisteredSkillRecord};
use super::lifecycle::SkillLifecycle;
use super::SkillStage;

pub const CHUCK_STONE_SKILL_ID: u32 = 0x19d;
pub const SKELETON_ARCHERY_SKILL_ID: u32 = 0x1a1;

const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;
const SKILL_USAGE_TARGET_MIN_DISTANCE: u32 = 5_004;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;

/// Стадии результата одного тика ИИ прямого снаряда; обёртка очереди и
/// сам полный `End` семейства остаются у планировщика старого пакета.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectProjectileOutcome {
    Pending,
    Rejected,
    Completed,
}

/// Живая фигура стороны прямого снаряда: переходный фасад старого `CMoveShape`.
pub trait DirectProjectileMoveShape {
    fn shape(&self) -> &CShape;

    fn shape_mut(&mut self) -> &mut CShape;

    fn set_moveable(&mut self, moveable: bool);
}

/// Переходные фасады прежнего владельца `CGame`, открывающие прямому снаряду
/// только прежние обращения; имена сохраняют исходную операцию.
pub trait DirectProjectileGame {
    /// Hub-исполнение монстра записи навыка (`CMonster` старого пакета).
    type MonsterExecution: MonsterSkillExecutionAccess;

    /// Адрес записи зарегистрированного навыка (поколенческий ключ,
    /// holder + slot); непрозрачен для исполнения.
    type SkillAddress: Copy;

    type MoveShape: DirectProjectileMoveShape;

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

    /// Живой addon категории оружия (`GAP_WEAPON_CATEGORY`, index 1) в слоте 2;
    /// резолв фабрики предметов и отсутствующего оружия остаётся у владельца.
    fn player_weapon_addon_category(&self, player_id: i32) -> Option<i32>;

    /// Живое здоровье фигуры цели (проверка умершего S до получения U).
    fn move_shape_health(&self, region_id: i32, identity: ShapeIdentity) -> Option<u32>;

    /// Допуск `IsAttackAble` между живыми фигурами источника и цели.
    fn live_skill_target_attackable_between(
        &self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
    ) -> bool;

    /// Первая фигура клетки региона (`GetShape` области): owner перечитывается
    /// на каждый вызов; неподвижный первый объект не открывает остальные.
    fn direct_projectile_shape_view_at(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeView>;

    /// Все фигуры клетки региона (`GetShapes`), пустой список при отказе.
    fn direct_projectile_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView>;
}

/// Контактные операции с runtime игрового хода: удар и End(1) умершей цели.
/// Отделены, потому что тип хода принадлежит старому main loop.
pub trait DirectProjectileContact<Runtime>: DirectProjectileGame {
    /// Общий удар семейства: RTTI CMoveShape и U != S, Calculate и исходный
    /// `OnBeenAttacked` остаются внутри прежнего helper-а владельца.
    fn apply_direct_projectile_attack(
        &mut self,
        address: Self::SkillAddress,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    );

    /// Полный `End(1)` экземпляра при NULL S на входе AI первого удара
    /// (`end_state_skill` семейства; возвращённый outcome отбрасывается, как и
    /// раньше).
    fn end_direct_projectile_instance(&mut self, address: Self::SkillAddress, runtime: &mut Runtime);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectProjectileKind {
    ChuckStone,
    SkeletonArchery,
}

impl DirectProjectileKind {
    const fn from_skill_id(id: u32) -> Option<Self> {
        match id {
            CHUCK_STONE_SKILL_ID => Some(Self::ChuckStone),
            SKELETON_ARCHERY_SKILL_ID => Some(Self::SkeletonArchery),
            _ => None,
        }
    }

    const fn marks_prepared(self) -> bool {
        matches!(self, Self::ChuckStone)
    }
}

fn direct_projectile_path<Game: DirectProjectileGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
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

fn player_has_direct_projectile_weapon<Game: DirectProjectileGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
) -> bool {
    let accepted = matches!(game.player_weapon_addon_category(player_id), Some(3 | 4));
    if !accepted {
        // В конкретной проверке нет системной строки, а effect mode14 намеренно
        // не отправляет BFE01 для этого семейства.
        game.update_registered_skill_visual(instance, 14);
    }
    accepted
}

pub fn check_cast<Game: DirectProjectileGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
) -> bool {
    let Some((original_region, original_identity)) = original_user else {
        return false;
    };
    let Some(source) = game.resolve_state_move_shape(original_region, original_identity) else {
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
        .and_then(RegisteredSkillRecord::direct_projectile_progress_mut)
    else {
        return false;
    };
    progress.begin_after_check();
    true
}

fn live_user<Game: DirectProjectileGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = game.resolve_state_move_shape(region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn live_sufferer<Game: DirectProjectileGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = game.resolve_skill_sufferer(skill.lifecycle())?;
    let target = game.resolve_state_move_shape(region, identity)?.shape();
    Some((target.get_region_id(), target.identity()))
}

fn target_position<Game: DirectProjectileGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    target: Option<(i32, ShapeIdentity)>,
) -> (i32, i32) {
    target
        .and_then(|(region, identity)| game.resolve_state_move_shape(region, identity))
        .map(|target| {
            (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            )
        })
        .unwrap_or_else(|| skill.lifecycle().destination())
}

fn first_blocking_cell<Game: DirectProjectileGame>(
    game: &Game,
    source: (i32, ShapeIdentity),
    base_region: i32,
    path: &[(i32, i32, u8)],
) -> Option<(usize, (i32, i32))> {
    for (index, &(x, y, block)) in path.iter().enumerate() {
        if block == BLOCK_UNFLY {
            return Some((index, (x, y)));
        }
        if block != BLOCK_SHAPE {
            continue;
        }
        // CServerRegion::GetShape возвращает первый объект клетки. Если он
        // неподвижен, native не перебирает следующие CMoveShape этой клетки.
        let target = game.direct_projectile_shape_view_at(base_region, x, y)
            .and_then(|view| game.resolve_state_move_shape(base_region, view.identity))
            .map(|target| (target.shape().get_region_id(), target.shape().identity()));
        if target.is_some_and(|target| game.live_skill_target_attackable_between(source, target)) {
            return Some((index, (x, y)));
        }
    }
    None
}

fn attack_cell<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    source: (i32, ShapeIdentity),
    end: (i32, i32),
    runtime: &mut Runtime,
) where
    Game: DirectProjectileGame + DirectProjectileContact<Runtime>,
{
    if end == (0, 0) {
        return;
    }
    let Some(source) = game.resolve_state_move_shape(source.0, source.1)
        .filter(|source| source.shape().is_assigned_to_server_region())
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else {
        return;
    };
    // GetShape даёт снимок контейнера клетки; до каждого Attack Rust заново
    // разрешает объект, чтобы обработчик конца или смерти не оставил ссылку.
    for view in game.direct_projectile_cell_views(source.0, end.0, end.1) {
        let Some(target) = game.resolve_state_move_shape(source.0, view.identity)
            .map(|target| (target.shape().get_region_id(), target.shape().identity()))
        else {
            continue;
        };
        // Здесь остаются только RTTI CMoveShape и защита от U == S. Общий
        // helper сохраняет эту границу, Calculate и исходный OnBeenAttacked.
        game.apply_direct_projectile_attack(instance, source, target, runtime);
    }
}

pub fn run_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> DirectProjectileOutcome
where
    Game: DirectProjectileGame + DirectProjectileContact<Runtime>,
{
    let Some((kind, available)) = game.registered_skill(instance).and_then(|skill| {
        let kind = DirectProjectileKind::from_skill_id(skill.id())?;
        let available = skill.direct_projectile_progress()?.is_available();
        Some((kind, available))
    }) else {
        return DirectProjectileOutcome::Rejected;
    };
    if !available {
        return DirectProjectileOutcome::Pending;
    }
    let Some(properties) = game.registered_skill(instance)
        .and_then(|skill| game.skill_base_properties(skill.id(), skill.level())).cloned()
    else {
        return DirectProjectileOutcome::Rejected;
    };

    let target = game.registered_skill(instance).and_then(|skill| live_sufferer(game, skill));
    if target.is_some_and(|target| game.move_shape_health(target.0, target.1) == Some(0)) {
        game.update_registered_skill_visual(instance, 10);
        // Умерший S завершает End(1) ещё до обязательного получения U.
        return DirectProjectileOutcome::Completed;
    }
    let Some((mut destination, source)) = game.registered_skill(instance).and_then(|skill| {
        live_user(game, skill).map(|source| (target_position(game, skill, target), source))
    }) else {
        return DirectProjectileOutcome::Rejected;
    };

    let Some(condition_checked) = game.registered_skill(instance)
        .and_then(|skill| skill.direct_projectile_progress())
        .map(|progress| progress.condition_checked())
    else {
        return DirectProjectileOutcome::Rejected;
    };
    if !condition_checked {
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(
                properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0,
            );
        }
        let Some(user) = game.resolve_state_move_shape(source.0, source.1) else {
            return DirectProjectileOutcome::Rejected;
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
        // Удалённая регистрация не подменяется новым экземпляром того же ID.
        if game.registered_skill(instance).is_none() {
            return DirectProjectileOutcome::Pending;
        }
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.set_moveable(false);
        }
        let Some(skill) = game.registered_skill_mut(instance) else {
            return DirectProjectileOutcome::Pending;
        };
        let Some(progress) = skill.direct_projectile_progress_mut() else {
            return DirectProjectileOutcome::Pending;
        };
        progress.mark_condition_checked();
        let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
    }

    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some((started, attacking_started)) = game.registered_skill(instance).and_then(|skill| {
        skill.direct_projectile_progress()
            .map(|progress| (skill.lifecycle().started_at_ms(), progress.attacking_started()))
    }) else {
        return DirectProjectileOutcome::Pending;
    };
    if !attacking_started {
        if now_milliseconds() < started.wrapping_add(delay) {
            return DirectProjectileOutcome::Pending;
        }
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.set_moveable(true);
        }
        let Some(skill) = game.registered_skill(instance) else {
            return DirectProjectileOutcome::Rejected;
        };
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let path = direct_projectile_path(game, skill, kind, maximum);
        let base_region = skill.lifecycle().user().0;
        let minimum = properties.query_property(SKILL_USAGE_TARGET_MIN_DISTANCE);
        if path_too_far(&path, maximum) || (minimum != 0 && path.len() < minimum as usize) {
            game.update_registered_skill_visual(instance, 11);
            return DirectProjectileOutcome::Rejected;
        }
        let blocking = first_blocking_cell(game, source, base_region, &path);
        let path_index = blocking.map_or(path.len(), |(index, _)| index);
        if let Some((_, endpoint)) = blocking {
            // Сначала S заменяется точкой, затем GetS ищет объект уже в ней.
            // Найденная цель закрепляется в регионе исходного U; координаты
            // этого вызова AI остаются самой клеткой препятствия.
            destination = endpoint;
            let Some(skill) = game.registered_skill_mut(instance) else {
                return DirectProjectileOutcome::Pending;
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
                return DirectProjectileOutcome::Pending;
            };
            let Some(progress) = skill.direct_projectile_progress_mut() else {
                return DirectProjectileOutcome::Pending;
            };
            progress.start_flight(flight);
        }
        game.update_registered_skill_visual(instance, 1);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return DirectProjectileOutcome::Pending;
        };
        {
            let Some(progress) = skill.direct_projectile_progress_mut() else {
                return DirectProjectileOutcome::Pending;
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
        return DirectProjectileOutcome::Pending;
    };
    if now_milliseconds()
        < started.wrapping_add(delay).wrapping_add(progress.missile_flying_time_ms())
    {
        return DirectProjectileOutcome::Pending;
    }
    // Координаты и NULL-проверка относятся к стеку текущего AI, не к новому
    // GetS после visual/Attack. Следующий AI сам вновь прочитает живую цель.
    attack_cell(game, instance, source, destination, runtime);
    if target.is_none() {
        game.end_direct_projectile_instance(instance, runtime);
    }
    let Some(auto_restart) = game.registered_skill(instance)
        .and_then(|skill| skill.direct_projectile_progress())
        .map(|progress| progress.auto_restart())
    else {
        return DirectProjectileOutcome::Pending;
    };
    if auto_restart {
        // Унаследованный CState::Restart пуст; после него AI возвращается.
        return DirectProjectileOutcome::Pending;
    }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
        let _ = skill.advance_execution(SkillStage::Attack, SkillStage::Apply);
    }
    DirectProjectileOutcome::Completed
}
