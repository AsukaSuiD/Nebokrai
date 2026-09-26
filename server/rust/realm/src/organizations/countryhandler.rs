//! Карта государств `CCountryHandler` из `countryhandler.cpp/.h`,
//! подтверждённая точной парой `worldserver.exe` и `worldserver.pdb`.
//!
//! `BTreeMap<u8, Option<Box<CCountry>>>` сохраняет unsigned country order и
//! nullable slots. Append заменяет прежнее значение; Rust освобождает его вместо
//! внутренней утечки. Save generation пропускает пустые slots и передаёт в БД
//! отдельные `CountrySaveSnapshot`; wire-контакт с `CGame` выделен в mini-trait
//! [`CountrySaveSink`], чья реализация остаётся в старом `game.rs`.
//!
//! `Run` сначала удаляет истёкшие top-info по отдельным wrapping ticks, затем
//! вызывает `CCountry::AI` только для страны с ненулевым ID и именем короля.
//! Суточное обновление идёт в том же country order. Временное извлечение owner-а
//! для governance-вызова устраняет aliasing, не меняя slot между сообщениями.
//!
//! Top-info packets сохраняют исходные поля и C-строки; ошибки отправки не
//! меняют очередь. Initial-config пишет размер всей карты и records без
//! отдельного key; пустой slot блокирует сериализацию вместо null-dereference.
//!
//! Решение по инвентарю: process-global `AtomicI32` next top-info ID оригинала
//! перенесён в поле `next_top_info_id` handler-а. Поле инициализируется нулём в
//! constructor и монотонно wrapping-инкрементируется при каждом
//! `add_one_top_info` без сброса: pre-increment от нуля выдаёт ту же
//! последовательность 1, 2, 3…, что `fetch_add` от единицы в оригинале, и так
//! же wrapping-переходит через `i32::MAX`. Один process handler оригинала
//! эквивалентен одному Rust owner-у, поэтому observable поведение не меняется.
//!
//! Run-, new-day- и initialize-отчёты живут вместе с владельцем;
//! `CountryHandlerSerializeError` дополняет `CountrySerializeError`, а
//! `CountryAppendDisposition` ссылается на саму `CCountry` и лежит с ней.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;

use crate::app::world_message::CMessage;
use crate::content::countryparam::CCountryParam;
use crate::organizations::country::{
    CCountry, CountryAiBlock, CountryAiReport, CountryExileResultContext,
    CountryKingSaveLimits, CountrySerializeError, CountrySetNewDayContext,
    CountrySetNewDayReport,
};
use crate::organizations::dbcountry::{CountrySaveSnapshot, DbCountryOwner};
use crate::persistence::rssetup::WorldTdsClient;

