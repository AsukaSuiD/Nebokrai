//! Ограничения уничтожения предметов исторического Miracle.
//!
//! Статус World `CGoodsDestroySetup::AddToByteArray` RVA `0x0003F810`:
//! `IMPLEMENTED`; text loader, singleton, Game decoder и queries ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp:71`.
//!
//! Wire: `u32 enabled`, signed type count, ordered vector `u16 goods_type`,
//! signed original-name count и ordered vector C-строк. Повторы в обоих
//! vector значимы. Bool передаётся четырьмя байтами как `0/1`, а не одним
//! байтом. Owned bytes и `Vec` заменяют MSVC string/vector lifetime, сохраняя
//! byte-exact original names.

use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsDestroySetup {
    enabled: bool,
    goods_types: Vec<u16>,
    original_names: Vec<Vec<u8>>,
}

impl GoodsDestroySetup {
    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn push_goods_type(&mut self, goods_type: u16) {
        self.goods_types.push(goods_type);
    }

    pub(crate) fn push_original_name(&mut self, original_name: Vec<u8>) {
        self.original_names.push(original_name);
    }

    pub(crate) fn clear_lists(&mut self) {
        self.goods_types.clear();
        self.original_names.clear();
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GoodsDestroySerializeError> {
        destination.extend_from_slice(&u32::from(self.enabled).to_le_bytes());
        write_count(
            destination,
            self.goods_types.len(),
            GoodsDestroyList::GoodsTypes,
        )?;
        for &goods_type in &self.goods_types {
            destination.extend_from_slice(&goods_type.to_le_bytes());
        }

        write_count(
            destination,
            self.original_names.len(),
            GoodsDestroyList::OriginalNames,
        )?;
        for (name_index, original_name) in self.original_names.iter().enumerate() {
            if original_name.contains(&0) {
                return Err(GoodsDestroySerializeError::NameContainsNul { name_index });
            }
            destination.extend_from_slice(original_name);
            destination.push(0);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroyList {
    GoodsTypes,
    OriginalNames,
}

impl fmt::Display for GoodsDestroyList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::GoodsTypes => "типов предметов",
            Self::OriginalNames => "исходных имён предметов",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDestroySerializeError {
    CountOutOfRange {
        list: GoodsDestroyList,
        count: usize,
    },
    NameContainsNul {
        name_index: usize,
    },
}

impl fmt::Display for GoodsDestroySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { list, count } => write!(
                formatter,
                "GoodsDestroy содержит {count} {list} вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul { name_index } => write!(
                formatter,
                "исходное имя GoodsDestroy #{name_index} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for GoodsDestroySerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    list: GoodsDestroyList,
) -> Result<(), GoodsDestroySerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| GoodsDestroySerializeError::CountOutOfRange { list, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация loader-а, singleton,
// Game decoder-а и queries, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp

// ============================================================================
// FUNCTION: CGoodsDestroySetup::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.h:25
// RVA: 0x00093320
// ADDRESS: 00493320
// PROTOTYPE: CGoodsDestroySetup * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004c5ac2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp
// RVA: 0x000C5AC2
// ADDRESS: 004c5ac2
// PROTOTYPE: undefined Catch@004c5ac2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsDestroySetup::CGoodsDestroySetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp:14
// RVA: 0x001C0DF0
// ADDRESS: 005c0df0
// PROTOTYPE: undefined __thiscall CGoodsDestroySetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsDestroySetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp:92
// RVA: 0x001C0E80
// ADDRESS: 005c0e80
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp

// ============================================================================
// FUNCTION: CGoodsDestroySetup::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.h:25
// RVA: 0x000013A0
// ADDRESS: 004013a0
// PROTOTYPE: CGoodsDestroySetup * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsDestroySetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp:71
// RVA: 0x0003F810
// ADDRESS: 0043f810
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsDestroySetup::CGoodsDestroySetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp:14
// RVA: 0x0003FB30
// ADDRESS: 0043fb30
// PROTOTYPE: undefined __thiscall CGoodsDestroySetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsDestroySetup::LoadConfig
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp:28
// RVA: 0x0003FBC0
// ADDRESS: 0043fbc0
// PROTOTYPE: bool __thiscall LoadConfig(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004977d2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\goodsdestructionconfig.cpp
// RVA: 0x000977D2
// ADDRESS: 004977d2
// PROTOTYPE: undefined Catch@004977d2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
