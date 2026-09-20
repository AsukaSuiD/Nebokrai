//! Диспетчер клиентских сообщений аукциона GameServer.
//!
//! Источник — точная пара GameServer EXE/PDB, владелец
//! `server/gameserver/appserver/message/onmsg_c2s_auction.cpp`. Реализованы
//! закрытие, выставление, снятие и покупка лота, поиск, страницы, открытие
//! аукциона и покупка расширения. Сохранены точные коды и байты сообщений,
//! пятисекундные ограничения, порядок списаний, журналов, клиентских и
//! межсерверных отправок, а также граница ожидающей операции Billing.
//!
//! Все эффекты выполняются синхронно в исходных ветвях. Результаты отправок и
//! причины отказов фиксируются через `tracing`, не накапливаются в отчётах.
//! `0x90A12` сохраняет отдельный от auction-enabled gate путь: запускает
//! quest-complete script и помещает синтетический `0x8FB02` в хвост общего
//! GameServer FIFO, поэтому продолжение исполняется только на следующем
//! message snapshot. Остальных отложенных эффектов у владельца нет.
//! Денежный порог и плата за выставление используют исходные `f32`
//! коэффициенты, x87-подобный порядок и усечение к нулю на каждой границе.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GOODS_PACKAGE_EXTENTION;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_GOODS_AUCTION_SCALE, GAP_ROLE_MINIMUM_LEVEL_LIMIT, GOODS_TYPE_CONSUMABLE,
    GOODS_TYPE_EQUIPMENT, GOODS_TYPE_USELESS,
};
use crate::gameserver::appserver::player::{
    AuctionBuyGate, AuctionListingGate, AuctionSelfGoodsRefresh,
};
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, colored_player_notice_message, game_wall_time_seconds,
};
use crate::nets::netserver::message::CMessage;
use crate::public::auctionnode::{AuctionListingNodeFields, CGoodsNode};

const CLIENT_AUCTION_CLOSE_MESSAGE: i32 = 0x0009_0A01;
const CLIENT_AUCTION_LIST_MESSAGE: i32 = 0x0009_0A02;
const CLIENT_AUCTION_CUT_MESSAGE: i32 = 0x0009_0A03;
const CLIENT_AUCTION_BUY_MESSAGE: i32 = 0x0009_0A04;
const CLIENT_AUCTION_REFRESH_MESSAGE: i32 = 0x0009_0A05;
const CLIENT_AUCTION_SEARCH_MESSAGE: i32 = 0x0009_0A06;
const CLIENT_AUCTION_PAGE_MESSAGE: i32 = 0x0009_0A07;
const CLIENT_AUCTION_SELF_MESSAGE: i32 = 0x0009_0A08;
const CLIENT_AUCTION_RELAY_MESSAGE: i32 = 0x0009_0A09;
const CLIENT_AUCTION_CELL_MESSAGE: i32 = 0x0009_0A0A;
const CLIENT_AUCTION_OPEN_MESSAGE: i32 = 0x0009_0A0B;
const CLIENT_AUCTION_BUY_EXTENSION_MESSAGE: i32 = 0x0009_0A0C;
const CLIENT_AUCTION_QUEST_RESPONSE_MESSAGE: i32 = 0x0009_0A12;
const CLIENT_AUCTION_OPEN_SETUP_MESSAGE: i32 = 0x000C_0706;
const WORLD_AUCTION_PAGE_MESSAGE: i32 = 0x0006_0803;
const WORLD_AUCTION_REFRESH_MESSAGE: i32 = 0x0006_080A;
const WORLD_AUCTION_SELF_MESSAGE: i32 = 0x0006_0802;
const WORLD_AUCTION_SEARCH_MESSAGE: i32 = 0x0006_080F;
const WORLD_AUCTION_RELAY_MESSAGE: i32 = 0x0006_080B;
const WORLD_AUCTION_CELL_MESSAGE: i32 = 0x0006_080C;
const WORLD_AUCTION_OPEN_MESSAGE: i32 = 0x0006_0810;
const WORLD_AUCTION_CUT_MESSAGE: i32 = 0x0006_0809;
const WORLD_AUCTION_CUT_LOG_MESSAGE: i32 = 0x0006_0216;
const WORLD_AUCTION_BUY_MESSAGE: i32 = 0x0006_0805;
const WORLD_GOODS_AUDIT_MESSAGE: i32 = 0x0006_0202;
const WORLD_AUCTION_ADD_NODE_MESSAGE: i32 = 0x0006_0801;
const WORLD_AUCTION_SALE_LOG_MESSAGE: i32 = 0x0006_0215;
const CLIENT_AUCTION_LIST_RESULT_MESSAGE: i32 = 0x000C_0701;

