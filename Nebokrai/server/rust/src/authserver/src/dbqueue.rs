//! DB-формы `dbqueue.h`, подтверждённые `authserver.exe` и `authserver.pdb` по
//! владельцам `message_func.cpp`, `cgame.cpp` и `authproc.cpp`. SQL-исполнение
//! находится в `dbaccess/authdb/authproc.rs`.
//!
//! Типизированные enum заменяют integer tag и `void *`, не меняя payload;
//! account и password остаются непротоколированными байтами. `ServerInfoQueue`
//! обновляет только `player_count` существующего ключа `(ls, ws, gs)` на его
//! прежней позиции, добавляет новый ключ в хвост и снимается целиком по FIFO.

use std::collections::VecDeque;
use std::mem;

use parking_lot::Mutex;

pub(crate) struct AuthQuestData {
    pub(crate) account: Vec<u8>,
    pub(crate) password: Vec<u8>,
    pub(crate) client_ip: u32,
    pub(crate) client_socket_id: i32,
}

impl AuthQuestData {
    pub(crate) fn new(
        account: Vec<u8>,
        password: Vec<u8>,
        client_ip: u32,
        client_socket_id: i32,
    ) -> Self {
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
pub(crate) struct LockUntil {
    pub(crate) year: u16,
    pub(crate) month: u16,
    pub(crate) day: u16,
    pub(crate) hour: u16,
    pub(crate) minute: u16,
    pub(crate) second: u16,
}

pub(crate) struct LockQuestData {
    pub(crate) account: Vec<u8>,
    pub(crate) until: LockUntil,
}

pub(crate) enum DbQuest {
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

pub(crate) struct AuthResultData {
    pub(crate) result: i32,
    pub(crate) account: Vec<u8>,
    pub(crate) client_ip: u32,
    pub(crate) client_socket_id: i32,
}

impl AuthResultData {
    pub(crate) fn new(
        result: i32,
        account: Vec<u8>,
        client_ip: u32,
        client_socket_id: i32,
    ) -> Self {
        Self {
            result,
            account,
            client_ip,
            client_socket_id,
        }
    }
}

pub(crate) struct AuthExResultData {
    pub(crate) result: i32,
    pub(crate) account: Vec<u8>,
    pub(crate) client_ip: u32,
    pub(crate) client_socket_id: i32,
    pub(crate) extra: [u8; 80],
}

impl AuthExResultData {
    pub(crate) fn new(
        result: i32,
        account: Vec<u8>,
        client_ip: u32,
        client_socket_id: i32,
    ) -> Self {
        Self {
            result,
            account,
            client_ip,
            client_socket_id,
            extra: [0; 80],
        }
    }
}

pub(crate) struct LockResultData {
    pub(crate) account: Vec<u8>,
    pub(crate) succeeded: bool,
}

pub(crate) enum DbResult {
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

pub(crate) struct ServerInfo {
    pub(crate) player_count: i32,
    pub(crate) game_server_id: i32,
    pub(crate) world_server_id: i32,
    pub(crate) login_server_id: i32,
}

impl ServerInfo {
    pub(crate) const fn new(
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

pub(crate) struct ServerInfoQueue {
    entries: Mutex<VecDeque<ServerInfo>>,
}

impl ServerInfoQueue {
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn push_back(&self, entry: ServerInfo) {
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

    pub(crate) fn pop_all(&self) -> VecDeque<ServerInfo> {
        mem::take(&mut *self.entries.lock())
    }
}

impl Default for ServerInfoQueue {
    fn default() -> Self {
        Self::new()
    }
}
