//! Post-init контексты рантайма WorldServer (JJC weekly reset и LeiTing
//! daily reset) в составе Realm.
//!
//! Источник контракта — та же точная пара, что у [`crate::app::world_runtime`]
//! (`.exe/Nworldserver.exe` + `.exe/WorldServer.pdb`, SHA-256 `F3AC454D…`,
//! RSDS совпадает). Платформенная подкладка повторяет выбранные CRT-вызовы
//! оригинала буквально: `_localtime`/`_mktime` через `libc::localtime_r` и
//! `libc::mktime` с полной нормализацией всех девяти полей `tm`, и Win32
//! `WritePrivateProfileString` вебсторони `JJcConfig.ini` через замену только
//! значения ключа (без перезаписи файла целиком). Типы owner-ов (`CJJcSystem`,
//! `CLeiTing`, `TiberiusRsJjcSys`, workers — см. их модули в `activities/`) уже
//! Realm; `ServerCommandHandle` — Shared; `CGame` сюда не тянется.
//!
//! Glue-логирование наблюдаемости остаётся process-edge строками; исходные
//! события typed (`JjcLogEvent`, worker events). Файл не вводит новых
//! Send-обязательств: dyn-совместимость сохраняется по правилу ADR-0013, а
//! форма владения detached worker-ами остаётся у модулей `activities/`.
//!
//! Здесь же runtime-швы `WorldJjcRuntimeContext` и `WorldLeiTingRuntimeContext`
//! с их worker-мостами (`WorldJjcWorkerContext`, `WorldLeiTingWorkerContext`)
//! и process-impl этих швов: посадка impl вне этого crate дала бы
//! orphan-нарушение, потому что process-контексты живут здесь. Мосты
//! конструируются владельцем хода (`world_main_loop`) через `new`; наблюдаемость
//! держится на исходно принятом JjcLogEvent-подходе.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use chrono::{Datelike, Timelike};

use nebokrai_shared::network::ServerCommandHandle;
use nebokrai_shared::resources::LeiTingLocalTime;

use crate::activities::jjcsystem::{
    JjcLocalTime, JjcLogEvent, JjcRank, JjcRunContext, JjcSystemTime,
};
use crate::activities::jjcmaintenanceworker::{
    WorldJjcWeekClearWorker, WorldJjcWeekClearWorkerEvent,
};
use crate::activities::leiting::LeiTingContext;
use crate::activities::leitingreset::LeiTingDatabaseResetRequest;
use crate::activities::leitingresetworker::{
    WorldLeiTingResetWorker, WorldLeiTingResetWorkerEvent,
};
use crate::activities::rsjjcsys::{RsJjcSysOwner, TiberiusRsJjcSys};
use crate::app::world_message::CMessage;
use crate::app::worldserver::{WorldLogLocalTime, WorldLogTextOwner};
use crate::persistence::rssetup::WorldDatabaseSettings;

struct WorldRuntimeLog {
    owner: WorldLogTextOwner,
    started_at: Instant,
    save_info_time_ms: u32,
}

impl WorldRuntimeLog {
    fn add(&self, payload: &[u8]) {
        let started_at = self.started_at;
        let _ = self.owner.add_log_text(
            payload,
            self.save_info_time_ms,
            move || started_at.elapsed().as_millis() as u32,
            current_world_log_time,
            |line| eprintln!("WorldServer: {}", String::from_utf8_lossy(line)),
        );
    }
}

fn current_world_log_time() -> WorldLogLocalTime {
    let now = chrono::Local::now();
    WorldLogLocalTime {
        year: now.year() as u16,
        month: now.month() as u16,
        day: now.day() as u16,
        hour: now.hour() as u16,
        minute: now.minute() as u16,
        second: now.second() as u16,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldPlatformTimeError {
    LocalTimeUnavailable { timestamp: i64 },
    NormalizedTimestampOutsideLegacyRange { timestamp: i64 },
}

impl fmt::Display for WorldPlatformTimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalTimeUnavailable { timestamp } => {
                write!(formatter, "localtime не представил timestamp {timestamp}")
            }
            Self::NormalizedTimestampOutsideLegacyRange { timestamp } => write!(
                formatter,
                "mktime вернул {timestamp}, непредставимый 32-битным time_t оригинала"
            ),
        }
    }
}

