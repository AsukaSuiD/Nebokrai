//! Самонакладываемые CManaShield и CMachineShield: gameserver.exe + GameServer.pdb,
//! исходные owner-ы appserver/skills/{mana,machine}shield.cpp.
//! Общая обвязка stateskill владеет Begin, visual, фазой и полным End.
//! Здесь остаются двойная проверка MP и наложение щита после задержки.
//! Исходный caller требует CPlayer: прямые GetMP/SetMP/SendSystemInfo не являются
//! виртуальным контрактом монстра. Чужой unchecked layout не имитируется.
//! Состояние принадлежит арене источника и продолжает жить после End навыка.

use super::kernel::{SkillStage, SkillTermination, skill_is_restored};
use super::machineshield::{MACHINE_SHIELD_SKILL_ID, MachineShieldOwner};
use super::manashield::{MANA_SHIELD_SKILL_ID, ManaShieldOwner};
use super::shieldstate::{DefenseShieldState, begin_primary_self_shield_state};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_player_state_skill, finish_player_state_skill, publish_state_skill_visual,
    state_skill_outcome,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::end_and_destroy_state_at;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

const MP_LOSS: u32 = 2;
const DELAY: u32 = 10_001;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

pub(crate) trait SelfShieldOwner {
    const SKILL_ID: u32;
    const VISUAL: SkillVisualEffectKind;

    /// Параметры читаются после End прежнего щита из таблицы начала AI.
    fn create_state(properties: &CSkillBaseProperties) -> DefenseShieldState;
}

impl<Owner: SelfShieldOwner> RegisteredStateSkill for Owner {
    const ID: u32 = Owner::SKILL_ID;
    const VISUAL: SkillVisualEffectKind = <Owner as SelfShieldOwner>::VISUAL;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 8, 13, 14];
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::UserOnly;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        check_cast::<Owner, Runtime>(game, instance, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        match execute_stage::<Owner, Runtime>(game, instance, runtime) {
            Some(argument) => end_state_skill(game, instance, argument, runtime),
            None => state_skill_outcome(QueuedSkillExecutionState::Pending),
        }
    }
}

pub(crate) const fn is_self_shield_skill(skill_id: u32) -> bool {
    matches!(skill_id, MANA_SHIELD_SKILL_ID | MACHINE_SHIELD_SKILL_ID)
}

pub(crate) fn publish_self_shield_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    match skill.id() {
        MANA_SHIELD_SKILL_ID => publish_state_skill_visual::<ManaShieldOwner>(game, skill, mode),
        MACHINE_SHIELD_SKILL_ID => publish_state_skill_visual::<MachineShieldOwner>(game, skill, mode),
        _ => {}
    }
}

pub(crate) fn cancel_player_self_shield_dispatch<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, skill_id: u32, player_ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    match skill_id {
        MANA_SHIELD_SKILL_ID => finish_player_state_skill::<ManaShieldOwner, Runtime>(
            game, player_id, player_ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
        ),
        MACHINE_SHIELD_SKILL_ID => finish_player_state_skill::<MachineShieldOwner, Runtime>(
            game, player_id, player_ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
        ),
        _ => false,
    }
}

pub(crate) fn execute_player_self_shield_dispatch<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    match dispatch.skill_id() {
        MANA_SHIELD_SKILL_ID => execute_player_state_skill::<ManaShieldOwner, Runtime>(
            game, player_id, dispatch, player_ai, runtime,
        ),
        MACHINE_SHIELD_SKILL_ID => execute_player_state_skill::<MachineShieldOwner, Runtime>(
            game, player_id, dispatch, player_ai, runtime,
        ),
        _ => state_skill_outcome(QueuedSkillExecutionState::Rejected),
    }
}

fn check_cast<Owner: SelfShieldOwner, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let (_, user) = skill.lifecycle().user();
    if user.object_type != 400 || game.find_player(user.id).is_none() { return false; }
    let Some(properties) = game.skill_base_properties(Owner::SKILL_ID, skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(user.id, b"GS0278");
        return false;
    }
    // Нулевой расход вообще не читает MP; ненулевой проверяется знаком
    // wrapping-разности, а не unsigned сравнением двух исходных величин.
    if properties.query_property(MP_LOSS) != 0 {
        let mana = game.find_player(user.id).expect("источник CheckCast разрешён").mana();
        let mp_loss = properties.query_property(MP_LOSS);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            game.update_registered_skill_visual(instance, 7);
            game.send_skill_system_info_with_unsigned(user.id, b"GS0288", properties.query_property(MP_LOSS));
            return false;
        }
    }
    if let Some(player) = game.find_player_mut(user.id) { player.set_skill_moveable(false); }
    true
}

/// Some(0/1) задаёт точный End; None оставляет начатый навык следующему AI.
fn execute_stage<Owner: SelfShieldOwner, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> Option<i32> {
    let skill = game.registered_skill(instance)?;
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) { return None; }
    let Some(properties) = game.skill_base_properties(Owner::SKILL_ID, skill.level()).cloned() else { return Some(0); };
    let (_, user) = skill.lifecycle().user();
    if user.object_type != 400 || game.find_player(user.id).is_none() { return Some(0); }
    if skill.execution_stage() == Some(SkillStage::Begin) {
        let mana = game.find_player(user.id)?.mana();
        let mp_loss = properties.query_property(MP_LOSS);
        let remaining = mana.wrapping_sub(mp_loss);
        if (remaining as i32) < 0 {
            game.update_registered_skill_visual(instance, 7);
            game.send_skill_system_info_with_unsigned(user.id, b"GS0288", properties.query_property(MP_LOSS));
            return Some(0);
        }
        game.find_player_mut(user.id)?.set_mana(remaining);
        let _ = game.publish_player_states(user.id);
        // OnChangeStates предшествует CAN. Сохранена таблица начала AI,
        // но параметры читаются в исходных поздних позициях после callbacks.
        let can_break = properties.query_property(CAN_BREAK);
        game.registered_skill_mut(instance)?.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(DELAY);
    let started = game.registered_skill(instance)?.lifecycle().started_at_ms();
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return None; }
    game.update_registered_skill_visual(instance, 1);
    let player = game.find_player(user.id)?;
    let region = player.shape().get_region_id();
    let identity = player.shape().identity();
    let previous = player.move_shape().find_state_position(|state| state.state_id() == Owner::SKILL_ID);
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, region, identity, position);
    }
    let state = Owner::create_state(&properties);
    // Новый щит начинается после полного старого End и добавляется в хвост.
    // Повторной замены по ID нет: callback мог добавить другой экземпляр.
    let _ = begin_primary_self_shield_state(
        game, region, identity, Some((region, identity)), Some((region, identity)),
        state, &mut || runtime.now_milliseconds(),
    );
    let _ = game.update_player_properties(user.id);
    Some(1)
}
