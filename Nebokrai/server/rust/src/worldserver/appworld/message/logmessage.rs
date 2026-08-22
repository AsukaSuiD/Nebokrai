//! WorldServer dispatcher-owner `OnLogMessage`.
//!
//! Корпус остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме player lifecycle leaf-ов
//! `0x5FB01/0x5FB02`, restore-role leaf `0x4FB03` и account cleanup leaf-ов
//! `0x4FB06/0x4FB07` со статусом `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`; исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\logmessage.cpp:30`.
//! Exact `0x004B1692..0x004B171F` читает account через `GetStr(..., 0x14)`,
//! затем signed player ID, удаляет первое совпадение из live deletion-list,
//! добавляет уникальный ID в хвост restore-list и посылает в текущий
//! LoginServer `0x1FF04 + char(0x15) + player_id + account\0` без priority.
//! Exact `0x004B0F78..0x004B1069` читает account через `GetStr(..., 0x100)`,
//! находит первое `_strcmpi` совпадение в login-list и строго выполняет
//! `team exit -> RemovePlayerLoadData -> RemoveLoginPlayer ->
//! AppendOfflinePlayer`.
//! Exact `0x004B106E..0x004B12C9` сначала ищет online account с достигнутым
//! GameServer: посылает `0x7F903 + player_id`, выполняет team-exit и немедленно
//! возвращается. Только без такого маршрута он чистит первый login account и
//! всегда отвечает LoginServer `0x1FF06 + account\0 + ""\0 + char(0)`.
//! Exact `0x004B1EF0..0x004B20CD` читает player ID и два legacy long,
//! различает offline/online/wrong-map кодами `0/-1/-2`, а на совпавшем map
//! дописывает полный `CPlayer` в исходное сообщение, меняет opcode на
//! `0x7F901`, отправляет тому же socket и только потом переводит игрока из
//! login/offline в online. Safe codec block не заменяется частичным wire.
//! EXE действительно вызывает два ignored `GetLong` после player ID, хотя
//! Linux-донор объявил forwarded body как `long + char + long`; строгий
//! 12-байтный donor-gate не перенесён, а исходный payload в success-ответе
//! остаётся byte-exact независимо от результата безопасных ignored reads.
//! Exact `0x004B20D2..0x004B23BB` принимает optional subtype-`1` snapshot,
//! очищает transient pet-вектор и faction-data flag, затем посылает LoginServer
//! `0x1FF06 + account\0 + name\0 + level`, выполняет login/online/offline и
//! team переходы и уведомляет каждого достигнутого online-друга через
//! `0x7F905 + friend_id + returned_name\0` в исходном list-порядке.
//! Добавленные Linux-донором peer/ownership/DB-preflight gates в EXE
//! отсутствуют и не перенесены; как и account-wide cancellation/in-flight
//! lifecycle из его очереди. `VecDeque` заменяет только старые list/deque
//! nodes, а существующая client FIFO — WinSock transport без изменения
//! wire/order.
//!
//! Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже сохранён как локальная
//! документация, а не как Rust-реализация.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::appworld::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::worldserver::appworld::organizingsystem::organizingctrl::COrganizingCtrl;
use crate::worldserver::appworld::player::{PlayerCodecError, PlayerPropertyCoefficients};
use crate::worldserver::appworld::session::csessionfactory::CSessionFactory;
use crate::worldserver::worldserver::game::{
    CGame, WorldLoginTimeoutTeamExit, WorldOnlinePlayerAppendOutcome,
    WorldOnlinePlayerRemoveOutcome, WorldReturnedPlayerDecode,
};
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

