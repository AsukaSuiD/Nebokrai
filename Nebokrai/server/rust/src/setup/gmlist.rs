//! Списки операторов и игровых персонажей с операторскими правами.
//!
//! Контракт World `CGMList::AddToByteArray`:;
//! loaders, accessors и Game decoder не входят в этот owner и остаются. Точная
//! Исходный owner PDB:
//!
//! Оригинал World serializer и Game decoder подтверждают wire: signed count и
//! ordered records `C-string name + i32 level` сначала для общего GM map,
//! затем для player GM map, после них — C-string god passport. Ключ карты
//! отдельно не передаётся. `BTreeMap<Vec<u8>, _>` заменяет
//! `std::map<std::string, _>` и сохраняет его лексикографический byte-order;
//! owned bytes заменяют C++ string lifetime. Уровень намеренно остаётся
//! полным `i32`: известные enum-значения не дают права отвергать иное значение
//! из данных. Точный EXE также подтверждает исходный god passport
//! `@^$^#SDFSDslfld/$dsl2a`; чтение `gmlist.ini` и `data/temp.ini` остаётся
//! отдельным loader-проходом. Невозможный signed count и внутренний NUL
//! блокируют весь append до изменения destination.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const DEFAULT_GOD_PASSPORT: &[u8] = b"@^$^#SDFSDslfld/$dsl2a";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct GmInfo {
    pub(crate) name: Vec<u8>,
    pub(crate) level: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGMList {
    gm_info: BTreeMap<Vec<u8>, GmInfo>,
    player_gm_info: BTreeMap<Vec<u8>, GmInfo>,
    god_passport: Vec<u8>,
}

impl Default for CGMList {
    fn default() -> Self {
        Self {
            gm_info: BTreeMap::new(),
            player_gm_info: BTreeMap::new(),
            god_passport: DEFAULT_GOD_PASSPORT.to_vec(),
        }
    }
}

impl CGMList {
    pub(crate) fn insert_gm(&mut self, info: GmInfo) -> Option<GmInfo> {
        self.gm_info.insert(info.name.clone(), info)
    }

    pub(crate) fn insert_player_gm(&mut self, info: GmInfo) -> Option<GmInfo> {
        self.player_gm_info.insert(info.name.clone(), info)
    }

    pub(crate) fn gm_info(&self) -> &BTreeMap<Vec<u8>, GmInfo> {
        &self.gm_info
    }

    pub(crate) fn player_gm_info(&self) -> &BTreeMap<Vec<u8>, GmInfo> {
        &self.player_gm_info
    }

    pub(crate) fn god_passport(&self) -> &[u8] {
        &self.god_passport
    }

    pub(crate) fn set_god_passport(&mut self, god_passport: Vec<u8>) {
        self.god_passport = god_passport;
    }

 /// Загружает один из двух оригинал whitespace-списков World GM.
 /// Неизвестные role-имена, как и в EXE, не создают map-entry.
    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
        collection: GmListCollection,
        passport_source: Option<&[u8]>,
    ) -> Result<usize, GmListLoadError> {
        let destination = match collection {
            GmListCollection::Gm => &mut self.gm_info,
            GmListCollection::PlayerGm => &mut self.player_gm_info,
        };
        destination.clear();
        let tokens: Vec<&[u8]> = source
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .collect();
        if tokens.len() % 2 != 0 {
            return Err(GmListLoadError::MissingRole {
                name: tokens.last().copied().unwrap_or_default().to_vec(),
            });
        }
        for pair in tokens.chunks_exact(2) {
            let level = match (collection, pair[1]) {
                (GmListCollection::Gm, b"admin") => Some(100),
                (GmListCollection::Gm, b"arch") => Some(90),
                (_, b"wizard") => Some(50),
                (_, b"guardian") => Some(40),
                (_, b"moderator") => Some(30),
                _ => None,
            };
            if let Some(level) = level {
                let info = GmInfo {
                    name: pair[0].to_vec(),
                    level,
                };
                destination.insert(info.name.clone(), info);
            }
        }
        if let Some(passport) = passport_source.and_then(|bytes| {
            bytes
                .split(|byte| byte.is_ascii_whitespace())
                .find(|token| !token.is_empty())
        }) {
            self.god_passport = passport.to_vec();
        }
        Ok(destination.len())
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), GmListSerializationBlock> {
        let mut payload = Vec::new();
        write_gm_map(&mut payload, GmListCollection::Gm, &self.gm_info)?;
        write_gm_map(
            &mut payload,
            GmListCollection::PlayerGm,
            &self.player_gm_info,
        )?;
        write_gm_string(
            &mut payload,
            None,
            GmListStringField::GodPassport,
            &self.god_passport,
        )?;
        destination.extend_from_slice(&payload);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmListCollection {
    Gm,
    PlayerGm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GmListStringField {
    Name,
    GodPassport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmListSerializationBlock {
    CountOutOfRange {
        collection: GmListCollection,
        count: usize,
    },
    StringContainsNul {
        collection: Option<GmListCollection>,
        entry_index: Option<usize>,
        field: GmListStringField,
    },
}

impl fmt::Display for GmListSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { collection, count } => write!(
                formatter,
                "CGMList {collection:?} содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                collection,
                entry_index,
                field,
            } => write!(
                formatter,
                "CGMList {collection:?} запись {entry_index:?}: поле {field:?} содержит внутренний NUL"
            ),
        }
    }
}

impl Error for GmListSerializationBlock {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GmListLoadError {
    MissingRole { name: Vec<u8> },
}

impl fmt::Display for GmListLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRole { name } => write!(
                formatter,
                "CGMList: для записи {:?} отсутствует role",
                String::from_utf8_lossy(name)
            ),
        }
    }
}

impl Error for GmListLoadError {}

fn write_gm_map(
    destination: &mut Vec<u8>,
    collection: GmListCollection,
    entries: &BTreeMap<Vec<u8>, GmInfo>,
) -> Result<(), GmListSerializationBlock> {
    let count = i32::try_from(entries.len()).map_err(|_| {
        GmListSerializationBlock::CountOutOfRange {
            collection,
            count: entries.len(),
        }
    })?;
    destination.extend_from_slice(&count.to_le_bytes());
    for (entry_index, info) in entries.values().enumerate() {
        write_gm_string(
            destination,
            Some((collection, entry_index)),
            GmListStringField::Name,
            &info.name,
        )?;
        destination.extend_from_slice(&info.level.to_le_bytes());
    }
    Ok(())
}

fn write_gm_string(
    destination: &mut Vec<u8>,
    entry: Option<(GmListCollection, usize)>,
    field: GmListStringField,
    value: &[u8],
) -> Result<(), GmListSerializationBlock> {
    if value.contains(&0) {
        return Err(GmListSerializationBlock::StringContainsNul {
            collection: entry.map(|(collection, _)| collection),
            entry_index: entry.map(|(_, entry_index)| entry_index),
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}
