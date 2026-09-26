//! Порог honor за убийство `HonorElimilateConfig` из WorldServer/GameServer.
//! Контракт подтверждён точными `Nworldserver.exe + WorldServer.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходный владелец
//! `setup/honorelimilateconfig.cpp`.
//!
//! Двоичный формат содержит два знаковых `i32`: разницу уровней и минимальный
//! уровень. После успешного открытия загрузчик позиционно читает две пары метка/значение
//! и сохраняет уже присвоенное поле при повреждённом хвосте; отсутствующий ресурс
//! не меняет состояние. Таблицы рангов чести отправляются отдельными подтипами.
//! Декодер Game также присваивает поля последовательно; безопасная обработка короткого буфера
//! сохраняет первое поле при обрыве второго.
//! Установленный экземпляр и его потребители остаются у владельца роли.

use std::error::Error;
use std::fmt;

use crate::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HonorElimilateConfig {
    pub level_difference: i32,
    pub minimum_level: i32,
}

impl HonorElimilateConfig {
    pub fn load_from_bytes(&mut self, source: &[u8]) {
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

    pub fn add_to_byte_array(&self, destination: &mut Vec<u8>) {
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(self.level_difference);
        writer.write_i32(self.minimum_level);
    }

    pub fn decord_from_byte_array(
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
pub struct HonorEliminateDecodeError {
    pub offset: usize,
    pub needed: usize,
    pub available: usize,
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

/// `operator>>(long)` принимает десятичный токен целиком; переполнение и ошибка
/// не присваивают цель, в отличие от семантики `_atol` других владельцев INI.
fn parse_legacy_i32(token: &[u8]) -> Option<i32> {
    std::str::from_utf8(token).ok()?.parse().ok()
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, HonorEliminateDecodeError> {
    let map = |block: LegacyReadBlock| HonorEliminateDecodeError {
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    };
    let mut reader = LegacyReader::at(source, *cursor).map_err(map)?;
    let value = reader.read_i32().map_err(map)?;
    *cursor = reader.position();
    Ok(value)
}
