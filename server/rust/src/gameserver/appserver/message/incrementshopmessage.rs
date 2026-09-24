//! Входной владелец Increment Shop GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/message/incrementshopmessage.cpp`. Материализованный жизненный
//! цикл просмотра `0x90601..05` сохраняет общие проверки смерти и региона,
//! процесс игрока и вложенное перемещаемое состояние, объявление, обход
//! категорий/поиска, клиентские пакеты `0xC0401/2/4`, World `0x5FD0A` и полный
//! запрос покупки `0xEF202`. Покупка заранее создаёт точную партию фабрики,
//! проверяет размещение в packet, скидочные стеки и устаревшие проверки
//! YuanBao; окончательная выдача и списание принадлежат парному владельцу
//! ответа Billing в `unibillmessage`. Выполненные эффекты не дублируются
//! отчётом, а диагностируются через `tracing`.

use std::net::Ipv4Addr;

use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_YUANBAO_DIKOU;
use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::setup::incrementshoplist::IncrementShopItem;
use tracing::{debug, trace};

const OPEN_INCREMENT_SHOP: i32 = 0x0009_0601;
const QUERY_INCREMENT_SHOP: i32 = 0x0009_0602;
const PURCHASE_INCREMENT_SHOP: i32 = 0x0009_0603;
const CLOSE_INCREMENT_SHOP: i32 = 0x0009_0604;
const FORWARD_INCREMENT_QUERY: i32 = 0x0009_0605;
const OPEN_INCREMENT_SHOP_RESPONSE: i32 = 0x000C_0401;
const QUERY_INCREMENT_SHOP_RESPONSE: i32 = 0x000C_0402;
const CANCEL_INCREMENT_SHOP_RESPONSE: i32 = 0x000C_0404;
const FORWARD_INCREMENT_QUERY_WORLD: i32 = 0x0005_FD0A;
const INCREMENT_PURCHASE_BILLING: i32 = 0x000E_F202;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameIncrementShopMessageError {
    MissingQueryMode,
    MissingSearchText,
    MissingForwardValue,
    MissingPurchasePage,
    MissingPurchaseGoods,
    MissingPurchaseQuantity,
    ItemCountOutsideLegacyRange { count: usize },
}

