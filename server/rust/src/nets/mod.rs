//! Общие сетевые владельцы исходного каталога `nets`.

#[allow(
    dead_code,
    reason = "базовый wire-буфер подключён до восстановления конкретных CMessage"
)]
pub(crate) mod basemessage;

#[allow(
    dead_code,
    reason = "исходящий client-owner подключён до восстановления конкретных net-направлений"
)]
pub(crate) mod clients;

#[allow(
    dead_code,
    reason = "GameServer message-owner подключён до полного around/session send-path"
)]
pub(crate) mod netserver;

#[allow(
    dead_code,
    reason = "World message-owner подключён до component receive и доменных обработчиков"
)]
pub(crate) mod networld;

#[allow(
    dead_code,
    reason = "очередь подключена к сборке до восстановления конкретного типа сообщения"
)]
pub(crate) mod msgqueue;

#[allow(
    dead_code,
    reason = "общие socket-факты подключены до восстановления clients и servers"
)]
pub(crate) mod mysocket;

#[allow(
    dead_code,
    reason = "общий server-owner подключён до component callbacks и message parsers"
)]
pub(crate) mod servers;