impl Error for WorldPlatformTimeError {}

fn local_tm(timestamp: i64) -> Result<libc::tm, WorldPlatformTimeError> {
    let timestamp = timestamp as libc::time_t;
    let mut local = std::mem::MaybeUninit::<libc::tm>::uninit();
    // `localtime_r` — потокобезопасная системная замена MSVC `_localtime`;
    // указатели живут только внутри этого вызова и результат сразу копируется.
    let result = unsafe { libc::localtime_r(&timestamp, local.as_mut_ptr()) };
    if result.is_null() {
        return Err(WorldPlatformTimeError::LocalTimeUnavailable {
            timestamp: timestamp as i64,
        });
    }
    // `localtime_r` при non-null результате полностью инициализировал `tm`.
    Ok(unsafe { local.assume_init() })
}

fn jjc_local_time(timestamp: i32) -> JjcLocalTime {
    let local = local_tm(i64::from(timestamp))
        .expect("signed 32-битный JJC timestamp обязан представляться системным localtime");
    JjcLocalTime {
        second: local.tm_sec,
        minute: local.tm_min,
        hour: local.tm_hour,
        month_day: local.tm_mday,
        month: local.tm_mon,
        year_since_1900: local.tm_year,
        week_day: local.tm_wday,
        year_day: local.tm_yday,
        daylight_saving: local.tm_isdst,
    }
}

pub fn current_lei_ting_local_time() -> LeiTingLocalTime {
    let timestamp = chrono::Local::now().timestamp();
    let local = local_tm(timestamp)
        .expect("текущее системное время обязано представляться localtime");
    lei_ting_time_from_tm(&local)
}

fn lei_ting_time_from_tm(local: &libc::tm) -> LeiTingLocalTime {
    LeiTingLocalTime {
        second: local.tm_sec,
        minute: local.tm_min,
        hour: local.tm_hour,
        month_day: local.tm_mday,
        month: local.tm_mon,
        year_since_1900: local.tm_year,
        week_day: local.tm_wday,
        year_day: local.tm_yday,
        daylight_saving: local.tm_isdst,
    }
}

fn normalize_lei_ting_time(
    local: &mut LeiTingLocalTime,
) -> Result<i32, WorldPlatformTimeError> {
    let mut native = libc::tm {
        tm_sec: local.second,
        tm_min: local.minute,
        tm_hour: local.hour,
        tm_mday: local.month_day,
        tm_mon: local.month,
        tm_year: local.year_since_1900,
        tm_wday: local.week_day,
        tm_yday: local.year_day,
        tm_isdst: local.daylight_saving,
        ..unsafe { std::mem::zeroed() }
    };
    // `_mktime` в EXE одновременно нормализовал все девять полей `tm`.
    let timestamp = unsafe { libc::mktime(&mut native) };
    *local = lei_ting_time_from_tm(&native);
    i32::try_from(timestamp).map_err(|_| {
        WorldPlatformTimeError::NormalizedTimestampOutsideLegacyRange {
            timestamp: timestamp as i64,
        }
    })
}

pub struct WorldJjcProcessContext {
    runtime: tokio::runtime::Handle,
    started_at: Instant,
    config_path: PathBuf,
    database: TiberiusRsJjcSys,
    worker: Arc<WorldJjcWeekClearWorker>,
    log: WorldRuntimeLog,
}

impl WorldJjcProcessContext {
    pub fn new(
        runtime: tokio::runtime::Handle,
        started_at: Instant,
        runtime_directory: &Path,
        settings: &WorldDatabaseSettings,
        worker: Arc<WorldJjcWeekClearWorker>,
        log: WorldLogTextOwner,
        save_info_time_ms: u32,
    ) -> Self {
        Self {
            runtime,
            started_at,
            config_path: runtime_directory.join("setup").join("JJcConfig.ini"),
            database: TiberiusRsJjcSys::new(settings),
            worker,
            log: WorldRuntimeLog {
                owner: log,
                started_at,
                save_info_time_ms,
            },
        }
    }
}

