//! 16-байтовый идентификатор `CGUID`, восстановленный из `public/guid.h` и
//! `public/guid.cpp`.
//!
//! Статус владельца: `IMPLEMENTED`; обработка повреждённой строки точной длины
//! локально отмечена как `BLOCKED_MISSING_FACT`.
//!
//! Точные варианты и существенные адреса исходных методов:
//! - `AuthServer/authserver.exe + AuthServer/authserver.pdb`: startup
//!   `GUID_IN_ACTIVE` RVA `0x0002B7E0`;
//! - `BillingServer/billingserver.exe + BillingServer/billingserver.pdb`:
//!   конструктор RVA `0x00013280`, присваивание `0x000132A0`, startup
//!   `GUID_IN_ACTIVE` `0x0002C810`;
//! - `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`: startup
//!   `GUID_IN_ACTIVE` RVA `0x000837D0`;
//! - `MiscServer/miscserver.exe + MiscServer/miscserver.pdb`: конструктор RVA
//!   `0x00005780`, деструктор `0x000057A0`, присваивание `0x000057B0`, startup
//!   `GUID_IN_ACTIVE` `0x00023A10`;
//! - `GameServer/gameserver.exe + GameServer/GameServer.pdb`: равенство RVA
//!   `0x00012BA0`, конструктор `0x0001D3F0`, присваивание `0x0001D410`,
//!   `tostring` `0x0001D440`, `CreateGUID` `0x0001D4A0`, сырой компаратор
//!   `0x0007BBB0`; пропущенный экспортом startup подтверждён дизассемблированием
//!   по RVA `0x00249E20`;
//! - `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`: конструктор
//!   RVA `0x00055B10`, присваивание `0x00055B30`, `tostring` `0x00055B60`,
//!   `CreateGUID` `0x00055BC0`, конструктор из UTF-16 `0x00055BE0`;
//!   пропущенный экспортом startup подтверждён дизассемблированием по RVA
//!   `0x0013B1E0`.
//!
//! Исходные владельцы PDB: пары `guid.h` и `guid.cpp` в каталогах
//! `h:\fengyun\fy_russia\src\public`,
//! `d:\complite_version\fengyun_russia\trunk\public` и
//! `e:\svn\fengyun_russia_dev\public`.
//!
//! Во всех вариантах живой layout совпадает с Windows `GUID`: 16 байт с
//! выравниванием 4. Нулевой конструктор, копирование четырёх 32-битных слов и
//! равенство четырёх слов заменены `Default`, `Copy` и побайтовым равенством.
//! Исходный `guid_compare` сравнивал именно 16 байт памяти слева направо, поэтому
//! Rust `Ord` намеренно не использует канонический числовой порядок UUID.
//!
//! `GUID_INVALID` доказан как 16 нулевых байт: во всех шести PDB символы лежат
//! в zero-initialized хвосте `.data` (RVA `0x0003B26C`, `0x0015BB48`,
//! `0x001E4EF4`, `0x000504FC`, `0x00AF3D9C`, `0x002BEEF4`). Все шесть startup-
//! вариантов создают `GUID_IN_ACTIVE` из строки
//! `{C02CE7F5-35F4-482D-B927-BBC9893AC6AD}`; для Game и World это подтверждено
//! точечным дизассемблированием после отсутствия тел в сыром экспорте.
//!
//! `getrandom` заменяет системный источник случайности `CoCreateGuid`, сохраняя
//! возможность ошибки, а `uuid` заменяет установку UUID v4/variant,
//! `IIDFromString` на корректном вводе и общую UUID-механику. Его `to_bytes_le`
//! прямо гарантирует смешанный Microsoft GUID endian. Узкий слой ниже отдельно
//! сохраняет старые сырые байты, 4-байтовое выравнивание, верхний регистр,
//! фигурные скобки и сырой порядок сравнения. Это Linux-native замена без
//! Windows FFI. Старые out-параметр и `bool` выражены как `Result<CGuid, _>`:
//! все девять найденных call sites игнорировали `bool`, но Rust-владелец не
//! скрывает отказ системного RNG и оставляет реакцию будущему вызывающему.
//!
//! `nullptr` строкового конструктора выражен как `Option<&str>`. Нулевой
//! указатель и UTF-16-строка длиной не 38 единиц доказанно дают `GUID_INVALID`.
//! Для повреждённой строки ровно из 38 единиц исходный `IIDFromString` мог
//! частично изменить уже обнулённый объект и вернуть ошибку, которую C++
//! игнорировал. `uuid` не возвращает частичный разбор, поэтому Rust API оставляет
//! этот случай явной ошибкой до восстановления точной наблюдаемой реакции.
//!
//! Остальные тела из старого owner-файла классифицированы как технический шум:
//! `sprintf`, `wcslen`, `std::basic_string`, `CIni` deleting-destructor и
//! `$E/$L` cleanup заменены форматированием, парсером, владением и константами
//! Rust либо остаются у собственных владельцев. Отдельных Rust-тел для них нет.

