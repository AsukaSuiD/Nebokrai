//! Сеть: framing, RLE и буферы сообщений направлений.
//! Сессия персонажа, разбор смысла команды и момент отправки — у владельца роли.

mod basemessage;
mod clients;
mod msgqueue;
mod mysocket;
mod serverclient;
mod socketcommands;
mod transport;

pub use basemessage::{decode_rle, encode_rle, CBaseMessage, RleDecodeError, RleEncodeError};
pub use clients::{
    connect, ClientConnectError, ClientSendError, ClientSendQueue, FlushOutcome, TransferCounter,
    CONNECT_TIMEOUT, INITIAL_RECEIVE_CAPACITY, MAX_RECEIVE_CHUNK,
};
pub use msgqueue::CMsgQueue;
pub use mysocket::{
    legacy_bind_endpoint, legacy_inet_addr, legacy_ipv4_word, SocketIdAllocator, DEFAULT_IP,
    DEFAULT_PORT, DEFAULT_SOCKET_TYPE,
};
pub use serverclient::{
    AddSendDataOutcome, CServerClient, ServerClientMessageContext, ServerClientSizeError,
    ServerSendBatch, ServerSendCompletion, DEFAULT_PERMITTED_SEND_BYTES,
};
pub use socketcommands::CSocketCommands;
pub use transport::{
    bind_tcp_ipv4, connect_tcp_ipv4, listen_tcp_ipv4, read_client_tcp_chunk, read_server_tcp_chunk,
    shutdown_tcp, try_write_tcp, write_once_tcp, CLIENT_RECEIVE_CHUNK, SERVER_RECEIVE_CHUNK,
};
