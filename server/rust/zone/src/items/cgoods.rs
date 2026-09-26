//! Object/addon core `CGoods` исторического GameServer: shape identity,
//! base-properties index, amount/price/ticket, ordered addon storage, stacking,
//! upgrade eligibility и persistence-wire кодек. Исходные owners
//! `appserver/goods/cgoods.h/.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Contract: docs/gameplay/items.md.
//!
//! Lookup реестра базовых свойств типизирован trait-швом
//! [`GoodsBasePropertiesLookup`]; журнал эффектов роста феи — шов
//! `FairyGrowEffectSink` (`fairyproperties.rs`). `Vec` и owned bytes заменяют
//! MSVC storage без смены порядка и signed 32-bit arithmetic; `Clone` использует
//! ту же исходную пару `Serialize(true) → Unserialize(true)` и новый constructor
//! target, а не Rust field-copy. Quirk-и: единственный legacy null-deref в
//! `CanStacked` при потерянном registry key выражен typed block-ом; общий
//! SetAddonPropertyValue меняет первый value во всех совпавших addon-ах и лишь
//! затем существующие ordinary/BF-проекции; отсутствующий catalog при reload
//! возвращает typed block после записи без отката. Constructor/release,
//! остальные time-поля, обратный codec и прочая gameplay mutation ещё требуют
//! реконструкции: достигнутый core не выдаётся за весь `0xCC`-byte legacy object.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#предметы-и-контейнеры

