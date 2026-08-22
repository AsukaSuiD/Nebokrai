//! Частично восстановленный read-side владелец `public/package.cpp`.
//!
//! Точная World-пара подтверждает package header из трёх little-endian `u32`,
//! инвертированные записи индекса размером `0x118`, ASCII-lowercase ключи и
//! копирование сжатого blob по `dwOffset/dwSize`. Это materialized ниже без
//! `FILE*`, ручных буферов и read-after-short-read дефектов.
//!
//! `DeCompressData` (LZO) и `DeCompress` (zlib) остаются `UNKNOWN` (исследовательский декомпилят хранится локально):
//! выбор/результат декомпрессора является следующей совместимой границей.
//! Сырой C++ ниже остаётся доказательной заготовкой, а не Rust-реализацией.

use std::collections::BTreeMap;

const PACKAGE_HEADER_LEN: usize = 12;
const FILE_INDEX_LEN: usize = 0x118;
const FILE_INDEX_NAME_LEN: usize = 256;

/// Поля одной подтверждённой `tagFileIndex` записи.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageFileIndex {
    name: Vec<u8>,
    offset: u32,
    size: u32,
    origin_size: u32,
    valid_size: u32,
    crc32: u32,
    compress_type: u32,
}

impl PackageFileIndex {
    pub(crate) fn size(&self) -> u32 {
        self.size
    }
    pub(crate) fn origin_size(&self) -> u32 {
        self.origin_size
    }
    pub(crate) fn valid_size(&self) -> u32 {
        self.valid_size
    }
    pub(crate) fn crc32(&self) -> u32 {
        self.crc32
    }
    pub(crate) fn compress_type(&self) -> u32 {
        self.compress_type
    }
}

/// Ошибка safe read-side, не являющаяся историческим bool `CPackage::Open`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PackageReadError {
    TruncatedHeader,
    IndexSizeOutsideFile,
    IndexCountOutsideHeader,
    IndexNameWithoutNul,
    DataOutsideFile,
    BufferTooSmall { required: u32, available: u32 },
}

/// Владеющий снимок `.pak`, доступный будущему `CClientResource/rfOpen`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageArchive {
    bytes: Vec<u8>,
    indexes: BTreeMap<Vec<u8>, PackageFileIndex>,
}

impl PackageArchive {
    /// Читает подтверждённую read-side часть `CPackage::Open`.
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Result<Self, PackageReadError> {
        if bytes.len() < PACKAGE_HEADER_LEN {
            return Err(PackageReadError::TruncatedHeader);
        }
        let index_head_size = read_u32(&bytes, 4).ok_or(PackageReadError::TruncatedHeader)?;
        let index_count = read_u32(&bytes, 8).ok_or(PackageReadError::TruncatedHeader)?;
        let capacity = index_head_size as usize / FILE_INDEX_LEN;
        if index_count as usize > capacity {
            return Err(PackageReadError::IndexCountOutsideHeader);
        }
        let index_end = PACKAGE_HEADER_LEN
            .checked_add(index_count as usize * FILE_INDEX_LEN)
            .ok_or(PackageReadError::IndexSizeOutsideFile)?;
        if index_end > bytes.len() {
            return Err(PackageReadError::IndexSizeOutsideFile);
        }

        let mut indexes = BTreeMap::new();
        for number in 0..index_count as usize {
            let start = PACKAGE_HEADER_LEN + number * FILE_INDEX_LEN;
            let mut raw = [0_u8; FILE_INDEX_LEN];
            raw.copy_from_slice(&bytes[start..start + FILE_INDEX_LEN]);
            for byte in &mut raw {
                *byte = !*byte;
            }
            let nul = raw[..FILE_INDEX_NAME_LEN]
                .iter()
                .position(|byte| *byte == 0)
                .ok_or(PackageReadError::IndexNameWithoutNul)?;
            let name = lowercase_ascii(&raw[..nul]);
            indexes.insert(
                name.clone(),
                PackageFileIndex {
                    name,
                    offset: read_u32(&raw, 256).expect("полная tagFileIndex"),
                    size: read_u32(&raw, 260).expect("полная tagFileIndex"),
                    origin_size: read_u32(&raw, 264).expect("полная tagFileIndex"),
                    valid_size: read_u32(&raw, 268).expect("полная tagFileIndex"),
                    crc32: read_u32(&raw, 272).expect("полная tagFileIndex"),
                    compress_type: read_u32(&raw, 276).expect("полная tagFileIndex"),
                },
            );
        }
        Ok(Self { bytes, indexes })
    }