const PLAYER_DETAIL_REQUEST: i32 = 0x0005_FB01;
const PLAYER_RETURN_REQUEST: i32 = 0x0005_FB02;
const RESTORE_ROLE_REQUEST: i32 = 0x0004_FB03;
const ACCOUNT_LOGIN_CLEANUP_REQUEST: i32 = 0x0004_FB06;
const ACCOUNT_DISCONNECT_REQUEST: i32 = 0x0004_FB07;
const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
const RESTORE_ROLE_STATUS: i8 = 0x15;
const ACCOUNT_DISCONNECT_GAME_RESPONSE: i32 = 0x0007_F903;
const ACCOUNT_DISCONNECT_LOGIN_RESPONSE: i32 = 0x0001_FF06;
const PLAYER_DETAIL_RESPONSE: i32 = 0x0007_F901;
const PLAYER_FRIEND_OFFLINE_RESPONSE: i32 = 0x0007_F905;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRestoreRoleOutcome {
    pub(crate) account: Vec<u8>,
    pub(crate) player_id: u32,
    pub(crate) player_id_complete: bool,
    pub(crate) response_type: i32,
    pub(crate) status: i8,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldAccountLoginCleanupOutcome {
    NotFound {
        account: Vec<u8>,
    },
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
pub(crate) enum WorldAccountDisconnectOutcome {
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerDetailOutcome {
    Rejected {
        player_id: u32,
        status: i32,
        source_socket_id: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    WrongGameServer {
        player_id: u32,
        owner_id: i32,
        source_map_id: i32,
        assigned_map_id: i32,
        source_socket_id: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        login_removed: bool,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
        faction_data_reset: bool,
    },
    PromotedOnline {
        player_id: u32,
        owner_id: i32,
        source_map_id: i32,
        source_socket_id: i32,
        response_type: i32,
        encoded_bytes: usize,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        login_removed: bool,
        offline_removal_completed: bool,
        online_append: WorldOnlinePlayerAppendOutcome,
    },
    SerializationBlocked {
        player_id: u32,
        error: PlayerCodecError,
    },
    SerializationOwnerMissing {
        player_id: u32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldReturnedPlayerFriendOutcome {
    pub(crate) friend_player_id: u32,
    pub(crate) game_server_index: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerReturnOutcome {
    PlayerOnlyOnWorld { player_id: u32 },
    DecodeBlocked { player_id: u32, error: PlayerCodecError },
    PlayerMissing { player_id: u32, subtype: i32 },
    Released {
        player_id: u32,
        subtype: i32,
        decode: Option<WorldReturnedPlayerDecode>,
        login_wire: Vec<u8>,
        login_delivery: Result<i32, SendMessageError>,
        login_removed: bool,
        online_removal: WorldOnlinePlayerRemoveOutcome,
        offline_inserted: bool,
        team_exit: Option<WorldLoginTimeoutTeamExit>,
        friends: Vec<WorldReturnedPlayerFriendOutcome>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldLogMessageOutcome {
    RestoreRole(WorldRestoreRoleOutcome),
    AccountLoginCleanup(WorldAccountLoginCleanupOutcome),
    AccountDisconnect(WorldAccountDisconnectOutcome),
    PlayerDetail(WorldPlayerDetailOutcome),
    PlayerReturn(WorldPlayerReturnOutcome),
}

pub(crate) enum WorldLogMessageDispatch {
    Handled(WorldLogMessageOutcome),
    Pending(CMessage),
}

pub(crate) fn on_log_message(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    session_factory: &mut CSessionFactory,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    message: CMessage,
) -> WorldLogMessageDispatch {
    match message.message_type() {
        PLAYER_DETAIL_REQUEST => player_detail(
            game,
            organizing,
            registry,
            coefficients,
            add_error_log_text,
            message,
        ),
        PLAYER_RETURN_REQUEST => player_return(
            game,
            organizing,
            session_factory,
            registry,
            coefficients,
            add_error_log_text,
            message,
        ),
        RESTORE_ROLE_REQUEST => restore_role(game, message),
        ACCOUNT_LOGIN_CLEANUP_REQUEST => {
            account_login_cleanup(game, session_factory, message)
        }
        ACCOUNT_DISCONNECT_REQUEST => account_disconnect(game, session_factory, message),
        _ => WorldLogMessageDispatch::Pending(message),
    }
}

fn player_return(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    session_factory: &mut CSessionFactory,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
    let subtype = message.base_mut().get_long().unwrap_or(0);
    let decode = if subtype == 1 {
        let decoded = {
            let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
            game.decord_returned_player(
                organizing,
                player_id,
                source,
                cursor,
                registry,
                coefficients,
            )
        };
        match decoded {
            Ok(decoded) => Some(decoded),
            Err(error) => {
                return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
                    WorldPlayerReturnOutcome::DecodeBlocked { player_id, error },
                ));
            }
        }
    } else {
        if subtype == 0 {
            let line = format!("Player {} Only On WorldServer!", player_id);
            let _ = add_error_log_text(line.as_bytes());
        }
        None
    };

    let Some(player) = game.returned_player_snapshot(player_id) else {
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
            if subtype == 0 {
                WorldPlayerReturnOutcome::PlayerOnlyOnWorld { player_id }
            } else {
                WorldPlayerReturnOutcome::PlayerMissing { player_id, subtype }
            },
        ));
    };

    let mut login = CMessage::new(ACCOUNT_DISCONNECT_LOGIN_RESPONSE);
    login.base_mut().add(&player.account);
    login.base_mut().add_char(0);
    login.base_mut().add(&player.name);
    login.base_mut().add_char(0);
    login.base_mut().add_byte(player.level);
    let login_wire = login.as_wire_bytes().to_vec();
    let login_delivery = login.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );

    let owner_id = player.owner_id as u32;
    let login_removed = game.remove_login_player(owner_id);
    let online_removal = game.remove_online_player(organizing, owner_id);
    let offline_inserted = game.append_offline_player_id(owner_id);
    let team_exit = if player.team_id == 0 {
        None
    } else {
        let session_id = game.get_team_session_id(player.team_id as u32);
        Some(game.exit_team_player(
            session_factory,
            session_id,
            player.owner_type,
            player.owner_id,
        ))
    };

    let mut friends = Vec::new();
    for friend_name in player.friend_names {
        let friend_player_id = game.online_player_id_by_name(&friend_name);
        if friend_player_id == 0 {
            continue;
        }
        let game_server_index = game.game_server_number_by_player_id(friend_player_id as i32);
        let mut presence = CMessage::new(PLAYER_FRIEND_OFFLINE_RESPONSE);
        presence.base_mut().add_ulong(friend_player_id);
        presence.base_mut().add(&player.name);
        presence.base_mut().add_char(0);
        let wire = presence.as_wire_bytes().to_vec();
        let delivery = game.send_msg_to_game_server(game_server_index, &presence);
        friends.push(WorldReturnedPlayerFriendOutcome {
            friend_player_id,
            game_server_index,
            wire,
            delivery,
        });
    }

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerReturn(
        WorldPlayerReturnOutcome::Released {
            player_id,
            subtype,
            decode,
            login_wire,
            login_delivery,
            login_removed,
            online_removal,
            offline_inserted,
            team_exit,
            friends,
        },
    ))
}

fn player_detail(
    game: &mut CGame,
    organizing: &mut COrganizingCtrl,
    registry: &GoodsBasePropertiesRegistry,
    coefficients: &PlayerPropertyCoefficients,
    add_error_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let player_id = message.base_mut().get_long().unwrap_or(0) as u32;
    let _ = message.base_mut().get_long();
    let _ = message.base_mut().get_long();

    let Some(player) = game.login_player_route_snapshot(player_id) else {
        let (status, error_text) = if game.online_player_by_id(player_id).is_some() {
            (
                -1,
                format!(
                    "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> Charactor Is Online!",
                    player_id
                ),
            )
        } else {
            (
                0,
                format!(
                    "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> Charactor Is Offline",
                    player_id
                ),
            )
        };
        let (wire, delivery) = send_player_detail_status(
            game,
            source_socket_id,
            status,
            player_id,
        );
        let _ = add_error_log_text(error_text.as_bytes());
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
            WorldPlayerDetailOutcome::Rejected {
                player_id,
                status,
                source_socket_id,
                response_type: PLAYER_DETAIL_RESPONSE,
                wire,
                delivery,
            },
        ));
    };

    let assigned_map_id = game.game_server_number_by_region_id(player.region_id);
    if assigned_map_id != source_map_id {
        let (wire, delivery) = send_player_detail_status(
            game,
            source_socket_id,
            -2,
            player_id,
        );
        let owner_id = player.owner_id as u32;
        let login_removed = game.remove_login_player(owner_id);
        let online_removal = game.remove_online_player(organizing, owner_id);
        let offline_inserted = game.append_offline_player_id(owner_id);
        let faction_data_reset = game.reset_map_player_faction_data(player.map_key);
        let error_text = format!(
            "MSG_S2W_LOG_QUEST_PLAYERDATA Invalid Request For Player Detail ID <{}> GameServer Invalid",
            player_id
        );
        let _ = add_error_log_text(error_text.as_bytes());
        return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
            WorldPlayerDetailOutcome::WrongGameServer {
                player_id,
                owner_id: player.owner_id,
                source_map_id,
                assigned_map_id,
                source_socket_id,
                response_type: PLAYER_DETAIL_RESPONSE,
                wire,
                delivery,
                login_removed,
                online_removal,
                offline_inserted,
                faction_data_reset,
            },
        ));
    }

    message.set_message_type(PLAYER_DETAIL_RESPONSE);
    let encoded = match game.encode_map_player_full_snapshot(
        organizing,
        player.map_key,
        registry,
        coefficients,
    ) {
        Ok(Some(encoded)) => encoded,
        Ok(None) => {
            return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
                WorldPlayerDetailOutcome::SerializationOwnerMissing { player_id },
            ));
        }
        Err(error) => {
            return WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
                WorldPlayerDetailOutcome::SerializationBlocked { player_id, error },
            ));
        }
    };
    let encoded_bytes = encoded.len();
    message.base_mut().add(&encoded);
    let wire = message.as_wire_bytes().to_vec();
    let sender = game.current_game_server_sender();
    let delivery = message.send_to_socket(sender.as_ref(), source_socket_id);
    let owner_id = player.owner_id as u32;
    let login_removed = game.remove_login_player(owner_id);
    game.remove_offline_player(owner_id);
    let online_append = game.append_online_player_id(organizing, player.owner_id);

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::PlayerDetail(
        WorldPlayerDetailOutcome::PromotedOnline {
            player_id,
            owner_id: player.owner_id,
            source_map_id,
            source_socket_id,
            response_type: PLAYER_DETAIL_RESPONSE,
            encoded_bytes,
            wire,
            delivery,
            login_removed,
            offline_removal_completed: true,
            online_append,
        },
    ))
}

