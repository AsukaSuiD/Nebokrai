//! Ежедневные действия `CThingSetup` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner `setup/leitingsetup.cpp`.
//!
//! Wire — signed count и шестибайтные `TID/max/point` records. Daily projection
//! считает постоянными TID `> 1999`, недельными — строго `1000 < TID < 2000`;
//! weekday снимается отдельно для каждого weekly элемента. Пустой результат
//! не очищает destination.
//!
//! Loader очищает owner, ищет `#`, добавляет нулевой record даже при поздней
//! ошибке extraction и затем прекращает scan. Файл без записей возвращает 0,
//! с записью — 1; out-of-range поле остаётся нулём.
//! Отрицательный signed count оригинальный decoder (decoder VA 0x4db5a0,
//! выход по JBE) принял бы за огромный unsigned loop; Rust отвечает явной
//! ошибкой `NegativeCount` вместо чтения за пределами буфера.
//! Установленный экземпляр и его потребители остаются у владельца роли.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::resources::read_to_marker as read_to;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeiTingLocalTime {
    pub second: i32,
    pub minute: i32,
    pub hour: i32,
    pub month_day: i32,
    pub month: i32,
    pub year_since_1900: i32,
    pub week_day: i32,
    pub year_day: i32,
    pub daylight_saving: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeiTingThingNode {
    pub thing_id: u16,
    pub max_count: u16,
    pub point: u16,
}

impl Default for LeiTingThingNode {
    fn default() -> Self {
        Self {
            thing_id: 0,
            max_count: 0,
            point: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeiTingDailyThing {
    pub thing_id: u16,
    pub count: u16,
    pub max_count: u16,
    pub point: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThingSetupCodecError {
    NegativeCount(i32),
    CountOutsideLegacyRange(usize),
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for ThingSetupCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeCount(count) => {
                write!(formatter, "отрицательное число LeiTing-записей: {count}")
            }
            Self::CountOutsideLegacyRange(count) => write!(
                formatter,
                "число LeiTing-записей {count} не помещается в signed long"
            ),
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "LeiTing payload оборван на {offset}: требуется {needed}, доступно {available}"
            ),
        }
    }
}

impl Error for ThingSetupCodecError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThingSetupTextField {
    ThingId,
    MaximumCount,
    Point,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThingSetupTextCutoffReason {
    UnexpectedEnd,
    InvalidUnsignedShort { token: Vec<u8> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThingSetupTextCutoff {
    pub zero_based_line: usize,
    pub field: ThingSetupTextField,
    pub reason: ThingSetupTextCutoffReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThingSetupLoadReport {
    pub loaded_count: usize,
    pub cutoff: Option<ThingSetupTextCutoff>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThingSetupEmptyFile;

impl fmt::Display for ThingSetupEmptyFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("LeitingAction.ini не содержит ни одной #-записи")
    }
}

impl Error for ThingSetupEmptyFile {}

#[derive(Debug)]
pub enum ThingSetupFileLoadError {
    Io(std::io::Error),
    Empty(ThingSetupEmptyFile),
}

impl fmt::Display for ThingSetupFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => {
                write!(
                    formatter,
                    "не удалось прочитать LeitingAction.ini: {source}"
                )
            }
            Self::Empty(source) => source.fmt(formatter),
        }
    }
}

impl Error for ThingSetupFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Empty(source) => Some(source),
        }
    }
}

#[derive(Clone, Default)]
pub struct CThingSetup {
    all_things: VecDeque<LeiTingThingNode>,
}

impl CThingSetup {
    pub const fn new() -> Self {
        Self {
            all_things: VecDeque::new(),
        }
    }

    pub const fn set_daily_update_stamp(local_time: &mut LeiTingLocalTime) {
        local_time.hour = 23;
        local_time.minute = 59;
        local_time.second = 59;
    }

    /// Стандартная filesystem-граница для standalone owner-а. World resource
    /// lifecycle передаёт уже прочитанные байты в `load_all_thing_list`.
    pub fn load_all_thing_list_from_file(
        &mut self,
        path: impl AsRef<Path>,
        add_log_text: impl FnMut(&[u8]),
    ) -> Result<ThingSetupLoadReport, ThingSetupFileLoadError> {
        self.all_things.clear();
        let path = path.as_ref();
        let source = std::fs::read(path).map_err(ThingSetupFileLoadError::Io)?;
        let display_path = path.to_string_lossy();
        self.load_all_thing_list(&source, display_path.as_bytes(), add_log_text)
            .map_err(ThingSetupFileLoadError::Empty)
    }

    pub fn clear_all_things_for_load(&mut self) {
        self.all_things.clear();
    }

