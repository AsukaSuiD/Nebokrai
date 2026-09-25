//! Login lifecycle `OnLogMessage` из `logmessage.cpp`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Перенесённые ветви диспетчера `OnLogMessage`: списки восстановления и
//! удаления, вход и отключение аккаунта изменяются в исходном порядке до
//! wire-ответа LoginServer; форма payload не меняется. Остальные ветки
//! Список персонажей `0x4FB01` живёт в соседнем `player_base.rs`; остальные
//! пока не перенесённые ветви остаются у владельца старого пакета.

use nebokrai_shared::resources::GlobeSetupSnapshot;

use crate::app::world_game_view::{
    WorldDeleteRoleCountryGate, WorldDeleteRoleDbView, WorldGameView, WorldLoginTimeoutTeamExit,
};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::organizations::organizingctrl::{
    OrganizingDeleteRoleBlock, OrganizingDeleteRoleOutcome, WorldDeleteRoleOrganizingGate,
};
use crate::organizations::organizingparam::COrganizingParam;
use crate::persistence::rssetup::WorldTdsClient;
use crate::persistence::writelog::{WorldPlayerDeleteLogWrite, WorldWriteLogCommand};
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

/// Ветвь `0x4FB06`: выход из команды и снятие отметок выполняются после
/// lookup по account в исходном порядке мутаций.
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

/// Ветвь `0x4FB07`: два маршрута завершения — online-игроку отправляется
/// пакет на его GameServer, затем оба маршрута снимают login-метку до ack
/// LoginServer.
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

