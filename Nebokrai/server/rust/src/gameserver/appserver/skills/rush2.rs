//! Второй прямой рывок CRush2 (0x7C).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rush2.cpp.
//!
//! Геометрия, проверка меча, расход ресурсов и visual совпадают с Rush;
//! дальность попадания проверяется строго. После попытки Begin/append и
//! отбрасывания создаётся отдельная атака без компонент урона. Она сохраняет
//! конструкторский skill-id, получает свежие сведения об источнике и проходит
//! обычный OnBeenAttacked, даже если Begin или ForceMove не выполнились.

use super::flash::master_info;
use super::rush::{execute_rush, prepare_rush_control, remove_previous_rush_state, rush_knockback};
use super::rushstate2::{begin_primary_rush_2_state, Rush2State, RUSH_2_STATE_ID};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const RUSH_2_SKILL_ID: u32 = 0x7c;

pub(crate) const fn is_rush_2_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == RUSH_2_SKILL_ID
}

pub(super) fn add_rush_2_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), keep: u32, runtime: &mut Runtime,
) {
    let Some((properties, source_region)) = prepare_rush_control(game, address, user, target) else { return; };
    let state = Rush2State::new(keep);
    remove_previous_rush_state(game, target, RUSH_2_STATE_ID);
    let _ = begin_primary_rush_2_state(
        game, target.0, target.1, Some(user), Some(target), state, &mut || runtime.now_milliseconds(),
    );
    rush_knockback(game, &properties, source_region, user, target);
    let Some(source) = game.find_player(user.1.id).filter(|_| user.1.object_type == 400) else { return; };
    let master = master_info(source);
    let contact = AttackInformation::for_master(master);
    game.apply_owned_skill_contact(master, target.1, target.0, contact, runtime);
}

pub(crate) fn execute_player_rush_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rush_2_dispatch(dispatch) {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_rush(game, player_id, instance, dispatch, runtime)
}
