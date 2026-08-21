//! Владелец конфигурации ежедневных LeiTing-действий.
//!
//! `SetDailyUpdateStamp`, `AddToByteArray`, GameServer
//! `DecordFromByteArray` и WorldServer `GetDailyThingList` — `IMPLEMENTED`;
//! text loader ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точные пары EXE/PDB и исходные
//! owners сохранены у raw-блоков.
//!
//! Исходный singleton/static deque заменён обычным `CThingSetup` и
//! `VecDeque`. Wire остаётся signed count, затем записи `u16 TID/max/point`
//! по шесть байт. Daily projection сохраняет странные границы: постоянны
//! только TID `> 1999`, недельны строго `1000 < TID < 2000`, а weekday
//! снимается отдельным platform-вызовом для каждого недельного элемента.
//! Если подходящих элементов нет, старый owner не очищал destination; Rust
//! также оставляет его без изменения.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingLocalTime {
    pub(crate) second: i32,
    pub(crate) minute: i32,
    pub(crate) hour: i32,
    pub(crate) month_day: i32,
    pub(crate) month: i32,
    pub(crate) year_since_1900: i32,
    pub(crate) week_day: i32,
    pub(crate) year_day: i32,
    pub(crate) daylight_saving: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingThingNode {
    pub(crate) thing_id: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeiTingDailyThing {
    pub(crate) thing_id: u16,
    pub(crate) count: u16,
    pub(crate) max_count: u16,
    pub(crate) point: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThingSetupCodecError {
    NegativeCount(i32),
    CountOutsideLegacyRange(usize),
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for ThingSetupCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeCount(count) => {
                write!(formatter, "отрицательное число LeiTing-записей: {count}")
            }
            Self::CountOutsideLegacyRange(count) => write!(
                formatter,
                "число LeiTing-записей {count} не помещается в signed long"
            ),
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "LeiTing payload оборван на {offset}: требуется {needed}, доступно {available}"
            ),
        }
    }
}

impl Error for ThingSetupCodecError {}

#[derive(Default)]
pub(crate) struct CThingSetup {
    all_things: VecDeque<LeiTingThingNode>,
}

impl CThingSetup {
    pub(crate) const fn new() -> Self {
        Self {
            all_things: VecDeque::new(),
        }
    }

    /// Exact `23:59:59`; остальные поля `tm` не меняются.
    pub(crate) const fn set_daily_update_stamp(local_time: &mut LeiTingLocalTime) {
        local_time.hour = 23;
        local_time.minute = 59;
        local_time.second = 59;
    }

    /// Кодирует WorldServer initial-config projection.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), ThingSetupCodecError> {
        let count = i32::try_from(self.all_things.len()).map_err(|_| {
            ThingSetupCodecError::CountOutsideLegacyRange(self.all_things.len())
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for thing in &self.all_things {
            destination.extend_from_slice(&thing.thing_id.to_le_bytes());
            destination.extend_from_slice(&thing.max_count.to_le_bytes());
            destination.extend_from_slice(&thing.point.to_le_bytes());
        }
        Ok(())
    }

    /// Декодирует GameServer initial-config projection, сохраняя prefix при
    /// безопасной ошибке вместо исходного безразмерного overread.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), ThingSetupCodecError> {
        self.all_things.clear();
        let count = read_i32(source, cursor)?;
        if count < 0 {
            return Err(ThingSetupCodecError::NegativeCount(count));
        }
        for _ in 0..count {
            self.all_things.push_back(LeiTingThingNode {
                thing_id: read_u16(source, cursor)?,
                max_count: read_u16(source, cursor)?,
                point: read_u16(source, cursor)?,
            });
        }
        Ok(())
    }

    /// Собирает exact daily list; `get_week_day` вызывается отдельно для
    /// каждого недельного TID, как старый `GetLocalTime` внутри цикла.
    pub(crate) fn get_daily_thing_list(
        &self,
        mut get_week_day: impl FnMut() -> u16,
        destination: &mut VecDeque<LeiTingDailyThing>,
    ) {
        let mut daily = VecDeque::new();
        for node in &self.all_things {
            let thing_id = node.thing_id;
            let selected = thing_id > 1999
                || (thing_id > 1000
                    && thing_id < 2000
                    && get_week_day() == (thing_id % 1000) % 7);
            if selected {
                daily.push_back(LeiTingDailyThing {
                    thing_id,
                    count: 0,
                    max_count: node.max_count,
                    point: node.point,
                });
            }
        }
        if !daily.is_empty() {
            *destination = daily;
        }
    }
}

