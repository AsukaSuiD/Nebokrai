//! Служебные ветви `onbillserver.cpp`, подтверждённые `miscserver.exe` и
//! `miscserver.pdb`.
//! В составе Realm `app/` — служебные ветви роли MiscServer.
//!
//! Close сначала ставит `m_bClientClose`, затем полностью дожидается reconnect,
//! не позволяя следующему FIFO-элементу его обогнать. Очистка auction room
//! предшествует попытке отправить пустой ack; отсутствующий client сохраняет
//! нулевой результат `Send`, а ошибка ответа не откатывает очистку.

use super::misc_game::{CGame, MiscClientConnectOutcome};
use crate::app::misc_message::{CMessage, MessageSender, SendMessageError};

const CLIENT_CLOSED: i32 = 0x0016_EA01;
const CLEAR_AUCTION_ROOM: i32 = 0x0016_EA02;
const AUCTION_ROOM_CLEARED: i32 = 0x0015_EB05;

#[derive(Debug)]
pub enum MiscFunctionOutcome {
    Unhandled,
    Reconnect(MiscClientConnectOutcome),
    AuctionRoomCleared { send: Result<i32, SendMessageError> },
}

pub async fn on_msg_m2m_function(
    message: &CMessage,
    game: &mut CGame,
) -> MiscFunctionOutcome {
    match message.message_type() {
        CLIENT_CLOSED => {
            game.mark_client_closed();
            MiscFunctionOutcome::Reconnect(game.reconnect().await)
        }
        CLEAR_AUCTION_ROOM => {
            game.auction_room_mut().clear();
            let response = CMessage::new(AUCTION_ROOM_CLEARED);
            let sender = game.net_client().map(|client| client as &dyn MessageSender);
            MiscFunctionOutcome::AuctionRoomCleared {
                send: response.send(sender, false),
            }
        }
        _ => MiscFunctionOutcome::Unhandled,
    }
}
