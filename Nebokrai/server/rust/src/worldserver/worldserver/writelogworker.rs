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
use crate::public::date::TagTime;
use crate::worldserver::appworld::message::writelogmessage::{
    WorldAuctionSaleLogEvent, WorldFairyLogEvent, WorldPlayerProgressLogEvent,
    WorldPlayerRelationLogEvent, WorldWriteLogCommand,
};

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
const INSERT_FAIRY_GROW_LOG_SQL: &str = "INSERT INTO fairy_grow_log(\
    playerid,IsJingPo,goodsid,goodsname,curlevel,growtime\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6)";
const INSERT_FAIRY_TAKE_LOG_SQL: &str = "INSERT INTO fairy_take_log(\
    player_id,goodsid,goodsname,cur_level,grow_rate,main_fetch,combinatedTimes,timer\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";
const INSERT_FAIRY_IMPLANTATION_LOG_SQL: &str = "INSERT INTO fairy_implantation_log(\
    playerid,goodsid,goodsname,prelevel,curlevel,westdiamond,im_time\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7)";
const INSERT_FAIRY_INCUBATE_LOG_SQL: &str = "INSERT INTO fairy_incubate_log(\
    playerid,goodsid,goodsname,inc_time\
) VALUES(@P1,@P2,@P3,@P4)";
const INSERT_FAIRY_SYNCRETIZE_LOG_SQL: &str = "INSERT INTO fairy_syncretize_log(\
    playerid,main_goodsid,main_goodsname,main_level,main_grow_rate,sec_goodsid,\
    sec_goodsname,sec_level,sec_grow_rate,west_patch,child_main_ability,child_goodsid,\
    child_goodsname,child_sy_times,child_row_rate,sy_time\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16)";
const INSERT_AUCTION_LOG_SQL: &str = "INSERT INTO AuctionLog(\
    dwBaseId,guidKey,opttype,moneytype,moneynum,amount,sxf,strdescri,guid,playerid,bNotice,log_time\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12)";
const INSERT_AUCTION_SALE_OPER_LOG_SQL: &str = "INSERT INTO AuctionSaleLog(\
    dwOpt,dwPlayerId,dwBaseIndex,guid,dwAmount,dwMoney,dwTimeType,dwFwf,date\
) VALUES('oper',@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";
const INSERT_AUCTION_SALE_CANCEL_LOG_SQL: &str = "INSERT INTO AuctionSaleLog(\
    dwOpt,dwPlayerId,guid,date\
) VALUES('cancel',@P1,@P2,@P3)";
const INSERT_AUCTION_SALE_RECEIVE_LOG_SQL: &str = "INSERT INTO AuctionSaleLog(\
    dwOpt,dwPlayerId,dwAmount,guid,date\
) VALUES('receive',@P1,@P2,@P3,@P4)";
const INSERT_PLAYER_LEVEL_LOG_SQL: &str = "INSERT INTO player_level_log(\
    player_id,player_name,exp,old_level,cur_level,map_id,pos_x,pos_y\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";
const INSERT_PLAYER_EXP_LOG_SQL: &str = "INSERT INTO player_exp_log(\
    player_id,player_name,exp,map_id,pos_x,pos_y,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7)";
const INSERT_PLAYER_DIED_LOG_SQL: &str = "INSERT INTO player_died_log(\
    player_id,player_name,map_id,pos_x,pos_y\
) VALUES(@P1,@P2,@P3,@P4,@P5)";
const INSERT_TEAM_LOG_SQL: &str = "INSERT INTO team_log(\
    captain_id,captain_name,player_id,player_name,map_id,pos_x,pos_y,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";
