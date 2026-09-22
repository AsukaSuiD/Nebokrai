//! Полный входной жизненный цикл магазина NPC GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/message/shopmessage.cpp`. Материализованы селекторы
//! `0x8FD01..06`: проверки процесса покупки, смерти, региона и расстояния до
//! NPC, покупка из `CTradeList`, размещение в packet, изменение кошелька,
//! продажа товара из hand, ремонт одного или всех предметов, региональный
//! налог с распространением владельцу superior, клиентские сообщения
//! `0xBFA04/05/07`, сообщения контейнеров и необязательный аудит `0x60202`.
//!
//! Владельцы Rust заменяют служебный код allocator/RTTI/visitor. Повреждённый
//! wire становится типизированной ошибкой; исходные wrapping, целочисленное
//! отношение прочности, x87-усечение налогов, налоговый множитель ремонта и
//! отсутствие rollback уже добавленного префикса покупки сохранены явно.
//! Синхронные эффекты не дублируются отчётами; их диагностические итоги
//! публикуются через `tracing`.

use crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsRemoved;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{
    CS2CContainerObjectMove, ContainerObjectMoveOperation,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_PARTICULAR_ATTRIBUTE, GOODS_TYPE_EQUIPMENT,
};
use crate::gameserver::appserver::listener::cgoodsrepairlistener::repair_visited_goods;
use crate::gameserver::appserver::listener::cgoodsrepairpricelistener::GoodsRepairPrice;
use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::gameserver::game::{
    CGame, colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;
use tracing::{debug, trace};

const BUY: i32 = 0x0008_fd01;
const SELL: i32 = 0x0008_fd02;
const REPAIR_ONE: i32 = 0x0008_fd03;
const REPAIR_ALL: i32 = 0x0008_fd04;
const DISTANCE_PROBE: i32 = 0x0008_fd05;
const CLOSE: i32 = 0x0008_fd06;
const CLIENT_REPAIR_ONE: i32 = 0x000b_fa04;
const CLIENT_REPAIR_ALL: i32 = 0x000b_fa05;
const CLIENT_CANCEL: i32 = 0x000b_fa07;
const WORLD_AUDIT: i32 = 0x0006_0202;
const PLAYER_TYPE: i32 = 400;
const HAND_EXTEND_ID: i32 = 3;
const EQUIPMENT_REPAIR_SLOTS: u32 = 17;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShopMessageError {
    MissingNpcId,
    MissingPage,
    MissingPositionX,
    MissingPositionY,
    MissingAmount,
    MissingRepairSlot,
}

pub(crate) fn dispatch_shop_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<(), ShopMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        BUY | SELL | REPAIR_ONE | REPAIR_ALL | DISTANCE_PROBE | CLOSE
    ) {
        return None;
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        trace!(message_type, ?region_id, "сообщение магазина пропущено: нет контекста игрока");
        return Some(Ok(()));
    };
    let Some(player) = game.find_player(player_id) else {
        trace!(message_type, player_id, ?region_id, "сообщение магазина пропущено: игрок не найден");
        return Some(Ok(()));
    };
    if player.current_progress() != PlayerProgress::Shopping {
        trace!(message_type, player_id, ?region_id, progress = ?player.current_progress(), "сообщение магазина пропущено из-за процесса игрока");
        return Some(Ok(()));
    }
    if player.is_dead() {
        cancel(game, player_id, Some("GS0079"));
        return Some(Ok(()));
    }
    let Some(region_id) = region_id.filter(|id| game.find_region(*id).is_some()) else {
        cancel(game, player_id, None);
        return Some(Ok(()));
    };

    let outcome = match message_type {
        BUY => handle_buy(message, game, player_id, region_id),
        SELL => handle_sell(message, game, player_id, region_id),
        REPAIR_ONE => handle_repair_one(message, game, player_id, region_id),
        REPAIR_ALL => handle_repair_all(message, game, player_id, region_id),
        DISTANCE_PROBE => handle_distance_probe(message, game, player_id, region_id),
        CLOSE => {
            let _ = game
                .find_player_mut(player_id)
                .expect("shop player остаётся live")
                .release_goods_session_state();
            debug!(player_id, region_id, "сессия магазина закрыта");
            Ok(())
        }
        _ => unreachable!(),
    };
    Some(outcome)
}

