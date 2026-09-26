//! Диспетчер игроков `OnPlayerMessage` WorldServer в составе Realm `app/`;
//! источник контракта — `Nworldserver.exe` и `WorldServer.pdb`.
//!
//! Владелец содержит четыре ветви перенаправления на месте:
//! `0x5FC01 -> 0x7FA08`, `0x5FC02 -> 0x7FA09`, `0x5FC03 -> 0x7FA0A`,
//! `0x5FC04 -> 0x7FA0B`. Только первая до общей рассылки дописывает знаковый
//! `m_lMapID`; остальные побайтно сохраняют содержимое. Все четыре вызывают
//! общий `CMessage::SendAll`, не выполняют `Update`, не читают содержимое и не
//! проверяют принадлежность сокета или карты и хвост. Неизвестный код операции
//! завершает владельца без изменения, отправки и передачи следующему
//! диспетчеру; Rust представляет это вариантом `NoOp`. Узкий game-view
//! [`WorldGameView`] заменяет прямую ссылку на владельца игры; реализация
//! у владельца игры делегирует inherent-методу.
//!
//! Записи компилятора для перехвата и раскрутки стека не являются отдельными
//! владельцами исходного кода.

use crate::app::world_game_view::WorldGameView;
use crate::app::world_message::{CMessage, SendMessageError};

const SET_SKILL_LEVEL: i32 = 0x0005_FC01;
const DELETE_SKILL: i32 = 0x0005_FC02;
const DELETE_PLAYER_GOODS: i32 = 0x0005_FC03;
const SET_PLAYER_LEVEL: i32 = 0x0005_FC04;

#[derive(Debug, Eq, PartialEq)]
pub enum WorldPlayerMessageOutcome {
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

pub enum WorldPlayerMessageDispatch {
    Handled(WorldPlayerMessageOutcome),
    Pending(CMessage),
}

pub fn on_player_message(
    game: &impl WorldGameView,
    mut message: CMessage,
) -> WorldPlayerMessageDispatch {
    let request_type = message.message_type();
    let (response_type, append_map_id) = match request_type {
        SET_SKILL_LEVEL => (0x0007_FA08, true),
        DELETE_SKILL => (0x0007_FA09, false),
        DELETE_PLAYER_GOODS => (0x0007_FA0A, false),
        SET_PLAYER_LEVEL => (0x0007_FA0B, false),
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
