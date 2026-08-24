//! World→Game auction handler.
//!
//! Точная пара GameServer EXE/PDB и owner
//! `server/gameserver/appserver/message/onmsg_w2s_auction.cpp` подтверждают
//! selectors `0x80401..0x80403`, `0x80409..0x8040D`, `0x8040F` и `0x80410`:
//! добавление временного `CGoodsNode` в Game-specific owner map, reconciliation
//! с World GUID-set, catalog/log/client relay, auction-state и полную YuanBao
//! container/client mutation, а stall-result либо сообщает отказ, либо
//! публикует `0x90201` в реальный локальный Game FIFO. GameServer primary map
//! хранит `GUID -> owner id`, а не MiscServer-owned node; это исключает
//! исходный stack-pointer lifetime без изменения наблюдаемого результата.
//! Обрезанный payload заменяет небезопасное чтение за буфером typed error-ом
//! с сохранением уже выполненных cursor/field effects. Остальные selectors
//! остаются RAW ниже.

use std::collections::BTreeMap;

use crate::gameserver::appserver::message::unibillmessage::IncrementShopBillingContext;
use crate::gameserver::appserver::player::PlayerYuanBaoChange;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::SendMessageError;
use crate::public::aucitionroom::GameAuctionRemoval;
use crate::public::auctionlog::{AuctionLogNode, AuctionLogSystemTime};
use crate::public::auctionnode::{CGoodsNode, GoodsNodeUnserializeError};

const WORLD_AUCTION_ADD_ITEM_MESSAGE: i32 = 0x0008_0401;
const WORLD_AUCTION_UNITY_MESSAGE: i32 = 0x0008_0402;
const WORLD_AUCTION_STATE_MESSAGE: i32 = 0x0008_0403;
const WORLD_AUCTION_DIRECT_RELAY_MESSAGE: i32 = 0x0008_0409;
const WORLD_AUCTION_PLAYER_RELAY_MESSAGE: i32 = 0x0008_040a;
const WORLD_AUCTION_LOG_NOTICE_MESSAGE: i32 = 0x0008_040b;
const WORLD_AUCTION_CONDITION_MESSAGE: i32 = 0x0008_040c;
const WORLD_AUCTION_STALL_RESULT_MESSAGE: i32 = 0x0008_040d;
const WORLD_AUCTION_BROADCAST_MESSAGE: i32 = 0x0008_040f;
const WORLD_AUCTION_YUAN_BAO_MESSAGE: i32 = 0x0008_0410;
const CLIENT_AUCTION_GOODS_REMOVED_MESSAGE: i32 = 0x000c_0702;
const CLIENT_AUCTION_DIRECT_RELAY_MESSAGE: i32 = 0x000c_0707;
const CLIENT_AUCTION_PLAYER_RELAY_MESSAGE: i32 = 0x000c_0708;
const CLIENT_AUCTION_CONDITION_MESSAGE: i32 = 0x000c_070a;
const CLIENT_AUCTION_BROADCAST_MESSAGE: i32 = 0x000c_010b;
const LOCAL_PLAYER_SHOP_OPEN_MESSAGE: i32 = 0x0009_0201;

