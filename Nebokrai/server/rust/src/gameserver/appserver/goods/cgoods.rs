//! Достигнутый object/addon core `CGoods` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходные owners
//! `server/gameserver/appserver/goods/cgoods.h/.cpp`. Материализованы shape
//! identity, base-properties index, amount/price/add-ticket/description,
//! ordered addon storage, first-match lookup с fallback в registry, exact
//! instance-addon mutation, DaKong modifier/cut/color semantics, stack classification/limit, equipment-upgrade
//! eligibility, timed equipment start-point, wrapping weight и адаптеры
//! ordinary/battle-fairy свойств к addon storage. BF-upgrade eligibility
//! проверяет catalog equipment type и instance-only level marker.
//! `Vec` и owned bytes заменяют MSVC storage, не меняя порядка и signed 32-bit
//! arithmetic.
//! `GetEnabledAddonProperties` сохраняет instance storage order и consumable
//! catalog fallback для полного player item-use addon loop.
//! NPC shop использует точный repair predicate и mutation durability value 2.
//! Единственный legacy null-deref в `CanStacked` при потерянном registry key
//! выражен typed block-ом, а не тихим `false`.
//!
//! Декодирование exact persistence-wire теперь включает `CShape`, legacy
//! description buffer, ordered addon/value storage и обе fairy-проекции;
//! `CGoodsFactory` и exp-config передаются явно вместо process-global owners.
//! Script durability getter/setter теперь сохраняют exact base-value storage,
//! включая запись `-1` без client update. Constructor/release, остальные
//! time-поля, обратный codec и прочая gameplay mutation ниже остаются RAW:
//! достигнутый core не выдаётся за весь 0xCC-byte legacy object.

use super::cbattlefairyproperty::{
    BattleFairyExpBlock, BattleFairyExpReport, BattleFairyPlayerFacts, CBattleFairyProperty,
};
use super::cgoodsbaseproperties::{
    CGoodsBaseProperties, EQUIP_PLACE_HEADGEAR, GAP_BAOSHI_COLOR, GAP_BF_AGILITY,
    GAP_BF_AGILITY_BASE, GAP_BF_BLAST, GAP_BF_BRAVE, GAP_BF_BRAVE_BASE, GAP_BF_CURRENT_EXP,
    GAP_BF_CURRENT_MAX_EXP, GAP_BF_CUT_HURT_SCALE, GAP_BF_HP, GAP_BF_LEVEL, GAP_BF_MODULE,
    GAP_BF_MP, GAP_BF_PULLULATERATE, GAP_BF_SPRITE, GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE,
    GAP_BF_STRENGH, GAP_BF_STRENGH_BASE, GAP_DAKONG_1, GAP_FAIRY_AGILITY,
    GAP_FAIRY_AGILITY_BASE_VALUE, GAP_FAIRY_COMBINATED_TIMES, GAP_FAIRY_EGG_ID, GAP_FAIRY_EXP,
    GAP_FAIRY_GROWING_RATE, GAP_FAIRY_HP, GAP_FAIRY_HP_BASE_VALUE, GAP_FAIRY_LEVEL,
    GAP_FAIRY_MAIN_ABILITY, GAP_FAIRY_MAX_COMBINATED_TIMES, GAP_FAIRY_MAX_EXP, GAP_FAIRY_RIPE_ID,
    GAP_FAIRY_RIPE_MAX_LEVEL, GAP_FAIRY_RIPE_MIN_LEVEL, GAP_FAIRY_STATE, GAP_FAIRY_STRENGTH,
    GAP_FAIRY_STRENGTH_BASE_VALUE, GAP_FAIRY_WAKAN, GAP_FAIRY_WAKAN_BASE_VALUE, GAP_FAIRY_YOUNG_ID,
    GAP_FUMO_PROPERTY, GAP_GOODS_LIFE_TYPE, GAP_GOODS_MAXIMUM_DURABILITY, GAP_GOODS_STACKING_LIMIT,
    GAP_GOODS_START_POINT, GAP_PARTICULAR_ATTRIBUTE, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
    GAP_WEAPON_LEVEL, GOODS_TYPE_CONSUMABLE, GOODS_TYPE_EQUIPMENT, GOODS_TYPE_USELESS,
};
use super::cgoodsfactory::CGoodsFactory;
use super::fairyproperties::{CFairyProperties, FairyExpBlock, FairyExpReport, FairyExpRuntime};
use crate::gameserver::appserver::shape::{CShape, ShapeDecodeError, ShapeIdentity};
use crate::public::guid::CGuid;

const GOODS_OBJECT_TYPE: i32 = 700;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoodsDecodeError {
    Shape(ShapeDecodeError),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    UnterminatedDescription {
        offset: usize,
        available: usize,
    },
    CollectionAllocationFailed {
        field: &'static str,
        count: u32,
    },
    MissingBaseProperties(GoodsBasePropertyBlock),
}

