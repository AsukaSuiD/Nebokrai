//! Метаданные исследования оригинала; сами по себе не доказывают совместимость.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

//! Состояние государства `CCountry` во время работы GameServer.
//!
//! Формат запуска подтверждён точными `worldserver.exe + worldserver.pdb` и
//! `gameserver.exe + GameServer.pdb`; исходные владельцы
//! `worldserver/appworld/country/country.cpp` и
//! `gameserver/appserver/country/country.cpp/.h`. World пишет один байт числа
//! министров, затем упорядоченные `(job:u8, player_id:i32)`; Game не очищает
//! карту государственных должностей, заменяет ячейку короля `1`, обнуляет
//! ячейки `2..7` и накладывает переданные записи по правилу «последняя
//! побеждает».
//!
//! `HasJob` возвращает должность первого совпадения в упорядоченном обходе
//! идентификаторов игроков либо ноль. `SetCountryTreasury` сохраняет локальное
//! изменение до отправки и точное сообщение World
//! `0x60314(country, selector=1, value)`; `0x7FF04/05` обновляют состояния
//! `CI` и короля до публикации клиенту. Фактическую отправку выполняет
//! диспетчер после освобождения изменяемого заимствования страны.
//! `AddToExileList` заменяет
//! прежнюю запись единственной текущей выборкой `timeGetTime` и сохраняет
//! упорядоченный обход идентификаторов игроков для `0x6030E/0x7FF15`.
//! Переключатель заданий публикует World `0x60315` до изменения локальной
//! карты, как исходный `SetQuestSwitch`; чтение не создаёт отсутствующий ключ.
//! Скалярные записи силы, уровня и опыта технологии, управления и материалов
//! сохраняют локальное изменение до отправки и селекторы `2/4/3/5/6` общего
//! сообщения World `0x60314`. Остаток времени изгнания использует одну выборку
//! `DWORD` с переполнением, знаковые миллисекунды, деление с усечением и
//! нижнюю границу ноль. `9014 / UpGradeTechLevel` в точном EXE только проверяет
//! наличие страны игрока и не изменяет состояние. Остальные методы управления
//! и сообщений ниже ещё сохраняют RAW. `BTreeMap` и владеющее состояние
//! заменяют узлы и указатели STL.

use std::collections::BTreeMap;
use thiserror::Error;

use crate::nets::netserver::message::CMessage;
use super::super::legacycodec::LegacyReader;

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

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("country snapshot обрывается на {field} в {offset}: нужно {required}, доступно {available}")]
pub(crate) struct CountryDecodeError {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) required: usize,
    pub(crate) available: usize,
}

