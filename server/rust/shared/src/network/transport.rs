//! Linux-владелец заменённой WinSock/IOCP транспортной механики.
//!
//! Статус владельца: `IMPLEMENTED` для создания и IPv4-bind TCP socket;
//! listen, connect, server/client read, server write и shutdown подключены вместе с
//! фактическими `CServer/CClient`; cadence и жизненный цикл задач остаются
//! runtime-владельцам сервисов.
//!
//! Tokio `1.53.1` выбран вместо ручного reactor: `TcpSocket` сохраняет отдельные
//! операции create, bind, listen и connect, а `TcpStream` предоставляет
//! readiness и неблокирующие try-read/write. Используется `TcpSocket::bind`, а
//! не `TcpListener::bind`, потому что последний включает `SO_REUSEADDR` на Unix,
//! тогда как исходный `CMySocket::WSACreate + Bind` этого не делал.
//!
//! Tokio гарантирует safe readiness/read/write и владение socket. `socket2`
//! даёт safe borrowed `SockRef::shutdown`, чтобы close flag разбудил read-loop
//! без raw-fd и `unsafe`; окончательное закрытие по-прежнему делает `Drop`.
//! Она не определяет legacy wire, размеры чтения, очереди, таймаут подключения,
//! повтор частичной отправки или реакцию сервиса на ошибку: эти инварианты
//! остаются у восстановленных владельцев.

//! Экземпляр соединения принадлежит владельцу роли; Shared несёт только
//! типы сокетов и атомарные операции чтения/записи/создания.

use std::io;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4};

use socket2::SockRef;

use tokio::net::{TcpListener, TcpSocket, TcpStream};

use super::mysocket::legacy_bind_endpoint;

/// Размер одного старого IOCP receive-buffer для принятого соединения.
pub const SERVER_RECEIVE_CHUNK: usize = 0x2000;
/// Размер одного старого receive-вызова исходящего `CClient`.
pub const CLIENT_RECEIVE_CHUNK: usize = 0x2800;

/// Создаёт IPv4 TCP socket и выполняет совместимый bind без начала listen.
pub fn bind_tcp_ipv4(address: Option<Ipv4Addr>, port: u32) -> io::Result<TcpSocket> {
    let socket = TcpSocket::new_v4()?;
    socket.bind(legacy_bind_endpoint(address, port).into())?;
    Ok(socket)
}

/// Переводит уже bind-нутый TCP socket в listen с исходным 32-битным backlog.
pub fn listen_tcp_ipv4(socket: TcpSocket, backlog: i32) -> io::Result<TcpListener> {
    socket.listen(backlog as u32)
}

/// Подключает ранее созданный TCP socket к уже разрешённому IPv4 endpoint.
pub async fn connect_tcp_ipv4(socket: TcpSocket, remote: SocketAddrV4) -> io::Result<TcpStream> {
    socket.connect(remote.into()).await
}

/// Выполняет одну неблокирующую запись без скрытого повтора либо ожидания.
pub fn try_write_tcp(stream: &TcpStream, buffer: &[u8]) -> io::Result<usize> {
    stream.try_write(buffer)
}

/// Ждёт readiness и выполняет не более одного успешного системного write.
///
/// `WouldBlock` после readiness считается ложным срабатыванием и повторяет
/// ожидание; частичный успешный результат возвращается вызывающему буквально.
pub async fn write_once_tcp(stream: &TcpStream, buffer: &[u8]) -> io::Result<usize> {
    loop {
        stream.writable().await?;
        match stream.try_write(buffer) {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
            result => return result,
        }
    }
}

/// Читает не более одного исходного server receive-block после readiness.
///
/// `WouldBlock` после readiness является ложным срабатыванием; нулевой размер
/// возвращается буквально и означает закрытие peer для server worker.
pub async fn read_server_tcp_chunk(stream: &TcpStream) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0; SERVER_RECEIVE_CHUNK];
    loop {
        stream.readable().await?;
        match stream.try_read(&mut buffer) {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
            Ok(received) => {
                buffer.truncate(received);
                return Ok(buffer);
            }
            Err(error) => return Err(error),
        }
    }
}

/// Читает не более одного исходного client receive-block после readiness.
///
/// Нулевой размер возвращается буквально: конкретный `net*`-владелец должен
/// преобразовать EOF в собственный `OnClose -> HandleClose` контракт.
pub async fn read_client_tcp_chunk(stream: &TcpStream) -> io::Result<Vec<u8>> {
    let mut buffer = vec![0; CLIENT_RECEIVE_CHUNK];
    loop {
        stream.readable().await?;
        match stream.try_read(&mut buffer) {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
            Ok(received) => {
                buffer.truncate(received);
                return Ok(buffer);
            }
            Err(error) => return Err(error),
        }
    }
}

/// Будит все операции общего TCP-соединения после исходного close flag.
///
/// `SockRef` не забирает владение Tokio socket и вызывает системный shutdown
/// без ручного raw-fd/unsafe. Фактический `Drop` выполняется после component
/// `OnClose` и удаления соединения следующим command snapshot.
pub fn shutdown_tcp(stream: &TcpStream) -> io::Result<()> {
    SockRef::from(stream).shutdown(Shutdown::Both)
}
