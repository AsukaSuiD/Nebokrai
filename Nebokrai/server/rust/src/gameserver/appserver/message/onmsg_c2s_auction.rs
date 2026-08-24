//! Клиентские lifecycle-входы аукциона GameServer.
//!
//! Точная пара GameServer EXE/PDB и исходный owner
//! `server/gameserver/appserver/message/onmsg_c2s_auction.cpp` подтверждают
//! close `0x90A01`, refresh/browse/relay `0x90A05..0A` и open `0x90A0B`: поиск сохраняет
//! player criteria и сбрасывает page, browse/self запросы уходят в World,
//! клиент открытия получает `0xC0706`, player-open меняется до World `0x60810`,
//! а выключенный аукцион возвращает `GPM013` до чтения payload.
//! Остальные gameplay selectors остаются RAW ниже.

use crate::gameserver::appserver::player::AuctionSelfGoodsRefresh;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const CLIENT_AUCTION_CLOSE_MESSAGE: i32 = 0x0009_0A01;
const CLIENT_AUCTION_REFRESH_MESSAGE: i32 = 0x0009_0A05;
const CLIENT_AUCTION_SEARCH_MESSAGE: i32 = 0x0009_0A06;
const CLIENT_AUCTION_PAGE_MESSAGE: i32 = 0x0009_0A07;
const CLIENT_AUCTION_SELF_MESSAGE: i32 = 0x0009_0A08;
const CLIENT_AUCTION_RELAY_MESSAGE: i32 = 0x0009_0A09;
const CLIENT_AUCTION_CELL_MESSAGE: i32 = 0x0009_0A0A;
const CLIENT_AUCTION_OPEN_MESSAGE: i32 = 0x0009_0A0B;
const CLIENT_AUCTION_OPEN_SETUP_MESSAGE: i32 = 0x000C_0706;
const WORLD_AUCTION_PAGE_MESSAGE: i32 = 0x0006_0803;
const WORLD_AUCTION_REFRESH_MESSAGE: i32 = 0x0006_080A;
const WORLD_AUCTION_SELF_MESSAGE: i32 = 0x0006_0802;
const WORLD_AUCTION_SEARCH_MESSAGE: i32 = 0x0006_080F;
const WORLD_AUCTION_RELAY_MESSAGE: i32 = 0x0006_080B;
const WORLD_AUCTION_CELL_MESSAGE: i32 = 0x0006_080C;
const WORLD_AUCTION_OPEN_MESSAGE: i32 = 0x0006_0810;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
}

pub(crate) fn dispatch_client_auction_message<Runtime, Tick>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
    mut tick_ms: Tick,
) -> Option<ClientAuctionMessageReport>
where
    Tick: FnMut(&mut Runtime) -> u32,
{
    let selector = message.message_type();
    if !matches!(
        selector,
        CLIENT_AUCTION_CLOSE_MESSAGE
            | CLIENT_AUCTION_REFRESH_MESSAGE
            | CLIENT_AUCTION_SEARCH_MESSAGE
            | CLIENT_AUCTION_PAGE_MESSAGE
            | CLIENT_AUCTION_SELF_MESSAGE
            | CLIENT_AUCTION_RELAY_MESSAGE
            | CLIENT_AUCTION_CELL_MESSAGE
            | CLIENT_AUCTION_OPEN_MESSAGE
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
