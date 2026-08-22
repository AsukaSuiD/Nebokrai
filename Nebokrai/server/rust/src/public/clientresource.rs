//! Частично восстановленный read-side `CClientResource`.
//!
//! `GetPackage` и `IsFileExist` materialized поверх точных `FilesInfo` и
//! `PackageArchive`: `BTreeMap<u32, _>` заменяет только STL-владение. Полный
//! `LoadEx` остаётся RAW из-за неустановленного return эпилога; готовый
//! registry принимает уже прочитанные `.ril` и `.pak` owners.

use std::collections::BTreeMap;

use crate::public::filesinfo::{FileInfo, FilesInfo};
use crate::public::package::PackageArchive;

/// Связанный read-side owner одного World resource набора.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClientResource {
    files_info: FilesInfo,
    packages: BTreeMap<u32, PackageArchive>,
}

impl ClientResource {
    /// Принимает результаты exact `.ril` и `.pak` owner-ов в порядке `LoadEx`.
    pub(crate) fn new(files_info: FilesInfo, packages: BTreeMap<u32, PackageArchive>) -> Self {
        Self {
            files_info,
            packages,
        }
    }

    /// Повторяет nullable `CClientResource::GetPackage`.
    pub(crate) fn package(&self, package_type: u32) -> Option<&PackageArchive> {
        self.packages.get(&package_type)
    }

    /// Повторяет `IsFileExist` после normalizing lookup text.
    pub(crate) fn file_info(&self, path: &[u8]) -> Option<&FileInfo> {
        let mut normalized = path.to_vec();
        for byte in &mut normalized {
            byte.make_ascii_lowercase();
            if *byte == b'/' {
                *byte = b'\\';
            }
        }
        if normalized.first() != Some(&b'\\') {
            normalized.insert(0, b'\\');
        }
        self.files_info.file_info_by_text(&normalized)
    }

    pub(crate) fn is_file_exist(&self, path: &[u8]) -> bool {
        self.file_info(path).is_some()
    }
}

// COMPONENT_VARIANT_BEGIN: ServerUpdate
// Точная пара: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SHA-256 EXE: 21CDB7E22DE1DD9F4530C29ACEEA0E337E251BD082E0DF6674B4FD90050B2252
// SHA-256 PDB: FF4C7D2515C83D99253A9F518D7CAD7FE18BB060CA9BF934C8053024424F2E77
// Исходный владелец PDB: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp

// ============================================================================
// FUNCTION: CClientResource::UpdateSave
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:373
// RVA: 0x00002840
// ADDRESS: 00402840
// PROTOTYPE: bool __thiscall UpdateSave(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::GetPackageForUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:463
// RVA: 0x00002930
// ADDRESS: 00402930
// PROTOTYPE: CPackage * __thiscall GetPackageForUpdate(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::~CClientResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:75
// RVA: 0x00003640
// ADDRESS: 00403640
// PROTOTYPE: void __thiscall ~CClientResource(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::CClientResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:66
// RVA: 0x000038A0
// ADDRESS: 004038a0
// PROTOTYPE: void __thiscall CClientResource(eResourceType param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, HWND__ * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::LoadExForAutoUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:292
// RVA: 0x00003990
// ADDRESS: 00403990
// PROTOTYPE: bool __thiscall LoadExForAutoUpdate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::ResetPackInfosForAutoUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp:318
// RVA: 0x00003B90
// ADDRESS: 00403b90
// PROTOTYPE: void __thiscall ResetPackInfosForAutoUpdate(list<tagPackFileInfo,std::allocator<tagPackFileInfo>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: __delayLoadHelper2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\clientresource.cpp
// RVA: 0x0002EEFA
// ADDRESS: 0042eefa
// PROTOTYPE: _func___cdecl_int * __cdecl __delayLoadHelper2(ImgDelayDescr * param_1, _func___cdecl_int * * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: ServerUpdate

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\clientresource.cpp

// ============================================================================
// FUNCTION: CClientResource::GetPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:455
// RVA: 0x001D1EA0
// ADDRESS: 005d1ea0
// PROTOTYPE: CPackage * __thiscall GetPackage(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005d2008
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x001D2008
// ADDRESS: 005d2008
// PROTOTYPE: undefined Catch@005d2008()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\clientresource.cpp

// ============================================================================
// FUNCTION: CClientResource::GetPackage
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:455
// RVA: 0x0004EC60
// ADDRESS: 0044ec60
// PROTOTYPE: CPackage * __thiscall GetPackage(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::LoadPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:127
// RVA: 0x0004F5C0
// ADDRESS: 0044f5c0
// PROTOTYPE: bool __thiscall LoadPackage(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::IsFileExist
// STATUS: IMPLEMENTED / ASCII_CONTRACT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:792
// RVA: 0x0004F750
// ADDRESS: 0044f750
// PROTOTYPE: bool __thiscall IsFileExist(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::FindFileList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:1878
// RVA: 0x0004F9E0
// ADDRESS: 0044f9e0
// PROTOTYPE: void __thiscall FindFileList(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, list<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::~CClientResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:75
// RVA: 0x0004FFD0
// ADDRESS: 0044ffd0
// PROTOTYPE: void __thiscall ~CClientResource(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::CClientResource
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:66
// RVA: 0x00050290
// ADDRESS: 00450290
// PROTOTYPE: undefined __thiscall CClientResource(eResourceType param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, HWND__ * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CClientResource::LoadEx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp:259
// RVA: 0x00050380
// ADDRESS: 00450380
// PROTOTYPE: bool __thiscall LoadEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d21e8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x000D21E8
// ADDRESS: 004d21e8
// PROTOTYPE: undefined Catch@004d21e8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d25a2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x000D25A2
// ADDRESS: 004d25a2
// PROTOTYPE: undefined Catch@004d25a2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00534fe0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\clientresource.cpp
// RVA: 0x00134FE0
// ADDRESS: 00534fe0
// PROTOTYPE: undefined Unwind@00534fe0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
