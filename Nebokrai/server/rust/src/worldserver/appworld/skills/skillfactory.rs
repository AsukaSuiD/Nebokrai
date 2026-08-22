//! Ordered cache и initial-config serializer навыков WorldServer.
//!
//! Статус `CSkillFactory::Serialize` RVA `0x00060E90` и безопасной замены
//! `ClearSkillCache` RVA `0x00060DC0`, `StringToUsage` `0x00060F80` и
//! `ClearUsageCache` `0x00061E00`: `IMPLEMENTED`; файловые загрузчики ниже
//! пока остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
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
//! `LoadUsage`, а `StringToUsage` возвращает нулевой `SKILL_USAGE_UNKNOW`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::skill::{CSkill, SkillSerializeError};

/// Exact `SKILL_USAGE_UNKNOW`: в перечислении ни один штатный usage не равен
/// нулю, и exact `StringToUsage` возвращает этот sentinel для отсутствия.
pub(crate) const UNKNOWN_SKILL_USAGE: u32 = 0;

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

    /// Точная запись строки `.usage`: `std::map::operator[]` заменял прежнее
    /// значение при том же byte-sensitive имени.
    pub(crate) fn set_usage_name(&mut self, name: &[u8], usage: u32) -> Option<u32> {
        self.usage_names
            .insert(visible_c_string(name).to_vec(), usage)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:207
// RVA: 0x00062050
// ADDRESS: 00462050
// PROTOTYPE: long __cdecl LoadConfigration(char * param_1)
//
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\skills\skillfactory.cpp:364
// RVA: 0x000628E0
// ADDRESS: 004628e0
// PROTOTYPE: long __cdecl LoadUsage(char * param_1)
//
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
