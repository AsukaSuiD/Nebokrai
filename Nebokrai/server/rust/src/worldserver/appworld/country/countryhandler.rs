//! Карта государств `CCountryHandler` из `countryhandler.cpp/.h`,
//! подтверждённая `worldserver.exe` и `worldserver.pdb`.
//!
//! `BTreeMap<u8, Option<Box<CCountry>>>` сохраняет unsigned country order и
//! nullable slots. Append заменяет прежнее значение; Rust освобождает его вместо
//! внутренней утечки. Save generation пропускает пустые slots и передаёт в БД
//! отдельные `CountrySaveSnapshot`.
//!
//! `Run` сначала удаляет истёкшие top-info по отдельным wrapping ticks, затем
//! вызывает `CCountry::AI` только для страны с ненулевым ID и именем короля.
//! Суточное обновление идёт в том же country order. Временное извлечение owner-а
//! для governance-вызова устраняет aliasing, не меняя slot между сообщениями.
//!
//! Top-info packets сохраняют исходные поля и C-строки; ошибки отправки не
//! меняют очередь. Initial-config пишет размер всей карты и records без
//! отдельного key; пустой slot блокирует сериализацию вместо null-dereference.

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

/// Наблюдаемые блоки `CCountryHandler::Run` после устранения внутреннего
/// null-slot lifecycle-дефекта.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CountryRunBlock {
    Ai {
        map_key: u8,
        source: CountryAiBlock,
    },
}

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
    fn send_all(&mut self, message: &CMessage) -> i32;
}

pub(crate) struct CCountryHandler {
    countries: BTreeMap<u8, Option<Box<CCountry>>>,
    top_infos: VecDeque<CountryTopInfo>,
    day: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryHandlerReleaseReport {
    pub(crate) released_countries: usize,
    pub(crate) released_top_infos: usize,
}

impl Default for CCountryHandler {
    fn default() -> Self {
        Self::with_reached_save_state()
    }
}

impl CCountryHandler {
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

    pub(crate) const fn with_reached_save_state() -> Self {
        Self {
            countries: BTreeMap::new(),
            top_infos: VecDeque::new(),
            day: 0,
        }
    }

 /// Потребляет singleton owner, как `Release` после удаления всех country.
 ///
 /// Старый метод virtual-удалял каждый ненулевой `CCountry*`, затем удалял
 /// сам process-global handler. Rust возвращает счётчики до обычного Drop;
 /// map/list nodes, vtable и deleting-destructor не являются контрактом.
    pub(crate) fn release(self) -> CountryHandlerReleaseReport {
        CountryHandlerReleaseReport {
            released_countries: self.countries.values().flatten().count(),
            released_top_infos: self.top_infos.len(),
        }
    }

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

 /// Временно передаёт concrete country-owner координирующему контексту.
 /// Это Rust-замена одновременного `CCountry*` и singleton handler alias.
    pub(crate) fn take_country_owner(&mut self, country_id: u8) -> Option<Box<CCountry>> {
        if country_id == 0 {
            return None;
        }
        self.countries.get_mut(&country_id)?.take()
    }

    pub(crate) fn restore_country_owner(&mut self, country_id: u8, owner: Box<CCountry>) {
        let slot = self
            .countries
            .get_mut(&country_id)
            .expect("временно снятая country сохраняет map-slot");
        assert!(slot.is_none(), "country slot не заменяется во время owner-call");
        *slot = Some(owner);
    }

    pub(crate) fn set_country_war_result(&mut self, country_id: u8, result: i32) -> bool {
        let Some(country) = self.get_country_mut(country_id) else {
            return false;
        };
        country.country_war_result = result;
        true
    }

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

 /// Строит `0x7FA04`: нулевой player ID, top-info ID, timer flag,
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

    pub(crate) fn generate_save_data(&self, game: &CGame, limits: CountryKingSaveLimits) {
        for country in self.countries.values().flatten() {
            game.append_db_country(country.clone_save_data(limits));
        }
    }

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
                continue;
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
