//! Глобальный gameplay snapshot исторического Miracle.
//!
//! Статус World `CGlobeSetup::AddToByteArray` RVA `0x00033470`:
//! `IMPLEMENTED`; loaders, accessors и Game decoder side effects ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:836`.
//!
//! Наблюдаемый протокол здесь намеренно является raw ABI snapshot: EXE сначала
//! копирует ровно `0x1114` байт static `m_stSetup`, затем дописывает полный
//! `CRegionRouter` wire с legacy `sendSelf=true` (параметр router serializer не
//! читает). Game decoder забирает те же `0x1114` байт без field conversion.
//! Поэтому fixed byte array — точная модель wire, а не перенос C++ ownership;
//! typed loaders/accessors могут безопасно накладывать подтверждённые offsets
//! поверх него. Static storage оригинала было zero-initialized, что Rust
//! сохраняет через `Default` без утечки padding/heap-мусора.

use crate::setup::regionrouter::{RegionRouter, RegionRouterSerializeError};

pub(crate) const GLOBE_SETUP_BLOB_LENGTH: usize = 0x1114;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobeSetupSnapshot {
    bytes: [u8; GLOBE_SETUP_BLOB_LENGTH],
}

impl Default for GlobeSetupSnapshot {
    fn default() -> Self {
        Self {
            bytes: [0; GLOBE_SETUP_BLOB_LENGTH],
        }
    }
}

impl GlobeSetupSnapshot {
    pub(crate) fn from_bytes(bytes: [u8; GLOBE_SETUP_BLOB_LENGTH]) -> Self {
        Self { bytes }
    }

    pub(crate) fn bytes_mut(&mut self) -> &mut [u8; GLOBE_SETUP_BLOB_LENGTH] {
        &mut self.bytes
    }

    pub(crate) fn add_to_byte_array(
        &self,
        router: &RegionRouter,
        destination: &mut Vec<u8>,
    ) -> Result<(), RegionRouterSerializeError> {
        destination.extend_from_slice(&self.bytes);
        router.add_to_byte_array(destination)
    }
}

// Сырой C++ ниже сохранён как локальная документация loaders, accessors и
// Game decoder side effects, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp

// ============================================================================
// FUNCTION: CGlobeSetup::GetBaseMaxRp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:865
// RVA: 0x0001D610
// ADDRESS: 0041d610
// PROTOTYPE: ushort __cdecl GetBaseMaxRp(uchar param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:844
// RVA: 0x0001E4A0
// ADDRESS: 0041e4a0
// PROTOTYPE: bool __cdecl DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//







// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp

// ============================================================================
// FUNCTION: CGlobeSetup::GetBaseMaxRp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:865
// RVA: 0x0002EFA0
// ADDRESS: 0042efa0
// PROTOTYPE: ushort __cdecl GetBaseMaxRp(uchar param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::LoadAuctionGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:706
// RVA: 0x0002F1C0
// ADDRESS: 0042f1c0
// PROTOTYPE: int __cdecl LoadAuctionGoodsList(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::LoadGameSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:739
// RVA: 0x0002F330
// ADDRESS: 0042f330
// PROTOTYPE: int __cdecl LoadGameSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:27
// RVA: 0x00030150
// ADDRESS: 00430150
// PROTOTYPE: int __cdecl Load(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGlobeSetup::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp:836
// RVA: 0x00033470
// ADDRESS: 00433470
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00498dce
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp
// RVA: 0x00098DCE
// ADDRESS: 00498dce
// PROTOTYPE: undefined Catch@00498dce()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00531e60
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\globesetup.cpp
// RVA: 0x00131E60
// ADDRESS: 00531e60
// PROTOTYPE: undefined Unwind@00531e60()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
