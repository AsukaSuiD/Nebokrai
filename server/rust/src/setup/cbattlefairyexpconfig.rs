//! Опыт fairy/battle fairy `CBattleFairyExpConfig` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/setup/cbattlefairyexpconfig.h/.cpp`.
//!
//! Wire — signed group count, затем ordered owner level, signed value count и
//! `u32` exp values. `CFairyExpConf` наследует этот serializer.
//!
//! Loader принимает `ZHUANHUNJINGYANLIEBIAO/ZhanHunPinZhong`, требует
//! `RenZhuLevel`, `MaxLevel` и не менее `MaxLevel - 1` значений. Duplicate level
//! или неполная группа очищает всю map; child `Level` не читается.
//! Game decoder очищает map, но создаёт key только при первом exp value;
//! повторная wire-группа того же owner level дописывается в прежний vector.
//! `dwExpUp` выбирает `level - 1`, возвращая ноль для нулевого level,
//! отсутствующей owner-группы и выхода за её границы.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::Event;

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CBattleFairyExpConfig {
    exp_lists: BTreeMap<u32, Vec<u32>>,
}

impl CBattleFairyExpConfig {
    pub(crate) fn insert_exp_list(
        &mut self,
        owner_level: u32,
        values: Vec<u32>,
    ) -> Option<Vec<u32>> {
        self.exp_lists.insert(owner_level, values)
    }

    pub(crate) fn contains_exp_list(&self, owner_level: u32) -> bool {
        self.exp_lists.contains_key(&owner_level)
    }

