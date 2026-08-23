//! World lifecycle-ветви LoginServer из `applogin/message/servermessage.cpp`.
//!
//! `0x1FE01`/`0xFF01` (World lifecycle) и `0x1FE02`/`0x1FE03` (CD-key
//! snapshot/clear), `0x1FE04` (World/Game telemetry) и
//! `0x1FE05`/`0x1FE06`/`0x1FE08` (`_serv_logs`). Контракт подтверждён точной
//! парой LoginServer EXE/PDB.
//!
//! Connect буквально читает `world_id/name`, сначала ставит numeric identity
//! socket, затем вызывает `AddWorld`, пишет операторский результат, отправляет
//! `0x4FC03 + area_id` тому же socket и только после этого условно ставит
//! запись `_serv_logs`. Даже rejected `AddWorld` не отменяет ack и connect-log;
//! ошибка ack также не пропускает более позднюю log-позицию. Disconnect берёт
//! ID из metadata, только для известного имени пишет lost-log и очищает CD-key,
//! после чего всегда вызывает `DelWorld`.
//!
//! `CGame` сохраняет setup-карту: connect/disconnect меняют её state-level, но
//! не состав. Обе мутации рассылают `0xAF509` account без выбранного World,
//! обновляют `CLoginQueue::m_nWordNum` и затем соответственно заменяют либо
//! удаляют `s_listCdkey[world_id]`. Проигнорированные исходником ошибки рассылки
//! и ack остаются typed-результатами, а не меняют порядок side effects.
//!
//! Строка connect-записи `_serv_logs` сохраняется byte-exact:
//! `WS%s` + CP936 `在` + `_strdate` + пробел + `_strtime` + CP936 `连接` +
//! `LS.`; поля queue равны `(peer_ip, -2, world_id, description)`. Два вызова
//! `Local::now` сохраняют отдельные позиции старых `_strdate/_strtime`.
//! MFC `UpdateDisplayWorldInfo` не переносится в Linux runtime: наблюдаемое
//! состояние уже принадлежит `CGame`, а Windows GUI не является контрактом.
//! FreeType/STL/SEH cleanup из сырого экспорта удалён как library/compiler noise.
//!
//! `0x1FE02` сохраняет short-circuit: при нулевом World ID второй `long` даже
//! не читается; любой ненулевой count создаёт owned DB-контекст, но строки
//! читаются только пока count положителен. Каждый account сначала независимо
//! передаётся `AddCdkey`, затем всегда попадает в DB snapshot, даже при
//! duplicate/неизвестном World. `UpdateOnlineUser2DB` выполняется отдельным
//! owned worker: DELETE и INSERT идут в исходном порядке, ошибки отдельных
//! statements не останавливают следующие. `0x1FE03` вызывает `ClearCDKey`
//! только для положительного count.
//!
//! `0x1FE04` безусловно добавляет один World snapshot. World player count,
//! map-ID port и оба GameServer numeric-поля сохраняют исходный 32-битный bit
//! pattern; Game-записи читаются только при положительном signed count.
//! Потерянные IPv4-varargs подтверждены точным EXE: точный EXE
//! старшему, что совпадает с общим `legacy_ipv4_word`.
//!
//! Три server-log opcode проверяют `dwServerInfoLogTime` до чтения payload и
//! при нуле немедленно возвращаются. `0x1FE06` ставит тип `-2`, `0x1FE08`
//! sign-расширяет исходный `char`, обе ветви глубоко копируют готовые `0x80`-
//! ограниченные bytes. Формат `0x1FE05` подтверждено точным EXE:
//! передаёт `_strdate`, `_strtime`, затем signed `m_lMapID`. Source IP и второй
//! `long` входят только в поля queue. Два `Local::now` сохраняют отдельные
//! позиции старых date/time вызовов; описание остаётся byte-exact.

use std::net::Ipv4Addr;

use chrono::Local;

use crate::loginserver::loginserver::game::{
    CGame, GameRouteError, OnlineUserUpdateStartError, PingGameServerInfo, PingWorldServerInfo,
    ServerInfoLogDisposition, WorldActivationOutcome, WorldDeactivationOutcome,
};
use crate::nets::netlogin::message::CMessage;

