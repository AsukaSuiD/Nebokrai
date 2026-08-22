//! WorldServer dispatcher-owner `OnWriteLogMessage`.
//!
//! Весь dispatcher RVA `0x000A8AB0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме
//! player progress `0x60206..0x60208`, team/killer `0x60209..0x6020A`,
//! chat/change-map `0x6020B..0x6020C`, increment-shop `0x6020D`, carriage
//! `0x6020E`, plain player log `0x6020F`, fairy `0x60210`, reserved no-op
//! `0x60211..0x60213`, auction
//! `0x60214..0x60217` и ciqing `0x60218` со статусом `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`; исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\writelogmessage.cpp:18`.
//!
//! Ветка читает `char type`, строки с exact границами `0x200/0x100`, signed
//! money/player, а только для type `0` — item-name `0x80` и signed amount;
//! последний `long` форматируется как IPv4 в порядке младшего байта первым.
//! Amount `>= 1001` не ставит DB-write и не публикует live-запись, а пишет
//! исходный operator/file error. В обычной ветке SQL сначала попадал в
//! `CWriteLogQueue`, затем тот же caller-time немедленно публиковался в
//! `CIncrementLog`; DB commit не ожидался. Это наблюдаемое расхождение со
//! старым Linux-донором, который перенёс Add в commit callback, не перенесено.
//! `VERIFIED_DISASSEMBLY`: exact `0x004A9D3D..0x004AA04F` обнуляет account и
//! level, при найденном player копирует account и берёт byte `[player+0x59C]`,
//! проверяет signed amount через `cmp 0x3E8/jle`, вызывает `GetLocalTime`,
//! затем `PushWriteLogData` `0x004AA023` и только после него `CIncrementLog::Add`
//! `0x004AA048`. Null player поэтому сохраняет пустой account и level `0`.
//! Ветка carriage `0x6020E` по exact `0x004AA07A..0x004AA127` читает три
//! signed long, два signed short и ещё один signed long, затем снимает один
//! `SYSTEMTIME` и ставит INSERT в общий FIFO. Машинный `_sprintf` неожиданно
//! подставляет `wDayOfWeek`, а не `wMonth`: event-time имеет вид
//! `year-weekday-day hour:minute:second`. Это DB-наблюдаемый quirk сохранён;
//! Linux-донор исправлял его на обычную календарную дату без доказательства.
//! Ветка plain log `0x6020F` по exact `0x004AA12C..0x004AA207` читает signed
//! player ID/log type и строку с границей `0x100`. Найденный player даёт полные
//! visible name/account; null lookup буквально пишет `"null"` в оба поля.
//! Только content проходил `CGame::CheckPoint`; Tiberius bind сохраняет то же
//! штатное значение без ручного escaping. Donor-truncation `32/32/255` в
//! машине отсутствует и не перенесена. Неэкранированные name/account оригинала
//! также bind-ятся как данные: SQL breakage/injection не является контрактом.
//! Ветка fairy `0x60210` по exact `0x004AA20C..0x004AA850` снимает время до
//! заголовка, затем читает type/player и до subtype payload требует online
//! player. Ошибки сохраняют исходные тексты `err:palyerid:%d is not online` и
//! `fairy log ! err type:%d`; unsigned jump-table принимает только type `0..4`.
//! Пять вариантов grow/take/implantation/incubate/syncretize сохраняют exact
//! wire-порядок, GUID marker и границы строк `0x40/0x20`. Grow-rate читается как
//! unsigned 32-bit, умножается на single `0.0001` и попадает в SQL с четырьмя
//! знаками. Fairy-time сохраняет тот же наблюдаемый weekday-вместо-month quirk,
//! что carriage. `CheckPoint` и неэкранированный grow goods-id заменены bind:
//! SQL injection/breakage были внутренним дефектом, не контрактом Miracle.
//! Auction `0x60214` по exact `0x004AA88C..0x004AA9BD` копирует wire-node
//! `0x150`, заменяет только `guidKey` новым системным GUID, ставит signed INSERT
//! в FIFO и сразу после enqueue публикует тот же node в live `CAuctionLog`.
//! SQL получает normal `year-month-day` caller-time, а live node сохраняет
//! собственный wire `SYSTEMTIME`. Donor-валидация полей, принудительный
//! `bNotice=0` и commit-before-live меняли этот контракт и не перенесены.
//! Sale `0x60215..0x60217` по exact `0x004AA9C2..0x004AAC24` сохраняет три
//! разных payload-порядка и `%u`-трактовку всех long. Отсутствие NUL в raw
//! `strDescri[256]` больше не даёт читать память за node: owned bytes и bind
//! устраняют только внутренний memory/SQL defect. Если системный генератор GUID
//! откажет, точное содержимое старого out-buffer неизвестно и проход явно
//! возвращает safe block без придуманной DB/live записи.
//! Player progress `0x60206..0x60208` по exact
//! `0x004A94AC..0x004A96CB` читает level/experience/death payload в трёх
//! разных порядках. Player ID и long-поля попадают в SQL как signed `%d`, а
//! оба level и log-type расширяются из char через `movzx`; отсутствующий player
//! даёт буквальное имя `"NULL"`. Старый Linux-донор ошибочно сменил player ID
//! на `%u`; это расхождение не перенесено. Параметризация исправляет только
//! неэкранированное имя и не добавляет отсутствующую в машине валидацию.
//! Team/killer `0x60209..0x6020A` по exact `0x004A96CB..0x004A9909` независимо
//! lookup-ят обоих игроков, подставляя `"NULL"` для каждого отсутствующего, и
//! расширяют log-type через `movzx`. Killer сохраняет обе wire-координаты.
//! Team читает обе, но machine `_sprintf` дважды передаёт последний `pos_y`,
//! поэтому DB `pos_x == pos_y`; этот наблюдаемый quirk сохранён явно, тогда как
//! Linux-донор молча использовал прочитанный `pos_x`.
//! Chat/change-map `0x6020B..0x6020C` по exact
//! `0x004A9909..0x004A9D38` используют signed long и unsigned chat/log type.
//! Chat отбрасывает пустой content до enqueue; private type `5` единственный
//! дочитывает receiver ID, остальные подтверждённые типы получают exact метки
//! `<public>/<Region>/<team>/<GM-code>/<world>/<country>`. Jump-table типов
//! `2`, `3` и значений вне `0..8` неожиданно попадает прямо в общий enqueue с
//! пустым SQL-buffer. `PushWriteLogData` `0x004ECC40` пустую строку не
//! фильтрует, поэтому этот DB-worker quirk сохранён отдельной typed-командой;
//! Linux-донор ошибочно делал ранний `return`. Параметризация заменила только
//! `FixSingleQuotes` и неэкранированные имена. Change-map сохраняет обычный
//! wire-порядок source/destination координат и не добавляет валидацию.
//! Exact outer jump-table VA `0x004AACA8` направляет все три wire ID
//! `0x60211..0x60213` прямо в epilogue `0x004AAC87`; Rust поэтому считает их
//! обработанными no-op, не создавая ложный pending owner. Ветка `0x60218` по
//! `0x004AAC26..0x004AAC82` читает пять signed long и без иных side effects
//! ставит `ciqinglog` INSERT в тот же FIFO.
//!
//! Rust хранит параметризуемую DB-команду вместо SQL-строки: будущий Tiberius
//! worker не должен повторять `_sprintf`, ручное quoting и stack buffers.
//! Значения полей, FIFO-позиция и enqueue-before-publish сохраняются. Donor-
//! added полная tail-validation и меньшие лимиты `32/255/32` отсутствуют в
//! EXE. Безопасные owned bytes также исправляют только внутренние переполнения
//! временных `account/CheckPoint` buffers, не меняя штатные значения.

