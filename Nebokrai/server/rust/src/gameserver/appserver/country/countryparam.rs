//! GameServer-владелец `CCountryParam`; файл сохраняет смешанный статус.
//!
//! Достигнутая `CServerRegion::GetReturnPoint` family читает
//! `m_mpMainRegions/m_mpMainRect/m_mpMainDir`, а writer этих карт является
//! prefix-частью `DecordFromByteArray` RVA `0x00020AF0`. Для неё
//! материализованы byte-exact cursor, точный пропуск предварительных 39 signed
//! scalar slots, очистка трёх карт, signed counts и записи
//! `country:u8 -> long/RECT/long`.
//! Семантика предварительных scalar slots и оставшийся country-tech/exile tail
//! не присваиваются этой узкой проекции и остаются RAW ниже.
//!
//! `BTreeMap` сохраняет `std::map` key-order. Return query повторяет три
//! mutating `operator[]`: отсутствующий country создаёт независимые нулевые
//! region/RECT/direction entries. Checked slice-boundary заменяет старый
//! безразмерный pointer read; truncated payload возвращает локальную typed
//! границу и не получает выдуманных байтов. Точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходники
//! `country/countryparam.h/.cpp`. Отдельные `std::map/_Tree` internals и
//! `Unwind@` сняты общей технической классификацией; singleton, constructor,
//! destructor, `Release` и непройденный decoder tail сохранены.

use std::collections::BTreeMap;

const COUNTRY_SCALAR_PREFIX_LEN: usize = 39 * 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryMainRect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryMainReturnPoint {
    pub(crate) region_id: i32,
    pub(crate) rect: CountryMainRect,
    pub(crate) direction: i32,
}

/// BLOCKED_MISSING_FACT: исходный decoder читает без размера buffer-а; safe
/// Rust сообщает точную недостающую границу вместо pointer UB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryParamInputBlock {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) required: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CCountryParam {
    pub(crate) main_regions: BTreeMap<u8, i32>,
    pub(crate) main_rects: BTreeMap<u8, CountryMainRect>,
    pub(crate) main_directions: BTreeMap<u8, i32>,
}

impl CCountryParam {
    /// Материализует достигнутый prefix исходного decoder-а до конца
    /// `m_mpMainDir`; cursor после успеха указывает на country-tech count.
    pub(crate) fn decode_main_return_prefix(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), CountryParamInputBlock> {
        take_country_bytes(
            source,
            cursor,
            COUNTRY_SCALAR_PREFIX_LEN,
            "39 scalar parameters",
        )?;

        self.main_regions.clear();
        self.main_rects.clear();
        self.main_directions.clear();

        let region_count = read_country_i32(source, cursor, "m_mpMainRegions count")?;
        for _ in 0..region_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mpMainRegions country")?;
            let region_id = read_country_i32(source, cursor, "m_mpMainRegions region")?;
            self.main_regions.insert(country, region_id);
        }

        let rect_count = read_country_i32(source, cursor, "m_mpMainRect count")?;
        for _ in 0..rect_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mpMainRect country")?;
            let rect = CountryMainRect {
                left: read_country_i32(source, cursor, "m_mpMainRect.left")?,
                top: read_country_i32(source, cursor, "m_mpMainRect.top")?,
                right: read_country_i32(source, cursor, "m_mpMainRect.right")?,
                bottom: read_country_i32(source, cursor, "m_mpMainRect.bottom")?,
            };
            self.main_rects.insert(country, rect);
        }

        let direction_count = read_country_i32(source, cursor, "m_mpMainDir count")?;
        for _ in 0..direction_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mpMainDir country")?;
            let direction = read_country_i32(source, cursor, "m_mpMainDir direction")?;
            self.main_directions.insert(country, direction);
        }
        Ok(())
    }

    pub(crate) fn main_return_point(&mut self, country: u8) -> CountryMainReturnPoint {
        let region_id = *self.main_regions.entry(country).or_insert(0);
        let rect = *self.main_rects.entry(country).or_default();
        let direction = *self.main_directions.entry(country).or_insert(0);
        CountryMainReturnPoint {
            region_id,
            rect,
            direction,
        }
    }
}

fn read_country_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, CountryParamInputBlock> {
    Ok(take_country_bytes(source, cursor, 1, field)?[0])
}

fn read_country_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, CountryParamInputBlock> {
    let bytes = take_country_bytes(source, cursor, 4, field)?;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("проверенный 4-байтовый срез"),
    ))
}

fn take_country_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    count: usize,
    field: &'static str,
) -> Result<&'a [u8], CountryParamInputBlock> {
    let offset = *cursor;
    let Some(end) = offset.checked_add(count) else {
        return Err(CountryParamInputBlock {
            field,
            offset,
            required: count,
            available: source.len().saturating_sub(offset),
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(CountryParamInputBlock {
            field,
            offset,
            required: count,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = end;
    Ok(bytes)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.cpp

// ============================================================================
// FUNCTION: CCountryParam::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.h:16
// RVA: 0x00001F50
// ADDRESS: 00401f50
// PROTOTYPE: CCountryParam * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryParam::~CCountryParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.cpp:24
// RVA: 0x00020910
// ADDRESS: 00420910
// PROTOTYPE: void __thiscall ~CCountryParam(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryParam::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.cpp:62
// RVA: 0x00020AF0
// IMPLEMENTED prefix main-return maps; country-tech/exile tail остаётся RAW.
// ADDRESS: 00420af0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryParam::CCountryParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.cpp:20
// RVA: 0x00020FF0
// ADDRESS: 00420ff0
// PROTOTYPE: undefined __thiscall CCountryParam(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryParam::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\countryparam.cpp:33
// RVA: 0x00021190
// ADDRESS: 00421190
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
