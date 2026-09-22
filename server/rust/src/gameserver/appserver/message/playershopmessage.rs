//! Полный входной жизненный цикл личной лавки GameServer.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и owner
//! `server/gameserver/appserver/message/playershopmessage.cpp` подтверждают
//! `0x90201..0x90208`: проверки, локальные сессии продавца и покупателя,
//! теневые товары и цены, открытие/закрытие/вход/выход/завершение, покупку за
//! наличные или через Billing и wire-обмен с клиентом, окружением и World.
//! Обработчик подключён к реальному FIFO сообщений; повреждённый payload
//! выражен типизированной ошибкой вместо недопустимого доступа. Проверка общего
//! состояния `0x186A4` остаётся явно названной границей исполнения. Уже
//! выполненные эффекты не дублируются отчётами и диагностируются через
//! `tracing`.

use crate::gameserver::appserver::player::PlayerProgress;
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::region::RegionCellAccessBlock;
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;
use nebokrai_shared::values::CGuid;
use tracing::{debug, trace};

const PLAYER_SHOP_OPEN_MESSAGE: i32 = 0x0009_0201;
const PLAYER_SHOP_SET_PRICE_MESSAGE: i32 = 0x0009_0202;
const PLAYER_SHOP_PURCHASE_MESSAGE: i32 = 0x0009_0203;
const PLAYER_SHOP_START_BUSINESS_MESSAGE: i32 = 0x0009_0204;
const PLAYER_SHOP_CLOSE_BUSINESS_MESSAGE: i32 = 0x0009_0205;
const PLAYER_SHOP_ENTER_MESSAGE: i32 = 0x0009_0206;
const PLAYER_SHOP_END_SESSION_MESSAGE: i32 = 0x0009_0207;
const PLAYER_SHOP_EXIT_MESSAGE: i32 = 0x0009_0208;
const CLIENT_PLAYER_SHOP_OPENED_MESSAGE: i32 = 0x000c_0001;
const CLIENT_PLAYER_SHOP_PRICE_MESSAGE: i32 = 0x000c_0003;
const CLIENT_PLAYER_SHOP_STARTED_MESSAGE: i32 = 0x000c_0004;
const CLIENT_PLAYER_SHOP_CLOSED_MESSAGE: i32 = 0x000c_0005;
const CLIENT_PLAYER_SHOP_ENTERED_MESSAGE: i32 = 0x000c_0006;
const WORLD_PLAYER_SHOP_REQUEST_MESSAGE: i32 = 0x0006_0811;
const WORLD_PLAYER_SHOP_OVER_MESSAGE: i32 = 0x0006_0812;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopMessageError {
    MissingSourceWorldServerId,
    MissingSessionId,
    MissingPlugId,
    MissingGoodsId,
    MissingPriceType,
    MissingPrice,
    MissingShopName,
    Coordinate(ShapeCoordinateBlock),
    RegionCell(RegionCellAccessBlock),
}

pub(crate) fn dispatch_player_shop_message<Context: GameContainerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<(), PlayerShopMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        PLAYER_SHOP_OPEN_MESSAGE..=PLAYER_SHOP_EXIT_MESSAGE
    ) {
        return None;
    }

    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        trace!(message_type, "сообщение личной лавки пропущено: нет контекста игрока");
        return Some(Ok(()));
    };
    let Some(player) = game.find_player(player_id) else {
        trace!(message_type, player_id, "сообщение личной лавки пропущено: игрок не найден");
        return Some(Ok(()));
    };
    if player.in_changing_server() || player.in_changing_region() {
        trace!(message_type, player_id, changing_server = player.in_changing_server(), changing_region = player.in_changing_region(), "сообщение личной лавки пропущено во время перехода");
        return Some(Ok(()));
    }

    if message_type == PLAYER_SHOP_OPEN_MESSAGE {
        let Some(source_world_server_id) = message.base_mut().get_long() else {
            return Some(Err(PlayerShopMessageError::MissingSourceWorldServerId));
        };
        return Some(open_player_shop(
            message,
            game,
            player_id,
            source_world_server_id,
        ));
    }
    Some(dispatch_player_shop_action(
        message_type,
        message,
        game,
        context,
        player_id,
    ))
}

