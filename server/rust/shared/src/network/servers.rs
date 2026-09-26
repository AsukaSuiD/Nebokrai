//! Общий владелец входящих TCP-соединений `CServer` (`nets/servers.cpp/.h`,
//! пары EXE/PDB Auth/Billing/Login/Game/World в `server/rust/src/manifest/`).
//! Реализованы очередь socket-команд, listener, admission, реестры socket/map
//! identity, IPv4-блокировка, таймер первого сообщения, счётчики и полный
//! command snapshot `DoNetThreadFunc`; конкретные callbacks передаются узким
//! trait и не смешивают owner с различающимися `CMessage`.
//!
//! Конкурентна только очередь команд; реестры изменяет один net-thread, и
//! `BTreeMap` сохраняет порядок исходного `std::map`. Порядок прохода
//! сохранён: атомарный снимок очереди → последовательное применение → не
//! более одного нового send на client → timeout-QUIT следующим снимком →
//! close после close flag. Receive-ошибка component возвращается отдельным
//! элементом и не прерывает snapshot; доказанное `ForbidAndQuit` выполняется
//! до следующей команды. Один send-action на соединение сохраняет порядок
//! байтов. Timing pending Windows send против Linux readiness отмечен
//! `BLOCKED_MISSING_FACT` и не выдаётся за полное совпадение.
//!
//! Сохранённые странности: `GetSocketIDByMapStr` по отсутствующему ключу
//! возвращал `1`; `AddAClient` увеличивал счётчик до проверки дубликата и не
//! компенсировал его; `DelOneClient` уменьшал счётчик до `OnClose`, а
//! переполнение `SENDALL` — после разрушения client. Admission и allow-list
//! (`inet_ntoa` + port, запись с port `0` совпадает только по IP, wrapping
//! `timeGetTime`) воспроизведены буквально. Конструкторские defaults: accept
//! 100 ms, net 1 ms, backlog `INT_MAX`, таймаут первого сообщения 8000 ms,
//! max clients 100, send buffer `0xC800`; `m_dwMaxMsgLen` до записи
//! service-owner — `None`. Auth `InitNetServer_Auth` после успешного `Host`
//! менял лимит блокировок на `10` и таймаут на config-значение.
//!
//! Producer-наборы сборок различались (у Auth нет `SendAll`, Login — строковая
//! map identity, Game — числовая); owner хранит доказанное объединение, а
//! компонент вызывает только свой подтверждённый набор. Игровое состояние и
//! момент публикации остаются у владеющей роли; hostname resolution — у
//! конкретного service-owner, его порядок различается.
//! Доказательства: docs/reconstruction/shared-technical.md#владелец-входящих-tcp-соединений-cserver

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use std::error::Error;
use std::fmt;

use tokio::net::{TcpListener, TcpStream};

use super::clients::TransferCounter;
use super::mysocket::{legacy_ipv4_word, SocketIdAllocator, DEFAULT_IP, DEFAULT_SOCKET_TYPE};
use super::serverclient::{
    AddSendDataOutcome, CServerClient, ServerClientSizeError, ServerSendBatch,
    ServerSendCompletion, DEFAULT_PERMITTED_SEND_BYTES,
};
use super::socketcommands::CSocketCommands;
use super::transport::{bind_tcp_ipv4, listen_tcp_ipv4, read_server_tcp_chunk, shutdown_tcp};
use crate::resources::read_to_marker as read_to;

/// Пауза исходного accept-thread при достижении лимита соединений.
pub const ACCEPT_AT_CAPACITY_DELAY: Duration = Duration::from_secs(1);
/// Исходная пауза между итерациями accept-thread.
pub const ACCEPT_THREAD_DELAY: Duration = Duration::from_millis(100);
/// Исходная пауза между снимками общей очереди команд.
pub const NET_THREAD_DELAY: Duration = Duration::from_millis(1);

const DEFAULT_MAX_CLIENTS: i32 = 100;
const DEFAULT_MAX_BACKLOG: i32 = i32::MAX;
const DEFAULT_NEW_ACCEPT_TIMEOUT_MS: i32 = 8000;
const DEFAULT_RECEIVE_RATE_LIMIT: i32 = 0x186A_0000;
const DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 1;

/// Адрес из исходного allow-list без предположения о кодировке файла.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowedAddress {
    ip: Vec<u8>,
    port: u16,
}

impl AllowedAddress {
    /// Сохраняет буквальную строку IP и host-order port исходной записи.
    pub fn new(ip: &[u8], port: u16) -> Self {
        Self {
            ip: ip.to_vec(),
            port,
        }
    }

    /// Проверяет исходное правило: port `0` означает совпадение только по IP.
    /// Поставочный Billing-конфиг может содержать Docker/DNS-имя; оно
    /// разрешается при admission и сопоставляется с фактическим IPv4 peer-а.
    pub fn matches(&self, peer_ip: &[u8], peer_port: u16) -> bool {
        if self.port != 0 && self.port != peer_port {
            return false;
        }
        if self.ip == peer_ip {
            return true;
        }
        let Ok(host) = std::str::from_utf8(&self.ip) else {
            return false;
        };
        let Ok(peer_text) = std::str::from_utf8(peer_ip) else {
            return false;
        };
        let Ok(peer) = peer_text.parse::<Ipv4Addr>() else {
            return false;
        };
        (host, 0).to_socket_addrs().ok().is_some_and(|addresses| {
            addresses.into_iter().any(|address| match address {
                SocketAddr::V4(address) => *address.ip() == peer,
                SocketAddr::V6(_) => false,
            })
        })
    }
}

/// Принятое TCP-соединение и его общий `CServerClient`-owner.
pub struct AcceptedServerClient {
    stream: Arc<TcpStream>,
    state: CServerClient,
}

impl AcceptedServerClient {
    fn new(stream: TcpStream, state: CServerClient) -> Self {
        Self {
            stream: Arc::new(stream),
            state,
        }
    }

    /// Возвращает shared transport handle для одной read/write operation.
    pub fn stream(&self) -> Arc<TcpStream> {
        Arc::clone(&self.stream)
    }

    /// Возвращает общее состояние принятого соединения.
    pub const fn state(&self) -> &CServerClient {
        &self.state
    }

