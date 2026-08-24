//! Клиентские lifecycle-входы аукциона GameServer.
//!
//! Точная пара GameServer EXE/PDB и исходный owner
//! `server/gameserver/appserver/message/onmsg_c2s_auction.cpp` подтверждают
//! close `0x90A01` и open `0x90A0B`: клиент получает `0xC0706`, player-open
//! меняется до World `0x60810`, а выключенный аукцион возвращает `GPM013`.
//! Остальные gameplay selectors остаются RAW ниже.

use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const CLIENT_AUCTION_CLOSE_MESSAGE: i32 = 0x0009_0A01;
const CLIENT_AUCTION_OPEN_MESSAGE: i32 = 0x0009_0A0B;
const CLIENT_AUCTION_OPEN_SETUP_MESSAGE: i32 = 0x000C_0706;
const WORLD_AUCTION_OPEN_MESSAGE: i32 = 0x0006_0810;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientAuctionLifecycleReport {
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
}

pub(crate) fn dispatch_client_auction_lifecycle(
    message: &CMessage,
    game: &mut CGame,
) -> Option<ClientAuctionLifecycleReport> {
    let selector = message.message_type();
    if !matches!(
        selector,
        CLIENT_AUCTION_CLOSE_MESSAGE | CLIENT_AUCTION_OPEN_MESSAGE
    ) {
        return None;
    }
    let Some(player_id) = message.player_id() else {
        return Some(ClientAuctionLifecycleReport::MissingPlayer { selector });
    };
    if game.find_player(player_id).is_none() {
        return Some(ClientAuctionLifecycleReport::MissingPlayer { selector });
    }
    if !game.auction_now() {
        let notice_delivery =
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GPM013"))
                .send_to_player(game.net_server(), player_id);
        return Some(ClientAuctionLifecycleReport::Disabled {
            player_id,
            notice_delivery,
        });
    }
    if selector == CLIENT_AUCTION_CLOSE_MESSAGE {
        game.find_player_mut(player_id)
            .expect("player проверен до close")
            .set_auction_open(false);
        return Some(ClientAuctionLifecycleReport::Closed { player_id });
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
    Some(ClientAuctionLifecycleReport::Opened {
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
