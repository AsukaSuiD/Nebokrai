//! Login lifecycle `OnLogMessage` из `logmessage.cpp`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Сюда идут логические отспечатки остановки/offlice нестандартных событий мира:
//! списки обновляются напоряд до wire-ответа LoginServer в той же форме;
//! сохранение исходной точности lexical info очереди этой волны записаны ниже.
//! Остальные ветки `0x4FB01..0x4FB07` и `0x5FB01..0x5FB02` у владельца старого
//! пакета до следующей волны.

use crate::app::world_game_view::{WorldGameView, WorldLoginTimeoutTeamExit};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::sessions::csessionfactory::CSessionFactory;

pub const RESTORE_ROLE_REQUEST: i32 = 0x0004_FB03;
pub const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
pub const RESTORE_ROLE_STATUS: i8 = 0x15;
pub const ACCOUNT_LOGIN_CLEANUP_REQUEST: i32 = 0x0004_FB06;
pub const ACCOUNT_DISCONNECT_REQUEST: i32 = 0x0004_FB07;
pub const ACCOUNT_DISCONNECT_GAME_RESPONSE: i32 = 0x0007_F903;
pub const ACCOUNT_DISCONNECT_LOGIN_RESPONSE: i32 = 0x0001_FF06;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldRestoreRoleOutcome {
    pub account: Vec<u8>,
    pub player_id: u32,
    pub player_id_complete: bool,
    pub response_type: i32,
    pub status: i8,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldAccountLoginCleanupOutcome {
    NotFound { account: Vec<u8> },
    Cleaned {
        account: Vec<u8>,
        player_id: u32,
        team_id: i32,
        team_session_id: i32,
        team_exit: WorldLoginTimeoutTeamExit,
        player_load_removed: bool,
        login_removed: bool,
        offline_inserted: bool,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldAccountDisconnectOutcome {
    OnlineRouted {
        account: Vec<u8>,
        player_id: u32,
        game_server_index: u32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        team_id: i32,
        team_session_id: i32,
        team_exit: WorldLoginTimeoutTeamExit,
    },
    LoginAcknowledged {
        account: Vec<u8>,
        released_player_id: Option<u32>,
        team_id: Option<i32>,
        team_session_id: Option<i32>,
        team_exit: Option<WorldLoginTimeoutTeamExit>,
        login_removed: bool,
        offline_inserted: bool,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

/// Ветвь `0x4FB03` без DB и без cross-owner: сплыть списки в хранении до
/// wire-ответа с конечным owner-полем пакета без изменения формы payload.
pub fn on_restore_role(
    game: &mut dyn WorldGameView,
    mut message: CMessage,
) -> WorldRestoreRoleOutcome {
    let account = message
        .base_mut()
        .get_str_bytes(0x14)
        .expect("literal 0x14 исключает zero-capacity GetStr");
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0) as u32;

    game.delete_deletion_player(player_id);
    game.append_restore_player(player_id);

    let mut response = CMessage::new(RESTORE_ROLE_RESPONSE);
    response.base_mut().add_char(RESTORE_ROLE_STATUS);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldRestoreRoleOutcome {
        account,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        response_type: RESTORE_ROLE_RESPONSE,
        status: RESTORE_ROLE_STATUS,
        wire,
        delivery,
    }
}

/// Ветвь `0x4FB06` с постановкой lookup по account. Уже оказываемые мутации
/// вэтом tid оптически preexisting形式 preview в funtccb ownership placements.
pub fn on_account_login_cleanup(
    game: &mut dyn WorldGameView,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldAccountLoginCleanupOutcome {
    let account = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let Some(player) = game.login_player_by_account(&account) else {
        return WorldAccountLoginCleanupOutcome::NotFound { account };
    };

    let team_session_id = game.get_team_session_id(player.team_id as u32);
    let team_exit = game.exit_team_player(
        session_factory,
        team_session_id,
        player.owner_type,
        player.owner_id,
    );
    let player_id = player.owner_id as u32;
    let player_load_removed = game.remove_player_load_data(player.owner_id);
    let login_removed = game.remove_login_player(player_id);
    let offline_inserted = game.append_offline_player_id(player_id);

    WorldAccountLoginCleanupOutcome::Cleaned {
        account,
        player_id,
        team_id: player.team_id,
        team_session_id,
        team_exit,
        player_load_removed,
        login_removed,
        offline_inserted,
    }
}

/// Ветвь `0x4FB07` хранит два маршрута отклонения учетной prive caприемном
/// исследованной формальной аппаратной математике предстоящего принципа.
pub fn on_account_disconnect(
    game: &mut dyn WorldGameView,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldAccountDisconnectOutcome {
    let account = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");

    if let Some(player) = game.online_player_route_by_account(&account) {
        let player_id = player.owner_id as u32;
        let mut response = CMessage::new(ACCOUNT_DISCONNECT_GAME_RESPONSE);
        response.base_mut().add_ulong(player_id);
        let wire = response.as_wire_bytes().to_vec();
        let delivery = game.send_msg_to_game_server(
            player.game_server_index as i32,
            &response,
        );
        let team_session_id = game.get_team_session_id(player.team_id as u32);
        let team_exit = game.exit_team_player(
            session_factory,
            team_session_id,
            player.owner_type,
            player.owner_id,
        );
        return WorldAccountDisconnectOutcome::OnlineRouted {
            account,
            player_id,
            game_server_index: player.game_server_index,
            response_type: ACCOUNT_DISCONNECT_GAME_RESPONSE,
            wire,
            delivery,
            team_id: player.team_id,
            team_session_id,
            team_exit,
        };
    }

    let mut released_player_id = None;
    let mut team_id = None;
    let mut team_session_id = None;
    let mut team_exit = None;
    let mut login_removed = false;
    let mut offline_inserted = false;
    if let Some(player) = game.login_player_by_account(&account) {
        let session_id = game.get_team_session_id(player.team_id as u32);
        let exit = game.exit_team_player(
            session_factory,
            session_id,
            player.owner_type,
            player.owner_id,
        );
        let player_id = player.owner_id as u32;
        login_removed = game.remove_login_player(player_id);
        offline_inserted = game.append_offline_player_id(player_id);
        released_player_id = Some(player_id);
        team_id = Some(player.team_id);
        team_session_id = Some(session_id);
        team_exit = Some(exit);
    }

    let mut response = CMessage::new(ACCOUNT_DISCONNECT_LOGIN_RESPONSE);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    response.base_mut().add_char(0);
    response.base_mut().add_char(0);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldAccountDisconnectOutcome::LoginAcknowledged {
        account,
        released_player_id,
        team_id,
        team_session_id,
        team_exit,
        login_removed,
        offline_inserted,
        response_type: ACCOUNT_DISCONNECT_LOGIN_RESPONSE,
        wire,
        delivery,
    }
}
