//! Владелец исторического `tagTime` (`public/date.cpp/.h`, пары EXE/PDB
//! Login/Game/World в `server/rust/src/manifest/`). Реализованы: оба
//! конструктора, `IsLeap`, пять сравнений, `AddDay/Hour/Minute/Second`,
//! `GetTimeDifference` и `GetFormatStr`.
//!
//! PDB задаёт восемь последовательных `u16` и размер `0x10`; сравнения
//! намеренно игнорируют `wDayOfWeek`/`wMilliseconds`, поэтому стандартные
//! `Eq`/`Ord` создали бы иной контракт и не реализованы. `chrono::Local`
//! заменяет только `GetLocalTime` (weekday в SYSTEMTIME-форме Sunday=0);
//! календарная арифметика буквально сохраняет наблюдаемые странности: stale
//! `wDayOfWeek`, component-wise difference, допуск month `0` через нулевой
//! элемент `dtab`, оборачивание `u16` year. Month за пределами `0..=12` и
//! signed overflow исходного `long` — локальный `BLOCKED_MISSING_FACT`, а не
//! придуманная нормализация.
//!
//! String constructor разбирает шесть colon-частей семантикой `atoi` и сужает
//! до `u16`; при отсутствии `:` старый `npos + 1` оборачивался в ноль, поэтому
//! та же строка используется для следующих частей. Переполнение `atoi` не
//! имеет доказанного результата и блокируется отдельно. Формат: без
//! zero-padding и секунд — `year-month-day hour:minute`.
//!
//! Экземпляры и их мутации остаются у владельца роли; shared держит только
//! представление и календарную арифметику.
//! Доказательства: docs/reconstruction/shared-technical.md#календарное-значение-tagtime

use chrono::{Datelike, Local, Timelike};

