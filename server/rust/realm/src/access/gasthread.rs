//! `CGasThread` LoginServer.
//!
//! Worker сохраняет одну FIFO GAS, 10-миллисекундную idle cadence, один
//! `CMyWinInet` с исторически повторно используемым receive-буфером и точную
//! таблицу ответных кодов. HTTP остаётся в выделенном blocking-thread. Вместо
//! небезопасного доступа исходного worker к глобальному `CGame` он публикует
//! owned-результаты; главный Login turn применяет DB/network-эффекты в том
//! же FIFO-порядке и подтверждает завершение до следующего HTTP-запроса. Эта
//! узкая сериализация сохраняет наблюдаемый межсистемный порядок одного GAS
//! worker и не вводит общий `Arc<Mutex<CGame>>`.
//!
//! Принудительный Win32 `TerminateThread`, COM init, singleton `CGasOperator`,
//! STL allocator/copy и EH cleanup заменены владением Rust, atomic stop,
//! `JoinHandle` и каналом. Короткий digest безопасно отклоняется вместо чтения
//! за массивом; fixed buffers заменены owned `Vec` без изменения HTTP-формата.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::{Datelike, Local, Timelike};

use super::gasoperator::format_ipv4;
use super::mywininet::{CMyWinInet, MyWinInetError};
use super::game::{CGame, GameRouteError, PrepareEnterOutcome};
use super::loginqueue::{CLoginQueue, QuestCdkey, TagPwdChecked};
use crate::app::login_message::CMessage;
use nebokrai_shared::protocol::message_digest;

const LOGIN_RESPONSE_MESSAGE_TYPE: i32 = 0x000A_F501;
const IDLE_INTERVAL: Duration = Duration::from_millis(10);
const NICKNAME_MARKER: &[u8] = b"\"nickname\"";

#[derive(Clone, Debug)]
pub struct GasVerificationConfig {
    pub address: Vec<u8>,
    pub signature_uppercase: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GasBlockedReason {
    PasswordDigestTooShort { actual_len: usize },
    SignatureUppercaseMissing,
    VerificationAddressEncodingUnsupported,
    VerificationAddressSchemeUnsupported,
}

#[derive(Debug)]
enum GasCheckResult {
    State {
        state: i32,
        transport_error: Option<MyWinInetError>,
    },
    Blocked(GasBlockedReason),
}

#[derive(Debug)]
enum GasWorkerWork {
    Direct(QuestCdkey),
    Checked {
        quest: QuestCdkey,
        result: GasCheckResult,
    },
}

#[derive(Debug)]
pub struct GasWorkerEvent {
    work: GasWorkerWork,
    completion: mpsc::Sender<()>,
}

#[derive(Debug)]
pub enum GasProcessOutcome {
    DirectFinished,
    DirectEntered(Result<(), GameRouteError>),
    DirectFailed(GameRouteError),
    AuthenticationRejected {
        state: i32,
        transport_error: Option<MyWinInetError>,
        send_result: Option<Result<i32, GameRouteError>>,
    },
    ActiveBan {
        send_result: Result<i32, GameRouteError>,
    },
    IpRejected {
        between_check: bool,
        send_result: Result<i32, GameRouteError>,
    },
    Accepted {
        send_result: Result<i32, GameRouteError>,
        procedure_legacy_result: bool,
    },
    DatabaseOwnerMissing,
    IpSetupMissing,
    Blocked(GasBlockedReason),
}

pub struct CGasThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    receiver: mpsc::Receiver<GasWorkerEvent>,
}

