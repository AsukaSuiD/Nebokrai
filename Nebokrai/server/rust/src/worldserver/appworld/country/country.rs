//! Save-владелец `CCountry` исторического `WorldServer`.
//!
//! Статус `CCountry::SetCountryPower/SetCountryTreasury/SetCountryTech` RVA
//! `0x000A4750/0x000A4790/0x000A47D0`, `CCountry::AddToByteArray` RVA `0x000C6E30`,
//! `CCountry::CloneCountryData` RVA `0x000C9CE0` и
//! `CCountry::CloneSaveData` RVA `0x000CC470` — `IMPLEMENTED`; остальной корпус
//! ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1564,1598`.
//!
//! Exact PDB задаёт `CCountry` размером `0xB8`, country/treasury/power/tech
//! поля по `+0x4..+0x18`, `CKing` по `+0x24`, minister-map по `+0x5C` и signed
//! `m_lCountryWarRes` по `+0xA8`, а `map<long,long> ExileMap` по `+0xAC`.
//! `GetExileResTime` RVA `0x000C6D70` сначала снимает 32-битный
//! `timeGetTime`, затем ищет signed player ID и считает
//! `_exile_time - now + started_at` с машинным wrapping, делением на 1000 к
//! нулю и нижней границей ноль. Точный `SuccessExiled`
//! `0x004C7EE0..0x004C81C5` вызывает `timeGetTime`, но не использует результат
//! и не наполняет `ExileMap`; Linux-донор добавлял `try_emplace`, то есть
//! исправлял наблюдаемую ошибку оригинала. Rust сохраняет exact поведение и
//! не выдумывает запись, пока её не подтвердит другой машинный owner.
//! Clone копирует country ID, treasury, power,
//! current/level-up tech exp, tech level, king identity/flags и war-result.
//! Три king-point ограничиваются соответствующими максимумами `CCountryParam`.
//! `COfficer::_bQuestSwitch` по exact PDB находится отдельно от
//! `_bAppointed/_bSalary`; поэтому live quest-флаги короля и министров не
//! смешиваются с их DB save-проекцией.
//!
//! Minister-map обходится в unsigned key-order, но ключ источника не копируется:
//! максимум первые шесть non-null `CMinister` вставляются по собственному
//! `_byteIDType`. Повторный `_byteIDType` оставляет первую запись, как
//! `std::map::insert`; последующий `CDBCountry::Save` наблюдает только позиции
//! `2..=7`. Rust `BTreeMap`, owned byte names и `Clone` заменяют только MSVC
//! map/string/object allocation. Live null minister исходник разыменовывал;
//! safe reached-state не назначает этому UB новое поведение и представляет
//! только живого owner-а. Allocation failure остаётся политикой стандартного
//! allocator-а.
//!
//! `CloneSaveData` вызывается только `CCountryHandler::GenerateSaveData`, после
//! чего копию наблюдает `CDBCountry::Save`. Поэтому Rust меняет форму API и
//! сразу возвращает полный `CountrySaveSnapshot`; скопированный
//! `_tech_lelup_exp`, который DB-owner не читает, остаётся локально
//! зафиксированным полем live save-state, но не выдумывается в DB-контракте.
//! Raw тела двух заменённых функций и compiler/STL cleanup удалены.
//!
//! Initial-config record сохраняет только наблюдаемую Game-проекцию: country
//! ID, четыре country scalars, три king points, king ID, war-result и ordered
//! minister map `job:u8 -> player_id:i32`. `tech_level_up_exp`, имена и flags
//! в этот wire не входят. `BTreeMap` сохраняет unsigned порядок; signed count
//! проверяется до записи вместо неограниченного `size_t -> long` narrowing.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::dbcountry::{
    CountryKingSaveSnapshot, CountryMinisterSaveSnapshot, CountrySaveSnapshot,
};

use super::countryparam::{CCountryParam, CountryParameterUnavailable};
use super::king::{KingPointUpdate, set_control_point, set_material_point, set_war_point};

