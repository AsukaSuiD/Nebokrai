//! GameServer GM message owner.
//!
//! Точная пара GameServer EXE/PDB и owner
//! `server/gameserver/appserver/message/gmmessage.cpp` подтверждают
//! ветви `0x5FF15`, `0x7FC01/02/04/05`, `0x7FC06`, `0x7FC08..0x7FC0F`,
//! `0x7FC11` и `0x7FC13`: requester ID читается до switch,
//! silence duration нормализуется к минимуму `1`, player map
//! обходится дважды в signed ID-order, а ответы `0x5FF0D/0x5FF10`
//! уходят WorldServer. Адресный `0x7FC0F` сохраняет length guards,
//! дописывает исходный ` By Game Server {local IP}` и посылает player-у
//! `0xBF806`; recoverable allocation failure возвращает `GS0029`.
//! Broadcast `0x7FC0D` сохраняет mode guard и публикует исходный
//! `0xBF806` либо `0xBF804` через общий `SendAll`.
//! Requester feedback `0x7FC0C` выбирает `GS0033/GS0034` и форматирует
//! подтверждённые EXE-вызовом и shipped read-only language resource аргументы
//! `%s/%d/%s`. Safe `Vec` заменяет raw `char[512]`; неизвестный format
//! specifier не воспроизводит vararg/buffer UB, а остаётся typed boundary.
//! Country broadcast `0x7FC13` сохраняет unsigned-long/byte compare,
//! signed player ID-order и отдельный `SendToPlayer` для каждого адресата.
//! Mass-kick `0x7FC09` оставляет requester и ставит `QuitClientByMapID` всем
//! остальным canonical player ID в исходном ordered map-pass.
//! Region-kick `0x7FC0A` обходит physical row-major area storage, сохраняет
//! порядок и повторы `FindShapes(400)`, оставляет requester и ставит тот же
//! `QuitClientByMapID` каждому найденному player ID. Owned `Vec` снимает общий
//! ID-snapshot перед queue pass; это не меняет наблюдаемый порядок, потому что
//! `KickPlayer` только ставит network command и не мутирует region registry.
//! Named kick `0x7FC06` сохраняет 24-byte GetStr boundary, выполняет kick до
//! exact World `0x5FF09` response и возвращает исходное имя в обоих outcomes.
//! Around-kick `0x7FC07` сохраняет 256-byte name, context field, X-major 7×7
//! scan, consecutive-only unique, ordered kicks и World
//! `0x5FD02(context,requester,-1,0,GS0030(count))` после side effects.
//! Presence feedback `0x7FC08` использует signed-char branch, `GS0025/GS0026`
//! с подтверждённым `%s` и адресный `0xBF806(-1,0,text)`.
//! Входящий list response `0x5FF15` сохраняет target-player cursor gate и
//! публикует каждую полученную строку отдельным адресным `0xBF806`.
//! Запрос `0x7FC11` обходит canonical GM map, оставляет только online entries,
//! форматирует пять подтверждённых level-шаблонов, меняет ID того же пакета на
//! `0x5FF15`, дописывает count/строки и возвращает его WorldServer.
//! Script-continuation family читает `(value, script ID)` после requester-а,
//! вызывает общий runtime `ScriptContinue`; только `0x7FC01` подавляет вызов
//! при отсутствующем player, остальные три передают исходный null-owner факт.
//! Эти цепочки имеют статус `IMPLEMENTED`.
//! `Vec` заменяет raw allocation; поле declared capacity сохраняет
//! исходные `sum(name_len + 2) + 0x40`, включая возможное
//! расхождение между двумя time-sensitive pass-ами. Непокрытые GM
//! selectors остаются RAW ниже.

