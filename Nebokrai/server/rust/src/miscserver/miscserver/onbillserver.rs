//! Свободный handler `OnMSG_M2M_Fuction` из `onbillserver.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`.
//!
//! Точная пара: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`;
//! SHA-256 EXE
//! `F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65`,
//! SHA-256 PDB
//! `ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7`.
//! Исходный путь PDB:
//! `h:\fengyun\fy_russia\src\server\miscserver\miscserver\onbillserver.cpp`.
//! `OnMSG_M2M_Fuction` — RVA `0x00003C10`.
//!
//! Handler знает два полных opcode. `0x0016EA01` сначала ставит
//! `m_bClientClose = true`, затем синхронно вызывает `CGame::ReConnect`.
//! Linux connect является `async`, поэтому Rust-функция ждёт его до возврата:
//! будущий snapshot-runner обязан await-ить handler и не может переставить
//! следующее сообщение раньше reconnect. Исходный `0/1` результат reconnect
//! игнорировался; typed outcome сохраняет его для диагностики.
//!
//! `0x0016EA02` сначала вызывает `CAuctionRoom::Clear`, затем строит пустое
//! `0x0015EB05` и отправляет его текущему nullable `CMyNetClient` без
//! приоритета. Отсутствующий client сохраняет доказанный нулевой результат
//! общего `CMessage::Send`; ошибка построения envelope остаётся typed-ошибкой
//! после уже выполненной очистки. Остальные opcode ничего не делают.
//!
//! Nullable входной `CMessage*` заменён обязательной ссылкой: `CMessage::Run`
//! вызывает handler только для живого owned сообщения. Два process-global
//! обращения `GetGame()` выражены одной mutable ссылкой на единственного
//! будущего `CGame`. `std::map<CGUID, bool>` и его tree/iterator функции в этом
//! translation unit не вызываются handler-ом и классифицированы как STL noise;
//! десять `$L` являются destructor/unwind cleanup. Они заменены стандартными
//! коллекциями, владением и `Drop`, поэтому отдельных Rust-тел не имеют.

use crate::miscserver::miscserver::game::{CGame, MiscClientConnectOutcome};
use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};

const CLIENT_CLOSED: i32 = 0x0016_EA01;
const CLEAR_AUCTION_ROOM: i32 = 0x0016_EA02;
const AUCTION_ROOM_CLEARED: i32 = 0x0015_EB05;

/// Наблюдаемый итог одного вызова исходного `OnMSG_M2M_Fuction`.
#[derive(Debug)]
pub(crate) enum MiscFunctionOutcome {
    /// Opcode не входил в две известные ветви handler-а.
    Unhandled,
    /// Close-флаг поставлен, а блокирующий reconnect полностью завершён.
    Reconnect(MiscClientConnectOutcome),
    /// Комната очищена до исходно игнорировавшейся попытки отправки ack.
    AuctionRoomCleared { send: Result<i32, SendMessageError> },
}

/// Обрабатывает две доказанные служебные ветви MiscServer.
pub(crate) async fn on_msg_m2m_function(
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