impl JjcRunContext for WorldJjcProcessContext {
    fn current_time_seconds(&mut self) -> i32 {
        chrono::Local::now().timestamp() as i32
    }

    fn local_time(&mut self, timestamp: i32) -> JjcLocalTime {
        jjc_local_time(timestamp)
    }

    fn system_time(&mut self) -> JjcSystemTime {
        let now = chrono::Local::now();
        JjcSystemTime {
            year: now.year() as u16,
            month: now.month() as u16,
            week_day: now.weekday().num_days_from_sunday() as u16,
            day: now.day() as u16,
            hour: now.hour() as u16,
            minute: now.minute() as u16,
            second: now.second() as u16,
            milliseconds: now.timestamp_subsec_millis() as u16,
        }
    }

    fn tick_count_ms(&mut self) -> u32 {
        self.started_at.elapsed().as_millis() as u32
    }

    fn load_jjc_rank(&mut self, ranks: &mut Vec<JjcRank>) -> bool {
        self.database.load_jjc_rank(ranks)
    }

    fn start_jjc_week_clear(&mut self) -> bool {
        self.worker.dispatch(self.runtime.clone()).is_ok()
    }

    fn clear_jjc_season(&mut self) -> bool {
        self.worker.clear_season(self.runtime.clone())
    }

    fn write_private_profile_string(
        &mut self,
        section: &[u8],
        key: &[u8],
        value: &[u8],
    ) -> bool {
        replace_ini_value(&self.config_path, section, key, value).is_ok()
    }

    fn log(&mut self, event: JjcLogEvent) {
        match event {
            JjcLogEvent::Closed => self.log.add(b"JJc is Closed."),
            JjcLogEvent::RankLoaded { success, item_count } => self.log.add(
                format!(
                    "Load JJc ranks from DB ({item_count} items) {}.",
                    if success { "OK" } else { "Fail" }
                )
                .as_bytes(),
            ),
            JjcLogEvent::WeekUpdateStarted { local_time } => self.log.add(
                format!(
                    "JJc week update start at {:04}-{:02}-{:02} {:02}:{:02}:{:02}.",
                    local_time.year,
                    local_time.month,
                    local_time.day,
                    local_time.hour,
                    local_time.minute,
                    local_time.second,
                )
                .as_bytes(),
            ),
            JjcLogEvent::WeekUpdateThreadFailed => {
                self.log.add(b"JJc week update thread can't begin.")
            }
            JjcLogEvent::SeasonUpdateStarted => self.log.add(b"JJc season update start."),
            JjcLogEvent::WeekOrSeasonUpdateFinished { elapsed_ms } => self.log.add(
                format!("JJc week or season update finished in {elapsed_ms} ms.").as_bytes(),
            ),
            JjcLogEvent::WeekClearDatabaseSlow { elapsed_ms } => self.log.add(
                format!("JJc week database clear used {elapsed_ms} ms.").as_bytes(),
            ),
            event => eprintln!("WorldServer: JJC runtime-событие {event:?}"),
        }
    }
}

/// Platform/log дополнение к `JjcRunContext`, необходимое concrete DB-worker-у.
/// Сам доменный `CJJcSystem` по-прежнему не знает о Tokio либо system threads.
///
/// Worker-мост и process-impl живут здесь же: посадка impl вне этого crate
/// стала бы orphan-нарушением, потому что process-контекст уже живёт здесь.
pub trait WorldJjcRuntimeContext: JjcRunContext {
    fn on_week_clear_spawn_failed(&mut self, error: io::Error);
    fn on_week_clear_worker_event(&mut self, event: WorldJjcWeekClearWorkerEvent);
}

