//! Диспетчер прочих сообщений GameServer.
//!
//! Источник — точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/message/othermessage.cpp`. Реализованные ветви сохраняют коды и
//! байты сообщений, порядок клиентских и межсерверных отправок, ограничения
//! чата, частичные списания публичного чата, сценарные ответы, переименование,
//! межсерверные изменения навыков и уровня, а также обновление LeiTing.
//! Канал чата `8` связан с `ParseGMCommand`: сохраняются две GM-карты,
//! script variables, файловый GMLog и условный World-аудит `0x6020B`.
//! Channel `9` использует подтверждённый setup-gate и immediate global
//! `RunLine`; `0x8FBF9` сохраняет временной god-passport и live GM-map level 200.
//!
//! Все наблюдаемые эффекты выполняются синхронно в исходных ветвях. Результаты
//! отправок не управляют дальнейшим выполнением и фиксируются через `tracing`;
//! временные деревья отчётов и списки результатов отправки не создаются.
//! Отложенных эффектов у этого владельца нет, поэтому `GameEffectJournal` здесь
//! не используется. Все селекторы владельца материализованы typed dispatcher-ом.

use crate::gameserver::appserver::player::{PlayerLeiTingDecodeBlock, PlayerTalkChannel};
use crate::gameserver::appserver::legacycodec::LegacyReader;
use crate::gameserver::appserver::message::gmmessage::parse_gm_command;
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::{ScriptExecutionContext, legacy_atoi};
use crate::gameserver::gameserver::game::{
    CGame, colored_player_notice_message, player_skill_learned_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::date::TagTime;
use crate::public::tools::{add_game_error_log_text, put_string_to_file};
use crate::setup::gmlist::GmInfo;

const PLAYER_RENAME_REQUEST: u32 = 0x0008_fb05;
const PLAYER_CHAT_REQUEST: u32 = 0x0008_fb01;
const PLAYER_SCRIPT_DIALOG_RESPONSE: u32 = 0x0008_fb02;
const PLAYER_GOODS_LINK_REQUEST: u32 = 0x0008_fb03;
const CHANGE_GAME_SERVER_FAILED: u32 = 0x0008_fb04;
const WORLD_GOODS_LINK_RESPONSE: u32 = 0x0007_fa07;
const WORLD_GOODS_LINK_PUBLISH: u32 = 0x0007_fa06;
const WORLD_GM_FEEDBACK: u32 = 0x0007_fa05;
const WORLD_INCREMENT_SHOP_PAGE: u32 = 0x0007_fa12;
const WORLD_REMOTE_SKILL_ADD: u32 = 0x0007_fa08;
const WORLD_REMOTE_SKILL_DELETE: u32 = 0x0007_fa09;
const WORLD_REMOTE_PLAYER_GOODS_DELETE: u32 = 0x0007_fa0a;
const WORLD_REMOTE_LEVEL_SET: u32 = 0x0007_fa0b;
const WORLD_KICK_ALL_PLAYERS: u32 = 0x0007_fa0c;
const WORLD_START_REGION_CLEAR: u32 = 0x0007_fa13;
const PLAYER_NPC_NAME_LIST_REQUEST: u32 = 0x0008_fb06;
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
const WORLD_HONOR_ELIMINATE_ACKNOWLEDGEMENT: u32 = 0x0007_fa16;
const WORLD_SCRIPT_CONTINUE: u32 = 0x0007_fa15;
const PLAYER_GOD_AUTH_REQUEST: u32 = 0x0008_fbf9;
const GM_GOD_LEVEL: i32 = 200;
const GOD_AUTH_PREFIX: &[u8] = b"sdf!@#$aurora-348sklhw9lsdhf!@Dfsdf89s*LKHL@#$@#;sldjkfnv/z[q";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOtherMessageError {
    MissingPlayerId,
    MissingField(&'static str),
    LeiTing(PlayerLeiTingDecodeBlock),
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

fn read_word(message: &mut CMessage, field: &'static str) -> Result<u16, GameOtherMessageError> {
    message
        .base_mut()
        .get_word()
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
    LegacyReader::at(wire, *cursor).ok()?.read_i32().ok()
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

fn format_skill_feedback(template: &[u8], player: &[u8], skill: &[u8], level: u16) -> Vec<u8> {
    let template = template.split(|byte| *byte == 0).next().unwrap_or_default();
    let strings = [player, skill];
    let mut string_index = 0;
    let mut offset = 0;
    let mut result = Vec::with_capacity(template.len().saturating_add(32));
    while offset < template.len() && result.len() < 0xff {
        if template.get(offset..offset + 2) == Some(b"%%") {
            result.push(b'%');
            offset += 2;
        } else if template.get(offset..offset + 2) == Some(b"%s") {
            if let Some(value) = strings.get(string_index) {
                result.extend_from_slice(value);
                string_index += 1;
            }
            offset += 2;
        } else if template.get(offset..offset + 2) == Some(b"%d") {
            result.extend_from_slice(level.to_string().as_bytes());
            offset += 2;
        } else {
            result.push(template[offset]);
            offset += 1;
        }
    }
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
) {
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
        return;
    }
    let Some(region) = game.find_region(region_id) else {
        return;
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
    for player_id in player_ids {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        let (Ok(player_x), Ok(player_y)) = (
            player.shape().get_tile_x(),
            player.shape().get_tile_y(),
        ) else {
            continue;
        };
        if player_x.wrapping_sub(tile_x).wrapping_abs() < area_width
            && player_y.wrapping_sub(tile_y).wrapping_abs() < area_height
        {
            let _ = message.send_to_player(game.net_server(), player_id);
        }
    }
}

fn public_talk_failure_message(country: bool) -> CMessage {
    let mut response = CMessage::new(if country { 0x000b_f815 } else { 0x000b_f814 });
    response.base_mut().add_byte(0);
    response
}

fn format_gm_command_log(
    player_name: &[u8],
    player_level: u8,
    region_id: i32,
    command: &[u8],
    localized: &[u8],
) -> Vec<u8> {
    let mut text = Vec::with_capacity(
        player_name.len() + command.len() + localized.len() + 48,
    );
    text.push(b'\'');
    text.extend_from_slice(player_name);
    text.extend_from_slice(b"' (lvl:");
    text.extend_from_slice(player_level.to_string().as_bytes());
    text.extend_from_slice(b" map:");
    text.extend_from_slice(region_id.to_string().as_bytes());
    text.extend_from_slice(b") => ");
    text.extend_from_slice(command);
    text.extend_from_slice(b" [");
    text.extend_from_slice(localized.split(|byte| *byte == 0).next().unwrap_or_default());
    text.push(b']');
    text
}

fn dispatch_gm_chat<Runtime: ScriptFunctionRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<(), GameOtherMessageError> {
    let message_type = PLAYER_CHAT_REQUEST;
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        tracing::trace!(message_type, "у GM-команды нет игрока");
        return Ok(());
    };
    let Some(region_id) = message
        .region_id()
        .filter(|region_id| game.find_region(*region_id).is_some())
    else {
        tracing::trace!(message_type, player_id, "у GM-команды нет live-региона");
        return Ok(());
    };
    let Some(player) = game.find_player_mut(player_id) else {
        tracing::trace!(message_type, player_id, "игрок GM-команды не найден");
        return Ok(());
    };
    if player.is_in_silence(runtime.now_milliseconds()) {
        let _ = colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0330"))
            .send_to_player(game.net_server(), player_id);
        tracing::trace!(message_type, player_id, "GM-команда запрещена молчанием");
        return Ok(());
    }

    let channel = read_long(message, "chat channel")?;
    debug_assert_eq!(channel, 8);
    let _owner_type = read_long(message, "chat owner type")?;
    let _owner_id = read_long(message, "chat owner id")?;
    let _sender_name = read_string(message, 0x200);
    let mut command = read_string(message, 0x200);
    if !parse_gm_command(game, runtime, player_id, region_id, &mut command) {
        tracing::trace!(message_type, player_id, "GM-команда отклонена");
        return Ok(());
    }

    let (player_name, player_level, tile_x, tile_y) = {
        let player = game
            .find_player(player_id)
            .expect("GM-command player остаётся live после script queue");
        (
            player.player_name().to_vec(),
            player.level(),
            player.shape().get_tile_x().unwrap_or_default(),
            player.shape().get_tile_y().unwrap_or_default(),
        )
    };
    let gm_level = game
        .gm_list()
        .player_gm_info()
        .get(&player_name)
        .or_else(|| game.gm_list().gm_info().get(&player_name))
        .map(|info| info.level)
        .unwrap_or_default();
    if gm_level != GM_GOD_LEVEL {
        let log = format_gm_command_log(
            &player_name,
            player_level,
            region_id,
            &command,
            game.get_string_by_id(b"GS0044"),
        );
        put_string_to_file("GMLog", &log);
        if game.log_system().gm_command_enabled() {
            let _ = send_player_chat_log(
                game,
                player_id,
                region_id,
                tile_x,
                tile_y,
                6,
                &command,
                None,
            );
        }
    }
    tracing::trace!(message_type, player_id, gm_level, "GM-команда принята");
    Ok(())
}

fn dispatch_god_auth(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), GameOtherMessageError> {
    let message_type = PLAYER_GOD_AUTH_REQUEST;
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        tracing::trace!(message_type, "у god-auth нет игрока");
        return Ok(());
    };
    let Some(player_name) = game
        .find_player(player_id)
        .map(|player| player.player_name().to_vec())
    else {
        tracing::trace!(message_type, player_id, "игрок god-auth не найден");
        return Ok(());
    };
    match read_char(message, "god-auth operation")? {
        0 => {
            let removed = game.gm_list_mut().remove_gm(&player_name).is_some();
            tracing::trace!(message_type, player_id, removed, "god-auth снят");
        }
        1 => {
            let supplied = read_string(message, 0x400);
            let now = TagTime::local_now();
            let passport = game.gm_list().god_passport();
            let mut expected = Vec::with_capacity(GOD_AUTH_PREFIX.len() + passport.len() + 16);
            expected.extend_from_slice(GOD_AUTH_PREFIX);
            expected.extend_from_slice(
                format!("{:04}{:02}{:02}{:02}", now.year, now.month, now.day, now.hour)
                    .as_bytes(),
            );
            expected.extend_from_slice(passport);
            if supplied == expected && !game.gm_list().gm_info().contains_key(&player_name) {
                let _ = game.gm_list_mut().insert_gm(GmInfo {
                    name: player_name,
                    level: GM_GOD_LEVEL,
                });
                tracing::trace!(message_type, player_id, "god-auth принят");
            } else {
                tracing::trace!(message_type, player_id, "god-auth отклонён");
            }
        }
        operation => {
            tracing::trace!(message_type, player_id, operation, "неизвестная god-auth операция");
        }
    }
    Ok(())
}

fn dispatch_client_script_chat<Runtime: ScriptFunctionRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<(), GameOtherMessageError> {
    let message_type = PLAYER_CHAT_REQUEST;
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        tracing::trace!(message_type, "у клиентской script-строки нет игрока");
        return Ok(());
    };
    let Some(region_id) = message
        .region_id()
        .filter(|region_id| game.find_region(*region_id).is_some())
    else {
        tracing::trace!(message_type, player_id, "у клиентской script-строки нет live-региона");
        return Ok(());
    };
    let Some(player) = game.find_player_mut(player_id) else {
        tracing::trace!(message_type, player_id, "игрок клиентской script-строки не найден");
        return Ok(());
    };
    if player.is_in_silence(runtime.now_milliseconds()) {
        let _ = colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0330"))
            .send_to_player(game.net_server(), player_id);
        tracing::trace!(message_type, player_id, "клиентская script-строка запрещена молчанием");
        return Ok(());
    }

    let channel = read_long(message, "chat channel")?;
    debug_assert_eq!(channel, 9);
    let _owner_type = read_long(message, "chat owner type")?;
    let _owner_id = read_long(message, "chat owner id")?;
    let sender_name = read_string(message, 0x100);
    let line = read_string(message, 0x400);
    let canonical_name = game
        .find_player(player_id)
        .expect("client-script player остаётся live после decode")
        .player_name()
        .to_vec();
    if sender_name != canonical_name {
        tracing::warn!(message_type, player_id, channel, "имя отправителя client-script не совпало");
        return Ok(());
    }
    if !game.globe_setup().allow_client_run_script() {
        tracing::trace!(message_type, player_id, "client-script отключён setup-ом");
        return Ok(());
    }

    let disposition = game.run_script_line(
        &line,
        ScriptExecutionContext {
            player_id: Some(player_id),
            region_id: Some(region_id),
            npc_id: None,
            ..ScriptExecutionContext::default()
        },
        runtime,
    );
    tracing::trace!(message_type, player_id, ?disposition, "client-script строка исполнена");
    Ok(())
}

fn dispatch_player_chat(
    message: &mut CMessage,
    game: &mut CGame,
    mut now_milliseconds: impl FnMut() -> u32,
) -> Result<(), GameOtherMessageError> {
    let message_type = PLAYER_CHAT_REQUEST;
    let channel = peek_long(message).ok_or(GameOtherMessageError::MissingField("chat channel"))?;
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        tracing::trace!(message_type, "у сообщения чата нет игрока");
        return Ok(());
    };
    let Some(region_id) = message.region_id() else {
        tracing::trace!(message_type, player_id, "у сообщения чата нет региона");
        return Ok(());
    };
    let Some(player) = game.find_player_mut(player_id) else {
        tracing::trace!(message_type, player_id, "игрок чата не найден");
        return Ok(());
    };
    if player.is_in_silence(now_milliseconds()) {
        let _ = colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0330"))
            .send_to_player(game.net_server(), player_id);
        tracing::trace!(message_type, player_id, channel, "чат запрещён молчанием");
        return Ok(());
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
        tracing::warn!(message_type, player_id, channel, "имя отправителя чата не совпало");
        return Ok(());
    }

    if matches!(channel, 0 | 1) && content.len() > 299 {
        tracing::trace!(message_type, player_id, channel, length = content.len(), "сообщение чата слишком длинное");
        return Ok(());
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
                let _ = cooldown_notice(game, b"you talk to fast!");
                tracing::trace!(message_type, player_id, channel, "не истёк интервал чата");
                return Ok(());
            }
            message.set_message_type(0x000b_f801);
            send_local_chat(message, game, region_id, tile_x, tile_y);
            if game.log_system().normal_chat_enabled() {
                let _ = send_player_chat_log(
                    game, player_id, region_id, tile_x, tile_y, 0, &content, None,
                );
            }
            tracing::trace!(message_type, player_id, channel, "чат доставлен");
            Ok(())
        }
        1 => {
            let interval = game.globe_setup().area_talk_interval_ms();
            if !cooldown(game, PlayerTalkChannel::Area, now_milliseconds(), interval) {
                let text = game.get_string_by_id(b"GS0049");
                let _ = cooldown_notice(game, text);
                tracing::trace!(message_type, player_id, channel, "не истёк интервал чата");
                return Ok(());
            }
            let required_level = game.globe_setup().region_chat_level_limit();
            if i32::from(player_level) < required_level {
                let text = format_legacy_level(game.get_string_by_id(b"GS0046"), required_level);
                let _ = cooldown_notice(game, &text);
                tracing::trace!(message_type, player_id, channel, required_level, player_level, "уровень игрока недостаточен для чата региона");
                return Ok(());
            }
            message.set_message_type(0x000b_f801);
            let _ = game
                .find_region(region_id)
                .map(|region| message.send_to_region(Some(region.base()), None, game))
                .unwrap_or_default();
            if game.log_system().region_chat_enabled() {
                let _ = send_player_chat_log(
                    game, player_id, region_id, tile_x, tile_y, 1, &content, None,
                );
            }
            tracing::trace!(message_type, player_id, channel, "чат доставлен");
            Ok(())
        }
        2 => {
            if faction_id <= 0 {
                tracing::trace!(message_type, player_id, channel, "у игрока нет фракции");
                return Ok(());
            }
            let interval = game.globe_setup().union_talk_interval_ms();
            if !cooldown(game, PlayerTalkChannel::Union, now_milliseconds(), interval) {
                let text = game.get_string_by_id(b"GS0049");
                let _ = cooldown_notice(game, text);
                tracing::trace!(message_type, player_id, channel, "не истёк интервал чата");
                return Ok(());
            }
            message.set_message_type(0x0005_fd01);
            let _ = message.send(game, false);
            tracing::trace!(message_type, player_id, channel, "чат передан WorldServer");
            Ok(())
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
                let _ = cooldown_notice(game, text);
                tracing::trace!(message_type, player_id, channel, "не истёк интервал чата");
                return Ok(());
            }
            let target_name = target_name.expect("private channel прочитал target name");
            let target_player_id = game
                .find_player_by_name(&target_name)
                .map(|player| player.player_id());
            let Some(target_player_id) = target_player_id else {
                message.set_message_type(0x0005_fd01);
                let _ = message.send(game, false);
                tracing::trace!(message_type, player_id, channel, "личный чат передан WorldServer");
                return Ok(());
            };
            message.set_message_type(0x000b_f801);
            let _ = message.send_to_player(game.net_server(), target_player_id);
            if target_player_id != player_id {
                let _ = message.send_to_player(game.net_server(), player_id);
                if game.log_system().private_chat_enabled() {
                    let _ = send_player_chat_log(
                        game,
                        player_id,
                        region_id,
                        tile_x,
                        tile_y,
                        5,
                        &content,
                        Some(target_player_id),
                    );
                }
            }
            tracing::trace!(message_type, player_id, channel, target_player_id, "личный чат доставлен");
            Ok(())
        }
        _ => unreachable!("dispatcher пропускает только materialized player-chat channels"),
    }
}

