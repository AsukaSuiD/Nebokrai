//! WorldServer dispatcher-owner `OnMSG_S2W_AUCTION`.
//!
//! Статус владельца: `IMPLEMENTED` для relay/DB queue/BaiTan/auction-bang
//! ветвей `0x60801..07/09/0F..14`; `0x60808/0A..0E` остаются owned `Pending`
//! до concrete GlobeSetup, SQL-load, auction notice и player-virtual
//! владельцев. Точная пара: `WorldServer/Nworldserver.exe +
//! WorldServer/WorldServer.pdb`, исходный owner
//! `e:\\svn\\fengyun_russia_dev\\server\\worldserver\\appworld\\message\\auction.cpp:10`,
//! RVA `0x000A5650`.
//!
//! `0x60801/04/06` создают один owned `DbNote`, декодируют `CGoodsNode` и
//! ставят соответствующий input operation; только non-DB item `0x60801`
//! зануляет buyer и немедленно отправляет `0x14ED01`. Реализованные relay меняют только literal opcode и сохраняют отсутствие
//! `Update` там, где его нет в EXE. `0x60807` добавляет source map как один
//! unsigned byte перед непрочитанным payload. `0x60811/12` сохраняют порядок
//! чтения и точные BaiTan mutations; `0x60814` проверяет только существование
//! online player и region GameServer, но не `bConnected`. Стандартные owned
//! buffers и `Result` заменяют allocator/exception plumbing, не меняя wire
//! layout и order side effects.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::dbaccess::worlddb::dbmisc::{CDbMisc, DbNote, OperatorType};
use crate::public::auctionlog::CAuctionLog;
use crate::public::auctionnode::{GoodsNodeSerializeError, GoodsNodeUnserializeError, GoodsState};
use crate::worldserver::worldserver::game::{CGame, WorldBaiTanRemoval};

const AUCTION_GAME_SERVER: u32 = 5;
const INSERT_AUCTION_ITEM: i32 = 0x0006_0801;
const MODIFY_AUCTION_STATE: i32 = 0x0006_0804;
const MODIFY_AUCTION_SALE: i32 = 0x0006_0806;
const FORWARD_INSERT_RESULT: i32 = 0x0006_0802;
const FORWARD_SEARCH_RESULT: i32 = 0x0006_0803;
const FORWARD_BUY: i32 = 0x0006_0805;
const FORWARD_SYNC: i32 = 0x0006_0807;
const FORWARD_LOG: i32 = 0x0006_0809;
const FORWARD_CONDITION: i32 = 0x0006_080F;
const SEND_AUCTION_BANG: i32 = 0x0006_0810;
const ADD_BAI_TAN_REQUEST: i32 = 0x0006_0811;
const REMOVE_BAI_TAN: i32 = 0x0006_0812;
const BROADCAST_AUCTION_RESULT: i32 = 0x0006_0813;
const FORWARD_PLAYER_RESULT: i32 = 0x0006_0814;

