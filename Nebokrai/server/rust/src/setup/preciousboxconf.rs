//! Конфигурация Precious Box исторического Miracle.
//!
//! Статус World `AddToByteArray` RVA `0x00046220`: `IMPLEMENTED`;
//! `load_conf`, singleton plumbing и Game decoder/random owner ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:316`.
//!
//! Wire начинается с signed количества box-ов. Далее ordered map выдаёт для
//! каждого box `i32 id + i32 odds_count`; каждая группа содержит
//! `i32 min_odds + i32 max_odds + i32 item_count`, а каждый предмет —
//! `i32 item_idx + i32 min_level + i32 max_level + i32 amount + bool-byte`.
//! Исходные 20-байтные MSVC Item содержат три байта padding после bool, но
//! serializer намеренно передаёт только 17 значимых байт.
//!
//! Важная неизвестность не замаскирована старым донором: точный World
//! `load_conf` очищает оба map-а и публикует XML только в `_box_conf`, тогда как
//! точный serializer читает `_box`. `archive/Miracle_server_linux` добавляет
//! отдельный `PreciousBoxRanges::Build`, которого нет в RAW/EXE owner-е.
//! Поэтому этот проход моделирует уже материализованный конечный `_box`, но не
//! выдаёт неподтверждённый XML→range ремонт за оригинальную семантику.
//! `BTreeMap` заменяет только MSVC ordered tree, `Vec`/`Drop` — ручной lifetime.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreciousBoxItem {
    pub(crate) item_idx: i32,
    pub(crate) min_level: i32,
    pub(crate) max_level: i32,
    pub(crate) amount: i32,
    pub(crate) broadcast: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBoxOdds {
    pub(crate) min_odds: i32,
    pub(crate) max_odds: i32,
    pub(crate) items: Vec<PreciousBoxItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBox {
    pub(crate) odds: Vec<PreciousBoxOdds>,
}

/// Safe owner уже материализованного World `_box` state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBoxConf {
    boxes: BTreeMap<i32, PreciousBox>,
}

impl PreciousBoxConf {
    /// Явная граница для будущего подтверждённого XML→range materializer-а.
    pub(crate) fn insert_box(&mut self, box_id: i32, value: PreciousBox) -> Option<PreciousBox> {
        self.boxes.insert(box_id, value)
    }

    pub(crate) fn clear(&mut self) {
        self.boxes.clear();
    }

    /// Дописывает exact compact wire без C++ Item padding.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PreciousBoxSerializeError> {
        write_count(destination, self.boxes.len(), PreciousBoxCount::Boxes)?;
        for (&box_id, box_value) in &self.boxes {
            destination.extend_from_slice(&box_id.to_le_bytes());
            write_count(
                destination,
                box_value.odds.len(),
                PreciousBoxCount::Odds { box_id },
            )?;
            for (odds_index, odds) in box_value.odds.iter().enumerate() {
                destination.extend_from_slice(&odds.min_odds.to_le_bytes());
                destination.extend_from_slice(&odds.max_odds.to_le_bytes());
                write_count(
                    destination,
                    odds.items.len(),
                    PreciousBoxCount::Items { box_id, odds_index },
                )?;
                for item in &odds.items {
                    destination.extend_from_slice(&item.item_idx.to_le_bytes());
                    destination.extend_from_slice(&item.min_level.to_le_bytes());
                    destination.extend_from_slice(&item.max_level.to_le_bytes());
                    destination.extend_from_slice(&item.amount.to_le_bytes());
                    destination.push(u8::from(item.broadcast));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreciousBoxCount {
    Boxes,
    Odds { box_id: i32 },
    Items { box_id: i32, odds_index: usize },
}

impl fmt::Display for PreciousBoxCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boxes => formatter.write_str("box-ов PreciousBox"),
            Self::Odds { box_id } => write!(formatter, "групп вероятностей box {box_id}"),
            Self::Items { box_id, odds_index } => {
                write!(formatter, "предметов группы {odds_index} box {box_id}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreciousBoxSerializeError {
    pub(crate) field: PreciousBoxCount,
    pub(crate) count: usize,
}

impl fmt::Display for PreciousBoxSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "количество {} ({}) не помещается в signed 32-битный диапазон",
            self.field, self.count
        )
    }
}

impl Error for PreciousBoxSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    field: PreciousBoxCount,
) -> Result<(), PreciousBoxSerializeError> {
    let count_i32 = i32::try_from(count).map_err(|_| PreciousBoxSerializeError { field, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация спорного World loader-а,
// singleton, Game decoder-а и random owner-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp

// ============================================================================
// FUNCTION: PreciousBoxConf::inst
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h:72
// RVA: 0x0009CE00
// ADDRESS: 0049ce00
// PROTOTYPE: PreciousBoxConf * __cdecl inst(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::random_item
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:394
// RVA: 0x001C57B0
// ADDRESS: 005c57b0
// PROTOTYPE: bool __thiscall random_item(long param_1, long * param_2, long * param_3, long * param_4, bool * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:355
// RVA: 0x001C7030
// ADDRESS: 005c7030
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::~PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:18
// RVA: 0x001C7640
// ADDRESS: 005c7640
// PROTOTYPE: void __thiscall ~PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:16
// RVA: 0x001C76E0
// ADDRESS: 005c76e0
// PROTOTYPE: undefined __thiscall PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp

// ============================================================================
// FUNCTION: PreciousBoxConf::inst
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h:72
// RVA: 0x000015D0
// ADDRESS: 004015d0
// PROTOTYPE: PreciousBoxConf * __cdecl inst(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044544c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp
// RVA: 0x0004544C
// ADDRESS: 0044544c
// PROTOTYPE: undefined Catch@0044544c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00445477
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp
// RVA: 0x00045477
// ADDRESS: 00445477
// PROTOTYPE: undefined FUN_00445477()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:316
// RVA: 0x00046220
// ADDRESS: 00446220
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::load_conf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:22
// RVA: 0x00048420
// ADDRESS: 00448420
// PROTOTYPE: bool __thiscall load_conf(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::~PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:18
// RVA: 0x00048FC0
// ADDRESS: 00448fc0
// PROTOTYPE: void __thiscall ~PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:16
// RVA: 0x00049060
// ADDRESS: 00449060
// PROTOTYPE: undefined __thiscall PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: WorldServer
