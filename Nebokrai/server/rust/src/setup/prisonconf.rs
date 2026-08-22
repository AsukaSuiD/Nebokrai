//! Конфигурация тюрьмы исторического Miracle.
//!
//! Статус World `load_conf` RVA `0x00045DA0` и `AddToByteArray` RVA
//! `0x00046040`: `IMPLEMENTED`; Game decoder и singleton plumbing ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! SHA-256 PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:27,65`.
//!
//! Loader сначала очищает signed-char ordered map, но сохраняет прежний
//! `_pk_val_enter` при ошибке открытия. Конструктор EXE его не инициализировал,
//! поэтому safe owner выражает ещё не загруженное значение через `Option`, а
//! serializer возвращает typed boundary вместо выдуманного нуля.
//!
//! Каждая `#`-запись читает signed ID, region, два signed short и одно поле
//! `char`. Реальный `prisonconf.ini` содержит направление `-1`, однако
//! formatted extraction в `char` берёт только первый символ: observable wire
//! содержит ASCII `'-'` (`0x2D`), не числовой `0xFF`. Этот quirk сохранён
//! явно. ID сужается до младшего signed byte; duplicate key заменяется.
//!
//! Wire: `i32 pk_threshold`, signed count, затем ordered записи
//! `i8 country + i32 region + i16 x + i16 y + i8 direction` — десять байт без
//! трёх padding-байт исходного 12-байтного `PrisonParam`. `BTreeMap<i8, _>`,
//! `std::fs` и `Drop` заменяют только MSVC tree/CRFile plumbing.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PrisonParam {
    pub(crate) region: i32,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) direction: i8,
}

/// Safe owner исходного singleton state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PrisonConf {
    pk_value_enter: Option<i32>,
    prison_params: BTreeMap<i8, PrisonParam>,
}