pub(crate) trait WorldAuctionRuntime: IncrementShopBillingContext {
    fn auction_wall_time_seconds(&mut self) -> u32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldAuctionMessageError {
    AddItemDecode(GoodsNodeUnserializeError),
    MissingUnityTerminator,
    TruncatedUnityGuid {
        offset: usize,
        needed: usize,
        available: usize,
    },
    MissingEnabledLong,
    MissingRelayPlayerId {
        selector: i32,
    },
    MissingConditionPlayerId,
    MissingConditionField {
        field: &'static str,
    },
    MissingLogPlayerId,
    MissingLogCount,
    TruncatedLogNode {
        record_index: u32,
    },
    LogDescriptionWithoutTerminator {
        record_index: u32,
    },
    LogNoticeTemplateMismatch,
    LogNoticeOutsideLegacyBuffer {
        length: usize,
    },
    MissingYuanBaoPlayerId,
    MissingYuanBaoAmount,
    MissingStallPlayerId,
    MissingStallResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameAuctionRemovalDispatch {
    pub(crate) removal: GameAuctionRemoval,
    pub(crate) delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldAuctionMessageReport {
    ItemAdded {
        guid: crate::public::guid::CGuid,
        owner_id: u32,
        added: bool,
    },
    GoodsUnified {
        world_goods: Vec<crate::public::guid::CGuid>,
        removals: Vec<GameAuctionRemovalDispatch>,
    },
    StateChanged {
        enabled: bool,
        last_check_seconds: u32,
    },
    ClientRelay {
        selector: i32,
        player_id: i32,
        payload: Vec<u8>,
        delivery: Option<i32>,
    },
    ClientBroadcast {
        delivery: Result<i32, SendMessageError>,
    },
    AuctionCondition {
        player_id: i32,
        player_found: bool,
        created_goods: Vec<u32>,
        delivery: Option<i32>,
    },
    AuctionLogNotices {
        player_id: i32,
        declared_records: u32,
        player_found: bool,
        processed_records: u32,
        deliveries: Vec<i32>,
    },
    YuanBaoChanged {
        player_id: i32,
        requested: u32,
        change: Option<PlayerYuanBaoChange>,
        deliveries: Vec<i32>,
    },
    StallResult {
        player_id: i32,
        result: i32,
        player_found: bool,
        notice_delivery: Option<i32>,
        local_open_queued: bool,
    },
}

/// Материализует достигнутые sync/relay/state/YuanBao ветви handler-а.
/// `None` означает, что сообщение должен идти в оставшийся auction owner.
pub(crate) fn dispatch_world_auction_message<Runtime: WorldAuctionRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<WorldAuctionMessageReport, WorldAuctionMessageError>> {
    match message.message_type() {
        WORLD_AUCTION_ADD_ITEM_MESSAGE => {
            let mut item = CGoodsNode::new();
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                item.unserialize(source, cursor)
            };
            if let Err(error) = decode {
                return Some(Err(WorldAuctionMessageError::AddItemDecode(error)));
            }
            let guid = item.guid();
            let owner_id = item.owner_id();
            let added = game.auction_room_mut().add_item_to_auction_room(&mut item);
            Some(Ok(WorldAuctionMessageReport::ItemAdded {
                guid,
                owner_id,
                added,
            }))
        }
        WORLD_AUCTION_UNITY_MESSAGE => {
            let mut world_goods = BTreeMap::new();
            loop {
                let Some(marker) = message.base_mut().get_char() else {
                    return Some(Err(WorldAuctionMessageError::MissingUnityTerminator));
                };
                if marker == 0 {
                    break;
                }

                let offset = message.base_mut().cursor();
                let wire = message.as_wire_bytes();
                let available = wire.len().saturating_sub(offset);
                let Some(presence) = wire.get(offset).copied() else {
                    return Some(Err(WorldAuctionMessageError::TruncatedUnityGuid {
                        offset,
                        needed: 1,
                        available,
                    }));
                };
                let needed = if presence == 0 { 1 } else { 17 };
                if available < needed {
                    return Some(Err(WorldAuctionMessageError::TruncatedUnityGuid {
                        offset,
                        needed,
                        available,
                    }));
                }
                if let Some(guid) = message.base_mut().get_guid() {
                    world_goods.insert(guid, false);
                }
            }

            let mut removals = Vec::new();
            while let Some(removal) = game.auction_room().next_unity_removal(&world_goods) {
                let delivery = if removal.owner_id != 0 {
                    let mut notice = CMessage::new(CLIENT_AUCTION_GOODS_REMOVED_MESSAGE);
                    notice.base_mut().add_ulong(removal.owner_id);
                    notice.base_mut().add_guid(removal.guid);
                    Some(notice.send_all(game.current_net_server()))
                } else {
                    None
                };
                game.auction_room_mut().commit_unity_removal(removal);
                removals.push(GameAuctionRemovalDispatch { removal, delivery });
            }
            Some(Ok(WorldAuctionMessageReport::GoodsUnified {
                world_goods: world_goods.into_keys().collect(),
                removals,
            }))
        }
        WORLD_AUCTION_STATE_MESSAGE => {
            let Some(enabled) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingEnabledLong));
            };
            let enabled = enabled != 0;
            let last_check_seconds = if enabled {
                runtime.auction_wall_time_seconds()
            } else {
                game.auction_last_check_seconds()
            };
            game.set_auction_state(enabled, last_check_seconds);
            Some(Ok(WorldAuctionMessageReport::StateChanged {
                enabled,
                last_check_seconds,
            }))
        }
        selector @ (WORLD_AUCTION_DIRECT_RELAY_MESSAGE | WORLD_AUCTION_PLAYER_RELAY_MESSAGE) => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingRelayPlayerId {
                    selector,
                }));
            };
            let payload = message.unread_bytes().to_vec();
            let player_found = selector == WORLD_AUCTION_DIRECT_RELAY_MESSAGE
                || game.find_player(player_id).is_some();
            let delivery = player_found.then(|| {
                let client_selector = if selector == WORLD_AUCTION_DIRECT_RELAY_MESSAGE {
                    CLIENT_AUCTION_DIRECT_RELAY_MESSAGE
                } else {
                    CLIENT_AUCTION_PLAYER_RELAY_MESSAGE
                };
                let mut response = CMessage::new(client_selector);
                response.base_mut().add(&payload);
                response.send_to_player(game.net_server(), player_id)
            });
            Some(Ok(WorldAuctionMessageReport::ClientRelay {
                selector,
                player_id,
                payload,
                delivery,
            }))
        }
        WORLD_AUCTION_CONDITION_MESSAGE => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingConditionPlayerId));
            };
            if game.find_player(player_id).is_none() {
                return Some(Ok(WorldAuctionMessageReport::AuctionCondition {
                    player_id,
                    player_found: false,
                    created_goods: Vec::new(),
                    delivery: None,
                }));
            }

            let mut response = CMessage::new(CLIENT_AUCTION_CONDITION_MESSAGE);
            let mut created_goods = Vec::new();
            loop {
                let Some(goods_index) = message.base_mut().get_long() else {
                    return Some(Err(WorldAuctionMessageError::MissingConditionField {
                        field: "goods index",
                    }));
                };
                let Some(second) = message.base_mut().get_long() else {
                    return Some(Err(WorldAuctionMessageError::MissingConditionField {
                        field: "second condition",
                    }));
                };
                let Some(third) = message.base_mut().get_long() else {
                    return Some(Err(WorldAuctionMessageError::MissingConditionField {
                        field: "third condition",
                    }));
                };
                let Some(goods) = game
                    .create_goods_batch(goods_index as u32, 1)
                    .into_iter()
                    .next()
                else {
                    response.base_mut().add_long(0);
                    break;
                };
                response.base_mut().add_long(1);
                response
                    .base_mut()
                    .add(&runtime.encode_goods_for_old_client(&goods));
                response.base_mut().add_long(second);
                response.base_mut().add_long(third);
                created_goods.push(goods_index as u32);
            }
            let delivery = response.send_to_player(game.net_server(), player_id);
            Some(Ok(WorldAuctionMessageReport::AuctionCondition {
                player_id,
                player_found: true,
                created_goods,
                delivery: Some(delivery),
            }))
        }
        WORLD_AUCTION_LOG_NOTICE_MESSAGE => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingLogPlayerId));
            };
            let Some(declared_records) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingLogCount));
            };
            let declared_records = declared_records as u32;
            if game.find_player(player_id).is_none() || declared_records == 0 {
                return Some(Ok(WorldAuctionMessageReport::AuctionLogNotices {
                    player_id,
                    declared_records,
                    player_found: game.find_player(player_id).is_some(),
                    processed_records: 0,
                    deliveries: Vec::new(),
                }));
            }

            let mut deliveries = Vec::new();
            for record_index in 0..declared_records {
                let mut bytes = [0; 0x150];
                if !message.base_mut().get(&mut bytes) {
                    return Some(Err(WorldAuctionMessageError::TruncatedLogNode {
                        record_index,
                    }));
                }
                let node = AuctionLogNode::from_legacy_bytes(&bytes);
                if node.notice == 0 && node.operation_type == 1 {
                    let template = game.get_string_by_id(b"GPM001");
                    let text = if template.first().is_none_or(|byte| *byte == 0) {
                        Vec::new()
                    } else {
                        let Some(description) = node.description() else {
                            return Some(Err(
                                WorldAuctionMessageError::LogDescriptionWithoutTerminator {
                                    record_index,
                                },
                            ));
                        };
                        match format_auction_log_notice(template, description, node.time) {
                            Ok(text) => text,
                            Err(error) => return Some(Err(error)),
                        }
                    };
                    deliveries.push(
                        colored_player_notice_message(0xffff_ffff, 0, &text)
                            .send_to_player(game.net_server(), player_id),
                    );
                }
            }
            Some(Ok(WorldAuctionMessageReport::AuctionLogNotices {
                player_id,
                declared_records,
                player_found: true,
                processed_records: declared_records,
                deliveries,
            }))
        }
        WORLD_AUCTION_BROADCAST_MESSAGE => {
            message.set_message_type(CLIENT_AUCTION_BROADCAST_MESSAGE);
            let delivery = message.send_all(game.current_net_server());
            Some(Ok(WorldAuctionMessageReport::ClientBroadcast { delivery }))
        }
        WORLD_AUCTION_STALL_RESULT_MESSAGE => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingStallPlayerId));
            };
            let Some(result) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingStallResult));
            };
            if game.find_player(player_id).is_none() {
                return Some(Ok(WorldAuctionMessageReport::StallResult {
                    player_id,
                    result,
                    player_found: false,
                    notice_delivery: None,
                    local_open_queued: false,
                }));
            }

            let mut notice_delivery = None;
            let mut local_open_queued = false;
            if result == 0 {
                notice_delivery = Some(
                    colored_player_notice_message(
                        0xffff_ff00,
                        0xffff_0000,
                        game.get_string_by_id(b"GPM009"),
                    )
                    .send_to_player(game.net_server(), player_id),
                );
            } else if let Some(net_server) = game.current_net_server() {
                let mut open = CMessage::new(LOCAL_PLAYER_SHOP_OPEN_MESSAGE);
                open.base_mut().add_long(game.server_ids().1);
                open.apply_player_context(player_id, None);
                net_server.publish_local_message(open);
                local_open_queued = true;
            }
            Some(Ok(WorldAuctionMessageReport::StallResult {
                player_id,
                result,
                player_found: true,
                notice_delivery,
                local_open_queued,
            }))
        }
        WORLD_AUCTION_YUAN_BAO_MESSAGE => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingYuanBaoPlayerId));
            };
            let Some(requested) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingYuanBaoAmount));
            };
            let requested = requested as u32;
            let Some(previous) = game.find_player(player_id).map(|player| player.yuan_bao()) else {
                return Some(Ok(WorldAuctionMessageReport::YuanBaoChanged {
                    player_id,
                    requested,
                    change: None,
                    deliveries: Vec::new(),
                }));
            };
            let created_currency = if previous < requested {
                game.create_goods_batch(
                    game.goods_factory().get_yuan_bao_index(),
                    requested.wrapping_sub(previous),
                )
            } else {
                Vec::new()
            };
            let change = game
                .set_player_yuan_bao(player_id, requested, created_currency)
                .expect("auction YuanBao player проверен перед mutation");
            let deliveries = runtime.publish_increment_shop_yuan_bao_change(&change);
            Some(Ok(WorldAuctionMessageReport::YuanBaoChanged {
                player_id,
                requested,
                change: Some(change),
                deliveries,
            }))
        }
        _ => None,
    }
}