pub const DELETE_ROLE_REQUEST: i32 = 0x0004_FB02;
pub const DELETE_ROLE_RESPONSE: i32 = 0x0001_FF03;
pub const DELETE_ROLE_REJECTED_STATUS: i8 = 0x13;
pub const DELETE_ROLE_SCHEDULED_STATUS: i8 = 0x14;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldPlayerDeleteLogOutcome {
    pub player_name: Vec<u8>,
    pub ip_address: Vec<u8>,
    pub queue_length_after: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldDeleteRoleDisposition {
    OrganizingRejected { code: i32 },
    AlreadyScheduled { deletion_time: i32 },
    Scheduled {
        deletion_time: i32,
        restore_was_present: bool,
        deletion_days: i8,
        delete_log: Option<WorldPlayerDeleteLogOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldDeleteRoleOutcome {
    OrganizingBlocked {
        account: Vec<u8>,
        player_id: u32,
        ip_address: Vec<u8>,
        source: OrganizingDeleteRoleBlock,
    },
    Responded {
        account: Vec<u8>,
        player_id: u32,
        ip_address: Vec<u8>,
        organizing: OrganizingDeleteRoleOutcome,
        disposition: WorldDeleteRoleDisposition,
        response_type: i32,
        status: i8,
        trailing_value: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

fn c_string_prefix(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

/// Ветвь `0x4FB02`: `_time` снимается до DB/organizing gates; organizing
/// gate и страновый has_job проходят узкими dyn-швами, три DB-запроса —
/// boxed future по ADR-0013; журнал удаления ставится в FIFO только после
/// success-ответа в очередь LoginServer.
#[allow(clippy::too_many_arguments, reason = "границы один к одному соответствуют owner-ам ветки delete-role")]
pub async fn on_delete_role(
    game: &mut dyn WorldGameView,
    organizing: &mut dyn WorldDeleteRoleOrganizingGate,
    country_gate: &mut dyn WorldDeleteRoleCountryGate,
    organizing_parameters: &COrganizingParam,
    globe_setup: &GlobeSetupSnapshot,
    db: &mut dyn WorldDeleteRoleDbView,
    mut player_database: Option<&mut WorldTdsClient>,
    delete_log_enabled: bool,
    mut request: CMessage,
) -> WorldDeleteRoleOutcome {
    let account = request
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let player_id = request.base_mut().get_long().unwrap_or(0) as u32;
    let ip_raw = request.base_mut().get_long().unwrap_or(0) as u32;
    let ip_address = format!(
        "{}.{}.{}.{}",
        ip_raw & 0xff,
        (ip_raw >> 8) & 0xff,
        (ip_raw >> 16) & 0xff,
        (ip_raw >> 24) & 0xff,
    )
    .into_bytes();
 // `_time` расположен до DB/organizing gates; в deletion-list
 // сохранялись младшие 32 бита Windows `long`.
    let deletion_time = chrono::Local::now().timestamp() as i32;

    let country = db
        .get_player_country_by_id(player_id, player_database.as_deref_mut())
        .await;
    let country_has_job = if country == 0 {
        false
    } else {
        country_gate.country_has_job(&*game, country, player_id as i32)
    };
    let organizing_outcome = match organizing.apply_delete_role(
        &*game,
        organizing_parameters,
        player_id as i32,
        country_has_job,
    ) {
        Ok(outcome) => outcome,
        Err(source) => {
            return WorldDeleteRoleOutcome::OrganizingBlocked {
                account,
                player_id,
                ip_address,
                source,
            };
        }
    };

    let organizing_code = organizing_outcome.legacy_code();
    if organizing_code != 0 {
        return send_delete_role_response(
            game,
            account,
            player_id,
            ip_address,
            organizing_outcome,
            WorldDeleteRoleDisposition::OrganizingRejected {
                code: organizing_code,
            },
            DELETE_ROLE_REJECTED_STATUS,
            organizing_code,
        );
    }

    let mut existing_deletion_time = game.deletion_player_time(player_id);
    if existing_deletion_time == 0 {
        existing_deletion_time = db
            .get_player_deletion_date(player_id, player_database.as_deref_mut())
            .await;
    }
    if existing_deletion_time != 0 {
        return send_delete_role_response(
            game,
            account,
            player_id,
            ip_address,
            organizing_outcome,
            WorldDeleteRoleDisposition::AlreadyScheduled {
                deletion_time: existing_deletion_time,
            },
            DELETE_ROLE_REJECTED_STATUS,
            0,
        );
    }

    let restore_was_present = game.is_restore_player_exist(player_id);
    game.delete_restore_player(player_id);
    game.append_deletion_player(player_id, deletion_time);
    let deletion_days = globe_setup.deletion_days() as u8 as i8;
    let mut response = CMessage::new(DELETE_ROLE_RESPONSE);
    response.base_mut().add_char(DELETE_ROLE_SCHEDULED_STATUS);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(c_string_prefix(&account));
    response.base_mut().add_char(0);
    response.base_mut().add_char(deletion_days);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );

 // optional log начинается только после постановки success-ответа в
 // LoginServer queue. Parameter binding заменяет `_sprintf`, не меняя FIFO.
    let delete_log = if delete_log_enabled {
        let player_name = db
            .get_player_name_by_id(player_id, player_database.as_deref_mut())
            .await;
        let queue_length_after = game.push_write_log_command(
            WorldWriteLogCommand::PlayerDeleteLog(WorldPlayerDeleteLogWrite {
                player_id: player_id as i32,
                player_name: player_name.clone(),
                ip_address: ip_address.clone(),
            }),
        );
        Some(WorldPlayerDeleteLogOutcome {
            player_name,
            ip_address: ip_address.clone(),
            queue_length_after,
        })
    } else {
        None
    };

    WorldDeleteRoleOutcome::Responded {
        account,
        player_id,
        ip_address,
        organizing: organizing_outcome,
        disposition: WorldDeleteRoleDisposition::Scheduled {
            deletion_time,
            restore_was_present,
            deletion_days,
            delete_log,
        },
        response_type: DELETE_ROLE_RESPONSE,
        status: DELETE_ROLE_SCHEDULED_STATUS,
        trailing_value: i32::from(deletion_days),
        wire,
        delivery,
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "typed outcome сохраняет все наблюдаемые поля одного exact ответа"
)]
fn send_delete_role_response(
    game: &mut dyn WorldGameView,
    account: Vec<u8>,
    player_id: u32,
    ip_address: Vec<u8>,
    organizing: OrganizingDeleteRoleOutcome,
    disposition: WorldDeleteRoleDisposition,
    status: i8,
    trailing_value: i32,
) -> WorldDeleteRoleOutcome {
    let mut response = CMessage::new(DELETE_ROLE_RESPONSE);
    response.base_mut().add_char(status);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(c_string_prefix(&account));
    response.base_mut().add_char(0);
    response.base_mut().add_long(trailing_value);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldDeleteRoleOutcome::Responded {
        account,
        player_id,
        ip_address,
        organizing,
        disposition,
        response_type: DELETE_ROLE_RESPONSE,
        status,
        trailing_value,
        wire,
        delivery,
    }
}
