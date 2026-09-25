//! Weather data-контракты `CServerRegion` исторического GameServer (порция 1):
//! startup-таблица сегментов/опций и результат секундного weather tick-а.
//! Исходный владелец — `appserver/serverregion.h/.cpp`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionWeather {
    pub weather_index: i32,
    pub fog_color: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionWeatherOption {
    pub cumulative_odds: i32,
    pub weather: Vec<ServerRegionWeather>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerRegionWeatherTime {
    pub time: i32,
    pub options: Vec<ServerRegionWeatherOption>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerRegionWeatherTick {
    Waiting {
        segment: usize,
        count: i32,
    },
    AdvancedWithoutSelection {
        segment: usize,
    },
    Changed {
        segment: usize,
        weather: Vec<ServerRegionWeather>,
    },
}
