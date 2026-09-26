//! Общие технические функции исходного `public/tools.cpp`.
//!
//! `IMPLEMENTED`: `IniDecoder` (Login/World), `PutStringToFile` (Billing/
//! Game), GameServer logging owners (`AddLogText`/`AddErrorLogText`/
//! `PutDebugString`); `GetLineDir` Game дополнительно `VERIFIED_DISASSEMBLY`.
//! Остальной корпус файла остаётся `UNKNOWN`. Пары EXE/PDB — в
//! `server/rust/src/manifest/`.
//!
//! `IniDecoder` выполняет побайтовое `~(byte - 0x0C)` с 8-битным wrapping;
//! NUL-терминатор принадлежал caller и не добавляется. `PutStringToFile`
//! создаёт `log`, дописывает `<name>_YYYY-MM-DD.txt` с локальной меткой
//! `\nN(MM-DD HH:MM:SS):`; файловые ошибки по-прежнему тихие, а process-wide
//! счётчик записей сохраняет monotonic increment без data race. `GetLineDir`
//! сначала wrapping-разности Windows `long`, округляет X до `float`, оставляет
//! Y точным целым и делит плоскость углами из точного EXE; для достигнутых
//! целочисленных координат — те же восемь направлений и `0` совпавших точек.
//! GameServer logging owners сохраняют дневной log, timestamp/CRLF и отдельный
//! process debug-файл; Win32 GUI log-window не влиял на игровой результат.
//!
//! Счётчик записей и имя debug-файла — process-wide helpers, общие для всех
//! ролей; журнальный каталог выбирает владелец процесса.
//! Доказательства: docs/reconstruction/shared-technical.md#технические-функции-publictoolscpp

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;

use chrono::{Datelike, Local, Timelike};

static LOG_RECORD_NUMBER: AtomicI32 = AtomicI32::new(0);
static DEBUG_FILE_NAME: OnceLock<PathBuf> = OnceLock::new();

/// Декодирует byte-exact содержимое старого `setup.dat` без интерпретации текста.
pub fn ini_decode(encoded: &[u8]) -> Vec<u8> {
    encoded
        .iter()
        .map(|byte| !byte.wrapping_sub(0x0c))
        .collect()
}

/// Тихо дописывает запись в датированный файл исходного каталога `log`.
pub fn put_string_to_file(name: &str, value: &[u8]) {
    let now = Local::now();
    let _ = fs::create_dir("log");
    let path = format!(
        "log/{name}_{:04}-{:02}-{:02}.txt",
        now.year(),
        now.month(),
        now.day()
    );
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let number = LOG_RECORD_NUMBER
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1);
    let prefix = format!(
        "\n{number}({:02}-{:02} {:02}:{:02}:{:02}):",
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let _ = file.write_all(prefix.as_bytes());
    let _ = file.write_all(value);
}

/// Linux-владелец общего `PutLogInfo`: дописывает byte-exact строку в
/// подтверждённый дневной `log/YYYY-MM-DD.txt`.
fn put_log_info(value: &[u8]) {
    let now = Local::now();
    let _ = fs::create_dir("log");
    let path = format!(
        "log/{:04}-{:02}-{:02}.txt",
        now.year(),
        now.month(),
        now.day()
    );
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = file.write_all(c_string_prefix(value));
}

/// Safe owner для GameServer `AddLogText`: сохраняет timestamp, CRLF и общий
/// дневной файл оригинала, но не воспроизводит небезопасный varargs buffer.
pub fn add_game_log_text(value: &[u8]) {
    add_game_log_record(value, false);
}

/// Safe owner для GameServer `AddErrorLogText`.
pub fn add_game_error_log_text(value: &[u8]) {
    add_game_log_record(value, true);
}

fn add_game_log_record(value: &[u8], error: bool) {
    let now = Local::now();
    let marker = if error { " <error> " } else { " " };
    let prefix = format!(
        "[{:02}-{:02} {:02}:{:02}:{:02}]{marker}",
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let mut record = Vec::with_capacity(prefix.len() + value.len() + 2);
    record.extend_from_slice(prefix.as_bytes());
    record.extend_from_slice(c_string_prefix(value));
    record.extend_from_slice(b"\r\n");
    put_log_info(&record);
}

/// Safe Linux owner для process-global `PutDebugString`. Имя debug-файла
/// фиксируется при первом вызове, как после `InitialDebugFileName` оригинала.
pub fn put_debug_string(value: &[u8]) {
    let path = DEBUG_FILE_NAME.get_or_init(|| {
        let now = Local::now();
        PathBuf::from(format!(
            "log/debug{}_{}_{}[{:02}_{:02}_{:02}].txt",
            now.year(),
            now.month(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        ))
    });
    let _ = fs::create_dir("log");
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let number = LOG_RECORD_NUMBER
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1);
    let now = Local::now();
    let prefix = format!(
        "\n{number}({:02}-{:02} {:02}:{:02}:{:02}):",
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let _ = file.write_all(prefix.as_bytes());
    let _ = file.write_all(c_string_prefix(value));
}

fn c_string_prefix(value: &[u8]) -> &[u8] {
    value.split(|byte| *byte == 0).next().unwrap_or_default()
}

/// Возвращает одно из восьми направлений исходной line-direction сетки.
pub fn get_line_direction(
    source_x: i32,
    source_y: i32,
    destination_x: i32,
    destination_y: i32,
) -> i32 {
    const LOWER_ANGLE: f64 = 0.392_599_999_904_632_57;
    const UPPER_ANGLE: f64 = 1.177_999_973_297_119_1;

    let delta_x = destination_x.wrapping_sub(source_x);
    let delta_y = destination_y.wrapping_sub(source_y);
    let x = f64::from(delta_x as f32);
    let y = f64::from(delta_y);
    if x == 0.0 && y == 0.0 {
        return 0;
    }

    let slope = (y / x).abs();
    if slope < LOWER_ANGLE.tan() {
        return if x > 0.0 { 2 } else { 6 };
    }
    if UPPER_ANGLE.tan() <= slope {
        return if y > 0.0 { 4 } else { 0 };
    }
    match (x > 0.0, y > 0.0) {
        (true, true) => 3,
        (true, false) => 1,
        (false, true) => 5,
        (false, false) => 7,
    }
}
