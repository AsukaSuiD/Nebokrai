//! Общий cast базовых и огненных снарядов Archery, BaseMagic, FireBolt, FireBall.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые appserver/skills/*.cpp.
//! Begin создаёт visual до Check; ошибки завершаются End0, выпуск — End1.
//! End очищает фазу перед Move1/SummonEnd, но оставляет время полёта у
//! зарегистрированного Archery/BaseMagic/FireBolt. FireBall такого поля не имеет.
//! Очередь и единственная база принадлежат существующему реестру игрока/монстра.
//!
//! FireBolt/FireBall списывают MP и вызывают OnChangeStates до CAN. Archery
//! проверяет смерть/самоцель до CAN и смерть повторно; Magic/FireBolt — после.
//! FireBall не требует S и не проверяет смерть в AI: один свежий S задаёт X/Y,
//! иначе берётся базовая точка. Magic/Bolt перечитывают GetS для Y и X отдельно.
//! Archery использует захваченного S; U всегда захвачен в начале AI. Самоцель
//! Archery/Magic даёт два visual10, Bolt — один. FireBall блокирует движение
//! в Check, остальные — после visual0 первого AI. Поздний отказ не возвращает MP.
//!
//! Выпуск сравнивает unsigned start+delay, разрешает движение до visual1.
//! Прицельные варианты дважды перечитывают S; поздняя смерть даёт
//! visual10→текст→visual10. Расстояние и Summon используют захваченные U/S.
//! Ball вызывает GetFacePos, но аргументы point-Summon не участвуют в его пути.
//! Его путь задаётся forced MAX, остальных — RealDistance. Исходно пустой
//! путь прекращает Summon; непустой очищает S до BLOCK2. Ball отсекает хвост,
//! допускает пустой остаток и ставит форму на текущую клетку U; другие отказывают.
//!
//! После MasterInfo все, кроме Archery, читают EM вхолостую. Огненные варианты
//! затем потребляют первый слот SoulCollect через полноценный End. MIN/MAX/EM
//! и души принадлежат независимому снаряду; Archery игнорирует эти три числа.
//! Add не определяет успешный End1. Только Ball явно кодирует и публикует BF502.
//! Нет prepared или повторного допуска/RP. SlotMap/Vec заменяют указатели/STL.

