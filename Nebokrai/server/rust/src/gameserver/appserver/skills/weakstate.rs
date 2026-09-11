//! Каноническое состояние ослабления `CWeakState` (`0x12E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/weakstate.cpp`. Состояние хранит прямоугольник призванной
//! области и исходное снижение атаки. Для игрока сохраняется legacy-маска
//! `0xFFFF` и ограничение `INT_MAX`; для монстра применяется то же знаковое
//! вычитание к обеим границам атаки. `CGame` отвечает только за каноническую
//! установку, перерасчёт независимого владельца и around-доставку.
//! Persisted-запись намеренно не содержит координаты области: exact EXE пишет
//! ID, type `2`, нулевой remaining time и attack loss. После загрузки нулевой
//! прямоугольник сохраняет native-поведение до первой проверки выхода.
//! Достигнутый AI обходит исходный набор поколенческих ключей общей арены:
//! повторные записи сохраняются, после удаления и публикаций следующий
//! экземпляр разрешается заново; новые экземпляры в этот проход не входят.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const WEAK_STATE_ID: u32 = 0x12e;
pub(crate) const WEAK_STATE_BYTES: usize = 16;
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

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WEAK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let _state_type = reader.read_u32()?;
        let _remaining_time = reader.read_u32()?;
        Ok(Self::new(reader.read_u32()?, 0, 0, 0, 0))
    }

    pub(crate) fn encoded(self) -> [u8; WEAK_STATE_BYTES] {
        let mut bytes = [0; WEAK_STATE_BYTES];
        for (index, value) in [WEAK_STATE_ID, 2, 0, self.attack_loss].into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
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

pub(crate) fn finish_player_weak_outside<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    _runtime: &mut Runtime,
) -> bool {
    let keys = game.find_player(player_id)
        .map(|player| player.move_shape().applied_state_keys::<WeakState>()).unwrap_or_default();
    let mut ended_any = false;
    for key in keys {
        let ended = game.find_player_mut(player_id).and_then(|player| {
            let region_id = player.server_region_id()?;
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            player.move_shape().applied_state::<WeakState>(key).filter(|state| !state.contains(x, y))?;
            let state = player.move_shape_mut().remove_applied_state_record::<WeakState>(key, WEAK_STATE_BYTES)?;
            Some((region_id, x, y, state))
        });
        let Some((region_id, x, y, state)) = ended else {
            continue;
        };
        send_weak_state_visual(
            game,
            region_id,
            ShapeIdentity {
                object_type: 400,
                id: player_id,
                ex_id: CGuid::GUID_INVALID,
            },
            x,
            y,
            state,
            false,
        );
        let _ = game.update_player_properties(player_id);
        ended_any = true;
    }
    ended_any
}

pub(crate) fn finish_monster_weak_outside(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
) -> bool {
    let keys = game.find_region(region_id)
        .and_then(|owner| owner.base().find_monster_by_id(monster_id))
        .map(|monster| monster.move_shape().applied_state_keys::<WeakState>()).unwrap_or_default();
    let mut ended_any = false;
    for key in keys {
        let ended = if let Some(mut owner) = game.take_region_owner(region_id) {
            let result = owner
                .base_mut()
                .find_monster_by_id_mut(monster_id)
                .and_then(|monster| {
                    let x = monster.move_shape().shape().get_tile_x().ok()?;
                    let y = monster.move_shape().shape().get_tile_y().ok()?;
                    monster.move_shape().applied_state::<WeakState>(key).filter(|state| !state.contains(x, y))?;
                    let state = monster.move_shape_mut().remove_applied_state_record::<WeakState>(key, WEAK_STATE_BYTES)?;
                    Some((x, y, state))
                });
            game.restore_region_owner(owner);
            result
        } else {
            None
        };
        let Some((x, y, state)) = ended else {
            continue;
        };
        send_weak_state_visual(
            game,
            region_id,
            ShapeIdentity {
                object_type: 600,
                id: monster_id,
                ex_id: CGuid::GUID_INVALID,
            },
            x,
            y,
            state,
            false,
        );
        ended_any = true;
    }
    ended_any
}