use std::cmp::Ordering;
use std::fmt;

use uuid::{Builder, Uuid};

/// GUID в точном 16-байтовом Microsoft layout старого wire и структур сервера.
///
/// Тип имеет размер 16 и выравнивание 4. Его порядок определяется сырыми
/// legacy-байтами, а не каноническим порядком [`Uuid`].
#[repr(C, align(4))]
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub(crate) struct CGuid {
    legacy_bytes: [u8; 16],
}

const _: () = {
    assert!(std::mem::size_of::<CGuid>() == 16);
    assert!(std::mem::align_of::<CGuid>() == 4);
};

impl CGuid {
    /// Нулевое значение исходного `CGUID::GUID_INVALID`.
    pub(crate) const GUID_INVALID: Self = Self::from_legacy_bytes([0; 16]);

    /// Фиксированный служебный идентификатор исходного `GUID_IN_ACTIVE`.
    pub(crate) const GUID_IN_ACTIVE: Self = Self::from_legacy_bytes([
        0xF5, 0xE7, 0x2C, 0xC0, 0xF4, 0x35, 0x2D, 0x48, 0xB9, 0x27, 0xBB, 0xC9, 0x89, 0x3A, 0xC6,
        0xAD,
    ]);

    /// Создаёт GUID из 16 байт в исходном Microsoft mixed-endian порядке.
    pub(crate) const fn from_legacy_bytes(legacy_bytes: [u8; 16]) -> Self {
        Self { legacy_bytes }
    }

    /// Возвращает ссылку на 16 байт в исходном wire-порядке.
    pub(crate) const fn as_legacy_bytes(&self) -> &[u8; 16] {
        &self.legacy_bytes
    }

    /// Передаёт вызывающему 16 байт в исходном wire-порядке.
    pub(crate) const fn into_legacy_bytes(self) -> [u8; 16] {
        self.legacy_bytes
    }

    /// Создаёт новый случайный GUID, сохраняя Microsoft mixed-endian layout.
    ///
    /// Возвращает ошибку системного источника случайности без частичного GUID.
    pub(crate) fn create() -> Result<Self, getrandom::Error> {
        let mut random_bytes = [0; 16];
        getrandom::fill(&mut random_bytes)?;

        Ok(Self::from_uuid(
            Builder::from_random_bytes(random_bytes).into_uuid(),
        ))
    }

    /// Восстанавливает доказанную часть конструктора из nullable UTF-16 строки.
    ///
    /// `None` и значение длиной не 38 UTF-16 code units дают
    /// [`Self::GUID_INVALID`]. Корректная строка с фигурными скобками разбирается
    /// библиотекой. Ошибка для повреждённой строки точной длины должна остаться
    /// видимой вызывающему: её нельзя превращать в нулевой или частичный GUID до
    /// закрытия локального `BLOCKED_MISSING_FACT` ниже.
    pub(crate) fn from_legacy_text(value: Option<&str>) -> Result<Self, uuid::Error> {
        let Some(value) = value else {
            return Ok(Self::GUID_INVALID);
        };

        if value.encode_utf16().count() != 38 {
            return Ok(Self::GUID_INVALID);
        }

        // BLOCKED_MISSING_FACT: какие именно 16 байт оставлял IIDFromString
        // целевой Windows-версии baseline при ошибке внутри строки длиной 0x26?
        // WorldServer RVA 0x00055BE0 сохраняет частично изменённый `this`, потому
        // что HRESULT проигнорирован:
        // `if (wcslen(param_1) == 0x26) IIDFromString(param_1, this);`
        Uuid::try_parse(value).map(Self::from_uuid)
    }

    /// Возвращает `true` только для исходного нулевого `GUID_INVALID`.
    pub(crate) fn is_invalid(self) -> bool {
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
pub(crate) const NULL_GUID: CGuid = CGuid::GUID_INVALID;
