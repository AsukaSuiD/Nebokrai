//! Эмоции `CEmotion` из WorldServer, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Loader не очищает static map: каждая `*` запись заменяет только свой signed
//! ID. Missing file не меняет state, открытый файл успешен даже без records;
//! malformed tail сохраняет полный префикс.
//!
//! Wire — signed count и ordered пары signed ID/value. `BTreeMap` и стандартный
//! файловый ввод заменяют MSVC map/CRFile без транзакционной подмены.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CEmotion {
    emotions: BTreeMap<i32, i32>,
}

impl CEmotion {
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, EmotionFileLoadError> {
        let source = std::fs::read(path).map_err(EmotionFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(EmotionFileLoadError::Format)
    }

    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, EmotionFormatError> {
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let mut applied = 0;
        while read_to(&mut tokens, b"*") {
            let emotion_id = read_i32(&mut tokens, "emotion ID")?;
            let value = read_i32(&mut tokens, "emotion value")?;
            self.emotions.insert(emotion_id, value);
            applied += 1;
        }
        Ok(applied)
    }

    pub(crate) fn repeated(&self, emotion_id: i32) -> i32 {
        self.emotions.get(&emotion_id).copied().unwrap_or(0)
    }

    pub(crate) fn serialize(&self, destination: &mut Vec<u8>) -> Result<(), EmotionSerializeError> {
        let count = i32::try_from(self.emotions.len()).map_err(|_| EmotionSerializeError {
            count: self.emotions.len(),
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for (&emotion_id, &value) in &self.emotions {
            destination.extend_from_slice(&emotion_id.to_le_bytes());
            destination.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EmotionFormatError {
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

#[derive(Debug)]
pub(crate) enum EmotionFileLoadError {
    Io(std::io::Error),
    Format(EmotionFormatError),
}

impl fmt::Display for EmotionFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for EmotionFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EmotionSerializeError {
    pub(crate) count: usize,
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

fn read_i32<'a>(
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