use std::net::Ipv4Addr;

use crate::nets::networld::message::CMessage;
use crate::public::auctionlog::{AuctionLogNode, AuctionLogSystemTime, CAuctionLog};
use crate::public::date::TagTime;
use crate::public::guid::CGuid;
use crate::public::tools::put_string_to_file;
use crate::worldserver::appworld::incrementlog::incrementlog::CIncrementLog;
use crate::worldserver::worldserver::game::CGame;
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

const PLAYER_LEVEL_LOG_MESSAGE: i32 = 0x0006_0206;
const PLAYER_EXP_LOG_MESSAGE: i32 = 0x0006_0207;
const PLAYER_DIED_LOG_MESSAGE: i32 = 0x0006_0208;
const TEAM_LOG_MESSAGE: i32 = 0x0006_0209;
const PLAYER_KILLER_LOG_MESSAGE: i32 = 0x0006_020A;
const CHAT_LOG_MESSAGE: i32 = 0x0006_020B;
const CHANGE_MAP_LOG_MESSAGE: i32 = 0x0006_020C;
const INCREMENT_LOG_MESSAGE: i32 = 0x0006_020D;
const CARRIAGE_LOG_MESSAGE: i32 = 0x0006_020E;
const PLAIN_LOG_MESSAGE: i32 = 0x0006_020F;
const FAIRY_LOG_MESSAGE: i32 = 0x0006_0210;
const RESERVED_WRITE_LOG_MESSAGES: std::ops::RangeInclusive<i32> =
    0x0006_0211..=0x0006_0213;
const AUCTION_LOG_MESSAGE: i32 = 0x0006_0214;
const AUCTION_SALE_OPER_LOG_MESSAGE: i32 = 0x0006_0215;
const AUCTION_SALE_CANCEL_LOG_MESSAGE: i32 = 0x0006_0216;
const AUCTION_SALE_RECEIVE_LOG_MESSAGE: i32 = 0x0006_0217;
const CIQING_LOG_MESSAGE: i32 = 0x0006_0218;

/// Параметры одной исходной INSERT-команды без самодельного SQL quoting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldIncrementLogWrite {
    pub(crate) context_id: Vec<u8>,
    pub(crate) entry_type: u8,
    pub(crate) money: i32,
    pub(crate) description: Vec<u8>,
    pub(crate) player_id: i32,
    pub(crate) player_account: Vec<u8>,
    pub(crate) player_level: u8,
    pub(crate) item_name: Vec<u8>,
    pub(crate) item_amount: i32,
    pub(crate) ip_address: Vec<u8>,
}

