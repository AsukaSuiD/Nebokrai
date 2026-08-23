//! Общие структуры организаций из `organizing.cpp/.h`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! `tagMemInfo` сохраняет 0xF0-байтную проекцию: fixed byte-имена, level/job,
//! title, 11 permission states, region, `tagTime` и contribution flag.
//! Faction/union wire передаёт permission и time блоки явно little-endian,
//! не включая padding всего Rust-объекта.
//!
//! Отсутствующий NUL в fixed buffers отклоняется вместо чтения за массивом.
//! `map::operator[]` оригинала мог создать лишь частично инициализированный
//! member, поэтому безопасный тип не реализует неявный `Default`: все поля
//! задаются вместе. Общие enums сохраняют исходные signed значения.

use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

const MEMBER_NAME_CAPACITY: usize = 32;
const MEMBER_TEXT_CAPACITY: usize = 64;
const PURVIEW_COUNT: usize = 11;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum EOperator {
    Delete = 0,
    Add = 1,
    Update = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum ECityState {
    No = 0,
    Duth = 1,
    Mass = 2,
    Fight = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum EPurview {
    Disband = 0,
    Exit = 1,
    DubJobLevel = 2,
    ConMem = 3,
    FireOut = 4,
    Pronounce = 5,
    LeaveWord = 6,
    EditLeaveWord = 7,
    ObtainTax = 8,
    OperCityGate = 9,
    EndueRor = 10,
}

impl EPurview {
    pub(crate) const fn from_wire_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Disband),
            1 => Some(Self::Exit),
            2 => Some(Self::DubJobLevel),
            3 => Some(Self::ConMem),
            4 => Some(Self::FireOut),
            5 => Some(Self::Pronounce),
            6 => Some(Self::LeaveWord),
            7 => Some(Self::EditLeaveWord),
            8 => Some(Self::ObtainTax),
            9 => Some(Self::OperCityGate),
            10 => Some(Self::EndueRor),
            _ => None,
        }
    }

    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}

impl ECityState {
    pub(crate) const fn from_wire_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Duth),
            2 => Some(Self::Mass),
            3 => Some(Self::Fight),
            _ => None,
        }
    }

    pub(crate) const fn wire_value(self) -> i32 {
        self as i32
    }
}

impl EOperator {
    pub(crate) const fn from_wire_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Delete),
            1 => Some(Self::Add),
            2 => Some(Self::Update),
            _ => None,
        }
    }

    pub(crate) const fn wire_value(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum EPurviewOwnState {
    No = 0,
    Forbid = 1,
    Permit = 2,
}

impl EPurviewOwnState {
    pub(crate) const fn from_wire_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Forbid),
            2 => Some(Self::Permit),
            _ => None,
        }
    }

    pub(crate) const fn wire_value(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberPurviewMutation {
    InvalidPurview,
    MemberNotFound,
    Unchanged,
    Changed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct TagTimeValue {
    pub(crate) year: u16,
    pub(crate) month: u16,
    pub(crate) day_of_week: u16,
    pub(crate) day: u16,
    pub(crate) hour: u16,
    pub(crate) minute: u16,
    pub(crate) second: u16,
    pub(crate) milliseconds: u16,
}

impl TagTimeValue {
    pub(crate) fn wire_bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        for (chunk, value) in bytes.chunks_exact_mut(2).zip([
            self.year,
            self.month,
            self.day_of_week,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.milliseconds,
        ]) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnterminatedMemberField {
    pub(crate) field: &'static str,
}

impl fmt::Display for UnterminatedMemberField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedMemberField {}

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct TagMemInfo {
    pub(crate) id: i32,
    pub(crate) name: [u8; MEMBER_NAME_CAPACITY],
    pub(crate) level: i32,
    pub(crate) occupation: i32,
    pub(crate) job_level: i32,
    pub(crate) title: [u8; MEMBER_TEXT_CAPACITY],
    pub(crate) purview: [EPurviewOwnState; PURVIEW_COUNT],
    pub(crate) region: [u8; MEMBER_TEXT_CAPACITY],
    pub(crate) last_online_time: TagTimeValue,
    pub(crate) contribute: bool,
}

impl TagMemInfo {
 /// Создаёт только полностью определённое member-значение; старого частично
 /// неинициализированного `map::operator[]` аналога намеренно нет.
    #[allow(
        clippy::too_many_arguments,
        reason = "параметры один к одному сохраняют десять доказанных data-полей tagMemInfo"
    )]
    pub(crate) const fn from_complete_fields(
        id: i32,
        name: [u8; MEMBER_NAME_CAPACITY],
        level: i32,
        occupation: i32,
        job_level: i32,
        title: [u8; MEMBER_TEXT_CAPACITY],
        purview: [EPurviewOwnState; PURVIEW_COUNT],
        region: [u8; MEMBER_TEXT_CAPACITY],
        last_online_time: TagTimeValue,
        contribute: bool,
    ) -> Self {
        Self {
            id,
            name,
            level,
            occupation,
            job_level,
            title,
            purview,
            region,
            last_online_time,
            contribute,
        }
    }

    pub(crate) fn name_wire_bytes(&self) -> Result<&[u8], UnterminatedMemberField> {
        terminated_field(&self.name, "strName")
    }

    pub(crate) fn title_wire_bytes(&self) -> Result<&[u8], UnterminatedMemberField> {
        terminated_field(&self.title, "strTitle")
    }

    pub(crate) fn region_wire_bytes(&self) -> Result<&[u8], UnterminatedMemberField> {
        terminated_field(&self.region, "strRegion")
    }

    pub(crate) fn purview_wire_bytes(&self) -> [u8; 44] {
        let mut bytes = [0; 44];
        for (chunk, state) in bytes.chunks_exact_mut(4).zip(self.purview) {
            chunk.copy_from_slice(&state.wire_value().to_le_bytes());
        }
        bytes
    }

    pub(crate) fn last_online_wire_bytes(&self) -> [u8; 16] {
        self.last_online_time.wire_bytes()
    }
}

fn terminated_field<'a>(
    field: &'a [u8],
    name: &'static str,
) -> Result<&'a [u8], UnterminatedMemberField> {
    let Some(terminator) = field.iter().position(|byte| *byte == 0) else {
 // Исходные перегрузки с `char*` продолжали бы чтение за фиксированным
 // массивом. Достижимость и наблюдаемая реакция такого
 // состояния не определены и не заменяются добавленным NUL или unsafe.
        return Err(UnterminatedMemberField { field: name });
    };
    Ok(&field[..=terminator])
}

const _: () = {
    assert!(size_of::<EOperator>() == 4);
    assert!(size_of::<EPurview>() == 4);
    assert!(size_of::<EPurviewOwnState>() == 4);
    assert!(size_of::<TagTimeValue>() == 0x10);
    assert!(size_of::<TagMemInfo>() == 0xF0);
    assert!(offset_of!(TagMemInfo, id) == 0x00);
    assert!(offset_of!(TagMemInfo, name) == 0x04);
    assert!(offset_of!(TagMemInfo, level) == 0x24);
    assert!(offset_of!(TagMemInfo, occupation) == 0x28);
    assert!(offset_of!(TagMemInfo, job_level) == 0x2C);
    assert!(offset_of!(TagMemInfo, title) == 0x30);
    assert!(offset_of!(TagMemInfo, purview) == 0x70);
    assert!(offset_of!(TagMemInfo, region) == 0x9C);
    assert!(offset_of!(TagMemInfo, last_online_time) == 0xDC);
    assert!(offset_of!(TagMemInfo, contribute) == 0xEC);
};
