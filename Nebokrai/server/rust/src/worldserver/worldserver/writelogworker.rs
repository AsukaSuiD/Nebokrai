//! DB-граница `DoSaveLog` / `ProcessWriteLogDataFunc` исторического WorldServer.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\game.cpp:5088,5230`,
//! RVA `0x0000D770` и `0x000095E0`.
//!
//! `VERIFIED_DISASSEMBLY`: exact `0x004097C0..0x00409A2F` сначала делает
//! `Sleep(1)`, проверяет exit/use-log/queue, один раз снимает `GetSize`, затем
//! для каждого slot вызывает `PopWriteLogData` до `ExecuteCn`. При SQL failure
//! команда уже потеряна: `0x00409953` печатает её, `0x00409961..0x00409A1A`
//! восстанавливает connection, `0x00409A1E` освобождает старый SQL и цикл идёт
//! к следующему slot без повторного Execute. Ошибка reconnect по
//! `0x00409A34..0x00409A7C` ждёт 10 секунд и повторяет только подключение.
//! Новые producer-записи сверх исходного снимка остаются следующему batch.
//!
//! Rust сохраняет этот наблюдаемый FIFO/data-loss контракт typed batch-ом:
//! failure-команда возвращается как `discarded`, но в очередь не ставится;
//! после reconnect caller продолжает оставшиеся slot-ы того же снимка. Старый
//! Linux-донор с peek-until-commit, повтором SQL и классификацией transient
//! ошибок менял этот контракт и здесь не используется как истина.
//!
//! ADO connection заменён отдельным `tiberius` connection, а строковый INSERT
//! — параметризованным запросом. Windows-1251 payload декодируется перед bind;
//! это сохраняет штатные значения и не воспроизводит `_sprintf`, ручное SQL
//! quoting, stack overflow и injection через неэкранированный `context_id`.
//! Provider из setup является ADO plumbing и Tiberius-у не передаётся.
//!
//! Остаются `UNKNOWN` (исследовательский декомпилят хранится локально): внешний thread/exit owner, 1-ms polling, exact
//! operator-log публикация и отменяемая замена бесконечного 10-sec reconnect.
//! Здесь материализованы connection open, typed Execute и checkpoint для
//! точного batch/reconnect-порядка; runtime-owner должен собрать их вместе.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use encoding_rs::WINDOWS_1251;
use tiberius::Query;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::dbaccess::worlddb::rssetup::{WorldDatabaseSettings, WorldTdsClient};
use crate::worldserver::appworld::message::writelogmessage::WorldWriteLogCommand;

const INSERT_INCREMENT_LOG_SQL: &str = "INSERT INTO increment_log(\
    context_id,type,money,description,player_id,player_acc,player_lel,item_name,item_amount,ip_addr\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10)";
const INSERT_CARRIAGE_LOG_SQL: &str = "INSERT INTO carriage_log(\
    player_id,carriage_idx,carriage_region_id,carriage_coordinate_x,carriage_coordinate_y,event_type,event_time\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7)";
const INSERT_PLAIN_LOG_SQL: &str = "INSERT INTO log(\
    player_id,player_name,player_account,content,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5)";
const INSERT_CIQING_LOG_SQL: &str = "INSERT INTO ciqinglog(\
    dwplayerid,dwInOut,dwType,dwBaseIndex,dwAmount\
) VALUES(@P1,@P2,@P3,@P4,@P5)";

/// Cloneable FIFO-owner для producer-а главного цикла и отдельного DB worker-а.
///
/// `std::sync::Mutex` заменяет Win32 critical section; poisoned lock является
/// Rust-only отказом, поэтому очередь продолжает владеть уже принятыми данными.
#[derive(Clone, Default)]
pub(crate) struct WorldWriteLogQueue {
    commands: Arc<Mutex<VecDeque<WorldWriteLogCommand>>>,
}

impl WorldWriteLogQueue {
    pub(crate) fn push(&self, command: WorldWriteLogCommand) -> usize {
        let mut commands = self.lock();
        commands.push_back(command);
        commands.len()
    }

