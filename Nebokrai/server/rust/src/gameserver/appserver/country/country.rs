//! Метаданные исследования оригинала; сами по себе не доказывают совместимость.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

//! Runtime-state государства `CCountry` GameServer.
//!
//! Startup wire подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходные owners
//! `worldserver/appworld/country/country.cpp` и
//! `gameserver/appserver/country/country.cpp/.h`. World пишет один byte
//! minister count, затем ordered `(job:u8, player_id:i32)`; Game не очищает
//! country-information map, заменяет king slot `1`, обнуляет slots `2..7` и
//! накладывает переданные записи с last-wins семантикой.
//!
//! `SetCountryTreasury` сохраняет local-before-send и exact World
//! `0x60314(country, selector=1, value)`; фактическую отправку выполняет
//! dispatcher после освобождения mutable country borrow. Остальные governance,
//! exile, quest и message методы владельца ниже ещё сохраняют RAW. `BTreeMap` и
//! owned state заменяют STL nodes/raw pointers.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::nets::netserver::message::CMessage;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CCountry {
    country_id: u8,
    pub(crate) treasury: i32,
    pub(crate) power: i32,
    pub(crate) tech_current_exp: i32,
    pub(crate) tech_level: i32,
    pub(crate) control_point: i32,
    pub(crate) material_point: i32,
    pub(crate) war_point: i32,
    pub(crate) country_war_result: i32,
    country_information: BTreeMap<u8, i32>,
    quest_switches: BTreeMap<u8, bool>,
    king_id: i32,
    exile_started_at_ms: BTreeMap<i32, i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryDecodeReport {
    pub(crate) country_id: u8,
    pub(crate) declared_ministers: u8,
    pub(crate) country_information_entries: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryDecodeError {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) required: usize,
    pub(crate) available: usize,
}

impl fmt::Display for CountryDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "country snapshot обрывается на {} в {}: нужно {}, доступно {}",
            self.field, self.offset, self.required, self.available
        )
    }
}

impl Error for CountryDecodeError {}

impl CCountry {
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<CountryDecodeReport, CountryDecodeError> {
        self.country_id = read_country_u8(source, cursor, "country ID")?;
        self.treasury = read_country_i32(source, cursor, "treasury")?;
        self.power = read_country_i32(source, cursor, "power")?;
        self.tech_current_exp = read_country_i32(source, cursor, "technology experience")?;
        self.tech_level = read_country_i32(source, cursor, "technology level")?;
        self.control_point = read_country_i32(source, cursor, "king control point")?;
        self.material_point = read_country_i32(source, cursor, "king material point")?;
        self.war_point = read_country_i32(source, cursor, "king war point")?;
        let king_id = read_country_i32(source, cursor, "king ID")?;
        self.country_information.insert(1, king_id);
        self.country_war_result = read_country_i32(source, cursor, "country war result")?;

        for job in 2..8 {
            self.country_information.insert(job, 0);
        }
        let declared_ministers = read_country_u8(source, cursor, "minister count")?;
        for _ in 0..declared_ministers {
            let job = read_country_u8(source, cursor, "minister job")?;
            let player_id = read_country_i32(source, cursor, "minister player ID")?;
            self.country_information.insert(job, player_id);
        }

        Ok(CountryDecodeReport {
            country_id: self.country_id,
            declared_ministers,
            country_information_entries: self.country_information.len(),
        })
    }

    pub(crate) const fn country_id(&self) -> u8 {
        self.country_id
    }

    pub(crate) fn country_information(&mut self, job: u8) -> i32 {
        *self.country_information.entry(job).or_insert(0)
    }

    /// Exact `CPlayer::get_country_identity` inner pass: `operator[]` создаёт
    /// отсутствующие slots, а первое совпадение `1..=8` побеждает.
    pub(crate) fn identity_for_player(&mut self, player_id: i32) -> u8 {
        for job in 1..=8 {
            if self.country_information(job) == player_id {
                return job;
            }
        }
        0
    }

    pub(crate) fn set_country_information(&mut self, job: u8, player_id: i32, active: u8) -> bool {
        if active == 1 {
            self.country_information.insert(job, player_id);
            self.king_id = 0;
        } else {
            self.country_information.insert(job, 0);
        }
        true
    }

    /// Exact `SetCountryTreasury`: сначала публикует новое значение, затем
    /// формирует World `0x60314(country, attribute=1, value)`. Фактическая
    /// отправка остаётся у caller-а, чтобы owned `CCountry` не держал ссылку
    /// на process/network singleton.
    pub(crate) fn set_country_treasury(&mut self, treasury: i32) -> CMessage {
        self.treasury = treasury;
        self.change_attribute_to_world_message(1, treasury)
    }

    fn change_attribute_to_world_message(&self, attribute: u8, value: i32) -> CMessage {
        let mut message = CMessage::new(0x60314);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_byte(attribute);
        message.base_mut().add_long(value);
        message
    }

    pub(crate) fn quest_switches(&self) -> &BTreeMap<u8, bool> {
        &self.quest_switches
    }

    pub(crate) fn exile_started_at_ms(&self) -> &BTreeMap<i32, i32> {
        &self.exile_started_at_ms
    }
}

fn read_country_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, CountryDecodeError> {
    Ok(take_country_bytes(source, cursor, 1, field)?[0])
}

fn read_country_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, CountryDecodeError> {
    Ok(i32::from_le_bytes(
        take_country_bytes(source, cursor, 4, field)?
            .try_into()
            .expect("country signed long уже проверен"),
    ))
}

fn take_country_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    required: usize,
    field: &'static str,
) -> Result<&'a [u8], CountryDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(required) else {
        return Err(CountryDecodeError {
            field,
            offset,
            required,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(CountryDecodeError {
            field,
            offset,
            required,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp

// ============================================================================
// FUNCTION: CCountry::SetCountryPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:144
// RVA: 0x000AC3C0
// ADDRESS: 004ac3c0
// PROTOTYPE: void __thiscall SetCountryPower(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetTechLevel
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:158
// RVA: 0x000AC400
// ADDRESS: 004ac400
// PROTOTYPE: void __thiscall SetTechLevel(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryTech
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:165
// RVA: 0x000AC420
// ADDRESS: 004ac420
// PROTOTYPE: void __thiscall SetCountryTech(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetControlPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:172
// RVA: 0x000AC440
// ADDRESS: 004ac440
// PROTOTYPE: void __thiscall SetControlPoint(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryMaterial
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:179
// RVA: 0x000AC460
// ADDRESS: 004ac460
// PROTOTYPE: void __thiscall SetCountryMaterial(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::HasJob
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:97
// RVA: 0x000AC750
// ADDRESS: 004ac750
// PROTOTYPE: uchar __thiscall HasJob(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetQuestSwitch
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:208
// RVA: 0x000AC7C0
// ADDRESS: 004ac7c0
// PROTOTYPE: bool __thiscall GetQuestSwitch(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetExileRestTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:218
// RVA: 0x000AC7F0
// ADDRESS: 004ac7f0
// PROTOTYPE: long __thiscall GetExileRestTime(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AddToExileList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:115
// RVA: 0x000ACE70
// ADDRESS: 004ace70
// PROTOTYPE: void __thiscall AddToExileList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetQuestSwitch
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp:192
// RVA: 0x000ACF70
// ADDRESS: 004acf70
// PROTOTYPE: uchar __thiscall SetQuestSwitch(uchar param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
