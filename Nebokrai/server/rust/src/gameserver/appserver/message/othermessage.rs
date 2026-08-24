//! Other-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/othermessage.cpp`. LeiTing `0x7FA17` замыкает ответ
//! WorldServer: player ID, exact partial-mutation codec и итоговый `0xBF73E`.
//! Остальные ветви ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::player::PlayerLeiTingDecodeBlock;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const WORLD_LEI_TING_UPDATE: u32 = 0x0007_fa17;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameOtherMessageOutcome {
    PlayerMissing,
    LeiTingUpdated { client_delivery: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOtherMessageError {
    MissingPlayerId,
    LeiTing(PlayerLeiTingDecodeBlock),
}

#[must_use = "other-message report сохраняет World decode и client publication"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameOtherMessageReport {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) outcome: GameOtherMessageOutcome,
}

pub(crate) fn dispatch_game_other_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<GameOtherMessageReport, GameOtherMessageError>> {
    let message_type = message.message_type() as u32;
    if message_type != WORLD_LEI_TING_UPDATE {
        return None;
    }
    let Some(player_id) = message.base_mut().get_long() else {
        return Some(Err(GameOtherMessageError::MissingPlayerId));
    };
    let Some(player) = game.find_player_mut(player_id) else {
        return Some(Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        }));
    };
    let decode = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        player.decode_lei_ting(source, cursor)
    };
    if let Err(block) = decode {
        return Some(Err(GameOtherMessageError::LeiTing(block)));
    }
    let payload = game
        .find_player(player_id)
        .expect("LeiTing player сохранён после decode")
        .encode_lei_ting();
    let mut response = CMessage::new(0x000b_f73e);
    response.base_mut().add(&payload);
    let client_delivery = response.send_to_player(game.net_server(), player_id);
    Some(Ok(GameOtherMessageReport {
        message_type,
        player_id,
        outcome: GameOtherMessageOutcome::LeiTingUpdated { client_delivery },
    }))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\othermessage.cpp

// ============================================================================
// FUNCTION: OnOtherMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\othermessage.cpp:42
// RVA: 0x00090E20
// ADDRESS: 00490e20
// PROTOTYPE: void __cdecl OnOtherMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
