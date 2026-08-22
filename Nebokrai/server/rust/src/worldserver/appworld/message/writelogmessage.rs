//! WorldServer dispatcher-owner `OnWriteLogMessage`.
//!
//! Весь dispatcher RVA `0x000A8AB0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме
//! increment-shop `0x6020D`, carriage `0x6020E`, plain player log `0x6020F`,
//! reserved no-op `0x60211..0x60213` и ciqing `0x60218` со статусом
//! `IMPLEMENTED`. Точная пара:
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
use crate::public::date::TagTime;
use crate::public::tools::put_string_to_file;
use crate::worldserver::appworld::incrementlog::incrementlog::CIncrementLog;
use crate::worldserver::worldserver::game::CGame;
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

const INCREMENT_LOG_MESSAGE: i32 = 0x0006_020D;
const CARRIAGE_LOG_MESSAGE: i32 = 0x0006_020E;
const PLAIN_LOG_MESSAGE: i32 = 0x0006_020F;
const RESERVED_WRITE_LOG_MESSAGES: std::ops::RangeInclusive<i32> =
    0x0006_0211..=0x0006_0213;
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

/// Typed очередь сохраняет старый FIFO, но оставляет SQL transport Tiberius-у.
#[derive(Clone, Debug)]
pub(crate) enum WorldWriteLogCommand {
    IncrementLog(WorldIncrementLogWrite),
    CarriageLog(WorldCarriageLogWrite),
    PlainLog(WorldPlainLogWrite),
    CiqingLog(WorldCiqingLogWrite),
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
pub(crate) enum WorldWriteLogMessageOutcome {
    IncrementLog(WorldIncrementLogMessageOutcome),
    CarriageLog(WorldCarriageLogMessageOutcome),
    PlainLog(WorldPlainLogMessageOutcome),
    CiqingLog(WorldCiqingLogMessageOutcome),
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
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldWriteLogMessageDispatch {
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
