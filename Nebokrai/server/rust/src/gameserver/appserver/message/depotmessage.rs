//! Владелец сообщений пароля хранилища GameServer.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `server/gameserver/appserver/message/depotmessage.cpp`
//! подтверждают всё семейство. После проверок игрока, смены сервера/региона и
//! текущего процесса `0x8FE06` сравнивает максимум шесть байтов пароля, сначала
//! отвечает `0xBFB07`, затем открывает bank и depot. `0x8FE07` проверяет старый
//! пароль, длину и ASCII-буквы/цифры нового, отвечает `0xBFB08` и только после
//! успеха меняет пароль игрока. `0x8FE08` синхронно закрывает оба контейнера и
//! завершает банковский процесс. Другие opcode хранилища не имеют эффектов.
//! `Vec<u8>` сохраняет C-строку без требования UTF-8; `CBank/CDepot` остаются
//! состоянием игрока. Диагностические причины публикуются через `tracing`.

use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use tracing::{debug, trace};

const DEPOT_CLIENT_FAMILY: u32 = 0x0008_FE00;
const DEPOT_SERVER_FAMILY: u32 = 0x0007_FB00;
const DEPOT_UNLOCK: i32 = 0x0008_FE06;
const DEPOT_CHANGE_PASSWORD: i32 = 0x0008_FE07;
const DEPOT_CLOSE: i32 = 0x0008_FE08;
const DEPOT_UNLOCK_RESPONSE: i32 = 0x000B_FB07;
const DEPOT_CHANGE_PASSWORD_RESPONSE: i32 = 0x000B_FB08;
const DEPOT_PASSWORD_BUFFER_SIZE: usize = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepotPasswordChangeStatus {
    Changed = 0,
    OldPasswordMismatch = 1,
    InvalidCharacter = 2,
    TooLong = 3,
}

/// Материализует весь `OnDepotMessage`; `None` оставляет сообщения других
/// family их собственным owner-ам.
pub(crate) fn dispatch_depot_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<()> {
    let message_type = message.message_type();
    let family = message_type as u32 & 0xFFFF_FF00;
    if family != DEPOT_CLIENT_FAMILY && family != DEPOT_SERVER_FAMILY {
        return None;
    }

    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        trace!(message_type, "сообщение хранилища пропущено: нет контекста игрока");
        return Some(());
    };
    let Some(player) = game.find_player(player_id) else {
        trace!(message_type, player_id, "сообщение хранилища пропущено: игрок не найден");
        return Some(());
    };
    let guard = if player.in_changing_server() {
        Some("смена сервера")
    } else if player.in_changing_region() {
        Some("смена региона")
    } else if player.current_progress() != PlayerProgress::Banking {
        Some("игрок не работает с хранилищем")
    } else {
        None
    };
    if let Some(reason) = guard {
        trace!(message_type, player_id, reason, "сообщение хранилища пропущено");
        return Some(());
    }

    match message_type {
        DEPOT_UNLOCK => {
            unlock_depot(message, game, player_id);
            Some(())
        }
        DEPOT_CHANGE_PASSWORD => {
            change_depot_password(message, game, player_id);
            Some(())
        }
        DEPOT_CLOSE => {
            let player = game
                .find_player_mut(player_id)
                .expect("player проверен до depot close");
            player.close_depot_storage();
            debug!(player_id, "хранилище игрока закрыто");
            Some(())
        }
        _ => {
            trace!(message_type, player_id, "неизвестное сообщение хранилища не имеет эффекта");
            Some(())
        }
    }
}

fn unlock_depot(message: &mut CMessage, game: &mut CGame, player_id: i32) {
    let password = message
        .base_mut()
        .get_str_bytes(DEPOT_PASSWORD_BUFFER_SIZE)
        .expect("ненулевая граница GetStr всегда даёт byte-string");
    let authenticated = game
        .find_player(player_id)
        .is_some_and(|player| player.depot_password() == password);

    let mut response = CMessage::new(DEPOT_UNLOCK_RESPONSE);
    response.add_byte(u8::from(authenticated));
    let _ = response.send_to_player(game.net_server(), player_id);

    if authenticated {
        game.find_player_mut(player_id)
            .expect("player проверен до depot unlock")
            .unlock_depot_storage();
    }
    debug!(player_id, authenticated, "обработано открытие хранилища");
}

fn change_depot_password(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
) {
    let old_password = message
        .base_mut()
        .get_str_bytes(DEPOT_PASSWORD_BUFFER_SIZE)
        .expect("ненулевая граница GetStr всегда даёт byte-string");
    let new_password = message
        .base_mut()
        .get_str_bytes(DEPOT_PASSWORD_BUFFER_SIZE)
        .expect("ненулевая граница GetStr всегда даёт byte-string");
    let old_matches = game
        .find_player(player_id)
        .is_some_and(|player| player.depot_password() == old_password);
    let status = if !old_matches {
        DepotPasswordChangeStatus::OldPasswordMismatch
    } else if new_password.len() >= DEPOT_PASSWORD_BUFFER_SIZE {
        DepotPasswordChangeStatus::TooLong
    } else if new_password
        .iter()
        .any(|byte| !byte.is_ascii_alphanumeric())
    {
        DepotPasswordChangeStatus::InvalidCharacter
    } else {
        DepotPasswordChangeStatus::Changed
    };

    let mut response = CMessage::new(DEPOT_CHANGE_PASSWORD_RESPONSE);
    response.add_byte(status as u8);
    let _ = response.send_to_player(game.net_server(), player_id);
    if status == DepotPasswordChangeStatus::Changed {
        game.find_player_mut(player_id)
            .expect("player проверен до смены depot password")
            .set_depot_password(&new_password);
    }
    debug!(player_id, ?status, "обработана смена пароля хранилища");
}