const WORLD_CONNECTED_MESSAGE_TYPE: i32 = 0x0001_FE01;
const WORLD_DISCONNECTED_MESSAGE_TYPE: i32 = 0x0000_FF01;
const WORLD_CONNECTED_ACK_MESSAGE_TYPE: i32 = 0x0004_FC03;
const WORLD_USERS_SNAPSHOT_MESSAGE_TYPE: i32 = 0x0001_FE02;
const CLEAR_ACCOUNTS_MESSAGE_TYPE: i32 = 0x0001_FE03;
const WORLD_TELEMETRY_MESSAGE_TYPE: i32 = 0x0001_FE04;
const GAME_SERVER_CONNECTED_LOG_MESSAGE_TYPE: i32 = 0x0001_FE05;
const WORLD_SERVER_LOG_MESSAGE_TYPE: i32 = 0x0001_FE06;
const TYPED_SERVER_LOG_MESSAGE_TYPE: i32 = 0x0001_FE08;
const SERVER_STRING_LIMIT: usize = 0x100;
const GAME_SERVER_IP_LIMIT: usize = 0xFF;
const SERVER_LOG_TEXT_LIMIT: usize = 0x80;

/// Наблюдаемый результат восстановленных ветвей `OnServerMessage`.
#[derive(Debug)]
pub(crate) enum ServerMessageOutcome {
    WorldConnected {
        world_id: i32,
        world_name: Vec<u8>,
        map_assignment: i32,
        activation: WorldActivationOutcome,
        acknowledgement: Result<i32, GameRouteError>,
        server_info_log: ServerInfoLogDisposition,
    },
    WorldDisconnected {
        world_id: i32,
        /// `Some` соответствует позиции исходного `AddLogText(...lost!)`.
        world_name: Option<Vec<u8>>,
        deactivation: WorldDeactivationOutcome,
    },
    WorldUsersSnapshot {
        world_id: i32,
        /// `None` сохраняет short-circuit нулевого World ID.
        declared_count: Option<i32>,
        processed_accounts: usize,
        added_accounts: usize,
        database_update: Option<Result<(), OnlineUserUpdateStartError>>,
    },
    AccountsCleared {
        declared_count: i32,
        processed_accounts: usize,
    },
    WorldTelemetrySnapshot {
        snapshot_index: usize,
        declared_game_server_count: i32,
        processed_game_servers: usize,
    },
    ServerInfoLog {
        message_type: i32,
        disposition: ServerInfoLogDisposition,
        source_ip: Option<u32>,
        server_type: Option<i32>,
        server_number: Option<i32>,
    },
    Unsupported {
        message_type: i32,
    },
}

/// Узкая композиция `OnServerMessage` с фактическим `CGame` LoginServer.
pub(crate) struct ServerMessageHandler<'a> {
    game: &'a mut CGame,
}

impl<'a> ServerMessageHandler<'a> {
    /// Связывает server-handler с текущим LoginServer owner.
    pub(crate) fn new(game: &'a mut CGame) -> Self {
        Self { game }
    }

    /// Выполняет все подтверждённые ветви и сохраняет default как no-op.
    pub(crate) fn on_server_message(
        &mut self,
        message: &mut CMessage,
    ) -> Result<ServerMessageOutcome, GameRouteError> {
        match message.message_type() {
            WORLD_CONNECTED_MESSAGE_TYPE => self.on_world_connected(message),
            WORLD_DISCONNECTED_MESSAGE_TYPE => Ok(self.on_world_disconnected(message)),
            WORLD_USERS_SNAPSHOT_MESSAGE_TYPE => Ok(self.on_world_users_snapshot(message)),
            CLEAR_ACCOUNTS_MESSAGE_TYPE => Ok(self.on_accounts_cleared(message)),
            WORLD_TELEMETRY_MESSAGE_TYPE => Ok(self.on_world_telemetry(message)),
            GAME_SERVER_CONNECTED_LOG_MESSAGE_TYPE => {
                Ok(self.on_game_server_connected_log(message))
            }
            WORLD_SERVER_LOG_MESSAGE_TYPE => Ok(self.on_world_server_log(message)),
            TYPED_SERVER_LOG_MESSAGE_TYPE => Ok(self.on_typed_server_log(message)),
            message_type => Ok(ServerMessageOutcome::Unsupported { message_type }),
        }
    }

