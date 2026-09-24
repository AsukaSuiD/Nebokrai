//! Владелец `TimeToReturn` исторического WorldServer (`setup/TimeToReturn.ini`),
//! перенесённый в Realm content — владельца событийных календарных таблиц Realm.
//! Источник контракта — та же точная пара, что у [`crate::app::world_message`].
//!
//! Машинно подтверждённые точки (первая секция `.exe/Nworldserver.exe`,
//! дизассемблер этого прохода):
//! - `on_time` `0x472DC0`: map-lookup по event ID (call map `find` `0x472D50`),
//!   существует только если entry есть; access World region через accessor
//!   `0x4017A0` (g_Game) → `0x413AC0` (region по signed map ID); только на
//!   существующем map entry — `CMessage(0x7FA13)` через ctor `0x422DA0`,
//!   `Add` param `+0x10` (map ID) через writer `0x423C00`, `Add` param `+0x24`
//!   (buffer time) тем же, затем send через `SendToMapID` `0x4230B0` с
//!   `[entry+0x4]` (map id entry владельца) → точное совпадение двух signed
//!   long и маршрута message; weekly flag `(param+0x28)` gate-ит re-register
//!   через `push 7` в `0x4A36E0` (`set time`, то же 7 дней, что Rust
//!   `add_day(7)`);
//! - `reload` `0x473CF0`: обходит весь map в порядке iterator-start до end и
//!   отменяет event каждого param (`[param+0x2C] → 0x463F60 → 0x4637F0`) —
//!   точное «reload отдельно отменяет IDs в map-order перед load»;
//! - `initialize` `0x473DF0` делает тот же entry-load через `load` `0x4736F0`;
//!   имя ресурса `setup\TimeToReturn.ini` видно в `.rdata`-строке лога —
//!   буквальное совпадение owner во всех entry-путях.
//!
//! `initialize/load/reload/on_time` следуют контракту `worldserver.exe` и
//! `worldserver.pdb`: owner читает в map-order секции `#` weekly и `*`
//! absolute `setup/TimeToReturn.ini`, а callback отправляет `0x7FA13` с двумя
//! signed long: map ID и buffer time. `BTreeMap`, typed `TimerId` и typed
//! errors заменяют STL, singleton и неинициализированные event IDs без смены
//! wire, календарных границ или порядка side effects. Оригинал load-epilogue
//! возвращает `1` и после missing-file log; safe `Result` отделяет эту legacy
//! mapping от ошибок повреждённого содержимого.
//!
//! Оригинал `load` очищал map, но не отменял уже созданные events; эта странность
//! сохранена. `reload` отдельно отменяет IDs в map-order перед `load`. Невалидный
//! календарный input не получает старую неинициализированную запись: Rust
//! возвращает явную ошибку вместо чтения неинициализированных значений.

use std::collections::BTreeMap;

use nebokrai_shared::resources::read_to_marker as read_to;
use nebokrai_shared::runtime::{CTimer, TimerId};
use nebokrai_shared::values::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};

use crate::app::world_message::{CMessage, SendMessageError};

#[derive(Clone, Copy, Debug)]
pub struct TimeToReturnCallbacks<Callback> {
    pub on_time: Callback,
}