    /// Возвращает общее состояние для единственного net-owner.
    pub const fn state_mut(&mut self) -> &mut CServerClient {
        &mut self.state
    }
}

/// Владеющая форма доказанных вариантов старого `tagSocketOper`.
pub enum ServerSocketCommand {
    /// Принятое соединение готово к добавлению в реестр net-owner.
    Add {
        /// Выданный process-local socket ID.
        socket_id: i32,
        /// Wrapping tick принятия для восьмисекундного контроля первого пакета.
        accepted_at_ms: u32,
        /// Владеющее соединение вместо старого `pBuf`.
        client: AcceptedServerClient,
    },
    /// Строковая identity присоединяется к существующему соединению.
    SetMapName {
        /// Socket ID назначения.
        socket_id: i32,
        /// Буквальные bytes старой C-string без завершающего NUL.
        map_name: Vec<u8>,
    },
    /// Числовая identity присоединяется к существующему соединению.
    SetMapId {
        /// Socket ID назначения.
        socket_id: i32,
        /// Исходный signed map ID.
        map_id: i32,
    },
    /// Соединение удаляется из реестра по socket ID.
    DeleteBySocketId { socket_id: i32 },
    /// Соединение получает close flag по socket ID.
    QuitBySocketId { socket_id: i32 },
    /// Соединение получает close flag через числовую identity.
    QuitByMapId { map_id: i32 },
    /// Соединение получает close flag через строковую identity.
    QuitByMapName { map_name: Vec<u8> },
    /// Все зарегистрированные соединения получают close flag.
    QuitAll,
    /// Один owned фрагмент, прочитанный старой completion operation.
    Receive { socket_id: i32, buffer: Vec<u8> },
    /// Owned payload отправляется одному socket ID.
    SendToSocket { socket_id: i32, buffer: Vec<u8> },
    /// Owned payload отправляется через числовую identity.
    SendToMapId { map_id: i32, buffer: Vec<u8> },
    /// Owned payload отправляется через строковую identity.
    SendToMapName { map_name: Vec<u8>, buffer: Vec<u8> },
    /// Owned payload добавляется каждому текущему соединению.
    SendAll { buffer: Vec<u8> },
    /// Завершилась одна ранее начатая send-operation.
    SendEnd { socket_id: i32 },
}

/// Клонируемая граница producers к единственному net-owner.
#[derive(Clone)]
pub struct ServerCommandHandle {
    commands: Arc<CSocketCommands<ServerSocketCommand>>,
}

impl ServerCommandHandle {
    fn new() -> Self {
        Self {
            commands: Arc::new(CSocketCommands::new()),
        }
    }

    /// Копирует непустой payload в команду отправки одному socket ID.
    pub fn send_by_socket_id(&self, socket_id: i32, buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendToSocket {
            socket_id,
            buffer,
        })
    }

    /// Копирует непустой payload в команду отправки по числовой identity.
    pub fn send_by_map_id(&self, map_id: i32, buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendToMapId {
            map_id,
            buffer,
        })
    }

    /// Копирует непустой payload и строковую identity в одну owned-команду.
    pub fn send_by_map_name(&self, map_name: &[u8], buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendToMapName {
            map_name: map_name.to_vec(),
            buffer,
        })
    }

    /// Копирует непустой payload в broadcast-команду.
    pub fn send_all(&self, buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendAll { buffer })
    }

    /// Ставит исходную безусловно успешную команду close по socket ID.
    pub fn quit_by_socket_id(&self, socket_id: i32) -> i32 {
        self.commands
            .push_back(ServerSocketCommand::QuitBySocketId { socket_id });
        1
    }

    /// Ставит команду close по числовой identity.
    pub fn quit_by_map_id(&self, map_id: i32) -> i32 {
        self.commands
            .push_back(ServerSocketCommand::QuitByMapId { map_id });
        1
    }

    /// Копирует строковую identity в команду close.
    pub fn quit_by_map_name(&self, map_name: &[u8]) -> i32 {
        self.commands.push_back(ServerSocketCommand::QuitByMapName {
            map_name: map_name.to_vec(),
        });
        1
    }

    /// Ставит команду присоединения числовой identity.
    pub fn set_client_map_id(&self, socket_id: i32, map_id: i32) -> i32 {
        self.commands
            .push_back(ServerSocketCommand::SetMapId { socket_id, map_id });
        1
    }

    /// Копирует строковую identity в команду присоединения.
    pub fn set_client_map_name(&self, socket_id: i32, map_name: &[u8]) -> i32 {
        self.commands.push_back(ServerSocketCommand::SetMapName {
            socket_id,
            map_name: map_name.to_vec(),
        });
        1
    }

    /// Ставит команду закрытия всех соединений.
    pub fn quit_all(&self) -> i32 {
        self.commands.push_back(ServerSocketCommand::QuitAll);
        1
    }

    /// Возвращает число ожидающих команд в исходном signed типе.
    pub fn pending(&self) -> i32 {
        self.commands.get_size()
    }

    /// Публикует один owned receive-completion из Linux transport worker.
    pub fn publish_receive(&self, socket_id: i32, buffer: Vec<u8>) {
        self.commands
            .push_back(ServerSocketCommand::Receive { socket_id, buffer });
    }

    /// Публикует закрытие либо transport-ошибку как исходный delete-command.
    pub fn publish_delete(&self, socket_id: i32) {
        self.commands
            .push_back(ServerSocketCommand::DeleteBySocketId { socket_id });
    }

    /// Публикует успешное завершение одной send-operation.
    pub fn publish_send_end(&self, socket_id: i32) {
        self.commands
            .push_back(ServerSocketCommand::SendEnd { socket_id });
    }

    fn push_nonempty(
        &self,
        buffer: &[u8],
        make: impl FnOnce(Vec<u8>) -> ServerSocketCommand,
    ) -> i32 {
        if buffer.is_empty() {
            return 0;
        }
        self.commands.push_back(make(buffer.to_vec()));
        1
    }

    fn push(&self, command: ServerSocketCommand) {
        self.commands.push_back(command);
    }

    fn take_all(&self) -> VecDeque<ServerSocketCommand> {
        self.commands.take_all()
    }
}