use super::othermessage::{GameOtherMessageRuntime, GameOtherScriptAction};
use crate::gameserver::gameserver::game::{
    CGame, GameKickAroundOutcome, GameKickAroundReport, GameKickPlayerReport,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const GM_LIST_RESPONSE_MESSAGE: i32 = 0x0005_FF15;
const GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE: i32 = 0x0007_FC01;
const GM_SCRIPT_CONTINUE_MESSAGE_1: i32 = 0x0007_FC02;
const GM_SCRIPT_CONTINUE_MESSAGE_2: i32 = 0x0007_FC04;
const GM_SCRIPT_CONTINUE_MESSAGE_3: i32 = 0x0007_FC05;
const GM_KICK_BY_NAME_MESSAGE: i32 = 0x0007_FC06;
const GM_KICK_AROUND_MESSAGE: i32 = 0x0007_FC07;
const GM_PRESENCE_FEEDBACK_MESSAGE: i32 = 0x0007_FC08;
const GM_KICK_OTHERS_MESSAGE: i32 = 0x0007_FC09;
const GM_KICK_REGION_MESSAGE: i32 = 0x0007_FC0A;
const GM_SET_SILENCE_MESSAGE: i32 = 0x0007_FC0B;
const GM_REQUESTER_FEEDBACK_MESSAGE: i32 = 0x0007_FC0C;
const GM_BROADCAST_MESSAGE: i32 = 0x0007_FC0D;
const GM_QUERY_SILENCE_MESSAGE: i32 = 0x0007_FC0E;
const GM_PRIVATE_NOTICE_MESSAGE: i32 = 0x0007_FC0F;
const GM_LIST_REQUEST_MESSAGE: i32 = 0x0007_FC11;
const GM_COUNTRY_BROADCAST_MESSAGE: i32 = 0x0007_FC13;
const GM_SET_SILENCE_RESPONSE: i32 = 0x0005_FF0D;
const GM_QUERY_SILENCE_RESPONSE: i32 = 0x0005_FF10;
const GM_KICK_BY_NAME_RESPONSE: i32 = 0x0005_FF09;
const GM_KICK_AROUND_RESPONSE: i32 = 0x0005_FD02;
const PLAYER_SYSTEM_MESSAGE: i32 = 0x000B_F806;
const PLAYER_ANNOUNCEMENT_MESSAGE: i32 = 0x000B_F804;
const GM_LEGACY_TEXT_LIMIT: usize = 0x100;
const GM_SHORT_NAME_LIMIT: usize = 0x18;
const GM_EMPTY_SILENCE_RESPONSE_LENGTH: u32 = 0x18;
const GM_SILENCE_RESPONSE_SLACK: usize = 0x40;
const GM_SILENCE_NAME_SEPARATOR: [u8; 2] = [0xA3, 0xBB];
const GM_PRIVATE_NOTICE_INVALID_LENGTH: i32 = 0x09FF_FFF9;
const GM_PRIVATE_NOTICE_SLACK: u32 = 0x40;
const GM_PRIVATE_NOTICE_SUFFIX: &[u8] = b" By Game Server ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageError {
    MissingRequesterId,
    MissingScriptContinuationValue,
    MissingScriptContinuationId,
    MissingListTargetPlayerId,
    MissingListReservedField,
    MissingListCount,
    MissingKickPlayerName,
    MissingKickAroundPlayerName,
    MissingKickAroundContext,
    MissingPresenceOutcome,
    MissingPresencePlayerName,
    MissingKickRegionId,
    MissingPlayerName,
    MissingDuration,
    MissingFeedbackPlayerName,
    MissingFeedbackValue,
    MissingFeedbackOutcome,
    MissingFeedbackText,
    UnsupportedFeedbackFormat,
    UnsupportedOnlineGmLevel { level: i32 },
    GmListCountOutsideLegacyRange { count: usize },
    MissingCountryBroadcastText,
    MissingCountry,
    MissingCountryBroadcastFirstField,
    MissingCountryBroadcastSecondField,
    MissingBroadcastText,
    MissingBroadcastFirstField,
    MissingBroadcastSecondField,
    MissingBroadcastMode,
    MissingPrivateNoticeLength,
    ZeroPrivateNoticeBufferContractUnknown { declared_length: i32 },
    DeclaredLengthOutsideLegacyRange { required: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageReport {
    ScriptContinued {
        requester_id: i32,
        script_id: i32,
        value: i32,
        player_present: bool,
        requested: bool,
    },
    ListRequest {
        requester_id: i32,
        entries: Vec<GmListPublishedEntry>,
        delivery: Result<i32, SendMessageError>,
    },
    Set {
        requester_id: i32,
        player_name: Vec<u8>,
        minutes: i32,
        player_id: Option<i32>,
        delivery: Result<i32, SendMessageError>,
    },
    Query {
        requester_id: i32,
        first_pass_count: usize,
        published_names: Vec<Vec<u8>>,
        declared_capacity: u32,
        delivery: Result<i32, SendMessageError>,
    },
    PrivateNoticeIgnored {
        requester_id: i32,
        declared_length: i32,
    },
    PrivateNotice {
        requester_id: i32,
        declared_length: i32,
        published_text: Vec<u8>,
        allocation_failed: bool,
        delivery: i32,
    },
    BroadcastIgnored {
        requester_id: i32,
        mode: i32,
    },
    Broadcast {
        requester_id: i32,
        text: Vec<u8>,
        first_field: i32,
        second_field: i32,
        mode: i32,
        response_type: i32,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterFeedback {
        requester_id: i32,
        player_name: Vec<u8>,
        value: i32,
        successful: bool,
        detail: Vec<u8>,
        string_id: &'static [u8],
        formatted_text: Vec<u8>,
        delivery: i32,
    },
    CountryBroadcast {
        requester_id: i32,
        country: u32,
        first_field: i32,
        second_field: i32,
        text: Vec<u8>,
        deliveries: Vec<(i32, i32)>,
    },
    KickOthers {
        requester_id: i32,
        kicks: Vec<GameKickPlayerReport>,
    },
    KickRegion {
        requester_id: i32,
        region_id: i32,
        region_found: bool,
        kicks: Vec<GameKickPlayerReport>,
    },
    KickByName {
        requester_id: i32,
        player_name: Vec<u8>,
        kick: Option<GameKickPlayerReport>,
        delivery: Result<i32, SendMessageError>,
    },
    KickAround {
        requester_id: i32,
        player_name: Vec<u8>,
        response_context: i32,
        traversal: GameKickAroundReport,
        formatted_text: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    PresenceFeedback {
        requester_id: i32,
        player_name: Vec<u8>,
        outcome: i8,
        string_id: &'static [u8],
        formatted_text: Vec<u8>,
        delivery: i32,
    },
    ListResponse {
        requester_id: i32,
        target_player_id: i32,
        target_found: bool,
        reserved_field: Option<i32>,
        declared_count: Option<i32>,
        published_texts: Vec<Vec<u8>>,
        deliveries: Vec<i32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GmListPublishedEntry {
    pub(crate) name: Vec<u8>,
    pub(crate) level: i32,
    pub(crate) string_id: &'static [u8],
    pub(crate) text: Vec<u8>,
}

/// Материализует связанные silence, broadcast и direct-notice ветви `OnGMMessage`.
/// `None` оставляет прочие selectors их ещё RAW owner-у.
pub(crate) fn dispatch_gm_message<Runtime: GameOtherMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GmMessageReport, GmMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        GM_LIST_RESPONSE_MESSAGE
            | GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE
            | GM_SCRIPT_CONTINUE_MESSAGE_1
            | GM_SCRIPT_CONTINUE_MESSAGE_2
            | GM_SCRIPT_CONTINUE_MESSAGE_3
            | GM_KICK_BY_NAME_MESSAGE
            | GM_KICK_AROUND_MESSAGE
            | GM_PRESENCE_FEEDBACK_MESSAGE
            | GM_KICK_OTHERS_MESSAGE
            | GM_KICK_REGION_MESSAGE
            | GM_SET_SILENCE_MESSAGE
            | GM_REQUESTER_FEEDBACK_MESSAGE
            | GM_BROADCAST_MESSAGE
            | GM_QUERY_SILENCE_MESSAGE
            | GM_PRIVATE_NOTICE_MESSAGE
            | GM_LIST_REQUEST_MESSAGE
            | GM_COUNTRY_BROADCAST_MESSAGE
    ) {
        return None;
    }
    let Some(requester_id) = message.base_mut().get_long() else {
        return Some(Err(GmMessageError::MissingRequesterId));
    };

    if matches!(
        message_type,
        GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE
            | GM_SCRIPT_CONTINUE_MESSAGE_1
            | GM_SCRIPT_CONTINUE_MESSAGE_2
            | GM_SCRIPT_CONTINUE_MESSAGE_3
    ) {
        let Some(value) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingScriptContinuationValue));
        };
        let Some(script_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingScriptContinuationId));
        };
        let player_present = game.find_player(requester_id).is_some();
        let requested = message_type != GM_SCRIPT_CONTINUE_IF_PRESENT_MESSAGE || player_present;
        if requested {
            runtime.run_other_script_action(
                game,
                GameOtherScriptAction::Continue {
                    script_id,
                    player_id: requester_id,
                    player_present,
                    value,
                },
            );
        }
        return Some(Ok(GmMessageReport::ScriptContinued {
            requester_id,
            script_id,
            value,
            player_present,
            requested,
        }));
    }

    if message_type == GM_LIST_REQUEST_MESSAGE {
        let mut entries = Vec::new();
        for info in game.gm_list().gm_info().values() {
            if game.find_player_by_name(&info.name).is_none() {
                continue;
            }
            let string_id = match info.level {
                100 => b"GS0035".as_slice(),
                90 => b"GS0036".as_slice(),
                50 => b"GS0037".as_slice(),
                40 => b"GSN0001".as_slice(),
                30 => b"GSN0002".as_slice(),
                level => {
                    return Some(Err(GmMessageError::UnsupportedOnlineGmLevel { level }));
                }
            };
            let text = match format_gm_template(
                game.get_string_by_id(string_id),
                &[GmFormatArgument::Text(&info.name)],
            ) {
                Ok(formatted) => formatted,
                Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
            };
            entries.push(GmListPublishedEntry {
                name: info.name.clone(),
                level: info.level,
                string_id,
                text,
            });
        }
        let count = match i32::try_from(entries.len()) {
            Ok(count) => count,
            Err(_) => {
                return Some(Err(GmMessageError::GmListCountOutsideLegacyRange {
                    count: entries.len(),
                }));
            }
        };
        message
            .base_mut()
            .set_message_type(GM_LIST_RESPONSE_MESSAGE);
        message.add_long(count);
        for entry in &entries {
            add_legacy_c_string(message, &entry.text);
        }
        let delivery = message.send(game, false);
        return Some(Ok(GmMessageReport::ListRequest {
            requester_id,
            entries,
            delivery,
        }));
    }

    if message_type == GM_LIST_RESPONSE_MESSAGE {
        let Some(target_player_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingListTargetPlayerId));
        };
        if game.find_player(target_player_id).is_none() {
            return Some(Ok(GmMessageReport::ListResponse {
                requester_id,
                target_player_id,
                target_found: false,
                reserved_field: None,
                declared_count: None,
                published_texts: Vec::new(),
                deliveries: Vec::new(),
            }));
        }
        let Some(reserved_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingListReservedField));
        };
        let Some(declared_count) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingListCount));
        };
        let mut published_texts = Vec::new();
        let mut deliveries = Vec::new();
        for _ in 0..declared_count.max(0) {
            let text = message
                .base_mut()
                .get_str_bytes(GM_LEGACY_TEXT_LIMIT)
                .expect("ненулевой GM list text limit");
            let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
            response.add_long(-1);
            response.add_long(0);
            add_legacy_c_string(&mut response, &text);
            deliveries.push(response.send_to_player(game.net_server(), target_player_id));
            published_texts.push(text);
        }
        return Some(Ok(GmMessageReport::ListResponse {
            requester_id,
            target_player_id,
            target_found: true,
            reserved_field: Some(reserved_field),
            declared_count: Some(declared_count),
            published_texts,
            deliveries,
        }));
    }

    if message_type == GM_KICK_BY_NAME_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_SHORT_NAME_LIMIT) else {
            return Some(Err(GmMessageError::MissingKickPlayerName));
        };
        let kick = game.kick_player_by_name(&player_name);
        let mut response = CMessage::new(GM_KICK_BY_NAME_RESPONSE);
        response.add_long(requester_id);
        response.add_byte(u8::from(kick.is_some()));
        add_legacy_c_string(&mut response, &player_name);
        let delivery = response.send(game, false);
        return Some(Ok(GmMessageReport::KickByName {
            requester_id,
            player_name,
            kick,
            delivery,
        }));
    }

    if message_type == GM_KICK_AROUND_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingKickAroundPlayerName));
        };
        let Some(response_context) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingKickAroundContext));
        };
        let traversal = game.kick_players_around_name(&player_name);
        if traversal.outcome != GameKickAroundOutcome::Completed {
            return Some(Ok(GmMessageReport::KickAround {
                requester_id,
                player_name,
                response_context,
                traversal,
                formatted_text: None,
                delivery: None,
            }));
        }
        let formatted_text = match format_gm_template(
            game.get_string_by_id(b"GS0030"),
            &[GmFormatArgument::Signed(
                traversal.matched_player_ids.len() as i32
            )],
        ) {
            Ok(formatted) => formatted,
            Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
        };
        let mut response = CMessage::new(GM_KICK_AROUND_RESPONSE);
        response.add_long(response_context);
        response.add_long(requester_id);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &formatted_text);
        let delivery = response.send(game, false);
        return Some(Ok(GmMessageReport::KickAround {
            requester_id,
            player_name,
            response_context,
            traversal,
            formatted_text: Some(formatted_text),
            delivery: Some(delivery),
        }));
    }

    if message_type == GM_PRESENCE_FEEDBACK_MESSAGE {
        let Some(outcome) = message.base_mut().get_char() else {
            return Some(Err(GmMessageError::MissingPresenceOutcome));
        };
        let Some(player_name) = message.base_mut().get_str_bytes(GM_SHORT_NAME_LIMIT) else {
            return Some(Err(GmMessageError::MissingPresencePlayerName));
        };
        let string_id = if outcome < 1 {
            b"GS0026".as_slice()
        } else {
            b"GS0025".as_slice()
        };
        let formatted_text = match format_gm_template(
            game.get_string_by_id(string_id),
            &[GmFormatArgument::Text(&player_name)],
        ) {
            Ok(formatted) => formatted,
            Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
        };
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &formatted_text);
        let delivery = response.send_to_player(game.net_server(), requester_id);
        return Some(Ok(GmMessageReport::PresenceFeedback {
            requester_id,
            player_name,
            outcome,
            string_id,
            formatted_text,
            delivery,
        }));
    }

    if message_type == GM_KICK_OTHERS_MESSAGE {
        return Some(Ok(GmMessageReport::KickOthers {
            requester_id,
            kicks: game.kick_players_except(requester_id),
        }));
    }

    if message_type == GM_KICK_REGION_MESSAGE {
        let Some(region_id) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingKickRegionId));
        };
        let region_found = game.find_region(region_id).is_some();
        return Some(Ok(GmMessageReport::KickRegion {
            requester_id,
            region_id,
            region_found,
            kicks: game.kick_players_in_region_except(region_id, requester_id),
        }));
    }

    if message_type == GM_SET_SILENCE_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingPlayerName));
        };
        let Some(minutes) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingDuration));
        };
        let minutes = minutes.max(1);
        let mut response = CMessage::new(GM_SET_SILENCE_RESPONSE);
        response.add_long(requester_id);
        add_legacy_c_string(&mut response, &player_name);
        response.add_long(minutes);
        let player_id =
            game.silence_player_by_name(&player_name, minutes, || runtime.other_now_milliseconds());
        let (success, text_id) = if player_id.is_some() {
            (1, b"GS0031".as_slice())
        } else {
            (0, b"GS0032".as_slice())
        };
        let localized = game.get_string_by_id(text_id).to_vec();
        response.add_byte(success);
        add_legacy_c_string(&mut response, &localized);
        let delivery = response.send(game, false);
        return Some(Ok(GmMessageReport::Set {
            requester_id,
            player_name,
            minutes,
            player_id,
            delivery,
        }));
    }

    if message_type == GM_REQUESTER_FEEDBACK_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingFeedbackPlayerName));
        };
        let Some(value) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingFeedbackValue));
        };
        let Some(outcome) = message.base_mut().get_char() else {
            return Some(Err(GmMessageError::MissingFeedbackOutcome));
        };
        let Some(detail) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingFeedbackText));
        };
        let successful = outcome != 0;
        let string_id = if successful {
            b"GS0033".as_slice()
        } else {
            b"GS0034".as_slice()
        };
        let formatted_text = match format_gm_feedback(
            game.get_string_by_id(string_id),
            &player_name,
            value,
            &detail,
        ) {
            Ok(formatted) => formatted,
            Err(()) => return Some(Err(GmMessageError::UnsupportedFeedbackFormat)),
        };
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &formatted_text);
        let delivery = response.send_to_player(game.net_server(), requester_id);
        return Some(Ok(GmMessageReport::RequesterFeedback {
            requester_id,
            player_name,
            value,
            successful,
            detail,
            string_id,
            formatted_text,
            delivery,
        }));
    }

    if message_type == GM_COUNTRY_BROADCAST_MESSAGE {
        let Some(text) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingCountryBroadcastText));
        };
        let Some(country) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingCountry));
        };
        let Some(first_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingCountryBroadcastFirstField));
        };
        let Some(second_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingCountryBroadcastSecondField));
        };
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(first_field);
        response.add_long(second_field);
        add_legacy_c_string(&mut response, &text);
        let deliveries = game
            .player_ids_in_country(country as u32)
            .into_iter()
            .map(|player_id| {
                (
                    player_id,
                    response.send_to_player(game.net_server(), player_id),
                )
            })
            .collect();
        return Some(Ok(GmMessageReport::CountryBroadcast {
            requester_id,
            country: country as u32,
            first_field,
            second_field,
            text,
            deliveries,
        }));
    }

    if message_type == GM_PRIVATE_NOTICE_MESSAGE {
        let Some(declared_length) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingPrivateNoticeLength));
        };
        if matches!(declared_length, 0 | GM_PRIVATE_NOTICE_INVALID_LENGTH) {
            return Some(Ok(GmMessageReport::PrivateNoticeIgnored {
                requester_id,
                declared_length,
            }));
        }
        let buffer_length = (declared_length as u32).wrapping_add(GM_PRIVATE_NOTICE_SLACK) as usize;
        if buffer_length == 0 {
            return Some(Err(
                GmMessageError::ZeroPrivateNoticeBufferContractUnknown { declared_length },
            ));
        }
        let mut published_text = Vec::new();
        let allocation_failed = published_text.try_reserve_exact(buffer_length).is_err();
        if allocation_failed {
            published_text.extend_from_slice(game.get_string_by_id(b"GS0029"));
        } else {
            published_text.extend_from_slice(
                &message
                    .base_mut()
                    .get_str_bytes(buffer_length)
                    .expect("ненулевой GM notice buffer"),
            );
            published_text.extend_from_slice(GM_PRIVATE_NOTICE_SUFFIX);
            published_text.extend_from_slice(game.net_server().local_ip());
        }
        let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
        response.add_long(-1);
        response.add_long(0);
        add_legacy_c_string(&mut response, &published_text);
        let delivery = response.send_to_player(game.net_server(), requester_id);
        return Some(Ok(GmMessageReport::PrivateNotice {
            requester_id,
            declared_length,
            published_text,
            allocation_failed,
            delivery,
        }));
    }

    if message_type == GM_BROADCAST_MESSAGE {
        let Some(text) = message.base_mut().get_str_bytes(GM_LEGACY_TEXT_LIMIT) else {
            return Some(Err(GmMessageError::MissingBroadcastText));
        };
        let Some(first_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingBroadcastFirstField));
        };
        let Some(second_field) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingBroadcastSecondField));
        };
        let Some(mode) = message.base_mut().get_long() else {
            return Some(Err(GmMessageError::MissingBroadcastMode));
        };
        let mut response = match mode {
            0 => {
                let mut response = CMessage::new(PLAYER_SYSTEM_MESSAGE);
                response.add_long(first_field);
                response.add_long(second_field);
                response
            }
            1 => {
                let mut response = CMessage::new(PLAYER_ANNOUNCEMENT_MESSAGE);
                response.add_long(0);
                response.add_long(-1);
                response.add_long(1);
                response.add_long(1);
                response
            }
            _ => {
                return Some(Ok(GmMessageReport::BroadcastIgnored { requester_id, mode }));
            }
        };
        add_legacy_c_string(&mut response, &text);
        let response_type = response.message_type();
        let delivery = response.send_all(game.current_net_server());
        return Some(Ok(GmMessageReport::Broadcast {
            requester_id,
            text,
            first_field,
            second_field,
            mode,
            response_type,
            delivery,
        }));
    }

    let first_pass = game.silenced_player_names_pass(|| runtime.other_now_milliseconds());
    if first_pass.is_empty() {
        let localized = game.get_string_by_id(b"GS0028").to_vec();
        let mut response = CMessage::new(GM_QUERY_SILENCE_RESPONSE);
        response.add_long(requester_id);
        response.add_ulong(GM_EMPTY_SILENCE_RESPONSE_LENGTH);
        add_legacy_c_string(&mut response, &localized);
        let delivery = response.send(game, false);
        return Some(Ok(GmMessageReport::Query {
            requester_id,
            first_pass_count: 0,
            published_names: Vec::new(),
            declared_capacity: GM_EMPTY_SILENCE_RESPONSE_LENGTH,
            delivery,
        }));
    }

    let required = first_pass.iter().try_fold(0usize, |total, name| {
        name.len()
            .checked_add(GM_SILENCE_NAME_SEPARATOR.len())
            .and_then(|name_length| total.checked_add(name_length))
    });
    let Some(required) = required.and_then(|length| length.checked_add(GM_SILENCE_RESPONSE_SLACK))
    else {
        return Some(Err(GmMessageError::DeclaredLengthOutsideLegacyRange {
            required: usize::MAX,
        }));
    };
    let Ok(declared_capacity) = u32::try_from(required) else {
        return Some(Err(GmMessageError::DeclaredLengthOutsideLegacyRange {
            required,
        }));
    };

    let published_names = game.silenced_player_names_pass(|| runtime.other_now_milliseconds());
    let mut names = Vec::with_capacity(required - GM_SILENCE_RESPONSE_SLACK);
    for name in &published_names {
        names.extend_from_slice(name);
        names.extend_from_slice(&GM_SILENCE_NAME_SEPARATOR);
    }
    let mut response = CMessage::new(GM_QUERY_SILENCE_RESPONSE);
    response.add_long(requester_id);
    response.add_ulong(declared_capacity);
    add_legacy_c_string(&mut response, &names);
    let delivery = response.send(game, false);
    Some(Ok(GmMessageReport::Query {
        requester_id,
        first_pass_count: first_pass.len(),
        published_names,
        declared_capacity,
        delivery,
    }))
}

