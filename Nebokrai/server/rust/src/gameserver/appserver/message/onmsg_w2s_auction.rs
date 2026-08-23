//! World→Game auction handler.
//!
//! Точная пара GameServer EXE/PDB и owner
//! `server/gameserver/appserver/message/onmsg_w2s_auction.cpp` подтверждают
//! selector `0x80403`: чтение Windows `long`, bool-проекцию и
//! вызов `CGame::SetAuctionState`; эта ветвь имеет статус `IMPLEMENTED`.
//! Для enabled-state время берётся только после чтения payload, как в
//! оригинале. Обрезанный payload заменяет небезопасное чтение за буфером
//! typed error-ом без мутации. Остальные auction selectors остаются RAW ниже.

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const WORLD_AUCTION_STATE_MESSAGE: i32 = 0x0008_0403;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldAuctionStateMessageError {
    MissingEnabledLong,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldAuctionStateMessageReport {
    pub(crate) enabled: bool,
    pub(crate) last_check_seconds: u32,
}

/// Материализует только exact `0x80403`-ветвь большого handler-а.
/// `None` означает, что сообщение должен идти в оставшийся auction owner.
pub(crate) fn dispatch_world_auction_state(
    message: &mut CMessage,
    game: &mut CGame,
    wall_time_seconds: impl FnOnce() -> u32,
) -> Option<Result<WorldAuctionStateMessageReport, WorldAuctionStateMessageError>> {
    if message.message_type() != WORLD_AUCTION_STATE_MESSAGE {
        return None;
    }
    let Some(enabled) = message.base_mut().get_long() else {
        return Some(Err(WorldAuctionStateMessageError::MissingEnabledLong));
    };
    let enabled = enabled != 0;
    let last_check_seconds = if enabled {
        wall_time_seconds()
    } else {
        game.auction_last_check_seconds()
    };
    game.set_auction_state(enabled, last_check_seconds);
    Some(Ok(WorldAuctionStateMessageReport {
        enabled,
        last_check_seconds,
    }))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_w2s_auction.cpp

// ============================================================================
// FUNCTION: OnMSG_W2S_AUCTION
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_w2s_auction.cpp:21
// RVA: 0x000983A0
// ADDRESS: 004983a0
// PROTOTYPE: void __cdecl OnMSG_W2S_AUCTION(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