    fn on_world_connected(
        &mut self,
        message: &mut CMessage,
    ) -> Result<ServerMessageOutcome, GameRouteError> {
        let world_id = message.base_mut().get_long().unwrap_or(0);
        let world_name = message
            .base_mut()
            .get_str_bytes(SERVER_STRING_LIMIT)
            .expect("ненулевая GetStr-граница задана константой");
        let socket_id = message.socket_id();
        let map_assignment = self.game.set_world_socket_map_id(socket_id, world_id)?;
        let activation = self.game.activate_world(world_id, &world_name);
        self.game.queue_world_connected_operator_log(&world_name);

        let mut acknowledgement = CMessage::new(WORLD_CONNECTED_ACK_MESSAGE_TYPE);
        acknowledgement.base_mut().add_long(self.game.area_id());
        let acknowledgement = self.game.send_to_world_socket(&acknowledgement, socket_id);
        let server_info_log =
            self.game
                .queue_world_connected_server_info_log(message.ip(), world_id, &world_name);

        Ok(ServerMessageOutcome::WorldConnected {
            world_id,
            world_name,
            map_assignment,
            activation,
            acknowledgement,
            server_info_log,
        })
    }

    fn on_world_disconnected(&mut self, message: &CMessage) -> ServerMessageOutcome {
        let world_id = message.map_id();
        let world_name = self.game.world_name_by_id(world_id).map(<[u8]>::to_vec);
        if let Some(world_name) = &world_name {
            self.game.queue_world_lost_operator_log(world_name);
            self.game.clear_cdkeys_by_world_id(world_id);
        }
        let deactivation = self.game.deactivate_world(world_id);
        ServerMessageOutcome::WorldDisconnected {
            world_id,
            world_name,
            deactivation,
        }
    }

    fn on_world_users_snapshot(&mut self, message: &mut CMessage) -> ServerMessageOutcome {
        let world_id = message.base_mut().get_long().unwrap_or(0);
        if world_id == 0 {
            return ServerMessageOutcome::WorldUsersSnapshot {
                world_id,
                declared_count: None,
                processed_accounts: 0,
                added_accounts: 0,
                database_update: None,
            };
        }
        let declared_count = message.base_mut().get_long().unwrap_or(0);
        if declared_count == 0 {
            return ServerMessageOutcome::WorldUsersSnapshot {
                world_id,
                declared_count: Some(declared_count),
                processed_accounts: 0,
                added_accounts: 0,
                database_update: None,
            };
        }

        let mut remaining = declared_count;
        let mut accounts = Vec::new();
        let mut added_accounts = 0;
        while remaining > 0 {
            let account = message
                .base_mut()
                .get_str_bytes(SERVER_STRING_LIMIT)
                .expect("ненулевая GetStr-граница задана константой");
            added_accounts += usize::from(self.game.add_cdkey(&account, world_id));
            accounts.push(account);
            remaining -= 1;
        }
        let processed_accounts = accounts.len();
        let database_update = Some(
            self.game
                .start_online_user_database_update(world_id, accounts),
        );
        ServerMessageOutcome::WorldUsersSnapshot {
            world_id,
            declared_count: Some(declared_count),
            processed_accounts,
            added_accounts,
            database_update,
        }
    }

    fn on_accounts_cleared(&mut self, message: &mut CMessage) -> ServerMessageOutcome {
        let declared_count = message.base_mut().get_long().unwrap_or(0);
        let mut remaining = declared_count;
        let mut processed_accounts = 0;
        while remaining > 0 {
            let account = message
                .base_mut()
                .get_str_bytes(SERVER_STRING_LIMIT)
                .expect("ненулевая GetStr-граница задана константой");
            self.game.clear_cdkey(&account);
            processed_accounts += 1;
            remaining -= 1;
        }
        ServerMessageOutcome::AccountsCleared {
            declared_count,
            processed_accounts,
        }
    }