/// Component callbacks, которые старый общий net-thread вызывал виртуально.
///
/// Trait не знает opcode и доменное состояние. Он сохраняет только места, где
/// пять вариантов `CServer::DoNetThreadFunc` передавали управление конкретному
/// производному `CServerClient/CServer`.
pub trait ServerComponentCallbacks {
    /// Ошибка конкретного receive-parser либо component callback.
    type Error;

    /// Обрабатывает уже накопленный общий receive-buffer одного соединения.
    fn on_receive(
        &mut self,
        client: &mut CServerClient,
        recv_time_ms: u32,
    ) -> Result<(), Self::Error>;

    /// Выбирает доказанное общее действие после конкретной receive-ошибки.
    fn on_receive_error(
        &mut self,
        client: &mut CServerClient,
        error: &Self::Error,
    ) -> ComponentReceiveErrorAction;

    /// Выполняет component `OnClose` непосредственно перед удалением client.
    fn on_close(&mut self, client: &mut CServerClient);

    /// Сохраняет component-реакцию на превышение receive-rate.
    fn on_receive_rate_exceeded(&mut self, client: &mut CServerClient, actual: i32, permitted: i32);

    /// Сохраняет virtual callback при PLAYERJOIN для отсутствующего socket.
    fn on_missing_map_id_client(&mut self, map_id: i32, socket_id: i32);

    /// Сохраняет virtual callback при CDKEYJOIN для отсутствующего socket.
    fn on_missing_map_name_client(&mut self, map_name: &[u8], socket_id: i32);
}

/// Общее действие, которое component явно доказал для своей receive-ошибки.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentReceiveErrorAction {
    /// Component не назначал общей сетевой реакции.
    None,
    /// Сначала заблокировать peer IPv4, затем поставить `QUIT` по socket ID.
    ForbidAndQuit,
}

// Обычные SEND-команды вызывали DelOneClient, а SENDALL имел собственный
// порядок erase/OnClose/destructor/count. Один removal helper изменил бы баг.
#[derive(Clone, Copy)]
enum SendOverflowRemoval {
    DelOneClient,
    Broadcast,
}

/// Ошибка одного общего command snapshot.
#[derive(Debug)]
pub enum ServerSnapshotError<ComponentError> {
    /// Общий client-buffer достиг неразрешённой signed 32-битной границы.
    ClientSize {
        /// Socket ID локального ошибочного пути.
        socket_id: i32,
        /// Неопределённая граница исходного signed размера.
        error: ServerClientSizeError,
    },
    /// Component receive/callback завершился собственной ошибкой.
    Component {
        /// Socket ID сообщения, которое остановило component callback.
        socket_id: i32,
        /// Ошибка конкретного направления.
        error: ComponentError,
    },
}

/// Owned Linux I/O, запущенный после применения текущего command snapshot.
pub enum ServerIoAction {
    /// Единственный последовательный read-loop принятого соединения.
    Receive {
        /// Socket ID для публикации completion-команд.
        socket_id: i32,
        /// Shared transport handle зарегистрированного клиента.
        stream: Arc<TcpStream>,
    },
    /// Одна исходная overlapped send-operation.
    Send {
        /// Socket ID для `SENDEND` либо `DELBYSOCKETID`.
        socket_id: i32,
        /// Shared transport handle зарегистрированного клиента.
        stream: Arc<TcpStream>,
        /// Owned batch, уже снятый с client accumulator.
        batch: ServerSendBatch,
        /// После этого write исходный close flag требует закрыть socket.
        shutdown_after: bool,
    },
}

/// Наблюдаемый итог завершившегося Linux I/O action.
#[derive(Debug)]
pub enum ServerIoCompletion {
    /// Receive-loop завершился EOF либо transport-ошибкой и поставил delete.
    ReceiveEnded {
        /// Socket ID завершившегося соединения.
        socket_id: i32,
        /// Ошибка transport; `None` означает чистый EOF.
        error: Option<io::Error>,
    },
    /// Send завершился и поставил `SENDEND` либо delete.
    SendEnded {
        /// Socket ID завершившейся операции.
        socket_id: i32,
        /// Полная запись batch либо terminal transport-ошибка.
        result: io::Result<ServerSendCompletion>,
    },
}

impl ServerIoAction {
    /// Исполняет owned action и публикует его результат в ту же command queue.
    pub async fn run(self, commands: ServerCommandHandle) -> ServerIoCompletion {
        match self {
            Self::Receive { socket_id, stream } => loop {
                match read_server_tcp_chunk(&stream).await {
                    Ok(buffer) if !buffer.is_empty() => {
                        commands.publish_receive(socket_id, buffer);
                    }
                    Ok(_) => {
                        commands.publish_delete(socket_id);
                        return ServerIoCompletion::ReceiveEnded {
                            socket_id,
                            error: None,
                        };
                    }
                    Err(error) => {
                        commands.publish_delete(socket_id);
                        return ServerIoCompletion::ReceiveEnded {
                            socket_id,
                            error: Some(error),
                        };
                    }
                }
            },
            Self::Send {
                socket_id,
                stream,
                batch,
                shutdown_after,
            } => {
                let result = batch.write_complete(&stream).await;
                if shutdown_after {
                    let _ = shutdown_tcp(&stream);
                }
                if result.is_ok() {
                    commands.publish_send_end(socket_id);
                } else {
                    // BLOCKED_MISSING_FACT: Linux write объединяет два
                    // Windows-пути: немедленный отказ WSASend удалял client
                    // прямо в send-проходе, а ошибка уже pending IOCP шла через
                    // worker delete-command. Точку отказа после readiness
                    // различить нельзя; сохраняется общий terminal delete.
                    commands.publish_delete(socket_id);
                }
                ServerIoCompletion::SendEnded { socket_id, result }
            }
        }
    }
}

/// Итог одного атомарно взятого snapshot старой очереди socket-команд.
pub struct ServerSnapshot<ComponentError> {
    io_actions: Vec<ServerIoAction>,
    processed_commands: i32,
    errors: Vec<ServerSnapshotError<ComponentError>>,
}

impl<ComponentError> ServerSnapshot<ComponentError> {
    /// Передаёт runtime I/O actions и локальные ошибки component-путей.
    pub fn into_parts(
        self,
    ) -> (
        Vec<ServerIoAction>,
        Vec<ServerSnapshotError<ComponentError>>,
    ) {
        (self.io_actions, self.errors)
    }

