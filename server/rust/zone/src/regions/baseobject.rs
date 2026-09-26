//! Базовая идентичность объектов GameServer (`CBaseObject`): type/ID/GUID,
//! graphics ID, byte-string имя, constructor-defaults и wire codec. Исходники
//! `appserver/baseobject.h/.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Фабрики npc/monster остаются в старом пакете связными с
//! ним проводами.
//!
//! Codec пишет три little-endian long, затем имя с NUL; `Vec<u8>` заменяет
//! `std::string` без навязывания UTF-8, а decoder останавливает отсутствие NUL
//! и выход за старый 256-байтовый буфер локальным `BLOCKED_MISSING_FACT`,
//! сохраняя уже прочитанные поля и cursor. Identity helpers: верхний DWORD
//! хранит type с исходным sign-extension отрицательного ID. Child-list/father
//! ownership и полный destructor не материализованы: helper строит только
//! достигнутую region-chain часть и не объявляет Rust layout копией старого ABI.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#npc-и-базовые-фигуры

use std::fmt;
use thiserror::Error;

use nebokrai_shared::values::CGuid;

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};

const LEGACY_NAME_CAPACITY: usize = 0x100;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum BaseObjectDecodeError {
    #[error("base object обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("base object name вышло за legacy buffer в {first_out_of_bounds_offset}")]
    LegacyNameOverflow { first_out_of_bounds_offset: usize },
}

#[derive(Clone, Eq, PartialEq)]
pub struct CBaseObject {
    object_type: i32,
    id: i32,
    ex_id: CGuid,
    graphics_id: i32,
    name: Vec<u8>,
    pub include_child: bool,
}

impl fmt::Debug for CBaseObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CBaseObject")
            .field("object_type", &self.object_type)
            .field("id", &self.id)
            .field("graphics_id", &self.graphics_id)
            .field("name", &self.name)
            .field("include_child", &self.include_child)
            .finish_non_exhaustive()
    }
}

impl CBaseObject {
    pub const fn with_reached_constructor_defaults() -> Self {
        Self {
            object_type: 0,
            id: 0,
            ex_id: CGuid::GUID_INVALID,
            graphics_id: 0,
            name: Vec::new(),
            include_child: true,
        }
    }

    pub const fn get_type(&self) -> i32 {
        self.object_type
    }

    pub const fn set_type(&mut self, object_type: i32) {
        self.object_type = object_type;
    }

    pub const fn get_id(&self) -> i32 {
        self.id
    }

    pub const fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub const fn get_ex_id(&self) -> CGuid {
        self.ex_id
    }

    pub const fn set_ex_id(&mut self, ex_id: CGuid) {
        self.ex_id = ex_id;
    }

    pub const fn get_graphics_id(&self) -> i32 {
        self.graphics_id
    }

    pub const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.graphics_id = graphics_id;
    }

    pub const fn get_hash_value(object_type: i32, id: i32) -> i64 {
        let high = object_type | (id >> 31);
        ((high as u32 as u64) << 32 | id as u32 as u64) as i64
    }

    pub const fn calculate_type(hash: i64) -> i32 {
        (hash >> 32) as i32
    }

    pub const fn calculate_id(hash: i64) -> i32 {
        hash as i32
    }

    pub fn get_name(&self) -> &[u8] {
        &self.name
    }

    pub fn set_name(&mut self, name: &[u8]) {
        let prefix_len = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.name.clear();
        self.name.extend_from_slice(&name[..prefix_len]);
    }

    pub fn add_to_byte_array(&self, destination: &mut Vec<u8>, _include_child: bool) -> bool {
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(self.object_type);
        writer.write_i32(self.id);
        writer.write_i32(self.graphics_id);
        writer.write_c_string(&self.name);
        true
    }

    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
    ) -> Result<bool, BaseObjectDecodeError> {
        self.object_type = read_i32(source, cursor, "m_lType")?;
        self.id = read_i32(source, cursor, "m_lID")?;
        self.graphics_id = read_i32(source, cursor, "m_lGraphicsID")?;
        self.name = read_name(source, cursor)?;
        Ok(true)
    }
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, BaseObjectDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        BaseObjectDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader
        .read_i32()
        .map_err(|block| BaseObjectDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_name(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, BaseObjectDecodeError> {
    let mut name = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = LegacyReader::at(source, offset).map_err(|block| {
            BaseObjectDecodeError::UnexpectedEnd {
                field: "m_strName",
                offset: block.offset,
                needed: 1,
                available: block.available,
            }
        })?;
        let byte = reader
            .read_u8()
            .map_err(|block| BaseObjectDecodeError::UnexpectedEnd {
                field: "m_strName",
                offset: block.offset,
                needed: block.needed,
                available: block.available,
            })?;
        *cursor = reader.position();
        if name.len() == LEGACY_NAME_CAPACITY {
            // BLOCKED_MISSING_FACT: этот байт уже выходил за local char[256].
            return Err(BaseObjectDecodeError::LegacyNameOverflow {
                first_out_of_bounds_offset: offset,
            });
        }
        if byte == 0 {
            return Ok(name);
        }
        name.push(byte);
    }
}
