//! Обработчики `appauth/message/message_func.cpp`, подтверждённые
//! `authserver.exe` и `authserver.pdb`. Они связывают LoginServer-соединения,
//! Auth DB-очереди, GM-команды и coalescing server-info с состоянием `CGame`.
//!
//! Таблица LoginServer остаётся упорядоченной по area ID; disconnect удаляет
//! только первую найденную для socket запись. Auth-строки приводятся к нижнему
//! регистру как Windows-1251 русской поставки; локаль другой Windows-сессии
//! остаётся внешней границей, а непредставимый одиночный байт сохраняется.
//! Нехватка числового payload сохраняет `GetLong == 0` без движения курсора.
//!
//! Операторские MFC side effects представлены FIFO событий. Отказ соединения
//! идёт через `ServerCommandHandle`, а server-info передаётся в общий
//! `ServerInfoQueue`; wire-порядок и ветви GM-команд определены непосредственно
//! обработчиками ниже.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::net::Ipv4Addr;

use encoding_rs::WINDOWS_1251;

use crate::authserver::src::cgame::AuthDbContext;
use crate::authserver::src::dbqueue::{
    AuthQuestData, DbQuest, LockQuestData, LockUntil, ServerInfo,
};
use crate::authserver::src::kl_ipfilter::IpFilter;
use crate::nets::netauth::message::{AuthMessageHandler, AuthMessageKind, CMessage, DispatchError};
use crate::nets::servers::ServerCommandHandle;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LoginServerNotice {
    Connected {
        address: Ipv4Addr,
        socket_id: i32,
        allowed: bool,
    },
    Registered {
        address: Ipv4Addr,
        area_id: i32,
        socket_id: i32,
    },
    Disconnected {
        address: Ipv4Addr,
        area_id: i32,
        socket_id: i32,
    },
}

impl fmt::Display for LoginServerNotice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connected {
                address,
                socket_id,
                allowed,
            } => write!(
                formatter,
                "соединение {address}, socket {socket_id}, доступ {}",
                if *allowed {
                    "разрешён"
                } else {
                    "отклонён"
                }
            ),
            Self::Registered {
                address,
                area_id,
                socket_id,
            } => write!(
                formatter,
                "зарегистрирован {address}, area {area_id}, socket {socket_id}"
            ),
            Self::Disconnected {
                address,
                area_id,
                socket_id,
            } => write!(
                formatter,
                "отключён {address}, area {area_id}, socket {socket_id}"
            ),
        }
    }
}

pub(crate) struct AuthMessageHandlers {
    login_servers: BTreeMap<i32, i32>,
    ip_filter_enabled: bool,
    ip_allower: IpFilter<true>,
    game: AuthDbContext,
    notices: VecDeque<LoginServerNotice>,
}

impl AuthMessageHandlers {
    pub(crate) fn new(
        ip_filter_enabled: bool,
        allowed_patterns: Vec<[u8; 4]>,
        game: AuthDbContext,
    ) -> Self {
        let mut ip_allower = IpFilter::new();
        ip_allower.replace_patterns(allowed_patterns);
        Self {
            login_servers: BTreeMap::new(),
            ip_filter_enabled,
            ip_allower,
            game,
            notices: VecDeque::new(),
        }
    }

    pub(crate) fn replace_ip_filter(&mut self, enabled: bool, allowed_patterns: Vec<[u8; 4]>) {
        self.ip_filter_enabled = enabled;
        self.ip_allower.replace_patterns(allowed_patterns);
    }

    pub(crate) fn login_server_socket_id(&self, area_id: i32) -> i32 {
        self.login_servers.get(&area_id).copied().unwrap_or(0)
    }

    pub(crate) fn login_server_area_id(&self, socket_id: i32) -> i32 {
        self.area_for_socket(socket_id).unwrap_or(0)
    }

