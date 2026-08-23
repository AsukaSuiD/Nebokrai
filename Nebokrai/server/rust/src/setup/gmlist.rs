//! Списки операторов и игровых персонажей с операторскими правами.
//!
//! Статус World `CGMList::AddToByteArray` RVA `0x000987D0`: `IMPLEMENTED`;
//! loaders, accessors и Game decoder ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:137`.
//!
//! Exact World serializer и Game decoder подтверждают wire: signed count и
//! ordered records `C-string name + i32 level` сначала для общего GM map,
//! затем для player GM map, после них — C-string god passport. Ключ карты
//! отдельно не передаётся. `BTreeMap<Vec<u8>, _>` заменяет
//! `std::map<std::string, _>` и сохраняет его лексикографический byte-order;
//! owned bytes заменяют C++ string lifetime. Уровень намеренно остаётся
//! полным `i32`: известные enum-значения не дают права отвергать иное значение
//! из данных. Точный EXE также подтверждает исходный god passport
//! `@^$^#SDFSDslfld/$dsl2a`; чтение `gmlist.ini` и `data/temp.ini` остаётся
//! отдельным loader-проходом. Невозможный signed count и внутренний NUL
//! блокируют весь append до изменения destination.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const DEFAULT_GOD_PASSPORT: &[u8] = b"@^$^#SDFSDslfld/$dsl2a";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GmInfo {
    pub(crate) name: Vec<u8>,
    pub(crate) level: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGMList {
    gm_info: BTreeMap<Vec<u8>, GmInfo>,
    player_gm_info: BTreeMap<Vec<u8>, GmInfo>,
    god_passport: Vec<u8>,
}

impl Default for CGMList {
    fn default() -> Self {
        Self {
            gm_info: BTreeMap::new(),
            player_gm_info: BTreeMap::new(),
            god_passport: DEFAULT_GOD_PASSPORT.to_vec(),
        }
    }
}

impl CGMList {
    pub(crate) fn insert_gm(&mut self, info: GmInfo) -> Option<GmInfo> {
        self.gm_info.insert(info.name.clone(), info)
    }

    pub(crate) fn insert_player_gm(&mut self, info: GmInfo) -> Option<GmInfo> {
        self.player_gm_info.insert(info.name.clone(), info)
    }

    pub(crate) fn gm_info(&self) -> &BTreeMap<Vec<u8>, GmInfo> {
        &self.gm_info
    }

    pub(crate) fn player_gm_info(&self) -> &BTreeMap<Vec<u8>, GmInfo> {
        &self.player_gm_info
    }

    pub(crate) fn god_passport(&self) -> &[u8] {
        &self.god_passport
    }

    pub(crate) fn set_god_passport(&mut self, god_passport: Vec<u8>) {
        self.god_passport = god_passport;
    }

