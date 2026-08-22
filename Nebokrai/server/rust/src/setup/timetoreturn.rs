//! Владелец `TimeToReturn` исторического WorldServer.
//!
//! `initialize/load/reload/on_time` RVA
//! `0x00073DF0/0x000736F0/0x00073CF0/0x00072DC0` восстановлены по
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, source PDB
//! `e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp`. Loader хранит
//! map-order записи, читает `#` weekly и `*` absolute секции
//! `setup/TimeToReturn.ini`, а callback отправляет `0x7FA13` с двумя signed
//! long: map ID и buffer time. `BTreeMap`, typed `TimerId` и typed errors
//! заменяют STL, singleton и неинициализированные event IDs без смены wire,
//! календарных границ или порядка side effects.
//!
//! Exact `load` очищал map, но не отменял уже созданные events; эта странность
//! сохранена. `reload` отдельно отменяет IDs в map-order перед `load`. Невалидный
//! календарный input не получает старую неинициализированную запись: Rust
//! возвращает явную ошибку на той же bool-гранящей позиции caller-а.

use std::collections::BTreeMap;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::date::{TagTime, TagTimeArithmeticBlock, TagTimeParseBlock};
use crate::public::readwrite::read_to;
use crate::public::timer::{CTimer, TimerId};

#[derive(Clone, Copy, Debug)]
pub(crate) struct TimeToReturnCallbacks<Callback> {
    pub(crate) on_time: Callback,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TimeToReturnParam {
    pub(crate) map_id: u32,
    pub(crate) return_date: TagTime,
    pub(crate) buffer_time: u32,
    pub(crate) loop_weekly: bool,
    pub(crate) event_id: TimerId,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimeToReturnLoadReport {
    pub(crate) resource_found: bool,
    pub(crate) weekly_registered: u32,
    pub(crate) absolute_registered: u32,
    pub(crate) weekly_past: u32,
    pub(crate) absolute_past: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimeToReturnLoadError {
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

pub(crate) trait TimeToReturnContext {
    fn game_server_number_by_region_id(&mut self, region_id: i32) -> Option<i32>;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TimeToReturnFireDisposition {
    MissingEntry,
    RegionMissing,
    Sent {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TimeToReturnFireReport {
    pub(crate) param_id: i32,
    pub(crate) disposition: TimeToReturnFireDisposition,
    pub(crate) rearmed_event_id: Option<TimerId>,
}

#[derive(Default)]
pub(crate) struct TimeToReturn {
    params: BTreeMap<i32, TimeToReturnParam>,
    next_id: i32,
}

impl TimeToReturn {
    pub(crate) const fn new() -> Self {
        Self {
            params: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub(crate) fn params(&self) -> &BTreeMap<i32, TimeToReturnParam> {
        &self.params
    }

    /// Exact `initialize`: это только вызов `load`.
    pub(crate) fn initialize<Callback: Copy>(
        &mut self,
        source: Option<&[u8]>,
        now: TagTime,
        timer: &mut CTimer<Callback>,
        callbacks: TimeToReturnCallbacks<Callback>,
    ) -> Result<TimeToReturnLoadReport, TimeToReturnLoadError> {
        self.load(source, now, timer, callbacks)
    }

    /// Exact `load`: очищает registry без отмены старых calendar events.
    pub(crate) fn load<Callback: Copy>(
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

    pub(crate) fn reload<Callback: Copy>(
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

    pub(crate) fn on_time<Callback: Copy, Context: TimeToReturnContext + ?Sized>(
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp

// ============================================================================
// FUNCTION: TimeToReturn::instance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp:17
// RVA: 0x00072C70
// ADDRESS: 00472c70
// PROTOTYPE: TimeToReturn * __cdecl instance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: TimeToReturn::uninitialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp:31
// RVA: 0x00072CF0
// ADDRESS: 00472cf0
// PROTOTYPE: void __cdecl uninitialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: TimeToReturn::on_time
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp:141
// RVA: 0x00072DC0
// ADDRESS: 00472dc0
// PROTOTYPE: void __stdcall on_time(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: TimeToReturn::load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp:37
// RVA: 0x000736F0
// ADDRESS: 004736f0
// PROTOTYPE: bool __cdecl load(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: TimeToReturn::reload
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp:168
// RVA: 0x00073CF0
// ADDRESS: 00473cf0
// PROTOTYPE: bool __cdecl reload(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: TimeToReturn::initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp:25
// RVA: 0x00073DF0
// ADDRESS: 00473df0
// PROTOTYPE: bool __cdecl initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTradeList::tagTrade::~tagTrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp
// RVA: 0x0009A950
// ADDRESS: 0049a950
// PROTOTYPE: void __thiscall ~tagTrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049ab76
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp
// RVA: 0x0009AB76
// ADDRESS: 0049ab76
// PROTOTYPE: undefined Catch@0049ab76()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049ad01
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp
// RVA: 0x0009AD01
// ADDRESS: 0049ad01
// PROTOTYPE: undefined Catch@0049ad01()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTradeList::tagTrade::tagTrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp
// RVA: 0x0009AF30
// ADDRESS: 0049af30
// PROTOTYPE: undefined __thiscall tagTrade(tagTrade * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049b4be
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp
// RVA: 0x0009B4BE
// ADDRESS: 0049b4be
// PROTOTYPE: undefined Catch@0049b4be()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00531f90
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\timetoreturn.cpp
// RVA: 0x00131F90
// ADDRESS: 00531f90
// PROTOTYPE: undefined Unwind@00531f90()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
