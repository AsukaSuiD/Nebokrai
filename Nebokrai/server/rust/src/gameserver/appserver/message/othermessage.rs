//! Other-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/othermessage.cpp`. LeiTing `0x7FA17` замыкает ответ
//! WorldServer: player ID, exact partial-mutation codec и итоговый `0xBF73E`.
//! Chat delivery `0x7FA01/02/0F/10/11` замыкает private/faction/global/country
//! WorldServer routes в exact `0xBF801/806/814/815/816` client wire.
//! Остальные ветви ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::player::PlayerLeiTingDecodeBlock;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const PRIVATE_CHAT_DELIVERY: u32 = 0x0007_fa01;
const FACTION_CHAT_DELIVERY: u32 = 0x0007_fa02;
const WORLD_CHAT_DELIVERY: u32 = 0x0007_fa0f;
const COUNTRY_CHAT_DELIVERY: u32 = 0x0007_fa10;
const COUNTRY_NOTICE_DELIVERY: u32 = 0x0007_fa11;
const WORLD_LEI_TING_UPDATE: u32 = 0x0007_fa17;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameOtherMessageOutcome {
    PlayerMissing,
    ChatIgnored {
        status: i8,
    },
    ChatDelivered {
        deliveries: Vec<i32>,
    },
    ChatBroadcast {
        delivery: Result<i32, SendMessageError>,
    },
    LeiTingUpdated {
        client_delivery: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOtherMessageError {
    MissingPlayerId,
    MissingField(&'static str),
    LeiTing(PlayerLeiTingDecodeBlock),
}

#[must_use = "other-message report сохраняет World decode и client publication"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameOtherMessageReport {
    pub(crate) message_type: u32,
    pub(crate) player_id: i32,
    pub(crate) outcome: GameOtherMessageOutcome,
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(value);
    message.base_mut().add_byte(0);
}

fn read_long(message: &mut CMessage, field: &'static str) -> Result<i32, GameOtherMessageError> {
    message
        .base_mut()
        .get_long()
        .ok_or(GameOtherMessageError::MissingField(field))
}

fn read_char(message: &mut CMessage, field: &'static str) -> Result<i8, GameOtherMessageError> {
    message
        .base_mut()
        .get_char()
        .ok_or(GameOtherMessageError::MissingField(field))
}

fn read_string(message: &mut CMessage, maximum: usize) -> Vec<u8> {
    message
        .base_mut()
        .get_str_bytes(maximum)
        .unwrap_or_default()
}

pub(crate) fn dispatch_game_other_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<GameOtherMessageReport, GameOtherMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        PRIVATE_CHAT_DELIVERY
            | FACTION_CHAT_DELIVERY
            | WORLD_CHAT_DELIVERY
            | COUNTRY_CHAT_DELIVERY
            | COUNTRY_NOTICE_DELIVERY
            | WORLD_LEI_TING_UPDATE
    ) {
        return None;
    }
    if message_type == WORLD_CHAT_DELIVERY {
        message.set_message_type(0x000b_f814);
        let delivery = message.send_all(game.current_net_server());
        return Some(Ok(GameOtherMessageReport {
            message_type,
            player_id: 0,
            outcome: GameOtherMessageOutcome::ChatBroadcast { delivery },
        }));
    }
    if message_type == COUNTRY_CHAT_DELIVERY {
        let result = (|| {
            let _legacy_ignored = read_char(message, "country chat prefix")?;
            let chat_type = read_char(message, "country chat type")? as u8;
            let country = read_char(message, "country chat country")? as u8;
            let sender = read_string(message, 0x20);
            let content = read_string(message, 0x400);
            let player_ids = game.player_ids_in_country(u32::from(country));
            let mut deliveries = Vec::with_capacity(player_ids.len());
            for player_id in player_ids {
                let mut response = CMessage::new(0x000b_f815);
                response.base_mut().add_byte(1);
                response.base_mut().add_byte(chat_type);
                add_c_string(&mut response, &sender);
                add_c_string(&mut response, &content);
                deliveries.push(response.send_to_player(game.net_server(), player_id));
            }
            Ok(GameOtherMessageReport {
                message_type,
                player_id: 0,
                outcome: GameOtherMessageOutcome::ChatDelivered { deliveries },
            })
        })();
        return Some(result);
    }
    if message_type == COUNTRY_NOTICE_DELIVERY {
        let result = (|| {
            let notice_type = read_char(message, "country notice type")? as u8;
            let country = read_char(message, "country notice country")? as u8;
            let content = read_string(message, 0x400);
            let player_ids = game.player_ids_in_country(u32::from(country));
            let mut deliveries = Vec::with_capacity(player_ids.len());
            for player_id in player_ids {
                let mut response = CMessage::new(0x000b_f816);
                response.base_mut().add_byte(notice_type);
                add_c_string(&mut response, &content);
                deliveries.push(response.send_to_player(game.net_server(), player_id));
            }
            Ok(GameOtherMessageReport {
                message_type,
                player_id: 0,
                outcome: GameOtherMessageOutcome::ChatDelivered { deliveries },
            })
        })();
        return Some(result);
    }
    if message_type == FACTION_CHAT_DELIVERY {
        let result = (|| {
            let player_id = read_long(message, "faction chat player id")?;
            let owner_type = read_long(message, "faction chat owner type")?;
            let owner_id = read_long(message, "faction chat owner id")?;
            if game.find_player(player_id).is_none() {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerMissing,
                });
            }
            let sender = read_string(message, 0x100);
            let content = read_string(message, 0x400);
            let mut response = CMessage::new(0x000b_f801);
            response.add_long(2);
            response.add_long(owner_type);
            response.add_long(owner_id);
            add_c_string(&mut response, &sender);
            add_c_string(&mut response, &content);
            let delivery = response.send_to_player(game.net_server(), player_id);
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::ChatDelivered {
                    deliveries: vec![delivery],
                },
            })
        })();
        return Some(result);
    }
    if message_type == PRIVATE_CHAT_DELIVERY {
        let result = (|| {
            let status = read_char(message, "private chat status")?;
            let owner_type = read_long(message, "private chat owner type")?;
            let owner_id = read_long(message, "private chat owner id")?;
            if status == 0 {
                let Some(player) = game.find_player(owner_id) else {
                    return Ok(GameOtherMessageReport {
                        message_type,
                        player_id: owner_id,
                        outcome: GameOtherMessageOutcome::PlayerMissing,
                    });
                };
                let delivery =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0047"))
                        .send_to_player(game.net_server(), player.player_id());
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id: owner_id,
                    outcome: GameOtherMessageOutcome::ChatDelivered {
                        deliveries: vec![delivery],
                    },
                });
            }
            if status != 1 && status != 2 {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id: owner_id,
                    outcome: GameOtherMessageOutcome::ChatIgnored { status },
                });
            }
            let target = read_string(message, 0x100);
            let sender = read_string(message, 0x100);
            let lookup_name = if status == 1 { &target } else { &sender };
            let Some(player_id) = game
                .find_player_by_name(lookup_name)
                .map(|player| player.player_id())
            else {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id: 0,
                    outcome: GameOtherMessageOutcome::PlayerMissing,
                });
            };
            let content = read_string(message, 0x400);
            let mut response = CMessage::new(0x000b_f801);
            response.add_long(4);
            response.add_long(owner_type);
            response.add_long(owner_id);
            add_c_string(&mut response, &target);
            add_c_string(&mut response, &sender);
            add_c_string(&mut response, &content);
            let delivery = response.send_to_player(game.net_server(), player_id);
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::ChatDelivered {
                    deliveries: vec![delivery],
                },
            })
        })();
        return Some(result);
    }
    let Some(player_id) = message.base_mut().get_long() else {
        return Some(Err(GameOtherMessageError::MissingPlayerId));
    };
    let Some(player) = game.find_player_mut(player_id) else {
        return Some(Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        }));
    };
    let decode = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        player.decode_lei_ting(source, cursor)
    };
    if let Err(block) = decode {
        return Some(Err(GameOtherMessageError::LeiTing(block)));
    }
    let payload = game
        .find_player(player_id)
        .expect("LeiTing player сохранён после decode")
        .encode_lei_ting();
    let mut response = CMessage::new(0x000b_f73e);
    response.base_mut().add(&payload);
    let client_delivery = response.send_to_player(game.net_server(), player_id);
    Some(Ok(GameOtherMessageReport {
        message_type,
        player_id,
        outcome: GameOtherMessageOutcome::LeiTingUpdated { client_delivery },
    }))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\othermessage.cpp

// ============================================================================
// FUNCTION: OnOtherMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\othermessage.cpp:42
// RVA: 0x00090E20
// ADDRESS: 00490e20
// PROTOTYPE: void __cdecl OnOtherMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
