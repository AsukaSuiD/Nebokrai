//! Менеджер запросов авторизации LoginServer из `authmanager.cpp` и `.h`.
//!
//! Статус всех семантических функций владельца — `IMPLEMENTED`: `init` RVA
//! `0x0001FBD0`, `send_quest_message` `0x0001FBF0`, `run` `0x0001FD20`,
//! конструктор `AuthQuest` `0x0001FF10`, `removeQuest` `0x000201A0`,
//! `OnResponseAuth` `0x00020200`, конструктор/деструктор manager
//! `0x00020350/0x00020390` и две формы `addQuest`
//! `0x000203E0/0x00020460`.
//!
//! Точная пара: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`;
//! SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходные пути PDB:
//! `d:\complite_version\fengyun_russia\trunk\server\loginserver\loginserver\authmanager.cpp`
//! и `.h`.
//!
//! Заявка хранит client IPv4/socket ID, byte-exact account/password и wrapping
//! `timeGetTime` начала. `addQuest` отбрасывает точный duplicate account; новую
//! запись он сначала добавляет в хвост списка, затем отправляет `0xCF501` и
//! только после этого вызывает первый virtual listener-slot. Rust сохраняет
//! порядок через синхронный callback первого listener-slot; результат send не
//! превращается в rollback и остаётся частью `AddQuestOutcome`.
//!
//! `run` сравнивает timeout строго как `timeout < now.wrapping_sub(start)` и
//! для каждой просроченной записи ставит synthetic `0xCF601` в ту же Auth FIFO.
//! Исходная странность сохранена: запись не удаляется, её start-time не
//! обновляется, поэтому до обработки первого synthetic response каждый новый
//! проход публикует ещё один timeout. Первый дошедший `0xCF601` удаляет account
//! и синхронно вызывает второй listener-slot; следующие копии становятся
//! `InvalidResponse`.
//!
//! Linux `CLOCK_BOOTTIME` через уже выбранный `rustix` заменяет suspend-aware
//! миллисекундный `timeGetTime`, а приведение к `u32` сохраняет wrapping.
//! `VecDeque` заменяет `std::list`, owned bytes — `std::string`, trait-вызов —
//! vtable listener. Вместо хранения сырого nullable listener-pointer Rust
//! принимает проверенный mutable borrow только на время синхронного вызова;
//! момент и порядок callbacks не меняются. Старые `AddLogText` представлены
//! `InvalidResponse` и числом timeout-событий для будущего logging-owner.
//! Глобальный `gAuthMgr`, `atexit`, SEH, allocators и ручные деструкторы не
//! получают отдельных Rust-аналогов.

use std::collections::VecDeque;

use rustix::time::{ClockId, clock_gettime};

use crate::nets::clients::ClientSendQueue;
use crate::nets::netlogin::message::{CMessage, SendMessageError};
use crate::nets::netlogin::mynetclient_auth::AuthClientEventPublisher;

const AUTH_QUEST_MESSAGE_TYPE: i32 = 0x000C_F501;
const AUTH_RESPONSE_MESSAGE_TYPE: i32 = 0x000C_F601;
const AUTH_TIMEOUT_RESULT: i32 = 4;
const DEFAULT_AUTH_TIMEOUT_MS: u32 = 1000;

/// Ожидающий ответ AuthServer запрос.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthQuest {
    client_ip: u32,
    client_socket_id: i32,
    account: Vec<u8>,
    password: Vec<u8>,
    start_time_ms: u32,
}

impl AuthQuest {
    /// Создаёт заявку с текущим wrapping boot tick.
    pub(crate) fn new(
        client_ip: u32,
        client_socket_id: i32,
        account: Vec<u8>,
        password: Vec<u8>,
    ) -> Self {
        Self {
            client_ip,
            client_socket_id,
            account,
            password,
            start_time_ms: legacy_tick_ms(),
        }
    }

    /// Возвращает byte-exact account для listener и duplicate-проверки.
    pub(crate) fn account(&self) -> &[u8] {
        &self.account
    }

    /// Возвращает исходный client IPv4.
    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    /// Возвращает исходный signed client socket ID.
    pub(crate) const fn client_socket_id(&self) -> i32 {
        self.client_socket_id
    }
}

/// Доказанная структура `AuthManager::AuthResult` для listener-вызова.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthResult {
    /// Код результата AuthServer.
    pub(crate) result: i32,
    /// Буквальный account без завершающего NUL.
    pub(crate) account: Vec<u8>,
    /// Исходный 32-битный client IPv4.
    pub(crate) client_ip: u32,
    /// Исходный signed client socket ID.
    pub(crate) client_socket_id: i32,
}

/// Два virtual slot исходного `AuthListener`.
pub(crate) trait AuthListener {
    /// Вызывается после вставки и попытки отправки новой уникальной заявки.
    fn on_quest(&mut self, quest: &AuthQuest);

    /// Вызывается после удаления совпавшей заявки из pending-списка.
    fn on_response(&mut self, result: &AuthResult);
}