    pub(crate) fn pop(&self) -> Option<WorldWriteLogCommand> {
        self.lock().pop_front()
    }

    pub(crate) fn len(&self) -> usize {
        self.lock().len()
    }

    async fn process_batch(
        &self,
        connection: &mut WorldTdsClient,
        batch: Option<WorldWriteLogBatch>,
    ) -> WorldWriteLogBatchProgress {
        let mut batch = batch.unwrap_or_else(|| WorldWriteLogBatch::from_snapshot_size(self.len()));
        while batch.remaining_slots != 0 {
            batch.remaining_slots -= 1;
            let Some(command) = self.pop() else {
                batch.empty_slots += 1;
                continue;
            };
            match execute_world_write_log_command(connection, &command).await {
                Ok(()) => batch.executed += 1,
                Err(error) => {
                    return WorldWriteLogBatchProgress::ReconnectRequired {
                        batch,
                        discarded: WorldWriteLogDiscardedCommand { command, error },
                    };
                }
            }
        }
        WorldWriteLogBatchProgress::Complete(batch)
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<WorldWriteLogCommand>> {
        self.commands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Owned-вход отдельного worker-а: setup snapshot и cloneable FIFO уже
/// отделены от остального `CGame` и могут жить на другом системном потоке.
#[derive(Clone)]
pub(crate) struct WorldWriteLogWorkerSpec {
    enabled: bool,
    settings: WorldDatabaseSettings,
    queue: WorldWriteLogQueue,
}

impl WorldWriteLogWorkerSpec {
    pub(crate) const fn new(
        enabled: bool,
        settings: WorldDatabaseSettings,
        queue: WorldWriteLogQueue,
    ) -> Self {
        Self {
            enabled,
            settings,
            queue,
        }
    }

    pub(crate) const fn enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn queue_length(&self) -> usize {
        self.queue.len()
    }

    pub(crate) async fn open_connection(
        &self,
    ) -> Result<WorldTdsClient, WorldWriteLogConnectionError> {
        open_world_write_log_connection(&self.settings).await
    }

    pub(crate) async fn process_batch(
        &self,
        connection: &mut WorldTdsClient,
        batch: Option<WorldWriteLogBatch>,
    ) -> WorldWriteLogBatchProgress {
        self.queue.process_batch(connection, batch).await
    }
}

/// Ошибка отдельного Log DB connection-owner без раскрытия setup credentials.
#[derive(Debug)]
pub(crate) enum WorldWriteLogConnectionError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
}

impl fmt::Display for WorldWriteLogConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(formatter, "не открыто соединение Log DB: {error}"),
            Self::Tds(error) => write!(formatter, "ошибка TDS Log DB: {error}"),
        }
    }
}

impl Error for WorldWriteLogConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
        }
    }
}

/// Снимок исходного `GetSize` перед одним batch-проходом `DoSaveLog`.
#[derive(Debug)]
pub(crate) struct WorldWriteLogBatch {
    pub(crate) snapshot_size: usize,
    pub(crate) remaining_slots: usize,
    pub(crate) executed: usize,
    pub(crate) empty_slots: usize,
}

impl WorldWriteLogBatch {
    pub(crate) const fn from_snapshot_size(snapshot_size: usize) -> Self {
        Self {
            snapshot_size,
            remaining_slots: snapshot_size,
            executed: 0,
            empty_slots: 0,
        }
    }
}

/// Один SQL failure после уже выполненного `PopWriteLogData`.
///
/// Команда намеренно остаётся в отчёте, а не возвращается в очередь: exact
/// worker переподключался, но потерянный SQL повторно не исполнял.
pub(crate) struct WorldWriteLogDiscardedCommand {
    pub(crate) command: WorldWriteLogCommand,
    pub(crate) error: tiberius::error::Error,
}

