//! Каноническое состояние рыцарского удара `CKnightCutState` (`0x67`).
//! Object Begin (0x005FCFC0) требует sufferer, но допускает NULL user.
//! restart_knight_cut_state: base Begin → visual SetRun(1)/Update(0) и его
//! base tail → move-lock, затем fight-lock. Timestamp не меняется: его задаёт
//! Unserialize 0x005EAAC0 отдельным clock до чтения оставшегося срока.
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Direct End (vtable 0x00661254 +0x1C, тело 0x005EA9A0) сначала
//! отправляет visual, затем снимает fight-lock и move-lock и удаляет точный ключ.
//! RemoveState вызывает UpdateProperty игрока; timer и Cure используют
//! этот же хвост без подмены direct End принудительным истечением.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/knightcutstate.cpp`. Состояние запрещает движение и бой,
//! публикует `0xBFE03/0xBFE04` и завершается по wrapping-часам общего AI.
//! Известная запись принадлежит только `CanonicalStateStorage`; отдельного
//! изменяемого RAW-представления для неё нет. Vtable exact EXE подтверждает
//! общий с `CBlindState` клиентский срок по `0x005F2CD0` и serializer-пару
//! `0x005F51E0/0x005EAAC0`: persisted-запись `ID + remaining time` занимает
//! 8 байт. Spatial login восстанавливает оба запрета и curable lifecycle.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const KNIGHT_CUT_STATE_ID: u32 = 0x67;
pub(crate) const KNIGHT_CUT_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnightCutState { started_at_ms: u32, keep_time_ms: u32 }

impl KnightCutState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self { started_at_ms, keep_time_ms }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != KNIGHT_CUT_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }


    pub(crate) fn encoded_for_install(self) -> [u8; KNIGHT_CUT_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; KNIGHT_CUT_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; KNIGHT_CUT_STATE_BYTES] {
        let mut bytes = [0; KNIGHT_CUT_STATE_BYTES];
        bytes[..4].copy_from_slice(&KNIGHT_CUT_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }
    pub(crate) const fn skill_id(self) -> u32 { KNIGHT_CUT_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точную точку круговой доставки состояния")]
pub(crate) fn send_knight_cut_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: KnightCutState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

fn send_owned_monster_visual(game: &CGame, region: &CServerRegion, shape: &crate::gameserver::appserver::shape::CShape, state: KnightCutState, begin: bool, now_milliseconds: impl FnMut() -> u32) {
    let identity = shape.identity();
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(state.skill_id() as i32);
    if begin { message.add_long(state.client_time(now_milliseconds)); message.add_long(0); }
    let _ = game.send_game_shape_around(region, shape, None, &message);
}

pub(crate) fn replace_player_knight_cut_state(game: &mut CGame, player_id: i32, state: KnightCutState, now_ms: u32) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| {
        let region_id = player.server_region_id()?; let identity = player.shape().identity();
        let tile_x = player.shape().get_tile_x().ok()?; let tile_y = player.shape().get_tile_y().ok()?;
        let previous = player.replace_knight_cut_state(state);
        if previous.is_some() { player.set_skill_moveable(true); player.set_skill_fightable(true); }
        player.set_skill_moveable(false); player.set_skill_fightable(false);
        Some((previous, region_id, identity, tile_x, tile_y))
    });
    let Some((previous, region_id, identity, tile_x, tile_y)) = installed else { return false };
    if let Some(previous) = previous { send_knight_cut_state_visual(game, region_id, identity, tile_x, tile_y, previous, false, || now_ms); }
    send_knight_cut_state_visual(game, region_id, identity, tile_x, tile_y, state, true, game_tick_milliseconds); true
}

pub(crate) fn replace_monster_knight_cut_state(game: &mut CGame, region: &mut CServerRegion, monster_id: i32, state: KnightCutState, now_ms: u32) -> bool {
    let installed = region.find_monster_by_id_mut(monster_id).map(|monster| {
        let shape = monster.move_shape().shape().clone(); let previous = monster.move_shape_mut().replace_knight_cut_state(state);
        if previous.is_some() { monster.move_shape_mut().set_moveable(true); monster.move_shape_mut().set_fightable(true); }
        monster.move_shape_mut().set_moveable(false); monster.move_shape_mut().set_fightable(false); (previous, shape)
    });
    let Some((previous, shape)) = installed else { return false };
    if let Some(previous) = previous { send_owned_monster_visual(game, region, &shape, previous, false, || now_ms); }
    send_owned_monster_visual(game, region, &shape, state, true, game_tick_milliseconds); true
}

pub(crate) fn restart_knight_cut_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<KnightCutState>(key)).copied()
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
        .filter(|shape| shape.applied_state::<KnightCutState>(key).is_some())
    {
        shape.set_moveable(false);
        shape.set_fightable(false);
    }
    true
}

pub(crate) fn update_knight_cut_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<KnightCutState>(key))
        .is_some_and(|state| state.expired(now_ms));
    expired && end_knight_cut_state(game, region_id, holder, key)
}

pub(crate) fn end_knight_cut_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: crate::gameserver::appserver::moveshape::StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<KnightCutState>(key)).copied()
    else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) else { return false };
    if shape.applied_state::<KnightCutState>(key).is_none() { return false }
    shape.set_fightable(true);
    shape.set_moveable(true);
    let removed = shape.remove_applied_state_record::<KnightCutState>(key, KNIGHT_CUT_STATE_BYTES).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}

pub(crate) fn finish_player_knight_cut_state_on_cure(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let Some(player) = game.find_player(player_id) else { return false };
    let Some(key) = player.move_shape().applied_state_key::<KnightCutState>() else { return false };
    let region_id = player.shape().get_region_id();
    let holder = player.shape().identity();
    end_knight_cut_state(game, region_id, holder, key)
}
