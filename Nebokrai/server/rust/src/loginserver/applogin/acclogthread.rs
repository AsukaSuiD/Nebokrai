//! Consumer account-журналов LoginServer из `applogin/acclogthread.cpp`.
//! Контракт подтверждён точной парой LoginServer EXE/PDB.
//!
//! Четыре SQL-шаблона, регистр имён, пробелы и формат времени без ведущих
//! нулей соответствуют producer-функциям `game.cpp` и runtime-журналам
//! компонента. ADO/COM и Windows thread
//! заменены Tiberius и owned Rust thread с current-thread Tokio runtime; на
//! каждую запись, как в оригинале, создаётся отдельное DB-соединение.
//! Windows-1251 декодируется только после byte-exact сборки SQL.
//!
//! Исходные producer-функции писали в `char[512]` через небезопасный `sprintf`.
//! Это внутреннее ограничение и stack-overflow не переносятся: owned SQL может
//! быть длиннее, сохраняя тот же текст и DB side effect.
//! STL/CRT, SEH, COM cleanup и compiler thunks отдельного контракта не имели.

use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use chrono::{Datelike, NaiveDateTime, Timelike};
use encoding_rs::WINDOWS_1251;

use crate::dbaccess::logindb::rscdkey::{
    LoginDatabaseSettings, RsCdKeyDatabaseError, connect_login_database,
};
use crate::loginserver::loginserver::acclogqueue::AccLogQueue;
use crate::loginserver::loginserver::game::AccountLogRecord;

use super::gasoperator::format_ipv4;

/// Вид исходной producer-функции, не содержащий персональных значений записи.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccLogRecordKind {
    AccountEnter,
    RoleEnter,
    SessionLeave,
    AccountLeave,
}

/// Стадия отдельного ADO/TDS цикла, на которой потеряна текущая запись.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccLogDatabaseOperation {
    Connect,
    Execute,
}

/// Operator-visible результат фоновой обработки без раскрытия SQL-текста.
pub(crate) enum AccLogThreadNotice {
    Executed {
        kind: AccLogRecordKind,
    },
    DatabaseFailure {
        kind: AccLogRecordKind,
        operation: AccLogDatabaseOperation,
        error: RsCdKeyDatabaseError,
    },
}

impl fmt::Debug for AccLogThreadNotice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Executed { kind } => formatter
                .debug_struct("Executed")
                .field("kind", kind)
                .finish(),
            Self::DatabaseFailure {
                kind,
                operation,
                error,
            } => formatter
                .debug_struct("DatabaseFailure")
                .field("kind", kind)
                .field("operation", operation)
                .field("error", error)
                .finish(),
        }
    }
}

/// Owned Rust-замена одного исторического `AccLogThread`.
pub(crate) struct AccLogThread {
    queue: Arc<AccLogQueue>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    notices: Receiver<AccLogThreadNotice>,
}

impl AccLogThread {
    /// Создаёт runtime до публикации thread, поэтому ошибка запуска не оставляет worker.
    pub(crate) fn start(
        queue: Arc<AccLogQueue>,
        settings: LoginDatabaseSettings,
    ) -> Result<Self, io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_queue = Arc::clone(&queue);
        let worker_stop = Arc::clone(&stop);
        let (notice_tx, notices) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("login-acc-log".to_owned())
            .spawn(move || {
                run_acc_log_thread(runtime, worker_queue, worker_stop, settings, notice_tx);
            })?;
        Ok(Self {
            queue,
            stop,
            handle: Some(handle),
            notices,
        })
    }

    /// Забирает накопленные результаты в порядке завершения записей.
    pub(crate) fn drain_notices(&mut self) -> Vec<AccLogThreadNotice> {
        self.notices.try_iter().collect()
    }
}