impl From<ShapeDecodeError> for GoodsDecodeError {
    fn from(value: ShapeDecodeError) -> Self {
        Self::Shape(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsBasePropertyBlock {
    pub(crate) index: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsAddonPropertyValue {
    pub(crate) id: u32,
    pub(crate) base_value: i32,
    pub(crate) modifier: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsAddonProperty {
    pub(crate) property_type: i32,
    pub(crate) is_enabled: i32,
    pub(crate) is_implicit_attribute: i32,
    pub(crate) values: Vec<GoodsAddonPropertyValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGoods {
    shape: CShape,
    base_properties_index: u32,
    amount: u32,
    price: u32,
    price_type: u32,
    add_ticket: u32,
    description: Vec<u8>,
    addon_properties: Vec<GoodsAddonProperty>,
    fairy_properties: Option<CFairyProperties>,
    battle_fairy_property: Option<CBattleFairyProperty>,
}

impl Default for CGoods {
    fn default() -> Self {
        Self::with_reached_constructor_defaults()
    }
}

impl CGoods {
    /// Достигнутый scalar/storage prefix constructor-а RVA `0x000CC3A0`.
    pub(crate) const fn with_reached_constructor_defaults() -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.base_object_mut().set_type(GOODS_OBJECT_TYPE);
        Self {
            shape,
            base_properties_index: 0,
            amount: 1,
            price: 0,
            price_type: 0,
            add_ticket: 0,
            description: Vec::new(),
            addon_properties: Vec::new(),
            fairy_properties: None,
            battle_fairy_property: None,
        }
    }

    pub(crate) const fn identity(&self) -> ShapeIdentity {
        self.shape.identity()
    }

    pub(crate) const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub(crate) const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub(crate) const fn set_ex_id(&mut self, ex_id: CGuid) {
        self.shape.base_object_mut().set_ex_id(ex_id);
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.shape.base_object_mut().set_name(name);
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.shape.base_object().get_name()
    }

    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape.base_object_mut().set_graphics_id(graphics_id);
    }

    pub(crate) const fn set_base_properties_index(&mut self, index: u32) {
        self.base_properties_index = index;
    }

    pub(crate) const fn base_properties_index(&self) -> u32 {
        self.base_properties_index
    }

    pub(crate) const fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    pub(crate) const fn amount(&self) -> u32 {
        self.amount
    }

    pub(crate) const fn set_price(&mut self, price: u32) {
        self.price = price;
    }

    pub(crate) const fn price(&self) -> u32 {
        self.price
    }

    pub(crate) const fn price_type(&self) -> u32 {
        self.price_type
    }

    pub(crate) const fn add_ticket(&self) -> u32 {
        self.add_ticket
    }

    pub(crate) fn set_add_ticket(&mut self, add_ticket: u32) {
        if !self.query_attribute(GAP_GOODS_STACKING_LIMIT) {
            self.add_ticket = add_ticket;
        }
    }

    pub(crate) fn set_description(&mut self, description: &[u8]) {
        let visible = description
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(description.len());
        self.description.clear();
        self.description.extend_from_slice(&description[..visible]);
    }

    pub(crate) fn description(&self) -> &[u8] {
        &self.description
    }

    pub(crate) fn clear_addon_properties(&mut self) {
        self.addon_properties.clear();
    }

    pub(crate) fn addon_properties(&self) -> &[GoodsAddonProperty] {
        &self.addon_properties
    }

    pub(crate) fn addon_properties_mut(&mut self) -> &mut Vec<GoodsAddonProperty> {
        &mut self.addon_properties
    }

    pub(crate) fn push_addon_property(&mut self, property: GoodsAddonProperty) {
        self.addon_properties.push(property);
    }

    /// Exact persisted `CGoods::Serialize` для межсерверного auction-node.
    /// Derived fairy-проекции не имеют отдельного wire: они восстанавливаются
    /// decoder-ом из base properties и addon list.
    pub(crate) fn serialize(&self, destination: &mut Vec<u8>, include_child: bool) -> bool {
        if !self.shape.encode_to_byte_array(destination, include_child) {
            return false;
        }
        destination.extend_from_slice(&self.base_properties_index.to_le_bytes());
        destination.extend_from_slice(&self.amount.to_le_bytes());
        destination.extend_from_slice(&self.price.to_le_bytes());
        destination.extend_from_slice(&self.description);
        destination.push(0);
        destination.extend_from_slice(&(self.addon_properties.len() as u32).to_le_bytes());
        for property in &self.addon_properties {
            destination.extend_from_slice(&property.property_type.to_le_bytes());
            destination.extend_from_slice(&property.is_enabled.to_le_bytes());
            destination.extend_from_slice(&property.is_implicit_attribute.to_le_bytes());
            destination.extend_from_slice(&(property.values.len() as u32).to_le_bytes());
            for value in &property.values {
                destination.extend_from_slice(&value.id.to_le_bytes());
                destination.extend_from_slice(&value.base_value.to_le_bytes());
                destination.extend_from_slice(&value.modifier.to_le_bytes());
            }
        }
        true
    }

    /// Exact `SerializeForOldClient` projection, используемая live
    /// `0xBF918` updates. В отличие от persisted `Serialize`, здесь идут
    /// catalog type/place/weight, current durability, modifier-only ordinary
    /// addons и отдельный DaKong tail. GUID сохраняет legacy 0/16 marker.
    pub(crate) fn serialize_for_old_client(
        &self,
        destination: &mut Vec<u8>,
        factory: &CGoodsFactory,
        da_kong_enabled: bool,
    ) -> bool {
        let Some(base) = factory.query_goods_base_properties(self.base_properties_index) else {
            return false;
        };
        let da_kong_count = self
            .addon_properties
            .iter()
            .filter(|property| {
                base.get_addon_property_values(property.property_type)
                    .is_empty()
            })
            .count();
        let normal_count = self.addon_properties.len().saturating_sub(da_kong_count);
        destination.extend_from_slice(&(self.base_properties_index as i32).to_le_bytes());
        destination.extend_from_slice(&self.identity().id.to_le_bytes());
        let guid = self.identity().ex_id;
        if guid == CGuid::GUID_INVALID {
            destination.push(0);
        } else {
            destination.push(16);
            destination.extend_from_slice(guid.as_legacy_bytes());
        }
        destination.extend_from_slice(&(self.amount as i32).to_le_bytes());
        destination.extend_from_slice(self.name());
        destination.push(0);
        destination.extend_from_slice(&(self.price as i32).to_le_bytes());
        destination.extend_from_slice(
            &base
                .equip_place()
                .wrapping_add(base.goods_type().wrapping_sub(GOODS_TYPE_CONSUMABLE))
                .to_le_bytes(),
        );
        destination.extend_from_slice(&(base.weight() as i32).to_le_bytes());
        destination.extend_from_slice(
            &self
                .addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2)
                .to_le_bytes(),
        );
        destination.extend_from_slice(&(normal_count as i32).to_le_bytes());
        for property in self.addon_properties.iter().take(normal_count) {
            let modifier = |id| {
                property
                    .values
                    .iter()
                    .find(|value| value.id == id)
                    .map_or(0, |value| value.modifier)
            };
            let value1 = if property.property_type == GAP_GOODS_LIFE_TYPE {
                let high = i64::from(self.addon_property_value(factory, GAP_GOODS_START_POINT, 1));
                let low = self.addon_property_value(factory, GAP_GOODS_START_POINT, 2) as u32;
                let start = (high << 32) | i64::from(low);
                if start == 0 {
                    modifier(1)
                } else {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |duration| duration.as_secs() as i64);
                    let elapsed = if start <= now { now - start } else { 0 };
                    let lifetime =
                        i64::from(self.addon_property_value(factory, GAP_GOODS_LIFE_TYPE, 1));
                    let remaining = if elapsed < lifetime {
                        lifetime - elapsed
                    } else {
                        0
                    };
                    let base_value = property
                        .values
                        .iter()
                        .find(|value| value.id == 1)
                        .map_or(0, |value| value.base_value);
                    remaining.wrapping_sub(i64::from(base_value)) as i32
                }
            } else {
                modifier(1)
            };
            destination.extend_from_slice(&(property.property_type as u16).to_le_bytes());
            destination.extend_from_slice(&value1.to_le_bytes());
            destination.extend_from_slice(&modifier(2).to_le_bytes());
        }
        let visible_da_kong_count = if da_kong_enabled { da_kong_count } else { 0 };
        destination.extend_from_slice(&(self.shape.get_pos_x() as i32).to_le_bytes());
        destination.extend_from_slice(&(self.shape.get_pos_y() as i32).to_le_bytes());
        destination.extend_from_slice(&(visible_da_kong_count as i32).to_le_bytes());
        for property in self
            .addon_properties
            .iter()
            .skip(normal_count)
            .take(visible_da_kong_count)
        {
            destination.extend_from_slice(&(property.property_type as u16).to_le_bytes());
            let mut base_value1 = 0;
            let mut base_value2 = 0;
            for value in &property.values {
                if value.id == 1 {
                    base_value1 = value.base_value;
                    destination.extend_from_slice(&value.modifier.to_le_bytes());
                }
                if value.id == 2 {
                    base_value2 = value.base_value;
                    destination.extend_from_slice(&value.modifier.to_le_bytes());
                }
            }
            destination.push(u8::from(property.is_enabled != 0));
            destination.push(u8::from(property.is_implicit_attribute != 0));
            destination.extend_from_slice(&base_value1.to_le_bytes());
            destination.extend_from_slice(&base_value2.to_le_bytes());
            destination.extend_from_slice(&10_000u16.to_le_bytes());
        }
        true
    }

    /// Exact `CGoods::Unserialize`: release, shape/scalar/string/addon wire и
    /// обе derived fairy-проекции выполняются в исходном порядке.
    pub(crate) fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        factory: &CGoodsFactory,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<(), GoodsDecodeError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        self.base_properties_index = 0;
        self.amount = 0;
        self.description.clear();
        self.addon_properties.clear();
        self.fairy_properties = None;
        self.battle_fairy_property = None;

        self.shape
            .decode_from_byte_array(source, cursor, include_child)?;
        self.base_properties_index =
            read_goods_wire_u32(source, cursor, "m_dwBasePropertiesIndex")?;
        self.amount = read_goods_wire_u32(source, cursor, "m_dwAmount")?;
        self.price = read_goods_wire_u32(source, cursor, "m_dwPrice")?;
        self.description = read_goods_wire_description(source, cursor)?;

        let property_count = read_goods_wire_u32(source, cursor, "m_vAddonProperties count")?;
        self.addon_properties
            .try_reserve_exact(property_count as usize)
            .map_err(|_| GoodsDecodeError::CollectionAllocationFailed {
                field: "m_vAddonProperties",
                count: property_count,
            })?;
        for _ in 0..property_count {
            let property_type = read_goods_wire_i32(source, cursor, "tagAddonProperty.gapType")?;
            let is_enabled = read_goods_wire_i32(source, cursor, "tagAddonProperty.bIsEnabled")?;
            let is_implicit_attribute =
                read_goods_wire_i32(source, cursor, "tagAddonProperty.bIsImplicitAttribute")?;
            let value_count =
                read_goods_wire_u32(source, cursor, "tagAddonProperty.vValues count")?;
            let mut values = Vec::new();
            values
                .try_reserve_exact(value_count as usize)
                .map_err(|_| GoodsDecodeError::CollectionAllocationFailed {
                    field: "tagAddonProperty.vValues",
                    count: value_count,
                })?;
            for _ in 0..value_count {
                values.push(GoodsAddonPropertyValue {
                    id: read_goods_wire_u32(source, cursor, "tagAddonPropertyValue.dwId")?,
                    base_value: read_goods_wire_i32(
                        source,
                        cursor,
                        "tagAddonPropertyValue.lBaseValue",
                    )?,
                    modifier: read_goods_wire_i32(
                        source,
                        cursor,
                        "tagAddonPropertyValue.lModifier",
                    )?,
                });
            }
            self.addon_properties.push(GoodsAddonProperty {
                property_type,
                is_enabled,
                is_implicit_attribute,
                values,
            });
        }

        self.load_fairy_properties(factory, ordinary_threshold)
            .map_err(GoodsDecodeError::MissingBaseProperties)?;
        self.load_battle_fairy_property(factory, battle_threshold)
            .map_err(GoodsDecodeError::MissingBaseProperties)?;
        Ok(())
    }

    pub(crate) const fn fairy_properties(&self) -> Option<&CFairyProperties> {
        self.fairy_properties.as_ref()
    }

    pub(crate) const fn fairy_properties_mut(&mut self) -> Option<&mut CFairyProperties> {
        self.fairy_properties.as_mut()
    }

    /// Exact `CGoods::HatchBegin`: только готовое яйцо без активного timer-а.
    pub(crate) fn hatch_begin(&mut self, current_tick: u32, egg_max_level: u32) -> bool {
        let Some(fairy) = self.fairy_properties_mut() else {
            return false;
        };
        if fairy.hatch_start_time != 0 || fairy.fairy_state != 0 || fairy.level < egg_max_level {
            return false;
        }
        fairy.hatch_start_time = current_tick;
        true
    }

    /// Exact `CGoods::HatchStop`: наличие fairy property достаточно, поэтому
    /// повторная остановка также считается успешной.
    pub(crate) fn hatch_stop(&mut self) -> bool {
        let Some(fairy) = self.fairy_properties_mut() else {
            return false;
        };
        fairy.hatch_start_time = 0;
        true
    }

    pub(crate) const fn battle_fairy_property(&self) -> Option<&CBattleFairyProperty> {
        self.battle_fairy_property.as_ref()
    }

    pub(crate) const fn battle_fairy_property_mut(&mut self) -> Option<&mut CBattleFairyProperty> {
        self.battle_fairy_property.as_mut()
    }

    /// Safe adapter legacy interior pointer: временно отделяет property owner,
    /// но возвращает его товару и при typed block-е.
    pub(crate) fn fairy_exp_up<Threshold>(
        &mut self,
        experience: &mut u32,
        runtime: FairyExpRuntime<'_>,
        threshold_for_level: Threshold,
    ) -> Result<Option<FairyExpReport>, FairyExpBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let Some(mut fairy) = self.fairy_properties.take() else {
            return Ok(None);
        };
        let result = fairy.exp_up(experience, runtime, threshold_for_level);
        self.fairy_properties = Some(fairy);
        result.map(Some)
    }