/// Наблюдаемый результат одной доказанной S2W auction-ветви.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldServerAuctionMessageOutcome {
    InputQueued {
        request_type: i32,
        operation: OperatorType,
    },
    Forwarded {
        request_type: i32,
        response_type: i32,
        route_map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Broadcast {
        request_type: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    AuctionBangSent {
        player_id: u32,
        map_id: u32,
        delivery: Result<i32, SendMessageError>,
    },
    BaiTanRequestAdded {
        ip: u32,
        player_id: i32,
        inserted: bool,
    },
    BaiTanRemoved {
        player_id: i32,
        removal: WorldBaiTanRemoval,
    },
    NoOp {
        request_type: i32,
    },
    GoodsNodeUnserializeBlocked {
        request_type: i32,
        source: GoodsNodeUnserializeError,
    },
    GoodsNodeSerializeBlocked {
        request_type: i32,
        source: GoodsNodeSerializeError,
    },
}

/// Сохранённое сообщение для ещё не восстановленной части S2W owner-а.
pub(crate) enum WorldServerAuctionMessageDispatch {
    Handled(WorldServerAuctionMessageOutcome),
    Pending(CMessage),
}

/// Исполняет доказанные S2W auction-ветви, не требующие сырого DB owner-а.
pub(crate) fn on_msg_s2w_auction(
    game: &mut CGame,
    auction_log: &CAuctionLog,
    db_misc: &CDbMisc,
    mut message: CMessage,
) -> WorldServerAuctionMessageDispatch {
    let request_type = message.message_type();
    let forward_to_auction = |message: &mut CMessage, response_type| {
        if !game
            .game_server(AUCTION_GAME_SERVER)
            .is_some_and(|game_server| game_server.connected)
        {
            return None;
        }
        message.set_message_type(response_type);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = message.send_to_map_id(
            game.current_game_server_sender().as_ref(),
            AUCTION_GAME_SERVER as i32,
        );
        Some(WorldServerAuctionMessageOutcome::Forwarded {
            request_type,
            response_type,
            route_map_id: AUCTION_GAME_SERVER as i32,
            wire,
            delivery,
        })
    };

    match request_type {
        INSERT_AUCTION_ITEM => {
            if !game
                .game_server(AUCTION_GAME_SERVER)
                .is_some_and(|game_server| game_server.connected)
            {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::NoOp { request_type },
                );
            }
            let mut note = Box::new(DbNote::new());
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                note.goods.unserialize(source, cursor)
            };
            if let Err(source) = decode {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::GoodsNodeUnserializeBlocked {
                        request_type,
                        source,
                    },
                );
            }
            if note.goods.is_db() {
                note.e_type = OperatorType::OT_IN_INSERT_NEW_ITEM;
                let _ = db_misc.push_item_to_list_in(note, false);
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::InputQueued {
                        request_type,
                        operation: OperatorType::OT_IN_INSERT_NEW_ITEM,
                    },
                );
            }
            note.goods.set_buyer_id(0);
            let bytes = match note.goods.serialize() {
                Ok(bytes) => bytes,
                Err(source) => {
                    return WorldServerAuctionMessageDispatch::Handled(
                        WorldServerAuctionMessageOutcome::GoodsNodeSerializeBlocked {
                            request_type,
                            source,
                        },
                    );
                }
            };
            let mut response = CMessage::new(0x0014_ED01);
            response.base_mut().add(&bytes);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                AUCTION_GAME_SERVER as i32,
            );
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::Forwarded {
                request_type,
                response_type: 0x0014_ED01,
                route_map_id: AUCTION_GAME_SERVER as i32,
                wire,
                delivery,
            })
        }
        FORWARD_INSERT_RESULT => forward_to_auction(&mut message, 0x0014_ED06)
            .map_or_else(
                || WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::NoOp { request_type }),
                WorldServerAuctionMessageDispatch::Handled,
            ),
        FORWARD_SEARCH_RESULT => forward_to_auction(&mut message, 0x0014_ED07)
            .map_or_else(
                || WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::NoOp { request_type }),
                WorldServerAuctionMessageDispatch::Handled,
            ),
        MODIFY_AUCTION_STATE => {
            let mut note = Box::new(DbNote::new());
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                note.goods.unserialize(source, cursor)
            };
            if let Err(source) = decode {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::GoodsNodeUnserializeBlocked {
                        request_type,
                        source,
                    },
                );
            }
            if note.goods.goods_state() == GoodsState::PRE_BUY {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::NoOp { request_type },
                );
            }
            note.e_type = OperatorType::OT_IN_MODIFY_STATE_A2B;
            let _ = db_misc.push_item_to_list_in(note, false);
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::InputQueued {
                request_type,
                operation: OperatorType::OT_IN_MODIFY_STATE_A2B,
            })
        }
        FORWARD_BUY => {
            message.set_message_type(0x0014_ED04);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                AUCTION_GAME_SERVER as i32,
            );
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::Forwarded {
                request_type,
                response_type: 0x0014_ED04,
                route_map_id: AUCTION_GAME_SERVER as i32,
                wire,
                delivery,
            })
        }
        MODIFY_AUCTION_SALE => {
            let mut note = Box::new(DbNote::new());
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                note.goods.unserialize(source, cursor)
            };
            if let Err(source) = decode {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::GoodsNodeUnserializeBlocked {
                        request_type,
                        source,
                    },
                );
            }
            note.e_type = OperatorType::OT_IN_MODIFY_STATE_A2S;
            let _ = db_misc.push_item_to_list_in(note, false);
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::InputQueued {
                request_type,
                operation: OperatorType::OT_IN_MODIFY_STATE_A2S,
            })
        }
        FORWARD_SYNC => {
            let source_map_id = message.map_id() as u8;
            let payload = {
                let base = message.base_mut();
                let (wire, cursor) = base.wire_bytes_and_cursor_mut();
                wire.get(*cursor..).unwrap_or_default().to_vec()
            };
            let mut response = CMessage::new(0x0014_ED05);
            response.base_mut().add_byte(source_map_id);
            response.base_mut().add(&payload);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                AUCTION_GAME_SERVER as i32,
            );
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::Forwarded {
                request_type,
                response_type: 0x0014_ED05,
                route_map_id: AUCTION_GAME_SERVER as i32,
                wire,
                delivery,
            })
        }
        FORWARD_LOG => {
            message.set_message_type(0x0014_ED08);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                AUCTION_GAME_SERVER as i32,
            );
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::Forwarded {
                request_type,
                response_type: 0x0014_ED08,
                route_map_id: AUCTION_GAME_SERVER as i32,
                wire,
                delivery,
            })
        }
        FORWARD_CONDITION => forward_to_auction(&mut message, 0x0014_ED09)
            .map_or_else(
                || WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::NoOp { request_type }),
                WorldServerAuctionMessageDispatch::Handled,
            ),
        SEND_AUCTION_BANG => {
            let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
            let map_id = message.map_id() as u32;
            let delivery = auction_log.send_auction_msg_to_game_server(
                player_id,
                map_id,
                game.current_game_server_sender().as_ref(),
            );
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionBangSent {
                    player_id,
                    map_id,
                    delivery,
                },
            )
        }
        ADD_BAI_TAN_REQUEST => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            let ip = message.base_mut().get_long().unwrap_or(0) as u32;
            let inserted = game.add_item_to_bai_tan_request_list(ip, player_id);
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::BaiTanRequestAdded {
                    ip,
                    player_id,
                    inserted,
                },
            )
        }
        REMOVE_BAI_TAN => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            let _unused = message.base_mut().get_long().unwrap_or(0);
            let removal = game.del_item_from_bai_tan_list(player_id);
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::BaiTanRemoved { player_id, removal },
            )
        }
        BROADCAST_AUCTION_RESULT => {
            message.set_message_type(0x0008_040F);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_all(game.current_game_server_sender().as_ref());
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::Broadcast {
                request_type,
                response_type: 0x0008_040F,
                wire,
                delivery,
            })
        }
        FORWARD_PLAYER_RESULT => {
            let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
            let value = message.base_mut().get_long().unwrap_or(0);
            let Some(game_server) = game.player_game_server(player_id as i32) else {
                return WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::NoOp { request_type });
            };
            let route_map_id = game_server.index as i32;
            let mut response = CMessage::new(0x0008_0410);
            response.base_mut().add_ulong(player_id);
            response.base_mut().add_long(value);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(game.current_game_server_sender().as_ref(), route_map_id);
            WorldServerAuctionMessageDispatch::Handled(WorldServerAuctionMessageOutcome::Forwarded {
                request_type,
                response_type: 0x0008_0410,
                route_map_id,
                wire,
                delivery,
            })
        }
        _ => WorldServerAuctionMessageDispatch::Pending(message),
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\auction.cpp

// ============================================================================
// FUNCTION: OnMSG_S2W_AUCTION
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\auction.cpp:10
// RVA: 0x000A5650
// ADDRESS: 004a5650
// PROTOTYPE: void __cdecl OnMSG_S2W_AUCTION(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
