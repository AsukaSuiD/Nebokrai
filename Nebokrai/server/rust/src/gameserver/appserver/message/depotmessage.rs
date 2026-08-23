//! GameServer depot-password message owner.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный owner `server/gameserver/appserver/message/depotmessage.cpp`
//! подтверждают всю family. После player/changing/progress guards `0x8FE06`
//! сравнивает максимум шесть password bytes, сначала отвечает `0xBFB07`, затем
//! открывает и bank, и depot. `0x8FE07` проверяет старый пароль, длину и ASCII
//! alphanumeric нового, отвечает `0xBFB08` и только после success меняет player
//! password. `0x8FE08` синхронно закрывает оба контейнера и завершает banking.
//! Другие depot opcodes не имеют side effects. `Vec<u8>` заменяет C-string без
//! требования UTF-8; `CBank/CDepot` остаются owned player state.

use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const DEPOT_CLIENT_FAMILY: u32 = 0x0008_FE00;
const DEPOT_SERVER_FAMILY: u32 = 0x0007_FB00;
const DEPOT_UNLOCK: i32 = 0x0008_FE06;
const DEPOT_CHANGE_PASSWORD: i32 = 0x0008_FE07;
const DEPOT_CLOSE: i32 = 0x0008_FE08;
const DEPOT_UNLOCK_RESPONSE: i32 = 0x000B_FB07;
const DEPOT_CHANGE_PASSWORD_RESPONSE: i32 = 0x000B_FB08;
const DEPOT_PASSWORD_BUFFER_SIZE: usize = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepotMessageIgnoreReason {
    MissingPlayerContext,
    MissingPlayer,
    ChangingServer,
    ChangingRegion,
    NotBanking,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DepotPasswordChangeStatus {
    Changed = 0,
    OldPasswordMismatch = 1,
    InvalidCharacter = 2,
    TooLong = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DepotMessageReport {
    Ignored {
        message_type: i32,
        player_id: Option<i32>,
        reason: DepotMessageIgnoreReason,
    },
    Unlock {
        player_id: i32,
        authenticated: bool,
        delivery: i32,
        bank_locked: bool,
        depot_locked: bool,
    },
    PasswordChange {
        player_id: i32,
        status: DepotPasswordChangeStatus,
        delivery: i32,
    },
    Closed {
        player_id: i32,
        bank_locked: bool,
        depot_locked: bool,
        progress: PlayerProgress,
    },
    Unknown {
        message_type: i32,
        player_id: i32,
    },
}

/// Материализует весь `OnDepotMessage`; `None` оставляет сообщения других
/// family их собственным owner-ам.
pub(crate) fn dispatch_depot_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<DepotMessageReport> {
    let message_type = message.message_type();
    let family = message_type as u32 & 0xFFFF_FF00;
    if family != DEPOT_CLIENT_FAMILY && family != DEPOT_SERVER_FAMILY {
        return None;
    }

    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        return Some(DepotMessageReport::Ignored {
            message_type,
            player_id: None,
            reason: DepotMessageIgnoreReason::MissingPlayerContext,
        });
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(DepotMessageReport::Ignored {
            message_type,
            player_id: Some(player_id),
            reason: DepotMessageIgnoreReason::MissingPlayer,
        });
    };
    let guard = if player.in_changing_server() {
        Some(DepotMessageIgnoreReason::ChangingServer)
    } else if player.in_changing_region() {
        Some(DepotMessageIgnoreReason::ChangingRegion)
    } else if player.current_progress() != PlayerProgress::Banking {
        Some(DepotMessageIgnoreReason::NotBanking)
    } else {
        None
    };
    if let Some(reason) = guard {
        return Some(DepotMessageReport::Ignored {
            message_type,
            player_id: Some(player_id),
            reason,
        });
    }

    match message_type {
        DEPOT_UNLOCK => Some(unlock_depot(message, game, player_id)),
        DEPOT_CHANGE_PASSWORD => Some(change_depot_password(message, game, player_id)),
        DEPOT_CLOSE => {
            let player = game
                .find_player_mut(player_id)
                .expect("player проверен до depot close");
            player.close_depot_storage();
            Some(DepotMessageReport::Closed {
                player_id,
                bank_locked: player.bank_locked(),
                depot_locked: player.depot_locked(),
                progress: player.current_progress(),
            })
        }
        _ => Some(DepotMessageReport::Unknown {
            message_type,
            player_id,
        }),
    }
}

fn unlock_depot(message: &mut CMessage, game: &mut CGame, player_id: i32) -> DepotMessageReport {
    let password = message
        .base_mut()
        .get_str_bytes(DEPOT_PASSWORD_BUFFER_SIZE)
        .expect("ненулевая граница GetStr всегда даёт byte-string");
    let authenticated = game
        .find_player(player_id)
        .is_some_and(|player| player.depot_password() == password);

    let mut response = CMessage::new(DEPOT_UNLOCK_RESPONSE);
    response.add_byte(u8::from(authenticated));
    let delivery = response.send_to_player(game.net_server(), player_id);

    if authenticated {
        game.find_player_mut(player_id)
            .expect("player проверен до depot unlock")
            .unlock_depot_storage();
    }
    let player = game
        .find_player(player_id)
        .expect("player сохраняется на время depot dispatch");
    DepotMessageReport::Unlock {
        player_id,
        authenticated,
        delivery,
        bank_locked: player.bank_locked(),
        depot_locked: player.depot_locked(),
    }
}

fn change_depot_password(
    message: &mut CMessage,
    game: &mut CGame,
    player_id: i32,
) -> DepotMessageReport {
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
    let delivery = response.send_to_player(game.net_server(), player_id);
    if status == DepotPasswordChangeStatus::Changed {
        game.find_player_mut(player_id)
            .expect("player проверен до смены depot password")
            .set_depot_password(&new_password);
    }
    DepotMessageReport::PasswordChange {
        player_id,
        status,
        delivery,
    }
}
