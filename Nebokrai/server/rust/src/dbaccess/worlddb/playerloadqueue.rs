//! FIFO запросов загрузки игроков, подтверждённая `worldserver.exe` и
//! `worldserver.pdb`.
//!
//! Owner сохраняет first-match removal, head-to-tail pop, отдельные lock
//! области и передачу владения batch-у. `Mutex<VecDeque<_>>` заменяет critical
//! section/STL; дополнительные limits, ожидание и shutdown policy не входят
//! в этот контракт.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::public::tools::put_string_to_file;

pub(crate) const PLAYER_LOAD_CDKEY_CAPACITY: usize = 20;
const PLAYER_LOAD_LOG_NAME: &str = "TemptLoadDataLog_";

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

    pub(crate) fn cdkey(&self) -> Option<&[u8]> {
        let end = self.cdkey.iter().position(|byte| *byte == 0)?;
        Some(&self.cdkey[..end])
    }

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
///
/// `Duplicate` сохраняет incoming record до границы вызывающего owner-а:
/// `CRsPlayer::GetPlayerData` немедленно освобождал его только на этой ветви,
/// а queued record становился собственностью FIFO. Rust затем автоматически
/// освобождает это значение у producer-а; ручной `operator_delete` не нужен.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerLoadPushOutcome {
    Queued,
    Duplicate(PlayerLoadQueueEntry),
}

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

    pub(crate) fn get_size(&self) -> u32 {
        self.entries.lock().len() as u32
    }

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

    pub(crate) fn pop_player_load_data_to_list(&self) -> VecDeque<PlayerLoadQueueEntry> {
        let mut entries = self.entries.lock();
        entries.drain(..).collect()
    }

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

// четыре owner-а выше сохраняют оригинал FIFO, first-match,
// duplicate bool-смысл и синхронные `PutStringToFile` эффекты. Для `Clear`
// verified EXE удаляет все records под одним lock; ложный ранний return
// allocator и Win32 lock internals заменены безопасными библиотечными типами.
