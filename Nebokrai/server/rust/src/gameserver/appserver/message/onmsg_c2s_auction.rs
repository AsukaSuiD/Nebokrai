//! Клиентские lifecycle-входы аукциона GameServer.
//!
//! Точная пара GameServer EXE/PDB и исходный owner
//! `server/gameserver/appserver/message/onmsg_c2s_auction.cpp` подтверждают
//! close `0x90A01`, cut `0x90A03` и auction controls `0x90A05..0C`: cut
//! сначала посылает exact `0x60216`, затем переписывает исходный wire в
//! `0x60809`; поиск сохраняет player criteria и сбрасывает page, browse/self
//! запросы уходят в World,
//! клиент открытия получает `0xC0706`, player-open меняется до World `0x60810`,
//! extension batch оплачивается `FZ0965`, логируется и добавляется в packet,
//! а выключенный аукцион возвращает `GPM013` до чтения payload.
//! Остальные gameplay selectors остаются RAW ниже.

use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_GOODS_PACKAGE_EXTENTION;
use crate::gameserver::appserver::player::{
    AuctionSelfGoodsRefresh, CiQingPacketAddition, CiQingPacketConsumption,
};
use crate::gameserver::gameserver::game::{
    CGame, OldClientGoodsCodec, colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const CLIENT_AUCTION_CLOSE_MESSAGE: i32 = 0x0009_0A01;
const CLIENT_AUCTION_CUT_MESSAGE: i32 = 0x0009_0A03;
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
const WORLD_GOODS_AUDIT_MESSAGE: i32 = 0x0006_0202;

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
    Runtime: OldClientGoodsCodec,
    Tick: FnMut(&mut Runtime) -> u32,
{
    let selector = message.message_type();
    if !matches!(
        selector,
        CLIENT_AUCTION_CLOSE_MESSAGE
            | CLIENT_AUCTION_CUT_MESSAGE
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
