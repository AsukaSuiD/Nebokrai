//! Ordered cache и initial-config serializer навыков WorldServer.
//!
//! Статус `CSkillFactory::Serialize` RVA `0x00060E90` и безопасной замены
//! `ClearSkillCache` RVA `0x00060DC0`, `StringToUsage` `0x00060F80`,
//! `ClearUsageCache` `0x00061E00`, content-половины `LoadConfigration`
//! `0x00062050` и `LoadUsage` `0x000628E0`: `IMPLEMENTED`; перечисление
//! ресурсов и файловые загрузчики cache ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp`.
//!
//! EXE пишет signed count map-а, затем в unsigned ascending-key порядке для
//! каждого slot-а `u32 length + record`. Null skill и skill с unknown type или
//! нулевым ID не удаляются из framing: им соответствует нулевая длина. Вопреки
//! сырому псевдокоду, точные инструкции после освобождения временного record-а
//! продолжают итерацию, а не выходят из функции.
//!
//! `BTreeMap` заменяет MSVC tree и сохраняет порядок. `Option<CSkill>` оставляет
//! выразимым доказанный null-slot, обычная вставка строит composite key
//! `id << 16 | level & 0xffff`. Замена duplicate key корректно освобождает
//! прежний owner вместо внутренней утечки старого `operator[]` call-site.
//! Отдельная byte-keyed карта usage сохраняет `operator[]`-перезапись в
//! `LoadUsage`, а `StringToUsage` возвращает `SKILL_USAGE_UNKNOW`
//! `0x7fff_ffff`.
//! `LoadConfigration`/`LoadUsage` получают уже прочитанные байты от внешнего
//! resource-owner-а: это заменяет только `CRFile`, `stringstream` и MessageBox,
//! сохраняя порядок публикации skill/usage и нормальный результат legacy.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::skill::{CSkill, SkillSerializeError, SkillTextError, SkillUsage};

/// `SKILL_USAGE_UNKNOW` из общего `SkillRelated.h` World/Game.
///
/// У PDB/EXE `StringToUsage` возвращает именно этот именованный sentinel;
/// public enum фиксирует его численное значение `0x7fff_ffff`. Он отличен от
/// нуля, поэтому неизвестная строка не должна превращаться в допустимый usage
/// в сериализуемой `(usage, cost)`-паре.
pub(crate) const UNKNOWN_SKILL_USAGE: u32 = 0x7fff_ffff;

/// Safe owner исходного process-global `g_mSkillMap`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CSkillFactory {
    skills: BTreeMap<u32, Option<CSkill>>,
    usage_names: BTreeMap<Vec<u8>, u32>,
}

impl CSkillFactory {
    /// Вставляет нормальный skill под exact composite key.
    pub(crate) fn insert(&mut self, skill: CSkill) -> Option<CSkill> {
        self.skills.insert(skill.cache_key(), Some(skill)).flatten()
    }

    /// Сохраняет выразимым legacy null-slot и его позицию в ordered framing.
    pub(crate) fn insert_slot(
        &mut self,
        key: u32,
        skill: Option<CSkill>,
    ) -> Option<Option<CSkill>> {
        self.skills.insert(key, skill)
    }

    pub(crate) fn get(&self, skill_id: u32, level: i32) -> Option<&CSkill> {
        let key = skill_id.wrapping_shl(16) | ((level as u32) & 0xffff);
        self.skills.get(&key).and_then(Option::as_ref)
    }

    pub(crate) fn clear_skill_cache(&mut self) {
        self.skills.clear();
    }

    /// Точная запись имени из `.usage`: `std::map::operator[]` заменял прежнее
    /// значение при том же byte-sensitive наборе байтов.
    pub(crate) fn set_usage_name(&mut self, name: &[u8], usage: u32) -> Option<u32> {
        self.usage_names.insert(name.to_vec(), usage)
    }

    /// Exact `StringToUsage`; null указатель и несуществующее имя возвращают
    /// `SKILL_USAGE_UNKNOW`, а не создают новую map-запись.
    pub(crate) fn string_to_usage(&self, name: Option<&[u8]>) -> u32 {
        name.and_then(|name| self.usage_names.get(visible_c_string(name)).copied())
            .unwrap_or(UNKNOWN_SKILL_USAGE)
    }

    /// Safe replacement `ClearUsageCache`; Rust Drop освобождает ключи вместе
    /// с map вместо ручного `_Tree::_Erase`.
    pub(crate) fn clear_usage_cache(&mut self) {
        self.usage_names.clear();
    }

