//! Правила вставки больших отверстий `CDaKongXiangQian` World/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `public/dakongxiangqian.h/.cpp`.
//!
//! `GetAddType` не зависит от загруженного `delux_modify`: EXE вставляет
//! фиксированный набор типов дополнительных свойств в переданное множество и
//! всегда возвращает `true`.
//! Владелец читает размеченный `data/dakongxiangqian.ini`: основной вектор и
//! три упорядоченные карты атрибутов очищаются до открытия, а
//! `DaKongDeluxModify` сохраняется и дополняется отдельным необязательным
//! ресурсом. `BTreeMap` заменяет только устройство карты MSVC, сохраняя порядок
//! в формате `0x2B`; разбор намеренно принимает частичный текст, как исходный
//! владелец форматированного потока. Декодер GameServer также очищает основное
//! состояние, но дописывает вектор особых модификаторов. Игровые запросы и RNG
//! используют общий генератор MSVCRT GameServer через тонкий адаптер, сохраняя
//! включённую верхнюю границу вероятности и особенности взвешенного выбора.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DaKongInfo {
    pub(crate) probability: i32,
    pub(crate) red: i32,
    pub(crate) green: i32,
    pub(crate) blue: i32,
    pub(crate) yellow: i32,
    pub(crate) cyan: i32,
    pub(crate) purple: i32,
    pub(crate) delux: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DaKongExternalAttribute {
    pub(crate) property_type: i32,
    pub(crate) probability: i32,
    pub(crate) minimum: i32,
    pub(crate) maximum: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DaKongDeluxModify {
    pub(crate) property_type: i32,
    pub(crate) add_type: i32,
}

type ExternalAttributeMap = BTreeMap<u32, Vec<DaKongExternalAttribute>>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CDaKongXiangQian {
    info: Vec<DaKongInfo>,
    external_attributes: [ExternalAttributeMap; 3],
    delux_modify: Vec<DaKongDeluxModify>,
    key: bool,
    log_key: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DaKongLoadReport {
    pub(crate) delux_modify_missing: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DaKongLoadError {
    MissingMainFile,
}

impl DaKongLoadError {
    pub(crate) const fn log_payload(self) -> &'static [u8] {
        match self {
            Self::MissingMainFile => b"error:file DaKongXiangQian.ini is not exist:",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DaKongSerializeError {
    CountOverflow,
}

impl CDaKongXiangQian {
    /// Статический World owner `GetAddType` ( ).
    ///
    /// Сохраняет содержимое переданного set и добавляет точно те 34 raw enum
    /// значения, которые EXE передаёт в `std::set::insert`; возвращаемый
    /// `true` является частью исходной сигнатуры, хотя caller его не
    /// использует.
    pub(crate) fn get_add_type(destination: &mut BTreeSet<i32>) -> bool {
        const ADDON_TYPES: [i32; 34] = [
            0x83, 0x82, 0x81, 0x80, 0x78, 0x77, 0x76, 0x75, 0x69, 0x61, 0x60, 0x5F, 0x5D, 0x5C,
            0x5B, 0x34, 0x33, 0x20, 0x1F, 0x1E, 0x1D, 0x1C, 0x1B, 0x1A, 0x19, 0x17, 0x15, 0x14,
            0x13, 0x12, 0x11, 0x10, 0x0F, 0x0E,
        ];

        destination.extend(ADDON_TYPES);
        true
    }

    /// Исходный переход перед открытием сохраняет особые модификаторы.
    pub(crate) fn clear_primary_state(&mut self) {
        self.info.clear();
        for attributes in &mut self.external_attributes {
            attributes.clear();
        }
    }

    /// Заменяет внутренние вызовы `rfOpen` владельца переданными ресурсами.
    pub(crate) fn load_from_resources(
        &mut self,
        main: Option<&[u8]>,
        delux_modify: Option<&[u8]>,
    ) -> Result<DaKongLoadReport, DaKongLoadError> {
        self.clear_primary_state();
        let Some(main) = main else {
            return Err(DaKongLoadError::MissingMainFile);
        };

        self.load_main(main);
        let mut report = DaKongLoadReport::default();
        if let Some(delux_modify) = delux_modify {
            self.load_delux_modify(delux_modify);
        } else {
            report.delux_modify_missing = true;
        }
        Ok(report)
    }

    fn load_main(&mut self, source: &[u8]) {
        let mut parser = MarkedTextParser::new(source);
        let Some(info_count) = parser.marked_i32() else {
            return;
        };
        for _ in 0..positive_count(info_count) {
            let Some(values) = parser.marked_i32s::<8>() else {
                return;
            };
            self.info.push(DaKongInfo {
                probability: values[0],
                red: values[1],
                green: values[2],
                blue: values[3],
                yellow: values[4],
                cyan: values[5],
                purple: values[6],
                delux: values[7],
            });
        }

        let Some(group_count) = parser.marked_i32() else {
            return;
        };
        for group in 0..positive_count(group_count) {
            let Some(goods_count) = parser.marked_i32() else {
                return;
            };
            for _ in 0..positive_count(goods_count) {
                let Some(goods_id) = parser.marked_i32() else {
                    return;
                };
                let Some(attribute_count) = parser.marked_i32() else {
                    return;
                };
                for _ in 0..positive_count(attribute_count) {
                    let Some(values) = parser.marked_i32s::<4>() else {
                        return;
                    };
                    if let Some(attributes) = self.external_attributes.get_mut(group) {
                        attributes.entry(goods_id as u32).or_default().push(
                            DaKongExternalAttribute {
                                property_type: values[0],
                                probability: values[1],
                                minimum: values[2],
                                maximum: values[3],
                            },
                        );
                    }
                }
            }
        }
    }

    fn load_delux_modify(&mut self, source: &[u8]) {
        let mut parser = WhitespaceParser::new(source);
        let Some(count) = parser.i32() else {
            return;
        };
        for _ in 0..positive_count(count) {
            let (Some(property_type), Some(_property_name), Some(add_type), Some(_add_name)) =
                (parser.i32(), parser.token(), parser.i32(), parser.token())
            else {
                return;
            };
            self.delux_modify.push(DaKongDeluxModify {
                property_type,
                add_type,
            });
        }
    }

    /// Wire: info, map1, map2, map3, then the accumulated deluxe vector.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), DaKongSerializeError> {
        write_count(destination, self.info.len())?;
        for info in &self.info {
            for value in [
                info.probability,
                info.red,
                info.green,
                info.blue,
                info.yellow,
                info.cyan,
                info.purple,
                info.delux,
            ] {
                destination.extend_from_slice(&value.to_le_bytes());
            }
        }
        for attributes in &self.external_attributes {
            write_count(destination, attributes.len())?;
            for (&goods_id, values) in attributes {
                destination.extend_from_slice(&(goods_id as i32).to_le_bytes());
                write_count(destination, values.len())?;
                for value in values {
                    for field in [
                        value.property_type,
                        value.probability,
                        value.minimum,
                        value.maximum,
                    ] {
                        destination.extend_from_slice(&field.to_le_bytes());
                    }
                }
            }
        }
        write_count(destination, self.delux_modify.len())?;
        for value in &self.delux_modify {
            destination.extend_from_slice(&value.property_type.to_le_bytes());
            destination.extend_from_slice(&value.add_type.to_le_bytes());
        }
        Ok(())
    }

    pub(crate) fn set_key(&mut self, enabled: bool) -> bool {
        self.key = enabled;
        enabled
    }

    pub(crate) const fn key(&self) -> bool {
        self.key
    }

    pub(crate) fn set_log_key(&mut self, enabled: bool) -> bool {
        self.log_key = enabled;
        enabled
    }

    pub(crate) const fn log_key(&self) -> bool {
        self.log_key
    }

    pub(crate) fn delux_modify(&self) -> &[DaKongDeluxModify] {
        &self.delux_modify
    }

    /// Exact `GetSuccessProbability`: roll `0..9999` сравнивается через
    /// `<=`, поэтому нулевая вероятность сохраняет один успешный исход.
    pub(crate) fn get_success_probability(
        &self,
        index: i32,
        mut random: impl FnMut(i32) -> i32,
    ) -> bool {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.info.get(index))
            .is_some_and(|info| random(10_000) <= info.probability)
    }

    pub(crate) fn get_color(&self, index: i32, mut random: impl FnMut(i32) -> i32) -> i32 {
        let Some(info) = usize::try_from(index)
            .ok()
            .and_then(|index| self.info.get(index))
        else {
            return -1;
        };
        choose_random(
            &[
                info.red,
                info.green,
                info.blue,
                info.yellow,
                info.cyan,
                info.purple,
                info.delux,
            ],
            &mut random,
        )
    }

    pub(crate) fn make_sure_external_attribute(
        &self,
        group: i32,
        goods_id: u32,
        mut random: impl FnMut(i32) -> i32,
    ) -> Option<(i32, i32)> {
        let attributes = usize::try_from(group)
            .ok()
            .and_then(|group| self.external_attributes.get(group))?
            .get(&goods_id)?;
        let probabilities: Vec<_> = attributes
            .iter()
            .map(|attribute| attribute.probability)
            .collect();
        let selected = usize::try_from(choose_random(&probabilities, &mut random)).ok()?;
        let attribute = attributes.get(selected)?;
        let width = attribute
            .maximum
            .wrapping_sub(attribute.minimum)
            .wrapping_add(1);
        Some((
            attribute.property_type,
            random(width).wrapping_add(attribute.minimum),
        ))
    }

    pub(crate) fn check_external_property(&self, goods_id: u32, group: i32) -> bool {
        usize::try_from(group.wrapping_sub(1))
            .ok()
            .and_then(|group| self.external_attributes.get(group))
            .is_some_and(|attributes| attributes.contains_key(&goods_id))
    }

    /// Воспроизводит статический Game decoder selector-а `0x2B`.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<DaKongDecodeReport, DaKongDecodeError> {
        self.clear_primary_state();

        let info_count = read_wire_i32(source, cursor)?;
        for _ in 0..info_count.max(0) {
            self.info.push(DaKongInfo {
                probability: read_wire_i32(source, cursor)?,
                red: read_wire_i32(source, cursor)?,
                green: read_wire_i32(source, cursor)?,
                blue: read_wire_i32(source, cursor)?,
                yellow: read_wire_i32(source, cursor)?,
                cyan: read_wire_i32(source, cursor)?,
                purple: read_wire_i32(source, cursor)?,
                delux: read_wire_i32(source, cursor)?,
            });
        }

        for map_index in 0..self.external_attributes.len() {
            let goods_count = read_wire_i32(source, cursor)?;
            for _ in 0..goods_count.max(0) {
                let goods_id = read_wire_u32(source, cursor)?;
                let attribute_count = read_wire_i32(source, cursor)?;
                for _ in 0..attribute_count.max(0) {
                    let attribute = DaKongExternalAttribute {
                        property_type: read_wire_i32(source, cursor)?,
                        probability: read_wire_i32(source, cursor)?,
                        minimum: read_wire_i32(source, cursor)?,
                        maximum: read_wire_i32(source, cursor)?,
                    };
                    self.external_attributes[map_index]
                        .entry(goods_id)
                        .or_default()
                        .push(attribute);
                }
            }
        }

        let delux_count = read_wire_i32(source, cursor)?;
        for _ in 0..delux_count.max(0) {
            self.delux_modify.push(DaKongDeluxModify {
                property_type: read_wire_i32(source, cursor)?,
                add_type: read_wire_i32(source, cursor)?,
            });
        }

        Ok(DaKongDecodeReport {
            info: self.info.len(),
            external_attributes: self
                .external_attributes
                .iter()
                .flat_map(BTreeMap::values)
                .map(Vec::len)
                .sum(),
            delux_modify: self.delux_modify.len(),
        })
    }
}

fn choose_random(weights: &[i32], random: &mut impl FnMut(i32) -> i32) -> i32 {
    if weights.is_empty() {
        return -1;
    }
    let mut roll = random(10_000).wrapping_add(1);
    for (index, weight) in weights.iter().enumerate() {
        roll = roll.wrapping_sub(*weight);
        if roll < 1 {
            return index as i32;
        }
    }
    -1
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DaKongDecodeReport {
    pub(crate) info: usize,
    pub(crate) external_attributes: usize,
    pub(crate) delux_modify: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DaKongDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for DaKongDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "DaKong snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for DaKongDecodeError {}

fn write_count(destination: &mut Vec<u8>, count: usize) -> Result<(), DaKongSerializeError> {
    let count = i32::try_from(count).map_err(|_| DaKongSerializeError::CountOverflow)?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn positive_count(value: i32) -> usize {
    usize::try_from(value).unwrap_or_default()
}

struct WhitespaceParser<'a> {
    remaining: &'a [u8],
}

impl<'a> WhitespaceParser<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self { remaining: source }
    }

    fn token(&mut self) -> Option<&'a [u8]> {
        let start = self
            .remaining
            .iter()
            .position(|byte| !byte.is_ascii_whitespace())?;
        self.remaining = &self.remaining[start..];
        let end = self
            .remaining
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(self.remaining.len());
        let (token, remaining) = self.remaining.split_at(end);
        self.remaining = remaining;
        Some(token)
    }

    fn i32(&mut self) -> Option<i32> {
        parse_legacy_i32(self.token()?)
    }
}

struct MarkedTextParser<'a> {
    tokens: WhitespaceParser<'a>,
}

