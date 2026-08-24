//! Полный входной lifecycle магазина NPC GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/shopmessage.cpp`. Материализованы selectors
//! `0x8FD01..06`: shopping/death/region/NPC-distance gates, покупка из
//! `CTradeList`, packet placement, wallet, продажа hand goods, ремонт одного
//! и всех предметов, региональный налог с superior propagation, client wires
//! `0xBFA04/05/07`, container wires и optional `0x60202` audit.
//!
//! Rust owners заменяют allocator/RTTI/visitor plumbing. Повреждённый wire
//! становится typed error; исходные wrapping, integer durability ratio,
//! ties-even rounding, repair tax multiplier и отсутствие rollback уже
//! добавленного purchase-prefix сохранены явно.

use crate::gameserver::appserver::container::camountlimitgoodscontainer::AmountLimitGoodsRemoved;
use crate::gameserver::appserver::cs2ccontainerobjectmove::{
    CS2CContainerObjectMove, ContainerObjectMoveOperation,
};
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_PARTICULAR_ATTRIBUTE, GOODS_TYPE_EQUIPMENT,
};
use crate::gameserver::appserver::player::{GoodsSessionPlayerRelease, PlayerProgress};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, OldClientGoodsCodec, colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::guid::CGuid;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShopIgnoreReason {
    MissingPlayerContext,
    MissingPlayer,
    WrongProgress(PlayerProgress),
    MissingTradeGoods,
    MissingGoodsProperties,
    EmptyHand,
    MissingHandGoods,
    MissingRepairGoods,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShopMessageOutcome {
    Ignored(ShopIgnoreReason),
    Cancelled {
        release: GoodsSessionPlayerRelease,
        delivery: i32,
        notice: Option<i32>,
    },
    Notice {
        string_id: &'static str,
        delivery: i32,
    },
    Bought {
        goods_id: u32,
        amount: u32,
        gross: u32,
        tax: u32,
        charged: u32,
        additions: usize,
        packet_deliveries: Vec<i32>,
        money_deliveries: Vec<i32>,
        tax_deliveries: Vec<Result<i32, SendMessageError>>,
        audit_delivery: Option<Result<i32, SendMessageError>>,
    },
    Sold {
        goods: ShapeIdentity,
        amount: u32,
        gross: u32,
        tax: u32,
        proceeds: u32,
        move_delivery: i32,
        money_deliveries: Vec<i32>,
        tax_deliveries: Vec<Result<i32, SendMessageError>>,
        audit_delivery: Option<Result<i32, SendMessageError>>,
    },
    SaleRolledBack {
        delivery: i32,
    },
    RepairedOne {
        slot: u8,
        price: u32,
        delivery: i32,
        money_deliveries: Vec<i32>,
    },
    RepairedAll {
        count: u32,
        price: u32,
        delivery: i32,
        money_deliveries: Vec<i32>,
    },
    DistanceAccepted,
    Closed {
        release: GoodsSessionPlayerRelease,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShopMessageReport {
    pub(crate) message_type: i32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) outcome: ShopMessageOutcome,
}

pub(crate) fn dispatch_shop_message<Context: OldClientGoodsCodec>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<ShopMessageReport, ShopMessageError>> {
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
        return Some(Ok(report(
            message_type,
            None,
            region_id,
            ShopMessageOutcome::Ignored(ShopIgnoreReason::MissingPlayerContext),
        )));
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(Ok(report(
            message_type,
            Some(player_id),
            region_id,
            ShopMessageOutcome::Ignored(ShopIgnoreReason::MissingPlayer),
        )));
    };
    if player.current_progress() != PlayerProgress::Shopping {
        return Some(Ok(report(
            message_type,
            Some(player_id),
            region_id,
            ShopMessageOutcome::Ignored(ShopIgnoreReason::WrongProgress(player.current_progress())),
        )));
    }
    if player.is_dead() {
        return Some(Ok(report(
            message_type,
            Some(player_id),
            region_id,
            cancel(game, player_id, Some("GS0079")),
        )));
    }
    let Some(region_id) = region_id.filter(|id| game.find_region(*id).is_some()) else {
        return Some(Ok(report(
            message_type,
            Some(player_id),
            region_id,
            cancel(game, player_id, None),
        )));
    };

    let outcome = match message_type {
        BUY => handle_buy(message, game, context, player_id, region_id),
        SELL => handle_sell(message, game, context, player_id, region_id),
        REPAIR_ONE => handle_repair_one(message, game, player_id, region_id),
        REPAIR_ALL => handle_repair_all(message, game, player_id, region_id),
        DISTANCE_PROBE => handle_distance_probe(message, game, player_id, region_id),
        CLOSE => Ok(ShopMessageOutcome::Closed {
            release: game
                .find_player_mut(player_id)
                .expect("shop player остаётся live")
                .release_goods_session_state(),
        }),
        _ => unreachable!(),
    };
    Some(outcome.map(|outcome| report(message_type, Some(player_id), Some(region_id), outcome)))
}

fn handle_buy<Context: OldClientGoodsCodec>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
    player_id: i32,
    region_id: i32,
) -> Result<ShopMessageOutcome, ShopMessageError> {
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
        return Ok(ShopMessageOutcome::Ignored(
            ShopIgnoreReason::MissingTradeGoods,
        ));
    }
    let Some(npc_name) = nearby_npc_name(game, player_id, region_id, npc_id)? else {
        return Ok(cancel(game, player_id, None));
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
        return Ok(ShopMessageOutcome::Ignored(
            ShopIgnoreReason::MissingTradeGoods,
        ));
    };
    let Some(properties) = game
        .goods_factory()
        .query_goods_base_properties(entry.goods_id)
    else {
        return Ok(ShopMessageOutcome::Ignored(
            ShopIgnoreReason::MissingGoodsProperties,
        ));
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
        return Ok(notice(game, player_id, "GS0080"));
    }
    let unit_price = properties.price();
    let gross = unit_price.wrapping_mul(amount);
    let tax_rate = game
        .find_region(region_id)
        .expect("region checked")
        .base()
        .tax_rate();
    let tax = (tax_rate as f64 * gross as f64 * 0.01).round_ties_even() as u32;
    let charged_f64 = tax as f64 + gross as f64;
    let money = game
        .find_player(player_id)
        .expect("shop player live")
        .money();
    if unit_price == 0
        || (money as f64) < charged_f64
        || money / unit_price < amount
        || charged_f64 < unit_price as f64
    {
        return Ok(notice(game, player_id, "GS0081"));
    }
    let charged = charged_f64.round_ties_even() as u32;
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
        .add_npc_shop_goods_to_packet(player_id, created, context)
        .expect("shop player live");
    let packet_deliveries = additions
        .iter()
        .flat_map(|addition| game.send_player_packet_addition(addition))
        .collect::<Vec<_>>();
    if !rejected.is_empty() || additions.iter().any(|addition| addition.position.is_none()) {
        return Ok(notice(game, player_id, "GS0082"));
    }
    let decrease = game
        .decrease_player_money(player_id, charged)
        .expect("shop player live");
    let money_deliveries = game.send_player_money_decrease(player_id, &decrease.outcome);
    let tax_deliveries = game.add_region_tax(region_id, tax);
    let audit_delivery = game.log_system().goods_trade_log_enabled().then(|| {
        send_audit(
            game,
            1,
            player_id,
            log_identity.ex_id,
            charged,
            &log_name,
            amount,
            region_id,
        )
    });
    Ok(ShopMessageOutcome::Bought {
        goods_id: entry.goods_id,
        amount,
        gross,
        tax,
        charged,
        additions: additions.len(),
        packet_deliveries,
        money_deliveries,
        tax_deliveries,
        audit_delivery,
    })
}

