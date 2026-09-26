//! DB-формы `dbqueue.h` AuthServer. SQL-исполнение находится в
//! [`crate::access::authproc`].
//!
//! Типизированные enum заменяют integer tag и `void *`, не меняя payload;
//! account и password остаются непротоколированными байтами. `ServerInfoQueue`
//! обновляет только `player_count` существующего ключа `(ls, ws, gs)` на его
//! прежней позиции, добавляет новый ключ в хвост и снимается целиком по FIFO.

use std::collections::VecDeque;
use std::mem;

use parking_lot::Mutex;

pub struct AuthQuestData {
    pub account: Vec<u8>,
    pub password: Vec<u8>,
    pub client_ip: u32,
    pub client_socket_id: i32,
}

impl AuthQuestData {
    pub fn new(account: Vec<u8>, password: Vec<u8>, client_ip: u32, client_socket_id: i32) -> Self {
        Self {
            account,
            password,
            client_ip,
            client_socket_id,
        }
    }
}

/// `wDayOfWeek` и `wMilliseconds` исходной `SYSTEMTIME` обнулялись и не
/// приходили по wire, поэтому здесь их нет.
pub struct LockUntil {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
}

pub struct LockQuestData {
    pub account: Vec<u8>,
    pub until: LockUntil,
}

pub enum DbQuest {
    Authenticate {
        return_socket_id: i32,
        request: AuthQuestData,
    },
    AuthenticateExtended {
        return_socket_id: i32,
        request: AuthQuestData,
    },
    Lock {
        return_socket_id: i32,
        request: LockQuestData,
    },
    WriteServerInfo,
}

pub struct AuthResultData {
    pub result: i32,
    pub account: Vec<u8>,
    pub client_ip: u32,
    pub client_socket_id: i32,
}

impl AuthResultData {
    pub fn new(result: i32, account: Vec<u8>, client_ip: u32, client_socket_id: i32) -> Self {
        Self {
            result,
            account,
            client_ip,
            client_socket_id,
        }
    }
}

pub struct AuthExResultData {
    pub result: i32,
    pub account: Vec<u8>,
    pub client_ip: u32,
    pub client_socket_id: i32,
    pub extra: [u8; 80],
}

impl AuthExResultData {
    pub fn new(result: i32, account: Vec<u8>, client_ip: u32, client_socket_id: i32) -> Self {
        Self {
            result,
            account,
            client_ip,
            client_socket_id,
            extra: [0; 80],
        }
    }
}

pub struct LockResultData {
    pub account: Vec<u8>,
    pub succeeded: bool,
}

pub enum DbResult {
    Authenticate {
        return_socket_id: i32,
        result: AuthResultData,
    },
    AuthenticateExtended {
        return_socket_id: i32,
        result: AuthExResultData,
    },
    Lock {
        return_socket_id: i32,
        result: LockResultData,
    },
}

pub struct ServerInfo {
    pub player_count: i32,
    pub game_server_id: i32,
    pub world_server_id: i32,
    pub login_server_id: i32,
}

impl ServerInfo {
    pub const fn new(
        player_count: i32,
        game_server_id: i32,
        world_server_id: i32,
        login_server_id: i32,
    ) -> Self {
        Self {
            player_count,
            game_server_id,
            world_server_id,
            login_server_id,
        }
    }

    fn has_same_key(&self, other: &Self) -> bool {
        self.login_server_id == other.login_server_id
            && self.world_server_id == other.world_server_id
            && self.game_server_id == other.game_server_id
    }
}

pub struct ServerInfoQueue {
    entries: Mutex<VecDeque<ServerInfo>>,
}

impl ServerInfoQueue {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push_back(&self, entry: ServerInfo) {
        let mut entries = self.entries.lock();
        if let Some(current) = entries
            .iter_mut()
            .find(|current| current.has_same_key(&entry))
        {
            current.player_count = entry.player_count;
            return;
        }
        entries.push_back(entry);
    }

    pub fn pop_all(&self) -> VecDeque<ServerInfo> {
        mem::take(&mut *self.entries.lock())
    }
}

impl Default for ServerInfoQueue {
    fn default() -> Self {
        Self::new()
    }
}