    /// Content-половина exact `LoadUsage` после открытия `CRFile`.
    ///
    /// Каждая пара whitespace-токенов сразу заменяет существующий key. При
    /// оборванной/переполненной паре старый код мог читать неинициализированные
    /// stack-поля; Rust останавливает именно эту недоказанную UB-границу, не
    /// придумывая очередное имя либо usage.
    pub(crate) fn load_usage(&mut self, source: &[u8]) -> Result<usize, SkillFactoryLoadError> {
        let mut cursor = 0;
        let mut loaded = 0;
        while let Some(name) = next_whitespace_token(source, &mut cursor) {
            let value = next_whitespace_token(source, &mut cursor).ok_or(
                SkillFactoryLoadError::MissingUsageValue {
                    name: name.to_vec(),
                },
            )?;
            let usage = parse_legacy_u32(value, "usage")?;
            self.set_usage_name(name, usage);
            loaded += 1;
        }
        Ok(loaded)
    }

    /// Content-половина exact `LoadConfigration` после открытия `CRFile`.
    ///
    /// `Read`/`ReadTo` последовательно ищут четыре ASCII marker-а. До первого
    /// `Level` одиночные токены игнорируются; далее каждый не-`Level` токен
    /// потребляет одно unsigned cost-слово. Неизвестный usage не попадает в
    /// skill record, а повтор composite key освобождает прежний owner.
    pub(crate) fn load_configuration(
        &mut self,
        source: &[u8],
    ) -> Result<SkillConfigurationLoadReport, SkillFactoryLoadError> {
        let mut cursor = 0;
        let skill_type = read_named_u32(source, &mut cursor, b"Type", "Type")?;
        let skill_id = read_named_u32(source, &mut cursor, b"ID", "ID")?;
        seek_marker(source, &mut cursor, b"Name", "Name")?;
        let skill_name = next_whitespace_token(source, &mut cursor)
            .ok_or(SkillFactoryLoadError::MissingMarkerValue { marker: "Name" })?;
        let target_self = read_named_u32(source, &mut cursor, b"Target", "Target")?;

        let mut active_key = None;
        let mut loaded_levels = 0;
        while let Some(token) = next_whitespace_token(source, &mut cursor) {
            if token == b"Level" {
                let level = next_whitespace_token(source, &mut cursor)
                    .ok_or(SkillFactoryLoadError::MissingMarkerValue { marker: "Level" })?;
                let level = parse_legacy_i32(level, "Level")?;
                let mut skill = CSkill::new(skill_id as i32);
                skill.set_skill_type(skill_type as i32);
                skill.set_level(level);
                skill.set_skill_name(skill_name)
                    .map_err(SkillFactoryLoadError::SkillText)?;
                skill.set_description(&[])
                    .map_err(SkillFactoryLoadError::SkillText)?;
                if target_self != 0 {
                    skill.set_target_type(1);
                }
                let key = skill.cache_key();
                self.insert(skill);
                active_key = Some(key);
                loaded_levels += 1;
                continue;
            }

            let Some(key) = active_key else {
                continue;
            };
            let cost = next_whitespace_token(source, &mut cursor).ok_or(
                SkillFactoryLoadError::MissingUsageValue {
                    name: token.to_vec(),
                },
            )?;
            let usage = self.string_to_usage(Some(token));
            let cost = parse_legacy_u32(cost, "cost")?;
            if usage != UNKNOWN_SKILL_USAGE {
                self.skills
                    .get_mut(&key)
                    .and_then(Option::as_mut)
                    .expect("активный skill slot публикуется перед его usage")
                    .add_usage(SkillUsage { usage, cost });
            }
        }

        Ok(SkillConfigurationLoadReport {
            skill_id,
            loaded_levels,
        })
    }

    /// Дописывает exact `count + ordered (length, record)` wire.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), SkillFactorySerializeError> {
        let count = i32::try_from(self.skills.len()).map_err(|_| {
            SkillFactorySerializeError::EntryCount {
                count: self.skills.len(),
            }
        })?;
        destination.extend_from_slice(&count.to_le_bytes());

        for (&key, skill) in &self.skills {
            let record = match skill {
                None => None,
                Some(skill) => skill
                    .serialize()
                    .map_err(|source| SkillFactorySerializeError::Skill { key, source })?,
            };
            let Some(record) = record else {
                destination.extend_from_slice(&0_u32.to_le_bytes());
                continue;
            };
            let length = u32::try_from(record.len()).map_err(|_| {
                SkillFactorySerializeError::RecordLength {
                    key,
                    length: record.len(),
                }
            })?;
            destination.extend_from_slice(&length.to_le_bytes());
            destination.extend_from_slice(&record);
        }
        Ok(())
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

/// Наблюдаемый итог одного `.skill` content-loader-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillConfigurationLoadReport {
    pub(crate) skill_id: u32,
    pub(crate) loaded_levels: usize,
}

