//! Входной owner Increment Shop GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, owner
//! `appserver/message/incrementshopmessage.cpp`. Материализованный browsing
//! lifecycle `0x90601/02/04/05` сохраняет общий dead/region gate, progress и
//! nesting moveable state, affiche, category/search traversal, client packets
//! `0xC0401/2/4` и World `0x5FD0A`. Поиск удаляет только ASCII space и, как
//! MSVC multimap, обходит page keys и insertion order. Покупка `0x90603`
//! остаётся ниже отдельной RAW Billing-транзакцией с goods/payment effects.

use crate::gameserver::appserver::player::{GoodsSessionPlayerRelease, PlayerProgress};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::setup::incrementshoplist::IncrementShopItem;

const OPEN_INCREMENT_SHOP: i32 = 0x0009_0601;
const QUERY_INCREMENT_SHOP: i32 = 0x0009_0602;
const CLOSE_INCREMENT_SHOP: i32 = 0x0009_0604;
const FORWARD_INCREMENT_QUERY: i32 = 0x0009_0605;
const OPEN_INCREMENT_SHOP_RESPONSE: i32 = 0x000C_0401;
const QUERY_INCREMENT_SHOP_RESPONSE: i32 = 0x000C_0402;
const CANCEL_INCREMENT_SHOP_RESPONSE: i32 = 0x000C_0404;
const FORWARD_INCREMENT_QUERY_WORLD: i32 = 0x0005_FD0A;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameIncrementShopMessageError {
    MissingQueryMode,
    MissingSearchText,
    MissingForwardValue,
    ItemCountOutsideLegacyRange { count: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IncrementShopPublishedItem {
    pub(crate) category: Option<u16>,
    pub(crate) goods_id: u32,
    pub(crate) overlapped_amount: u32,
    pub(crate) deduction_goods_id: u32,
    pub(crate) yuan_bao_price: u32,
    pub(crate) icon_id: u8,
    pub(crate) description: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameIncrementShopMessageOutcome {
    MissingPlayer,
    Dead {
        delivery: i32,
    },
    RegionMissing {
        release: GoodsSessionPlayerRelease,
        delivery: i32,
    },
    Opened {
        transition: GoodsSessionPlayerRelease,
        affiche: Vec<u8>,
        delivery: i32,
    },
    OpenRejected {
        progress: PlayerProgress,
        delivery: i32,
    },
    QueryRejected {
        progress: PlayerProgress,
        delivery: i32,
    },
    Query {
        mode: i8,
        normalized_search: Option<Vec<u8>>,
        items: Vec<IncrementShopPublishedItem>,
        delivery: i32,
    },
    Closed {
        release: Option<GoodsSessionPlayerRelease>,
    },
    Forwarded {
        value: i32,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameIncrementShopMessageReport {
    pub(crate) message_type: i32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) outcome: GameIncrementShopMessageOutcome,
}

pub(crate) fn dispatch_increment_shop_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<GameIncrementShopMessageReport, GameIncrementShopMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        OPEN_INCREMENT_SHOP | QUERY_INCREMENT_SHOP | CLOSE_INCREMENT_SHOP | FORWARD_INCREMENT_QUERY
    ) {
        return None;
    }

    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        return Some(Ok(GameIncrementShopMessageReport {
            message_type,
            player_id: None,
            region_id,
            outcome: GameIncrementShopMessageOutcome::MissingPlayer,
        }));
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(Ok(GameIncrementShopMessageReport {
            message_type,
            player_id: Some(player_id),
            region_id,
            outcome: GameIncrementShopMessageOutcome::MissingPlayer,
        }));
    };
    if player.is_dead() {
        let mut response = CMessage::new(OPEN_INCREMENT_SHOP_RESPONSE);
        response.add_byte(0);
        let delivery = response.send_to_player(game.net_server(), player_id);
        return Some(Ok(GameIncrementShopMessageReport {
            message_type,
            player_id: Some(player_id),
            region_id,
            outcome: GameIncrementShopMessageOutcome::Dead { delivery },
        }));
    }
    if region_id.and_then(|id| game.find_region(id)).is_none() {
        let release = game
            .find_player_mut(player_id)
            .expect("resolved Increment Shop player остаётся live")
            .release_goods_session_state();
        let delivery = CMessage::new(CANCEL_INCREMENT_SHOP_RESPONSE)
            .send_to_player(game.net_server(), player_id);
        return Some(Ok(GameIncrementShopMessageReport {
            message_type,
            player_id: Some(player_id),
            region_id,
            outcome: GameIncrementShopMessageOutcome::RegionMissing { release, delivery },
        }));
    }

    let outcome = match message_type {
        OPEN_INCREMENT_SHOP => {
            let progress = game
                .find_player(player_id)
                .expect("resolved Increment Shop player остаётся live")
                .current_progress();
            let mut response = CMessage::new(OPEN_INCREMENT_SHOP_RESPONSE);
            if progress == PlayerProgress::None {
                let transition = game
                    .find_player_mut(player_id)
                    .expect("resolved Increment Shop player остаётся live")
                    .begin_equipment_session(PlayerProgress::Increment, true);
                let affiche = game.increment_shop_list().affiche().to_vec();
                response.add_byte(1);
                add_c_string(&mut response, &affiche);
                let delivery = response.send_to_player(game.net_server(), player_id);
                GameIncrementShopMessageOutcome::Opened {
                    transition,
                    affiche,
                    delivery,
                }
            } else {
                response.add_byte(0);
                let delivery = response.send_to_player(game.net_server(), player_id);
                GameIncrementShopMessageOutcome::OpenRejected { progress, delivery }
            }
        }
        QUERY_INCREMENT_SHOP => {
            let progress = game
                .find_player(player_id)
                .expect("resolved Increment Shop player остаётся live")
                .current_progress();
            let mut response = CMessage::new(QUERY_INCREMENT_SHOP_RESPONSE);
            if progress != PlayerProgress::Increment {
                response.add_long(0);
                let delivery = response.send_to_player(game.net_server(), player_id);
                GameIncrementShopMessageOutcome::QueryRejected { progress, delivery }
            } else {
                let mode = message
                    .base_mut()
                    .get_char()
                    .ok_or(GameIncrementShopMessageError::MissingQueryMode);
                let mode = match mode {
                    Ok(mode) => mode,
                    Err(error) => return Some(Err(error)),
                };
                let (normalized_search, selected): (Option<Vec<u8>>, Vec<&IncrementShopItem>) =
                    if mode == -1 {
                        let search = message
                            .base_mut()
                            .get_str_bytes(0x80)
                            .ok_or(GameIncrementShopMessageError::MissingSearchText);
                        let search = match search {
                            Ok(search) => search,
                            Err(error) => return Some(Err(error)),
                        };
                        let normalized = search
                            .into_iter()
                            .filter(|byte| *byte != b' ')
                            .collect::<Vec<_>>();
                        let selected = if normalized.is_empty() {
                            Vec::new()
                        } else {
                            game.increment_shop_list().search_items(&normalized)
                        };
                        (Some(normalized), selected)
                    } else {
                        (
                            None,
                            game.increment_shop_list()
                                .items_on_page(mode as u8)
                                .iter()
                                .collect(),
                        )
                    };
                let count = match i32::try_from(selected.len()) {
                    Ok(count) => count,
                    Err(_) => {
                        return Some(Err(
                            GameIncrementShopMessageError::ItemCountOutsideLegacyRange {
                                count: selected.len(),
                            },
                        ));
                    }
                };
                response.add_long(count);
                let include_category = mode != -1;
                let mut items = Vec::with_capacity(selected.len());
                for item in selected {
                    response.add_ulong(item.goods_id);
                    response.add_ulong(item.overlapped_amount);
                    if include_category {
                        response.base_mut().add_word(item.category);
                    }
                    response.add_ulong(item.deduction_goods_id);
                    response.add_ulong(item.yuan_bao_price);
                    response.add_byte(item.icon_id);
                    add_c_string(&mut response, &item.description);
                    items.push(IncrementShopPublishedItem {
                        category: include_category.then_some(item.category),
                        goods_id: item.goods_id,
                        overlapped_amount: item.overlapped_amount,
                        deduction_goods_id: item.deduction_goods_id,
                        yuan_bao_price: item.yuan_bao_price,
                        icon_id: item.icon_id,
                        description: item.description.clone(),
                    });
                }
                let delivery = response.send_to_player(game.net_server(), player_id);
                GameIncrementShopMessageOutcome::Query {
                    mode,
                    normalized_search,
                    items,
                    delivery,
                }
            }
        }
        CLOSE_INCREMENT_SHOP => {
            let release = (game
                .find_player(player_id)
                .expect("resolved Increment Shop player остаётся live")
                .current_progress()
                == PlayerProgress::Increment)
                .then(|| {
                    game.find_player_mut(player_id)
                        .expect("resolved Increment Shop player остаётся live")
                        .release_goods_session_state()
                });
            GameIncrementShopMessageOutcome::Closed { release }
        }
        FORWARD_INCREMENT_QUERY => {
            let value = match message.base_mut().get_long() {
                Some(value) => value,
                None => return Some(Err(GameIncrementShopMessageError::MissingForwardValue)),
            };
            let mut response = CMessage::new(FORWARD_INCREMENT_QUERY_WORLD);
            response.add_long(player_id);
            response.add_long(value);
            let delivery = response.send(game, false);
            GameIncrementShopMessageOutcome::Forwarded { value, delivery }
        }
        _ => unreachable!("Increment Shop selector проверен до player gates"),
    };
    Some(Ok(GameIncrementShopMessageReport {
        message_type,
        player_id: Some(player_id),
        region_id,
        outcome,
    }))
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let prefix = &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())];
    message.base_mut().add(prefix);
    message.base_mut().add_byte(0);
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\incrementshopmessage.cpp

// ============================================================================
// FUNCTION: OnIncrementShopMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\incrementshopmessage.cpp:41
// RVA: 0x00088130
// ADDRESS: 00488130
// PROTOTYPE: void __cdecl OnIncrementShopMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