    fn on_world_telemetry(&mut self, message: &mut CMessage) -> ServerMessageOutcome {
        let player_count = message.base_mut().get_long().unwrap_or(0) as u32;
        let ip = Ipv4Addr::from(message.ip().to_le_bytes())
            .to_string()
            .into_bytes();
        let port = message.map_id() as u32;
        let declared_game_server_count = message.base_mut().get_long().unwrap_or(0);
        let mut remaining = declared_game_server_count;
        let mut game_servers = Vec::new();
        while remaining > 0 {
            let ip = message
                .base_mut()
                .get_str_bytes(GAME_SERVER_IP_LIMIT)
                .expect("ненулевая GetStr-граница задана константой");
            let port = message.base_mut().get_long().unwrap_or(0) as u32;
            let player_count = message.base_mut().get_long().unwrap_or(0) as u32;
            game_servers.push(PingGameServerInfo::new(ip, port, player_count));
            remaining -= 1;
        }
        let processed_game_servers = game_servers.len();
        let snapshot_index = self
            .game
            .append_ping_world_server_info(PingWorldServerInfo::new(
                ip,
                port,
                player_count,
                game_servers,
            ));
        ServerMessageOutcome::WorldTelemetrySnapshot {
            snapshot_index,
            declared_game_server_count,
            processed_game_servers,
        }
    }

    fn on_game_server_connected_log(&mut self, message: &mut CMessage) -> ServerMessageOutcome {
        let message_type = message.message_type();
        let disposition = self.game.server_info_log_disposition();
        if disposition != ServerInfoLogDisposition::Queued {
            return server_info_log_outcome(message_type, disposition, None, None, None);
        }

        let source_ip = message.base_mut().get_long().unwrap_or(0) as u32;
        let server_number = message.base_mut().get_long().unwrap_or(0);
        let date = Local::now().format("%m/%d/%y").to_string();
        let time = Local::now().format("%H:%M:%S").to_string();
        let server_type = message.map_id();
        let mut description = Vec::with_capacity(date.len() + time.len() + 18);
        description.extend_from_slice(b"GS");
        description.extend_from_slice(&[0xD4, 0xDA]);
        description.extend_from_slice(date.as_bytes());
        description.push(b' ');
        description.extend_from_slice(time.as_bytes());
        description.extend_from_slice(&[0xC1, 0xAC, 0xBD, 0xD3]);
        description.extend_from_slice(b"WS(");
        description.extend_from_slice(server_type.to_string().as_bytes());
        description.extend_from_slice(b").");
        self.game
            .push_server_info_log(source_ip, server_type, server_number, description);
        server_info_log_outcome(
            message_type,
            ServerInfoLogDisposition::Queued,
            Some(source_ip),
            Some(server_type),
            Some(server_number),
        )
    }

    fn on_world_server_log(&mut self, message: &mut CMessage) -> ServerMessageOutcome {
        self.on_supplied_server_log(message, -2)
    }

    fn on_typed_server_log(&mut self, message: &mut CMessage) -> ServerMessageOutcome {
        let message_type = message.message_type();
        let disposition = self.game.server_info_log_disposition();
        if disposition != ServerInfoLogDisposition::Queued {
            return server_info_log_outcome(message_type, disposition, None, None, None);
        }
        let server_type = i32::from(message.base_mut().get_char().unwrap_or(0));
        self.on_supplied_server_log_after_gate(message, server_type)
    }

    fn on_supplied_server_log(
        &mut self,
        message: &mut CMessage,
        server_type: i32,
    ) -> ServerMessageOutcome {
        let message_type = message.message_type();
        let disposition = self.game.server_info_log_disposition();
        if disposition != ServerInfoLogDisposition::Queued {
            return server_info_log_outcome(message_type, disposition, None, None, None);
        }
        self.on_supplied_server_log_after_gate(message, server_type)
    }

    fn on_supplied_server_log_after_gate(
        &mut self,
        message: &mut CMessage,
        server_type: i32,
    ) -> ServerMessageOutcome {
        let source_ip = message.base_mut().get_long().unwrap_or(0) as u32;
        let server_number = message.base_mut().get_long().unwrap_or(0);
        let description = message
            .base_mut()
            .get_str_bytes(SERVER_LOG_TEXT_LIMIT)
            .expect("ненулевая GetStr-граница задана константой");
        self.game
            .push_server_info_log(source_ip, server_type, server_number, description);
        server_info_log_outcome(
            message.message_type(),
            ServerInfoLogDisposition::Queued,
            Some(source_ip),
            Some(server_type),
            Some(server_number),
        )
    }
}

fn server_info_log_outcome(
    message_type: i32,
    disposition: ServerInfoLogDisposition,
    source_ip: Option<u32>,
    server_type: Option<i32>,
    server_number: Option<i32>,
) -> ServerMessageOutcome {
    ServerMessageOutcome::ServerInfoLog {
        message_type,
        disposition,
        source_ip,
        server_type,
        server_number,
    }
}