impl<'a> MarkedTextParser<'a> {
    fn new(source: &'a [u8]) -> Self {
        Self {
            tokens: WhitespaceParser::new(source),
        }
    }

    fn marked_i32(&mut self) -> Option<i32> {
        while let Some(token) = self.tokens.token() {
            if token.len() >= 2 && token[0] == b'#' {
                return self.tokens.i32();
            }
        }
        None
    }

    fn marked_i32s<const N: usize>(&mut self) -> Option<[i32; N]> {
        while let Some(token) = self.tokens.token() {
            if token.len() >= 2 && token[0] == b'#' {
                let mut values = [0; N];
                for value in &mut values {
                    *value = self.tokens.i32()?;
                }
                return Some(values);
            }
        }
        None
    }
}

fn parse_legacy_i32(token: &[u8]) -> Option<i32> {
    let mut bytes = token.iter().copied().peekable();
    let negative = matches!(bytes.peek(), Some(b'-'));
    if matches!(bytes.peek(), Some(b'-' | b'+')) {
        bytes.next();
    }
    let mut parsed = false;
    let mut result = 0_i32;
    for byte in bytes {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        parsed = true;
        result = result.saturating_mul(10).saturating_add(i32::from(digit));
    }
    parsed.then_some(if negative {
        result.saturating_neg()
    } else {
        result
    })
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, DaKongDecodeError> {
    Ok(i32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, DaKongDecodeError> {
    Ok(u32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], DaKongDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(DaKongDecodeError {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes.try_into().expect("размер DaKong scalar уже проверен"))
}
