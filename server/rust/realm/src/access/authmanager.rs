//! Менеджер `authmanager.cpp/.h`. Он хранит pending Auth-запросы и синхронные
//! listener-callbacks; `CLOCK_BOOTTIME` сохраняет wrapping ticks `timeGetTime`.
//!
//! Точный duplicate account отбрасывается. Новая запись сначала попадает в
//! хвост, затем отправляется `0xCF501`, после чего вызывается первый listener-
//! slot; ошибка send не откатывает вставку. Timeout использует строгое
//! `timeout < now - start` и публикует synthetic `0xCF601`, не удаляя запись и
//! не обновляя tick, поэтому повторяется до обработки первого ответа. Этот ответ
//! удаляет account до второго listener callback; последующие копии невалидны.

use std::collections::VecDeque;

use rustix::time::{ClockId, clock_gettime};

use nebokrai_shared::network::ClientSendQueue;

use crate::app::login_auth_client::AuthClientEventPublisher;
use crate::app::login_message::{CMessage, SendMessageError};

const AUTH_QUEST_MESSAGE_TYPE: i32 = 0x000C_F501;
const AUTH_RESPONSE_MESSAGE_TYPE: i32 = 0x000C_F601;
const AUTH_TIMEOUT_RESULT: i32 = 4;
const DEFAULT_AUTH_TIMEOUT_MS: u32 = 1000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthQuest {
    client_ip: u32,
    client_socket_id: i32,
    account: Vec<u8>,
    password: Vec<u8>,
    start_time_ms: u32,
}

impl AuthQuest {
    pub fn new(
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

    pub fn account(&self) -> &[u8] {
        &self.account
    }

    pub const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub const fn client_socket_id(&self) -> i32 {
        self.client_socket_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthResult {
    pub result: i32,
    pub account: Vec<u8>,
    pub client_ip: u32,
    pub client_socket_id: i32,
}

pub trait AuthListener {
    fn on_quest(&mut self, quest: &AuthQuest);

    fn on_response(&mut self, result: &AuthResult);
}

#[derive(Debug, Eq, PartialEq)]
pub enum AddQuestOutcome {
    Duplicate,
    Added {
        send_result: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthResponseOutcome {
    Response(AuthResult),
    InvalidResponse { account: Vec<u8> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthRunOutcome {
    pub published_timeouts: usize,
}

pub struct AuthManager {
    pending: VecDeque<AuthQuest>,
    timeout_ms: u32,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            timeout_ms: DEFAULT_AUTH_TIMEOUT_MS,
        }
    }

    pub fn init(&mut self, timeout_ms: u32) -> bool {
        self.timeout_ms = timeout_ms;
        true
    }

    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    pub fn add_quest(
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

    pub fn run(&self, publisher: &AuthClientEventPublisher) -> AuthRunOutcome {
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

    pub fn on_response_auth(
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

    pub fn pending_count(&self) -> usize {
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
