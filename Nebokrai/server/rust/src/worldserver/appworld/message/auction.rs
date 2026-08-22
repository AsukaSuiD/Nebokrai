//! WorldServer dispatcher-owner `OnMSG_S2W_AUCTION`.
//!
//! Статус владельца: `IMPLEMENTED` для relay/DB queue/BaiTan/auction-bang
//! ветвей `0x60801..14`. Точная пара: `WorldServer/Nworldserver.exe +
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
//! `0x6080B` сначала мутирует page, затем строит `0x80409`; C-string без NUL
//! остаётся typed boundary после этой мутации. `0x6080C` строит `0x8040A` в
//! доказанном порядке second-ID, first-ID, log data, только потом `Update`.
//! `0x6080D` требует online player, после чего `CollectNoNotice` сначала
//! помечает live records и по одному публикует exact SQL в общий FIFO, затем
//! строит, обновляет и отправляет `0x8040B` в source map.
//! `0x6080E` читает unsigned player ID и декодирует полный player-wire с
//! текущего cursor только у online owner-а; virtual/CRT plumbing заменён
//! существующим безопасным codec-ом без нового wire-формата.
//! `0x60808` строит `0x80403` с signed long `0/1`: единица возможна только
//! для подключённого GameServer `5` и `CGlobeSetup::bAuction != 0`; `Update`
//! перед source-map send отсутствует и намеренно не добавляется.
//! `0x6080A` читает owner, goods-limit и money-limit. Ровно в порядке EXE
//! вызывает DB owner для `BACK(3)`, затем при неполном результате `UNDO(4)` и
//! `SUCCESSED(2)` с остатком; `LoadMoneyById` вызывается всегда и публикует
//! свой output note независимо от числа товаров.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::dbaccess::worlddb::dbmisc::{CDbMisc, DbMiscContext, DbNote, OperatorType};
use crate::public::auctionlog::{
    AuctionGoodsLogWriteOutcome, AuctionLogPageBlock, AuctionLogPageWriteDisposition,
    AuctionLogTimeBlock, AuctionNoticeCollection, AuctionNoticeWriteQueue, CAuctionLog,
};
use crate::public::auctionnode::{GoodsNodeSerializeError, GoodsNodeUnserializeError, GoodsState};
use crate::public::guid::CGuid;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::worldserver::appworld::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::worldserver::appworld::message::writelogmessage::WorldWriteLogCommand;
use crate::worldserver::appworld::player::{PlayerCodecError, PlayerPropertyCoefficients};
use crate::worldserver::worldserver::game::{CGame, WorldBaiTanRemoval};