/// Узкий adapter, связывающий подтверждённый `CJJcSystem::Run` с одним
/// `WorldJjcWeekClearWorker`, не передавая mutable game-owner в поток.
/// Конструируется владельцем хода через `new`; поля остаются приватными.
pub struct WorldJjcWorkerContext<'a, Context> {
    context: &'a mut Context,
    worker: &'a WorldJjcWeekClearWorker,
    runtime: tokio::runtime::Handle,
}

impl<'a, Context> WorldJjcWorkerContext<'a, Context> {
    pub fn new(
        context: &'a mut Context,
        worker: &'a WorldJjcWeekClearWorker,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            context,
            worker,
            runtime,
        }
    }
}

impl<Context: WorldJjcRuntimeContext> JjcRunContext for WorldJjcWorkerContext<'_, Context> {
    fn current_time_seconds(&mut self) -> i32 {
        self.context.current_time_seconds()
    }

    fn local_time(&mut self, timestamp: i32) -> JjcLocalTime {
        self.context.local_time(timestamp)
    }

    fn system_time(&mut self) -> JjcSystemTime {
        self.context.system_time()
    }

    fn tick_count_ms(&mut self) -> u32 {
        self.context.tick_count_ms()
    }

    fn load_jjc_rank(&mut self, ranks: &mut Vec<JjcRank>) -> bool {
        self.context.load_jjc_rank(ranks)
    }

    fn start_jjc_week_clear(&mut self) -> bool {
        match self.worker.dispatch(self.runtime.clone()) {
            Ok(()) => true,
            Err(error) => {
                self.context.on_week_clear_spawn_failed(error);
                false
            }
        }
    }

    fn clear_jjc_season(&mut self) -> bool {
        let returned = self.worker.clear_season(self.runtime.clone());
        while let Some(event) = self.worker.try_next_event() {
            self.context.on_week_clear_worker_event(event);
        }
        returned
    }

    fn write_private_profile_string(
        &mut self,
        section: &[u8],
        key: &[u8],
        value: &[u8],
    ) -> bool {
        self.context
            .write_private_profile_string(section, key, value)
    }

    fn log(&mut self, event: JjcLogEvent) {
        self.context.log(event);
    }
}

impl WorldJjcRuntimeContext for WorldJjcProcessContext {
    fn on_week_clear_spawn_failed(&mut self, error: io::Error) {
        eprintln!("WorldServer: не создан JJC week-clear worker: {error}");
    }

    fn on_week_clear_worker_event(&mut self, event: WorldJjcWeekClearWorkerEvent) {
        eprintln!("WorldServer: JJC DB worker: {event:?}");
    }
}

pub struct WorldLeiTingProcessContext {
    runtime: tokio::runtime::Handle,
    sender: Option<ServerCommandHandle>,
    worker: Arc<WorldLeiTingResetWorker>,
    log: WorldRuntimeLog,
}

impl WorldLeiTingProcessContext {
    pub fn new(
        runtime: tokio::runtime::Handle,
        sender: Option<ServerCommandHandle>,
        worker: Arc<WorldLeiTingResetWorker>,
        log: WorldLogTextOwner,
        started_at: Instant,
        save_info_time_ms: u32,
    ) -> Self {
        Self {
            runtime,
            sender,
            worker,
            log: WorldRuntimeLog {
                owner: log,
                started_at,
                save_info_time_ms,
            },
        }
    }

    /// Glue-строка worker-событий старого process lifecycle: текст уходит в
    /// тот же `WorldLogTextOwner` без смены порядка записей.
    pub fn add_log(&self, payload: &[u8]) {
        self.log.add(payload);
    }
}

impl LeiTingContext for WorldLeiTingProcessContext {
    type Block = WorldPlatformTimeError;

    fn add_update_start_log(&mut self) {
        let now = chrono::Local::now();
        self.log.add(
            format!(
                "UpdateLeiTing Start at :{}-{}-{} {}:{}:{} \r\n",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            )
            .as_bytes(),
        );
    }

    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block> {
        let signed_timestamp = i64::from(timestamp as i32);
        local_tm(signed_timestamp).map(|local| lei_ting_time_from_tm(&local))
    }