    /// Возвращает число команд исходного snapshot до новых completion-событий.
    pub const fn processed_commands(&self) -> i32 {
        self.processed_commands
    }
}

/// Ошибка создания Linux listener из доказанных параметров `Host`.
#[derive(Debug)]
pub enum ServerHostError {
    /// Все найденные call sites передают `SOCK_STREAM`; другой тип не доказан.
    UnsupportedSocketType(i32),
    /// Transport не смог bind-нуть либо перевести socket в listen.
    Io(io::Error),
}

impl fmt::Display for ServerHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSocketType(socket_type) => {
                write!(
                    formatter,
                    "для CServer не восстановлен socket type {socket_type}"
                )
            }
            Self::Io(error) => write!(formatter, "не удалось создать TCP listener: {error}"),
        }
    }
}

impl Error for ServerHostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedSocketType(_) => None,
            Self::Io(error) => Some(error),
        }
    }
}

/// Результат попытки начать единственный исходный blocking accept.
pub enum AcceptStart {
    /// `Host` ещё не создал listener.
    NotListening,
    /// Signed client count достиг исходного ограничения.
    AtCapacity,
    /// Listener может выполнить один accept.
    Pending(ServerPendingAccept),
}

/// Одна ожидающая accept-operation без Windows thread/WSAAccept plumbing.
pub struct ServerPendingAccept {
    listener: Arc<TcpListener>,
}

impl ServerPendingAccept {
    /// Принимает ровно одно IPv4 TCP-соединение.
    pub async fn accept(self) -> io::Result<(TcpStream, SocketAddrV4)> {
        let (stream, peer) = self.listener.accept().await?;
        match peer {
            SocketAddr::V4(peer) => Ok((stream, peer)),
            SocketAddr::V6(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "IPv4 CServer получил IPv6 peer address",
            )),
        }
    }
}

/// Результат admission уже принятого transport-соединения.
#[derive(Debug, Eq, PartialEq)]
pub enum AdmissionOutcome {
    /// Лимит был достигнут между началом и завершением accept.
    AtCapacity,
    /// Peer отсутствует в включённом allow-list.
    AddressRejected,
    /// Peer IPv4 ещё находится во временном forbid-map.
    TemporarilyForbidden,
    /// Owned `ADD` поставлен единственному net-owner.
    Queued { socket_id: i32 },
}

/// Установленный экземпляр и его потребители остаются у владельца роли.
/// Установленный экземпляр и его потребители остаются у владельца роли.
/// Общий state-owner одного исторического `CServer`.
pub struct CServer {
    listener: Option<Arc<TcpListener>>,
    commands: ServerCommandHandle,
    clients: BTreeMap<i32, AcceptedServerClient>,
    map_id_socket_id: BTreeMap<i32, i32>,
    map_name_socket_id: BTreeMap<Vec<u8>, i32>,
    forbidden_ips: BTreeMap<u32, u32>,
    new_accept_sockets: BTreeMap<i32, u32>,
    allowed_addresses: Vec<AllowedAddress>,
    local_ip: Vec<u8>,
    local_ipv4_word: u32,
    socket_ids: SocketIdAllocator,
    client_count: i32,
    max_clients: i32,
    max_backlog: i32,
    new_accept_timeout_ms: i32,
    receive_rate_limit: i32,
    forbid_time_ms: i32,
    max_in_flight_sends: i32,
    permitted_send_bytes: i32,
    check_receive_rate: bool,
    check_message_content: bool,
    maximum_message_length: Option<u32>,
    check_allowed_address: bool,
    send_counter: TransferCounter,
    receive_counter: TransferCounter,
}

impl CServer {
    /// Создаёт пустой server-owner с доказанными constructor defaults.
    pub fn new(now_ms: u32) -> Self {
        Self {
            listener: None,
            commands: ServerCommandHandle::new(),
            clients: BTreeMap::new(),
            map_id_socket_id: BTreeMap::new(),
            map_name_socket_id: BTreeMap::new(),
            forbidden_ips: BTreeMap::new(),
            new_accept_sockets: BTreeMap::new(),
            allowed_addresses: Vec::new(),
            local_ip: DEFAULT_IP.to_string().into_bytes(),
            local_ipv4_word: legacy_ipv4_word(DEFAULT_IP),
            socket_ids: SocketIdAllocator::new(),
            client_count: 0,
            max_clients: DEFAULT_MAX_CLIENTS,
            max_backlog: DEFAULT_MAX_BACKLOG,
            new_accept_timeout_ms: DEFAULT_NEW_ACCEPT_TIMEOUT_MS,
            receive_rate_limit: DEFAULT_RECEIVE_RATE_LIMIT,
            forbid_time_ms: 0,
            max_in_flight_sends: DEFAULT_MAX_IN_FLIGHT_SENDS,
            permitted_send_bytes: DEFAULT_PERMITTED_SEND_BYTES,
            check_receive_rate: false,
            check_message_content: false,
            maximum_message_length: None,
            check_allowed_address: false,
            send_counter: TransferCounter::new(now_ms),
            receive_counter: TransferCounter::new(now_ms),
        }
    }

    /// Создаёт, bind-ит и переводит IPv4 TCP socket в listen.
    ///
    /// Неиспользованный исходный bool сохранён в сигнатуре: ни один из пяти
    /// `Host` не читал его. Windows threads будут заменены живым component
    /// runtime, а не создаются как пустое состояние здесь.
    pub fn host(
        &mut self,
        port: u32,
        address: Option<Ipv4Addr>,
        socket_type: i32,
        _legacy_flag: bool,
    ) -> Result<(), ServerHostError> {
        if socket_type != DEFAULT_SOCKET_TYPE {
            return Err(ServerHostError::UnsupportedSocketType(socket_type));
        }
        let socket = bind_tcp_ipv4(address, port).map_err(ServerHostError::Io)?;
        let listener = listen_tcp_ipv4(socket, self.max_backlog).map_err(ServerHostError::Io)?;
        self.listener = Some(Arc::new(listener));
        Ok(())
    }

    /// Возвращает producer handle конкретного исторического сервиса.
    pub fn command_handle(&self) -> ServerCommandHandle {
        self.commands.clone()
    }

