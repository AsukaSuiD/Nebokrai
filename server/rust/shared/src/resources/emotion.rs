//! Общий формат и таблица эмоций `CEmotion` для Realm и Zone.
//! Исходный владелец: `server/setup/emotion.cpp/.h`; World `Nworldserver.exe`
//! с совпадающим `WorldServer.pdb` и Game `gameserver.exe` с `GameServer.pdb`.
//! Основания формата: World `LoadSetup` VA 0x004A0CA0,
//! `Serialize` VA 0x004A03F0; Game `Unserialize` VA 0x004D80C0,
//! `IsEmotionRepeated` VA 0x004D7870.
//!
//! Загрузка не очищает таблицу: запись после `*` заменяет свой signed ID.
//! Отсутствие файла не меняет состояние; открытый файл успешен и без записей.
//! Rust сохраняет разобранный префикс при ошибке в конце текста.
//!
//! Передача: signed count и пары signed ID/value в порядке map. `BTreeMap`
//! заменяет MSVC map, а байты предоставляет владелец ресурса. Game-приёмник
//! также не очищает таблицу, применяет полные записи сразу и
//! отклоняет отрицательный count, который в оригинале небезопасно декрементился
//! до выхода за входной буфер.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::marker::read_to_marker;
use crate::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CEmotion {
    emotions: BTreeMap<i32, i32>,
}

impl CEmotion {
    pub fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), EmotionDecodeError> {
        let declared = read_wire_i32(source, cursor, "emotion count")?;
        if declared < 0 {
            return Err(EmotionDecodeError::NegativeCount { declared });
        }

        for _ in 0..declared as usize {
            let emotion_id = read_wire_i32(source, cursor, "emotion ID")?;
            let value = read_wire_i32(source, cursor, "emotion value")?;
            self.emotions.insert(emotion_id, value);
        }
        Ok(())
    }

    pub fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, EmotionFormatError> {
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let mut applied = 0;
        while read_to_marker(&mut tokens, b"*") {
            let emotion_id = read_text_i32(&mut tokens, "emotion ID")?;
            let value = read_text_i32(&mut tokens, "emotion value")?;
            self.emotions.insert(emotion_id, value);
            applied += 1;
        }
        Ok(applied)
    }

    pub fn repeated(&self, emotion_id: i32) -> i32 {
        self.emotions.get(&emotion_id).copied().unwrap_or(0)
    }

    pub fn serialize(&self, destination: &mut Vec<u8>) -> Result<(), EmotionSerializeError> {
        let count = i32::try_from(self.emotions.len()).map_err(|_| EmotionSerializeError {
            count: self.emotions.len(),
        })?;
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(count);
        for (&emotion_id, &value) in &self.emotions {
            writer.write_i32(emotion_id);
            writer.write_i32(value);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmotionDecodeError {
    Field {
        field: &'static str,
        offset: usize,
        available: usize,
    },
    NegativeCount {
        declared: i32,
    },
}

impl fmt::Display for EmotionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Field {
                field,
                offset,
                available,
            } => write!(
                formatter,
                "emotion snapshot обрывается на {field} в {offset}: нужно 4, доступно {available}"
            ),
            Self::NegativeCount { declared } => write!(
                formatter,
                "emotion snapshot содержит отрицательный count {declared}"
            ),
        }
    }
}

impl Error for EmotionDecodeError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmotionFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for EmotionFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => {
                write!(formatter, "после маркера отсутствует поле {field}")
            }
            Self::InvalidLong { field, token } => write!(
                formatter,
                "поле {field} не является signed long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for EmotionFormatError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmotionSerializeError {
    pub count: usize,
}

impl fmt::Display for EmotionSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "emotion map содержит {} записей вне signed 32-битного диапазона",
            self.count
        )
    }
}

impl Error for EmotionSerializeError {}

fn read_text_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, EmotionFormatError> {
    let token = tokens
        .next()
        .ok_or(EmotionFormatError::UnexpectedEnd { field })?;
    let text = std::str::from_utf8(token).map_err(|_| EmotionFormatError::InvalidLong {
        field,
        token: token.to_vec(),
    })?;
    text.parse::<i32>()
        .map_err(|_| EmotionFormatError::InvalidLong {
            field,
            token: token.to_vec(),
        })
}

fn read_wire_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, EmotionDecodeError> {
    LegacyReader::read_i32_from(source, cursor).map_err(|block: LegacyReadBlock| {
        EmotionDecodeError::Field {
            field,
            offset: block.offset,
            available: block.available,
        }
    })
}
