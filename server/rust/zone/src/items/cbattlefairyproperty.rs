//! Свойства и конфигурация battle fairy исторического GameServer,
//! перенесённые в Zone `items/` — владельца ядра товаров.
//!
//! Тела перенесены буквально из прежнего
//! `src/gameserver/appserver/goods/cbattlefairyproperty.rs`; отличия —
//! нормализация `pub(crate)`→`pub` на границе crate и объявленные швы
//! переноса (не расхождения): GAP-константы — Zone `content/goods.rs`,
//! `ShapeIdentity` — Zone `regions`, lookup реестра базовых свойств —
//! trait-шов [`GoodsBasePropertiesLookup`] из `cgoods.rs` (реализация
//! прежнего владельца — `CGoodsFactory` старого пакета до его собственной
//! волны). Невостребованная приватная константа `LEGACY_STRING_SIZE`
//! (мёртвая уже в старом пакете, где dead_code скрыт blanket-allow)
//! не перенесена, чтобы Zone сохранял нуль предупреждений.
//!
//! Из точных `gameserver.exe + GameServer.pdb`; исходный owner
//! `gameserver/appserver/goods/cbattlefairyproperty.h/.cpp`.
//!
//! Достигнут selector `0x2D`: signed count и raw 0x7C MSVC records. Парный
//! World serializer допускает только SSO-строки до 15 байт, потому что heap
//! pointer другого процесса непереносим. Decoder сохраняет это ограничение,
//! очищает список до count и публикует только полные records. Constructor и
//! signed modifier-based `ExpUp/LevelUp` также материализованы; player lookup
//! и обязательный old-client update выражены типизированными facts/effects,
//! а диагностические сведения уровня публикуются в месте изменения. При
//! `LevelUp` произведение base на pullulate rate округляется до `f32`, после
//! чего полная сумма с текущим целым усекается FISTP к нулю.
//! Process-global lazy singleton заменён обычным explicit owner-ом; пустой
//! `vecUpLevelReleated` сохранён как owned vector. Поля, которые exact
//! constructor не инициализировал и эти методы не читают, намеренно не
//! получают выдуманных defaults из позднего донора.

use thiserror::Error;

use super::cgoods::{CGoods, GoodsBasePropertiesLookup};
use crate::content::goods::{
    GAP_BF_AGILITY, GAP_BF_AGILITY_BASE, GAP_BF_ATTACK, GAP_BF_ATTACK_BASE, GAP_BF_BRAVE,
    GAP_BF_BRAVE_BASE, GAP_BF_CURRENT_EXP, GAP_BF_CURRENT_MAX_EXP, GAP_BF_HP, GAP_BF_LEVEL,
    GAP_BF_MAX_HP, GAP_BF_MAX_LEVEL, GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_PULLULATERATE, GAP_BF_SPRITE,
    GAP_BF_SPRITE_BASE, GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE, GAP_BF_STRENGH,
    GAP_BF_STRENGH_BASE, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
};
use crate::regions::ShapeIdentity;
use nebokrai_shared::protocol::LegacyReader;

const COMPOSE_RECORD_SIZE: usize = 0x7c;
const LEGACY_STRING_INLINE_CAPACITY: u32 = 15;
pub const BATTLE_FAIRY_GOODS_UPDATE_MESSAGE_TYPE: u32 = 0x0b_f918;

#[derive(Clone, Debug, Default)]
pub struct BattleFairyCompose {
    pub fetch_stone: Vec<u8>,
    pub fetch_body: Vec<u8>,
    pub material: Vec<u8>,
    pub deplete_fetch: u32,
    pub success_rate: f32,
    pub battle_fairy: Vec<u8>,
    pub index: u32,
}

impl PartialEq for BattleFairyCompose {
    fn eq(&self, other: &Self) -> bool {
        self.fetch_stone == other.fetch_stone
            && self.fetch_body == other.fetch_body
            && self.material == other.material
            && self.deplete_fetch == other.deplete_fetch
            && self.success_rate.to_bits() == other.success_rate.to_bits()
            && self.battle_fairy == other.battle_fairy
            && self.index == other.index
    }
}

