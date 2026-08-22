//! WorldServer dispatcher-owner `OnWriteLogMessage`.
//!
//! Весь dispatcher RVA `0x000A8AB0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме
//! increment-shop producer `0x6020D` со статусом `IMPLEMENTED`. Точная пара:
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

/// Typed очередь сохраняет старый FIFO, но оставляет SQL transport Tiberius-у.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldWriteLogCommand {
    IncrementLog(WorldIncrementLogWrite),
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

pub(crate) enum WorldWriteLogMessageDispatch {
    Handled(WorldIncrementLogMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутую increment-shop ветку `OnWriteLogMessage`.
pub(crate) fn on_write_log_message(
    game: &mut CGame,
    increment_log: &mut CIncrementLog,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
    mut message: CMessage,
) -> WorldWriteLogMessageDispatch {
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
            WorldIncrementLogMessageOutcome::RejectedItemAmount {
                player_id,
                player_account,
                item_name,
                item_amount,
                payload_complete,
                operator_log,
                file_text: truncated_file_text,
            },
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
    WorldWriteLogMessageDispatch::Handled(WorldIncrementLogMessageOutcome::Queued {
        record,
        time,
        payload_complete,
        queue_length_after,
        live_published,
    })
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
