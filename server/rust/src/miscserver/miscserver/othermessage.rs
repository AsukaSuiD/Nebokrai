//! Ответные ветви `miscserver/othermessage.cpp`, подтверждённые
//! `miscserver.exe` и `miscserver.pdb`.
//!
//! Оба ответа несут одно нулевое 32-битное поле и отправляются без приоритета.
//! Без client-а `0x7F809` завершается до создания ответа, тогда как `0x7F80B`
//! всё равно вызывает `Send` и получает нулевой результат. Остальные opcode —
//! no-op.

use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};

const WORLD_STATUS_REQUEST: i32 = 0x0007_F809;
const WORLD_STATUS_RESPONSE: i32 = 0x0005_FA0A;
const WORLD_SYNC_REQUEST: i32 = 0x0007_F80B;
const WORLD_SYNC_RESPONSE: i32 = 0x0005_FA0C;

#[derive(Debug)]
pub(crate) enum OtherMessageOutcome {
    Unhandled,
    MissingClient,
    Response {
        message_type: i32,
        send: Result<i32, SendMessageError>,
    },
}

pub(crate) fn on_other_msg(
    message: &CMessage,
    sender: Option<&dyn MessageSender>,
) -> OtherMessageOutcome {
    let response_type = match message.message_type() {
        WORLD_STATUS_REQUEST if sender.is_none() => return OtherMessageOutcome::MissingClient,
        WORLD_STATUS_REQUEST => WORLD_STATUS_RESPONSE,
        WORLD_SYNC_REQUEST => WORLD_SYNC_RESPONSE,
        _ => return OtherMessageOutcome::Unhandled,
    };

    let mut response = CMessage::new(response_type);
    response.base_mut().add_long(0);
    OtherMessageOutcome::Response {
        message_type: response_type,
        send: response.send(sender, false),
    }
}
