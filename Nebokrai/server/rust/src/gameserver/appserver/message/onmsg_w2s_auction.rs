//! World→Game auction handler.
//!
//! Точная пара GameServer EXE/PDB и owner
//! `server/gameserver/appserver/message/onmsg_w2s_auction.cpp` подтверждают
//! selectors `0x80401..0x80410`, кроме ещё RAW `0x80411+`:
//! добавление временного `CGoodsNode` в Game-specific owner map, reconciliation
//! с World GUID-set, catalog/log/client relay, auction-state и полную YuanBao
//! container/client mutation, а stall-result либо сообщает отказ, либо
//! публикует `0x90201` в реальный локальный Game FIFO. Self/all lists сохраняют
//! signed count/marker quirks, переводят nested goods в old-client wire и
//! после self-list публикуют открытый player-container scale. Return `0x80404`
//! декодирует сохранённый goods, исключает duplicate GUID, логирует `0x60217`,
//! пополняет auction wallet либо new/stack auction goods, применяет bind и
//! публикует container/update/notice/scale effects. Полный `AddByteGS2WS`
//! остаётся локальной границей persisted player snapshot, поэтому частичный
//! `0x6080E` не создаётся. GameServer primary map
//! хранит `GUID -> owner id`, а не MiscServer-owned node; это исключает
//! исходный stack-pointer lifetime без изменения наблюдаемого результата.
//! Buy-result `0x80406` хранит один trusted pending node у player, возвращает
//! competing/offline node, списывает gold с exact client wallet effect либо
//! передаёт YuanBao trade в Billing, а success публикует два `0x60214`,
//! `0x60806` и buyer/seller self-query в исходном порядке.
//! Обрезанный payload заменяет небезопасное чтение за буфером typed error-ом
//! с сохранением уже выполненных cursor/field effects. Остальные selectors
//! остаются RAW ниже.

use std::collections::BTreeMap;

