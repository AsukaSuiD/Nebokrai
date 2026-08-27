//! Общий владелец входящих TCP-соединений `CServer`, восстановленный из
//! `nets/servers.cpp` и `nets/servers.h`.
//!
//! Статус владельца: `IMPLEMENTED` для очереди socket-команд, listener,
//! admission, реестров socket/map identity, временной блокировки IPv4,
//! таймера первого сообщения, общих счётчиков и полного command snapshot
//! `DoNetThreadFunc`. Конкретные virtual callbacks передаются узким trait и не
//! смешивают общий server-owner с различающимися `CMessage`.
//!
//! Точные варианты корпуса:
//! - Auth: `AuthServer/authserver.exe + AuthServer/authserver.pdb`, SHA-256
//!   `AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15` /
//!   `26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5`;
//! - Billing: `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`,
//!   `FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19` /
//!   `F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21`;
//! - Login: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`,
//!   `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876` /
//!   `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`;
//! - Game: `GameServer/gameserver.exe + GameServer/GameServer.pdb`,
//!   `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E` /
//!   `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`;
//! - World: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//!   `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1` /
//!   `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//!
//! Исходные пути PDB: `h:\fengyun\fy_russia\src\nets\servers.{cpp,h}`,
//! `d:\complite_version\fengyun_russia\trunk\nets\servers.{cpp,h}` и
//! `e:\svn\fengyun_russia_dev\nets\servers.{cpp,h}`.
//!
//! Существенные RVA по порядку Auth / Billing / Login / Game / World:
//! - `Host`: `0x00012680` / `0x0000C690` / `0x0006A780` / `0x00018D80` /
//!   `0x00028160`;
//! - `OnAccept`: `0x00010A10` / `0x0000AA00` / `0x00068DB0` / `0x000176F0` /
//!   `0x00026740`;
//! - `AddAClient`: `0x00011050` / `0x0000B060` / `0x00069330` /
//!   `0x00017900` / `0x00026DA0`;
//! - `DelOneClient`: `0x000105C0` / `0x0000A3D0` / `0x00068490` /
//!   `0x000172C0` / `0x00026020`;
//! - `DoNetThreadFunc`: `0x00011C80` / `0x0000BC90` / `0x00069AE0` /
//!   `0x000180C0` / `0x00027740`.
//! - `LoadAllowedClient`: Auth `0x000110D0`, Billing `0x0000B0E0`; оба
//!   emitted-варианта совпадают.
//!
//! Все варианты передавали изменения одному net-thread через
//! `CSocketCommands`; только очередь была concurrent, а `std::map`-реестры
//! изменялись одним владельцем. `ServerCommandHandle` и
//! `CSocketCommands<ServerSocketCommand>` сохраняют эту границу, а
//! `BTreeMap` — детерминированный порядок исходного `std::map`. Owned `Vec` и
//! `Arc<TcpStream>` заменяют ручные копии, указатели и `operator_delete`.
//!
//! `DoNetThreadFunc` во всех пяти вариантах атомарно снимал текущую очередь,
//! последовательно применял команды, запускал не более одной новой send-
//! операции на client за проход, публиковал timeout-QUIT для следующего
//! snapshot и только затем закрывал socket с выставленным close flag. Rust
//! сохраняет этот порядок в `process_command_snapshot`. Ошибка component
//! receive не прерывает оставшийся snapshot: она возвращается отдельным
//! элементом результата, как старый virtual `OnReceive` возвращал управление
//! общему циклу.
//! Component может отдельно вернуть доказанное действие `ForbidAndQuit` для
//! receive-ошибки. Общий owner выполняет его сразу после component diagnostic
//! и до следующей команды snapshot; opcode и причина ошибки при этом остаются
//! неизвестны общему слою.
//!
//! `DoWorkerThreadFunc` после receive completion копировал `0x2000`-блок в
//! owned команду и немедленно перевыставлял один read; после send completion
//! публиковал `SENDEND`, даже если системный completion был частичным. Один
//! `ServerIoAction::Receive` сохраняет последовательный outstanding read, а
//! owned send-action — уже доказанное отбрасывание partial-хвоста. Tokio
//! readiness заменяет IOCP, а `socket2::SockRef::shutdown` — `closesocket` на
//! стадии close flag, не забирая владение socket у read-задачи.
//! Если новый send и close встречаются в одном snapshot, действие выполняет
//! один write до shutdown: это сохраняет исходный порядок `WSASend -> close`.
//! Точная timing-семантика pending Windows send против Linux readiness локально
//! отмечена `BLOCKED_MISSING_FACT` и не выдаётся за полное совпадение.
//!
//! Набор producer-вариантов различался: Auth не emitted `SendAll`, Login
//! добавлял quit/set по строковой map identity, Game — quit/get по числовой,
//! Billing и World использовали set числовой identity. Общий owner хранит
//! доказанное объединение команд, но компонент может вызывать только свой
//! подтверждённый набор.
//!
//! `Host` во всех пяти вариантах создавал/bind-ил TCP socket, создавал IOCP и
//! три вида Windows threads, затем вызывал `listen` с default backlog
//! `INT_MAX`. Rust `host` выполняет доказанные bind/listen через Tokio;
//! отдельные IOCP/thread handle не получают пустых аналогов. Runtime запускает
//! возвращённые I/O actions и хранит их task lifecycle у конкретного сервиса.
//!
//! Admission сначала сравнивал signed client count с default `100`, затем
//! принимал IPv4 socket. Опциональный allow-list сравнивал canonical
//! `inet_ntoa` и port; запись с port `0` совпадала только по IP. Опциональный
//! forbid-map использовал wrapping `timeGetTime`. Принятое соединение получало
//! process-local socket ID, собственный `CServerClient` и команду `ADD`.
//! `LoadAllowedClient` очищал список до попытки открытия, отбрасывал первый
//! label, читал numeric bool, а затем через общий `ReadTo("#")` повторял пары
//! `IP-string/u16`. Успешно открытый файл всегда давал success, даже без
//! маркеров; ошибка открытия оставляла прежний enable-флаг с пустым списком.
//! Байтовый parser на стандартном разделении по ASCII whitespace сохраняет
//! подтверждённую грамматику оригинального `allowed_ls.ini` без требования
//! UTF-8; malformed numeric формы локализованы отдельно.
//!
//! Унаследованные от `CMySocket` dotted IPv4 и x86 DWORD сохраняются как
//! owned bytes/u32. Завершающий C NUL заменён длиной `Vec`; hostname resolution
//! остаётся конкретному service-owner, потому что его порядок различается.
//!
//! Сохранены странности порядка. `GetSocketIDByMapStr` возвращал `1`, а не `0`,
//! если ключ отсутствовал. `AddAClient` увеличивал счётчик до проверки
//! дубликата, разрушал старый client и только затем вставлял новый, не
//! компенсируя счётчик. Обычный `DelOneClient` уменьшал счётчик до `OnClose` и
//! удаления из map, а переполнение `SENDALL` сначала стирало client, вызывало
//! `OnClose`, разрушало его и лишь потом уменьшало счётчик. Эти порядки не
//! объединены в один удобный Rust-путь.
//!
//! Конструктор задавал: accept sleep `100 ms`, net sleep `1 ms`, backlog
//! `INT_MAX`, send interval нового соединения `8000 ms`, receive-rate limit
//! `0x186A0000`, max in-flight send `1`, max clients `100`, per-client send
//! buffer `0xC800`; `m_bCheck`, `m_bCheckMsgCon` и allow-list checking изначально
//! выключены. `m_dwMaxMsgLen` конструктор не назначал, поэтому до поздней записи
//! service-owner он представлен `None`. STL/CRT/SEH, vtable, WinSock/IOCP
//! allocation и thread-handle cleanup удалены как технический шум.
//! Auth `InitNetServer_Auth` после успешного `Host` менял
//! `m_lMaxBlockConnetNum` на `10` и `m_lSendInterTime` на config-значение.
//! Rust сохраняет этот порядок: первый setter уже не может изменить backlog
//! существующего listener, второй меняет таймаут первого сообщения.

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};