fn format_auction_log_notice(
    template: &[u8],
    description: &[u8],
    time: AuctionLogSystemTime,
) -> Result<Vec<u8>, WorldAuctionMessageError> {
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let numbers = [
        time.year,
        time.month,
        time.day,
        time.hour,
        time.minute,
        time.second,
    ];
    let mut number_index = 0usize;
    let mut description_written = false;
    let mut output = Vec::new();
    let mut cursor = 0usize;
    while cursor < template.len() {
        if template[cursor..].starts_with(b"%s") {
            if description_written {
                return Err(WorldAuctionMessageError::LogNoticeTemplateMismatch);
            }
            output.extend_from_slice(description);
            description_written = true;
            cursor += 2;
        } else if template[cursor..].starts_with(b"%ld") {
            let Some(value) = numbers.get(number_index) else {
                return Err(WorldAuctionMessageError::LogNoticeTemplateMismatch);
            };
            output.extend_from_slice(value.to_string().as_bytes());
            number_index += 1;
            cursor += 3;
        } else {
            output.push(template[cursor]);
            cursor += 1;
        }
        if output.len() > 1027 {
            return Err(WorldAuctionMessageError::LogNoticeOutsideLegacyBuffer {
                length: output.len(),
            });
        }
    }
    if !description_written || number_index != numbers.len() {
        return Err(WorldAuctionMessageError::LogNoticeTemplateMismatch);
    }
    Ok(output)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_w2s_auction.cpp

// ============================================================================
// FUNCTION: OnMSG_W2S_AUCTION
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_w2s_auction.cpp:21
// RVA: 0x000983A0
// ADDRESS: 004983a0
// PROTOTYPE: void __cdecl OnMSG_W2S_AUCTION(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