    /// Начинает один accept либо буквально сообщает исходную причину ожидания.
    pub fn begin_accept(&self) -> AcceptStart {
        if self.client_count >= self.max_clients {
            return AcceptStart::AtCapacity;
        }
        match &self.listener {
            Some(listener) => AcceptStart::Pending(ServerPendingAccept {
                listener: Arc::clone(listener),
            }),
            None => AcceptStart::NotListening,
        }
    }

    /// Проверяет peer и ставит принятое соединение в очередь `ADD`.
    pub fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.queue_accepted_with(stream, peer, now_ms, CServerClient::new)
    }

    /// Проверяет peer и вызывает доказанную component-замену virtual
    /// `CreateServerClient` перед постановкой `ADD`.
    pub fn queue_accepted_with(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
        create_client: impl FnOnce(i32, u32, u32) -> CServerClient,
    ) -> AdmissionOutcome {
        if self.client_count >= self.max_clients {
            return AdmissionOutcome::AtCapacity;
        }

        let peer_text = peer.ip().to_string();
        if self.check_allowed_address && !self.is_allowed_address(peer_text.as_bytes(), peer.port())
        {
            return AdmissionOutcome::AddressRejected;
        }

        let peer_ipv4 = legacy_ipv4_word(*peer.ip());
        if self.check_receive_rate && self.find_forbidden_ip(peer_ipv4, now_ms) {
            return AdmissionOutcome::TemporarilyForbidden;
        }

        let socket_id = self.socket_ids.next();
        let state = create_client(socket_id, peer_ipv4, now_ms);
        self.commands.push(ServerSocketCommand::Add {
            socket_id,
            accepted_at_ms: now_ms,
            client: AcceptedServerClient::new(stream, state),
        });
        AdmissionOutcome::Queued { socket_id }
    }

    /// Атомарно забирает текущий снимок команд для единственного net-owner.
    pub fn take_commands(&self) -> VecDeque<ServerSocketCommand> {
        self.commands.take_all()
    }

    /// Применяет ровно один атомарный snapshot старого `DoNetThreadFunc`.
    ///
    /// Новые completion-команды, опубликованные во время выполнения, остаются
    /// следующему проходу. Component и безопасные size-ошибки локализуются в
    /// результате и не уничтожают остальные уже снятые команды.
    pub fn process_command_snapshot<Callbacks>(
        &mut self,
        callbacks: &mut Callbacks,
        now_ms: u32,
    ) -> ServerSnapshot<Callbacks::Error>
    where
        Callbacks: ServerComponentCallbacks,
    {
        let commands = self.take_commands();
        let processed_commands = commands.len() as u32 as i32;
        let mut io_actions = Vec::new();
        let mut errors = Vec::new();

        for command in commands {
            match command {
                ServerSocketCommand::Add {
                    socket_id,
                    accepted_at_ms,
                    client,
                } => {
                    let stream = client.stream();
                    self.client_count = self.client_count.wrapping_add(1);
                    if let Some(displaced) = self.clients.remove(&socket_id) {
                        let _ = shutdown_tcp(&displaced.stream());
                        drop(displaced);
                    }
                    self.new_accept_sockets.insert(socket_id, accepted_at_ms);
                    self.clients.insert(socket_id, client);
                    io_actions.push(ServerIoAction::Receive { socket_id, stream });
                }
                ServerSocketCommand::SetMapName {
                    socket_id,
                    map_name,
                } => {
                    if !self.assign_map_name(socket_id, &map_name) {
                        callbacks.on_missing_map_name_client(&map_name, socket_id);
                    }
                }
                ServerSocketCommand::SetMapId { socket_id, map_id } => {
                    if !self.assign_map_id(socket_id, map_id) {
                        callbacks.on_missing_map_id_client(map_id, socket_id);
                    }
                }
                ServerSocketCommand::DeleteBySocketId { socket_id } => {
                    self.remove_client_with_callback(socket_id, callbacks);
                }
                ServerSocketCommand::QuitBySocketId { socket_id } => {
                    self.mark_client_closing(socket_id);
                }
                ServerSocketCommand::QuitByMapId { map_id } => {
                    let socket_id = self.get_socket_id_by_map_id(map_id);
                    self.remove_map_id(map_id);
                    self.mark_client_closing(socket_id);
                }
                ServerSocketCommand::QuitByMapName { map_name } => {
                    // Сохранена странность: отсутствующая строковая identity
                    // даёт socket ID 1 и может закрыть первое соединение.
                    let socket_id = self.get_socket_id_by_map_name(&map_name);
                    self.remove_map_name(&map_name);
                    self.mark_client_closing(socket_id);
                }
                ServerSocketCommand::QuitAll => self.mark_all_clients_closing(),
                ServerSocketCommand::Receive { socket_id, buffer } => {
                    self.process_receive_command(
                        socket_id,
                        &buffer,
                        callbacks,
                        now_ms,
                        &mut errors,
                    );
                }
                ServerSocketCommand::SendToSocket { socket_id, buffer } => {
                    self.buffer_send_command(
                        socket_id,
                        &buffer,
                        SendOverflowRemoval::DelOneClient,
                        callbacks,
                        &mut errors,
                    );
                }
                ServerSocketCommand::SendToMapId { map_id, buffer } => {
                    let socket_id = self.get_socket_id_by_map_id(map_id);
                    if socket_id != 0 {
                        self.buffer_send_command(
                            socket_id,
                            &buffer,
                            SendOverflowRemoval::DelOneClient,
                            callbacks,
                            &mut errors,
                        );
                    }
                }
                ServerSocketCommand::SendToMapName { map_name, buffer } => {
                    let socket_id = self.get_socket_id_by_map_name(&map_name);
                    if socket_id != 0 {
                        self.buffer_send_command(
                            socket_id,
                            &buffer,
                            SendOverflowRemoval::DelOneClient,
                            callbacks,
                            &mut errors,
                        );
                    }
                }
                ServerSocketCommand::SendAll { buffer } => {
                    let socket_ids: Vec<i32> = self.clients.keys().copied().collect();
                    for socket_id in socket_ids {
                        self.buffer_send_command(
                            socket_id,
                            &buffer,
                            SendOverflowRemoval::Broadcast,
                            callbacks,
                            &mut errors,
                        );
                    }
                }
                ServerSocketCommand::SendEnd { socket_id } => {
                    if let Some(client) = self.clients.get_mut(&socket_id) {
                        client.state_mut().finish_send_operation();
                    }
                }
            }
        }

        let socket_ids: Vec<i32> = self.clients.keys().copied().collect();
        let mut shutdown_after_send = Vec::new();
        for socket_id in socket_ids {
            let action = {
                let Some(client) = self.clients.get_mut(&socket_id) else {
                    continue;
                };
                // Один batch на соединение сохраняет порядок байтов,
                // включая дописывание хвоста после частичного Linux write.
                if client.state().io_operations() != 0
                    || client.state().io_operations() >= self.max_in_flight_sends
                {
                    None
                } else {
                    let closing = client.state().is_closing();
                    client
                        .state_mut()
                        .begin_send()
                        .map(|batch| (client.stream(), batch, closing))
                }
            };
            if let Some((stream, batch, shutdown_after)) = action {
                self.add_send_size(batch.requested_bytes() as u32 as i32, now_ms);
                if shutdown_after {
                    shutdown_after_send.push(socket_id);
                }
                io_actions.push(ServerIoAction::Send {
                    socket_id,
                    stream,
                    batch,
                    shutdown_after,
                });
            }
        }

        // Как старый DoNewAcceptSocket, публикует QUIT только в следующий
        // snapshot, а не закрывает просроченные соединения внутри этого прохода.
        self.expire_new_accepts(now_ms);

        for (&socket_id, client) in &mut self.clients {
            // QUIT во время предыдущей записи не должен обрезать её или
            // принятый до QUIT следующий batch. Дожидаемся SENDEND.
            if client.state().io_operations() != 0 && !shutdown_after_send.contains(&socket_id) {
                continue;
            }
            if client.state_mut().begin_close() {
                if shutdown_after_send.contains(&socket_id) {
                    // BLOCKED_MISSING_FACT: Windows WSASend уже был submitted
                    // до closesocket, а Tokio write становится syscall только
                    // после readiness. Составной action сохраняет порядок
                    // bytes-before-shutdown, но точная pending-send timing
                    // между платформами не объявляется доказанной.
                } else {
                    // `closesocket` не проверял результат. Shutdown лишь будит
                    // read-loop; OnClose выполнится по его delete-command.
                    let _ = shutdown_tcp(&client.stream());
                }
            }
        }

        ServerSnapshot {
            io_actions,
            processed_commands,
            errors,
        }
    }

    fn process_receive_command<Callbacks>(
        &mut self,
        socket_id: i32,
        buffer: &[u8],
        callbacks: &mut Callbacks,
        now_ms: u32,
        errors: &mut Vec<ServerSnapshotError<Callbacks::Error>>,
    ) where
        Callbacks: ServerComponentCallbacks,
    {
        self.acknowledge_first_receive(socket_id);
        let Ok(received) = i32::try_from(buffer.len()) else {
            errors.push(ServerSnapshotError::ClientSize {
                socket_id,
                error: ServerClientSizeError::ReceiveSizeOverflowReactionUnknown,
            });
            return;
        };

        if self.check_receive_rate {
            self.add_receive_size(received, now_ms);
            let exceeded = {
                let Some(client) = self.clients.get_mut(&socket_id) else {
                    return;
                };
                if client.state().is_closing() {
                    return;
                }
                let actual = client.state_mut().add_package_size(received, now_ms);
                if actual > self.receive_rate_limit {
                    callbacks.on_receive_rate_exceeded(
                        client.state_mut(),
                        actual,
                        self.receive_rate_limit,
                    );
                    Some(client.state().message_context().peer_ipv4)
                } else {
                    None
                }
            };
            if let Some(peer_ipv4) = exceeded {
                self.add_forbidden_ip(peer_ipv4, now_ms);
                self.commands.quit_by_socket_id(socket_id);
                return;
            }
        }

        let component_failure = {
            let Some(client) = self.clients.get_mut(&socket_id) else {
                return;
            };
            if client.state().is_closing() {
                return;
            }
            if let Err(error) = client.state_mut().add_receive_data(buffer) {
                errors.push(ServerSnapshotError::ClientSize { socket_id, error });
                return;
            }
            match callbacks.on_receive(client.state_mut(), now_ms) {
                Ok(()) => None,
                Err(error) => {
                    let action = callbacks.on_receive_error(client.state_mut(), &error);
                    let peer_ipv4 = client.state().message_context().peer_ipv4;
                    Some((error, action, peer_ipv4))
                }
            }
        };

        if let Some((error, action, peer_ipv4)) = component_failure {
            errors.push(ServerSnapshotError::Component { socket_id, error });
            if action == ComponentReceiveErrorAction::ForbidAndQuit {
                self.add_forbidden_ip(peer_ipv4, now_ms);
                self.commands.quit_by_socket_id(socket_id);
            }
        }
    }

    fn buffer_send_command<Callbacks>(
        &mut self,
        socket_id: i32,
        buffer: &[u8],
        removal: SendOverflowRemoval,
        callbacks: &mut Callbacks,
        errors: &mut Vec<ServerSnapshotError<Callbacks::Error>>,
    ) where
        Callbacks: ServerComponentCallbacks,
    {
        let outcome = {
            let Some(client) = self.clients.get_mut(&socket_id) else {
                return;
            };
            client
                .state_mut()
                .add_send_data(buffer, self.permitted_send_bytes)
        };
        match outcome {
            Ok(AddSendDataOutcome::LimitExceeded) => {
                tracing::warn!(
                    socket_id,
                    incoming_bytes = buffer.len(),
                    pending_bytes = self
                        .clients
                        .get(&socket_id)
                        .map(|client| client.state().pending_send_bytes()),
                    permitted_bytes = self.permitted_send_bytes,
                    "соединение закрывается: превышен лимит очереди отправки"
                );
                match removal {
                    SendOverflowRemoval::DelOneClient => {
                        self.remove_client_with_callback(socket_id, callbacks);
                    }
                    SendOverflowRemoval::Broadcast => {
                        self.remove_broadcast_overflow_client(socket_id, callbacks);
                    }
                }
            }
            Ok(AddSendDataOutcome::Buffered | AddSendDataOutcome::IgnoredWhileClosing) => {}
            Err(error) => errors.push(ServerSnapshotError::ClientSize { socket_id, error }),
        }
    }

    fn remove_client_with_callback<Callbacks>(
        &mut self,
        socket_id: i32,
        callbacks: &mut Callbacks,
    ) -> Option<AcceptedServerClient>
    where
        Callbacks: ServerComponentCallbacks,
    {
        if !self.clients.contains_key(&socket_id) {
            return None;
        }
        self.client_count = self.client_count.wrapping_sub(1);
        callbacks.on_close(self.clients.get_mut(&socket_id)?.state_mut());
        let removed = self.clients.remove(&socket_id)?;
        let _ = shutdown_tcp(&removed.stream());
        Some(removed)
    }

    fn remove_broadcast_overflow_client<Callbacks>(
        &mut self,
        socket_id: i32,
        callbacks: &mut Callbacks,
    ) where
        Callbacks: ServerComponentCallbacks,
    {
        let Some(mut removed) = self.clients.remove(&socket_id) else {
            return;
        };
        callbacks.on_close(removed.state_mut());
        let _ = shutdown_tcp(&removed.stream());
        drop(removed);
        self.client_count = self.client_count.wrapping_sub(1);
    }

    /// Возвращает соединение по socket ID вместо nullable C++ pointer.
    pub fn client(&self, socket_id: i32) -> Option<&AcceptedServerClient> {
        self.clients.get(&socket_id)
    }

    /// Возвращает соединение единственному изменяющему net-owner.
    pub fn client_mut(&mut self, socket_id: i32) -> Option<&mut AcceptedServerClient> {
        self.clients.get_mut(&socket_id)
    }

    /// Возвращает исходный signed счётчик зарегистрированных соединений.
    pub const fn client_count(&self) -> i32 {
        self.client_count
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    ///
    /// Это намеренно не сравнение [`Self::client_count`] с нулём: сохранённая
    /// странность duplicate `ADD` могла рассинхронизировать signed счётчик и
    /// фактический размер map, а `ExitWorkerThread` ждал именно map size.
    pub fn has_clients(&self) -> bool {
        !self.clients.is_empty()
    }

    /// Ставит close flag найденному соединению; отсутствие остаётся no-op.
    pub fn mark_client_closing(&mut self, socket_id: i32) {
        if let Some(client) = self.clients.get_mut(&socket_id) {
            client.state_mut().mark_closing();
        }
    }

    /// Ставит close flag всем соединениям в порядке socket ID.
    pub fn mark_all_clients_closing(&mut self) {
        for client in self.clients.values_mut() {
            client.state_mut().mark_closing();
        }
    }

    /// Присоединяет числовую identity и обновляет её routing-index.
    pub fn assign_map_id(&mut self, socket_id: i32, map_id: i32) -> bool {
        let Some(client) = self.clients.get_mut(&socket_id) else {
            return false;
        };
        client.state_mut().set_map_id(map_id);
        self.map_id_socket_id.insert(map_id, socket_id);
        true
    }

    /// Присоединяет строковую identity и обновляет её routing-index.
    pub fn assign_map_name(&mut self, socket_id: i32, map_name: &[u8]) -> bool {
        let Some(client) = self.clients.get_mut(&socket_id) else {
            return false;
        };
        client.state_mut().set_map_name(map_name);
        self.map_name_socket_id.insert(map_name.to_vec(), socket_id);
        true
    }

    /// Возвращает socket ID числовой identity либо исходный ноль.
    pub fn get_socket_id_by_map_id(&self, map_id: i32) -> i32 {
        self.map_id_socket_id.get(&map_id).copied().unwrap_or(0)
    }

    /// Возвращает socket ID строковой identity либо исходную странность `1`.
    pub fn get_socket_id_by_map_name(&self, map_name: &[u8]) -> i32 {
        self.map_name_socket_id.get(map_name).copied().unwrap_or(1)
    }

    /// Удаляет числовой routing-index и возвращает прежний socket ID.
    pub fn remove_map_id(&mut self, map_id: i32) -> Option<i32> {
        self.map_id_socket_id.remove(&map_id)
    }

    /// Удаляет строковый routing-index и возвращает прежний socket ID.
    pub fn remove_map_name(&mut self, map_name: &[u8]) -> Option<i32> {
        self.map_name_socket_id.remove(map_name)
    }

    /// Снимает восьмисекундный контроль после первого receive этого socket ID.
    pub fn acknowledge_first_receive(&mut self, socket_id: i32) {
        self.new_accept_sockets.remove(&socket_id);
    }

    /// Ставит close-команды соединениям без первого пакета и удаляет их таймеры.
    pub fn expire_new_accepts(&mut self, now_ms: u32) -> Vec<i32> {
        let timeout = self.new_accept_timeout_ms as u32;
        let expired: Vec<i32> = self
            .new_accept_sockets
            .iter()
            .filter_map(|(&socket_id, &accepted_at)| {
                (now_ms.wrapping_sub(accepted_at) >= timeout).then_some(socket_id)
            })
            .collect();
        for socket_id in &expired {
            self.commands.quit_by_socket_id(*socket_id);
            self.new_accept_sockets.remove(socket_id);
        }
        expired
    }

    /// Загружает общий `allowed_*.ini`, очищая прежний список до открытия.
    ///
    /// Ошибка открытия оставляет прежний флаг проверки, но уже пустой список.
    /// Успешное открытие возвращает `Ok` даже без маркеров, как исходный метод.
    pub fn load_allowed_clients(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        self.allowed_addresses.clear();
        let source = fs::read(path)?;
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());

        let _label = tokens.next();
        let enabled = match tokens.next() {
            Some(b"0") => false,
            Some(b"1") => true,
            _ => return Ok(()),
        };
        self.check_allowed_address = enabled;
        if !enabled {
            return Ok(());
        }

        while read_to(&mut tokens, b"#") {
            let Some(ip) = tokens.next() else {
                break;
            };
            let Some(port) = tokens.next().and_then(parse_decimal_u16) else {
                break;
            };
            self.allowed_addresses.push(AllowedAddress::new(ip, port));
        }
        Ok(())
    }

    /// Сохраняет две записи унаследованного `CMySocket` после `Host`.
    pub fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.local_ip = ip.to_vec();
        self.local_ipv4_word = ipv4_word;
    }

    /// Возвращает dotted IPv4 без исходного завершающего NUL.
    pub fn local_ip(&self) -> &[u8] {
        &self.local_ip
    }

    /// Возвращает network bytes в исходном x86-представлении `unsigned long`.
    pub const fn local_ipv4_word(&self) -> u32 {
        self.local_ipv4_word
    }

    /// Проверяет один canonical peer против текущего allow-list.
    pub fn is_allowed_address(&self, peer_ip: &[u8], peer_port: u16) -> bool {
        self.allowed_addresses
            .iter()
            .any(|allowed| allowed.matches(peer_ip, peer_port))
    }

    /// Включает либо выключает исходный receive-rate/forbid механизм.
    pub fn configure_receive_guard(
        &mut self,
        enabled: bool,
        bytes_per_second: i32,
        forbid_time_ms: i32,
    ) {
        self.check_receive_rate = enabled;
        self.receive_rate_limit = bytes_per_second;
        self.forbid_time_ms = forbid_time_ms;
    }

    /// Задаёт доказанные component defaults/настройки send-ограничений.
    pub fn configure_send_limits(&mut self, max_in_flight_sends: i32, permitted_send_bytes: i32) {
        self.max_in_flight_sends = max_in_flight_sends;
        self.permitted_send_bytes = permitted_send_bytes;
    }

    /// Задаёт исходный signed предел одновременно принятых соединений.
    pub fn configure_max_clients(&mut self, max_clients: i32) {
        self.max_clients = max_clients;
    }

    /// Повторяет восемь поздних записей service-owner после успешного `Host`.
    ///
    /// Порядок параметров и присваиваний соответствует старым offset
    /// `+0x10C`, `+0x14C`, `+0x110`, `+0x148`, `+0x10D`, `+0x114`, `+0x118`,
    /// `+0x150`. Поля message validation сохраняются как состояние общего
    /// owner; конкретный component решает, читает ли он их в своём parser-е.
    #[allow(
        clippy::too_many_arguments,
        reason = "это одна точная последовательность восьми записей CServer"
    )]
    pub fn configure_transport_after_host(
        &mut self,
        check_receive_rate: bool,
        max_in_flight_sends: i32,
        receive_rate_limit: u32,
        max_clients: i32,
        check_message_content: bool,
        forbid_time_ms: u32,
        maximum_message_length: u32,
        permitted_send_bytes: i32,
    ) {
        self.check_receive_rate = check_receive_rate;
        self.max_in_flight_sends = max_in_flight_sends;
        self.receive_rate_limit = receive_rate_limit as i32;
        self.max_clients = max_clients;
        self.check_message_content = check_message_content;
        self.forbid_time_ms = forbid_time_ms as i32;
        self.maximum_message_length = Some(maximum_message_length);
        self.permitted_send_bytes = permitted_send_bytes;
    }

    /// Повторяет две Auth-записи после `Host`, включая слишком поздний backlog.
    pub fn configure_accept_limits_after_host(
        &mut self,
        max_backlog: i32,
        new_accept_timeout_ms: i32,
    ) {
        self.max_backlog = max_backlog;
        self.new_accept_timeout_ms = new_accept_timeout_ms;
    }

    /// Запоминает wrapping tick временной блокировки IPv4.
    pub fn add_forbidden_ip(&mut self, peer_ipv4: u32, now_ms: u32) {
        self.forbidden_ips.insert(peer_ipv4, now_ms);
    }

    /// Проверяет временную блокировку и удаляет истёкшую запись.
    pub fn find_forbidden_ip(&mut self, peer_ipv4: u32, now_ms: u32) -> bool {
        let Some(&started_at) = self.forbidden_ips.get(&peer_ipv4) else {
            return false;
        };
        if now_ms.wrapping_sub(started_at) <= self.forbid_time_ms as u32 {
            return true;
        }
        self.forbidden_ips.remove(&peer_ipv4);
        false
    }

    /// Добавляет подтверждённый объём send к общему счётчику сервиса.
    pub fn add_send_size(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.send_counter.add(amount, now_ms)
    }

    /// Добавляет принятый объём к общему счётчику сервиса.
    pub fn add_receive_size(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.receive_counter.add(amount, now_ms)
    }

    /// Возвращает лимит per-client receive-rate для component worker.
    pub const fn receive_rate_limit(&self) -> i32 {
        self.receive_rate_limit
    }

    /// Возвращает максимум одновременно незавершённых send одного клиента.
    pub const fn max_in_flight_sends(&self) -> i32 {
        self.max_in_flight_sends
    }

    /// Возвращает предел накопленного send-buffer одного клиента.
    pub const fn permitted_send_bytes(&self) -> i32 {
        self.permitted_send_bytes
    }
}