    pub(crate) fn dw_exp_up(&self, equip_level: u32, level: u32) -> u32 {
        if level == 0 {
            return 0;
        }
        self.exp_lists
            .get(&equip_level)
            .and_then(|values| values.get((level - 1) as usize))
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn exp_lists(&self) -> &BTreeMap<u32, Vec<u32>> {
        &self.exp_lists
    }

    pub(crate) fn clear(&mut self) {
        self.exp_lists.clear();
    }

    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<BattleFairyExpLoadReport, BattleFairyExpLoadError> {
        self.clear();
        let result = self.load_from_bytes_after_clear(source);
        if result.is_err() {
            self.clear();
        }
        result
    }

    fn load_from_bytes_after_clear(
        &mut self,
        source: &[u8],
    ) -> Result<BattleFairyExpLoadReport, BattleFairyExpLoadError> {
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut group_count = 0usize;
        let mut active_group: Option<ActiveBattleFairyGroup> = None;
        let mut report = BattleFairyExpLoadReport::default();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    let name = start.name().as_ref().to_vec();
                    if !root_seen {
                        if name != b"ZHUANHUNJINGYANLIEBIAO" {
                            return Err(BattleFairyExpLoadError::MissingRoot);
                        }
                        root_seen = true;
                    } else if depth == 1 && name == b"ZhanHunPinZhong" {
                        let owner_level = required_atol_attribute(
                            &start,
                            b"RenZhuLevel",
                            BattleFairyExpLoadError::MissingOwnerLevel,
                        )? as u32;
                        if self.exp_lists.contains_key(&owner_level) {
                            return Err(BattleFairyExpLoadError::DuplicateOwnerLevel(owner_level));
                        }
                        let max_level = required_atol_attribute(
                            &start,
                            b"MaxLevel",
                            BattleFairyExpLoadError::MissingMaxLevel,
                        )? as u32;
                        self.exp_lists.insert(owner_level, Vec::new());
                        active_group = Some(ActiveBattleFairyGroup {
                            owner_level,
                            max_level,
                        });
                        group_count += 1;
                    } else if depth == 2 && name == b"ZhanHun" {
                        let experience = required_atol_attribute(
                            &start,
                            b"UpdateLevelExp",
                            BattleFairyExpLoadError::MissingUpdateLevelExp,
                        )? as u32;
                        let group = active_group
                            .as_ref()
                            .ok_or(BattleFairyExpLoadError::ZhanHunOutsideGroup)?;
                        self.exp_lists
                            .get_mut(&group.owner_level)
                            .expect("active BattleFairy group создаётся до children")
                            .push(experience);
                    }
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    let name = empty.name().as_ref().to_vec();
                    if !root_seen {
                        if name != b"ZHUANHUNJINGYANLIEBIAO" {
                            return Err(BattleFairyExpLoadError::MissingRoot);
                        }
                        root_seen = true;
                    } else if depth == 1 && name == b"ZhanHunPinZhong" {
                        let owner_level = required_atol_attribute(
                            &empty,
                            b"RenZhuLevel",
                            BattleFairyExpLoadError::MissingOwnerLevel,
                        )? as u32;
                        if self.exp_lists.contains_key(&owner_level) {
                            return Err(BattleFairyExpLoadError::DuplicateOwnerLevel(owner_level));
                        }
                        let max_level = required_atol_attribute(
                            &empty,
                            b"MaxLevel",
                            BattleFairyExpLoadError::MissingMaxLevel,
                        )? as u32;
                        self.exp_lists.insert(owner_level, Vec::new());
                        group_count += 1;
                        ensure_minimum_exp_values(owner_level, max_level, 0)?;
                        report.groups += 1;
                    } else if depth == 2 && name == b"ZhanHun" {
                        let experience = required_atol_attribute(
                            &empty,
                            b"UpdateLevelExp",
                            BattleFairyExpLoadError::MissingUpdateLevelExp,
                        )? as u32;
                        let group = active_group
                            .as_ref()
                            .ok_or(BattleFairyExpLoadError::ZhanHunOutsideGroup)?;
                        self.exp_lists
                            .get_mut(&group.owner_level)
                            .expect("active BattleFairy group создаётся до children")
                            .push(experience);
                    }
                }
                Ok(Event::End(end)) => {
                    if depth == 0 {
                        return Err(BattleFairyExpLoadError::Xml("лишний closing tag".into()));
                    }
                    depth -= 1;
                    if depth == 1 && end.name().as_ref() == b"ZhanHunPinZhong" {
                        let group = active_group
                            .take()
                            .ok_or(BattleFairyExpLoadError::ZhanHunOutsideGroup)?;
                        let count = self
                            .exp_lists
                            .get(&group.owner_level)
                            .expect("active BattleFairy group остаётся в map")
                            .len();
                        ensure_minimum_exp_values(group.owner_level, group.max_level, count)?;
                        report.groups += 1;
                        report.experience_values += count;
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(error) => return Err(BattleFairyExpLoadError::Xml(error.to_string())),
            }
            buffer.clear();
        }
        if !root_seen || group_count == 0 {
            return Err(BattleFairyExpLoadError::MissingRoot);
        }
        if active_group.is_some() || depth != 0 {
            return Err(BattleFairyExpLoadError::Xml(
                "незавершённый XML element".into(),
            ));
        }
        Ok(report)
    }

    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<BattleFairyExpLoadReport, BattleFairyExpFileLoadError> {
        self.clear();
        let source = std::fs::read(path).map_err(BattleFairyExpFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(BattleFairyExpFileLoadError::Format)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), BattleFairyExpSerializeError> {
        write_count(destination, self.exp_lists.len(), None)?;
        for (&owner_level, values) in &self.exp_lists {
            destination.extend_from_slice(&owner_level.to_le_bytes());
            write_count(destination, values.len(), Some(owner_level))?;
            for &value in values {
                destination.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(())
    }

    /// Воспроизводит `CBattleFairyExpConfig::DecordFromByteArray` GameServer.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), BattleFairyExpDecodeError> {
        self.exp_lists.clear();
        let group_count = read_wire_i32(source, cursor)?;
        for _ in 0..group_count.max(0) {
            let owner_level = read_wire_u32(source, cursor)?;
            let value_count = read_wire_i32(source, cursor)?;
            for _ in 0..value_count.max(0) {
                let value = read_wire_u32(source, cursor)?;
                self.exp_lists.entry(owner_level).or_default().push(value);
            }
        }
        tracing::trace!(
            groups = self.exp_lists.len(),
            experience_values = self.exp_lists.values().map(Vec::len).sum::<usize>(),
            "таблица опыта боевых фей декодирована"
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct ActiveBattleFairyGroup {
    owner_level: u32,
    max_level: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BattleFairyExpLoadReport {
    pub(crate) groups: usize,
    pub(crate) experience_values: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyExpLoadError {
    MissingRoot,
    MissingOwnerLevel,
    DuplicateOwnerLevel(u32),
    MissingMaxLevel,
    MissingUpdateLevelExp,
    InsufficientExperienceValues {
        owner_level: u32,
        max_level: u32,
        actual: usize,
    },
    ZhanHunOutsideGroup,
    Xml(String),
}

impl BattleFairyExpLoadError {
    pub(crate) const fn string_id(&self) -> &'static [u8] {
        match self {
            Self::MissingRoot | Self::ZhanHunOutsideGroup | Self::Xml(_) => b"ZHGS0031",
            Self::MissingOwnerLevel => b"ZHGS0032",
            Self::DuplicateOwnerLevel(_) => b"ZHGS0033",
            Self::MissingMaxLevel => b"ZHGS0034",
            Self::MissingUpdateLevelExp => b"ZHGS0035",
            Self::InsufficientExperienceValues { .. } => b"ZHGS0036",
        }
    }
}

impl fmt::Display for BattleFairyExpLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRoot => formatter.write_str("отсутствует ZHUANHUNJINGYANLIEBIAO"),
            Self::MissingOwnerLevel => formatter.write_str("отсутствует RenZhuLevel"),
            Self::DuplicateOwnerLevel(level) => write!(formatter, "повторный RenZhuLevel {level}"),
            Self::MissingMaxLevel => formatter.write_str("отсутствует MaxLevel"),
            Self::MissingUpdateLevelExp => formatter.write_str("отсутствует UpdateLevelExp"),
            Self::InsufficientExperienceValues {
                owner_level,
                max_level,
                actual,
            } => write!(
                formatter,
                "RenZhuLevel {owner_level} содержит {actual} exp values при MaxLevel {max_level}"
            ),
            Self::ZhanHunOutsideGroup => {
                formatter.write_str("ZhanHun находится вне ZhanHunPinZhong")
            }
            Self::Xml(error) => write!(formatter, "некорректный XML: {error}"),
        }
    }
}

impl Error for BattleFairyExpLoadError {}

#[derive(Debug)]
pub(crate) enum BattleFairyExpFileLoadError {
    Io(io::Error),
    Format(BattleFairyExpLoadError),
}

impl fmt::Display for BattleFairyExpFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for BattleFairyExpFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

fn required_atol_attribute(
    start: &quick_xml::events::BytesStart<'_>,
    expected: &[u8],
    missing: BattleFairyExpLoadError,
) -> Result<i32, BattleFairyExpLoadError> {
    let value = start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == expected)
        .map(|attribute| attribute.value.into_owned())
        .ok_or(missing)?;
    Ok(legacy_atol(&value))
}

fn ensure_minimum_exp_values(
    owner_level: u32,
    max_level: u32,
    actual: usize,
) -> Result<(), BattleFairyExpLoadError> {
    let actual_plus_one = actual.saturating_add(1);
    if actual_plus_one < max_level as usize {
        return Err(BattleFairyExpLoadError::InsufficientExperienceValues {
            owner_level,
            max_level,
            actual,
        });
    }
    Ok(())
}

fn legacy_atol(value: &[u8]) -> i32 {
    let mut bytes = value
        .iter()
        .copied()
        .skip_while(u8::is_ascii_whitespace)
        .peekable();
    let negative = matches!(bytes.peek(), Some(b'-'));
    if matches!(bytes.peek(), Some(b'-' | b'+')) {
        bytes.next();
    }
    let mut saw_digit = false;
    let mut result = 0_i32;
    for byte in bytes {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        saw_digit = true;
        result = result.saturating_mul(10).saturating_add(i32::from(digit));
    }
    if !saw_digit {
        0
    } else if negative {
        result.saturating_neg()
    } else {
        result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyExpSerializeError {
    pub(crate) owner_level: Option<u32>,
    pub(crate) count: usize,
}

impl fmt::Display for BattleFairyExpSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.owner_level {
            Some(owner_level) => write!(
                formatter,
                "таблица owner level {owner_level} содержит {} значений вне signed 32-битного диапазона",
                self.count
            ),
            None => write!(
                formatter,
                "FairyExp содержит {} групп вне signed 32-битного диапазона",
                self.count
            ),
        }
    }
}

impl Error for BattleFairyExpSerializeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyExpDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for BattleFairyExpDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "FairyExp snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for BattleFairyExpDecodeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    owner_level: Option<u32>,
) -> Result<(), BattleFairyExpSerializeError> {
    let count_i32 =
        i32::try_from(count).map_err(|_| BattleFairyExpSerializeError { owner_level, count })?;
    LegacyWriter::new(destination).write_i32(count_i32);
    Ok(())
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, BattleFairyExpDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(|block| BattleFairyExpDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, BattleFairyExpDecodeError> {
    LegacyReader::read_u32_from(source, cursor).map_err(|block| BattleFairyExpDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}