    /// Safe adapter двойного mutable borrow исходных `property + goods`.
    pub(crate) fn battle_fairy_exp_up<Threshold>(
        &mut self,
        factory: &CGoodsFactory,
        player: Option<BattleFairyPlayerFacts<'_>>,
        experience: &mut u32,
        threshold_for_level: Threshold,
    ) -> Result<Option<BattleFairyExpReport>, BattleFairyExpBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let Some(mut battle_fairy) = self.battle_fairy_property.take() else {
            return Ok(None);
        };
        let result =
            battle_fairy.exp_up(Some(self), factory, player, experience, threshold_for_level);
        self.battle_fairy_property = Some(battle_fairy);
        result.map(Some)
    }

    /// Exact ordinary-fairy loader обрывает traversal на первом addon-е без
    /// values и затем переписывает raw modifier `GAP_FAIRY_MAX_EXP` из config.
    pub(crate) fn load_fairy_properties<Threshold>(
        &mut self,
        factory: &CGoodsFactory,
        mut threshold_for_level: Threshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let properties = factory
            .query_goods_base_properties(self.base_properties_index)
            .ok_or(GoodsBasePropertyBlock {
                index: self.base_properties_index,
            })?;
        if properties.equip_place() != EQUIP_PLACE_HEADGEAR {
            return Ok(false);
        }

        let fairy = self
            .fairy_properties
            .get_or_insert_with(CFairyProperties::new);
        for addon in &self.addon_properties {
            let Some(value) = addon.values.first() else {
                return Ok(true);
            };
            let total = value.base_value.wrapping_add(value.modifier) as u32;
            match addon.property_type {
                GAP_ROLE_MINIMUM_LEVEL_LIMIT => fairy.equip_level = total,
                GAP_FAIRY_STATE => fairy.fairy_state = total,
                GAP_FAIRY_COMBINATED_TIMES => fairy.combinated_times = total,
                GAP_FAIRY_MAX_COMBINATED_TIMES => fairy.max_combinated_times = total,
                GAP_FAIRY_LEVEL => fairy.level = total,
                GAP_FAIRY_RIPE_MIN_LEVEL => fairy.ripe_min_level = total,
                GAP_FAIRY_RIPE_MAX_LEVEL => fairy.ripe_max_level = total,
                GAP_FAIRY_EXP => fairy.link_experience(value.modifier),
                GAP_FAIRY_MAIN_ABILITY => fairy.main_ability = total,
                GAP_FAIRY_GROWING_RATE => fairy.growing_rate = total,
                GAP_FAIRY_STRENGTH => fairy.strength = total,
                GAP_FAIRY_AGILITY => fairy.agility = total,
                GAP_FAIRY_WAKAN => fairy.wakan = total,
                GAP_FAIRY_HP => fairy.hp = total,
                GAP_FAIRY_STRENGTH_BASE_VALUE => fairy.base_strength = total,
                GAP_FAIRY_AGILITY_BASE_VALUE => fairy.base_agility = total,
                GAP_FAIRY_WAKAN_BASE_VALUE => fairy.base_wakan = total,
                GAP_FAIRY_HP_BASE_VALUE => fairy.base_hp = total,
                GAP_FAIRY_EGG_ID => fairy.egg_id = total,
                GAP_FAIRY_YOUNG_ID => fairy.young_id = total,
                GAP_FAIRY_RIPE_ID => fairy.ripe_id = total,
                _ => {}
            }
        }

        let equip_level = fairy.equip_level;
        let level = fairy.level;
        for addon in &mut self.addon_properties {
            if addon.property_type != GAP_FAIRY_MAX_EXP {
                continue;
            }
            let value = addon
                .values
                .first_mut()
                .expect("пустой addon завершил первый exact traversal");
            value.modifier = threshold_for_level(equip_level, level) as i32;
            fairy.max_exp = value.base_value.wrapping_add(value.modifier) as u32;
        }
        Ok(true)
    }

    pub(crate) fn save_fairy_properties(
        &mut self,
        factory: &CGoodsFactory,
    ) -> Result<bool, GoodsBasePropertyBlock> {
        let properties = factory
            .query_goods_base_properties(self.base_properties_index)
            .ok_or(GoodsBasePropertyBlock {
                index: self.base_properties_index,
            })?;
        if properties.equip_place() != EQUIP_PLACE_HEADGEAR {
            return Ok(false);
        }
        let Some(fairy) = self.fairy_properties.as_ref() else {
            return Ok(false);
        };
        for addon in &mut self.addon_properties {
            let Some(value) = addon.values.first_mut() else {
                continue;
            };
            if addon.property_type == GAP_FAIRY_EXP {
                if let Some(experience) = fairy.experience() {
                    value.modifier = experience;
                }
                continue;
            }
            let stored = match addon.property_type {
                GAP_FAIRY_STATE => fairy.fairy_state,
                GAP_FAIRY_COMBINATED_TIMES => fairy.combinated_times,
                GAP_FAIRY_LEVEL => fairy.level,
                GAP_FAIRY_MAX_EXP => fairy.max_exp,
                GAP_FAIRY_GROWING_RATE => fairy.growing_rate,
                GAP_FAIRY_STRENGTH => fairy.strength,
                GAP_FAIRY_AGILITY => fairy.agility,
                GAP_FAIRY_WAKAN => fairy.wakan,
                GAP_FAIRY_HP => fairy.hp,
                GAP_FAIRY_STRENGTH_BASE_VALUE => fairy.base_strength,
                GAP_FAIRY_AGILITY_BASE_VALUE => fairy.base_agility,
                GAP_FAIRY_WAKAN_BASE_VALUE => fairy.base_wakan,
                GAP_FAIRY_HP_BASE_VALUE => fairy.base_hp,
                _ => continue,
            };
            value.modifier = (stored as i32).wrapping_sub(value.base_value);
        }
        Ok(true)
    }

    pub(crate) fn load_battle_fairy_property<Threshold>(
        &mut self,
        factory: &CGoodsFactory,
        mut threshold_for_level: Threshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let properties = factory
            .query_goods_base_properties(self.base_properties_index)
            .ok_or(GoodsBasePropertyBlock {
                index: self.base_properties_index,
            })?;
        if properties.equip_place() != EQUIP_PLACE_HEADGEAR {
            return Ok(false);
        }

        let mut battle = self.battle_fairy_property.take().unwrap_or_default();
        for addon in &self.addon_properties {
            let Some(value) = addon.values.first() else {
                self.battle_fairy_property = Some(battle);
                return Ok(true);
            };
            let total = value.base_value.wrapping_add(value.modifier) as u32;
            match addon.property_type {
                GAP_ROLE_MINIMUM_LEVEL_LIMIT => battle.equip_level = total,
                GAP_BF_LEVEL => battle.current_level = total,
                GAP_BF_CURRENT_EXP => battle.set_current_experience_linked(true),
                GAP_BF_BLAST => battle.blast = total,
                GAP_BF_BRAVE | GAP_BF_BRAVE_BASE => battle.brave = total,
                GAP_BF_AGILITY => battle.agility = total,
                GAP_BF_SPRITUALISM => battle.spritualism = total,
                GAP_BF_STRENGH => battle.strength = total,
                GAP_BF_PULLULATERATE => battle.set_pullulate_rate(total as f32),
                GAP_BF_MODULE => battle.module = Some(total),
                GAP_BF_AGILITY_BASE => battle.agility_base = Some(total),
                GAP_BF_SPRITUALISM_BASE => battle.spritualism_base = Some(total),
                GAP_BF_STRENGH_BASE => battle.strength_base = Some(total),
                GAP_BF_CUT_HURT_SCALE => battle.immunity = Some(total),
                _ => {}
            }
        }

        let equip_level = battle.equip_level;
        let current_level = battle.current_level;
        for addon in &mut self.addon_properties {
            if addon.property_type != GAP_BF_CURRENT_MAX_EXP {
                continue;
            }
            let value = addon
                .values
                .first_mut()
                .expect("пустой addon завершил первый exact traversal");
            value.modifier = threshold_for_level(equip_level, current_level) as i32;
            battle.max_exp = value.base_value.wrapping_add(value.modifier) as u32;
        }
        self.battle_fairy_property = Some(battle);
        Ok(true)
    }

    /// Exact SaveBF нормализует выбранные first values через собственный
    /// `GetAddonPropertyValues`; property object служит только presence gate.
    pub(crate) fn save_battle_fairy_property(
        &mut self,
        factory: &CGoodsFactory,
    ) -> Result<bool, GoodsBasePropertyBlock> {
        let properties = factory
            .query_goods_base_properties(self.base_properties_index)
            .ok_or(GoodsBasePropertyBlock {
                index: self.base_properties_index,
            })?;
        if properties.equip_place() != EQUIP_PLACE_HEADGEAR || self.battle_fairy_property.is_none()
        {
            return Ok(false);
        }
        const SAVED_TYPES: [i32; 16] = [
            GAP_BF_LEVEL,
            GAP_BF_CURRENT_MAX_EXP,
            GAP_BF_HP,
            GAP_BF_MP,
            GAP_BF_SPRITE,
            GAP_BF_BLAST,
            GAP_BF_BRAVE,
            GAP_BF_AGILITY,
            GAP_BF_SPRITUALISM,
            GAP_BF_STRENGH,
            GAP_BF_PULLULATERATE,
            GAP_BF_BRAVE_BASE,
            GAP_BF_AGILITY_BASE,
            GAP_BF_SPRITUALISM_BASE,
            GAP_BF_STRENGH_BASE,
            GAP_BF_CUT_HURT_SCALE,
        ];
        let totals: Vec<_> = SAVED_TYPES
            .iter()
            .map(|property_type| {
                (
                    *property_type,
                    self.addon_property_value(factory, *property_type, 1),
                )
            })
            .collect();
        for addon in &mut self.addon_properties {
            let Some(value) = addon.values.first_mut() else {
                continue;
            };
            let Some((_, total)) = totals
                .iter()
                .find(|(property_type, _)| *property_type == addon.property_type)
            else {
                continue;
            };
            value.modifier = total.wrapping_sub(value.base_value);
        }
        Ok(true)
    }

    /// Общий storage-prefix обоих exact copy overload-ов.
    pub(crate) fn copy_addon_properties_core_from(&mut self, source: &Self) {
        self.addon_properties.clone_from(&source.addon_properties);
    }

    pub(crate) fn copy_fairy_addon_properties_from<Threshold>(
        &mut self,
        source: &Self,
        factory: &CGoodsFactory,
        threshold_for_level: Threshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        self.copy_addon_properties_core_from(source);
        self.load_fairy_properties(factory, threshold_for_level)
    }

    pub(crate) fn copy_battle_fairy_addon_properties_from<Threshold>(
        &mut self,
        source: &Self,
        factory: &CGoodsFactory,
        threshold_for_level: Threshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        self.copy_addon_properties_core_from(source);
        self.load_battle_fairy_property(factory, threshold_for_level)
    }

    pub(crate) fn query_attribute(&self, property_type: i32) -> bool {
        self.addon_properties
            .iter()
            .any(|property| property.property_type == property_type)
    }

    /// Exact `HasAddonPropertyValues` смотрит только catalog addon-values и
    /// не проверяет instance storage.
    pub(crate) fn has_addon_property_values(
        &self,
        factory: &CGoodsFactory,
        property_type: i32,
    ) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                !properties
                    .get_addon_property_values(property_type)
                    .is_empty()
            })
    }

    /// Exact `CanUpgraded` проверяет catalog type, но сам upgrade marker ищет
    /// только среди instance-addon-ов. Registry fallback здесь не применяется.
    pub(crate) fn can_upgraded(&self, factory: &CGoodsFactory) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.query_attribute(GAP_WEAPON_LEVEL)
            })
    }

    /// Exact `CanBFEquipeUpgrade`: catalog type обязан быть equipment, marker
    /// уровня ищется только среди instance-addon-ов, без registry fallback.
    pub(crate) fn can_battle_fairy_equipment_upgrade(&self, factory: &CGoodsFactory) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.query_attribute(super::cgoodsbaseproperties::GAP_BF_WEAPON_LEVEL)
            })
    }

    /// Exact `CanReparied`: только equipment без particular-attribute bit 0.
    pub(crate) fn can_repair(&self, factory: &CGoodsFactory) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 1 == 0
            })
    }

    pub(crate) fn repair_durability(&mut self, factory: &CGoodsFactory) -> bool {
        if !self.can_repair(factory) {
            return false;
        }
        let maximum = self.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 1);
        self.set_addon_property_value_first_core(GAP_GOODS_MAXIMUM_DURABILITY, 2, maximum)
    }

    /// Exact durability owner читает `base_value`, не сумму с modifier.
    pub(crate) fn current_durability(&self) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == GAP_GOODS_MAXIMUM_DURABILITY)
            .and_then(|property| property.values.iter().find(|value| value.id == 2))
            .map_or(-1, |value| value.base_value)
    }

    /// Ноль и отсутствующий value-id `2` возвращают `-1`. Отрицательное
    /// значение, включая `-1`, записывается до возврата как в native owner-е.
    pub(crate) fn set_current_durability(&mut self, durability: i32) -> i32 {
        if durability == 0 {
            return -1;
        }
        let Some(value) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == GAP_GOODS_MAXIMUM_DURABILITY)
            .and_then(|property| property.values.iter_mut().find(|value| value.id == 2))
        else {
            return -1;
        };
        value.base_value = durability;
        durability
    }

    /// Exact `QueryDaKongCount` считает только непрерывный prefix семи
    /// instance/base addon-слотов со значением value-id 1 в диапазоне 2..=8.
    pub(crate) fn da_kong_count(&self, factory: &CGoodsFactory) -> u32 {
        let mut count = 0;
        for offset in 0..=6 {
            let value = self.addon_property_value(factory, GAP_DAKONG_1 + offset, 1);
            if value < 2 {
                return count;
            }
            if 8 < value {
                break;
            }
            count += 1;
        }
        count
    }

    pub(crate) fn addon_property_value(
        &self,
        factory: &CGoodsFactory,
        property_type: i32,
        value_id: u32,
    ) -> i32 {
        if let Some(value) = self
            .addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .and_then(|property| property.values.iter().find(|value| value.id == value_id))
        {
            return value.base_value.wrapping_add(value.modifier);
        }
        factory
            .query_goods_base_properties(self.base_properties_index)
            .and_then(|properties| {
                properties
                    .get_addon_property_values(property_type)
                    .iter()
                    .find(|value| value.id == value_id)
            })
            .map_or(0, |value| value.base_value)
    }

    /// Exact `GetEnabledAddonProperties`: instance-addon-ы идут в storage
    /// order; catalog fallback используется только когда среди них нет ни
    /// одного enabled property и товар остаётся consumable.
    pub(crate) fn enabled_addon_properties(&self, factory: &CGoodsFactory) -> Vec<i32> {
        let enabled: Vec<_> = self
            .addon_properties
            .iter()
            .filter(|property| property.is_enabled == 1)
            .map(|property| property.property_type)
            .collect();
        if !enabled.is_empty() {
            return enabled;
        }
        factory
            .query_goods_base_properties(self.base_properties_index)
            .filter(|properties| properties.goods_type() == GOODS_TYPE_CONSUMABLE)
            .map(|properties| properties.valid_addon_properties().collect())
            .unwrap_or_default()
    }

    /// Storage-prefix `SetAddonPropertyValue` меняет modifier первого
    /// совпавшего value во всех instance-addon-ах данного типа и не создаёт
    /// отсутствующие записи. Registry fallback при записи не используется;
    /// последующий fairy/battle-fairy reload остаётся у их owner-ов.
    pub(crate) fn set_addon_property_value_core(
        &mut self,
        property_type: i32,
        value_id: u32,
        value: i32,
    ) -> bool {
        let mut changed = false;
        for property in self
            .addon_properties
            .iter_mut()
            .filter(|property| property.property_type == property_type)
        {
            if let Some(found) = property
                .values
                .iter_mut()
                .find(|candidate| candidate.id == value_id)
            {
                found.modifier = value.wrapping_sub(found.base_value);
                changed = true;
            }
        }
        changed
    }

    /// DaKong-facing exact prefix `SetAddonPropertyValue`: native функция
    /// останавливается на первой паре property/value.
    pub(crate) fn set_addon_property_value_first_core(
        &mut self,
        property_type: i32,
        value_id: u32,
        value: i32,
    ) -> bool {
        let Some(found) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == property_type)
            .and_then(|property| {
                property
                    .values
                    .iter_mut()
                    .find(|value| value.id == value_id)
            })
        else {
            return false;
        };
        found.modifier = value.wrapping_sub(found.base_value);
        true
    }

    /// Safe replacement pointer-а `lCurrentExp`: первый instance value
    /// возвращает именно modifier, не сумму base+modifier.
    pub(crate) fn instance_addon_modifier(&self, property_type: i32, value_id: u32) -> Option<i32> {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .and_then(|property| property.values.iter().find(|value| value.id == value_id))
            .map(|value| value.modifier)
    }

    pub(crate) fn set_instance_addon_modifier(
        &mut self,
        property_type: i32,
        value_id: u32,
        modifier: i32,
    ) -> bool {
        let Some(value) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == property_type)
            .and_then(|property| {
                property
                    .values
                    .iter_mut()
                    .find(|value| value.id == value_id)
            })
        else {
            return false;
        };
        value.modifier = modifier;
        true
    }

    /// Exact `SetAddonPropertyModifier`: меняет только первое найденное
    /// instance-value и хранит переданное число именно как modifier.
    pub(crate) fn set_addon_property_modifier_core(
        &mut self,
        property_type: i32,
        value_id: u32,
        modifier: i32,
    ) -> bool {
        self.set_instance_addon_modifier(property_type, value_id, modifier)
    }

    /// `SetFuMoProperty` хранит выбранный тип и его величину в единственном
    /// дополнении `GAP_FUMO_PROPERTY`. Повторный вызов полностью заменяет обе
    /// пары значений, а не добавляет второе дополнение того же типа.
    pub(crate) fn set_fu_mo_property(&mut self, property_type: i32, value: i32) -> i32 {
        if let Some(property) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == GAP_FUMO_PROPERTY)
        {
            property
                .values
                .resize(2, GoodsAddonPropertyValue::default());
            property.values[0] = GoodsAddonPropertyValue {
                id: 1,
                base_value: 0,
                modifier: property_type,
            };
            property.values[1] = GoodsAddonPropertyValue {
                id: 2,
                base_value: 0,
                modifier: value,
            };
            return 1;
        }
        self.addon_properties.push(GoodsAddonProperty {
            property_type: GAP_FUMO_PROPERTY,
            is_enabled: 1,
            is_implicit_attribute: 0,
            values: vec![
                GoodsAddonPropertyValue {
                    id: 1,
                    base_value: 0,
                    modifier: property_type,
                },
                GoodsAddonPropertyValue {
                    id: 2,
                    base_value: 0,
                    modifier: value,
                },
            ],
        });
        1
    }

    /// Exact `CutAddonPropertyValue`: существующий modifier уменьшается и
    /// clamp-ится к нулю. Отрицательный расход создаёт instance-property из
    /// универсальной пары значений либо из схемы source goods.
    pub(crate) fn cut_addon_property_value(
        &mut self,
        factory: &CGoodsFactory,
        property_type: i32,
        value_id: u32,
        amount: i32,
        source_goods_index: u32,
    ) -> bool {
        if let Some(value) = self
            .addon_properties
            .iter_mut()
            .find(|property| property.property_type == property_type)
            .and_then(|property| {
                property
                    .values
                    .iter_mut()
                    .find(|value| value.id == value_id)
            })
        {
            value.modifier = value.modifier.wrapping_sub(amount).max(0);
            return true;
        }
        if amount >= 0 {
            return true;
        }

        let new_modifier = amount.wrapping_neg();
        if source_goods_index == 0 {
            self.addon_properties.push(GoodsAddonProperty {
                property_type,
                is_enabled: 1,
                is_implicit_attribute: 0,
                values: [1, 2]
                    .into_iter()
                    .map(|id| GoodsAddonPropertyValue {
                        id,
                        base_value: 0,
                        modifier: if id == value_id { new_modifier } else { 0 },
                    })
                    .collect(),
            });
            return true;
        }

        if let Some(source) = factory
            .query_goods_base_properties(source_goods_index)
            .and_then(|properties| {
                properties
                    .addon_properties()
                    .iter()
                    .find(|property| property.property_type == property_type)
            })
        {
            self.addon_properties.push(GoodsAddonProperty {
                property_type,
                is_enabled: source.is_enabled,
                is_implicit_attribute: source.is_implicit_attribute,
                values: source
                    .values
                    .iter()
                    .map(|value| GoodsAddonPropertyValue {
                        id: value.id,
                        base_value: 0,
                        modifier: if value.id == value_id {
                            new_modifier
                        } else {
                            0
                        },
                    })
                    .collect(),
            });
        }
        true
    }

    /// Exact `QueryEnchanseColor`: считает цвет catalog gem-а в семи
    /// сохранённых socket index-ах, включая непоследовательные отверстия.
    pub(crate) fn query_enchanse_color(&self, factory: &CGoodsFactory, color: i32) -> i32 {
        (0..7).fold(0i32, |count, offset| {
            let gem_index = self.addon_property_value(factory, GAP_DAKONG_1 + offset, 2) as u32;
            let gem_color = factory
                .query_goods_base_properties(gem_index)
                .and_then(|properties| {
                    properties
                        .get_addon_property_values(GAP_BAOSHI_COLOR)
                        .iter()
                        .find(|value| value.id == 1)
                })
                .map_or(0, |value| value.base_value);
            count.wrapping_add(i32::from(gem_color != 0 && gem_color == color))
        })
    }

    pub(crate) fn goods_time_type(&self, factory: &CGoodsFactory) -> u32 {
        self.addon_property_value(factory, GAP_GOODS_LIFE_TYPE, 2) as u32
    }

    pub(crate) fn goods_lifetime(&self, factory: &CGoodsFactory) -> u32 {
        self.addon_property_value(factory, GAP_GOODS_LIFE_TYPE, 1) as u32
    }

    pub(crate) fn start_point(&self, factory: &CGoodsFactory) -> u64 {
        let high = self.addon_property_value(factory, GAP_GOODS_START_POINT, 1) as u32;
        let low = self.addon_property_value(factory, GAP_GOODS_START_POINT, 2) as u32;
        (u64::from(high) << 32) | u64::from(low)
    }

    /// Legacy setter последовательно пишет low, затем high. На повреждённой
    /// addon-схеме первая запись может состояться без второй, что намеренно не
    /// сворачивается в атомарную замену.
    pub(crate) fn set_start_point(&mut self, start_point: u64) {
        let _ =
            self.set_addon_property_value_core(GAP_GOODS_START_POINT, 2, start_point as u32 as i32);
        let _ = self.set_addon_property_value_core(
            GAP_GOODS_START_POINT,
            1,
            (start_point >> 32) as u32 as i32,
        );
    }

    /// Timed-prefix `CEquipmentContainer::Add`: типы 2/4 получают текущую
    /// точку только при нулевом старте. Возвращает факт попытки legacy setter-а.
    pub(crate) fn initialize_equipment_start_point(
        &mut self,
        factory: &CGoodsFactory,
        now: u64,
    ) -> bool {
        if matches!(self.goods_time_type(factory), 2 | 4) && self.start_point(factory) == 0 {
            self.set_start_point(now);
            true
        } else {
            false
        }
    }

    pub(crate) fn can_stack(
        &self,
        factory: &CGoodsFactory,
    ) -> Result<bool, GoodsBasePropertyBlock> {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .ok_or(GoodsBasePropertyBlock {
                index: self.base_properties_index,
            })
            .map(|properties| {
                matches!(
                    properties.goods_type(),
                    GOODS_TYPE_CONSUMABLE | GOODS_TYPE_USELESS
                ) && properties.has_addon_property(GAP_GOODS_STACKING_LIMIT)
            })
    }

    pub(crate) fn max_stack_number(&self, factory: &CGoodsFactory) -> u32 {
        let Some(properties) = factory.query_goods_base_properties(self.base_properties_index)
        else {
            return 1;
        };
        if !matches!(
            properties.goods_type(),
            GOODS_TYPE_CONSUMABLE | GOODS_TYPE_USELESS
        ) {
            return 1;
        }
        properties
            .get_addon_property_values(GAP_GOODS_STACKING_LIMIT)
            .iter()
            .find(|value| value.id == 1)
            .map_or(1, |value| value.base_value as u32)
    }

    pub(crate) fn weight(&self, factory: &CGoodsFactory) -> u32 {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .map_or(0, |properties: &CGoodsBaseProperties| {
                properties.weight().wrapping_mul(self.amount)
            })
    }
}

