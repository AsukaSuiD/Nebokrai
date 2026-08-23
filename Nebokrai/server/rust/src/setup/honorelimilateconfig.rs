//! Порог honor за убийство `HonorElimilateConfig` из WorldServer/GameServer.
//! Контракт подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `setup/honorelimilateconfig.cpp`.
//!
//! Wire содержит два signed `i32`: level difference и minimum level.
//! После успешного открытия loader позиционно читает две пары label/value и
//! сохраняет уже присвоенное поле при повреждённом хвосте; missing resource
//! не меняет state. Honor rank tables отправляются отдельными subtypes.
//! Game decoder также присваивает поля последовательно; safe short-buffer
//! сохраняет первое поле при обрыве второго.

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct HonorElimilateConfig {
    pub(crate) level_difference: i32,
    pub(crate) minimum_level: i32,
}

impl HonorElimilateConfig {
    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) {
        let mut tokens = source
            .split(|byte: &u8| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        let _ = tokens.next();
        if let Some(value) = tokens.next().and_then(parse_legacy_i32) {
            self.level_difference = value;
        }
        let _ = tokens.next();
        if let Some(value) = tokens.next().and_then(parse_legacy_i32) {
            self.minimum_level = value;
        }
    }

    pub(crate) fn add_to_byte_array(&self, destination: &mut Vec<u8>) {
        destination.extend_from_slice(&self.level_difference.to_le_bytes());
        destination.extend_from_slice(&self.minimum_level.to_le_bytes());
    }

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), HonorEliminateDecodeError> {
        self.level_difference = read_wire_i32(source, cursor)?;
        self.minimum_level = read_wire_i32(source, cursor)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HonorEliminateDecodeError {
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

impl fmt::Display for HonorEliminateDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HonorEliminate snapshot обрывается на {}: нужно {}, доступно {}",
            self.offset, self.needed, self.available
        )
    }
}

impl Error for HonorEliminateDecodeError {}

/// `operator>>(long)` принимает десятичный token целиком; overflow/failure
/// не присваивают target, в отличие от `_atol`-семантики других INI owner-ов.
fn parse_legacy_i32(token: &[u8]) -> Option<i32> {
    std::str::from_utf8(token).ok()?.parse().ok()
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, HonorEliminateDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
        return Err(HonorEliminateDecodeError {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes
            .try_into()
            .expect("HonorEliminate scalar содержит четыре байта"),
    ))
}