use super::archeryphalanx::CArcheryPhalanx;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::basemagicphalanx::CBaseMagicPhalanx;
use super::baseprojectilecheck::{base_projectile_attack_path, check_base_projectile_cast};
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{spend_cast_mana, terminal};
use super::soulcollectstate::consume_soul_collect_snapshot;
use super::fireboltphalanx::CFireBoltPhalanx;
use super::fireballphalanx::CFireBallPhalanx;
use super::skillfactory::SkillOwner;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::weaponattack::source_master;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::appserver::summonshape::SummonedSkillShape;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseProjectileKind { Archery, Magic, FireBolt, FireBall }
impl BaseProjectileKind {
    pub(crate) const fn from_skill_id(id: u32) -> Option<Self> {
        match id {
            super::archery::ARCHERY_SKILL_ID => Some(Self::Archery),
            super::basemagic::BASE_MAGIC_SKILL_ID => Some(Self::Magic),
            super::firebolt::FIRE_BOLT_SKILL_ID => Some(Self::FireBolt),
            super::fireball::FIRE_BALL_SKILL_ID => Some(Self::FireBall),
            _ => None,
        }
    }
    pub(crate) const fn owner(self) -> SkillOwner {
        match self {
            Self::Archery => SkillOwner::CArchery, Self::Magic => SkillOwner::CBaseMagic,
            Self::FireBolt => SkillOwner::CFireBolt, Self::FireBall => SkillOwner::CFireBall,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BaseProjectileProgress {
    attack_time_ms: i32,
}
impl BaseProjectileProgress {
    pub(crate) const fn attack_time_ms(&self) -> i32 { self.attack_time_ms }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BaseProjectileExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}
impl BaseProjectileExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started) }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn distance(game: &CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity)) -> Option<i32> {
    let source = game.skill_shape_view(source)?;
    let target = game.skill_shape_view(target)?;
    Some(source.real_distance(Some(target)))
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, source.0, source.1).is_none() { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(kind) = BaseProjectileKind::from_skill_id(skill.id()) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let length = if kind == BaseProjectileKind::FireBall {
        properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE)
    } else {
        let Some(target) = target else { return; };
        let Some(length) = distance(game, source, target) else { return; };
        length as u32
    };
    let mut path = base_projectile_attack_path(game, instance, length);
    if path.is_empty() { return; }
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    let destination = skill.lifecycle().destination();
    skill.lifecycle_mut().set_point_target(destination);
    if let Some(block) = path.iter().position(|cell| cell.2 == 2) {
        if kind == BaseProjectileKind::FireBall { path.truncate(block); } else { return; }
    }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    if kind != BaseProjectileKind::Archery {
        let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    }
    let souls = if matches!(kind, BaseProjectileKind::FireBolt | BaseProjectileKind::FireBall) {
        consume_soul_collect_snapshot(game, source)
    } else { (0, 0) };
    let Some(skill) = game.registered_skill(instance) else { return; };
    let (flight, target, speed) = match kind {
        BaseProjectileKind::FireBall =>
            (0, None, properties.query_property(SKILL_USAGE_SUMMONED_SPEED)),
        BaseProjectileKind::FireBolt => {
            let Some(target) = target.and_then(|target| resolve_state_move_shape(game, target.0, target.1)) else { return; };
            let target = target.shape().identity();
            let Some(flight) = skill.base_projectile_progress().map(BaseProjectileProgress::attack_time_ms) else { return; };
            (flight, Some(target), 0)
        }
        BaseProjectileKind::Archery | BaseProjectileKind::Magic => {
            let Some(flight) = skill.base_projectile_progress().map(BaseProjectileProgress::attack_time_ms) else { return; };
            let Some(target) = target.and_then(|target| resolve_state_move_shape(game, target.0, target.1)) else { return; };
            (flight, Some(target.shape().identity()), 0)
        }
    };
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let level = skill.level();
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = match (kind, target) {
        (BaseProjectileKind::Archery, Some(target)) => SummonedSkillShape::Archery(
            CArcheryPhalanx::new(id, master, started, lifetime, level, flight as u32, target)),
        (BaseProjectileKind::Magic, Some(target)) => SummonedSkillShape::BaseMagic(
            CBaseMagicPhalanx::new(id, master, started, lifetime, level, minimum, maximum,
                element_modifier, flight as u32, target)),
        (BaseProjectileKind::FireBolt, Some(target)) => SummonedSkillShape::FireBolt(
            CFireBoltPhalanx::new(id, master, started, lifetime, level, minimum, maximum,
                element_modifier, target, flight as u32, souls.0, souls.1)),
        (BaseProjectileKind::FireBall, _) => {
            let points = path.iter().map(|cell| (cell.0, cell.1)).collect();
            let phalanx = CFireBallPhalanx::new(id, master, started, lifetime, level, minimum, maximum,
                element_modifier, points, speed, souls.0, souls.1 as u32);
            let position = path.first().map(|cell| (cell.0, cell.1)).or_else(|| {
                let source = resolve_state_move_shape(game, source.0, source.1)?;
                let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
                let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
                Some((x, y))
            });
            if let Some((x, y)) = position {
                let _ = game.spawn_fire_ball_phalanx(source, phalanx, x, y, started, runtime);
            }
            return;
        }
        _ => return,
    };
    let (x, y, _) = path[0];
    let _ = game.spawn_base_projectile(source, phalanx, x, y, started, runtime);
}

fn direction_target(
    game: &CGame, instance: RegisteredSkill, kind: BaseProjectileKind,
    captured_target: (i32, ShapeIdentity),
) -> Option<&crate::gameserver::appserver::moveshape::CMoveShape> {
    let target = match kind {
        BaseProjectileKind::Archery => captured_target,
        BaseProjectileKind::Magic | BaseProjectileKind::FireBolt | BaseProjectileKind::FireBall =>
            resolve_skill_sufferer(game, game.registered_skill(instance)?.lifecycle())?,
    };
    resolve_state_move_shape(game, target.0, target.1)
}