fn dispatch_public_talk(
    message_type: u32,
    message: &mut CMessage,
    game: &mut CGame,
    now_milliseconds: impl FnOnce() -> u32,
) -> Result<(), GameOtherMessageError> {
    let country_channel = message_type == COUNTRY_TALK_REQUEST;
    let content = read_string(message, 0x400);
    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        tracing::trace!(message_type, "у публичного сообщения нет игрока");
        return Ok(());
    };
    let Some(player) = game.find_player(player_id) else {
        tracing::trace!(message_type, player_id, "игрок публичного сообщения не найден");
        return Ok(());
    };
    if player.silence_minutes() > 0 {
        let _ = colored_player_notice_message(0xffff_0000, 0, game.get_string_by_id(b"GS0330"))
            .send_to_player(game.net_server(), player_id);
        tracing::trace!(message_type, player_id, country_channel, "публичный чат запрещён молчанием");
        return Ok(());
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
        let _ = colored_player_notice_message(0xffff_0000, 0, game.get_string_by_id(b"GS0049"))
            .send_to_player(game.net_server(), player_id);
        tracing::trace!(message_type, player_id, country_channel, "не истёк интервал публичного чата");
        return Ok(());
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
        let _ = public_talk_failure_message(country_channel)
            .send_to_player(game.net_server(), player_id);
        tracing::trace!(
            message_type,
            player_id,
            country_channel,
            country_job,
            goods_base_index,
            goods_required,
            matching_stacks = matching_goods.len(),
            money_required,
            money_available,
            "недостаточно средств для публичного чата"
        );
        return Ok(());
    }

    if !official {
        let goods_consumption = if goods_required > 0 {
            let goods_id = matching_goods[0];
            game.find_player_mut(player_id)
                .expect("public-talk player остаётся live при goods mutation")
                .remove_packet_goods_by_id(goods_id, goods_required as u32)
        } else {
            None
        };
        let _ = goods_consumption
            .as_ref()
            .map(|consumption| game.send_player_packet_consumption(consumption))
            .unwrap_or_default();
        let money = (money_required != 0)
            .then(|| game.decrease_player_money(player_id, money_required))
            .flatten();
        let _ = money
            .as_ref()
            .map(|change| game.send_player_money_decrease(player_id, &change.outcome))
            .unwrap_or_default();
    }

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
    let _ = relay.send(game, false);

    let mut log = CMessage::new(0x0006_020b);
    log.base_mut().add_byte(if country_channel { 8 } else { 7 });
    log.base_mut().add_long(player_id);
    log.base_mut().add_long(region_id);
    log.base_mut().add_long(tile_x);
    log.base_mut().add_long(tile_y);
    add_c_string(&mut log, &content);
    let _ = log.send(game, false);
    tracing::trace!(message_type, player_id, country_channel, country_job, player_country, "публичный чат передан");
    Ok(())
}

