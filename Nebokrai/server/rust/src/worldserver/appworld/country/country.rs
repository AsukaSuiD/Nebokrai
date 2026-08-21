//! Save-владелец `CCountry` исторического `WorldServer`.
//!
//! Статус `CCountry::AddToByteArray` RVA `0x000C6E30`,
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
//! `m_lCountryWarRes` по `+0xA8`. Clone копирует country ID, treasury, power,
//! current/level-up tech exp, tech level, king identity/flags и war-result.
//! Три king-point ограничиваются соответствующими максимумами `CCountryParam`.
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
    pub(crate) country_war_result: i32,
    pub(crate) ministers: BTreeMap<u8, CountryMinisterState>,
}

impl CCountry {
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
