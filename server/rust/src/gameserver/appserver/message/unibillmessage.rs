//! Входной владелец UniBill GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/message/unibillmessage.cpp`. Материализован связный Increment
//! Shop response `0xFF002`: баланс Billing, эффект валютного контейнера,
//! повторная партия фабрики, обновления новых/сложенных товаров packet,
//! скидочное списание, `GS0082` и необязательный аудит World `0x6020D`.
//! Подмножество обмена игроков
//! `0xFF003/type 2` синхронизирует обе YuanBao wallet, возобновляет
//! отложенное подтверждение trader и завершает сессию. Personal-shop `type 1`
//! синхронизирует оба кошелька и продолжает отложенный перенос товара через
//! plugs продавца/покупателя. Подмножество аукциона `type 3` завершает
//! ожидающий `CGoodsNode`: журналирует сделку, посылает подтверждение и
//! обновление, синхронизирует оба кошелька YuanBao либо `0x60814` офлайн-
//! продавцу. Auth/refresh `0xFF001/0xFF004` выполняют поиск player/account,
//! точное изменение баланса и журнал отказа; журнал Win32 GUI заменён stderr,
//! а совместимый файловый приёмник `Bill` сохранён через `put_string_to_file`.
//! Все достигнутые изменения баланса YuanBao сразу публикуют конкретные
//! `C0101/C0102` для extend `5` через `CGame`; контекст предоставляет только
//! обязательный codec старого клиента для создания валютных товаров. Порядок
//! эффектов сохраняется внутри owner-а, а диагностическая история публикуется
//! через `tracing` вместо возвращаемого дерева.
//! Общий отказ `0xFF003` содержит только buyer/seller/result (12 байтов),
//! как формирует `CBillingPlayerManager::run`. Балансы и trade_type читаются
//! только при result == 0; короткий отказ не меняет кошельки и сеансы.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use nebokrai_shared::protocol::LegacyReader;
use crate::gameserver::appserver::player::PlayerYuanBaoChange;
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;
use crate::public::auctionlog::AuctionLogSystemTime;
use crate::public::date::TagTime;
use crate::public::tools::put_string_to_file;
use tracing::{debug, trace, warn};

const INCREMENT_PURCHASE_RESPONSE: i32 = 0x000F_F002;
const BILLING_AUTH_RESPONSE: i32 = 0x000F_F001;
const BILLING_TRADE_RESPONSE: i32 = 0x000F_F003;
const BILLING_REFRESH_RESPONSE: i32 = 0x000F_F004;
const INCREMENT_PURCHASE_AUDIT: i32 = 0x0006_020D;

