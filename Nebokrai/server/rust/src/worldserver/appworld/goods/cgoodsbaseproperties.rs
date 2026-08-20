//! Владелец базовых свойств товаров исторического `WorldServer`.
//!
//! Статус `GetGoodsType/GetEquipPlace` RVA `0x000DEA40/0x000DEA50`,
//! `GetAddonPropertyValues` RVA `0x000D4E50` и `GetOccurProbability` RVA
//! `0x000D49C0` — `IMPLEMENTED`; остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:285`;
//! inline getter скомпонован линкером по одному RVA с равными scalar-getter-ами.
//!
//! Exact PDB задаёт `GOODS_TYPE` как signed `int`: `GT_USELESS = 0`,
//! `GT_CONSUMABLE = 1`, `GT_EQUIPMENT = 2`, а соседний signed
//! `EQUIP_PLACE m_epEquipPlace` лежит по `+0x60`. Его значения `0..16`
//! буквально соответствуют `EP_UNKNOWN..EP_LINGBAO`. Он же подтверждает
//! `GAP_PARTICULAR_ATTRIBUTE = 0x0D`, `GAP_GOODS_STACKING_LIMIT = 0x26`,
//! `GAP_WEAPON_LEVEL = 0x30` и
//! layout `tagAddonPropertyValue`: unsigned `dwId` по `+0`, signed
//! `lBaseValue` по `+4`, modifier-флаг по `+8`, затем vector modifier-ов.
//! Достигнутый Rust-owner пока хранит только поля, читаемые восстановленным
//! stacking-путём; `Load/Serialize/Unserialize` и остальные поля остаются raw.
//!
//! `GetAddonPropertyValues` останавливается на первом property совпавшего типа
//! и копирует все его values. Заимствованный slice заменяет временную копию
//! только внутри синхронного read-only вызова `CGoods::GetMaxStackNumber`:
//! порядок, первое совпадение и значения сохраняются, а STL allocation/copy/
//! destruction не являются наблюдаемым контрактом.

pub(crate) const GOODS_TYPE_USELESS: i32 = 0;
pub(crate) const GOODS_TYPE_CONSUMABLE: i32 = 1;
pub(crate) const GOODS_TYPE_EQUIPMENT: i32 = 2;
pub(crate) const GAP_PARTICULAR_ATTRIBUTE: i32 = 0x0d;
pub(crate) const GAP_GOODS_STACKING_LIMIT: i32 = 0x26;
pub(crate) const GAP_WEAPON_LEVEL: i32 = 0x30;

/// Достигнутые scalar-поля исходного `tagAddonPropertyValue`.
pub(crate) struct GoodsBaseAddonPropertyValue {
    id: u32,
    base_value: i32,
}

struct GoodsBaseAddonProperty {
    property_type: i32,
    occur_probability: u32,
    values: Vec<GoodsBaseAddonPropertyValue>,
}

/// Достигнутая stacking-часть исходного `CGoodsBaseProperties`.
pub(crate) struct CGoodsBaseProperties {
    goods_type: i32,
    equip_place: i32,
    addon_properties: Vec<GoodsBaseAddonProperty>,
}

impl CGoodsBaseProperties {
    /// Возвращает exact signed `GOODS_TYPE` без дополнительных эффектов.
    pub(crate) const fn get_goods_type(&self) -> i32 {
        self.goods_type
    }

    /// Возвращает exact signed `EQUIP_PLACE` без дополнительных эффектов.
    pub(crate) const fn get_equip_place(&self) -> i32 {
        self.equip_place
    }

    /// Возвращает values первого property совпавшего numeric-типа.
    pub(crate) fn get_addon_property_values(
        &self,
        property_type: i32,
    ) -> &[GoodsBaseAddonPropertyValue] {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(&[], |property| property.values.as_slice())
    }

    /// Возвращает probability первого property совпавшего numeric-типа.
    pub(crate) fn get_occur_probability(&self, property_type: i32) -> u32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.occur_probability)
    }
}

impl GoodsBaseAddonPropertyValue {
    /// Возвращает exact unsigned идентификатор значения.
    pub(crate) const fn id(&self) -> u32 {
        self.id
    }

