//! Владелец конфигурации ежедневных LeiTing-действий.
//!
//! `SetDailyUpdateStamp`, `AddToByteArray`, GameServer
//! `DecordFromByteArray`, WorldServer `LoadAllThingList` и
//! `GetDailyThingList` —. Точные пары EXE/PDB и исходные owners
//! сохранены у raw-блоков.
//!
//! Исходный singleton/static deque заменён обычным `CThingSetup` и
//! `VecDeque`. Wire остаётся signed count, затем записи `u16 TID/max/point`
//! по шесть байт. Daily projection сохраняет странные границы: постоянны
//! только TID `> 1999`, недельны строго `1000 < TID < 2000`, а weekday
//! снимается отдельным platform-вызовом для каждого недельного элемента.
//! Если подходящих элементов нет, старый owner не очищал destination; Rust
//! также оставляет его без изменения.
//!
//! Text loader очищает owner до открытия, ищет byte-оригинал whitespace-маркеры
//! `#` общим `ReadTo`, логирует каждую добавленную запись и считает пустым файл
//! без единого маркера. Open/empty возвращает `0`, хотя бы одна запись — `1`.
//! После найденного маркера исходный код нулями инициализировал node, добавлял
//! его даже при fail-state formatted extraction и лишь затем прекращал scan;
//! safe parser сохраняет node/prefix transition и явно сообщает место
//! остановки в load-report. Точное значение, которое старый MSVC мог записать
//! при numeric overflow, не переносится: штатный файл содержит только малые
//! положительные `u16`, а malformed/out-of-range поле безопасно остаётся
//! нулём. `std::fs` заменяет только `CRFile` plumbing.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingLocalTime {
    pub(crate) second: i32,
    pub(crate) minute: i32,
    pub(crate) hour: i32,
    pub(crate) month_day: i32,
    pub(crate) month: i32,
    pub(crate) year_since_1900: i32,
    pub(crate) week_day: i32,
    pub(crate) year_day: i32,
    pub(crate) daylight_saving: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingThingNode {
    pub(crate) thing_id: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
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
pub(crate) struct LeiTingDailyThing {
    pub(crate) thing_id: u16,
    pub(crate) count: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThingSetupCodecError {
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
pub(crate) enum ThingSetupTextField {
    ThingId,
    MaximumCount,
    Point,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ThingSetupTextCutoffReason {
    UnexpectedEnd,
    InvalidUnsignedShort { token: Vec<u8> },
}

/// Safe-диагностика исходного stream fail-state после уже добавленного node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ThingSetupTextCutoff {
    pub(crate) zero_based_line: usize,
    pub(crate) field: ThingSetupTextField,
    pub(crate) reason: ThingSetupTextCutoffReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ThingSetupLoadReport {
    pub(crate) loaded_count: usize,
    pub(crate) cutoff: Option<ThingSetupTextCutoff>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ThingSetupEmptyFile;

impl fmt::Display for ThingSetupEmptyFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("LeitingAction.ini не содержит ни одной #-записи")
    }
}

impl Error for ThingSetupEmptyFile {}

#[derive(Debug)]
pub(crate) enum ThingSetupFileLoadError {
    Io(std::io::Error),
    Empty(ThingSetupEmptyFile),
}

impl fmt::Display for ThingSetupFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => {
                write!(formatter, "не удалось прочитать LeitingAction.ini: {source}")
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
pub(crate) struct CThingSetup {
    all_things: VecDeque<LeiTingThingNode>,
}

impl CThingSetup {
    pub(crate) const fn new() -> Self {
        Self {
            all_things: VecDeque::new(),
        }
    }

 /// Оригинал `23:59:59`; остальные поля `tm` не меняются.
    pub(crate) const fn set_daily_update_stamp(local_time: &mut LeiTingLocalTime) {
        local_time.hour = 23;
        local_time.minute = 59;
        local_time.second = 59;
    }

 /// Стандартная filesystem-граница для standalone owner-а. World resource
 /// lifecycle передаёт уже прочитанные байты в `load_all_thing_list`.
    pub(crate) fn load_all_thing_list_from_file(
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

 /// Очищает owner до следующей попытки открытия, как оригинал loader.
    pub(crate) fn clear_all_things_for_load(&mut self) {
        self.all_things.clear();
    }

 /// Повторяет byte-token loader и его логи. Невалидное поле безопасно
 /// оставляет ноль/прочитанный prefix в уже добавляемой записи и завершает
 /// дальнейший scan, как fail-state исходного `istream`.
    pub(crate) fn load_all_thing_list(
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
                (
                    ThingSetupTextField::MaximumCount,
                    &mut thing.max_count,
                ),
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

 /// Кодирует WorldServer initial-config projection.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), ThingSetupCodecError> {
        let count = i32::try_from(self.all_things.len()).map_err(|_| {
            ThingSetupCodecError::CountOutsideLegacyRange(self.all_things.len())
        })?;
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
    pub(crate) fn decord_from_byte_array(
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

 /// Собирает оригинал daily list; `get_week_day` вызывается отдельно для
 /// каждого недельного TID, как старый `GetLocalTime` внутри цикла.
    pub(crate) fn get_daily_thing_list(
        &self,
        mut get_week_day: impl FnMut() -> u16,
        destination: &mut VecDeque<LeiTingDailyThing>,
    ) {
        let mut daily = VecDeque::new();
        for node in &self.all_things {
            let thing_id = node.thing_id;
            let selected = thing_id > 1999
                || (thing_id > 1000
                    && thing_id < 2000
                    && get_week_day() == (thing_id % 1000) % 7);
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
