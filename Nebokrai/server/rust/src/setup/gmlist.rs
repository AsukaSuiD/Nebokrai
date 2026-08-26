//! Операторы `CGMList` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner `setup/gmlist.cpp`.
//!
//! Wire пишет два ordered map: signed count и `name\0 + i32 level`, затем god
//! passport. Keys задают byte-лексикографический порядок и отдельно не идут.
//! Level остаётся произвольным `i32`; исходный passport равен
//! `@^$^#SDFSDslfld/$dsl2a`. Внутренний NUL или невозможный count блокирует
//! append до изменения destination.

use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

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

    /// Очищает каждую map только прямо перед её count; поэтому
    /// обрыв в GM block ещё не меняет player-GM map и passport.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), GmListDecodeError> {
        self.gm_info.clear();
        decode_gm_map(source, cursor, &mut self.gm_info)?;

        self.player_gm_info.clear();
        decode_gm_map(source, cursor, &mut self.player_gm_info)?;

        self.god_passport = read_wire_c_string(source, cursor)?;
        tracing::trace!(gm = self.gm_info.len(), player_gm = self.player_gm_info.len(), "список GM декодирован");
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

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum GmListSerializationBlock {
    #[error("CGMList {collection:?} содержит {count} записей вне signed 32-битного диапазона")]
    CountOutOfRange {
        collection: GmListCollection,
        count: usize,
    },
    #[error("CGMList {collection:?} запись {entry_index:?}: поле {field:?} содержит внутренний NUL")]
    StringContainsNul {
        collection: Option<GmListCollection>,
        entry_index: Option<usize>,
        field: GmListStringField,
    },
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum GmListDecodeError {
    #[error("GMList snapshot обрывается на {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("GMList string с {offset} не завершена нулём")]
    MissingStringTerminator {
        offset: usize,
    },
}

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

impl std::error::Error for GmListLoadError {}

fn write_gm_map(
    destination: &mut Vec<u8>,
    collection: GmListCollection,
    entries: &BTreeMap<Vec<u8>, GmInfo>,
) -> Result<(), GmListSerializationBlock> {
    let count =
        i32::try_from(entries.len()).map_err(|_| GmListSerializationBlock::CountOutOfRange {
            collection,
            count: entries.len(),
        })?;
    let mut writer = LegacyWriter::new(destination);
    writer.write_i32(count);
    for (entry_index, info) in entries.values().enumerate() {
        write_gm_string(
            writer.destination_mut(),
            Some((collection, entry_index)),
            GmListStringField::Name,
            &info.name,
        )?;
        writer.write_i32(info.level);
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
    LegacyWriter::new(destination).write_c_string(value);
    Ok(())
}

fn decode_gm_map(
    source: &[u8],
    cursor: &mut usize,
    destination: &mut BTreeMap<Vec<u8>, GmInfo>,
) -> Result<(), GmListDecodeError> {
    let count = read_wire_i32(source, cursor)?;
    for _ in 0..count.max(0) {
        let name = read_wire_c_string(source, cursor)?;
        let level = read_wire_i32(source, cursor)?;
        destination.insert(name.clone(), GmInfo { name, level });
    }
    Ok(())
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, GmListDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(map_read_block)
}

fn map_read_block(block: LegacyReadBlock) -> GmListDecodeError {
    GmListDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}

fn read_wire_c_string(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, GmListDecodeError> {
    let offset = *cursor;
    let mut reader = LegacyReader::at(source, offset).map_err(map_read_block)?;
    let value = reader
        .read_c_string(reader.remaining())
        .map_err(|_| GmListDecodeError::MissingStringTerminator { offset })?
        .to_vec();
    *cursor = reader.position();
    Ok(value)
}