/// Наблюдаемый результат `addQuest`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AddQuestOutcome {
    /// Account уже ожидает ответа; send и listener не вызываются.
    Duplicate,
    /// Заявка добавлена, а результат исходного `SendToAS` сохранён отдельно.
    Added {
        /// Результат построения и постановки `0xCF501` в Auth send-очередь.
        send_result: Result<i32, SendMessageError>,
    },
}

/// Результат синхронного `OnResponseAuth`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuthResponseOutcome {
    /// Первая совпавшая заявка удалена и listener вызван.
    Response(AuthResult),
    /// Ответ не соответствовал ни одному ожидающему account.
    InvalidResponse { account: Vec<u8> },
}

/// Результат одного `AuthManager::run`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthRunOutcome {
    /// Число synthetic timeout-сообщений, поставленных в Auth FIFO.
    pub(crate) published_timeouts: usize,
}

/// Владелец ожидающих Auth-запросов и их timeout cadence.
pub(crate) struct AuthManager {
    pending: VecDeque<AuthQuest>,
    timeout_ms: u32,
}

impl AuthManager {
    /// Создаёт пустой список с исходным timeout `1000 ms`.
    pub(crate) fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            timeout_ms: DEFAULT_AUTH_TIMEOUT_MS,
        }
    }

    /// Устанавливает исходный `setup.authTimeOut` и сохраняет успешный результат.
    pub(crate) fn init(&mut self, timeout_ms: u32) -> bool {
        self.timeout_ms = timeout_ms;
        true
    }

    /// Обновляет timeout при доказанном `ReLoadSetup` без очистки заявок.
    pub(crate) fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    /// Добавляет уникальную заявку, отправляет её и затем вызывает первый slot.
    pub(crate) fn add_quest(
        &mut self,
        quest: AuthQuest,
        sender: &ClientSendQueue,
        on_quest: impl FnOnce(&AuthQuest),
    ) -> AddQuestOutcome {
        if self
            .pending
            .iter()
            .any(|pending| pending.account == quest.account)
        {
            return AddQuestOutcome::Duplicate;
        }

        self.pending.push_back(quest);
        let stored = self
            .pending
            .back()
            .expect("заявка только что добавлена в pending-список");
        let send_result = send_quest_message(stored, sender);
        on_quest(stored);
        AddQuestOutcome::Added { send_result }
    }

    /// Публикует timeout для каждой просроченной заявки без её изменения.
    pub(crate) fn run(&self, publisher: &AuthClientEventPublisher) -> AuthRunOutcome {
        let now_ms = legacy_tick_ms();
        let mut published_timeouts = 0;

        for quest in &self.pending {
            if self.timeout_ms < now_ms.wrapping_sub(quest.start_time_ms) {
                let mut message = CMessage::new(AUTH_RESPONSE_MESSAGE_TYPE);
                message.base_mut().add_long(AUTH_TIMEOUT_RESULT);
                add_legacy_string(&mut message, &quest.account);
                message.base_mut().add_ulong(quest.client_ip);
                message.base_mut().add_long(quest.client_socket_id);
                publisher.publish_message(message);
                published_timeouts += 1;
            }
        }

        AuthRunOutcome { published_timeouts }
    }

    /// Разбирает `0xCF601`, удаляет первую заявку и вызывает response-listener.
    pub(crate) fn on_response_auth(
        &mut self,
        message: &mut CMessage,
        listener: &mut dyn AuthListener,
    ) -> AuthResponseOutcome {
        let result = message.base_mut().get_long().unwrap_or(0);
        let account = message.get_string();
        let client_ip = message.base_mut().get_long().unwrap_or(0) as u32;
        let client_socket_id = message.base_mut().get_long().unwrap_or(0);

        let Some(index) = self
            .pending
            .iter()
            .position(|quest| quest.account == account)
        else {
            return AuthResponseOutcome::InvalidResponse { account };
        };
        self.pending.remove(index);
        let response = AuthResult {
            result,
            account,
            client_ip,
            client_socket_id,
        };
        listener.on_response(&response);
        AuthResponseOutcome::Response(response)
    }

    /// Возвращает число ещё ожидающих заявок.
    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

fn send_quest_message(
    quest: &AuthQuest,
    sender: &ClientSendQueue,
) -> Result<i32, SendMessageError> {
    let mut message = CMessage::new(AUTH_QUEST_MESSAGE_TYPE);
    add_legacy_string(&mut message, &quest.account);
    add_legacy_string(&mut message, &quest.password);
    message.base_mut().add_ulong(quest.client_ip);
    message.base_mut().add_long(quest.client_socket_id);
    message.send_to_auth(sender)
}

fn add_legacy_string(message: &mut CMessage, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    message.base_mut().add(&value[..end]);
    message.base_mut().add_byte(0);
}

fn legacy_tick_ms() -> u32 {
    let now = clock_gettime(ClockId::Boottime);
    let seconds_ms = (now.tv_sec as u64).wrapping_mul(1000);
    let nanoseconds_ms = (now.tv_nsec as u64) / 1_000_000;
    seconds_ms.wrapping_add(nanoseconds_ms) as u32
}
