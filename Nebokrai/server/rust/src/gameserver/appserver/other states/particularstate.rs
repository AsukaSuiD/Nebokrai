//! CParticularState, gameserver.exe + GameServer.pdb, исходный owner
//! appserver/other states/particularstate.cpp. Экземпляры принадлежат общей
//! арене CMoveShape; Vec/SlotMap и заимствования заменяют CState*/ручной lifetime.
//! Ctor0x004F9440 принимает любой DWORD additional; goods/duplicate-gates —
//! ответственность caller. Object Begin0x004F9710 требует non-NULL sufferer:
//! base self/self clock → visual loop1/BFE03 → base tail → checkstamp=0,
//! затем append. NULL-user restart сохраняет User и не читает часы.
//! Base timestamp не потребляется AI/codec и не хранится фиктивным таймером.
//! Visual0x004F97C0 использует actual sufferer; absent/ended gates подавляют
//! пакет, но существующий ресурс всегда получает base tail. Общая арена
//! материализует residual visual первичного Begin без повторной отправки.
//! AI0x004F9900 сначала читает clock: после границы 2000 проверяет Player
//! sufferer и ненулевые GAP_EXCEPTION_STATE packet → equipment по listener.
//! Checkstamp всегда нулевой и не продвигается; death-gate отсутствует.
//! End0x005FD420: optional visual/BFE04 → свежий sufferer → RemoveState,
//! без state.ended и принудительного удаления из другой арены.
//! Vtable0x00653684 SetRegion меняет только sufferer. Client time=0,
//! additional=DWORD; Serialize0x005E23D0/Unserialize0x00601350 сохраняют
//! ID/additional (8 байт), без часов, включая additional=0 при загрузке.
//! Drop освобождает ресурс без End/пакетов; отказ native allocator не эмулируется.
//! Координатный/typed-target Begin0x004F9540/0x004F9610 остаются RAW ниже.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    StatePropertyTarget, begin_applied_state_visual, begin_base_applied_state,
    default_client_state_time, remove_applied_state_from, resolve_applied_state_sufferer,
    resolve_state_move_shape, update_applied_state_end_visual, update_applied_state_visual_base,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const PARTICULAR_STATE_ID: u32 = 0x186a5;
pub(crate) const PARTICULAR_STATE_BYTES: usize = 8;
const PARTICULAR_STATE_CHECK_INTERVAL_MS: u32 = 2_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ParticularState {
    additional_data: u32,
}

impl ParticularState {
    pub(crate) const fn new(additional_data: u32) -> Self {
        Self { additional_data }
    }

    pub(crate) const fn additional_data(&self) -> u32 {
        self.additional_data
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_u32()?;
        Ok(Self::new(reader.read_u32()?))
    }

    pub(crate) fn encoded(&self) -> [u8; PARTICULAR_STATE_BYTES] {
        let mut record = [0; PARTICULAR_STATE_BYTES];
        record[..4].copy_from_slice(&PARTICULAR_STATE_ID.to_le_bytes());
        record[4..].copy_from_slice(&self.additional_data.to_le_bytes());
        record
    }

    pub(crate) const fn state_id(&self) -> i32 {
        PARTICULAR_STATE_ID as i32
    }

    pub(crate) const fn client_state_time(&self) -> i32 {
        default_client_state_time()
    }

    pub(crate) const fn due(&self, now_ms: u32) -> bool {
        PARTICULAR_STATE_CHECK_INTERVAL_MS <= now_ms
    }
}

pub(crate) fn begin_primary_particular_state(
    player: &mut CPlayer,
    additional: u32,
    now: &mut dyn FnMut() -> u32,
    send: &mut dyn FnMut(&CPlayer, &CMessage),
) -> StateKey {
    let state = ParticularState::new(additional);
    let _ = now();
    let participant = (
        player.shape().get_region_id(),
        ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..player.shape().identity() },
    );
    let message = particular_state_begin_message(participant.1, &state);
    send(player, &message);
    let record = state.encoded();
    let shape = player.move_shape_mut();
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, Some(participant));
    shape.set_applied_state_sufferer(key, Some(participant));
    key
}

pub(crate) fn restart_particular_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    if begin_applied_state_visual(game, region_id, holder, key, 1) {
        update_particular_state_begin_visual(game, region_id, holder, key);
    }
    true
}

fn particular_state_begin_message(target: ShapeIdentity, state: &ParticularState) -> CMessage {
    let mut message = CMessage::new(0x000b_fe03);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(state.state_id());
    message.add_long(state.client_state_time());
    message.add_ulong(state.additional_data());
    message
}

fn update_particular_state_begin_visual(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    let Some(ended) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state_visual_ended(key))
    else { return false };
    if !ended {
        if let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key) {
            let message = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ParticularState>(key))
                .map(|state| particular_state_begin_message(target, state));
            if let Some(message) = message {
                let _ = game.send_move_shape_around(target_region, target, &message);
            }
        }
    }
    update_applied_state_visual_base(game, region_id, holder, key);
    true
}

pub(crate) fn update_particular_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    runtime: &mut Runtime,
) -> bool {
    let checked_at_ms = runtime.now_milliseconds();
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key))
    else { return false };
    if !state.due(checked_at_ms) {
        return false;
    }
    let additional = state.additional_data();
    if let Some((_, target)) = resolve_applied_state_sufferer(game, region_id, holder, key) {
        if target.object_type == 400
            && game.find_player(target.id).is_some_and(|player| {
                player.particular_state_goods_present(additional, game.goods_factory())
            })
        {
            return false;
        }
    }
    end_particular_state(game, region_id, holder, key)
}

pub(crate) fn end_particular_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ParticularState>(key)).is_none()
    {
        return false;
    }
    update_applied_state_end_visual(
        game, region_id, holder, key, StatePropertyTarget::Sufferer,
    );
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false };
    remove_applied_state_from(game, region_id, holder, key, target, PARTICULAR_STATE_BYTES)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\particularstate.cpp

// ============================================================================
// FUNCTION: CParticularState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\particularstate.cpp:58
// RVA: 0x000F9540
// ADDRESS: 004f9540
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CParticularState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\particularstate.cpp:78
// RVA: 0x000F9610
// ADDRESS: 004f9610
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