impl Drop for AccLogThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.queue.wake_all();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_acc_log_thread(
    runtime: tokio::runtime::Runtime,
    queue: Arc<AccLogQueue>,
    stop: Arc<AtomicBool>,
    settings: LoginDatabaseSettings,
    notices: Sender<AccLogThreadNotice>,
) {
    while let Some(record) = queue.pop(&stop) {
        let kind = record_kind(&record);
        let sql = build_sql(record);
        let mut client = match runtime.block_on(connect_login_database(settings.clone())) {
            Ok(client) => client,
            Err(error) => {
                let _ = notices.send(AccLogThreadNotice::DatabaseFailure {
                    kind,
                    operation: AccLogDatabaseOperation::Connect,
                    error,
                });
                continue;
            }
        };
        if let Err(error) = runtime.block_on(client.execute(sql, &[])) {
            let _ = notices.send(AccLogThreadNotice::DatabaseFailure {
                kind,
                operation: AccLogDatabaseOperation::Execute,
                error: error.into(),
            });
            continue;
        }
        let _ = notices.send(AccLogThreadNotice::Executed { kind });
    }
}

fn record_kind(record: &AccountLogRecord) -> AccLogRecordKind {
    match record {
        AccountLogRecord::Enter(_) => AccLogRecordKind::AccountEnter,
        AccountLogRecord::RoleEnter(_) => AccLogRecordKind::RoleEnter,
        AccountLogRecord::SessionLeave(_) => AccLogRecordKind::SessionLeave,
        AccountLogRecord::Leave(_) => AccLogRecordKind::AccountLeave,
    }
}

fn build_sql(record: AccountLogRecord) -> String {
    let mut sql = Vec::new();
    match record {
        AccountLogRecord::Enter(record) => {
            append(
                &mut sql,
                b"INSERT INTO LogInfo(Account,AccountEnterTime,IP) VALUES('",
            );
            append(&mut sql, legacy_c_string_prefix(&record.account));
            append(&mut sql, b"','");
            append(&mut sql, &legacy_time(record.recorded_at));
            append(&mut sql, b"','");
            append(&mut sql, &format_ipv4(record.client_ip));
            append(&mut sql, b"')");
        }
        AccountLogRecord::RoleEnter(record) => {
            append(&mut sql, b"UPDATE LogInfo SET RoleEnterTime='");
            append(&mut sql, &legacy_time(record.recorded_at));
            append(&mut sql, b"', RoleName='");
            append(&mut sql, legacy_c_string_prefix(&record.role_name));
            append(&mut sql, b"', RoleLevel='");
            append(&mut sql, record.role_level.to_string().as_bytes());
            append(&mut sql, b"', WorldNumber='");
            append(&mut sql, record.world_number.to_string().as_bytes());
            append(
                &mut sql,
                b"' WHERE LogInfoID IN (SELECT TOP 1 LogInfoID FROM logInfo WHERE Account = '",
            );
            append(&mut sql, legacy_c_string_prefix(&record.account));
            append(&mut sql, b"' ORDER BY AccountEnterTime DESC)");
        }
        AccountLogRecord::SessionLeave(record) => {
            let time = legacy_time(record.recorded_at);
            append(&mut sql, b"UPDATE logInfo SET RoleLeaveTime = '");
            append(&mut sql, &time);
            append(&mut sql, b"', AccountLeaveTime = '");
            append(&mut sql, &time);
            append(
                &mut sql,
                b"' WHERE LogInfoID IN (SELECT TOP 1 LogInfoID FROM logInfo WHERE Account = '",
            );
            append(&mut sql, legacy_c_string_prefix(&record.account));
            append(&mut sql, b"' ORDER BY AccountEnterTime DESC)");
        }
        AccountLogRecord::Leave(record) => {
            append(&mut sql, b"UPDATE logInfo SET AccountLeaveTime = '");
            append(&mut sql, &legacy_time(record.recorded_at));
            append(
                &mut sql,
                b"' WHERE LogInfoID IN (SELECT TOP 1 LogInfoID FROM logInfo WHERE Account = '",
            );
            append(&mut sql, legacy_c_string_prefix(&record.account));
            append(&mut sql, b"' ORDER BY AccountEnterTime DESC)");
        }
    }
    let (sql, _, _) = WINDOWS_1251.decode(&sql);
    sql.into_owned()
}

fn legacy_time(value: NaiveDateTime) -> Vec<u8> {
    format!(
        "{}-{}-{} {}:{}:{}",
        value.year(),
        value.month(),
        value.day(),
        value.hour(),
        value.minute(),
        value.second()
    )
    .into_bytes()
}

fn append(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(value);
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    &value[..end]
}