impl fmt::Debug for WorldWriteLogDiscardedCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorldWriteLogDiscardedCommand")
            .field("command", &world_write_log_command_name(&self.command))
            .field("error", &self.error)
            .finish()
    }
}

/// Checkpoint batch-а: после failure caller обязан восстановить connection и
/// продолжить тот же `batch`, не повторяя отброшенную команду.
#[derive(Debug)]
pub(crate) enum WorldWriteLogBatchProgress {
    Complete(WorldWriteLogBatch),
    ReconnectRequired {
        batch: WorldWriteLogBatch,
        discarded: WorldWriteLogDiscardedCommand,
    },
}

/// Техническая замена ADO `CreateCn/Open`: отдельный Tiberius connection с теми
/// же server/database/user/password из LogSystem setup.
pub(crate) async fn open_world_write_log_connection(
    settings: &WorldDatabaseSettings,
) -> Result<WorldTdsClient, WorldWriteLogConnectionError> {
    let config = settings.tds_config();
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(WorldWriteLogConnectionError::Connect)?;
    tcp.set_nodelay(true)
        .map_err(WorldWriteLogConnectionError::Connect)?;
    tiberius::Client::connect(config, tcp.compat_write())
        .await
        .map_err(WorldWriteLogConnectionError::Tds)
}

/// Выполняет одну typed DB-команду параметризованным запросом.
pub(crate) async fn execute_world_write_log_command(
    connection: &mut WorldTdsClient,
    command: &WorldWriteLogCommand,
) -> Result<(), tiberius::error::Error> {
    match command {
        WorldWriteLogCommand::IncrementLog(record) => {
            let mut query = Query::new(INSERT_INCREMENT_LOG_SQL);
            query.bind(decode_legacy_text(&record.context_id));
            query.bind(i32::from(record.entry_type));
            query.bind(record.money);
            query.bind(decode_legacy_text(&record.description));
            query.bind(record.player_id);
            query.bind(decode_legacy_text(&record.player_account));
            query.bind(i32::from(record.player_level));
            query.bind(decode_legacy_text(&record.item_name));
            query.bind(record.item_amount);
            query.bind(decode_legacy_text(&record.ip_address));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::CarriageLog(record) => {
            let mut query = Query::new(INSERT_CARRIAGE_LOG_SQL);
            query.bind(record.player_id);
            query.bind(record.carriage_id);
            query.bind(record.region_id);
            query.bind(i32::from(record.coordinate_x));
            query.bind(i32::from(record.coordinate_y));
            query.bind(record.event_type);
            query.bind(format!(
                "{}-{}-{} {}:{}:{}",
                record.event_time.year,
                record.event_time.day_of_week,
                record.event_time.day,
                record.event_time.hour,
                record.event_time.minute,
                record.event_time.second,
            ));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::PlainLog(record) => {
            let mut query = Query::new(INSERT_PLAIN_LOG_SQL);
            query.bind(record.player_id);
            query.bind(decode_legacy_text(&record.player_name));
            query.bind(decode_legacy_text(&record.player_account));
            query.bind(decode_legacy_text(&record.content));
            query.bind(record.log_type);
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::CiqingLog(record) => {
            let mut query = Query::new(INSERT_CIQING_LOG_SQL);
            query.bind(record.player_id);
            query.bind(record.in_out);
            query.bind(record.entry_type);
            query.bind(record.base_index);
            query.bind(record.amount);
            query.execute(connection).await?;
            Ok(())
        }
    }
}

fn decode_legacy_text(bytes: &[u8]) -> String {
    WINDOWS_1251.decode(bytes).0.into_owned()
}

fn world_write_log_command_name(command: &WorldWriteLogCommand) -> &'static str {
    match command {
        WorldWriteLogCommand::IncrementLog(_) => "IncrementLog",
        WorldWriteLogCommand::CarriageLog(_) => "CarriageLog",
        WorldWriteLogCommand::PlainLog(_) => "PlainLog",
        WorldWriteLogCommand::CiqingLog(_) => "CiqingLog",
    }
}