fn handle_buy(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<(), ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    let page = message
        .base_mut()
        .get_byte()
        .ok_or(ShopMessageError::MissingPage)?;
    let x = message
        .base_mut()
        .get_byte()
        .ok_or(ShopMessageError::MissingPositionX)?;
    let y = message
        .base_mut()
        .get_byte()
        .ok_or(ShopMessageError::MissingPositionY)?;
    let mut amount = message
        .base_mut()
        .get_long()
        .map(|value| value as u32)
        .ok_or(ShopMessageError::MissingAmount)?;
    if amount == 0 {
        trace!(player_id, region_id, "покупка с нулевым количеством пропущена");
        return Ok(());
    }
    let Some(npc_name) = nearby_npc_name(game, player_id, region_id, npc_id)? else {
        cancel(game, player_id, None);
        return Ok(());
    };
    let Some(entry) =
        game.trade_list()
            .get_trade(&npc_name)
            .and_then(|trade| {
                trade.goods().iter().find(|goods| {
                    goods.page == page && goods.position_x == x && goods.position_y == y
                })
            })
            .copied()
    else {
        trace!(player_id, region_id, npc_id, "товар не найден в торговом списке NPC");
        return Ok(());
    };
    let Some(properties) = game
        .goods_factory()
        .query_goods_base_properties(entry.goods_id)
    else {
        trace!(player_id, region_id, goods_id = entry.goods_id, "свойства товара магазина не найдены");
        return Ok(());
    };
    if properties.goods_type() == GOODS_TYPE_EQUIPMENT {
        amount = 1;
    }
    let burden = game
        .find_player(player_id)
        .expect("shop player live")
        .current_burden(game.goods_factory());
    let maximum_burden = u32::from(
        game.find_player(player_id)
            .expect("shop player live")
            .combat_properties()
            .burden,
    );
    let added_burden = u32::from(entry.amount)
        .wrapping_mul(properties.weight())
        .wrapping_mul(amount);
    if maximum_burden < burden.wrapping_add(added_burden) {
        notice(game, player_id, "GS0080");
        return Ok(());
    }
    let unit_price = properties.price();
    let gross = unit_price.wrapping_mul(amount);
    let tax_rate = game
        .find_region(region_id)
        .expect("region checked")
        .base()
        .tax_rate();
    let gross_f64 = f64::from(unit_price) * f64::from(amount);
    let tax = (f64::from(tax_rate) * gross_f64 * 0.01_f64).trunc() as i64 as u32;
    let charged_f64 = f64::from(tax as i32) + gross_f64;
    let money = game
        .find_player(player_id)
        .expect("shop player live")
        .money();
    if unit_price == 0
        || (money as f64) < charged_f64
        || money / unit_price < amount
        || charged_f64 < unit_price as f64
    {
        notice(game, player_id, "GS0081");
        return Ok(());
    }
    let charged = charged_f64.trunc() as i64 as u32;
    let created = game.create_goods_batch(entry.goods_id, amount);
    if created.is_empty()
        || !game
            .find_player(player_id)
            .expect("shop player live")
            .packet()
            .is_space_enough_for_goods(&created, game.goods_factory())
    {
        return Ok(notice(game, player_id, "GS0082"));
    }
    let log_identity = created
        .last()
        .expect("непустой factory batch проверен")
        .identity();
    let log_name = created
        .last()
        .map(CGoods::name)
        .unwrap_or_default()
        .to_vec();
    let (additions, rejected) = game
        .add_npc_shop_goods_to_packet(player_id, created)
        .expect("shop player live");
    for addition in &additions {
        let _ = game.send_player_packet_addition(addition);
    }
    if !rejected.is_empty() || additions.iter().any(|addition| addition.position.is_none()) {
        notice(game, player_id, "GS0082");
        return Ok(());
    }
    let decrease = game
        .decrease_player_money(player_id, charged)
        .expect("shop player live");
    let _ = game.send_player_money_decrease(player_id, &decrease.outcome);
    let _ = game.add_region_tax(region_id, tax);
    if game.log_system().goods_trade_log_enabled() {
        send_audit(
            game,
            1,
            player_id,
            log_identity.ex_id,
            charged,
            &log_name,
            amount,
            region_id,
        );
    }
    debug!(player_id, region_id, goods_id = entry.goods_id, amount, gross, tax, charged, additions = additions.len(), "товар куплен у NPC");
    Ok(())
}

