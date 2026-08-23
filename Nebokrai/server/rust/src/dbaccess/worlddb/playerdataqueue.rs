//! FIFO загруженных игроков WorldServer из точной пары EXE/PDB.
//!
//! Запись сохраняет 20-byte account buffer, player/client IDs и nullable
//! player-owner. Size и pop блокируются независимо, FIFO передаёт владение,
//! reset меняет day/week/month по исходной mask, а clear уничтожает все записи
//! под одним lock. `Arc<Mutex<VecDeque<_>>>` и `Box` заменяют Win32/STL lifetime
//! без изменения порядка и без трактовки buffer без NUL как C-строки.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::worldserver::appworld::player::CPlayer;

pub(crate) const PLAYER_DATA_CDKEY_CAPACITY: usize = 20;

/// Owned-представление одного исходного `tagPlayerDataQueue`.
pub(crate) struct PlayerDataQueueEntry {
    cdkey: [u8; PLAYER_DATA_CDKEY_CAPACITY],
    player_id: u32,
    client_ip: u32,
    player: Option<Box<CPlayer>>,
}

impl PlayerDataQueueEntry {
 /// Создаёт точную запись producer-а после завершения DB-load.
    pub(crate) const fn new(
        cdkey: [u8; PLAYER_DATA_CDKEY_CAPACITY],
        player_id: u32,
        client_ip: u32,
        player: Option<Box<CPlayer>>,
    ) -> Self {
        Self {
            cdkey,
            player_id,
            client_ip,
            player,
        }
    }

 /// Возвращает account bytes до обязательного NUL старого fixed buffer-а.
    pub(crate) fn cdkey(&self) -> Option<&[u8]> {
        let end = self.cdkey.iter().position(|byte| *byte == 0)?;
        Some(&self.cdkey[..end])
    }

    pub(crate) const fn player_id(&self) -> u32 {
        self.player_id
    }

    pub(crate) const fn client_ip(&self) -> u32 {
        self.client_ip
    }

 /// Забирает nullable player-owner, оставляя record без указателя.
    pub(crate) fn take_player(&mut self) -> Option<Box<CPlayer>> {
        self.player.take()
    }
}

/// Потокобезопасная cloneable FIFO загруженных player-record-ов.
#[derive(Clone)]
pub(crate) struct CPlayerDataQueue {
    entries: Arc<Mutex<VecDeque<PlayerDataQueueEntry>>>,
}

impl CPlayerDataQueue {
    pub(crate) fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

 /// Снимает самостоятельный 32-битный snapshot текущего размера.
    pub(crate) fn get_size(&self) -> u32 {
        self.entries.lock().len() as u32
    }

 /// Забирает первый record либо возвращает старый `nullptr` как `None`.
    pub(crate) fn pop_player_data(&self) -> Option<PlayerDataQueueEntry> {
        self.entries.lock().pop_front()
    }

 /// Передаёт non-null record в хвост и возвращает исходный successful bool.
    pub(crate) fn push_player_data(&self, entry: PlayerDataQueueEntry) -> bool {
        self.entries.lock().push_back(entry);
        true
    }

 /// Обновляет каждого non-null queued player, сохраняя FIFO и записи.
    pub(crate) fn reset_honor_eliminate_info(&self, rank_mask: u32) {
        let mut entries = self.entries.lock();
        for entry in entries.iter_mut() {
            if let Some(player) = entry.player.as_deref_mut() {
                player.reset_honor_eliminate_info(rank_mask);
            }
        }
    }

 /// Уничтожает всех player-owner-ов и records в FIFO-порядке под одним lock.
    pub(crate) fn clear(&self) {
        let mut entries = self.entries.lock();
        while let Some(mut entry) = entries.pop_front() {
            drop(entry.player.take());
            drop(entry);
        }
    }
}