    pub(crate) fn login_server_socket_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.login_servers.values().copied()
    }

    pub(crate) fn pop_notice(&mut self) -> Option<LoginServerNotice> {
        self.notices.pop_front()
    }

    fn on_ls_connect(&mut self, message: &CMessage, sender: &ServerCommandHandle) {
        let address = peer_address(message);
        let allowed = !self.ip_filter_enabled || self.ip_allower.is_allowed(address.octets());
        self.notices.push_back(LoginServerNotice::Connected {
            address,
            socket_id: message.socket_id(),
            allowed,
        });
        if !allowed {
            sender.quit_by_socket_id(message.socket_id());
        }
    }

    fn on_ls_disconnect(&mut self, message: &CMessage) {
        let socket_id = message.socket_id();
        let area_id = self.area_for_socket(socket_id).unwrap_or(0);
        self.notices.push_back(LoginServerNotice::Disconnected {
            address: peer_address(message),
            area_id,
            socket_id,
        });

        if let Some(area_id) = self.area_for_socket(socket_id) {
            self.login_servers.remove(&area_id);
        }
    }

    fn on_ls_get_info(&mut self, message: &mut CMessage) {
        let area_id = message.base_mut().get_long().unwrap_or(0);
        let socket_id = message.socket_id();
        self.notices.push_back(LoginServerNotice::Registered {
            address: peer_address(message),
            area_id,
            socket_id,
        });
        self.login_servers.insert(area_id, socket_id);
    }

    fn on_update_server_info_response(&self, message: &mut CMessage) {
        let player_count = message.base_mut().get_long().unwrap_or(0);
        let game_server_id = message.base_mut().get_long().unwrap_or(0);
        let world_server_id = message.base_mut().get_long().unwrap_or(0);
        let login_server_id = message.base_mut().get_long().unwrap_or(0);
        self.game.push_server_info(ServerInfo::new(
            player_count,
            game_server_id,
            world_server_id,
            login_server_id,
        ));
    }

    fn on_auth_account(&self, message: &mut CMessage, extended: bool) {
        let account = legacy_lowercase(message.get_str());
        let password = legacy_lowercase(message.get_str());
        let client_ip = message.base_mut().get_long().unwrap_or(0) as u32;
        let client_socket_id = message.base_mut().get_long().unwrap_or(0);
        let request = AuthQuestData::new(account, password, client_ip, client_socket_id);
        let quest = if extended {
            DbQuest::AuthenticateExtended {
                return_socket_id: message.socket_id(),
                request,
            }
        } else {
            DbQuest::Authenticate {
                return_socket_id: message.socket_id(),
                request,
            }
        };
        self.game.push_quest(quest);
    }

    fn on_gm_lock_account(&self, message: &mut CMessage) {
        let account = message.get_str();
        let until = LockUntil {
            year: message.base_mut().get_word().unwrap_or(0),
            month: message.base_mut().get_word().unwrap_or(0),
            day: message.base_mut().get_word().unwrap_or(0),
            hour: message.base_mut().get_word().unwrap_or(0),
            minute: message.base_mut().get_word().unwrap_or(0),
            second: message.base_mut().get_word().unwrap_or(0),
        };
        self.game.push_quest(DbQuest::Lock {
            return_socket_id: message.socket_id(),
            request: LockQuestData { account, until },
        });
    }

    fn on_gm_kick_player(
        &self,
        message: &mut CMessage,
        sender: &ServerCommandHandle,
    ) -> Result<(), DispatchError> {
        let area_id = message.base_mut().get_long().unwrap_or(0);
        let account = message.get_str();
        let reason = message.base_mut().get_char().unwrap_or(0);
        let operator = message.get_str();
        let target_socket_id = self.login_server_socket_id(area_id);

        if target_socket_id == 0 {
            let mut response = CMessage::new(0x0010_F201);
            response.base_mut().add_char(0);
            add_legacy_string(response.base_mut(), &operator);
            let detail = format!("AuthServer : Invalid login server area id : {area_id}!");
            add_legacy_string(response.base_mut(), detail.as_bytes());
            response
                .send_to_login(sender, message.socket_id())
                .map_err(DispatchError::OutgoingMessage)?;
        } else {
            let mut forward = CMessage::new(0x000C_F701);
            forward.base_mut().add_long(message.socket_id());
            add_legacy_string(forward.base_mut(), &account);
            forward.base_mut().add_char(reason);
            add_legacy_string(forward.base_mut(), &operator);
            forward
                .send_to_login(sender, target_socket_id)
                .map_err(DispatchError::OutgoingMessage)?;
        }
        Ok(())
    }

    fn on_kick_player_response(
        &self,
        message: &mut CMessage,
        sender: &ServerCommandHandle,
    ) -> Result<(), DispatchError> {
        let return_socket_id = message.base_mut().get_long().unwrap_or(0);
        let result = message.base_mut().get_char().unwrap_or(0);
        let mut response = CMessage::new(0x0010_F201);
        response.base_mut().add_char(result);
        if result == 0 {
            let account = message.get_str();
            let detail = message.get_str();
            add_legacy_string(response.base_mut(), &account);
            add_legacy_string(response.base_mut(), &detail);
        } else {
            let detail = message.get_str();
            add_legacy_string(response.base_mut(), &detail);
        }
        response
            .send_to_login(sender, return_socket_id)
            .map_err(DispatchError::OutgoingMessage)?;
        Ok(())
    }

    fn area_for_socket(&self, socket_id: i32) -> Option<i32> {
        self.login_servers
            .iter()
            .find_map(|(&area_id, &registered_socket)| {
                (registered_socket == socket_id).then_some(area_id)
            })
    }
}

impl AuthMessageHandler for AuthMessageHandlers {
    fn handle(
        &mut self,
        kind: AuthMessageKind,
        message: &mut CMessage,
        sender: &ServerCommandHandle,
    ) -> Result<(), DispatchError> {
        match kind {
            AuthMessageKind::LoginServerConnected => self.on_ls_connect(message, sender),
            AuthMessageKind::LoginServerDisconnected => self.on_ls_disconnect(message),
            AuthMessageKind::GetLoginServerInfo => self.on_ls_get_info(message),
            AuthMessageKind::UpdateServerInfoResponse => {
                self.on_update_server_info_response(message)
            }
            AuthMessageKind::AuthenticateAccount => self.on_auth_account(message, false),
            AuthMessageKind::AuthenticateAccountExtended => self.on_auth_account(message, true),
            AuthMessageKind::GmLockAccount => self.on_gm_lock_account(message),
            AuthMessageKind::GmKickPlayer => self.on_gm_kick_player(message, sender)?,
            AuthMessageKind::KickPlayerResponse => self.on_kick_player_response(message, sender)?,
        }
        Ok(())
    }
}

fn peer_address(message: &CMessage) -> Ipv4Addr {
    Ipv4Addr::from(message.ip().to_le_bytes())
}

fn add_legacy_string(message: &mut crate::nets::basemessage::CBaseMessage, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    message.add(&value[..end]);
    message.add_byte(0);
}

fn legacy_lowercase(value: Vec<u8>) -> Vec<u8> {
    value
        .into_iter()
        .map(|byte| {
            let source = [byte];
            let (decoded, _, decode_failed) = WINDOWS_1251.decode(&source);
            if decode_failed {
                return byte;
            }
            let lowered = decoded.to_lowercase();
            let (encoded, _, encode_failed) = WINDOWS_1251.encode(&lowered);
            if !encode_failed && encoded.len() == 1 {
                encoded[0]
            } else {
                byte
            }
        })
        .collect()
}
