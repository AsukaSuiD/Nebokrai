//! DB snapshots goods-домена, извлечённые из мирового `dbgoods`.
//! Источник контракта — точная пара `Nworldserver.exe` и `WorldServer.pdb`.
//!
//! Типы — чистые данные обмена между codec товара и DB-владельцами:
//! значение addon-свойства, блок числа значений, снимок property, вариант
//! набора свойств и снимок товара. Поля `GoodsAddonPropertySnapshot` открыты:
//! после извлечения прямой доступ legacy save-логики прежнего модуля
//! стал межкрейтовым.

use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug)]
pub struct GoodsAddonPropertyValue {
    pub id: u32,
    pub base_value: i32,
    pub modifier: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct GoodsAddonValueCountBlock {
    pub value_count: usize,
}

pub struct GoodsAddonPropertySnapshot {
    pub property_type: u32,
    pub occur_probability: u32,
    pub values: Vec<GoodsAddonPropertyValue>,
}

impl GoodsAddonPropertySnapshot {
 /// Создаёт только диапазон, в котором исходный `unsigned char` loop
 /// действительно достигал конца vector.
    pub fn from_legacy_parts(
        property_type: u32,
        occur_probability: u32,
        values: Vec<GoodsAddonPropertyValue>,
    ) -> Result<Self, GoodsAddonValueCountBlock> {
        if values.len() > u8::MAX as usize {
            return Err(GoodsAddonValueCountBlock {
                value_count: values.len(),
            });
        }
        Ok(Self {
            property_type,
            occur_probability,
            values,
        })
    }

 /// Возвращает исходные части property для другого DB-owner-а. Порядок
 /// values и все три 32-битных поля остаются исходным `tagAddonProperty`.
    pub fn legacy_parts(&self) -> (u32, u32, &[GoodsAddonPropertyValue]) {
        (self.property_type, self.occur_probability, &self.values)
    }
}

pub enum GoodsPropertiesSnapshot {
    Available(Vec<GoodsAddonPropertySnapshot>),
    MissingBaseProperties,
}

pub struct GoodsObjectSnapshot {
    pub goods_id: CGuid,
    pub base_properties_index: u32,
    pub name: Vec<u8>,
    pub price: u32,
    pub amount: u32,
    pub properties: GoodsPropertiesSnapshot,
}