fn send_player_detail_status(
    game: &CGame,
    socket_id: i32,
    status: i32,
    player_id: u32,
) -> (Vec<u8>, Result<i32, SendMessageError>) {
    let mut response = CMessage::new(PLAYER_DETAIL_RESPONSE);
    response.base_mut().add_long(status);
    response.base_mut().add_ulong(player_id);
    let wire = response.as_wire_bytes().to_vec();
    let sender = game.current_game_server_sender();
    let delivery = response.send_to_socket(sender.as_ref(), socket_id);
    (wire, delivery)
}

fn restore_role(game: &mut CGame, mut message: CMessage) -> WorldLogMessageDispatch {
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
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::RestoreRole(
        WorldRestoreRoleOutcome {
            account,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            response_type: RESTORE_ROLE_RESPONSE,
            status: RESTORE_ROLE_STATUS,
            wire,
            delivery,
        },
    ))
}

fn account_login_cleanup(
    game: &mut CGame,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    let account = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");
    let Some(player) = game.login_player_by_account(&account) else {
        return WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountLoginCleanup(
                WorldAccountLoginCleanupOutcome::NotFound { account },
            ),
        );
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

    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::AccountLoginCleanup(
        WorldAccountLoginCleanupOutcome::Cleaned {
            account,
            player_id,
            team_id: player.team_id,
            team_session_id,
            team_exit,
            player_load_removed,
            login_removed,
            offline_inserted,
        },
    ))
}

fn account_disconnect(
    game: &mut CGame,
    session_factory: &mut CSessionFactory,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
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
        return WorldLogMessageDispatch::Handled(
            WorldLogMessageOutcome::AccountDisconnect(
                WorldAccountDisconnectOutcome::OnlineRouted {
                    account,
                    player_id,
                    game_server_index: player.game_server_index,
                    response_type: ACCOUNT_DISCONNECT_GAME_RESPONSE,
                    wire,
                    delivery,
                    team_id: player.team_id,
                    team_session_id,
                    team_exit,
                },
            ),
        );
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
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::AccountDisconnect(
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
        },
    ))
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\logmessage.cpp

// ============================================================================
// FUNCTION: OnLogMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\logmessage.cpp:30
// RVA: 0x000B0D10
// ADDRESS: 004b0d10
// PROTOTYPE: void __cdecl OnLogMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