pub(crate) fn dispatch_client_auction_message<Runtime, Tick>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    mut tick_ms: Tick,
) -> Option<()>
where
    Runtime: GameContainerMessageRuntime,
    Tick: FnMut(&mut Runtime) -> u32,
{
    let selector = message.message_type();
    if !matches!(
        selector,
        CLIENT_AUCTION_CLOSE_MESSAGE
            | CLIENT_AUCTION_LIST_MESSAGE
            | CLIENT_AUCTION_CUT_MESSAGE
            | CLIENT_AUCTION_BUY_MESSAGE
            | CLIENT_AUCTION_REFRESH_MESSAGE
            | CLIENT_AUCTION_SEARCH_MESSAGE
            | CLIENT_AUCTION_PAGE_MESSAGE
            | CLIENT_AUCTION_SELF_MESSAGE
            | CLIENT_AUCTION_RELAY_MESSAGE
            | CLIENT_AUCTION_CELL_MESSAGE
            | CLIENT_AUCTION_OPEN_MESSAGE
            | CLIENT_AUCTION_BUY_EXTENSION_MESSAGE
            | CLIENT_AUCTION_QUEST_RESPONSE_MESSAGE
    ) {
        return None;
    }
    let Some(player_id) = message.player_id() else {
        tracing::trace!(selector, "у сообщения аукциона нет игрока");
        return Some(());
    };
    if game.find_player(player_id).is_none() {
        tracing::trace!(selector, player_id, "игрок сообщения аукциона не найден");
        return Some(());
    }
    if selector == CLIENT_AUCTION_QUEST_RESPONSE_MESSAGE {
        let quest_id = message.base_mut().get_long().unwrap_or_default() as u16;
        let response = message
            .base_mut()
            .get_str_bytes(0x20)
            .unwrap_or_default();
        let script_id = game.queue_player_quest_complete_script(player_id, quest_id);
        if let Some(script_id) = script_id {
            let region_id = game
                .find_player(player_id)
                .and_then(|player| player.server_region_id());
            let mut continuation = CMessage::new(0x0008_fb02);
            continuation.base_mut().add_long(script_id);
            continuation.base_mut().add_long(1);
            continuation.base_mut().add(&response);
            continuation.base_mut().add_byte(0);
            continuation.apply_player_context(player_id, region_id);
            game.net_server()
                .event_publisher()
                .publish_message(continuation);
        }
        tracing::trace!(
            selector,
            player_id,
            quest_id,
            ?script_id,
            response_bytes = response.len(),
            "ответ аукционного задания поставлен в GameServer FIFO"
        );
        return Some(());
    }
    if !game.auction_now() {
        let notice_delivery =
            colored_player_notice_message(0xffff_0000, 0, game.get_string_by_id(b"GPM013"))
                .send_to_player(game.net_server(), player_id);
        tracing::trace!(selector, player_id, notice_delivery, "аукцион отключён");
        return Some(());
    }
    if selector == CLIENT_AUCTION_CLOSE_MESSAGE {
        game.find_player_mut(player_id)
            .expect("player проверен до close")
            .set_auction_open(false);
        tracing::trace!(selector, player_id, "аукцион закрыт игроком");
        return Some(());
    }
    if selector == CLIENT_AUCTION_LIST_MESSAGE {
        return Some(dispatch_auction_listing(
            message,
            game,
            runtime,
            &mut tick_ms,
            player_id,
        ));
    }
    if selector == CLIENT_AUCTION_CUT_MESSAGE {
        let goods_id = message.base_mut().get_guid().unwrap_or_default();
        let cut_log_delivery = (goods_id != crate::public::guid::CGuid::GUID_INVALID).then(|| {
            let mut audit = CMessage::new(WORLD_AUCTION_CUT_LOG_MESSAGE);
            audit.base_mut().add_long(player_id);
            audit.base_mut().add_guid(goods_id);
            audit.send(game, false)
        });
        message.set_message_type(WORLD_AUCTION_CUT_MESSAGE);
        message.base_mut().update();
        let world_delivery = message.send(game, false);
        tracing::trace!(selector, player_id, ?goods_id, ?cut_log_delivery, ?world_delivery, "снятие лота передано WorldServer");
        return Some(());
    }
    if selector == CLIENT_AUCTION_BUY_MESSAGE {
        let Some(advertised_yuan_bao) = message.base_mut().get_long() else {
            tracing::warn!(selector, player_id, field = "advertised yuan bao", "неполное сообщение аукциона");
            return Some(());
        };
        let available_yuan_bao = game
            .find_player(player_id)
            .expect("auction buy player проверен")
            .yuan_bao();
        if (available_yuan_bao as i32) < advertised_yuan_bao {
            let notice_delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM021"))
                    .send_to_player(game.net_server(), player_id);
            tracing::trace!(selector, player_id, advertised_yuan_bao, available_yuan_bao, notice_delivery, "недостаточно YuanBao для покупки лота");
            return Some(());
        }
        let gate = game
            .find_player_mut(player_id)
            .expect("auction buy player проверен перед throttle")
            .begin_auction_buy(|| tick_ms(runtime));
        let mut goods_id = crate::public::guid::CGuid::GUID_INVALID;
        let world_delivery = if matches!(gate, AuctionBuyGate::Ready { .. }) {
            goods_id = message.base_mut().get_guid().unwrap_or_default();
            if game.auction_room().contains_goods(goods_id) {
                let mut request = CMessage::new(WORLD_AUCTION_BUY_MESSAGE);
                request.base_mut().add_long(player_id);
                request.base_mut().add_guid(goods_id);
                Some(request.send(game, false))
            } else {
                None
            }
        } else {
            None
        };
        tracing::trace!(selector, player_id, advertised_yuan_bao, ?goods_id, ?gate, ?world_delivery, "запрос покупки лота обработан");
        return Some(());
    }
    if selector == CLIENT_AUCTION_REFRESH_MESSAGE {
        let refresh = game
            .refresh_player_auction_self_goods(player_id, || tick_ms(runtime))
            .expect("player проверен до refresh");
        let world_delivery = match refresh {
            AuctionSelfGoodsRefresh::Requested {
                goods_space,
                wallet_space,
                ..
            } => {
                let mut world = CMessage::new(WORLD_AUCTION_REFRESH_MESSAGE);
                world.base_mut().add_long(player_id);
                world.base_mut().add_ulong(goods_space);
                world.base_mut().add_ulong(wallet_space);
                Some(world.send(game, false))
            }
            AuctionSelfGoodsRefresh::Throttled { .. } => None,
        };
        tracing::trace!(selector, player_id, ?refresh, ?world_delivery, "собственные лоты обновлены");
        return Some(());
    }
    if selector == CLIENT_AUCTION_SEARCH_MESSAGE {
        let name = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
        let mut read = || message.base_mut().get_long();
        let lower_level = match read() {
            Some(value) => value,
            None => return Some(()),
        };
        let upper_level = match read() {
            Some(value) => value,
            None => return Some(()),
        };
        let use_self = match read() {
            Some(value) => value,
            None => return Some(()),
        };
        let money_type = match read() {
            Some(value) => value,
            None => return Some(()),
        };
        let weapon_type = match read() {
            Some(value) => value,
            None => return Some(()),
        };
        game.find_player_mut(player_id)
            .expect("player проверен до search")
            .begin_auction_search(
                &name,
                lower_level,
                upper_level,
                use_self,
                money_type,
                weapon_type,
            );
        let mut world = CMessage::new(WORLD_AUCTION_SEARCH_MESSAGE);
        world.base_mut().add_long(player_id);
        for value in [lower_level, upper_level, use_self, money_type, weapon_type] {
            world.base_mut().add_long(value);
        }
        world.base_mut().add(&name);
        world.base_mut().add_byte(0);
        let world_delivery = world.send(game, false);
        tracing::trace!(selector, player_id, ?world_delivery, "поиск аукциона передан WorldServer");
        return Some(());
    }
    if selector == CLIENT_AUCTION_PAGE_MESSAGE {
        let Some(page) = message.base_mut().get_long() else {
            tracing::warn!(selector, player_id, field = "page", "неполное сообщение аукциона");
            return Some(());
        };
        let mut world = CMessage::new(WORLD_AUCTION_PAGE_MESSAGE);
        world.base_mut().add_long(player_id);
        world.base_mut().add_long(page);
        let world_delivery = world.send(game, false);
        tracing::trace!(selector, player_id, page, ?world_delivery, "страница аукциона запрошена");
        return Some(());
    }
    if selector == CLIENT_AUCTION_SELF_MESSAGE {
        let mut world = CMessage::new(WORLD_AUCTION_SELF_MESSAGE);
        world.base_mut().add_long(player_id);
        let world_delivery = world.send(game, false);
        tracing::trace!(selector, player_id, ?world_delivery, "собственные лоты запрошены");
        return Some(());
    }
    if selector == CLIENT_AUCTION_RELAY_MESSAGE {
        message.set_message_type(WORLD_AUCTION_RELAY_MESSAGE);
        let world_delivery = message.send(game, false);
        tracing::trace!(selector, player_id, ?world_delivery, "сообщение аукциона передано WorldServer");
        return Some(());
    }
    if selector == CLIENT_AUCTION_CELL_MESSAGE {
        let Some(position) = message.base_mut().get_long() else {
            tracing::warn!(selector, player_id, field = "auction position", "неполное сообщение аукциона");
            return Some(());
        };
        let position = position as u32;
        let Some(goods) = game
            .find_player(player_id)
            .and_then(|player| player.auction_goods_identity_at(position))
        else {
            tracing::trace!(selector, player_id, position, "лот игрока не найден");
            return Some(());
        };
        let mut world = CMessage::new(WORLD_AUCTION_CELL_MESSAGE);
        world.base_mut().add_ulong(position);
        world.base_mut().add_long(player_id);
        world.base_mut().add_guid(goods.ex_id);
        let world_delivery = world.send(game, false);
        tracing::trace!(selector, player_id, position, ?world_delivery, "ячейка аукциона передана WorldServer");
        return Some(());
    }
    if selector == CLIENT_AUCTION_BUY_EXTENSION_MESSAGE {
        let Some(goods_index) = message.base_mut().get_long() else {
            tracing::warn!(selector, player_id, field = "extension goods index", "неполное сообщение аукциона");
            return Some(());
        };
        let Some(amount) = message.base_mut().get_long() else {
            tracing::warn!(selector, player_id, field = "extension goods amount", "неполное сообщение аукциона");
            return Some(());
        };
        let goods_index = goods_index as u32;
        let amount = amount as u32;
        let blocked = |reason: &'static str| {
            tracing::trace!(selector, player_id, goods_index, amount, reason, "покупка расширения аукциона отклонена");
        };
        if amount as i32 <= 0 {
            blocked("некорректное количество");
            return Some(());
        }
        if game
            .find_player(player_id)
            .is_none_or(|player| player.packet().space() < amount)
        {
            blocked("недостаточно места в рюкзаке");
            return Some(());
        }
        let Some(properties) = game.goods_factory().query_goods_base_properties(goods_index) else {
            blocked("не найдены свойства предмета");
            return Some(());
        };
        if !properties.has_enabled_addon_property(GAP_GOODS_PACKAGE_EXTENTION) {
            blocked("нет свойства расширения");
            return Some(());
        }
        let values = properties.get_addon_property_values(GAP_GOODS_PACKAGE_EXTENTION);
        let kind = values.iter().find(|value| value.id == 1).map_or(0, |value| value.base_value);
        if kind != 3 {
            blocked("неверный вид расширения");
            return Some(());
        }
        let price = values.iter().find(|value| value.id == 2).map_or(0, |value| value.base_value) as u32;
        let required_crystals = price.wrapping_mul(amount).wrapping_mul(100);
        let crystal_index = game.goods_factory().query_goods_id_by_original_name(Some(b"FZ0965"));
        if crystal_index == 0 {
            blocked("не найден предмет оплаты");
            return Some(());
        }
        if game.find_player(player_id).is_none_or(|player| {
            player.check_item_in_packet(crystal_index) < required_crystals
        }) {
            blocked("недостаточно кристаллов");
            return Some(());
        }
        let consumptions = game
            .find_player_mut(player_id)
            .expect("player проверен до crystal removal")
            .remove_item_in_packet(crystal_index, required_crystals);
        if consumptions.is_empty() {
            blocked("не удалось списать кристаллы");
            return Some(());
        }
        for consumption in &consumptions {
            let _ = game.send_player_packet_consumption(consumption);
        }

        let created = game.create_goods_batch(goods_index, amount);
        if let Some(first) = created.first() {
            let player = game.find_player(player_id).expect("player существует до extension audit");
            let mut audit = CMessage::new(WORLD_GOODS_AUDIT_MESSAGE);
            audit.base_mut().add_byte(b'n');
            audit.base_mut().add_long(player_id);
            audit.base_mut().add_short(player.pk_count() as i16);
            audit.base_mut().add_ulong(player.money());
            audit.base_mut().add_ulong(player.depot_money());
            audit.base_mut().add_guid(first.identity().ex_id);
            audit.base_mut().add_ulong(first.price());
            audit.base_mut().add(first.name());
            audit.base_mut().add_byte(0);
            audit.base_mut().add_ulong(created.len() as u32);
            audit.base_mut().add_long(player.server_region_id().unwrap_or_default());
            audit.base_mut().add_ulong(player.shape().get_tile_x().unwrap_or_default() as u32);
            audit.base_mut().add_ulong(player.shape().get_tile_y().unwrap_or_default() as u32);
            audit.base_mut().add_ulong(player.client_ip());
            let _ = audit.send(game, false);
        }
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
            .add_goods_to_player_packet(player_id, created, &mut encode)
            .expect("player существует до extension packet add");
        for addition in &additions {
            let _ = game.send_player_packet_addition(addition);
        }
        tracing::trace!(
            selector,
            player_id,
            goods_index,
            amount,
            required_crystals,
            consumptions = consumptions.len(),
            additions = additions.len(),
            rejected = rejected.len(),
            "расширение аукциона куплено"
        );
        return Some(());
    }

    let mut setup = CMessage::new(CLIENT_AUCTION_OPEN_SETUP_MESSAGE);
    for value in game.globe_setup().auction_open_values() {
        setup.base_mut().add_ulong(value);
    }
    let setup_delivery = setup.send_to_player(game.net_server(), player_id);
    game.find_player_mut(player_id)
        .expect("player проверен до open")
        .set_auction_open(true);
    let mut world = CMessage::new(WORLD_AUCTION_OPEN_MESSAGE);
    world.base_mut().add_long(player_id);
    let world_delivery = world.send(game, false);
    tracing::trace!(selector, player_id, setup_delivery, ?world_delivery, "аукцион открыт");
    Some(())
}

