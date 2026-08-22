//! Владелец конфигурации объединения боевых фей исторического `WorldServer`.
//!
//! Статус `GetInstance`, `tagCompose`/owner constructor,
//! `bLoadCombineConfig` и `AddToByteArray_Combine` RVA
//! `0x00001410/0x00040090/0x00040860/0x00040980/0x0003FF50` —
//! `IMPLEMENTED`; оставшиеся блоки ниже являются compiler/STL cleanup. Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp:53,64,365`.
//!
//! Singleton с process-lifetime leak заменён caller-owned registry, как и в
//! очищенном C++ reference. Exact loader возвращает false только при ошибке
//! открытия; после успешного чтения он очищает compose-vector и возвращает
//! true даже для пустого или повреждённого потока. Один временный `tagCompose`
//! переиспользуется между `#`-записями: при частичном последнем record уже
//! прочитанные поля меняются, остальные сохраняют прошлые значения, record
//! всё равно добавляется. Этот наблюдаемый malformed-input quirk сохранён.
//! Конструктор копирования `tagCompose` копирует четыре строки, расход,
//! побитово точный `f32` шанса и индекс; это обычный
//! `BattleFairyCompose: Clone`. Деструктор очищает только четыре строки и
//! заменён структурным Drop.
//!
//! Wire `0x2D` подтверждён exact ASM: signed 32-bit count и по `0x7C` сырых
//! bytes каждого MSVC `tagCompose`. GameServer decoder зануляет 124 bytes,
//! копирует record и вызывает string copy constructor; поэтому корректно
//! декодируются только SSO-строки длиной до 15 bytes. Для них Rust формирует
//! точный 32-bit MSVC layout: четыре нулевых allocator/padding bytes, 16-byte
//! inline buffer, size и capacity `15`. Старый путь для длинной строки посылал
//! адрес heap-памяти WorldServer и затем разыменовывал его в другом процессе —
//! внешний, но неработоспособный pointer/lifetime-дефект. Он не воспроизводится
//! молча: serializer возвращает typed compatibility error.

use std::error::Error;
use std::fmt;
use std::path::Path;

const COMPOSE_RECORD_SIZE: usize = 0x7c;
const LEGACY_STRING_SIZE: usize = 0x1c;
const LEGACY_STRING_INLINE_CAPACITY: usize = 15;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct BattleFairyCompose {
    pub(crate) fetch_stone: Vec<u8>,
    pub(crate) fetch_body: Vec<u8>,
    pub(crate) material: Vec<u8>,
    pub(crate) deplete_fetch: u32,
    pub(crate) success_rate: f32,
    pub(crate) battle_fairy: Vec<u8>,
    pub(crate) index: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BattleFairyLevel {
    pub(crate) equipment_level: i32,
    pub(crate) success_rate: f32,
    pub(crate) up_value: i32,
    pub(crate) total_value: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyComposeWireError {
    CountOutsideLegacyRange {
        count: usize,
    },
    StringRequiresLegacyHeapPointer {
        record: usize,
        field: &'static str,
        length: usize,
    },
}

impl fmt::Display for BattleFairyComposeWireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutsideLegacyRange { count } => {
                write!(
                    formatter,
                    "compose count {count} не помещается в signed legacy long"
                )
            }
            Self::StringRequiresLegacyHeapPointer {
                record,
                field,
                length,
            } => write!(
                formatter,
                "compose record {record}, поле {field} длиной {length} требует непереносимый MSVC heap pointer"
            ),
        }
    }
}

impl Error for BattleFairyComposeWireError {}

/// Достигнутое owning-состояние `CBattleFairyProperty`, не копия его ABI.
pub(crate) struct CBattleFairyProperty {
    compose: Vec<BattleFairyCompose>,
    levels: Vec<BattleFairyLevel>,
}

