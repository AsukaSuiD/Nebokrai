//! Таблица опыта боевых духов исторического Miracle.
//!
//! Статус World `CBattleFairyExpConfig::AddToByteArray` RVA `0x00049120`:
//! `IMPLEMENTED`; XML loaders, singleton plumbing и Game decoder/query ниже
//! остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp:175`.
//!
//! `CFairyExpConf` наследует этот owner и заполняет тот же protected map своим
//! XML loader-ом. Поэтому reconnect call-site получает `CFairyExpConf`
//! singleton, но вызывает именно base serializer. Wire: signed group count,
//! затем ordered `u32 owner_level + signed value_count + u32 exp...`; все поля
//! передаются little-endian по четыре байта. `BTreeMap<u32, Vec<u32>>`
//! сохраняет MSVC map/vector контракт, а Rust ownership заменяет singleton
//! allocation и ручной lifetime без изменения wire.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Safe owner общего state `CBattleFairyExpConfig`/`CFairyExpConf`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CBattleFairyExpConfig {
    exp_lists: BTreeMap<u32, Vec<u32>>,
}

impl CBattleFairyExpConfig {
    /// Явная граница для последующего точного XML loader-а производного owner-а.
    pub(crate) fn insert_exp_list(
        &mut self,
        owner_level: u32,
        values: Vec<u32>,
    ) -> Option<Vec<u32>> {
        self.exp_lists.insert(owner_level, values)
    }

    pub(crate) fn clear(&mut self) {
        self.exp_lists.clear();
    }

    /// Дописывает exact ordered map/vector wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), BattleFairyExpSerializeError> {
        write_count(destination, self.exp_lists.len(), None)?;
        for (&owner_level, values) in &self.exp_lists {
            destination.extend_from_slice(&owner_level.to_le_bytes());
            write_count(destination, values.len(), Some(owner_level))?;
            for &value in values {
                destination.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyExpSerializeError {
    pub(crate) owner_level: Option<u32>,
    pub(crate) count: usize,
}

impl fmt::Display for BattleFairyExpSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.owner_level {
            Some(owner_level) => write!(
                formatter,
                "таблица owner level {owner_level} содержит {} значений вне signed 32-битного диапазона",
                self.count
            ),
            None => write!(
                formatter,
                "FairyExp содержит {} групп вне signed 32-битного диапазона",
                self.count
            ),
        }
    }
}

impl Error for BattleFairyExpSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    owner_level: Option<u32>,
) -> Result<(), BattleFairyExpSerializeError> {
    let count_i32 = i32::try_from(count).map_err(|_| BattleFairyExpSerializeError {
        owner_level,
        count,
    })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация loaders, singleton и
// Game decoder/query, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.h:36
// RVA: 0x0009CD20
// ADDRESS: 0049cd20
// PROTOTYPE: CBattleFairyExpConfig * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::dwExpUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp:218
// RVA: 0x001C77A0
// ADDRESS: 005c77a0
// PROTOTYPE: ulong __thiscall dwExpUp(ulong param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp:193
// RVA: 0x001C7800
// ADDRESS: 005c7800
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.h:36
// RVA: 0x00001330
// ADDRESS: 00401330
// PROTOTYPE: CBattleFairyExpConfig * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::CBattleFairyExpConfig
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp:25
// RVA: 0x0003EFA0
// ADDRESS: 0043efa0
// PROTOTYPE: undefined __thiscall CBattleFairyExpConfig(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::bLoadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp:38
// RVA: 0x0003F200
// ADDRESS: 0043f200
// PROTOTYPE: bool __thiscall bLoadSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyExpConfig::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\cbattlefairyexpconfig.cpp:175
// RVA: 0x00049120
// ADDRESS: 00449120
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
