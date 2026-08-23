//! Auction-dispatcher `OnMSG_M2W_AUCTION` WorldServer.
//!
//! Источник контракта — `worldserver.exe` и
//! `worldserver.pdb`, исходный owner `OnMSG_M2W_AUCTION`.
//!
//! `0x15EB01` и non-`STATE_PRE_BUY` путь `0x15EB02` передают owned `DbNote`
//! в действующую input queue с operation. `0x15EB02` в
//! `STATE_PRE_BUY` находит buyer GameServer, проверяет `bConnected`, затем
//! строит и обновляет `0x80406 + 1 + CGoodsNode`; `0x15EB05` вызывает тот же
//! `DoneOT_IN_READ_AUCTION` через live DB context. `0x15EB03/04` сначала снимают один map byte, затем без преобразования
//! переносят весь непрочитанный хвост в новый `0x80401/0x80402`. `0x15EB06`
//! меняет opcode in-place на `0x80406` и делает broadcast без `Update`.
//! `0x15EB07/08` снимают player ID, находят его GameServer через регион,
//! проверяют `bConnected`, только затем меняют opcode, вызывают `Update` и
//! отправляют тому же numeric map ID. Rust-owned байты и `Result` заменяют
//! allocator/exception plumbing. Безопасные ошибки `CGoodsNode` остаются
//! точными typed boundaries: старый void decoder мог читать/писать вне buffer,
//! но это не становится придуманной игровой реакцией.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::dbaccess::worlddb::dbmisc::{CDbMisc, DbMiscContext, DbNote, OperatorType};
use crate::public::auctionnode::{GoodsNodeSerializeError, GoodsNodeUnserializeError, GoodsState};
use crate::worldserver::worldserver::game::CGame;

const MODIFY_AUCTION_SALE: i32 = 0x0015_EB01;
const MODIFY_AUCTION_BUY: i32 = 0x0015_EB02;
const FORWARD_AUCTION_STATE: i32 = 0x0015_EB03;
const FORWARD_AUCTION_RESULT: i32 = 0x0015_EB04;
const REFRESH_AUCTION_OWNERS: i32 = 0x0015_EB05;
const BROADCAST_AUCTION_RESULT: i32 = 0x0015_EB06;
const FORWARD_PLAYER_SEARCH: i32 = 0x0015_EB07;
const FORWARD_PLAYER_GOODS: i32 = 0x0015_EB08;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldMiscAuctionMessageOutcome {
    pub(crate) request_type: i32,
    pub(crate) response_type: Option<i32>,
    pub(crate) route_map_id: Option<i32>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) queue_operation: Option<OperatorType>,
    pub(crate) unserialize_block: Option<GoodsNodeUnserializeError>,
    pub(crate) serialize_block: Option<GoodsNodeSerializeError>,
}

/// Результат M2W auction-dispatcher-а; все literal case текущего owner-а уже
/// обработаны, а default switch является no-op.
pub(crate) enum WorldMiscAuctionMessageDispatch {
    Handled(WorldMiscAuctionMessageOutcome),
    Pending(CMessage),
}

pub(crate) fn on_msg_m2w_auction(
    game: &CGame,
    db_misc: &CDbMisc,
    db_misc_context: &mut impl DbMiscContext,
    mut message: CMessage,
) -> WorldMiscAuctionMessageDispatch {
    let request_type = message.message_type();
    match request_type {
        MODIFY_AUCTION_SALE => {
            let mut note = Box::new(DbNote::new());
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                note.goods.unserialize(source, cursor)
            };
            if let Err(source) = decode {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                    queue_operation: None,
                    unserialize_block: Some(source),
                    serialize_block: None,
                });
            }
            note.e_type = OperatorType::OT_IN_MODIFY_STATE_A2S;
            let _ = db_misc.push_item_to_list_in(note, false);
            WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                request_type,
                response_type: None,
                route_map_id: None,
                wire: Vec::new(),
                delivery: None,
                queue_operation: Some(OperatorType::OT_IN_MODIFY_STATE_A2S),
                unserialize_block: None,
                serialize_block: None,
            })
        }
        MODIFY_AUCTION_BUY => {
            let mut note = Box::new(DbNote::new());
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                note.goods.unserialize(source, cursor)
            };
            if let Err(source) = decode {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                    queue_operation: None,
                    unserialize_block: Some(source),
                    serialize_block: None,
                });
            }
            if note.goods.goods_state() != GoodsState::PRE_BUY {
                note.e_type = OperatorType::OT_IN_MODIFY_STATE_A2B;
                let _ = db_misc.push_item_to_list_in(note, false);
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                    queue_operation: Some(OperatorType::OT_IN_MODIFY_STATE_A2B),
                    unserialize_block: None,
                    serialize_block: None,
                });
            }
            let Some(game_server) = game.player_game_server(note.goods.buyer_id() as i32) else {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                    queue_operation: None,
                    unserialize_block: None,
                    serialize_block: None,
                });
            };
            if !game_server.connected {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                    queue_operation: None,
                    unserialize_block: None,
                    serialize_block: None,
                });
            }
            let bytes = match note.goods.serialize() {
                Ok(bytes) => bytes,
                Err(source) => {
                    return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                        request_type,
                        response_type: None,
                        route_map_id: None,
                        wire: Vec::new(),
                        delivery: None,
                        queue_operation: None,
                        unserialize_block: None,
                        serialize_block: Some(source),
                    });
                }
            };
            let route_map_id = game_server.index as i32;
            let mut response = CMessage::new(0x0008_0406);
            response.base_mut().add_long(1);
            response.base_mut().add(&bytes);
            response.base_mut().update();
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(game.current_game_server_sender().as_ref(), route_map_id);
            WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                request_type,
                response_type: Some(0x0008_0406),
                route_map_id: Some(route_map_id),
                wire,
                delivery: Some(delivery),
                queue_operation: None,
                unserialize_block: None,
                serialize_block: None,
            })
        }
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
                queue_operation: None,
                unserialize_block: None,
                serialize_block: None,
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
                queue_operation: None,
                unserialize_block: None,
                serialize_block: None,
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
                    queue_operation: None,
                    unserialize_block: None,
                    serialize_block: None,
                });
            };
            if !game_server.connected {
                return WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                    request_type,
                    response_type: None,
                    route_map_id: None,
                    wire: Vec::new(),
                    delivery: None,
                    queue_operation: None,
                    unserialize_block: None,
                    serialize_block: None,
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
                queue_operation: None,
                unserialize_block: None,
                serialize_block: None,
            })
        }
        REFRESH_AUCTION_OWNERS => {
            db_misc.done_ot_in_read_auction(db_misc_context);
            WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
                request_type,
                response_type: None,
                route_map_id: None,
                wire: Vec::new(),
                delivery: None,
                queue_operation: None,
                unserialize_block: None,
                serialize_block: None,
            })
        }
        _ => WorldMiscAuctionMessageDispatch::Handled(WorldMiscAuctionMessageOutcome {
            request_type,
            response_type: None,
            route_map_id: None,
            wire: Vec::new(),
            delivery: None,
            queue_operation: None,
            unserialize_block: None,
            serialize_block: None,
        }),
    }
}
