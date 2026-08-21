//! Журнал аукциона исторического `WorldServer`.
//!
//! Статус `CAuctionLog`, destructor, `AddItem`, `ComputePage`,
//! `AddByteAtCurPage`, `AddByteGoodsLog`, `CollectNoNotice` и
//! `SendAuctionMsg2GS` — `IMPLEMENTED`; `LoadItem` и `UpdateAuctionBangDB`
//! ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.h/.cpp`.
//!
//! PDB задаёт `stLogNode` размером `0x150` и exact offsets: восемь `long`
//! `0x00..0x1c`, `SYSTEMTIME` `0x20`, `strDescri[256]` `0x30`, GUID-ы
//! `0x130/0x140`. `stBangNode` — три `unsigned long`, размер `0x0c`.
//! `BTreeMap<i32, Vec<_>>` заменяет MSVC `multimap<long, stLogNode>`:
//! numeric порядок ключей и insertion order равных ключей сохраняются, а
//! allocator/tree cleanup переданы Rust. `m_mapPlayerPage` также заменён
//! `BTreeMap`; process-static singleton выражается явным owned `CAuctionLog`.
//!
//! `ComputePage` проверен по machine-коду `0x0044B130..0x0044B1DF`.
//! Первый запрос создаёт page `0` и возвращает `true`; направление `1`
//! увеличивает page, откатывает его при неполной следующей странице, но в
//! обоих случаях возвращает `false`. Направление `2` возвращает `true` только
//! после реального уменьшения, прочие значения сбрасывают page в ноль и
//! возвращают `true`. Страница содержит ровно до 17 записей.
//!
//! `AddByteGoodsLog` не ищет максимальный timestamp: machine-код
//! `0x0044A038..0x0044A073` сравнивает время каждой подходящей записи с
//! нулевым `SYSTEMTIME` и перезаписывает результат, поэтому выбирается
//! последняя вставленная подходящая запись с представимым временем. Для
//! корректных современных DB-дат `chrono` заменяет `_mktime`; нормализация
//! повреждённых полей старым CRT не объявляется контрактом и возвращает typed
//! block. Миллисекунды и day-of-week, как в exact comparator, не участвуют.
//!
//! `CollectNoNotice` намеренно не фильтрует уже отмеченные записи: копирует все
//! записи игрока, затем по одной ставит live `bNotice=1` и в том же порядке
//! передаёт SQL внешней write-log очереди, после чего пишет старые снимки
//! целиком по `0x150` байт. Лимиты, batch-транзакция и pending-claim из старого
//! Linux C++ донора меняли этот контракт и не перенесены. Формирование SQL,
//! контейнеры и message storage используют стандартные Rust типы; тонкий
//! queue-trait оставляет владельцу очереди только исходный порядок публикации.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::public::guid::CGuid;

const AUCTION_LOG_RECORDS_PER_PAGE: i32 = 17;
const AUCTION_LOG_NODE_SIZE: usize = 0x150;
const AUCTION_LOG_DESCRIPTION_SIZE: usize = 0x100;

/// Exact Windows `SYSTEMTIME` аукционного журнала.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct AuctionLogSystemTime {
    pub(crate) year: u16,
    pub(crate) month: u16,
    pub(crate) day_of_week: u16,
    pub(crate) day: u16,
    pub(crate) hour: u16,
    pub(crate) minute: u16,
    pub(crate) second: u16,
    pub(crate) milliseconds: u16,
}