use crate::gameserver::appserver::container::cgoodscontainer::GoodsStackMergeOutcome;
use crate::gameserver::appserver::container::cvolumelimitgoodscontainer::VolumeGoodsAddOutcome;
use crate::gameserver::appserver::container::cwallet::{
    CurrencyDecreaseOutcome, CurrencyIncreaseOutcome,
};
use crate::gameserver::appserver::cs2ccontainerobjectamountchange::CS2CContainerObjectAmountChange;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{
    CS2CContainerObjectMove, ContainerObjectMoveOperation,
};
use crate::gameserver::appserver::goods::cgoods::GoodsDecodeError;
use crate::gameserver::appserver::message::unibillmessage::{
    IncrementShopBillingContext, auction_billing_local_system_time,
};
use crate::gameserver::appserver::player::{
    AuctionSelfGoodsRefresh, PlayerAuctionGoodsReturn, PlayerAuctionMoneyChange,
    PlayerYuanBaoChange,
};
use crate::gameserver::gameserver::game::{
    CGame, PersonalShopRecollection, colored_player_notice_message, game_wall_time_seconds,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::SendMessageError;
use crate::public::aucitionroom::GameAuctionRemoval;
use crate::public::auctionlog::{AuctionLogNode, AuctionLogSystemTime};
use crate::public::auctionnode::{CGoodsNode, GoodsNodeSerializeError, GoodsNodeUnserializeError};

const WORLD_AUCTION_ADD_ITEM_MESSAGE: i32 = 0x0008_0401;
const WORLD_AUCTION_UNITY_MESSAGE: i32 = 0x0008_0402;
const WORLD_AUCTION_STATE_MESSAGE: i32 = 0x0008_0403;
const WORLD_AUCTION_RETURN_GOODS_MESSAGE: i32 = 0x0008_0404;
const WORLD_AUCTION_REFRESH_SELF_GOODS_MESSAGE: i32 = 0x0008_0405;
const WORLD_AUCTION_BUY_RESULT_MESSAGE: i32 = 0x0008_0406;
const WORLD_AUCTION_SELF_LIST_MESSAGE: i32 = 0x0008_0407;
const WORLD_AUCTION_ALL_LIST_MESSAGE: i32 = 0x0008_0408;
const WORLD_AUCTION_DIRECT_RELAY_MESSAGE: i32 = 0x0008_0409;
const WORLD_AUCTION_PLAYER_RELAY_MESSAGE: i32 = 0x0008_040a;
const WORLD_AUCTION_LOG_NOTICE_MESSAGE: i32 = 0x0008_040b;
const WORLD_AUCTION_CONDITION_MESSAGE: i32 = 0x0008_040c;
const WORLD_AUCTION_STALL_RESULT_MESSAGE: i32 = 0x0008_040d;
const WORLD_AUCTION_RECOLLECT_STALLS_MESSAGE: i32 = 0x0008_040e;
const WORLD_AUCTION_BROADCAST_MESSAGE: i32 = 0x0008_040f;
const WORLD_AUCTION_YUAN_BAO_MESSAGE: i32 = 0x0008_0410;
const CLIENT_AUCTION_GOODS_REMOVED_MESSAGE: i32 = 0x000c_0702;
const CLIENT_AUCTION_ALL_LIST_MESSAGE: i32 = 0x000c_0703;
const CLIENT_AUCTION_SELF_LIST_MESSAGE: i32 = 0x000c_0704;
const CLIENT_AUCTION_DIRECT_RELAY_MESSAGE: i32 = 0x000c_0707;
const CLIENT_AUCTION_PLAYER_RELAY_MESSAGE: i32 = 0x000c_0708;
const CLIENT_AUCTION_CONDITION_MESSAGE: i32 = 0x000c_070a;
const CLIENT_AUCTION_SCALE_MESSAGE: i32 = 0x000c_0709;
const CLIENT_AUCTION_BROADCAST_MESSAGE: i32 = 0x000c_010b;
const LOCAL_PLAYER_SHOP_OPEN_MESSAGE: i32 = 0x0009_0201;
const GAME_AUCTION_REFRESH_SELF_GOODS_MESSAGE: i32 = 0x0006_080a;
const WORLD_AUCTION_RETURN_LOG_MESSAGE: i32 = 0x0006_0217;
const WORLD_AUCTION_RETURN_NODE_MESSAGE: i32 = 0x0006_0801;
const WORLD_AUCTION_QUERY_SELF_MESSAGE: i32 = 0x0006_0802;
const WORLD_AUCTION_BUY_SUCCEEDED_MESSAGE: i32 = 0x0006_0806;
const WORLD_AUCTION_LOG_MESSAGE: i32 = 0x0006_0214;
const BILLING_AUCTION_BUY_MESSAGE: i32 = 0x000e_f203;
const CLIENT_GOODS_UPDATE_MESSAGE: i32 = 0x000b_f918;
const PLAYER_TYPE: i32 = 400;
const AUCTION_GOODS_EXTEND_ID: i32 = 14;
const AUCTION_MONEY_EXTEND_ID: i32 = 15;

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
    MissingReturnField(&'static str),
    TruncatedReturnGoods {
        declared: usize,
        available: usize,
    },
    ReturnGoodsDecode(GoodsDecodeError),
    MissingBuyResultField(&'static str),
    BuyNodeDecode(GoodsNodeUnserializeError),
    BuyNodeSerialize(GoodsNodeSerializeError),
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
    MissingRefreshSelfGoodsPlayerId,
    MissingListPlayerId {
        selector: i32,
    },
    MissingListCount {
        field: &'static str,
    },
    MissingListMarker {
        record_index: u32,
    },
    ListNodeDecode(GoodsNodeUnserializeError),
    ListGoodsDecode(GoodsDecodeError),
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
    GoodsReturned {
        player_id: i32,
        goods_id: crate::public::guid::CGuid,
        bind_type: i32,
        declared_size: i32,
        player_found: bool,
        duplicate: bool,
        log_delivery: Option<Result<i32, SendMessageError>>,
        money_change: Option<PlayerAuctionMoneyChange>,
        goods_return: Option<PlayerAuctionGoodsReturn>,
        deliveries: Vec<i32>,
        snapshot_refresh_required: bool,
    },
    BuyResult {
        result: i32,
        player_id: Option<i32>,
        buyer_found: bool,
        money_type: Option<u8>,
        world_deliveries: Vec<Result<i32, SendMessageError>>,
        client_deliveries: Vec<i32>,
        billing_delivery: Option<Result<i32, SendMessageError>>,
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
    StallsRecollected {
        recollections: Vec<PersonalShopRecollection>,
    },
    SelfGoodsRefresh {
        player_id: i32,
        refresh: Option<AuctionSelfGoodsRefresh>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    AuctionList {
        selector: i32,
        player_id: i32,
        player_found: bool,
        declared_records: u32,
        emitted_records: u32,
        delivery: Option<i32>,
        scale_delivery: Option<i32>,
    },
}

/// Материализует достигнутые sync/relay/state/YuanBao ветви handler-а.
/// `None` означает, что сообщение должен идти в оставшийся auction owner.
pub(crate) fn dispatch_world_auction_message<Runtime, Tick>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    mut tick_ms: Tick,
) -> Option<Result<WorldAuctionMessageReport, WorldAuctionMessageError>>
where
    Runtime: IncrementShopBillingContext,
    Tick: FnMut(&mut Runtime) -> u32,
{
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
                game_wall_time_seconds() as u32
            } else {
                game.auction_last_check_seconds()
            };
            game.set_auction_state(enabled, last_check_seconds);
            Some(Ok(WorldAuctionMessageReport::StateChanged {
                enabled,
                last_check_seconds,
            }))
        }
        WORLD_AUCTION_RETURN_GOODS_MESSAGE => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingReturnField(
                    "player id",
                )));
            };
            let goods_id = match message.base_mut().get_guid() {
                Some(value) => value,
                None => {
                    return Some(Err(WorldAuctionMessageError::MissingReturnField(
                        "goods guid",
                    )));
                }
            };
            if message.base_mut().get_long().is_none() {
                return Some(Err(WorldAuctionMessageError::MissingReturnField(
                    "goods base index",
                )));
            }
            let Some(bind_type) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingReturnField(
                    "bind type",
                )));
            };
            let Some(declared_size) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingReturnField(
                    "goods size",
                )));
            };
            let Some(player) = game.find_player(player_id) else {
                return Some(Ok(WorldAuctionMessageReport::GoodsReturned {
                    player_id,
                    goods_id,
                    bind_type,
                    declared_size,
                    player_found: false,
                    duplicate: false,
                    log_delivery: None,
                    money_change: None,
                    goods_return: None,
                    deliveries: Vec::new(),
                    snapshot_refresh_required: false,
                }));
            };
            if player.get_goods_by_id(goods_id).is_some() {
                return Some(Ok(WorldAuctionMessageReport::GoodsReturned {
                    player_id,
                    goods_id,
                    bind_type,
                    declared_size,
                    player_found: true,
                    duplicate: true,
                    log_delivery: None,
                    money_change: None,
                    goods_return: None,
                    deliveries: Vec::new(),
                    snapshot_refresh_required: false,
                }));
            }
            if declared_size <= 1 {
                return Some(Ok(WorldAuctionMessageReport::GoodsReturned {
                    player_id,
                    goods_id,
                    bind_type,
                    declared_size,
                    player_found: true,
                    duplicate: false,
                    log_delivery: None,
                    money_change: None,
                    goods_return: None,
                    deliveries: Vec::new(),
                    snapshot_refresh_required: false,
                }));
            }

            let declared = declared_size as usize;
            let payload = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                let available = source.len().saturating_sub(*cursor);
                if available < declared {
                    return Some(Err(WorldAuctionMessageError::TruncatedReturnGoods {
                        declared,
                        available,
                    }));
                }
                let end = *cursor + declared;
                let payload = source[*cursor..end].to_vec();
                *cursor = end;
                payload
            };
            let incoming = match game.decode_auction_goods(&payload) {
                Ok(goods) => goods,
                Err(error) => {
                    return Some(Err(WorldAuctionMessageError::ReturnGoodsDecode(error)));
                }
            };
            let incoming_amount = incoming.amount();
            let incoming_index = incoming.base_properties_index();
            let pre_bind_payload = runtime.encode_goods_for_old_client(&incoming);
            let mut audit = CMessage::new(WORLD_AUCTION_RETURN_LOG_MESSAGE);
            audit.base_mut().add_long(player_id);
            audit.base_mut().add_ulong(incoming_amount);
            audit.base_mut().add_guid(goods_id);
            let log_delivery = Some(audit.send(game, false));
            let mut deliveries = Vec::new();

            if incoming_index == game.goods_factory().get_gold_coin_index() {
                let previous = game
                    .find_player(player_id)
                    .expect("auction return player проверен")
                    .auction_money();
                let created_currency = if previous == 0 {
                    game.create_goods_batch(incoming_index, incoming_amount)
                } else {
                    Vec::new()
                };
                let money_change = game
                    .increase_player_auction_money(player_id, incoming_amount, created_currency)
                    .expect("auction return player проверен перед wallet mutation");
                match &money_change.outcome {
                    CurrencyIncreaseOutcome::Created(added) => {
                        let stored = game
                            .find_player(player_id)
                            .and_then(|player| player.auction_money_goods())
                            .expect("created auction currency хранится в player wallet");
                        let mut move_message = CS2CContainerObjectMove::default();
                        move_message.set_operation(ContainerObjectMoveOperation::NewObject);
                        move_message.set_destination_container(
                            added.owner_type,
                            added.owner_id,
                            added.position,
                        );
                        move_message.set_destination_container_extend_id(AUCTION_MONEY_EXTEND_ID);
                        move_message.set_destination_object(
                            added.identity.object_type,
                            added.identity.ex_id,
                        );
                        move_message.set_destination_object_amount(added.amount);
                        move_message.set_object_stream(runtime.encode_goods_for_old_client(stored));
                        deliveries.push(move_message.send_to_player(game, player_id));
                    }
                    CurrencyIncreaseOutcome::Increased(change) => {
                        let mut amount = CS2CContainerObjectAmountChange::default();
                        amount.set_source_container(
                            change.owner_type,
                            change.owner_id,
                            change.position,
                        );
                        amount.set_source_container_extend_id(AUCTION_MONEY_EXTEND_ID);
                        amount.set_object(change.identity.object_type, change.identity.ex_id);
                        amount.set_object_amount(change.new_amount);
                        deliveries.push(amount.send_to_player(game, player_id));
                    }
                    CurrencyIncreaseOutcome::NoChange
                    | CurrencyIncreaseOutcome::InvalidStoredCurrency { .. }
                    | CurrencyIncreaseOutcome::CapacityExceeded { .. }
                    | CurrencyIncreaseOutcome::CreationFailed => {}
                }
                deliveries.push(
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM002"))
                        .send_to_player(game.net_server(), player_id),
                );
                return Some(Ok(WorldAuctionMessageReport::GoodsReturned {
                    player_id,
                    goods_id,
                    bind_type,
                    declared_size,
                    player_found: true,
                    duplicate: false,
                    log_delivery,
                    money_change: Some(money_change),
                    goods_return: None,
                    deliveries,
                    snapshot_refresh_required: false,
                }));
            }

            let goods_return = game.return_player_auction_goods(player_id, incoming, bind_type);
            let snapshot_refresh_required = goods_return.is_some();
            if let Some(returned) = &goods_return {
                match &returned.outcome {
                    VolumeGoodsAddOutcome::Added(added) => {
                        let mut move_message = CS2CContainerObjectMove::default();
                        move_message.set_operation(ContainerObjectMoveOperation::NewObject);
                        move_message.set_destination_container(
                            added.owner_type,
                            added.owner_id,
                            returned.position,
                        );
                        move_message.set_destination_container_extend_id(AUCTION_GOODS_EXTEND_ID);
                        move_message.set_destination_object(
                            added.identity.object_type,
                            added.identity.ex_id,
                        );
                        move_message.set_destination_object_amount(added.amount);
                        move_message.set_object_stream(pre_bind_payload);
                        deliveries.push(move_message.send_to_player(game, player_id));
                    }
                    VolumeGoodsAddOutcome::Stack(GoodsStackMergeOutcome::Merged {
                        target, ..
                    }) => {
                        let mut amount = CS2CContainerObjectAmountChange::default();
                        amount.set_source_container(PLAYER_TYPE, player_id, returned.position);
                        amount.set_source_container_extend_id(AUCTION_GOODS_EXTEND_ID);
                        amount.set_object(target.object_type, target.ex_id);
                        amount.set_object_amount(
                            returned
                                .resulting_amount
                                .expect("merged auction goods имеют итоговое amount"),
                        );
                        deliveries.push(amount.send_to_player(game, player_id));
                    }
                    VolumeGoodsAddOutcome::Stack(_) | VolumeGoodsAddOutcome::Rejected(_) => {}
                }

                if let Some(identity) = returned.resulting_goods {
                    let stored = game
                        .find_player(player_id)
                        .and_then(|player| player.get_goods_by_id(identity.ex_id))
                        .expect("успешно возвращённый goods хранится в auction container");
                    let payload = runtime.encode_goods_for_old_client(stored);
                    let mut update = CMessage::new(CLIENT_GOODS_UPDATE_MESSAGE);
                    update.base_mut().add_long(player_id);
                    update.base_mut().add_guid(identity.ex_id);
                    update.base_mut().add_ulong(payload.len() as u32);
                    update.base_mut().add(&payload);
                    deliveries.push(update.send_to_player(game.net_server(), player_id));

                    let notice_id = match bind_type {
                        3 => Some(b"GPM003".as_slice()),
                        2 => Some(b"GPM004".as_slice()),
                        4 => Some(b"GPM005".as_slice()),
                        _ => None,
                    };
                    if let Some(notice_id) = notice_id {
                        deliveries.push(
                            colored_player_notice_message(
                                0xffff_ffff,
                                0,
                                game.get_string_by_id(notice_id),
                            )
                            .send_to_player(game.net_server(), player_id),
                        );
                    }
                }

                if let Some(goods_ids) = game
                    .find_player(player_id)
                    .and_then(|player| player.auction_scale_goods_ids())
                {
                    let mut scale = CMessage::new(CLIENT_AUCTION_SCALE_MESSAGE);
                    scale.base_mut().add_ulong(goods_ids.len() as u32);
                    for goods_id in goods_ids {
                        scale.base_mut().add_guid(goods_id);
                    }
                    deliveries.push(scale.send_to_player(game.net_server(), player_id));
                }
                // `AddByteGS2WS` требует полный persisted player snapshot.
                // Его RAW owner сохранён в player.rs; посылать частичный
                // `0x6080E` здесь было бы wire-несовместимой заглушкой.
            }
            Some(Ok(WorldAuctionMessageReport::GoodsReturned {
                player_id,
                goods_id,
                bind_type,
                declared_size,
                player_found: true,
                duplicate: false,
                log_delivery,
                money_change: None,
                goods_return,
                deliveries,
                snapshot_refresh_required,
            }))
        }
        WORLD_AUCTION_REFRESH_SELF_GOODS_MESSAGE => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(
                    WorldAuctionMessageError::MissingRefreshSelfGoodsPlayerId,
                ));
            };
            let refresh = game.refresh_player_auction_self_goods(player_id, || tick_ms(runtime));
            let delivery = match refresh {
                Some(AuctionSelfGoodsRefresh::Requested {
                    goods_space,
                    wallet_space,
                    ..
                }) => {
                    let mut response = CMessage::new(GAME_AUCTION_REFRESH_SELF_GOODS_MESSAGE);
                    response.base_mut().add_long(player_id);
                    response.base_mut().add_ulong(goods_space);
                    response.base_mut().add_ulong(wallet_space);
                    Some(response.send(game, false))
                }
                Some(AuctionSelfGoodsRefresh::Throttled { .. }) | None => None,
            };
            Some(Ok(WorldAuctionMessageReport::SelfGoodsRefresh {
                player_id,
                refresh,
                delivery,
            }))
        }
        WORLD_AUCTION_BUY_RESULT_MESSAGE => {
            let Some(result) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingBuyResultField(
                    "result",
                )));
            };
            if result == 0 {
                let Some(player_id) = message.base_mut().get_long() else {
                    return Some(Err(WorldAuctionMessageError::MissingBuyResultField(
                        "player id",
                    )));
                };
                return Some(Ok(WorldAuctionMessageReport::BuyResult {
                    result,
                    player_id: Some(player_id),
                    buyer_found: game.find_player(player_id).is_some(),
                    money_type: None,
                    world_deliveries: Vec::new(),
                    client_deliveries: Vec::new(),
                    billing_delivery: None,
                }));
            }
            if result != 1 {
                return Some(Ok(WorldAuctionMessageReport::BuyResult {
                    result,
                    player_id: None,
                    buyer_found: false,
                    money_type: None,
                    world_deliveries: Vec::new(),
                    client_deliveries: Vec::new(),
                    billing_delivery: None,
                }));
            }

            let mut incoming = CGoodsNode::new();
            let decode = {
                let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                incoming.unserialize(source, cursor)
            };
            if let Err(error) = decode {
                return Some(Err(WorldAuctionMessageError::BuyNodeDecode(error)));
            }
            let player_id = incoming.buyer_id() as i32;
            let Some(player) = game.find_player(player_id) else {
                incoming.prepare_return_to_auction();
                let delivery =
                    send_auction_node_world(&incoming, WORLD_AUCTION_RETURN_NODE_MESSAGE, game)
                        .map_err(WorldAuctionMessageError::BuyNodeSerialize);
                return match delivery {
                    Ok(delivery) => Some(Ok(WorldAuctionMessageReport::BuyResult {
                        result,
                        player_id: Some(player_id),
                        buyer_found: false,
                        money_type: Some(incoming.money_type()),
                        world_deliveries: vec![delivery],
                        client_deliveries: Vec::new(),
                        billing_delivery: None,
                    })),
                    Err(error) => Some(Err(error)),
                };
            };

            let had_pending = player.current_auction_buy_node().is_some();
            let mut world_deliveries = Vec::new();
            if had_pending {
                incoming.prepare_return_to_auction();
                match send_auction_node_world(&incoming, WORLD_AUCTION_RETURN_NODE_MESSAGE, game) {
                    Ok(delivery) => world_deliveries.push(delivery),
                    Err(error) => {
                        return Some(Err(WorldAuctionMessageError::BuyNodeSerialize(error)));
                    }
                }
            } else {
                let stored = game
                    .find_player_mut(player_id)
                    .expect("auction buyer проверен")
                    .set_current_auction_buy_node(incoming);
                debug_assert!(stored);
            }

            let mut node = game
                .find_player_mut(player_id)
                .expect("auction buyer проверен перед DoneCurAucBuyNode")
                .take_current_auction_buy_node()
                .expect("pending либо только что установлен");
            let money_type = node.money_type();
            let mut client_deliveries = Vec::new();
            let mut billing_delivery = None;
            if money_type == 1 {
                if node.seller_name().first().copied().unwrap_or(0) != 0 {
                    let buyer = game
                        .find_player(player_id)
                        .expect("auction buyer проверен перед Billing");
                    let buyer_ip = buyer.client_ip_text();
                    let (login_server_id, world_server_id) = game.server_ids();
                    let mut billing = CMessage::new(BILLING_AUCTION_BUY_MESSAGE);
                    billing.base_mut().add_long(3);
                    billing.base_mut().add_long(player_id);
                    billing.base_mut().add_ulong(node.seller_id());
                    add_c_string(&mut billing, buyer.account());
                    add_c_string(&mut billing, node.seller_name());
                    add_c_string(&mut billing, &buyer_ip);
                    add_c_string(&mut billing, node.seller_ip());
                    add_c_string(&mut billing, buyer.player_name());
                    add_c_string(&mut billing, node.seller_name());
                    billing.base_mut().add_ulong(node.seller_money());
                    billing.base_mut().add_ulong(node.base_index());
                    billing.base_mut().add_long(node.amount());
                    billing.base_mut().add_long(0);
                    billing.base_mut().add_long(0);
                    billing.base_mut().add_long(login_server_id);
                    billing.base_mut().add_long(world_server_id);
                    billing
                        .base_mut()
                        .add_guid(crate::public::guid::CGuid::GUID_INVALID);
                    billing_delivery = Some(billing.send_to_bs(game, false));
                }
                let stored = game
                    .find_player_mut(player_id)
                    .expect("auction buyer проверен после Billing")
                    .set_current_auction_buy_node(node);
                debug_assert!(stored);
            } else if money_type == 0 {
                let price = node.seller_money();
                let enough = game
                    .find_player(player_id)
                    .is_some_and(|buyer| buyer.money() >= price);
                if !enough {
                    node.prepare_return_to_auction();
                    match send_auction_node_world(&node, WORLD_AUCTION_RETURN_NODE_MESSAGE, game) {
                        Ok(delivery) => world_deliveries.push(delivery),
                        Err(error) => {
                            return Some(Err(WorldAuctionMessageError::BuyNodeSerialize(error)));
                        }
                    }
                } else {
                    let buyer_log_time = auction_billing_local_system_time();
                    let (buyer_log, mut seller_log, notice) =
                        build_auction_buy_log_effects(&node, game, buyer_log_time);
                    let mut buyer_audit = CMessage::new(WORLD_AUCTION_LOG_MESSAGE);
                    buyer_audit.base_mut().add(&buyer_log.to_legacy_bytes());
                    world_deliveries.push(buyer_audit.send(game, false));
                    client_deliveries.push(
                        colored_player_notice_message(0xffff_ffff, 0xffff_0000, &notice)
                            .send_to_player(game.net_server(), player_id),
                    );
                    seller_log.time = auction_billing_local_system_time();
                    let mut seller_audit = CMessage::new(WORLD_AUCTION_LOG_MESSAGE);
                    seller_audit.base_mut().add(&seller_log.to_legacy_bytes());
                    world_deliveries.push(seller_audit.send(game, false));
                    let decrease = game
                        .decrease_player_money(player_id, price)
                        .expect("auction buyer проверен перед gold decrease");
                    client_deliveries.extend(send_auction_money_decrease(
                        player_id,
                        &decrease.outcome,
                        game,
                    ));
                    match send_auction_node_world(&node, WORLD_AUCTION_BUY_SUCCEEDED_MESSAGE, game)
                    {
                        Ok(delivery) => world_deliveries.push(delivery),
                        Err(error) => {
                            return Some(Err(WorldAuctionMessageError::BuyNodeSerialize(error)));
                        }
                    }
                    for query_player_id in [player_id, node.seller_id() as i32] {
                        if query_player_id == player_id
                            || game.find_player(query_player_id).is_some()
                        {
                            let mut query = CMessage::new(WORLD_AUCTION_QUERY_SELF_MESSAGE);
                            query.base_mut().add_long(query_player_id);
                            world_deliveries.push(query.send(game, false));
                        }
                    }
                }
            } else {
                node.prepare_return_to_auction();
                match send_auction_node_world(&node, WORLD_AUCTION_RETURN_NODE_MESSAGE, game) {
                    Ok(delivery) => world_deliveries.push(delivery),
                    Err(error) => {
                        return Some(Err(WorldAuctionMessageError::BuyNodeSerialize(error)));
                    }
                }
            }
            Some(Ok(WorldAuctionMessageReport::BuyResult {
                result,
                player_id: Some(player_id),
                buyer_found: true,
                money_type: Some(money_type),
                world_deliveries,
                client_deliveries,
                billing_delivery,
            }))
        }
        selector @ (WORLD_AUCTION_SELF_LIST_MESSAGE | WORLD_AUCTION_ALL_LIST_MESSAGE) => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingListPlayerId {
                    selector,
                }));
            };
            if game.find_player(player_id).is_none() {
                return Some(Ok(WorldAuctionMessageReport::AuctionList {
                    selector,
                    player_id,
                    player_found: false,
                    declared_records: 0,
                    emitted_records: 0,
                    delivery: None,
                    scale_delivery: None,
                }));
            }

            let Some(first_count) = message.base_mut().get_long() else {
                return Some(Err(WorldAuctionMessageError::MissingListCount {
                    field: "declared records",
                }));
            };
            let declared_records = first_count as u32;
            let record_count = if selector == WORLD_AUCTION_ALL_LIST_MESSAGE {
                let Some(available_count) = message.base_mut().get_long() else {
                    return Some(Err(WorldAuctionMessageError::MissingListCount {
                        field: "available records",
                    }));
                };
                if available_count < first_count {
                    available_count
                } else {
                    first_count
                }
            } else {
                first_count
            };

            let client_selector = if selector == WORLD_AUCTION_SELF_LIST_MESSAGE {
                CLIENT_AUCTION_SELF_LIST_MESSAGE
            } else {
                CLIENT_AUCTION_ALL_LIST_MESSAGE
            };
            let mut response = CMessage::new(client_selector);
            response.base_mut().add_ulong(record_count as u32);
            let mut emitted_records = 0u32;
            let mut record_index = 0i32;
            let mut self_records_remaining = record_count as u32;
            while if selector == WORLD_AUCTION_SELF_LIST_MESSAGE {
                self_records_remaining != 0
            } else {
                record_index < record_count
            } {
                let Some(marker) = message.base_mut().get_long() else {
                    return Some(Err(WorldAuctionMessageError::MissingListMarker {
                        record_index: record_index as u32,
                    }));
                };
                response.base_mut().add_ulong(marker as u32);
                if marker != 1 {
                    if selector == WORLD_AUCTION_ALL_LIST_MESSAGE {
                        break;
                    }
                    record_index = record_index.wrapping_add(1);
                    self_records_remaining = self_records_remaining.wrapping_sub(1);
                    continue;
                }

                let mut node = CGoodsNode::new();
                let decode = {
                    let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                    node.unserialize(source, cursor)
                };
                if let Err(error) = decode {
                    return Some(Err(WorldAuctionMessageError::ListNodeDecode(error)));
                }
                let remaining = node
                    .add_ticket()
                    .saturating_sub(game_wall_time_seconds() as u32);
                let old_client_payload = if node.goods_bytes().is_empty() {
                    Vec::new()
                } else {
                    let goods = match game.decode_auction_goods(node.goods_bytes()) {
                        Ok(goods) => goods,
                        Err(error) => {
                            return Some(Err(WorldAuctionMessageError::ListGoodsDecode(error)));
                        }
                    };
                    runtime.encode_goods_for_old_client(&goods)
                };
                response.base_mut().add_ulong(remaining);
                response.base_mut().add_ulong(u32::from(node.money_type()));
                response.base_mut().add_ulong(node.seller_money());
                response
                    .base_mut()
                    .add_ulong(old_client_payload.len() as u32);
                response.base_mut().add(&old_client_payload);
                emitted_records = emitted_records.wrapping_add(1);
                record_index = record_index.wrapping_add(1);
                self_records_remaining = self_records_remaining.wrapping_sub(1);
            }

            let delivery = Some(response.send_to_player(game.net_server(), player_id));
            let scale_delivery = if selector == WORLD_AUCTION_SELF_LIST_MESSAGE {
                game.find_player(player_id).and_then(|player| {
                    let goods_ids = player.auction_scale_goods_ids()?;
                    let mut scale = CMessage::new(CLIENT_AUCTION_SCALE_MESSAGE);
                    scale.base_mut().add_ulong(goods_ids.len() as u32);
                    for goods_id in goods_ids {
                        scale.base_mut().add_guid(goods_id);
                    }
                    Some(scale.send_to_player(game.net_server(), player_id))
                })
            } else {
                None
            };
            Some(Ok(WorldAuctionMessageReport::AuctionList {
                selector,
                player_id,
                player_found: true,
                declared_records,
                emitted_records,
                delivery,
                scale_delivery,
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
        WORLD_AUCTION_RECOLLECT_STALLS_MESSAGE => {
            let recollections = game.recollect_personal_shops();
            Some(Ok(WorldAuctionMessageReport::StallsRecollected {
                recollections,
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

pub(super) fn send_auction_node_world(
    node: &CGoodsNode,
    selector: i32,
    game: &CGame,
) -> Result<Result<i32, SendMessageError>, GoodsNodeSerializeError> {
    let payload = node.serialize()?;
    let mut message = CMessage::new(selector);
    message.base_mut().add(&payload);
    Ok(message.send(game, false))
}

fn send_auction_money_decrease(
    player_id: i32,
    outcome: &CurrencyDecreaseOutcome,
    game: &CGame,
) -> Vec<i32> {
    match outcome {
        CurrencyDecreaseOutcome::Decreased(change) => {
            let mut amount = CS2CContainerObjectAmountChange::default();
            amount.set_source_container(change.owner_type, change.owner_id, change.position);
            amount.set_source_container_extend_id(4);
            amount.set_object(change.identity.object_type, change.identity.ex_id);
            amount.set_object_amount(change.new_amount);
            vec![amount.send_to_player(game, player_id)]
        }
        CurrencyDecreaseOutcome::Removed(removed) => {
            let identity = removed.goods.identity();
            let mut deleted = CS2CContainerObjectMove::default();
            deleted.set_operation(ContainerObjectMoveOperation::DeleteObject);
            deleted.set_source_container(removed.owner_type, removed.owner_id, removed.position);
            deleted.set_source_container_extend_id(4);
            deleted.set_source_object(identity.object_type, identity.ex_id, removed.amount);
            vec![deleted.send_to_player(game, player_id)]
        }
        CurrencyDecreaseOutcome::NoChange
        | CurrencyDecreaseOutcome::InvalidStoredCurrency { .. } => Vec::new(),
    }
}

pub(super) fn build_auction_buy_log_effects(
    node: &CGoodsNode,
    game: &CGame,
    time: AuctionLogSystemTime,
) -> (AuctionLogNode, AuctionLogNode, Vec<u8>) {
    let price = node.seller_money() as i32;
    let (fee, seller_money) = if node.money_type() == 0 {
        let setup = game.globe_setup();
        let fee = ((price as f32) * setup.auction_factor_c())
            .round()
            .max(setup.auction_service_fee_minimum().round())
            .min(setup.auction_service_fee_maximum().round()) as i32;
        if fee <= price {
            (fee, price.wrapping_sub(fee))
        } else {
            (price, 0)
        }
    } else if price > 2 {
        (3, price.wrapping_sub(3))
    } else {
        (price, 0)
    };
    let mut description = [0; 0x100];
    let goods_name = node
        .goods_name()
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default();
    let copy_len = goods_name.len().min(description.len().saturating_sub(1));
    description[..copy_len].copy_from_slice(&goods_name[..copy_len]);
    let common = AuctionLogNode {
        base_id: node.base_index() as i32,
        operation_type: -1,
        money_type: i32::from(node.money_type()),
        money_num: price,
        player_id: node.buyer_id() as i32,
        amount: node.amount(),
        fee: 0,
        notice: 0,
        time,
        description,
        guid: node.guid(),
        guid_key: crate::public::guid::CGuid::GUID_INVALID,
    };
    let buyer_log = common.clone();
    let seller_log = AuctionLogNode {
        operation_type: 1,
        money_num: seller_money,
        player_id: node.seller_id() as i32,
        fee,
        ..common
    };
    let money_name = game.get_string_by_id(b"GS0015");
    let catalog_name = game
        .goods_factory()
        .query_goods_name(node.base_index())
        .unwrap_or_default();
    let notice = format_auction_buy_notice(
        game.get_string_by_id(b"GPM018"),
        price,
        money_name,
        catalog_name,
    );
    (buyer_log, seller_log, notice)
}

fn format_auction_buy_notice(
    template: &[u8],
    price: i32,
    money_name: &[u8],
    goods_name: &[u8],
) -> Vec<u8> {
    enum Argument<'a> {
        Number(i32),
        Text(&'a [u8]),
    }
    let arguments = [
        Argument::Number(price),
        Argument::Text(money_name),
        Argument::Text(goods_name),
    ];
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let mut output = Vec::new();
    let mut argument = 0usize;
    let mut cursor = 0usize;
    while cursor < template.len() && output.len() < 131 {
        if template[cursor] == b'%' && template.get(cursor + 1) == Some(&b'%') {
            output.push(b'%');
            cursor += 2;
            continue;
        }
        if template[cursor] == b'%' {
            let conversion = template.get(cursor + 1).copied();
            let rendered = arguments
                .get(argument)
                .and_then(|value| match (conversion, value) {
                    (Some(b'd' | b'i'), Argument::Number(value)) => {
                        Some(value.to_string().into_bytes())
                    }
                    (Some(b's'), Argument::Text(value)) => Some(
                        value
                            .split(|byte| *byte == 0)
                            .next()
                            .unwrap_or_default()
                            .to_vec(),
                    ),
                    _ => None,
                });
            if let Some(rendered) = rendered {
                let remaining = 131usize.saturating_sub(output.len());
                output.extend_from_slice(&rendered[..rendered.len().min(remaining)]);
                argument += 1;
                cursor += 2;
                continue;
            }
        }
        output.push(template[cursor]);
        cursor += 1;
    }
    output
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(value);
    message.base_mut().add_byte(0);
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
