//! Аукционный журнал Realm: контракт записей `CAuctionLog::stLogNode`.
//!
//! Источник — точная пара `Nworldserver.exe`/`WorldServer.pdb`
//! (RSDS 289F1FB3-96A0-4FF4-8B5D-1FD17B50B751), PDB-путь владельца
//! `e:\svn\...\public\auctionroom\auctionlog.cpp`. `#[repr(C)]` layout 0x150
//! байт сохранён.

use chrono::NaiveDate;

use nebokrai_shared::values::CGuid;

pub const AUCTION_LOG_NODE_SIZE: usize = 0x150;
pub const AUCTION_LOG_DESCRIPTION_SIZE: usize = 0x100;

/// Оригинал Windows `SYSTEMTIME` аукционного журнала.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct AuctionLogSystemTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

impl AuctionLogSystemTime {
    pub fn to_legacy_bytes(self) -> [u8; 0x10] {
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

    pub fn is_after_zero_baseline(self) -> Result<bool, AuctionLogTimeBlock> {
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

/// Невоспроизводимая безопасно `_mktime`-нормализация повреждённой DB-даты.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuctionLogTimeBlock {
    pub time: AuctionLogSystemTime,
}

/// Полный PDB-layout `CAuctionLog::stLogNode`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AuctionLogNode {
    pub base_id: i32,
    pub operation_type: i32,
    pub money_type: i32,
    pub money_num: i32,
    pub player_id: i32,
    pub amount: i32,
    pub fee: i32,
    pub notice: i32,
    pub time: AuctionLogSystemTime,
    pub description: [u8; AUCTION_LOG_DESCRIPTION_SIZE],
    pub guid: CGuid,
    pub guid_key: CGuid,
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
    /// Декодирует точный raw `stLogNode[0x150]`, который World handler копировал
    /// через `_GetBufferFromByteArray` без field-wise wire преобразований.
    pub fn from_legacy_bytes(bytes: &[u8; AUCTION_LOG_NODE_SIZE]) -> Self {
        let read_i32 = |offset: usize| {
            i32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("поле stLogNode помещается в fixed layout"),
            )
        };
        let read_u16 = |offset: usize| {
            u16::from_le_bytes(
                bytes[offset..offset + 2]
                    .try_into()
                    .expect("SYSTEMTIME поле помещается в fixed layout"),
            )
        };
        let mut description = [0; AUCTION_LOG_DESCRIPTION_SIZE];
        description.copy_from_slice(&bytes[0x30..0x130]);
        let guid = CGuid::from_legacy_bytes(
            bytes[0x130..0x140]
                .try_into()
                .expect("auction GUID имеет 16 байт"),
        );
        let guid_key = CGuid::from_legacy_bytes(
            bytes[0x140..0x150]
                .try_into()
                .expect("auction GUID key имеет 16 байт"),
        );
        Self {
            base_id: read_i32(0x00),
            operation_type: read_i32(0x04),
            money_type: read_i32(0x08),
            money_num: read_i32(0x0c),
            player_id: read_i32(0x10),
            amount: read_i32(0x14),
            fee: read_i32(0x18),
            notice: read_i32(0x1c),
            time: AuctionLogSystemTime {
                year: read_u16(0x20),
                month: read_u16(0x22),
                day_of_week: read_u16(0x24),
                day: read_u16(0x26),
                hour: read_u16(0x28),
                minute: read_u16(0x2a),
                second: read_u16(0x2c),
                milliseconds: read_u16(0x2e),
            },
            description,
            guid,
            guid_key,
        }
    }

    pub fn description(&self) -> Option<&[u8]> {
        self.description
            .iter()
            .position(|byte| *byte == 0)
            .map(|terminator| &self.description[..terminator])
    }

    pub fn to_legacy_bytes(&self) -> [u8; AUCTION_LOG_NODE_SIZE] {
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

    pub fn description_wire_bytes(&self) -> Option<&[u8]> {
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