const AUCTION_GAME_SERVER: u32 = 5;
const INSERT_AUCTION_ITEM: i32 = 0x0006_0801;
const MODIFY_AUCTION_STATE: i32 = 0x0006_0804;
const MODIFY_AUCTION_SALE: i32 = 0x0006_0806;
const REQUEST_AUCTION_STATE: i32 = 0x0006_0808;
const LOAD_AUCTION_RETURNS: i32 = 0x0006_080A;
const REQUEST_AUCTION_HISTORY: i32 = 0x0006_080B;
const REQUEST_AUCTION_GOODS_LOG: i32 = 0x0006_080C;
const COLLECT_AUCTION_NOTICE: i32 = 0x0006_080D;
const APPLY_AUCTION_PLAYER_SNAPSHOT: i32 = 0x0006_080E;
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
    AuctionHistoryPage {
        player_id: i32,
        direction: i32,
        compute_result: bool,
        write: AuctionLogPageWriteDisposition,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    AuctionHistoryPageBlocked {
        player_id: i32,
        direction: i32,
        compute_result: bool,
        source: AuctionLogPageBlock,
    },
    AuctionGoodsLog {
        first_id: i32,
        player_id: i32,
        guid: CGuid,
        write: AuctionGoodsLogWriteOutcome,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    AuctionGoodsLogBlocked {
        first_id: i32,
        player_id: i32,
        guid: CGuid,
        source: AuctionLogTimeBlock,
    },
    AuctionNoticeCollected {
        collection: AuctionNoticeCollection,
        queued_sql_count: usize,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    AuctionPlayerSnapshotApplied {
        player_id: u32,
        decode: Result<bool, PlayerCodecError>,
        cursor_after: usize,
    },
    AuctionState {
        auction_enabled: bool,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    AuctionReturnsLoaded {
        owner_id: i32,
        requested_goods: i32,
        money_limit: i32,
        loaded_back: i32,
        loaded_undo: Option<i32>,
        loaded_succeeded: Option<i32>,
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

/// Тонкая граница `CAuctionLog -> CGame::m_qWriteLogData` для одной ветви.
struct WorldAuctionNoticeWriteQueue<'a> {
    game: &'a CGame,
    queued_sql_count: usize,
}

impl<'a> WorldAuctionNoticeWriteQueue<'a> {
    const fn new(game: &'a CGame) -> Self {
        Self {
            game,
            queued_sql_count: 0,
        }
    }
}

impl AuctionNoticeWriteQueue for WorldAuctionNoticeWriteQueue<'_> {
    fn push_auction_notice_sql(&mut self, sql: String) {
        let _ = self
            .game
            .push_write_log_command(WorldWriteLogCommand::AuctionNoticeSql(sql));
        self.queued_sql_count += 1;
    }
}

/// Исполняет доказанные S2W auction-ветви, не требующие сырого DB owner-а.
pub(crate) fn on_msg_s2w_auction(
    game: &mut CGame,
    auction_log: &mut CAuctionLog,
    db_misc: &CDbMisc,
    db_misc_context: &mut impl DbMiscContext,
    globe_setup: &GlobeSetupSnapshot,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
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
        REQUEST_AUCTION_STATE => {
            let auction_enabled = game
                .game_server(AUCTION_GAME_SERVER)
                .is_some_and(|game_server| game_server.connected)
                && globe_setup.auction_enabled();
            let mut response = CMessage::new(0x0008_0403);
            response.base_mut().add_long(i32::from(auction_enabled));
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                message.map_id(),
            );
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionState {
                    auction_enabled,
                    wire,
                    delivery,
                },
            )
        }
        LOAD_AUCTION_RETURNS => {
            let owner_id = message.base_mut().get_long().unwrap_or(0);
            let requested_goods = message.base_mut().get_long().unwrap_or(0);
            let money_limit = message.base_mut().get_long().unwrap_or(0);

            let loaded_back =
                db_misc.load_owner_back_goods(db_misc_context, owner_id, requested_goods);
            let (loaded_undo, loaded_succeeded) = if loaded_back < requested_goods {
                let remaining_after_back = requested_goods - loaded_back;
                let loaded_undo = db_misc.load_owner_undo_goods(
                    db_misc_context,
                    owner_id,
                    remaining_after_back,
                );
                let loaded_succeeded = (loaded_undo < remaining_after_back).then(|| {
                    db_misc.load_owner_succ_goods(
                        db_misc_context,
                        owner_id,
                        remaining_after_back - loaded_undo,
                    )
                });
                (Some(loaded_undo), loaded_succeeded)
            } else {
                (None, None)
            };
            db_misc.load_money_by_id(db_misc_context, owner_id, money_limit);

            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionReturnsLoaded {
                    owner_id,
                    requested_goods,
                    money_limit,
                    loaded_back,
                    loaded_undo,
                    loaded_succeeded,
                },
            )
        }
        REQUEST_AUCTION_HISTORY => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            let direction = message.base_mut().get_long().unwrap_or(0);
            let compute_result = auction_log.compute_page(direction, player_id);
            let mut response = CMessage::new(0x0008_0409);
            let write = match auction_log.add_byte_at_current_page(player_id, Some(&mut response)) {
                Ok(write) => write,
                Err(source) => {
                    return WorldServerAuctionMessageDispatch::Handled(
                        WorldServerAuctionMessageOutcome::AuctionHistoryPageBlocked {
                            player_id,
                            direction,
                            compute_result,
                            source,
                        },
                    );
                }
            };
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                message.map_id(),
            );
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionHistoryPage {
                    player_id,
                    direction,
                    compute_result,
                    write,
                    wire,
                    delivery,
                },
            )
        }
        REQUEST_AUCTION_GOODS_LOG => {
            let first_id = message.base_mut().get_long().unwrap_or(0);
            let player_id = message.base_mut().get_long().unwrap_or(0);
            let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
            let mut response = CMessage::new(0x0008_040A);
            response.base_mut().add_long(player_id);
            response.base_mut().add_long(first_id);
            let write = auction_log.add_byte_goods_log(player_id, guid, Some(&mut response));
            if let AuctionGoodsLogWriteOutcome::BlockedMissingFact(source) = write {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::AuctionGoodsLogBlocked {
                        first_id,
                        player_id,
                        guid,
                        source,
                    },
                );
            }
            response.base_mut().update();
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                message.map_id(),
            );
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionGoodsLog {
                    first_id,
                    player_id,
                    guid,
                    write,
                    wire,
                    delivery,
                },
            )
        }
        COLLECT_AUCTION_NOTICE => {
            let player_id = message.base_mut().get_long().unwrap_or(0);
            if game.online_player_by_id(player_id as u32).is_none() {
                return WorldServerAuctionMessageDispatch::Handled(
                    WorldServerAuctionMessageOutcome::NoOp { request_type },
                );
            }
            let mut response = CMessage::new(0x0008_040B);
            let mut write_queue = WorldAuctionNoticeWriteQueue::new(game);
            let collection = auction_log.collect_no_notice(player_id, &mut response, &mut write_queue);
            response.base_mut().update();
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                message.map_id(),
            );
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionNoticeCollected {
                    collection,
                    queued_sql_count: write_queue.queued_sql_count,
                    wire,
                    delivery,
                },
            )
        }
        APPLY_AUCTION_PLAYER_SNAPSHOT => {
            let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
            let (decode, cursor_after) = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                let decode = game.decord_online_player_by_id(
                    player_id,
                    source,
                    cursor,
                    registry,
                    coefficients,
                );
                (decode, *cursor)
            };
            WorldServerAuctionMessageDispatch::Handled(
                WorldServerAuctionMessageOutcome::AuctionPlayerSnapshotApplied {
                    player_id,
                    decode,
                    cursor_after,
                },
            )
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\auction.cpp:10
// RVA: 0x000A5650
// ADDRESS: 004a5650
// PROTOTYPE: void __cdecl OnMSG_S2W_AUCTION(CMessage * param_1)
//
// IMPLEMENTED_OWNER: `on_msg_s2w_auction` выше покрывает все literal case
// `0x60801..=0x60814`, включая exact смену opcode, порядок decode/DB/send и
// сохранённые safe-границы повреждённого wire. Raw switch оставлен только как
// локальное доказательство уже материализованного owner-а.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