pub(crate) fn dispatch_game_other_message<Runtime: ScriptFunctionRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameOtherMessageError>> {
    let message_type = message.message_type() as u32;
    if message_type == PLAYER_GOD_AUTH_REQUEST {
        return Some(dispatch_god_auth(message, game));
    }
    if message_type == PLAYER_SCRIPT_DIALOG_RESPONSE {
        message.resolve_player_context(game);
        let player_id = message.player_id();
        let context_present =
            player_id
                .zip(message.region_id())
                .is_some_and(|(player_id, region_id)| {
                    game.find_player(player_id).is_some() && game.find_region(region_id).is_some()
                });
        let Some(player_id) = player_id.filter(|_| context_present) else {
            tracing::trace!(message_type, player_id, "нет контекста ответа сценарному окну");
            return Some(Ok(()));
        };
        let result = (|| {
            let script_id = read_long(message, "script dialog id")?;
            let mode = read_long(message, "script dialog mode")?;
            match mode {
                1 => {
                    let text = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
                    let value = legacy_atoi(&text);
                    let _ = game.continue_player_script(script_id, player_id, value);
                    tracing::trace!(message_type, player_id, script_id, mode, value, "сценарий продолжен ответом строки");
                }
                -1 => {
                    let _ = game.delete_player_script(script_id, player_id, false);
                    tracing::trace!(message_type, player_id, script_id, mode, "сценарий удалён");
                }
                0 => {
                    let value = read_long(message, "script dialog numeric response")?;
                    let _ = game.continue_player_script(script_id, player_id, value);
                    tracing::trace!(message_type, player_id, script_id, mode, value, "сценарий продолжен числовым ответом");
                }
                _ => tracing::trace!(message_type, player_id, script_id, mode, "режим сценарного окна проигнорирован"),
            }
            Ok(())
        })();
        return Some(result);
    }
    if message_type == CHANGE_GAME_SERVER_FAILED {
        message.resolve_player_context(game);
        let player = message.player_id().and_then(|player_id| {
            game.find_player(player_id)
                .map(|player| (player_id, player.player_name().to_vec()))
        });
        if let Some((player_id, player_name)) = &player {
            let text = format!(
                "({player_id}){} CONNECTGAMESERVER For Changing GS FAILED!!!!",
                String::from_utf8_lossy(player_name)
            );
            add_game_error_log_text(text.as_bytes());
        }
        let socket_id = message.socket_id();
        let quit_result = game
            .net_server()
            .command_handle()
            .quit_by_socket_id(socket_id);
        let resolved_player_id = player.as_ref().map(|(player_id, _)| *player_id);
        tracing::warn!(message_type, socket_id, player_id = resolved_player_id, quit_result, error_logged = player.is_some(), "смена GameServer завершилась ошибкой");
        return Some(Ok(()));
    }
    if message_type == WORLD_SCRIPT_CONTINUE {
        let result = (|| {
            let requested_player_id = read_long(message, "script player id")?;
            let script_id = read_long(message, "script id")?;
            let value = read_long(message, "script continuation value")?;
            let player_present = game.find_player(requested_player_id).is_some();
            let _ =
                game.continue_player_script_function(script_id, requested_player_id, 9314, value);
            tracing::trace!(message_type, requested_player_id, player_present, script_id, value, "продолжен сценарий от WorldServer");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == PLAYER_CHAT_REQUEST {
        let channel = peek_long(message)?;
        if channel == 8 {
            return Some(dispatch_gm_chat(message, game, runtime));
        }
        if channel == 9 {
            return Some(dispatch_client_script_chat(message, game, runtime));
        }
        if matches!(channel, 0 | 1 | 2 | 4) {
            return Some(dispatch_player_chat(message, game, || {
                runtime.now_milliseconds()
            }));
        }
        return None;
    }
    if message_type == PLAYER_GOODS_LINK_REQUEST {
        let result = (|| {
            let link_index = read_long(message, "goods-link index")?;
            message.resolve_player_context(game);
            let Some(player_id) = message.player_id() else {
                tracing::trace!(message_type, "у запроса ссылки на предмет нет игрока");
                return Ok(());
            };
            let mut relay = CMessage::new(0x0005_fd04);
            relay.base_mut().add_long(player_id);
            relay.base_mut().add_long(link_index);
            let _ = relay.send(game, false);
            tracing::trace!(message_type, player_id, link_index, "запрос ссылки на предмет передан WorldServer");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == PLAYER_NPC_NAME_LIST_REQUEST {
        message.resolve_player_context(game);
        let Some(player_id) = message
            .player_id()
            .filter(|player_id| game.find_player(*player_id).is_some())
        else {
            tracing::trace!(message_type, player_id = message.player_id(), "игрок списка NPC не найден");
            return Some(Ok(()));
        };
        if let Some(region) = message
            .region_id()
            .and_then(|region_id| game.find_region(region_id))
        {
            let region = region.base();
            let count = region.npc_name_list_count();
            let names = region.npc_name_list();
            let mut response = CMessage::new(0x000b_f813);
            response.base_mut().add_long(count);
            response.base_mut().add(names);
            let _ = response.send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, count, bytes = names.len(), "список имён NPC доставлен");
        } else {
            tracing::trace!(message_type, player_id, "регион списка NPC не найден");
        }
        return Some(Ok(()));
    }
    if matches!(message_type, WORLD_TALK_REQUEST | COUNTRY_TALK_REQUEST) {
        return Some(dispatch_public_talk(
            message_type,
            message,
            game,
            &mut || runtime.now_milliseconds(),
        ));
    }
    if message_type == PLAYER_RENAME_REQUEST {
        message.set_message_type(WORLD_PLAYER_RENAME_REQUEST);
        let _ = message.send(game, false);
        tracing::trace!(message_type, player_id = message.player_id(), "запрос переименования передан WorldServer");
        return Some(Ok(()));
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
            | WORLD_GM_FEEDBACK
            | WORLD_GOODS_LINK_PUBLISH
            | WORLD_GOODS_LINK_RESPONSE
            | WORLD_INCREMENT_SHOP_PAGE
            | WORLD_REMOTE_SKILL_ADD
            | WORLD_REMOTE_SKILL_DELETE
            | WORLD_REMOTE_PLAYER_GOODS_DELETE
            | WORLD_REMOTE_LEVEL_SET
            | WORLD_KICK_ALL_PLAYERS
            | WORLD_START_REGION_CLEAR
            | WORLD_HONOR_ELIMINATE_ACKNOWLEDGEMENT
            | WORLD_LEI_TING_UPDATE
    ) {
        return None;
    }
    if message_type == WORLD_GOODS_LINK_PUBLISH {
        let result = (|| {
            let link_type = read_long(message, "goods-link publish type")?;
            let first_parameter = read_long(message, "goods-link publish parameter")?;
            let player_id = read_long(message, "goods-link publish player id")?;
            let Some(player) = game.find_player(player_id) else {
                tracing::trace!(message_type, player_id, "игрок публикации ссылки не найден");
                return Ok(());
            };
            let region_id = player.server_region_id();
            let tile_x = player.shape().get_tile_x().ok();
            let tile_y = player.shape().get_tile_y().ok();
            match link_type {
                0 => {
                    message.set_message_type(0x000b_f801);
                    if let Some(((region_id, tile_x), tile_y)) = region_id
                        .zip(tile_x)
                        .zip(tile_y)
                    {
                        send_local_chat(message, game, region_id, tile_x, tile_y);
                    }
                }
                1 => {
                    message.set_message_type(0x000b_f801);
                    if let Some(region) =
                        region_id.and_then(|region_id| game.find_region(region_id))
                    {
                        let _ = message.send_to_region(Some(region.base()), None, game);
                    }
                }
                _ => {}
            }
            tracing::trace!(message_type, player_id, link_type, first_parameter, "ссылка на предмет опубликована");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_REMOTE_SKILL_ADD {
        let result = (|| {
            let player_name = read_string(message, 0x100);
            let Some(player_id) = game
                .find_player_by_name(&player_name)
                .map(|player| player.player_id())
            else {
                tracing::trace!(message_type, "игрок добавляемого навыка не найден");
                return Ok(());
            };
            let skill_name = read_string(message, 0x100);
            let level = read_word(message, "remote skill level")?;
            let requester_id = read_long(message, "remote skill requester id")?;
            let target_map_id = read_long(message, "remote skill target map id")?;
            let mutation = game.add_remote_player_skill(player_id, &skill_name, level);
            let client_delivery = mutation.as_ref().and_then(|mutation| {
                player_skill_learned_message(
                    0x000b_f71d,
                    mutation.skill_id,
                    mutation.skill_level,
                    i32::from(level),
                    &skill_name,
                    game.skill_factory(),
                    false,
                )
                .map(|response| response.send_to_player(game.net_server(), player_id))
            });
            let feedback_text = format_skill_feedback(
                game.get_string_by_id(b"GS0048"),
                &player_name,
                &skill_name,
                level,
            );
            let mut feedback = CMessage::new(0x0005_fd02);
            feedback.base_mut().add_long(target_map_id);
            feedback.base_mut().add_long(requester_id);
            feedback.base_mut().add_long(-1);
            feedback.base_mut().add_long(0);
            add_c_string(&mut feedback, &feedback_text);
            let feedback_delivery = feedback.send(game, false);
            tracing::trace!(message_type, player_id, level, legacy_result = mutation.is_some_and(|mutation| mutation.legacy_result), client_delivery, ?feedback_delivery, "удалённо добавлен навык");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_REMOTE_SKILL_DELETE {
        let result = (|| {
            let player_name = read_string(message, 0x100);
            let Some(player_id) = game
                .find_player_by_name(&player_name)
                .map(|player| player.player_id())
            else {
                tracing::trace!(message_type, "игрок удаляемого навыка не найден");
                return Ok(());
            };
            let skill_name = read_string(message, 0x100);
            let mutation = game
                .delete_remote_player_skill(player_id, &skill_name)
                .expect("remote skill player проверен до mutation");
            let mut response = CMessage::new(0x000b_f71e);
            add_c_string(&mut response, &skill_name);
            let client_delivery = response.send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, legacy_result = mutation.legacy_result, client_delivery, "удалённо удалён навык");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_REMOTE_PLAYER_GOODS_DELETE {
        let player_name = read_string(message, 0x100);
        let Some(player_id) = game
            .find_player_by_name(&player_name)
            .map(|player| player.player_id())
        else {
            tracing::trace!(message_type, "игрок удаляемого предмета не найден");
            return Some(Ok(()));
        };
        let goods_name = read_string(message, 0x100);
        let requested = match read_long(message, "remote goods amount") {
            Ok(requested) => requested as u32,
            Err(error) => return Some(Err(error)),
        };
        let base_index = game
            .goods_factory()
            .query_goods_id_by_original_name(Some(&goods_name));
        let removed = game.delete_script_goods(player_id, base_index, requested, runtime);
        tracing::trace!(message_type, player_id, requested, removed, "удалённо удалены предметы");
        return Some(Ok(()));
    }
    if message_type == WORLD_REMOTE_LEVEL_SET {
        let result = (|| {
            let player_name = read_string(message, 0x100);
            let Some(player_id) = game
                .find_player_by_name(&player_name)
                .map(|player| player.player_id())
            else {
                tracing::trace!(message_type, "игрок изменяемого уровня не найден");
                return Ok(());
            };
            let level = read_char(message, "remote player level")? as u8;
            let mutation = game
                .find_player_mut(player_id)
                .expect("remote level player проверен до mutation")
                .apply_remote_level(level);
            let faction_delivery = (mutation.previous_level != level && mutation.faction_id > 0)
                .then(|| {
                    let mut faction = CMessage::new(0x0006_012a);
                    faction.base_mut().add_long(mutation.faction_id);
                    faction.base_mut().add_long(mutation.player_id);
                    faction.base_mut().add_long(1);
                    faction.base_mut().add_ulong(u32::from(mutation.level));
                    faction.send(game, false)
                });
            let next_experience = game.player_list().level_experience(level);
            let mut response = CMessage::new(0x000b_f708);
            response.base_mut().add_byte(mutation.level);
            response.base_mut().add_ulong(0);
            response.base_mut().add_ulong(next_experience);
            let client_delivery = response.send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, previous_level = mutation.previous_level, level, next_experience, ?faction_delivery, client_delivery, "удалённо изменён уровень");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_KICK_ALL_PLAYERS {
        let kicked = game.kick_all_players();
        tracing::trace!(message_type, count = kicked, "все игроки отключены");
        return Some(Ok(()));
    }
    if message_type == WORLD_START_REGION_CLEAR {
        let result = (|| {
            let region_id = read_long(message, "clear-player region id")?;
            let buffer_seconds = read_long(message, "clear-player buffer seconds")?;
            let start = game.find_region(region_id).is_some().then(|| {
                game.start_region_clear_player(
                    region_id,
                    buffer_seconds,
                    runtime.now_milliseconds(),
                )
                .expect("clear-player region проверен до timer mutation")
            });
            tracing::trace!(message_type, region_id, buffer_seconds, scheduled = start.is_some(), "очистка региона запланирована");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_HONOR_ELIMINATE_ACKNOWLEDGEMENT {
        let result = (|| {
            let player_id = read_long(message, "honor eliminate player id")?;
            let accepted = read_char(message, "honor eliminate acknowledgement")? != 0;
            let mutation = accepted
                .then(|| {
                    game.find_player_mut(player_id)
                        .map(|player| player.acknowledge_honor_eliminate())
                })
                .flatten();
            tracing::trace!(message_type, player_id, accepted, mutated = mutation.is_some(), "подтверждены очки устранения");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_INCREMENT_SHOP_PAGE {
        let result = (|| {
            let player_id = read_long(message, "increment-shop page player id")?;
            if game.find_player(player_id).is_none() {
                tracing::trace!(message_type, player_id, "игрок страницы магазина не найден");
                return Ok(());
            }
            message.set_message_type(0x000c_0405);
            let delivery = message.send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, delivery, "страница магазина доставлена");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_GM_FEEDBACK {
        let result = (|| {
            let target_map_id = read_long(message, "GM feedback target map id")?;
            let player_id = read_long(message, "GM feedback requester player id")?;
            let color = read_long(message, "GM feedback color")? as u32;
            let notice_type = read_long(message, "GM feedback notice type")? as u32;
            let text = read_string(message, 0x100);
            if game.find_player(player_id).is_none() {
                tracing::trace!(message_type, player_id, "получатель ответа GM не найден");
                return Ok(());
            }
            let delivery = colored_player_notice_message(color, notice_type, &text)
                .send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, target_map_id, color, notice_type, delivery, "ответ GM доставлен");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_GOODS_LINK_RESPONSE {
        let result = (|| {
            let player_id = read_long(message, "goods-link requester player id")?;
            if game.find_player(player_id).is_none() {
                tracing::trace!(message_type, player_id, "получатель ссылки на предмет не найден");
                return Ok(());
            }
            message.set_message_type(0x000b_f80d);
            let delivery = message.send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, delivery, "ссылка на предмет доставлена");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_INFO_DELIVERY {
        message.set_message_type(0x000b_f803);
        let result = (|| {
            let target_id = read_long(message, "world info region id")?;
            if target_id == 0 {
                let _ = message.send_all(game.current_net_server());
            } else {
                if let Some(region) = game.find_region(target_id) {
                    let _ = message.send_to_region(Some(region.base()), None, game);
                }
            }
            tracing::trace!(message_type, target_id, "информационное сообщение доставлено");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_TOP_INFO_DELIVERY {
        message.set_message_type(0x000b_f804);
        let result = (|| {
            let target_id = read_long(message, "world top-info player id")?;
            if target_id == 0 {
                let _ = message.send_all(game.current_net_server());
            } else {
                let _ = message.send_to_player(game.net_server(), target_id);
            }
            tracing::trace!(message_type, target_id, "верхнее информационное сообщение доставлено");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_CHAT_DELIVERY {
        message.set_message_type(0x000b_f814);
        let delivery = message.send_all(game.current_net_server());
        tracing::trace!(message_type, ?delivery, "сообщение чата разослано всем игрокам");
        return Some(Ok(()));
    }
    if message_type == COUNTRY_CHAT_DELIVERY {
        let result = (|| {
            let _legacy_ignored = read_char(message, "country chat prefix")?;
            let chat_type = read_char(message, "country chat type")? as u8;
            let country = read_char(message, "country chat country")? as u8;
            let sender = read_string(message, 0x20);
            let content = read_string(message, 0x400);
            let player_ids = game.player_ids_in_country(u32::from(country));
            for player_id in player_ids {
                let mut response = CMessage::new(0x000b_f815);
                response.base_mut().add_byte(1);
                response.base_mut().add_byte(chat_type);
                add_c_string(&mut response, &sender);
                add_c_string(&mut response, &content);
                let _ = response.send_to_player(game.net_server(), player_id);
            }
            tracing::trace!(message_type, country, chat_type, "сообщение страны доставлено");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == COUNTRY_NOTICE_DELIVERY {
        let result = (|| {
            let notice_type = read_char(message, "country notice type")? as u8;
            let country = read_char(message, "country notice country")? as u8;
            let content = read_string(message, 0x400);
            let player_ids = game.player_ids_in_country(u32::from(country));
            for player_id in player_ids {
                let mut response = CMessage::new(0x000b_f816);
                response.base_mut().add_byte(notice_type);
                add_c_string(&mut response, &content);
                let _ = response.send_to_player(game.net_server(), player_id);
            }
            tracing::trace!(message_type, country, notice_type, "уведомление страны доставлено");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == FACTION_CHAT_DELIVERY {
        let result = (|| {
            let player_id = read_long(message, "faction chat player id")?;
            let owner_type = read_long(message, "faction chat owner type")?;
            let owner_id = read_long(message, "faction chat owner id")?;
            if game.find_player(player_id).is_none() {
                tracing::trace!(message_type, player_id, "получатель чата фракции не найден");
                return Ok(());
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
            tracing::trace!(message_type, player_id, delivery, "чат фракции доставлен");
            Ok(())
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
                    tracing::trace!(message_type, player_id = owner_id, "получатель отказа личного чата не найден");
                    return Ok(());
                };
                let delivery =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0047"))
                        .send_to_player(game.net_server(), player.player_id());
                tracing::trace!(message_type, player_id = owner_id, delivery, "отказ личного чата доставлен");
                return Ok(());
            }
            if status != 1 && status != 2 {
                tracing::trace!(message_type, player_id = owner_id, status, "статус личного чата проигнорирован");
                return Ok(());
            }
            let target = read_string(message, 0x100);
            let sender = read_string(message, 0x100);
            let lookup_name = if status == 1 { &target } else { &sender };
            let Some(player_id) = game
                .find_player_by_name(lookup_name)
                .map(|player| player.player_id())
            else {
                tracing::trace!(message_type, "получатель личного чата не найден");
                return Ok(());
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
            tracing::trace!(message_type, player_id, status, delivery, "личный чат доставлен");
            Ok(())
        })();
        return Some(result);
    }
    if message_type == WORLD_PLAYER_RENAME_RESPONSE {
        let result = (|| {
            let player_id = read_long(message, "rename player id")?;
            let rename_result = read_char(message, "rename result")?;
            let new_name = read_string(message, 0x20);
            let Some(player) = game.find_player_mut(player_id) else {
                tracing::trace!(message_type, player_id, "переименовываемый игрок не найден");
                return Ok(());
            };
            message.set_message_type(PLAYER_RENAME_RESPONSE);
            if rename_result == 0 {
                player
                    .movement_shape_mut()
                    .base_object_mut()
                    .set_name(&new_name);
                let delivery = game.send_player_shape_around(player_id, None, message);
                tracing::trace!(message_type, player_id, ?delivery, "переименование принято");
                return Ok(());
            }
            let delivery = message.send_to_player(game.net_server(), player_id);
            tracing::trace!(message_type, player_id, rename_result, delivery, "переименование отклонено");
            Ok(())
        })();
        return Some(result);
    }
    let Some(player_id) = message.base_mut().get_long() else {
        return Some(Err(GameOtherMessageError::MissingPlayerId));
    };
    let Some(player) = game.find_player_mut(player_id) else {
        tracing::trace!(message_type, player_id, "игрок обновления LeiTing не найден");
        return Some(Ok(()));
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
    tracing::trace!(message_type, player_id, client_delivery, "LeiTing обновлён");
    Some(Ok(()))
}