const COMMON_MONTH_DAYS: [i32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_MONTH_DAYS: [i32; 13] = [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const STRING_COMPONENTS: [&str; 6] = ["year", "month", "day", "hour", "minute", "second"];

/// Byte-for-field представление старого `tagTime` без Windows ABI-зависимости.
#[derive(Clone, Copy, Debug, Default)]
pub struct TagTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

/// Safe-граница старой signed arithmetic и `dtab` indexing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TagTimeArithmeticBlock {
    InvalidMonth { month: u16 },
    SignedOverflow { operation: &'static str },
}

/// Недоказанный результат CRT `atoi` после signed overflow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TagTimeParseBlock {
    pub component: &'static str,
}

impl TagTime {
    /// Создаёт запись без дополнительной валидации, как копирование WORD-полей.
    pub const fn from_fields(fields: [u16; 8]) -> Self {
        Self {
            year: fields[0],
            month: fields[1],
            day_of_week: fields[2],
            day: fields[3],
            hour: fields[4],
            minute: fields[5],
            second: fields[6],
            milliseconds: fields[7],
        }
    }

    /// Возвращает локальные поля в той же нумерации, что Win32 `SYSTEMTIME`.
    pub fn local_now() -> Self {
        let now = Local::now();
        Self {
            year: now.year() as u16,
            month: now.month() as u16,
            day_of_week: now.weekday().num_days_from_sunday() as u16,
            day: now.day() as u16,
            hour: now.hour() as u16,
            minute: now.minute() as u16,
            second: now.second() as u16,
            milliseconds: now.timestamp_subsec_millis() as u16,
        }
    }

    pub const fn fields(self) -> [u16; 8] {
        [
            self.year,
            self.month,
            self.day_of_week,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.milliseconds,
        ]
    }

    /// Разбирает legacy `year:month:day:hour:minute:second` через `atoi`.
    pub fn from_legacy_string(source: &[u8]) -> Result<Self, TagTimeParseBlock> {
        let mut remaining = source;
        let mut fields = [0u16; 6];
        for (index, component) in STRING_COMPONENTS.into_iter().enumerate() {
            let delimiter = remaining.iter().position(|byte| *byte == b':');
            let part = delimiter.map_or(remaining, |offset| &remaining[..offset]);
            fields[index] = legacy_atoi(part, component)? as u16;
            if let Some(offset) = delimiter {
                remaining = &remaining[offset + 1..];
            }
        }
        Ok(Self {
            year: fields[0],
            month: fields[1],
            day_of_week: 0,
            day: fields[2],
            hour: fields[3],
            minute: fields[4],
            second: fields[5],
            milliseconds: 0,
        })
    }

    /// Повторяет `tagTime::IsLeap` для signed Windows `long`.
    pub const fn is_leap(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Повторяет `tagTime::operator==`, включая игнорирование двух полей.
    pub fn legacy_eq(self, other: Self) -> bool {
        self.comparison_fields() == other.comparison_fields()
    }

    /// Повторяет `tagTime::operator>` как лексикографию шести полей.
    pub fn legacy_gt(self, other: Self) -> bool {
        self.comparison_fields() > other.comparison_fields()
    }

    /// Повторяет `tagTime::operator>=` через исходные `>` и `==`.
    pub fn legacy_ge(self, other: Self) -> bool {
        self.legacy_gt(other) || self.legacy_eq(other)
    }

    /// Повторяет `tagTime::operator<` через исходные `>` и `==`.
    pub fn legacy_lt(self, other: Self) -> bool {
        !self.legacy_gt(other) && !self.legacy_eq(other)
    }

    /// Повторяет `tagTime::operator<=` как отрицание исходного `>`.
    pub fn legacy_le(self, other: Self) -> bool {
        !self.legacy_gt(other)
    }

    /// Сдвигает календарные year/month/day, не меняя stale day-of-week.
    pub fn add_day(&mut self, amount: i32) -> Result<&mut Self, TagTimeArithmeticBlock> {
        if amount == 0 {
            return Ok(self);
        }
        if amount > 0 {
            let mut remaining = amount.checked_add(i32::from(self.day)).ok_or(
                TagTimeArithmeticBlock::SignedOverflow {
                    operation: "amount + wDay",
                },
            )?;
            self.day = 0;
            loop {
                let days = month_days(self.year, self.month)?;
                if remaining <= days {
                    break;
                }
                remaining -= days;
                let old_month = self.month;
                self.month = old_month.wrapping_add(1);
                if self.month > 12 {
                    self.year = self.year.wrapping_add(1);
                    self.month = old_month.wrapping_sub(11);
                }
            }
            self.day = remaining as u16;
            return Ok(self);
        }

        let magnitude = amount
            .checked_neg()
            .ok_or(TagTimeArithmeticBlock::SignedOverflow {
                operation: "-amount",
            })?;
        if magnitude < i32::from(self.day) {
            self.day = self.day.wrapping_add(amount as u16);
            return Ok(self);
        }
        let mut remaining = magnitude.checked_sub(i32::from(self.day)).ok_or(
            TagTimeArithmeticBlock::SignedOverflow {
                operation: "-amount - wDay",
            },
        )?;
        move_to_previous_month(self);
        self.day = month_days(self.year, self.month)? as u16;
        loop {
            let days = month_days(self.year, self.month)?;
            if remaining < days {
                break;
            }
            remaining -= days;
            move_to_previous_month(self);
            self.day = month_days(self.year, self.month)? as u16;
        }
        self.day = self.day.wrapping_sub(remaining as u16);
        Ok(self)
    }

    /// Сдвигает часы с исходным quotient/remainder и переносом через `AddDay`.
    pub fn add_hour(&mut self, amount: i32) -> Result<&mut Self, TagTimeArithmeticBlock> {
        if amount == 0 {
            return Ok(self);
        }
        let remainder = amount % 24;
        let _ = self.add_day(amount / 24)?;
        let combined = i32::from(self.hour) + remainder;
        if remainder < 1 && combined < 0 {
            let _ = self.add_day(-1)?;
            self.hour = self.hour.wrapping_add(remainder as u16).wrapping_add(24);
        } else if remainder > 0 && combined > 23 {
            let _ = self.add_day(1)?;
            self.hour = self.hour.wrapping_add(remainder as u16).wrapping_sub(24);
        } else {
            self.hour = self.hour.wrapping_add(remainder as u16);
        }
        Ok(self)
    }

    /// Сдвигает минуты с исходным переносом через `AddHour`.
    pub fn add_minute(&mut self, amount: i32) -> Result<&mut Self, TagTimeArithmeticBlock> {
        if amount == 0 {
            return Ok(self);
        }
        let remainder = amount % 60;
        let _ = self.add_hour(amount / 60)?;
        let combined = i32::from(self.minute) + remainder;
        if remainder < 1 && combined < 0 {
            let _ = self.add_hour(-1)?;
            self.minute = self.minute.wrapping_add(remainder as u16).wrapping_add(60);
        } else if remainder > 0 && combined > 59 {
            let _ = self.add_hour(1)?;
            self.minute = self.minute.wrapping_add(remainder as u16).wrapping_sub(60);
        } else {
            self.minute = self.minute.wrapping_add(remainder as u16);
        }
        Ok(self)
    }

    /// Сдвигает секунды с исходным переносом через `AddMinute`.
    pub fn add_second(&mut self, amount: i32) -> Result<&mut Self, TagTimeArithmeticBlock> {
        if amount == 0 {
            return Ok(self);
        }
        let remainder = amount % 60;
        let _ = self.add_minute(amount / 60)?;
        let combined = i32::from(self.second) + remainder;
        if remainder < 1 && combined < 0 {
            let _ = self.add_minute(-1)?;
            self.second = self.second.wrapping_add(remainder as u16).wrapping_add(60);
        } else if remainder > 0 && combined > 59 {
            let _ = self.add_minute(1)?;
            self.second = self.second.wrapping_add(remainder as u16).wrapping_sub(60);
        } else {
            self.second = self.second.wrapping_add(remainder as u16);
        }
        Ok(self)
    }

    /// Возвращает исходную component-wise абсолютную разность двух записей.
    pub fn get_time_difference(self, other: Self) -> Result<Self, TagTimeArithmeticBlock> {
        if !self.legacy_gt(other) && !self.legacy_eq(other) {
            subtract_time_components(other, self)
        } else {
            subtract_time_components(self, other)
        }
    }

    /// Форматирует ровно `year-month-day hour:minute` без zero-padding.
    pub fn get_format_string(self) -> String {
        format!(
            "{}-{}-{} {}:{}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }

    const fn comparison_fields(self) -> [u16; 6] {
        [
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        ]
    }
}

fn month_days(year: u16, month: u16) -> Result<i32, TagTimeArithmeticBlock> {
    let table = if TagTime::is_leap(i32::from(year)) {
        &LEAP_MONTH_DAYS
    } else {
        &COMMON_MONTH_DAYS
    };
    table
        .get(usize::from(month))
        .copied()
        .ok_or(TagTimeArithmeticBlock::InvalidMonth { month })
}

fn move_to_previous_month(time: &mut TagTime) {
    let old_month = time.month;
    time.month = old_month.wrapping_sub(1);
    if time.month == 0 {
        time.year = time.year.wrapping_sub(1);
        time.month = old_month.wrapping_add(11);
    }
}

fn subtract_time_components(
    mut value: TagTime,
    subtrahend: TagTime,
) -> Result<TagTime, TagTimeArithmeticBlock> {
    let _ = value.add_second(-i32::from(subtrahend.second))?;
    let _ = value.add_minute(-i32::from(subtrahend.minute))?;
    let _ = value.add_hour(-i32::from(subtrahend.hour))?;
    let _ = value.add_day(-i32::from(subtrahend.day))?;
    let year_delta = -i32::from(subtrahend.year);
    if year_delta != 0 {
        let target_year = i32::from(value.year) + year_delta;
        if value.day == 29 && value.month == 2 && !TagTime::is_leap(target_year) {
            value.day = 1;
            value.month = 3;
        }
        value.year = value.year.wrapping_add(year_delta as u16);
    }
    Ok(value)
}

fn legacy_atoi(source: &[u8], component: &'static str) -> Result<i32, TagTimeParseBlock> {
    let mut source = source;
    while source.first().is_some_and(u8::is_ascii_whitespace) {
        source = &source[1..];
    }
    let (negative, digits) = match source.first() {
        Some(b'-') => (true, &source[1..]),
        Some(b'+') => (false, &source[1..]),
        _ => (false, source),
    };
    let mut value = 0i64;
    let mut found_digit = false;
    for &byte in digits {
        if !byte.is_ascii_digit() {
            break;
        }
        found_digit = true;
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(i64::from(byte - b'0')))
            .ok_or(TagTimeParseBlock { component })?;
    }
    if !found_digit {
        return Ok(0);
    }
    if negative {
        value = -value;
    }
    i32::try_from(value).map_err(|_| TagTimeParseBlock { component })
}