    /// Повторяет byte-token loader и его логи. Невалидное поле безопасно
    /// оставляет ноль/прочитанный prefix в уже добавляемой записи и завершает
    /// дальнейший scan, как fail-state исходного `istream`.
    pub fn load_all_thing_list(
        &mut self,
        source: &[u8],
        file_name: &[u8],
        mut add_log_text: impl FnMut(&[u8]),
    ) -> Result<ThingSetupLoadReport, ThingSetupEmptyFile> {
        self.all_things.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let mut cutoff = None;
        let mut line = 0usize;

        while read_to(&mut tokens, b"#") {
            let mut thing = LeiTingThingNode::default();
            for (field, destination) in [
                (ThingSetupTextField::ThingId, &mut thing.thing_id),
                (ThingSetupTextField::MaximumCount, &mut thing.max_count),
                (ThingSetupTextField::Point, &mut thing.point),
            ] {
                if let Err(reason) = read_formatted_u16(&mut tokens, destination) {
                    cutoff = Some(ThingSetupTextCutoff {
                        zero_based_line: line,
                        field,
                        reason,
                    });
                    break;
                }
            }

            self.all_things.push_back(thing);
            add_log_text(
                format!(
                    "<leiting>line {line}: {},{},{}",
                    thing.thing_id, thing.max_count, thing.point
                )
                .as_bytes(),
            );
            line += 1;
            if cutoff.is_some() {
                break;
            }
        }

        if line == 0 {
            add_log_text(b"<Error>The File LeitingAction.ini is Empty!");
            return Err(ThingSetupEmptyFile);
        }

        let mut summary = format!("We have {line} line data in ").into_bytes();
        summary.extend_from_slice(file_name);
        add_log_text(&summary);
        Ok(ThingSetupLoadReport {
            loaded_count: line,
            cutoff,
        })
    }

    pub fn add_to_byte_array(&self, destination: &mut Vec<u8>) -> Result<(), ThingSetupCodecError> {
        let count = i32::try_from(self.all_things.len())
            .map_err(|_| ThingSetupCodecError::CountOutsideLegacyRange(self.all_things.len()))?;
        destination.extend_from_slice(&count.to_le_bytes());
        for thing in &self.all_things {
            destination.extend_from_slice(&thing.thing_id.to_le_bytes());
            destination.extend_from_slice(&thing.max_count.to_le_bytes());
            destination.extend_from_slice(&thing.point.to_le_bytes());
        }
        Ok(())
    }

    /// Декодирует GameServer initial-config projection, сохраняя prefix при
    /// безопасной ошибке вместо исходного безразмерного overread.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), ThingSetupCodecError> {
        self.all_things.clear();
        let count = read_i32(source, cursor)?;
        if count < 0 {
            return Err(ThingSetupCodecError::NegativeCount(count));
        }
        for _ in 0..count {
            self.all_things.push_back(LeiTingThingNode {
                thing_id: read_u16(source, cursor)?,
                max_count: read_u16(source, cursor)?,
                point: read_u16(source, cursor)?,
            });
        }
        Ok(())
    }

    pub fn all_things(&self) -> &VecDeque<LeiTingThingNode> {
        &self.all_things
    }

    /// Собирает оригинал daily list; `get_week_day` вызывается отдельно для
    /// каждого недельного TID, как старый `GetLocalTime` внутри цикла.
    pub fn get_daily_thing_list(
        &self,
        mut get_week_day: impl FnMut() -> u16,
        destination: &mut VecDeque<LeiTingDailyThing>,
    ) {
        let mut daily = VecDeque::new();
        for node in &self.all_things {
            let thing_id = node.thing_id;
            let selected = thing_id > 1999
                || (thing_id > 1000 && thing_id < 2000 && get_week_day() == (thing_id % 1000) % 7);
            if selected {
                daily.push_back(LeiTingDailyThing {
                    thing_id,
                    count: 0,
                    max_count: node.max_count,
                    point: node.point,
                });
            }
        }
        if !daily.is_empty() {
            *destination = daily;
        }
    }
}

fn read_formatted_u16<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    destination: &mut u16,
) -> Result<(), ThingSetupTextCutoffReason> {
    let Some(token) = tokens.next() else {
        return Err(ThingSetupTextCutoffReason::UnexpectedEnd);
    };
    let value = std::str::from_utf8(token)
        .ok()
        .and_then(|text| text.parse::<u16>().ok())
        .ok_or_else(|| ThingSetupTextCutoffReason::InvalidUnsignedShort {
            token: token.to_vec(),
        })?;
    *destination = value;
    Ok(())
}

fn read_u16(source: &[u8], cursor: &mut usize) -> Result<u16, ThingSetupCodecError> {
    let offset = *cursor;
    let end = offset.saturating_add(2);
    let Some(bytes) = source.get(offset..end) else {
        return Err(ThingSetupCodecError::UnexpectedEnd {
            offset,
            needed: 2,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = end;
    Ok(u16::from_le_bytes(
        bytes.try_into().expect("slice содержит ровно два байта"),
    ))
}

fn read_i32(source: &[u8], cursor: &mut usize) -> Result<i32, ThingSetupCodecError> {
    let offset = *cursor;
    let end = offset.saturating_add(4);
    let Some(bytes) = source.get(offset..end) else {
        return Err(ThingSetupCodecError::UnexpectedEnd {
            offset,
            needed: 4,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = end;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("slice содержит ровно четыре байта"),
    ))
}

// документацией оставшегося text loader-а и variant-доказательств.
