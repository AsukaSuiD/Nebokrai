//! Other-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/othermessage.cpp`. LeiTing `0x7FA17` замыкает ответ
//! WorldServer: player ID, exact partial-mutation codec и итоговый `0xBF73E`.
//! Chat delivery `0x7FA01/02/0F/10/11` замыкает private/faction/global/country
//! WorldServer routes в exact `0xBF801/806/814/815/816` client wire.
//! Rename round trip `0x8FB05 -> 0x5FD05 -> 0x7FA0E` сохраняет World/DB
//! решение, меняет canonical player name только при result `0` и публикует
//! exact `0xBF80F` вокруг игрока либо только самому игроку при отказе.
//! World info `0x7FA03/04` доводит nation/country notices до exact
//! `0xBF803/804`: ненулевой target выбирает region/player, ноль сохраняет
//! исходный broadcast fallback.
//! Player chat `0x8FB01` сохраняет lazy silence, отдельные wrapping cooldown,
//! local distance/region/faction/private delivery и conditional chat-log.
//! Goods-link lookup `0x8FB03 -> 0x5FD04 -> 0x7FA07` сохраняет requester/link
//! identities и публикует World result клиенту как exact `0xBF80D` wire.
//! Public talk `0x8FB07/08` сохраняет silence/cooldown, exact setup-cost,
//! ordered item/money mutations, World `0x5FD07/08` и chat-log `0x6020B`.
//! Остальные ветви ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::player::{
    CiQingPacketConsumption, PlayerLeiTingDecodeBlock, PlayerMoneyDecrease, PlayerTalkChannel,
};
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const PLAYER_RENAME_REQUEST: u32 = 0x0008_fb05;
const PLAYER_CHAT_REQUEST: u32 = 0x0008_fb01;
const PLAYER_GOODS_LINK_REQUEST: u32 = 0x0008_fb03;
const WORLD_GOODS_LINK_RESPONSE: u32 = 0x0007_fa07;
const WORLD_PLAYER_RENAME_REQUEST: i32 = 0x0005_fd05;
const WORLD_PLAYER_RENAME_RESPONSE: u32 = 0x0007_fa0e;
const PLAYER_RENAME_RESPONSE: i32 = 0x000b_f80f;
const WORLD_INFO_DELIVERY: u32 = 0x0007_fa03;
const WORLD_TOP_INFO_DELIVERY: u32 = 0x0007_fa04;
const WORLD_TALK_REQUEST: u32 = 0x0008_fb07;
const COUNTRY_TALK_REQUEST: u32 = 0x0008_fb08;
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
    RenameForwarded {
        delivery: Result<i32, SendMessageError>,
    },
    RenameAccepted {
        new_name: Vec<u8>,
        delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
    RenameRejected {
        result: i8,
        delivery: i32,
    },
    InfoDelivered {
        target_id: i32,
        delivery: GameInfoDelivery,
    },
    PublicTalk(GamePublicTalkOutcome),
    PlayerChat(GamePlayerChatOutcome),
    GoodsLinkLookupForwarded {
        link_index: i32,
        delivery: Result<i32, SendMessageError>,
    },
    GoodsLinkLookupDelivered {
        delivery: i32,
    },
    LeiTingUpdated {
        client_delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerChatOutcome {
    Silenced {
        channel: i32,
        delivery: i32,
    },
    InvalidSender {
        channel: i32,
        sender: Vec<u8>,
    },
    ContentTooLong {
        channel: i32,
        length: usize,
    },
    Cooldown {
        channel: i32,
        delivery: i32,
    },
    RegionLevelRestricted {
        required_level: i32,
        player_level: u8,
        delivery: i32,
    },
    Factionless,
    Delivered {
        channel: i32,
        target_player_id: Option<i32>,
        client_deliveries: Vec<i32>,
        world_delivery: Option<Result<i32, SendMessageError>>,
        log_delivery: Option<Result<i32, SendMessageError>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePublicTalkCostReport {
    pub(crate) goods_base_index: u32,
    pub(crate) goods_required: i32,
    pub(crate) goods_consumption: Option<CiQingPacketConsumption>,
    pub(crate) goods_deliveries: Vec<i32>,
    pub(crate) money_required: u32,
    pub(crate) money: Option<PlayerMoneyDecrease>,
    pub(crate) money_deliveries: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GamePublicTalkOutcome {
    Silenced {
        country: bool,
        delivery: i32,
    },
    Cooldown {
        country: bool,
        delivery: i32,
    },
    InsufficientCost {
        country: bool,
        country_job: u8,
        goods_base_index: u32,
        goods_required: i32,
        matching_stacks: usize,
        money_required: u32,
        money_available: u32,
        delivery: i32,
    },
    Relayed {
        country: bool,
        country_job: u8,
        player_country: u8,
        content: Vec<u8>,
        cost: Option<GamePublicTalkCostReport>,
        world_delivery: Result<i32, SendMessageError>,
        log_delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameInfoDelivery {
    Broadcast(Result<i32, SendMessageError>),
    Region(Option<i32>),
    Player(i32),
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

fn peek_long(message: &mut CMessage) -> Option<i32> {
    let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
    let end = cursor.checked_add(4)?;
    Some(i32::from_le_bytes(wire.get(*cursor..end)?.try_into().ok()?))
}

fn format_legacy_level(template: &[u8], level: i32) -> Vec<u8> {
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let Some(marker) = template.windows(2).position(|window| window == b"%d") else {
        return template[..template.len().min(0xff)].to_vec();
    };
    let value = level.to_string();
    let mut result = Vec::with_capacity(template.len().saturating_add(value.len()));
    result.extend_from_slice(&template[..marker]);
    result.extend_from_slice(value.as_bytes());
    result.extend_from_slice(&template[marker + 2..]);
    result.truncate(0xff);
    result
}

fn send_player_chat_log(
    game: &CGame,
    player_id: i32,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
    log_type: u8,
    content: &[u8],
    receiver_id: Option<i32>,
) -> Result<i32, SendMessageError> {
    let mut log = CMessage::new(0x0006_020b);
    log.base_mut().add_byte(log_type);
    log.base_mut().add_long(player_id);
    log.base_mut().add_long(region_id);
    log.base_mut().add_long(tile_x);
    log.base_mut().add_long(tile_y);
    add_c_string(&mut log, content);
    if let Some(receiver_id) = receiver_id {
        log.base_mut().add_long(receiver_id);
    }
    log.send(game, false)
}

fn send_local_chat(
    message: &CMessage,
    game: &CGame,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<i32> {
    const OFFSETS: [(i32, i32); 9] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let area_width = game.globe_setup().area_width();
    let area_height = game.globe_setup().area_height();
    if area_width <= 0 || area_height <= 0 {
        return Vec::new();
    }
    let Some(region) = game.find_region(region_id) else {
        return Vec::new();
    };
    let center_x = tile_x / area_width;
    let center_y = tile_y / area_height;
    let mut player_ids = Vec::new();
    for (offset_x, offset_y) in OFFSETS {
        region.base().find_player_ids_in_area(
            center_x.wrapping_add(offset_x),
            center_y.wrapping_add(offset_y),
            &mut player_ids,
        );
    }
    player_ids
        .into_iter()
        .filter_map(|player_id| {
            let player = game.find_player(player_id)?;
            let player_x = player.shape().get_tile_x().ok()?;
            let player_y = player.shape().get_tile_y().ok()?;
            (player_x.wrapping_sub(tile_x).wrapping_abs() < area_width
                && player_y.wrapping_sub(tile_y).wrapping_abs() < area_height)
                .then(|| message.send_to_player(game.net_server(), player_id))
        })
        .collect()
}

fn public_talk_failure_message(country: bool) -> CMessage {
    let mut response = CMessage::new(if country { 0x000b_f815 } else { 0x000b_f814 });
    response.base_mut().add_byte(0);
    response
}

fn dispatch_player_chat(
    message: &mut CMessage,
    game: &mut CGame,
    mut now_milliseconds: impl FnMut() -> u32,
) -> Result<GameOtherMessageReport, GameOtherMessageError> {
    let message_type = PLAYER_CHAT_REQUEST;
    let channel = peek_long(message).ok_or(GameOtherMessageError::MissingField("chat channel"))?;
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id: 0,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        });
    };
    let Some(region_id) = message.region_id() else {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        });
    };
    let Some(player) = game.find_player_mut(player_id) else {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        });
    };
    if player.is_in_silence(now_milliseconds()) {
        let delivery =
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0330"))
                .send_to_player(game.net_server(), player_id);
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Silenced {
                channel,
                delivery,
            }),
        });
    }

    let decoded_channel = read_long(message, "chat channel")?;
    debug_assert_eq!(decoded_channel, channel);
    let _owner_type = read_long(message, "chat owner type")?;
    let _owner_id = read_long(message, "chat owner id")?;
    let target_name = (channel == 4).then(|| read_string(message, 0x100));
    let sender_name = read_string(message, 0x100);
    let content = read_string(message, 0x400);
    let (canonical_name, player_level, faction_id, tile_x, tile_y) = {
        let player = game
            .find_player(player_id)
            .expect("chat player остаётся live после decode");
        (
            player.player_name().to_vec(),
            player.level(),
            player.faction_id(),
            player.shape().get_tile_x().unwrap_or_default(),
            player.shape().get_tile_y().unwrap_or_default(),
        )
    };
    if sender_name != canonical_name {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::InvalidSender {
                channel,
                sender: sender_name,
            }),
        });
    }

    if matches!(channel, 0 | 1) && content.len() > 299 {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::ContentTooLong {
                channel,
                length: content.len(),
            }),
        });
    }

    let cooldown = |game: &mut CGame, channel: PlayerTalkChannel, now_ms: u32, interval_ms: u32| {
        game.find_player_mut(player_id)
            .expect("chat player остаётся live при timestamp mutation")
            .begin_talk(channel, now_ms, interval_ms)
    };
    let cooldown_notice = |game: &CGame, text: &[u8]| {
        colored_player_notice_message(0xffff_0000, 0, text)
            .send_to_player(game.net_server(), player_id)
    };

    match channel {
        0 => {
            let interval = game.globe_setup().normal_talk_interval_ms();
            if !cooldown(
                game,
                PlayerTalkChannel::Normal,
                now_milliseconds(),
                interval,
            ) {
                let delivery = cooldown_notice(game, b"you talk to fast!");
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Cooldown {
                        channel,
                        delivery,
                    }),
                });
            }
            message.set_message_type(0x000b_f801);
            let client_deliveries = send_local_chat(message, game, region_id, tile_x, tile_y);
            let log_delivery = game.log_system().normal_chat_enabled().then(|| {
                send_player_chat_log(
                    game, player_id, region_id, tile_x, tile_y, 0, &content, None,
                )
            });
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Delivered {
                    channel,
                    target_player_id: None,
                    client_deliveries,
                    world_delivery: None,
                    log_delivery,
                }),
            })
        }
        1 => {
            let interval = game.globe_setup().area_talk_interval_ms();
            if !cooldown(game, PlayerTalkChannel::Area, now_milliseconds(), interval) {
                let text = game.get_string_by_id(b"GS0049");
                let delivery = cooldown_notice(game, text);
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Cooldown {
                        channel,
                        delivery,
                    }),
                });
            }
            let required_level = game.globe_setup().region_chat_level_limit();
            if i32::from(player_level) < required_level {
                let text = format_legacy_level(game.get_string_by_id(b"GS0046"), required_level);
                let delivery = cooldown_notice(game, &text);
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(
                        GamePlayerChatOutcome::RegionLevelRestricted {
                            required_level,
                            player_level,
                            delivery,
                        },
                    ),
                });
            }
            message.set_message_type(0x000b_f801);
            let delivery = game
                .find_region(region_id)
                .map(|region| message.send_to_region(Some(region.base()), None, game))
                .unwrap_or_default();
            let log_delivery = game.log_system().region_chat_enabled().then(|| {
                send_player_chat_log(
                    game, player_id, region_id, tile_x, tile_y, 1, &content, None,
                )
            });
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Delivered {
                    channel,
                    target_player_id: None,
                    client_deliveries: vec![delivery],
                    world_delivery: None,
                    log_delivery,
                }),
            })
        }
        2 => {
            if faction_id <= 0 {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(
                        GamePlayerChatOutcome::Factionless,
                    ),
                });
            }
            let interval = game.globe_setup().union_talk_interval_ms();
            if !cooldown(game, PlayerTalkChannel::Union, now_milliseconds(), interval) {
                let text = game.get_string_by_id(b"GS0049");
                let delivery = cooldown_notice(game, text);
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Cooldown {
                        channel,
                        delivery,
                    }),
                });
            }
            message.set_message_type(0x0005_fd01);
            let world_delivery = Some(message.send(game, false));
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Delivered {
                    channel,
                    target_player_id: None,
                    client_deliveries: Vec::new(),
                    world_delivery,
                    log_delivery: None,
                }),
            })
        }
        4 => {
            let interval = game.globe_setup().private_talk_interval_ms();
            if !cooldown(
                game,
                PlayerTalkChannel::Private,
                now_milliseconds(),
                interval,
            ) {
                let text = game.get_string_by_id(b"GS0049");
                let delivery = cooldown_notice(game, text);
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Cooldown {
                        channel,
                        delivery,
                    }),
                });
            }
            let target_name = target_name.expect("private channel прочитал target name");
            let target_player_id = game
                .find_player_by_name(&target_name)
                .map(|player| player.player_id());
            let Some(target_player_id) = target_player_id else {
                message.set_message_type(0x0005_fd01);
                let world_delivery = Some(message.send(game, false));
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerChat(
                        GamePlayerChatOutcome::Delivered {
                            channel,
                            target_player_id: None,
                            client_deliveries: Vec::new(),
                            world_delivery,
                            log_delivery: None,
                        },
                    ),
                });
            };
            message.set_message_type(0x000b_f801);
            let mut client_deliveries =
                vec![message.send_to_player(game.net_server(), target_player_id)];
            let log_delivery = if target_player_id == player_id {
                None
            } else {
                client_deliveries.push(message.send_to_player(game.net_server(), player_id));
                game.log_system().private_chat_enabled().then(|| {
                    send_player_chat_log(
                        game,
                        player_id,
                        region_id,
                        tile_x,
                        tile_y,
                        5,
                        &content,
                        Some(target_player_id),
                    )
                })
            };
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::PlayerChat(GamePlayerChatOutcome::Delivered {
                    channel,
                    target_player_id: Some(target_player_id),
                    client_deliveries,
                    world_delivery: None,
                    log_delivery,
                }),
            })
        }
        _ => unreachable!("dispatcher пропускает только materialized player-chat channels"),
    }
}

