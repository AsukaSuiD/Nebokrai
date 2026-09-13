//! Прицельный полёт Strike, YakshaSlash и Seal.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/strike.cpp,
//! yakshaslash.cpp и seal.cpp. Player и Monster используют один зарегистрированный
//! progress; kernel хранит общие U/S, фазу, CAN, prepared и время Begin.
//!
//! Каждый AI захватывает таблицу и полные U/S до callbacks. Seal перед проверкой
//! смерти S дополнительно вызывает её live DoesTargetEffective(U); другие владельцы
//! сохраняют свои исходные допуски. Strike и Seal списывают MP только у Player
//! и публикуют OnChangeStates. Затем CAN→S.Y/X→U.Y/X→направление→visual0→condition.
//! Выпуск ждёт unsigned start+delay, включает Move1 и строит свежий путь.
//! Strike и Seal проверяют дальность и именованное препятствие, Yaksha — только
//! BLOCK_UNFLY. Seal перед путём завершает End(1), когда U+10 ниже S. Flight
//! равен числу клеток, умноженному на время клетки;
//! visual1 предшествует attacking и prepared. Контакт требует attacking
//! и второго абсолютного срока start+delay+flight, затем общего End(1).
//! Calculate и состояние остаются у владельцев навыков, без копии AI.
//!
//! Visual передаёт свежего U и, для выпуска, свежего S без point fallback.
//! NULL S пропускает пакет, но сохраняет базовый visual-tail. End очищает
//! фазу, attacking и flight до свежего U Move1 и общего Attack End.
//! Локальный Vec владеет путём; никаких региональных форм полёта не создаётся.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{CastPathBlock, check_skill_path, spend_cast_mana, terminal};
use super::seal::{apply_seal_attack, check_seal_cast};
use super::skillfactory::SkillOwner;
use super::strike::{apply_strike_attack, check_strike_cast};
use super::yakshaslash::{apply_yaksha_slash_attack, check_yaksha_slash};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
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

const MISSILE_TIME: u32 = 10_008;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TargetedProjectileProfile {
    Strike,
    YakshaSlash,
    Seal,
}

impl TargetedProjectileProfile {
    const fn from_owner(owner: SkillOwner) -> Option<Self> {
        match owner {
            SkillOwner::CStrike => Some(Self::Strike),
            SkillOwner::CYakshaSlash => Some(Self::YakshaSlash),
            SkillOwner::CSeal => Some(Self::Seal),
            _ => None,
        }
    }

    const fn spends_player_mana(self) -> bool {
        matches!(self, Self::Strike | Self::Seal)
    }

    const fn checks_named_path(self) -> bool {
        matches!(self, Self::Strike | Self::Seal)
    }

    const fn rechecks_target_effectiveness(self) -> bool {
        matches!(self, Self::Seal)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TargetedProjectileProgress {
    pub(crate) attacking: bool,
    pub(crate) flight: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TargetedProjectileExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    progress: TargetedProjectileProgress,
}

impl TargetedProjectileExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), progress: TargetedProjectileProgress::default() }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn progress(&self) -> &TargetedProjectileProgress { &self.progress }
    pub(crate) fn progress_mut(&mut self) -> &mut TargetedProjectileProgress { &mut self.progress }

    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.progress = TargetedProjectileProgress::default();
        true
    }
}