fn handle_sell<Context: OldClientGoodsCodec>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
    player_id: i32,
    region_id: i32,
) -> Result<ShopMessageOutcome, ShopMessageError> {
    if game
        .find_player(player_id)
        .expect("shop player live")
        .hand()
        .traversing_goods()
        .len()
        == 0
    {
        return Ok(ShopMessageOutcome::Ignored(ShopIgnoreReason::EmptyHand));
    }
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    match npc_distance(game, player_id, region_id, npc_id)? {
        None => return Ok(cancel(game, player_id, None)),
        Some(distance) if 10 < distance => {
            return Ok(cancel_with_distance_notice(game, player_id));
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
            return Ok(ShopMessageOutcome::Ignored(
                ShopIgnoreReason::MissingHandGoods,
            ));
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
        return Ok(notice(game, player_id, "GS0086"));
    }
    if snapshot.4 == 0 {
        return Ok(notice(game, player_id, "GS0084"));
    }
    let gross = snapshot.4.wrapping_mul(snapshot.2);
    let tax_rate = game
        .find_region(region_id)
        .expect("region checked")
        .base()
        .tax_rate();
    let proceeds = ((1.0 - tax_rate as f32 * 0.01) * gross as f32).round_ties_even() as u32;
    let maximum = game
        .goods_factory()
        .query_goods_max_stack_number(game.goods_factory().get_gold_coin_index());
    let money = game
        .find_player(player_id)
        .expect("shop player live")
        .money();
    if maximum < money.wrapping_add(proceeds) {
        return Ok(notice(game, player_id, "GS0085"));
    }
    let removed = game
        .find_player_mut(player_id)
        .expect("shop player live")
        .hand_mut()
        .remove_goods(snapshot.0.ex_id);
    let Some(removed) = removed else {
        return Ok(ShopMessageOutcome::SaleRolledBack {
            delivery: CS2CContainerObjectMove::default().send_to_player(game, player_id),
        });
    };
    let move_delivery = send_hand_delete(game, player_id, &removed);
    let (_, money_deliveries) = game
        .increase_player_money(player_id, proceeds, context)
        .expect("shop player live");
    let tax = gross.wrapping_sub(proceeds);
    let tax_deliveries = game.add_region_tax(region_id, tax);
    let audit_delivery = game.log_system().goods_sell_to_npc_log_enabled().then(|| {
        send_audit(
            game,
            2,
            player_id,
            snapshot.0.ex_id,
            gross,
            &snapshot.1,
            snapshot.2,
            region_id,
        )
    });
    Ok(ShopMessageOutcome::Sold {
        goods: snapshot.0,
        amount: snapshot.2,
        gross,
        tax,
        proceeds,
        move_delivery,
        money_deliveries,
        tax_deliveries,
        audit_delivery,
    })
}

fn handle_repair_one(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<ShopMessageOutcome, ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    match npc_distance(game, player_id, region_id, npc_id)? {
        None => return Ok(cancel(game, player_id, None)),
        Some(distance) if 10 < distance => {
            return Ok(cancel_with_distance_notice(game, player_id));
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
        return Ok(ShopMessageOutcome::Ignored(
            ShopIgnoreReason::MissingRepairGoods,
        ));
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
        return Ok(notice(game, player_id, "GS0088"));
    }
    let repaired = repair_slot_mut(game, player_id, slot);
    if !repaired {
        return Ok(notice(game, player_id, "GS0087"));
    }
    let decrease = game
        .decrease_player_money(player_id, price)
        .expect("shop player live");
    let money_deliveries = game.send_player_money_decrease(player_id, &decrease.outcome);
    let mut response = CMessage::new(CLIENT_REPAIR_ONE);
    response.add_byte(b':');
    response.add_byte(slot);
    response.add_ulong(price);
    Ok(ShopMessageOutcome::RepairedOne {
        slot,
        price,
        delivery: response.send_to_player(game.net_server(), player_id),
        money_deliveries,
    })
}

fn handle_repair_all(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<ShopMessageOutcome, ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    match npc_distance(game, player_id, region_id, npc_id)? {
        None => return Ok(cancel(game, player_id, None)),
        Some(distance) if 10 < distance => {
            return Ok(cancel_with_distance_notice(game, player_id));
        }
        Some(_) => {}
    }
    let repair_factor = game.globe_setup().repair_factor();
    let packet_slots = game
        .find_player(player_id)
        .expect("shop player live")
        .packet()
        .size();
    let mut slots = Vec::new();
    for slot in 0..packet_slots + EQUIPMENT_REPAIR_SLOTS {
        if let Some(goods) = repair_slot(game, player_id, slot as u8)
            .filter(|goods| goods.can_repair(game.goods_factory()))
        {
            slots.push((
                slot as u8,
                game.goods_factory()
                    .calculate_repair_price(goods, repair_factor),
            ));
        }
    }
    if slots.is_empty() {
        return Ok(notice(game, player_id, "GS0089"));
    }
    let base_price = slots
        .iter()
        .fold(0u32, |total, (_, price)| total.wrapping_add(*price));
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
        return Ok(notice(game, player_id, "GS0088"));
    }
    let decrease = game
        .decrease_player_money(player_id, price)
        .expect("shop player live");
    let money_deliveries = game.send_player_money_decrease(player_id, &decrease.outcome);
    let mut response = CMessage::new(CLIENT_REPAIR_ALL);
    response.add_byte(b':');
    response.add_ulong(price);
    let delivery = response.send_to_player(game.net_server(), player_id);
    for (slot, _) in &slots {
        let _ = repair_slot_mut(game, player_id, *slot);
    }
    Ok(ShopMessageOutcome::RepairedAll {
        count: slots.len() as u32,
        price,
        delivery,
        money_deliveries,
    })
}

fn handle_distance_probe(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
) -> Result<ShopMessageOutcome, ShopMessageError> {
    let npc_id = message
        .base_mut()
        .get_long()
        .ok_or(ShopMessageError::MissingNpcId)?;
    let Some(distance) = npc_distance(game, player_id, region_id, npc_id)? else {
        return Ok(ShopMessageOutcome::DistanceAccepted);
    };
    if 11 <= distance {
        return Ok(notice(game, player_id, "GS0083"));
    }
    Ok(ShopMessageOutcome::DistanceAccepted)
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
    goods.is_some_and(|goods| factory.repair_equipment(goods))
}

fn cancel(game: &mut CGame, player_id: i32, string_id: Option<&'static str>) -> ShopMessageOutcome {
    let release = game
        .find_player_mut(player_id)
        .expect("shop player live")
        .release_goods_session_state();
    let delivery = CMessage::new(CLIENT_CANCEL).send_to_player(game.net_server(), player_id);
    let notice = string_id.map(|id| match notice(game, player_id, id) {
        ShopMessageOutcome::Notice { delivery, .. } => delivery,
        _ => 0,
    });
    ShopMessageOutcome::Cancelled {
        release,
        delivery,
        notice,
    }
}

fn cancel_with_distance_notice(game: &mut CGame, player_id: i32) -> ShopMessageOutcome {
    let distance = match notice(game, player_id, "GS0083") {
        ShopMessageOutcome::Notice { delivery, .. } => delivery,
        _ => 0,
    };
    match cancel(game, player_id, None) {
        ShopMessageOutcome::Cancelled {
            release, delivery, ..
        } => ShopMessageOutcome::Cancelled {
            release,
            delivery,
            notice: Some(distance),
        },
        outcome => outcome,
    }
}

fn notice(game: &CGame, player_id: i32, string_id: &'static str) -> ShopMessageOutcome {
    let delivery =
        colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id.as_bytes()))
            .send_to_player(game.net_server(), player_id);
    ShopMessageOutcome::Notice {
        string_id,
        delivery,
    }
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
) -> Result<i32, SendMessageError> {
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
    message.send(game, false)
}

fn add_c_string(message: &mut CMessage, bytes: &[u8]) {
    let visible = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(visible);
    message.add_byte(0);
}

fn report(
    message_type: i32,
    player_id: Option<i32>,
    region_id: Option<i32>,
    outcome: ShopMessageOutcome,
) -> ShopMessageReport {
    ShopMessageReport {
        message_type,
        player_id,
        region_id,
        outcome,
    }
}
