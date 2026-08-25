//! Общие технические функции исходного `public/tools.cpp`.
//!
//! Статус `IniDecoder` LoginServer RVA `0x00020A90`, WorldServer RVA
//! `0x00053C50`, `PutStringToFile` BillingServer RVA `0x0000FC60` и GameServer
//! RVA `0x0001CEB0`, а также GameServer `GetLineDir` RVA `0x0001D080` —
//! `AddLogText`/`AddErrorLogText`/`PutDebugString` GameServer — `IMPLEMENTED`;
//! `GetLineDir` также `VERIFIED_DISASSEMBLY`. Остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точные пары:
//! `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`,
//! `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb` и
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//! Исходные пути PDB:
//! `h:\fengyun\fy_russia\src\public\tools.cpp:198`,
//! `d:\complite_version\fengyun_russia\trunk\public\tools.cpp:769` и
//! `e:\svn\fengyun_russia_dev\public\tools.cpp:198/769`.
//!
//! Оба бинарника содержат одинаковое побайтовое преобразование: для каждого
//! входного байта вычисляется `~(byte - 0x0C)` с 8-битным wrapping. Старые
//! raw pointers и отдельный заранее обнулённый output-buffer заменены owned
//! `Vec<u8>` той же длины; NUL-терминатор принадлежал caller и не добавляется
//! самой функцией.
//!
//! Совпадающие Billing/Game `PutStringToFile` создают `log`, открывают
//! `<name>_YYYY-MM-DD.txt` в append-режиме и дописывают локальную метку
//! `\nN(MM-DD HH:MM:SS):` и переданные bytes. `chrono::Local`, `create_dir`
//! и `OpenOptions::append` заменяют `GetLocalTime`, `CreateDirectoryA` и
//! `fopen("a+")`; все файловые ошибки по-прежнему остаются тихими. Старый
//! process-static `num` был общим счётчиком logging-функций; атомарный счётчик
//! сохраняет его monotonic increment без воспроизведения C++ data race.
//!
//! GameServer `GetLineDir` сначала выполняет wrapping-разности Windows `long`,
//! округляет X-разность до `float`, оставляет Y-разность точным целым в x87 и
//! делит плоскость углами из exact EXE `0.39259999990463257` и
//! `1.1779999732971191`. `f64::tan` заменяет x87 `fptan`; для достигнутых
//! целочисленных region-координат сохраняются те же восемь направлений и
//! исходный результат `0` для совпавших точек.
//!
//! Общие GameServer logging owners сохраняют дневной log, timestamp/CRLF,
//! отдельный process debug-файл и C-string prefix. `OnceLock`, безопасное
//! форматирование и append заменяют process globals, CRT varargs и Win32 GUI;
//! log-window не влиял на игровой результат. Остальные файловые и временные
//! владельцы этого крупного общего файла ещё не реализованы.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};

use chrono::{Datelike, Local, Timelike};

static LOG_RECORD_NUMBER: AtomicI32 = AtomicI32::new(0);
static DEBUG_FILE_NAME: OnceLock<PathBuf> = OnceLock::new();

/// Декодирует byte-exact содержимое старого `setup.dat` без интерпретации текста.
pub(crate) fn ini_decode(encoded: &[u8]) -> Vec<u8> {
    encoded
        .iter()
        .map(|byte| !byte.wrapping_sub(0x0c))
        .collect()
}

