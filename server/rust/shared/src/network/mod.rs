//! Сеть: framing, RLE и буферы сообщений направлений.
//! Сессия персонажа, разбор смысла команды и момент отправки — у владельца роли.

mod basemessage; // CBaseMessage: базовый 16-байтовый wire-буфер и RLE-кодек.
mod clients; // общий исходящий TCP-клиент направлений.
mod msgqueue; // потокобезопасная очередь сообщений.
mod mysocket; // общие наблюдаемые факты исходного CMySocket (endpoint, IP-слово).
mod serverclient; // состояние принятого TCP-соединения CServerClient.
mod servers; // CServer: владелец входящих TCP-соединений.
mod socketcommands; // потокобезопасная очередь сокетных команд.
mod transport; // Linux-владелец заменённой WinSock/IOCP транспортной механики.

pub use basemessage::{decode_rle, encode_rle, CBaseMessage, RleDecodeError, RleEncodeError}; // буфер CBaseMessage и RLE-кодек.
pub use clients::{
    connect, ClientConnectError, ClientSendError, ClientSendQueue, FlushOutcome, TransferCounter,
    CONNECT_TIMEOUT, INITIAL_RECEIVE_CAPACITY, MAX_RECEIVE_CHUNK,
}; // исходящий TCP-клиент: connect, отправка и счётчики передачи.
pub use msgqueue::CMsgQueue; // потокобезопасная очередь сообщений.
pub use mysocket::{
    legacy_bind_endpoint, legacy_inet_addr, legacy_ipv4_word, SocketIdAllocator, DEFAULT_IP,
    DEFAULT_PORT, DEFAULT_SOCKET_TYPE,
}; // socket-константы и legacy-представления endpoint/IP.
pub use serverclient::{
    AddSendDataOutcome, CServerClient, ServerClientMessageContext, ServerClientSizeError,
    ServerSendBatch, ServerSendCompletion, DEFAULT_PERMITTED_SEND_BYTES,
}; // состояние принятого соединения: очередь отправки и её исходы.
pub use servers::{
    AcceptStart, AdmissionOutcome, AllowedAddress, CServer, ComponentReceiveErrorAction,
    ServerCommandHandle, ServerComponentCallbacks, ServerHostError, ServerIoAction,
    ServerIoCompletion, ServerSnapshot, ServerSnapshotError, ServerSocketCommand,
    ACCEPT_AT_CAPACITY_DELAY, ACCEPT_THREAD_DELAY, NET_THREAD_DELAY,
}; // принимающий сервер: callbacks, снимки и cadence-константы.
pub use socketcommands::CSocketCommands; // очередь сокетных команд.
pub use transport::{
    bind_tcp_ipv4, connect_tcp_ipv4, listen_tcp_ipv4, read_client_tcp_chunk, read_server_tcp_chunk,
    shutdown_tcp, try_write_tcp, write_once_tcp, CLIENT_RECEIVE_CHUNK, SERVER_RECEIVE_CHUNK,
}; // TCP-примитивы заменённой WinSock/IOCP механики и размеры приёмных чанков.