    fn current_week_day(&mut self) -> u16 {
        chrono::Local::now().weekday().num_days_from_sunday() as u16
    }

    fn send_all(&mut self, message: &CMessage) {
        let _ = message.send_all(self.sender.as_ref());
    }

    fn add_database_begin_log(&mut self) {
        self.log
            .add(b"UpdateLeiTing Start, ResetAllLeitingInDB Begin");
    }

    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block> {
        normalize_lei_ting_time(local_time)
    }

    fn reset_all_lei_ting_in_database(&mut self, update_kind: u32, stamp: i32) {
        let request = LeiTingDatabaseResetRequest { update_kind, stamp };
        if let Err(error) = self.worker.dispatch(request, self.runtime.clone()) {
            eprintln!(
                "WorldServer: не создан LeiTing DB worker для kind {} stamp {}: {error}",
                request.update_kind, request.stamp
            );
        }
    }

    fn add_update_end_log(&mut self) {
        self.log.add(b"LeiTing All Update End");
    }
}

/// Platform/log дополнение к `LeiTingContext`, необходимое concrete DB-worker-у.
/// Сам доменный `CLeiTing` по-прежнему не знает о Tokio либо system threads.
///
/// Worker-мост и process-impl живут здесь же: посадка impl вне этого crate
/// стала бы orphan-нарушением, потому что process-контекст уже живёт здесь.
pub trait WorldLeiTingRuntimeContext: LeiTingContext {
    fn on_database_reset_spawn_failed(
        &mut self,
        request: LeiTingDatabaseResetRequest,
        error: io::Error,
    );

    fn on_database_reset_worker_event(&mut self, event: WorldLeiTingResetWorkerEvent);
}

/// Узкий adapter, связывающий подтверждённый `CLeiTing::Run` с одним
/// `WorldLeiTingResetWorker`, не передавая mutable game-owner в поток.
/// Конструируется владельцем хода через `new`; поля остаются приватными.
pub struct WorldLeiTingWorkerContext<'a, Context> {
    context: &'a mut Context,
    worker: &'a WorldLeiTingResetWorker,
    runtime: tokio::runtime::Handle,
}

impl<'a, Context> WorldLeiTingWorkerContext<'a, Context> {
    pub fn new(
        context: &'a mut Context,
        worker: &'a WorldLeiTingResetWorker,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            context,
            worker,
            runtime,
        }
    }
}

impl<Context: WorldLeiTingRuntimeContext> LeiTingContext
    for WorldLeiTingWorkerContext<'_, Context>
{
    type Block = Context::Block;

    fn add_update_start_log(&mut self) {
        self.context.add_update_start_log();
    }

    fn local_time_from_timestamp(
        &mut self,
        timestamp: u32,
    ) -> Result<LeiTingLocalTime, Self::Block> {
        self.context.local_time_from_timestamp(timestamp)
    }

    fn current_week_day(&mut self) -> u16 {
        self.context.current_week_day()
    }

    fn send_all(&mut self, message: &CMessage) {
        self.context.send_all(message);
    }

    fn add_database_begin_log(&mut self) {
        self.context.add_database_begin_log();
    }

    fn mktime(&mut self, local_time: &mut LeiTingLocalTime) -> Result<i32, Self::Block> {
        self.context.mktime(local_time)
    }

    fn reset_all_lei_ting_in_database(&mut self, update_kind: u32, stamp: i32) {
        let request = LeiTingDatabaseResetRequest { update_kind, stamp };
        if let Err(error) = self.worker.dispatch(request, self.runtime.clone()) {
            self.context
                .on_database_reset_spawn_failed(request, error);
        }
    }

    fn add_update_end_log(&mut self) {
        self.context.add_update_end_log();
    }
}