/// Три текущих максимума `CCountryParam`, читаемые во время clone.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CountryKingSaveLimits {
    pub(crate) control_point: i32,
    pub(crate) material_point: i32,
    pub(crate) war_point: i32,
}

/// Достигнутая live-форма одного minister owner-а.
#[derive(Clone, Debug)]
pub(crate) struct CountryMinisterState {
    pub(crate) id_type: u8,
    pub(crate) quest_switch: bool,
    pub(crate) snapshot: CountryMinisterSaveSnapshot,
}

/// Достигнутая save-часть живого `CCountry` без копирования MSVC layout.
#[derive(Clone, Debug)]
pub(crate) struct CCountry {
    pub(crate) country_id: u8,
    pub(crate) treasury: i32,
    pub(crate) power: i32,
    pub(crate) tech_current_exp: i32,
    pub(crate) tech_level_up_exp: i32,
    pub(crate) tech_level: i32,
    pub(crate) king: CountryKingSaveSnapshot,
    pub(crate) king_quest_switch: bool,
    pub(crate) country_war_result: i32,
    pub(crate) ministers: BTreeMap<u8, CountryMinisterState>,
    pub(crate) exile_started_at_ms: BTreeMap<i32, i32>,
}

/// Наблюдаемый результат exact `CCountry::GetExileResTime`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryExileTimeLookup {
    pub(crate) started_at_ms: Option<i32>,
    pub(crate) sampled_at_ms: u32,
    pub(crate) remaining_ms: i32,
    pub(crate) remaining_seconds: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryQuestSwitchTarget {
    King,
    Minister { job: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryQuestSwitchUpdate {
    pub(crate) target: CountryQuestSwitchTarget,
    pub(crate) previous: bool,
    pub(crate) applied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryScalarUpdate {
    Treasury {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    Power {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    TechnologyExperience {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    TechnologyLevel {
        requested: i32,
        previous: i32,
        applied: i32,
    },
    KingPoint(KingPointUpdate),
}

impl CCountry {
    /// Повторяет signed 32-битную арифметику `GetExileResTime` после уже
    /// снятого `timeGetTime`; отсутствие записи не требует `_exile_time`.
    pub(crate) fn exile_remaining_time(
        &self,
        player_id: i32,
        sampled_at_ms: u32,
        parameters: &CCountryParam,
    ) -> Result<CountryExileTimeLookup, CountryParameterUnavailable> {
        let Some(&started_at_ms) = self.exile_started_at_ms.get(&player_id) else {
            return Ok(CountryExileTimeLookup {
                started_at_ms: None,
                sampled_at_ms,
                remaining_ms: 0,
                remaining_seconds: 0,
            });
        };
        let exile_time_ms = parameters
            .exile_time_ms()
            .ok_or(CountryParameterUnavailable {
                field: "_exile_time",
            })?;
        let remaining_ms = exile_time_ms
            .wrapping_sub(sampled_at_ms as i32)
            .wrapping_add(started_at_ms);
        let remaining_seconds = (remaining_ms / 1_000).max(0);
        Ok(CountryExileTimeLookup {
            started_at_ms: Some(started_at_ms),
            sampled_at_ms,
            remaining_ms,
            remaining_seconds,
        })
    }

    /// Повторяет exact выбор `CKing` либо `GetMinister(2..=7)` opcode `0x60315`.
    pub(crate) fn set_quest_switch(
        &mut self,
        job: u8,
        enabled: bool,
    ) -> Option<CountryQuestSwitchUpdate> {
        if job == 1 {
            let previous = self.king_quest_switch;
            self.king_quest_switch = enabled;
            return Some(CountryQuestSwitchUpdate {
                target: CountryQuestSwitchTarget::King,
                previous,
                applied: enabled,
            });
        }
        if !(2..=7).contains(&job) {
            return None;
        }
        let minister = self.ministers.get_mut(&job)?;
        let previous = minister.quest_switch;
        minister.quest_switch = enabled;
        Some(CountryQuestSwitchUpdate {
            target: CountryQuestSwitchTarget::Minister { job },
            previous,
            applied: enabled,
        })
    }

    /// Применяет selector server opcode `0x60314` к достигнутому live-state.
    pub(crate) fn apply_server_scalar(
        &mut self,
        selector: i8,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<Option<CountryScalarUpdate>, CountryParameterUnavailable> {
        let update = match selector {
            1 => self.set_country_treasury(requested, parameters)?,
            2 => self.set_country_power(requested, parameters)?,
            3 => self.set_country_technology(requested),
            4 => {
                let previous = self.tech_level;
                let applied = requested.max(0);
                self.tech_level = applied;
                CountryScalarUpdate::TechnologyLevel {
                    requested,
                    previous,
                    applied,
                }
            }
            5 => CountryScalarUpdate::KingPoint(set_control_point(
                &mut self.king,
                requested,
                parameters,
            )?),
            6 => CountryScalarUpdate::KingPoint(set_material_point(
                &mut self.king,
                requested,
                parameters,
            )?),
            7 => CountryScalarUpdate::KingPoint(set_war_point(
                &mut self.king,
                requested,
                parameters,
            )?),
            _ => return Ok(None),
        };
        Ok(Some(update))
    }

    pub(crate) fn set_country_power(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<CountryScalarUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_country_power()
            .ok_or(CountryParameterUnavailable {
                field: "_max_country_power",
            })?;
        let previous = self.power;
        let applied = requested.max(0).min(maximum);
        self.power = applied;
        Ok(CountryScalarUpdate::Power {
            requested,
            previous,
            applied,
        })
    }

    pub(crate) fn set_country_treasury(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<CountryScalarUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_country_treasury()
            .ok_or(CountryParameterUnavailable {
                field: "_max_country_treasury",
            })?;
        let previous = self.treasury;
        let applied = requested.max(0).min(maximum);
        self.treasury = applied;
        Ok(CountryScalarUpdate::Treasury {
            requested,
            previous,
            applied,
        })
    }

    pub(crate) fn set_country_technology(&mut self, requested: i32) -> CountryScalarUpdate {
        let previous = self.tech_current_exp;
        let applied = requested.min(self.tech_level_up_exp);
        self.tech_current_exp = applied;
        CountryScalarUpdate::TechnologyExperience {
            requested,
            previous,
            applied,
        }
    }

    /// Дописывает один точный country record для `CCountryHandler` wire.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), CountrySerializeError> {
        let minister_count = self.ministers.len();
        let minister_count_i32 = i32::try_from(minister_count)
            .map_err(|_| CountrySerializeError::MinisterCountOutOfRange { minister_count })?;

        destination.push(self.country_id);
        destination.extend_from_slice(&self.treasury.to_le_bytes());
        destination.extend_from_slice(&self.power.to_le_bytes());
        destination.extend_from_slice(&self.tech_current_exp.to_le_bytes());
        destination.extend_from_slice(&self.tech_level.to_le_bytes());
        destination.extend_from_slice(&self.king.control_point.to_le_bytes());
        destination.extend_from_slice(&self.king.material_point.to_le_bytes());
        destination.extend_from_slice(&self.king.war_point.to_le_bytes());
        destination.extend_from_slice(&self.king.id.to_le_bytes());
        destination.extend_from_slice(&self.country_war_result.to_le_bytes());
        destination.extend_from_slice(&minister_count_i32.to_le_bytes());
        for (&job, minister) in &self.ministers {
            destination.push(job);
            destination.extend_from_slice(&minister.snapshot.id.to_le_bytes());
        }
        Ok(())
    }

    /// Создаёт отдельную DB-наблюдаемую копию country state.
    pub(crate) fn clone_save_data(&self, limits: CountryKingSaveLimits) -> CountrySaveSnapshot {
        let mut cloned_ministers = BTreeMap::new();
        for minister in self.ministers.values().take(6) {
            cloned_ministers
                .entry(minister.id_type)
                .or_insert_with(|| minister.snapshot.clone());
        }

        let ministers = std::array::from_fn(|index| {
            let id_type = index as u8 + 2;
            cloned_ministers.remove(&id_type)
        });

        // CloneCountryData копировал это поле, хотя единственный следующий
        // consumer CDBCountry::Save его не читал.
        let _tech_level_up_exp = self.tech_level_up_exp;

        CountrySaveSnapshot {
            country_id: self.country_id,
            treasury: self.treasury,
            power: self.power,
            tech_current_exp: self.tech_current_exp,
            tech_level: self.tech_level,
            king: CountryKingSaveSnapshot {
                id: self.king.id,
                name: self.king.name.clone(),
                appointed: self.king.appointed,
                salary_received: self.king.salary_received,
                control_point: self.king.control_point.min(limits.control_point),
                material_point: self.king.material_point.min(limits.material_point),
                war_point: self.king.war_point.min(limits.war_point),
            },
            country_war_result: self.country_war_result,
            ministers,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountrySerializeError {
    MinisterCountOutOfRange { minister_count: usize },
}

impl fmt::Display for CountrySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinisterCountOutOfRange { minister_count } => write!(
                formatter,
                "CCountry содержит {minister_count} министров вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for CountrySerializeError {}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp

// ============================================================================
// FUNCTION: Catch@00444c1a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp
// RVA: 0x00044C1A
// ADDRESS: 00444c1a
// PROTOTYPE: undefined Catch@00444c1a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryPower
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h:57
// RVA: 0x000A4750
// ADDRESS: 004a4750
// PROTOTYPE: long __thiscall SetCountryPower(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryTreasury
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h:82
// RVA: 0x000A4790
// ADDRESS: 004a4790
// PROTOTYPE: long __thiscall SetCountryTreasury(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetCountryTech
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.h:118
// RVA: 0x000A47D0
// ADDRESS: 004a47d0
// PROTOTYPE: long __thiscall SetCountryTech(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::ChangeKingControlPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1552
// RVA: 0x000C6740
// ADDRESS: 004c6740
// PROTOTYPE: long __thiscall ChangeKingControlPoint(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendWorldMsg
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1670
// RVA: 0x000C67D0
// ADDRESS: 004c67d0
// PROTOTYPE: void __thiscall SendWorldMsg(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendPrivateMsg
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1712
// RVA: 0x000C6870
// ADDRESS: 004c6870
// PROTOTYPE: void __thiscall SendPrivateMsg(char * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendBaseInfoToClient
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:52
// RVA: 0x000C6A80
// ADDRESS: 004c6a80
// PROTOTYPE: bool __thiscall SendBaseInfoToClient(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:332
// RVA: 0x000C6D30
// ADDRESS: 004c6d30
// PROTOTYPE: long __thiscall GetInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetExileResTime
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1319
// RVA: 0x000C6D70
// ADDRESS: 004c6d70
// PROTOTYPE: long __thiscall GetExileResTime(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1466
// RVA: 0x000C6DE0
// ADDRESS: 004c6de0
// PROTOTYPE: CMinister * __thiscall GetMinister(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountry::AddToByteArray, WorldServer RVA 0x000C6E30.
// Реализация находится выше; STL traversal свёрнут в provenance.

// ============================================================================
// FUNCTION: CCountry::NewTerm
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1764
// RVA: 0x000C6F40
// ADDRESS: 004c6f40
// PROTOTYPE: void __thiscall NewTerm(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SendCountryMsg
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1686
// RVA: 0x000C7090
// ADDRESS: 004c7090
// PROTOTYPE: void __thiscall SendCountryMsg(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::IsKing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:566
// RVA: 0x000C7160
// ADDRESS: 004c7160
// PROTOTYPE: bool __thiscall IsKing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::IsMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:598
// RVA: 0x000C7320
// ADDRESS: 004c7320
// PROTOTYPE: bool __thiscall IsMinister(long param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanOperate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:638
// RVA: 0x000C7520
// ADDRESS: 004c7520
// PROTOTYPE: bool __thiscall CanOperate(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Exile
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1156
// RVA: 0x000C7AD0
// ADDRESS: 004c7ad0
// PROTOTYPE: long __thiscall Exile(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SuccessExiled
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1254
// RVA: 0x000C7EE0
// ADDRESS: 004c7ee0
// PROTOTYPE: bool __thiscall SuccessExiled(long param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Silence
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1342
// RVA: 0x000C81D0
// ADDRESS: 004c81d0
// PROTOTYPE: long __thiscall Silence(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Absolve
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1411
// RVA: 0x000C8570
// ADDRESS: 004c8570
// PROTOTYPE: long __thiscall Absolve(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::HasJob
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1790
// RVA: 0x000C8820
// ADDRESS: 004c8820
// PROTOTYPE: uchar __thiscall HasJob(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AddVilTax2Treasure
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1810
// RVA: 0x000C8880
// ADDRESS: 004c8880
// PROTOTYPE: void __thiscall AddVilTax2Treasure(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AppointMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:956
// RVA: 0x000C8FA0
// ADDRESS: 004c8fa0
// PROTOTYPE: long __thiscall AppointMinister(long param_1, uchar param_2, uchar param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountry::CloneCountryData, WorldServer RVA 0x000C9CE0.
// Реализация достигнутой save-проекции находится выше.

// ============================================================================
// FUNCTION: CCountry::SetMinisterfromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1648
// RVA: 0x000C9EB0
// ADDRESS: 004c9eb0
// PROTOTYPE: void __thiscall SetMinisterfromDB(uchar param_1, CMinister * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::DeposeMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:943
// RVA: 0x000C9F90
// ADDRESS: 004c9f90
// PROTOTYPE: long __thiscall DeposeMinister(uchar param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetNewDay
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1733
// RVA: 0x000C9FD0
// ADDRESS: 004c9fd0
// PROTOTYPE: void __thiscall SetNewDay(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::~CCountry
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:35
// RVA: 0x000CA0C0
// ADDRESS: 004ca0c0
// PROTOTYPE: void __thiscall ~CCountry(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::InitialOLPlayersList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:114
// RVA: 0x000CA360
// ADDRESS: 004ca360
// PROTOTYPE: bool __thiscall InitialOLPlayersList(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Sort
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:174
// RVA: 0x000CA6E0
// ADDRESS: 004ca6e0
// PROTOTYPE: bool __thiscall Sort(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanAscend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:204
// RVA: 0x000CA830
// ADDRESS: 004ca830
// PROTOTYPE: bool __thiscall CanAscend(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CanDemise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:294
// RVA: 0x000CAC30
// ADDRESS: 004cac30
// PROTOTYPE: bool __thiscall CanDemise(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::GetPlayersList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:345
// RVA: 0x000CAD90
// ADDRESS: 004cad90
// PROTOTYPE: long __thiscall GetPlayersList(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::DeposeKing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:811
// RVA: 0x000CB030
// ADDRESS: 004cb030
// PROTOTYPE: long __thiscall DeposeKing(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:1526
// RVA: 0x000CB710
// ADDRESS: 004cb710
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::CCountry
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:28
// RVA: 0x000CB760
// ADDRESS: 004cb760
// PROTOTYPE: undefined __thiscall CCountry(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::RegisterKing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:395
// RVA: 0x000CB8F0
// ADDRESS: 004cb8f0
// PROTOTYPE: long __thiscall RegisterKing(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::SetKing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:796
// RVA: 0x000CC290
// ADDRESS: 004cc290
// PROTOTYPE: long __thiscall SetKing(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountry::Demise
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp:916
// RVA: 0x000CC320
// ADDRESS: 004cc320
// PROTOTYPE: long __thiscall Demise(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountry::CloneSaveData, WorldServer RVA 0x000CC470.
// Отдельный heap-owner заменён готовым `CountrySaveSnapshot` выше.


// ============================================================================
// FUNCTION: Unwind@0052e2b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\country.cpp
// RVA: 0x0012E2B0
// ADDRESS: 0052e2b0
// PROTOTYPE: undefined Unwind@0052e2b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
