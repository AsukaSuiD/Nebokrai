//! Свободные обработчики AuthServer из `appauth/message/message_func.cpp`.
//!
//! Контракт всех девяти функций таблицы Auth `InitMsgFuncPool`: восстановлено.
//! Исходный путь PDB:
//! Связанное состояние и helpers подтверждены в `cgame.cpp/.h`: `addLSItem`
//! `0x000020E0`, `gmaGetSocketID` `0x00003E60`, `gmaGetAreaID` `0x00003EA0`,
//! `delLSItem` `0x00004350`, `CheckConnection` `0x000058E0` и inline
//! `gmaRemoveLS` `0x00016A70`.
//!
//! Synthetic connect/disconnect получают socket ID и peer IPv4 из runtime-
//! metadata `CMessage`. Connect сначала публикует операторское событие, затем
//! при включённом исходном `mIPAllower` проверяет адрес и ставит
//! `QuitClientBySocketID` при отказе. Disconnect сначала находит первый area ID
//! по socket ID в порядке `std::map`, публикует событие и затем удаляет ровно
//! первую найденную запись. `BTreeMap<i32, i32>` сохраняет исходный
//! `area_id -> socket_id`, упорядоченный поиск и замену значения через
//! `operator[]` в `LSGetInfo`.
//! Ответ `0xCF802` читает четыре Windows `long` в порядке player/GS/WS/LS и
//! передаёт их в тот же coalescing owner `ServerInfoQueue`, который DB worker
//! снимает целиком при `WriteServerInfo`. Нехватка payload сохраняет старый
//! `GetLong == 0` без движения курсора.
//!
//! Auth/auth-ex читают account, password, client IPv4 и client socket, приводят
//! обе строки к lower-case и ставят owned DB quest с socket ID текущего
//! LoginServer. Точный EXE вызывает `CharLowerA`, а язык преобразования получал
//! из внешней Windows-сессии и внутри процесса не закреплял. Linux-вариант
//! фиксирует Windows-1251 русской поставки через `encoding_rs`; для запуска
//! оригинала под иной системной локалью преобразование байтов `0x80..=0xff`
//! остаётся внешней границей. Непредставимый одиночный byte сохраняется
//! буквально. Промежуточный общий `sprintf`-буфер dotted IPv4 использовался
//! только Windows GUI/log путями и не получает отдельного server-state.
//!
//! GM lock читает account и шесть `u16` полей времени в исходном порядке.
//! GM kick маршрутизирует `0xCF701` по area ID, сохраняя socket отправителя,
//! account, однобайтовую причину и operator string. Нулевой socket sentinel
//! возвращает `0x10F201` отправителю. Пропущенный декомпилятом аргумент
//! `0x00016100`: diagnostic действительно содержит исходный area ID. Ответ
//! kick пересылается сохранённому socket ID; result `0` несёт две строки,
//! ненулевой result — одну.
//!
//! Старые `AddLogText` и MFC `ListBox` не получают Windows/Rust GUI-аналога:
//! обработчик складывает структурированные события в FIFO, а управляемый
//! main-loop снимает их в том же порядке и передаёт процессной диагностике.
//! Выходная команда отказа идёт через owned `ServerCommandHandle`; handler не знает
//! внутреннее устройство `CServer`. Недостаток четырёх payload-байт у
//! `LSGetInfo` сохраняет старый `CBaseMessage::GetLong == 0` без движения
//! курсора. Реализованные девять доменных функций удалены из сырого блока
//! вместе с их STL/MFC/SEH-шумом; несвязанные compiler/library helpers остаются
//! provenance-корпусом. Единственная локальная граница таблицы — `0x10F101` в
//! `nets/netauth/message.rs`; временного handler здесь нет.

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

/// Структурированное операторское событие вместо старых debug/MFC side effects.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LoginServerNotice {
    /// LoginServer прислал synthetic connect до применения IP-фильтра.
    Connected {
        /// Peer IPv4 соединения.
        address: Ipv4Addr,
        /// Исходный socket ID.
        socket_id: i32,
        /// Результат старого `CheckConnection`.
        allowed: bool,
    },
    /// LoginServer прислал свой area ID и зарегистрирован в таблице `CGame`.
    Registered {
        /// Peer IPv4 соединения.
        address: Ipv4Addr,
        /// Переданный LoginServer area ID.
        area_id: i32,
        /// Socket ID, записанный как значение map.
        socket_id: i32,
    },
    /// LoginServer прислал synthetic disconnect до удаления map-записи.
    Disconnected {
        /// Peer IPv4 соединения.
        address: Ipv4Addr,
        /// Первый найденный area ID либо старый sentinel `0`.
        area_id: i32,
        /// Исходный socket ID.
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

/// Конкретные Auth-handler’ы и принадлежавшее `CGame` состояние LoginServer.
pub(crate) struct AuthMessageHandlers {
    login_servers: BTreeMap<i32, i32>,
    ip_filter_enabled: bool,
    ip_allower: IpFilter<true>,
    game: AuthDbContext,
    notices: VecDeque<LoginServerNotice>,
}

impl AuthMessageHandlers {
    /// Создаёт обработчики с уже разобранными IPv4-шаблонами `mIPAllower`.
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

    /// Атомарно заменяет настройки `mIPAllower` после успешного config reload.
    pub(crate) fn replace_ip_filter(&mut self, enabled: bool, allowed_patterns: Vec<[u8; 4]>) {
        self.ip_filter_enabled = enabled;
        self.ip_allower.replace_patterns(allowed_patterns);
    }

    /// Возвращает socket ID для area ID либо исходный sentinel `0`.
    pub(crate) fn login_server_socket_id(&self, area_id: i32) -> i32 {
        self.login_servers.get(&area_id).copied().unwrap_or(0)
    }

    /// Возвращает первый area ID для socket ID либо исходный sentinel `0`.
    pub(crate) fn login_server_area_id(&self, socket_id: i32) -> i32 {
        self.area_for_socket(socket_id).unwrap_or(0)
    }

    /// Возвращает socket ID зарегистрированных LoginServer в порядке area ID.
    pub(crate) fn login_server_socket_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.login_servers.values().copied()
    }

    /// Забирает следующее операторское событие в исходном порядке handler’ов.
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