impl AuctionLogSystemTime {
    fn to_legacy_bytes(self) -> [u8; 0x10] {
        let mut bytes = [0; 0x10];
        for (index, value) in [
            self.year,
            self.month,
            self.day_of_week,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.milliseconds,
        ]
        .into_iter()
        .enumerate()
        {
            let offset = index * 2;
            bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    fn is_after_zero_baseline(self) -> Result<bool, AuctionLogTimeBlock> {
        if self.year == 0
            && self.month == 0
            && self.day == 0
            && self.hour == 0
            && self.minute == 0
            && self.second == 0
        {
            return Ok(false);
        }
        let Some(date) = NaiveDate::from_ymd_opt(
            i32::from(self.year),
            u32::from(self.month),
            u32::from(self.day),
        ) else {
            return Err(AuctionLogTimeBlock { time: self });
        };
        let Some(time) = date.and_hms_opt(
            u32::from(self.hour),
            u32::from(self.minute),
            u32::from(self.second),
        ) else {
            return Err(AuctionLogTimeBlock { time: self });
        };

        Ok(time.and_utc().timestamp() > -1)
    }
}

/// Полный PDB-layout `CAuctionLog::stLogNode`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct AuctionLogNode {
    pub(crate) base_id: i32,
    pub(crate) operation_type: i32,
    pub(crate) money_type: i32,
    pub(crate) money_num: i32,
    pub(crate) player_id: i32,
    pub(crate) amount: i32,
    pub(crate) fee: i32,
    pub(crate) notice: i32,
    pub(crate) time: AuctionLogSystemTime,
    pub(crate) description: [u8; AUCTION_LOG_DESCRIPTION_SIZE],
    pub(crate) guid: CGuid,
    pub(crate) guid_key: CGuid,
}

impl Default for AuctionLogNode {
    fn default() -> Self {
        Self {
            base_id: 0,
            operation_type: 0,
            money_type: 0,
            money_num: 0,
            player_id: 0,
            amount: 0,
            fee: 0,
            notice: 0,
            time: Default::default(),
            description: [0; AUCTION_LOG_DESCRIPTION_SIZE],
            guid: CGuid::GUID_INVALID,
            guid_key: CGuid::GUID_INVALID,
        }
    }
}

impl AuctionLogNode {
    fn to_legacy_bytes(&self) -> [u8; AUCTION_LOG_NODE_SIZE] {
        let mut bytes = [0; AUCTION_LOG_NODE_SIZE];
        for (offset, value) in [
            (0x00, self.base_id),
            (0x04, self.operation_type),
            (0x08, self.money_type),
            (0x0c, self.money_num),
            (0x10, self.player_id),
            (0x14, self.amount),
            (0x18, self.fee),
            (0x1c, self.notice),
        ] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[0x20..0x30].copy_from_slice(&self.time.to_legacy_bytes());
        bytes[0x30..0x130].copy_from_slice(&self.description);
        bytes[0x130..0x140].copy_from_slice(self.guid.as_legacy_bytes());
        bytes[0x140..0x150].copy_from_slice(self.guid_key.as_legacy_bytes());
        bytes
    }

    fn description_wire_bytes(&self) -> Option<&[u8]> {
        let terminator = self.description.iter().position(|byte| *byte == 0)?;
        Some(&self.description[..=terminator])
    }
}

const _: () = {
    assert!(std::mem::size_of::<AuctionLogSystemTime>() == 0x10);
    assert!(std::mem::size_of::<AuctionLogNode>() == AUCTION_LOG_NODE_SIZE);
    assert!(std::mem::offset_of!(AuctionLogNode, player_id) == 0x10);
    assert!(std::mem::offset_of!(AuctionLogNode, time) == 0x20);
    assert!(std::mem::offset_of!(AuctionLogNode, description) == 0x30);
    assert!(std::mem::offset_of!(AuctionLogNode, guid) == 0x130);
    assert!(std::mem::offset_of!(AuctionLogNode, guid_key) == 0x140);
};

/// Exact 12-байтовый элемент рассылки `AuctionMostGoods`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct AuctionBangNode {
    pub(crate) base_id: u32,
    pub(crate) count: u32,
    pub(crate) operation: u32,
}

const _: () = assert!(std::mem::size_of::<AuctionBangNode>() == 0x0c);

/// Невоспроизводимая безопасно `_mktime`-нормализация повреждённой DB-даты.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuctionLogTimeBlock {
    pub(crate) time: AuctionLogSystemTime,
}

