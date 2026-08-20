//! Владелец очереди загруженных игроков исторического `WorldServer`.
//!
//! `CPlayerLoadQueue::GetSize` RVA `0x000E6080`,
//! `CPlayerDataQueue::PopPlayerData` RVA `0x000E6190`, constructor
//! RVA `0x000E6210`, `Clear` RVA `0x000E6230` и `PushPlayerData`
//! RVA `0x000E65C0` имеют статус `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\playerdataqueue.cpp:25,36,49,65`.
//!
//! Точный `tagPlayerDataQueue` занимает `0x20` байт в x86-процессе:
//! `char szCdkey[20]`, `unsigned nPlayerID`, `unsigned long dwClientIP` и
//! nullable owning `CPlayer*`. Rust не копирует ABI-layout: fixed account
//! buffer остаётся `[u8; 20]`, scalar signedness сохраняется, а nullable
//! pointer становится `Option<Box<CPlayer>>`.
//! Fixed buffer без NUL остаётся локальной `BLOCKED_MISSING_FACT`: исходные
//! C-string consumers читали бы за `tagPlayerDataQueue`, поэтому safe API не
//! назначает ему двадцатибайтовую нормализацию.
//!
//! Старые `CRITICAL_SECTION + std::deque<tagPlayerDataQueue*>` заменены
//! `parking_lot::Mutex<VecDeque<_>>`. `GetSize` и `PopPlayerData` по-прежнему
//! берут блокировку независимо: поэтому snapshot может быть ненулевым, а
//! последующий pop вернуть `None`, если другой consumer успел забрать запись.
//! FIFO-порядок и передача владения при pop сохранены. Typed push не может
//! получить старый null record и потому соответствует только исходной
//! successful ветви.
//!
//! `ResetHonorElimilateInfo` RVA `0x000E6110` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально): он
//! меняет ещё не достигнутые honor-поля каждого queued player. Для `Clear`
//! exact EXE `0x004E6242..0x004E6321` опровергает ложный ранний return
//! декомпилятора: цикл уничтожает все player/record owners, tidy-ит deque и
//! только затем снимает lock. `Mutex<VecDeque<_>>` и `Box/Drop` сохраняют этот
//! порядок без ручных STL/allocator/destructor internals.

use std::collections::VecDeque;

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

/// Потокобезопасная FIFO загруженных player-record-ов.
pub(crate) struct CPlayerDataQueue {
    entries: Mutex<VecDeque<PlayerDataQueueEntry>>,
}

impl CPlayerDataQueue {
    pub(crate) const fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::new()),
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

    /// Уничтожает всех player-owner-ов и records в FIFO-порядке под одним lock.
    pub(crate) fn clear(&self) {
        let mut entries = self.entries.lock();
        while let Some(mut entry) = entries.pop_front() {
            drop(entry.player.take());
            drop(entry);
        }
    }
}

// Неперенесённый контракт (локальный анализ): `CPlayerDataQueue::ResetHonorElimilateInfo` RVA 0x000E6110.
// Для каждого queued non-null `pPlayer`: при `flags & 2` обнуляет weeks,
// при `flags & 4` — months, затем безусловно days honor-eliminate counter.
// BLOCKED_MISSING_FACT: соответствующие поля `CPlayer::tagBaseProperty` ещё не
// достигнуты текущим call-chain; метод не вызывается ProcessPlayerDataQueue.

// IMPLEMENTED: `CPlayerDataQueue::Clear` RVA 0x000E6230 находится выше.
// VERIFIED_DISASSEMBLY:
// Exact EXE `0x004E6242..0x004E6309` проходит все deque records, virtual-удаляет
// каждый non-null player, обнуляет pointer, удаляет record и возвращается к
// условию; `0x004E630E..0x004E6321` tidy-ит deque и снимает critical section.
// Ложный ранний return декомпилятора не переносится. `Mutex<VecDeque<_>>` и
// `Box/Drop` заменяют только critical section, deque allocation и destructors.