impl Eq for BattleFairyCompose {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CBattleFairyProperty {
    compose: Vec<BattleFairyCompose>,
    up_level_related: Vec<()>,
    current_exp_linked: bool,
    pub max_exp: u32,
    pub equip_level: u32,
    pub current_level: u32,
    pub max_level: u32,
    pub change_mode_level: u32,
    pub life: u32,
    pub mp: u32,
    pub attack: u32,
    pub sprit: u32,
    pub blast: u32,
    pub brave: u32,
    pub agility: u32,
    pub spritualism: u32,
    pub strength: u32,
    pub usual_id: u32,
    pub change_id: u32,
    pub potential: u32,
    pub status: i32,
    pullulate_rate_bits: u32,
    pub module: Option<u32>,
    pub agility_base: Option<u32>,
    pub spritualism_base: Option<u32>,
    pub strength_base: Option<u32>,
    pub immunity: Option<u32>,
    pub attack_potential: u32,
    pub blast_potential: u32,
    pub strength_potential: u32,
    pub spritualism_potential: u32,
    pub brave_potential: u32,
    pub sprit_potential: u32,
    pub agility_potential: u32,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum BattleFairyExpUpResult {
    #[default]
    None = 0,
    ExpUp = 1,
    LevelUp = 2,
    ChangeState = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyExpBlock {
    CurrentExperienceUnlinked,
}

#[derive(Clone, Copy, Debug)]
pub struct BattleFairyPlayerFacts {
    pub player_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleFairyGoodsUpdate {
    pub player_id: i32,
    pub goods: ShapeIdentity,
}

#[must_use = "результат содержит остаток опыта и обязательный old-client update"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleFairyExpReport {
    pub result: BattleFairyExpUpResult,
    pub remaining_experience: u32,
    pub goods_update: Option<BattleFairyGoodsUpdate>,
}

impl CBattleFairyProperty {
    pub fn compose(&self) -> &[BattleFairyCompose] {
        &self.compose
    }

    /// Safe binding legacy `lCurrentExp` к первому instance modifier.
    pub fn link_from_goods(&mut self, goods: &CGoods, factory: &impl GoodsBasePropertiesLookup) {
        self.current_exp_linked = goods
            .instance_addon_modifier(GAP_BF_CURRENT_EXP, 1)
            .is_some();
        self.equip_level =
            goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1) as u32;
        self.current_level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1) as u32;
        self.max_level = goods.addon_property_value(factory, GAP_BF_MAX_LEVEL, 1) as u32;
        self.max_exp = goods.addon_property_value(factory, GAP_BF_CURRENT_MAX_EXP, 1) as u32;
    }

    pub const fn unlink_current_experience(&mut self) {
        self.current_exp_linked = false;
    }

    pub const fn set_current_experience_linked(&mut self, linked: bool) {
        self.current_exp_linked = linked;
    }

    pub const fn pullulate_rate(&self) -> f32 {
        f32::from_bits(self.pullulate_rate_bits)
    }

    pub const fn set_pullulate_rate(&mut self, rate: f32) {
        self.pullulate_rate_bits = rate.to_bits();
    }

    pub fn exp_up<Threshold>(
        &mut self,
        goods: Option<&mut CGoods>,
        factory: &impl GoodsBasePropertiesLookup,
        player: Option<BattleFairyPlayerFacts>,
        experience: &mut u32,
        mut threshold_for_level: Threshold,
    ) -> Result<BattleFairyExpReport, BattleFairyExpBlock>
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let Some(player) = player else {
            return Ok(BattleFairyExpReport {
                result: BattleFairyExpUpResult::None,
                remaining_experience: *experience,
                goods_update: None,
            });
        };
        let Some(goods) = goods else {
            return Ok(BattleFairyExpReport {
                result: BattleFairyExpUpResult::None,
                remaining_experience: *experience,
                goods_update: None,
            });
        };

