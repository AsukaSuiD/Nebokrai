//! Общий цикл совместимых навыков-состояний игрока и монстра.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/stateskill.cpp
//! и Begin/AI подключённого семейства атак, усилений и щитов.
//!
//! Общий Begin записывает базу перед OnBeginSkill; concrete visual loop1 создаётся до
//! CheckCast. Отказ выполняет конкретную диагностику и полный End(0), успех разрешает первый
//! AI. Монстр получает отдельные часы Begin и события Attack. Публикация
//! реального AI/региона сохраняется через вложенные обработчики.
//! Общая обвязка не хранит копии исполнения: payload и visual принадлежат
//! зарегистрированному навыку CMoveShape, ключ SlotMap переживает callbacks
//! и не разрешается повторно по ID после замены экземпляра.
//! End возвращает движение, выполняет AfterUse/reuse и освобождает visual
//! через единый каталог. Наложенные состояния и их пересчёт остаются у навыка.
//! Завершение команды отделено от End и не меняет выбранный ID навыка.
//! Прочие навыки используют ниже только формат сообщений и частичный AfterUse;
//! их специальные Begin/End не подключаются к семейству по одному сходству имени.

use super::kernel::{PlayerSkillExecution, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::monsterai::{MonsterSkillCallOutcome, finish_monster_skill_call};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_owned_skill_begin_object, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;

/// Поле цели в подтверждённых вариантах пакета применения.
pub(crate) enum StateSkillVisualTarget {
    Sufferer,
    SuffererOrUser,
    UserOnly,
}

/// Аргумент конкретного Begin живёт только на стеке вызова. NULL объект
/// не равен прежней S, которую базовый Begin при NULL оставляет неизменной.
pub(crate) enum StateSkillBeginTarget {
    Object(Option<(i32, ShapeIdentity)>),
    Resolved,
}

impl StateSkillBeginTarget {
    pub(crate) fn resolve(self, game: &CGame, skill: &MoveShapeSkill, fallback_to_user: bool) -> Option<(i32, ShapeIdentity)> {
        match self {
            Self::Object(target) => target,
            Self::Resolved => resolve_skill_sufferer(game, skill.lifecycle()).or_else(|| {
                if !fallback_to_user { return None; }
                let (region, identity) = skill.lifecycle().user();
                let user = resolve_state_move_shape(game, region, identity)?.shape();
                Some((user.get_region_id(), user.identity()))
            }),
        }
    }
}

/// Совместимое семейство с общей базой Begin, visual loop1 и отдельным первым AI.
/// Здесь нет хранилища исполнения: payload остаётся в зарегистрированном навыке.
pub(crate) trait RegisteredStateSkill {
    const ID: u32;
    const VISUAL: SkillVisualEffectKind;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 10, 11, 13, 15];
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::Sufferer;
    const BEGIN_FAILURE_VISUAL: Option<u32> = Some(2);

    fn visual_flight_time(_skill: &MoveShapeSkill) -> Option<u32> { None }

    fn player_execution(dispatch: PlayerSkillDispatch, started: u32) -> PlayerSkillExecution {
        let mut kernel = SkillExecutionKernel::begin(dispatch, started);
        kernel.clear_phase_for_end();
        PlayerSkillExecution::State(kernel)
    }

    fn prepare_monster(_skill: &mut MoveShapeSkill) {}

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool;

    /// Конкретный AI сам вызывает полный End на достигнутой ветке.
    /// Внешняя очередь не повторяет его по возвращённому outcome.
    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome;
}

