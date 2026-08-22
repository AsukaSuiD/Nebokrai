//! Increment-shop журнал исторического WorldServer.
//!
//! `CIncrementLog` RVA `0x00072010/0x00072890..0x00072C00` восстановлен по
//! точной паре `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\incrementlog\incrementlog.cpp:7,16,29,58,69,83,102`.
//! DB-owner — `dbaccess/worlddb/dbincrementlog.cpp:13`.
//!
//! Registry остаётся player-keyed, записи внутри игрока сохраняют insertion/
//! DB cursor order. Страница содержит максимум `17` записей, считается от
//! newest к oldest и внутри страницы также идёт в обратном порядке. Exact EXE
//! `0x00472AA5` сбрасывает страницу только при `begin > size`, поэтому
//! `begin == size` успешно кодирует нулевую страницу. Отрицательная страница
//! становится нулевой; `page * 17` использует x86 wrapping arithmetic.
//!
//! `BTreeMap<i32, Vec<_>>`, owned byte-строки и обычный `Drop` заменяют только
//! `std::map<long, vector<pointer>*>`, ручные allocation/delete и singleton.
//! Отдельный mutex не нужен: owner передаётся как единственная mutable Rust-
//! ссылка. Donor-added календарные/ID/retention/description limits не являются
//! контрактом EXE и не перенесены. Если wrapping page даёт отрицательный
//! индекс либо переполняет следующий page-end, оригинал уходил в vector
//! range/UB; safe Rust возвращает локальный block и не выдаёт внутренний
//! memory defect за protocol-семантику.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::dbincrementlog::{
    DbIncrementLogLoadFailure, load_recent,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::public::date::TagTime;

const LOGS_PER_PAGE: i32 = 17;

#[derive(Clone, Debug)]
pub(crate) struct IncrementLogEntry {
    pub(crate) time: TagTime,
    pub(crate) entry_type: u8,
    pub(crate) money: i32,
    pub(crate) description: Vec<u8>,
}

#[derive(Debug)]
pub(crate) enum IncrementLogLoadOutcome {
    Loaded { rows: usize },
    Failed {
        rows_read: usize,
        source: DbIncrementLogLoadFailure,
    },
}

impl IncrementLogLoadOutcome {
    pub(crate) const fn succeeded(&self) -> bool {
        matches!(self, Self::Loaded { .. })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncrementLogPageBlock {
    EntryCountOutsideLegacyRange { entries: usize },
    WrappedPageProducedNegativeIndex { page: i32, begin: i32 },
    PageEndOutsideLegacyRange { page: i32, begin: i32 },
}

impl fmt::Display for IncrementLogPageBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCountOutsideLegacyRange { entries } => write!(
                formatter,
                "число increment-log записей {entries} не помещается в Windows long"
            ),
            Self::WrappedPageProducedNegativeIndex { page, begin } => write!(
                formatter,
                "страница increment-log {page} дала отрицательный x86-индекс {begin}"
            ),
            Self::PageEndOutsideLegacyRange { page, begin } => write!(
                formatter,
                "конец страницы increment-log {page} от индекса {begin} вышел за Windows long"
            ),
        }
    }
}

impl Error for IncrementLogPageBlock {}

#[derive(Default)]
pub(crate) struct CIncrementLog {
    entries: BTreeMap<i32, Vec<IncrementLogEntry>>,
}

impl CIncrementLog {
    pub(crate) const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub(crate) async fn load(
        &mut self,
        active_connection: Option<&mut WorldTdsClient>,
        retention_days: u32,
    ) -> IncrementLogLoadOutcome {
        let loaded = load_recent(active_connection, retention_days).await;
        let rows_read = loaded.rows.len();
        for row in loaded.rows {
            self.add(
                row.player_id,
                row.time,
                row.entry_type,
                row.money,
                &row.description,
            );
        }
        match loaded.completion {
            Ok(()) => IncrementLogLoadOutcome::Loaded {
                rows: rows_read,
            },
            Err(source) => IncrementLogLoadOutcome::Failed {
                rows_read,
                source,
            },
        }
    }

    pub(crate) fn uninitialize(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn add(
        &mut self,
        player_id: i32,
        time: TagTime,
        entry_type: u8,
        money: i32,
        description: &[u8],
    ) -> bool {
        if player_id == 0 {
            return false;
        }
        let description = description
            .iter()
            .copied()
            .take_while(|byte| *byte != 0)
            .collect();
        self.entries
            .entry(player_id)
            .or_default()
            .push(IncrementLogEntry {
                time,
                entry_type,
                money,
                description,
            });
        true
    }

    pub(crate) fn entries_by_player(&self, player_id: i32) -> Option<&[IncrementLogEntry]> {
        (player_id != 0)
            .then(|| self.entries.get(&player_id).map(Vec::as_slice))
            .flatten()
    }

    pub(crate) fn entries_count_by_player(&self, player_id: i32) -> usize {
        self.entries_by_player(player_id).map_or(0, <[_]>::len)
    }

    pub(crate) fn add_page_to_byte_array(
        &self,
        output: &mut Vec<u8>,
        page: i32,
        player_id: i32,
    ) -> Result<bool, IncrementLogPageBlock> {
        let Some(entries) = self.entries_by_player(player_id) else {
            return Ok(false);
        };
        let size = i32::try_from(entries.len()).map_err(|_| {
            IncrementLogPageBlock::EntryCountOutsideLegacyRange {
                entries: entries.len(),
            }
        })?;
        let page = page.max(0);
        let mut begin = page.wrapping_mul(LOGS_PER_PAGE);
        if size < begin {
            begin = 0;
        }
        if begin < 0 {
            return Err(IncrementLogPageBlock::WrappedPageProducedNegativeIndex {
                page,
                begin,
            });
        }
        let end = begin
            .checked_add(LOGS_PER_PAGE)
            .ok_or(IncrementLogPageBlock::PageEndOutsideLegacyRange { page, begin })?
            .min(size);
        output.extend_from_slice(&end.wrapping_sub(begin).to_le_bytes());
        let pages = size / LOGS_PER_PAGE + i32::from(size % LOGS_PER_PAGE != 0);
        output.extend_from_slice(&pages.to_le_bytes());

        for index in ((size - end)..(size - begin)).rev() {
            let entry = &entries[index as usize];
            append_time(output, entry.time);
            output.push(entry.entry_type);
            output.extend_from_slice(&entry.money.to_le_bytes());
            output.extend_from_slice(&entry.description);
            output.push(0);
        }
        Ok(true)
    }
}

fn append_time(output: &mut Vec<u8>, time: TagTime) {
    for value in [
        time.year,
        time.month,
        time.day_of_week,
        time.day,
        time.hour,
        time.minute,
        time.second,
        time.milliseconds,
    ] {
        output.extend_from_slice(&value.to_le_bytes());
    }
}
