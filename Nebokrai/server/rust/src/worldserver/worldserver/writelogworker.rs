//! Worker журнала WorldServer с явным владением.
//!
//! Контракт очереди и reconnect-loop подтверждён точной парой WorldServer
//! `worldserver.exe`/`worldserver.pdb`. Worker извлекает SQL-команду до выполнения; при DB-ошибке уже
//! извлечённая команда теряется, соединение пересоздаётся, а очередь продолжает
//! работу с последующего элемента. Ошибка initial/reconnect connect повторяется
//! через исходный десятисекундный интервал; успешный idle loop сохраняет
//! миллисекундную cadence.
//!
//! `WorldWriteLogQueue` владеет командами, Tiberius заменяет ADO/COM, а
//! `JoinHandle`, atomics и `Condvar` заменяют Win32 thread/stop primitives.
//! Shutdown может прервать reconnect-ожидание и всегда присоединяет worker;
//! это безопасная техническая граница вместо зависания `Release`, не rollback
//! потерянной команды и не изменение SQL.

use std::error::Error;
use std::fmt;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use encoding_rs::WINDOWS_1251;
use tiberius::Query;
use tokio::net::TcpStream;
use tokio::runtime::Handle;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::dbaccess::worlddb::rssetup::{WorldDatabaseSettings, WorldTdsClient};
use crate::dbaccess::worlddb::writelogqueue::WorldWriteLogQueue;
use crate::public::date::TagTime;
use crate::worldserver::appworld::message::writelogmessage::{
    WorldAuctionSaleLogEvent, WorldFactionLogWrite, WorldFairyLogEvent,
    WorldGoodsCraftLogEvent, WorldPlayerProgressLogEvent, WorldPlayerRelationLogEvent,
    WorldWriteLogCommand,
};

const INSERT_INCREMENT_LOG_SQL: &str = "INSERT INTO increment_log(\
    context_id,type,money,description,player_id,player_acc,player_lel,item_name,item_amount,ip_addr\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10)";
const INSERT_LARGESS_LOG_SQL: &str = "INSERT INTO goods_largess_log(\
    cdkey,PlayerId,send_time,goods_id,goods_index,goods_name,goods_lel,goods_num,sent_num,cur_sent_num,res\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11)";
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
const INSERT_GOODS_TRADE_LOG_SQL: &str = "INSERT INTO goods_trade_log(\
    seller_id,seller_name,seller_cur_money,s_map_id,s_pos_x,s_pos_y,purchaser_id,purchaser_name,\
    purchaser_cur_money,p_map_id,p_pos_x,p_pos_y,goods_id,goods_name,price,amount,log_type,ip_addr_buyer,ip_addr_seller\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16,@P17,@P18,@P19)";
const INSERT_GOODS_LOG_SQL: &str = "INSERT INTO goods_log(\
    player_id,player_name,pk_count,cur_money,cur_bank,goods_id,goods_name,goods_num,price,map_id,pos_x,pos_y,log_type,ip_addr\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14)";
const INSERT_GOODS_UPGRADE_LOG_SQL: &str = "INSERT INTO goods_upgrade_log(\
    player_id,player_name,goods_id,goods_name,gem1_id,gem1_name,gem2_id,gem2_name,gem3_id,gem3_name,gem4_id,gem4_name,map_id,pos_x,pos_y,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12,@P13,@P14,@P15,@P16)";
const INSERT_GOODS_GEM_EXCHANGE_LOG_SQL: &str = "INSERT INTO goods_gem_exchange_log(\
    player_id,player_name,d_gem_id,d_gem_name,s_gem_id,s_gem_name,s_gem_amount,map_id,pos_x,pos_y\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10)";
const INSERT_GOODS_JEWELRY_MADE_LOG_SQL: &str = "INSERT INTO goods_jewelry_made_log(\
    player_id,player_name,goods_id,goods_name,material_id,material_name,jade_id,jade_name,jade_amount,map_id,pos_x,pos_y\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,@P12)";
const INSERT_CHAT_LOG_SQL: &str = "INSERT INTO chat_log(\
    sender_id,sender_name,map_id,pos_x,pos_y,receiver_id,receiver_name,content,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9)";
