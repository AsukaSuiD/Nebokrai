//! Codec error-типы мирового `CShape` и его `CBaseObject`, извлечённые из
//! старого пакета в Realm `regions/`. Источник контракта — точная пара
//! `Nworldserver.exe` и `WorldServer.pdb`.
//!
//! Сохранён исходный порядок вариантов: `ShapeDecodeError::BaseObject`
//! оборачивает `BaseObjectDecodeError`, а `UnexpectedEnd` и
//! `LegacyNameOverflow` хранят те же offset-счётформы. Полный codec и
//! объектное дерево придут сюда отдельным связным шагом; этот файл — их
//! типовая опора, уже нужная семьям player/cgoods/moveshape при их переносе.

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseObjectDecodeError {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    LegacyNameOverflow {
        first_out_of_bounds_offset: usize,
    },
}

impl fmt::Display for BaseObjectDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
            Self::LegacyNameOverflow {
                first_out_of_bounds_offset,
            } => write!(
                formatter,
                "имя вышло за старый 256-байтовый буфер на offset {first_out_of_bounds_offset}"
            ),
        }
    }
}

impl Error for BaseObjectDecodeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeDecodeError {
    BaseObject(BaseObjectDecodeError),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for ShapeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseObject(error) => error.fmt(formatter),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for ShapeDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BaseObject(error) => Some(error),
            Self::UnexpectedEnd { .. } => None,
        }
    }
}

impl From<BaseObjectDecodeError> for ShapeDecodeError {
    fn from(error: BaseObjectDecodeError) -> Self {
        Self::BaseObject(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShapeTileCoordinateBlock {
    pub axis: &'static str,
    pub value_bits: u32,
}