/// Typed safe boundary для некорректного legacy token stream.
#[derive(Debug)]
pub(crate) enum SkillFactoryLoadError {
    MissingMarker { marker: &'static str },
    MissingMarkerValue { marker: &'static str },
    MissingUsageValue { name: Vec<u8> },
    InvalidNumber { field: &'static str, value: Vec<u8> },
    SkillText(SkillTextError),
}

impl fmt::Display for SkillFactoryLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMarker { marker } => write!(formatter, "skill source: не найден marker {marker}"),
            Self::MissingMarkerValue { marker } => {
                write!(formatter, "skill source: marker {marker} не имеет значения")
            }
            Self::MissingUsageValue { name } => write!(
                formatter,
                "skill source: usage {:?} не имеет cost-значения",
                String::from_utf8_lossy(name)
            ),
            Self::InvalidNumber { field, value } => write!(
                formatter,
                "skill source: {field} содержит недопустимое число {:?}",
                String::from_utf8_lossy(value)
            ),
            Self::SkillText(source) => source.fmt(formatter),
        }
    }
}

impl Error for SkillFactoryLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SkillText(source) => Some(source),
            Self::MissingMarker { .. }
            | Self::MissingMarkerValue { .. }
            | Self::MissingUsageValue { .. }
            | Self::InvalidNumber { .. } => None,
        }
    }
}

fn next_whitespace_token<'a>(source: &'a [u8], cursor: &mut usize) -> Option<&'a [u8]> {
    *cursor += source[*cursor..]
        .iter()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count();
    let start = *cursor;
    *cursor += source[*cursor..]
        .iter()
        .take_while(|byte| !byte.is_ascii_whitespace())
        .count();
    (start != *cursor).then_some(&source[start..*cursor])
}

fn seek_marker(
    source: &[u8],
    cursor: &mut usize,
    marker: &[u8],
    marker_name: &'static str,
) -> Result<(), SkillFactoryLoadError> {
    while let Some(token) = next_whitespace_token(source, cursor) {
        if token == marker {
            return Ok(());
        }
    }
    Err(SkillFactoryLoadError::MissingMarker {
        marker: marker_name,
    })
}

fn read_named_u32(
    source: &[u8],
    cursor: &mut usize,
    marker: &[u8],
    marker_name: &'static str,
) -> Result<u32, SkillFactoryLoadError> {
    seek_marker(source, cursor, marker, marker_name)?;
    let value = next_whitespace_token(source, cursor)
        .ok_or(SkillFactoryLoadError::MissingMarkerValue { marker: marker_name })?;
    parse_legacy_u32(value, marker_name)
}

fn parse_legacy_u32(value: &[u8], field: &'static str) -> Result<u32, SkillFactoryLoadError> {
    let (negative, digits) = match value.first() {
        Some(b'+') => (false, &value[1..]),
        Some(b'-') => (true, &value[1..]),
        _ => (false, value),
    };
    let magnitude = parse_decimal_magnitude(digits, field, value)?;
    if magnitude > u64::from(u32::MAX) {
        return Err(SkillFactoryLoadError::InvalidNumber {
            field,
            value: value.to_vec(),
        });
    }
    let value = magnitude as u32;
    Ok(if negative { value.wrapping_neg() } else { value })
}

fn parse_legacy_i32(value: &[u8], field: &'static str) -> Result<i32, SkillFactoryLoadError> {
    let (negative, digits) = match value.first() {
        Some(b'+') => (false, &value[1..]),
        Some(b'-') => (true, &value[1..]),
        _ => (false, value),
    };
    let magnitude = parse_decimal_magnitude(digits, field, value)?;
    let limit = if negative {
        i64::from(i32::MAX) + 1
    } else {
        i64::from(i32::MAX)
    } as u64;
    if magnitude > limit {
        return Err(SkillFactoryLoadError::InvalidNumber {
            field,
            value: value.to_vec(),
        });
    }
    if negative && magnitude == (i64::from(i32::MAX) + 1) as u64 {
        Ok(i32::MIN)
    } else if negative {
        Ok(-(magnitude as i32))
    } else {
        Ok(magnitude as i32)
    }
}

