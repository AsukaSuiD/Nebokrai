//! Каноническое состояние божественного благословения `CGodBlessState` (`0x12F`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godblessstate.cpp`. Состояние владеет сроком и тремя
//! прибавками. Игрок сохраняет сужение прибавок до `u16` и ограничение атаки
//! `INT_MAX`; монстр применяет исходное wrapping-сложение полных `u32`.
//! `CGame` только координирует независимых владельцев и around-доставку.
//! Обе идентичности используют общую 20-байтовую persisted-запись: ID,
//! remaining time и три прибавки; загрузка активируется при spatial login.
//! Обе concrete vtable направляют `GetRemainedTime` на точное тело
//! `0x00601480` с отдельным вторым чтением часов для положительного остатка.
//! Достигнутый AI обходит исходный набор поколенческих ключей общей арены:
//! повторные записи сохраняются, после удаления и публикаций следующий
//! экземпляр разрешается заново; новые экземпляры в этот проход не входят.

use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const GOD_BLESS_STATE_ID: u32 = 0x12f;
pub(crate) const GOD_BLESS_STATE_BYTES: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GodBlessState {
    skill_id: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    minimum_attack_gain: u32,
    maximum_attack_gain: u32,
    element_gain: u32,
}

impl GodBlessState {
    pub(crate) const fn new(skill_id: u32, started_at_ms: u32, keep_time_ms: u32, minimum_attack_gain: u32, maximum_attack_gain: u32, element_gain: u32) -> Self {
        debug_assert!(matches!(skill_id, GOD_BLESS_STATE_ID | super::godblessstate2::GOD_BLESS_STATE_2_ID));
        Self { skill_id, started_at_ms, keep_time_ms, minimum_attack_gain, maximum_attack_gain, element_gain }
    }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        if !matches!(skill_id, GOD_BLESS_STATE_ID | super::godblessstate2::GOD_BLESS_STATE_2_ID) {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(skill_id, 0, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?, reader.read_u32()?))
    }
    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
    pub(crate) const fn skill_id(self) -> u32 { self.skill_id }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
    pub(crate) fn encoded_for_install(self) -> [u8; GOD_BLESS_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; GOD_BLESS_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; GOD_BLESS_STATE_BYTES] {
        let mut bytes = [0; GOD_BLESS_STATE_BYTES];
        for (index, value) in [self.skill_id, remaining_time_ms, self.minimum_attack_gain, self.maximum_attack_gain, self.element_gain].into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
    pub(crate) fn apply_to_player(self, mut properties: PlayerCombatProperties) -> PlayerCombatProperties {
        properties.minimum_attack = properties.minimum_attack.wrapping_add(self.minimum_attack_gain as u16 as u32).min(i32::MAX as u32);
        properties.maximum_attack = properties.maximum_attack.wrapping_add(self.maximum_attack_gain as u16 as u32).min(i32::MAX as u32);
        properties.element_modify = properties.element_modify.wrapping_add(self.element_gain as u16 as i32);
        properties
    }
    pub(crate) const fn apply_to_monster(self, minimum: u32, maximum: u32, element: i32) -> (u32, u32, i32) {
        (minimum.wrapping_add(self.minimum_attack_gain), maximum.wrapping_add(self.maximum_attack_gain), element.wrapping_add(self.element_gain as i32))
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической around-доставки")]
pub(crate) fn send_god_bless_state_visual(game: &mut CGame, region_id: i32, target: ShapeIdentity, tile_x: i32, tile_y: i32, state: GodBlessState, begin: bool, now_ms: u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(|| now_ms)); message.add_long(0); }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn finish_player_god_bless<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    _runtime: &mut Runtime,
) -> bool {
    let keys = game.find_player(player_id)
        .map(|player| player.move_shape().applied_state_keys::<GodBlessState>()).unwrap_or_default();
    let mut ended_any = false;
    for key in keys {
        if !game.find_player(player_id)
            .and_then(|player| player.move_shape().applied_state::<GodBlessState>(key))
            .is_some_and(|state| state.expired(now_ms)) {
            continue;
        }
        let ended = game.find_player_mut(player_id).and_then(|player| {
            let region = player.server_region_id()?;
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            let state = player.move_shape_mut().remove_applied_state_record::<GodBlessState>(key, GOD_BLESS_STATE_BYTES)?;
            Some((region, x, y, state))
        });
        let Some((region, x, y, state)) = ended else {
            continue;
        };
        send_god_bless_state_visual(
            game,
            region,
            ShapeIdentity {
                object_type: 400,
                id: player_id,
                ex_id: CGuid::GUID_INVALID,
            },
            x,
            y,
            state,
            false,
            now_ms,
        );
        let _ = game.update_player_properties(player_id);
        ended_any = true;
    }
    ended_any
}

pub(crate) fn finish_monster_god_bless(
    game: &mut CGame,
    region_id: i32,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let keys = game.find_region(region_id)
        .and_then(|owner| owner.base().find_monster_by_id(monster_id))
        .map(|monster| monster.move_shape().applied_state_keys::<GodBlessState>()).unwrap_or_default();
    let mut ended_any = false;
    for key in keys {
        let ended = if let Some(mut owner) = game.take_region_owner(region_id) {
            let result = owner
                .base_mut()
                .find_monster_by_id_mut(monster_id)
                .and_then(|monster| {
                    let x = monster.move_shape().shape().get_tile_x().ok()?;
                    let y = monster.move_shape().shape().get_tile_y().ok()?;
                    monster.move_shape().applied_state::<GodBlessState>(key).filter(|state| state.expired(now_ms))?;
                    let state = monster.move_shape_mut().remove_applied_state_record::<GodBlessState>(key, GOD_BLESS_STATE_BYTES)?;
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
        send_god_bless_state_visual(
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
            now_ms,
        );
        ended_any = true;
    }
    ended_any
}
