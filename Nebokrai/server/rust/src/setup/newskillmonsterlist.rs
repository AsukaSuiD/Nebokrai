//! Списки монстров новых навыков `CNewSkillMonserConf` из WorldServer,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Wire пишет ordered skill ID, signed count и localized monster C-строки.
//! Duplicate skill ID заменяет всю группу, повторы и порядок имён сохраняются.
//!
//! Loader очищает map, требует `NewSkillMonsterList/monsterlist/monster` и
//! атрибуты `index/strorgname`. StringTable miss даёт пустое имя. `quick-xml`
//! принимает исторические unquoted ASCII attributes после узкой нормализации.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NewSkillMonsterConf {
    groups: BTreeMap<u32, Vec<Vec<u8>>>,
}

impl NewSkillMonsterConf {
    pub(crate) fn insert_group(
        &mut self,
        skill_id: u32,
        names: Vec<Vec<u8>>,
    ) -> Option<Vec<Vec<u8>>> {
        self.groups.insert(skill_id, names)
    }

    pub(crate) fn clear(&mut self) {
        self.groups.clear();
    }

    pub(crate) fn load_from_bytes<ResolveName>(
        &mut self,
        source: &[u8],
        resolve_name: &mut ResolveName,
    ) -> Result<NewSkillMonsterLoadReport, NewSkillMonsterLoadError>
    where
        ResolveName: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.clear();
        let normalized = normalize_legacy_attributes(source);
        let mut reader = Reader::from_reader(normalized.as_slice());
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut monsterlist_seen = false;
        let mut active_group: Option<ActiveMonsterGroup> = None;
        let mut report = NewSkillMonsterLoadReport::default();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    let name = start.name().as_ref().to_vec();
                    if !root_seen {
                        if name != b"NewSkillMonsterList" {
                            return Err(NewSkillMonsterLoadError::RootMissing);
                        }
                        root_seen = true;
                    } else if depth == 1 && name == b"monsterlist" {
                        let skill_id = read_index(&start).map_err(|error| {
                            self.clear();
                            error
                        })?;
                        monsterlist_seen = true;
                        active_group = Some(ActiveMonsterGroup::new(skill_id));
                    } else if depth == 2 && name == b"monster" {
                        let Some(key) = attribute(&start, b"strorgname") else {
                            self.clear();
                            return Err(NewSkillMonsterLoadError::MissingOriginalName);
                        };
                        let group = active_group
                            .as_mut()
                            .ok_or(NewSkillMonsterLoadError::MonsterOutsideGroup)?;
                        group.names.push(resolve_name(&key).unwrap_or_default());
                    }
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    let name = empty.name().as_ref().to_vec();
                    if !root_seen {
                        if name != b"NewSkillMonsterList" {
                            return Err(NewSkillMonsterLoadError::RootMissing);
                        }
                        root_seen = true;
                    } else if depth == 1 && name == b"monsterlist" {
                        return Err(NewSkillMonsterLoadError::MissingMonster);
                    } else if depth == 2 && name == b"monster" {
                        let Some(key) = attribute(&empty, b"strorgname") else {
                            self.clear();
                            return Err(NewSkillMonsterLoadError::MissingOriginalName);
                        };
                        let group = active_group
                            .as_mut()
                            .ok_or(NewSkillMonsterLoadError::MonsterOutsideGroup)?;
                        group.names.push(resolve_name(&key).unwrap_or_default());
                    }
                }
                Ok(Event::End(end)) => {
                    if depth == 0 {
                        return Err(NewSkillMonsterLoadError::Xml("лишний closing tag".into()));
                    }
                    depth -= 1;
                    if depth == 1 && end.name().as_ref() == b"monsterlist" {
                        let group = active_group
                            .take()
                            .ok_or(NewSkillMonsterLoadError::MonsterOutsideGroup)?;
                        if group.names.is_empty() {
                            return Err(NewSkillMonsterLoadError::MissingMonster);
                        }
                        report.cumulative_monster_count += group.names.len();
                        report.read_monster_counts.push(report.cumulative_monster_count);
                        self.groups.insert(group.skill_id, group.names);
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(error) => return Err(NewSkillMonsterLoadError::Xml(error.to_string())),
            }
            buffer.clear();
        }
        if !root_seen {
            return Err(NewSkillMonsterLoadError::RootMissing);
        }
        if !monsterlist_seen {
            return Err(NewSkillMonsterLoadError::MissingMonsterList);
        }
        if active_group.is_some() || depth != 0 {
            return Err(NewSkillMonsterLoadError::Xml("незавершённый XML element".into()));
        }
        Ok(report)
    }

    pub(crate) fn load_from_file<ResolveName>(
        &mut self,
        path: impl AsRef<Path>,
        resolve_name: &mut ResolveName,
    ) -> Result<NewSkillMonsterLoadReport, NewSkillMonsterFileLoadError>
    where
        ResolveName: FnMut(&[u8]) -> Option<Vec<u8>>,
    {
        self.clear();
        let source = std::fs::read(path).map_err(NewSkillMonsterFileLoadError::Io)?;
        self.load_from_bytes(&source, resolve_name)
            .map_err(NewSkillMonsterFileLoadError::Format)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), NewSkillMonsterSerializeError> {
        write_count(destination, self.groups.len(), None)?;
        for (&skill_id, names) in &self.groups {
            destination.extend_from_slice(&skill_id.to_le_bytes());
            write_count(destination, names.len(), Some(skill_id))?;
            for (name_index, name) in names.iter().enumerate() {
                if name.contains(&0) {
                    return Err(NewSkillMonsterSerializeError::NameContainsNul {
                        skill_id,
                        name_index,
                    });
                }
                destination.extend_from_slice(name);
                destination.push(0);
            }
        }
        Ok(())
    }
}