use crate::nets::clients::TransferCounter;
use crate::nets::mysocket::{DEFAULT_IP, DEFAULT_SOCKET_TYPE, SocketIdAllocator, legacy_ipv4_word};
use crate::nets::serverclient::{
    AddSendDataOutcome, CServerClient, DEFAULT_PERMITTED_SEND_BYTES, ServerClientSizeError,
    ServerSendBatch, ServerSendCompletion,
};
use crate::nets::socketcommands::CSocketCommands;
use crate::public::readwrite::read_to;
use crate::transport::{bind_tcp_ipv4, listen_tcp_ipv4, read_server_tcp_chunk, shutdown_tcp};

/// Пауза исходного accept-thread при достижении лимита соединений.
pub(crate) const ACCEPT_AT_CAPACITY_DELAY: Duration = Duration::from_secs(1);
/// Исходная пауза между итерациями accept-thread.
pub(crate) const ACCEPT_THREAD_DELAY: Duration = Duration::from_millis(100);
/// Исходная пауза между снимками общей очереди команд.
pub(crate) const NET_THREAD_DELAY: Duration = Duration::from_millis(1);

const DEFAULT_MAX_CLIENTS: i32 = 100;
const DEFAULT_MAX_BACKLOG: i32 = i32::MAX;
const DEFAULT_NEW_ACCEPT_TIMEOUT_MS: i32 = 8000;
const DEFAULT_RECEIVE_RATE_LIMIT: i32 = 0x186A_0000;
const DEFAULT_MAX_IN_FLIGHT_SENDS: i32 = 1;

