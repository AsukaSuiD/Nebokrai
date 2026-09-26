//! Precious Box `PreciousBoxConf` из WorldServer/GameServer.
//! Контракт подтверждён точными `Nworldserver.exe + WorldServer.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/setup/preciousboxconf.h/.cpp`.
//!
//! Wire пишет ordered boxes, odds groups и 17 значимых bytes каждого item;
//! три padding bytes C++ record не передаются.
//!
//! XML loader очищает `_box_conf` и `_box`, но публикует records только в
//! `_box_conf`, тогда как serializer читает `_box`. Поэтому после reload
//! subtype `0x1E` остаётся пустым. `quick-xml` и owned maps заменяют TinyXML/STL.
//! Game decoder очищает только `_box` и публикует box после полного разбора
//! его временного odds-vector. Safe short-buffer поэтому оставляет прежние
//! полные box-ы, но не текущий; неизвестный UB безразмерного pointer отброшен.
//! `random_item` сохраняет GameServer order: один roll `[0, 10000)`, первый
//! подходящий half-open odds range, затем равномерный item и inclusive level.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::protocol::LegacyReader;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreciousBoxItem {
    pub item_idx: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub amount: i32,
    pub broadcast: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreciousBoxOdds {
    pub min_odds: i32,
    pub max_odds: i32,
    pub items: Vec<PreciousBoxItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreciousBox {
    pub odds: Vec<PreciousBoxOdds>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PreciousBoxConfig {
    odds: Vec<PreciousBoxConfigOdds>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PreciousBoxConfigOdds {
    odds: i32,
    max_odds: i32,
    items: Vec<PreciousBoxItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreciousBoxConf {
    box_config: BTreeMap<i32, PreciousBoxConfig>,
    boxes: BTreeMap<i32, PreciousBox>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreciousBoxLoadReport {
    diagnostics: Vec<PreciousBoxLoadDiagnostic>,
}

impl PreciousBoxLoadReport {
    pub fn diagnostics(&self) -> &[PreciousBoxLoadDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreciousBoxLoadDiagnostic {
    MissingNum,
    DuplicateNum(i32),
    MissingProbability,
    MissingOriginalName,
    MissingGoodsIndex(Vec<u8>),
}

impl PreciousBoxLoadDiagnostic {
    pub fn log_payload(&self) -> Vec<u8> {
        match self {
            Self::MissingNum => b"PreciousBox Must Has Num".to_vec(),
            Self::DuplicateNum(number) => {
                format!("Ignore Repeat PreciousBox Num {number}! ").into_bytes()
            }
            Self::MissingProbability => b"PreciousBox Probability Is NULL!".to_vec(),
            Self::MissingOriginalName => b"PreciousBox Goods OriName Is NULL!".to_vec(),
            Self::MissingGoodsIndex(name) => {
                let mut payload = b"Property ".to_vec();
                payload.extend_from_slice(name);
                payload.extend_from_slice(b"  has no opposite Index.");
                payload
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreciousBoxLoadError {
    MissingResource,
    InvalidDocument,
}

impl PreciousBoxLoadError {
    pub const fn log_payload(self) -> Option<&'static [u8]> {
        match self {
            Self::MissingResource => Some(b"PreciousBox Setup Not Found, Function Stop."),
            Self::InvalidDocument => None,
        }
    }
}

impl PreciousBoxConf {
    pub fn insert_box(&mut self, box_id: i32, value: PreciousBox) -> Option<PreciousBox> {
        self.boxes.insert(box_id, value)
    }

    pub fn clear(&mut self) {
        self.box_config.clear();
        self.boxes.clear();
    }

    pub fn boxes(&self) -> &BTreeMap<i32, PreciousBox> {
        &self.boxes
    }

    pub fn random_item(
        &self,
        box_id: i32,
        mut random: impl FnMut(i32) -> i32,
    ) -> Option<PreciousBoxItem> {
        let box_value = self.boxes.get(&box_id)?;
        let odds_roll = random(10_000);
        let odds = box_value
            .odds
            .iter()
            .find(|odds| odds.min_odds <= odds_roll && odds_roll < odds.max_odds)?;
        let item_count = i32::try_from(odds.items.len())
            .ok()
            .filter(|count| *count > 0)?;
        let mut item = *odds.items.get(random(item_count) as usize)?;
        let level_width = i64::from(item.max_level) - i64::from(item.min_level) + 1;
        if (2..=i64::from(i32::MAX)).contains(&level_width) {
            item.min_level = item.min_level.wrapping_add(random(level_width as i32));
        }
        item.max_level = item.min_level;
        Some(item)
    }

    pub fn load_from_bytes(
        &mut self,
        source: Option<&[u8]>,
        mut query_goods_id: impl FnMut(&[u8]) -> u32,
    ) -> Result<PreciousBoxLoadReport, PreciousBoxLoadError> {
        self.clear();
        let Some(source) = source else {
            return Err(PreciousBoxLoadError::MissingResource);
        };
        self.load_from_bytes_after_clear(source, &mut query_goods_id)
    }

    fn load_from_bytes_after_clear(
        &mut self,
        source: &[u8],
        query_goods_id: &mut impl FnMut(&[u8]) -> u32,
    ) -> Result<PreciousBoxLoadReport, PreciousBoxLoadError> {
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut active_box: Option<ActivePreciousBox> = None;
        let mut active_odds: Option<ActivePreciousOdds> = None;
        let mut report = PreciousBoxLoadReport::default();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    self.process_start(
                        &start,
                        depth,
                        &mut root_seen,
                        &mut active_box,
                        &mut active_odds,
                        &mut report,
                        query_goods_id,
                    )?;
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    self.process_start(
                        &empty,
                        depth,
                        &mut root_seen,
                        &mut active_box,
                        &mut active_odds,
                        &mut report,
                        query_goods_id,
                    )?;
                    self.process_end(
                        empty.name().as_ref(),
                        depth,
                        &mut active_box,
                        &mut active_odds,
                    );
                }
                Ok(Event::End(end)) => {
                    if depth == 0 {
                        return Err(PreciousBoxLoadError::InvalidDocument);
                    }
                    depth -= 1;
                    self.process_end(
                        end.name().as_ref(),
                        depth,
                        &mut active_box,
                        &mut active_odds,
                    );
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(PreciousBoxLoadError::InvalidDocument),
            }
            buffer.clear();
        }
        if !root_seen || depth != 0 || active_box.is_some() || active_odds.is_some() {
            return Err(PreciousBoxLoadError::InvalidDocument);
        }
        Ok(report)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_start(
        &mut self,
        start: &BytesStart<'_>,
        depth: usize,
        root_seen: &mut bool,
        active_box: &mut Option<ActivePreciousBox>,
        active_odds: &mut Option<ActivePreciousOdds>,
        report: &mut PreciousBoxLoadReport,
        query_goods_id: &mut impl FnMut(&[u8]) -> u32,
    ) -> Result<(), PreciousBoxLoadError> {
        let name = start.name();
        if !*root_seen {
            if name.as_ref() != ROOT {
                return Err(PreciousBoxLoadError::InvalidDocument);
            }
            *root_seen = true;
        } else if depth == 1 && name.as_ref() == BOX {
            if enabled(start) {
                match attr(start, NUM) {
                    Some(value) => {
                        let number = legacy_atol(&value);
                        if self.box_config.contains_key(&number) {
                            report
                                .diagnostics
                                .push(PreciousBoxLoadDiagnostic::DuplicateNum(number));
                        } else {
                            *active_box = Some(ActivePreciousBox {
                                number,
                                odds: Vec::new(),
                            });
                        }
                    }
                    None => report
                        .diagnostics
                        .push(PreciousBoxLoadDiagnostic::MissingNum),
                }
            }
        } else if depth == 2 && name.as_ref() == ODDS {
            if active_box.is_some() && enabled(start) {
                if let Some(value) = attr(start, PROBABILITY) {
                    let (odds, max_odds) = parse_pair(&value, b'/');
                    *active_odds = Some(ActivePreciousOdds {
                        odds,
                        max_odds,
                        items: Vec::new(),
                    });
                } else {
                    report
                        .diagnostics
                        .push(PreciousBoxLoadDiagnostic::MissingProbability);
                }
            }
        } else if depth == 3 && name.as_ref() == ITEM {
            if let Some(odds) = active_odds.as_mut().filter(|_| enabled(start)) {
                let Some(original_name) = attr(start, ORIGINAL_NAME) else {
                    report
                        .diagnostics
                        .push(PreciousBoxLoadDiagnostic::MissingOriginalName);
                    return Ok(());
                };
                let item_idx = query_goods_id(&original_name);
                if item_idx == 0 {
                    report
                        .diagnostics
                        .push(PreciousBoxLoadDiagnostic::MissingGoodsIndex(original_name));
                    return Ok(());
                }
                let (min_level, max_level) = attr(start, LEVEL)
                    .map(|value| parse_pair(&value, b'-'))
                    .unwrap_or((0, 0));
                let amount = attr(start, COUNT)
                    .map(|value| legacy_atoi(&value))
                    .unwrap_or(1);
                let broadcast = attr(start, BROADCAST)
                    .map(|value| legacy_atoi(&value) != 0)
                    .unwrap_or(false);
                odds.items.push(PreciousBoxItem {
                    item_idx: item_idx as i32,
                    min_level,
                    max_level,
                    amount,
                    broadcast,
                });
            }
        }
        Ok(())
    }

    fn process_end(
        &mut self,
        name: &[u8],
        depth: usize,
        active_box: &mut Option<ActivePreciousBox>,
        active_odds: &mut Option<ActivePreciousOdds>,
    ) {
        if depth == 2 && name == ODDS {
            if let Some(odds) = active_odds.take().filter(|odds| !odds.items.is_empty()) {
                if let Some(active_box) = active_box.as_mut() {
                    active_box.odds.push(PreciousBoxConfigOdds {
                        odds: odds.odds,
                        max_odds: odds.max_odds,
                        items: odds.items,
                    });
                }
            }
        } else if depth == 1 && name == BOX {
            if let Some(active_box) = active_box
                .take()
                .filter(|box_conf| !box_conf.odds.is_empty())
            {
                self.box_config.insert(
                    active_box.number,
                    PreciousBoxConfig {
                        odds: active_box.odds,
                    },
                );
            }
        }
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PreciousBoxSerializeError> {
        write_count(destination, self.boxes.len(), PreciousBoxCount::Boxes)?;
        for (&box_id, box_value) in &self.boxes {
            destination.extend_from_slice(&box_id.to_le_bytes());
            write_count(
                destination,
                box_value.odds.len(),
                PreciousBoxCount::Odds { box_id },
            )?;
            for (odds_index, odds) in box_value.odds.iter().enumerate() {
                destination.extend_from_slice(&odds.min_odds.to_le_bytes());
                destination.extend_from_slice(&odds.max_odds.to_le_bytes());
                write_count(
                    destination,
                    odds.items.len(),
                    PreciousBoxCount::Items { box_id, odds_index },
                )?;
                for item in &odds.items {
                    destination.extend_from_slice(&item.item_idx.to_le_bytes());
                    destination.extend_from_slice(&item.min_level.to_le_bytes());
                    destination.extend_from_slice(&item.max_level.to_le_bytes());
                    destination.extend_from_slice(&item.amount.to_le_bytes());
                    destination.push(u8::from(item.broadcast));
                }
            }
        }
        Ok(())
    }

    /// Воспроизводит `PreciousBoxConf::DecordFromByteArray` GameServer.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, PreciousBoxDecodeError> {
        self.boxes.clear();
        let box_count = read_wire_i32(source, cursor)?;
        for _ in 0..box_count.max(0) {
            let box_id = read_wire_i32(source, cursor)?;
            let odds_count = read_wire_i32(source, cursor)?;
            let mut odds_groups = Vec::new();
            odds_groups
                .try_reserve(odds_count.max(0) as usize)
                .map_err(|source| PreciousBoxDecodeError::Allocation {
                    field: PreciousBoxDecodeField::Odds { box_id },
                    source,
                })?;

            for odds_index in 0..odds_count.max(0) {
                let min_odds = read_wire_i32(source, cursor)?;
                let max_odds = read_wire_i32(source, cursor)?;
                let item_count = read_wire_i32(source, cursor)?;
                let mut items = Vec::new();
                items
                    .try_reserve(item_count.max(0) as usize)
                    .map_err(|source| PreciousBoxDecodeError::Allocation {
                        field: PreciousBoxDecodeField::Items {
                            box_id,
                            odds_index: odds_index as usize,
                        },
                        source,
                    })?;
                for _ in 0..item_count.max(0) {
                    items.push(PreciousBoxItem {
                        item_idx: read_wire_i32(source, cursor)?,
                        min_level: read_wire_i32(source, cursor)?,
                        max_level: read_wire_i32(source, cursor)?,
                        amount: read_wire_i32(source, cursor)?,
                        broadcast: read_wire_u8(source, cursor)? != 0,
                    });
                }
                odds_groups.push(PreciousBoxOdds {
                    min_odds,
                    max_odds,
                    items,
                });
            }
            self.boxes.insert(box_id, PreciousBox { odds: odds_groups });
        }
        Ok(self.boxes.len())
    }
}

#[derive(Debug)]
struct ActivePreciousBox {
    number: i32,
    odds: Vec<PreciousBoxConfigOdds>,
}

#[derive(Debug)]
struct ActivePreciousOdds {
    odds: i32,
    max_odds: i32,
    items: Vec<PreciousBoxItem>,
}

// Имена TinyXML были GBK-строками; значения XML намеренно остаются байтами.
const ROOT: &[u8] = &[0xB0, 0xD9, 0xB1, 0xA6, 0xCF, 0xE4, 0xC5, 0xE4, 0xD6, 0xC3];
const BOX: &[u8] = &[0xB0, 0xD9, 0xB1, 0xA6, 0xCF, 0xE4];
const ODDS: &[u8] = &[0xBC, 0xB8, 0xC2, 0xCA, 0xC0, 0xE0];
const ITEM: &[u8] = &[0xB5, 0xC0, 0xBE, 0xDF];
const ENABLED: &[u8] = &[0xC6, 0xF4, 0xD3, 0xC3];
const NUM: &[u8] = &[0xB1, 0xE0, 0xBA, 0xC5];
const PROBABILITY: &[u8] = &[0xBC, 0xB8, 0xC2, 0xCA];
const ORIGINAL_NAME: &[u8] = &[0xD4, 0xAD, 0xCA, 0xBC, 0xC3, 0xFB];
const COUNT: &[u8] = &[0xCA, 0xFD, 0xC1, 0xBF];
const LEVEL: &[u8] = &[0xB5, 0xC8, 0xBC, 0xB6];
const BROADCAST: &[u8] = &[0xB9, 0xE3, 0xB8, 0xE6];

fn attr(start: &BytesStart<'_>, name: &[u8]) -> Option<Vec<u8>> {
    start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == name)
        .map(|attribute| attribute.value.into_owned())
}

fn enabled(start: &BytesStart<'_>) -> bool {
    attr(start, ENABLED).is_some_and(|value| value.first() == Some(&b'1'))
}

fn parse_pair(value: &[u8], separator: u8) -> (i32, i32) {
    let Some(position) = value.iter().position(|byte| *byte == separator) else {
        let value = legacy_atoi(value);
        return (value, value);
    };
    (
        legacy_atoi(&value[..position]),
        legacy_atoi(&value[position + 1..]),
    )
}

fn legacy_atol(value: &[u8]) -> i32 {
    legacy_atoi(value)
}

fn legacy_atoi(value: &[u8]) -> i32 {
    let mut bytes = value
        .iter()
        .copied()
        .skip_while(u8::is_ascii_whitespace)
        .peekable();
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
    if parsed {
        if negative {
            result.saturating_neg()
        } else {
            result
        }
    } else {
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreciousBoxCount {
    Boxes,
    Odds { box_id: i32 },
    Items { box_id: i32, odds_index: usize },
}

impl fmt::Display for PreciousBoxCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boxes => formatter.write_str("box-ов PreciousBox"),
            Self::Odds { box_id } => write!(formatter, "групп вероятностей box {box_id}"),
            Self::Items { box_id, odds_index } => {
                write!(formatter, "предметов группы {odds_index} box {box_id}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreciousBoxSerializeError {
    pub field: PreciousBoxCount,
    pub count: usize,
}

impl fmt::Display for PreciousBoxSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "количество {} ({}) не помещается в signed 32-битный диапазон",
            self.field, self.count
        )
    }
}

impl Error for PreciousBoxSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreciousBoxDecodeField {
    Odds { box_id: i32 },
    Items { box_id: i32, odds_index: usize },
}

impl fmt::Display for PreciousBoxDecodeField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Odds { box_id } => write!(formatter, "odds box {box_id}"),
            Self::Items { box_id, odds_index } => {
                write!(formatter, "items odds {odds_index} box {box_id}")
            }
        }
    }
}

#[derive(Debug)]
pub enum PreciousBoxDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    Allocation {
        field: PreciousBoxDecodeField,
        source: std::collections::TryReserveError,
    },
}

impl fmt::Display for PreciousBoxDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "PreciousBox snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::Allocation { field, .. } => {
                write!(formatter, "не удалось выделить память для {field}")
            }
        }
    }
}

impl Error for PreciousBoxDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedEnd { .. } => None,
            Self::Allocation { source, .. } => Some(source),
        }
    }
}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    field: PreciousBoxCount,
) -> Result<(), PreciousBoxSerializeError> {
    let count_i32 = i32::try_from(count).map_err(|_| PreciousBoxSerializeError { field, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

fn read_wire_u8(source: &[u8], cursor: &mut usize) -> Result<u8, PreciousBoxDecodeError> {
    LegacyReader::read_u8_from(source, cursor).map_err(map_read_block)
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, PreciousBoxDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(map_read_block)
}

fn map_read_block(
    block: crate::protocol::LegacyReadBlock,
) -> PreciousBoxDecodeError {
    PreciousBoxDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