    /// Возвращает exact signed базовое значение.
    pub(crate) const fn base_value(&self) -> i32 {
        self.base_value
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp

// ============================================================================
// FUNCTION: std::_Tree<std::_Tmap_traits<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,char*,std::less<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>,std::allocator<std::pair<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_const_,char*>_>,0>_>::_Min
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00055C50
// ADDRESS: 00455c50
// PROTOTYPE: _Node * __cdecl _Min(_Node * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004575d6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x000575D6
// ADDRESS: 004575d6
// PROTOTYPE: undefined Catch@004575d6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045767e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0005767E
// ADDRESS: 0045767e
// PROTOTYPE: undefined Catch@0045767e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00457881
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00057881
// ADDRESS: 00457881
// PROTOTYPE: undefined Catch@00457881()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00457d73
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00057D73
// ADDRESS: 00457d73
// PROTOTYPE: undefined Catch@00457d73()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00457e02
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00057E02
// ADDRESS: 00457e02
// PROTOTYPE: undefined Catch@00457e02()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045887a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0005887A
// ADDRESS: 0045887a
// PROTOTYPE: undefined Catch@0045887a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00458afe
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00058AFE
// ADDRESS: 00458afe
// PROTOTYPE: undefined Catch@00458afe()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00459015
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00059015
// ADDRESS: 00459015
// PROTOTYPE: undefined Catch@00459015()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004590d4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x000590D4
// ADDRESS: 004590d4
// PROTOTYPE: undefined Catch@004590d4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004591ec
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x000591EC
// ADDRESS: 004591ec
// PROTOTYPE: undefined Catch@004591ec()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00459420
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00059420
// ADDRESS: 00459420
// PROTOTYPE: undefined Catch@00459420()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00459bc4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00059BC4
// ADDRESS: 00459bc4
// PROTOTYPE: undefined Catch@00459bc4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00459dcb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00059DCB
// ADDRESS: 00459dcb
// PROTOTYPE: undefined Catch@00459dcb()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00459e5a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x00059E5A
// ADDRESS: 00459e5a
// PROTOTYPE: undefined Catch@00459e5a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetPrice
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:53
// RVA: 0x000D4930
// ADDRESS: 004d4930
// PROTOTYPE: ulong __thiscall GetPrice(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetWeight
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:61
// RVA: 0x000D4940
// ADDRESS: 004d4940
// PROTOTYPE: ulong __thiscall GetWeight(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:39
// RVA: 0x000D4960
// ADDRESS: 004d4960
// PROTOTYPE: char * __thiscall GetName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetDescribe
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:46
// RVA: 0x000D4970
// ADDRESS: 004d4970
// PROTOTYPE: char * __thiscall GetDescribe(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetIconID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:182
// RVA: 0x000D4980
// ADDRESS: 004d4980
// PROTOTYPE: ulong __thiscall GetIconID(ICON_TYPE param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetOccurProbability
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:198
// RVA: 0x000D49C0
// ADDRESS: 004d49c0
// PROTOTYPE: ulong __thiscall GetOccurProbability(GOODS_ADDON_PROPERTIES param_1)
//
// IMPLEMENTED выше; сохраняются первое совпадение и unsigned ноль при отсутствии.

// ============================================================================
// FUNCTION: CGoodsBaseProperties::IsImplicit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:215
// RVA: 0x000D4A10
// ADDRESS: 004d4a10
// PROTOTYPE: int __thiscall IsImplicit(GOODS_ADDON_PROPERTIES param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::tagAddonProperty::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:425
// RVA: 0x000D4B20
// ADDRESS: 004d4b20
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:118
// RVA: 0x000D4BD0
// ADDRESS: 004d4bd0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::tagAddonPropertyValue::~tagAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:326
// RVA: 0x000D4D70
// ADDRESS: 004d4d70
// PROTOTYPE: void __thiscall ~tagAddonPropertyValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetValidAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:303
// RVA: 0x000D4D90
// ADDRESS: 004d4d90
// PROTOTYPE: void __thiscall GetValidAddonProperties(vector<CGoodsBaseProperties::GOODS_ADDON_PROPERTIES,std::allocator<CGoodsBaseProperties::GOODS_ADDON_PROPERTIES>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetAddonPropertyValues
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:285
// RVA: 0x000D4E50
// ADDRESS: 004d4e50
// PROTOTYPE: void __thiscall GetAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1, vector<CGoodsBaseProperties::tagAddonPropertyValue,std::allocator<CGoodsBaseProperties::tagAddonPropertyValue>_> * param_2)
//
// IMPLEMENTED выше как `get_addon_property_values`; slice сохраняет первое
// property-совпадение и исходный порядок values без STL copy-noise.
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::tagAddonProperty::tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:390
// RVA: 0x000D4EE0
// ADDRESS: 004d4ee0
// PROTOTYPE: undefined __thiscall tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::tagAddonProperty::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:407
// RVA: 0x000D4F00
// ADDRESS: 004d4f00
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::tagAddonProperty::~tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:400
// RVA: 0x000D4F90
// ADDRESS: 004d4f90
// PROTOTYPE: void __thiscall ~tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::CGoodsBaseProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:15
// RVA: 0x000D5010
// ADDRESS: 004d5010
// PROTOTYPE: undefined __thiscall CGoodsBaseProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoodsBaseProperties::~CGoodsBaseProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:26
// RVA: 0x000D5080
// ADDRESS: 004d5080
// PROTOTYPE: void __thiscall ~CGoodsBaseProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f090
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F090
// ADDRESS: 0052f090
// PROTOTYPE: undefined Unwind@0052f090()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f0d0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F0D0
// ADDRESS: 0052f0d0
// PROTOTYPE: undefined Unwind@0052f0d0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f0f0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F0F0
// ADDRESS: 0052f0f0
// PROTOTYPE: undefined Unwind@0052f0f0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f110
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F110
// ADDRESS: 0052f110
// PROTOTYPE: undefined Unwind@0052f110()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@0052f180
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F180
// ADDRESS: 0052f180
// PROTOTYPE: undefined Unwind@0052f180()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@0052f1c3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F1C3
// ADDRESS: 0052f1c3
// PROTOTYPE: undefined Unwind@0052f1c3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@0052f20f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F20F
// ADDRESS: 0052f20f
// PROTOTYPE: undefined Unwind@0052f20f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f230
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F230
// ADDRESS: 0052f230
// PROTOTYPE: undefined Unwind@0052f230()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: Unwind@0052f2a1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp
// RVA: 0x0012F2A1
// ADDRESS: 0052f2a1
// PROTOTYPE: undefined Unwind@0052f2a1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