/// Наблюдаемые блоки `CCountryHandler::Run` после устранения внутреннего
/// null-slot lifecycle-дефекта.
#[derive(Debug, Eq, PartialEq)]
pub enum CountryRunBlock {
    Ai {
        map_key: u8,
        source: CountryAiBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryRunReport {
    pub expired_top_info_ids: Vec<i32>,
    pub ai_country_ids: Vec<u8>,
    pub ai_reports: Vec<(u8, CountryAiReport)>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryHandlerNewDayEntry {
    pub map_key: u8,
    pub report: CountrySetNewDayReport,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryHandlerNewDayReport {
    pub requested_day: i32,
    pub countries: Vec<CountryHandlerNewDayEntry>,
    pub skipped_null_country_keys: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryHandlerInitializeReport {
    pub local_day: i32,
    pub database_loaded: bool,
    pub new_day: Option<CountryHandlerNewDayReport>,
    pub legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryHandlerReleaseReport {
    pub released_countries: usize,
    pub released_top_infos: usize,
}

pub trait CountryInfoDeliveryContext {
    fn send_all(&mut self, message: &CMessage) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryHandlerSerializeError {
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

struct CountryTopInfo {
    id: i32,
    timer_flag: i32,
    param: i32,
    started_at_ms: u32,
    #[allow(
        dead_code,
        reason = "top-info payload сохраняется по исходному packet-layout; Run/expiry читают только id, timer flag, param и started_at до подключения читателя очереди"
    )]
    info: Vec<u8>,
}

/// Синк country snapshots во владельца сохранения; реализация принадлежит
/// внешнему host-у (старый пакет делегирует её `CGame`).
pub trait CountrySaveSink {
    fn append_db_country(&self, country: CountrySaveSnapshot);
}

#[derive(Debug)]
pub enum CountryAppendDisposition {
    NullRejected,
    Stored {
        country_id: u8,
        previous: Option<Box<CCountry>>,
    },
}

pub struct CCountryHandler {
    countries: BTreeMap<u8, Option<Box<CCountry>>>,
    top_infos: VecDeque<CountryTopInfo>,
    next_top_info_id: i32,
    day: i32,
}

impl Default for CCountryHandler {
    fn default() -> Self {
        Self::with_reached_save_state()
    }
}

impl CCountryHandler {
    pub fn append_country(
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

    pub fn set_new_day<Context: CountrySetNewDayContext + ?Sized>(
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

    pub const fn with_reached_save_state() -> Self {
        Self {
            countries: BTreeMap::new(),
            top_infos: VecDeque::new(),
            next_top_info_id: 0,
            day: 0,
        }
    }

 /// Потребляет singleton owner, как `Release` после удаления всех country.
 ///
 /// Старый метод virtual-удалял каждый ненулевой `CCountry*`, затем удалял
 /// сам process-global handler. Rust возвращает счётчики до обычного Drop;
 /// map/list nodes, vtable и deleting-destructor не являются контрактом.
    pub fn release(self) -> CountryHandlerReleaseReport {
        CountryHandlerReleaseReport {
            released_countries: self.countries.values().flatten().count(),
            released_top_infos: self.top_infos.len(),
        }
    }

    pub async fn initialize<Database, Context>(
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

    pub fn add_to_byte_array(
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

    pub fn get_country(&self, country_id: u8) -> Option<&CCountry> {
        if country_id == 0 {
            return None;
        }
        self.countries.get(&country_id)?.as_deref()
    }

    pub fn get_country_mut(&mut self, country_id: u8) -> Option<&mut CCountry> {
        if country_id == 0 {
            return None;
        }
        self.countries.get_mut(&country_id)?.as_deref_mut()
    }

 /// Временно передаёт concrete country-owner координирующему контексту.
 /// Это Rust-замена одновременного `CCountry*` и singleton handler alias.
    pub fn take_country_owner(&mut self, country_id: u8) -> Option<Box<CCountry>> {
        if country_id == 0 {
            return None;
        }
        self.countries.get_mut(&country_id)?.take()
    }

    pub fn restore_country_owner(&mut self, country_id: u8, owner: Box<CCountry>) {
        let slot = self
            .countries
            .get_mut(&country_id)
            .expect("временно снятая country сохраняет map-slot");
        assert!(slot.is_none(), "country slot не заменяется во время owner-call");
        *slot = Some(owner);
    }

    pub fn set_country_war_result(&mut self, country_id: u8, result: i32) -> bool {
        let Some(country) = self.get_country_mut(country_id) else {
            return false;
        };
        country.country_war_result = result;
        true
    }

    pub fn send_info_to_client<Context: CountryInfoDeliveryContext + ?Sized>(
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
    pub fn send_top_info_to_client<Context: CountryInfoDeliveryContext + ?Sized>(
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

    pub fn generate_save_data(&self, sink: &impl CountrySaveSink, limits: CountryKingSaveLimits) {
        for country in self.countries.values().flatten() {
            sink.append_db_country(country.clone_save_data(limits));
        }
    }

    pub fn add_one_top_info<GetTick>(
        &mut self,
        timer_flag: i32,
        param: i32,
        info: &[u8],
        mut get_tick: GetTick,
    ) -> i32
    where
        GetTick: FnMut() -> u32,
    {
        self.next_top_info_id = self.next_top_info_id.wrapping_add(1);
        let id = self.next_top_info_id;
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

    pub fn run<GetTick, Context>(
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
