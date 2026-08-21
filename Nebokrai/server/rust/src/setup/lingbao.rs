//! Конфигурация LingBao исторического Miracle.
//!
//! Статус World `CLingBaoSetup::AddByteLingBao` RVA `0x0007C7C0`:
//! `IMPLEMENTED`; loader, queries и Game decoder ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp:316` и соседний
//! `lingbao.h`.
//!
//! Wire: signed total multimap count, затем ordered C-string key, `u32 ticket`,
//! три signed vector count-а и подряд записи `8×u32`, `5×u32`, `3×u32`.
//! `BTreeMap<Vec<u8>, Vec<_>>` заменяет `std::multimap<std::string, _>`:
//! сохраняет byte-order ключей и insertion-order эквивалентных ключей. Rust
//! пишет поля little-endian вместо raw ABI-copy, сохраняя размеры 32/20/12
//! bytes без padding. Внутренний NUL и невозможные signed counts блокируют весь
//! append до изменения destination. Точная token-parser семантика loader-а
//! остаётся отдельным проходом.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoFirstNode {
    pub(crate) node_type: u32,
    pub(crate) use_ticket: u32,
    pub(crate) probability_1: u32,
    pub(crate) add_value_1: u32,
    pub(crate) probability_2: u32,
    pub(crate) add_value_2: u32,
    pub(crate) probability_3: u32,
    pub(crate) add_value_3: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoSecondNode {
    pub(crate) node_type: u32,
    pub(crate) probability: u32,
    pub(crate) use_ticket: u32,
    pub(crate) add_value_min: u32,
    pub(crate) add_value_max: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoThirdNode {
    pub(crate) node_type: u32,
    pub(crate) use_ticket: u32,
    pub(crate) use_ticket_max: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LingBaoNodeInfo {
    pub(crate) ticket: u32,
    pub(crate) first: Vec<LingBaoFirstNode>,
    pub(crate) second: Vec<LingBaoSecondNode>,
    pub(crate) third: Vec<LingBaoThirdNode>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CLingBaoSetup {
    entries: BTreeMap<Vec<u8>, Vec<LingBaoNodeInfo>>,
}

impl CLingBaoSetup {
    pub(crate) fn insert(&mut self, name: Vec<u8>, info: LingBaoNodeInfo) {
        self.entries.entry(name).or_default().push(info);
    }

    pub(crate) fn entries(&self) -> &BTreeMap<Vec<u8>, Vec<LingBaoNodeInfo>> {
        &self.entries
    }

    pub(crate) fn add_byte_ling_bao(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), LingBaoSerializationBlock> {
        let total_count = self.entries.values().try_fold(0usize, |total, group| {
            total.checked_add(group.len())
        });
        let total_count = total_count.ok_or(LingBaoSerializationBlock::EntryCountOutOfRange {
            count: usize::MAX,
        })?;
        let count = i32::try_from(total_count).map_err(|_| {
            LingBaoSerializationBlock::EntryCountOutOfRange { count: total_count }
        })?;

        let mut payload = Vec::new();
        payload.extend_from_slice(&count.to_le_bytes());
        let mut entry_index = 0usize;
        for (name, group) in &self.entries {
            for info in group {
                if name.contains(&0) {
                    return Err(LingBaoSerializationBlock::NameContainsNul { entry_index });
                }
                payload.extend_from_slice(name);
                payload.push(0);
                payload.extend_from_slice(&info.ticket.to_le_bytes());
                write_ling_bao_count(
                    &mut payload,
                    entry_index,
                    LingBaoNodeSection::First,
                    info.first.len(),
                )?;
                write_ling_bao_count(
                    &mut payload,
                    entry_index,
                    LingBaoNodeSection::Second,
                    info.second.len(),
                )?;
                write_ling_bao_count(
                    &mut payload,
                    entry_index,
                    LingBaoNodeSection::Third,
                    info.third.len(),
                )?;
                for node in &info.first {
                    write_ling_bao_fields(
                        &mut payload,
                        &[
                            node.node_type,
                            node.use_ticket,
                            node.probability_1,
                            node.add_value_1,
                            node.probability_2,
                            node.add_value_2,
                            node.probability_3,
                            node.add_value_3,
                        ],
                    );
                }
                for node in &info.second {
                    write_ling_bao_fields(
                        &mut payload,
                        &[
                            node.node_type,
                            node.probability,
                            node.use_ticket,
                            node.add_value_min,
                            node.add_value_max,
                        ],
                    );
                }
                for node in &info.third {
                    write_ling_bao_fields(
                        &mut payload,
                        &[node.node_type, node.use_ticket, node.use_ticket_max],
                    );
                }
                entry_index += 1;
            }
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LingBaoNodeSection {
    First,
    Second,
    Third,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LingBaoSerializationBlock {
    EntryCountOutOfRange {
        count: usize,
    },
    NodeCountOutOfRange {
        entry_index: usize,
        section: LingBaoNodeSection,
        count: usize,
    },
    NameContainsNul {
        entry_index: usize,
    },
}

impl fmt::Display for LingBaoSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntryCountOutOfRange { count } => write!(
                formatter,
                "CLingBaoSetup содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::NodeCountOutOfRange {
                entry_index,
                section,
                count,
            } => write!(
                formatter,
                "CLingBaoSetup #{entry_index} {section:?} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul { entry_index } => {
                write!(formatter, "CLingBaoSetup #{entry_index}: имя содержит внутренний NUL")
            }
        }
    }
}

impl Error for LingBaoSerializationBlock {}

fn write_ling_bao_count(
    destination: &mut Vec<u8>,
    entry_index: usize,
    section: LingBaoNodeSection,
    count: usize,
) -> Result<(), LingBaoSerializationBlock> {
    let count = i32::try_from(count).map_err(|_| {
        LingBaoSerializationBlock::NodeCountOutOfRange {
            entry_index,
            section,
            count,
        }
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_ling_bao_fields(destination: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\lingbao.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp

// ============================================================================
// FUNCTION: CLingBaoSetup::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\lingbao.h:78
// RVA: 0x0009D280
// ADDRESS: 0049d280
// PROTOTYPE: CLingBaoSetup * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::GetAddResult
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp:112
// RVA: 0x001C9370
// ADDRESS: 005c9370
// PROTOTYPE: void __thiscall GetAddResult(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1, ulong param_2, map<unsigned_long,unsigned_long,std::less<unsigned_long>,std::allocator<std::pair<unsigned_long_const_,unsigned_long>_>_> * param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::DecodeFromArrayLingBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp:352
// RVA: 0x001CA240
// ADDRESS: 005ca240
// PROTOTYPE: void __thiscall DecodeFromArrayLingBao(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\lingbao.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp

// ============================================================================
// FUNCTION: CLingBaoSetup::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\lingbao.h:78
// RVA: 0x000156C0
// ADDRESS: 004156c0
// PROTOTYPE: CLingBaoSetup * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::AddByteLingBao
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp:316
// RVA: 0x0007C7C0
// ADDRESS: 0047c7c0
// PROTOTYPE: void __thiscall AddByteLingBao(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLingBaoSetup::LoadLingBaoSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\lingbao.cpp:9
// RVA: 0x0007DB70
// ADDRESS: 0047db70
// PROTOTYPE: void __thiscall LoadLingBaoSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
