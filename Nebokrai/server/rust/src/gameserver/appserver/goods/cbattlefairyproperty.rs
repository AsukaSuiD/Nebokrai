//! Свойства и конфигурация battle fairy из точных `gameserver.exe +
//! GameServer.pdb`; исходный owner
//! `gameserver/appserver/goods/cbattlefairyproperty.h/.cpp`.
//!
//! Достигнут selector `0x2D`: signed count и raw 0x7C MSVC records. Парный
//! World serializer допускает только SSO-строки до 15 байт, потому что heap
//! pointer другого процесса непереносим. Decoder сохраняет это ограничение,
//! очищает список до count и публикует только полные records. Constructor и
//! signed modifier-based `ExpUp/LevelUp` также материализованы; player lookup,
//! level-log и обязательный old-client update выражены typed facts/effects.
//! Process-global lazy singleton заменён обычным explicit owner-ом; пустой
//! `vecUpLevelReleated` сохранён как owned vector. Поля, которые exact
//! constructor не инициализировал и эти методы не читают, намеренно не
//! получают выдуманных defaults из позднего донора.

use std::error::Error;
use std::fmt;

use super::cgoods::CGoods;
use super::cgoodsbaseproperties::{
    GAP_BF_AGILITY, GAP_BF_AGILITY_BASE, GAP_BF_ATTACK, GAP_BF_ATTACK_BASE, GAP_BF_BRAVE,
    GAP_BF_BRAVE_BASE, GAP_BF_CURRENT_EXP, GAP_BF_CURRENT_MAX_EXP, GAP_BF_HP, GAP_BF_LEVEL,
    GAP_BF_MAX_HP, GAP_BF_MAX_LEVEL, GAP_BF_MAX_MP, GAP_BF_MP, GAP_BF_PULLULATERATE, GAP_BF_SPRITE,
    GAP_BF_SPRITE_BASE, GAP_BF_SPRITUALISM, GAP_BF_SPRITUALISM_BASE, GAP_BF_STRENGH,
    GAP_BF_STRENGH_BASE, GAP_ROLE_MINIMUM_LEVEL_LIMIT,
};
use super::cgoodsfactory::CGoodsFactory;
use crate::gameserver::appserver::shape::ShapeIdentity;

const COMPOSE_RECORD_SIZE: usize = 0x7c;
const LEGACY_STRING_SIZE: usize = 0x1c;
const LEGACY_STRING_INLINE_CAPACITY: u32 = 15;
pub(crate) const BATTLE_FAIRY_GOODS_UPDATE_MESSAGE_TYPE: u32 = 0x0b_f918;