#[derive(Clone, Copy, Debug)]
pub struct TimeToReturnParam {
    pub map_id: u32,
    pub return_date: TagTime,
    pub buffer_time: u32,
    pub loop_weekly: bool,
    pub event_id: TimerId,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimeToReturnLoadReport {
    pub resource_found: bool,
    pub weekly_registered: u32,
    pub absolute_registered: u32,
    pub weekly_past: u32,
    pub absolute_past: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeToReturnLoadError {
    ResourceMissing,
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
    TimeParse(TagTimeParseBlock),
    Arithmetic(TagTimeArithmeticBlock),
}

impl From<TagTimeParseBlock> for TimeToReturnLoadError {
    fn from(value: TagTimeParseBlock) -> Self {
        Self::TimeParse(value)
    }
}

impl From<TagTimeArithmeticBlock> for TimeToReturnLoadError {
    fn from(value: TagTimeArithmeticBlock) -> Self {
        Self::Arithmetic(value)
    }
}

pub trait TimeToReturnContext {
    fn game_server_number_by_region_id(&mut self, region_id: i32) -> Option<i32>;
    fn send_to_map_id(&mut self, message: &CMessage, map_id: i32) -> Result<i32, SendMessageError>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum TimeToReturnFireDisposition {
    MissingEntry,
    RegionMissing,
    Sent {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimeToReturnFireReport {
    pub param_id: i32,
    pub disposition: TimeToReturnFireDisposition,
    pub rearmed_event_id: Option<TimerId>,
}

#[derive(Default)]
pub struct TimeToReturn {
    params: BTreeMap<i32, TimeToReturnParam>,
    next_id: i32,
}

impl TimeToReturn {
    pub const fn new() -> Self {
        Self {
            params: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn params(&self) -> &BTreeMap<i32, TimeToReturnParam> {
        &self.params
    }

    pub fn initialize<Callback: Copy>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: TimeToReturnCallbacks<Callback>,
    ) -> Result<TimeToReturnLoadReport, TimeToReturnLoadError> {
        self.load(source, now, timer, callbacks)
    }

    pub fn load<Callback: Copy>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: TimeToReturnCallbacks<Callback>,
    ) -> Result<TimeToReturnLoadReport, TimeToReturnLoadError> {
        self.params.clear();
        let Some(source) = source else {
            return Err(TimeToReturnLoadError::ResourceMissing);
        };
        let mut report = TimeToReturnLoadReport {
            resource_found: true,
            ..TimeToReturnLoadReport::default()
        };

        let mut weekly_tokens = tokens(source);
        while read_to(&mut weekly_tokens, b"#") {
            let map_id = next_i32(&mut weekly_tokens, "map_id")? as u32;
            let weekday = next_i32(&mut weekly_tokens, "weekday")?;
            let hour = next_i32(&mut weekly_tokens, "hour")? as u16;
            let minute = next_i32(&mut weekly_tokens, "minute")? as u16;
            let second = next_i32(&mut weekly_tokens, "second")? as u16;
            let buffer_time = next_i32(&mut weekly_tokens, "buffer_time")? as u32;
            let mut return_date = now;
            return_date.hour = hour;
            return_date.minute = minute;
            return_date.second = second;
            let difference = i32::from(now.day_of_week) - weekday;
            if difference < 0 {
                let _ = return_date.add_day(-difference)?;
            } else if difference > 0 {
                let _ = return_date.add_day(7 - difference)?;
            }
            if return_date.legacy_lt(now) {
                let _ = return_date.add_day(7)?;
            }
            if !return_date.legacy_gt(now) {
                report.weekly_past = report.weekly_past.wrapping_add(1);
                continue;
            }
            self.register(
                map_id,
                return_date,
                buffer_time,
                true,
                timer,
                callbacks.on_time,
            );
            report.weekly_registered = report.weekly_registered.wrapping_add(1);
        }

        let mut absolute_tokens = tokens(source);
        while read_to(&mut absolute_tokens, b"*") {
            let map_id = next_i32(&mut absolute_tokens, "map_id")? as u32;
            let time_text = next_token(&mut absolute_tokens, "return_date")?;
            let buffer_time = next_i32(&mut absolute_tokens, "buffer_time")? as u32;
            let return_date = TagTime::from_legacy_string(time_text)?;
            if !return_date.legacy_gt(now) {
                report.absolute_past = report.absolute_past.wrapping_add(1);
                continue;
            }
            self.register(
                map_id,
                return_date,
                buffer_time,
                false,
                timer,
                callbacks.on_time,
            );
            report.absolute_registered = report.absolute_registered.wrapping_add(1);
        }
        Ok(report)
    }

    pub fn reload<Callback: Copy>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: TimeToReturnCallbacks<Callback>,
    ) -> Result<TimeToReturnLoadReport, TimeToReturnLoadError> {
        for param in self.params.values() {
            let _ = timer.kill_time_event(param.event_id);
        }
        self.load(source, now, timer, callbacks)
    }

    pub fn on_time<Callback: Copy, Context: TimeToReturnContext + ?Sized>(
        &mut self,
        param_id: i32,
        timer: &mut CTimer<Callback>,
        callbacks: TimeToReturnCallbacks<Callback>,
        context: &mut Context,
    ) -> TimeToReturnFireReport {
        let Some(param) = self.params.get_mut(&param_id) else {
            return TimeToReturnFireReport {
                param_id,
                disposition: TimeToReturnFireDisposition::MissingEntry,
                rearmed_event_id: None,
            };
        };
        let disposition = match context.game_server_number_by_region_id(param.map_id as i32) {
            Some(map_id) => {
                let mut message = CMessage::new(0x7fa13);
                message.base_mut().add_long(param.map_id as i32);
                message.base_mut().add_long(param.buffer_time as i32);
                let wire = message.as_wire_bytes().to_vec();
                let delivery = context.send_to_map_id(&message, map_id);
                TimeToReturnFireDisposition::Sent {
                    map_id,
                    wire,
                    delivery,
                }
            }
            None => TimeToReturnFireDisposition::RegionMissing,
        };
        let rearmed_event_id = if param.loop_weekly {
            let _ = param.return_date.add_day(7);
            let event_id = timer.set_time_event(param.return_date, callbacks.on_time, param_id);
            param.event_id = event_id;
            Some(event_id)
        } else {
            None
        };
        TimeToReturnFireReport {
            param_id,
            disposition,
            rearmed_event_id,
        }
    }

    fn register<Callback: Copy>(
        &mut self,
        map_id: u32,
        return_date: TagTime,
        buffer_time: u32,
        loop_weekly: bool,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let event_id = timer.set_time_event(return_date, callback, id);
        self.params.insert(
            id,
            TimeToReturnParam {
                map_id,
                return_date,
                buffer_time,
                loop_weekly,
                event_id,
            },
        );
    }
}

fn tokens(source: &[u8]) -> impl Iterator<Item = &[u8]> {
    source
        .split(u8::is_ascii_whitespace)
        .filter(|token| !token.is_empty())
}

fn next_token<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<&'a [u8], TimeToReturnLoadError> {
    tokens
        .next()
        .ok_or(TimeToReturnLoadError::MissingValue { field })
}

fn next_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, TimeToReturnLoadError> {
    std::str::from_utf8(next_token(tokens, field)?)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(TimeToReturnLoadError::InvalidValue { field })
}
