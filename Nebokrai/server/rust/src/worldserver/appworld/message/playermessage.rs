//! Player-dispatcher `OnPlayerMessage` WorldServer.
//!
//! Источник контракта — `worldserver.exe` и
//! `worldserver.pdb`. Owner содержит четыре in-place relay branch:
//! `0x5FC01 -> 0x7FA08`, `0x5FC02 -> 0x7FA09`, `0x5FC03 -> 0x7FA0A`,
//! `0x5FC04 -> 0x7FA0B`. Только первая до broadcast дописывает signed
//! `m_lMapID`; остальные сохраняют payload byte-for-byte. Все четыре вызывают
//! общий `CMessage::SendAll`, не делают `Update`, не читают payload и не
//! проверяют socket/map ownership или хвост. Неизвестный opcode завершает
//! owner без mutation/send и
//! без передачи следующему dispatcher-у; Rust представляет это `NoOp`.
//!
//! Compiler catch/unwind-записи не являются отдельными source-owner-ами.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

const SET_SKILL_LEVEL: i32 = 0x0005_FC01;
const DELETE_SKILL: i32 = 0x0005_FC02;
const ADD_SKILL: i32 = 0x0005_FC03;
const USE_SKILL: i32 = 0x0005_FC04;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerMessageOutcome {
    NoOp {
        request_type: i32,
    },
    Relay {
        request_type: i32,
        response_type: i32,
        appended_map_id: Option<i32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

pub(crate) enum WorldPlayerMessageDispatch {
    Handled(WorldPlayerMessageOutcome),
    Pending(CMessage),
}

pub(crate) fn on_player_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldPlayerMessageDispatch {
    let request_type = message.message_type();
    let (response_type, append_map_id) = match request_type {
        SET_SKILL_LEVEL => (0x0007_FA08, true),
        DELETE_SKILL => (0x0007_FA09, false),
        ADD_SKILL => (0x0007_FA0A, false),
        USE_SKILL => (0x0007_FA0B, false),
        _ => {
            return WorldPlayerMessageDispatch::Handled(WorldPlayerMessageOutcome::NoOp {
                request_type,
            });
        }
    };

    message.set_message_type(response_type);
    let appended_map_id = append_map_id.then(|| {
        let map_id = message.map_id();
        message.base_mut().add_long(map_id);
        map_id
    });
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldPlayerMessageDispatch::Handled(WorldPlayerMessageOutcome::Relay {
        request_type,
        response_type,
        appended_map_id,
        wire,
        delivery,
    })
}
