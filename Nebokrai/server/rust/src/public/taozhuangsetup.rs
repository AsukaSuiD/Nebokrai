//! Конфигурация комплектов TaoZhuang исторического Miracle.
//!
//! Статус World `CTaoZhuangSetup::AddByteToArray` RVA `0x00088340`:
//! `IMPLEMENTED`; text loader, gameplay queries и Game runtime ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точные World/Game serializer и decoder имеют одинаковый
//! контракт; World EXE SHA-256
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:170` и соседний
//! header.
//!
//! Wire сначала содержит ordered skill set, затем ordered item map. Item:
//! четыре `u32`, три C-string, фактический equipment-name count и ordered
//! имена, фактический add-item count и nested ordered property/skill maps.
//! Поля `item_count/equipment_count/add_property_count/add_skill_count`
//! передаются как сохранённые declared `u32`, а не пересчитываются; это
//! отличается от следующих за ними фактических container count-ов и сохранено
//! явно. `BTreeSet/BTreeMap` заменяют STL и сохраняют unsigned/byte order,
//! owned bytes — string lifetime. Внутренний NUL и невозможные signed counts
//! блокируют весь append до изменения destination. Точный text-loader остаётся
//! отдельным проходом.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TaoZhuangAddItem {
    pub(crate) number: u32,
    pub(crate) declared_property_count: u32,
    pub(crate) properties: BTreeMap<u32, u32>,
    pub(crate) declared_skill_count: u32,
    pub(crate) skills: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TaoZhuangItem {
    pub(crate) id: u32,
    pub(crate) color: u32,
    pub(crate) declared_item_count: u32,
    pub(crate) declared_equipment_count: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) description: Vec<u8>,
    pub(crate) script: Vec<u8>,
    pub(crate) equipment_names: BTreeSet<Vec<u8>>,
    additions: BTreeMap<u32, TaoZhuangAddItem>,
}

impl TaoZhuangItem {
    pub(crate) fn insert_addition(
        &mut self,
        addition: TaoZhuangAddItem,
    ) -> Option<TaoZhuangAddItem> {
        self.additions.insert(addition.number, addition)
    }

    pub(crate) fn additions(&self) -> &BTreeMap<u32, TaoZhuangAddItem> {
        &self.additions
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CTaoZhuangSetup {
    skill_ids: BTreeSet<u32>,
    items: BTreeMap<u32, TaoZhuangItem>,
}

impl CTaoZhuangSetup {
    pub(crate) fn insert_skill(&mut self, skill_id: u32) -> bool {
        self.skill_ids.insert(skill_id)
    }

    pub(crate) fn insert_item(&mut self, item: TaoZhuangItem) -> Option<TaoZhuangItem> {
        self.items.insert(item.id, item)
    }

    pub(crate) fn skill_ids(&self) -> &BTreeSet<u32> {
        &self.skill_ids
    }

    pub(crate) fn items(&self) -> &BTreeMap<u32, TaoZhuangItem> {
        &self.items
    }

    pub(crate) fn add_byte_to_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), TaoZhuangSerializationBlock> {
        let mut payload = Vec::new();
        write_tao_zhuang_count(
            &mut payload,
            TaoZhuangCountSection::SkillIds,
            self.skill_ids.len(),
        )?;
        for skill_id in &self.skill_ids {
            payload.extend_from_slice(&skill_id.to_le_bytes());
        }

        write_tao_zhuang_count(
            &mut payload,
            TaoZhuangCountSection::Items,
            self.items.len(),
        )?;
        for item in self.items.values() {
            for value in [
                item.id,
                item.color,
                item.declared_item_count,
                item.declared_equipment_count,
            ] {
                payload.extend_from_slice(&value.to_le_bytes());
            }
            write_tao_zhuang_string(
                &mut payload,
                item.id,
                None,
                TaoZhuangStringField::Name,
                &item.name,
            )?;
            write_tao_zhuang_string(
                &mut payload,
                item.id,
                None,
                TaoZhuangStringField::Description,
                &item.description,
            )?;
            write_tao_zhuang_string(
                &mut payload,
                item.id,
                None,
                TaoZhuangStringField::Script,
                &item.script,
            )?;

            write_tao_zhuang_count(
                &mut payload,
                TaoZhuangCountSection::EquipmentNames { item_id: item.id },
                item.equipment_names.len(),
            )?;
            for (equipment_index, name) in item.equipment_names.iter().enumerate() {
                write_tao_zhuang_string(
                    &mut payload,
                    item.id,
                    Some(equipment_index),
                    TaoZhuangStringField::EquipmentName,
                    name,
                )?;
            }

            write_tao_zhuang_count(
                &mut payload,
                TaoZhuangCountSection::Additions { item_id: item.id },
                item.additions.len(),
            )?;
            for addition in item.additions.values() {
                payload.extend_from_slice(&addition.number.to_le_bytes());
                payload.extend_from_slice(&addition.declared_property_count.to_le_bytes());
                for (&property_id, &value) in &addition.properties {
                    payload.extend_from_slice(&property_id.to_le_bytes());
                    payload.extend_from_slice(&value.to_le_bytes());
                }
                payload.extend_from_slice(&addition.declared_skill_count.to_le_bytes());
                for (&skill_id, &value) in &addition.skills {
                    payload.extend_from_slice(&skill_id.to_le_bytes());
                    payload.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TaoZhuangCountSection {
    SkillIds,
    Items,
    EquipmentNames { item_id: u32 },
    Additions { item_id: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TaoZhuangStringField {
    Name,
    Description,
    Script,
    EquipmentName,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TaoZhuangSerializationBlock {
    CountOutOfRange {
        section: TaoZhuangCountSection,
        count: usize,
    },
    StringContainsNul {
        item_id: u32,
        equipment_index: Option<usize>,
        field: TaoZhuangStringField,
    },
}

impl fmt::Display for TaoZhuangSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { section, count } => write!(
                formatter,
                "CTaoZhuangSetup {section:?} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                item_id,
                equipment_index,
                field,
            } => write!(
                formatter,
                "CTaoZhuangSetup item {item_id}, equipment {equipment_index:?}: поле {field:?} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for TaoZhuangSerializationBlock {}

fn write_tao_zhuang_count(
    destination: &mut Vec<u8>,
    section: TaoZhuangCountSection,
    count: usize,
) -> Result<(), TaoZhuangSerializationBlock> {
    let count = i32::try_from(count)
        .map_err(|_| TaoZhuangSerializationBlock::CountOutOfRange { section, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_tao_zhuang_string(
    destination: &mut Vec<u8>,
    item_id: u32,
    equipment_index: Option<usize>,
    field: TaoZhuangStringField,
    value: &[u8],
) -> Result<(), TaoZhuangSerializationBlock> {
    if value.contains(&0) {
        return Err(TaoZhuangSerializationBlock::StringContainsNul {
            item_id,
            equipment_index,
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp

// ============================================================================
// FUNCTION: Catch@0041d2ec
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp
// RVA: 0x0001D2EC
// ADDRESS: 0041d2ec
// PROTOTYPE: undefined Catch@0041d2ec()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::AddResultToPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:356
// RVA: 0x000E26B0
// ADDRESS: 004e26b0
// PROTOTYPE: void __thiscall AddResultToPlayer(ulong param_1, ulong param_2, CPlayer * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::AddByteToArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:170
// RVA: 0x000E2830
// ADDRESS: 004e2830
// PROTOTYPE: void __thiscall AddByteToArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::QueryTaoZhuangIdByName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:322
// RVA: 0x000E2B80
// ADDRESS: 004e2b80
// PROTOTYPE: long __thiscall QueryTaoZhuangIdByName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::IsCollectAll
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:406
// RVA: 0x000E2C80
// ADDRESS: 004e2c80
// PROTOTYPE: bool __thiscall IsCollectAll(ulong param_1, ulong param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:29
// RVA: 0x000E3FC0
// ADDRESS: 004e3fc0
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::DeCodeFromByte
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:244
// RVA: 0x000E4020
// ADDRESS: 004e4020
// PROTOTYPE: void __thiscall DeCodeFromByte(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::CTaoZhuangSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:7
// RVA: 0x000E44D0
// ADDRESS: 004e44d0
// PROTOTYPE: undefined __thiscall CTaoZhuangSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::GetSingleInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:426
// RVA: 0x000E4560
// ADDRESS: 004e4560
// PROTOTYPE: CTaoZhuangSetup * __cdecl GetSingleInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp

// ============================================================================
// FUNCTION: CTaoZhuangSetup::AddByteToArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:170
// RVA: 0x00088340
// ADDRESS: 00488340
// PROTOTYPE: void __thiscall AddByteToArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:29
// RVA: 0x00089A80
// ADDRESS: 00489a80
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::ReadFile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:35
// RVA: 0x00089AE0
// ADDRESS: 00489ae0
// PROTOTYPE: bool __thiscall ReadFile(basic_string<char,std::char_traits<char>,std::allocator<char>_> param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::CTaoZhuangSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:7
// RVA: 0x0008A460
// ADDRESS: 0048a460
// PROTOTYPE: undefined __thiscall CTaoZhuangSetup(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaoZhuangSetup::GetSingleInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\taozhuangsetup.cpp:426
// RVA: 0x0008A4F0
// ADDRESS: 0048a4f0
// PROTOTYPE: CTaoZhuangSetup * __cdecl GetSingleInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
