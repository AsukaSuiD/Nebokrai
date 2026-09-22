//! Billing-обработчики `appbilling/billingmessage.cpp`, подтверждённые
//! `billingserver.exe` и `billingserver.pdb`. Они декодируют запрос баланса,
//! покупку и обмен в общие FIFO `CBillingPlayerManager`.
//!
//! Пустая identity в запросе баланса остаётся no-op и не потребляет следующий
//! `long`. Числа сохраняют общий fallback `GetLong == 0`, неполный GUID —
//! предварительный `GUID_INVALID`; GameServer ID берётся из metadata сообщения.
//! Owned-записи заменяют глубокие C++-копии, сохраняя порядок постановки в FIFO.

use crate::billingserver::appbilling::billingplayermanager::{
    CBillingPlayerManager, TagAccInfo, TagTradeNode, TagTradeNodeParts,
};
use crate::nets::netbilling::message::CMessage;
use nebokrai_shared::values::CGuid;

const ACCOUNT_BALANCE_REQUEST: i32 = 0x000E_F201;
const INCREMENT_PURCHASE_REQUEST: i32 = 0x000E_F202;
const PLAYER_TRADE_REQUEST: i32 = 0x000E_F203;
const BILLING_STRING_LIMIT: usize = 0x20;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BillingMessageOutcome {
    AccountRequestIgnoredEmpty,
    AccountRequestQueued { request: TagAccInfo, queued: bool },
    IncrementPurchaseQueued { request: TagTradeNode, queued: bool },
    PlayerTradeQueued { request: TagTradeNode, queued: bool },
    Unsupported { message_type: i32 },
}

pub(crate) struct BillingMessageHandler<'a> {
    queues: &'a CBillingPlayerManager,
}

impl<'a> BillingMessageHandler<'a> {
    pub(crate) const fn new(queues: &'a CBillingPlayerManager) -> Self {
        Self { queues }
    }

    pub(crate) fn on_billing_message(&self, message: &mut CMessage) -> BillingMessageOutcome {
        match message.message_type() {
            ACCOUNT_BALANCE_REQUEST => self.on_account_request(message),
            INCREMENT_PURCHASE_REQUEST => self.on_increment_purchase(message),
            PLAYER_TRADE_REQUEST => self.on_player_trade(message),
            message_type => BillingMessageOutcome::Unsupported { message_type },
        }
    }

    fn on_account_request(&self, message: &mut CMessage) -> BillingMessageOutcome {
        let player_identity = read_string(message);
        if player_identity.is_empty() {
            return BillingMessageOutcome::AccountRequestIgnoredEmpty;
        }

        let player_id = read_long(message);
        let request = TagAccInfo::new(player_id, player_identity, message.socket_id());
        let queued = self.queues.push_account_request(request.clone());
        BillingMessageOutcome::AccountRequestQueued { request, queued }
    }

    fn on_increment_purchase(&self, message: &mut CMessage) -> BillingMessageOutcome {
        let buyer_id = read_long(message);
        let buyer_identity = read_string(message);
        let buyer_ip = read_string(message);
        let buyer_name = read_string(message);
        let yuanbao = read_long(message) as u32;
        let goods_id = read_long(message);
        let goods_number = read_long(message);
        let session_id = read_long(message);
        let login_server_id = read_long(message);
        let world_server_id = read_long(message);

        let request = TagTradeNode::from_parts(TagTradeNodeParts {
            trade_type: 0,
            buyer_id,
            seller_id: 0,
            buyer_identity,
            seller_identity: Vec::new(),
            buyer_ip,
            seller_ip: Vec::new(),
            buyer_name,
            seller_name: Vec::new(),
            yuanbao,
            goods_id,
            goods_number,
            game_server_id: message.socket_id(),
            session_id,
            plugin_id: 0,
            login_server_id,
            world_server_id,
            goods_guid: CGuid::GUID_INVALID,
        });
        let queued = self.queues.push_trade_request(request.clone());
        BillingMessageOutcome::IncrementPurchaseQueued { request, queued }
    }

    fn on_player_trade(&self, message: &mut CMessage) -> BillingMessageOutcome {
        let trade_type = read_long(message);
        let buyer_id = read_long(message);
        let seller_id = read_long(message);
        let buyer_identity = read_string(message);
        let seller_identity = read_string(message);
        let buyer_ip = read_string(message);
        let seller_ip = read_string(message);
        let buyer_name = read_string(message);
        let seller_name = read_string(message);
        let yuanbao = read_long(message) as u32;
        let goods_id = read_long(message);
        let goods_number = read_long(message);
        let session_id = read_long(message);
        let plugin_id = read_long(message);
        let login_server_id = read_long(message);
        let world_server_id = read_long(message);
        let goods_guid = message.base_mut().get_guid().unwrap_or(CGuid::GUID_INVALID);

        let request = TagTradeNode::from_parts(TagTradeNodeParts {
            trade_type,
            buyer_id,
            seller_id,
            buyer_identity,
            seller_identity,
            buyer_ip,
            seller_ip,
            buyer_name,
            seller_name,
            yuanbao,
            goods_id,
            goods_number,
            game_server_id: message.socket_id(),
            session_id,
            plugin_id,
            login_server_id,
            world_server_id,
            goods_guid,
        });
        let queued = self.queues.push_trade_request(request.clone());
        BillingMessageOutcome::PlayerTradeQueued { request, queued }
    }
}

fn read_long(message: &mut CMessage) -> i32 {
    message.base_mut().get_long().unwrap_or(0)
}

fn read_string(message: &mut CMessage) -> Vec<u8> {
    message
        .base_mut()
        .get_str_bytes(BILLING_STRING_LIMIT)
        .expect("ненулевая GetStr-граница задана константой")
}