pub(crate) fn state_skill_outcome(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

/// Общий wire-каркас семейства; перечень отказов, fallback и полёт — его
/// реальные игровые различия. Базовый хвост visual исполняет внешний dispatcher.
pub(crate) fn publish_state_skill_visual<Skill: RegisteredStateSkill>(
    game: &CGame, skill: &MoveShapeSkill, mode: u32,
) {
    if skill.id() != Skill::ID || skill.visual_effect().is_none_or(|effect| {
        effect.kind() != Skill::VISUAL || effect.is_ended()
    }) { return; }
    let (user_region, user) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, user_region, user).map(|shape| shape.shape()) else { return; };
    let mut message = CMessage::new(0x000b_fe01);
    if Skill::VISUAL_FAILURES.contains(&mode) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let target = match mode {
        0 => None,
        1 if matches!(Skill::VISUAL_TARGET, StateSkillVisualTarget::UserOnly) => None,
        1 => {
            let target = resolve_skill_sufferer(game, skill.lifecycle())
                .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
                .or_else(|| {
                    if matches!(Skill::VISUAL_TARGET, StateSkillVisualTarget::SuffererOrUser) {
                        resolve_state_move_shape(game, user_region, user)
                    } else { None }
                });
            let Some(target) = target else { return; };
            Some(target.shape())
        }
        _ => return,
    };
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
        if let Some(flight_time) = Skill::visual_flight_time(skill) {
            message.add_ulong(flight_time);
        }
    } else if mode == 0 {
        message.add_long(source.get_direction());
    } else {
        // Щиты адресованы источнику; их пакет заканчивается двумя нулями,
        // не содержит повторной identity и вообще не вызывает GetSufferer.
        message.add_long(0);
        message.add_long(0);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

pub(crate) fn end_state_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, argument: i32, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let (termination, state) = if argument == 0 {
        (SkillTermination::Rejected, QueuedSkillExecutionState::Rejected)
    } else {
        (SkillTermination::Completed, QueuedSkillExecutionState::Completed)
    };
    // Повторный End того же экземпляра после callback допустим. Новый экземпляр
    // с тем же ID никогда не подхватывается вместо захваченного ключа.
    let _ = game.end_registered_instance(address, argument, termination, runtime);
    state_skill_outcome(state)
}

fn begin_state_skill<Skill: RegisteredStateSkill, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill_mut(address) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    skill.replace_visual_effect(SkillVisualEffect::new(Skill::VISUAL, 1));
    if !Skill::check_cast(game, address, begin_target, runtime) {
        if let Some(mode) = Skill::BEGIN_FAILURE_VISUAL {
            game.update_registered_skill_visual(address, mode);
        }
        return end_state_skill(game, address, 0, runtime);
    }
    if let Some(skill) = game.registered_skill_mut(address) {
        let _ = skill.advance_execution(SkillStage::Idle, SkillStage::Begin);
    }
    state_skill_outcome(QueuedSkillExecutionState::Begun)
}