/// Первая UB-граница C-string сериализации одной страницы.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuctionLogPageBlock {
    pub(crate) player_id: i32,
    pub(crate) record_index: usize,
}

/// Наблюдаемый результат `AddByteAtCurPage` до возможной string-границы.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionLogPageWriteDisposition {
    MissingMessage,
    MissingPage,
    Written { page: i32, record_count: usize },
}

/// Наблюдаемый результат `AddByteGoodsLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuctionGoodsLogWriteOutcome {
    MissingMessage,
    Written { found: bool },
    BlockedMissingFact(AuctionLogTimeBlock),
}

/// Граница исходной `CWriteLogQueue::PushWriteLogData`.
pub(crate) trait AuctionNoticeWriteQueue {
    fn push_auction_notice_sql(&mut self, sql: String);
}

/// Итог `CollectNoNotice` после публикации SQL и полного wire payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuctionNoticeCollection {
    pub(crate) player_id: i32,
    pub(crate) record_count: usize,
}

/// Owned-состояние исходного `CAuctionLog` без process-static singleton-а.
pub(crate) struct CAuctionLog {
    /// Constructor не инициализировал это слово; оно становится известным
    /// только после достигнутой daily-ranking ветви.
    old_auction_day: Option<i32>,
    log_list: BTreeMap<i32, Vec<AuctionLogNode>>,
    player_pages: BTreeMap<i32, i32>,
    goods_list: Vec<AuctionBangNode>,
}

impl Default for CAuctionLog {
    fn default() -> Self {
        Self::new()
    }
}

impl CAuctionLog {
    /// Создаёт только доказанные constructor-ом пустые контейнеры.
    pub(crate) const fn new() -> Self {
        Self {
            old_auction_day: None,
            log_list: BTreeMap::new(),
            player_pages: BTreeMap::new(),
            goods_list: Vec::new(),
        }
    }

    pub(crate) const fn old_auction_day(&self) -> Option<i32> {
        self.old_auction_day
    }

    pub(crate) fn set_old_auction_day(&mut self, day: i32) {
        self.old_auction_day = Some(day);
    }

    /// Multimap insert всегда принимает ещё одну запись и возвращает `true`.
    pub(crate) fn add_item(&mut self, item: AuctionLogNode) -> bool {
        self.log_list.entry(item.player_id).or_default().push(item);
        true
    }

    /// Сохраняет exact переход page и исторически странный bool результата.
    pub(crate) fn compute_page(&mut self, direction: i32, player_id: i32) -> bool {
        let Some(page) = self.player_pages.get_mut(&player_id) else {
            self.player_pages.insert(player_id, 0);
            return true;
        };

        match direction {
            1 => {
                let count = self
                    .log_list
                    .get(&player_id)
                    .map_or(0, Vec::len) as u32 as i32;
                *page = page.wrapping_add(1);
                if count < page.wrapping_mul(AUCTION_LOG_RECORDS_PER_PAGE) {
                    *page = page.wrapping_sub(1);
                }
                false
            }
            2 if *page > 0 => {
                *page -= 1;
                true
            }
            2 => false,
            _ => {
                *page = 0;
                true
            }
        }
    }

