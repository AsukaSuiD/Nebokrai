//! Реестр обслуживающих Zone мира: назначения регионов, записи GameServer и
//! ping-индекс — primary state владельца `regions` (прежние transitional-поля
//! `CGame`). Формат-типы (`WorldRegionAssignment`, `WorldGameServerEntry`,
//! `WorldPingGameServerInfo`, `WorldRegionOwner`) перенесены из
//! `app/world_hub_entries`, `app/worldserver` и `app/world_runtime`; те же
//! пути сохранены re-export-ами для прежних consumers и старого пакета.
//!
//! `CGame` хранит только composition handle `region_registry` и делегирует
//! прежний pub facade. Поля публичны — Init/reload/runtime-оркестрация
//! мутирует их в прежнем доказанном порядке (silent-overwrite legacy dup, без
//! typed-исходов; прецедент `content::WorldContentCatalogs`). Опции
//! `Option<…>` несут различимые «uninitialized» исходы (NullRegionPointer,
//! PortUnavailable, Uninitialized) и не схлопываются. Основания layout —
//! docs/reconstruction/realm-services.md (§принятые Game-соединения,
//! §диспетчер servermessage).

use std::collections::BTreeMap;

use crate::regions::worldcityregion::CWorldCityRegion;
use crate::regions::worldcountrywarregion::WorldCountryWarRegion;
use crate::regions::worldregion::CWorldRegion;
use crate::regions::worldvillageregion::CWorldVillageRegion;

pub struct WorldRegionAssignment {
    pub region: Option<WorldRegionOwner>,
    pub game_server_index: u32,
    pub region_type: Option<i32>,
}

/// Минимальная действующая часть исходного `CGame::tagGameServer`.
///
/// Его оригинал-деструктор освобождал только `strIP`; `ip: Vec<u8>` освобождается
/// структурным Drop без отдельной инфраструктуры строки MSVC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldGameServerEntry {
    pub connected: bool,
    pub index: u32,
    pub ip: Vec<u8>,
    pub port: Option<u32>,
    pub received_player_data: Option<i32>,
}

/// Семантическая замена старого 36-байтового `tagPingGameServerInfo`.
///
/// Ветка `0x5FA0A` подтверждает `std::string strIP` и два signed `long`:
/// map ID из metadata сообщения и число игроков из payload. Rust-layout не
/// выдаётся за Windows ABI; owned bytes и `Vec` заменяют только STL-владение.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldPingGameServerInfo {
    pub ip: Vec<u8>,
    pub map_id: i32,
    pub player_count: i32,
}

pub enum WorldRegionOwner {
    Base(Box<CWorldRegion>),
    Village(Box<CWorldVillageRegion>),
    City(Box<CWorldCityRegion>),
    Country(Box<WorldCountryWarRegion>),
}

/// Действующий реестр обслуживающих Zone мира.
///
/// Constructor-ное состояние — пустые коллекции и остановленный ping;
/// наполнение выполняет Init/reload/runtime-оркестрация через pub-поля в
/// прежнем порядке.
pub struct WorldRegionRegistry {
    pub regions: BTreeMap<i32, WorldRegionAssignment>,
    pub game_servers: BTreeMap<u32, WorldGameServerEntry>,
    pub ping_game_servers: Vec<WorldPingGameServerInfo>,
    pub ping_in_progress: bool,
    pub last_ping_game_server_time_ms: u32,
}

impl WorldRegionRegistry {
    /// Часы берутся снаружи, повторяя ctor-семантику `legacy_tick_ms()` в `CGame::new`.
    pub fn new(last_ping_game_server_time_ms: u32) -> Self {
        Self {
            regions: BTreeMap::new(),
            game_servers: BTreeMap::new(),
            ping_game_servers: Vec::new(),
            ping_in_progress: false,
            last_ping_game_server_time_ms,
        }
    }
}
