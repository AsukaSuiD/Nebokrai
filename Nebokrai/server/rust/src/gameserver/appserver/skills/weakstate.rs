//! Каноническое состояние ослабления `CWeakState` (`0x12E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/weakstate.cpp`. Состояние хранит прямоугольник призванной
//! области и исходное снижение атаки. Для игрока сохраняется legacy-маска
//! `0xFFFF` и ограничение `INT_MAX`; для монстра применяется то же знаковое
//! вычитание к обеим границам атаки. `CGame` отвечает только за каноническую
//! установку, перерасчёт независимого владельца и around-доставку.
//! Persisted-запись не содержит координаты: Serialize 0x00606FC0 пишет ID,
//! raw type, остаток срока и attack loss. Unserialize 0x006071F0 читает type,
//! затем один clock для timestamp и сохраняет remaining/attack loss.
//! Runtime-конструктор 0x00606CA0 задаёт type=2 и keep=0; DB сохраняет
//! остальные типы. AI 0x00606EF0 у type=1 читает часы для строгого срока,
//! у type=2 проверяет область без часов, прочие типы сразу вызывают End.
//! SetRegion 0x00606FA0 вызывает End только у type=2. Для других типов
//! setter сохраняет sufferer-region у того же экземпляра общей арены.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! Exact vtable 0x006621B4: End 0x005FD420 выполняет visual →
//! GetSufferer → RemoveState. Прямой End не проверяет выход из области;
//! Обе ветви AI и SetRegion используют тот же exact-key End.

//! Restart воспроизводит только Begin(NULL, holder) (0x00607150):
//! базовый Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Begin создаёт принадлежащий записи loop=1 visual без немедленного пакета.

//! OnUpdateProperties 0x00607020 после GetSufferer выполняет существующий
//! visual Update(0) перед RTTI-формулой. BFE03 читает client-time через
//! 0x00605E10 и additional=0, затем base visual tail. Игрок ограничивает loss
//! текущей атакой и сужает до WORD; монстр складывает отрицательный полный
//! DWORD loss с обоими модификаторами без предварительного ограничения.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
};
use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const WEAK_STATE_ID: u32 = 0x12e;
pub(crate) const WEAK_STATE_BYTES: usize = 16;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WeakState {
    state_type: u32,
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_loss: u32,
    center_x: i32,
    center_y: i32,
    length: i32,
    height: i32,
}

impl WeakState {
    pub(crate) const fn new(attack_loss: u32, center_x: i32, center_y: i32, length: i32, height: i32) -> Self {
        Self { state_type: 2, started_at_ms: 0, keep_time_ms: 0, attack_loss, center_x, center_y, length, height }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WEAK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let state_type = reader.read_u32()?;
        let keep_time_ms = reader.read_u32()?;
        Ok(Self {
            state_type,
            started_at_ms: now_ms,
            keep_time_ms,
            ..Self::new(reader.read_u32()?, 0, 0, 0, 0)
        })
    }

    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; WEAK_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    pub(crate) fn encoded_for_install(self) -> [u8; WEAK_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; WEAK_STATE_BYTES] {
        let mut bytes = [0; WEAK_STATE_BYTES];
        for (index, value) in [WEAK_STATE_ID, self.state_type, remaining, self.attack_loss].into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub(crate) const fn attack_loss(self) -> u32 { self.attack_loss }
    pub(crate) const fn skill_id(self) -> u32 { WEAK_STATE_ID }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

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

pub(crate) fn update_weak_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<WeakState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).copied()
    else { return false; };
    if target.object_type == 600 {
        if let Some(monster) = game.find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id)) {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.minimum_attack = modifiers.minimum_attack.wrapping_sub(state.attack_loss as i32);
            modifiers.maximum_attack = modifiers.maximum_attack.wrapping_sub(state.attack_loss as i32);
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|properties| state.apply_to_player(properties));
        }
    }
    true
}

pub(crate) fn restart_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, region_id, holder) else { return false };
    let Some(state) = shape.applied_state::<WeakState>(key) else { return false };
    match state.state_type {
        1 => {
            if state.started_at_ms.wrapping_add(state.keep_time_ms) >= now() {
                return false;
            }
        }
        2 => {
            let (Ok(x), Ok(y)) = (shape.shape().get_tile_x(), shape.shape().get_tile_y()) else {
                return false;
            };
            if state.contains(x, y) {
                return false;
            }
        }
        _ => {}
    }
    end_weak_state(game, region_id, holder, key)
}

pub(crate) fn set_weak_state_region(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) {
    let Some(state_type) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key))
        .map(|state| state.state_type)
    else { return; };
    if state_type == 2 {
        let _ = end_weak_state(game, region_id, holder, key);
    } else if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        let _ = shape.set_applied_state_sufferer_region(key, region_id);
    }
}

pub(crate) fn end_weak_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WeakState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| {
            shape.remove_applied_state_record::<WeakState>(key, WEAK_STATE_BYTES)
        }).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}
