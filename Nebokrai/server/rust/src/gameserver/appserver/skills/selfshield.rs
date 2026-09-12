//! Наложение ManaShield и MachineShield на захваченного источника.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/{mana,machine}shield.cpp.
//! Общие Begin, Check, AI, visual и End принадлежат selfstatecast.
//!
//! После visual1 удаляется первый непустой слот нужного ID без фильтра RTTI
//! или ended: End, затем destructor свежего остатка той же позиции.
//! Только после этого сохранённая таблица AI отдаёт параметры нового щита.
//! Primary Begin(U,U) предшествует append; UpdateProperty вызывается затем
//! независимо от результата установки. Ни смерти, ни раннего region-gate нет.
//! Захваченный U остаётся полным CMoveShape: прямой доступ к MP игрока
//! ограничен первой фазой общего caller-а, не наложением состояния.

use super::machineshield::{MACHINE_SHIELD_SKILL_ID, MachineShieldOwner};
use super::manashield::{MANA_SHIELD_SKILL_ID, ManaShieldOwner};
use super::shieldstate::{DefenseShieldState, begin_primary_self_shield_state};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{end_and_destroy_state_at, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) trait SelfShieldOwner {
    const SKILL_ID: u32;

    /// Параметры читаются после End прежнего щита из таблицы начала AI.
    fn create_state(properties: &CSkillBaseProperties) -> DefenseShieldState;
}

pub(crate) const fn is_self_shield_skill(skill_id: u32) -> bool {
    matches!(skill_id, MANA_SHIELD_SKILL_ID | MACHINE_SHIELD_SKILL_ID)
}

pub(super) fn apply_self_shield_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), skill_id: u32,
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    match skill_id {
        MANA_SHIELD_SKILL_ID => apply_state::<ManaShieldOwner, Runtime>(game, source, properties, runtime),
        MACHINE_SHIELD_SKILL_ID => apply_state::<MachineShieldOwner, Runtime>(game, source, properties, runtime),
        _ => {}
    }
}

fn apply_state<Owner: SelfShieldOwner, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    if let Some((position, _)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == Owner::SKILL_ID))
    {
        let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    }
    let state = Owner::create_state(properties);
    let _ = begin_primary_self_shield_state(
        game, source.0, source.1, Some(source), Some(source), state,
        &mut || runtime.now_milliseconds(),
    );
    let _ = game.update_move_shape_properties(source.0, source.1);
}
