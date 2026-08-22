//! Владелец очереди запросов загрузки игроков исторического `WorldServer`.
//!
//! `GetSize` RVA `0x000E6080`, `Clear` RVA `0x000EADB0`,
//! `RemovePlayerLoadData` RVA `0x000EAE90`, `PushPlayerLoadData` RVA
//! `0x000EB2C0` и `PopPlayerLoadDataToList` RVA `0x000EB400` имеют статус
//! `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`; исходный owner
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp:17-105`.
//!
//! Exact x86-layout записи подтверждён обращениями `+0x14/+0x18`: сначала
//! `char szCdkey[20]`, затем signed `long nPlayerID` и `unsigned long
//! dwClientIP`. Старые `CRITICAL_SECTION + deque<tagPlayerLoadQueue*>`
//! заменены cloneable `Arc<parking_lot::Mutex<VecDeque<_>>>`; FIFO, поиск
//! первого совпадения и область блокировки сохранены. `Arc` отделяет только
//! lifetime shared очереди от `CGame`, чтобы точный системный worker мог её
//! использовать. Rust-владелец безопасно уничтожает запись после remove/clear
//! вместо сохранения внутренних утечек старого raw-pointer контейнера.
//! Добавленные Linux-донором limit, condition-variable,
//! in-flight/cancellation maps и account-wide remove в EXE отсутствуют.
//!
//! Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже сохранён как локальная
//! документация, а не как Rust-реализация.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::public::tools::put_string_to_file;

pub(crate) const PLAYER_LOAD_CDKEY_CAPACITY: usize = 20;
const PLAYER_LOAD_LOG_NAME: &str = "TemptLoadDataLog_";

/// Owned-представление точного `tagPlayerLoadQueue` без x86 ABI-зависимости.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerLoadQueueEntry {
    cdkey: [u8; PLAYER_LOAD_CDKEY_CAPACITY],
    player_id: i32,
    client_ip: u32,
}

impl PlayerLoadQueueEntry {
    pub(crate) const fn new(
        cdkey: [u8; PLAYER_LOAD_CDKEY_CAPACITY],
        player_id: i32,
        client_ip: u32,
    ) -> Self {
        Self {
            cdkey,
            player_id,
            client_ip,
        }
    }

    /// Возвращает старую C-string часть fixed buffer-а.
    pub(crate) fn cdkey(&self) -> Option<&[u8]> {
        let end = self.cdkey.iter().position(|byte| *byte == 0)?;
        Some(&self.cdkey[..end])
    }

    /// Копирует весь exact fixed buffer для следующего queue-record owner-а.
    pub(crate) const fn fixed_cdkey(&self) -> [u8; PLAYER_LOAD_CDKEY_CAPACITY] {
        self.cdkey
    }

    pub(crate) const fn player_id(&self) -> i32 {
        self.player_id
    }

    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }
}

/// Исходный bool различал duplicate (`true`) и новую очередь (`false`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLoadPushOutcome {
    Queued,
    Duplicate(PlayerLoadQueueEntry),
}

/// Потокобезопасная cloneable FIFO ожидающих DB-load запросов.
#[derive(Clone)]
pub(crate) struct CPlayerLoadQueue {
    entries: Arc<Mutex<VecDeque<PlayerLoadQueueEntry>>>,
}

impl CPlayerLoadQueue {
    pub(crate) fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Снимает самостоятельный 32-битный snapshot размера очереди.
    pub(crate) fn get_size(&self) -> u32 {
        self.entries.lock().len() as u32
    }

    /// Добавляет новую запись либо возвращает duplicate producer-у.
    pub(crate) fn push_player_load_data(
        &self,
        entry: PlayerLoadQueueEntry,
    ) -> PlayerLoadPushOutcome {
        let mut entries = self.entries.lock();
        if let Some(existing) = entries
            .iter()
            .find(|existing| existing.player_id == entry.player_id)
        {
            put_string_to_file(
                PLAYER_LOAD_LOG_NAME,
                &format_player_load_log(existing, b" Request For Data Once More."),
            );
            return PlayerLoadPushOutcome::Duplicate(entry);
        }

        entries.push_back(entry);
        PlayerLoadPushOutcome::Queued
    }

    /// Удаляет и возвращает только первое совпадение по signed player ID.
    pub(crate) fn remove_player_load_data(
        &self,
        player_id: i32,
    ) -> Option<PlayerLoadQueueEntry> {
        let mut entries = self.entries.lock();
        let index = entries
            .iter()
            .position(|entry| entry.player_id == player_id)?;
        put_string_to_file(
            PLAYER_LOAD_LOG_NAME,
            &format_player_load_log(
                entries
                    .get(index)
                    .expect("index найден в неизменённой очереди"),
                b" EXIT, Cancel Request For Data.",
            ),
        );
        entries.remove(index)
    }

    /// Переносит всю текущую очередь в FIFO-список одним lock-owner-ом.
    pub(crate) fn pop_player_load_data_to_list(&self) -> VecDeque<PlayerLoadQueueEntry> {
        let mut entries = self.entries.lock();
        entries.drain(..).collect()
    }

    /// Уничтожает все записи в FIFO-порядке под исходной областью блокировки.
    pub(crate) fn clear(&self) {
        let mut entries = self.entries.lock();
        while let Some(entry) = entries.pop_front() {
            drop(entry);
        }
    }
}

fn format_player_load_log(entry: &PlayerLoadQueueEntry, suffix: &[u8]) -> Vec<u8> {
    let account = entry.cdkey().unwrap_or(&entry.cdkey);
    let tail = format!(" Pid:{} IP:0X{:08X}", entry.player_id, entry.client_ip);
    let mut line = Vec::with_capacity(4 + account.len() + tail.len() + suffix.len());
    line.extend_from_slice(b"Acc:");
    line.extend_from_slice(account);
    line.extend_from_slice(tail.as_bytes());
    line.extend_from_slice(suffix);
    line
}

// IMPLEMENTED_OWNER: четыре owner-а выше сохраняют exact FIFO, first-match,
// duplicate bool-смысл и синхронные `PutStringToFile` эффекты. Локальные STL,
// allocator и Win32 lock internals заменены безопасными библиотечными типами.

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp

// ============================================================================
// FUNCTION: CPlayerLoadQueue::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp:105
// RVA: 0x000EADB0
// ADDRESS: 004eadb0
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerLoadQueue::~CPlayerLoadQueue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp:17
// RVA: 0x000EAE70
// ADDRESS: 004eae70
// PROTOTYPE: void __thiscall ~CPlayerLoadQueue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerLoadQueue::RemovePlayerLoadData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp:56
// RVA: 0x000EAE90
// ADDRESS: 004eae90
// PROTOTYPE: void __thiscall RemovePlayerLoadData(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerLoadQueue::PushPlayerLoadData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp:23
// RVA: 0x000EB2C0
// ADDRESS: 004eb2c0
// PROTOTYPE: bool __thiscall PushPlayerLoadData(tagPlayerLoadQueue * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerLoadQueue::PopPlayerLoadDataToList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerloadqueue.cpp:78
// RVA: 0x000EB400
// ADDRESS: 004eb400
// PROTOTYPE: bool __thiscall PopPlayerLoadDataToList(list<CPlayerLoadQueue::tagPlayerLoadQueue*,std::allocator<CPlayerLoadQueue::tagPlayerLoadQueue*>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
