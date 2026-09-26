//! Управляемый reconnect worker направления WorldServer -> LoginServer в
//! составе Realm `app/`. Источник контракта — та же точная пара, что у
//! [`crate::app::world_server`] (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`,
//! SHA-256 `F3AC454D…`, RSDS совпадает).
//!
//! Машинно подтверждённые точки (S_PUB32 `.exe/Nworldserver.exe`, первая
//! секция):
//! - `ConnectLoginServerFunc` `1:000023b0` (VA `0x4033B0`): начальная проверка
//!   exit-флага `[0x56E48C]`, затем цикл `Sleep(0x1F40)` (исходный cadence 8
//!   секунд) -> `ReConnectLoginServer` `1:00002280` (VA `0x403280`) через
//!   process-global `g_pGame` `[0x56E47C]`; выход при `== 1`, а exit-флаг
//!   проверяется только после неуспешной попытки — спящий worker не
//!   прерывается stop, как и полный `WaitForSingleObject` исходного
//!   `CreateConnectLoginThread` `1:00003320`;
//! - replacement client после успешного connect передаётся main-loop через
//!   typed handoff, поэтому смена network-owner-а происходит в исходной
//!   позиции `0x3FC03` и не обгоняет сообщения (обработку остаётся
//!   выполнять process-owner `CGame`).
//!
//! Worker повторяет попытку с исходной cadence, пока соединение не
//! опубликовано либо owned shutdown не отменит ожидание. Snapshot endpoint-а
//! и producer исходной World FIFO живёт здесь в `WorldLoginReconnectSpec`
//! (поля публичны: snapshot собирает владелец `CGame` за пределами
//! библиотеки). Фактическая попытка bind/connect бывшего owner-метода
//! `CGame` живёт здесь как [`WorldLoginReconnectSpec::reconnect_once`] без
//! mutable игры; разрешение endpoint-а (`resolve_login_endpoint`) разделяет
//! и initial client-owner.
//! Tokio runtime/blocking заменяет Win32 thread message и handle; stop всегда
//! дожидается завершения задачи. Data-итоги restart server-диспетчера
//! (`WorldLoginReconnectThreadStart`, `WorldLoginReconnectThreadRestart`)
//! живут здесь рядом с outcome/spec; setup thread-сборка и lifecycle
//! thread-owner-а (`connect_login_worker`) остаются у `CGame`.

use std::error::Error;
use std::fmt;
use std::io;
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tokio::runtime::Handle;

use nebokrai_shared::network::{ClientConnectError, bind_tcp_ipv4};

use super::world_client::CMyNetClient;
use super::world_server::WorldServerEventSender;

const LOGIN_RECONNECT_INTERVAL: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldLoginReconnect {
    pub endpoint: SocketAddrV4,
}

/// Итог awaitable-замены `ConnectLoginServerFunc`.
///
/// Причина неуспешных попыток исходной функцией не публиковалась: она знала
/// только `ReConnectLoginServer == 1`, поэтому report сохраняет лишь число
/// попыток и terminal state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldLoginReconnectWorkerOutcome {
    StoppedBeforeRetry,
    StoppedAfterFailedRetry { attempts: u32 },
    Reconnected {
        attempts: u32,
        reconnect: WorldLoginReconnect,
    },
}

#[derive(Debug)]
pub enum WorldLoginReconnectError {
    MissingSetupField(&'static str),
    LoginAddressEncodingUnsupported,
    LoginAddressResolution,
    Bind(io::Error),
    Connect(ClientConnectError),
    MissingNetworkServerOwner,
}

impl fmt::Display for WorldLoginReconnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSetupField(field) => {
                write!(formatter, "World setup не назначил поле {field}")
            }
            Self::LoginAddressEncodingUnsupported => formatter.write_str(
                "кодировка LoginServer-адреса не поддерживается безопасным Linux resolver",
            ),
            Self::LoginAddressResolution => {
                formatter.write_str("LoginServer-адрес не разрешён в IPv4")
            }
            Self::Bind(error) => write!(formatter, "не создан reconnect socket: {error}"),
            Self::Connect(error) => {
                write!(
                    formatter,
                    "WorldServer повторно не подключён к LoginServer: {error}"
                )
            }
            Self::MissingNetworkServerOwner => formatter.write_str(
                "подключённый LoginServer client некуда передать: World server-owner отсутствует",
            ),
        }
    }
}

impl Error for WorldLoginReconnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Bind(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::MissingSetupField(_) | Self::LoginAddressEncodingUnsupported
            | Self::LoginAddressResolution
            | Self::MissingNetworkServerOwner => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginEndpointError {
    EncodingUnsupported,
    Resolution,
}

impl From<LoginEndpointError> for WorldLoginReconnectError {
    fn from(error: LoginEndpointError) -> Self {
        match error {
            LoginEndpointError::EncodingUnsupported => Self::LoginAddressEncodingUnsupported,
            LoginEndpointError::Resolution => Self::LoginAddressResolution,
        }
    }
}

pub fn resolve_login_endpoint(raw_host: &[u8], port: u32) -> Result<SocketAddrV4, LoginEndpointError> {
    let host = legacy_c_string_prefix(raw_host);
    let host = std::str::from_utf8(host).map_err(|_| LoginEndpointError::EncodingUnsupported)?;
    (host, port as u16)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| {
            addresses.find_map(|address| match address {
                SocketAddr::V4(address) => Some(address),
                SocketAddr::V6(_) => None,
            })
        })
        .ok_or(LoginEndpointError::Resolution)
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}