impl CGasThread {
    pub fn start(
        queue: Arc<CLoginQueue>,
        config: GasVerificationConfig,
    ) -> Result<Self, io::Error> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let (sender, receiver) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("login-gas".to_owned())
            .spawn(move || run_worker(queue, config, worker_stop, sender))?;
        Ok(Self {
            stop,
            handle: Some(handle),
            receiver,
        })
    }

    pub fn drain_events(&mut self) -> Vec<GasWorkerEvent> {
        self.receiver.try_iter().collect()
    }

    fn stop_and_join(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CGasThread {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

pub fn apply_worker_event(game: &mut CGame, event: GasWorkerEvent) -> GasProcessOutcome {
    let outcome = match event.work {
        GasWorkerWork::Direct(quest) => apply_direct(game, &quest),
        GasWorkerWork::Checked { quest, result } => match result {
            GasCheckResult::Blocked(reason) => GasProcessOutcome::Blocked(reason),
            GasCheckResult::State {
                state,
                transport_error,
            } => {
                if state == 0 {
                    apply_accepted(game, &quest)
                } else {
                    GasProcessOutcome::AuthenticationRejected {
                        state,
                        transport_error,
                        send_result: send_state_response(game, quest.socket_id(), state),
                    }
                }
            }
        },
    };
    let _ = event.completion.send(());
    outcome
}

fn run_worker(
    queue: Arc<CLoginQueue>,
    config: GasVerificationConfig,
    stop: Arc<AtomicBool>,
    sender: mpsc::Sender<GasWorkerEvent>,
) {
    let mut wininet = CMyWinInet::default();
    while !stop.load(Ordering::Acquire) {
        while let Some(mut quest) = queue.pop_gas_quest() {
            let work = if quest.world_server().is_empty() {
                let result = check_account(&mut wininet, &config, &mut quest);
                GasWorkerWork::Checked { quest, result }
            } else {
                GasWorkerWork::Direct(quest)
            };
            let (completion, completed) = mpsc::channel();
            if sender.send(GasWorkerEvent { work, completion }).is_err() {
                return;
            }
            loop {
                match completed.recv_timeout(IDLE_INTERVAL) {
                    Ok(()) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if stop.load(Ordering::Acquire) {
                            return;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
        }
        thread::sleep(IDLE_INTERVAL);
    }
}

fn check_account(
    wininet: &mut CMyWinInet,
    config: &GasVerificationConfig,
    quest: &mut QuestCdkey,
) -> GasCheckResult {
    let address = legacy_c_string_prefix(&config.address);
    if let Err(error) = wininet.init(address) {
        return match error {
            MyWinInetError::InvalidUrlEncoding => {
                GasCheckResult::Blocked(GasBlockedReason::VerificationAddressEncodingUnsupported)
            }
            MyWinInetError::UnsupportedScheme => {
                GasCheckResult::Blocked(GasBlockedReason::VerificationAddressSchemeUnsupported)
            }
            error => transport_failure(error),
        };
    }
    let content = match form_content(quest, config.signature_uppercase) {
        Ok(content) => content,
        Err(reason) => {
            wininet.close();
            return GasCheckResult::Blocked(reason);
        }
    };
    if let Err(error) = wininet.send(&content) {
        wininet.close();
        return transport_failure(error);
    }
    let response = match wininet.recv() {
        Ok(response) => response,
        Err(error) => {
            wininet.close();
            return transport_failure(error);
        }
    };
    wininet.close();

    match analyse_response(response.as_deref()) {
        Ok((state, Some(nickname))) => {
            quest.replace_account_with_nickname(nickname);
            GasCheckResult::State {
                state,
                transport_error: None,
            }
        }
        Ok((state, None)) => GasCheckResult::State {
            state,
            transport_error: None,
        },
        Err(reason) => GasCheckResult::Blocked(reason),
    }
}

fn transport_failure(error: MyWinInetError) -> GasCheckResult {
    GasCheckResult::State {
        state: 100,
        transport_error: Some(error),
    }
}

fn form_content(
    quest: &QuestCdkey,
    signature_uppercase: Option<i32>,
) -> Result<Vec<u8>, GasBlockedReason> {
    let password = lower_hex_16(quest.password_digest())?;
    let account = legacy_c_string_prefix(quest.account());
    let signature_len = account.len() + 1 + password.len() + b"|DaYeZaiCi".len();

    let mut signature_input = Vec::with_capacity(signature_len);
    signature_input.extend_from_slice(account);
    signature_input.push(b'|');
    signature_input.extend_from_slice(&password);
    signature_input.extend_from_slice(b"|DaYeZaiCi");
    let mut signature = hex_digest(message_digest(&signature_input, 1));
    match signature_uppercase {
        Some(1) => signature.make_ascii_uppercase(),
        Some(_) => {}
        None => return Err(GasBlockedReason::SignatureUppercaseMissing),
    }

    let content_len = b"username=".len()
        + account.len()
        + b"&password=".len()
        + password.len()
        + b"&hash=".len()
        + signature.len();
    let mut content = Vec::with_capacity(content_len);
    content.extend_from_slice(b"username=");
    content.extend_from_slice(account);
    content.extend_from_slice(b"&password=");
    content.extend_from_slice(&password);
    content.extend_from_slice(b"&hash=");
    content.extend_from_slice(&signature);
    Ok(content)
}

fn analyse_response(response: Option<&[u8]>) -> Result<(i32, Option<Vec<u8>>), GasBlockedReason> {
    let Some(response) = response.filter(|response| !response.is_empty()) else {
        return Ok((100, None));
    };
    // `AnalysisRet` читал эти позиции внутри фиксированного 1024-байтового
    // receive-буфера. После C-string конца там оставались нули, поэтому
    // короткий ответ не был выходом за границу: он попадал в -11, а `'-'` в
    // позиции 10 с NUL в позиции 11 — в default -6.
    if response.get(10).copied().unwrap_or(0) == b'-' {
        let state = match response.get(11).copied().unwrap_or(0) {
            b'1' => -1,
            b'2' => -2,
            b'3' => -3,
            b'4' => -4,
            b'5' => -5,
            b'7' => -7,
            b'8' => -8,
            _ => -6,
        };
        return Ok((state, None));
    }

    let Some(nickname) = extract_nickname(response) else {
        return Ok((-11, None));
    };
    if nickname.is_empty() {
        return Ok((-10, None));
    }
    Ok((0, Some(nickname.to_vec())))
}

fn extract_nickname(response: &[u8]) -> Option<&[u8]> {
    if response.len() >= 0x1_0000 || response.len() <= NICKNAME_MARKER.len() {
        return None;
    }
    let marker = response
        .windows(NICKNAME_MARKER.len())
        .position(|window| window == NICKNAME_MARKER)?;
    let opening_quote = marker.checked_add(NICKNAME_MARKER.len() + 1)?;
    let after_opening = opening_quote.checked_add(1)?;
    let tail = response.get(after_opening..)?;
    let length = tail.iter().position(|byte| *byte == b'\"')?;
    Some(&tail[..length])
}

fn apply_direct(game: &mut CGame, quest: &QuestCdkey) -> GasProcessOutcome {
    let checked = TagPwdChecked::new(
        quest.socket_id(),
        quest.client_ip(),
        quest.account().to_vec(),
        quest.world_server().to_vec(),
        false,
    );
    match game.prepare_enter(&checked) {
        Ok(PrepareEnterOutcome::Continue) => GasProcessOutcome::DirectEntered(game.enter_game(
            &checked,
            game.login_queue().is_no_queue_account(checked.account()),
        )),
        Ok(PrepareEnterOutcome::Finished | PrepareEnterOutcome::MatrixRegistrationRequired) => {
            GasProcessOutcome::DirectFinished
        }
        Err(error) => GasProcessOutcome::DirectFailed(error),
    }
}

fn apply_accepted(game: &mut CGame, quest: &QuestCdkey) -> GasProcessOutcome {
    let ban_time = match game.rs_cdkey_owner_mut() {
        Some(owner) => owner.get_ban_time(quest.account()),
        None => return GasProcessOutcome::DatabaseOwnerMissing,
    };
    if ban_time.is_some_and(|ban_time| Local::now().naive_local() < ban_time) {
        let ban_time = ban_time.expect("действующий GAS ban_time только что проверен");
        let mut response = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
        response.base_mut().add_char(0x10);
        response.base_mut().add_char(0);
        response.base_mut().add_short(ban_time.year() as i16);
        response.base_mut().add_short(ban_time.month() as i16);
        response.base_mut().add_short(ban_time.day() as i16);
        response.base_mut().add_short(ban_time.hour() as i16);
        response.base_mut().add_short(ban_time.minute() as i16);
        return GasProcessOutcome::ActiveBan {
            send_result: game.send_to_client(&response, quest.socket_id()),
        };
    }

    let Some((check_allowed, check_forbidden, check_between)) = game.login_setup().ip_checks()
    else {
        return GasProcessOutcome::IpSetupMissing;
    };
    let allowed = match game.rs_cdkey_owner_mut() {
        Some(owner) => owner.ip_is_allowed(check_allowed, quest.client_ip()),
        None => return GasProcessOutcome::DatabaseOwnerMissing,
    };
    if !allowed {
        return GasProcessOutcome::IpRejected {
            between_check: false,
            send_result: send_login_code(game, quest.socket_id(), 0x12),
        };
    }
    let forbidden = match game.rs_cdkey_owner_mut() {
        Some(owner) => owner.ip_is_forbidden(check_forbidden, quest.client_ip()),
        None => return GasProcessOutcome::DatabaseOwnerMissing,
    };
    if forbidden {
        return GasProcessOutcome::IpRejected {
            between_check: false,
            send_result: send_login_code(game, quest.socket_id(), 0x12),
        };
    }
    let between = match game.rs_cdkey_owner_mut() {
        Some(owner) => owner.is_between_ip(check_between, quest.account(), quest.client_ip()),
        None => return GasProcessOutcome::DatabaseOwnerMissing,
    };
    if !between {
        return GasProcessOutcome::IpRejected {
            between_check: true,
            send_result: send_login_code(game, quest.socket_id(), 0x11),
        };
    }

    let mut response = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
    response.base_mut().add_char(2);
    add_legacy_string(&mut response, quest.account());
    game.add_world_info_to_message_for_account(&mut response, quest.account());
    let send_result = game.send_to_client(&response, quest.socket_id());

    game.push_back_pwd_checked(TagPwdChecked::new(
        quest.socket_id(),
        quest.client_ip(),
        quest.account().to_vec(),
        quest.world_server().to_vec(),
        false,
    ));
    let ip = format_ipv4(quest.client_ip());
    let password = lower_hex_16(quest.password_digest())
        .expect("успешный CheckAcc уже подтвердил 16 байт password digest");
    let procedure_legacy_result = game.execute_proce(quest.nickname(), &ip, &password, 0);
    GasProcessOutcome::Accepted {
        send_result,
        procedure_legacy_result,
    }
}

fn send_state_response(
    game: &CGame,
    socket_id: i32,
    state: i32,
) -> Option<Result<i32, GameRouteError>> {
    let payload: &[u8] = match state {
        100 | -7 | -4 => &[7],
        -11 | -10 => &[b'Y', 1],
        -8 => &[b'Y', 0],
        -6 | -3 => &[5],
        -5 => b"T",
        -2 | -1 => b"?",
        _ => return None,
    };
    let mut response = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
    for byte in payload {
        response.base_mut().add_char(*byte as i8);
    }
    Some(game.send_to_client(&response, socket_id))
}

fn send_login_code(game: &CGame, socket_id: i32, code: i8) -> Result<i32, GameRouteError> {
    let mut response = CMessage::new(LOGIN_RESPONSE_MESSAGE_TYPE);
    response.base_mut().add_char(code);
    game.send_to_client(&response, socket_id)
}

fn lower_hex_16(bytes: &[u8]) -> Result<Vec<u8>, GasBlockedReason> {
    let Some(bytes) = bytes.get(..16) else {
        return Err(GasBlockedReason::PasswordDigestTooShort {
            actual_len: bytes.len(),
        });
    };
    Ok(hex_digest(
        bytes.try_into().expect("срез имеет ровно 16 байт"),
    ))
}

fn hex_digest(bytes: [u8; 16]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = Vec::with_capacity(32);
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> 4)]);
        encoded.push(HEX[usize::from(byte & 0x0f)]);
    }
    encoded
}

fn add_legacy_string(message: &mut CMessage, value: &[u8]) {
    message.base_mut().add(legacy_c_string_prefix(value));
    message.base_mut().add_byte(0);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}
