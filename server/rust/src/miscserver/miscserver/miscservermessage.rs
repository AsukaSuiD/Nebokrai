//! Auction-handler `miscservermessage.cpp`, подтверждённый `miscserver.exe` и
//! `miscserver.pdb` для `0x14ED01` и `0x14ED04..0x14ED09`.
//!
//! Добавление безусловно увеличивает wrapping add-счётчик; auction room владеет
//! отказом invalid/duplicate и del-счётчиком. Operation `3` сохраняет дефект
//! helper-а: он возвращает `false` даже после мутации, поэтому нулевой ack
//! отправляется всегда. Operation `1` игнорирует тот же return и не отвечает.
//!
//! Sync выполняется не более раза за turn, использует строгое wrapping-условие
//! 120 секунд и ставит done-count только после полного batch helper-а. Page
//! меняется до построения ответа; последующая безопасная ошибка сериализации не
//! откатывает эту мутацию. Короткие числа становятся нулями, короткий GUID —
//! `GUID_INVALID`.
//!
//! Исходный `UnSerialize` не имел длины. Безопасная граница сохраняет уже
//! выполненные счётчики, очистки, присваивания и cursor, но отклоняет неполный
//! goods до передачи комнате; недоопределённое чтение за буфер не имитируется.

use std::collections::BTreeMap;

use crate::miscserver::miscserver::game::{CGame, legacy_tick_ms};
use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};
use crate::public::aucitionroom::{
    AddAuctionItemMissingGoodsType, AuctionPageBuildError, UnityGoodsBuild,
};
use crate::public::auctionnode::{CGoodsNode, GoodsNodeSerializeError, GoodsNodeUnserializeError};
use crate::public::auctionroom::PlayerOptNode;
use crate::public::guid::CGuid;

const ADD_AUCTION_ITEM: i32 = 0x0014_ED01;
const QUEUE_AUCTION_OPERATION: i32 = 0x0014_ED04;
const UNITY_AUCTION_GOODS: i32 = 0x0014_ED05;
const QUERY_SELF_AUCTION: i32 = 0x0014_ED06;
const QUERY_AUCTION_PAGE: i32 = 0x0014_ED07;
const QUEUE_OWNER_AUCTION_OPERATION: i32 = 0x0014_ED08;
const MODIFY_AUCTION_SEARCH: i32 = 0x0014_ED09;
const AUCTION_OPERATION_RESPONSE: i32 = 0x0015_EB06;
const SELF_AUCTION_RESPONSE: i32 = 0x0015_EB07;
const AUCTION_PAGE_RESPONSE: i32 = 0x0015_EB08;
const WORLD_REQUEST_OPERATION: u32 = 3;
const OWNER_REQUEST_OPERATION: u32 = 1;
const SYNC_WAIT_MILLISECONDS: u32 = 120_000;
const UNITY_COUNT_WARNING_THRESHOLD: u32 = 10_000;
const SEARCH_GOODS_NAME_BYTES: usize = 0x100;

#[derive(Debug)]
pub(crate) enum WorldAuctionOutcome {
    Unhandled,
    ItemAdded,
    ItemRejected,
    UnserializeRejected(GoodsNodeUnserializeError),
    MissingGoodsType(AddAuctionItemMissingGoodsType),
    OperationWithoutResponse,
    OperationResponse { send: Result<i32, SendMessageError> },
    UnityAlreadyProcessed,
    UnityWaiting {
        count_warning: bool,
        enabled_now: bool,
    },
    UnitySerializeBlocked {
        count_warning: bool,
        sends: Vec<Result<i32, SendMessageError>>,
        error: GoodsNodeSerializeError,
    },
    UnityCompleted {
        count_warning: bool,
        sends: Vec<Result<i32, SendMessageError>>,
    },
    SelfAuctionSerializeBlocked(GoodsNodeSerializeError),
    SelfAuctionResponse { send: Result<i32, SendMessageError> },
    AuctionPageBuildBlocked(AuctionPageBuildError),
    AuctionPageResponse { send: Result<i32, SendMessageError> },
    OwnerOperationIgnored,
    OwnerOperationRequested,
    SearchConditionModified,
}