    /// Загружает один из двух exact whitespace-списков World GM.
    /// Неизвестные role-имена, как и в EXE, не создают map-entry.
    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
        collection: GmListCollection,
        passport_source: Option<&[u8]>,
    ) -> Result<usize, GmListLoadError> {
        let destination = match collection {
            GmListCollection::Gm => &mut self.gm_info,
            GmListCollection::PlayerGm => &mut self.player_gm_info,
        };
        destination.clear();
        let tokens: Vec<&[u8]> = source
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .collect();
        if tokens.len() % 2 != 0 {
            return Err(GmListLoadError::MissingRole {
                name: tokens.last().copied().unwrap_or_default().to_vec(),
            });
        }
        for pair in tokens.chunks_exact(2) {
            let level = match (collection, pair[1]) {
                (GmListCollection::Gm, b"admin") => Some(100),
                (GmListCollection::Gm, b"arch") => Some(90),
                (_, b"wizard") => Some(50),
                (_, b"guardian") => Some(40),
                (_, b"moderator") => Some(30),
                _ => None,
            };
            if let Some(level) = level {
                let info = GmInfo {
                    name: pair[0].to_vec(),
                    level,
                };
                destination.insert(info.name.clone(), info);
            }
        }
        if let Some(passport) = passport_source.and_then(|bytes| {
            bytes
                .split(|byte| byte.is_ascii_whitespace())
                .find(|token| !token.is_empty())
        }) {
            self.god_passport = passport.to_vec();
        }
        Ok(destination.len())
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GmListSerializationBlock> {
        let mut payload = Vec::new();
        write_gm_map(&mut payload, GmListCollection::Gm, &self.gm_info)?;
        write_gm_map(
            &mut payload,
            GmListCollection::PlayerGm,
            &self.player_gm_info,
        )?;
        write_gm_string(
            &mut payload,
            None,
            GmListStringField::GodPassport,
            &self.god_passport,
        )?;
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmListCollection {
    Gm,
    PlayerGm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmListStringField {
    Name,
    GodPassport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmListSerializationBlock {
    CountOutOfRange {
        collection: GmListCollection,
        count: usize,
    },
    StringContainsNul {
        collection: Option<GmListCollection>,
        entry_index: Option<usize>,
        field: GmListStringField,
    },
}

impl fmt::Display for GmListSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { collection, count } => write!(
                formatter,
                "CGMList {collection:?} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                collection,
                entry_index,
                field,
            } => write!(
                formatter,
                "CGMList {collection:?} запись {entry_index:?}: поле {field:?} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for GmListSerializationBlock {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmListLoadError {
    MissingRole { name: Vec<u8> },
}

impl fmt::Display for GmListLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRole { name } => write!(
                formatter,
                "CGMList: для записи {:?} отсутствует role",
                String::from_utf8_lossy(name)
            ),
        }
    }
}

impl Error for GmListLoadError {}

fn write_gm_map(
    destination: &mut Vec<u8>,
    collection: GmListCollection,
    entries: &BTreeMap<Vec<u8>, GmInfo>,
) -> Result<(), GmListSerializationBlock> {
    let count = i32::try_from(entries.len()).map_err(|_| {
        GmListSerializationBlock::CountOutOfRange {
            collection,
            count: entries.len(),
        }
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    for (entry_index, info) in entries.values().enumerate() {
        write_gm_string(
            destination,
            Some((collection, entry_index)),
            GmListStringField::Name,
            &info.name,
        )?;
        destination.extend_from_slice(&info.level.to_le_bytes());
    }
    Ok(())
}

fn write_gm_string(
    destination: &mut Vec<u8>,
    entry: Option<(GmListCollection, usize)>,
    field: GmListStringField,
    value: &[u8],
) -> Result<(), GmListSerializationBlock> {
    if value.contains(&0) {
        return Err(GmListSerializationBlock::StringContainsNul {
            collection: entry.map(|(collection, _)| collection),
            entry_index: entry.map(|(_, entry_index)| entry_index),
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp

// ============================================================================
// FUNCTION: CGMList::GetInfoByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:116
// RVA: 0x000DEC50
// ADDRESS: 004dec50
// PROTOTYPE: tagGMInfo * __cdecl GetInfoByName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGMList::GetPlayerInfoByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:127
// RVA: 0x000DED70
// ADDRESS: 004ded70
// PROTOTYPE: tagGMInfo * __cdecl GetPlayerInfoByName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGMList::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:161
// RVA: 0x000DEE90
// ADDRESS: 004dee90
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp

// ============================================================================
// FUNCTION: _FactionNpcName::~_FactionNpcName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007E940
// ADDRESS: 0047e940
// PROTOTYPE: void __thiscall ~_FactionNpcName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: _FactionNpcName::_FactionNpcName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007ED80
// ADDRESS: 0047ed80
// PROTOTYPE: undefined __thiscall _FactionNpcName(_FactionNpcName * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047f0db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007F0DB
// ADDRESS: 0047f0db
// PROTOTYPE: undefined Catch@0047f0db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047f456
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007F456
// ADDRESS: 0047f456
// PROTOTYPE: undefined Catch@0047f456()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047f82d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007F82D
// ADDRESS: 0047f82d
// PROTOTYPE: undefined Catch@0047f82d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047fc42
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007FC42
// ADDRESS: 0047fc42
// PROTOTYPE: undefined Catch@0047fc42()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0047ff13
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x0007FF13
// ADDRESS: 0047ff13
// PROTOTYPE: undefined Catch@0047ff13()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004803d8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x000803D8
// ADDRESS: 004803d8
// PROTOTYPE: undefined Catch@004803d8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00480cac
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x00080CAC
// ADDRESS: 00480cac
// PROTOTYPE: undefined Catch@00480cac()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00480d69
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x00080D69
// ADDRESS: 00480d69
// PROTOTYPE: undefined Catch@00480d69()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGMList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:137
// RVA: 0x000987D0
// ADDRESS: 004987d0
// PROTOTYPE: bool __cdecl AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGMList::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:28
// RVA: 0x00099090
// ADDRESS: 00499090
// PROTOTYPE: void __cdecl Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGMList::PlayerClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:32
// RVA: 0x000990D0
// ADDRESS: 004990d0
// PROTOTYPE: void __cdecl PlayerClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGMList::Load
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp:36
// RVA: 0x000995A0
// ADDRESS: 004995a0
// PROTOTYPE: bool __cdecl Load(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00530e60
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x00130E60
// ADDRESS: 00530e60
// PROTOTYPE: undefined Unwind@00530e60()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00530e80
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x00130E80
// ADDRESS: 00530e80
// PROTOTYPE: undefined Unwind@00530e80()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00530ef0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\gmlist.cpp
// RVA: 0x00130EF0
// ADDRESS: 00530ef0
// PROTOTYPE: undefined Unwind@00530ef0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: WorldServer