pub(crate) fn execute_player_state_skill<Skill: RegisteredStateSkill, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != Skill::ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    let Some(address) = game.registered_player_skill(player_id, Skill::ID) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let Some(skill) = game.registered_skill(address) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if let Some(previous) = skill.player_dispatch() {
        if previous != dispatch {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        return game.with_published_player_ai(player_id, ai, |game| Skill::run_ai(game, address, runtime));
    }
    let Some(source) = game.find_player(player_id).map(|player| player.shape().get_region_id()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let begin_target = match dispatch {
        PlayerSkillDispatch::Point { .. } => StateSkillBeginTarget::Resolved,
        _ => StateSkillBeginTarget::Object(dispatch.object_target()
            .and_then(|target| game.player_skill_begin_object(source, target))),
    };
    game.with_published_player_ai(player_id, ai, |game| {
        if !game.begin_player_skill_with_combat(player_id, dispatch, runtime.now_milliseconds()) {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(skill) = game.registered_skill(address) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        let execution = Skill::player_execution(dispatch, skill.lifecycle().started_at_ms());
        if !game.begin_player_skill_execution(player_id, execution) {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        begin_state_skill::<Skill, Runtime>(game, address, begin_target, runtime)
    })
}

pub(crate) fn finish_player_state_skill<Skill: RegisteredStateSkill, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI,
    argument: i32, termination: SkillTermination, runtime: &mut Runtime,
) -> bool {
    let Some(address) = game.registered_player_skill(player_id, Skill::ID) else { return false; };
    let Some(dispatch) = game.registered_skill(address).and_then(MoveShapeSkill::player_dispatch) else { return false; };
    game.with_published_player_ai(player_id, ai, |game| {
        let _ = game.end_registered_instance(address, argument, termination, runtime);
    });
    game.finish_registered_player_command(Some(address), ai, dispatch, termination)
}

pub(crate) fn execute_owned_state_skill<Skill: RegisteredStateSkill, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_mut() else { return false; };
    let Some(monster) = region_owner.base().find_monster_by_id(monster_id) else { return false; };
    let source = monster.move_shape().shape().identity();
    let region_id = monster.move_shape().shape().get_region_id();
    if let Some(cast) = monster.current_active_attack_cast(game.skill_factory()) {
        if cast.dispatch().skill_id != Skill::ID { return false; }
        return game.with_published_region(owner, |game| {
            let Some(address) = game.registered_move_shape_skill(region_id, source, Skill::ID) else { return false; };
            let _ = Skill::run_ai(game, address, runtime);
            true
        }).unwrap_or(false);
    }
    let target_object = resolve_owned_skill_begin_object(game, region_owner.base(), target);
    let started = runtime.now_milliseconds();
    if !region_owner.base_mut().find_monster_by_id_mut(monster_id).is_some_and(|monster| {
        monster.prepare_base_attack_cast(target, Skill::ID, skill_level, started, target_object, game.skill_factory())
    }) { return false; }
    let outcome = game.with_published_region(owner, |game| {
        let Some(address) = game.registered_move_shape_skill(region_id, source, Skill::ID) else {
            return MonsterSkillCallOutcome::NotHandled;
        };
        let Some(skill) = game.registered_skill_mut(address) else { return MonsterSkillCallOutcome::NotHandled; };
        let Some(kernel) = skill.monster_kernel_mut() else { return MonsterSkillCallOutcome::NotHandled; };
        kernel.clear_phase_for_end();
        Skill::prepare_monster(skill);
        if begin_state_skill::<Skill, Runtime>(game, address, StateSkillBeginTarget::Object(target_object), runtime)
            .state != QueuedSkillExecutionState::Begun {
            return MonsterSkillCallOutcome::BeginRejected;
        }
        if let Some(monster) = game.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        {
            monster.enqueue_base_attack_cast(runtime.now_milliseconds());
        }
        MonsterSkillCallOutcome::Handled
    }).unwrap_or(MonsterSkillCallOutcome::Handled);
    let Some(region_owner) = owner.as_mut() else { return true; };
    finish_monster_skill_call(game, region_owner.base_mut(), monster_id, outcome, runtime)
}

pub(crate) fn finish_state_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    game.after_use_player_skill(player_id, skill_id, runtime);
}

impl CGame {
    pub(crate) fn send_self_state_skill_failure(
        &self,
        message_type: i32,
        player_id: i32,
        action: u8,
    ) {
        let mut message = CMessage::new(message_type);
        message.add_byte(0);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn send_self_state_skill_cast(
        &mut self,
        message_type: i32,
        player_id: i32,
        skill_id: u32,
        skill_level: i32,
        action: u8,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let identity = player.shape().identity();
        let mut message = CMessage::new(message_type);
        message.add_byte(action);
        message.add_long(skill_id as i32);
        message.base_mut().add_short(skill_level as i16);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        if action == 1 {
            message.add_long(player.shape().get_direction());
        } else {
            message.add_long(identity.object_type);
            message.add_long(identity.id);
            message.add_long(player.shape().get_tile_x().unwrap_or_default());
            message.add_long(player.shape().get_tile_y().unwrap_or_default());
        }
        let _ = self.send_player_shape_around(player_id, None, &message);
    }
}