impl CBattleFairyProperty {
    /// Создаёт два пустых vector-а; остальные exact scalar defaults равны нулю
    /// и не имеют consumer-а в matching World PDB.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            compose: Vec::new(),
            levels: Vec::new(),
        }
    }

    pub(crate) fn compose(&self) -> &[BattleFairyCompose] {
        &self.compose
    }

    pub(crate) fn levels(&self) -> &[BattleFairyLevel] {
        &self.levels
    }

    /// При open-error сохраняет старое состояние, как исходный early return.
    pub(crate) fn load_combine_config_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        let source = std::fs::read(path)?;
        self.load_combine_config(&source);
        Ok(())
    }

    /// Загружает marker/token формат с exact reuse частичного record-а.
    pub(crate) fn load_combine_config(&mut self, source: &[u8]) {
        self.compose.clear();
        let mut cursor = 0usize;
        let mut current = BattleFairyCompose::default();
        while let Some(marker) = source[cursor..].iter().position(|byte| *byte == b'#') {
            cursor += marker + 1;
            let mut failed = false;
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.fetch_stone);
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.fetch_body);
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.material);
            read_u32_field(source, &mut cursor, &mut failed, &mut current.deplete_fetch);
            read_f32_field(source, &mut cursor, &mut failed, &mut current.success_rate);
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.battle_fairy);
            read_u32_field(source, &mut cursor, &mut failed, &mut current.index);
            self.compose.push(current.clone());
            if failed {
                break;
            }
        }
    }

    /// Кодирует client-facing `0x2D` payload в доказанном MSVC SSO-domain.
    pub(crate) fn serialize_combine(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), BattleFairyComposeWireError> {
        let count = i32::try_from(self.compose.len()).map_err(|_| {
            BattleFairyComposeWireError::CountOutsideLegacyRange {
                count: self.compose.len(),
            }
        })?;
        for (record, compose) in self.compose.iter().enumerate() {
            validate_legacy_string(record, "strFetchStone", &compose.fetch_stone)?;
            validate_legacy_string(record, "strFetchBody", &compose.fetch_body)?;
            validate_legacy_string(record, "strMaterial", &compose.material)?;
            validate_legacy_string(record, "strBattleFairy", &compose.battle_fairy)?;
        }
        destination.extend_from_slice(&count.to_le_bytes());
        for compose in &self.compose {
            let mut record = [0u8; COMPOSE_RECORD_SIZE];
            write_legacy_string(&mut record, 0x00, &compose.fetch_stone);
            write_legacy_string(&mut record, 0x1c, &compose.fetch_body);
            write_legacy_string(&mut record, 0x38, &compose.material);
            record[0x54..0x58].copy_from_slice(&compose.deplete_fetch.to_le_bytes());
            record[0x58..0x5c].copy_from_slice(&compose.success_rate.to_bits().to_le_bytes());
            write_legacy_string(&mut record, 0x5c, &compose.battle_fairy);
            record[0x78..0x7c].copy_from_slice(&compose.index.to_le_bytes());
            destination.extend_from_slice(&record);
        }
        Ok(())
    }
}

fn next_token<'source>(source: &'source [u8], cursor: &mut usize) -> Option<&'source [u8]> {
    while source.get(*cursor).is_some_and(u8::is_ascii_whitespace) {
        *cursor += 1;
    }
    let start = *cursor;
    while source
        .get(*cursor)
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        *cursor += 1;
    }
    (start != *cursor).then_some(&source[start..*cursor])
}

fn read_bytes_field(source: &[u8], cursor: &mut usize, failed: &mut bool, out: &mut Vec<u8>) {
    if *failed {
        return;
    }
    let Some(token) = next_token(source, cursor) else {
        *failed = true;
        return;
    };
    out.clear();
    out.extend_from_slice(token);
}

fn read_u32_field(source: &[u8], cursor: &mut usize, failed: &mut bool, out: &mut u32) {
    if *failed {
        return;
    }
    let value = next_token(source, cursor)
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse().ok());
    match value {
        Some(value) => *out = value,
        None => *failed = true,
    }
}

fn read_f32_field(source: &[u8], cursor: &mut usize, failed: &mut bool, out: &mut f32) {
    if *failed {
        return;
    }
    let value = next_token(source, cursor)
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse().ok());
    match value {
        Some(value) => *out = value,
        None => *failed = true,
    }
}

