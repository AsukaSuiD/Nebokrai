//! Startup registry и lookup-часть `CGoodsFactory` GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/goods/cgoodsfactory.cpp`. Selector `0x00`
//! сначала освобождает три прежних map, затем читает `u32 count` и ordered
//! records `goods_id + CGoodsBaseProperties`; ID и оба byte-string index-а
//! используют last-write-wins. Lookup miss и null name возвращают ноль/`None`.
//!
//! Парный WorldServer serializer подтверждает wire. `BTreeMap` и owned values
//! заменяют MSVC tree/raw pointers без изменения порядка. Создание предметов,
//! upgrade и addon mutation ниже остаются RAW до materialization `CGoods` и
//! container lifecycle; startup registry и его runtime lookup-ы уже исполняемы.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::cgoods::{CGoods, GoodsAddonProperty, GoodsAddonPropertyValue};
use super::cgoodsbaseproperties::{
    CGoodsBaseProperties, GoodsBasePropertiesDecodeError, ICON_TYPE_GROUND,
};
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsFactoryDecodeReport {
    pub(crate) declared_records: usize,
    pub(crate) unique_goods: usize,
    pub(crate) unique_original_names: usize,
    pub(crate) unique_names: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsFactoryDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        available: usize,
    },
    BaseProperties(GoodsBasePropertiesDecodeError),
}

impl fmt::Display for GoodsFactoryDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                available,
            } => write!(
                formatter,
                "goods registry обрывается на {field} в {offset}: нужно 4, доступно {available}"
            ),
            Self::BaseProperties(error) => error.fmt(formatter),
        }
    }
}

impl Error for GoodsFactoryDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedEnd { .. } => None,
            Self::BaseProperties(error) => Some(error),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGoodsFactory {
    goods: BTreeMap<u32, CGoodsBaseProperties>,
    original_name_index: BTreeMap<Vec<u8>, u32>,
    name_index: BTreeMap<Vec<u8>, u32>,
}

impl CGoodsFactory {
    /// Достигнутый object/addon prefix `CreateGoods` RVA `0x000682E0`.
    /// Fairy/BattleFairy loaders остаются отдельной незамкнутой suffix-веткой.
    pub(crate) fn create_goods_core<Random, Guid>(
        &self,
        goods_index: u32,
        mut random: Random,
        mut create_guid: Guid,
    ) -> Option<CGoods>
    where
        Random: FnMut(i32) -> i32,
        Guid: FnMut() -> CGuid,
    {
        let properties = self.query_goods_base_properties(goods_index)?;
        let mut goods = CGoods::with_reached_constructor_defaults();
        goods.set_base_properties_index(goods_index);
        goods.set_name(properties.name());
        goods.set_description(properties.description());
        goods.set_price(properties.price());
        goods.set_graphics_id(properties.get_icon_id(ICON_TYPE_GROUND) as i32);
        goods.set_amount(1);

        for property in properties
            .addon_properties()
            .iter()
            .filter(|property| property.is_enabled == 1)
        {
            let roll = random(10_000) as u32;
            if roll >= property.occur_probability {
                continue;
            }
            let mut values = Vec::with_capacity(property.values.len());
            for value in &property.values {
                let mut modifier = 0;
                if value.is_modifier_enabled != 0 {
                    let modifier_roll = random(10_000) as u32;
                    let mut cumulative = 0u32;
                    if let Some(selected) = value.modifiers.iter().find(|candidate| {
                        let selected =
                            modifier_roll.wrapping_sub(cumulative) < candidate.probability;
                        cumulative = cumulative.wrapping_add(candidate.probability);
                        selected
                    }) {
                        let width = selected.upper_limit.wrapping_sub(selected.lower_limit);
                        modifier = random(width).wrapping_add(selected.lower_limit);
                    }
                }
                values.push(GoodsAddonPropertyValue {
                    id: value.id,
                    base_value: value.base_value,
                    modifier,
                });
            }
            goods.push_addon_property(GoodsAddonProperty {
                property_type: property.property_type,
                is_enabled: 1,
                is_implicit_attribute: property.is_implicit_attribute,
                values,
            });
        }
        goods.set_ex_id(create_guid());
        Some(goods)
    }