/// Поля `carriage_log`, включая SYSTEMTIME, снятый до enqueue.
#[derive(Clone, Debug)]
pub(crate) struct WorldCarriageLogWrite {
    pub(crate) player_id: i32,
    pub(crate) carriage_id: i32,
    pub(crate) region_id: i32,
    pub(crate) coordinate_x: i16,
    pub(crate) coordinate_y: i16,
    pub(crate) event_type: i32,
    pub(crate) event_time: TagTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlainLogWrite {
    pub(crate) player_id: i32,
    pub(crate) player_name: Vec<u8>,
    pub(crate) player_account: Vec<u8>,
    pub(crate) content: Vec<u8>,
    pub(crate) log_type: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCiqingLogWrite {
    pub(crate) player_id: i32,
    pub(crate) in_out: i32,
    pub(crate) entry_type: i32,
    pub(crate) base_index: i32,
    pub(crate) amount: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct WorldFairyLogWrite {
    pub(crate) player_id: i32,
    pub(crate) event_time: TagTime,
    pub(crate) event: WorldFairyLogEvent,
}

#[derive(Clone, Debug)]
pub(crate) enum WorldFairyLogEvent {
    Grow {
        is_jing_po: i32,
        goods_id: Vec<u8>,
        goods_name: Vec<u8>,
        current_level: i32,
    },
    Take {
        goods_id: CGuid,
        goods_name: Vec<u8>,
        current_level: i32,
        grow_rate_raw: u32,
        main_fetch: i32,
        combined_times: i32,
    },
    Implantation {
        goods_name: Vec<u8>,
        goods_id: CGuid,
        previous_level: i32,
        current_level: i32,
        west_diamond: i32,
    },
    Incubate {
        goods_id: CGuid,
        goods_name: Vec<u8>,
    },
    Syncretize {
        main_goods_id: CGuid,
        main_goods_name: Vec<u8>,
        main_level: i32,
        main_grow_rate_raw: u32,
        secondary_goods_id: CGuid,
        secondary_goods_name: Vec<u8>,
        secondary_level: i32,
        secondary_grow_rate_raw: u32,
        west_patch: i32,
        child_goods_id: CGuid,
        child_goods_name: Vec<u8>,
        child_main_ability: i32,
        child_syncretize_times: i32,
        child_grow_rate_raw: u32,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct WorldAuctionLogWrite {
    pub(crate) record: AuctionLogNode,
    pub(crate) log_time: TagTime,
}

#[derive(Clone, Debug)]
pub(crate) struct WorldAuctionSaleLogWrite {
    pub(crate) event_time: TagTime,
    pub(crate) event: WorldAuctionSaleLogEvent,
}

#[derive(Clone, Debug)]
pub(crate) enum WorldAuctionSaleLogEvent {
    Oper {
        player_id: u32,
        base_index: u32,
        guid: CGuid,
        amount: u32,
        money: u32,
        time_type: u32,
        fee: u32,
    },
    Cancel {
        player_id: u32,
        guid: CGuid,
    },
    Receive {
        player_id: u32,
        amount: u32,
        guid: CGuid,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerProgressLogWrite {
    pub(crate) player_id: i32,
    pub(crate) player_name: Vec<u8>,
    pub(crate) event: WorldPlayerProgressLogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerProgressLogEvent {
    Level {
        experience: i32,
        old_level: u8,
        current_level: u8,
        map_id: i32,
        position_x: i32,
        position_y: i32,
    },
    Experience {
        experience: i32,
        map_id: i32,
        position_x: i32,
        position_y: i32,
        log_type: u8,
    },
    Died {
        map_id: i32,
        position_x: i32,
        position_y: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerRelationLogWrite {
    pub(crate) first_player_id: i32,
    pub(crate) first_player_name: Vec<u8>,
    pub(crate) second_player_id: i32,
    pub(crate) second_player_name: Vec<u8>,
    pub(crate) event: WorldPlayerRelationLogEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldPlayerRelationLogEvent {
    Team {
        map_id: i32,
        wire_position_x: i32,
        position_y: i32,
        log_type: u8,
    },
    Killer {
        map_id: i32,
        position_x: i32,
        position_y: i32,
        log_type: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldChatLogWrite {
    pub(crate) sender_id: i32,
    pub(crate) sender_name: Vec<u8>,
    pub(crate) map_id: i32,
    pub(crate) position_x: i32,
    pub(crate) position_y: i32,
    pub(crate) receiver_id: i32,
    pub(crate) receiver_name: Vec<u8>,
    pub(crate) content: Vec<u8>,
    pub(crate) log_type: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldChangeMapLogWrite {
    pub(crate) player_id: i32,
    pub(crate) player_name: Vec<u8>,
    pub(crate) money: i32,
    pub(crate) bank: i32,
    pub(crate) source_map_id: i32,
    pub(crate) source_position_x: i32,
    pub(crate) source_position_y: i32,
    pub(crate) destination_map_id: i32,
    pub(crate) destination_position_x: i32,
    pub(crate) destination_position_y: i32,
    pub(crate) log_type: u8,
}

/// Typed очередь сохраняет старый FIFO, но оставляет SQL transport Tiberius-у.
#[derive(Clone, Debug)]
pub(crate) enum WorldWriteLogCommand {
    IncrementLog(WorldIncrementLogWrite),
    CarriageLog(WorldCarriageLogWrite),
    PlainLog(WorldPlainLogWrite),
    CiqingLog(WorldCiqingLogWrite),
    FairyLog(WorldFairyLogWrite),
    AuctionLog(WorldAuctionLogWrite),
    AuctionSaleLog(WorldAuctionSaleLogWrite),
    PlayerProgressLog(WorldPlayerProgressLogWrite),
    PlayerRelationLog(WorldPlayerRelationLogWrite),
    ChatLog(WorldChatLogWrite),
    /// Exact chat jump-table ставил в FIFO очищенный SQL-buffer.
    LegacyEmptyChatSql { log_type: u8 },
    ChangeMapLog(WorldChangeMapLogWrite),
}

#[derive(Debug)]
pub(crate) enum WorldIncrementLogMessageOutcome {
    Queued {
        record: WorldIncrementLogWrite,
        time: TagTime,
        payload_complete: [bool; 8],
        queue_length_after: usize,
        live_published: bool,
    },
    RejectedItemAmount {
        player_id: i32,
        player_account: Vec<u8>,
        item_name: Vec<u8>,
        item_amount: i32,
        payload_complete: [bool; 8],
        operator_log: AddLogTextDisposition,
        file_text: Vec<u8>,
    },
}

#[derive(Debug)]
pub(crate) struct WorldCarriageLogMessageOutcome {
    pub(crate) record: WorldCarriageLogWrite,
    pub(crate) payload_complete: [bool; 6],
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) struct WorldPlainLogMessageOutcome {
    pub(crate) record: WorldPlainLogWrite,
    pub(crate) player_found: bool,
    pub(crate) payload_complete: [bool; 3],
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) struct WorldCiqingLogMessageOutcome {
    pub(crate) record: WorldCiqingLogWrite,
    pub(crate) payload_complete: [bool; 5],
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) enum WorldFairyPayloadCompleteness {
    Grow([bool; 4]),
    Take([bool; 6]),
    Implantation([bool; 5]),
    Incubate([bool; 2]),
    Syncretize([bool; 14]),
}

#[derive(Debug)]
pub(crate) enum WorldFairyLogMessageOutcome {
    Queued {
        record: WorldFairyLogWrite,
        header_complete: [bool; 2],
        payload_complete: WorldFairyPayloadCompleteness,
        queue_length_after: usize,
    },
    PlayerOffline {
        fairy_type: i32,
        player_id: i32,
        event_time: TagTime,
        header_complete: [bool; 2],
        operator_log: AddLogTextDisposition,
    },
    InvalidType {
        fairy_type: i32,
        player_id: i32,
        event_time: TagTime,
        header_complete: [bool; 2],
        operator_log: AddLogTextDisposition,
    },
}

#[derive(Debug)]
pub(crate) enum WorldAuctionLogMessageOutcome {
    Queued {
        write: WorldAuctionLogWrite,
        payload_complete: bool,
        queue_length_after: usize,
        live_published: bool,
    },
    BlockedGuidGeneration {
        record: AuctionLogNode,
        log_time: TagTime,
        payload_complete: bool,
    },
}

#[derive(Debug)]
pub(crate) enum WorldAuctionSalePayloadCompleteness {
    Oper([bool; 7]),
    Cancel([bool; 2]),
    Receive([bool; 3]),
}

#[derive(Debug)]
pub(crate) struct WorldAuctionSaleLogMessageOutcome {
    pub(crate) write: WorldAuctionSaleLogWrite,
    pub(crate) payload_complete: WorldAuctionSalePayloadCompleteness,
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) enum WorldPlayerProgressPayloadCompleteness {
    Level([bool; 7]),
    Experience([bool; 6]),
    Died([bool; 4]),
}

#[derive(Debug)]
pub(crate) struct WorldPlayerProgressLogMessageOutcome {
    pub(crate) write: WorldPlayerProgressLogWrite,
    pub(crate) player_found: bool,
    pub(crate) payload_complete: WorldPlayerProgressPayloadCompleteness,
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) struct WorldPlayerRelationLogMessageOutcome {
    pub(crate) write: WorldPlayerRelationLogWrite,
    pub(crate) players_found: [bool; 2],
    pub(crate) payload_complete: [bool; 6],
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) enum WorldChatPayloadCompleteness {
    FixedReceiver([bool; 6]),
    PrivateReceiver([bool; 7]),
}

#[derive(Debug)]
pub(crate) enum WorldChatLogMessageOutcome {
    Queued {
        write: WorldChatLogWrite,
        sender_found: bool,
        receiver_found: Option<bool>,
        payload_complete: WorldChatPayloadCompleteness,
        queue_length_after: usize,
    },
    EmptyContent {
        log_type: u8,
        sender_id: i32,
        sender_found: bool,
        payload_complete: [bool; 6],
    },
    LegacyEmptySqlQueued {
        log_type: u8,
        sender_id: i32,
        sender_found: bool,
        payload_complete: [bool; 6],
        queue_length_after: usize,
    },
}

#[derive(Debug)]
pub(crate) struct WorldChangeMapLogMessageOutcome {
    pub(crate) write: WorldChangeMapLogWrite,
    pub(crate) player_found: bool,
    pub(crate) payload_complete: [bool; 10],
    pub(crate) queue_length_after: usize,
}

#[derive(Debug)]
pub(crate) enum WorldWriteLogMessageOutcome {
    IncrementLog(WorldIncrementLogMessageOutcome),
    CarriageLog(WorldCarriageLogMessageOutcome),
    PlainLog(WorldPlainLogMessageOutcome),
    CiqingLog(WorldCiqingLogMessageOutcome),
    FairyLog(WorldFairyLogMessageOutcome),
    AuctionLog(WorldAuctionLogMessageOutcome),
    AuctionSaleLog(WorldAuctionSaleLogMessageOutcome),
    PlayerProgressLog(WorldPlayerProgressLogMessageOutcome),
    PlayerRelationLog(WorldPlayerRelationLogMessageOutcome),
    ChatLog(WorldChatLogMessageOutcome),
    ChangeMapLog(WorldChangeMapLogMessageOutcome),
    ReservedNoOp { message_type: i32 },
}

pub(crate) enum WorldWriteLogMessageDispatch {
    Handled(WorldWriteLogMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутые write-log ветки и exact reserved no-op IDs.
pub(crate) fn on_write_log_message(
    game: &mut CGame,
    increment_log: &mut CIncrementLog,
    auction_log: &mut CAuctionLog,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldWriteLogMessageDispatch {
    let dispatch_time = TagTime::local_now();
    if RESERVED_WRITE_LOG_MESSAGES.contains(&message.message_type()) {
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::ReservedNoOp {
                message_type: message.message_type(),
            },
        );
    }
    if message.message_type() == CIQING_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(WorldWriteLogMessageOutcome::CiqingLog(
            on_ciqing_log_message(game, message),
        ));
    }
    if message.message_type() == FAIRY_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(WorldWriteLogMessageOutcome::FairyLog(
            on_fairy_log_message(game, add_log_text, message),
        ));
    }
    if message.message_type() == AUCTION_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(WorldWriteLogMessageOutcome::AuctionLog(
            on_auction_log_message(game, auction_log, dispatch_time, message),
        ));
    }
    if matches!(
        message.message_type(),
        AUCTION_SALE_OPER_LOG_MESSAGE
            | AUCTION_SALE_CANCEL_LOG_MESSAGE
            | AUCTION_SALE_RECEIVE_LOG_MESSAGE
    ) {
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::AuctionSaleLog(on_auction_sale_log_message(
                game,
                dispatch_time,
                message,
            )),
        );
    }
    if matches!(
        message.message_type(),
        PLAYER_LEVEL_LOG_MESSAGE | PLAYER_EXP_LOG_MESSAGE | PLAYER_DIED_LOG_MESSAGE
    ) {
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::PlayerProgressLog(
                on_player_progress_log_message(game, message),
            ),
        );
    }
    if matches!(
        message.message_type(),
        TEAM_LOG_MESSAGE | PLAYER_KILLER_LOG_MESSAGE
    ) {
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::PlayerRelationLog(
                on_player_relation_log_message(game, message),
            ),
        );
    }
    if message.message_type() == CHAT_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(WorldWriteLogMessageOutcome::ChatLog(
            on_chat_log_message(game, message),
        ));
    }
    if message.message_type() == CHANGE_MAP_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::ChangeMapLog(on_change_map_log_message(game, message)),
        );
    }
    if message.message_type() == PLAIN_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(WorldWriteLogMessageOutcome::PlainLog(
            on_plain_log_message(game, message),
        ));
    }
    if message.message_type() == CARRIAGE_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::CarriageLog(on_carriage_log_message(game, message)),
        );
    }
    if message.message_type() != INCREMENT_LOG_MESSAGE {
        return WorldWriteLogMessageDispatch::Pending(message);
    }

    let decoded_type = message.base_mut().get_char();
    let (context_id, context_complete) = get_limited_string(&mut message, 0x200);
    let decoded_money = message.base_mut().get_long();
    let (description, description_complete) = get_limited_string(&mut message, 0x100);
    let decoded_player_id = message.base_mut().get_long();
    let entry_type = decoded_type.unwrap_or(0) as u8;
    let money = decoded_money.unwrap_or(0);
    let player_id = decoded_player_id.unwrap_or(0);

    let (item_name, item_name_complete, decoded_item_amount) = if entry_type == 0 {
        let (item_name, complete) = get_limited_string(&mut message, 0x80);
        (item_name, complete, message.base_mut().get_long())
    } else {
        (Vec::new(), true, Some(0))
    };
    let item_amount = decoded_item_amount.unwrap_or(0);
    let decoded_ip = message.base_mut().get_long();
    let ip = decoded_ip.unwrap_or(0) as u32;
    let ip_address = Ipv4Addr::from(ip.to_le_bytes()).to_string().into_bytes();
    let (player_account, player_level) = game
        .map_player(player_id as u32)
        .map(|player| {
            (
                player
                    .get_account()
                    .iter()
                    .copied()
                    .take_while(|byte| *byte != 0)
                    .collect::<Vec<_>>(),
                player.get_level(),
            )
        })
        .unwrap_or_default();
    let payload_complete = [
        decoded_type.is_some(),
        context_complete,
        decoded_money.is_some(),
        description_complete,
        decoded_player_id.is_some(),
        item_name_complete,
        decoded_item_amount.is_some(),
        decoded_ip.is_some(),
    ];

    if item_amount >= 1001 {
        let file_text = increment_amount_error(&player_account, &item_name, item_amount);
        let operator_log = add_log_text(&file_text);
        let mut truncated_file_text = file_text.clone();
        truncated_file_text.truncate(0x7f);
        put_string_to_file("Increment_error_log", &truncated_file_text);
        return WorldWriteLogMessageDispatch::Handled(
            WorldWriteLogMessageOutcome::IncrementLog(
                WorldIncrementLogMessageOutcome::RejectedItemAmount {
                    player_id,
                    player_account,
                    item_name,
                    item_amount,
                    payload_complete,
                    operator_log,
                    file_text: truncated_file_text,
                },
            ),
        );
    }

    let record = WorldIncrementLogWrite {
        context_id,
        entry_type,
        money,
        description: description.clone(),
        player_id,
        player_account,
        player_level,
        item_name,
        item_amount,
        ip_address,
    };
    let time = TagTime::local_now();
    let queue_length_after = game.push_write_log_command(
        WorldWriteLogCommand::IncrementLog(record.clone()),
    );
    let live_published = increment_log.add(
        player_id,
        time,
        entry_type,
        money,
        &description,
    );
    WorldWriteLogMessageDispatch::Handled(WorldWriteLogMessageOutcome::IncrementLog(
        WorldIncrementLogMessageOutcome::Queued {
            record,
            time,
            payload_complete,
            queue_length_after,
            live_published,
        },
    ))
}

fn on_auction_log_message(
    game: &CGame,
    auction_log: &mut CAuctionLog,
    log_time: TagTime,
    mut message: CMessage,
) -> WorldAuctionLogMessageOutcome {
    let (mut record, payload_complete) = get_auction_log_node(&mut message);
    let Ok(guid_key) = CGuid::create() else {
        return WorldAuctionLogMessageOutcome::BlockedGuidGeneration {
            record,
            log_time,
            payload_complete,
        };
    };
    record.guid_key = guid_key;
    let write = WorldAuctionLogWrite {
        record: record.clone(),
        log_time,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::AuctionLog(write.clone()));
    let live_published = auction_log.add_item(record);
    WorldAuctionLogMessageOutcome::Queued {
        write,
        payload_complete,
        queue_length_after,
        live_published,
    }
}

fn on_player_progress_log_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldPlayerProgressLogMessageOutcome {
    let (player_id, event, payload_complete) = match message.message_type() {
        PLAYER_LEVEL_LOG_MESSAGE => {
            let player_id = message.base_mut().get_long();
            let experience = message.base_mut().get_long();
            let old_level = message.base_mut().get_char();
            let current_level = message.base_mut().get_char();
            let map_id = message.base_mut().get_long();
            let position_x = message.base_mut().get_long();
            let position_y = message.base_mut().get_long();
            (
                player_id,
                WorldPlayerProgressLogEvent::Level {
                    experience: experience.unwrap_or(0),
                    old_level: old_level.unwrap_or(0) as u8,
                    current_level: current_level.unwrap_or(0) as u8,
                    map_id: map_id.unwrap_or(0),
                    position_x: position_x.unwrap_or(0),
                    position_y: position_y.unwrap_or(0),
                },
                WorldPlayerProgressPayloadCompleteness::Level([
                    player_id.is_some(),
                    experience.is_some(),
                    old_level.is_some(),
                    current_level.is_some(),
                    map_id.is_some(),
                    position_x.is_some(),
                    position_y.is_some(),
                ]),
            )
        }
        PLAYER_EXP_LOG_MESSAGE => {
            let log_type = message.base_mut().get_char();
            let player_id = message.base_mut().get_long();
            let experience = message.base_mut().get_long();
            let map_id = message.base_mut().get_long();
            let position_x = message.base_mut().get_long();
            let position_y = message.base_mut().get_long();
            (
                player_id,
                WorldPlayerProgressLogEvent::Experience {
                    experience: experience.unwrap_or(0),
                    map_id: map_id.unwrap_or(0),
                    position_x: position_x.unwrap_or(0),
                    position_y: position_y.unwrap_or(0),
                    log_type: log_type.unwrap_or(0) as u8,
                },
                WorldPlayerProgressPayloadCompleteness::Experience([
                    log_type.is_some(),
                    player_id.is_some(),
                    experience.is_some(),
                    map_id.is_some(),
                    position_x.is_some(),
                    position_y.is_some(),
                ]),
            )
        }
        PLAYER_DIED_LOG_MESSAGE => {
            let player_id = message.base_mut().get_long();
            let map_id = message.base_mut().get_long();
            let position_x = message.base_mut().get_long();
            let position_y = message.base_mut().get_long();
            (
                player_id,
                WorldPlayerProgressLogEvent::Died {
                    map_id: map_id.unwrap_or(0),
                    position_x: position_x.unwrap_or(0),
                    position_y: position_y.unwrap_or(0),
                },
                WorldPlayerProgressPayloadCompleteness::Died([
                    player_id.is_some(),
                    map_id.is_some(),
                    position_x.is_some(),
                    position_y.is_some(),
                ]),
            )
        }
        _ => unreachable!("player progress decoder вызывается только для трёх wire ID"),
    };
    let player_id = player_id.unwrap_or(0);
    let player = game.map_player(player_id as u32);
    let player_found = player.is_some();
    let player_name = player
        .map(|player| visible_c_string(player.get_name()))
        .unwrap_or_else(|| b"NULL".to_vec());
    let write = WorldPlayerProgressLogWrite {
        player_id,
        player_name,
        event,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::PlayerProgressLog(write.clone()));
    WorldPlayerProgressLogMessageOutcome {
        write,
        player_found,
        payload_complete,
        queue_length_after,
    }
}

fn on_player_relation_log_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldPlayerRelationLogMessageOutcome {
    let log_type = message.base_mut().get_char();
    let first_player_id = message.base_mut().get_long();
    let second_player_id = message.base_mut().get_long();
    let first_player_id_value = first_player_id.unwrap_or(0);
    let second_player_id_value = second_player_id.unwrap_or(0);
    let first_player = game.map_player(first_player_id_value as u32);
    let second_player = game.map_player(second_player_id_value as u32);
    let players_found = [first_player.is_some(), second_player.is_some()];
    let first_player_name = first_player
        .map(|player| visible_c_string(player.get_name()))
        .unwrap_or_else(|| b"NULL".to_vec());
    let second_player_name = second_player
        .map(|player| visible_c_string(player.get_name()))
        .unwrap_or_else(|| b"NULL".to_vec());
    let map_id = message.base_mut().get_long();
    let position_x = message.base_mut().get_long();
    let position_y = message.base_mut().get_long();
    let event = if message.message_type() == TEAM_LOG_MESSAGE {
        WorldPlayerRelationLogEvent::Team {
            map_id: map_id.unwrap_or(0),
            wire_position_x: position_x.unwrap_or(0),
            position_y: position_y.unwrap_or(0),
            log_type: log_type.unwrap_or(0) as u8,
        }
    } else {
        WorldPlayerRelationLogEvent::Killer {
            map_id: map_id.unwrap_or(0),
            position_x: position_x.unwrap_or(0),
            position_y: position_y.unwrap_or(0),
            log_type: log_type.unwrap_or(0) as u8,
        }
    };
    let write = WorldPlayerRelationLogWrite {
        first_player_id: first_player_id_value,
        first_player_name,
        second_player_id: second_player_id_value,
        second_player_name,
        event,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::PlayerRelationLog(write.clone()));
    WorldPlayerRelationLogMessageOutcome {
        write,
        players_found,
        payload_complete: [
            log_type.is_some(),
            first_player_id.is_some(),
            second_player_id.is_some(),
            map_id.is_some(),
            position_x.is_some(),
            position_y.is_some(),
        ],
        queue_length_after,
    }
}

fn on_chat_log_message(game: &CGame, mut message: CMessage) -> WorldChatLogMessageOutcome {
    let decoded_log_type = message.base_mut().get_char();
    let decoded_sender_id = message.base_mut().get_long();
    let log_type = decoded_log_type.unwrap_or(0) as u8;
    let sender_id = decoded_sender_id.unwrap_or(0);
    let sender = game.map_player(sender_id as u32);
    let sender_found = sender.is_some();
    let sender_name = sender
        .map(|player| visible_c_string(player.get_name()))
        .unwrap_or_else(|| b"NULL".to_vec());
    let map_id = message.base_mut().get_long();
    let position_x = message.base_mut().get_long();
    let position_y = message.base_mut().get_long();
    let (content, content_complete) = get_limited_string(&mut message, 0x200);
    let common_completeness = [
        decoded_log_type.is_some(),
        decoded_sender_id.is_some(),
        map_id.is_some(),
        position_x.is_some(),
        position_y.is_some(),
        content_complete,
    ];

    if content.is_empty() {
        return WorldChatLogMessageOutcome::EmptyContent {
            log_type,
            sender_id,
            sender_found,
            payload_complete: common_completeness,
        };
    }

    let (receiver_id, receiver_name, receiver_found, payload_complete) = match log_type {
        0 => (
            0,
            b"<public>".to_vec(),
            None,
            WorldChatPayloadCompleteness::FixedReceiver(common_completeness),
        ),
        1 => (
            0,
            b"<Region>".to_vec(),
            None,
            WorldChatPayloadCompleteness::FixedReceiver(common_completeness),
        ),
        4 => (
            0,
            b"<team>".to_vec(),
            None,
            WorldChatPayloadCompleteness::FixedReceiver(common_completeness),
        ),
        5 => {
            let decoded_receiver_id = message.base_mut().get_long();
            let receiver_id = decoded_receiver_id.unwrap_or(0);
            let receiver = game.map_player(receiver_id as u32);
            let receiver_found = receiver.is_some();
            let receiver_name = receiver
                .map(|player| visible_c_string(player.get_name()))
                .unwrap_or_else(|| b"NULL".to_vec());
            (
                receiver_id,
                receiver_name,
                Some(receiver_found),
                WorldChatPayloadCompleteness::PrivateReceiver([
                    common_completeness[0],
                    common_completeness[1],
                    common_completeness[2],
                    common_completeness[3],
                    common_completeness[4],
                    common_completeness[5],
                    decoded_receiver_id.is_some(),
                ]),
            )
        }
        6 => (
            0,
            b"<GM-code>".to_vec(),
            None,
            WorldChatPayloadCompleteness::FixedReceiver(common_completeness),
        ),
        7 => (
            0,
            b"<world>".to_vec(),
            None,
            WorldChatPayloadCompleteness::FixedReceiver(common_completeness),
        ),
        8 => (
            0,
            b"<country>".to_vec(),
            None,
            WorldChatPayloadCompleteness::FixedReceiver(common_completeness),
        ),
        _ => {
            let queue_length_after = game.push_write_log_command(
                WorldWriteLogCommand::LegacyEmptyChatSql { log_type },
            );
            return WorldChatLogMessageOutcome::LegacyEmptySqlQueued {
                log_type,
                sender_id,
                sender_found,
                payload_complete: common_completeness,
                queue_length_after,
            };
        }
    };

    let write = WorldChatLogWrite {
        sender_id,
        sender_name,
        map_id: map_id.unwrap_or(0),
        position_x: position_x.unwrap_or(0),
        position_y: position_y.unwrap_or(0),
        receiver_id,
        receiver_name,
        content,
        log_type,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::ChatLog(write.clone()));
    WorldChatLogMessageOutcome::Queued {
        write,
        sender_found,
        receiver_found,
        payload_complete,
        queue_length_after,
    }
}

fn on_change_map_log_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldChangeMapLogMessageOutcome {
    let log_type = message.base_mut().get_char();
    let player_id = message.base_mut().get_long();
    let player_id_value = player_id.unwrap_or(0);
    let player = game.map_player(player_id_value as u32);
    let player_found = player.is_some();
    let player_name = player
        .map(|player| visible_c_string(player.get_name()))
        .unwrap_or_else(|| b"NULL".to_vec());
    let money = message.base_mut().get_long();
    let bank = message.base_mut().get_long();
    let source_map_id = message.base_mut().get_long();
    let source_position_x = message.base_mut().get_long();
    let source_position_y = message.base_mut().get_long();
    let destination_map_id = message.base_mut().get_long();
    let destination_position_x = message.base_mut().get_long();
    let destination_position_y = message.base_mut().get_long();
    let write = WorldChangeMapLogWrite {
        player_id: player_id_value,
        player_name,
        money: money.unwrap_or(0),
        bank: bank.unwrap_or(0),
        source_map_id: source_map_id.unwrap_or(0),
        source_position_x: source_position_x.unwrap_or(0),
        source_position_y: source_position_y.unwrap_or(0),
        destination_map_id: destination_map_id.unwrap_or(0),
        destination_position_x: destination_position_x.unwrap_or(0),
        destination_position_y: destination_position_y.unwrap_or(0),
        log_type: log_type.unwrap_or(0) as u8,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::ChangeMapLog(write.clone()));
    WorldChangeMapLogMessageOutcome {
        write,
        player_found,
        payload_complete: [
            log_type.is_some(),
            player_id.is_some(),
            money.is_some(),
            bank.is_some(),
            source_map_id.is_some(),
            source_position_x.is_some(),
            source_position_y.is_some(),
            destination_map_id.is_some(),
            destination_position_x.is_some(),
            destination_position_y.is_some(),
        ],
        queue_length_after,
    }
}

fn on_auction_sale_log_message(
    game: &CGame,
    event_time: TagTime,
    mut message: CMessage,
) -> WorldAuctionSaleLogMessageOutcome {
    let (event, payload_complete) = match message.message_type() {
        AUCTION_SALE_OPER_LOG_MESSAGE => {
            let player_id = message.base_mut().get_long();
            let base_index = message.base_mut().get_long();
            let (guid, guid_complete) = get_guid(&mut message);
            let amount = message.base_mut().get_long();
            let money = message.base_mut().get_long();
            let time_type = message.base_mut().get_long();
            let fee = message.base_mut().get_long();
            (
                WorldAuctionSaleLogEvent::Oper {
                    player_id: player_id.unwrap_or(0) as u32,
                    base_index: base_index.unwrap_or(0) as u32,
                    guid,
                    amount: amount.unwrap_or(0) as u32,
                    money: money.unwrap_or(0) as u32,
                    time_type: time_type.unwrap_or(0) as u32,
                    fee: fee.unwrap_or(0) as u32,
                },
                WorldAuctionSalePayloadCompleteness::Oper([
                    player_id.is_some(),
                    base_index.is_some(),
                    guid_complete,
                    amount.is_some(),
                    money.is_some(),
                    time_type.is_some(),
                    fee.is_some(),
                ]),
            )
        }
        AUCTION_SALE_CANCEL_LOG_MESSAGE => {
            let player_id = message.base_mut().get_long();
            let (guid, guid_complete) = get_guid(&mut message);
            (
                WorldAuctionSaleLogEvent::Cancel {
                    player_id: player_id.unwrap_or(0) as u32,
                    guid,
                },
                WorldAuctionSalePayloadCompleteness::Cancel([
                    player_id.is_some(),
                    guid_complete,
                ]),
            )
        }
        AUCTION_SALE_RECEIVE_LOG_MESSAGE => {
            let player_id = message.base_mut().get_long();
            let amount = message.base_mut().get_long();
            let (guid, guid_complete) = get_guid(&mut message);
            (
                WorldAuctionSaleLogEvent::Receive {
                    player_id: player_id.unwrap_or(0) as u32,
                    amount: amount.unwrap_or(0) as u32,
                    guid,
                },
                WorldAuctionSalePayloadCompleteness::Receive([
                    player_id.is_some(),
                    amount.is_some(),
                    guid_complete,
                ]),
            )
        }
        _ => unreachable!("auction sale decoder вызывается только для трёх wire ID"),
    };
    let write = WorldAuctionSaleLogWrite { event_time, event };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::AuctionSaleLog(write.clone()));
    WorldAuctionSaleLogMessageOutcome {
        write,
        payload_complete,
        queue_length_after,
    }
}

fn on_fairy_log_message(
    game: &CGame,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldFairyLogMessageOutcome {
    let event_time = TagTime::local_now();
    let decoded_type = message.base_mut().get_long();
    let decoded_player_id = message.base_mut().get_long();
    let fairy_type = decoded_type.unwrap_or(0);
    let player_id = decoded_player_id.unwrap_or(0);
    let header_complete = [decoded_type.is_some(), decoded_player_id.is_some()];

    if game.map_player(player_id as u32).is_none() {
        let text = format!("err:palyerid:{player_id} is not online").into_bytes();
        return WorldFairyLogMessageOutcome::PlayerOffline {
            fairy_type,
            player_id,
            event_time,
            header_complete,
            operator_log: add_log_text(&text),
        };
    }

    let (event, payload_complete) = match fairy_type {
        0 => {
            let is_jing_po = message.base_mut().get_long();
            let (goods_id, goods_id_complete) = get_limited_string(&mut message, 0x40);
            let (goods_name, goods_name_complete) = get_limited_string(&mut message, 0x20);
            let current_level = message.base_mut().get_long();
            (
                WorldFairyLogEvent::Grow {
                    is_jing_po: is_jing_po.unwrap_or(0),
                    goods_id,
                    goods_name,
                    current_level: current_level.unwrap_or(0),
                },
                WorldFairyPayloadCompleteness::Grow([
                    is_jing_po.is_some(),
                    goods_id_complete,
                    goods_name_complete,
                    current_level.is_some(),
                ]),
            )
        }
        1 => {
            let (goods_id, goods_id_complete) = get_guid(&mut message);
            let (goods_name, goods_name_complete) = get_limited_string(&mut message, 0x20);
            let current_level = message.base_mut().get_long();
            let grow_rate = message.base_mut().get_long();
            let main_fetch = message.base_mut().get_long();
            let combined_times = message.base_mut().get_long();
            (
                WorldFairyLogEvent::Take {
                    goods_id,
                    goods_name,
                    current_level: current_level.unwrap_or(0),
                    grow_rate_raw: grow_rate.unwrap_or(0) as u32,
                    main_fetch: main_fetch.unwrap_or(0),
                    combined_times: combined_times.unwrap_or(0),
                },
                WorldFairyPayloadCompleteness::Take([
                    goods_id_complete,
                    goods_name_complete,
                    current_level.is_some(),
                    grow_rate.is_some(),
                    main_fetch.is_some(),
                    combined_times.is_some(),
                ]),
            )
        }
        2 => {
            let (goods_name, goods_name_complete) = get_limited_string(&mut message, 0x20);
            let (goods_id, goods_id_complete) = get_guid(&mut message);
            let previous_level = message.base_mut().get_long();
            let current_level = message.base_mut().get_long();
            let west_diamond = message.base_mut().get_long();
            (
                WorldFairyLogEvent::Implantation {
                    goods_name,
                    goods_id,
                    previous_level: previous_level.unwrap_or(0),
                    current_level: current_level.unwrap_or(0),
                    west_diamond: west_diamond.unwrap_or(0),
                },
                WorldFairyPayloadCompleteness::Implantation([
                    goods_name_complete,
                    goods_id_complete,
                    previous_level.is_some(),
                    current_level.is_some(),
                    west_diamond.is_some(),
                ]),
            )
        }
        3 => {
            let (goods_id, goods_id_complete) = get_guid(&mut message);
            let (goods_name, goods_name_complete) = get_limited_string(&mut message, 0x20);
            (
                WorldFairyLogEvent::Incubate {
                    goods_id,
                    goods_name,
                },
                WorldFairyPayloadCompleteness::Incubate([
                    goods_id_complete,
                    goods_name_complete,
                ]),
            )
        }
        4 => {
            let (main_goods_id, main_goods_id_complete) = get_guid(&mut message);
            let (main_goods_name, main_goods_name_complete) =
                get_limited_string(&mut message, 0x20);
            let main_level = message.base_mut().get_long();
            let main_grow_rate = message.base_mut().get_long();
            let (secondary_goods_id, secondary_goods_id_complete) = get_guid(&mut message);
            let (secondary_goods_name, secondary_goods_name_complete) =
                get_limited_string(&mut message, 0x20);
            let secondary_level = message.base_mut().get_long();
            let secondary_grow_rate = message.base_mut().get_long();
            let west_patch = message.base_mut().get_long();
            let (child_goods_id, child_goods_id_complete) = get_guid(&mut message);
            let (child_goods_name, child_goods_name_complete) =
                get_limited_string(&mut message, 0x20);
            let child_main_ability = message.base_mut().get_long();
            let child_syncretize_times = message.base_mut().get_long();
            let child_grow_rate = message.base_mut().get_long();
            (
                WorldFairyLogEvent::Syncretize {
                    main_goods_id,
                    main_goods_name,
                    main_level: main_level.unwrap_or(0),
                    main_grow_rate_raw: main_grow_rate.unwrap_or(0) as u32,
                    secondary_goods_id,
                    secondary_goods_name,
                    secondary_level: secondary_level.unwrap_or(0),
                    secondary_grow_rate_raw: secondary_grow_rate.unwrap_or(0) as u32,
                    west_patch: west_patch.unwrap_or(0),
                    child_goods_id,
                    child_goods_name,
                    child_main_ability: child_main_ability.unwrap_or(0),
                    child_syncretize_times: child_syncretize_times.unwrap_or(0),
                    child_grow_rate_raw: child_grow_rate.unwrap_or(0) as u32,
                },
                WorldFairyPayloadCompleteness::Syncretize([
                    main_goods_id_complete,
                    main_goods_name_complete,
                    main_level.is_some(),
                    main_grow_rate.is_some(),
                    secondary_goods_id_complete,
                    secondary_goods_name_complete,
                    secondary_level.is_some(),
                    secondary_grow_rate.is_some(),
                    west_patch.is_some(),
                    child_goods_id_complete,
                    child_goods_name_complete,
                    child_main_ability.is_some(),
                    child_syncretize_times.is_some(),
                    child_grow_rate.is_some(),
                ]),
            )
        }
        _ => {
            let text = format!("fairy log ! err type:{fairy_type}").into_bytes();
            return WorldFairyLogMessageOutcome::InvalidType {
                fairy_type,
                player_id,
                event_time,
                header_complete,
                operator_log: add_log_text(&text),
            };
        }
    };

    let record = WorldFairyLogWrite {
        player_id,
        event_time,
        event,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::FairyLog(record.clone()));
    WorldFairyLogMessageOutcome::Queued {
        record,
        header_complete,
        payload_complete,
        queue_length_after,
    }
}

fn on_ciqing_log_message(game: &CGame, mut message: CMessage) -> WorldCiqingLogMessageOutcome {
    let player_id = message.base_mut().get_long();
    let in_out = message.base_mut().get_long();
    let entry_type = message.base_mut().get_long();
    let base_index = message.base_mut().get_long();
    let amount = message.base_mut().get_long();
    let payload_complete = [
        player_id.is_some(),
        in_out.is_some(),
        entry_type.is_some(),
        base_index.is_some(),
        amount.is_some(),
    ];
    let record = WorldCiqingLogWrite {
        player_id: player_id.unwrap_or(0),
        in_out: in_out.unwrap_or(0),
        entry_type: entry_type.unwrap_or(0),
        base_index: base_index.unwrap_or(0),
        amount: amount.unwrap_or(0),
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::CiqingLog(record.clone()));
    WorldCiqingLogMessageOutcome {
        record,
        payload_complete,
        queue_length_after,
    }
}

fn on_plain_log_message(game: &CGame, mut message: CMessage) -> WorldPlainLogMessageOutcome {
    let player_id = message.base_mut().get_long();
    let log_type = message.base_mut().get_long();
    let (content, content_complete) = get_limited_string(&mut message, 0x100);
    let player_id_value = player_id.unwrap_or(0);
    let player = game.map_player(player_id_value as u32);
    let player_found = player.is_some();
    let (player_name, player_account) = player.map_or_else(
        || (b"null".to_vec(), b"null".to_vec()),
        |player| {
            (
                visible_c_string(player.get_name()),
                visible_c_string(player.get_account()),
            )
        },
    );
    let record = WorldPlainLogWrite {
        player_id: player_id_value,
        player_name,
        player_account,
        content,
        log_type: log_type.unwrap_or(0),
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::PlainLog(record.clone()));
    WorldPlainLogMessageOutcome {
        record,
        player_found,
        payload_complete: [player_id.is_some(), log_type.is_some(), content_complete],
        queue_length_after,
    }
}

fn on_carriage_log_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldCarriageLogMessageOutcome {
    let player_id = message.base_mut().get_long();
    let carriage_id = message.base_mut().get_long();
    let region_id = message.base_mut().get_long();
    let coordinate_x = message.base_mut().get_short();
    let coordinate_y = message.base_mut().get_short();
    let event_type = message.base_mut().get_long();
    let event_time = TagTime::local_now();
    let payload_complete = [
        player_id.is_some(),
        carriage_id.is_some(),
        region_id.is_some(),
        coordinate_x.is_some(),
        coordinate_y.is_some(),
        event_type.is_some(),
    ];
    let record = WorldCarriageLogWrite {
        player_id: player_id.unwrap_or(0),
        carriage_id: carriage_id.unwrap_or(0),
        region_id: region_id.unwrap_or(0),
        coordinate_x: coordinate_x.unwrap_or(0),
        coordinate_y: coordinate_y.unwrap_or(0),
        event_type: event_type.unwrap_or(0),
        event_time,
    };
    let queue_length_after =
        game.push_write_log_command(WorldWriteLogCommand::CarriageLog(record.clone()));
    WorldCarriageLogMessageOutcome {
        record,
        payload_complete,
        queue_length_after,
    }
}

fn get_limited_string(message: &mut CMessage, maximum: usize) -> (Vec<u8>, bool) {
    let complete = {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        wire.get(*cursor..)
            .is_some_and(|remaining| remaining.iter().take(maximum).any(|byte| *byte == 0))
    };
    let value = message
        .base_mut()
        .get_str_bytes(maximum)
        .unwrap_or_default();
    (value, complete)
}

fn get_auction_log_node(message: &mut CMessage) -> (AuctionLogNode, bool) {
    let mut bytes = [0_u8; 0x150];
    let complete = message.base_mut().get(&mut bytes);
    let long = |offset: usize| {
        i32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("offset поля проверен PDB-layout аукционного node"),
        )
    };
    let word = |offset: usize| {
        u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("offset SYSTEMTIME проверен PDB-layout аукционного node"),
        )
    };
    let mut description = [0_u8; 0x100];
    description.copy_from_slice(&bytes[0x30..0x130]);
    let guid = CGuid::from_legacy_bytes(
        bytes[0x130..0x140]
            .try_into()
            .expect("GUID занимает exact 16 байт"),
    );
    let guid_key = CGuid::from_legacy_bytes(
        bytes[0x140..0x150]
            .try_into()
            .expect("guidKey занимает exact 16 байт"),
    );
    (
        AuctionLogNode {
            base_id: long(0x00),
            operation_type: long(0x04),
            money_type: long(0x08),
            money_num: long(0x0c),
            player_id: long(0x10),
            amount: long(0x14),
            fee: long(0x18),
            notice: long(0x1c),
            time: AuctionLogSystemTime {
                year: word(0x20),
                month: word(0x22),
                day_of_week: word(0x24),
                day: word(0x26),
                hour: word(0x28),
                minute: word(0x2a),
                second: word(0x2c),
                milliseconds: word(0x2e),
            },
            description,
            guid,
            guid_key,
        },
        complete,
    )
}

fn get_guid(message: &mut CMessage) -> (CGuid, bool) {
    let complete = {
        let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match wire.get(*cursor).copied() {
            Some(0) => true,
            Some(_) => wire.len().saturating_sub(*cursor) >= 17,
            None => false,
        }
    };
    let value = message
        .base_mut()
        .get_guid()
        .unwrap_or(CGuid::GUID_INVALID);
    (value, complete)
}

fn visible_c_string(value: &[u8]) -> Vec<u8> {
    value
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .collect()
}

fn increment_amount_error(account: &[u8], item_name: &[u8], amount: i32) -> Vec<u8> {
    let mut text = b"CDKEY:[".to_vec();
    text.extend_from_slice(account);
    text.extend_from_slice(b"] buy goods:[");
    text.extend_from_slice(item_name);
    text.extend_from_slice(b"], Receive error Number[");
    text.extend_from_slice(amount.to_string().as_bytes());
    text.extend_from_slice(b"] from GameServer!!");
    text
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\writelogmessage.cpp

// ============================================================================
// FUNCTION: Catch@0043ec16
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\writelogmessage.cpp
// RVA: 0x0003EC16
// ADDRESS: 0043ec16
// PROTOTYPE: undefined Catch@0043ec16()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043ece6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\writelogmessage.cpp
// RVA: 0x0003ECE6
// ADDRESS: 0043ece6
// PROTOTYPE: undefined Catch@0043ece6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: OnWriteLogMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\writelogmessage.cpp:18
// RVA: 0x000A8AB0
// ADDRESS: 004a8ab0
// PROTOTYPE: void __cdecl OnWriteLogMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052de70
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\writelogmessage.cpp
// RVA: 0x0012DE70
// ADDRESS: 0052de70
// PROTOTYPE: undefined Unwind@0052de70()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