const INSERT_CHANGE_MAP_LOG_SQL: &str = "INSERT INTO change_map_log(\
    player_id,player_name,money,bank,s_map_id,s_pos_x,s_pos_y,d_map_id,d_pos_x,d_pos_y,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11)";
const INSERT_PLAYER_DELETE_LOG_SQL: &str = "INSERT INTO player_delete_log(\
    player_id,player_name,ip_addr\
) VALUES(@P1,@P2,@P3)";
const INSERT_FACTION_LOG_SQL: &str = "INSERT INTO faction_log(\
    faction_id,faction_name,player_id,player_name,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5)";
const INSERT_FACTION_MEMBER_LOG_SQL: &str = "INSERT INTO faction_member_log(\
    member_id,member_name,manager_id,manager_name,faction_id,faction_name,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7)";
const INSERT_FACTION_TITLE_LOG_SQL: &str = "INSERT INTO faction_title_log(\
    member_id,member_name,old_title,new_title,manager_id,manager_name,faction_id,faction_name\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";
const INSERT_FACTION_PURVIEW_LOG_SQL: &str = "INSERT INTO faction_purview_log(\
    member_id,member_name,purview,manager_id,manager_name,faction_id,faction_name,log_type\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8)";
const INSERT_FACTION_LEVEL_LOG_SQL: &str = "INSERT INTO faction_level_log(\
    faction_id,faction_name,lel,master_id,master_name\
) VALUES(@P1,@P2,@P3,@P4,@P5)";
const INSERT_FACTION_EXPERIENCE_LOG_SQL: &str = "INSERT INTO faction_experience_log(\
    faction_id,faction_name,member_id,member_name,before_exp,exp\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6)";
const INSERT_FACTION_MASTER_LOG_SQL: &str = "INSERT INTO faction_master_log(\
    old_master_id,old_master_name,new_master_id,new_master_name,faction_id,faction_name\
) VALUES(@P1,@P2,@P3,@P4,@P5,@P6)";
const WRITE_LOG_POLL_INTERVAL: Duration = Duration::from_millis(1);
const WRITE_LOG_RECONNECT_INTERVAL: Duration = Duration::from_secs(10);