    pub(crate) fn release(&mut self) {
        self.goods.clear();
        self.original_name_index.clear();
        self.name_index.clear();
    }

    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<GoodsFactoryDecodeReport, GoodsFactoryDecodeError> {
        self.release();
        let count = read_factory_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            let goods_id = read_factory_u32(source, cursor, "goods id")?;
            let mut properties = CGoodsBaseProperties::default();
            properties
                .unserialize(source, cursor)
                .map_err(GoodsFactoryDecodeError::BaseProperties)?;
            self.original_name_index
                .insert(properties.original_name().to_vec(), goods_id);
            self.name_index.insert(properties.name().to_vec(), goods_id);
            self.goods.insert(goods_id, properties);
        }
        Ok(GoodsFactoryDecodeReport {
            declared_records: count as usize,
            unique_goods: self.goods.len(),
            unique_original_names: self.original_name_index.len(),
            unique_names: self.name_index.len(),
        })
    }

    pub(crate) const fn goods(&self) -> &BTreeMap<u32, CGoodsBaseProperties> {
        &self.goods
    }

    pub(crate) fn query_goods_base_properties(
        &self,
        goods_id: u32,
    ) -> Option<&CGoodsBaseProperties> {
        self.goods.get(&goods_id)
    }

    pub(crate) fn query_goods_original_name(&self, goods_id: u32) -> Option<&[u8]> {
        self.query_goods_base_properties(goods_id)
            .map(CGoodsBaseProperties::original_name)
    }

    pub(crate) fn query_goods_name(&self, goods_id: u32) -> Option<&[u8]> {
        self.query_goods_base_properties(goods_id)
            .map(CGoodsBaseProperties::name)
    }

    pub(crate) fn query_goods_id_by_original_name(&self, name: Option<&[u8]>) -> u32 {
        name.map(visible_c_string)
            .and_then(|name| self.original_name_index.get(name).copied())
            .unwrap_or(0)
    }

    pub(crate) fn query_goods_id_by_name(&self, name: Option<&[u8]>) -> u32 {
        name.map(visible_c_string)
            .and_then(|name| self.name_index.get(name).copied())
            .unwrap_or(0)
    }

    pub(crate) fn query_goods_base_properties_by_original_name(
        &self,
        name: Option<&[u8]>,
    ) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(self.query_goods_id_by_original_name(name))
    }

    pub(crate) fn query_goods_base_properties_by_name(
        &self,
        name: Option<&[u8]>,
    ) -> Option<&CGoodsBaseProperties> {
        self.query_goods_base_properties(self.query_goods_id_by_name(name))
    }

    pub(crate) fn get_gold_coin_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"MONEY"))
    }

    pub(crate) fn get_yuan_bao_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"YUANBAO"))
    }

    pub(crate) fn get_ji_fen_index(&self) -> u32 {
        self.query_goods_id_by_original_name(Some(b"JIFEN"))
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn read_factory_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, GoodsFactoryDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
        return Err(GoodsFactoryDecodeError::UnexpectedEnd {
            field,
            offset,
            available,
        });
    };
    *cursor += 4;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("goods scalar содержит четыре байта"),
    ))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp

// ============================================================================
// FUNCTION: CGoodsFactory::GarbageCollect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:481
// RVA: 0x00063A20
// ADDRESS: 00463a20
// PROTOTYPE: int __cdecl GarbageCollect(CGoods * * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::RepairEquipment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:605
// RVA: 0x00063A50
// ADDRESS: 00463a50
// PROTOTYPE: int __cdecl RepairEquipment(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::AddExterndProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1133
// RVA: 0x00063AA0
// ADDRESS: 00463aa0
// PROTOTYPE: int __cdecl AddExterndProperty(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CalculateRepairPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:497
// RVA: 0x00063D50
// ADDRESS: 00463d50
// PROTOTYPE: ulong __cdecl CalculateRepairPrice(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::EquipmentWaste
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:623
// RVA: 0x00063E50
// ADDRESS: 00463e50
// PROTOTYPE: int __cdecl EquipmentWaste(CGoods * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UnserializeGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:443
// RVA: 0x000640A0
// ADDRESS: 004640a0
// PROTOTYPE: CGoods * __cdecl UnserializeGoods(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CalculateVendPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:544
// RVA: 0x00064180
// ADDRESS: 00464180
// PROTOTYPE: ulong __cdecl CalculateVendPrice(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::Upgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:794
// RVA: 0x000642E0
// ADDRESS: 004642e0
// PROTOTYPE: int __cdecl Upgrade(CGoods * param_1, GOODS_ADDON_PROPERTIES param_2, GOODS_ADDON_PROPERTIES param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UpgradeBFEquipment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1215
// RVA: 0x00064410
// ADDRESS: 00464410
// PROTOTYPE: int __cdecl UpgradeBFEquipment(CGoods * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::UpgradeEquipment
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:652
// RVA: 0x00064850
// ADDRESS: 00464850
// PROTOTYPE: int __cdecl UpgradeEquipment(CGoods * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsMaxStackNumber
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:896
// RVA: 0x00066F70
// ADDRESS: 00466f70
// PROTOTYPE: ulong __cdecl QueryGoodsMaxStackNumber(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryGoodsBasePropertiesValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:919
// RVA: 0x000670D0
// ADDRESS: 004670d0
// PROTOTYPE: long __cdecl QueryGoodsBasePropertiesValue(ulong param_1, GOODS_ADDON_PROPERTIES param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:240
// RVA: 0x00068010
// ADDRESS: 00468010
// PROTOTYPE: void __cdecl CreateAddonProperties(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:312
// RVA: 0x000682E0
// ADDRESS: 004682e0
// PROTOTYPE: CGoods * __cdecl CreateGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CreateGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:370
// RVA: 0x00068450
// ADDRESS: 00468450
// PROTOTYPE: void __cdecl CreateGoods(ulong param_1, ulong param_2, vector<CGoods*,std::allocator<CGoods*>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DealWithExternAttr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1066
// RVA: 0x00068580
// ADDRESS: 00468580
// PROTOTYPE: int __cdecl DealWithExternAttr(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::CutPreData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1362
// RVA: 0x00068700
// ADDRESS: 00468700
// PROTOTYPE: int __cdecl CutPreData(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::QueryPropertyValue7
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1458
// RVA: 0x00068AB0
// ADDRESS: 00468ab0
// PROTOTYPE: long __cdecl QueryPropertyValue7(CGoods * param_1, GOODS_ADDON_PROPERTIES param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DaKongDeluxModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1628
// RVA: 0x00068ED0
// ADDRESS: 00468ed0
// PROTOTYPE: int __cdecl DaKongDeluxModify(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DealEnchaseGem
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:974
// RVA: 0x000690E0
// ADDRESS: 004690e0
// PROTOTYPE: int __cdecl DealEnchaseGem(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DaKongModify
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:1348
// RVA: 0x00069470
// ADDRESS: 00469470
// PROTOTYPE: int __cdecl DaKongModify(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::DaKongXiangQian
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:968
// RVA: 0x000694A0
// ADDRESS: 004694a0
// PROTOTYPE: int __cdecl DaKongXiangQian(CGoods * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsFactory::ReCreateAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoodsfactory.cpp:167
// RVA: 0x000694C0
// ADDRESS: 004694c0
// PROTOTYPE: void __cdecl ReCreateAddonProperties(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
