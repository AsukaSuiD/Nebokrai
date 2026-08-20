//! Владелец базовых свойств товаров исторического `WorldServer`.
//!
//! Статус constructor RVA `0x000D5010`,
//! `GetPrice/GetWeight/GetName/GetDescribe/GetIconID` RVA
//! `0x000D4930/0x000D4940/0x000D4960/0x000D4970/0x000D4980`,
//! `GetGoodsType/GetEquipPlace` RVA `0x000DEA40/0x000DEA50`,
//! `GetAddonPropertyValues/GetValidAddonProperties` RVA
//! `0x000D4E50/0x000D4D90`, `GetOccurProbability/IsImplicit` RVA
//! `0x000D49C0/0x000D4A10` — `IMPLEMENTED`; остальной корпус ниже остаётся
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
//! unsigned `m_dwWeight` по `+0x3C`; exact getter состоит из одной загрузки.
//! PDB также подтверждает
//! `GAP_PARTICULAR_ATTRIBUTE = 0x0D`, `GAP_GOODS_STACKING_LIMIT = 0x26`,
//! `GAP_WEAPON_LEVEL = 0x30` и
//! layout `tagAddonPropertyValue`: unsigned `dwId` по `+0`, signed
//! `lBaseValue` по `+4`, modifier-флаг по `+8`, затем vector modifier-ов.
//! Rust-owner хранит все поля записи, заполняемые exact
//! `CGoodsFactory::Load`: original/localized name, описание, цену, вес, тип,
//! equip-place, три icon-а и addon-ы с modifier-ами. Неиспользуемые поля
//! входного формата фабрика только потребляет, как и оригинал.
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
pub(crate) const ICON_TYPE_CONTAINER: i32 = 0;
pub(crate) const ICON_TYPE_GROUND: i32 = 1;
pub(crate) const ICON_TYPE_EQUIPPED: i32 = 2;

struct GoodsBaseIcon {
    icon_type: i32,
    icon_id: u32,
}

pub(super) struct GoodsBaseAddonPropertyValueModifier {
    probability: u32,
    lower_limit: i32,
    upper_limit: i32,
}

/// Достигнутые scalar-поля исходного `tagAddonPropertyValue`.
pub(crate) struct GoodsBaseAddonPropertyValue {
    id: u32,
    base_value: i32,
    is_modifier_enabled: i32,
    modifiers: Vec<GoodsBaseAddonPropertyValueModifier>,
}

struct GoodsBaseAddonProperty {
    property_type: i32,
    is_enabled: i32,
    is_implicit_attribute: i32,
    occur_probability: u32,
    values: Vec<GoodsBaseAddonPropertyValue>,
}

/// Достигнутая stacking-часть исходного `CGoodsBaseProperties`.
pub(crate) struct CGoodsBaseProperties {
    original_name: Vec<u8>,
    name: Vec<u8>,
    description: Vec<u8>,
    price: u32,
    weight: u32,
    icons: Vec<GoodsBaseIcon>,
    goods_type: i32,
    equip_place: i32,
    addon_properties: Vec<GoodsBaseAddonProperty>,
}

impl CGoodsBaseProperties {
    /// Создаёт exact пустое состояние constructor-а `0x004D5010`.
    pub(super) const fn with_constructor_defaults() -> Self {
        Self {
            original_name: Vec::new(),
            name: Vec::new(),
            description: Vec::new(),
            price: 0,
            weight: 0,
            icons: Vec::new(),
            goods_type: GOODS_TYPE_USELESS,
            equip_place: 0,
            addon_properties: Vec::new(),
        }
    }

    /// Заимствует byte-exact исходное имя без завершающего NUL.
    pub(crate) fn get_original_name(&self) -> &[u8] {
        &self.original_name
    }

    /// Заимствует byte-exact имя без завершающего NUL.
    pub(crate) fn get_name(&self) -> &[u8] {
        &self.name
    }

    /// Заимствует byte-exact описание без завершающего NUL.
    pub(crate) fn get_description(&self) -> &[u8] {
        &self.description
    }

    /// Возвращает exact unsigned базовую цену.
    pub(crate) const fn get_price(&self) -> u32 {
        self.price
    }

    /// Возвращает exact unsigned вес одной единицы товара.
    pub(crate) const fn get_weight(&self) -> u32 {
        self.weight
    }

    /// Возвращает exact signed `GOODS_TYPE` без дополнительных эффектов.
    pub(crate) const fn get_goods_type(&self) -> i32 {
        self.goods_type
    }

    /// Возвращает exact signed `EQUIP_PLACE` без дополнительных эффектов.
    pub(crate) const fn get_equip_place(&self) -> i32 {
        self.equip_place
    }