pub(crate) fn dispatch_increment_shop_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<(), GameIncrementShopMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        OPEN_INCREMENT_SHOP
            | QUERY_INCREMENT_SHOP
            | PURCHASE_INCREMENT_SHOP
            | CLOSE_INCREMENT_SHOP
            | FORWARD_INCREMENT_QUERY
    ) {
        return None;
    }

    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        trace!(message_type, ?region_id, "сообщение Increment Shop пропущено: нет игрока");
        return Some(Ok(()));
    };
    let Some(player) = game.find_player(player_id) else {
        trace!(message_type, player_id, ?region_id, "сообщение Increment Shop пропущено: игрок не найден");
        return Some(Ok(()));
    };
    if player.is_dead() {
        let mut response = CMessage::new(OPEN_INCREMENT_SHOP_RESPONSE);
        response.add_byte(0);
        let _ = response.send_to_player(game.net_server(), player_id);
        trace!(message_type, player_id, ?region_id, "Increment Shop отклонён: игрок мёртв");
        return Some(Ok(()));
    }
    if region_id.and_then(|id| game.find_region(id)).is_none() {
        let _ = game
            .find_player_mut(player_id)
            .expect("resolved Increment Shop player остаётся live")
            .release_goods_session_state();
        let _ = CMessage::new(CANCEL_INCREMENT_SHOP_RESPONSE)
            .send_to_player(game.net_server(), player_id);
        trace!(message_type, player_id, ?region_id, "Increment Shop закрыт: регион не найден");
        return Some(Ok(()));
    }

    match message_type {
        OPEN_INCREMENT_SHOP => {
            let progress = game
                .find_player(player_id)
                .expect("resolved Increment Shop player остаётся live")
                .current_progress();
            let mut response = CMessage::new(OPEN_INCREMENT_SHOP_RESPONSE);
            if progress == PlayerProgress::None {
                let _ = game
                    .find_player_mut(player_id)
                    .expect("resolved Increment Shop player остаётся live")
                    .begin_equipment_session(PlayerProgress::Increment, true);
                response.add_byte(1);
                add_c_string(&mut response, game.increment_shop_list().affiche());
                let _ = response.send_to_player(game.net_server(), player_id);
                debug!(player_id, "Increment Shop открыт");
            } else {
                response.add_byte(0);
                let _ = response.send_to_player(game.net_server(), player_id);
                trace!(player_id, ?progress, "открытие Increment Shop отклонено процессом игрока");
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
                let _ = response.send_to_player(game.net_server(), player_id);
                trace!(player_id, ?progress, "запрос Increment Shop отклонён процессом игрока");
            } else {
                let mode = message
                    .base_mut()
                    .get_char()
                    .ok_or(GameIncrementShopMessageError::MissingQueryMode);
                let mode = match mode {
                    Ok(mode) => mode,
                    Err(error) => return Some(Err(error)),
                };
                let selected: Vec<&IncrementShopItem> =
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
                        selected
                    } else {
                        game.increment_shop_list()
                            .items_on_page(mode as u8)
                            .iter()
                            .collect()
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
                }
                let _ = response.send_to_player(game.net_server(), player_id);
                debug!(player_id, mode, count, "результаты Increment Shop отправлены");
            }
        }
        PURCHASE_INCREMENT_SHOP => {
            let progress = game
                .find_player(player_id)
                .expect("resolved Increment Shop player остаётся live")
                .current_progress();
            if progress != PlayerProgress::Increment {
                trace!(player_id, ?progress, "покупка Increment Shop отклонена процессом игрока");
            } else {
                let page = match message.base_mut().get_char() {
                    Some(page) => page,
                    None => return Some(Err(GameIncrementShopMessageError::MissingPurchasePage)),
                };
                let goods_id = match message.base_mut().get_long() {
                    Some(goods_id) => goods_id as u32,
                    None => return Some(Err(GameIncrementShopMessageError::MissingPurchaseGoods)),
                };
                let requested_quantity = match message.base_mut().get_long() {
                    Some(quantity) => quantity as u32,
                    None => {
                        return Some(Err(GameIncrementShopMessageError::MissingPurchaseQuantity));
                    }
                };
                if goods_id == 0 || requested_quantity == 0 {
                    trace!(player_id, goods_id, requested_quantity, "пустой запрос покупки Increment Shop пропущен");
                } else {
                    let Some(item) = game
                        .increment_shop_list()
                        .get_item(goods_id, page as u8)
                        .cloned()
                    else {
                        trace!(player_id, goods_id, page, "товар Increment Shop не найден в каталоге");
                        return Some(Ok(()));
                    };
                    if game
                        .goods_factory()
                        .query_goods_base_properties(goods_id)
                        .is_none()
                    {
                        trace!(player_id, goods_id, "свойства товара Increment Shop не найдены");
                        return Some(Ok(()));
                    }
                    let goods_amount = item.overlapped_amount.wrapping_mul(requested_quantity);
                    let created = game.create_goods_batch(goods_id, goods_amount);
                    if created.is_empty() {
                        trace!(player_id, goods_id, goods_amount, "фабрика не создала товары Increment Shop");
                        return Some(Ok(()));
                    }
                    if !increment_batch_fits_packet(game, player_id, &created) {
                        let _ = colored_player_notice_message(
                            0xffff_ffff,
                            0,
                            game.get_string_by_id(b"GS0082"),
                        )
                        .send_to_player(game.net_server(), player_id);
                        trace!(player_id, goods_id, goods_amount, "в packet недостаточно места для покупки Increment Shop");
                    } else {
                        let (deduction_value, available_deduction) = game
                            .find_player(player_id)
                            .map(|player| {
                                let matches = player
                                    .packet()
                                    .base()
                                    .get_goods_by_base_properties(item.deduction_goods_id);
                                let deduction_value = matches
                                    .first()
                                    .map(|goods| {
                                        goods
                                            .addon_property_value(
                                                game.goods_factory(),
                                                GAP_YUANBAO_DIKOU,
                                                1,
                                            )
                                            .max(0) as u32
                                    })
                                    .unwrap_or(0);
                                let available = matches
                                    .iter()
                                    .fold(0u32, |total, goods| total.wrapping_add(goods.amount()));
                                (deduction_value, available)
                            })
                            .unwrap_or_default();
                        let deduction_amount = goods_amount.min(available_deduction);
                        let deduction_goods_id =
                            if deduction_amount != 0 && item.deduction_goods_id != 0 {
                                item.deduction_goods_id
                            } else {
                                0
                            };
                        let undiscounted =
                            f64::from(requested_quantity) * f64::from(item.yuan_bao_price);
                        let discounted =
                            undiscounted - f64::from(deduction_value) * f64::from(deduction_amount);
                        let available = game
                            .find_player(player_id)
                            .map_or(0, |player| player.yuan_bao());
                        let valid = item.yuan_bao_price == 0
                            || (f64::from(available) >= discounted
                                && available / item.yuan_bao_price >= goods_amount
                                && discounted >= f64::from(item.yuan_bao_price));
                        if !valid {
                            let required = legacy_f64_to_u32(discounted);
                            let _ = colored_player_notice_message(
                                0xffff_ffff,
                                0,
                                game.get_string_by_id(b"GS0038"),
                            )
                            .send_to_player(game.net_server(), player_id);
                            trace!(player_id, goods_id, required, available, "для покупки Increment Shop недостаточно YuanBao");
                        } else {
                            let charge = legacy_f64_to_u32(discounted);
                            let player = game
                                .find_player(player_id)
                                .expect("resolved Increment Shop player остаётся live");
                            let account = player.account().to_vec();
                            let ip = Ipv4Addr::from(player.client_ip().to_le_bytes())
                                .to_string()
                                .into_bytes();
                            let name = player.shape().base_object().get_name().to_vec();
                            let (login_server_id, world_server_id) = game.server_ids();
                            let mut request = CMessage::new(INCREMENT_PURCHASE_BILLING);
                            request.add_long(player_id);
                            add_c_string(&mut request, &account);
                            add_c_string(&mut request, &ip);
                            add_c_string(&mut request, &name);
                            request.add_ulong(charge);
                            request.add_ulong(goods_id);
                            request.add_ulong(goods_amount);
                            request.add_ulong(deduction_goods_id);
                            request.add_long(login_server_id);
                            request.add_long(world_server_id);
                            let _ = request.send_to_bs(game, false);
                            debug!(player_id, page, goods_id, requested_quantity, goods_amount, charge, deduction_goods_id, deduction_amount, "запрос покупки Increment Shop отправлен в Billing");
                        }
                    }
                }
            }
        }
        CLOSE_INCREMENT_SHOP => {
            if game
                .find_player(player_id)
                .expect("resolved Increment Shop player остаётся live")
                .current_progress()
                == PlayerProgress::Increment
            {
                let _ = game
                    .find_player_mut(player_id)
                    .expect("resolved Increment Shop player остаётся live")
                    .release_goods_session_state();
                debug!(player_id, "Increment Shop закрыт");
            }
        }
        FORWARD_INCREMENT_QUERY => {
            let value = match message.base_mut().get_long() {
                Some(value) => value,
                None => return Some(Err(GameIncrementShopMessageError::MissingForwardValue)),
            };
            let mut response = CMessage::new(FORWARD_INCREMENT_QUERY_WORLD);
            response.add_long(player_id);
            response.add_long(value);
            let _ = response.send(game, false);
            debug!(player_id, value, "запрос Increment Shop перенаправлен в World");
        }
        _ => unreachable!("Increment Shop selector проверен до player gates"),
    }
    Some(Ok(()))
}

pub(crate) fn increment_batch_fits_packet(
    game: &CGame,
    player_id: i32,
    created: &[crate::gameserver::appserver::goods::cgoods::CGoods],
) -> bool {
    let Some(player) = game.find_player(player_id) else {
        return false;
    };
    let mut packet = player.packet().clone();
    for goods in created.iter().cloned() {
        let mut incoming = Some(goods);
        let outcome = packet.add_goods(&mut incoming, game.goods_factory(), true);
        if incoming.is_some()
            || matches!(
                outcome,
                crate::gameserver::appserver::container::cvolumelimitgoodscontainer::VolumeGoodsAddOutcome::Rejected(_)
            )
        {
            return false;
        }
    }
    true
}

fn legacy_f64_to_u32(value: f64) -> u32 {
    (value.trunc() as i64) as u32
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
// COMPONENT_VARIANT_END: GameServer
