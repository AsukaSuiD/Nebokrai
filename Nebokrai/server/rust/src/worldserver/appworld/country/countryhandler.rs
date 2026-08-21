//! Country-map владелец исторического `WorldServer`.
//!
//! Статус `CCountryHandler::GetCountry` RVA `0x00036C40`,
//! `AddToByteArray` RVA `0x000449F0`,
//! `send_info_to_client` RVA `0x00044760`, `SendTopInfoToClient` RVA
//! `0x00044800`, `GenerateSaveData` RVA `0x00044970`,
//! `SetNewDay` RVA `0x00044A70`, `Initialize` RVA `0x00044B10`,
//! `Append` RVA `0x000452B0`,
//! `AddOneTopInfo` RVA `0x00045130` и полный `Run` RVA `0x00045040` —
//! `IMPLEMENTED`; остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:49,157,204`.
//!
//! Exact PDB задаёт `m_pCountrys` по `+0x0C` как
//! `std::map<unsigned char, CCountry*>`. Generator проходит его в unsigned
//! key-order, пропускает null country, вызывает concrete `CloneSaveData` и
//! только для non-null результата вызывает `CGame::AppendDBCountry`. Rust
//! `BTreeMap<u8, Option<Box<_>>>` сохраняет map-order и nullable value, а owned
//! `CountrySaveSnapshot` заменяет отдельный heap-object без Windows ABI.
//! Allocation failure не получает выдуманного продолжения старой null/UB
//! ветви. Полный raw-блок generator-а и inlined STL traversal удалены.
//!
//! `AddOneTopInfo` использует отдельный process-static signed ID с exact
//! initial `1` по `0x0056A9F0`, wrapping увеличивает его до clock-call, затем
//! запись добавляется в хвост. `Run` снимает отдельный wrapping tick для каждой
//! top-info записи и удаляет все `timer == 2 && param <= elapsed`, продолжая
//! обход после erase; это подтверждено exact EXE `0x00445060..0x004450AD`.
//! Затем country-map обходится в unsigned key-order. Concrete `CCountry::AI`
//! вызывается только при ненулевом king ID и непустом king name; его typed
//! report сохраняется рядом с map key. Null country исходно разыменовывался и
//! получает локальный `BLOCKED_MISSING_FACT`, а не молчаливый skip.
//! `send_info_to_client` строит `0x7FA03` из четырёх consecutive unsigned long
//! и C-строки; один overload target `0x00423C00` для обоих нулей/title/color
//! подтверждён exact EXE.
//! `SendTopInfoToClient` тем же сетевым owner-ом строит `0x7FA04` из нулевого
//! player ID, signed top-info ID, timer flag, parameter и C-строки; return
//! исходный void игнорировал, typed delivery сохраняется вызывающим adapter-ом.
//! `SetNewDay` exact `0x00444A70..0x00444B04` проходит unsigned country-key
//! map-order, пропускает null values и вызывает `CCountry::SetNewDay` с тем же
//! signed day. `BTreeMap` заменяет только MSVC tree traversal.
//! `Append` отвергает null, затем делает `operator[]` по country byte и
//! безусловно заменяет value. Rust сохраняет last-write-wins, но освобождает
//! прежний owned country вместо исходной внутренней утечки pointer-а.
//! `Initialize` exact `0x00444B10..0x00444B4D` снимает local day-of-month и
//! записывает `m_nDay` до `CDBCountry::Load`; при false немедленно возвращает
//! false, при true вызывает `SetNewDay(m_nDay)` и возвращает true. Источник
//! локального времени передан caller-ом, а DB owner/connection — явно вместо
//! process singleton и самостоятельно открываемого ADO connection.
//!
//! Initial-config wire начинается signed размером всей country-map и затем
//! содержит `CCountry` records в unsigned key-order; отдельный map key не
//! передаётся. Исходник без проверки разыменовывал null country. Safe Rust
//! останавливает эту недопустимую внутреннюю state-границу до изменения
//! destination, не выдавая старый null-dereference за протокол. `BTreeMap` и
//! owned buffer заменяют только MSVC tree/vector plumbing.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::dbaccess::worlddb::dbcountry::DbCountryOwner;
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::CMessage;
use crate::worldserver::appworld::country::country::{
    CCountry, CountryAiBlock, CountryAiReport, CountryExileResultContext,
    CountryKingSaveLimits, CountrySerializeError, CountrySetNewDayContext,
    CountrySetNewDayReport,
};
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::worldserver::game::CGame;