fn handle_sell(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<(), ShopMessageError> {
    if game
        .find_player(player_id)
        .expect("shop player live")
        .hand()
        .traversing_goods()
        .len()
        == 0
    {
        trace!(player_id, region_id, "продажа пропущена: рука пуста");
        return Ok(());
    }
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    match npc_distance(game, player_id, region_id, npc_id)? {
        None => {
            cancel(game, player_id, None);
            return Ok(());
        }
        Some(distance) if 10 < distance => {
            cancel_with_distance_notice(game, player_id);
            return Ok(());
        }
        Some(_) => {}
    }
    let snapshot = {
        let Some(goods) = game
            .find_player(player_id)
            .expect("shop player live")
            .hand()
            .get_goods(0)
        else {
            trace!(player_id, region_id, "товар для продажи не найден в руке");
            return Ok(());
        };
        (
            goods.identity(),
            goods.name().to_vec(),
            goods.amount(),
            goods.addon_property_value(game.goods_factory(), GAP_PARTICULAR_ATTRIBUTE, 1),
            game.goods_factory().calculate_vend_price(
                goods,
                game.globe_setup().base_price_rate(),
                game.globe_setup().trade_in_rate(),
            ),
        )
    };
    if snapshot.3 as u32 & 0x100 != 0 {
        notice(game, player_id, "GS0086");
        return Ok(());
    }
    if snapshot.4 == 0 {
        notice(game, player_id, "GS0084");
        return Ok(());
    }
    let gross = snapshot.4.wrapping_mul(snapshot.2);
    let tax_rate = game
        .find_region(region_id)
        .expect("region checked")
        .base()
        .tax_rate();
    let gross_float = gross as f32;
    let proceeds = ((1.0 - f64::from(tax_rate) * f64::from(0.01_f32))
        * f64::from(gross_float))
    .trunc() as i32 as u32;
    let maximum = game
        .goods_factory()
        .query_goods_max_stack_number(game.goods_factory().get_gold_coin_index());
    let money = game
        .find_player(player_id)
        .expect("shop player live")
        .money();
    if maximum < money.wrapping_add(proceeds) {
        notice(game, player_id, "GS0085");
        return Ok(());
    }
    let removed = game
        .find_player_mut(player_id)
        .expect("shop player live")
        .hand_mut()
        .remove_goods(snapshot.0.ex_id);
    let Some(removed) = removed else {
        let _ = CS2CContainerObjectMove::default().send_to_player(game, player_id);
        trace!(player_id, region_id, "продажа отменена после неудачного изъятия товара");
        return Ok(());
    };
    let _ = send_hand_delete(game, player_id, &removed);
    let _ = game
        .increase_player_money(player_id, proceeds)
        .expect("shop player live");
    let tax_product = (tax_rate as u32).wrapping_mul(gross);
    let tax = (f64::from(tax_product) * f64::from(0.01_f32)).trunc() as i32 as u32;
    let _ = game.add_region_tax(region_id, tax);
    if game.log_system().goods_sell_to_npc_log_enabled() {
        send_audit(
            game,
            2,
            player_id,
            snapshot.0.ex_id,
            gross,
            &snapshot.1,
            snapshot.2,
            region_id,
        );
    }
    debug!(player_id, region_id, amount = snapshot.2, gross, tax, proceeds, "товар продан NPC");
    Ok(())
}

fn handle_repair_one(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<(), ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    match npc_distance(game, player_id, region_id, npc_id)? {
        None => {
            cancel(game, player_id, None);
            return Ok(());
        }
        Some(distance) if 10 < distance => {
            cancel_with_distance_notice(game, player_id);
            return Ok(());
        }
        Some(_) => {}
    }
    let slot = message
        .base_mut()
        .get_byte()
        .ok_or(ShopMessageError::MissingRepairSlot)?;
    let base_price = repair_slot(game, player_id, slot).map(|goods| {
        game.goods_factory()
            .calculate_repair_price(goods, game.globe_setup().repair_factor())
    });
    let Some(base_price) = base_price else {
        trace!(player_id, region_id, slot, "предмет для ремонта не найден");
        return Ok(());
    };
    let multiplier = (game
        .find_region(region_id)
        .expect("region checked")
        .base()
        .tax_rate()
        / 100
        + 1) as u32;
    let price = base_price.wrapping_mul(multiplier);
    if game
        .find_player(player_id)
        .expect("shop player live")
        .money()
        < price
    {
        notice(game, player_id, "GS0088");
        return Ok(());
    }
    let repaired = repair_slot_mut(game, player_id, slot);
    if !repaired {
        notice(game, player_id, "GS0087");
        return Ok(());
    }
    let decrease = game
        .decrease_player_money(player_id, price)
        .expect("shop player live");
    let _ = game.send_player_money_decrease(player_id, &decrease.outcome);
    let mut response = CMessage::new(CLIENT_REPAIR_ONE);
    response.add_byte(b':');
    response.add_byte(slot);
    response.add_ulong(price);
    let _ = response.send_to_player(game.net_server(), player_id);
    debug!(player_id, region_id, slot, price, "предмет отремонтирован");
    Ok(())
}

fn handle_repair_all(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<(), ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    match npc_distance(game, player_id, region_id, npc_id)? {
        None => {
            cancel(game, player_id, None);
            return Ok(());
        }
        Some(distance) if 10 < distance => {
            cancel_with_distance_notice(game, player_id);
            return Ok(());
        }
        Some(_) => {}
    }
    let repair_factor = game.globe_setup().repair_factor();
    let packet_slots = game
        .find_player(player_id)
        .expect("shop player live")
        .packet()
        .size();
    let mut repair_slots = Vec::new();
    let mut repair_price = GoodsRepairPrice::default();
    for slot in 0..packet_slots + EQUIPMENT_REPAIR_SLOTS {
        if let Some(goods) = repair_slot(game, player_id, slot as u8) {
            let _ = repair_price.visit(game.goods_factory(), goods, repair_factor);
            if goods.can_repair(game.goods_factory()) {
                repair_slots.push(slot as u8);
            }
        }
    }
    if repair_price.equipment_count() == 0 {
        notice(game, player_id, "GS0089");
        return Ok(());
    }
    let base_price = repair_price.price();
    let multiplier = (game
        .find_region(region_id)
        .expect("region checked")
        .base()
        .tax_rate()
        / 100
        + 1) as u32;
    let price = base_price.wrapping_mul(multiplier);
    if game
        .find_player(player_id)
        .expect("shop player live")
        .money()
        < price
    {
        notice(game, player_id, "GS0088");
        return Ok(());
    }
    let decrease = game
        .decrease_player_money(player_id, price)
        .expect("shop player live");
    let _ = game.send_player_money_decrease(player_id, &decrease.outcome);
    let mut response = CMessage::new(CLIENT_REPAIR_ALL);
    response.add_byte(b':');
    response.add_ulong(price);
    let _ = response.send_to_player(game.net_server(), player_id);
    for slot in &repair_slots {
        let _ = repair_slot_mut(game, player_id, *slot);
    }
    debug!(player_id, region_id, count = repair_price.equipment_count(), price, "все подходящие предметы отремонтированы");
    Ok(())
}

fn handle_distance_probe(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<(), ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    let Some(distance) = npc_distance(game, player_id, region_id, npc_id)? else {
        return Ok(());
    };
    if 11 <= distance {
        notice(game, player_id, "GS0083");
    }
    Ok(())
}

fn nearby_npc_name(
    game: &CGame,
    player_id: i32,
    region_id: i32,
    npc_id: i32,
) -> Result<Option<Vec<u8>>, ShopMessageError> {
    let Some(distance) = npc_distance(game, player_id, region_id, npc_id)? else {
        return Ok(None);
    };
    if 10 < distance {
        return Ok(None);
    }
    Ok(game
        .find_region(region_id)
        .and_then(|region| region.base().find_npc_by_id(npc_id))
        .map(|npc| npc.name().to_vec()))
}

fn npc_distance(
    game: &CGame,
    player_id: i32,
    region_id: i32,
    npc_id: i32,
) -> Result<Option<i32>, ShopMessageError> {
    let player = game
        .find_player(player_id)
        .and_then(|player| player.shape_view());
    let npc = game
        .find_region(region_id)
        .and_then(|region| region.base().find_npc_by_id(npc_id))
        .and_then(|npc| npc.shape_view());
    Ok(player.zip(npc).map(|(player, npc)| player.distance(npc)))
}

fn repair_slot(game: &CGame, player_id: i32, slot: u8) -> Option<&CGoods> {
    let player = game.find_player(player_id)?;
    let position = u32::from(slot);
    let packet_slots = player.packet().size();
    if position < packet_slots {
        player.packet().get_goods(position)
    } else {
        player.equipment().get_goods(position - packet_slots)
    }
}

fn repair_slot_mut(game: &mut CGame, player_id: i32, slot: u8) -> bool {
    let position = u32::from(slot);
    let factory = game.goods_factory().clone();
    let player = game.find_player_mut(player_id).expect("shop player live");
    let packet_slots = player.packet().size();
    let goods = if position < packet_slots {
        player.packet_mut().get_goods_mut(position)
    } else {
        player
            .equipment_mut()
            .get_goods_mut(position - packet_slots)
    };
    goods.is_some_and(|goods| {
        let repairable = goods.can_repair(&factory);
        let _ = repair_visited_goods(&factory, goods);
        repairable
    })
}

fn cancel(game: &mut CGame, player_id: i32, string_id: Option<&'static str>) {
    let _ = game
        .find_player_mut(player_id)
        .expect("shop player live")
        .release_goods_session_state();
    let _ = CMessage::new(CLIENT_CANCEL).send_to_player(game.net_server(), player_id);
    if let Some(string_id) = string_id {
        notice(game, player_id, string_id);
    }
}

fn cancel_with_distance_notice(game: &mut CGame, player_id: i32) {
    notice(game, player_id, "GS0083");
    cancel(game, player_id, None);
}

fn notice(game: &CGame, player_id: i32, string_id: &'static str) {
    let _ =
        colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id.as_bytes()))
            .send_to_player(game.net_server(), player_id);
}

