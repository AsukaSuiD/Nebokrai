//! Каноническое состояние ослабления `CWeakState` (`0x12E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/weakstate.cpp`. Состояние хранит прямоугольник призванной
//! области и исходное снижение атаки. Для игрока сохраняется legacy-маска
//! `0xFFFF` и ограничение `INT_MAX`; для монстра применяется то же знаковое
//! вычитание к обеим границам атаки. `CGame` отвечает только за каноническую
//! установку, перерасчёт независимого владельца и around-доставку.
//! DB-восстановление без координат области не материализуется: неизвестная
//! запись остаётся в закрытом legacy codec `CanonicalStateStorage`.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const WEAK_STATE_ID: u32 = 0x12e;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WeakState {
    attack_loss: u32,
    center_x: i32,
    center_y: i32,
    length: i32,
    height: i32,
}

impl WeakState {
    pub(crate) const fn new(attack_loss: u32, center_x: i32, center_y: i32, length: i32, height: i32) -> Self {
        Self { attack_loss, center_x, center_y, length, height }
    }

    pub(crate) const fn attack_loss(self) -> u32 { self.attack_loss }
    pub(crate) const fn skill_id(self) -> u32 { WEAK_STATE_ID }

    pub(crate) const fn contains(self, tile_x: i32, tile_y: i32) -> bool {
        let start_x = self.center_x.wrapping_sub(self.length / 2);
        let start_y = self.center_y.wrapping_sub(self.height / 2);
        tile_x >= start_x
            && tile_x < start_x.wrapping_add(self.length)
            && tile_y >= start_y
            && tile_y < start_y.wrapping_add(self.height)
    }

    pub(crate) fn apply_to_player(self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        let minimum_loss = properties.minimum_attack.min(self.attack_loss) & 0xffff;
        let maximum_loss = properties.maximum_attack.min(self.attack_loss) & 0xffff;
        properties.minimum_attack = properties.minimum_attack.wrapping_sub(minimum_loss).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_sub(maximum_loss).min(i32::MAX as u32);
        properties
    }

    pub(crate) const fn apply_to_monster(self, minimum: u32, maximum: u32) -> (u32, u32) {
        (minimum.wrapping_sub(self.attack_loss), maximum.wrapping_sub(self.attack_loss))
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической around-доставки")]
pub(crate) fn send_weak_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: WeakState, begin: bool) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(WEAK_STATE_ID as i32);
    if begin {
        message.add_long(0);
        message.add_long(state.attack_loss() as i32);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}