    pub(crate) fn file_size(&self, name: &[u8]) -> u32 {
        self.indexes
            .get(&lowercase_ascii(name))
            .map_or(0, PackageFileIndex::size)
    }

    /// Повторяет successful copy-path `ExtractToBuf`, не декомпрессируя blob.
    pub(crate) fn extract_compressed(
        &self,
        name: &[u8],
        capacity: u32,
    ) -> Result<Option<(PackageFileIndex, Vec<u8>)>, PackageReadError> {
        let Some(index) = self.indexes.get(&lowercase_ascii(name)) else {
            return Ok(None);
        };
        if index.size > capacity {
            return Err(PackageReadError::BufferTooSmall {
                required: index.size,
                available: capacity,
            });
        }
        let start = index.offset as usize;
        let end = start
            .checked_add(index.size as usize)
            .ok_or(PackageReadError::DataOutsideFile)?;
        let payload = self
            .bytes
            .get(start..end)
            .ok_or(PackageReadError::DataOutsideFile)?
            .to_vec();
        Ok(Some((index.clone(), payload)))
    }
}

fn lowercase_ascii(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes
        .get(offset..offset + 4)
        .map(|part| u32::from_le_bytes(part.try_into().expect("ровно четыре байта")))
}

// COMPONENT_VARIANT_BEGIN: ServerUpdate
// Точная пара: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SHA-256 EXE: 21CDB7E22DE1DD9F4530C29ACEEA0E337E251BD082E0DF6674B4FD90050B2252
// SHA-256 PDB: FF4C7D2515C83D99253A9F518D7CAD7FE18BB060CA9BF934C8053024424F2E77
// Исходный владелец PDB: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp

