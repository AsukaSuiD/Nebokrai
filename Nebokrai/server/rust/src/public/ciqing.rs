//! Конфигурация CiQing исторического Miracle.
//!
//! Статус World `CCiQingSetup::AddByteToArray` RVA `0x00086080`:
//! `IMPLEMENTED`; text loader, queries, RNG и Game runtime ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точные пары World/Game EXE+PDB подтверждают одинаковый
//! serializer/decoder; World EXE SHA-256
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB: `e:\svn\fengyun_russia_dev\public\ciqing.cpp:139` и
//! соседний `ciqing.h`.
//!
//! Wire содержит три insertion-order секции: make records по шесть `u32`,
//! compose records и improve records по три `u32`; все counts signed `i32`.
//! Compose точно передаёт `source_a` дважды, затем `source_b, money,
//! probability, crystal, result_count` и пары `probability/result`. Это не
//! исправлено как опечатка: exact World serializer и Game decoder совместно
//! подтверждают наблюдаемый positional quirk. Rust хранит named поля и `Vec`,
//! но пишет доказанный порядок явно little-endian. Declared result count
//! остаётся частью owner-state, тогда как wire, как оригинал, берёт реальный
//! размер result-vector. Невозможный signed count блокирует append до изменения
//! destination. Точный text-loader остаётся отдельным проходом.

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingMakeNode {
    pub(crate) destination_base_index: u32,
    pub(crate) equipment_position: u32,
    pub(crate) source_a_base_index: u32,
    pub(crate) source_a_count: u32,
    pub(crate) source_b_base_index: u32,
    pub(crate) source_b_count: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingComposeNode {
    pub(crate) source_a_base_index: u32,
    pub(crate) source_b_base_index: u32,
    pub(crate) compose_probability: u32,
    pub(crate) money: u32,
    pub(crate) crystal_count: u32,
    pub(crate) declared_result_count: u32,
    pub(crate) results: Vec<(u32, u32)>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CiQingImproveNode {
    pub(crate) level: u32,
    pub(crate) base_index: u32,
    pub(crate) probability: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CCiQingSetup {
    make: Vec<CiQingMakeNode>,
    compose: Vec<CiQingComposeNode>,
    improve: Vec<CiQingImproveNode>,
}

impl CCiQingSetup {
    pub(crate) fn push_make(&mut self, node: CiQingMakeNode) {
        self.make.push(node);
    }

    pub(crate) fn push_compose(&mut self, node: CiQingComposeNode) {
        self.compose.push(node);
    }

    pub(crate) fn push_improve(&mut self, node: CiQingImproveNode) {
        self.improve.push(node);
    }

    pub(crate) fn make(&self) -> &[CiQingMakeNode] {
        &self.make
    }

    pub(crate) fn compose(&self) -> &[CiQingComposeNode] {
        &self.compose
    }

    pub(crate) fn improve(&self) -> &[CiQingImproveNode] {
        &self.improve
    }

    pub(crate) fn add_byte_to_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), CiQingSerializationBlock> {
        let mut payload = Vec::new();
        write_ciqing_count(&mut payload, CiQingCountSection::Make, self.make.len())?;
        for node in &self.make {
            write_u32_fields(
                &mut payload,
                &[
                    node.destination_base_index,
                    node.equipment_position,
                    node.source_a_base_index,
                    node.source_a_count,
                    node.source_b_base_index,
                    node.source_b_count,
                ],
            );
        }

        write_ciqing_count(
            &mut payload,
            CiQingCountSection::Compose,
            self.compose.len(),
        )?;
        for (compose_index, node) in self.compose.iter().enumerate() {
            write_u32_fields(
                &mut payload,
                &[
                    node.source_a_base_index,
                    node.source_a_base_index,
                    node.source_b_base_index,
                    node.money,
                    node.compose_probability,
                    node.crystal_count,
                ],
            );
            write_ciqing_count(
                &mut payload,
                CiQingCountSection::ComposeResults { compose_index },
                node.results.len(),
            )?;
            for &(probability, result_base_index) in &node.results {
                write_u32_fields(&mut payload, &[probability, result_base_index]);
            }
        }

        write_ciqing_count(
            &mut payload,
            CiQingCountSection::Improve,
            self.improve.len(),
        )?;
        for node in &self.improve {
            write_u32_fields(
                &mut payload,
                &[node.level, node.base_index, node.probability],
            );
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CiQingCountSection {
    Make,
    Compose,
    ComposeResults { compose_index: usize },
    Improve,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CiQingSerializationBlock {
    pub(crate) section: CiQingCountSection,
    pub(crate) count: usize,
}

impl fmt::Display for CiQingSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CCiQingSetup {:?} содержит {} записей вне signed 32-битного диапазона",
            self.section, self.count
        )
    }
}

impl Error for CiQingSerializationBlock {}

fn write_ciqing_count(
    destination: &mut Vec<u8>,
    section: CiQingCountSection,
    count: usize,
) -> Result<(), CiQingSerializationBlock> {
    let count = i32::try_from(count).map_err(|_| CiQingSerializationBlock { section, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_u32_fields(destination: &mut Vec<u8>, values: &[u32]) {
    for value in values {
        destination.extend_from_slice(&value.to_le_bytes());
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\ciqing.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\ciqing.cpp

// ============================================================================
// FUNCTION: CCiQingSetup::stComposeNode::stComposeNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.h:54
// RVA: 0x0003BF30
// ADDRESS: 0043bf30
// PROTOTYPE: undefined __thiscall stComposeNode(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::AddByteToArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:139
// RVA: 0x000E4740
// ADDRESS: 004e4740
// PROTOTYPE: void __thiscall AddByteToArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::GetNeedNumberById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:358
// RVA: 0x000E4900
// ADDRESS: 004e4900
// PROTOTYPE: bool __thiscall GetNeedNumberById(ulong param_1, stMakeNode * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::GetImproveNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:397
// RVA: 0x000E4950
// ADDRESS: 004e4950
// PROTOTYPE: bool __thiscall GetImproveNode(ulong param_1, stImproveGaiLv * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::RandChoise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:417
// RVA: 0x000E4990
// ADDRESS: 004e4990
// PROTOTYPE: ulong __thiscall RandChoise(stComposeNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::GetComputeNode
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:374
// RVA: 0x000E56A0
// ADDRESS: 004e56a0
// PROTOTYPE: bool __thiscall GetComputeNode(ulong param_1, ulong param_2, stComposeNode * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::CCiQingSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:20
// RVA: 0x000E5FC0
// ADDRESS: 004e5fc0
// PROTOTYPE: undefined __thiscall CCiQingSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::GetSingleInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:436
// RVA: 0x000E6010
// ADDRESS: 004e6010
// PROTOTYPE: CCiQingSetup * __cdecl GetSingleInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::DeByteFromArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:198
// RVA: 0x000E6110
// ADDRESS: 004e6110
// PROTOTYPE: void __thiscall DeByteFromArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\ciqing.cpp

// ============================================================================
// FUNCTION: CCiQingSetup::AddByteToArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:139
// RVA: 0x00086080
// ADDRESS: 00486080
// PROTOTYPE: void __thiscall AddByteToArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::CCiQingSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:20
// RVA: 0x00087650
// ADDRESS: 00487650
// PROTOTYPE: undefined __thiscall CCiQingSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:191
// RVA: 0x000876A0
// ADDRESS: 004876a0
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::GetSingleInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:436
// RVA: 0x000876F0
// ADDRESS: 004876f0
// PROTOTYPE: CCiQingSetup * __cdecl GetSingleInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCiQingSetup::ReadSetupFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp:34
// RVA: 0x000877F0
// ADDRESS: 004877f0
// PROTOTYPE: bool __thiscall ReadSetupFile(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049318d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp
// RVA: 0x0009318D
// ADDRESS: 0049318d
// PROTOTYPE: undefined Catch@0049318d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049327d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp
// RVA: 0x0009327D
// ADDRESS: 0049327d
// PROTOTYPE: undefined Catch@0049327d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0049357f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp
// RVA: 0x0009357F
// ADDRESS: 0049357f
// PROTOTYPE: undefined Catch@0049357f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00493632
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp
// RVA: 0x00093632
// ADDRESS: 00493632
// PROTOTYPE: undefined Catch@00493632()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00531a20
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\ciqing.cpp
// RVA: 0x00131A20
// ADDRESS: 00531a20
// PROTOTYPE: undefined Unwind@00531a20()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: WorldServer