impl Default for CServer {
    fn default() -> Self {
        Self::new(0)
    }
}

fn parse_decimal_u16(token: &[u8]) -> Option<u16> {
    if token.is_empty() {
        return None;
    }
    token.iter().try_fold(0_u16, |value, byte| {
        let digit = byte.checked_sub(b'0')?;
        (digit <= 9)
            .then_some(())
            .and_then(|()| value.checked_mul(10)?.checked_add(u16::from(digit)))
    })
}

// BLOCKED_MISSING_FACT: при маркере `#` без полной пары IP/u16 старый
// formatted extraction продолжал конструировать Address из частично
// записанного stack-объекта (Auth RVA 0x000110D0):
// `stream >> ip; stream >> port; allowed_list.push_back(Address(ip, port));`.
// Наблюдаемый результат malformed-файла не доказан; безопасный Rust сохраняет
// уже полные записи и прекращает разбор, не объявляя это поведением оригинала.
// То же относится к знаку, переполнению и иной неканонической форме u16, а
// также к numeric bool вне подтверждённых `0`/`1`.

// BLOCKED_MISSING_FACT: component callbacks отсутствующей map identity
// типизированы, но их конкретные эффекты ещё принадлежат будущим
// Billing/Login/Game/World owners. Общий reducer не предоставляет default no-op
// и заставляет каждое направление закрыть свою достижимость либо семантику.
