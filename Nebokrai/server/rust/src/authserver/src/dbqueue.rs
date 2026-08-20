//! Типизированные элементы DB-очередей AuthServer из `dbqueue.h`.
//!
//! Статус владельца: `IMPLEMENTED` для форм quest/result и coalescing-очереди
//! `ServerInfo`. SQL-исполнение принадлежит `dbaccess/authdb/authproc.rs`.
//!
//! Точная пара: `AuthServer/authserver.exe + AuthServer/authserver.pdb`;
//! исходный владелец PDB:
//! `h:\fengyun\fy_russia\src\server\authserver\src\dbqueue.h`.
//! Экспортированные тела: `AuthExResultData` RVA `0x00004DA0`,
//! `AuthQuestData` `0x000164D0`, специализированный
//! `my_fucking_list<ServerInfo>::push_back` `0x00016B60`. Поля остальных
//! вариантов подтверждены их созданием и
//! потреблением в `message_func.cpp`, `cgame.cpp` и `authproc.cpp`.
//!
//! Старый `db_element_type { socket_id, type, void *data }` заменён двумя
//! enum: компилятор теперь связывает tag с правильным owned payload. Это не
//! меняет доменную семантику и устраняет ручные `new/delete`, nullable `void*`
//! и ветви удаления по integer tag. Account/password остаются bytes: их
//! кодировка не угадывается и credential bytes нельзя включать в логи.
//!
//! `ServerInfoQueue` сохраняет отдельный доказанный контракт старого custom-
//! списка: ключ `(ls_id, ws_id, gs_id)` обновляет только `player_count` на
//! прежней позиции, новый ключ добавляется в хвост, `pop_all` сохраняет порядок.
//! Неиспользуемые Windows condition/semaphore не получают пустого Rust-аналога.

use std::collections::VecDeque;
use std::mem;

use parking_lot::Mutex;

/// Данные обычного и расширенного запроса авторизации.
pub(crate) struct AuthQuestData {
    /// Account bytes после исходного `CharLowerA` в message-handler’е.
    pub(crate) account: Vec<u8>,
    /// Credential bytes после исходного `CharLowerA` в message-handler’е.
    pub(crate) password: Vec<u8>,
    /// IPv4 клиента в исходном 32-битном представлении.
    pub(crate) client_ip: u32,
    /// Идентификатор клиентского socket внутри LoginServer.
    pub(crate) client_socket_id: i32,
}