impl CCountry {
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), CountryDecodeError> {
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

        tracing::trace!(country_id = self.country_id, declared_ministers, country_information_entries = self.country_information.len(), "состояние страны декодировано");
        Ok(())
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

    pub(crate) fn has_job(&self, player_id: i32) -> u8 {
        self.country_information
            .iter()
            .find_map(|(&job, &owner_id)| (owner_id == player_id).then_some(job))
            .unwrap_or(0)
    }

    pub(crate) fn set_country_information(
        &mut self,
        job: u8,
        player_id: i32,
        active: u8,
    ) {
        let previous_king_id = self.king_id;
        let applied_player_id = if active == 1 { player_id } else { 0 };
        let previous_player_id = self
            .country_information
            .insert(job, applied_player_id)
            .unwrap_or(0);
        if active == 1 {
            self.king_id = 0;
        }
        tracing::trace!(country_id = self.country_id, job, player_id, active, previous_player_id, applied_player_id, previous_king_id, applied_king_id = self.king_id, "сведения о стране изменены");
    }

    pub(crate) fn set_king_id(&mut self, king_id: i32) {
        let previous = self.king_id;
        self.king_id = king_id;
        tracing::trace!(country_id = self.country_id, previous, applied = king_id, "правитель страны изменён");
    }

    pub(crate) const fn king_id(&self) -> i32 {
        self.king_id
    }

    pub(crate) fn add_to_exile_list(
        &mut self,
        player_id: i32,
        sampled_at_ms: u32,
    ) {
        let applied_started_at_ms = sampled_at_ms as i32;
        let previous_started_at_ms = self
            .exile_started_at_ms
            .insert(player_id, applied_started_at_ms);
        tracing::trace!(country_id = self.country_id, player_id, ?previous_started_at_ms, applied_started_at_ms, "запись изгнания изменена");
    }

    pub(crate) fn exile_player_ids(&self) -> Vec<i32> {
        self.exile_started_at_ms.keys().copied().collect()
    }

    pub(crate) fn exile_rest_time(
        &self,
        player_id: i32,
        sampled_at_ms: u32,
        exile_time_ms: Option<i32>,
    ) -> Result<i32, &'static str> {
        let Some(&started_at_ms) = self.exile_started_at_ms.get(&player_id) else {
            tracing::trace!(country_id = self.country_id, player_id, sampled_at_ms, "запись изгнания отсутствует");
            return Ok(0);
        };
        let exile_time_ms = exile_time_ms.ok_or("_exile_time")?;
        let remaining_ms = exile_time_ms
            .wrapping_sub(sampled_at_ms as i32)
            .wrapping_add(started_at_ms);
        let remaining_seconds = (remaining_ms / 1_000).max(0);
        tracing::trace!(country_id = self.country_id, player_id, started_at_ms, sampled_at_ms, remaining_ms, remaining_seconds, "остаток времени изгнания вычислен");
        Ok(remaining_seconds)
    }

    pub(crate) fn quest_switch_message(&self, job: u8, enabled: bool) -> CMessage {
        let mut message = CMessage::new(0x0006_0315);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_byte(job);
        message.base_mut().add_byte(u8::from(enabled));
        message
    }

    /// Caller отправляет `quest_switch_message` до этого вызова, сохраняя exact
    /// `SetQuestSwitch` ordering: World side effect предшествует local map write.
    pub(crate) fn apply_quest_switch(
        &mut self,
        job: u8,
        enabled: bool,
    ) {
        let previous = self.quest_switches.insert(job, enabled);
        tracing::trace!(country_id = self.country_id, job, ?previous, applied = enabled, "переключатель задания страны изменён");
    }

    /// Exact `SetCountryTreasury`: сначала публикует новое значение, затем
    /// формирует World `0x60314(country, attribute=1, value)`. Фактическая
    /// отправка остаётся у caller-а, чтобы owned `CCountry` не держал ссылку
    /// на process/network singleton.
    pub(crate) fn set_country_treasury(&mut self, treasury: i32) -> CMessage {
        self.treasury = treasury;
        self.change_attribute_to_world_message(1, treasury)
    }

    pub(crate) fn set_script_scalar(
        &mut self,
        selector: u8,
        value: i32,
    ) -> CMessage {
        let previous = match selector {
            1 => std::mem::replace(&mut self.treasury, value),
            2 => std::mem::replace(&mut self.power, value),
            3 => std::mem::replace(&mut self.tech_current_exp, value),
            4 => std::mem::replace(&mut self.tech_level, value),
            5 => std::mem::replace(&mut self.control_point, value),
            6 => std::mem::replace(&mut self.material_point, value),
            _ => unreachable!("CCountry scalar selector ограничен exact owner-ами"),
        };
        tracing::trace!(country_id = self.country_id, selector, previous, applied = value, "скаляр страны изменён");
        self.change_attribute_to_world_message(selector, value)
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

    pub(crate) fn quest_switch(&self, job: u8) -> bool {
        self.quest_switches.get(&job).copied().unwrap_or(false)
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
    let mut reader = country_reader(source, *cursor, field, 1)?;
    let value = reader.read_u8().map_err(|block| country_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_country_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, CountryDecodeError> {
    let mut reader = country_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| country_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn country_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    required: usize,
) -> Result<LegacyReader<'source>, CountryDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| CountryDecodeError {
        field,
        offset: block.offset,
        required,
        available: block.available,
    })
}

fn country_error(
    field: &'static str,
    block: super::super::legacycodec::LegacyReadBlock,
) -> CountryDecodeError {
    CountryDecodeError {
        field,
        offset: block.offset,
        required: block.needed,
        available: block.available,
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\country\country.cpp

// ============================================================================
// FUNCTION: CCountry::SetCountryPower
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// FUNCTION: CCountry::GetQuestSwitch
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
// STATUS: IMPLEMENTED
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