/// Адрес из исходного allow-list без предположения о кодировке файла.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AllowedAddress {
    ip: Vec<u8>,
    port: u16,
}

impl AllowedAddress {
    /// Сохраняет буквальную строку IP и host-order port исходной записи.
    pub(crate) fn new(ip: &[u8], port: u16) -> Self {
        Self {
            ip: ip.to_vec(),
            port,
        }
    }

    /// Проверяет исходное правило: port `0` означает совпадение только по IP.
    pub(crate) fn matches(&self, peer_ip: &[u8], peer_port: u16) -> bool {
        self.ip == peer_ip && (self.port == 0 || self.port == peer_port)
    }
}

/// Принятое TCP-соединение и его общий `CServerClient`-owner.
pub(crate) struct AcceptedServerClient {
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
    pub(crate) fn stream(&self) -> Arc<TcpStream> {
        Arc::clone(&self.stream)
    }

    /// Возвращает общее состояние принятого соединения.
    pub(crate) const fn state(&self) -> &CServerClient {
        &self.state
    }

    /// Возвращает общее состояние для единственного net-owner.
    pub(crate) const fn state_mut(&mut self) -> &mut CServerClient {
        &mut self.state
    }
}

/// Владеющая форма доказанных вариантов старого `tagSocketOper`.
pub(crate) enum ServerSocketCommand {
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
pub(crate) struct ServerCommandHandle {
    commands: Arc<CSocketCommands<ServerSocketCommand>>,
}

impl ServerCommandHandle {
    fn new() -> Self {
        Self {
            commands: Arc::new(CSocketCommands::new()),
        }
    }

    /// Копирует непустой payload в команду отправки одному socket ID.
    pub(crate) fn send_by_socket_id(&self, socket_id: i32, buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendToSocket {
            socket_id,
            buffer,
        })
    }

