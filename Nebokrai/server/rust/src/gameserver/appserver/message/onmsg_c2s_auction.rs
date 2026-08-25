//! Клиентские lifecycle-входы аукциона GameServer.
//!
//! Точная пара GameServer EXE/PDB и исходный owner
//! `server/gameserver/appserver/message/onmsg_c2s_auction.cpp` подтверждают
//! close `0x90A01`, listing `0x90A02`, cut `0x90A03`, buy `0x90A04` и auction controls
//! `0x90A05..0C`. Listing принимает временный goods, применяет auction-scale,
//! сериализует полный `CGoodsNode`, отправляет World `0x60801`, sale-log
//! `0x60215`, client-result `0xC0701` и notice; ненулевой YuanBao fee сохраняет
//! pending node у конкретной недоступной границы process-global `g_BillMap`.
//! Полный persisted-player snapshot `0x6080E` остаётся у этой же недоступной
//! runtime-границы. Cut
//! сначала посылает exact `0x60216`, затем переписывает исходный wire в
//! `0x60809`; buy сохраняет signed client YuanBao precheck, общий 5-секундный
//! gate и только для живого GUID посылает `0x60805`; поиск сохраняет player
//! criteria и сбрасывает page, browse/self
//! запросы уходят в World,
//! клиент открытия получает `0xC0706`, player-open меняется до World `0x60810`,
//! extension batch оплачивается `FZ0965`, логируется и добавляется в packet,
//! а выключенный аукцион возвращает `GPM013` до чтения payload.
//! Остальные gameplay selectors остаются RAW ниже.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GOODS_PACKAGE_EXTENTION;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::{
    GAP_GOODS_AUCTION_SCALE, GAP_ROLE_MINIMUM_LEVEL_LIMIT, GOODS_TYPE_CONSUMABLE,
    GOODS_TYPE_EQUIPMENT, GOODS_TYPE_USELESS,
};
use crate::gameserver::appserver::message::unibillmessage::IncrementShopBillingContext;
use crate::gameserver::appserver::player::{
    AuctionBuyGate, AuctionListingGate, AuctionSelfGoodsRefresh, CiQingPacketAddition,
    CiQingPacketConsumption,
};
use crate::gameserver::gameserver::game::{
    CGame, OldClientGoodsCodec, colored_player_notice_message, game_wall_time_seconds,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::auctionnode::{AuctionListingNodeFields, CGoodsNode, GoodsNodeSerializeError};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionExtensionBuyBlock {
    InvalidAmount,
    InsufficientPacketSpace,
    MissingGoodsProperties,
    WrongExtensionKind,
    MissingCrystalGoods,
    InsufficientCrystals,
    CrystalRemovalFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuctionExtensionBuyReport {
    pub(crate) player_id: i32,
    pub(crate) goods_index: u32,
    pub(crate) amount: u32,
    pub(crate) required_crystals: u32,
    pub(crate) consumptions: Vec<CiQingPacketConsumption>,
    pub(crate) consumption_deliveries: Vec<Vec<i32>>,
    pub(crate) additions: Vec<CiQingPacketAddition>,
    pub(crate) addition_deliveries: Vec<Vec<i32>>,
    pub(crate) rejected_goods: Vec<crate::gameserver::appserver::shape::ShapeIdentity>,
    pub(crate) audit_delivery: Option<Result<i32, SendMessageError>>,
    pub(crate) block: Option<AuctionExtensionBuyBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ClientAuctionMessageReport {
    MissingPlayer {
        selector: i32,
    },
    Closed {
        player_id: i32,
    },
    Listed {
        player_id: i32,
        gate: Option<AuctionListingGate>,
        world_deliveries: Vec<Result<i32, SendMessageError>>,
        client_deliveries: Vec<i32>,
        snapshot_refreshes: u8,
    },
    ListingRejected {
        player_id: i32,
        gate: Option<AuctionListingGate>,
        reason: &'static str,
        notice_delivery: Option<i32>,
        snapshot_refreshes: u8,
    },
    ListingBillingPending {
        player_id: i32,
        gate: Option<AuctionListingGate>,
        fee: u32,
        snapshot_refreshes: u8,
    },
    ListingSerializeBlocked {
        player_id: i32,
        gate: Option<AuctionListingGate>,
        error: GoodsNodeSerializeError,
        snapshot_refreshes: u8,
    },
    Disabled {
        player_id: i32,
        notice_delivery: i32,
    },
    Opened {
        player_id: i32,
        setup_delivery: i32,
        world_delivery: Result<i32, SendMessageError>,
    },
    Forwarded {
        selector: i32,
        player_id: i32,
        world_selector: i32,
        world_delivery: Result<i32, SendMessageError>,
    },
    CutForwarded {
        player_id: i32,
        goods_id: crate::public::guid::CGuid,
        cut_log_delivery: Option<Result<i32, SendMessageError>>,
        world_delivery: Result<i32, SendMessageError>,
    },
    BuyRequested {
        player_id: i32,
        advertised_yuan_bao: i32,
        goods_id: crate::public::guid::CGuid,
        gate: AuctionBuyGate,
        world_delivery: Option<Result<i32, SendMessageError>>,
    },
    BuyFundsRejected {
        player_id: i32,
        advertised_yuan_bao: i32,
        available_yuan_bao: u32,
        notice_delivery: i32,
    },
    Truncated {
        selector: i32,
        field: &'static str,
    },
    Refreshed {
        player_id: i32,
        refresh: AuctionSelfGoodsRefresh,
        world_delivery: Option<Result<i32, SendMessageError>>,
    },
    GoodsMissing {
        player_id: i32,
        position: u32,
    },
    ExtensionBought(AuctionExtensionBuyReport),
}

pub(crate) fn dispatch_client_auction_message<Runtime, Tick>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    mut tick_ms: Tick,
) -> Option<ClientAuctionMessageReport>
where
    Runtime: OldClientGoodsCodec + IncrementShopBillingContext,
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
    ) {
        return None;
    }
    let Some(player_id) = message.player_id() else {
        return Some(ClientAuctionMessageReport::MissingPlayer { selector });
    };
    if game.find_player(player_id).is_none() {
        return Some(ClientAuctionMessageReport::MissingPlayer { selector });
    }
    if !game.auction_now() {
        let notice_delivery =
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM013"))
                .send_to_player(game.net_server(), player_id);
        return Some(ClientAuctionMessageReport::Disabled {
            player_id,
            notice_delivery,
        });
    }
    if selector == CLIENT_AUCTION_CLOSE_MESSAGE {
        game.find_player_mut(player_id)
            .expect("player проверен до close")
            .set_auction_open(false);
        return Some(ClientAuctionMessageReport::Closed { player_id });
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
        return Some(ClientAuctionMessageReport::CutForwarded {
            player_id,
            goods_id,
            cut_log_delivery,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_BUY_MESSAGE {
        let Some(advertised_yuan_bao) = message.base_mut().get_long() else {
            return Some(ClientAuctionMessageReport::Truncated {
                selector,
                field: "advertised yuan bao",
            });
        };
        let available_yuan_bao = game
            .find_player(player_id)
            .expect("auction buy player проверен")
            .yuan_bao();
        if (available_yuan_bao as i32) < advertised_yuan_bao {
            let notice_delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM021"))
                    .send_to_player(game.net_server(), player_id);
            return Some(ClientAuctionMessageReport::BuyFundsRejected {
                player_id,
                advertised_yuan_bao,
                available_yuan_bao,
                notice_delivery,
            });
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
        return Some(ClientAuctionMessageReport::BuyRequested {
            player_id,
            advertised_yuan_bao,
            goods_id,
            gate,
            world_delivery,
        });
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
        return Some(ClientAuctionMessageReport::Refreshed {
            player_id,
            refresh,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_SEARCH_MESSAGE {
        let name = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
        let mut read = |field| {
            message
                .base_mut()
                .get_long()
                .ok_or(ClientAuctionMessageReport::Truncated { selector, field })
        };
        let lower_level = match read("lower level") {
            Ok(value) => value,
            Err(report) => return Some(report),
        };
        let upper_level = match read("upper level") {
            Ok(value) => value,
            Err(report) => return Some(report),
        };
        let use_self = match read("use self") {
            Ok(value) => value,
            Err(report) => return Some(report),
        };
        let money_type = match read("money type") {
            Ok(value) => value,
            Err(report) => return Some(report),
        };
        let weapon_type = match read("weapon type") {
            Ok(value) => value,
            Err(report) => return Some(report),
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
        return Some(ClientAuctionMessageReport::Forwarded {
            selector,
            player_id,
            world_selector: WORLD_AUCTION_SEARCH_MESSAGE,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_PAGE_MESSAGE {
        let Some(page) = message.base_mut().get_long() else {
            return Some(ClientAuctionMessageReport::Truncated {
                selector,
                field: "page",
            });
        };
        let mut world = CMessage::new(WORLD_AUCTION_PAGE_MESSAGE);
        world.base_mut().add_long(player_id);
        world.base_mut().add_long(page);
        let world_delivery = world.send(game, false);
        return Some(ClientAuctionMessageReport::Forwarded {
            selector,
            player_id,
            world_selector: WORLD_AUCTION_PAGE_MESSAGE,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_SELF_MESSAGE {
        let mut world = CMessage::new(WORLD_AUCTION_SELF_MESSAGE);
        world.base_mut().add_long(player_id);
        let world_delivery = world.send(game, false);
        return Some(ClientAuctionMessageReport::Forwarded {
            selector,
            player_id,
            world_selector: WORLD_AUCTION_SELF_MESSAGE,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_RELAY_MESSAGE {
        message.set_message_type(WORLD_AUCTION_RELAY_MESSAGE);
        let world_delivery = message.send(game, false);
        return Some(ClientAuctionMessageReport::Forwarded {
            selector,
            player_id,
            world_selector: WORLD_AUCTION_RELAY_MESSAGE,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_CELL_MESSAGE {
        let Some(position) = message.base_mut().get_long() else {
            return Some(ClientAuctionMessageReport::Truncated {
                selector,
                field: "auction position",
            });
        };
        let position = position as u32;
        let Some(goods) = game
            .find_player(player_id)
            .and_then(|player| player.auction_goods_identity_at(position))
        else {
            return Some(ClientAuctionMessageReport::GoodsMissing {
                player_id,
                position,
            });
        };
        let mut world = CMessage::new(WORLD_AUCTION_CELL_MESSAGE);
        world.base_mut().add_ulong(position);
        world.base_mut().add_long(player_id);
        world.base_mut().add_guid(goods.ex_id);
        let world_delivery = world.send(game, false);
        return Some(ClientAuctionMessageReport::Forwarded {
            selector,
            player_id,
            world_selector: WORLD_AUCTION_CELL_MESSAGE,
            world_delivery,
        });
    }
    if selector == CLIENT_AUCTION_BUY_EXTENSION_MESSAGE {
        let Some(goods_index) = message.base_mut().get_long() else {
            return Some(ClientAuctionMessageReport::Truncated {
                selector,
                field: "extension goods index",
            });
        };
        let Some(amount) = message.base_mut().get_long() else {
            return Some(ClientAuctionMessageReport::Truncated {
                selector,
                field: "extension goods amount",
            });
        };
        let goods_index = goods_index as u32;
        let amount = amount as u32;
        let mut report = AuctionExtensionBuyReport {
            player_id,
            goods_index,
            amount,
            required_crystals: 0,
            consumptions: Vec::new(),
            consumption_deliveries: Vec::new(),
            additions: Vec::new(),
            addition_deliveries: Vec::new(),
            rejected_goods: Vec::new(),
            audit_delivery: None,
            block: None,
        };
        let block = |report: &mut AuctionExtensionBuyReport, block| {
            report.block = Some(block);
            ClientAuctionMessageReport::ExtensionBought(report.clone())
        };
        if amount as i32 <= 0 {
            return Some(block(&mut report, AuctionExtensionBuyBlock::InvalidAmount));
        }
        if game
            .find_player(player_id)
            .is_none_or(|player| player.packet().space() < amount)
        {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::InsufficientPacketSpace,
            ));
        }
        let Some(properties) = game
            .goods_factory()
            .query_goods_base_properties(goods_index)
        else {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::MissingGoodsProperties,
            ));
        };
        if !properties.has_enabled_addon_property(GAP_GOODS_PACKAGE_EXTENTION) {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::MissingGoodsProperties,
            ));
        }
        let values = properties.get_addon_property_values(GAP_GOODS_PACKAGE_EXTENTION);
        let kind = values
            .iter()
            .find(|value| value.id == 1)
            .map_or(0, |value| value.base_value);
        if kind != 3 {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::WrongExtensionKind,
            ));
        }
        let price = values
            .iter()
            .find(|value| value.id == 2)
            .map_or(0, |value| value.base_value) as u32;
        report.required_crystals = price.wrapping_mul(amount).wrapping_mul(100);
        let crystal_index = game
            .goods_factory()
            .query_goods_id_by_original_name(Some(b"FZ0965"));
        if crystal_index == 0 {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::MissingCrystalGoods,
            ));
        }
        if game.find_player(player_id).is_none_or(|player| {
            player.check_item_in_packet(crystal_index) < report.required_crystals
        }) {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::InsufficientCrystals,
            ));
        }
        report.consumptions = game
            .find_player_mut(player_id)
            .expect("player проверен до crystal removal")
            .remove_item_in_packet(crystal_index, report.required_crystals);
        if report.consumptions.is_empty() {
            return Some(block(
                &mut report,
                AuctionExtensionBuyBlock::CrystalRemovalFailed,
            ));
        }
        report.consumption_deliveries = report
            .consumptions
            .iter()
            .map(|consumption| game.send_player_packet_consumption(consumption))
            .collect();

        let created = game.create_goods_batch(goods_index, amount);
        if let Some(first) = created.first() {
            let player = game
                .find_player(player_id)
                .expect("player существует до extension audit");
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
            audit
                .base_mut()
                .add_long(player.server_region_id().unwrap_or_default());
            audit
                .base_mut()
                .add_ulong(player.shape().get_tile_x().unwrap_or_default() as u32);
            audit
                .base_mut()
                .add_ulong(player.shape().get_tile_y().unwrap_or_default() as u32);
            audit.base_mut().add_ulong(player.client_ip());
            report.audit_delivery = Some(audit.send(game, false));
        }
        let mut encode = |goods: &CGoods| runtime.encode_goods_for_old_client(goods);
        let (additions, rejected) = game
            .add_goods_to_player_packet(player_id, created, &mut encode)
            .expect("player существует до extension packet add");
        report.addition_deliveries = additions
            .iter()
            .map(|addition| game.send_player_packet_addition(addition))
            .collect();
        report.additions = additions;
        report.rejected_goods = rejected.iter().map(|goods| goods.identity()).collect();
        return Some(ClientAuctionMessageReport::ExtensionBought(report));
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
    Some(ClientAuctionMessageReport::Opened {
        player_id,
        setup_delivery,
        world_delivery,
    })
}

fn dispatch_auction_listing<Runtime, Tick>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    tick_ms: &mut Tick,
    player_id: i32,
) -> ClientAuctionMessageReport
where
    Runtime: OldClientGoodsCodec + IncrementShopBillingContext,
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
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "five-second gate",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
        }

        for field in ["client field 0", "client field 1"] {
            if message.base_mut().get_long().is_none() {
                return ClientAuctionMessageReport::ListingRejected {
                    player_id,
                    gate,
                    reason: field,
                    notice_delivery: None,
                    snapshot_refreshes: 1,
                };
            }
        }
        let Some(seller_money_signed) = message.base_mut().get_long() else {
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "seller money",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
        };
        let Some(client_goods_type) = message.base_mut().get_long() else {
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "client goods type",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
        };

        let setup = game.globe_setup();
        let price_gate = ((seller_money_signed as f32) * setup.auction_factor_c())
            .round()
            .max(setup.auction_yuan_fee_minimum().round()) as i32;
        if seller_money_signed < price_gate {
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "seller money below fee floor",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
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
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "auction limit",
                notice_delivery: Some(notice_delivery),
                snapshot_refreshes: 1,
            };
        }
        if game
            .find_player(player_id)
            .is_none_or(|player| player.yuan_bao() < listing_fee)
        {
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "insufficient yuan bao listing fee",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
        }
        let Some(listing_goods) = game
            .find_player(player_id)
            .and_then(|player| player.auction_listing().get_goods(0))
        else {
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "missing auction listing goods",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
        };
        let base_index = listing_goods.base_properties_index();
        if !game.globe_setup().auction_goods_allowed(base_index) {
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "goods not allowed in auction",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
        }
        if game
            .auction_room()
            .contains_goods(listing_goods.identity().ex_id)
        {
            let notice_delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM017"))
                    .send_to_player(game.net_server(), player_id);
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "duplicate auction guid",
                notice_delivery: Some(notice_delivery),
                snapshot_refreshes: 1,
            };
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
            return ClientAuctionMessageReport::ListingRejected {
                player_id,
                gate,
                reason: "missing goods base properties after delete",
                notice_delivery: None,
                snapshot_refreshes: 1,
            };
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

    finish_current_auction_listing(game, runtime, player_id, gate, !had_pending)
}

fn auction_yuan_listing_fee(
    setup: &crate::setup::globesetup::GlobeSetupSnapshot,
    auction_time: u32,
) -> u32 {
    let mut fee = (((auction_time / 0xe10) as f32)
        * setup.auction_factor_b()
        * setup.auction_base_yuan_bao())
    .round();
    if fee < setup.auction_yuan_fee_minimum() {
        fee = setup.auction_yuan_fee_minimum().round();
    }
    if setup.auction_yuan_fee_maximum() < fee {
        fee = setup.auction_yuan_fee_maximum().round();
    }
    fee as u32
}

fn finish_current_auction_listing<Runtime: OldClientGoodsCodec + IncrementShopBillingContext>(
    game: &mut CGame,
    runtime: &mut Runtime,
    player_id: i32,
    gate: Option<AuctionListingGate>,
    fresh_request: bool,
) -> ClientAuctionMessageReport {
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
        return ClientAuctionMessageReport::ListingBillingPending {
            player_id,
            gate,
            fee,
            snapshot_refreshes: u8::from(fresh_request),
        };
    }

    let node = game
        .find_player(player_id)
        .and_then(|player| player.current_auction_node())
        .expect("DoneCurAucNode вызывается только для pending node")
        .clone();
    let payload = match node.serialize() {
        Ok(payload) => payload,
        Err(error) => {
            return ClientAuctionMessageReport::ListingSerializeBlocked {
                player_id,
                gate,
                error,
                snapshot_refreshes: u8::from(fresh_request),
            };
        }
    };
    let mut world_deliveries = Vec::new();
    let mut add = CMessage::new(WORLD_AUCTION_ADD_NODE_MESSAGE);
    add.base_mut().add(&payload);
    world_deliveries.push(add.send(game, false));

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
    world_deliveries.push(sale_log.send(game, false));

    let goods = game
        .decode_auction_goods(node.goods_bytes())
        .expect("только что сериализованный listing goods обязан декодироваться");
    let old_client_payload = runtime.encode_goods_for_old_client(&goods);
    let mut client = CMessage::new(CLIENT_AUCTION_LIST_RESULT_MESSAGE);
    client.base_mut().add_long(1);
    client.base_mut().add_ulong(node.auction_time());
    client.base_mut().add_ulong(u32::from(node.money_type()));
    client.base_mut().add_ulong(node.seller_money());
    client.base_mut().add_ulong(old_client_payload.len() as u32);
    client.base_mut().add(&old_client_payload);
    let mut client_deliveries = vec![client.send_to_player(game.net_server(), player_id)];

    let cleared = game
        .find_player_mut(player_id)
        .expect("auction listing player проверен перед clear")
        .take_current_auction_node();
    debug_assert!(cleared.is_some());
    game.find_player_mut(player_id)
        .expect("auction listing player проверен перед fee clear")
        .set_auction_listing_fee(0);
    client_deliveries.push(
        colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM016"))
            .send_to_player(game.net_server(), player_id),
    );
    ClientAuctionMessageReport::Listed {
        player_id,
        gate,
        world_deliveries,
        client_deliveries,
        // AddItemToAuction вызывает AddByteGS2WS; fresh handler затем делает
        // это повторно. Полный persisted player snapshot остаётся владельцу
        // уже существующей 0x6080E границы.
        snapshot_refreshes: 1 + u8::from(fresh_request),
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_c2s_auction.cpp

// ============================================================================
// FUNCTION: OnMSG_C2S_AUCTION
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_c2s_auction.cpp:32
// RVA: 0x000877B0
// ADDRESS: 004877b0
// PROTOTYPE: void __cdecl OnMSG_C2S_AUCTION(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049822b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_c2s_auction.cpp
// RVA: 0x0009822B
// ADDRESS: 0049822b
// PROTOTYPE: undefined Catch@0049822b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsNode::CGoodsNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\onmsg_c2s_auction.cpp
// RVA: 0x00098240
// ADDRESS: 00498240
// PROTOTYPE: undefined __thiscall CGoodsNode(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