impl WorldLeiTingRuntimeContext for WorldLeiTingProcessContext {
    fn on_database_reset_spawn_failed(
        &mut self,
        request: LeiTingDatabaseResetRequest,
        error: io::Error,
    ) {
        eprintln!(
            "WorldServer: не создан LeiTing DB worker для kind {} stamp {}: {error}",
            request.update_kind, request.stamp
        );
    }

    fn on_database_reset_worker_event(&mut self, event: WorldLeiTingResetWorkerEvent) {
        match event {
            WorldLeiTingResetWorkerEvent::Started(_) => {
                self.add_log(b"Strictest Enforcement update thread begin.")
            }
            WorldLeiTingResetWorkerEvent::Finished { outcome, .. } => {
                eprintln!("WorldServer: LeiTing DB worker завершён: {outcome:?}")
            }
        }
    }
}

/// Построчная замена значения `key=` внутри `[section]` INI-файла без
/// перезаписи остальных строк — Rust-воплощение Win32
/// `WritePrivateProfileString`, которым `CJJcSystem` сохранял недельный
/// счётчик в `JJcConfig.ini`.
fn replace_ini_value(
    path: &Path,
    section: &[u8],
    key: &[u8],
    value: &[u8],
) -> io::Result<()> {
    let source = std::fs::read(path).unwrap_or_default();
    let mut section_start = None;
    let mut section_end = source.len();
    let mut value_range = None;
    let mut current_section_matches = false;
    let mut line_start = 0;

    while line_start < source.len() {
        let line_end = source[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(source.len(), |offset| line_start + offset);
        let content_end = line_end
            - usize::from(line_end > line_start && source[line_end - 1] == b'\r');
        let line = &source[line_start..content_end];
        let trimmed = trim_ascii(line);
        if trimmed.starts_with(b"[") && trimmed.ends_with(b"]") {
            if current_section_matches {
                section_end = line_start;
                break;
            }
            current_section_matches = trimmed[1..trimmed.len() - 1]
                .eq_ignore_ascii_case(section);
            if current_section_matches {
                section_start = Some(if line_end < source.len() { line_end + 1 } else { line_end });
            }
        } else if current_section_matches
            && let Some(equal) = line.iter().position(|byte| *byte == b'=')
            && trim_ascii(&line[..equal]).eq_ignore_ascii_case(key)
        {
            let mut value_start = line_start + equal + 1;
            while value_start < content_end && source[value_start].is_ascii_whitespace() {
                value_start += 1;
            }
            value_range = Some(value_start..content_end);
            break;
        }
        line_start = if line_end < source.len() { line_end + 1 } else { source.len() };
    }

    let mut updated = source;
    if let Some(range) = value_range {
        updated.splice(range, value.iter().copied());
    } else if let Some(insert_at) = section_start.map(|_| section_end) {
        let mut line = key.to_vec();
        line.extend_from_slice(b"=");
        line.extend_from_slice(value);
        line.extend_from_slice(b"\r\n");
        updated.splice(insert_at..insert_at, line);
    } else {
        if !updated.is_empty() && !updated.ends_with(b"\n") {
            updated.extend_from_slice(b"\r\n");
        }
        updated.extend_from_slice(b"[");
        updated.extend_from_slice(section);
        updated.extend_from_slice(b"]\r\n");
        updated.extend_from_slice(key);
        updated.extend_from_slice(b"=");
        updated.extend_from_slice(value);
        updated.extend_from_slice(b"\r\n");
    }
    std::fs::write(path, updated)
}

fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldMainLoopContextBuildError {
    MissingDatabaseSettings,
    MissingDbMiscOwner,
    MissingLargessOwner,
}

impl fmt::Display for WorldMainLoopContextBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDatabaseSettings => {
                formatter.write_str("World DB settings не опубликованы после Init")
            }
            Self::MissingDbMiscOwner => {
                formatter.write_str("World DbMisc owner не опубликован после Init")
            }
            Self::MissingLargessOwner => {
                formatter.write_str("World Largess owner не опубликован после Init")
            }
        }
    }
}

impl Error for WorldMainLoopContextBuildError {}
