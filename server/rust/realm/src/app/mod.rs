//! Композиция и сетевой край Realm: сообщения направлений, слушающие порты и
//! места диспетчеризации объединённого Realm-процесса по [карте владельцев].
//! Старого процессного `realmserver` ещё нет; компонент растёт по мере
//! переноса владельцев World/Auth/Login/Billing/Misc.
//!
//! [карте владельцев]: ../../../../docs/architecture/realm-and-zone.md

pub mod misc_client;
pub mod misc_message;
pub mod world_client;
pub mod world_message;
pub mod world_server;
pub mod world_server_client;
