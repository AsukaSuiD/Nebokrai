//! Совместимый CGUID по символам public/guid.h и public/guid.cpp.
//!
//! Microsoft layout, сырое сравнение и строковое представление; генерация —
//! собственный Linux-адаптер через getrandom/uuid. Привязки EXE/PDB и границы:
//! docs/architecture/values-and-compatibility.md, раздел «GUID».
//! Неизвестный результат ошибочного IIDFromString отмечен у разбора строки.

use std::cmp::Ordering;
use std::fmt;

use uuid::{Builder, Uuid};

#[doc(no_inline)]
pub use uuid::Error as GuidParseError;

/// 16-байтовый GUID в Microsoft layout для сообщений и структур сервера.
///
/// Тип имеет размер 16 и выравнивание 4. Его порядок определяется сырыми
/// legacy-байтами, а не каноническим порядком [`Uuid`].
#[repr(C, align(4))]
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct CGuid {
    legacy_bytes: [u8; 16],
}

const _: () = {
    assert!(std::mem::size_of::<CGuid>() == 16);
    assert!(std::mem::align_of::<CGuid>() == 4);
};

impl CGuid {
    /// Нулевое значение исходного `CGUID::GUID_INVALID`.
    pub const GUID_INVALID: Self = Self::from_legacy_bytes([0; 16]);

    /// Фиксированный служебный идентификатор исходного `GUID_IN_ACTIVE`.
    pub const GUID_IN_ACTIVE: Self = Self::from_legacy_bytes([
        0xF5, 0xE7, 0x2C, 0xC0, 0xF4, 0x35, 0x2D, 0x48, 0xB9, 0x27, 0xBB, 0xC9, 0x89, 0x3A, 0xC6,
        0xAD,
    ]);

    /// Создаёт GUID из 16 байт в исходном Microsoft mixed-endian порядке.
    pub const fn from_legacy_bytes(legacy_bytes: [u8; 16]) -> Self {
        Self { legacy_bytes }
    }

    /// Возвращает ссылку на 16 байт в исходном wire-порядке.
    pub const fn as_legacy_bytes(&self) -> &[u8; 16] {
        &self.legacy_bytes
    }

    /// Передаёт вызывающему 16 байт в исходном wire-порядке.
    pub const fn into_legacy_bytes(self) -> [u8; 16] {
        self.legacy_bytes
    }

    /// Создаёт новый случайный GUID, сохраняя Microsoft mixed-endian layout.
    ///
    /// Возвращает ошибку системного источника случайности без частичного GUID.
    pub fn create() -> Result<Self, getrandom::Error> {
        let mut random_bytes = [0; 16];
        getrandom::fill(&mut random_bytes)?;

        Ok(Self::from_uuid(
            Builder::from_random_bytes(random_bytes).into_uuid(),
        ))
    }

    /// Читает nullable-строку с проверкой длины в единицах UTF-16.
    ///
    /// Отсутствие строки и длина не 38 дают нулевой GUID; ошибка разбора
    /// строки длиной 38 остаётся явной (см. локальное неизвестное ниже).
    pub fn from_legacy_text(value: Option<&str>) -> Result<Self, uuid::Error> {
        let Some(value) = value else {
            return Ok(Self::GUID_INVALID);
        };

        if value.encode_utf16().count() != 38 {
            return Ok(Self::GUID_INVALID);
        }

        // UNKNOWN: результат ошибочного IIDFromString для строки длиной 38.
        // World RVA 0x00055BE0 игнорирует HRESULT; неизвестно, какие байты
        // останутся в объекте. Здесь возвращаем ошибку без догадки о значении.
        Uuid::try_parse(value).map(Self::from_uuid)
    }

    /// Возвращает `true` только для исходного нулевого `GUID_INVALID`.
    pub fn is_invalid(self) -> bool {
        self == Self::GUID_INVALID
    }

    fn from_uuid(uuid: Uuid) -> Self {
        Self::from_legacy_bytes(uuid.to_bytes_le())
    }
}

impl Ord for CGuid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.legacy_bytes.cmp(&other.legacy_bytes)
    }
}

impl PartialOrd for CGuid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for CGuid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.legacy_bytes;
        let data1 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let data2 = u16::from_le_bytes([bytes[4], bytes[5]]);
        let data3 = u16::from_le_bytes([bytes[6], bytes[7]]);

        write!(
            formatter,
            "{{{data1:08X}-{data2:04X}-{data3:04X}-{0:02X}{1:02X}-{2:02X}{3:02X}{4:02X}{5:02X}{6:02X}{7:02X}}}",
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        )
    }
}

impl fmt::Debug for CGuid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CGuid({self})")
    }
}

/// Нулевой `NULL_GUID`, который старые translation units копировали из
/// `CGUID::GUID_INVALID` при startup.
pub const NULL_GUID: CGuid = CGuid::GUID_INVALID;