impl WorldWriteLogQueue {
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

#[derive(Debug)]
pub(crate) enum WorldWriteLogWorkerEvent {
    InitialConnectionFailed(WorldWriteLogConnectionError),
    CommandDiscarded(WorldWriteLogDiscardedCommand),
    ReconnectStarted,
    ReconnectFailed(WorldWriteLogConnectionError),
    ReconnectSucceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldWriteLogWorkerReport {
    Disabled,
    InitialConnectionFailed,
    DrainedAndStopped {
        completed_batches: usize,
        executed: usize,
        discarded: usize,
    },
    ReconnectCancelled {
        completed_batches: usize,
        executed: usize,
        discarded: usize,
        queue_remaining: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldWriteLogWorkerCompletion {
    Returned(WorldWriteLogWorkerReport),
    Panicked,
}

pub(crate) struct WorldWriteLogWorker {
    signal: Arc<WorldWriteLogWorkerSignal>,
    handle: Option<JoinHandle<WorldWriteLogWorkerReport>>,
    events: Receiver<WorldWriteLogWorkerEvent>,
}

impl WorldWriteLogWorker {
    pub(crate) fn start(
        spec: WorldWriteLogWorkerSpec,
        runtime: Handle,
    ) -> Result<Self, io::Error> {
        let signal = Arc::new(WorldWriteLogWorkerSignal::default());
        let worker_signal = Arc::clone(&signal);
        let (event_sender, events) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("world-write-log".to_owned())
            .spawn(move || run_world_write_log_worker(spec, runtime, worker_signal, event_sender))?;
        Ok(Self {
            signal,
            handle: Some(handle),
            events,
        })
    }

    pub(crate) fn request_exit(&self) {
        self.signal.request_exit();
    }

    pub(crate) fn try_next_event(&self) -> Option<WorldWriteLogWorkerEvent> {
        match self.events.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }

    pub(crate) fn join(&mut self) -> Option<WorldWriteLogWorkerCompletion> {
        self.handle.take().map(join_world_write_log_worker)
    }
}

impl Drop for WorldWriteLogWorker {
    fn drop(&mut self) {
        self.request_exit();
        let _ = self.join();
    }
}

#[derive(Default)]
struct WorldWriteLogWorkerSignal {
    exit: AtomicBool,
    reconnect_wait: Mutex<()>,
    reconnect_wakeup: Condvar,
}

impl WorldWriteLogWorkerSignal {
    fn request_exit(&self) {
        self.exit.store(true, Ordering::Release);
        self.reconnect_wakeup.notify_all();
    }

    fn exit_requested(&self) -> bool {
        self.exit.load(Ordering::Acquire)
    }

    fn wait_reconnect_or_exit(&self) -> bool {
        if self.exit_requested() {
            return true;
        }
        let guard = self
            .reconnect_wait
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = self
            .reconnect_wakeup
            .wait_timeout_while(guard, WRITE_LOG_RECONNECT_INTERVAL, |_| {
                !self.exit_requested()
            })
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.exit_requested()
    }
}

fn join_world_write_log_worker(
    handle: JoinHandle<WorldWriteLogWorkerReport>,
) -> WorldWriteLogWorkerCompletion {
    match handle.join() {
        Ok(report) => WorldWriteLogWorkerCompletion::Returned(report),
        Err(_) => WorldWriteLogWorkerCompletion::Panicked,
    }
}

fn run_world_write_log_worker(
    spec: WorldWriteLogWorkerSpec,
    runtime: Handle,
    signal: Arc<WorldWriteLogWorkerSignal>,
    events: Sender<WorldWriteLogWorkerEvent>,
) -> WorldWriteLogWorkerReport {
    if !spec.enabled() {
        return WorldWriteLogWorkerReport::Disabled;
    }

    let mut connection = match runtime.block_on(spec.open_connection()) {
        Ok(connection) => Some(connection),
        Err(error) => {
            let _ = events.send(WorldWriteLogWorkerEvent::InitialConnectionFailed(error));
            return WorldWriteLogWorkerReport::InitialConnectionFailed;
        }
    };
    let mut completed_batches = 0usize;
    let mut completed_executed = 0usize;
    let mut discarded = 0usize;
    let mut pending_batch = None;

    loop {
        thread::sleep(WRITE_LOG_POLL_INTERVAL);
        if signal.exit_requested() && spec.queue_length() == 0 {
            return WorldWriteLogWorkerReport::DrainedAndStopped {
                completed_batches,
                executed: completed_executed,
                discarded,
            };
        }
        if pending_batch.is_none() && spec.queue_length() == 0 {
            continue;
        }

        let progress = runtime.block_on(spec.process_batch(
            connection
                .as_mut()
                .expect("соединение существует вне reconnect-участка"),
            pending_batch.take(),
        ));
        match progress {
            WorldWriteLogBatchProgress::Complete(batch) => {
                completed_batches += 1;
                completed_executed += batch.executed;
            }
            WorldWriteLogBatchProgress::ReconnectRequired {
                batch,
                discarded: failed,
            } => {
                discarded += 1;
                let _ = events.send(WorldWriteLogWorkerEvent::CommandDiscarded(failed));
                pending_batch = Some(batch);
                connection.take();
                let _ = events.send(WorldWriteLogWorkerEvent::ReconnectStarted);
                loop {
                    match runtime.block_on(spec.open_connection()) {
                        Ok(reconnected) => {
                            connection = Some(reconnected);
                            let _ = events.send(WorldWriteLogWorkerEvent::ReconnectSucceeded);
                            break;
                        }
                        Err(error) => {
                            let _ = events.send(WorldWriteLogWorkerEvent::ReconnectFailed(error));
                            if signal.wait_reconnect_or_exit() {
                                let in_progress = pending_batch
                                    .as_ref()
                                    .map_or(0, |batch| batch.executed);
                                return WorldWriteLogWorkerReport::ReconnectCancelled {
                                    completed_batches,
                                    executed: completed_executed + in_progress,
                                    discarded,
                                    queue_remaining: spec.queue_length(),
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

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
/// Команда намеренно остаётся в отчёте, а не возвращается в очередь:
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
        WorldWriteLogCommand::LargessLog(record) => {
            let mut query = Query::new(INSERT_LARGESS_LOG_SQL);
            query.bind(decode_legacy_c_text(&record.account));
            query.bind(record.player_id);
            query.bind(decode_legacy_c_text(&record.send_time));
            query.bind(decode_legacy_c_text(&record.goods_id));
            query.bind(record.goods_index as i32);
            query.bind(decode_legacy_c_text(&record.goods_name));
            query.bind(record.goods_level);
            query.bind(record.send_num);
            query.bind(record.sent_num);
            query.bind(record.current_sent_num);
            query.bind(decode_legacy_c_text(&record.result));
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
        WorldWriteLogCommand::AuctionNoticeSql(sql) => {
 // `CollectNoNotice` кладёт в общий FIFO уже собранный точный
 // UPDATE. Строка создаётся только owner-ом из CGuid/opttype, а
 // worker сохраняет его отдельный порядок и failure contract.
            Query::new(sql.as_str()).execute(connection).await?;
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
 // `_sprintf` передавал последний long для обеих координат.
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
        WorldWriteLogCommand::GoodsTradeLog(write) => {
            let mut query = Query::new(INSERT_GOODS_TRADE_LOG_SQL);
            query.bind(write.seller_id);
            query.bind(decode_legacy_text(&write.seller_name));
            query.bind(write.seller_current_money);
            query.bind(write.seller_map_id);
            query.bind(write.seller_position_x);
            query.bind(write.seller_position_y);
            query.bind(write.purchaser_id);
            query.bind(decode_legacy_text(&write.purchaser_name));
            query.bind(write.purchaser_current_money);
            query.bind(write.purchaser_map_id);
            query.bind(write.purchaser_position_x);
            query.bind(write.purchaser_position_y);
            query.bind(write.goods_id.to_string());
            query.bind(decode_legacy_text(&write.goods_name));
            query.bind(write.price);
            query.bind(write.amount);
            query.bind(i32::from(write.log_type));
            query.bind(decode_legacy_text(&write.buyer_ip_address));
            query.bind(decode_legacy_text(&write.seller_ip_address));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::GoodsLog(write) => {
            let mut query = Query::new(INSERT_GOODS_LOG_SQL);
            query.bind(write.player_id);
            query.bind(decode_legacy_text(&write.player_name));
            query.bind(i32::from(write.pk_count));
            query.bind(write.current_money);
            query.bind(write.current_bank);
            query.bind(write.goods_id.to_string());
            query.bind(decode_legacy_text(&write.goods_name));
            query.bind(write.goods_amount);
            query.bind(write.price);
            query.bind(write.map_id);
            query.bind(write.position_x);
            query.bind(write.position_y);
            query.bind(i32::from(write.log_type));
            query.bind(decode_legacy_text(&write.ip_address));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::GoodsCraftLog(write) => {
            match &write.event {
                WorldGoodsCraftLogEvent::Upgrade {
                    goods_id,
                    goods_name,
                    gems,
                    map_id,
                    position_x,
                    position_y,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_GOODS_UPGRADE_LOG_SQL);
                    query.bind(write.player_id);
                    query.bind(decode_legacy_text(&write.player_name));
                    query.bind(goods_id.to_string());
                    query.bind(decode_legacy_text(goods_name));
                    for (gem_id, gem_name) in gems {
                        query.bind(gem_id.to_string());
                        query.bind(decode_legacy_text(gem_name));
                    }
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.bind(i32::from(*log_type));
                    query.execute(connection).await?;
                }
                WorldGoodsCraftLogEvent::GemExchange {
                    destination_gem_id,
                    destination_gem_name,
                    source_gem_id,
                    source_gem_name,
                    source_gem_amount,
                    map_id,
                    position_x,
                    position_y,
                } => {
                    let mut query = Query::new(INSERT_GOODS_GEM_EXCHANGE_LOG_SQL);
                    query.bind(write.player_id);
                    query.bind(decode_legacy_text(&write.player_name));
                    query.bind(destination_gem_id.to_string());
                    query.bind(decode_legacy_text(destination_gem_name));
                    query.bind(source_gem_id.to_string());
                    query.bind(decode_legacy_text(source_gem_name));
                    query.bind(*source_gem_amount);
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.execute(connection).await?;
                }
                WorldGoodsCraftLogEvent::JewelryMade {
                    goods_id,
                    goods_name,
                    material_id,
                    material_name,
                    jade_id,
                    jade_name,
                    jade_amount,
                    map_id,
                    position_x,
                    position_y,
                } => {
                    let mut query = Query::new(INSERT_GOODS_JEWELRY_MADE_LOG_SQL);
                    query.bind(write.player_id);
                    query.bind(decode_legacy_text(&write.player_name));
                    query.bind(goods_id.to_string());
                    query.bind(decode_legacy_text(goods_name));
                    query.bind(material_id.to_string());
                    query.bind(decode_legacy_text(material_name));
                    query.bind(jade_id.to_string());
                    query.bind(decode_legacy_text(jade_name));
                    query.bind(*jade_amount);
                    query.bind(*map_id);
                    query.bind(*position_x);
                    query.bind(*position_y);
                    query.execute(connection).await?;
                }
            }
            Ok(())
        }
        WorldWriteLogCommand::ChatLog(write) => {
            let mut query = Query::new(INSERT_CHAT_LOG_SQL);
            query.bind(write.sender_id);
            query.bind(decode_legacy_text(&write.sender_name));
            query.bind(write.map_id);
            query.bind(write.position_x);
            query.bind(write.position_y);
            query.bind(write.receiver_id);
            query.bind(decode_legacy_text(&write.receiver_name));
            query.bind(decode_legacy_text(&write.content));
            query.bind(i32::from(write.log_type));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::LegacyEmptyChatSql { log_type: _ } => {
 // jump-table ставил очищенный `_Dest` в FIFO; ExecuteCn затем
 // исполнял именно пустую строку. Query сохраняет тот же DB-запрос,
 // оставляя transport-specific success/failure самому SQL Server.
            Query::new("").execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::ChangeMapLog(write) => {
            let mut query = Query::new(INSERT_CHANGE_MAP_LOG_SQL);
            query.bind(write.player_id);
            query.bind(decode_legacy_text(&write.player_name));
            query.bind(write.money);
            query.bind(write.bank);
            query.bind(write.source_map_id);
            query.bind(write.source_position_x);
            query.bind(write.source_position_y);
            query.bind(write.destination_map_id);
            query.bind(write.destination_position_x);
            query.bind(write.destination_position_y);
            query.bind(i32::from(write.log_type));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::PlayerDeleteLog(write) => {
            let mut query = Query::new(INSERT_PLAYER_DELETE_LOG_SQL);
            query.bind(write.player_id);
            query.bind(decode_legacy_text(&write.player_name));
            query.bind(decode_legacy_text(&write.ip_address));
            query.execute(connection).await?;
            Ok(())
        }
        WorldWriteLogCommand::FactionLog(write) => {
            match write {
                WorldFactionLogWrite::Faction {
                    faction_id,
                    faction_name,
                    player_id,
                    player_name,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_FACTION_LOG_SQL);
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.bind(*player_id);
                    query.bind(decode_legacy_text(player_name));
                    query.bind(*log_type);
                    query.execute(connection).await?;
                }
                WorldFactionLogWrite::Member {
                    member_id,
                    member_name,
                    manager_id,
                    manager_name,
                    faction_id,
                    faction_name,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_FACTION_MEMBER_LOG_SQL);
                    query.bind(*member_id);
                    query.bind(decode_legacy_text(member_name));
                    query.bind(*manager_id);
                    query.bind(decode_legacy_text(manager_name));
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.bind(*log_type);
                    query.execute(connection).await?;
                }
                WorldFactionLogWrite::Title {
                    member_id,
                    member_name,
                    old_title,
                    new_title,
                    manager_id,
                    manager_name,
                    faction_id,
                    faction_name,
                } => {
                    let mut query = Query::new(INSERT_FACTION_TITLE_LOG_SQL);
                    query.bind(*member_id);
                    query.bind(decode_legacy_text(member_name));
                    query.bind(decode_legacy_text(old_title));
                    query.bind(decode_legacy_text(new_title));
                    query.bind(*manager_id);
                    query.bind(decode_legacy_text(manager_name));
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.execute(connection).await?;
                }
                WorldFactionLogWrite::Purview {
                    member_id,
                    member_name,
                    purview,
                    manager_id,
                    manager_name,
                    faction_id,
                    faction_name,
                    log_type,
                } => {
                    let mut query = Query::new(INSERT_FACTION_PURVIEW_LOG_SQL);
                    query.bind(*member_id);
                    query.bind(decode_legacy_text(member_name));
                    query.bind(*purview);
                    query.bind(*manager_id);
                    query.bind(decode_legacy_text(manager_name));
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.bind(*log_type);
                    query.execute(connection).await?;
                }
                WorldFactionLogWrite::Level {
                    faction_id,
                    faction_name,
                    level,
                    master_id,
                    master_name,
                } => {
                    let mut query = Query::new(INSERT_FACTION_LEVEL_LOG_SQL);
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.bind(*level);
                    query.bind(*master_id);
                    query.bind(decode_legacy_text(master_name));
                    query.execute(connection).await?;
                }
                WorldFactionLogWrite::Experience {
                    faction_id,
                    faction_name,
                    member_id,
                    member_name,
                    before_experience,
                    experience,
                } => {
                    let mut query = Query::new(INSERT_FACTION_EXPERIENCE_LOG_SQL);
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.bind(*member_id);
                    query.bind(decode_legacy_text(member_name));
                    query.bind(*before_experience);
                    query.bind(*experience);
                    query.execute(connection).await?;
                }
                WorldFactionLogWrite::Master {
                    old_master_id,
                    old_master_name,
                    new_master_id,
                    new_master_name,
                    faction_id,
                    faction_name,
                } => {
                    let mut query = Query::new(INSERT_FACTION_MASTER_LOG_SQL);
                    query.bind(*old_master_id);
                    query.bind(decode_legacy_text(old_master_name));
                    query.bind(*new_master_id);
                    query.bind(decode_legacy_text(new_master_name));
                    query.bind(*faction_id);
                    query.bind(decode_legacy_text(faction_name));
                    query.execute(connection).await?;
                }
            }
            Ok(())
        }
    }
}

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
        WorldWriteLogCommand::LargessLog(_) => "LargessLog",
        WorldWriteLogCommand::CarriageLog(_) => "CarriageLog",
        WorldWriteLogCommand::PlainLog(_) => "PlainLog",
        WorldWriteLogCommand::CiqingLog(_) => "CiqingLog",
        WorldWriteLogCommand::FairyLog(_) => "FairyLog",
        WorldWriteLogCommand::AuctionLog(_) => "AuctionLog",
        WorldWriteLogCommand::AuctionSaleLog(_) => "AuctionSaleLog",
        WorldWriteLogCommand::AuctionNoticeSql(_) => "AuctionNoticeSql",
        WorldWriteLogCommand::PlayerProgressLog(_) => "PlayerProgressLog",
        WorldWriteLogCommand::PlayerRelationLog(_) => "PlayerRelationLog",
        WorldWriteLogCommand::GoodsTradeLog(_) => "GoodsTradeLog",
        WorldWriteLogCommand::GoodsLog(_) => "GoodsLog",
        WorldWriteLogCommand::GoodsCraftLog(_) => "GoodsCraftLog",
        WorldWriteLogCommand::ChatLog(_) => "ChatLog",
        WorldWriteLogCommand::LegacyEmptyChatSql { .. } => "LegacyEmptyChatSql",
        WorldWriteLogCommand::ChangeMapLog(_) => "ChangeMapLog",
        WorldWriteLogCommand::PlayerDeleteLog(_) => "PlayerDeleteLog",
        WorldWriteLogCommand::FactionLog(_) => "FactionLog",
    }
}
