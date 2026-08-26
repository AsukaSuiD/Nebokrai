//! Полный входной lifecycle personal shop GameServer.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и owner
//! `server/gameserver/appserver/message/playershopmessage.cpp` подтверждают
//! `0x90201..0x90208`: guards, local seller/buyer session, shadow goods и цены,
//! open/close/enter/exit/end, cash/Billing purchase и client/around/World wire.
//! Rust handler подключён к реальному message FIFO; malformed payload выражен
//! typed error вместо invalid access. Проверка общего state `0x186A4` остаётся
//! явно названной runtime-границей до материализации state owner-а.

use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::region::RegionCellAccessBlock;
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, PersonalShopPurchaseReport, PersonalShopTerminalReport,
    colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::guid::CGuid;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopIgnoreReason {
    MissingPlayerContext,
    MissingPlayer,
    ChangingServer,
    ChangingRegion,
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopOpenOutcome {
    Dead,
    ProgressConflict,
    MissingRegion,
    OutsideStallArea,
    SessionCreationFailed,
    LocalSession {
        session_id: i32,
        plug_id: i32,
        seller_volume: u32,
        seller_extend_id: i32,
        seller_shop_opened: bool,
        seller_name_empty: bool,
        listener_attach: [bool; 2],
        client_delivery: i32,
    },
    WorldRequested,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopMessageReport {
    Ignored {
        player_id: Option<i32>,
        reason: PlayerShopIgnoreReason,
    },
    Opened {
        player_id: i32,
        source_world_server_id: i32,
        outcome: PlayerShopOpenOutcome,
        notice_delivery: Option<i32>,
        world_request: Option<Result<i32, SendMessageError>>,
        world_completion: Option<Result<i32, SendMessageError>>,
    },
    Action {
        player_id: i32,
        outcome: PlayerShopActionOutcome,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopActionOutcome {
    Ignored,
    Notice {
        string_id: &'static [u8],
        delivery: i32,
    },
    PriceSet {
        session_id: i32,
        plug_id: i32,
        goods_id: CGuid,
        delivery: i32,
    },
    Purchased(PersonalShopPurchaseReport),
    BusinessStarted {
        session_id: i32,
        plug_id: i32,
        name: Vec<u8>,
        delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
    BusinessClosed {
        session_id: i32,
        plug_id: i32,
        delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
    BuyerEntered {
        session_id: i32,
        buyer_plug_id: i32,
        goods_count: u32,
        delivery: i32,
    },
    SessionEnded(PersonalShopTerminalReport),
    BuyerExited(PersonalShopTerminalReport),
}

pub(crate) fn dispatch_player_shop_message<Context: GameContainerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<PlayerShopMessageReport, PlayerShopMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        PLAYER_SHOP_OPEN_MESSAGE..=PLAYER_SHOP_EXIT_MESSAGE
    ) {
        return None;
    }

    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        return Some(Ok(PlayerShopMessageReport::Ignored {
            player_id: None,
            reason: PlayerShopIgnoreReason::MissingPlayerContext,
        }));
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(Ok(PlayerShopMessageReport::Ignored {
            player_id: Some(player_id),
            reason: PlayerShopIgnoreReason::MissingPlayer,
        }));
    };
    if player.in_changing_server() || player.in_changing_region() {
        return Some(Ok(PlayerShopMessageReport::Ignored {
            player_id: Some(player_id),
            reason: if player.in_changing_server() {
                PlayerShopIgnoreReason::ChangingServer
            } else {
                PlayerShopIgnoreReason::ChangingRegion
            },
        }));
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
) -> Result<PlayerShopMessageReport, PlayerShopMessageError> {
    let outcome = match message_type {
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
                PlayerShopActionOutcome::Ignored
            } else if game
                .session_factory()
                .personal_shop_seller(plug_id)
                .is_some_and(|seller| seller.shop_opened())
            {
                action_notice(game, player_id, b"GS0075")
            } else if !personal_shop_goods_live(game, plug_id, goods_id) {
                action_notice(game, player_id, b"GS0074")
            } else {
                let stored = game
                    .session_factory_mut()
                    .personal_shop_seller_mut(plug_id)
                    .expect("seller ownership проверен")
                    .set_goods_price(goods_id, price_type, price);
                if !stored {
                    action_notice(game, player_id, b"GS0074")
                } else {
                    let mut response = CMessage::new(CLIENT_PLAYER_SHOP_PRICE_MESSAGE);
                    response.add_long(session_id);
                    response.add_long(plug_id);
                    response.base_mut().add_guid(goods_id);
                    response.add_ulong(price_type);
                    response.add_ulong(price);
                    PlayerShopActionOutcome::PriceSet {
                        session_id,
                        plug_id,
                        goods_id,
                        delivery: response.send_to_player(game.net_server(), player_id),
                    }
                }
            }
        }
        PLAYER_SHOP_PURCHASE_MESSAGE => {
            let (session_id, buyer_plug_id) = read_session_plug(message)?;
            let goods_id = message
                .base_mut()
                .get_guid()
                .ok_or(PlayerShopMessageError::MissingGoodsId)?;
            if !game.personal_shop_session_available(session_id, context) {
                action_notice(game, player_id, b"GS0078")
            } else if !buyer_owned_by(game, session_id, buyer_plug_id, player_id) {
                PlayerShopActionOutcome::Ignored
            } else {
                PlayerShopActionOutcome::Purchased(game.purchase_personal_shop_goods(
                    session_id,
                    buyer_plug_id,
                    goods_id,
                    context,
                ))
            }
        }
        PLAYER_SHOP_START_BUSINESS_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            let mut name = message
                .base_mut()
                .get_str_bytes(0x200)
                .ok_or(PlayerShopMessageError::MissingShopName)?;
            if name.len() >= 0x12 {
                action_notice(game, player_id, b"GS0076")
            } else if !seller_owned_by(game, session_id, plug_id, player_id)
                || game
                    .session_factory()
                    .personal_shop_seller(plug_id)
                    .is_none_or(|seller| seller.shop_opened())
            {
                PlayerShopActionOutcome::Ignored
            } else if !game
                .words_filter()
                .check_with_numeric_gate(&mut name, false, true)
            {
                action_notice(game, player_id, b"GS0334")
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
                let delivery = opened
                    .then(|| {
                        let mut response = CMessage::new(CLIENT_PLAYER_SHOP_STARTED_MESSAGE);
                        response.add_long(player_id);
                        response.add_long(session_id);
                        response.add_long(plug_id);
                        add_c_string(&mut response, &name);
                        game.send_player_shape_around(player_id, None, &response)
                    })
                    .flatten();
                PlayerShopActionOutcome::BusinessStarted {
                    session_id,
                    plug_id,
                    name,
                    delivery,
                }
            }
        }
        PLAYER_SHOP_CLOSE_BUSINESS_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            if !seller_owned_by(game, session_id, plug_id, player_id) {
                PlayerShopActionOutcome::Ignored
            } else {
                let was_open = game
                    .session_factory()
                    .personal_shop_seller(plug_id)
                    .is_some_and(|seller| seller.shop_opened());
                if !was_open {
                    PlayerShopActionOutcome::Ignored
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
                    PlayerShopActionOutcome::BusinessClosed {
                        session_id,
                        plug_id,
                        delivery: game.send_player_shape_around(player_id, None, &response),
                    }
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
                || !game.personal_shop_session_available(session_id, context)
            {
                PlayerShopActionOutcome::Ignored
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
                PlayerShopActionOutcome::BuyerEntered {
                    session_id,
                    buyer_plug_id,
                    goods_count,
                    delivery: response.send_to_player(game.net_server(), player_id),
                }
            } else {
                action_notice(game, player_id, b"GS0077")
            }
        }
        PLAYER_SHOP_END_SESSION_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            if seller_owned_by(game, session_id, plug_id, player_id)
                && game.personal_shop_session_available(session_id, context)
            {
                PlayerShopActionOutcome::SessionEnded(game.finish_personal_shop_session(session_id))
            } else {
                PlayerShopActionOutcome::Ignored
            }
        }
        PLAYER_SHOP_EXIT_MESSAGE => {
            let (session_id, plug_id) = read_session_plug(message)?;
            if buyer_owned_by(game, session_id, plug_id, player_id)
                && game.personal_shop_session_available(session_id, context)
            {
                PlayerShopActionOutcome::BuyerExited(
                    game.exit_personal_shop_buyer(session_id, plug_id),
                )
            } else {
                PlayerShopActionOutcome::Ignored
            }
        }
        _ => unreachable!(),
    };
    Ok(PlayerShopMessageReport::Action { player_id, outcome })
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
    context: &mut Context,
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
            Some((position, context.encode_goods_for_old_client(goods), *price))
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
) -> PlayerShopActionOutcome {
    PlayerShopActionOutcome::Notice {
        string_id,
        delivery: colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id))
            .send_to_player(game.net_server(), player_id),
    }
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
) -> Result<PlayerShopMessageReport, PlayerShopMessageError> {
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

    let mut notice_delivery = None;
    let mut world_request = None;
    let mut world_completion = None;
    let outcome = if dead {
        PlayerShopOpenOutcome::Dead
    } else if progress != PlayerProgress::None {
        notice_delivery = Some(
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0072"))
                .send_to_player(game.net_server(), player_id),
        );
        PlayerShopOpenOutcome::ProgressConflict
    } else if let Some(region_id) = region_id {
        let Some(region) = game.find_region(region_id) else {
            if source_world_server_id == world_server_id {
                world_completion = Some(send_world_shop_completion(game, player_id));
            }
            return Ok(PlayerShopMessageReport::Opened {
                player_id,
                source_world_server_id,
                outcome: PlayerShopOpenOutcome::MissingRegion,
                notice_delivery,
                world_request,
                world_completion,
            });
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
            notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0073"))
                    .send_to_player(game.net_server(), player_id),
            );
            if source_world_server_id == world_server_id {
                world_completion = Some(send_world_shop_completion(game, player_id));
            }
            return Ok(PlayerShopMessageReport::Opened {
                player_id,
                source_world_server_id,
                outcome: PlayerShopOpenOutcome::OutsideStallArea,
                notice_delivery,
                world_request,
                world_completion,
            });
        }

        if !game.globe_setup().auction_enabled() || source_world_server_id != 0 {
            let Some((session_id, plug_id)) = game
                .session_factory_mut()
                .create_personal_shop_seller_session(player_id)
            else {
                return Ok(PlayerShopMessageReport::Opened {
                    player_id,
                    source_world_server_id,
                    outcome: PlayerShopOpenOutcome::SessionCreationFailed,
                    notice_delivery,
                    world_request,
                    world_completion,
                });
            };
            let seller = game
                .session_factory()
                .personal_shop_seller(plug_id)
                .expect("personal-shop factory публикует typed seller");
            let seller_volume = seller.goods().size();
            let seller_extend_id = seller.goods().base().base().container_extend_id();
            let seller_shop_opened = seller.shop_opened();
            let seller_name_empty = seller.shop_name().is_empty();
            let listener_attach = {
                let player = game
                    .find_player_mut(player_id)
                    .expect("player жив во время session insertion");
                let listener_attach = player.attach_equipment_session_listener(plug_id);
                player.set_current_progress_snapshot(PlayerProgress::OpenStall);
                listener_attach
            };
            let mut response = CMessage::new(CLIENT_PLAYER_SHOP_OPENED_MESSAGE);
            response.base_mut().add_long(session_id);
            response.base_mut().add_long(plug_id);
            let client_delivery = response.send_to_player(game.net_server(), player_id);
            PlayerShopOpenOutcome::LocalSession {
                session_id,
                plug_id,
                seller_volume,
                seller_extend_id,
                seller_shop_opened,
                seller_name_empty,
                listener_attach,
                client_delivery,
            }
        } else {
            let mut request = CMessage::new(WORLD_PLAYER_SHOP_REQUEST_MESSAGE);
            request.base_mut().add_long(player_id);
            request.base_mut().add_ulong(message.ip());
            world_request = Some(request.send(game, false));
            PlayerShopOpenOutcome::WorldRequested
        }
    } else {
        PlayerShopOpenOutcome::MissingRegion
    };

    if matches!(
        outcome,
        PlayerShopOpenOutcome::Dead
            | PlayerShopOpenOutcome::ProgressConflict
            | PlayerShopOpenOutcome::MissingRegion
    ) && source_world_server_id == world_server_id
    {
        world_completion = Some(send_world_shop_completion(game, player_id));
    }
    Ok(PlayerShopMessageReport::Opened {
        player_id,
        source_world_server_id,
        outcome,
        notice_delivery,
        world_request,
        world_completion,
    })
}

fn send_world_shop_completion(game: &CGame, player_id: i32) -> Result<i32, SendMessageError> {
    let client_ip = game
        .find_player(player_id)
        .expect("completion относится к live player")
        .client_ip();
    let mut completion = CMessage::new(WORLD_PLAYER_SHOP_OVER_MESSAGE);
    completion.base_mut().add_long(player_id);
    completion.base_mut().add_ulong(client_ip);
    completion.send(game, false)
}