fn read_u16(source: &[u8], cursor: &mut usize) -> Result<u16, ThingSetupCodecError> {
    let offset = *cursor;
    let end = offset.saturating_add(2);
    let Some(bytes) = source.get(offset..end) else {
        return Err(ThingSetupCodecError::UnexpectedEnd {
            offset,
            needed: 2,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = end;
    Ok(u16::from_le_bytes(
        bytes.try_into().expect("slice содержит ровно два байта"),
    ))
}

fn read_i32(source: &[u8], cursor: &mut usize) -> Result<i32, ThingSetupCodecError> {
    let offset = *cursor;
    let end = offset.saturating_add(4);
    let Some(bytes) = source.get(offset..end) else {
        return Err(ThingSetupCodecError::UnexpectedEnd {
            offset,
            needed: 4,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = end;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("slice содержит ровно четыре байта"),
    ))
}

// Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже является локальной
// документацией оставшегося text loader-а и variant-доказательств.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp

// ============================================================================
// FUNCTION: CThingSetup::SetDailyUpdateStamp
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp:144
// RVA: 0x000DB340
// ADDRESS: 004db340
// PROTOTYPE: void __cdecl SetDailyUpdateStamp(tm * param_1)
//
// IMPLEMENTED_OWNER: `CThingSetup::set_daily_update_stamp` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CThingSetup::DecordFromByteArray
// STATUS: IMPLEMENTED_PARTIAL
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp:81
// RVA: 0x000DB5A0
// ADDRESS: 004db5a0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// IMPLEMENTED_OWNER: `CThingSetup::decord_from_byte_array` выше сохраняет
// clear/prefix/order; безразмерный overread заменён typed safe-границей.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c8d10
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C8D10
// ADDRESS: 005c8d10
// PROTOTYPE: undefined Catch@005c8d10()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c8fb3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C8FB3
// ADDRESS: 005c8fb3
// PROTOTYPE: undefined Catch@005c8fb3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c9272
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C9272
// ADDRESS: 005c9272
// PROTOTYPE: undefined Catch@005c9272()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c9964
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C9964
// ADDRESS: 005c9964
// PROTOTYPE: undefined Catch@005c9964()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c9a34
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C9A34
// ADDRESS: 005c9a34
// PROTOTYPE: undefined Catch@005c9a34()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c9b03
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C9B03
// ADDRESS: 005c9b03
// PROTOTYPE: undefined Catch@005c9b03()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::stNodeInfo::stNodeInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C9C00
// ADDRESS: 005c9c00
// PROTOTYPE: undefined __thiscall stNodeInfo(stNodeInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c9f4e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x001C9F4E
// ADDRESS: 005c9f4e
// PROTOTYPE: undefined Catch@005c9f4e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//











// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp

// ============================================================================
// FUNCTION: Catch@0047cb90
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007CB90
// ADDRESS: 0047cb90
// PROTOTYPE: undefined Catch@0047cb90()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047ce33
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007CE33
// ADDRESS: 0047ce33
// PROTOTYPE: undefined Catch@0047ce33()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d0f2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007D0F2
// ADDRESS: 0047d0f2
// PROTOTYPE: undefined Catch@0047d0f2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d294
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007D294
// ADDRESS: 0047d294
// PROTOTYPE: undefined Catch@0047d294()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d364
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007D364
// ADDRESS: 0047d364
// PROTOTYPE: undefined Catch@0047d364()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d433
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007D433
// ADDRESS: 0047d433
// PROTOTYPE: undefined Catch@0047d433()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::stNodeInfo::stNodeInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007D530
// ADDRESS: 0047d530
// PROTOTYPE: undefined __thiscall stNodeInfo(stNodeInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047d87e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x0007D87E
// ADDRESS: 0047d87e
// PROTOTYPE: undefined Catch@0047d87e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CThingSetup::SetDailyUpdateStamp
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp:144
// RVA: 0x00081AC0
// ADDRESS: 00481ac0
// PROTOTYPE: void __cdecl SetDailyUpdateStamp(tm * param_1)
//
// IMPLEMENTED_OWNER: `CThingSetup::set_daily_update_stamp` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CThingSetup::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp:68
// RVA: 0x00081AE0
// ADDRESS: 00481ae0
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED_OWNER: `CThingSetup::add_to_byte_array` выше сохраняет signed
// count и шесть raw field-байт каждой записи.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CThingSetup::LoadAllThingList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp:17
// RVA: 0x00081DB0
// ADDRESS: 00481db0
// PROTOTYPE: int __cdecl LoadAllThingList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CThingSetup::GetDailyThingList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp:121
// RVA: 0x00081FF0
// ADDRESS: 00481ff0
// PROTOTYPE: void __cdecl GetDailyThingList(deque<tagThing,std::allocator<tagThing>_> * param_1)
//
// IMPLEMENTED_OWNER: `CThingSetup::get_daily_thing_list` выше сохраняет
// TID-gates, отдельный weekday-read и conditional swap.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@00530d50
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\leitingsetup.cpp
// RVA: 0x00130D50
// ADDRESS: 00530d50
// PROTOTYPE: undefined Unwind@00530d50()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: WorldServer