static NEXT_COUNTRY_TOP_INFO_ID: AtomicI32 = AtomicI32::new(1);

struct CountryTopInfo {
    id: i32,
    timer_flag: i32,
    param: i32,
    started_at_ms: u32,
    info: Vec<u8>,
}

/// Safe-граница исходного null country pointer во время `Run`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryRunBlock {
    NullCountry {
        map_key: u8,
    },
    Ai {
        map_key: u8,
        source: CountryAiBlock,
    },
}

/// Safe-границы serializer-а всей country-map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryHandlerSerializeError {
    CountryCountOutOfRange { country_count: usize },
    NullCountry { map_key: u8 },
    Country {
        map_key: u8,
        source: CountrySerializeError,
    },
}

impl fmt::Display for CountryHandlerSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountryCountOutOfRange { country_count } => write!(
                formatter,
                "CCountryHandler содержит {country_count} стран вне signed 32-битного диапазона"
            ),
            Self::NullCountry { map_key } => {
                write!(formatter, "CCountryHandler содержит null country по ключу {map_key}")
            }
            Self::Country { map_key, source } => {
                write!(formatter, "страна по ключу {map_key} не сериализована: {source}")
            }
        }
    }
}

impl Error for CountryHandlerSerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Country { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Полный ordered результат `CCountryHandler::Run`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryRunReport {
    pub(crate) expired_top_info_ids: Vec<i32>,
    pub(crate) ai_country_ids: Vec<u8>,
    pub(crate) ai_reports: Vec<(u8, CountryAiReport)>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryHandlerNewDayEntry {
    pub(crate) map_key: u8,
    pub(crate) report: CountrySetNewDayReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryHandlerNewDayReport {
    pub(crate) requested_day: i32,
    pub(crate) countries: Vec<CountryHandlerNewDayEntry>,
    pub(crate) skipped_null_country_keys: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CountryHandlerInitializeReport {
    pub(crate) local_day: i32,
    pub(crate) database_loaded: bool,
    pub(crate) new_day: Option<CountryHandlerNewDayReport>,
    pub(crate) legacy_result: bool,
}

#[derive(Debug)]
pub(crate) enum CountryAppendDisposition {
    NullRejected,
    Stored {
        country_id: u8,
        previous: Option<Box<CCountry>>,
    },
}

pub(crate) trait CountryInfoDeliveryContext {
    /// Синхронно повторяет `CMessage::SendAll`; старый return игнорировался.
    fn send_all(&mut self, message: &CMessage) -> i32;
}

/// Достигнутая save-часть исходного singleton owner-а.
pub(crate) struct CCountryHandler {
    countries: BTreeMap<u8, Option<Box<CCountry>>>,
    top_infos: VecDeque<CountryTopInfo>,
    day: i32,
}

impl CCountryHandler {
    /// Повторяет exact `Append`: null reject, затем `operator[]` overwrite.
    pub(crate) fn append_country(
        &mut self,
        country: Option<Box<CCountry>>,
    ) -> CountryAppendDisposition {
        let Some(country) = country else {
            return CountryAppendDisposition::NullRejected;
        };
        let country_id = country.country_id;
        let previous = self.countries.insert(country_id, Some(country)).flatten();
        CountryAppendDisposition::Stored {
            country_id,
            previous,
        }
    }

    /// Повторяет exact ordered map traversal `CCountryHandler::SetNewDay`.
    pub(crate) fn set_new_day<Context: CountrySetNewDayContext + ?Sized>(
        &mut self,
        requested_day: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryHandlerNewDayReport {
        let mut countries = Vec::new();
        let mut skipped_null_country_keys = Vec::new();
        for (&map_key, country) in &mut self.countries {
            let Some(country) = country.as_deref_mut() else {
                skipped_null_country_keys.push(map_key);
                continue;
            };
            countries.push(CountryHandlerNewDayEntry {
                map_key,
                report: country.set_new_day(requested_day, parameters, context),
            });
        }
        CountryHandlerNewDayReport {
            requested_day,
            countries,
            skipped_null_country_keys,
        }
    }

    /// Создаёт доказанный пустой country-map.
    pub(crate) const fn with_reached_save_state() -> Self {
        Self {
            countries: BTreeMap::new(),
            top_infos: VecDeque::new(),
            day: 0,
        }
    }

    /// Повторяет exact Initialize: local day записывается до DB load.
    pub(crate) async fn initialize<Database, Context>(
        &mut self,
        local_day: i32,
        database: &mut Database,
        active_connection: Option<&mut WorldTdsClient>,
        parameters: &mut CCountryParam,
        context: &mut Context,
    ) -> CountryHandlerInitializeReport
    where
        Database: DbCountryOwner,
        Context: CountrySetNewDayContext + ?Sized,
    {
        self.day = local_day;
        let database_loaded = database
            .load(self, parameters, active_connection)
            .await;
        let new_day = database_loaded
            .then(|| self.set_new_day(self.day, parameters, context));
        CountryHandlerInitializeReport {
            local_day,
            database_loaded,
            new_day,
            legacy_result: database_loaded,
        }
    }

    /// Дописывает точную ordered country-map для initial-config subtype `0x19`.
    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), CountryHandlerSerializeError> {
        let country_count = self.countries.len();
        let country_count_i32 = i32::try_from(country_count).map_err(|_| {
            CountryHandlerSerializeError::CountryCountOutOfRange { country_count }
        })?;

        let mut records = Vec::new();
        for (&map_key, country) in &self.countries {
            let country = country
                .as_deref()
                .ok_or(CountryHandlerSerializeError::NullCountry { map_key })?;
            country.add_to_byte_array(&mut records).map_err(|source| {
                CountryHandlerSerializeError::Country { map_key, source }
            })?;
        }

        destination.extend_from_slice(&country_count_i32.to_le_bytes());
        destination.extend_from_slice(&records);
        Ok(())
    }

    /// Возвращает живую страну по unsigned ID; ноль всегда равен `nullptr`.
    pub(crate) fn get_country(&self, country_id: u8) -> Option<&CCountry> {
        if country_id == 0 {
            return None;
        }
        self.countries.get(&country_id)?.as_deref()
    }

    pub(crate) fn get_country_mut(&mut self, country_id: u8) -> Option<&mut CCountry> {
        if country_id == 0 {
            return None;
        }
        self.countries.get_mut(&country_id)?.as_deref_mut()
    }

    /// Пишет reached `m_lCountryWarRes`; miss/null сохраняет исходный no-op.
    pub(crate) fn set_country_war_result(&mut self, country_id: u8, result: i32) -> bool {
        let Some(country) = self.get_country_mut(country_id) else {
            return false;
        };
        country.country_war_result = result;
        true
    }

    /// Строит exact `0x7FA03` и синхронно передаёт его исходному SendAll-owner-у.
    pub(crate) fn send_info_to_client<Context: CountryInfoDeliveryContext + ?Sized>(
        &self,
        info: &CStr,
        title: u32,
        color: u32,
        context: &mut Context,
    ) -> i32 {
        let mut message = CMessage::new(0x7fa03);
        message.base_mut().add_ulong(0);
        message.base_mut().add_ulong(0);
        message.base_mut().add_ulong(title);
        message.base_mut().add_ulong(color);
        message.base_mut().add_str(Some(info));
        context.send_all(&message)
    }

    /// Строит exact `0x7FA04`: нулевой player ID, top-info ID, timer flag,
    /// parameter и C-string, затем синхронный `SendAll`.
    pub(crate) fn send_top_info_to_client<Context: CountryInfoDeliveryContext + ?Sized>(
        &self,
        top_info_id: i32,
        timer_flag: i32,
        param: i32,
        info: &[u8],
        context: &mut Context,
    ) -> i32 {
        let end = info
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(info.len());
        let info = CString::new(&info[..end])
            .expect("legacy C-string prefix не содержит внутреннего NUL");
        let mut message = CMessage::new(0x7fa04);
        message.base_mut().add_long(0);
        message.base_mut().add_long(top_info_id);
        message.base_mut().add_long(timer_flag);
        message.base_mut().add_long(param);
        message.base_mut().add_str(Some(&info));
        context.send_all(&message)
    }

    /// Дописывает отдельную save-копию каждой живой страны в DB-list.
    pub(crate) fn generate_save_data(&self, game: &CGame, limits: CountryKingSaveLimits) {
        for country in self.countries.values().flatten() {
            game.append_db_country(country.clone_save_data(limits));
        }
    }

    /// Добавляет top-info в хвост и возвращает прежний process-static ID.
    pub(crate) fn add_one_top_info<GetTick>(
        &mut self,
        timer_flag: i32,
        param: i32,
        info: &[u8],
        mut get_tick: GetTick,
    ) -> i32
    where
        GetTick: FnMut() -> u32,
    {
        let id = NEXT_COUNTRY_TOP_INFO_ID.fetch_add(1, Ordering::Relaxed);
        let started_at_ms = get_tick();
        let end = info
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(info.len());
        self.top_infos.push_back(CountryTopInfo {
            id,
            timer_flag,
            param,
            started_at_ms,
            info: info[..end].to_vec(),
        });
        id
    }

    /// Выполняет top-info expiry и условные country AI в исходном порядке.
    pub(crate) fn run<GetTick, Context>(
        &mut self,
        _minute_delta: i32,
        mut get_tick: GetTick,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<CountryRunReport, CountryRunBlock>
    where
        GetTick: FnMut() -> u32,
        Context: CountryExileResultContext + ?Sized,
    {
        let mut retained = VecDeque::with_capacity(self.top_infos.len());
        let mut expired_top_info_ids = Vec::new();
        while let Some(top_info) = self.top_infos.pop_front() {
            let now_ms = get_tick();
            let expired = top_info.timer_flag == 2
                && (top_info.param as u32) <= now_ms.wrapping_sub(top_info.started_at_ms);
            if expired {
                expired_top_info_ids.push(top_info.id);
            } else {
                retained.push_back(top_info);
            }
        }
        self.top_infos = retained;

        let mut ai_country_ids = Vec::new();
        let mut ai_reports = Vec::new();
        for (&map_key, country) in &mut self.countries {
            let Some(country) = country.as_deref_mut() else {
                return Err(CountryRunBlock::NullCountry { map_key });
            };
            if country.king.id != 0 && !country.king.name.is_empty() {
                let report = country
                    .ai(parameters, &mut get_tick, context)
                    .map_err(|source| CountryRunBlock::Ai { map_key, source })?;
                ai_country_ids.push(map_key);
                ai_reports.push((map_key, report));
            }
        }
        Ok(CountryRunReport {
            expired_top_info_ids,
            ai_country_ids,
            ai_reports,
        })
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp

// ============================================================================
// FUNCTION: CCountryHandler::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.h:29
// RVA: 0x000014F0
// ADDRESS: 004014f0
// PROTOTYPE: CCountryHandler * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryHandler::GetCountry
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.h:38
// RVA: 0x00036C40
//
// Реализовано выше: unsigned key, zero/miss/null gates сохранены.
//

// ============================================================================
// FUNCTION: CCountryHandler::send_info_to_client
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:145
// RVA: 0x00044760
//
// Реализовано выше: exact `0x7FA03` wire-order и SendAll context.
//

// ============================================================================
// FUNCTION: CCountryHandler::SendTopInfoToClient
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:166
// RVA: 0x00044800
// ADDRESS: 00444800
// PROTOTYPE: void __thiscall SendTopInfoToClient(long param_1, long param_2, long param_3, char * param_4)
//
// Реализовано выше: exact `0x7FA04` wire-order и SendAll context.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryHandler::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:38
// RVA: 0x000448E0
// ADDRESS: 004448e0
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: CCountryHandler::GenerateSaveData, WorldServer RVA 0x00044970.
// Реализация находится выше; STL traversal свёрнут в provenance.

// IMPLEMENTED: CCountryHandler::AddToByteArray, WorldServer RVA 0x000449F0.
// Реализация находится выше; STL traversal свёрнут в provenance.

// ============================================================================
// FUNCTION: CCountryHandler::SetNewDay
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:124
// RVA: 0x00044A70
// ADDRESS: 00444a70
// PROTOTYPE: void __thiscall SetNewDay(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryHandler::Initialize
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:21
// RVA: 0x00044B10
// ADDRESS: 00444b10
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// VERIFIED_DISASSEMBLY, IMPLEMENTED: полный `CCountryHandler::Run` RVA
// `0x00045040` находится выше; заменённые list/tree traversals удалены.

// VERIFIED_DISASSEMBLY, IMPLEMENTED: `CCountryHandler::AddOneTopInfo` RVA
// `0x00045130` находится выше; exact static ID initial равен `1`.

// ============================================================================
// FUNCTION: CCountryHandler::~CCountryHandler
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:16
// RVA: 0x00045250
// ADDRESS: 00445250
// PROTOTYPE: void __thiscall ~CCountryHandler(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryHandler::Append
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:66
// RVA: 0x000452B0
// ADDRESS: 004452b0
// PROTOTYPE: bool __thiscall Append(CCountry * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCountryHandler::CCountryHandler
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryhandler.cpp:12
// RVA: 0x000452E0
// ADDRESS: 004452e0
// PROTOTYPE: undefined __thiscall CCountryHandler(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