fn dispatch_player_shop_action<Context: GameContainerMessageRuntime>(
    message_type: i32,
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
    player_id: i32,
) -> Result<(), PlayerShopMessageError> {
    match message_type {
        PLAYER_SHOP_SET_PRICE_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            let goods_id = message
                .base_mut()
                .get_guid()
                .ok_or(PlayerShopMessageError::MissingGoodsId)?;
            let price_type = message
                .base_mut()
                .get_long()
                .map(|value| value as u32)
                .ok_or(PlayerShopMessageError::MissingPriceType)?;
            let price = message
                .base_mut()
                .get_long()
                .map(|value| value as u32)
                .ok_or(PlayerShopMessageError::MissingPrice)?;
            if !seller_owned_by(game, session_id, plug_id, player_id) {
                trace!(player_id, session_id, plug_id, "изменение цены отклонено владельцем сессии");
            } else if game
                .session_factory()
                .personal_shop_seller(plug_id)
                .is_some_and(|seller| seller.shop_opened())
            {
                action_notice(game, player_id, b"GS0075");
            } else if !personal_shop_goods_live(game, plug_id, goods_id) {
                action_notice(game, player_id, b"GS0074");
            } else {
                let stored = game
                    .session_factory_mut()
                    .personal_shop_seller_mut(plug_id)
                    .expect("seller ownership проверен")
                    .set_goods_price(goods_id, price_type, price);
                if !stored {
                    action_notice(game, player_id, b"GS0074");
                } else {
                    let mut response = CMessage::new(CLIENT_PLAYER_SHOP_PRICE_MESSAGE);
                    response.add_long(session_id);
                    response.add_long(plug_id);
                    response.base_mut().add_guid(goods_id);
                    response.add_ulong(price_type);
                    response.add_ulong(price);
                    let _ = response.send_to_player(game.net_server(), player_id);
                    debug!(player_id, session_id, plug_id, price_type, price, "цена товара личной лавки изменена");
                }
            }
        }
        PLAYER_SHOP_PURCHASE_MESSAGE => {
            let (session_id, buyer_plug_id) = read_session_plug(message)?;
            let goods_id = message
                .base_mut()
                .get_guid()
                .ok_or(PlayerShopMessageError::MissingGoodsId)?;
            if !game.personal_shop_session_available(session_id) {
                action_notice(game, player_id, b"GS0078");
            } else if !buyer_owned_by(game, session_id, buyer_plug_id, player_id) {
                trace!(player_id, session_id, buyer_plug_id, "покупка отклонена владельцем сессии");
            } else {
                game.purchase_personal_shop_goods(
                    session_id,
                    buyer_plug_id,
                    goods_id,
                    context,
                );
                debug!(player_id, session_id, buyer_plug_id, "обработана покупка в личной лавке");
            }
        }
        PLAYER_SHOP_START_BUSINESS_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            let mut name = message
                .base_mut()
                .get_str_bytes(0x200)
                .ok_or(PlayerShopMessageError::MissingShopName)?;
            if name.len() >= 0x12 {
                action_notice(game, player_id, b"GS0076");
            } else if !seller_owned_by(game, session_id, plug_id, player_id)
                || game
                    .session_factory()
                    .personal_shop_seller(plug_id)
                    .is_none_or(|seller| seller.shop_opened())
            {
                trace!(player_id, session_id, plug_id, "открытие торговли отклонено состоянием сессии");
            } else if !game
                .words_filter()
                .check_with_numeric_gate(&mut name, false, true)
            {
                action_notice(game, player_id, b"GS0334");
            } else {
                let seller = game
                    .session_factory_mut()
                    .personal_shop_seller_mut(plug_id)
                    .expect("seller ownership проверен");
                seller.set_shop_name(&name);
                let opened = seller.open_for_business();
                game.find_player_mut(player_id)
                    .expect("seller player проверен")
                    .set_personal_shop_flag(session_id, plug_id);
                let _ = opened
                    .then(|| {
                        let mut response = CMessage::new(CLIENT_PLAYER_SHOP_STARTED_MESSAGE);
                        response.add_long(player_id);
                        response.add_long(session_id);
                        response.add_long(plug_id);
                        add_c_string(&mut response, &name);
                        game.send_player_shape_around(player_id, None, &response)
                    })
                    .flatten();
                debug!(player_id, session_id, plug_id, opened, "личная лавка открыта для торговли");
            }
        }
        PLAYER_SHOP_CLOSE_BUSINESS_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            if !seller_owned_by(game, session_id, plug_id, player_id) {
                trace!(player_id, session_id, plug_id, "закрытие торговли отклонено владельцем сессии");
            } else {
                let was_open = game
                    .session_factory()
                    .personal_shop_seller(plug_id)
                    .is_some_and(|seller| seller.shop_opened());
                if !was_open {
                    trace!(player_id, session_id, plug_id, "личная лавка уже закрыта");
                } else {
                    game.session_factory_mut()
                        .personal_shop_seller_mut(plug_id)
                        .expect("seller ownership проверен")
                        .close_down();
                    game.find_player_mut(player_id)
                        .expect("seller player проверен")
                        .set_personal_shop_flag(0, 0);
                    let mut response = CMessage::new(CLIENT_PLAYER_SHOP_CLOSED_MESSAGE);
                    response.add_long(player_id);
                    response.add_long(session_id);
                    response.add_long(plug_id);
                    let _ = game.send_player_shape_around(player_id, None, &response);
                    debug!(player_id, session_id, plug_id, "торговля личной лавки закрыта");
                }
            }
        }
        PLAYER_SHOP_ENTER_MESSAGE => {
            let session_id = message
                .base_mut()
                .get_long()
                .ok_or(PlayerShopMessageError::MissingSessionId)?;
            if game
                .find_player(player_id)
                .is_none_or(|player| player.current_progress() != PlayerProgress::None)
                || !game.personal_shop_session_available(session_id)
            {
                trace!(player_id, session_id, "вход в личную лавку отклонён состоянием игрока или сессии");
            } else if let Some(buyer_plug_id) = game
                .session_factory_mut()
                .insert_personal_shop_buyer(session_id, player_id)
            {
                game.find_player_mut(player_id)
                    .expect("buyer player проверен")
                    .set_current_progress_snapshot(PlayerProgress::Shopping);
                let goods = personal_shop_goods_list(game, session_id, context);
                let goods_count = goods
                    .get(..4)
                    .and_then(|bytes| LegacyReader::new(bytes).read_u32().ok())
                    .unwrap_or(0);
                let mut response = CMessage::new(CLIENT_PLAYER_SHOP_ENTERED_MESSAGE);
                response.add_long(session_id);
                response.add_long(buyer_plug_id);
                response.base_mut().add(&goods);
                let _ = response.send_to_player(game.net_server(), player_id);
                debug!(player_id, session_id, buyer_plug_id, goods_count, "покупатель вошёл в личную лавку");
            } else {
                action_notice(game, player_id, b"GS0077");
            }
        }
        PLAYER_SHOP_END_SESSION_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            if seller_owned_by(game, session_id, plug_id, player_id)
                && game.personal_shop_session_available(session_id)
            {
                game.finish_personal_shop_session(session_id);
                debug!(player_id, session_id, plug_id, "сессия личной лавки завершена продавцом");
            } else {
                trace!(player_id, session_id, plug_id, "завершение личной лавки отклонено");
            }
        }
        PLAYER_SHOP_EXIT_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            if buyer_owned_by(game, session_id, plug_id, player_id)
                && game.personal_shop_session_available(session_id)
            {
                game.exit_personal_shop_buyer(session_id, plug_id);
                debug!(player_id, session_id, plug_id, "покупатель вышел из личной лавки");
            } else {
                trace!(player_id, session_id, plug_id, "выход из личной лавки отклонён");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn read_session_plug(message: &mut CMessage) -> Result<(i32, i32), PlayerShopMessageError> {
    let session_id = message
        .base_mut()
        .get_long()
        .ok_or(PlayerShopMessageError::MissingSessionId)?;
    let plug_id = message
        .base_mut()
        .get_long()
        .ok_or(PlayerShopMessageError::MissingPlugId)?;
    Ok((session_id, plug_id))
}

fn seller_owned_by(game: &CGame, session_id: i32, plug_id: i32, player_id: i32) -> bool {
    game.session_factory()
        .query_plug(plug_id)
        .is_some_and(|plug| plug.session_id() == session_id && plug.has_owner(400, player_id))
        && game
            .session_factory()
            .personal_shop_seller(plug_id)
            .is_some()
}

fn buyer_owned_by(game: &CGame, session_id: i32, plug_id: i32, player_id: i32) -> bool {
    game.session_factory()
        .personal_shop_buyer(plug_id)
        .is_some_and(|buyer| buyer.session_id() == session_id && buyer.owner_id() == player_id)
}

fn personal_shop_goods_live(game: &CGame, seller_plug_id: i32, goods_id: CGuid) -> bool {
    let Some(previous) = game
        .session_factory()
        .personal_shop_seller(seller_plug_id)
        .and_then(|seller| {
            seller
                .goods()
                .base()
                .base()
                .original_container_information(goods_id)
        })
    else {
        return false;
    };
    game.find_player(previous.container_id)
        .and_then(|player| {
            player.trade_source_goods(
                previous.container_extend_id,
                previous.goods_position,
                goods_id,
            )
        })
        .is_some()
}

fn personal_shop_goods_list<Context: GameContainerMessageRuntime>(
    game: &CGame,
    session_id: i32,
    _context: &mut Context,
) -> Vec<u8> {
    let Some(seller_plug_id) = game
        .session_factory()
        .personal_shop_seller_plug_id(session_id)
    else {
        return Vec::new();
    };
    let Some(seller) = game
        .session_factory()
        .personal_shop_seller(seller_plug_id)
        .filter(|seller| seller.shop_opened())
    else {
        return Vec::new();
    };
    let entries: Vec<_> = seller
        .prices()
        .iter()
        .filter_map(|(goods_id, price)| {
            let previous = seller
                .goods()
                .base()
                .base()
                .original_container_information(*goods_id)?;
            let position = seller.goods().query_goods_position(*goods_id)?;
            let goods = game
                .find_player(previous.container_id)?
                .trade_source_goods(
                    previous.container_extend_id,
                    previous.goods_position,
                    *goods_id,
                )?;
            Some((position, game.encode_goods_for_old_client(goods), *price))
        })
        .collect();
    let mut wire = Vec::new();
    LegacyWriter::new(&mut wire).write_u32(entries.len() as u32);
    for (position, goods, price) in entries {
        let mut writer = LegacyWriter::new(&mut wire);
        writer.write_u32(position);
        writer.write_bytes(&goods);
        writer.write_u32(price.price_type);
        writer.write_u32(price.price);
    }
    wire
}

fn action_notice(
    game: &CGame,
    player_id: i32,
    string_id: &'static [u8],
) {
    let _ = colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id))
        .send_to_player(game.net_server(), player_id);
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(value);
    message.add_byte(0);
}