use super::cbattlefairyproperty::{
    BattleFairyExpBlock, BattleFairyExpReport, BattleFairyPlayerFacts, CBattleFairyProperty,
};
use crate::content::goods::{
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
use super::fairyproperties::{
    CFairyProperties, FairyExpBlock, FairyExpReport, FairyExpRuntime, FairyGrowEffectSink,
};
use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use crate::regions::shape::{CShape, ShapeDecodeError};
use crate::regions::ShapeIdentity;
use nebokrai_shared::values::CGuid;
use thiserror::Error;

/// Lookup реестра базовых свойств товара у его владельца. Реализация —
/// `CGoodsFactory` Zone `content/goodsfactory.rs`; имя операции сохраняет
/// исходный `QueryGoodsBaseProperties`.
pub trait GoodsBasePropertiesLookup {
    fn query_goods_base_properties(&self, goods_id: u32) -> Option<&CGoodsBaseProperties>;
}

const GOODS_OBJECT_TYPE: i32 = 700;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GoodsDecodeError {
    #[error(transparent)]
    Shape(#[from] ShapeDecodeError),
    #[error("goods обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("goods description с {offset} не завершено NUL при {available} доступных байтах")]
    UnterminatedDescription {
        offset: usize,
        available: usize,
    },
    #[error("не удалось выделить goods collection {field} для {count} записей")]
    CollectionAllocationFailed {
        field: &'static str,
        count: u32,
    },
    #[error("не найдены base properties goods с index {}", .0.index)]
    MissingBaseProperties(GoodsBasePropertyBlock),
}

#[derive(Debug, Error)]
pub enum GoodsCloneError {
    #[error("CGoods::Serialize отклонил persisted clone payload")]
    EncodeRejected,
    #[error(transparent)]
    Decode(#[from] GoodsDecodeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoodsBasePropertyBlock {
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoodsAddonPropertyValue {
    pub id: u32,
    pub base_value: i32,
    pub modifier: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GoodsAddonProperty {
    pub property_type: i32,
    pub is_enabled: i32,
    pub is_implicit_attribute: i32,
    pub values: Vec<GoodsAddonPropertyValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CGoods {
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
    pub const fn with_reached_constructor_defaults() -> Self {
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

    pub const fn identity(&self) -> ShapeIdentity {
        self.shape.identity()
    }

    pub const fn shape(&self) -> &CShape {
        &self.shape
    }

    pub const fn shape_mut(&mut self) -> &mut CShape {
        &mut self.shape
    }

    pub const fn set_ex_id(&mut self, ex_id: CGuid) {
        self.shape.base_object_mut().set_ex_id(ex_id);
    }

    pub fn set_name(&mut self, name: &[u8]) {
        self.shape.base_object_mut().set_name(name);
    }

    pub fn name(&self) -> &[u8] {
        self.shape.base_object().get_name()
    }

    pub const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.shape.base_object_mut().set_graphics_id(graphics_id);
    }

    pub const fn set_base_properties_index(&mut self, index: u32) {
        self.base_properties_index = index;
    }

    pub const fn base_properties_index(&self) -> u32 {
        self.base_properties_index
    }

    pub const fn set_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    pub const fn amount(&self) -> u32 {
        self.amount
    }

    pub const fn set_price(&mut self, price: u32) {
        self.price = price;
    }

    pub const fn price(&self) -> u32 {
        self.price
    }

    pub const fn price_type(&self) -> u32 {
        self.price_type
    }

    pub const fn add_ticket(&self) -> u32 {
        self.add_ticket
    }

    pub fn set_add_ticket(&mut self, add_ticket: u32) {
        if !self.query_attribute(GAP_GOODS_STACKING_LIMIT) {
            self.add_ticket = add_ticket;
        }
    }

    pub fn set_description(&mut self, description: &[u8]) {
        let visible = description
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(description.len());
        self.description.clear();
        self.description.extend_from_slice(&description[..visible]);
    }

    pub fn description(&self) -> &[u8] {
        &self.description
    }

    pub fn clear_addon_properties(&mut self) {
        self.addon_properties.clear();
    }

    pub fn addon_properties(&self) -> &[GoodsAddonProperty] {
        &self.addon_properties
    }

    pub fn addon_properties_mut(&mut self) -> &mut Vec<GoodsAddonProperty> {
        &mut self.addon_properties
    }

    pub fn push_addon_property(&mut self, property: GoodsAddonProperty) {
        self.addon_properties.push(property);
    }

    /// Exact persisted `CGoods::Serialize` для native Clone и
    /// межсерверного auction-node.
    /// Derived fairy-проекции не имеют отдельного wire: они восстанавливаются
    /// decoder-ом из base properties и addon list.
    pub fn serialize(&self, destination: &mut Vec<u8>, include_child: bool) -> bool {
        if !self.shape.add_to_byte_array(destination, include_child) {
            return false;
        }
        let mut writer = LegacyWriter::new(destination);
        writer.write_u32(self.base_properties_index);
        writer.write_u32(self.amount);
        writer.write_u32(self.price);
        writer.write_c_string(&self.description);
        writer.write_u32(self.addon_properties.len() as u32);
        for property in &self.addon_properties {
            writer.write_i32(property.property_type);
            writer.write_i32(property.is_enabled);
            writer.write_i32(property.is_implicit_attribute);
            writer.write_u32(property.values.len() as u32);
            for value in &property.values {
                writer.write_u32(value.id);
                writer.write_i32(value.base_value);
                writer.write_i32(value.modifier);
            }
        }
        true
    }

    /// Exact `CGoods::Clone` RVA `0x000CB470`: source сериализуется с
    /// `include_child = true`, а новый constructor target декодирует тот же
    /// payload. Runtime-only `price_type/add_ticket` намеренно не копируются;
    /// fairy-проекции восстанавливаются decoder-ом из catalog/addon данных.
    pub fn clone_via_persisted_codec<OrdinaryThreshold, BattleThreshold>(
        &self,
        factory: &impl GoodsBasePropertiesLookup,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<Self, GoodsCloneError>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        let mut payload = Vec::new();
        if !self.serialize(&mut payload, true) {
            return Err(GoodsCloneError::EncodeRejected);
        }
        let mut cloned = Self::with_reached_constructor_defaults();
        let mut cursor = 0;
        cloned.unserialize(
            &payload,
            &mut cursor,
            true,
            factory,
            ordinary_threshold,
            battle_threshold,
        )?;
        debug_assert_eq!(cursor, payload.len());
        Ok(cloned)
    }

    /// Exact `SerializeForOldClient` projection, используемая live
    /// `0xBF918` updates. В отличие от persisted `Serialize`, здесь идут
    /// catalog type/place/weight, current durability, modifier-only ordinary
    /// addons и отдельный DaKong tail. GUID сохраняет legacy 0/16 marker.
    pub fn serialize_for_old_client(
        &self,
        destination: &mut Vec<u8>,
        factory: &impl GoodsBasePropertiesLookup,
        da_kong_enabled: bool,
    ) -> bool {
        let Some(base) = factory.query_goods_base_properties(self.base_properties_index) else {
            return false;
        };
        let da_kong_count = self
            .addon_properties
            .iter()
            // QueryNoInSelfPropertyType (VA 0x004CB7C0) проверяет наличие
            // типа в каталоге, даже если у него пустой список значений.
            .filter(|property| !base.has_addon_property(property.property_type))
            .count();
        let normal_count = self.addon_properties.len().saturating_sub(da_kong_count);
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(self.base_properties_index as i32);
        writer.write_i32(self.identity().id);
        let guid = self.identity().ex_id;
        if guid == CGuid::GUID_INVALID {
            writer.write_u8(0);
        } else {
            writer.write_u8(16);
            writer.write_bytes(guid.as_legacy_bytes());
        }
        writer.write_i32(self.amount as i32);
        writer.write_c_string(self.name());
        writer.write_i32(self.price as i32);
        writer.write_i32(
            base.equip_place()
                .wrapping_add(base.goods_type().wrapping_sub(GOODS_TYPE_CONSUMABLE)),
        );
        writer.write_i32(base.weight() as i32);
        writer.write_i32(self.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 2));
        writer.write_i32(normal_count as i32);
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
            writer.write_u16(property.property_type as u16);
            writer.write_i32(value1);
            writer.write_i32(modifier(2));
        }
        let visible_da_kong_count = if da_kong_enabled { da_kong_count } else { 0 };
        writer.write_i32(self.shape.get_pos_x() as i32);
        writer.write_i32(self.shape.get_pos_y() as i32);
        writer.write_i32(visible_da_kong_count as i32);
        for property in self
            .addon_properties
            .iter()
            .skip(normal_count)
            .take(visible_da_kong_count)
        {
            writer.write_u16(property.property_type as u16);
            let mut base_value1 = 0;
            let mut base_value2 = 0;
            for value in &property.values {
                if value.id == 1 {
                    base_value1 = value.base_value;
                    writer.write_i32(value.modifier);
                }
                if value.id == 2 {
                    base_value2 = value.base_value;
                    writer.write_i32(value.modifier);
                }
            }
            writer.write_u8(u8::from(property.is_enabled != 0));
            writer.write_u8(u8::from(property.is_implicit_attribute != 0));
            writer.write_i32(base_value1);
            writer.write_i32(base_value2);
            writer.write_u16(10_000);
        }
        true
    }

    /// Exact `CGoods::Unserialize`: release, shape/scalar/string/addon wire и
    /// обе derived fairy-проекции выполняются в исходном порядке.
    pub fn unserialize<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        factory: &impl GoodsBasePropertiesLookup,
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

    pub const fn fairy_properties(&self) -> Option<&CFairyProperties> {
        self.fairy_properties.as_ref()
    }

    pub const fn fairy_properties_mut(&mut self) -> Option<&mut CFairyProperties> {
        self.fairy_properties.as_mut()
    }

    /// Exact `CGoods::HatchBegin`: только готовое яйцо без активного timer-а.
    pub fn hatch_begin(&mut self, current_tick: u32, egg_max_level: u32) -> bool {
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
    pub fn hatch_stop(&mut self) -> bool {
        let Some(fairy) = self.fairy_properties_mut() else {
            return false;
        };
        fairy.hatch_start_time = 0;
        true
    }

    pub const fn battle_fairy_property(&self) -> Option<&CBattleFairyProperty> {
        self.battle_fairy_property.as_ref()
    }

    pub const fn battle_fairy_property_mut(&mut self) -> Option<&mut CBattleFairyProperty> {
        self.battle_fairy_property.as_mut()
    }

    /// Safe adapter legacy interior pointer: временно отделяет property owner,
    /// но возвращает его товару и при typed block-е.
    pub fn fairy_exp_up<Threshold, Effects>(
        &mut self,
        experience: &mut u32,
        runtime: FairyExpRuntime<'_>,
        threshold_for_level: Threshold,
    ) -> Result<Option<FairyExpReport<Effects>>, FairyExpBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
        Effects: FairyGrowEffectSink,
    {
        let Some(mut fairy) = self.fairy_properties.take() else {
            return Ok(None);
        };
        let result = fairy.exp_up(experience, runtime, threshold_for_level);
        self.fairy_properties = Some(fairy);
        result.map(Some)
    }

    /// Safe adapter двойного mutable borrow исходных `property + goods`.
    pub fn battle_fairy_exp_up<Threshold>(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
        player: Option<BattleFairyPlayerFacts>,
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
    pub fn load_fairy_properties<Threshold>(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
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

    pub fn save_fairy_properties(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
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

    pub fn load_battle_fairy_property<Threshold>(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
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
    pub fn save_battle_fairy_property(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
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
    pub fn copy_addon_properties_core_from(&mut self, source: &Self) {
        self.addon_properties.clone_from(&source.addon_properties);
    }

    pub fn copy_fairy_addon_properties_from<Threshold>(
        &mut self,
        source: &Self,
        factory: &impl GoodsBasePropertiesLookup,
        threshold_for_level: Threshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        self.copy_addon_properties_core_from(source);
        self.load_fairy_properties(factory, threshold_for_level)
    }

    pub fn copy_battle_fairy_addon_properties_from<Threshold>(
        &mut self,
        source: &Self,
        factory: &impl GoodsBasePropertiesLookup,
        threshold_for_level: Threshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        self.copy_addon_properties_core_from(source);
        self.load_battle_fairy_property(factory, threshold_for_level)
    }

    pub fn query_attribute(&self, property_type: i32) -> bool {
        self.addon_properties
            .iter()
            .any(|property| property.property_type == property_type)
    }

    /// Exact `HasAddonPropertyValues` смотрит только catalog addon-values и
    /// не проверяет instance storage.
    pub fn has_addon_property_values(
        &self,
        factory: &impl GoodsBasePropertiesLookup,
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
    pub fn can_upgraded(&self, factory: &impl GoodsBasePropertiesLookup) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.query_attribute(GAP_WEAPON_LEVEL)
            })
    }

    /// Exact `CanBFEquipeUpgrade`: catalog type обязан быть equipment, marker
    /// уровня ищется только среди instance-addon-ов, без registry fallback.
    pub fn can_battle_fairy_equipment_upgrade(&self, factory: &impl GoodsBasePropertiesLookup) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.query_attribute(crate::content::goods::GAP_BF_WEAPON_LEVEL)
            })
    }

    /// Exact `CanReparied`: только equipment без particular-attribute bit 0.
    pub fn can_repair(&self, factory: &impl GoodsBasePropertiesLookup) -> bool {
        factory
            .query_goods_base_properties(self.base_properties_index)
            .is_some_and(|properties| {
                properties.goods_type() == GOODS_TYPE_EQUIPMENT
                    && self.addon_property_value(factory, GAP_PARTICULAR_ATTRIBUTE, 1) & 1 == 0
            })
    }

    pub fn repair_durability(&mut self, factory: &impl GoodsBasePropertiesLookup) -> bool {
        if !self.can_repair(factory) {
            return false;
        }
        let maximum = self.addon_property_value(factory, GAP_GOODS_MAXIMUM_DURABILITY, 1);
        self.set_addon_property_value_first_core(GAP_GOODS_MAXIMUM_DURABILITY, 2, maximum)
    }

    /// Exact durability owner читает `base_value`, не сумму с modifier.
    pub fn current_durability(&self) -> i32 {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == GAP_GOODS_MAXIMUM_DURABILITY)
            .and_then(|property| property.values.iter().find(|value| value.id == 2))
            .map_or(-1, |value| value.base_value)
    }

    /// Ноль и отсутствующий value-id `2` возвращают `-1`. Отрицательное
    /// значение, включая `-1`, записывается до возврата как в native owner-е.
    pub fn set_current_durability(&mut self, durability: i32) -> i32 {
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
    pub fn da_kong_count(&self, factory: &impl GoodsBasePropertiesLookup) -> u32 {
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

    pub fn addon_property_value(
        &self,
        factory: &impl GoodsBasePropertiesLookup,
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
    pub fn enabled_addon_properties(&self, factory: &impl GoodsBasePropertiesLookup) -> Vec<i32> {
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
    pub fn set_addon_property_value_core(
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

    pub fn set_addon_property_value<OrdinaryThreshold, BattleThreshold>(
        &mut self,
        property_type: i32,
        value_id: u32,
        value: i32,
        factory: &impl GoodsBasePropertiesLookup,
        ordinary_threshold: OrdinaryThreshold,
        battle_threshold: BattleThreshold,
    ) -> Result<bool, GoodsBasePropertyBlock>
    where
        OrdinaryThreshold: FnMut(u32, u32) -> u32,
        BattleThreshold: FnMut(u32, u32) -> u32,
    {
        if !self.set_addon_property_value_core(property_type, value_id, value) {
            return Ok(false);
        }
        if self.fairy_properties.is_some() {
            self.load_fairy_properties(factory, ordinary_threshold)?;
        }
        if self.battle_fairy_property.is_some() {
            self.load_battle_fairy_property(factory, battle_threshold)?;
        }
        Ok(true)
    }

    /// Ограниченный storage-adapter старых DaKong callers: первая пара
    /// property/value, в отличие от общего setter по всем addon-ам.
    pub fn set_addon_property_value_first_core(
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

    /// Exact `SetAddonPropertyBaseValues`: меняет `base_value` первой пары,
    /// не поглощая её modifier в записанное значение.
    pub fn set_addon_property_base_value_first_core(
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
                    .find(|candidate| candidate.id == value_id)
            })
        else {
            return false;
        };
        found.base_value = value;
        true
    }

    /// Safe replacement pointer-а `lCurrentExp`: первый instance value
    /// возвращает именно modifier, не сумму base+modifier.
    pub fn instance_addon_modifier(&self, property_type: i32, value_id: u32) -> Option<i32> {
        self.addon_properties
            .iter()
            .find(|property| property.property_type == property_type)
            .and_then(|property| property.values.iter().find(|value| value.id == value_id))
            .map(|value| value.modifier)
    }

    pub fn set_instance_addon_modifier(
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
    pub fn set_addon_property_modifier_core(
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
    pub fn set_fu_mo_property(&mut self, property_type: i32, value: i32) -> i32 {
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
    pub fn cut_addon_property_value(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
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

    /// Точный `QueryEnchanseColor`: считает цвет камня из каталога в семи
    /// сохранённых индексах отверстий, включая непоследовательные отверстия.
    pub fn query_enchanse_color(&self, factory: &impl GoodsBasePropertiesLookup, color: i32) -> i32 {
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

    pub fn goods_time_type(&self, factory: &impl GoodsBasePropertiesLookup) -> u32 {
        self.addon_property_value(factory, GAP_GOODS_LIFE_TYPE, 2) as u32
    }

    pub fn goods_lifetime(&self, factory: &impl GoodsBasePropertiesLookup) -> u32 {
        self.addon_property_value(factory, GAP_GOODS_LIFE_TYPE, 1) as u32
    }

    pub fn set_goods_lifetime(&mut self, lifetime: u32) {
        let _ = self.set_addon_property_value_core(GAP_GOODS_LIFE_TYPE, 1, lifetime as i32);
    }

    pub fn set_goods_time_type(&mut self, time_type: u32) {
        let _ = self.set_addon_property_value_core(GAP_GOODS_LIFE_TYPE, 2, time_type as i32);
    }

    pub fn start_point(&self, factory: &impl GoodsBasePropertiesLookup) -> u64 {
        let high = self.addon_property_value(factory, GAP_GOODS_START_POINT, 1) as u32;
        let low = self.addon_property_value(factory, GAP_GOODS_START_POINT, 2) as u32;
        (u64::from(high) << 32) | u64::from(low)
    }

    /// Исходный метод последовательно пишет младшую, затем старшую часть. При
    /// повреждённой схеме дополнительных свойств первая запись может
    /// состояться без второй; это намеренно не сворачивается в атомарную
    /// замену.
    pub fn set_start_point(&mut self, start_point: u64) {
        let _ =
            self.set_addon_property_value_core(GAP_GOODS_START_POINT, 2, start_point as u32 as i32);
        let _ = self.set_addon_property_value_core(
            GAP_GOODS_START_POINT,
            1,
            (start_point >> 32) as u32 as i32,
        );
    }

    /// Начальная ветвь `CEquipmentContainer::Add` для временных предметов:
    /// типы 2 и 4 получают текущую точку только при нулевом старте. Возвращает
    /// факт попытки исходного метода записи.
    pub fn initialize_equipment_start_point(
        &mut self,
        factory: &impl GoodsBasePropertiesLookup,
        now: u64,
    ) -> bool {
        if matches!(self.goods_time_type(factory), 2 | 4) && self.start_point(factory) == 0 {
            self.set_start_point(now);
            true
        } else {
            false
        }
    }

    pub fn can_stack(
        &self,
        factory: &impl GoodsBasePropertiesLookup,
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

    pub fn max_stack_number(&self, factory: &impl GoodsBasePropertiesLookup) -> u32 {
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

    pub fn weight(&self, factory: &impl GoodsBasePropertiesLookup) -> u32 {
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
    let mut reader = LegacyReader::at(source, offset).map_err(|block| {
        GoodsDecodeError::UnexpectedEnd {
            field: "m_strDescribe",
            offset: block.offset,
            needed: 1,
            available: block.available,
        }
    })?;
    let bytes = reader
        .read_c_string(0x404)
        .map_err(|_| GoodsDecodeError::UnterminatedDescription { offset, available })?;
    *cursor = reader.position();
    Ok(bytes.to_vec())
}

fn read_goods_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GoodsDecodeError> {
    let mut reader = goods_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| goods_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_goods_wire_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, GoodsDecodeError> {
    let mut reader = goods_reader(source, *cursor, field, 4)?;
    let value = reader.read_u32().map_err(|block| goods_read_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn goods_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, GoodsDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| GoodsDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed,
        available: block.available,
    })
}

fn goods_read_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> GoodsDecodeError {
    GoodsDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