fn dispatch_public_talk(
    message_type: u32,
    message: &mut CMessage,
    game: &mut CGame,
    now_milliseconds: impl FnOnce() -> u32,
) -> Result<GameOtherMessageReport, GameOtherMessageError> {
    let country_channel = message_type == COUNTRY_TALK_REQUEST;
    let content = read_string(message, 0x400);
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id: 0,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        });
    };
    let Some(player) = game.find_player(player_id) else {
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PlayerMissing,
        });
    };
    if player.silence_minutes() > 0 {
        let delivery =
            colored_player_notice_message(0xffff_0000, 0, game.get_string_by_id(b"GS0330"))
                .send_to_player(game.net_server(), player_id);
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PublicTalk(GamePublicTalkOutcome::Silenced {
                country: country_channel,
                delivery,
            }),
        });
    }

    let interval_ms = game.globe_setup().public_talk_interval_ms(country_channel);
    if !game
        .find_player_mut(player_id)
        .expect("public-talk player проверен до timestamp mutation")
        .begin_talk(
            if country_channel {
                PlayerTalkChannel::Country
            } else {
                PlayerTalkChannel::World
            },
            now_milliseconds(),
            interval_ms,
        )
    {
        let delivery =
            colored_player_notice_message(0xffff_0000, 0, game.get_string_by_id(b"GS0049"))
                .send_to_player(game.net_server(), player_id);
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PublicTalk(GamePublicTalkOutcome::Cooldown {
                country: country_channel,
                delivery,
            }),
        });
    }

    let (player_country, player_name, region_id, tile_x, tile_y, money_available) = {
        let player = game
            .find_player(player_id)
            .expect("public-talk player остаётся live после timestamp mutation");
        (
            player.country(),
            player.player_name().to_vec(),
            player.server_region_id().unwrap_or_default(),
            player.shape().get_tile_x().unwrap_or_default(),
            player.shape().get_tile_y().unwrap_or_default(),
            player.money(),
        )
    };
    let country_job = if country_channel {
        game.country_handler_mut()
            .country_mut(player_country)
            .map(|country| country.has_job(player_id))
            .unwrap_or(0)
    } else {
        0
    };
    let official = country_channel && country_job != 0;
    let goods_name = game
        .globe_setup()
        .public_talk_goods_name(country_channel)
        .to_vec();
    let goods_required = game.globe_setup().public_talk_goods_amount(country_channel);
    let money_required = game.globe_setup().public_talk_money(country_channel);
    let goods_base_index = game
        .goods_factory()
        .query_goods_id_by_original_name(Some(&goods_name));
    let matching_goods: Vec<_> = game
        .find_player(player_id)
        .expect("public-talk player остаётся live при cost lookup")
        .packet()
        .base()
        .get_goods_by_base_properties(goods_base_index)
        .into_iter()
        .map(|goods| goods.identity().ex_id)
        .collect();
    let enough_goods = goods_required == 0
        || (goods_required > 0 && matching_goods.len() >= goods_required as usize);
    if !official && (!enough_goods || money_available < money_required) {
        let delivery = public_talk_failure_message(country_channel)
            .send_to_player(game.net_server(), player_id);
        return Ok(GameOtherMessageReport {
            message_type,
            player_id,
            outcome: GameOtherMessageOutcome::PublicTalk(GamePublicTalkOutcome::InsufficientCost {
                country: country_channel,
                country_job,
                goods_base_index,
                goods_required,
                matching_stacks: matching_goods.len(),
                money_required,
                money_available,
                delivery,
            }),
        });
    }

    let cost = if official {
        None
    } else {
        let goods_consumption = if goods_required > 0 {
            let goods_id = matching_goods[0];
            game.find_player_mut(player_id)
                .expect("public-talk player остаётся live при goods mutation")
                .remove_packet_goods_by_id(goods_id, goods_required as u32)
        } else {
            None
        };
        let goods_deliveries = goods_consumption
            .as_ref()
            .map(|consumption| game.send_player_packet_consumption(consumption))
            .unwrap_or_default();
        let money = (money_required != 0)
            .then(|| game.decrease_player_money(player_id, money_required))
            .flatten();
        let money_deliveries = money
            .as_ref()
            .map(|change| game.send_player_money_decrease(player_id, &change.outcome))
            .unwrap_or_default();
        Some(GamePublicTalkCostReport {
            goods_base_index,
            goods_required,
            goods_consumption,
            goods_deliveries,
            money_required,
            money,
            money_deliveries,
        })
    };

    let mut relay = CMessage::new(if country_channel {
        0x0005_fd08
    } else {
        0x0005_fd07
    });
    relay.base_mut().add_byte(1);
    if country_channel {
        relay.base_mut().add_byte(country_job);
        relay.base_mut().add_byte(player_country);
    }
    add_c_string(&mut relay, &player_name);
    add_c_string(&mut relay, &content);
    let world_delivery = relay.send(game, false);

    let mut log = CMessage::new(0x0006_020b);
    log.base_mut().add_byte(if country_channel { 8 } else { 7 });
    log.base_mut().add_long(player_id);
    log.base_mut().add_long(region_id);
    log.base_mut().add_long(tile_x);
    log.base_mut().add_long(tile_y);
    add_c_string(&mut log, &content);
    let log_delivery = log.send(game, false);
    Ok(GameOtherMessageReport {
        message_type,
        player_id,
        outcome: GameOtherMessageOutcome::PublicTalk(GamePublicTalkOutcome::Relayed {
            country: country_channel,
            country_job,
            player_country,
            content,
            cost,
            world_delivery,
            log_delivery,
        }),
    })
}