fn validate_legacy_string(
    record: usize,
    field: &'static str,
    value: &[u8],
) -> Result<(), BattleFairyComposeWireError> {
    if value.len() <= LEGACY_STRING_INLINE_CAPACITY {
        return Ok(());
    }
    Err(
        BattleFairyComposeWireError::StringRequiresLegacyHeapPointer {
            record,
            field,
            length: value.len(),
        },
    )
}

fn write_legacy_string(record: &mut [u8; COMPOSE_RECORD_SIZE], offset: usize, value: &[u8]) {
    let buffer = offset + 4;
    record[buffer..buffer + value.len()].copy_from_slice(value);
    record[offset + 0x14..offset + 0x18].copy_from_slice(&(value.len() as u32).to_le_bytes());
    record[offset + 0x18..offset + LEGACY_STRING_SIZE]
        .copy_from_slice(&(LEGACY_STRING_INLINE_CAPACITY as u32).to_le_bytes());
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp

// ============================================================================
// FUNCTION: CBattleFairyProperty::GetInstance
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.h:86
// RVA: 0x00001410
// ADDRESS: 00401410
// PROTOTYPE: CBattleFairyProperty * __cdecl GetInstance(void)
//
// Process-global raw singleton заменён caller-owned `CBattleFairyProperty`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::AddToByteArray_Combine
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp:365
// RVA: 0x0003FF50
// ADDRESS: 0043ff50
// PROTOTYPE: bool __thiscall AddToByteArray_Combine(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED выше как `serialize_combine`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::tagCompose
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.h:53
// RVA: 0x00040090
// ADDRESS: 00440090
// PROTOTYPE: undefined __thiscall tagCompose(void)
//
// IMPLEMENTED через `BattleFairyCompose::default`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::CBattleFairyProperty
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp:53
// RVA: 0x00040860
// ADDRESS: 00440860
// PROTOTYPE: undefined __thiscall CBattleFairyProperty(void)
//
// IMPLEMENTED выше как `with_constructor_defaults`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::bLoadCombineConfig
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp:64
// RVA: 0x00040980
// ADDRESS: 00440980
// PROTOTYPE: bool __thiscall bLoadCombineConfig(void)
//
// IMPLEMENTED выше; exact ASM подтверждает false только при open-error,
// ранний clear после успешного открытия и безусловный true normal-return.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00451ded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00051DED
// ADDRESS: 00451ded
// PROTOTYPE: undefined Catch@00451ded()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00451ea6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00051EA6
// ADDRESS: 00451ea6
// PROTOTYPE: undefined Catch@00451ea6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00452089
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00052089
// ADDRESS: 00452089
// PROTOTYPE: undefined Catch@00452089()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00452213
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00052213
// ADDRESS: 00452213
// PROTOTYPE: undefined Catch@00452213()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00452a3a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00052A3A
// ADDRESS: 00452a3a
// PROTOTYPE: undefined Catch@00452a3a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00452aee
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00052AEE
// ADDRESS: 00452aee
// PROTOTYPE: undefined Catch@00452aee()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00452d9e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00052D9E
// ADDRESS: 00452d9e
// PROTOTYPE: undefined Catch@00452d9e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00452e5d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x00052E5D
// ADDRESS: 00452e5d
// PROTOTYPE: undefined Catch@00452e5d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052ee60
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x0012EE60
// ADDRESS: 0052ee60
// PROTOTYPE: undefined Unwind@0052ee60()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052eee0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x0012EEE0
// ADDRESS: 0052eee0
// PROTOTYPE: undefined Unwind@0052eee0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052ef20
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x0012EF20
// ADDRESS: 0052ef20
// PROTOTYPE: undefined Unwind@0052ef20()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052ef40
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cbattlefairyproperty.cpp
// RVA: 0x0012EF40
// ADDRESS: 0052ef40
// PROTOTYPE: undefined Unwind@0052ef40()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