    /// Добавляет player/page и до семнадцати сокращённых записей.
    pub(crate) fn add_byte_at_current_page(
        &self,
        player_id: i32,
        message: Option<&mut CMessage>,
    ) -> Result<AuctionLogPageWriteDisposition, AuctionLogPageBlock> {
        let Some(message) = message else {
            return Ok(AuctionLogPageWriteDisposition::MissingMessage);
        };
        let Some(&page) = self.player_pages.get(&player_id) else {
            return Ok(AuctionLogPageWriteDisposition::MissingPage);
        };

        let records = self
            .log_list
            .get(&player_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let skip = page.wrapping_mul(AUCTION_LOG_RECORDS_PER_PAGE);
        let page_records = if skip < 0 {
            &[][..]
        } else {
            records
                .get(usize::try_from(skip).expect("неотрицательный i32 помещается в usize")..)
                .unwrap_or(&[])
        };
        let page_records =
            &page_records[..page_records.len().min(AUCTION_LOG_RECORDS_PER_PAGE as usize)];

        message.base_mut().add_long(player_id);
        message.base_mut().add_long(page_records.len() as i32);
        for (record_index, record) in page_records.iter().enumerate() {
            message.base_mut().add(&record.time.to_legacy_bytes());
            message.base_mut().add_long(record.operation_type);
            message.base_mut().add_long(record.money_type);
            message.base_mut().add_long(record.money_num);
            message.base_mut().add_long(record.amount);
            message.base_mut().add_long(record.fee);
            let Some(description) = record.description_wire_bytes() else {
                return Err(AuctionLogPageBlock {
                    player_id,
                    record_index,
                });
            };
            message.base_mut().add(description);
        }

        Ok(AuctionLogPageWriteDisposition::Written {
            page,
            record_count: page_records.len(),
        })
    }

    /// Добавляет последнюю вставленную подходящую запись одного GUID.
    pub(crate) fn add_byte_goods_log(
        &self,
        player_id: i32,
        guid: CGuid,
        message: Option<&mut CMessage>,
    ) -> AuctionGoodsLogWriteOutcome {
        let Some(message) = message else {
            return AuctionGoodsLogWriteOutcome::MissingMessage;
        };

        let mut selected = None;
        if let Some(records) = self.log_list.get(&player_id) {
            for record in records {
                if record.player_id != player_id || record.guid != guid {
                    continue;
                }
                match record.time.is_after_zero_baseline() {
                    Ok(true) => selected = Some(record),
                    Ok(false) => {}
                    Err(block) => {
                        return AuctionGoodsLogWriteOutcome::BlockedMissingFact(block);
                    }
                }
            }
        }

        let Some(record) = selected else {
            message.base_mut().add_long(0);
            return AuctionGoodsLogWriteOutcome::Written { found: false };
        };
        message.base_mut().add_long(1);
        message.base_mut().add_long(record.money_type);
        message.base_mut().add(&record.time.to_legacy_bytes());
        message.base_mut().add_long(record.money_num);
        message.base_mut().add_long(record.amount);
        AuctionGoodsLogWriteOutcome::Written { found: true }
    }

    /// Публикует все записи игрока, включая ранее отмеченные, в exact порядке.
    pub(crate) fn collect_no_notice<Q: AuctionNoticeWriteQueue>(
        &mut self,
        player_id: i32,
        message: &mut CMessage,
        write_queue: &mut Q,
    ) -> AuctionNoticeCollection {
        let mut snapshots = Vec::new();
        if let Some(records) = self.log_list.get_mut(&player_id) {
            snapshots.reserve(records.len());
            for record in records {
                let snapshot = record.clone();
                record.notice = 1;
                write_queue.push_auction_notice_sql(format!(
                    "UPDATE AuctionLog set bNotice = 1 where guidkey = '{}' and guid = '{}' and opttype = {}",
                    snapshot.guid_key, snapshot.guid, snapshot.operation_type,
                ));
                snapshots.push(snapshot);
            }
        }

        message.base_mut().add_long(player_id);
        message.base_mut().add_long(snapshots.len() as i32);
        for snapshot in &snapshots {
            message.base_mut().add(&snapshot.to_legacy_bytes());
        }

        AuctionNoticeCollection {
            player_id,
            record_count: snapshots.len(),
        }
    }

    /// Строит `0x8040C` и сохраняет исходную отправку по numeric map ID.
    pub(crate) fn send_auction_msg_to_game_server(
        &self,
        player_id: u32,
        map_id: u32,
        sender: Option<&ServerCommandHandle>,
    ) -> Result<i32, SendMessageError> {
        let mut message = CMessage::new(0x0008_040c);
        message.base_mut().add_ulong(player_id);
        for goods in &self.goods_list {
            message.base_mut().add_ulong(goods.base_id);
            message.base_mut().add_ulong(goods.count);
            message.base_mut().add_ulong(goods.operation);
        }
        message.send_to_map_id(sender, map_id as i32)
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp

// ============================================================================
// FUNCTION: CAuctionLog::GetInstance
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.h:117
// RVA: 0x000016B0
// ADDRESS: 004016b0
// PROTOTYPE: CAuctionLog * __cdecl GetInstance(void)
//
// Process-static lazy pointer заменён явным Rust-owner и `CAuctionLog::new`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::SendAuctionMsg2GS
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:460
// RVA: 0x00049D60
// ADDRESS: 00449d60
// PROTOTYPE: void __thiscall SendAuctionMsg2GS(ulong param_1, ulong param_2)
//
// Реализовано выше через готовый World `CMessage` и server sender.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::AddByteGoodsLog
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:369
// RVA: 0x00049F70
// ADDRESS: 00449f70
// PROTOTYPE: void __thiscall AddByteGoodsLog(long param_1, CGUID param_2, CMessage * param_3)
//
// Реализовано выше; zero-SYSTEMTIME comparator подтверждён machine-кодом.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::ComputePage
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:277
// RVA: 0x0004B130
// ADDRESS: 0044b130
// PROTOTYPE: bool __thiscall ComputePage(long param_1, long param_2)
//
// Реализовано выше с exact bool по `0x0044B130..0x0044B1DF`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::AddItem
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:244
// RVA: 0x0004B2E0
// ADDRESS: 0044b2e0
// PROTOTYPE: bool __thiscall AddItem(stLogNode param_1)
//
// Реализовано выше как ordered multimap-key insertion.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::~CAuctionLog
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:19
// RVA: 0x0004B4D0
// ADDRESS: 0044b4d0
// PROTOTYPE: void __thiscall ~CAuctionLog(void)
//
// Очистка трёх контейнеров передана Rust ownership/Drop.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::AddByteAtCurPage
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:322
// RVA: 0x0004B590
// ADDRESS: 0044b590
// PROTOTYPE: void __thiscall AddByteAtCurPage(long param_1, CMessage * param_2)
//
// Реализовано выше с exact page-size 17 и wire-порядком полей.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::CollectNoNotice
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:415
// RVA: 0x0004B840
// ADDRESS: 0044b840
// PROTOTYPE: void __thiscall CollectNoNotice(long param_1, CMessage * param_2)
//
// Реализовано выше с последовательным live update, SQL queue и full-struct wire.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::CAuctionLog
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:14
// RVA: 0x0004BAF0
// ADDRESS: 0044baf0
// PROTOTYPE: undefined __thiscall CAuctionLog(void)
//
// Реализовано выше; неинициализированный `m_lAucOldDay` сохранён как `Option`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::LoadItem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:26
// RVA: 0x0004BBB0
// ADDRESS: 0044bbb0
// PROTOTYPE: bool __thiscall LoadItem(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044cbb1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:115
// RVA: 0x0004CBB1
// ADDRESS: 0044cbb1
// PROTOTYPE: undefined Catch@0044cbb1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionLog::UpdateAuctionBangDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:129
// RVA: 0x0004CBF0
// ADDRESS: 0044cbf0
// PROTOTYPE: bool __thiscall UpdateAuctionBangDB(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044d3e9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:204
// RVA: 0x0004D3E9
// ADDRESS: 0044d3e9
// PROTOTYPE: undefined Catch@0044d3e9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044d464
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:172
// RVA: 0x0004D464
// ADDRESS: 0044d464
// PROTOTYPE: undefined Catch@0044d464()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044d4a6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:227
// RVA: 0x0004D4A6
// ADDRESS: 0044d4a6
// PROTOTYPE: undefined Catch@0044d4a6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0044d4e1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp:234
// RVA: 0x0004D4E1
// ADDRESS: 0044d4e1
// PROTOTYPE: undefined FUN_0044d4e1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535ac0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\auctionlog.cpp
// RVA: 0x00135AC0
// ADDRESS: 00535ac0
// PROTOTYPE: undefined Unwind@00535ac0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