pub(crate) fn dispatch_game_other_message(
    message: &mut CMessage,
    game: &mut CGame,
    mut now_milliseconds: impl FnMut() -> u32,
) -> Option<Result<GameOtherMessageReport, GameOtherMessageError>> {
    let message_type = message.message_type() as u32;
    if message_type == PLAYER_CHAT_REQUEST {
        let channel = peek_long(message)?;
        if matches!(channel, 0 | 1 | 2 | 4) {
            return Some(dispatch_player_chat(message, game, now_milliseconds));
        }
        return None;
    }
    if message_type == PLAYER_GOODS_LINK_REQUEST {
        let result = (|| {
            let link_index = read_long(message, "goods-link index")?;
            message.resolve_player_context(game);
            let Some(player_id) = message.player_id() else {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id: 0,
                    outcome: GameOtherMessageOutcome::PlayerMissing,
                });
            };
            let mut relay = CMessage::new(0x0005_fd04);
            relay.base_mut().add_long(player_id);
            relay.base_mut().add_long(link_index);
            let delivery = relay.send(game, false);
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::GoodsLinkLookupForwarded {
                    link_index,
                    delivery,
                },
            })
        })();
        return Some(result);
    }
    if matches!(message_type, WORLD_TALK_REQUEST | COUNTRY_TALK_REQUEST) {
        return Some(dispatch_public_talk(
            message_type,
            message,
            game,
            &mut now_milliseconds,
        ));
    }
    if message_type == PLAYER_RENAME_REQUEST {
        message.set_message_type(WORLD_PLAYER_RENAME_REQUEST);
        let delivery = message.send(game, false);
        return Some(Ok(GameOtherMessageReport {
            message_type,
            player_id: message.player_id().unwrap_or(0),
            outcome: GameOtherMessageOutcome::RenameForwarded { delivery },
        }));
    }
    if !matches!(
        message_type,
        PRIVATE_CHAT_DELIVERY
            | FACTION_CHAT_DELIVERY
            | WORLD_CHAT_DELIVERY
            | COUNTRY_CHAT_DELIVERY
            | COUNTRY_NOTICE_DELIVERY
            | WORLD_PLAYER_RENAME_RESPONSE
            | WORLD_INFO_DELIVERY
            | WORLD_TOP_INFO_DELIVERY
            | WORLD_GOODS_LINK_RESPONSE
            | WORLD_LEI_TING_UPDATE
    ) {
        return None;
    }
    if message_type == WORLD_GOODS_LINK_RESPONSE {
        let result = (|| {
            let player_id = read_long(message, "goods-link requester player id")?;
            if game.find_player(player_id).is_none() {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerMissing,
                });
            }
            message.set_message_type(0x000b_f80d);
            let delivery = message.send_to_player(game.net_server(), player_id);
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::GoodsLinkLookupDelivered { delivery },
            })
        })();
        return Some(result);
    }
    if message_type == WORLD_INFO_DELIVERY {
        message.set_message_type(0x000b_f803);
        let result = (|| {
            let target_id = read_long(message, "world info region id")?;
            let delivery = if target_id == 0 {
                GameInfoDelivery::Broadcast(message.send_all(game.current_net_server()))
            } else {
                GameInfoDelivery::Region(
                    game.find_region(target_id)
                        .map(|region| message.send_to_region(Some(region.base()), None, game)),
                )
            };
            Ok(GameOtherMessageReport {
                message_type,
                player_id: 0,
                outcome: GameOtherMessageOutcome::InfoDelivered {
                    target_id,
                    delivery,
                },
            })
        })();
        return Some(result);
    }
    if message_type == WORLD_TOP_INFO_DELIVERY {
        message.set_message_type(0x000b_f804);
        let result = (|| {
            let target_id = read_long(message, "world top-info player id")?;
            let delivery = if target_id == 0 {
                GameInfoDelivery::Broadcast(message.send_all(game.current_net_server()))
            } else {
                GameInfoDelivery::Player(message.send_to_player(game.net_server(), target_id))
            };
            Ok(GameOtherMessageReport {
                message_type,
                player_id: target_id,
                outcome: GameOtherMessageOutcome::InfoDelivered {
                    target_id,
                    delivery,
                },
            })
        })();
        return Some(result);
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
    if message_type == WORLD_PLAYER_RENAME_RESPONSE {
        let result = (|| {
            let player_id = read_long(message, "rename player id")?;
            let rename_result = read_char(message, "rename result")?;
            let new_name = read_string(message, 0x20);
            let Some(player) = game.find_player_mut(player_id) else {
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::PlayerMissing,
                });
            };
            message.set_message_type(PLAYER_RENAME_RESPONSE);
            if rename_result == 0 {
                player
                    .movement_shape_mut()
                    .base_object_mut()
                    .set_name(&new_name);
                let delivery = game.send_player_shape_around(player_id, None, message);
                return Ok(GameOtherMessageReport {
                    message_type,
                    player_id,
                    outcome: GameOtherMessageOutcome::RenameAccepted { new_name, delivery },
                });
            }
            let delivery = message.send_to_player(game.net_server(), player_id);
            Ok(GameOtherMessageReport {
                message_type,
                player_id,
                outcome: GameOtherMessageOutcome::RenameRejected {
                    result: rename_result,
                    delivery,
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
