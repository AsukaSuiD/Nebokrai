//! Три входные Billing-ветви из `appbilling/billingmessage.cpp`.
//!
//!
//! `0xEF201` читает строковую player identity с границей `0x20` и только при
//! её непустом первом byte читает numeric player ID. Запись
//! `tagAccInfo(player_id, identity, message.socket_id)` затем безусловно
//! добавляется в AC FIFO. Пустая identity остаётся тихим no-op и не потребляет
//! следующий `long`.
//!
//! `0xEF202` сначала читает buyer ID, три строки buyer identity/IP/name, затем
//! yuanbao, goods ID/count, session ID, LoginServer ID и WorldServer ID. Он
//! создаёт `tagTradeNode` типа `0` с нулевыми seller/plugin полями, пустыми
//! seller-строками, `GUID_INVALID` и GameServer ID из message metadata. Это
//! purchase-запись: пустой seller ID позднее выбирает `BuyItemCode` в
//! `CBillingPlayerManager::Run`.
//!
//! `0xEF203` читает type/buyer/seller, шесть строк в порядке buyer identity,
//! seller identity, buyer IP, seller IP, buyer name, seller name, затем
//! yuanbao, goods ID/count, session/plugin/Login/World ID и GUID. GameServer ID
//! снова берётся из socket metadata. Обе ветви глубоко копируют готовую запись
//! в общую TR FIFO до operator log; owned Rust-запись сохраняет тот же порядок.
//!
//! Все numeric поля используют старое `GetLong == 0` при нехватке payload;
//! каждая строка использует уже восстановленный ограниченный `GetStr`. Нулевой
//! либо неполный GUID остаётся предварительно созданным `GUID_INVALID`, как у
//! исходного `CGUID local; GetGUID(...)`. Три старых `sprintf/AddLogText`
//! представлены typed outcome с полной записью: `CGame::ProcessMessage`
//! сохраняет их без доступа к очереди и без глобального `char[]`.
//!
//! Временные `std::string`, локальные record-копии, SEH и единственный `$L`
//! cleanup удалены как library/compiler noise; их lifetime выражен `Vec`,
//! `Clone` и `Drop`. Обработчик только ставит записи в общие FIFO; запуск и
//! исполнение DB workers остаются у восстановленного manager lifecycle.

use crate::billingserver::appbilling::billingplayermanager::{
    CBillingPlayerManager, TagAccInfo, TagTradeNode, TagTradeNodeParts,
};
use crate::nets::netbilling::message::CMessage;
use crate::public::guid::CGuid;

const ACCOUNT_BALANCE_REQUEST: i32 = 0x000E_F201;
const INCREMENT_PURCHASE_REQUEST: i32 = 0x000E_F202;
const PLAYER_TRADE_REQUEST: i32 = 0x000E_F203;
const BILLING_STRING_LIMIT: usize = 0x20;

/// Наблюдаемый результат одной ветви Billing `OnBillingMessage`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum BillingMessageOutcome {
    /// Пустая identity в `0xEF201` остановила ветвь до numeric ID.
    AccountRequestIgnoredEmpty,
    /// Account-запрос скопирован в AC FIFO.
    AccountRequestQueued { request: TagAccInfo, queued: bool },
    /// Обычная покупка скопирована в TR FIFO.
    IncrementPurchaseQueued { request: TagTradeNode, queued: bool },
    /// Player-to-player trade скопирован в TR FIFO.
    PlayerTradeQueued { request: TagTradeNode, queued: bool },
    /// Неизвестный тип сохраняет исходный no-op.
    Unsupported { message_type: i32 },
}

/// Узкая композиция свободного handler с общими FIFO Billing workers.
pub(crate) struct BillingMessageHandler<'a> {
    queues: &'a CBillingPlayerManager,
}

impl<'a> BillingMessageHandler<'a> {
    /// Связывает handler с единственным общим владельцем трёх static FIFO.
    pub(crate) const fn new(queues: &'a CBillingPlayerManager) -> Self {
        Self { queues }
    }

    /// Выполняет три подтверждённые ветви, сохраняя неизвестный opcode как no-op.
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
