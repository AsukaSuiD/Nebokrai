//! Списки монстров для новых навыков исторического Miracle.
//!
//! Статус World `CNewSkillMonserConf::AddToByteArray` RVA `0x0003CB00`:
//! `IMPLEMENTED`; XML loader, singleton и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:144`.
//!
//! Wire: signed group count, затем ordered `u32 skill_id + signed name_count`
//! и vector локализованных monster-name C-строк. XML loader назначал vector
//! через `map::operator[]`, поэтому повторный skill ID заменял предыдущую
//! группу целиком; порядок и повторы имён внутри vector значимы. Историческая
//! опечатка `Monser` сохраняется только в имени исходного owner-а.
//! `BTreeMap<u32, Vec<Vec<u8>>>` и owned bytes заменяют MSVC containers/string
//! lifetime без изменения unsigned key-order и wire.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Safe owner исходного `m_mSkillMonsterList`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NewSkillMonsterConf {
    groups: BTreeMap<u32, Vec<Vec<u8>>>,
}

impl NewSkillMonsterConf {
    /// Сохраняет last-wins assignment loader-а для duplicate skill ID.
    pub(crate) fn insert_group(
        &mut self,
        skill_id: u32,
        names: Vec<Vec<u8>>,
    ) -> Option<Vec<Vec<u8>>> {
        self.groups.insert(skill_id, names)
    }

    pub(crate) fn clear(&mut self) {
        self.groups.clear();
    }

    /// Дописывает exact ordered map/vector/C-string wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), NewSkillMonsterSerializeError> {
        write_count(destination, self.groups.len(), None)?;
        for (&skill_id, names) in &self.groups {
            destination.extend_from_slice(&skill_id.to_le_bytes());
            write_count(destination, names.len(), Some(skill_id))?;
            for (name_index, name) in names.iter().enumerate() {
                if name.contains(&0) {
                    return Err(NewSkillMonsterSerializeError::NameContainsNul {
                        skill_id,
                        name_index,
                    });
                }
                destination.extend_from_slice(name);
                destination.push(0);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NewSkillMonsterSerializeError {
    CountOutOfRange {
        skill_id: Option<u32>,
        count: usize,
    },
    NameContainsNul {
        skill_id: u32,
        name_index: usize,
    },
}

impl fmt::Display for NewSkillMonsterSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange {
                skill_id: Some(skill_id),
                count,
            } => write!(
                formatter,
                "skill {skill_id} содержит {count} имён вне signed 32-битного диапазона"
            ),
            Self::CountOutOfRange {
                skill_id: None,
                count,
            } => write!(
                formatter,
                "NewSkillMonster содержит {count} групп вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul {
                skill_id,
                name_index,
            } => write!(
                formatter,
                "имя {name_index} в группе skill {skill_id} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for NewSkillMonsterSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    skill_id: Option<u32>,
) -> Result<(), NewSkillMonsterSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| NewSkillMonsterSerializeError::CountOutOfRange { skill_id, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация XML loader-а, singleton
// и Game decoder-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp

// ============================================================================
// FUNCTION: CNewSkillMonserConf::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.h:31
// RVA: 0x0009CCB0
// ADDRESS: 0049ccb0
// PROTOTYPE: CNewSkillMonserConf * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterList::tagDropGoods::tagDrop::~tagDrop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x000C6270
// ADDRESS: 004c6270
// PROTOTYPE: void __thiscall ~tagDrop(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::tagPropertiesUpgrade::tagPropertiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x000C62D0
// ADDRESS: 004c62d0
// PROTOTYPE: undefined __thiscall tagPropertiesUpgrade(tagPropertiesUpgrade * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::tagPropertiesUpgrade::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x000C6510
// ADDRESS: 004c6510
// PROTOTYPE: tagPropertiesUpgrade * __thiscall operator=(tagPropertiesUpgrade * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004c6df6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x000C6DF6
// ADDRESS: 004c6df6
// PROTOTYPE: undefined Catch@004c6df6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:170
// RVA: 0x001C46B0
// ADDRESS: 005c46b0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::~CNewSkillMonserConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:25
// RVA: 0x001C48F0
// ADDRESS: 005c48f0
// PROTOTYPE: void __thiscall ~CNewSkillMonserConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::CNewSkillMonserConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:20
// RVA: 0x001C4980
// ADDRESS: 005c4980
// PROTOTYPE: undefined __thiscall CNewSkillMonserConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp

// ============================================================================
// FUNCTION: CNewSkillMonserConf::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.h:31
// RVA: 0x00001250
// ADDRESS: 00401250
// PROTOTYPE: CNewSkillMonserConf * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGame::tagSysBroadcast::~tagSysBroadcast
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0002C470
// ADDRESS: 0042c470
// PROTOTYPE: void __thiscall ~tagSysBroadcast(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::tagPropertiesUpgrade::tagPropertiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0002C850
// ADDRESS: 0042c850
// PROTOTYPE: undefined __thiscall tagPropertiesUpgrade(tagPropertiesUpgrade * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::tagPropertiesUpgrade::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0002CA90
// ADDRESS: 0042ca90
// PROTOTYPE: tagPropertiesUpgrade * __thiscall operator=(tagPropertiesUpgrade * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0042d66c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0002D66C
// ADDRESS: 0042d66c
// PROTOTYPE: undefined Catch@0042d66c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0042d716
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0002D716
// ADDRESS: 0042d716
// PROTOTYPE: undefined Catch@0042d716()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:144
// RVA: 0x0003CB00
// ADDRESS: 0043cb00
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::LoadNewSkillMonserConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:41
// RVA: 0x0003E090
// ADDRESS: 0043e090
// PROTOTYPE: bool __thiscall LoadNewSkillMonserConf(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::~CNewSkillMonserConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:25
// RVA: 0x0003E520
// ADDRESS: 0043e520
// PROTOTYPE: void __thiscall ~CNewSkillMonserConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNewSkillMonserConf::CNewSkillMonserConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp:20
// RVA: 0x0003E5B0
// ADDRESS: 0043e5b0
// PROTOTYPE: undefined __thiscall CNewSkillMonserConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052d280
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0012D280
// ADDRESS: 0052d280
// PROTOTYPE: undefined Unwind@0052d280()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052d2a0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\newskillmonsterlist.cpp
// RVA: 0x0012D2A0
// ADDRESS: 0052d2a0
// PROTOTYPE: undefined Unwind@0052d2a0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//









// COMPONENT_VARIANT_END: WorldServer
