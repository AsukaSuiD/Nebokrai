//! Владелец GMA-сообщений GameServer.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `server/gameserver/appserver/message/gmamessage.cpp`
//! подтверждают всё семейство: `0x80001` читает ID запроса и 256-байтовое имя
//! игрока, выполняет упорядоченный `FindPlayer -> KickPlayer` и отвечает World
//! сообщением `0x60401`; неуспех дополнительно несёт исходную английскую
//! строку протокола. `0x80002` отвечает текущим размером канонической карты
//! игроков через `0x60402`. Любой другой `0x800xx` создаёт предупреждение
//! `tracing`. Сетевые эффекты выполняются синхронно и не дублируются отчётом.

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use tracing::{debug, warn};

const GMA_FAMILY: u32 = 0x0008_0000;
const GMA_KICK_PLAYER: i32 = 0x0008_0001;
const GMA_PLAYER_COUNT: i32 = 0x0008_0002;
const GMA_KICK_RESPONSE: i32 = 0x0006_0401;
const GMA_PLAYER_COUNT_RESPONSE: i32 = 0x0006_0402;
const GMA_TEXT_LIMIT: usize = 0x100;
const GMA_KICK_MISSING_DETAIL: &[u8] = b"GameServer : Can NOT find the player!";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmaMessageError {
    MissingRequestId,
    MissingPlayerName,
}

/// Материализует всю `OnGMAMessage`; `None` оставляет сообщения других family
/// их собственным owner-ам.
pub(crate) fn dispatch_gma_message(
    message: &mut CMessage,
    game: &CGame,
) -> Option<Result<(), GmaMessageError>> {
    let message_type = message.message_type();
    if message_type as u32 & 0xFFFF_FF00 != GMA_FAMILY {
        return None;
    }

    if message_type == GMA_KICK_PLAYER {
        let Some(request_id) = message.base_mut().get_long() else {
            return Some(Err(GmaMessageError::MissingRequestId));
        };
        let Some(player_name) = message.base_mut().get_str_bytes(GMA_TEXT_LIMIT) else {
            return Some(Err(GmaMessageError::MissingPlayerName));
        };
        let kick = game.kick_player_by_name(&player_name);
        let mut response = CMessage::new(GMA_KICK_RESPONSE);
        response.add_long(request_id);
        response.add_byte(u8::from(kick));
        add_legacy_c_string(&mut response, &player_name);
        if !kick {
            add_legacy_c_string(&mut response, GMA_KICK_MISSING_DETAIL);
        }
        let _ = response.send(game, false);
        debug!(request_id, player_found = kick, player_name_len = legacy_c_string_prefix(&player_name).len(), "обработана GMA-команда отключения игрока");
        return Some(Ok(()));
    }

    if message_type == GMA_PLAYER_COUNT {
        let count = game.player_count();
        let mut response = CMessage::new(GMA_PLAYER_COUNT_RESPONSE);
        response.add_ulong(count);
        let _ = response.send(game, false);
        debug!(count, "отправлено число игроков для GMA");
        return Some(Ok(()));
    }

    warn!(message_type, "получено неизвестное GMA-сообщение");
    Some(Ok(()))
}

fn add_legacy_c_string(message: &mut CMessage, value: &[u8]) {
    message.base_mut().add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}
