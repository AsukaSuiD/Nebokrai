//! Композиция и сетевой край Zone: сообщение направления GameServer и
//! принятые/исходящие владельцы соединений Game↔World↔Billing по
//! [карте владельцев]. Старого процессного `zoneserver` ещё нет; компонент
//! растёт по мере переноса владельца Game.
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

pub mod game_client; // CMyNetClient: исходящий клиент направления GameServer.
pub mod game_message; // CMessage: сообщение направления GameServer.
pub mod game_server; // CMyNetServer: listener и network/send owner направления GameServer.
pub mod game_server_client; // состояние принятого игрового client-соединения.