pub(crate) fn on_msg_w2m_auction(message: &mut CMessage, game: &mut CGame) -> WorldAuctionOutcome {
    if message.message_type() == QUEUE_AUCTION_OPERATION {
        let player_id = message.base_mut().get_long().unwrap_or(0);
        let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
        let pushed =
            game.auction_room_mut()
                .push_item_to_opt_list(guid, WORLD_REQUEST_OPERATION, player_id);
        if pushed {
            return WorldAuctionOutcome::OperationWithoutResponse;
        }

        let mut response = CMessage::new(AUCTION_OPERATION_RESPONSE);
        response.base_mut().add_long(0);
        response.base_mut().add_long(player_id);
        let sender = game.net_client().map(|client| client as &dyn MessageSender);
        return WorldAuctionOutcome::OperationResponse {
            send: response.send(sender, false),
        };
    }

    if message.message_type() == UNITY_AUCTION_GOODS {
        return on_unity_auction_goods(message, game);
    }

    if message.message_type() == QUERY_SELF_AUCTION {
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let mut response = CMessage::new(SELF_AUCTION_RESPONSE);
        response.base_mut().add_ulong(player_id);
        if let Err(error) = game
            .auction_room()
            .add_byte_auction_self_to_client(&mut response, player_id)
        {
            return WorldAuctionOutcome::SelfAuctionSerializeBlocked(error);
        }
        let sender = game.net_client().map(|client| client as &dyn MessageSender);
        return WorldAuctionOutcome::SelfAuctionResponse {
            send: response.send(sender, false),
        };
    }

    if message.message_type() == QUERY_AUCTION_PAGE {
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let operation = message.base_mut().get_long().unwrap_or(0) as u32;
        if let Err(error) = game
            .auction_room_mut()
            .compute_player_page(player_id, operation)
        {
            return WorldAuctionOutcome::AuctionPageBuildBlocked(error);
        }

        let mut response = CMessage::new(AUCTION_PAGE_RESPONSE);
        response.base_mut().add_ulong(player_id);
        if let Err(error) = game
            .auction_room()
            .add_byte_at_page_by_time(player_id, &mut response)
        {
            return WorldAuctionOutcome::AuctionPageBuildBlocked(error);
        }
        let sender = game.net_client().map(|client| client as &dyn MessageSender);
        return WorldAuctionOutcome::AuctionPageResponse {
            send: response.send(sender, false),
        };
    }

    if message.message_type() == QUEUE_OWNER_AUCTION_OPERATION {
        let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
        if guid == CGuid::GUID_INVALID {
            return WorldAuctionOutcome::OwnerOperationIgnored;
        }

        let _ = game
            .auction_room_mut()
            .push_item_to_opt_list(guid, OWNER_REQUEST_OPERATION, 0);
        return WorldAuctionOutcome::OwnerOperationRequested;
    }

    if message.message_type() == MODIFY_AUCTION_SEARCH {
        let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
        let low_level = message.base_mut().get_long().unwrap_or(0);
        let up_level = message.base_mut().get_long().unwrap_or(0);
        let use_self = message.base_mut().get_long().unwrap_or(0);
        let money_type = message.base_mut().get_long().unwrap_or(0);
        let weapon_type = message.base_mut().get_long().unwrap_or(0);
        let name_bytes = message
            .base_mut()
            .get_str_bytes(SEARCH_GOODS_NAME_BYTES)
            .unwrap_or_default();
        let mut goods_name = [0; SEARCH_GOODS_NAME_BYTES];
        goods_name[..name_bytes.len()].copy_from_slice(&name_bytes);

        let condition = PlayerOptNode::from_search_request(
            player_id,
            low_level,
            up_level,
            use_self,
            money_type,
            weapon_type,
            goods_name,
        );
        game.auction_room_mut()
            .modify_player_seach_condition(condition);
        return WorldAuctionOutcome::SearchConditionModified;
    }

    if message.message_type() != ADD_AUCTION_ITEM {
        return WorldAuctionOutcome::Unhandled;
    }

    game.count_new_auction_item();
    let mut item = Box::new(CGoodsNode::new());
    let unserialize = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        item.unserialize(source, cursor)
    };
    if let Err(error) = unserialize {
        // Неполный узел не передаётся комнате вместо исходного чтения за буфером.
        return WorldAuctionOutcome::UnserializeRejected(error);
    }

    match game.add_auction_item(item) {
        Ok(true) => WorldAuctionOutcome::ItemAdded,
        Ok(false) => WorldAuctionOutcome::ItemRejected,
        Err(error) => WorldAuctionOutcome::MissingGoodsType(error),
    }
}

fn on_unity_auction_goods(message: &mut CMessage, game: &mut CGame) -> WorldAuctionOutcome {
    if game.auction_sync_count() != 0 {
        return WorldAuctionOutcome::UnityAlreadyProcessed;
    }

    let map_id = message.base_mut().get_byte().unwrap_or(0);
    let declared_count = message.base_mut().get_long().unwrap_or(0) as u32;
    let count_warning = declared_count > UNITY_COUNT_WARNING_THRESHOLD;
    let mut existing = BTreeMap::new();
    for _ in 0..declared_count {
        let guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);
        existing.insert(guid, false);
    }

    if !game.auction_sync_enabled() {
        let enabled_now = game
            .auction_sync_start_time()
            .wrapping_add(SYNC_WAIT_MILLISECONDS)
            < legacy_tick_ms();
        if enabled_now {
            game.enable_auction_sync();
        }
        return WorldAuctionOutcome::UnityWaiting {
            count_warning,
            enabled_now,
        };
    }

    let UnityGoodsBuild { messages, blocked } = game.auction_room().unity_goods(&existing, map_id);
    let sender = game.net_client().map(|client| client as &dyn MessageSender);
    let sends = messages
        .into_iter()
        .map(|message| message.send(sender, false))
        .collect();
    if let Some(error) = blocked {
        return WorldAuctionOutcome::UnitySerializeBlocked {
            count_warning,
            sends,
            error,
        };
    }

    game.finish_auction_sync_turn();
    WorldAuctionOutcome::UnityCompleted {
        count_warning,
        sends,
    }
}
