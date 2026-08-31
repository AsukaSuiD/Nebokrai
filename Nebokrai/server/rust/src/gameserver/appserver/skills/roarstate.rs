//! Каноническое состояние подавления `CRoarState` (`0x83`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/roarstate.cpp`. Состояние до строгого истечения срока
//! уменьшает обе границы физической атаки и дополнительную стихийную атаку,
//! ограничивая каждое уменьшение текущим значением. Визуальные сообщения
//! сохраняют `0xBFE03/0xBFE04`; порядок относительно других достигнутых
//! состояниями свойств принадлежит `CanonicalStateStorage`. Vtable exact EXE
//! подтверждает общий с `CBlindState` клиентский срок по `0x005F2CD0`.
//!
//! Не достигнуто: восстановление `CRoarState::Unserialize` из старого DB-потока.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const ROAR_STATE_ID: u32 = 0x83;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RoarState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_loss: i32,
    element_attack_loss: i32,
}

impl RoarState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, attack_loss: i32, element_attack_loss: i32) -> Self {
        Self { started_at_ms, keep_time_ms, attack_loss, element_attack_loss }
    }
    pub(crate) const fn skill_id(self) -> u32 { ROAR_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms
    }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
    pub(crate) fn apply_to_player(self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        let attack_loss = self.attack_loss.max(0) as u32;
        let minimum_loss = properties.minimum_attack.min(attack_loss) & 0xffff;
        let maximum_loss = properties.maximum_attack.min(attack_loss) & 0xffff;
        let element_loss = properties.add_element_attack.min(self.element_attack_loss.max(0) as u32);
        properties.minimum_attack = properties.minimum_attack.wrapping_sub(minimum_loss).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_sub(maximum_loss).min(i32::MAX as u32);
        properties.add_element_attack = properties.add_element_attack.wrapping_sub(element_loss);
        properties
    }
    pub(crate) const fn apply_to_monster(self, minimum: u32, maximum: u32, element: i32) -> (u32, u32, i32) {
        let attack_loss = self.attack_loss as u32;
        let element_loss = self.element_attack_loss;
        (
            minimum.wrapping_sub(if minimum < attack_loss { minimum } else { attack_loss }),
            maximum.wrapping_sub(if maximum < attack_loss { maximum } else { attack_loss }),
            element.wrapping_sub(if element < element_loss { element } else { element_loss }),
        )
    }
}

pub(crate) fn send_roar_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    x: i32,
    y: i32,
    state: RoarState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(ROAR_STATE_ID as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, x, y, &message);
}

pub(crate) fn finish_player_roar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    _runtime: &mut Runtime,
) -> bool {
    let ended = game.find_player_mut(player_id).and_then(|player| {
        let region_id = player.server_region_id()?;
        let x = player.shape().get_tile_x().ok()?;
        let y = player.shape().get_tile_y().ok()?;
        let identity = player.shape().identity();
        let state = player.take_expired_roar_state(now_ms)?;
        Some((region_id, x, y, identity, state))
    });
    let Some((region_id, x, y, identity, state)) = ended else {
        return false;
    };
    send_roar_state_visual(game, region_id, identity, x, y, state, false, || now_ms);
    let _ = game.update_player_properties(player_id);
    true
}

pub(crate) fn finish_monster_roar(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let ended = if let Some(mut owner) = game.take_region_owner(region_id) {
        let ended = owner
            .base_mut()
            .find_monster_by_id_mut(monster_id)
            .and_then(|monster| {
                let x = monster.move_shape().shape().get_tile_x().ok()?;
                let y = monster.move_shape().shape().get_tile_y().ok()?;
                let identity = monster.move_shape().shape().identity();
                let state = monster.move_shape_mut().take_expired_roar_state(now_ms)?;
                Some((x, y, identity, state))
            });
        game.restore_region_owner(owner);
        ended
    } else {
        None
    };
    let Some((x, y, identity, state)) = ended else {
        return false;
    };
    send_roar_state_visual(game, region_id, identity, x, y, state, false, || now_ms);
    true
}

// Сохранён только недостигнутый DB-контракт исходного владельца.
// `CRoarState::Unserialize` читает три `DWORD`: длительность, снижение атаки и
// снижение стихийной атаки, после чего начинает новый срок по системным часам.
// Подключать его без вызывающей цепочки старого кодека состояний нельзя.
