//! Каноническое состояние связывания `CBoaLockState` (`0xD2`).
//! Object Begin (0x005FB860) требует sufferer, но допускает NULL user.
//! restart_boa_lock_state: base Begin → visual SetRun(1)/Update(0) и его
//! base tail → move-lock. Timestamp не меняется: его задаёт
//! Unserialize 0x005EAAC0 отдельным clock до чтения оставшегося срока.
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Direct End (vtable 0x006610DC +0x1C, тело 0x005FB800) сначала
//! отправляет visual, затем снимает move-lock и удаляет точный ключ.
//! RemoveState вызывает общий virtual UpdateProperty; timer и Cure используют
//! этот же хвост без подмены direct End принудительным истечением.
//! Vtable 0x006610dc, слот +0x0c: CBlindState::AI (0x005d5ba0).
//! Срок проверяется как start.wrapping_add(keep) < now, включая keep == 0;
//! elapsed-сравнение не сохраняет исходный переход DWORD через ноль.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/boalockstate.cpp`. Состояние запрещает только движение,
//! заменяет прежний экземпляр того же ID с end-пакетом перед begin-пакетом,
//! использует строгую беззнаковую проверку срока и публикует точные
//! `0xBFE03/0xBFE04`. Единственный экземпляр принадлежит
//! `CanonicalStateStorage`; сырой state-вектор не дублируется. Vtable exact
//! EXE подтверждает общий с `CBlindState` клиентский срок по `0x005F2CD0` и
//! serializer-пару `0x005F51E0/0x005EAAC0`: persisted-запись
//! `ID + remaining time` занимает 8 байт. Spatial login восстанавливает
//! вложенный запрет движения.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const BOA_LOCK_STATE_ID: u32 = 0xd2;
pub(crate) const BOA_LOCK_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BoaLockState { started_at_ms: u32, keep_time_ms: u32 }
impl BoaLockState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self { Self { started_at_ms, keep_time_ms } }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BOA_LOCK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }
    pub(crate) fn encoded_for_install(self) -> [u8; BOA_LOCK_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; BOA_LOCK_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; BOA_LOCK_STATE_BYTES] {
        let mut bytes = [0; BOA_LOCK_STATE_BYTES];
        bytes[..4].copy_from_slice(&BOA_LOCK_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }
    pub(crate) const fn skill_id(self) -> u32 { BOA_LOCK_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32 }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_boa_lock_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, x: i32, y: i32, state: BoaLockState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32); if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); } let _ = game.send_shape_position_around(region_id, x, y, &message);
}
fn send_monster_visual(game: &CGame, region: &CServerRegion, shape: &crate::gameserver::appserver::shape::CShape, state: BoaLockState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let identity = shape.identity(); let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32); if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); } let _ = game.send_game_shape_around(region, shape, None, &message);
}
pub(crate) fn replace_player_boa_lock_state(game: &mut CGame, player_id: i32, state: BoaLockState, now_ms: u32) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| { let old = player.replace_boa_lock_state(state); if old.is_some() { player.set_skill_moveable(true); } player.set_skill_moveable(false); Some((old, player.server_region_id()?, player.shape().identity(), player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?)) }); let Some((old, region, identity, x, y)) = installed else { return false }; if let Some(old) = old { send_boa_lock_state_visual(game, region, identity, x, y, old, false, || now_ms); } send_boa_lock_state_visual(game, region, identity, x, y, state, true, game_tick_milliseconds); let _ = game.publish_player_states(player_id); true
}
pub(crate) fn replace_monster_boa_lock_state(game: &CGame, region: &mut CServerRegion, monster_id: i32, state: BoaLockState, now_ms: u32) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).map(|monster| { let shape = monster.move_shape().shape().clone(); let old = monster.move_shape_mut().replace_boa_lock_state(state); if old.is_some() { monster.move_shape_mut().set_moveable(true); } monster.move_shape_mut().set_moveable(false); (old, shape) }); let Some((old, shape)) = installed else { return false }; if let Some(old) = old { send_monster_visual(game, region, &shape, old, false, || now_ms); } send_monster_visual(game, region, &shape, state, true, game_tick_milliseconds); true
}
pub(crate) fn restart_boa_lock_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BoaLockState>(key)).copied()
    else { return false };
    if !crate::gameserver::appserver::states::state::begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false }
    if crate::gameserver::appserver::states::state::begin_applied_state_visual(
        game, region_id, holder, key, 1,
    ) {
        let mut message = CMessage::new(0x000b_fe03);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_ulong(crate::gameserver::appserver::states::state::timed_client_state_time(
            state.started_at_ms, state.keep_time_ms, &mut *now,
        ));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = crate::gameserver::appserver::states::state::update_applied_state_visual_base(
            game, region_id, holder, key,
        );
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder)
        .filter(|shape| shape.applied_state::<BoaLockState>(key).is_some())
    {
        shape.set_moveable(false);
    }
    true
}

pub(crate) fn update_boa_lock_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BoaLockState>(key))
        .is_some_and(|state| state.expired(now_ms));
    expired && end_boa_lock_state(game, region_id, holder, key)
}

pub(crate) fn end_boa_lock_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BoaLockState>(key)).copied()
    else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    if shape.applied_state::<BoaLockState>(key).is_none() { return false }
    shape.set_moveable(true);
    let removed = shape.remove_applied_state_record::<BoaLockState>(key, BOA_LOCK_STATE_BYTES).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}
