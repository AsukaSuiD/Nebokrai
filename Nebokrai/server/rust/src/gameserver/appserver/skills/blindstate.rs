//! Каноническая загружаемая часть `CBlindState` (`0x76`).
//! Истечение получает ключ конкретного экземпляра общей арены; проверка
//! срока и End не подменяют его первым состоянием с тем же ID.
//! Vtable 0x00662214, слот +0x0c: CBlindState::AI (0x005d5ba0).
//! Срок проверяется как start.wrapping_add(keep) < now, включая keep == 0;
//! elapsed-сравнение не сохраняет исходный переход DWORD через ноль.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/blindstate.cpp`. Достигнутый путь сохраняет восьмибайтную
//! запись `state ID + remaining time`, строгую беззнаковую границу таймера,
//! запреты движения и боя, завершение при `ACTION_DEFENSE` и точные сообщения
//! `0xBFE03/0xBFE04`. Единственным владельцем состояния остаётся
//! `CanonicalStateStorage`; неизменённый legacy payload служит его кодеком.
//! Перегрузки `Begin`, для которых ещё нет настоящего создающего caller-а,
//! сохранены ниже как RAW.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp

// ============================================================================
// FUNCTION: CBlindState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:106
// RVA: 0x001D5BA0
// ADDRESS: 005d5ba0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:112
// RVA: 0x001E3B30
// ADDRESS: 005e3b30
// PROTOTYPE: void __thiscall OnChangeRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:90
// RVA: 0x001EA9A0
// ADDRESS: 005ea9a0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:130
// RVA: 0x001EAAC0
// ADDRESS: 005eaac0
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::GetRemainedTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:37
// RVA: 0x001F2CD0
// ADDRESS: 005f2cd0
// PROTOTYPE: ulong __thiscall GetRemainedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:120
// RVA: 0x001F51E0
// ADDRESS: 005f51e0
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::OnAction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.h:33
// RVA: 0x00207370
// ADDRESS: 00607370
// PROTOTYPE: void __thiscall OnAction(tagAction param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::CBlindState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:24
// RVA: 0x00207380
// ADDRESS: 00607380
// PROTOTYPE: undefined __thiscall CBlindState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::~CBlindState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:33
// RVA: 0x002073F0
// ADDRESS: 006073f0
// PROTOTYPE: void __thiscall ~CBlindState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:58
// RVA: 0x00207400
// ADDRESS: 00607400
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:74
// RVA: 0x002074E0
// ADDRESS: 006074e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:44
// RVA: 0x002075E0
// ADDRESS: 006075e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBlindStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\blindstate.cpp:141
// RVA: 0x002076A0
// ADDRESS: 006076a0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//







// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const BLIND_STATE_ID: u32 = 0x76;
pub(crate) const BLIND_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BlindState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl BlindState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != BLIND_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(0, reader.read_u32()?))
    }

    pub(crate) const fn skill_id(self) -> u32 {
        BLIND_STATE_ID
    }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self {
        self.started_at_ms = now_ms;
        self
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Exact `GetRemainedTime`: deadline-check и положительный остаток читают
    /// wrapping clock независимо.
    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "поля задают точку фактической круговой доставки"
)]
pub(crate) fn send_blind_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BlindState,
    begin: bool,
    mut now_milliseconds: impl FnMut() -> u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_ulong(state.client_state_time(&mut now_milliseconds));
        message.add_ulong(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

fn finish_player_blind_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    expired_key: Option<crate::gameserver::appserver::moveshape::StateKey>,
) -> bool {
    let finished = game.find_player_mut(player_id).and_then(|player| {
        let state = if let Some(key) = expired_key {
            player.take_expired_blind_state(key, now_ms)?
        } else {
            player.take_blind_state()?
        };
        player.set_skill_moveable(true);
        player.set_skill_fightable(true);
        Some((
            state,
            player.server_region_id()?,
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    });
    let Some((state, region_id, identity, tile_x, tile_y)) = finished else {
        return false;
    };
    send_blind_state_visual(
        game, region_id, identity, tile_x, tile_y, state, false, || now_ms,
    );
    true
}

pub(crate) fn expire_player_blind_state(
    game: &mut CGame,
    player_id: i32,
    key: crate::gameserver::appserver::moveshape::StateKey,
    now_ms: u32,
) -> bool {
    finish_player_blind_state(game, player_id, now_ms, Some(key))
}

pub(crate) fn finish_player_blind_state_on_defense(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    finish_player_blind_state(game, player_id, now_ms, None)
}
