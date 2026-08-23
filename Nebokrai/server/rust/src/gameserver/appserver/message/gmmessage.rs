//! GameServer GM message owner.
//!
//! Точная пара GameServer EXE/PDB и owner
//! `server/gameserver/appserver/message/gmmessage.cpp` подтверждают
//! ветви `0x7FC0B/0x7FC0E/0x7FC0F`: requester ID читается до switch,
//! silence duration нормализуется к минимуму `1`, player map
//! обходится дважды в signed ID-order, а ответы `0x5FF0D/0x5FF10`
//! уходят WorldServer. Адресный `0x7FC0F` сохраняет length guards,
//! дописывает исходный ` By Game Server {local IP}` и посылает player-у
//! `0xBF806`; recoverable allocation failure возвращает `GS0029`.
//! Эти цепочки имеют статус `IMPLEMENTED`.
//! `Vec` заменяет raw allocation; поле declared capacity сохраняет
//! исходные `sum(name_len + 2) + 0x40`, включая возможное
//! расхождение между двумя time-sensitive pass-ами. Непокрытые GM
//! selectors остаются RAW ниже.

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::{CMessage, SendMessageError};

const GM_SET_SILENCE_MESSAGE: i32 = 0x0007_FC0B;
const GM_QUERY_SILENCE_MESSAGE: i32 = 0x0007_FC0E;
const GM_PRIVATE_NOTICE_MESSAGE: i32 = 0x0007_FC0F;
const GM_SET_SILENCE_RESPONSE: i32 = 0x0005_FF0D;
const GM_QUERY_SILENCE_RESPONSE: i32 = 0x0005_FF10;
const PLAYER_SYSTEM_MESSAGE: i32 = 0x000B_F806;
const GM_SILENCE_NAME_LIMIT: usize = 0x100;
const GM_EMPTY_SILENCE_RESPONSE_LENGTH: u32 = 0x18;
const GM_SILENCE_RESPONSE_SLACK: usize = 0x40;
const GM_SILENCE_NAME_SEPARATOR: [u8; 2] = [0xA3, 0xBB];
const GM_PRIVATE_NOTICE_INVALID_LENGTH: i32 = 0x09FF_FFF9;
const GM_PRIVATE_NOTICE_SLACK: u32 = 0x40;
const GM_PRIVATE_NOTICE_SUFFIX: &[u8] = b" By Game Server ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageError {
    MissingRequesterId,
    MissingPlayerName,
    MissingDuration,
    MissingPrivateNoticeLength,
    ZeroPrivateNoticeBufferContractUnknown { declared_length: i32 },
    DeclaredLengthOutsideLegacyRange { required: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmMessageReport {
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
}

/// Материализует связанные silence и direct-notice ветви `OnGMMessage`.
/// `None` оставляет прочие selectors их ещё RAW owner-у.
pub(crate) fn dispatch_gm_message(
    message: &mut CMessage,
    game: &mut CGame,
    mut now_milliseconds: impl FnMut() -> u32,
) -> Option<Result<GmMessageReport, GmMessageError>> {
    let message_type = message.message_type();
    if !matches!(
        message_type,
        GM_SET_SILENCE_MESSAGE | GM_QUERY_SILENCE_MESSAGE | GM_PRIVATE_NOTICE_MESSAGE
    ) {
        return None;
    }
    let Some(requester_id) = message.base_mut().get_long() else {
        return Some(Err(GmMessageError::MissingRequesterId));
    };

    if message_type == GM_SET_SILENCE_MESSAGE {
        let Some(player_name) = message.base_mut().get_str_bytes(GM_SILENCE_NAME_LIMIT) else {
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
        let player_id = game.silence_player_by_name(&player_name, minutes, &mut now_milliseconds);
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

    let first_pass = game.silenced_player_names_pass(&mut now_milliseconds);
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

    let published_names = game.silenced_player_names_pass(&mut now_milliseconds);
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
    let prefix = value
        .get(
            ..value
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(value.len()),
        )
        .expect("C-string prefix всегда внутри slice");
    message.base_mut().add(prefix);
    message.add_byte(0);
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