impl PrisonConf {
    /// Очищает map до открытия, не назначая отсутствующий constructor scalar.
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, PrisonConfFileLoadError> {
        self.prison_params.clear();
        let source = std::fs::read(path).map_err(PrisonConfFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(PrisonConfFileLoadError::Format)
    }

    pub(crate) fn load_from_bytes(
        &mut self,
        source: &[u8],
    ) -> Result<usize, PrisonConfFormatError> {
        self.prison_params.clear();
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let _ignored_name = next_token(&mut tokens, "имя PK-порога")?;
        self.pk_value_enter = Some(read_i32(&mut tokens, "PK-порог входа")?);

        let mut applied = 0;
        while read_to(&mut tokens, b"#") {
            let country = read_i32(&mut tokens, "ID страны")? as i8;
            let region = read_i32(&mut tokens, "регион тюрьмы")?;
            let x = read_i16(&mut tokens, "координата X")?;
            let y = read_i16(&mut tokens, "координата Y")?;
            let direction_token = next_token(&mut tokens, "направление")?;
            let direction =
                direction_token
                    .first()
                    .copied()
                    .ok_or(PrisonConfFormatError::UnexpectedEnd {
                        field: "направление",
                    })? as i8;
            self.prison_params.insert(
                country,
                PrisonParam {
                    region,
                    x,
                    y,
                    direction,
                },
            );
            applied += 1;
        }
        Ok(applied)
    }

    pub(crate) fn get_param(&self, country: i8) -> Option<&PrisonParam> {
        self.prison_params.get(&country)
    }

    /// Missing-file ветвь loader-а очищает только map, сохраняя прежний scalar.
    pub(crate) fn clear_prison_params(&mut self) {
        self.prison_params.clear();
    }

    /// Дописывает exact compact wire без C++ struct padding.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PrisonConfSerializeError> {
        let pk_value_enter = self
            .pk_value_enter
            .ok_or(PrisonConfSerializeError::PkValueUnavailable)?;
        let count = i32::try_from(self.prison_params.len()).map_err(|_| {
            PrisonConfSerializeError::EntryCount {
                count: self.prison_params.len(),
            }
        })?;
        destination.extend_from_slice(&pk_value_enter.to_le_bytes());
        destination.extend_from_slice(&count.to_le_bytes());
        for (&country, param) in &self.prison_params {
            destination.push(country as u8);
            destination.extend_from_slice(&param.region.to_le_bytes());
            destination.extend_from_slice(&param.x.to_le_bytes());
            destination.extend_from_slice(&param.y.to_le_bytes());
            destination.push(param.direction as u8);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PrisonConfFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for PrisonConfFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => write!(formatter, "отсутствует поле {field}"),
            Self::InvalidLong { field, token } => write!(
                formatter,
                "поле {field} не является подходящим signed long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for PrisonConfFormatError {}

#[derive(Debug)]
pub(crate) enum PrisonConfFileLoadError {
    Io(std::io::Error),
    Format(PrisonConfFormatError),
}

impl fmt::Display for PrisonConfFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for PrisonConfFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrisonConfSerializeError {
    PkValueUnavailable,
    EntryCount { count: usize },
}

impl fmt::Display for PrisonConfSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PkValueUnavailable => formatter.write_str("PK-порог PrisonConf ещё не загружен"),
            Self::EntryCount { count } => write!(
                formatter,
                "PrisonConf содержит {count} записей вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for PrisonConfSerializeError {}

fn next_token<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<&'source [u8], PrisonConfFormatError> {
    tokens
        .next()
        .ok_or(PrisonConfFormatError::UnexpectedEnd { field })
}

fn read_i32<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i32, PrisonConfFormatError> {
    let token = next_token(tokens, field)?;
    parse_signed(token, field)
}

fn read_i16<'source>(
    tokens: &mut impl Iterator<Item = &'source [u8]>,
    field: &'static str,
) -> Result<i16, PrisonConfFormatError> {
    let token = next_token(tokens, field)?;
    let value = parse_signed(token, field)?;
    i16::try_from(value).map_err(|_| invalid_long(field, token))
}

fn parse_signed(token: &[u8], field: &'static str) -> Result<i32, PrisonConfFormatError> {
    let text = std::str::from_utf8(token).map_err(|_| invalid_long(field, token))?;
    text.parse::<i32>().map_err(|_| invalid_long(field, token))
}

fn invalid_long(field: &'static str, token: &[u8]) -> PrisonConfFormatError {
    PrisonConfFormatError::InvalidLong {
        field,
        token: token.to_vec(),
    }
}

// Сырой C++ ниже сохранён как локальная документация Game decoder-а,
// singleton и оставшихся call-site деталей, а не как Rust-реализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\prisonconf.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp

// ============================================================================
// FUNCTION: tagQuest::~tagQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x00061740
// ADDRESS: 00461740
// PROTOTYPE: void __thiscall ~tagQuest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::tagQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x000618C0
// ADDRESS: 004618c0
// PROTOTYPE: undefined __thiscall tagQuest(tagQuest * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x00061AA0
// ADDRESS: 00461aa0
// PROTOTYPE: tagQuest * __thiscall operator=(tagQuest * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00461fb1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x00061FB1
// ADDRESS: 00461fb1
// PROTOTYPE: undefined Catch@00461fb1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::get_inst
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.h:34
// RVA: 0x0009CD90
// ADDRESS: 0049cd90
// PROTOTYPE: PrisonConf * __cdecl get_inst(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::~PrisonConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:21
// RVA: 0x001C5460
// ADDRESS: 005c5460
// PROTOTYPE: void __thiscall ~PrisonConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:85
// RVA: 0x001C54F0
// ADDRESS: 005c54f0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::PrisonConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:17
// RVA: 0x001C55C0
// ADDRESS: 005c55c0
// PROTOTYPE: undefined __thiscall PrisonConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\prisonconf.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp

// ============================================================================
// FUNCTION: PrisonConf::get_inst
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.h:34
// RVA: 0x00001560
// ADDRESS: 00401560
// PROTOTYPE: PrisonConf * __cdecl get_inst(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::~PrisonConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:21
// RVA: 0x00045D10
// ADDRESS: 00445d10
// PROTOTYPE: void __thiscall ~PrisonConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::load_conf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:27
// RVA: 0x00045DA0
// ADDRESS: 00445da0
// PROTOTYPE: bool __thiscall load_conf(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::PrisonConf
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:17
// RVA: 0x00045FE0
// ADDRESS: 00445fe0
// PROTOTYPE: undefined __thiscall PrisonConf(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: PrisonConf::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp:65
// RVA: 0x00046040
// ADDRESS: 00446040
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::~tagQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x00066930
// ADDRESS: 00466930
// PROTOTYPE: void __thiscall ~tagQuest(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::tagQuest
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x00066BC0
// ADDRESS: 00466bc0
// PROTOTYPE: undefined __thiscall tagQuest(tagQuest * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagQuest::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x00066DA0
// ADDRESS: 00466da0
// PROTOTYPE: tagQuest * __thiscall operator=(tagQuest * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004672b1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x000672B1
// ADDRESS: 004672b1
// PROTOTYPE: undefined Catch@004672b1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052fd50
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\prisonconf.cpp
// RVA: 0x0012FD50
// ADDRESS: 0052fd50
// PROTOTYPE: undefined Unwind@0052fd50()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