// ============================================================================
// FUNCTION: Catch@0040484a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0000484A
// ADDRESS: 0040484a
// PROTOTYPE: undefined __stdcall Catch@0040484a(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00404e92
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x00004E92
// ADDRESS: 00404e92
// PROTOTYPE: undefined __stdcall Catch@00404e92(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DeCompressData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:50
// RVA: 0x000071F0
// ADDRESS: 004071f0
// PROTOTYPE: void __cdecl DeCompressData(uchar * param_1, ulong param_2, uchar * param_3, ulong * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DeCompress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:66
// RVA: 0x00007230
// ADDRESS: 00407230
// PROTOTYPE: int __cdecl DeCompress(uchar * param_1, ulong param_2, uchar * param_3, ulong * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::CloseFileHandle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:106
// RVA: 0x00007250
// ADDRESS: 00407250
// PROTOTYPE: void __thiscall CloseFileHandle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::WriteIndexEx
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:922
// RVA: 0x000077A0
// ADDRESS: 004077a0
// PROTOTYPE: bool __thiscall WriteIndexEx(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::OpenFileHandle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:92
// RVA: 0x00008140
// ADDRESS: 00408140
// PROTOTYPE: void __thiscall OpenFileHandle(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::AddFileData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:480
// RVA: 0x000082B0
// ADDRESS: 004082b0
// PROTOTYPE: ulong __thiscall AddFileData(char * param_1, uchar * param_2, ulong * param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::ExtractToFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:550
// RVA: 0x000084B0
// ADDRESS: 004084b0
// PROTOTYPE: ulong __thiscall ExtractToFile(char * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::ExtractToBuf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:689
// RVA: 0x000089E0
// ADDRESS: 004089e0
// PROTOTYPE: bool __thiscall ExtractToBuf(char * param_1, uchar * * param_2, int param_3, tagFileIndex * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::GetEmptyPart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:745
// RVA: 0x00008B90
// ADDRESS: 00408b90
// PROTOTYPE: bool __thiscall GetEmptyPart(ulong param_1, tagFileIndex * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::Create
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:200
// RVA: 0x00009380
// ADDRESS: 00409380
// PROTOTYPE: bool __thiscall Create(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::ClearData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:231
// RVA: 0x00009590
// ADDRESS: 00409590
// PROTOTYPE: bool __thiscall ClearData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::AddEmptyPart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:738
// RVA: 0x00009630
// ADDRESS: 00409630
// PROTOTYPE: void __thiscall AddEmptyPart(tagFileIndex * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::~CPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:87
// RVA: 0x000099C0
// ADDRESS: 004099c0
// PROTOTYPE: void __thiscall ~CPackage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::CPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:73
// RVA: 0x00009BC0
// ADDRESS: 00409bc0
// PROTOTYPE: void __thiscall CPackage(CClientResource * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::Open
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:131
// RVA: 0x00009CC0
// ADDRESS: 00409cc0
// PROTOTYPE: bool __thiscall Open(char * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0040a00a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:191
// RVA: 0x0000A00A
// ADDRESS: 0040a00a
// PROTOTYPE: undefined4 __stdcall Catch@0040a00a(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::WriteData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:1081
// RVA: 0x0000A030
// ADDRESS: 0040a030
// PROTOTYPE: ulong __thiscall WriteData(tagFileIndex * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::OpenForUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:115
// RVA: 0x0000A4B0
// ADDRESS: 0040a4b0
// PROTOTYPE: bool __thiscall OpenForUpdate(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::AddFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp:457
// RVA: 0x0000A620
// ADDRESS: 0040a620
// PROTOTYPE: ulong __thiscall AddFile(char * param_1, int param_2, ulong param_3, ulong param_4, ulong param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L116461
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C930
// ADDRESS: 0043c930
// PROTOTYPE: undefined __stdcall $L116461(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L116754
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C950
// ADDRESS: 0043c950
// PROTOTYPE: undefined __stdcall $L116754(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L117558
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C970
// ADDRESS: 0043c970
// PROTOTYPE: undefined __stdcall $L117558(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L117559
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C978
// ADDRESS: 0043c978
// PROTOTYPE: undefined __stdcall $L117559(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L117560
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C980
// ADDRESS: 0043c980
// PROTOTYPE: undefined __stdcall $L117560(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L118572
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C9A0
// ADDRESS: 0043c9a0
// PROTOTYPE: undefined __stdcall $L118572(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L118859
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C9C0
// ADDRESS: 0043c9c0
// PROTOTYPE: undefined __stdcall $L118859(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L120533
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C9E0
// ADDRESS: 0043c9e0
// PROTOTYPE: undefined __stdcall $L120533(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L120534
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003C9E8
// ADDRESS: 0043c9e8
// PROTOTYPE: undefined __stdcall $L120534(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L121296
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA00
// ADDRESS: 0043ca00
// PROTOTYPE: undefined __stdcall $L121296(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L121297
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA0B
// ADDRESS: 0043ca0b
// PROTOTYPE: undefined __stdcall $L121297(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L121298
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA16
// ADDRESS: 0043ca16
// PROTOTYPE: undefined __stdcall $L121298(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L121299
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA21
// ADDRESS: 0043ca21
// PROTOTYPE: undefined __stdcall $L121299(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L121618
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA40
// ADDRESS: 0043ca40
// PROTOTYPE: undefined __stdcall $L121618(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L122415
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA60
// ADDRESS: 0043ca60
// PROTOTYPE: undefined __stdcall $L122415(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L122416
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA68
// ADDRESS: 0043ca68
// PROTOTYPE: undefined __stdcall $L122416(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L122417
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA70
// ADDRESS: 0043ca70
// PROTOTYPE: undefined __stdcall $L122417(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126065
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA90
// ADDRESS: 0043ca90
// PROTOTYPE: undefined __stdcall $L126065(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126066
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CA9B
// ADDRESS: 0043ca9b
// PROTOTYPE: undefined __stdcall $L126066(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126067
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CAA6
// ADDRESS: 0043caa6
// PROTOTYPE: undefined __stdcall $L126067(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126265
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CAC0
// ADDRESS: 0043cac0
// PROTOTYPE: undefined __stdcall $L126265(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126486
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CAE0
// ADDRESS: 0043cae0
// PROTOTYPE: undefined __stdcall $L126486(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126487
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CAE8
// ADDRESS: 0043cae8
// PROTOTYPE: undefined __stdcall $L126487(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126488
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CAF0
// ADDRESS: 0043caf0
// PROTOTYPE: undefined __stdcall $L126488(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L126489
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CAFB
// ADDRESS: 0043cafb
// PROTOTYPE: undefined __stdcall $L126489(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L127544
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CB10
// ADDRESS: 0043cb10
// PROTOTYPE: undefined __stdcall $L127544(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L127545
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CB18
// ADDRESS: 0043cb18
// PROTOTYPE: undefined __stdcall $L127545(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L127547
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CB20
// ADDRESS: 0043cb20
// PROTOTYPE: undefined __stdcall $L127547(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: $L129476
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: ServerUpdate
// ARTIFACT: GameServer/ServerUpdate.exe + GameServer/ServerUpdate.pdb
// SOURCE: d:\йЈЋдє‘\fengyun_els\src\public\package.cpp
// RVA: 0x0003CB40
// ADDRESS: 0043cb40
// PROTOTYPE: undefined __stdcall $L129476(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: ServerUpdate

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\package.cpp