#[derive(Clone, Debug, Default)]
pub(crate) struct BattleFairyCompose {
    pub(crate) fetch_stone: Vec<u8>,
    pub(crate) fetch_body: Vec<u8>,
    pub(crate) material: Vec<u8>,
    pub(crate) deplete_fetch: u32,
    pub(crate) success_rate: f32,
    pub(crate) battle_fairy: Vec<u8>,
    pub(crate) index: u32,
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
pub(crate) struct CBattleFairyProperty {
    compose: Vec<BattleFairyCompose>,
    up_level_related: Vec<()>,
    current_exp_linked: bool,
    pub(crate) max_exp: u32,
    pub(crate) equip_level: u32,
    pub(crate) current_level: u32,
    pub(crate) max_level: u32,
    pub(crate) change_mode_level: u32,
    pub(crate) life: u32,
    pub(crate) mp: u32,
    pub(crate) attack: u32,
    pub(crate) sprit: u32,
    pub(crate) blast: u32,
    pub(crate) brave: u32,
    pub(crate) agility: u32,
    pub(crate) spritualism: u32,
    pub(crate) strength: u32,
    pub(crate) usual_id: u32,
    pub(crate) change_id: u32,
    pub(crate) potential: u32,
    pub(crate) status: i32,
    pullulate_rate_bits: u32,
    pub(crate) module: Option<u32>,
    pub(crate) agility_base: Option<u32>,
    pub(crate) spritualism_base: Option<u32>,
    pub(crate) strength_base: Option<u32>,
    pub(crate) immunity: Option<u32>,
    pub(crate) attack_potential: u32,
    pub(crate) blast_potential: u32,
    pub(crate) strength_potential: u32,
    pub(crate) spritualism_potential: u32,
    pub(crate) brave_potential: u32,
    pub(crate) sprit_potential: u32,
    pub(crate) agility_potential: u32,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum BattleFairyExpUpResult {
    #[default]
    None = 0,
    ExpUp = 1,
    LevelUp = 2,
    ChangeState = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyExpBlock {
    CurrentExperienceUnlinked,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BattleFairyPlayerFacts<'a> {
    pub(crate) player_id: i32,
    pub(crate) account: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyLevelLog {
    pub(crate) player_id: i32,
    pub(crate) account: Vec<u8>,
    pub(crate) goods_name: Vec<u8>,
    pub(crate) level: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyGoodsUpdate {
    pub(crate) player_id: i32,
    pub(crate) goods: ShapeIdentity,
}

#[must_use = "report содержит level logs и обязательный old-client update"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyExpReport {
    pub(crate) result: BattleFairyExpUpResult,
    pub(crate) remaining_experience: u32,
    pub(crate) level_logs: Vec<BattleFairyLevelLog>,
    pub(crate) goods_update: Option<BattleFairyGoodsUpdate>,
}

impl CBattleFairyProperty {
    pub(crate) fn compose(&self) -> &[BattleFairyCompose] {
        &self.compose
    }

    /// Safe binding legacy `lCurrentExp` к первому instance modifier.
    pub(crate) fn link_from_goods(&mut self, goods: &CGoods, factory: &CGoodsFactory) {
        self.current_exp_linked = goods
            .instance_addon_modifier(GAP_BF_CURRENT_EXP, 1)
            .is_some();
        self.equip_level =
            goods.addon_property_value(factory, GAP_ROLE_MINIMUM_LEVEL_LIMIT, 1) as u32;
        self.current_level = goods.addon_property_value(factory, GAP_BF_LEVEL, 1) as u32;
        self.max_level = goods.addon_property_value(factory, GAP_BF_MAX_LEVEL, 1) as u32;
        self.max_exp = goods.addon_property_value(factory, GAP_BF_CURRENT_MAX_EXP, 1) as u32;
    }

    pub(crate) const fn unlink_current_experience(&mut self) {
        self.current_exp_linked = false;
    }

    pub(crate) const fn set_current_experience_linked(&mut self, linked: bool) {
        self.current_exp_linked = linked;
    }

    pub(crate) const fn pullulate_rate(&self) -> f32 {
        f32::from_bits(self.pullulate_rate_bits)
    }

    pub(crate) const fn set_pullulate_rate(&mut self, rate: f32) {
        self.pullulate_rate_bits = rate.to_bits();
    }

    pub(crate) fn exp_up<Threshold>(
        &mut self,
        goods: Option<&mut CGoods>,
        factory: &CGoodsFactory,
        player: Option<BattleFairyPlayerFacts<'_>>,
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
                level_logs: Vec::new(),
                goods_update: None,
            });
        };
        let Some(goods) = goods else {
            return Ok(BattleFairyExpReport {
                result: BattleFairyExpUpResult::None,
                remaining_experience: *experience,
                level_logs: Vec::new(),
                goods_update: None,
            });
        };

        let mut result = BattleFairyExpUpResult::None;
        let mut level_logs = Vec::new();
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
            level_logs.push(BattleFairyLevelLog {
                player_id: player.player_id,
                account: visible_c_bytes(player.account),
                goods_name: goods.name().to_vec(),
                level: goods.addon_property_value(factory, GAP_BF_LEVEL, 1),
            });
            result = result.max(level_result);
            let _ = goods.set_addon_property_value_core(GAP_BF_CURRENT_EXP, 1, 0);
        }

        Ok(BattleFairyExpReport {
            result,
            remaining_experience: *experience,
            level_logs,
            goods_update: Some(BattleFairyGoodsUpdate {
                player_id: player.player_id,
                goods: goods.identity(),
            }),
        })
    }

    fn level_up<Threshold>(
        &mut self,
        goods: &mut CGoods,
        factory: &CGoodsFactory,
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
            let value = goods.addon_property_value(factory, value_property, 1) as f32;
            let grown = (value + base * pullulate_rate).round() as i32;
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

    pub(crate) fn decord_byte_array_combine(
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
                deplete_fetch: u32::from_le_bytes(
                    record[0x54..0x58].try_into().expect("поле 4 байта"),
                ),
                success_rate: f32::from_bits(u32::from_le_bytes(
                    record[0x58..0x5c].try_into().expect("поле 4 байта"),
                )),
                battle_fairy: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strBattleFairy",
                    0x5c,
                )?,
                index: u32::from_le_bytes(record[0x78..0x7c].try_into().expect("поле 4 байта")),
            });
        }
        Ok(self.compose.len())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyComposeDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    NonPortableString {
        record: usize,
        field: &'static str,
        length: u32,
        capacity: u32,
    },
}

impl fmt::Display for BattleFairyComposeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "BattleFairy combine snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::NonPortableString {
                record,
                field,
                length,
                capacity,
            } => write!(
                formatter,
                "BattleFairy combine record {record}, {field}: непереносимый MSVC string length {length}, capacity {capacity}"
            ),
        }
    }
}

impl Error for BattleFairyComposeDecodeError {}

fn decode_legacy_string(
    record: &[u8; COMPOSE_RECORD_SIZE],
    record_index: usize,
    field: &'static str,
    offset: usize,
) -> Result<Vec<u8>, BattleFairyComposeDecodeError> {
    let length = u32::from_le_bytes(
        record[offset + 0x14..offset + 0x18]
            .try_into()
            .expect("MSVC string length содержит четыре байта"),
    );
    let capacity = u32::from_le_bytes(
        record[offset + 0x18..offset + LEGACY_STRING_SIZE]
            .try_into()
            .expect("MSVC string capacity содержит четыре байта"),
    );
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
    Ok(i32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], BattleFairyComposeDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(BattleFairyComposeDecodeError::UnexpectedEnd {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("размер BattleFairy combine record уже проверен"))
}

fn visible_c_bytes(bytes: &[u8]) -> Vec<u8> {
    bytes[..bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())]
        .to_vec()
}
