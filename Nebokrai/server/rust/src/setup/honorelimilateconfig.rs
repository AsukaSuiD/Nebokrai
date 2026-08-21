//! Ограничения начисления honor за убийство в историческом Miracle.
//!
//! Статус World `HonorElimilateConfig::AddToByteArray` RVA `0x0008A560`:
//! `IMPLEMENTED`; singleton lifecycle, loader и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp:50`.
//!
//! Exact World serializer и Game decoder подтверждают единственный wire:
//! `level_difference`, затем `minimum_level`, оба signed little-endian `long`.
//! Отдельные таблицы `CHonorRanks` сюда не входят и отправляются следующими
//! subtype `0x27/0x28`. Два typed `i32` заменяют process-global singleton без
//! изменения payload; неизвестный legacy return loader-а не выдумывается.

/// Восьмибайтовый wire-value вместо singleton `HonorElimilateConfig`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HonorElimilateConfig {
    pub(crate) level_difference: i32,
    pub(crate) minimum_level: i32,
}

impl HonorElimilateConfig {
    pub(crate) fn add_to_byte_array(&self, destination: &mut Vec<u8>) {
        destination.extend_from_slice(&self.level_difference.to_le_bytes());
        destination.extend_from_slice(&self.minimum_level.to_le_bytes());
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp

// ============================================================================
// FUNCTION: HonorElimilateConfig::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp:8
// RVA: 0x000F83A0
// ADDRESS: 004f83a0
// PROTOTYPE: HonorElimilateConfig * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: HonorElimilateConfig::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp:56
// RVA: 0x000F83E0
// ADDRESS: 004f83e0
// PROTOTYPE: void __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp

// ============================================================================
// FUNCTION: HonorElimilateConfig::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp:50
// RVA: 0x0008A560
// ADDRESS: 0048a560
// PROTOTYPE: void __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: HonorElimilateConfig::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp:8
// RVA: 0x0008A590
// ADDRESS: 0048a590
// PROTOTYPE: HonorElimilateConfig * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: HonorElimilateConfig::LoadConfig
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\honorelimilateconfig.cpp:27
// RVA: 0x0008A5D0
// ADDRESS: 0048a5d0
// PROTOTYPE: bool __thiscall LoadConfig(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