impl AuthQuestData {
    /// Сохраняет четыре поля в форме, переданной LoginServer.
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

/// Шесть переданных календарных полей блокировки аккаунта.
///
/// `wDayOfWeek` и `wMilliseconds` исходной `SYSTEMTIME` обнулялись и не
/// приходили по wire, поэтому здесь их нет.
pub(crate) struct LockUntil {
    /// Полный год.
    pub(crate) year: u16,
    /// Месяц.
    pub(crate) month: u16,
    /// День месяца.
    pub(crate) day: u16,
    /// Час.
    pub(crate) hour: u16,
    /// Минута.
    pub(crate) minute: u16,
    /// Секунда.
    pub(crate) second: u16,
}

/// Данные запроса блокировки аккаунта.
pub(crate) struct LockQuestData {
    /// Account bytes в форме message-handler’а.
    pub(crate) account: Vec<u8>,
    /// Переданный срок блокировки.
    pub(crate) until: LockUntil,
}

/// Одна типизированная команда исходной `mDBQuestQueue`.
pub(crate) enum DbQuest {
    /// Проверка аккаунта процедурой `sp_auth`.
    Authenticate {
        /// Socket LoginServer, которому принадлежит ответ.
        return_socket_id: i32,
        /// Параметры аккаунта и клиентского соединения.
        request: AuthQuestData,
    },
    /// Расширенная проверка аккаунта процедурой `sp_authex`.
    AuthenticateExtended {
        /// Socket LoginServer, которому принадлежит ответ.
        return_socket_id: i32,
        /// Параметры аккаунта и клиентского соединения.
        request: AuthQuestData,
    },
    /// Блокировка аккаунта процедурой `sp_lock`.
    Lock {
        /// Socket LoginServer, которому принадлежит ответ.
        return_socket_id: i32,
        /// Аккаунт и переданный срок блокировки.
        request: LockQuestData,
    },
    /// Запись текущего coalesced-снимка серверов процедурой `sp_writelog`.
    WriteServerInfo,
}

/// Результат обычной авторизации до wire-сериализации.
pub(crate) struct AuthResultData {
    /// Уже преобразованный legacy protocol result.
    pub(crate) result: i32,
    /// Account bytes запроса.
    pub(crate) account: Vec<u8>,
    /// IPv4 клиента в исходном 32-битном представлении.
    pub(crate) client_ip: u32,
    /// Идентификатор клиентского socket внутри LoginServer.
    pub(crate) client_socket_id: i32,
}

impl AuthResultData {
    /// Создаёт результат с сохранением исходных signedness и client identity.
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

/// Результат расширенной авторизации до wire-сериализации.
pub(crate) struct AuthExResultData {
    /// Уже преобразованный legacy protocol result.
    pub(crate) result: i32,
    /// Account bytes запроса.
    pub(crate) account: Vec<u8>,
    /// IPv4 клиента в исходном 32-битном представлении.
    pub(crate) client_ip: u32,
    /// Идентификатор клиентского socket внутри LoginServer.
    pub(crate) client_socket_id: i32,
    /// Условный payload результата `3` либо `7`, нулевой в constructor.
    pub(crate) extra: [u8; 80],
}

impl AuthExResultData {
    /// Создаёт результат и дословно обнуляет исходный 80-байтовый payload.
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

/// Результат блокировки аккаунта до wire-сериализации.
pub(crate) struct LockResultData {
    /// Account bytes запроса.
    pub(crate) account: Vec<u8>,
    /// Результат исходной DB-процедуры.
    pub(crate) succeeded: bool,
}

/// Один типизированный элемент исходной `mDBResultQueue`.
pub(crate) enum DbResult {
    /// Ответ обычной авторизации.
    Authenticate {
        /// Socket LoginServer, которому отправляется результат.
        return_socket_id: i32,
        /// Данные ответа.
        result: AuthResultData,
    },
    /// Ответ расширенной авторизации.
    AuthenticateExtended {
        /// Socket LoginServer, которому отправляется результат.
        return_socket_id: i32,
        /// Данные ответа и условный extra payload.
        result: AuthExResultData,
    },
    /// Ответ блокировки аккаунта.
    Lock {
        /// Socket LoginServer, которому отправляется результат.
        return_socket_id: i32,
        /// Данные ответа.
        result: LockResultData,
    },
}

/// Последний player count одного Game/World/Login server tuple.
pub(crate) struct ServerInfo {
    /// Последнее число игроков.
    pub(crate) player_count: i32,
    /// Исходный GameServer ID.
    pub(crate) game_server_id: i32,
    /// Исходный WorldServer ID.
    pub(crate) world_server_id: i32,
    /// Исходный LoginServer ID.
    pub(crate) login_server_id: i32,
}

impl ServerInfo {
    /// Создаёт запись в том порядке полей, который читает Auth message-handler.
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

/// Coalescing FIFO server-info исходной `mDBLogQueue`.
pub(crate) struct ServerInfoQueue {
    entries: Mutex<VecDeque<ServerInfo>>,
}

impl ServerInfoQueue {
    /// Создаёт пустую очередь server-info.
    pub(crate) fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
        }
    }

    /// Обновляет count существующего ключа на месте либо добавляет новый ключ.
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

    /// Передаёт весь текущий coalesced-снимок с сохранением порядка ключей.
    pub(crate) fn pop_all(&self) -> VecDeque<ServerInfo> {
        mem::take(&mut *self.entries.lock())
    }
}

impl Default for ServerInfoQueue {
    fn default() -> Self {
        Self::new()
    }
}
