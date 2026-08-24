//! Владелец region-message dispatcher-а GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/regionmessage.cpp`. Достигнутый `0x8F801` завершает
//! локальный `CPlayer::ChangeRegion`: проверяет live player/region context,
//! снимает `m_bInChangingRegion`, переносит client IP, добавляет player в
//! destination spatial registry и лишь затем передаёт serialization/weather/
//! state tail runtime-owner-у. Остальные opcodes ниже остаются RAW.

use crate::gameserver::gameserver::game::{CGame, GameRegionEnterContext, GameRegionEnterReport};
use crate::nets::netserver::message::CMessage;

const ENTER_CHANGED_REGION: u32 = 0x0008_f801;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameRegionMessageError {
    InvalidPayloadSize { expected: usize, actual: usize },
    MissingEntryToken,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRegionMessageReport {
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) entry: Option<GameRegionEnterReport>,
}

pub(crate) fn dispatch_game_region_message<Context: GameRegionEnterContext>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<GameRegionMessageReport, GameRegionMessageError>> {
    if message.message_type() as u32 != ENTER_CHANGED_REGION {
        return None;
    }
    let actual = message
        .base_mut()
        .as_wire_bytes()
        .len()
        .saturating_sub(message.base_mut().cursor());
    if actual != 4 {
        return Some(Err(GameRegionMessageError::InvalidPayloadSize {
            expected: 4,
            actual,
        }));
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(entry_token) = message.base_mut().get_long() else {
        return Some(Err(GameRegionMessageError::MissingEntryToken));
    };
    let entry = match (player_id, region_id) {
        (Some(player_id), Some(region_id)) => game.enter_changed_player_region(
            player_id,
            region_id,
            entry_token,
            message.ip(),
            message.socket_id(),
            context,
        ),
        _ => None,
    };
    Some(Ok(GameRegionMessageReport {
        player_id,
        region_id,
        entry,
    }))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\regionmessage.cpp

// ============================================================================
// FUNCTION: CServerRegion::OnMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\regionmessage.cpp:21
// RVA: 0x001BB270
// ADDRESS: 005bb270
// PROTOTYPE: void __thiscall OnMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