// ============================================================================
// FUNCTION: DeCompressData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:50
// RVA: 0x001D1A20
// ADDRESS: 005d1a20
// PROTOTYPE: void __cdecl DeCompressData(uchar * param_1, ulong param_2, uchar * param_3, ulong * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DeCompress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:66
// RVA: 0x001D1A60
// ADDRESS: 005d1a60
// PROTOTYPE: int __cdecl DeCompress(uchar * param_1, ulong param_2, uchar * param_3, ulong * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::ExtractToBuf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:656
// RVA: 0x001D1BB0
// ADDRESS: 005d1bb0
// PROTOTYPE: bool __thiscall ExtractToBuf(char * param_1, uchar * * param_2, int param_3, ulong * param_4, ulong * param_5, ulong * param_6, ulong * param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::GetFileSize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:716
// RVA: 0x001D1D70
// ADDRESS: 005d1d70
// PROTOTYPE: long __thiscall GetFileSize(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\package.cpp

// ============================================================================
// FUNCTION: DeCompressData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:50
// RVA: 0x000CFED0
// ADDRESS: 004cfed0
// PROTOTYPE: void __cdecl DeCompressData(uchar * param_1, ulong param_2, uchar * param_3, ulong * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: DeCompress
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:66
// RVA: 0x000CFF10
// ADDRESS: 004cff10
// PROTOTYPE: int __cdecl DeCompress(uchar * param_1, ulong param_2, uchar * param_3, ulong * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::ExtractToBuf
// STATUS: IMPLEMENTED / RESULT_MAPPING_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:656
// RVA: 0x000D04A0
// ADDRESS: 004d04a0
// PROTOTYPE: bool __thiscall ExtractToBuf(char * param_1, uchar * * param_2, int param_3, ulong * param_4, ulong * param_5, ulong * param_6, ulong * param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::GetFileSize
// STATUS: IMPLEMENTED / RESULT_MAPPING_SPLIT
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:716
// RVA: 0x000D0660
// ADDRESS: 004d0660
// PROTOTYPE: long __thiscall GetFileSize(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::OpenFileHandle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:92
// RVA: 0x000D0D60
// ADDRESS: 004d0d60
// PROTOTYPE: void __thiscall OpenFileHandle(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::~CPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:87
// RVA: 0x000D19F0
// ADDRESS: 004d19f0
// PROTOTYPE: void __thiscall ~CPackage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::CPackage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:73
// RVA: 0x000D1BF0
// ADDRESS: 004d1bf0
// PROTOTYPE: undefined __thiscall CPackage(CClientResource * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPackage::Open
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:131
// RVA: 0x000D1D00
// ADDRESS: 004d1d00
// PROTOTYPE: bool __thiscall Open(char * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d2050
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\package.cpp:191
// RVA: 0x000D2050
// ADDRESS: 004d2050
// PROTOTYPE: undefined Catch@004d2050()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
