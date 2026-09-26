//! Weather data-контракты `CServerRegion`: startup-таблица сегментов/опций
//! и результат секундного weather tick-а; сами tick/change-действия —
//! periodic fragment `CServerRegion::AI` и exact `ChangeWeather`. Исходный
//! владелец — `appserver/serverregion.h/.cpp`; точная пара `gameserver.exe`
//! + `GameServer/GameServer.pdb`.
//! `ChangeWeather` (`0x00080210`) собирает единственную запись с нулевым
//! цветом тумана, заменяет ей `m_vectorWeather +0x200` и вызывает
//! `SendWeatherInfo(player=0)` (`0x0007C610`), сериализующий `0xBF507`;
//! публикацией настоящих сеансов владеет достигнутый `CGame` caller.

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

/// Тело секундного weather fragment-а `CServerRegion::AI`; caller уже применил
/// общий one-second gate monster/weather prefix-а. Counter растёт wrapping;
/// при достижении `lTime` текущего сегмента сбрасывается, segment циклически
/// переходит к следующему, из его опций первой подходит первая cumulative
/// RNG-option, и `m_vectorWeather` заменяется её записями.
pub fn advance_weather_tick(
    setup: &[ServerRegionWeatherTime],
    current: &mut Vec<ServerRegionWeather>,
    segment: &mut usize,
    count: &mut i32,
    mut random_below: impl FnMut(i32) -> i32,
) -> ServerRegionWeatherTick {
    *count = count.wrapping_add(1);
    let Some(current_time) = setup.get(*segment) else {
        return ServerRegionWeatherTick::Waiting {
            segment: *segment,
            count: *count,
        };
    };
    if current_time.time > *count {
        return ServerRegionWeatherTick::Waiting {
            segment: *segment,
            count: *count,
        };
    }

    *count = 0;
    *segment = segment.wrapping_add(1);
    if *segment >= setup.len() {
        *segment = 0;
    }
    let advanced = *segment;
    let options = &setup[advanced].options;
    if options.is_empty() {
        current.clear();
        return ServerRegionWeatherTick::Changed {
            segment: advanced,
            weather: Vec::new(),
        };
    }

    let odds = random_below(100);
    let Some(option) = options.iter().find(|option| odds < option.cumulative_odds) else {
        return ServerRegionWeatherTick::AdvancedWithoutSelection { segment: advanced };
    };
    current.clone_from(&option.weather);
    ServerRegionWeatherTick::Changed {
        segment: advanced,
        weather: current.clone(),
    }
}

/// Тело exact `CServerRegion::ChangeWeather` (`0x00080210`) без publish-хвоста:
/// единственная запись с нулевым цветом тумана заменяет `m_vectorWeather`.
pub fn change_weather(current: &mut Vec<ServerRegionWeather>, weather_index: i32) {
    current.clear();
    current.push(ServerRegionWeather {
        weather_index,
        fog_color: 0,
    });
}