fn open_player_shop(
    message: &CMessage,
    game: &mut CGame,
    player_id: i32,
    source_world_server_id: i32,
) -> Result<(), PlayerShopMessageError> {
    let (_, world_server_id) = game.server_ids();
    let (dead, progress, region_id) = {
        let player = game
            .find_player(player_id)
            .expect("player проверен до открытия personal shop");
        (
            player.is_dead(),
            player.current_progress(),
            message.region_id(),
        )
    };

    if dead {
        if source_world_server_id == world_server_id {
            send_world_shop_completion(game, player_id);
        }
        trace!(player_id, source_world_server_id, "открытие личной лавки отклонено: игрок мёртв");
        return Ok(());
    }
    if progress != PlayerProgress::None {
        action_notice(game, player_id, b"GS0072");
        if source_world_server_id == world_server_id {
            send_world_shop_completion(game, player_id);
        }
        trace!(player_id, source_world_server_id, ?progress, "открытие личной лавки отклонено текущим процессом");
        return Ok(());
    }
    let Some(region_id) = region_id else {
        if source_world_server_id == world_server_id {
            send_world_shop_completion(game, player_id);
        }
        trace!(player_id, source_world_server_id, "открытие личной лавки отклонено: нет региона");
        return Ok(());
    };
    let Some(region) = game.find_region(region_id) else {
        if source_world_server_id == world_server_id {
            send_world_shop_completion(game, player_id);
        }
        trace!(player_id, source_world_server_id, region_id, "открытие личной лавки отклонено: регион не найден");
        return Ok(());
    };
    let (tile_x, tile_y) = {
        let player = game
            .find_player(player_id)
            .expect("player проверен перед region lookup");
        (
            player
                .shape()
                .get_tile_x()
                .map_err(PlayerShopMessageError::Coordinate)?,
            player
                .shape()
                .get_tile_y()
                .map_err(PlayerShopMessageError::Coordinate)?,
        )
    };
    let block = region
        .base()
        .region
        .get_block(tile_x, tile_y)
        .map_err(PlayerShopMessageError::RegionCell)?;
    if block != 2 {
        action_notice(game, player_id, b"GS0073");
        if source_world_server_id == world_server_id {
            send_world_shop_completion(game, player_id);
        }
        trace!(player_id, source_world_server_id, region_id, block, "игрок находится вне зоны личных лавок");
        return Ok(());
    }

    if !game.globe_setup().auction_enabled() || source_world_server_id != 0 {
        let now_ms = game.current_tick_ms();
        let Some((session_id, plug_id)) = game
            .session_factory_mut()
            .create_personal_shop_seller_session(now_ms, player_id)
        else {
            trace!(player_id, source_world_server_id, "сессию продавца личной лавки создать не удалось");
            return Ok(());
        };
        {
            let player = game
                .find_player_mut(player_id)
                .expect("player жив во время session insertion");
            let _ = player.attach_equipment_session_listener(plug_id);
            player.set_current_progress_snapshot(PlayerProgress::OpenStall);
        }
        let mut response = CMessage::new(CLIENT_PLAYER_SHOP_OPENED_MESSAGE);
        response.base_mut().add_long(session_id);
        response.base_mut().add_long(plug_id);
        let _ = response.send_to_player(game.net_server(), player_id);
        debug!(player_id, source_world_server_id, session_id, plug_id, "создана локальная сессия личной лавки");
    } else {
        let mut request = CMessage::new(WORLD_PLAYER_SHOP_REQUEST_MESSAGE);
        request.base_mut().add_long(player_id);
        request.base_mut().add_ulong(message.ip());
        let _ = request.send(game, false);
        debug!(player_id, source_world_server_id, "запрошена World-сессия личной лавки");
    }
    Ok(())
}

fn send_world_shop_completion(game: &CGame, player_id: i32) {
    let client_ip = game
        .find_player(player_id)
        .expect("completion относится к live player")
        .client_ip();
    let mut completion = CMessage::new(WORLD_PLAYER_SHOP_OVER_MESSAGE);
    completion.base_mut().add_long(player_id);
    completion.base_mut().add_ulong(client_ip);
    let _ = completion.send(game, false);
}
