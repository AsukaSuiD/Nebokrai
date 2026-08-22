//! WorldServer dispatcher-owner `OnMSG_M2W_AUCTION`.
//!
//! Статус владельца: `IMPLEMENTED` для relay-ветвей `0x15EB03/04/06/07/08`;
//! `0x15EB01/02/05` остаются owned `Pending`, пока не восстановлен их общий
//! concrete DB-owner. Точная пара: `WorldServer/Nworldserver.exe +
//! WorldServer/WorldServer.pdb`, исходный owner
//! `e:\\svn\\fengyun_russia_dev\\server\\worldserver\\appworld\\message\\onmsg_m2w_auction.cpp:12`,
//! RVA `0x000A5230`.
//!
//! `0x15EB03/04` сначала снимают один map byte, затем без преобразования
//! переносят весь непрочитанный хвост в новый `0x80401/0x80402`. `0x15EB06`
//! меняет opcode in-place на `0x80406` и делает broadcast без `Update`.
//! `0x15EB07/08` снимают player ID, находят его GameServer через регион,
//! проверяют `bConnected`, только затем меняют opcode, вызывают `Update` и
//! отправляют тому же numeric map ID. Rust-owned байты и `Result` заменяют
//! allocator/exception plumbing; отсутствие одного payload-поля сохраняет
//! принятую безопасную границу `0`, не назначая чтению за буфером игровую
//! семантику.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

const FORWARD_AUCTION_STATE: i32 = 0x0015_EB03;
const FORWARD_AUCTION_RESULT: i32 = 0x0015_EB04;
const BROADCAST_AUCTION_RESULT: i32 = 0x0015_EB06;
const FORWARD_PLAYER_SEARCH: i32 = 0x0015_EB07;
const FORWARD_PLAYER_GOODS: i32 = 0x0015_EB08;

/// Наблюдаемый результат одной реализованной M2W auction-ветви.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMiscAuctionMessageOutcome {
    pub(crate) request_type: i32,
    pub(crate) response_type: Option<i32>,
    pub(crate) route_map_id: Option<i32>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

/// Сохранённое сообщение для ещё не восстановленного DB owner-а.
pub(crate) enum WorldMiscAuctionMessageDispatch {
    Handled(WorldMiscAuctionMessageOutcome),
    Pending(CMessage),
}

/// Исполняет доказанные relay-ветви `OnMSG_M2W_AUCTION`.
pub(crate) fn on_msg_m2w_auction(
    game: &CGame,
    mut message: CMessage,
) -> WorldMiscAuctionMessageDispatch {
    let request_type = message.message_type();
    match request_type {
        FORWARD_AUCTION_STATE | FORWARD_AUCTION_RESULT => {
            let route_map_id = i32::from(message.base_mut().get_byte().unwrap_or(0));
            let payload = {
                let base = message.base_mut();
                let (wire, cursor) = base.wire_bytes_and_cursor_mut();
                wire.get(*cursor..).unwrap_or_default().to_vec()
            };
            let response_type = if request_type == FORWARD_AUCTION_STATE {
                0x0008_0401
            } else {
                0x0008_0402
            };
            let mut response = CMessage::new(response_type);
            response.base_mut().add(&payload);
            let wire = response.as_wire_bytes().to_vec();
            let delivery =
                response.send_to_map_id(game.current_game_server_sender().as_ref(), route_map_id);
            WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                request_type,
                response_type: Some(response_type),
                route_map_id: Some(route_map_id),
                wire,
                delivery: Some(delivery),
            })
        }
        BROADCAST_AUCTION_RESULT => {
            message.set_message_type(0x0008_0406);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_all(game.current_game_server_sender().as_ref());
            WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                request_type,
                response_type: Some(0x0008_0406),
                route_map_id: None,
                wire,
                delivery: Some(delivery),
            })
        }
        FORWARD_PLAYER_SEARCH | FORWARD_PLAYER_GOODS => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            let Some(game_server) = game.player_game_server(player_id) else {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                });
            };
            if !game_server.connected {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                });
            }
            let response_type = if request_type == FORWARD_PLAYER_SEARCH {
                0x0008_0407
            } else {
                0x0008_0408
            };
            let route_map_id = game_server.index as i32;
            message.set_message_type(response_type);
            message.base_mut().update();
            let wire = message.as_wire_bytes().to_vec();
            let delivery =
                message.send_to_map_id(game.current_game_server_sender().as_ref(), route_map_id);
            WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                request_type,
                response_type: Some(response_type),
                route_map_id: Some(route_map_id),
                wire,
                delivery: Some(delivery),
            })
        }
        _ => WorldMiscAuctionMessageDispatch::Pending(message),
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\onmsg_m2w_auction.cpp

// ============================================================================
// FUNCTION: OnMSG_M2W_AUCTION
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\onmsg_m2w_auction.cpp:12
// RVA: 0x000A5230
// ADDRESS: 004a5230
// PROTOTYPE: void __cdecl OnMSG_M2W_AUCTION(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






























































// ============================================================================
// FUNCTION: Unwind@005329f1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\onmsg_m2w_auction.cpp
// RVA: 0x001329F1
// ADDRESS: 005329f1
// PROTOTYPE: undefined Unwind@005329f1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00532a06
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\onmsg_m2w_auction.cpp
// RVA: 0x00132A06
// ADDRESS: 00532a06
// PROTOTYPE: undefined Unwind@00532a06()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//









// COMPONENT_VARIANT_END: WorldServer