        let mut result = BattleFairyExpUpResult::None;
        while *experience != 0 {
            let max_level = goods.addon_property_value(factory, GAP_BF_MAX_LEVEL, 1);
            let current_level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1);
            if max_level <= current_level {
                break;
            }
            if !self.current_exp_linked {
                return Err(BattleFairyExpBlock::CurrentExperienceUnlinked);
            }
            let current_exp = goods
                .instance_addon_modifier(GAP_BF_CURRENT_EXP, 1)
                .ok_or(BattleFairyExpBlock::CurrentExperienceUnlinked)?;
            let maximum_exp = goods.addon_property_value(factory, GAP_BF_CURRENT_MAX_EXP, 1);
            let accumulated = current_exp.wrapping_add(*experience as i32);
            let _ = goods.set_instance_addon_modifier(GAP_BF_CURRENT_EXP, 1, accumulated);
            if accumulated < maximum_exp {
                *experience = 0;
                result = result.max(BattleFairyExpUpResult::ExpUp);
                break;
            }

            *experience = accumulated.wrapping_sub(maximum_exp) as u32;
            let level_result = self.level_up(goods, factory, &mut threshold_for_level);
            tracing::trace!(
                player_id = player.player_id,
                goods_id = ?goods.identity().ex_id,
                level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1),
                "уровень боевой феи повышен"
            );
            result = result.max(level_result);
            let _ = goods.set_addon_property_value_core(GAP_BF_CURRENT_EXP, 1, 0);
        }

        Ok(BattleFairyExpReport {
            result,
            remaining_experience: *experience,
            goods_update: Some(BattleFairyGoodsUpdate {
                player_id: player.player_id,
                goods: goods.identity(),
            }),
        })
    }

    fn level_up<Threshold>(
        &mut self,
        goods: &mut CGoods,
        factory: &impl GoodsBasePropertiesLookup,
        threshold_for_level: &mut Threshold,
    ) -> BattleFairyExpUpResult
    where
        Threshold: FnMut(u32, u32) -> u32,
    {
        let level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_LEVEL, 1, level.wrapping_add(1));
        let max_level = goods.addon_property_value(factory, GAP_BF_MAX_LEVEL, 1) as u32;
        let current_level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1) as u32;
        self.max_exp = threshold_for_level(max_level, current_level);

        let loaded_max_exp = threshold_for_level(self.equip_level, current_level);
        let _ = goods.set_instance_addon_modifier(GAP_BF_CURRENT_MAX_EXP, 1, loaded_max_exp as i32);
        self.max_exp = goods.addon_property_value(factory, GAP_BF_CURRENT_MAX_EXP, 1) as u32;
        let pullulate_rate =
            goods.addon_property_value(factory, GAP_BF_PULLULATERATE, 1) as f32 * 0.0001_f32;
        let _ = goods.set_addon_property_value_core(GAP_BF_CURRENT_EXP, 1, 0);

        for (value_property, base_property) in [
            (GAP_BF_BRAVE, GAP_BF_BRAVE_BASE),
            (GAP_BF_AGILITY, GAP_BF_AGILITY_BASE),
            (GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE),
            (GAP_BF_STRENGH, GAP_BF_STRENGH_BASE),
            (GAP_BF_SPRITE, GAP_BF_SPRITE_BASE),
            (GAP_BF_ATTACK, GAP_BF_ATTACK_BASE),
        ] {
            let base = goods.addon_property_value(factory, base_property, 1) as f32;
            let growth = base * pullulate_rate;
            let value = goods.addon_property_value(factory, value_property, 1);
            let grown = (f64::from(value) + f64::from(growth)).trunc() as i32;
            let _ = goods.set_addon_property_value_core(value_property, 1, grown);
        }

        let strength = goods.addon_property_value(factory, GAP_BF_STRENGH, 1);
        let spiritualism = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_HP, 1, strength);
        let _ = goods.set_addon_property_value_core(GAP_BF_MAX_MP, 1, spiritualism);
        let maximum_hp = goods.addon_property_value(factory, GAP_BF_MAX_HP, 1);
        let maximum_mp = goods.addon_property_value(factory, GAP_BF_MAX_MP, 1);
        let _ = goods.set_addon_property_value_core(GAP_BF_HP, 1, maximum_hp);
        let _ = goods.set_addon_property_value_core(GAP_BF_MP, 1, maximum_mp);
        self.current_level = current_level;
        self.brave = goods.addon_property_value(factory, GAP_BF_BRAVE, 1) as u32;
        self.agility = goods.addon_property_value(factory, GAP_BF_AGILITY, 1) as u32;
        self.spritualism = goods.addon_property_value(factory, GAP_BF_SPRITUALISM, 1) as u32;
        self.strength = goods.addon_property_value(factory, GAP_BF_STRENGH, 1) as u32;
        self.pullulate_rate_bits =
            (goods.addon_property_value(factory, GAP_BF_PULLULATERATE, 1) as f32).to_bits();
        BattleFairyExpUpResult::LevelUp
    }

    pub fn decord_byte_array_combine(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, BattleFairyComposeDecodeError> {
        self.compose.clear();
        let count = read_wire_i32(source, cursor)?;
        for record_index in 0..count.max(0) {
            let record = read_wire_array::<COMPOSE_RECORD_SIZE>(source, cursor)?;
            self.compose.push(BattleFairyCompose {
                fetch_stone: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strFetchStone",
                    0x00,
                )?,
                fetch_body: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strFetchBody",
                    0x1c,
                )?,
                material: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strMaterial",
                    0x38,
                )?,
                deplete_fetch: read_record_u32(&record, 0x54),
                success_rate: f32::from_bits(read_record_u32(&record, 0x58)),
                battle_fairy: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strBattleFairy",
                    0x5c,
                )?,
                index: read_record_u32(&record, 0x78),
            });
        }
        Ok(self.compose.len())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum BattleFairyComposeDecodeError {
    #[error("BattleFairy combine snapshot обрывается на {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("BattleFairy combine record {record}, {field}: непереносимый MSVC string length {length}, capacity {capacity}")]
    NonPortableString {
        record: usize,
        field: &'static str,
        length: u32,
        capacity: u32,
    },
}