fn read_goods_wire_description(
    source: &[u8],
    cursor: &mut usize,
) -> Result<Vec<u8>, GoodsDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let searchable = available.min(0x404);
    let Some(bytes) = source.get(offset..offset.saturating_add(searchable)) else {
        return Err(GoodsDecodeError::UnexpectedEnd {
            field: "m_strDescribe",
            offset,
            needed: 1,
            available,
        });
    };
    let Some(length) = bytes.iter().position(|byte| *byte == 0) else {
        return Err(GoodsDecodeError::UnterminatedDescription { offset, available });
    };
    *cursor = offset + length + 1;
    Ok(bytes[..length].to_vec())
}

fn read_goods_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GoodsDecodeError> {
    Ok(i32::from_le_bytes(read_goods_wire(source, cursor, field)?))
}

fn read_goods_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, GoodsDecodeError> {
    Ok(u32::from_le_bytes(read_goods_wire(source, cursor, field)?))
}

fn read_goods_wire<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], GoodsDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(GoodsDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(GoodsDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor = end;
    Ok(bytes
        .try_into()
        .expect("slice содержит ровно запрошенное число байт"))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp

// ============================================================================
// FUNCTION: CGoods::IsFairy
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.h:270
// RVA: 0x000AEB20
// ADDRESS: 004aeb20
// PROTOTYPE: bool __thiscall IsFairy(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:599
// RVA: 0x000C9730
// ADDRESS: 004c9730
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:604
// RVA: 0x000C9750
// ADDRESS: 004c9750
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetOriginalName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1839
// RVA: 0x000C97D0
// ADDRESS: 004c97d0
// PROTOTYPE: char * __thiscall GetOriginalName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::HatchBegin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1245
// RVA: 0x000C9840
// ADDRESS: 004c9840
// PROTOTYPE: bool __thiscall HatchBegin(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::HatchStop
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1260
// RVA: 0x000C9880
// ADDRESS: 004c9880
// PROTOTYPE: bool __thiscall HatchStop(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CanUpgraded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:130
// RVA: 0x000C9930
// ADDRESS: 004c9930
// PROTOTYPE: int __thiscall CanUpgraded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetAddonPropertyBaseValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:375
// RVA: 0x000C9AB0
// ADDRESS: 004c9ab0
// PROTOTYPE: int __thiscall SetAddonPropertyBaseValues(GOODS_ADDON_PROPERTIES param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:733
// RVA: 0x000C9B80
// ADDRESS: 004c9b80
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::LoadFairyPropertiesFromGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1042
// RVA: 0x000C9E00
// ADDRESS: 004c9e00
// PROTOTYPE: bool __thiscall LoadFairyPropertiesFromGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SaveFairyPropertiesToGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1165
// RVA: 0x000CA1C0
// ADDRESS: 004ca1c0
// PROTOTYPE: void __thiscall SaveFairyPropertiesToGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::LoadBFPropertyFromGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1660
// RVA: 0x000CA340
// ADDRESS: 004ca340
// PROTOTYPE: bool __thiscall LoadBFPropertyFromGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:427
// RVA: 0x000CA7B0
// ADDRESS: 004ca7b0
// PROTOTYPE: int __thiscall SetAddonPropertyValue(GOODS_ADDON_PROPERTIES param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:540
// RVA: 0x000CA8C0
// ADDRESS: 004ca8c0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetGoodsLifeTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1806
// RVA: 0x000CA9B0
// ADDRESS: 004ca9b0
// PROTOTYPE: void __thiscall SetGoodsLifeTime(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetGoodsTimeType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1815
// RVA: 0x000CA9D0
// ADDRESS: 004ca9d0
// PROTOTYPE: void __thiscall SetGoodsTimeType(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetStartPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1825
// RVA: 0x000CA9F0
// ADDRESS: 004ca9f0
// PROTOTYPE: void __thiscall SetStartPoint(__uint64 param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::SetName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:833
// RVA: 0x000CADC0
// ADDRESS: 004cadc0
// PROTOTYPE: void __thiscall SetName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Clear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:718
// RVA: 0x000CB030
// ADDRESS: 004cb030
// PROTOTYPE: void __thiscall Clear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::HasAddonPropertyValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:360
// RVA: 0x000CB3B0
// ADDRESS: 004cb3b0
// PROTOTYPE: bool __thiscall HasAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Clone
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:521
// RVA: 0x000CB470
// ADDRESS: 004cb470
// PROTOTYPE: int __thiscall Clone(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:702
// RVA: 0x000CB530
// ADDRESS: 004cb530
// PROTOTYPE: undefined __thiscall tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryAttrbuteInGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1308
// RVA: 0x000CB550
// ADDRESS: 004cb550
// PROTOTYPE: bool __thiscall QueryAttrbuteInGoodsList(GOODS_ADDON_PROPERTIES param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryDaKongCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1336
// RVA: 0x000CB640
// ADDRESS: 004cb640
// PROTOTYPE: int __thiscall QueryDaKongCount(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::QueryNoInSelfPropertyType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1561
// RVA: 0x000CB7C0
// ADDRESS: 004cb7c0
// PROTOTYPE: long __thiscall QueryNoInSelfPropertyType(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SaveBFPropertyToGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1575
// RVA: 0x000CB830
// ADDRESS: 004cb830
// PROTOTYPE: void __thiscall SaveBFPropertyToGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetGoodsLifeTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1810
// RVA: 0x000CBA00
// ADDRESS: 004cba00
// PROTOTYPE: ulong __thiscall GetGoodsLifeTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetGoodsTimeType
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1820
// RVA: 0x000CBA10
// ADDRESS: 004cba10
// PROTOTYPE: ulong __thiscall GetGoodsTimeType(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetStartPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1831
// RVA: 0x000CBA20
// ADDRESS: 004cba20
// PROTOTYPE: __uint64 __thiscall GetStartPoint(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::GetAddonPropertyValues
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:250
// RVA: 0x000CBBE0
// ADDRESS: 004cbbe0
// PROTOTYPE: void __thiscall GetAddonPropertyValues(GOODS_ADDON_PROPERTIES param_1, vector<CGoods::tagAddonPropertyValue,std::allocator<CGoods::tagAddonPropertyValue>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:749
// RVA: 0x000CBDC0
// ADDRESS: 004cbdc0
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SerializeForOldClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:862
// RVA: 0x000CBE90
// ADDRESS: 004cbe90
// PROTOTYPE: int __thiscall SerializeForOldClient(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:23
// RVA: 0x000CC3A0
// ADDRESS: 004cc3a0
// PROTOTYPE: undefined __thiscall CGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:617
// RVA: 0x000CC440
// ADDRESS: 004cc440
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::~CGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:48
// RVA: 0x000CC580
// ADDRESS: 004cc580
// PROTOTYPE: void __thiscall ~CGoods(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::SetFuMoProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:458
// RVA: 0x000CC610
// ADDRESS: 004cc610
// PROTOTYPE: int __thiscall SetFuMoProperty(GOODS_ADDON_PROPERTIES param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CopyAddonProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1236
// RVA: 0x000CC9A0
// ADDRESS: 004cc9a0
// PROTOTYPE: void __thiscall CopyAddonProperties(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::CopyBFAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:1762
// RVA: 0x000CCDC0
// ADDRESS: 004ccdc0
// PROTOTYPE: void __thiscall CopyBFAddonProperty(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonPropertyValue::~tagAddonPropertyValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:661
// RVA: 0x000D3D00
// ADDRESS: 004d3d00
// PROTOTYPE: void __thiscall ~tagAddonPropertyValue(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d421d
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D421D
// ADDRESS: 004d421d
// PROTOTYPE: undefined Catch@004d421d()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d42d6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D42D6
// ADDRESS: 004d42d6
// PROTOTYPE: undefined Catch@004d42d6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d44c1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D44C1
// ADDRESS: 004d44c1
// PROTOTYPE: undefined Catch@004d44c1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGoods::tagAddonProperty::~tagAddonProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp:711
// RVA: 0x000D4660
// ADDRESS: 004d4660
// PROTOTYPE: void __thiscall ~tagAddonProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d490a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D490A
// ADDRESS: 004d490a
// PROTOTYPE: undefined Catch@004d490a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d49be
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D49BE
// ADDRESS: 004d49be
// PROTOTYPE: undefined Catch@004d49be()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d4c1e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D4C1E
// ADDRESS: 004d4c1e
// PROTOTYPE: undefined Catch@004d4c1e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d4cdd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D4CDD
// ADDRESS: 004d4cdd
// PROTOTYPE: undefined Catch@004d4cdd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d4e7c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D4E7C
// ADDRESS: 004d4e7c
// PROTOTYPE: undefined Catch@004d4e7c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d5290
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D5290
// ADDRESS: 004d5290
// PROTOTYPE: undefined Catch@004d5290()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d5354
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D5354
// ADDRESS: 004d5354
// PROTOTYPE: undefined Catch@004d5354()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d5554
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D5554
// ADDRESS: 004d5554
// PROTOTYPE: undefined Catch@004d5554()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004d55e3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cgoods.cpp
// RVA: 0x000D55E3
// ADDRESS: 004d55e3
// PROTOTYPE: undefined Catch@004d55e3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