fn add_legacy_c_string(message: &mut CMessage, value: &[u8]) {
    message.base_mut().add(legacy_c_string_prefix(value));
    message.add_byte(0);
}

fn format_gm_feedback(
    template: &[u8],
    player_name: &[u8],
    value: i32,
    detail: &[u8],
) -> Result<Vec<u8>, ()> {
    let arguments = [
        GmFormatArgument::Text(player_name),
        GmFormatArgument::Signed(value),
        GmFormatArgument::Text(detail),
    ];
    format_gm_template(template, &arguments)
}

enum GmFormatArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

fn format_gm_template(template: &[u8], arguments: &[GmFormatArgument<'_>]) -> Result<Vec<u8>, ()> {
    let template = legacy_c_string_prefix(template);
    let mut output = Vec::with_capacity(template.len());
    let mut argument_index = 0usize;
    let mut offset = 0usize;
    while offset < template.len() {
        if template[offset] != b'%' {
            output.push(template[offset]);
            offset += 1;
            continue;
        }
        let Some(specifier) = template.get(offset + 1).copied() else {
            return Err(());
        };
        if specifier == b'%' {
            output.push(b'%');
            offset += 2;
            continue;
        }
        let Some(argument) = arguments.get(argument_index) else {
            return Err(());
        };
        match (specifier, argument) {
            (b's', GmFormatArgument::Text(text)) => {
                output.extend_from_slice(legacy_c_string_prefix(text));
            }
            (b'd' | b'i', GmFormatArgument::Signed(value)) => {
                output.extend_from_slice(value.to_string().as_bytes());
            }
            (b'u', GmFormatArgument::Signed(value)) => {
                output.extend_from_slice((*value as u32).to_string().as_bytes());
            }
            _ => return Err(()),
        }
        argument_index += 1;
        offset += 2;
    }
    Ok(output)
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\gmmessage.cpp

// ============================================================================
// FUNCTION: OnGMMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\gmmessage.cpp:19
// RVA: 0x0009B210
// ADDRESS: 0049b210
// PROTOTYPE: void __cdecl OnGMMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ParseGMCommand
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\gmmessage.cpp:591
// RVA: 0x0009C7F0
// ADDRESS: 0049c7f0
// PROTOTYPE: bool __cdecl ParseGMCommand(CMessage * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