pub(crate) fn auction_billing_local_system_time() -> AuctionLogSystemTime {
    let [
        year,
        month,
        day_of_week,
        day,
        hour,
        minute,
        second,
        milliseconds,
    ] = TagTime::local_now().fields();
    AuctionLogSystemTime {
        year,
        month,
        day_of_week,
        day,
        hour,
        minute,
        second,
        milliseconds,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementShopBillingMessageError {
    MissingField(&'static str),
    AuctionNodeSerialize(crate::public::auctionnode::GoodsNodeSerializeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BillingTradeResponsePrefix {
    Rejected {
        buyer_id: i32,
        seller_id: i32,
        result: i32,
    },
    Success {
        trade_type: i32,
    },
}

// Предварительное чтение не сдвигает курсор CMessage: успешный ответ целиком
// разбирает прежний обработчик, сохраняя порядок его прикладных эффектов.
fn read_billing_trade_response_prefix(
    source: &[u8],
) -> Result<BillingTradeResponsePrefix, IncrementShopBillingMessageError> {
    let mut reader = LegacyReader::new(source);
    let mut read_long = |field| {
        reader
            .read_i32()
            .map_err(|_| IncrementShopBillingMessageError::MissingField(field))
    };
    let buyer_id = read_long("billing trade buyer id")?;
    let seller_id = read_long("billing trade seller id")?;
    let result = read_long("billing trade result")?;
    if result != 0 {
        return Ok(BillingTradeResponsePrefix::Rejected {
            buyer_id,
            seller_id,
            result,
        });
    }
    let _buyer_yuan_bao = read_long("billing trade buyer yuan bao")?;
    let _seller_yuan_bao = read_long("billing trade seller yuan bao")?;
    let trade_type = read_long("billing trade type")?;
    Ok(BillingTradeResponsePrefix::Success { trade_type })
}

pub(crate) fn dispatch_increment_shop_billing_message<Context: GameContainerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<(), IncrementShopBillingMessageError>> {
    if message.message_type() == BILLING_AUTH_RESPONSE {
        return Some(dispatch_billing_auth(message, game));
    }
    if message.message_type() == BILLING_REFRESH_RESPONSE {
        return Some(dispatch_billing_refresh(message, game));
    }
    if message.message_type() == BILLING_TRADE_RESPONSE {
        let trade_type = match read_billing_trade_response_prefix(message.unread_bytes()) {
            Ok(BillingTradeResponsePrefix::Rejected {
                buyer_id,
                seller_id,
                result,
            }) => {
                warn!(buyer_id, seller_id, result, "Billing отклонил сделку");
                return Some(Ok(()));
            }
            Ok(BillingTradeResponsePrefix::Success { trade_type }) => trade_type,
            Err(error) => return Some(Err(error)),
        };
        if trade_type == 2 {
            return Some(dispatch_player_billing_trade(message, game, context));
        }
        if trade_type == 1 {
            return Some(dispatch_personal_shop_billing_trade(message, game, context));
        }
        if trade_type == 3 {
            return Some(dispatch_auction_billing_trade(message, game));
        }
        return None;
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
    if game.find_player(player_id).is_none() {
        trace!(player_id, "ответ Billing для Increment Shop пропущен: игрок не найден");
        return Some(Ok(()));
    }
    let result = match read_long(message, "result") {
        Ok(value) => value,
        Err(error) => return Some(Err(error)),
    };
    if result != 0 {
        warn!(player_id, result, "Billing отклонил покупку Increment Shop");
        return Some(Ok(()));
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
    let _ = game.send_player_yuan_bao_change(&yuan_bao_change);

    let created = game.create_goods_batch(goods_id, goods_amount);
    if created.is_empty() {
        trace!(player_id, goods_id, goods_amount, "фабрика не создала оплаченную партию Increment Shop");
        return Some(Ok(()));
    }
    if !super::incrementshopmessage::increment_batch_fits_packet(game, player_id, &created) {
        send_no_packet_space(game, player_id);
        return Some(Ok(()));
    }
    let goods_name = created
        .last()
        .map(|goods| goods.name().to_vec())
        .unwrap_or_default();
    let goods_factory = game.goods_factory().clone();
    let da_kong_enabled = game.globe_setup().da_kong_key();
    let mut encode = |goods: &CGoods| {
        let mut payload = Vec::new();
        let _ = goods.serialize_for_old_client(
            &mut payload,
            &goods_factory,
            da_kong_enabled,
        );
        payload
    };
    let (additions, rejected) = game
        .add_increment_shop_goods_to_packet(player_id, created, &mut encode)
        .expect("UniBill player проверен перед packet add");
    let additions_succeeded = rejected.is_empty()
        && additions
            .iter()
            .all(|addition| addition.resulting_amount.is_some());
    for addition in additions {
        if addition.resulting_amount.is_some() {
            let _ = game.send_player_packet_addition(&addition);
        }
    }
    if !additions_succeeded {
        send_no_packet_space(game, player_id);
        return Some(Ok(()));
    }

    if deduction_goods_id != 0 {
        let consumptions = game
            .find_player_mut(player_id)
            .expect("UniBill player проверен перед deduction removal")
            .remove_item_in_packet(deduction_goods_id, goods_amount);
        for consumption in consumptions {
            let _ = game.send_player_packet_consumption(&consumption);
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
        let _ = audit.send(game, false);
    }
    debug!(player_id, last_point, charged, goods_id, goods_amount, deduction_goods_id, transaction_len = transaction.len(), "покупка Increment Shop завершена по ответу Billing");
    Some(Ok(()))
}

fn dispatch_billing_auth(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), IncrementShopBillingMessageError> {
    let player_id = read_billing_long(message, "billing auth player id")?;
    if game.find_player(player_id).is_none() {
        return Ok(());
    }
    let result = read_billing_long(message, "billing auth result")?;
    if result != 0 {
        log_billing_failure(message.socket_id(), "MSG_B2S_BILLING_AUTHEN", result);
        return Ok(());
    }
    let current = read_billing_long(message, "billing auth yuan bao")? as u32;
    let change = set_billing_yuan_bao(game, player_id, current)
        .expect("billing auth player проверен перед balance mutation");
    let _ = game.send_player_yuan_bao_change(&change);
    debug!(player_id, current, "баланс YuanBao применён после авторизации Billing");
    Ok(())
}

fn dispatch_billing_refresh(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), IncrementShopBillingMessageError> {
    let account = message.base_mut().get_str_bytes(0x40).ok_or(
        IncrementShopBillingMessageError::MissingField("billing refresh account"),
    )?;
    let player_id = game
        .find_player_by_account(&account)
        .map_or(0, |player| player.player_id());
    if player_id == 0 {
        return Ok(());
    }
    let result = read_billing_long(message, "billing refresh result")?;
    if result != 0 {
        log_billing_failure(message.socket_id(), "MSG_B2S_BILLING_REFRESH", result);
        return Ok(());
    }
    let current = read_billing_long(message, "billing refresh yuan bao")? as u32;
    let change = set_billing_yuan_bao(game, player_id, current)
        .expect("billing refresh account lookup вернул canonical player");
    let _ = game.send_player_yuan_bao_change(&change);
    debug!(player_id, current, "баланс YuanBao обновлён по ответу Billing");
    Ok(())
}

fn dispatch_auction_billing_trade(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), IncrementShopBillingMessageError> {
    let buyer_id = read_billing_long(message, "auction buyer id")?;
    let seller_id = read_billing_long(message, "auction seller id")?;
    let result = read_billing_long(message, "auction trade result")?;
    let buyer_yuan_bao = read_billing_long(message, "auction buyer yuan bao")? as u32;
    let seller_yuan_bao = read_billing_long(message, "auction seller yuan bao")? as u32;
    let trade_type = read_billing_long(message, "auction trade type")?;
    debug_assert_eq!(trade_type, 3);
    if game.find_player(buyer_id).is_none() {
        return Ok(());
    }
    if result != 0 {
        warn!(buyer_id, seller_id, result, "Billing отклонил аукционную сделку");
        return Ok(());
    }

    if let Some(node) = game
        .find_player_mut(buyer_id)
        .expect("Billing auction buyer проверен")
        .take_current_auction_buy_node()
    {
        let buyer_time = auction_billing_local_system_time();
        let (buyer_log, mut seller_log, notice) =
            super::onmsg_w2s_auction::build_auction_buy_log_effects(&node, game, buyer_time);
        let mut buyer_audit = CMessage::new(0x0006_0214);
        buyer_audit.base_mut().add(&buyer_log.to_legacy_bytes());
        let _ = buyer_audit.send(game, false);
        let _ = colored_player_notice_message(0xffff_ffff, 0xffff_0000, &notice)
            .send_to_player(game.net_server(), buyer_id);
        seller_log.time = auction_billing_local_system_time();
        let mut seller_audit = CMessage::new(0x0006_0214);
        seller_audit.base_mut().add(&seller_log.to_legacy_bytes());
        let _ = seller_audit.send(game, false);
        let _ = super::onmsg_w2s_auction::send_auction_node_world(&node, 0x0006_0806, game)
            .map_err(IncrementShopBillingMessageError::AuctionNodeSerialize)?;
        for query_player_id in [buyer_id, node.seller_id() as i32] {
            if query_player_id == buyer_id || game.find_player(query_player_id).is_some() {
                let mut query = CMessage::new(0x0006_0802);
                query.base_mut().add_long(query_player_id);
                let _ = query.send(game, false);
            }
        }
    }

    let buyer_change = set_billing_yuan_bao(game, buyer_id, buyer_yuan_bao)
        .expect("Billing auction buyer проверен перед balance");
    let _ = game.send_player_yuan_bao_change(&buyer_change);

    let seller_online = game.find_player(seller_id).is_some();
    if seller_online {
        let seller_change = set_billing_yuan_bao(game, seller_id, seller_yuan_bao)
            .expect("online auction seller проверен перед mutation");
        let _ = game.send_player_yuan_bao_change(&seller_change);
    } else {
        let mut offline = CMessage::new(0x0006_0814);
        offline.base_mut().add_long(seller_id);
        offline.base_mut().add_ulong(seller_yuan_bao);
        let _ = offline.send(game, false);
    }
    debug!(buyer_id, seller_id, seller_online, buyer_yuan_bao, seller_yuan_bao, "аукционная сделка завершена по ответу Billing");
    Ok(())
}

fn dispatch_player_billing_trade<Context: GameContainerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Result<(), IncrementShopBillingMessageError> {
    let payer_id = read_billing_long(message, "player trade payer id")?;
    let receiver_id = read_billing_long(message, "player trade receiver id")?;
    let result = read_billing_long(message, "player trade result")?;
    let payer_yuan_bao = read_billing_long(message, "player trade payer yuan bao")? as u32;
    let receiver_yuan_bao = read_billing_long(message, "player trade receiver yuan bao")? as u32;
    let trade_type = read_billing_long(message, "player trade type")?;
    debug_assert_eq!(trade_type, 2);
    if game.find_player(payer_id).is_none() || game.find_player(receiver_id).is_none() {
        return Ok(());
    }
    if result != 0 {
        warn!(payer_id, receiver_id, result, "Billing отклонил обмен игроков");
        return Ok(());
    }

    let payer_change = set_billing_yuan_bao(game, payer_id, payer_yuan_bao)
        .expect("Billing trade payer проверен перед balance mutation");
    let _ = game.send_player_yuan_bao_change(&payer_change);
    let receiver_change = set_billing_yuan_bao(game, receiver_id, receiver_yuan_bao)
        .expect("Billing trade receiver проверен перед balance mutation");
    let _ = game.send_player_yuan_bao_change(&receiver_change);

    let session_id = read_billing_long(message, "player trade session id")?;
    let plug_id = read_billing_long(message, "player trade plug id")?;
    let amount = read_billing_long(message, "player trade amount")? as u32;
    let _goods_id =
        message
            .base_mut()
            .get_guid()
            .ok_or(IncrementShopBillingMessageError::MissingField(
                "player trade goods guid",
            ))?;
    let transaction = message.base_mut().get_str_bytes(0x200).ok_or(
        IncrementShopBillingMessageError::MissingField("player trade transaction"),
    )?;
    let _ = game.complete_player_trade_after_billing(
        session_id,
        plug_id,
        payer_id,
        amount,
        &transaction,
        context,
    );
    debug!(payer_id, receiver_id, session_id, plug_id, amount, transaction_len = transaction.len(), "обмен игроков завершён по ответу Billing");
    Ok(())
}

fn dispatch_personal_shop_billing_trade<Context: GameContainerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Result<(), IncrementShopBillingMessageError> {
    let buyer_id = read_billing_long(message, "personal-shop buyer id")?;
    let seller_id = read_billing_long(message, "personal-shop seller id")?;
    let result = read_billing_long(message, "personal-shop trade result")?;
    let buyer_yuan_bao = read_billing_long(message, "personal-shop buyer yuan bao")? as u32;
    let seller_yuan_bao = read_billing_long(message, "personal-shop seller yuan bao")? as u32;
    let trade_type = read_billing_long(message, "personal-shop trade type")?;
    debug_assert_eq!(trade_type, 1);
    if game.find_player(buyer_id).is_none() || game.find_player(seller_id).is_none() {
        return Ok(());
    }
    if result != 0 {
        warn!(buyer_id, seller_id, result, "Billing отклонил сделку личной лавки");
        return Ok(());
    }
    let buyer_change = set_billing_yuan_bao(game, buyer_id, buyer_yuan_bao)
        .expect("Billing personal-shop buyer проверен перед balance");
    let _ = game.send_player_yuan_bao_change(&buyer_change);
    let seller_change = set_billing_yuan_bao(game, seller_id, seller_yuan_bao)
        .expect("Billing personal-shop seller проверен перед balance");
    let _ = game.send_player_yuan_bao_change(&seller_change);

    let session_id = read_billing_long(message, "personal-shop session id")?;
    let buyer_plug_id = read_billing_long(message, "personal-shop buyer plug id")?;
    let amount = read_billing_long(message, "personal-shop trade amount")? as u32;
    let goods_id =
        message
            .base_mut()
            .get_guid()
            .ok_or(IncrementShopBillingMessageError::MissingField(
                "personal-shop goods guid",
            ))?;
    let transaction = message.base_mut().get_str_bytes(0x200).ok_or(
        IncrementShopBillingMessageError::MissingField("personal-shop transaction"),
    )?;
    let completion =
        game.complete_personal_shop_billing_goods(session_id, buyer_plug_id, goods_id, context);
    if matches!(
        completion,
        crate::gameserver::gameserver::game::PersonalShopBillingCompletion::ShopUnavailable
    ) {
        let _ = colored_player_notice_message(0xffff_ffff, 0, b"Personal Shop has been CLOSED")
            .send_to_player(game.net_server(), buyer_id);
    }
    debug!(buyer_id, seller_id, session_id, buyer_plug_id, amount, transaction_len = transaction.len(), "сделка личной лавки завершена по ответу Billing");
    Ok(())
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

fn set_billing_yuan_bao(
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

fn send_no_packet_space(game: &CGame, player_id: i32) {
    let _ = colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0082"))
        .send_to_player(game.net_server(), player_id);
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let prefix = &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())];
    message.base_mut().add(prefix);
    message.base_mut().add_byte(0);
}

fn log_billing_failure(socket_id: i32, operation: &str, result: i32) {
    let line = format!("{socket_id} : Receive {operation} : (RES){result}...");
    eprintln!("{line}");
    put_string_to_file("Bill", line.as_bytes());
}