fn decode_legacy_string(
    record: &[u8; COMPOSE_RECORD_SIZE],
    record_index: usize,
    field: &'static str,
    offset: usize,
) -> Result<Vec<u8>, BattleFairyComposeDecodeError> {
    let length = read_record_u32(record, offset + 0x14);
    let capacity = read_record_u32(record, offset + 0x18);
    if capacity > LEGACY_STRING_INLINE_CAPACITY || length > LEGACY_STRING_INLINE_CAPACITY {
        return Err(BattleFairyComposeDecodeError::NonPortableString {
            record: record_index,
            field,
            length,
            capacity,
        });
    }
    let length = length as usize;
    Ok(record[offset + 4..offset + 4 + length].to_vec())
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, BattleFairyComposeDecodeError> {
    let mut reader = battle_fairy_reader(source, *cursor, 4)?;
    let value = reader.read_i32().map_err(battle_fairy_read_error)?;
    *cursor = reader.position();
    Ok(value)
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], BattleFairyComposeDecodeError> {
    let mut reader = battle_fairy_reader(source, *cursor, N)?;
    let bytes = reader.read_bytes(N).map_err(battle_fairy_read_error)?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn battle_fairy_reader(
    source: &[u8],
    cursor: usize,
    needed: usize,
) -> Result<LegacyReader<'_>, BattleFairyComposeDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| {
        BattleFairyComposeDecodeError::UnexpectedEnd {
            offset: block.offset,
            needed,
            available: block.available,
        }
    })
}

fn battle_fairy_read_error(
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> BattleFairyComposeDecodeError {
    BattleFairyComposeDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}


fn read_record_u32(record: &[u8], offset: usize) -> u32 {
    LegacyReader::at(record, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("фиксированная запись содержит поле DWORD")
}
