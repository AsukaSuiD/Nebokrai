//! Состояние полной блокировки `CStrikeState` (`0xDD`).
//! Vtable 0x00662154, слот +0x0c: CBlindState::AI (0x005d5ba0).
//! Срок проверяется как start.wrapping_add(keep) < now, включая keep == 0;
//! elapsed-сравнение не сохраняет исходный переход DWORD через ноль.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/strikestate.cpp`. Достигнутый путь загрузки сохраняет
//! exact-пару `CBlindState::Serialize/Unserialize` `0x005F51E0/0x005EAAC0`:
//! идентификатор и остаток срока занимают восемь байт. Player-login повторно
//! начинает срок, восстанавливает вложенные запреты движения/боя и публикует
//! begin-визуал, а logout сохраняет отдельный остаток для каждого экземпляра
//! состояния. AI и End общие с CBlindState, но OnAction +0x34→0x00601A70
//! является ret 4 и не снимает Strike при Defense. Создание остаётся RAW до
//! настоящего runtime-caller-а.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const STRIKE_STATE_ID: u32 = 0xdd;
pub(crate) const STRIKE_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StrikeState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl StrikeState {
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != STRIKE_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self { started_at_ms: 0, keep_time_ms: reader.read_u32()? })
    }

    pub(crate) const fn skill_id(self) -> u32 {
        STRIKE_STATE_ID
    }
    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 { timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) }
}

pub(crate) fn send_strike_state_visual(game: &mut CGame, region_id: i32, identity: ShapeIdentity, tile_x: i32, tile_y: i32, state: StrikeState, begin: bool, now_ms: u32) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 }); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(STRIKE_STATE_ID as i32); if begin { message.add_ulong(state.client_time(|| now_ms)); message.add_ulong(0); } let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp

// ============================================================================
// FUNCTION: CStrikeState::CStrikeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp:24
// RVA: 0x00206830
// ADDRESS: 00606830
// PROTOTYPE: undefined __thiscall CStrikeState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CStrikeState::~CStrikeState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp:33
// RVA: 0x002068A0
// ADDRESS: 006068a0
// PROTOTYPE: void __thiscall ~CStrikeState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CStrikeState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp:71
// RVA: 0x002068B0
// ADDRESS: 006068b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CStrikeState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp:91
// RVA: 0x00206990
// ADDRESS: 00606990
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CStrikeState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp:54
// RVA: 0x00206AA0
// ADDRESS: 00606aa0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CStrikeStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\strikestate.cpp:164
// RVA: 0x00206B60
// ADDRESS: 00606b60
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