pub(crate) fn run_targeted_projectile_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(profile) = TargetedProjectileProfile::from_owner(skill.owner())
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity);
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let (Some(source), Some(target)) = (source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let targets_self = std::ptr::eq(source, target);
    let user = (source.shape().get_region_id(), source.shape().identity());
    let target = (target.shape().get_region_id(), target.shape().identity());
    let player = (user.1.object_type == 400).then_some(user.1.id);
    if profile.rechecks_target_effectiveness()
        && !game.live_skill_target_attackable_between(user, target)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let target_dead = game.move_shape_health(target.0, target.1) == Some(0);
    if target_dead || (targets_self && profile != TargetedProjectileProfile::Seal) {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player {
            match profile {
                TargetedProjectileProfile::Strike => {
                    game.send_skill_system_info(player, if target_dead { b"GS0285" } else { b"GS0286" });
                }
                TargetedProjectileProfile::Seal if target_dead => {
                    game.send_skill_system_info(player, b"GS0285");
                }
                TargetedProjectileProfile::YakshaSlash | TargetedProjectileProfile::Seal => {}
            }
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if profile.spends_player_mana() && !spend_cast_mana(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(MoveShapeSkill::targeted_projectile_progress).map(|progress| progress.attacking)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if runtime.now_milliseconds() >= started.wrapping_add(delay) {
            if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
            if profile == TargetedProjectileProfile::Seal {
                let Some(source_level) = game.move_shape_level(user.0, user.1) else {
                    return terminal(QueuedSkillExecutionState::Rejected);
                };
                let Some(target_level) = game.move_shape_level(target.0, target.1) else {
                    return terminal(QueuedSkillExecutionState::Rejected);
                };
                if u32::from(source_level) + 10 < u32::from(target_level) {
                    game.update_registered_skill_visual(instance, 2);
                    return terminal(QueuedSkillExecutionState::Completed);
                }
            }
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let path = game.skill_target_path(skill.lifecycle());
            if profile.checks_named_path() {
                if !check_skill_path(game, instance, &properties, &path, player,
                    CastPathBlock::Named { target, message: b"GS0296" })
                { return terminal(QueuedSkillExecutionState::Rejected); }
            } else if path.iter().any(|cell| cell.2 == 2) {
                game.update_registered_skill_visual(instance, 15);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let flight = properties.query_property(MISSILE_TIME).wrapping_mul(path.len() as u32);
            if let Some(progress) = game.registered_skill_mut(instance)
                .and_then(MoveShapeSkill::targeted_projectile_progress_mut)
            { progress.flight = flight; }
            game.update_registered_skill_visual(instance, 1);
            if let Some(skill) = game.registered_skill_mut(instance) {
                if let Some(progress) = skill.targeted_projectile_progress_mut() { progress.attacking = true; }
                skill.lifecycle_mut().mark_prepared();
                let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
                let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
            }
        }
        if game.registered_skill(instance).and_then(MoveShapeSkill::targeted_projectile_progress)
            .is_none_or(|progress| !progress.attacking)
        { return terminal(QueuedSkillExecutionState::Pending); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(progress) = skill.targeted_projectile_progress() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let deadline = skill.lifecycle().started_at_ms().wrapping_add(delay).wrapping_add(progress.flight);
    if runtime.now_milliseconds() < deadline { return terminal(QueuedSkillExecutionState::Pending); }
    match profile {
        TargetedProjectileProfile::Strike => {
            apply_strike_attack(game, instance, user, target, &properties, runtime);
        }
        TargetedProjectileProfile::YakshaSlash => {
            apply_yaksha_slash_attack(game, instance, user, target, runtime);
        }
        TargetedProjectileProfile::Seal => {
            apply_seal_attack(game, instance, user, target, &properties, runtime);
        }
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn execute_player_targeted_projectile<Runtime: GameMainLoopRuntime>(
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
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::TargetedProjectile,
        |game, instance, _, runtime| {
            let owner = game.registered_skill(instance).map(MoveShapeSkill::owner);
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            let accepted = match owner {
                Some(SkillOwner::CStrike) => check_strike_cast(game, instance, original_user, target, runtime),
                Some(SkillOwner::CYakshaSlash) => check_yaksha_slash(game, instance, original_user, target, runtime),
                Some(SkillOwner::CSeal) => check_seal_cast(game, instance, original_user, target, runtime),
                _ => false,
            };
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| TargetedProjectileExecutionState::begin(dispatch, started).into(), run_targeted_projectile_ai,
    )
}

pub(crate) fn publish_targeted_projectile_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CStrike | SkillOwner::CYakshaSlash | SkillOwner::CSeal)
        || skill.visual_effect().is_none_or(|effect|
            effect.kind() != SkillVisualEffectKind::TargetedProjectile || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let failure = matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15)
        && !(skill.owner() == SkillOwner::CSeal && mode == 14);
    if failure {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let target = match mode {
        0 => None,
        1 => {
            let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
            Some(target.shape())
        }
        _ => return,
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if mode == 0 { 1 } else { 2 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
        let Some(progress) = skill.targeted_projectile_progress() else { return; };
        message.add_ulong(progress.flight);
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}