fn parse_decimal_magnitude(
    digits: &[u8],
    field: &'static str,
    original: &[u8],
) -> Result<u64, SkillFactoryLoadError> {
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Err(SkillFactoryLoadError::InvalidNumber {
            field,
            value: original.to_vec(),
        });
    }
    digits.iter().try_fold(0_u64, |value, digit| {
        value
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(*digit - b'0')))
            .ok_or_else(|| SkillFactoryLoadError::InvalidNumber {
                field,
                value: original.to_vec(),
            })
    })
}

/// Safe serialization boundary для невозможного legacy registry-state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SkillFactorySerializeError {
    EntryCount {
        count: usize,
    },
    Skill {
        key: u32,
        source: SkillSerializeError,
    },
    RecordLength {
        key: u32,
        length: usize,
    },
}

impl fmt::Display for SkillFactorySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCount { count } => write!(
                formatter,
                "skill cache содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::Skill { key, source } => {
                write!(formatter, "skill slot 0x{key:08X}: {source}")
            }
            Self::RecordLength { key, length } => write!(
                formatter,
                "skill slot 0x{key:08X} имеет record длиной {length} байт вне u32"
            ),
        }
    }
}

impl Error for SkillFactorySerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Skill { source, .. } => Some(source),
            Self::EntryCount { .. } | Self::RecordLength { .. } => None,
        }
    }
}

// Сырой C++ ниже остаётся документацией ещё не восстановленных loaders и
// usage-name cache, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp

// ============================================================================
// FUNCTION: CSkillFactory::ClearSkillCache
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:144
// RVA: 0x00060DC0
// ADDRESS: 00460dc0
// PROTOTYPE: void __cdecl ClearSkillCache(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:291
// RVA: 0x00060E90
// ADDRESS: 00460e90
// PROTOTYPE: int __cdecl Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::StringToUsage
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:403
// RVA: 0x00060F80
// ADDRESS: 00460f80
// PROTOTYPE: tagSkillUsage __cdecl StringToUsage(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::ClearUsageCache
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:398
// RVA: 0x00061E00
// ADDRESS: 00461e00
// PROTOTYPE: void __cdecl ClearUsageCache(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::LoadConfigration
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:207
// RVA: 0x00062050
// ADDRESS: 00462050
// PROTOTYPE: long __cdecl LoadConfigration(char * param_1)
//
// Реализовано выше как `load_configuration`: resource-open передаёт bytes
// явно, а content-order и publication skill cache сохранены.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::LoadSkillCache
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:154
// RVA: 0x00062510
// ADDRESS: 00462510
// PROTOTYPE: int __cdecl LoadSkillCache(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::LoadUsage
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:364
// RVA: 0x000628E0
// ADDRESS: 004628e0
// PROTOTYPE: long __cdecl LoadUsage(char * param_1)
//
// Реализовано выше как `load_usage`: внешний owner открывает resource, затем
// передаёт его byte stream без CRT/MessageBox plumbing.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkillFactory::LoadUsageCache
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:326
// RVA: 0x00062A70
// ADDRESS: 00462a70
// PROTOTYPE: int __cdecl LoadUsageCache(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::tagOrginEquip::~tagOrginEquip
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008D500
// ADDRESS: 0048d500
// PROTOTYPE: void __thiscall ~tagOrginEquip(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSynthesis::tagSynthesis::~tagSynthesis
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008D700
// ADDRESS: 0048d700
// PROTOTYPE: void __thiscall ~tagSynthesis(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048dcd1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008DCD1
// ADDRESS: 0048dcd1
// PROTOTYPE: undefined Catch@0048dcd1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048de56
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008DE56
// ADDRESS: 0048de56
// PROTOTYPE: undefined Catch@0048de56()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048dee2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008DEE2
// ADDRESS: 0048dee2
// PROTOTYPE: undefined Catch@0048dee2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048e582
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008E582
// ADDRESS: 0048e582
// PROTOTYPE: undefined Catch@0048e582()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048e7d3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008E7D3
// ADDRESS: 0048e7d3
// PROTOTYPE: undefined Catch@0048e7d3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048ec02
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008EC02
// ADDRESS: 0048ec02
// PROTOTYPE: undefined Catch@0048ec02()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0048ecbc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x0008ECBC
// ADDRESS: 0048ecbc
// PROTOTYPE: undefined Catch@0048ecbc()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005317c0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x001317C0
// ADDRESS: 005317c0
// PROTOTYPE: undefined Unwind@005317c0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00531800
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x00131800
// ADDRESS: 00531800
// PROTOTYPE: undefined Unwind@00531800()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00531820
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp
// RVA: 0x00131820
// ADDRESS: 00531820
// PROTOTYPE: undefined Unwind@00531820()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