fn target_failure(game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, text: &[u8]) {
    game.update_registered_skill_visual(instance, 10);
    if let Some(player) = player { game.send_skill_system_info(player, text); }
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(kind) = BaseProjectileKind::from_skill_id(skill.id()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity);
    let target = if kind == BaseProjectileKind::FireBall { None } else {
        resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
    };
    let Some(source) = source else {
        if kind != BaseProjectileKind::FireBall { game.update_registered_skill_visual(instance, 10); }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.is_none() && kind != BaseProjectileKind::FireBall {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let is_self = target.is_some_and(|target| std::ptr::eq(source, target));
    let source = (source.shape().get_region_id(), source.shape().identity());
    let target = target.map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if stage == SkillStage::Begin {
        if matches!(kind, BaseProjectileKind::FireBolt | BaseProjectileKind::FireBall)
            && !spend_cast_mana(game, instance, player, &properties)
        { return terminal(QueuedSkillExecutionState::Rejected); }
        if kind != BaseProjectileKind::Archery {
            let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
            if let Some(skill) = game.registered_skill_mut(instance) {
                skill.lifecycle_mut().set_available(can_break != 0);
            }
        }
        if target.is_some_and(|target| game.move_shape_health(target.0, target.1) == Some(0)) {
            target_failure(game, instance, player, b"GS0285");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if is_self {
            if kind != BaseProjectileKind::FireBolt { game.update_registered_skill_visual(instance, 10); }
            target_failure(game, instance, player, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if kind == BaseProjectileKind::Archery {
            let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
            if let Some(skill) = game.registered_skill_mut(instance) {
                skill.lifecycle_mut().set_available(can_break != 0);
            }
            if target.is_some_and(|target| game.move_shape_health(target.0, target.1) == Some(0)) {
                target_failure(game, instance, player, b"GS0285");
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let (target_x, target_y) = if kind == BaseProjectileKind::FireBall {
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            match resolve_skill_sufferer(game, skill.lifecycle())
                .and_then(|target| resolve_state_move_shape(game, target.0, target.1))
            {
                Some(target) => (target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN)),
                None => skill.lifecycle().destination(),
            }
        } else {
            let Some(target) = target else { return terminal(QueuedSkillExecutionState::Rejected); };
            let Some(target_y) = direction_target(game, instance, kind, target)
                .map(|shape| shape.shape().get_tile_y().unwrap_or(i32::MIN))
            else { return terminal(QueuedSkillExecutionState::Rejected); };
            let Some(target_x) = direction_target(game, instance, kind, target)
                .map(|shape| shape.shape().get_tile_x().unwrap_or(i32::MIN))
            else { return terminal(QueuedSkillExecutionState::Rejected); };
            (target_x, target_y)
        };
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if kind != BaseProjectileKind::FireBall
            && let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1)
        { user.set_moveable(false); }
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    if kind == BaseProjectileKind::FireBall {
        game.update_registered_skill_visual(instance, 1);
        if let Some(source) = resolve_state_move_shape(game, source.0, source.1) {
            let _ = source.shape().get_face_position();
        }
        summon(game, instance, source, None, runtime);
        return terminal(QueuedSkillExecutionState::Completed);
    }
    if game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle())).is_none() {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let fresh_target = game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
    let Some(fresh_target) = fresh_target else {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.move_shape_health(fresh_target.0, fresh_target.1) == Some(0) {
        target_failure(game, instance, player, b"GS0285");
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(target) = target else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(length) = distance(game, source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::base_projectile_progress_mut) {
        progress.attack_time_ms = length.wrapping_mul(speed as i32);
    }
    game.update_registered_skill_visual(instance, 1);
    summon(game, instance, source, Some(target), runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_base_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(kind) = game.registered_skill(instance).and_then(|skill| BaseProjectileKind::from_skill_id(skill.id())) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let original_user = game.find_player(player_id).map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        original_user.and_then(|(region, _)| dispatch.object_target()
            .and_then(|target| game.player_skill_begin_object(region, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::BaseProjectile,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            check_base_projectile_cast(game, instance, kind, original_user, target, runtime)
        },
        |dispatch, started| BaseProjectileExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

struct BaseProjectileSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for BaseProjectileSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::BaseProjectile;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;
    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(instance) else { return false; };
        let (region, identity) = skill.lifecycle().user();
        let user = resolve_state_move_shape(game, region, identity)
            .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
        let target = begin_target.resolve(game, skill, false);
        let Some(kind) = BaseProjectileKind::from_skill_id(ID) else { return false; };
        check_base_projectile_cast(game, instance, kind, user, target, runtime)
    }
    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_base_projectile<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<BaseProjectileSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