fn send_hand_delete(game: &CGame, player_id: i32, removed: &AmountLimitGoodsRemoved) -> i32 {
    let mut message = CS2CContainerObjectMove::default();
    message.set_operation(ContainerObjectMoveOperation::DeleteObject);
    message.set_source_container(PLAYER_TYPE, player_id, removed.position.unwrap_or_default());
    message.set_source_container_extend_id(HAND_EXTEND_ID);
    let identity = removed.goods.identity();
    message.set_source_object(identity.object_type, identity.ex_id, removed.amount);
    message.send_to_player(game, player_id)
}

fn send_audit(
    game: &CGame,
    reason: u8,
    player_id: i32,
    guid: CGuid,
    price: u32,
    name: &[u8],
    amount: u32,
    region_id: i32,
) {
    let player = game.find_player(player_id).expect("shop player live");
    let mut message = CMessage::new(WORLD_AUDIT);
    message.add_byte(reason);
    message.add_long(player_id);
    message.base_mut().add_short(player.pk_count() as i16);
    message.add_ulong(player.money());
    message.add_ulong(player.depot_money());
    message.base_mut().add_guid(guid);
    message.add_ulong(price);
    add_c_string(&mut message, name);
    message.add_ulong(amount);
    message.add_long(region_id);
    message.add_ulong(player.shape().get_tile_x().unwrap_or_default() as u32);
    message.add_ulong(player.shape().get_tile_y().unwrap_or_default() as u32);
    message.add_ulong(player.client_ip());
    let _ = message.send(game, false);
}

fn add_c_string(message: &mut CMessage, bytes: &[u8]) {
    let visible = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(visible);
    message.add_byte(0);
}
