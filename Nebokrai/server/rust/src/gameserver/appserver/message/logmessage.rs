//! World/log-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/logmessage.cpp`. Friend presence `0x7F904/905` сохраняет
//! исходный `(playerId, friendName)` payload, меняет только message type на
//! `0xBF404/405` и адресно пересылает указанному игроку. Остальные ветви
//! `OnLogMessage` ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const FRIEND_ONLINE: u32 = 0x0007_f904;
const FRIEND_OFFLINE: u32 = 0x0007_f905;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameLogMessageError {
    MissingPlayerId,
}

#[must_use = "log-message report сохраняет presence forwarding"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameLogMessageReport {
    pub(crate) source_type: u32,
    pub(crate) client_type: u32,
    pub(crate) player_id: i32,
    pub(crate) delivery: i32,
}

pub(crate) fn dispatch_game_log_message(
    message: &mut CMessage,
    game: &CGame,
) -> Option<Result<GameLogMessageReport, GameLogMessageError>> {
    let source_type = message.message_type() as u32;
    let client_type = match source_type {
        FRIEND_ONLINE => 0x000b_f404,
        FRIEND_OFFLINE => 0x000b_f405,
        _ => return None,
    };
    let Some(player_id) = message.base_mut().get_long() else {
        return Some(Err(GameLogMessageError::MissingPlayerId));
    };
    message.base_mut().set_message_type(client_type as i32);
    let delivery = message.send_to_player(game.net_server(), player_id);
    Some(Ok(GameLogMessageReport {
        source_type,
        client_type,
        player_id,
        delivery,
    }))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\logmessage.cpp

// ============================================================================
// FUNCTION: OnLogMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\logmessage.cpp:30
// RVA: 0x0009F140
// ADDRESS: 0049f140
// PROTOTYPE: void __cdecl OnLogMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a0620
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\logmessage.cpp:219
// RVA: 0x000A0620
// ADDRESS: 004a0620
// PROTOTYPE: undefined Catch@004a0620()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