/// Тихо дописывает запись в датированный файл исходного каталога `log`.
pub(crate) fn put_string_to_file(name: &str, value: &[u8]) {
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
pub(crate) fn add_game_log_text(value: &[u8]) {
    add_game_log_record(value, false);
}

/// Safe owner для GameServer `AddErrorLogText`.
pub(crate) fn add_game_error_log_text(value: &[u8]) {
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
pub(crate) fn put_debug_string(value: &[u8]) {
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
pub(crate) fn get_line_direction(
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

// COMPONENT_VARIANT_BEGIN: AuthServer
// Точная пара: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SHA-256 EXE: AE0022429C135553092364F01838FA6EF8E631D558C96278123FF3ADE6AD3B15
// SHA-256 PDB: 26F8936605024F56B0A2C3BBB1923BCACD3DF9E17221FCC20AB38070E28403D5
// Исходный владелец PDB: h:\fengyun\fy_russia\src\public\tools.cpp

// ============================================================================
// FUNCTION: InitialDebugFileName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:151
// RVA: 0x0000BA10
// ADDRESS: 0040ba10
// PROTOTYPE: void __cdecl InitialDebugFileName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutDebugString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:172
// RVA: 0x0000BAA0
// ADDRESS: 0040baa0
// PROTOTYPE: void __cdecl PutDebugString(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutLogInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:249
// RVA: 0x0000BB80
// ADDRESS: 0040bb80
// PROTOTYPE: void __cdecl PutLogInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L88362
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002AC70
// ADDRESS: 0042ac70
// PROTOTYPE: undefined __stdcall $L88362(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L89902
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002AC90
// ADDRESS: 0042ac90
// PROTOTYPE: undefined __stdcall $L89902(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B980
// ADDRESS: 0042b980
// PROTOTYPE: void __cdecl $E2(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: AuthServer
// ARTIFACT: AuthServer/authserver.exe + AuthServer/authserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B990
// ADDRESS: 0042b990
// PROTOTYPE: void __cdecl $E5(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: AuthServer

// COMPONENT_VARIANT_BEGIN: BillingServer
// Точная пара: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SHA-256 EXE: FA32E3C043CB49965686129696A4EB34B733ACA1D60CAF57D369F97D5E68FB19
// SHA-256 PDB: F900CD0330BEFF32AC071B107AB653FD403CD18746896B3C0187C5751ACA0B21
// Исходный владелец PDB: h:\fengyun\fy_russia\src\public\tools.cpp

// ============================================================================
// FUNCTION: InitialDebugFileName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:151
// RVA: 0x0000FAF0
// ADDRESS: 0040faf0
// PROTOTYPE: void __cdecl InitialDebugFileName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutDebugString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:172
// RVA: 0x0000FB80
// ADDRESS: 0040fb80
// PROTOTYPE: void __cdecl PutDebugString(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutLogInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:249
// RVA: 0x0000FD80
// ADDRESS: 0040fd80
// PROTOTYPE: void __cdecl PutLogInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L67938
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B620
// ADDRESS: 0042b620
// PROTOTYPE: undefined __stdcall $L67938(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68117
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B640
// ADDRESS: 0042b640
// PROTOTYPE: undefined __stdcall $L68117(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68118
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B64B
// ADDRESS: 0042b64b
// PROTOTYPE: undefined __stdcall $L68118(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68119
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B656
// ADDRESS: 0042b656
// PROTOTYPE: undefined __stdcall $L68119(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68120
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B65E
// ADDRESS: 0042b65e
// PROTOTYPE: undefined __stdcall $L68120(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68121
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B666
// ADDRESS: 0042b666
// PROTOTYPE: undefined __stdcall $L68121(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68122
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B66E
// ADDRESS: 0042b66e
// PROTOTYPE: undefined __stdcall $L68122(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68123
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B676
// ADDRESS: 0042b676
// PROTOTYPE: undefined __stdcall $L68123(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68124
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B681
// ADDRESS: 0042b681
// PROTOTYPE: undefined __stdcall $L68124(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68125
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B68C
// ADDRESS: 0042b68c
// PROTOTYPE: undefined __stdcall $L68125(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68126
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B697
// ADDRESS: 0042b697
// PROTOTYPE: undefined __stdcall $L68126(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L68127
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6A2
// ADDRESS: 0042b6a2
// PROTOTYPE: undefined __stdcall $L68127(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69240
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6C0
// ADDRESS: 0042b6c0
// PROTOTYPE: undefined __cdecl $L69240(void * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69241
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6CB
// ADDRESS: 0042b6cb
// PROTOTYPE: undefined __stdcall $L69241(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69242
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6D6
// ADDRESS: 0042b6d6
// PROTOTYPE: undefined __stdcall $L69242(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69243
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6E1
// ADDRESS: 0042b6e1
// PROTOTYPE: undefined __stdcall $L69243(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69244
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6EC
// ADDRESS: 0042b6ec
// PROTOTYPE: undefined __stdcall $L69244(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69245
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B6F7
// ADDRESS: 0042b6f7
// PROTOTYPE: undefined __stdcall $L69245(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69247
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B702
// ADDRESS: 0042b702
// PROTOTYPE: undefined __stdcall $L69247(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69233
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B70D
// ADDRESS: 0042b70d
// PROTOTYPE: undefined __stdcall $L69233(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69234
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B718
// ADDRESS: 0042b718
// PROTOTYPE: undefined __stdcall $L69234(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69235
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B723
// ADDRESS: 0042b723
// PROTOTYPE: undefined __stdcall $L69235(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69236
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B72E
// ADDRESS: 0042b72e
// PROTOTYPE: undefined __stdcall $L69236(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69237
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B739
// ADDRESS: 0042b739
// PROTOTYPE: undefined __stdcall $L69237(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69239
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B744
// ADDRESS: 0042b744
// PROTOTYPE: undefined __stdcall $L69239(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L69232
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002B74F
// ADDRESS: 0042b74f
// PROTOTYPE: undefined __stdcall $L69232(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $E2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: BillingServer
// ARTIFACT: BillingServer/billingserver.exe + BillingServer/billingserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x0002CA80
// ADDRESS: 0042ca80
// PROTOTYPE: void __cdecl $E2(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: BillingServer

// COMPONENT_VARIANT_BEGIN: LoginServer
// Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SHA-256 EXE: 1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876
// SHA-256 PDB: FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C
// Исходный владелец PDB: d:\complite_version\fengyun_russia\trunk\public\tools.cpp

// ============================================================================
// FUNCTION: random
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: LoginServer
// ARTIFACT: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SOURCE: d:\complite_version\fengyun_russia\trunk\public\tools.cpp:42
// RVA: 0x00020610
// ADDRESS: 00420610
// PROTOTYPE: int __cdecl random(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFileLength
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: LoginServer
// ARTIFACT: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SOURCE: d:\complite_version\fengyun_russia\trunk\public\tools.cpp:98
// RVA: 0x00020710
// ADDRESS: 00420710
// PROTOTYPE: int __cdecl GetFileLength(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: InitialDebugFileName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: LoginServer
// ARTIFACT: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SOURCE: d:\complite_version\fengyun_russia\trunk\public\tools.cpp:151
// RVA: 0x00020750
// ADDRESS: 00420750
// PROTOTYPE: void __cdecl InitialDebugFileName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutDebugString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: LoginServer
// ARTIFACT: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SOURCE: d:\complite_version\fengyun_russia\trunk\public\tools.cpp:172
// RVA: 0x000207E0
// ADDRESS: 004207e0
// PROTOTYPE: void __cdecl PutDebugString(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutStringToFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: LoginServer
// ARTIFACT: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SOURCE: d:\complite_version\fengyun_russia\trunk\public\tools.cpp:198
// RVA: 0x000208C0
// ADDRESS: 004208c0
// PROTOTYPE: void __cdecl PutStringToFile(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutLogInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: LoginServer
// ARTIFACT: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb
// SOURCE: d:\complite_version\fengyun_russia\trunk\public\tools.cpp:249
// RVA: 0x000209E0
// ADDRESS: 004209e0
// PROTOTYPE: void __cdecl PutLogInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: LoginServer

// COMPONENT_VARIANT_BEGIN: MiscServer
// Точная пара: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SHA-256 EXE: F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65
// SHA-256 PDB: ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7
// Исходный владелец PDB: h:\fengyun\fy_russia\src\public\tools.cpp

// ============================================================================
// FUNCTION: random
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:42
// RVA: 0x00005E30
// ADDRESS: 00405e30
// PROTOTYPE: int __cdecl random(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutDebugString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:172
// RVA: 0x00005F30
// ADDRESS: 00405f30
// PROTOTYPE: void __cdecl PutDebugString(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutLogInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp:249
// RVA: 0x00006040
// ADDRESS: 00406040
// PROTOTYPE: void __cdecl PutLogInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L70274
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: MiscServer
// ARTIFACT: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SOURCE: h:\fengyun\fy_russia\src\public\tools.cpp
// RVA: 0x00023140
// ADDRESS: 00423140
// PROTOTYPE: undefined __stdcall $L70274(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: MiscServer

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\tools.cpp

// ============================================================================
// FUNCTION: random
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:42
// RVA: 0x0001CBA0
// ADDRESS: 0041cba0
// PROTOTYPE: int __cdecl random(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFileLength
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:98
// RVA: 0x0001CD00
// ADDRESS: 0041cd00
// PROTOTYPE: int __cdecl GetFileLength(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: InitialDebugFileName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:151
// RVA: 0x0001CD40
// ADDRESS: 0041cd40
// PROTOTYPE: void __cdecl InitialDebugFileName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutDebugString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:172
// RVA: 0x0001CDD0
// ADDRESS: 0041cdd0
// PROTOTYPE: void __cdecl PutDebugString(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: GameServer `PutStringToFile` совпадает с общей реализацией
// выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: PutLogInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:249
// RVA: 0x0001CFD0
// ADDRESS: 0041cfd0
// PROTOTYPE: void __cdecl PutLogInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: GameServer `GetLineDir` материализован выше; покрытый
// raw-блок удалён.

// ============================================================================
// FUNCTION: GetCurTickCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:734
// RVA: 0x0001D190
// ADDRESS: 0041d190
// PROTOTYPE: ulong __cdecl GetCurTickCount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00429f13
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp
// RVA: 0x00029F13
// ADDRESS: 00429f13
// PROTOTYPE: undefined Catch@00429f13()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\tools.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\tools.cpp

// ============================================================================
// FUNCTION: Catch@004510e3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp
// RVA: 0x000510E3
// ADDRESS: 004510e3
// PROTOTYPE: undefined Catch@004510e3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: random
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:42
// RVA: 0x00053560
// ADDRESS: 00453560
// PROTOTYPE: int __cdecl random(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFileLength
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:98
// RVA: 0x000536C0
// ADDRESS: 004536c0
// PROTOTYPE: int __cdecl GetFileLength(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: InitialDebugFileName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:151
// RVA: 0x00053700
// ADDRESS: 00453700
// PROTOTYPE: void __cdecl InitialDebugFileName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutDebugString
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:172
// RVA: 0x00053790
// ADDRESS: 00453790
// PROTOTYPE: void __cdecl PutDebugString(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutStringToFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:198
// RVA: 0x00053870
// ADDRESS: 00453870
// PROTOTYPE: void __cdecl PutStringToFile(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PutLogInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:249
// RVA: 0x00053990
// ADDRESS: 00453990
// PROTOTYPE: void __cdecl PutLogInfo(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFilePath
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:272
// RVA: 0x00053A40
// ADDRESS: 00453a40
// PROTOTYPE: char * __cdecl GetFilePath(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFileExtName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:288
// RVA: 0x00053A90
// ADDRESS: 00453a90
// PROTOTYPE: char * __cdecl GetFileExtName(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetFileDialog
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:370
// RVA: 0x00053AE0
// ADDRESS: 00453ae0
// PROTOTYPE: bool __cdecl GetFileDialog(bool param_1, char * param_2, char * param_3, char * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ReplaceLine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:438
// RVA: 0x00053C20
// ADDRESS: 00453c20
// PROTOTYPE: void __cdecl ReplaceLine(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CompareSystemTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:808
// RVA: 0x00053C80
// ADDRESS: 00453c80
// PROTOTYPE: long __cdecl CompareSystemTime(_SYSTEMTIME param_1, _SYSTEMTIME param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FindFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.cpp:615
// RVA: 0x00053D40
// ADDRESS: 00453d40
// PROTOTYPE: void __cdecl FindFile(char * param_1, char * param_2, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ToString<int>
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\tools.h:116
// RVA: 0x00084030
// ADDRESS: 00484030
// PROTOTYPE: basic_string<char,std::char_traits<char>,std::allocator<char>_> __cdecl ToString<int>(int * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