/// Snapshot, достаточный для одной попытки reconnect вне mutable `CGame`.
///
/// Optional-поля намеренно переносятся без ранней валидации: исходный worker
/// создавался всегда, а ошибка setup/server-owner обнаруживалась уже после
/// bind/connect в каждой конкретной попытке.
#[derive(Clone)]
pub struct WorldLoginReconnectSpec {
    pub login_ip: Vec<u8>,
    pub login_port: Option<u32>,
    pub event_sender: Option<WorldServerEventSender>,
}

impl WorldLoginReconnectSpec {
    /// Подключает один новый LoginServer client и передаёт его World FIFO —
    /// точная попытка свободного `ConnectLoginServerFunc`.
    ///
    /// Тело совпадает с бывшим owner-методом `CGame`: неудачные
    /// bind/resolution/connect закрывают ещё не опубликованный client и не
    /// откатывают ничего извне; успех передаёт replacement owner в исходную
    /// World FIFO, где связанная обработка выполнит замену и control-send в
    /// исходной позиции `0x3FC03`.
    pub async fn reconnect_once(&self) -> Result<WorldLoginReconnect, WorldLoginReconnectError> {
 // держал новый CMyNetClient только в локальном pointer.
        let mut client = CMyNetClient::new();
        let socket = match bind_tcp_ipv4(None, 0) {
            Ok(socket) => socket,
            Err(error) => {
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::Bind(error));
            }
        };
        let login_port = match self.login_port {
            Some(port) => port,
            None => {
 // Старый dwLoginPort здесь был
 // неинициализирован; неизвестное значение не выбираем.
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::MissingSetupField("dwLoginPort"));
            }
        };
        let endpoint = match resolve_login_endpoint(&self.login_ip, login_port) {
            Ok(endpoint) => endpoint,
            Err(error) => {
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::from(error));
            }
        };

        if let Err(error) = client.connect(socket, endpoint).await {
            let _legacy_result = client.close();
            return Err(WorldLoginReconnectError::Connect(error));
        }

        let event_sender = match self.event_sender.as_ref() {
            Some(server) => server,
            None => {
 // Исходник после успешного connect
 // разыменовывал обязательный g_pGame->s_pNetServer. Safe Rust
 // закрывает ещё не опубликованный owner и не имитирует UB.
                let _legacy_result = client.close();
                return Err(WorldLoginReconnectError::MissingNetworkServerOwner);
            }
        };
        event_sender.publish_reconnected_login_client(client);

        Ok(WorldLoginReconnect { endpoint })
    }
}

#[derive(Default)]
struct WorldLoginReconnectWorkerSignal {
    exit: AtomicBool,
}

#[derive(Debug)]
pub enum WorldLoginReconnectWorkerCompletion {
    Returned(WorldLoginReconnectWorkerOutcome),
    Panicked,
}

#[derive(Debug)]
pub enum WorldLoginReconnectThreadStart {
    Started,
    SpawnFailed(io::Error),
}

#[derive(Debug)]
pub struct WorldLoginReconnectThreadRestart {
    pub previous_completion: Option<WorldLoginReconnectWorkerCompletion>,
    pub started: WorldLoginReconnectThreadStart,
}

pub struct WorldLoginReconnectWorker {
    signal: Arc<WorldLoginReconnectWorkerSignal>,
    handle: Option<JoinHandle<WorldLoginReconnectWorkerOutcome>>,
}

impl WorldLoginReconnectWorker {
    pub fn start(
        spec: WorldLoginReconnectSpec,
        runtime: Handle,
    ) -> Result<Self, io::Error> {
        let signal = Arc::new(WorldLoginReconnectWorkerSignal::default());
        let worker_signal = Arc::clone(&signal);
        let handle = thread::Builder::new()
            .name("world-login-reconnect".to_owned())
            .spawn(move || run_worker(spec, runtime, worker_signal))?;
        Ok(Self {
            signal,
            handle: Some(handle),
        })
    }

    pub fn request_exit(&self) {
        self.signal.exit.store(true, Ordering::Relaxed);
    }

    pub fn join(&mut self) -> Option<WorldLoginReconnectWorkerCompletion> {
        self.handle.take().map(|handle| match handle.join() {
            Ok(outcome) => WorldLoginReconnectWorkerCompletion::Returned(outcome),
            Err(_) => WorldLoginReconnectWorkerCompletion::Panicked,
        })
    }

    pub fn stop(&mut self) -> Option<WorldLoginReconnectWorkerCompletion> {
        self.request_exit();
        self.join()
    }
}

impl Drop for WorldLoginReconnectWorker {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn run_worker(
    spec: WorldLoginReconnectSpec,
    runtime: Handle,
    signal: Arc<WorldLoginReconnectWorkerSignal>,
) -> WorldLoginReconnectWorkerOutcome {
    if signal.exit.load(Ordering::Relaxed) {
        return WorldLoginReconnectWorkerOutcome::StoppedBeforeRetry;
    }

    let mut attempts = 0_u32;
    loop {
        thread::sleep(LOGIN_RECONNECT_INTERVAL);
        attempts = attempts.wrapping_add(1);
        if let Ok(reconnect) = runtime.block_on(spec.reconnect_once()) {
            return WorldLoginReconnectWorkerOutcome::Reconnected {
                attempts,
                reconnect,
            };
        }
        if signal.exit.load(Ordering::Relaxed) {
            return WorldLoginReconnectWorkerOutcome::StoppedAfterFailedRetry { attempts };
        }
    }
}
