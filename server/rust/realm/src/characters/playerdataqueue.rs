//! FIFO загруженных игроков, подтверждённая `worldserver.exe` и
//! `worldserver.pdb`.
//! Перенесена в Realm `characters/`.
//!
//! Запись сохраняет 20-byte account buffer, player/client IDs и nullable
//! player-owner. Size и pop блокируются независимо, FIFO передаёт владение,
//! reset меняет day/week/month по исходной mask, а clear уничтожает все записи
//! под одним lock. `Arc<Mutex<VecDeque<_>>>` и `Box` заменяют Win32/STL lifetime
//! без изменения порядка и без трактовки buffer без NUL как C-строки.
//!
//! Хранимый игрок параметризован (`P`), поэтому очередь старого `CPlayer` не
//! знает; единственная операция над payload, сброс honor-eliminate масок,
//! выделена в узкий трейт `PlayerDataResetHonorEliminate` у владельца игрока.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;

/// Узкий сброс honor-eliminate масок по загруженному игроку; реализация у
/// владельца игрока (старый `CPlayer`), имя совпадает с inherent-методом
/// специально (inherent priority исключает рекурсию).
pub trait PlayerDataResetHonorEliminate {
    fn reset_honor_eliminate_info(&mut self, rank_mask: u32);
}

pub const PLAYER_DATA_CDKEY_CAPACITY: usize = 20;

pub struct PlayerDataQueueEntry<P> {
    cdkey: [u8; PLAYER_DATA_CDKEY_CAPACITY],
    player_id: u32,
    client_ip: u32,
    player: Option<Box<P>>,
}

impl<P> PlayerDataQueueEntry<P> {
    pub const fn new(
        cdkey: [u8; PLAYER_DATA_CDKEY_CAPACITY],
        player_id: u32,
        client_ip: u32,
        player: Option<Box<P>>,
    ) -> Self {
        Self {
            cdkey,
            player_id,
            client_ip,
            player,
        }
    }

    pub fn cdkey(&self) -> Option<&[u8]> {
        let end = self.cdkey.iter().position(|byte| *byte == 0)?;
        Some(&self.cdkey[..end])
    }

    pub const fn player_id(&self) -> u32 {
        self.player_id
    }

    pub const fn client_ip(&self) -> u32 {
        self.client_ip
    }

    pub fn take_player(&mut self) -> Option<Box<P>> {
        self.player.take()
    }
}

pub struct CPlayerDataQueue<P> {
    entries: Arc<Mutex<VecDeque<PlayerDataQueueEntry<P>>>>,
}

// Ручная реализация вместо derive: `Arc` clone не требует `P: Clone`.
impl<P> Clone for CPlayerDataQueue<P> {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
        }
    }
}

impl<P> CPlayerDataQueue<P> {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn get_size(&self) -> u32 {
        self.entries.lock().len() as u32
    }

    pub fn pop_player_data(&self) -> Option<PlayerDataQueueEntry<P>> {
        self.entries.lock().pop_front()
    }

    pub fn push_player_data(&self, entry: PlayerDataQueueEntry<P>) -> bool {
        self.entries.lock().push_back(entry);
        true
    }

    pub fn reset_honor_eliminate_info(&self, rank_mask: u32)
    where
        P: PlayerDataResetHonorEliminate,
    {
        let mut entries = self.entries.lock();
        for entry in entries.iter_mut() {
            if let Some(player) = entry.player.as_deref_mut() {
                player.reset_honor_eliminate_info(rank_mask);
            }
        }
    }

    pub fn clear(&self) {
        let mut entries = self.entries.lock();
        while let Some(mut entry) = entries.pop_front() {
            drop(entry.player.take());
            drop(entry);
        }
    }
}
