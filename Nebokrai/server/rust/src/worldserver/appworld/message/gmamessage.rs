//! WorldServer dispatcher-owner `OnGMAMessage`.
//!
//! Статус `IMPLEMENTED_PARTIAL`: exact `0x004A6020..0x004A609F`
//! материализован для transport branches `0x4FD04`, `0x60401`, `0x60402`;
//! `0x4FD01` и его kick-player helper остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmamessage.cpp`.
//! `0x4FD04 -> 0x80002` вызывает `SendAll`; `0x60401 -> 0x20101` вызывает
//! неприоритетный `Send(false)` в LoginServer; `0x60402 -> 0x20104` дописывает
//! signed source map ID, 32-битные биты `dwNumber` и также вызывает
//! `Send(false)`. Payload не читается, `Update` и ownership/tail gates
//! отсутствуют. До setup `dwNumber` в оригинале был неинициализирован; Rust не
//! выбирает произвольные биты и возвращает typed safe-block без внешнего send.
//!
//! Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже сохранён как локальная
//! документация, а не как Rust-реализация.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmaMessageOutcome {
    LoginRelay {
        request_type: i32,
        response_type: i32,
        appended_map_id: Option<i32>,
        appended_world_number: Option<u32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    GameServerBroadcast {
        request_type: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    MissingWorldNumber {
        request_type: i32,
        response_type: i32,
        map_id: i32,
    },
}

pub(crate) enum WorldGmaMessageDispatch {
    Handled(WorldGmaMessageOutcome),
    Pending(CMessage),
}

/// Исполняет три достигнутые transport-ветви exact `OnGMAMessage`.
pub(crate) fn on_gma_message(game: &CGame, mut message: CMessage) -> WorldGmaMessageDispatch {
    let request_type = message.message_type();
    match request_type {
        0x0004_FD04 => {
            let response_type = 0x0008_0002;
            message.set_message_type(response_type);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_all(game.current_game_server_sender().as_ref());
            WorldGmaMessageDispatch::Handled(WorldGmaMessageOutcome::GameServerBroadcast {
                request_type,
                response_type,
                wire,
                delivery,
            })
        }
        0x0006_0401 => send_login_relay(game, message, request_type, 0x0002_0101, None),
        0x0006_0402 => {
            let response_type = 0x0002_0104;
            let map_id = message.map_id();
            let Some(world_number) = game.configured_world_number() else {
                return WorldGmaMessageDispatch::Handled(
                    WorldGmaMessageOutcome::MissingWorldNumber {
                        request_type,
                        response_type,
                        map_id,
                    },
                );
            };
            message.set_message_type(response_type);
            message.base_mut().add_long(map_id);
            message.base_mut().add_ulong(world_number);
            send_login_relay(
                game,
                message,
                request_type,
                response_type,
                Some((map_id, world_number)),
            )
        }
        _ => WorldGmaMessageDispatch::Pending(message),
    }
}

fn send_login_relay(
    game: &CGame,
    mut message: CMessage,
    request_type: i32,
    response_type: i32,
    appended: Option<(i32, u32)>,
) -> WorldGmaMessageDispatch {
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldGmaMessageDispatch::Handled(WorldGmaMessageOutcome::LoginRelay {
        request_type,
        response_type,
        appended_map_id: appended.map(|value| value.0),
        appended_world_number: appended.map(|value| value.1),
        wire,
        delivery,
    })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmamessage.cpp

// ============================================================================
// FUNCTION: FUN_004a5db0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmamessage.cpp:11
// RVA: 0x000A5DB0
// ADDRESS: 004a5db0
// PROTOTYPE: undefined FUN_004a5db0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: OnGMAMessage
// STATUS: IMPLEMENTED_PARTIAL
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmamessage.cpp:108
// RVA: 0x000A6020
// ADDRESS: 004a6020
// PROTOTYPE: void __cdecl OnGMAMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//












// COMPONENT_VARIANT_END: WorldServer