fn trace_listing_rejection(
    player_id: i32,
    gate: &Option<AuctionListingGate>,
    reason: &'static str,
    notice_delivery: Option<i32>,
) {
    tracing::trace!(player_id, ?gate, reason, notice_delivery, "выставление лота отклонено");
}

fn dispatch_auction_listing<Runtime, Tick>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    tick_ms: &mut Tick,
    player_id: i32,
) where
    Runtime: GameContainerMessageRuntime,
    Tick: FnMut(&mut Runtime) -> u32,
{
    let had_pending = game
        .find_player(player_id)
        .is_some_and(|player| player.current_auction_node().is_some());
    let mut gate = None;
    if !had_pending {
        let listing_gate = game
            .find_player_mut(player_id)
            .expect("auction listing player проверен")
            .begin_auction_listing(|| tick_ms(runtime));
        gate = Some(listing_gate);
        if !matches!(listing_gate, AuctionListingGate::Ready { .. }) {
            trace_listing_rejection(player_id, &gate, "five-second gate", None);
            return;
        }

        for field in ["client field 0", "client field 1"] {
            if message.base_mut().get_long().is_none() {
                trace_listing_rejection(player_id, &gate, field, None);
                return;
            }
        }
        let Some(seller_money_signed) = message.base_mut().get_long() else {
            trace_listing_rejection(player_id, &gate, "seller money", None);
            return;
        };
        let Some(client_goods_type) = message.base_mut().get_long() else {
            trace_listing_rejection(player_id, &gate, "client goods type", None);
            return;
        };

        let setup = game.globe_setup();
        let price_gate = (f64::from(seller_money_signed)
            * f64::from(setup.auction_factor_c()))
        .trunc() as i32;
        let price_gate = price_gate.max(setup.auction_yuan_fee_minimum().trunc() as i32);
        if seller_money_signed < price_gate {
            trace_listing_rejection(player_id, &gate, "seller money below fee floor", None);
            return;
        }
        let time_setting = setup.auction_time_setting();
        let auction_time = match time_setting {
            0 => 0x3840,
            1 => 0x7080,
            2 => 0x15180,
            other => other.wrapping_mul(0xe10) as u32,
        };
        let listing_fee = auction_yuan_listing_fee(setup, auction_time);
        let (player_maximum, global_maximum) = (
            setup.auction_player_maximum(),
            setup.auction_global_maximum(),
        );
        let (owner_goods_count, global_goods_count) = (
            game.auction_room().owner_goods_count(player_id),
            game.auction_room().goods_count(),
        );
        let extension_bonus = game
            .find_player(player_id)
            .expect("auction listing player проверен перед limit")
            .auction_listing_extension_bonus(game.goods_factory());
        let limit_tick = tick_ms(runtime);
        let allowed = game
            .find_player_mut(player_id)
            .expect("auction listing player проверен перед limit mutation")
            .begin_auction_limit_check(
                limit_tick,
                owner_goods_count,
                global_goods_count,
                player_maximum,
                global_maximum,
                extension_bonus,
            );
        if !allowed {
            let notice_delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM011"))
                    .send_to_player(game.net_server(), player_id);
            trace_listing_rejection(player_id, &gate, "auction limit", Some(notice_delivery));
            return;
        }
        if game
            .find_player(player_id)
            .is_none_or(|player| player.yuan_bao() < listing_fee)
        {
            trace_listing_rejection(player_id, &gate, "insufficient yuan bao listing fee", None);
            return;
        }
        let Some(listing_goods) = game
            .find_player(player_id)
            .and_then(|player| player.auction_listing().get_goods(0))
        else {
            trace_listing_rejection(player_id, &gate, "missing auction listing goods", None);
            return;
        };
        let base_index = listing_goods.base_properties_index();
        if !game.globe_setup().auction_goods_allowed(base_index) {
            trace_listing_rejection(player_id, &gate, "goods not allowed in auction", None);
            return;
        }
        if game
            .auction_room()
            .contains_goods(listing_goods.identity().ex_id)
        {
            let notice_delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM017"))
                    .send_to_player(game.net_server(), player_id);
            trace_listing_rejection(player_id, &gate, "duplicate auction guid", Some(notice_delivery));
            return;
        }

        let mut goods = game
            .find_player_mut(player_id)
            .expect("auction listing player проверен перед delete")
            .take_auction_listing_goods()
            .expect("slot 0 проверен перед exact full delete");
        let Some(properties) = game
            .goods_factory()
            .query_goods_base_properties(goods.base_properties_index())
        else {
            trace_listing_rejection(player_id, &gate, "missing goods base properties after delete", None);
            return;
        };
        let goods_type = match properties.goods_type() {
            GOODS_TYPE_USELESS => 0,
            GOODS_TYPE_CONSUMABLE => 1,
            GOODS_TYPE_EQUIPMENT => properties.equip_place().wrapping_add(1) as u8,
            _ => client_goods_type as u8,
        };
        let level_limit =
            goods.addon_property_value(game.goods_factory(), GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1)
                as u32;
        let _stored = goods.set_addon_property_value_core(GAP_GOODS_AUCTION_SCALE, 1, 1);
        let (account, seller_ip) = {
            let player = game
                .find_player(player_id)
                .expect("auction listing player проверен перед node");
            (player.account().to_vec(), player.client_ip_text())
        };
        let seller_time = game_wall_time_seconds() as u32;
        let end_time = (game_wall_time_seconds() as u32).wrapping_add(auction_time);
        let mut goods_bytes = Vec::new();
        let encoded = goods.serialize(&mut goods_bytes, true);
        debug_assert!(encoded);
        let node = CGoodsNode::from_player_listing(AuctionListingNodeFields {
            account: &account,
            owner_id: player_id as u32,
            auction_time,
            goods_type,
            npc_price: goods.price() as i32,
            amount: goods.amount() as i32,
            guid: goods.identity().ex_id,
            level_limit,
            goods_name: goods.name(),
            base_index: goods.base_properties_index(),
            seller_money: seller_money_signed as u32,
            seller_time,
            seller_ip: &seller_ip,
            end_time,
            goods_bytes,
        });
        let stored = game
            .find_player_mut(player_id)
            .expect("auction listing player проверен перед pending store")
            .set_current_auction_node(node);
        debug_assert!(stored);
    }

    let fresh_request = !had_pending;
    finish_current_auction_listing(game, runtime, player_id, gate, fresh_request);
    if fresh_request {
        let snapshot_delivery = game.send_player_snapshot_update(player_id, runtime);
        tracing::trace!(
            player_id,
            ?snapshot_delivery,
            "свежий запрос аукциона обновил снимок игрока в World"
        );
    }
}

