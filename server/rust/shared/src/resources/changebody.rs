//! Ограничения смены тела `CChangeBodyConf` из WorldServer/GameServer.
//! Контракт подтверждён точными `Nworldserver.exe + WorldServer.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner `setup/changebody.h/.cpp`.
//!
//! Owner очищает vector до открытия XML, принимает direct `Goods` children
//! `RestrictionsGoodsList` и пишет signed count с `u32` items. Missing `index`
//! очищает результат; diagnostics сохраняют StringTable IDs `GS1148..1151`.
//! `quick-xml` заменяет TinyXML.
//! Game decoder немедленно очищает vector и сохраняет каждый полный `u32`;
//! safe short-buffer оставляет подтверждённый decoded prefix.
//! Установленный экземпляр и его потребители остаются у владельца роли.

use std::error::Error;
use std::fmt;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CChangeBodyConf {
    restrictions_goods: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeBodyLoadError {
    MissingRootOrGoods,
    MissingIndex,
}

impl ChangeBodyLoadError {
    pub const fn string_id(self) -> &'static [u8] {
        match self {
            Self::MissingRootOrGoods => b"GS1150",
            Self::MissingIndex => b"GS1151",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeBodySerializeError {
    CountOverflow,
}

impl CChangeBodyConf {
    pub fn clear(&mut self) {
        self.restrictions_goods.clear();
    }

    pub fn restrictions_goods(&self) -> &[u32] {
        &self.restrictions_goods
    }

    pub fn load_from_bytes(&mut self, source: &[u8]) -> Result<(), ChangeBodyLoadError> {
        self.clear();
        let result = self.load_from_bytes_after_clear(source);
        if matches!(result, Err(ChangeBodyLoadError::MissingIndex)) {
            self.clear();
        }
        result
    }

    fn load_from_bytes_after_clear(&mut self, source: &[u8]) -> Result<(), ChangeBodyLoadError> {
        let mut reader = Reader::from_reader(source);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();
        let mut depth = 0usize;
        let mut root_seen = false;
        let mut goods_seen = false;

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(start)) => {
                    self.process_start(&start, depth, &mut root_seen, &mut goods_seen)?;
                    depth += 1;
                }
                Ok(Event::Empty(empty)) => {
                    self.process_start(&empty, depth, &mut root_seen, &mut goods_seen)?;
                }
                Ok(Event::End(_)) => {
                    if depth == 0 {
                        return Err(ChangeBodyLoadError::MissingRootOrGoods);
                    }
                    depth -= 1;
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(_) => return Err(ChangeBodyLoadError::MissingRootOrGoods),
            }
            buffer.clear();
        }
        if root_seen && goods_seen && depth == 0 {
            Ok(())
        } else {
            Err(ChangeBodyLoadError::MissingRootOrGoods)
        }
    }

    fn process_start(
        &mut self,
        start: &BytesStart<'_>,
        depth: usize,
        root_seen: &mut bool,
        goods_seen: &mut bool,
    ) -> Result<(), ChangeBodyLoadError> {
        let name = start.name();
        if !*root_seen {
            if name.as_ref() != b"RestrictionsGoodsList" {
                return Err(ChangeBodyLoadError::MissingRootOrGoods);
            }
            *root_seen = true;
        } else if depth == 1 && name.as_ref() == b"Goods" {
            let index = required_index(start)?;
            self.restrictions_goods.push(index);
            *goods_seen = true;
        }
        Ok(())
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), ChangeBodySerializeError> {
        let count = i32::try_from(self.restrictions_goods.len())
            .map_err(|_| ChangeBodySerializeError::CountOverflow)?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(count);
        for &goods_id in &self.restrictions_goods {
            writer.write_u32(goods_id);
        }
        Ok(())
    }

    /// Воспроизводит `CChangeBodyConf::DecordFromByteArray` GameServer.
    pub fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, ChangeBodyDecodeError> {
        self.restrictions_goods.clear();
        let count = read_wire_i32(source, cursor)?;
        self.restrictions_goods
            .try_reserve(count.max(0) as usize)
            .map_err(ChangeBodyDecodeError::Allocation)?;
        for _ in 0..count.max(0) {
            self.restrictions_goods.push(read_wire_u32(source, cursor)?);
        }
        Ok(self.restrictions_goods.len())
    }
}

#[derive(Debug)]
pub enum ChangeBodyDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    Allocation(std::collections::TryReserveError),
}

impl fmt::Display for ChangeBodyDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "ChangeBody snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::Allocation(_) => formatter.write_str("не удалось выделить ChangeBody snapshot"),
        }
    }
}

impl Error for ChangeBodyDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedEnd { .. } => None,
            Self::Allocation(source) => Some(source),
        }
    }
}

fn required_index(start: &BytesStart<'_>) -> Result<u32, ChangeBodyLoadError> {
    let value = start
        .attributes()
        .with_checks(false)
        .filter_map(Result::ok)
        .find(|attribute| attribute.key.as_ref() == b"index")
        .map(|attribute| attribute.value.into_owned())
        .ok_or(ChangeBodyLoadError::MissingIndex)?;
    Ok(legacy_atol(&value) as u32)
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

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, ChangeBodyDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(map_read_block)
}

fn read_wire_u32(source: &[u8], cursor: &mut usize) -> Result<u32, ChangeBodyDecodeError> {
    LegacyReader::read_u32_from(source, cursor).map_err(map_read_block)
}

fn map_read_block(block: LegacyReadBlock) -> ChangeBodyDecodeError {
    ChangeBodyDecodeError::UnexpectedEnd {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    }
}
