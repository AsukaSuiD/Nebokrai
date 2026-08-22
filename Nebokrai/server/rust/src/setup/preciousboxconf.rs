//! Конфигурация Precious Box исторического Miracle.
//!
//! Статус World `load_conf` RVA `0x00048420` и `AddToByteArray` RVA
//! `0x00046220`: `IMPLEMENTED`; singleton plumbing и Game decoder/random owner
//! ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:316`.
//!
//! Wire начинается с signed количества box-ов. Далее ordered map выдаёт для
//! каждого box `i32 id + i32 odds_count`; каждая группа содержит
//! `i32 min_odds + i32 max_odds + i32 item_count`, а каждый предмет —
//! `i32 item_idx + i32 min_level + i32 max_level + i32 amount + bool-byte`.
//! Исходные 20-байтные MSVC Item содержат три байта padding после bool, но
//! serializer намеренно передаёт только 17 значимых байт.
//!
//! Точный World `load_conf` очищает оба map-а и публикует XML только в
//! `_box_conf`, тогда как serializer читает `_box`. `archive/Miracle_server_linux`
//! добавляет отдельный `PreciousBoxRanges::Build`, которого нет в RAW/EXE owner-е:
//! Rust сохраняет подтверждённый пустой `0x1E` wire после XML reload и не выдаёт
//! этот неподтверждённый ремонт за оригинальную семантику. `quick-xml` заменяет
//! TinyXML, `BTreeMap` — только MSVC ordered tree, `Vec`/`Drop` — ручной lifetime.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PreciousBoxItem {
    pub(crate) item_idx: i32,
    pub(crate) min_level: i32,
    pub(crate) max_level: i32,
    pub(crate) amount: i32,
    pub(crate) broadcast: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBoxOdds {
    pub(crate) min_odds: i32,
    pub(crate) max_odds: i32,
    pub(crate) items: Vec<PreciousBoxItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBox {
    pub(crate) odds: Vec<PreciousBoxOdds>,
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

/// Safe owner уже материализованного World `_box` state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBoxConf {
    box_config: BTreeMap<i32, PreciousBoxConfig>,
    boxes: BTreeMap<i32, PreciousBox>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PreciousBoxLoadReport {
    diagnostics: Vec<PreciousBoxLoadDiagnostic>,
}

impl PreciousBoxLoadReport {
    pub(crate) fn diagnostics(&self) -> &[PreciousBoxLoadDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PreciousBoxLoadDiagnostic {
    MissingNum,
    DuplicateNum(i32),
    MissingProbability,
    MissingOriginalName,
    MissingGoodsIndex(Vec<u8>),
}

impl PreciousBoxLoadDiagnostic {
    pub(crate) fn log_payload(&self) -> Vec<u8> {
        match self {
            Self::MissingNum => b"PreciousBox Must Has Num".to_vec(),
            Self::DuplicateNum(number) => format!("Ignore Repeat PreciousBox Num {number}! ").into_bytes(),
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
pub(crate) enum PreciousBoxLoadError {
    MissingResource,
    InvalidDocument,
}

impl PreciousBoxLoadError {
    pub(crate) const fn log_payload(self) -> Option<&'static [u8]> {
        match self {
            Self::MissingResource => Some(b"PreciousBox Setup Not Found, Function Stop."),
            Self::InvalidDocument => None,
        }
    }
}

impl PreciousBoxConf {
    /// Явная граница для будущего подтверждённого XML→range materializer-а.
    pub(crate) fn insert_box(&mut self, box_id: i32, value: PreciousBox) -> Option<PreciousBox> {
        self.boxes.insert(box_id, value)
    }

    pub(crate) fn clear(&mut self) {
        self.box_config.clear();
        self.boxes.clear();
    }

    /// Exact resource adapter. `query_goods_id` is the already-loaded World factory.
    pub(crate) fn load_from_bytes(
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
                    None => report.diagnostics.push(PreciousBoxLoadDiagnostic::MissingNum),
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
            if let Some(active_box) = active_box.take().filter(|box_conf| !box_conf.odds.is_empty()) {
                self.box_config.insert(
                    active_box.number,
                    PreciousBoxConfig {
                        odds: active_box.odds,
                    },
                );
            }
        }
    }

    /// Дописывает exact compact wire без C++ Item padding.
    pub(crate) fn add_to_byte_array(
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

// Raw TinyXML names are GBK byte strings; XML values deliberately stay bytes.
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
    let mut bytes = value.iter().copied().skip_while(u8::is_ascii_whitespace).peekable();
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
        if negative { result.saturating_neg() } else { result }
    } else {
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreciousBoxCount {
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
pub(crate) struct PreciousBoxSerializeError {
    pub(crate) field: PreciousBoxCount,
    pub(crate) count: usize,
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

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    field: PreciousBoxCount,
) -> Result<(), PreciousBoxSerializeError> {
    let count_i32 = i32::try_from(count).map_err(|_| PreciousBoxSerializeError { field, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// Сырой C++ ниже сохранён как локальная документация спорного World loader-а,
// singleton, Game decoder-а и random owner-а, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp

// ============================================================================
// FUNCTION: PreciousBoxConf::inst
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h:72
// RVA: 0x0009CE00
// ADDRESS: 0049ce00
// PROTOTYPE: PreciousBoxConf * __cdecl inst(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::random_item
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:394
// RVA: 0x001C57B0
// ADDRESS: 005c57b0
// PROTOTYPE: bool __thiscall random_item(long param_1, long * param_2, long * param_3, long * param_4, bool * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:355
// RVA: 0x001C7030
// ADDRESS: 005c7030
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::~PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:18
// RVA: 0x001C7640
// ADDRESS: 005c7640
// PROTOTYPE: void __thiscall ~PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:16
// RVA: 0x001C76E0
// ADDRESS: 005c76e0
// PROTOTYPE: undefined __thiscall PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp

// ============================================================================
// FUNCTION: PreciousBoxConf::inst
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.h:72
// RVA: 0x000015D0
// ADDRESS: 004015d0
// PROTOTYPE: PreciousBoxConf * __cdecl inst(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044544c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp
// RVA: 0x0004544C
// ADDRESS: 0044544c
// PROTOTYPE: undefined Catch@0044544c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00445477
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp
// RVA: 0x00045477
// ADDRESS: 00445477
// PROTOTYPE: undefined FUN_00445477()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:316
// RVA: 0x00046220
// ADDRESS: 00446220
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::load_conf
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:22
// RVA: 0x00048420
// ADDRESS: 00448420
// PROTOTYPE: bool __thiscall load_conf(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::~PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:18
// RVA: 0x00048FC0
// ADDRESS: 00448fc0
// PROTOTYPE: void __thiscall ~PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PreciousBoxConf::PreciousBoxConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\preciousboxconf.cpp:16
// RVA: 0x00049060
// ADDRESS: 00449060
// PROTOTYPE: undefined __thiscall PreciousBoxConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: WorldServer