fn auction_yuan_listing_fee(
    setup: &crate::setup::globesetup::GlobeSetupSnapshot,
    auction_time: u32,
) -> u32 {
    let mut fee = (f64::from(auction_time / 0xe10)
        * f64::from(setup.auction_factor_b())
        * f64::from(setup.auction_base_yuan_bao()))
    .trunc() as i32;
    if f64::from(fee) < f64::from(setup.auction_yuan_fee_minimum()) {
        fee = setup.auction_yuan_fee_minimum().trunc() as i32;
    }
    if f64::from(setup.auction_yuan_fee_maximum()) < f64::from(fee) {
        fee = setup.auction_yuan_fee_maximum().trunc() as i32;
    }
    fee as u32
}

fn finish_current_auction_listing<Runtime: GameContainerMessageRuntime>(
    game: &mut CGame,
    runtime: &mut Runtime,
    player_id: i32,
    gate: Option<AuctionListingGate>,
    fresh_request: bool,
) {
    let setup_minimum = game.globe_setup().auction_yuan_fee_minimum();
    if setup_minimum != 0.0 {
        let node = game
            .find_player(player_id)
            .and_then(|player| player.current_auction_node())
            .expect("DoneCurAucNode вызывается только для pending node");
        let fee = auction_yuan_listing_fee(game.globe_setup(), node.auction_time());
        game.find_player_mut(player_id)
            .expect("auction listing player проверен перед billing pending")
            .set_auction_listing_fee(fee);
        // `SendAucAbOpt` помещал order type 0xA1 в process-global g_BillMap.
        // Его producer/consumer и подтверждение отсутствуют в достигнутом
        // runtime; pending node остаётся живым и не получает фиктивный success.
        tracing::trace!(player_id, ?gate, fee, fresh_request, "оплата выставления лота ожидает Billing");
        return;
    }

    let node = game
        .find_player(player_id)
        .and_then(|player| player.current_auction_node())
        .expect("DoneCurAucNode вызывается только для pending node")
        .clone();
    let payload = match node.serialize() {
        Ok(payload) => payload,
        Err(error) => {
            tracing::error!(player_id, ?gate, ?error, fresh_request, "лот не удалось сериализовать");
            return;
        }
    };
    let mut add = CMessage::new(WORLD_AUCTION_ADD_NODE_MESSAGE);
    add.base_mut().add(&payload);
    let add_delivery = add.send(game, false);

    let fee = game
        .find_player(player_id)
        .expect("auction listing player проверен перед sale log")
        .auction_listing_fee();
    let mut sale_log = CMessage::new(WORLD_AUCTION_SALE_LOG_MESSAGE);
    sale_log.base_mut().add_long(player_id);
    sale_log.base_mut().add_ulong(node.base_index());
    sale_log.base_mut().add_guid(node.guid());
    sale_log.base_mut().add_long(node.amount());
    sale_log.base_mut().add_long(node.npc_price());
    sale_log.base_mut().add_ulong(node.auction_time() / 0xe10);
    sale_log.base_mut().add_ulong(fee);
    let sale_log_delivery = sale_log.send(game, false);

    let goods = game
        .decode_auction_goods(node.goods_bytes())
        .expect("только что сериализованный listing goods обязан декодироваться");
    let old_client_payload = game.encode_goods_for_old_client(&goods);
    let mut client = CMessage::new(CLIENT_AUCTION_LIST_RESULT_MESSAGE);
    client.base_mut().add_long(1);
    client.base_mut().add_ulong(node.auction_time());
    client.base_mut().add_ulong(u32::from(node.money_type()));
    client.base_mut().add_ulong(node.seller_money());
    client.base_mut().add_ulong(old_client_payload.len() as u32);
    client.base_mut().add(&old_client_payload);
    let client_delivery = client.send_to_player(game.net_server(), player_id);

    let cleared = game
        .find_player_mut(player_id)
        .expect("auction listing player проверен перед clear")
        .take_current_auction_node();
    debug_assert!(cleared.is_some());
    game.find_player_mut(player_id)
        .expect("auction listing player проверен перед fee clear")
        .set_auction_listing_fee(0);
    let notice_delivery =
        colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM016"))
            .send_to_player(game.net_server(), player_id);
    let snapshot_delivery = game.send_player_snapshot_update(player_id, runtime);
    tracing::trace!(player_id, ?gate, ?add_delivery, ?sale_log_delivery, client_delivery, notice_delivery, ?snapshot_delivery, snapshot_refreshes = 1 + u8::from(fresh_request), "лот выставлен");
}
