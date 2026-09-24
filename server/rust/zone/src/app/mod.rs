//! Композиция и сетевой край Zone: сообщение направления GameServer и
//! принятые/исходящие владельцы соединений Game↔World↔Billing по
//! [карте владельцев]. Старого процессного `zoneserver` ещё нет; компонент
//! растёт по мере переноса владельца Game.
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

pub mod game_client;
pub mod game_message;
pub mod game_server;
pub mod game_server_client;
