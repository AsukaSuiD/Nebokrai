//! Входной owner UniBill GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/unibillmessage.cpp`. Материализован связный Increment
//! Shop response `0xFF002`: Billing balance, currency-container effect,
//! повторная factory batch, packet new/stack updates, скидочное списание,
//! `GS0082` и optional World audit `0x6020D`. Auction subset `0xFF003/type 3`
//! завершает pending `CGoodsNode`: логирует сделку, посылает success/self
//! refresh, синхронизирует обе YuanBao wallet или `0x60814` offline seller.
//! Остальные UniBill cases остаются ниже RAW до их session owners.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::player::{
    CiQingPacketAddition, CiQingPacketConsumption, PlayerYuanBaoChange,
};
use crate::gameserver::gameserver::game::{
    CGame, OldClientGoodsCodec, colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const INCREMENT_PURCHASE_RESPONSE: i32 = 0x000F_F002;
const BILLING_TRADE_RESPONSE: i32 = 0x000F_F003;
const INCREMENT_PURCHASE_AUDIT: i32 = 0x0006_020D;

pub(crate) trait IncrementShopBillingContext: OldClientGoodsCodec {
    fn publish_increment_shop_yuan_bao_change(&mut self, change: &PlayerYuanBaoChange) -> Vec<i32>;
    fn auction_billing_local_system_time(
        &mut self,
    ) -> crate::public::auctionlog::AuctionLogSystemTime;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopBillingMessageError {
    MissingField(&'static str),
    AuctionNodeSerialize(crate::public::auctionnode::GoodsNodeSerializeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopBillingOutcome {
    MissingPlayer,
    BillingRejected { result: i32 },
    EmptyFactoryBatch,
    PacketRejected { notice_delivery: i32 },
    AuctionTradeCompleted { seller_id: i32, seller_online: bool },
    AuctionTradeRejected { seller_id: i32, result: i32 },
    Completed,
}

#[must_use = "UniBill report сохраняет balance, inventory и audit effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IncrementShopBillingReport {
    pub(crate) player_id: i32,
    pub(crate) last_point: Option<u32>,
    pub(crate) charged: Option<u32>,
    pub(crate) goods_id: Option<u32>,
    pub(crate) goods_amount: Option<u32>,
    pub(crate) deduction_goods_id: Option<u32>,
    pub(crate) transaction: Vec<u8>,
    pub(crate) yuan_bao_change: Option<PlayerYuanBaoChange>,
    pub(crate) auction_seller_change: Option<PlayerYuanBaoChange>,
    pub(crate) yuan_bao_deliveries: Vec<i32>,
    pub(crate) additions: Vec<CiQingPacketAddition>,
    pub(crate) addition_deliveries: Vec<Vec<i32>>,
    pub(crate) consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) consumption_deliveries: Vec<Vec<i32>>,
    pub(crate) audit_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) auction_world_deliveries: Vec<Result<i32, SendMessageError>>,
    pub(crate) outcome: IncrementShopBillingOutcome,
}

pub(crate) fn dispatch_increment_shop_billing_message<Context: IncrementShopBillingContext>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<IncrementShopBillingReport, IncrementShopBillingMessageError>> {
    if message.message_type() == BILLING_TRADE_RESPONSE {
        let unread = message.unread_bytes();
        if unread.len() < 24 {
            return Some(Err(IncrementShopBillingMessageError::MissingField(
                "auction billing prefix",
            )));
        }
        let trade_type = i32::from_le_bytes(
            unread[20..24]
                .try_into()
                .expect("проверенный Billing trade prefix"),
        );
        if trade_type != 3 {
            return None;
        }
        return Some(dispatch_auction_billing_trade(message, game, context));
    }
    if message.message_type() != INCREMENT_PURCHASE_RESPONSE {
        return None;
    }
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(IncrementShopBillingMessageError::MissingField(field))
    };
    let player_id = match read_long(message, "player id") {
        Ok(value) => value,
        Err(error) => return Some(Err(error)),
    };
    let mut report = IncrementShopBillingReport {
        player_id,
        last_point: None,
        charged: None,
        goods_id: None,
        goods_amount: None,
        deduction_goods_id: None,
        transaction: Vec::new(),
        yuan_bao_change: None,
        auction_seller_change: None,
        yuan_bao_deliveries: Vec::new(),
        additions: Vec::new(),
        addition_deliveries: Vec::new(),
        consumptions: Vec::new(),
        consumption_deliveries: Vec::new(),
        audit_delivery: None,
        auction_world_deliveries: Vec::new(),
        outcome: IncrementShopBillingOutcome::MissingPlayer,
    };
    if game.find_player(player_id).is_none() {
        return Some(Ok(report));
    }
    let result = match read_long(message, "result") {
        Ok(value) => value,
        Err(error) => return Some(Err(error)),
    };
    if result != 0 {
        report.outcome = IncrementShopBillingOutcome::BillingRejected { result };
        return Some(Ok(report));
    }
    let last_point = match read_long(message, "last point") {
        Ok(value) => value as u32,
        Err(error) => return Some(Err(error)),
    };
    let charged = match read_long(message, "charged yuanbao") {
        Ok(value) => value as u32,
        Err(error) => return Some(Err(error)),
    };
    let goods_id = match read_long(message, "goods id") {
        Ok(value) => value as u32,
        Err(error) => return Some(Err(error)),
    };
    let goods_amount = match read_long(message, "goods amount") {
        Ok(value) => value as u32,
        Err(error) => return Some(Err(error)),
    };
    let deduction_goods_id = match read_long(message, "deduction goods id") {
        Ok(value) => value as u32,
        Err(error) => return Some(Err(error)),
    };
    let transaction = match message.base_mut().get_str_bytes(0x200) {
        Some(value) => value,
        None => {
            return Some(Err(IncrementShopBillingMessageError::MissingField(
                "transaction",
            )));
        }
    };
    report.last_point = Some(last_point);
    report.charged = Some(charged);
    report.goods_id = Some(goods_id);
    report.goods_amount = Some(goods_amount);
    report.deduction_goods_id = Some(deduction_goods_id);
    report.transaction = transaction.clone();

    let current_yuan_bao = game
        .find_player(player_id)
        .expect("UniBill player проверен перед balance mutation")
        .yuan_bao();
    let currency_created = if current_yuan_bao < last_point {
        game.create_goods_batch(
            game.goods_factory().get_yuan_bao_index(),
            last_point.wrapping_sub(current_yuan_bao),
        )
    } else {
        Vec::new()
    };
    let yuan_bao_change = game
        .set_player_yuan_bao(player_id, last_point, currency_created)
        .expect("UniBill player проверен перед balance mutation");
    report.yuan_bao_deliveries = context.publish_increment_shop_yuan_bao_change(&yuan_bao_change);
    report.yuan_bao_change = Some(yuan_bao_change);

    let created = game.create_goods_batch(goods_id, goods_amount);
    if created.is_empty() {
        report.outcome = IncrementShopBillingOutcome::EmptyFactoryBatch;
        return Some(Ok(report));
    }
    if !super::incrementshopmessage::increment_batch_fits_packet(game, player_id, &created) {
        let notice_delivery = send_no_packet_space(game, player_id);
        report.outcome = IncrementShopBillingOutcome::PacketRejected { notice_delivery };
        return Some(Ok(report));
    }
    let goods_name = created
        .last()
        .map(|goods| goods.name().to_vec())
        .unwrap_or_default();
    let mut encode = |goods: &CGoods| context.encode_goods_for_old_client(goods);
    let (additions, rejected) = game
        .add_increment_shop_goods_to_packet(player_id, created, &mut encode)
        .expect("UniBill player проверен перед packet add");
    let additions_succeeded = rejected.is_empty()
        && additions
            .iter()
            .all(|addition| addition.resulting_amount.is_some());
    for addition in additions {
        if addition.resulting_amount.is_some() {
            report
                .addition_deliveries
                .push(game.send_player_packet_addition(&addition));
        }
        report.additions.push(addition);
    }
    if !additions_succeeded {
        let notice_delivery = send_no_packet_space(game, player_id);
        report.outcome = IncrementShopBillingOutcome::PacketRejected { notice_delivery };
        return Some(Ok(report));
    }

    if deduction_goods_id != 0 {
        let consumptions = game
            .find_player_mut(player_id)
            .expect("UniBill player проверен перед deduction removal")
            .remove_item_in_packet(deduction_goods_id, goods_amount);
        for consumption in consumptions {
            report
                .consumption_deliveries
                .push(game.send_player_packet_consumption(&consumption));
            report.consumptions.push(consumption);
        }
    }
    if game.log_system().increment_log_enabled() {
        let mut audit = CMessage::new(INCREMENT_PURCHASE_AUDIT);
        audit.add_byte(0);
        add_c_string(&mut audit, &transaction);
        audit.add_ulong(charged);
        add_c_string(&mut audit, game.get_string_by_id(b"GS0103"));
        audit.add_long(player_id);
        add_c_string(&mut audit, &goods_name);
        audit.add_ulong(goods_amount);
        let client_ip = game
            .find_player(player_id)
            .map_or(0, |player| player.client_ip());
        audit.add_ulong(client_ip);
        report.audit_delivery = Some(audit.send(game, false));
    }
    report.outcome = IncrementShopBillingOutcome::Completed;
    Some(Ok(report))
}

fn dispatch_auction_billing_trade<Context: IncrementShopBillingContext>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Result<IncrementShopBillingReport, IncrementShopBillingMessageError> {
    let buyer_id = read_billing_long(message, "auction buyer id")?;
    let seller_id = read_billing_long(message, "auction seller id")?;
    let result = read_billing_long(message, "auction trade result")?;
    let buyer_yuan_bao = read_billing_long(message, "auction buyer yuan bao")? as u32;
    let seller_yuan_bao = read_billing_long(message, "auction seller yuan bao")? as u32;
    let trade_type = read_billing_long(message, "auction trade type")?;
    debug_assert_eq!(trade_type, 3);
    let mut report = IncrementShopBillingReport {
        player_id: buyer_id,
        last_point: Some(buyer_yuan_bao),
        charged: None,
        goods_id: None,
        goods_amount: None,
        deduction_goods_id: None,
        transaction: Vec::new(),
        yuan_bao_change: None,
        auction_seller_change: None,
        yuan_bao_deliveries: Vec::new(),
        additions: Vec::new(),
        addition_deliveries: Vec::new(),
        consumptions: Vec::new(),
        consumption_deliveries: Vec::new(),
        audit_delivery: None,
        auction_world_deliveries: Vec::new(),
        outcome: IncrementShopBillingOutcome::MissingPlayer,
    };
    if game.find_player(buyer_id).is_none() {
        return Ok(report);
    }
    if result != 0 {
        report.outcome = IncrementShopBillingOutcome::AuctionTradeRejected { seller_id, result };
        return Ok(report);
    }

    if let Some(node) = game
        .find_player_mut(buyer_id)
        .expect("Billing auction buyer проверен")
        .take_current_auction_buy_node()
    {
        let buyer_time = context.auction_billing_local_system_time();
        let (buyer_log, mut seller_log, notice) =
            super::onmsg_w2s_auction::build_auction_buy_log_effects(&node, game, buyer_time);
        let mut buyer_audit = CMessage::new(0x0006_0214);
        buyer_audit.base_mut().add(&buyer_log.to_legacy_bytes());
        report
            .auction_world_deliveries
            .push(buyer_audit.send(game, false));
        report.yuan_bao_deliveries.push(
            colored_player_notice_message(0xffff_ffff, 0xffff_0000, &notice)
                .send_to_player(game.net_server(), buyer_id),
        );
        seller_log.time = context.auction_billing_local_system_time();
        let mut seller_audit = CMessage::new(0x0006_0214);
        seller_audit.base_mut().add(&seller_log.to_legacy_bytes());
        report
            .auction_world_deliveries
            .push(seller_audit.send(game, false));
        report.auction_world_deliveries.push(
            super::onmsg_w2s_auction::send_auction_node_world(&node, 0x0006_0806, game)
                .map_err(IncrementShopBillingMessageError::AuctionNodeSerialize)?,
        );
        for query_player_id in [buyer_id, node.seller_id() as i32] {
            if query_player_id == buyer_id || game.find_player(query_player_id).is_some() {
                let mut query = CMessage::new(0x0006_0802);
                query.base_mut().add_long(query_player_id);
                report
                    .auction_world_deliveries
                    .push(query.send(game, false));
            }
        }
    }

    let buyer_change = set_auction_yuan_bao(game, buyer_id, buyer_yuan_bao)
        .expect("Billing auction buyer проверен перед balance");
    report
        .yuan_bao_deliveries
        .extend(context.publish_increment_shop_yuan_bao_change(&buyer_change));
    report.yuan_bao_change = Some(buyer_change);

    let seller_online = game.find_player(seller_id).is_some();
    if seller_online {
        let seller_change = set_auction_yuan_bao(game, seller_id, seller_yuan_bao)
            .expect("online auction seller проверен перед mutation");
        report
            .yuan_bao_deliveries
            .extend(context.publish_increment_shop_yuan_bao_change(&seller_change));
        report.auction_seller_change = Some(seller_change);
    } else {
        let mut offline = CMessage::new(0x0006_0814);
        offline.base_mut().add_long(seller_id);
        offline.base_mut().add_ulong(seller_yuan_bao);
        report
            .auction_world_deliveries
            .push(offline.send(game, false));
    }
    report.outcome = IncrementShopBillingOutcome::AuctionTradeCompleted {
        seller_id,
        seller_online,
    };
    Ok(report)
}

fn read_billing_long(
    message: &mut CMessage,
    field: &'static str,
) -> Result<i32, IncrementShopBillingMessageError> {
    message
        .base_mut()
        .get_long()
        .ok_or(IncrementShopBillingMessageError::MissingField(field))
}

fn set_auction_yuan_bao(
    game: &mut CGame,
    player_id: i32,
    current: u32,
) -> Option<PlayerYuanBaoChange> {
    let previous = game.find_player(player_id)?.yuan_bao();
    let created = if previous < current {
        game.create_goods_batch(
            game.goods_factory().get_yuan_bao_index(),
            current.wrapping_sub(previous),
        )
    } else {
        Vec::new()
    };
    game.set_player_yuan_bao(player_id, current, created)
}

fn send_no_packet_space(game: &CGame, player_id: i32) -> i32 {
    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0082"))
        .send_to_player(game.net_server(), player_id)
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let prefix = &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())];
    message.base_mut().add(prefix);
    message.base_mut().add_byte(0);
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\unibillmessage.cpp
// ============================================================================
// FUNCTION: OnUniBillMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\unibillmessage.cpp:166
// RVA: 0x00087050
// ADDRESS: 00487050
// PROTOTYPE: void __cdecl OnUniBillMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c4ac6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\unibillmessage.cpp
// RVA: 0x001C4AC6
// ADDRESS: 005c4ac6
// PROTOTYPE: undefined Catch@005c4ac6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c4b66
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\unibillmessage.cpp
// RVA: 0x001C4B66
// ADDRESS: 005c4b66
// PROTOTYPE: undefined Catch@005c4b66()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