    /// Копирует непустой payload в команду отправки по числовой identity.
    pub(crate) fn send_by_map_id(&self, map_id: i32, buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendToMapId {
            map_id,
            buffer,
        })
    }

    /// Копирует непустой payload и строковую identity в одну owned-команду.
    pub(crate) fn send_by_map_name(&self, map_name: &[u8], buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendToMapName {
            map_name: map_name.to_vec(),
            buffer,
        })
    }

    /// Копирует непустой payload в broadcast-команду.
    pub(crate) fn send_all(&self, buffer: &[u8]) -> i32 {
        self.push_nonempty(buffer, |buffer| ServerSocketCommand::SendAll { buffer })
    }

    /// Ставит исходную безусловно успешную команду close по socket ID.
    pub(crate) fn quit_by_socket_id(&self, socket_id: i32) -> i32 {
        self.commands
            .push_back(ServerSocketCommand::QuitBySocketId { socket_id });
        1
    }

    /// Ставит команду close по числовой identity.
    pub(crate) fn quit_by_map_id(&self, map_id: i32) -> i32 {
        self.commands
            .push_back(ServerSocketCommand::QuitByMapId { map_id });
        1
    }

    /// Копирует строковую identity в команду close.
    pub(crate) fn quit_by_map_name(&self, map_name: &[u8]) -> i32 {
        self.commands.push_back(ServerSocketCommand::QuitByMapName {
            map_name: map_name.to_vec(),
        });
        1
    }

    /// Ставит команду присоединения числовой identity.
    pub(crate) fn set_client_map_id(&self, socket_id: i32, map_id: i32) -> i32 {
        self.commands
            .push_back(ServerSocketCommand::SetMapId { socket_id, map_id });
        1
    }

    /// Копирует строковую identity в команду присоединения.
    pub(crate) fn set_client_map_name(&self, socket_id: i32, map_name: &[u8]) -> i32 {
        self.commands.push_back(ServerSocketCommand::SetMapName {
            socket_id,
            map_name: map_name.to_vec(),
        });
        1
    }

    /// Ставит команду закрытия всех соединений.
    pub(crate) fn quit_all(&self) -> i32 {
        self.commands.push_back(ServerSocketCommand::QuitAll);
        1
    }

    /// Возвращает число ожидающих команд в исходном signed типе.
    pub(crate) fn pending(&self) -> i32 {
        self.commands.get_size()
    }

    /// Публикует один owned receive-completion из Linux transport worker.
    pub(crate) fn publish_receive(&self, socket_id: i32, buffer: Vec<u8>) {
        self.commands
            .push_back(ServerSocketCommand::Receive { socket_id, buffer });
    }

    /// Публикует закрытие либо transport-ошибку как исходный delete-command.
    pub(crate) fn publish_delete(&self, socket_id: i32) {
        self.commands
            .push_back(ServerSocketCommand::DeleteBySocketId { socket_id });
    }

    /// Публикует успешное завершение одной send-operation.
    pub(crate) fn publish_send_end(&self, socket_id: i32) {
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
pub(crate) trait ServerComponentCallbacks {
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
pub(crate) enum ComponentReceiveErrorAction {
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
pub(crate) enum ServerSnapshotError<ComponentError> {
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
pub(crate) enum ServerIoAction {
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
pub(crate) enum ServerIoCompletion {
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
        /// Completion с потерянным partial-хвостом либо transport-ошибка.
        result: io::Result<ServerSendCompletion>,
    },
}

impl ServerIoAction {
    /// Исполняет owned action и публикует его результат в ту же command queue.
    pub(crate) async fn run(self, commands: ServerCommandHandle) -> ServerIoCompletion {
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
                let result = batch.write_once(&stream).await;
                if shutdown_after {
                    let _ = shutdown_tcp(&stream);
                }
                if result.is_ok() {
                    commands.publish_send_end(socket_id);
                } else {
                    // BLOCKED_MISSING_FACT: Linux write_once объединяет два
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
pub(crate) struct ServerSnapshot<ComponentError> {
    io_actions: Vec<ServerIoAction>,
    processed_commands: i32,
    errors: Vec<ServerSnapshotError<ComponentError>>,
}

impl<ComponentError> ServerSnapshot<ComponentError> {
    /// Передаёт runtime I/O actions и локальные ошибки component-путей.
    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<ServerIoAction>,
        Vec<ServerSnapshotError<ComponentError>>,
    ) {
        (self.io_actions, self.errors)
    }

    /// Возвращает число команд исходного snapshot до новых completion-событий.
    pub(crate) const fn processed_commands(&self) -> i32 {
        self.processed_commands
    }
}

/// Ошибка создания Linux listener из доказанных параметров `Host`.
#[derive(Debug, Error)]
pub(crate) enum ServerHostError {
    /// Все найденные call sites передают `SOCK_STREAM`; другой тип не доказан.
    #[error("для CServer не восстановлен socket type {0}")]
    UnsupportedSocketType(i32),
    /// Transport не смог bind-нуть либо перевести socket в listen.
    #[error("не удалось создать TCP listener: {0}")]
    Io(#[source] io::Error),
}

/// Результат попытки начать единственный исходный blocking accept.
pub(crate) enum AcceptStart {
    /// `Host` ещё не создал listener.
    NotListening,
    /// Signed client count достиг исходного ограничения.
    AtCapacity,
    /// Listener может выполнить один accept.
    Pending(ServerPendingAccept),
}

/// Одна ожидающая accept-operation без Windows thread/WSAAccept plumbing.
pub(crate) struct ServerPendingAccept {
    listener: Arc<TcpListener>,
}

impl ServerPendingAccept {
    /// Принимает ровно одно IPv4 TCP-соединение.
    pub(crate) async fn accept(self) -> io::Result<(TcpStream, SocketAddrV4)> {
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
pub(crate) enum AdmissionOutcome {
    /// Лимит был достигнут между началом и завершением accept.
    AtCapacity,
    /// Peer отсутствует в включённом allow-list.
    AddressRejected,
    /// Peer IPv4 ещё находится во временном forbid-map.
    TemporarilyForbidden,
    /// Owned `ADD` поставлен единственному net-owner.
    Queued { socket_id: i32 },
}

/// Общий state-owner одного исторического `CServer`.
pub(crate) struct CServer {
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
    pub(crate) fn new(now_ms: u32) -> Self {
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
    pub(crate) fn host(
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
    pub(crate) fn command_handle(&self) -> ServerCommandHandle {
        self.commands.clone()
    }

    /// Начинает один accept либо буквально сообщает исходную причину ожидания.
    pub(crate) fn begin_accept(&self) -> AcceptStart {
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
    pub(crate) fn queue_accepted(
        &mut self,
        stream: TcpStream,
        peer: SocketAddrV4,
        now_ms: u32,
    ) -> AdmissionOutcome {
        self.queue_accepted_with(stream, peer, now_ms, CServerClient::new)
    }

    /// Проверяет peer и вызывает доказанную component-замену virtual
    /// `CreateServerClient` перед постановкой `ADD`.
    pub(crate) fn queue_accepted_with(
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
    pub(crate) fn take_commands(&self) -> VecDeque<ServerSocketCommand> {
        self.commands.take_all()
    }

    /// Применяет ровно один атомарный snapshot старого `DoNetThreadFunc`.
    ///
    /// Новые completion-команды, опубликованные во время выполнения, остаются
    /// следующему проходу. Component и безопасные size-ошибки локализуются в
    /// результате и не уничтожают остальные уже снятые команды.
    pub(crate) fn process_command_snapshot<Callbacks>(
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
                if client.state().io_operations() >= self.max_in_flight_sends {
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
            Ok(AddSendDataOutcome::LimitExceeded) => match removal {
                SendOverflowRemoval::DelOneClient => {
                    self.remove_client_with_callback(socket_id, callbacks);
                }
                SendOverflowRemoval::Broadcast => {
                    self.remove_broadcast_overflow_client(socket_id, callbacks);
                }
            },
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
    pub(crate) fn client(&self, socket_id: i32) -> Option<&AcceptedServerClient> {
        self.clients.get(&socket_id)
    }

    /// Возвращает соединение единственному изменяющему net-owner.
    pub(crate) fn client_mut(&mut self, socket_id: i32) -> Option<&mut AcceptedServerClient> {
        self.clients.get_mut(&socket_id)
    }

    /// Возвращает исходный signed счётчик зарегистрированных соединений.
    pub(crate) const fn client_count(&self) -> i32 {
        self.client_count
    }

    /// Сообщает, остались ли записи в исходном `m_Clients` map.
    ///
    /// Это намеренно не сравнение [`Self::client_count`] с нулём: сохранённая
    /// странность duplicate `ADD` могла рассинхронизировать signed счётчик и
    /// фактический размер map, а `ExitWorkerThread` ждал именно map size.
    pub(crate) fn has_clients(&self) -> bool {
        !self.clients.is_empty()
    }

    /// Ставит close flag найденному соединению; отсутствие остаётся no-op.
    pub(crate) fn mark_client_closing(&mut self, socket_id: i32) {
        if let Some(client) = self.clients.get_mut(&socket_id) {
            client.state_mut().mark_closing();
        }
    }

    /// Ставит close flag всем соединениям в порядке socket ID.
    pub(crate) fn mark_all_clients_closing(&mut self) {
        for client in self.clients.values_mut() {
            client.state_mut().mark_closing();
        }
    }

    /// Присоединяет числовую identity и обновляет её routing-index.
    pub(crate) fn assign_map_id(&mut self, socket_id: i32, map_id: i32) -> bool {
        let Some(client) = self.clients.get_mut(&socket_id) else {
            return false;
        };
        client.state_mut().set_map_id(map_id);
        self.map_id_socket_id.insert(map_id, socket_id);
        true
    }

    /// Присоединяет строковую identity и обновляет её routing-index.
    pub(crate) fn assign_map_name(&mut self, socket_id: i32, map_name: &[u8]) -> bool {
        let Some(client) = self.clients.get_mut(&socket_id) else {
            return false;
        };
        client.state_mut().set_map_name(map_name);
        self.map_name_socket_id.insert(map_name.to_vec(), socket_id);
        true
    }

    /// Возвращает socket ID числовой identity либо исходный ноль.
    pub(crate) fn get_socket_id_by_map_id(&self, map_id: i32) -> i32 {
        self.map_id_socket_id.get(&map_id).copied().unwrap_or(0)
    }

    /// Возвращает socket ID строковой identity либо исходную странность `1`.
    pub(crate) fn get_socket_id_by_map_name(&self, map_name: &[u8]) -> i32 {
        self.map_name_socket_id.get(map_name).copied().unwrap_or(1)
    }

    /// Удаляет числовой routing-index и возвращает прежний socket ID.
    pub(crate) fn remove_map_id(&mut self, map_id: i32) -> Option<i32> {
        self.map_id_socket_id.remove(&map_id)
    }

    /// Удаляет строковый routing-index и возвращает прежний socket ID.
    pub(crate) fn remove_map_name(&mut self, map_name: &[u8]) -> Option<i32> {
        self.map_name_socket_id.remove(map_name)
    }

    /// Снимает восьмисекундный контроль после первого receive этого socket ID.
    pub(crate) fn acknowledge_first_receive(&mut self, socket_id: i32) {
        self.new_accept_sockets.remove(&socket_id);
    }

    /// Ставит close-команды соединениям без первого пакета и удаляет их таймеры.
    pub(crate) fn expire_new_accepts(&mut self, now_ms: u32) -> Vec<i32> {
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
    pub(crate) fn load_allowed_clients(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
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
    pub(crate) fn set_local_identity(&mut self, ip: &[u8], ipv4_word: u32) {
        self.local_ip = ip.to_vec();
        self.local_ipv4_word = ipv4_word;
    }

    /// Возвращает dotted IPv4 без исходного завершающего NUL.
    pub(crate) fn local_ip(&self) -> &[u8] {
        &self.local_ip
    }

    /// Возвращает network bytes в исходном x86-представлении `unsigned long`.
    pub(crate) const fn local_ipv4_word(&self) -> u32 {
        self.local_ipv4_word
    }

    /// Проверяет один canonical peer против текущего allow-list.
    pub(crate) fn is_allowed_address(&self, peer_ip: &[u8], peer_port: u16) -> bool {
        self.allowed_addresses
            .iter()
            .any(|allowed| allowed.matches(peer_ip, peer_port))
    }

    /// Включает либо выключает исходный receive-rate/forbid механизм.
    pub(crate) fn configure_receive_guard(
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
    pub(crate) fn configure_send_limits(
        &mut self,
        max_in_flight_sends: i32,
        permitted_send_bytes: i32,
    ) {
        self.max_in_flight_sends = max_in_flight_sends;
        self.permitted_send_bytes = permitted_send_bytes;
    }

    /// Задаёт исходный signed предел одновременно принятых соединений.
    pub(crate) fn configure_max_clients(&mut self, max_clients: i32) {
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
    pub(crate) fn configure_transport_after_host(
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
    pub(crate) fn configure_accept_limits_after_host(
        &mut self,
        max_backlog: i32,
        new_accept_timeout_ms: i32,
    ) {
        self.max_backlog = max_backlog;
        self.new_accept_timeout_ms = new_accept_timeout_ms;
    }

    /// Запоминает wrapping tick временной блокировки IPv4.
    pub(crate) fn add_forbidden_ip(&mut self, peer_ipv4: u32, now_ms: u32) {
        self.forbidden_ips.insert(peer_ipv4, now_ms);
    }

    /// Проверяет временную блокировку и удаляет истёкшую запись.
    pub(crate) fn find_forbidden_ip(&mut self, peer_ipv4: u32, now_ms: u32) -> bool {
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
    pub(crate) fn add_send_size(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.send_counter.add(amount, now_ms)
    }

    /// Добавляет принятый объём к общему счётчику сервиса.
    pub(crate) fn add_receive_size(&mut self, amount: i32, now_ms: u32) -> i32 {
        self.receive_counter.add(amount, now_ms)
    }

    /// Возвращает лимит per-client receive-rate для component worker.
    pub(crate) const fn receive_rate_limit(&self) -> i32 {
        self.receive_rate_limit
    }

    /// Возвращает максимум одновременно незавершённых send одного клиента.
    pub(crate) const fn max_in_flight_sends(&self) -> i32 {
        self.max_in_flight_sends
    }

    /// Возвращает предел накопленного send-buffer одного клиента.
    pub(crate) const fn permitted_send_bytes(&self) -> i32 {
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