const INSERT_PLAYER_KILLER_LOG_SQL: &str = "INSERT INTO player_killer_log(\
    player_id,player_name,murderer_id,murderer_name,map_id,pos_x,pos_y,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";

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
            query.bind(legacy_log_event_time(record.event_time));
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
        WorldWriteLogCommand::FairyLog(record) => {
            let event_time = legacy_log_event_time(record.event_time);
            match &record.event {
                WorldFairyLogEvent::Grow {
                    is_jing_po,
                    goods_id,
                    goods_name,
                    current_level,
                } => {
                    let mut query = Query::new(INSERT_FAIRY_GROW_LOG_SQL);
                    query.bind(record.player_id);
                    query.bind(*is_jing_po);
                    query.bind(decode_legacy_text(goods_id));
                    query.bind(decode_legacy_text(goods_name));
                    query.bind(*current_level);
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
                WorldFairyLogEvent::Take {
                    goods_id,
                    goods_name,
                    current_level,
                    grow_rate_raw,
                    main_fetch,
                    combined_times,
                } => {
                    let mut query = Query::new(INSERT_FAIRY_TAKE_LOG_SQL);
                    query.bind(record.player_id);
                    query.bind(goods_id.to_string());
                    query.bind(decode_legacy_text(goods_name));
                    query.bind(*current_level);
                    query.bind(legacy_fairy_rate(*grow_rate_raw));
                    query.bind(*main_fetch);
                    query.bind(*combined_times);
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
                WorldFairyLogEvent::Implantation {
                    goods_name,
                    goods_id,
                    previous_level,
                    current_level,
                    west_diamond,
                } => {
                    let mut query = Query::new(INSERT_FAIRY_IMPLANTATION_LOG_SQL);
                    query.bind(record.player_id);
                    query.bind(goods_id.to_string());
                    query.bind(decode_legacy_text(goods_name));
                    query.bind(*previous_level);
                    query.bind(*current_level);
                    query.bind(*west_diamond);
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
                WorldFairyLogEvent::Incubate {
                    goods_id,
                    goods_name,
                } => {
                    let mut query = Query::new(INSERT_FAIRY_INCUBATE_LOG_SQL);
                    query.bind(record.player_id);
                    query.bind(goods_id.to_string());
                    query.bind(decode_legacy_text(goods_name));
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
                WorldFairyLogEvent::Syncretize {
                    main_goods_id,
                    main_goods_name,
                    main_level,
                    main_grow_rate_raw,
                    secondary_goods_id,
                    secondary_goods_name,
                    secondary_level,
                    secondary_grow_rate_raw,
                    west_patch,
                    child_goods_id,
                    child_goods_name,
                    child_main_ability,
                    child_syncretize_times,
                    child_grow_rate_raw,
                } => {
                    let mut query = Query::new(INSERT_FAIRY_SYNCRETIZE_LOG_SQL);
                    query.bind(record.player_id);
                    query.bind(main_goods_id.to_string());
                    query.bind(decode_legacy_text(main_goods_name));
                    query.bind(*main_level);
                    query.bind(legacy_fairy_rate(*main_grow_rate_raw));
                    query.bind(secondary_goods_id.to_string());
                    query.bind(decode_legacy_text(secondary_goods_name));
                    query.bind(*secondary_level);
                    query.bind(legacy_fairy_rate(*secondary_grow_rate_raw));
                    query.bind(*west_patch);
                    query.bind(*child_main_ability);
                    query.bind(child_goods_id.to_string());
                    query.bind(decode_legacy_text(child_goods_name));
                    query.bind(*child_syncretize_times);
                    query.bind(legacy_fairy_rate(*child_grow_rate_raw));
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
            }
            Ok(())
        }
        WorldWriteLogCommand::AuctionLog(write) => {
            let record = &write.record;
            let mut query = Query::new(INSERT_AUCTION_LOG_SQL);
            query.bind(record.base_id);
            query.bind(record.guid_key.to_string());
            query.bind(record.operation_type);
            query.bind(record.money_type);
            query.bind(record.money_num);
            query.bind(record.amount);
            query.bind(record.fee);
            query.bind(decode_legacy_c_text(&record.description));
            query.bind(record.guid.to_string());
            query.bind(record.player_id);
            query.bind(record.notice);
            query.bind(calendar_log_event_time(write.log_time));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::AuctionSaleLog(write) => {
            let event_time = calendar_log_event_time(write.event_time);
            match &write.event {
                WorldAuctionSaleLogEvent::Oper {
                    player_id,
                    base_index,
                    guid,
                    amount,
                    money,
                    time_type,
                    fee,
                } => {
                    let mut query = Query::new(INSERT_AUCTION_SALE_OPER_LOG_SQL);
                    query.bind(i64::from(*player_id));
                    query.bind(i64::from(*base_index));
                    query.bind(guid.to_string());
                    query.bind(i64::from(*amount));
                    query.bind(i64::from(*money));
                    query.bind(i64::from(*time_type));
                    query.bind(i64::from(*fee));
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
                WorldAuctionSaleLogEvent::Cancel { player_id, guid } => {
                    let mut query = Query::new(INSERT_AUCTION_SALE_CANCEL_LOG_SQL);
                    query.bind(i64::from(*player_id));
                    query.bind(guid.to_string());
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
                WorldAuctionSaleLogEvent::Receive {
                    player_id,
                    amount,
                    guid,
                } => {
                    let mut query = Query::new(INSERT_AUCTION_SALE_RECEIVE_LOG_SQL);
                    query.bind(i64::from(*player_id));
                    query.bind(i64::from(*amount));
                    query.bind(guid.to_string());
                    query.bind(event_time);
                    query.execute(connection).await?;
                }
            }
            Ok(())
        }
        WorldWriteLogCommand::PlayerProgressLog(write) => {
            match &write.event {
                WorldPlayerProgressLogEvent::Level {
                    experience,
                    old_level,
                    current_level,
                    map_id,
                    position_x,
                    position_y,
                } => {
                    let mut query = Query::new(INSERT_PLAYER_LEVEL_LOG_SQL);
                    query.bind(write.player_id);
                    query.bind(decode_legacy_text(&write.player_name));
                    query.bind(*experience);
                    query.bind(i32::from(*old_level));
                    query.bind(i32::from(*current_level));
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.execute(connection).await?;
                }
                WorldPlayerProgressLogEvent::Experience {
                    experience,
                    map_id,
                    position_x,
                    position_y,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_PLAYER_EXP_LOG_SQL);
                    query.bind(write.player_id);
                    query.bind(decode_legacy_text(&write.player_name));
                    query.bind(*experience);
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.bind(i32::from(*log_type));
                    query.execute(connection).await?;
                }
                WorldPlayerProgressLogEvent::Died {
                    map_id,
                    position_x,
                    position_y,
                } => {
                    let mut query = Query::new(INSERT_PLAYER_DIED_LOG_SQL);
                    query.bind(write.player_id);
                    query.bind(decode_legacy_text(&write.player_name));
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.execute(connection).await?;
                }
            }
            Ok(())
        }
        WorldWriteLogCommand::PlayerRelationLog(write) => {
            match &write.event {
                WorldPlayerRelationLogEvent::Team {
                    map_id,
                    wire_position_x: _,
                    position_y,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_TEAM_LOG_SQL);
                    query.bind(write.first_player_id);
                    query.bind(decode_legacy_text(&write.first_player_name));
                    query.bind(write.second_player_id);
                    query.bind(decode_legacy_text(&write.second_player_name));
                    query.bind(*map_id);
                    // Exact `_sprintf` передавал последний long для обеих координат.
                    query.bind(*position_y);
                    query.bind(*position_y);
                    query.bind(i32::from(*log_type));
                    query.execute(connection).await?;
                }
                WorldPlayerRelationLogEvent::Killer {
                    map_id,
                    position_x,
                    position_y,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_PLAYER_KILLER_LOG_SQL);
                    query.bind(write.first_player_id);
                    query.bind(decode_legacy_text(&write.first_player_name));
                    query.bind(write.second_player_id);
                    query.bind(decode_legacy_text(&write.second_player_name));
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.bind(i32::from(*log_type));
                    query.execute(connection).await?;
                }
            }
            Ok(())
        }
    }
}

/// Повторяет ошибочную подстановку `wDayOfWeek` на месте месяца.
fn legacy_log_event_time(time: TagTime) -> String {
    format!(
        "{}-{}-{} {}:{}:{}",
        time.year, time.day_of_week, time.day, time.hour, time.minute, time.second,
    )
}

fn calendar_log_event_time(time: TagTime) -> String {
    format!(
        "{}-{}-{} {}:{}:{}",
        time.year, time.month, time.day, time.hour, time.minute, time.second,
    )
}

/// Машина трактовала wire-long как unsigned, умножала на single `0.0001` и
/// печатала `%10.4f`; bind получает то же числовое значение после округления.
fn legacy_fairy_rate(raw: u32) -> f64 {
    let single = (f64::from(raw) * f64::from(0.0001_f32)) as f32;
    format!("{single:.4}")
        .parse()
        .expect("фиксированный формат конечного f32 обязан разбираться как f64")
}

fn decode_legacy_text(bytes: &[u8]) -> String {
    WINDOWS_1251.decode(bytes).0.into_owned()
}

fn decode_legacy_c_text(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    decode_legacy_text(&bytes[..end])
}

fn world_write_log_command_name(command: &WorldWriteLogCommand) -> &'static str {
    match command {
        WorldWriteLogCommand::IncrementLog(_) => "IncrementLog",
        WorldWriteLogCommand::CarriageLog(_) => "CarriageLog",
        WorldWriteLogCommand::PlainLog(_) => "PlainLog",
        WorldWriteLogCommand::CiqingLog(_) => "CiqingLog",
        WorldWriteLogCommand::FairyLog(_) => "FairyLog",
        WorldWriteLogCommand::AuctionLog(_) => "AuctionLog",
        WorldWriteLogCommand::AuctionSaleLog(_) => "AuctionSaleLog",
        WorldWriteLogCommand::PlayerProgressLog(_) => "PlayerProgressLog",
        WorldWriteLogCommand::PlayerRelationLog(_) => "PlayerRelationLog",
    }
}