    /// Возвращает icon первого совпавшего numeric-типа либо исходный `0`.
    pub(crate) fn get_icon_id(&self, icon_type: i32) -> u32 {
        self.icons
            .iter()
            .find(|icon| icon.icon_type == icon_type)
            .map_or(0, |icon| icon.icon_id)
    }

    /// Обходит тип каждого enabled property в исходном vector-order.
    pub(super) fn valid_addon_property_types(&self) -> impl Iterator<Item = i32> + '_ {
        self.addon_properties
            .iter()
            .filter(|property| property.is_enabled == 1)
            .map(|property| property.property_type)
    }

    /// Возвращает implicit-флаг первого property совпавшего типа.
    pub(crate) fn is_implicit(&self, property_type: i32) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .map_or(0, |property| property.is_implicit_attribute)
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

    pub(super) fn set_loaded_names(
        &mut self,
        original_name: Vec<u8>,
        name: Vec<u8>,
        description: Vec<u8>,
    ) {
        self.original_name = original_name;
        self.name = name;
        self.description = description;
    }

    pub(super) const fn set_loaded_scalars(&mut self, price: u32, weight: u32, raw_type: u32) {
        self.price = price;
        self.weight = weight;
        if raw_type == 1 {
            self.goods_type = GOODS_TYPE_CONSUMABLE;
        } else if raw_type >= 2 && raw_type <= 17 {
            self.goods_type = GOODS_TYPE_EQUIPMENT;
            self.equip_place = raw_type as i32 - 1;
        }
    }

    pub(super) fn push_loaded_icon(&mut self, icon_type: i32, icon_id: u32) {
        self.icons.push(GoodsBaseIcon { icon_type, icon_id });
    }

    pub(super) fn push_loaded_addon_property(
        &mut self,
        property_type: u16,
        is_enabled: bool,
        is_implicit_attribute: bool,
        first_base_value: i32,
        second_base_value: i32,
        occur_probability: u16,
    ) {
        self.addon_properties.push(GoodsBaseAddonProperty {
            property_type: i32::from(property_type),
            is_enabled: i32::from(is_enabled),
            is_implicit_attribute: i32::from(is_implicit_attribute),
            occur_probability: u32::from(occur_probability),
            values: vec![
                GoodsBaseAddonPropertyValue {
                    id: 1,
                    base_value: first_base_value,
                    is_modifier_enabled: 0,
                    modifiers: Vec::new(),
                },
                GoodsBaseAddonPropertyValue {
                    id: 2,
                    base_value: second_base_value,
                    is_modifier_enabled: 0,
                    modifiers: Vec::new(),
                },
            ],
        });
    }

    /// Возвращает `false` только для неизвестного value-id, как original loop.
    pub(super) fn push_loaded_modifier(
        &mut self,
        property_index: usize,
        value_id: u32,
        lower_limit: i32,
        upper_limit: i32,
        probability: u16,
    ) -> bool {
        let Some(value) = self
            .addon_properties
            .get_mut(property_index)
            .and_then(|property| {
                property
                    .values
                    .iter_mut()
                    .find(|value| value.id == value_id)
            })
        else {
            return false;
        };
        value.is_modifier_enabled = 1;
        value.modifiers.push(GoodsBaseAddonPropertyValueModifier {
            probability: u32::from(probability),
            lower_limit,
            upper_limit,
        });
        true
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

    pub(super) const fn is_modifier_enabled(&self) -> bool {
        self.is_modifier_enabled != 0
    }

    pub(super) fn modifiers(&self) -> &[GoodsBaseAddonPropertyValueModifier] {
        &self.modifiers
    }
}

impl GoodsBaseAddonPropertyValueModifier {
    pub(super) const fn probability(&self) -> u32 {
        self.probability
    }

    pub(super) const fn lower_limit(&self) -> i32 {
        self.lower_limit
    }

    pub(super) const fn upper_limit(&self) -> i32 {
        self.upper_limit
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:61
// RVA: 0x000D4940
// ADDRESS: 004d4940
// PROTOTYPE: ulong __thiscall GetWeight(void)
//
// IMPLEMENTED выше как прямой unsigned scalar-getter.

// ============================================================================
// FUNCTION: CGoodsBaseProperties::GetName
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\goods\cgoodsbaseproperties.cpp:15
// RVA: 0x000D5010
// ADDRESS: 004d5010
// PROTOTYPE: undefined __thiscall CGoodsBaseProperties(void)
//
// IMPLEMENTED выше как `with_constructor_defaults`; Rust `Vec` заменяет только
// STL storage, а все три строки, оба vector-а и четыре scalar-а получают exact
// пустые/нулевые значения.
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
