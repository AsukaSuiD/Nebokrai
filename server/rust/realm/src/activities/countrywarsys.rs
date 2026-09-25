//! Блоки загрузки и перезагрузки войны стран `CountryWarSys`, вынесенные
//! сюда заранее: расписание, singleton-owner и ветки lifecycle остаются в
//! старом `appworld/country/countrywarsys.rs` до шага переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use nebokrai_shared::values::{TagTimeArithmeticBlock, TagTimeParseBlock};

use crate::app::world_message::SendMessageError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryWarLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
    TimeParse(TagTimeParseBlock),
    Arithmetic(TagTimeArithmeticBlock),
}

impl From<TagTimeParseBlock> for CountryWarLoadError {
    fn from(value: TagTimeParseBlock) -> Self {
        Self::TimeParse(value)
    }
}

impl From<TagTimeArithmeticBlock> for CountryWarLoadError {
    fn from(value: TagTimeArithmeticBlock) -> Self {
        Self::Arithmetic(value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryWarEndReport {
    pub reset_regions: usize,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryWarReloadEvent {
    PrepareBegin,
    PrepareEnd,
    DeclareBegin,
    DeclareEnd,
    InfoBegin,
    Begin,
    InfoEnd,
    End,
    Clear,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryWarReloadBlock {
    MissingEventId {
        war_id: i32,
        event: CountryWarReloadEvent,
        kill_requests: u32,
        killed_events: u32,
    },
    Load {
        source: CountryWarLoadError,
        kill_requests: u32,
        killed_events: u32,
        end_war: CountryWarEndReport,
    },
}
