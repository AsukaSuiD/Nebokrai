//! Направленный громовой удар ThunderBlow2 (0x14D).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderblow2.cpp/.h.
//!
//! Зарегистрированный Attack Begin создаёт loop1-visual перед Check; отказ
//! вызывает End(0) без дополнительного пакета. Check требует отдельную S,
//! проверяет reuse, длину локального пути и ненулевую цену MP. Движение не
//! блокируется. AI сохраняет таблицу свойств и исходных U/S на текущий проход;
//! после списания MP идут OnChangeStates, CAN, направление и visual0.
//!
//! Выпуск и попадание отдельно читают абсолютный unsigned срок start+delay.
//! При выпуске identity-цель превращается в точку, но отбрасывание и удар
//! используют S, захваченную в начале AI. Визуальный ресурс заново разрешает S.
//! Отсутствие региона источника отменяет только отбрасывание. Локальные пути
//! не сохраняются между AI. End сбрасывает фазу и attacking перед Attack-base;
//! движения он не возвращает. Неиспользуемое исходное поле missile всегда
//! равно нулю и не материализуется. Формула и raw-контакт без RP принадлежат
//! impactattack, общий lifecycle и поколенческий ключ — playercast.

use super::baseattack::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::impactattack::{apply_thunder_blow_2_attack, knock_back_impact_target};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
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
use crate::public::tools::get_line_direction;

pub(crate) const THUNDER_BLOW_2_SKILL_ID: u32 = 0x14d;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const PILLAR_STATE_ID: u32 = 0x74;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ThunderBlow2Execution {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    attacking_started: bool,
}

impl ThunderBlow2Execution {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), attacking_started: false }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.attacking_started = false;
        true
    }
}

fn outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code {
        10 => b"GS0286",
        11 => b"GS0290",
        13 => b"GS0278",
        _ => return,
    };
    game.send_skill_system_info(player_id, text);
}

fn mana_failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, properties: &CSkillBaseProperties) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    if target.is_none_or(|(_, target)| target == player.shape().identity()) {
        failure(game, instance, player_id, 10);
        return false;
    }
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        if maximum < path.len() as u32 {
            failure(game, instance, player_id, 11);
            return false;
        }
    }
    if properties.query_property(USER_MP_LOSE) == 0 { return false; }
    let mana = player.mana();
    let loss = properties.query_property(USER_MP_LOSE);
    if (mana.wrapping_sub(loss) as i32) < 0 {
        mana_failure(game, instance, player_id, &properties);
        return false;
    }
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return outcome(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return outcome(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle());
    let (Some(source), Some(target)) = (source, target) else { return outcome(QueuedSkillExecutionState::Rejected); };
    if stage == SkillStage::Begin {
        if source.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(source.1.id) else { return outcome(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                mana_failure(game, instance, source.1.id, &properties);
                return outcome(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
            game.publish_player_states(source.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
        let destination = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map_or_else(|| skill.lifecycle().destination(), |target| {
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            });
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return outcome(QueuedSkillExecutionState::Rejected); };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<ThunderBlow2Execution>()).map(|state| state.attacking_started)
    else { return outcome(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(skill) = game.registered_skill(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
        let started = skill.lifecycle().started_at_ms();
        if runtime.now_milliseconds() < started.wrapping_add(delay) { return outcome(QueuedSkillExecutionState::Pending); }
        let saved_target = skill.lifecycle().sufferer().1;
        if saved_target.object_type != 0 && saved_target.id != 0 {
            let fresh_target = resolve_skill_sufferer(game, skill.lifecycle());
            let Some((region, identity)) = fresh_target
                .filter(|(region, identity)| game.move_shape_health(*region, *identity).is_some_and(|hp| hp != 0))
            else {
                game.update_registered_skill_visual(instance, 10);
                return outcome(QueuedSkillExecutionState::Rejected);
            };
            let Some(target_shape) = resolve_state_move_shape(game, region, identity) else { return outcome(QueuedSkillExecutionState::Rejected); };
            let x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
            let y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
            let Some(skill) = game.registered_skill_mut(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
            skill.lifecycle_mut().set_point_target((x, y));
        }
        let Some(skill) = game.registered_skill(instance) else { return outcome(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
            if maximum < path.len() as u32 {
                if source.1.object_type == PLAYER_TYPE { failure(game, instance, source.1.id, 11); }
                else { game.update_registered_skill_visual(instance, 11); }
                return outcome(QueuedSkillExecutionState::Rejected);
            }
        }
        if let Some(region_id) = resolve_state_move_shape(game, source.0, source.1)
            .filter(|source| source.shape().is_assigned_to_server_region())
            .map(|source| source.shape().get_region_id())
            .filter(|region| game.find_region(*region).is_some())
        {
            let target_level = game.move_shape_level(target.0, target.1);
            let source_level = game.move_shape_level(source.0, source.1);
            if target_level.zip(source_level).is_some_and(|(target, source)| target <= source)
                && resolve_state_move_shape(game, target.0, target.1)
                    .is_some_and(|target| !target.has_state_by_skill_id(PILLAR_STATE_ID))
            {
                knock_back_impact_target(game, source, target, region_id, &properties);
            }
        }
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<ThunderBlow2Execution>()) {
            state.attacking_started = true;
        }
        drop(path);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return outcome(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return outcome(QueuedSkillExecutionState::Pending); }
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0)
        || resolve_state_move_shape(game, target.0, target.1)
            .map(|target| target.shape().get_region_id())
            .is_none_or(|region| !game.live_skill_target_attackable(region, source.1, target.1))
    {
        game.update_registered_skill_visual(instance, 3);
        return outcome(QueuedSkillExecutionState::Rejected);
    }
    apply_thunder_blow_2_attack(game, instance, source, target, runtime);
    outcome(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_thunder_blow_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != THUNDER_BLOW_2_SKILL_ID { return outcome(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ThunderBlow2,
        |game, instance, player_id, runtime| {
            let target = game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
            check_cast(game, instance, player_id, target, runtime)
        },
        |dispatch, started| ThunderBlow2Execution::begin(dispatch, started).into(), run_ai,
    )
}