struct ActiveMonsterGroup {
    skill_id: u32,
    names: Vec<Vec<u8>>,
}

impl ActiveMonsterGroup {
    fn new(skill_id: u32) -> Self {
        Self {
            skill_id,
            names: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NewSkillMonsterLoadReport {
    pub(crate) cumulative_monster_count: usize,
    pub(crate) read_monster_counts: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NewSkillMonsterLoadError {
    RootMissing,
    MissingMonsterList,
    MissingIndex,
    MissingMonster,
    MissingOriginalName,
    MonsterOutsideGroup,
    Xml(String),
}

impl fmt::Display for NewSkillMonsterLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMissing => formatter.write_str("отсутствует NewSkillMonsterList root"),
            Self::MissingMonsterList => formatter.write_str("отсутствует monsterlist"),
            Self::MissingIndex => formatter.write_str("у monsterlist отсутствует index"),
            Self::MissingMonster => formatter.write_str("monsterlist не содержит monster"),
            Self::MissingOriginalName => formatter.write_str("у monster отсутствует strorgname"),
            Self::MonsterOutsideGroup => formatter.write_str("monster находится вне monsterlist"),
            Self::Xml(error) => write!(formatter, "некорректный XML: {error}"),
        }
    }
}

impl Error for NewSkillMonsterLoadError {}

impl NewSkillMonsterLoadError {
    pub(crate) fn log_payload(&self) -> &'static [u8] {
        match self {
            Self::RootMissing | Self::Xml(_) => {
                b"error:error format of original name in NewSkillMonsterList!!"
            }
            Self::MissingIndex => b"error:there is no index number!!",
            Self::MissingOriginalName => {
                b"error:there is no original name configure in NewSkillMonsterList!!"
            }
            Self::MissingMonsterList | Self::MissingMonster | Self::MonsterOutsideGroup => {
                b"error: error format of original name in NewSkillMonsterList!!"
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum NewSkillMonsterFileLoadError {
    Io(io::Error),
    Format(NewSkillMonsterLoadError),
}

impl fmt::Display for NewSkillMonsterFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for NewSkillMonsterFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

fn read_index(start: &quick_xml::events::BytesStart<'_>) -> Result<u32, NewSkillMonsterLoadError> {
    let value = attribute(start, b"index").ok_or(NewSkillMonsterLoadError::MissingIndex)?;
    Ok(legacy_atoi(&value) as u32)
}

fn attribute(start: &quick_xml::events::BytesStart<'_>, expected: &[u8]) -> Option<Vec<u8>> {
    start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == expected)
        .map(|attribute| attribute.value.into_owned())
}

fn legacy_atoi(value: &[u8]) -> i32 {
    let value = value.iter().copied().skip_while(u8::is_ascii_whitespace);
    let mut value = value.peekable();
    let negative = matches!(value.peek(), Some(b'-'));
    if matches!(value.peek(), Some(b'-' | b'+')) {
        value.next();
    }
    let mut parsed = false;
    let mut result = 0_i32;
    for byte in value {
        let Some(digit) = byte.checked_sub(b'0').filter(|digit| *digit <= 9) else {
            break;
        };
        parsed = true;
        result = result.saturating_mul(10).saturating_add(i32::from(digit));
    }
    if !parsed {
        0
    } else if negative {
        result.saturating_neg()
    } else {
        result
    }
}

/// TinyXML принимает unquoted ASCII attributes, хотя strict XML этого не делает.
/// Превращаем только `name=value` без quotes внутри XML start-tag в quoted form;
/// comment/text bytes не меняются, а дерево обрабатывает библиотека.
fn normalize_legacy_attributes(source: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(source.len());
    let mut index = 0;
    while index < source.len() {
        if source[index..].starts_with(b"<!--") {
            let end = source[index + 4..]
                .windows(3)
                .position(|window| window == b"-->")
                .map(|offset| index + 4 + offset + 3)
                .unwrap_or(source.len());
            normalized.extend_from_slice(&source[index..end]);
            index = end;
            continue;
        }
        if source[index] != b'<' {
            normalized.push(source[index]);
            index += 1;
            continue;
        }
        normalized.push(b'<');
        index += 1;
        while index < source.len() && source[index] != b'>' {
            if source[index] != b'=' {
                normalized.push(source[index]);
                index += 1;
                continue;
            }
            normalized.push(b'=');
            index += 1;
            while index < source.len() && source[index].is_ascii_whitespace() {
                normalized.push(source[index]);
                index += 1;
            }
            if index == source.len() || matches!(source[index], b'\'' | b'"') {
                continue;
            }
            normalized.push(b'"');
            while index < source.len()
                && !source[index].is_ascii_whitespace()
                && source[index] != b'>'
                && source[index] != b'/'
            {
                normalized.push(source[index]);
                index += 1;
            }
            normalized.push(b'"');
        }
        if index < source.len() {
            normalized.push(b'>');
            index += 1;
        }
    }
    normalized
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NewSkillMonsterSerializeError {
    CountOutOfRange {
        skill_id: Option<u32>,
        count: usize,
    },
    NameContainsNul {
        skill_id: u32,
        name_index: usize,
    },
}

impl fmt::Display for NewSkillMonsterSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange {
                skill_id: Some(skill_id),
                count,
            } => write!(
                formatter,
                "skill {skill_id} содержит {count} имён вне signed 32-битного диапазона"
            ),
            Self::CountOutOfRange {
                skill_id: None,
                count,
            } => write!(
                formatter,
                "NewSkillMonster содержит {count} групп вне signed 32-битного диапазона"
            ),
            Self::NameContainsNul {
                skill_id,
                name_index,
            } => write!(
                formatter,
                "имя {name_index} в группе skill {skill_id} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for NewSkillMonsterSerializeError {}

fn write_count(
    destination: &mut Vec<u8>,
    count: usize,
    skill_id: Option<u32>,
) -> Result<(), NewSkillMonsterSerializeError> {
    let count_i32 = i32::try_from(count)
        .map_err(|_| NewSkillMonsterSerializeError::CountOutOfRange { skill_id, count })?;
    destination.extend_from_slice(&count_i32.to_le_bytes());
    Ok(())
}

// и Game decoder-а, а не как Rust-реализация.
