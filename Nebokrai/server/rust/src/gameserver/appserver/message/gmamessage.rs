//! GameServer GMA message owner.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный owner `server/gameserver/appserver/message/gmamessage.cpp`
//! подтверждают всю family: `0x80001` читает request ID и 256-byte player
//! name, логирует команду, выполняет ordered `FindPlayer -> KickPlayer` и
//! отвечает World `0x60401`; failure дополнительно несёт исходный английский
//! detail. `0x80002` отвечает текущим размером canonical player map через
//! `0x60402`. Любой другой `0x800xx` только возвращает исходный warning-log
//! во внешний runtime-boundary report.
//! `Vec<u8>` сохраняет byte-string/vararg log без требования UTF-8; network
//! side effects и legacy `KickPlayer == false` возвращаются в typed report.

use crate::gameserver::gameserver::game::{CGame, GameKickPlayerReport};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const GMA_FAMILY: u32 = 0x0008_0000;
const GMA_KICK_PLAYER: i32 = 0x0008_0001;
const GMA_PLAYER_COUNT: i32 = 0x0008_0002;
const GMA_KICK_RESPONSE: i32 = 0x0006_0401;
const GMA_PLAYER_COUNT_RESPONSE: i32 = 0x0006_0402;
const GMA_TEXT_LIMIT: usize = 0x100;
const GMA_KICK_LOG_PREFIX: &[u8] = b"Receive KICK_PLAYER command [name : ";
const GMA_KICK_LOG_SUFFIX: &[u8] = b"].";
const GMA_KICK_MISSING_DETAIL: &[u8] = b"GameServer : Can NOT find the player!";
const GMA_UNKNOWN_WARNING: &[u8] = b"WARNING : Unknown GMA message!";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmaMessageError {
    MissingRequestId,
    MissingPlayerName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmaMessageReport {
    KickPlayer {
        request_id: i32,
        player_name: Vec<u8>,
        log_text: Vec<u8>,
        kick: Option<GameKickPlayerReport>,
        delivery: Result<i32, SendMessageError>,
    },
    PlayerCount {
        count: u32,
        delivery: Result<i32, SendMessageError>,
    },
    Unknown {
        message_type: i32,
        log_text: Vec<u8>,
    },
}

/// Материализует всю `OnGMAMessage`; `None` оставляет сообщения других family
/// их собственным owner-ам.
pub(crate) fn dispatch_gma_message(
    message: &mut CMessage,
    game: &CGame,
) -> Option<Result<GmaMessageReport, GmaMessageError>> {
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
        let mut log_text = Vec::with_capacity(
            GMA_KICK_LOG_PREFIX.len() + player_name.len() + GMA_KICK_LOG_SUFFIX.len(),
        );
        log_text.extend_from_slice(GMA_KICK_LOG_PREFIX);
        log_text.extend_from_slice(legacy_c_string_prefix(&player_name));
        log_text.extend_from_slice(GMA_KICK_LOG_SUFFIX);

        let kick = game.kick_player_by_name(&player_name);
        let mut response = CMessage::new(GMA_KICK_RESPONSE);
        response.add_long(request_id);
        response.add_byte(u8::from(kick.is_some()));
        add_legacy_c_string(&mut response, &player_name);
        if kick.is_none() {
            add_legacy_c_string(&mut response, GMA_KICK_MISSING_DETAIL);
        }
        let delivery = response.send(game, false);
        return Some(Ok(GmaMessageReport::KickPlayer {
            request_id,
            player_name,
            log_text,
            kick,
            delivery,
        }));
    }

    if message_type == GMA_PLAYER_COUNT {
        let count = game.player_count();
        let mut response = CMessage::new(GMA_PLAYER_COUNT_RESPONSE);
        response.add_ulong(count);
        return Some(Ok(GmaMessageReport::PlayerCount {
            count,
            delivery: response.send(game, false),
        }));
    }

    Some(Ok(GmaMessageReport::Unknown {
        message_type,
        log_text: GMA_UNKNOWN_WARNING.to_vec(),
    }))
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
