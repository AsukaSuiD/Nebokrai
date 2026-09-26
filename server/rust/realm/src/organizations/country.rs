//! Государство `CCountry` из `country.cpp/.h`, подтверждённое точной парой
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Владелец хранит казну, силу, технологии, короля, министров, дневные лимиты
//! и состояние country war. Административные операции сохраняют порядок
//! проверок, списания control points, wrapping-счётчиков, сообщений королю и
//! рассылок GameServer; уже выполненные изменения при позднем отказе не
//! откатываются. Exile, silence, absolve, назначение и смещение министров,
//! передача престола и регистрация короля намеренно сохраняют исторические
//! особенности: `map::operator[]` может создать пустой minister slot,
//! `IsMinister` ищет по всем должностям, а отдельные проверки используют
//! player ID вместо faction ID.
//!
//! `AI` и смена дня используют 32-битные wrapping ticks; налоги деревень
//! начисляются в порядке region ID с исходными ограничениями казны; country-
//! сообщения идут только подключённым GameServer, private target `0` — король.
//! Initial-config и save snapshots различаются составом полей и порядком
//! министров; initial-config передаёт minister count одним byte. `BTreeMap`,
//! owned-строки и ограниченное форматирование заменяют STL, сырые указатели и
//! переполнение буферов, не меняя wire и БД.
//!
//! Data king points (`KingPointKind`, `KingPointUpdate`) лежат здесь, рядом с
//! использующей их `CCountry`; сам `CKing` — в соседнем `organizations/king`.
//!
//! Контекстные трейты `CCountry` (`CountryNewTermContext`,
//! `CountryVillageTaxContext`, `CountrySetNewDayContext`,
//! `CountryExileResultContext`, `CountryHasJobContext`,
//! `CountryPlayersListContext`) ссылаются только на data-типы этого файла,
//! `CMessage` и `SendMessageError` realm — сигнатуры и дефолтные impl
//! сохранены буквально. `CCountry` работает с теми же трейтами напрямую;
//! game-контакты `CCountryHandler::generate_save_data` выделены в mini-trait
//! [`CountrySaveSink`](crate::organizations::countryhandler::CountrySaveSink).

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::ffi::CString;
use std::fmt;

use crate::app::world_message::{CMessage, SendMessageError};
use crate::content::countryparam::{
    CCountryParam, CountryParameterUnavailable, CountryTechLevelLookup,
};
use crate::organizations::dbcountry::{
    CountryKingSaveSnapshot, CountryMinisterSaveSnapshot, CountrySaveSnapshot,
};
use crate::organizations::king::{
    change_control_point, set_control_point, set_material_point, set_war_point,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryGovernanceContextBlock {
    FactionMasterLookup,
    PlayerFactionLookup,
    UnionLookup,
    OwnedCityMutation,
    FactionDemise,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KingPointKind {
    Control,
    Material,
    War,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KingPointUpdate {
    pub kind: KingPointKind,
    pub requested: i32,
    pub previous: i32,
    pub applied: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct CountryKingSaveLimits {
    pub control_point: i32,
    pub material_point: i32,
    pub war_point: i32,
}

#[derive(Clone, Debug)]
pub struct CountryMinisterState {
    pub id_type: u8,
    pub quest_switch: bool,
    pub snapshot: CountryMinisterSaveSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryConstructorReport {
    pub next_technology: CountryTechLevelLookup,
}

#[derive(Clone, Debug)]
pub enum CountryMinisterFromDbUpdate {
    Inserted,
    Replaced { previous: Option<CountryMinisterState> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryExileTimeLookup {
    pub started_at_ms: Option<i32>,
    pub sampled_at_ms: u32,
    pub remaining_ms: i32,
    pub remaining_seconds: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryQuestSwitchTarget {
    King,
    Minister { job: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryQuestSwitchUpdate {
    pub target: CountryQuestSwitchTarget,
    pub previous: bool,
    pub applied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryMinisterTermReset {
    pub job: u8,
    pub previous_appointed: bool,
    pub previous_salary_received: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryNewTermReport {
    pub previous_king_appointed: bool,
    pub previous_king_salary_received: bool,
    pub minister_resets: Vec<CountryMinisterTermReset>,
    pub previous_silence_count: i32,
    pub previous_pk_count: i32,
    pub previous_exile_count: i32,
    pub previous_absolve_count: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryVillageTaxRegion {
    pub map_key: i32,
    pub name: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryVillageTaxContextBlock {
    UninitializedRegionType { map_key: i32 },
    UninitializedRegionCountry { map_key: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryVillageTaxUpdate {
    pub map_key: i32,
    pub region_name: Vec<u8>,
    pub previous_treasury: i32,
    pub daily_treasury: i32,
    pub applied_treasury: i32,
    pub log: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryVillageTaxReport {
    pub updates: Vec<CountryVillageTaxUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CountryVillageTaxBlock {
    Parameter(CountryParameterUnavailable),
    Context(CountryVillageTaxContextBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountrySetNewDayDisposition {
    FirstDay,
    TaxBlocked(CountryVillageTaxBlock),
    Rolled {
        tax: Option<CountryVillageTaxReport>,
        term: CountryNewTermReport,
        minister_slot_inserted: bool,
        minister_deposed: Option<CountryAppointMinisterReport>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountrySetNewDayReport {
    pub requested_day: i32,
    pub previous_day: i32,
    pub applied_day: i32,
    pub previous_silence_count: i32,
    pub previous_pk_count: i32,
    pub disposition: CountrySetNewDayDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryScalarUpdate {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryExileTextArgument<'a> {
    Text(&'a [u8]),
    Signed(i32),
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryExileMessageDelivery {
    pub map_id: i32,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryExileTarget {
    pub name: Vec<u8>,
    pub country: Option<u8>,
    pub level: u8,
    pub credit: u32,
    pub pk_count: u16,
    pub is_god: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryOnlinePlayer {
    pub id: i32,
    pub name: Vec<u8>,
    pub country: Option<u8>,
    pub occupation: u8,
    pub level: u8,
    pub is_god: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryPlayerInfo {
    pub id: i32,
    pub name: Vec<u8>,
    pub occupation: u8,
    pub level: u8,
    pub faction_name: Vec<u8>,
    pub is_faction_master: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryPlayersListContextBlock {
    PlayerFactionLookup,
    FactionMasterLookup,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryPlayersListReport {
    pub page: i32,
    pub start_index: u32,
    pub total: u32,
    pub entries: Vec<CountryPlayerInfo>,
    pub map_id: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
    pub logs: Vec<Vec<u8>>,
    pub legacy_result: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryInitialKingRejection {
    PlayerMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    FactionMissing,
    KingMismatch,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryInitialKingDisposition {
    Rejected {
        reason: CountryInitialKingRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    Applied {
        faction_id: i32,
        control_point_update: KingPointUpdate,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        world_wire: Option<Vec<u8>>,
        world_delivery: Option<Result<i32, SendMessageError>>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryInitialKingReport {
    pub player_id: i32,
    pub text: Vec<u8>,
    pub legacy_result: i32,
    pub disposition: CountryInitialKingDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountrySetKingReport {
    pub player_id: i32,
    pub depose: CountryDeposeKingReport,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
    pub legacy_result: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryFactionSnapshot {
    pub faction_id: i32,
    pub name: Vec<u8>,
    pub owned_cities: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryAbsolveCounterReset {
    pub previous_kill_count: u32,
    pub previous_pk_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryDemiseRejection {
    CountryAtWar,
    InsufficientControlPoint,
    AppointmentPending,
    SamePlayer,
    PlayerMissing,
    KingMissing,
    InsufficientCredit,
    InsufficientLevel,
    FactionMissing,
    OwnedCityConflict,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    OldKingNameMissing,
    OldKingNotFactionMaster,
    OldFactionMissing,
    OldFactionCityMissing,
    FactionTransferRejected,
    OldKingDeposeFailed,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryCanDemiseDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryDemiseRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryCityTransferReport {
    pub city_id: i32,
    pub old_faction_id: i32,
    pub cleared_old_cities: Vec<i32>,
    pub new_faction_id: i32,
    pub new_union_id: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryDeposeKingReport {
    pub old_king_id: i32,
    pub legacy_result: i32,
    pub mode: u8,
    pub minister_reports: Vec<CountryDeposeMinisterReport>,
    pub appointment_wire: Option<Vec<u8>>,
    pub appointment_delivery: Option<Result<i32, SendMessageError>>,
    pub world_wire: Option<Vec<u8>>,
    pub world_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryAiReport {
    pub current_tick_ms: u32,
    pub previous_timestamp_ms: u32,
    pub interval_ms: Option<i32>,
    pub deadline_ms: Option<u32>,
    pub due: bool,
    pub timestamp_updated: bool,
    pub control_point_update: Option<KingPointUpdate>,
    pub depose: Option<CountryDeposeKingReport>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryAiBlock {
    Parameter {
        report: CountryAiReport,
        source: CountryParameterUnavailable,
    },
    Depose {
        report: CountryAiReport,
        source: CountryGovernanceContextBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryRegisterKingDisposition {
    Rejected(CountryDemiseRejection),
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    Applied {
        city_transfer: Option<CountryCityTransferReport>,
        faction_transferred: bool,
        depose: CountryDeposeKingReport,
        control_point_update: KingPointUpdate,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        world_wire: Option<Vec<u8>>,
        world_delivery: Option<Result<i32, SendMessageError>>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryRegisterKingReport {
    pub player_id: i32,
    pub text: Vec<u8>,
    pub disposition: CountryRegisterKingDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryDemiseDisposition {
    Rejected(CountryDemiseRejection),
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    Applied {
        old_king_id: i32,
        register: CountryRegisterKingReport,
        control_point_delivery: CountryExileMessageDelivery,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryDemiseReport {
    pub target_player_id: i32,
    pub legacy_result: i32,
    pub text: Vec<u8>,
    pub disposition: CountryDemiseDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountrySuccessExiledDisposition {
    PlayerMissing {
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        king_map_id: i32,
        control_point_update: Option<KingPointUpdate>,
        control_point_delivery: Option<CountryExileMessageDelivery>,
    },
    Successful {
        control_point_update: KingPointUpdate,
        control_point_delivery: CountryExileMessageDelivery,
        previous_exile_count: i32,
        applied_exile_count: i32,
        country_deliveries: Vec<CountryExileMessageDelivery>,
    },
    Failed {
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountrySuccessExiledReport {
    pub player_id: i32,
    pub success: bool,
    pub text: Vec<u8>,
    pub disposition: CountrySuccessExiledDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryExileRejection {
    CountryAtWar,
    InsufficientControlPoint,
    DailyLimitReached,
    TargetIsKing,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    ExileRectMissing,
    TargetPkTooHigh,
    TargetRouteMissing,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryCanExileDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryExileRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryExileRequestDisposition {
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryExileRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    Sent {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountrySilenceRejection {
    CountryAtWar,
    InsufficientControlPoint,
    DailyLimitReached,
    TargetIsKing,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    TargetIsGod,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryCanSilenceDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountrySilenceRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountrySilenceDisposition {
    Rejected {
        reason: CountrySilenceRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        previous_silence_count: i32,
        applied_silence_count: i32,
        control_point_update: Option<KingPointUpdate>,
    },
    Applied {
        previous_silence_count: i32,
        applied_silence_count: i32,
        control_point_update: KingPointUpdate,
        control_point_delivery: CountryExileMessageDelivery,
        target_delivery: CountryExileMessageDelivery,
        target_wire: Vec<u8>,
        country_deliveries: Vec<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountrySilenceReport {
    pub player_id: i32,
    pub legacy_result: i32,
    pub text: Vec<u8>,
    pub disposition: CountrySilenceDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryAbsolveRejection {
    CountryAtWar,
    InsufficientControlPoint,
    DailyLimitReached,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    TargetMutationUnavailable,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryCanAbsolveDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryAbsolveRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryAbsolveDisposition {
    Rejected {
        reason: CountryAbsolveRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        counter_reset: CountryAbsolveCounterReset,
        control_point_update: Option<KingPointUpdate>,
    },
    Applied {
        counter_reset: CountryAbsolveCounterReset,
        control_point_update: KingPointUpdate,
        control_point_delivery: CountryExileMessageDelivery,
        previous_absolve_count: i32,
        applied_absolve_count: i32,
        broadcast_wire: Vec<u8>,
        broadcast_delivery: Result<i32, SendMessageError>,
        country_deliveries: Vec<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryAbsolveReport {
    pub player_id: i32,
    pub legacy_result: i32,
    pub text: Vec<u8>,
    pub disposition: CountryAbsolveDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryCanDeposeMinisterDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryBaseInfoDisposition {
    KingMissing,
    ParameterUnavailable(CountryParameterUnavailable),
    MinisterCountOutOfRange { minister_count: usize },
    Sent {
        map_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        log: Vec<u8>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryDeposeMinisterDisposition {
    SlotMissing { inserted_null_slot: bool },
    Applied {
        previous_player_id: i32,
        previous_name: Vec<u8>,
        country_deliveries: Vec<CountryExileMessageDelivery>,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        control_point_delivery: CountryExileMessageDelivery,
        base_info: CountryBaseInfoDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryDeposeMinisterReport {
    pub job: u8,
    pub mode: u8,
    pub legacy_result: i32,
    pub text: Vec<u8>,
    pub disposition: CountryDeposeMinisterDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryAppointMinisterRejection {
    CountryAtWar,
    InsufficientControlPoint,
    TargetIsKing,
    JobUnavailable,
    JobOccupied,
    AppointmentPending,
    TargetMissing,
    TargetCountryUnavailable,
    TargetFromAnotherCountry,
    TargetAlreadyHasJob { job: u8 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryCanAppointMinisterDisposition {
    Allowed,
    ParameterUnavailable(CountryParameterUnavailable),
    Rejected {
        reason: CountryAppointMinisterRejection,
        text: Vec<u8>,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountryAppointMinisterDisposition {
    Rejected {
        reason: CountryAppointMinisterRejection,
        private_delivery: Option<CountryExileMessageDelivery>,
    },
    ParameterUnavailable {
        block: CountryParameterUnavailable,
        appointment_flag_set: bool,
        control_point_update: Option<KingPointUpdate>,
    },
    Applied {
        control_point_update: KingPointUpdate,
        country_deliveries: Vec<CountryExileMessageDelivery>,
        appointment_wire: Vec<u8>,
        appointment_delivery: Result<i32, SendMessageError>,
        control_point_delivery: CountryExileMessageDelivery,
        base_info: CountryBaseInfoDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryAppointMinisterReport {
    pub player_id: i32,
    pub job: u8,
    pub mode: u8,
    pub legacy_result: i32,
    pub text: Vec<u8>,
    pub disposition: CountryAppointMinisterDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountrySerializeError {
    MinisterCountOutOfRange { minister_count: usize },
}

impl fmt::Display for CountrySerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinisterCountOutOfRange { minister_count } => write!(
                formatter,
                "CCountry содержит {minister_count} министров вне byte-диапазона"
            ),
        }
    }
}

impl Error for CountrySerializeError {}

pub trait CountryNewTermContext {
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
}

pub trait CountryVillageTaxContext {
    fn village_regions(
        &mut self,
        country_id: u8,
    ) -> Result<Vec<CountryVillageTaxRegion>, CountryVillageTaxContextBlock>;
    fn country_name(&mut self, country_id: u8) -> Vec<u8>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8>;
    fn put_king_log(&mut self, text: &[u8]);
}

pub trait CountrySetNewDayContext:
    CountryNewTermContext + CountryVillageTaxContext + CountryExileResultContext
{
}

impl<Context> CountrySetNewDayContext for Context where
    Context: CountryNewTermContext + CountryVillageTaxContext + CountryExileResultContext + ?Sized
{
}

pub trait CountryExileResultContext {
    fn map_player_name(&mut self, player_id: i32) -> Option<Vec<u8>>;
    fn online_player(&mut self, player_id: i32) -> Option<CountryExileTarget>;
    fn reset_online_player_murder_counters(
        &mut self,
        player_id: i32,
    ) -> Option<CountryAbsolveCounterReset>;
    fn faction_id_by_master_player(
        &mut self,
        _player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::FactionMasterLookup)
    }
    fn faction_id_by_player(
        &mut self,
        _player_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::PlayerFactionLookup)
    }
    fn faction_snapshot(&mut self, _faction_id: i32) -> Option<CountryFactionSnapshot> {
        None
    }
    fn union_id_for_faction(
        &mut self,
        _faction_id: i32,
    ) -> Result<i32, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::UnionLookup)
    }
    fn clear_faction_owned_cities(
        &mut self,
        _faction_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::OwnedCityMutation)
    }
    fn add_faction_owned_city(
        &mut self,
        _faction_id: i32,
        _city_id: i32,
    ) -> Result<(), CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::OwnedCityMutation)
    }
    fn refresh_owned_city(
        &mut self,
        _city_id: i32,
        _faction_id: i32,
        _union_id: i32,
        _country_id: Option<u8>,
    ) {
    }
    fn demise_faction(
        &mut self,
        _faction_id: i32,
        _old_master_id: i32,
        _new_master_id: i32,
        _country_id: u8,
        _king_id: i32,
        _demise_faction: bool,
    ) -> Result<bool, CountryGovernanceContextBlock> {
        Err(CountryGovernanceContextBlock::FactionDemise)
    }
    fn current_tick_ms(&mut self) -> u32 {
        0
    }
    fn country_name(&mut self, country_id: u8) -> Vec<u8>;
    fn country_identity_name(&mut self, identity: u8) -> Vec<u8>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8>;
    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
    fn send_to_connected_game_servers(
        &mut self,
        message: &CMessage,
    ) -> Vec<CountryExileMessageDelivery>;
    fn send_all(&mut self, message: &CMessage) -> Result<i32, SendMessageError>;
    fn put_king_log(&mut self, text: &[u8]);
}

/// Узкая граница единственных трёх эффектов, которые достигает
/// `CCountry::HasJob`: локализованное имя страны, форматирование `WS0034` и
/// запись отрицательной проверки короля в исторический файл `king`.
pub trait CountryHasJobContext {
    fn country_name(&mut self, country_id: u8) -> Vec<u8>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8>;
    fn put_king_log(&mut self, text: &[u8]);
}

impl<Context: CountryExileResultContext + ?Sized> CountryHasJobContext for Context {
    fn country_name(&mut self, country_id: u8) -> Vec<u8> {
        CountryExileResultContext::country_name(self, country_id)
    }

    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8> {
        CountryExileResultContext::format_world_string(self, string_id, arguments)
    }

    fn put_king_log(&mut self, text: &[u8]) {
        CountryExileResultContext::put_king_log(self, text);
    }
}

pub trait CountryPlayersListContext {
    fn online_players(&mut self) -> Vec<CountryOnlinePlayer>;
    fn player_faction(
        &mut self,
        player_id: i32,
    ) -> Result<(Vec<u8>, bool), CountryPlayersListContextBlock>;
    fn country_name(&mut self, country_id: u8) -> Vec<u8>;
    fn format_world_string(
        &mut self,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
    ) -> Vec<u8>;
    fn game_server_number_by_player_id(&mut self, player_id: i32) -> i32;
    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError>;
    fn put_king_log(&mut self, text: &[u8]);
}

#[derive(Clone, Debug)]
pub struct CCountry {
    pub country_id: u8,
    pub treasury: i32,
    pub power: i32,
    pub tech_current_exp: i32,
    pub tech_level_up_exp: i32,
    pub tech_level: i32,
    pub king: CountryKingSaveSnapshot,
    pub king_quest_switch: bool,
    pub country_war_result: i32,
    pub ministers: BTreeMap<u8, CountryMinisterState>,
    pub null_minister_slots: BTreeSet<u8>,
    pub city_id: i32,
    pub demise_faction: bool,
    pub king_timestamp_ms: u32,
    pub is_warring: bool,
    pub day: i32,
    pub silence_count: i32,
    pub pk_count: i32,
    pub exile_count: i32,
    pub absolve_count: i32,
    pub exile_started_at_ms: BTreeMap<i32, i32>,
}

enum DemiseTargetBlock {
    Rejected(CountryDemiseRejection, Vec<u8>),
    Parameter(CountryParameterUnavailable),
    Context(CountryGovernanceContextBlock),
}

impl CCountry {
    pub fn with_constructor_state(
        parameters: &mut CCountryParam,
    ) -> (Self, CountryConstructorReport) {
        let next_technology = parameters.technology_level_or_insert(1);
        (
            Self {
                country_id: 0,
                treasury: 0,
                power: 0,
                tech_current_exp: 0,
                tech_level_up_exp: next_technology.country_tech_exp,
                tech_level: 0,
                king: CountryKingSaveSnapshot {
                    id: 0,
                    name: Vec::new(),
                    appointed: false,
                    salary_received: false,
                    control_point: 0,
                    material_point: 0,
                    war_point: 0,
                },
                king_quest_switch: false,
                country_war_result: 0,
                ministers: BTreeMap::new(),
                null_minister_slots: BTreeSet::new(),
                city_id: 0,
                demise_faction: false,
                king_timestamp_ms: 0,
                is_warring: false,
                day: 0,
                silence_count: 0,
                pk_count: 0,
                exile_count: 0,
                absolve_count: 0,
                exile_started_at_ms: BTreeMap::new(),
            },
            CountryConstructorReport { next_technology },
        )
    }

    fn get_minister(&self, job: u8) -> Option<&CountryMinisterState> {
        if !(2..=7).contains(&job) {
            return None;
        }
        self.ministers.get(&job)
    }

    fn get_minister_mut(&mut self, job: u8) -> Option<&mut CountryMinisterState> {
        if !(2..=7).contains(&job) {
            return None;
        }
        self.ministers.get_mut(&job)
    }

 /// Выполняет `CCountry::AI`: unsigned wrapping deadline, одно
 /// списание control point и optional `DeposeKing(4)`.
    pub fn ai<Context, GetTick>(
        &mut self,
        parameters: &CCountryParam,
        mut get_tick: GetTick,
        context: &mut Context,
    ) -> Result<CountryAiReport, CountryAiBlock>
    where
        Context: CountryExileResultContext + ?Sized,
        GetTick: FnMut() -> u32,
    {
        let current_tick_ms = get_tick();
        let previous_timestamp_ms = self.king_timestamp_ms;
        let mut report = CountryAiReport {
            current_tick_ms,
            previous_timestamp_ms,
            interval_ms: None,
            deadline_ms: None,
            due: false,
            timestamp_updated: false,
            control_point_update: None,
            depose: None,
        };
        let Some(interval_ms) = parameters.king_control_point_decay_interval() else {
            return Err(CountryAiBlock::Parameter {
                report,
                source: CountryParameterUnavailable {
                    field: "_dec_king_control_point_interval",
                },
            });
        };
        report.interval_ms = Some(interval_ms);
        let deadline_ms = previous_timestamp_ms.wrapping_add(interval_ms as u32);
        report.deadline_ms = Some(deadline_ms);
        if current_tick_ms < deadline_ms {
            return Ok(report);
        }
        report.due = true;

        let Some(decay) = parameters.king_control_point_decay() else {
            return Err(CountryAiBlock::Parameter {
                report,
                source: CountryParameterUnavailable {
                    field: "_dec_king_control_point_time",
                },
            });
        };
        self.king_timestamp_ms = current_tick_ms;
        report.timestamp_updated = true;
        let control_point_update = match change_control_point(
            &mut self.king,
            decay.wrapping_neg(),
            parameters,
        ) {
            Ok(update) => update,
            Err(source) => {
                return Err(CountryAiBlock::Parameter { report, source });
            }
        };
        report.control_point_update = Some(control_point_update);
        if self.king.control_point < 0 {
            let depose = match self.depose_king(4, parameters, context) {
                Ok(depose) => depose,
                Err(source) => {
                    return Err(CountryAiBlock::Depose { report, source });
                }
            };
            report.depose = Some(depose);
        }
        Ok(report)
    }

    pub fn set_minister_from_db(
        &mut self,
        job: u8,
        minister: Option<CountryMinisterState>,
    ) -> CountryMinisterFromDbUpdate {
        let existed = self.ministers.contains_key(&job) || self.null_minister_slots.contains(&job);
        let previous = self.ministers.remove(&job);
        self.null_minister_slots.remove(&job);
        match minister {
            Some(minister) => {
                self.ministers.insert(job, minister);
            }
            None => {
                self.null_minister_slots.insert(job);
            }
        }
        if existed {
            CountryMinisterFromDbUpdate::Replaced { previous }
        } else {
            CountryMinisterFromDbUpdate::Inserted
        }
    }

    pub fn set_new_day<Context: CountrySetNewDayContext + ?Sized>(
        &mut self,
        requested_day: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountrySetNewDayReport {
        let previous_day = self.day;
        let previous_silence_count = std::mem::replace(&mut self.silence_count, 0);
        let previous_pk_count = std::mem::replace(&mut self.pk_count, 0);
        if previous_day == 0 {
            self.day = requested_day;
            return CountrySetNewDayReport {
                requested_day,
                previous_day,
                applied_day: self.day,
                previous_silence_count,
                previous_pk_count,
                disposition: CountrySetNewDayDisposition::FirstDay,
            };
        }

        let tax = if self.king.id == 0 {
            None
        } else {
            match self.add_village_tax_to_treasury(parameters, context) {
                Ok(report) => Some(report),
                Err(block) => {
                    return CountrySetNewDayReport {
                        requested_day,
                        previous_day,
                        applied_day: self.day,
                        previous_silence_count,
                        previous_pk_count,
                        disposition: CountrySetNewDayDisposition::TaxBlocked(block),
                    };
                }
            }
        };
        let term = self.new_term(context);

        let minister_slot_inserted = !self.ministers.contains_key(&7)
            && self.null_minister_slots.insert(7);
        let should_depose_minister = self
            .ministers
            .get(&7)
            .is_some_and(|minister| minister.snapshot.id != 0);
        let minister_deposed = should_depose_minister
            .then(|| self.appoint_minister(0, 7, 8, parameters, context));
        self.day = requested_day;
        CountrySetNewDayReport {
            requested_day,
            previous_day,
            applied_day: self.day,
            previous_silence_count,
            previous_pk_count,
            disposition: CountrySetNewDayDisposition::Rolled {
                tax,
                term,
                minister_slot_inserted,
                minister_deposed,
            },
        }
    }

    pub fn add_village_tax_to_treasury<Context: CountryVillageTaxContext + ?Sized>(
        &mut self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<CountryVillageTaxReport, CountryVillageTaxBlock> {
        let regions = context
            .village_regions(self.country_id)
            .map_err(CountryVillageTaxBlock::Context)?;
        if regions.is_empty() {
            return Ok(CountryVillageTaxReport { updates: Vec::new() });
        }
        let daily_treasury = parameters.daily_country_treasury().ok_or(
            CountryVillageTaxBlock::Parameter(CountryParameterUnavailable {
                field: "_daily_country_treasury",
            }),
        )?;
        let maximum_treasury = parameters.max_country_treasury().ok_or(
            CountryVillageTaxBlock::Parameter(CountryParameterUnavailable {
                field: "_max_country_treasury",
            }),
        )?;
        let mut updates = Vec::with_capacity(regions.len());
        for region in regions {
            let previous_treasury = self.treasury;
            let wrapping_sum = previous_treasury.wrapping_add(daily_treasury);
            let non_negative_sum = if wrapping_sum < 0 { 0 } else { wrapping_sum };
            self.treasury = if non_negative_sum < maximum_treasury {
                non_negative_sum
            } else {
                maximum_treasury
            };
            let country_name = context.country_name(self.country_id);
            let log = context.format_world_string(
                b"WS0091",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&region.name),
                    CountryExileTextArgument::Signed(daily_treasury),
                    CountryExileTextArgument::Signed(self.treasury),
                ],
            );
            context.put_king_log(&log);
            updates.push(CountryVillageTaxUpdate {
                map_key: region.map_key,
                region_name: region.name,
                previous_treasury,
                daily_treasury,
                applied_treasury: self.treasury,
                log,
            });
        }
        Ok(CountryVillageTaxReport { updates })
    }

    pub fn new_term<Context: CountryNewTermContext + ?Sized>(
        &mut self,
        context: &mut Context,
    ) -> CountryNewTermReport {
        let previous_king_appointed = self.king.appointed;
        let previous_king_salary_received = self.king.salary_received;
        self.king.appointed = false;
        self.king.salary_received = false;

        let minister_resets = self
            .ministers
            .iter_mut()
            .map(|(&job, minister)| {
                let reset = CountryMinisterTermReset {
                    job,
                    previous_appointed: minister.snapshot.appointed,
                    previous_salary_received: minister.snapshot.salary_received,
                };
                minister.snapshot.appointed = false;
                minister.snapshot.salary_received = false;
                reset
            })
            .collect();

        let message = CMessage::new(0x0007_FF14);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_all(&message);

        let previous_silence_count = std::mem::replace(&mut self.silence_count, 0);
        let previous_pk_count = std::mem::replace(&mut self.pk_count, 0);
        let previous_exile_count = std::mem::replace(&mut self.exile_count, 0);
        let previous_absolve_count = std::mem::replace(&mut self.absolve_count, 0);
        CountryNewTermReport {
            previous_king_appointed,
            previous_king_salary_received,
            minister_resets,
            previous_silence_count,
            previous_pk_count,
            previous_exile_count,
            previous_absolve_count,
            wire,
            delivery,
        }
    }

    pub fn get_info<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryBaseInfoDisposition {
        self.send_base_info_to_client(parameters, context)
    }

    pub fn set_king<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<CountrySetKingReport, CountryGovernanceContextBlock> {
        let depose = self.depose_king(3, parameters, context)?;
        self.king.id = player_id;
        let mut message = CMessage::new(0x0007_FF05);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_long(player_id);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_all(&message);
        Ok(CountrySetKingReport {
            player_id,
            depose,
            wire,
            delivery,
            legacy_result: player_id,
        })
    }

    pub fn register_initial_king<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryInitialKingReport {
        let Some(player) = context.online_player(player_id) else {
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::PlayerMissing,
                b"WS0016",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryInitialKingReport {
                player_id,
                text: Vec::new(),
                legacy_result: 0,
                disposition: CountryInitialKingDisposition::Rejected {
                    reason: CountryInitialKingRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::TargetFromAnotherCountry,
                b"WS0022",
                &[],
                true,
                context,
            );
        }
        let faction_id = match context.faction_id_by_player(player_id) {
            Ok(faction_id) => faction_id,
            Err(block) => {
                return CountryInitialKingReport {
                    player_id,
                    text: Vec::new(),
                    legacy_result: 0,
                    disposition: CountryInitialKingDisposition::ContextBlocked(block),
                };
            }
        };
        let Some(faction) = context.faction_snapshot(faction_id).filter(|_| faction_id > 0) else {
            let country_name = context.country_name(self.country_id);
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::FactionMissing,
                b"WS0023",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&player.name),
                ],
                false,
                context,
            );
        };
        if self.king.id != player_id {
            let country_name = context.country_name(self.country_id);
            return self.reject_initial_king(
                player_id,
                CountryInitialKingRejection::KingMismatch,
                b"WS0024",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&player.name),
                ],
                false,
                context,
            );
        }
        let Some(default_control_point) = parameters.default_king_control_point() else {
            return CountryInitialKingReport {
                player_id,
                text: Vec::new(),
                legacy_result: 0,
                disposition: CountryInitialKingDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_def_king_control_point" },
                ),
            };
        };
        let control_point_update = match set_control_point(
            &mut self.king,
            default_control_point,
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return CountryInitialKingReport {
                    player_id,
                    text: Vec::new(),
                    legacy_result: 0,
                    disposition: CountryInitialKingDisposition::ParameterUnavailable(block),
                };
            }
        };
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0025",
            &[
                CountryExileTextArgument::Text(&faction.name),
                CountryExileTextArgument::Text(&player.name),
                CountryExileTextArgument::Text(&country_name),
            ],
        ));
        context.put_king_log(&text);
        self.king.name = player.name;
        self.king_timestamp_ms = context.current_tick_ms();
        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(player_id);
        appointment.base_mut().add_byte(1);
        appointment.base_mut().add_byte(1);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let world = self.send_world_message(&text, context);
        CountryInitialKingReport {
            player_id,
            text,
            legacy_result: player_id,
            disposition: CountryInitialKingDisposition::Applied {
                faction_id,
                control_point_update,
                appointment_wire,
                appointment_delivery,
                world_wire: world.as_ref().map(|(wire, _)| wire.clone()),
                world_delivery: world.map(|(_, delivery)| delivery),
            },
        }
    }

    fn reject_initial_king<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountryInitialKingRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryInitialKingReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountryInitialKingReport {
            player_id,
            text,
            legacy_result: 0,
            disposition: CountryInitialKingDisposition::Rejected {
                reason,
                private_delivery,
            },
        }
    }

    pub fn authorize_king_for_players<Context: CountryPlayersListContext + ?Sized>(
        &self,
        candidate: i32,
        context: &mut Context,
    ) -> bool {
        if candidate != 0 && self.king.id == candidate {
            return true;
        }
        let country_name = context.country_name(self.country_id);
        let string_id = if candidate == 0 { b"WS0033" } else { b"WS0034" };
        let text = legacy_country_text(context.format_world_string(
            string_id,
            &[CountryExileTextArgument::Text(&country_name)],
        ));
        context.put_king_log(&text);
        false
    }

    pub fn get_players_list<Context: CountryPlayersListContext + ?Sized>(
        &self,
        page: i32,
        context: &mut Context,
    ) -> Result<CountryPlayersListReport, CountryPlayersListContextBlock> {
        let mut sorted_players = Vec::new();
        for (online_index, player) in context.online_players().into_iter().enumerate() {
            if player.country != Some(self.country_id) || player.level < 10 || player.is_god {
                continue;
            }
            let (faction_name, is_faction_master) = context.player_faction(player.id)?;
            sorted_players.push((
                online_index,
                CountryPlayerInfo {
                    id: player.id,
                    name: player.name,
                    occupation: player.occupation,
                    level: player.level,
                    faction_name,
                    is_faction_master,
                },
            ));
        }

        let country_name = context.country_name(self.country_id);
        let mut initial_log = country_name.clone();
        initial_log.extend_from_slice(b" : Successfully InitialOLPlayersList!");
        let initial_log = legacy_country_text(initial_log);
        context.put_king_log(&initial_log);

        // проверка multimap traversal: level по убыванию, равные ключи в
        // обратном порядке исходного online-list.
        sorted_players.sort_by(|(left_index, left), (right_index, right)| {
            right
                .level
                .cmp(&left.level)
                .then_with(|| right_index.cmp(left_index))
        });
        let mut sort_log = country_name.clone();
        sort_log.extend_from_slice(b" : Successfully Sort!");
        let sort_log = legacy_country_text(sort_log);
        context.put_king_log(&sort_log);

        let total = sorted_players.len() as u32;
        let total_signed = total as i32;
        let list_log = legacy_country_text(context.format_world_string(
            b"WS0021",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Signed(total_signed),
            ],
        ));
        context.put_king_log(&list_log);

        let start_index = page
            .wrapping_mul(3)
            .wrapping_sub(3)
            .wrapping_mul(4) as u32;
        let (end_index, entries) = if start_index < total {
            let end_index = start_index.wrapping_add(12).min(total);
            let entries = sorted_players[start_index as usize..end_index as usize]
                .iter()
                .map(|(_, player)| player.clone())
                .collect::<Vec<_>>();
            (end_index, entries)
        } else {
            (start_index, Vec::new())
        };
        let count = end_index.wrapping_sub(start_index) as i32;

        let mut message = CMessage::new(0x0007_FF08);
        message.base_mut().add_long(self.king.id);
        message.base_mut().add_long(count);
        message.base_mut().add_long(total_signed);
        for player in &entries {
            let name = CString::new(legacy_c_string_prefix(&player.name))
                .expect("player-name C-string prefix не содержит NUL");
            let faction_name = CString::new(legacy_c_string_prefix(&player.faction_name))
                .expect("faction-name C-string prefix не содержит NUL");
            message.base_mut().add_long(player.id);
            message.base_mut().add_str(Some(&name));
            message.base_mut().add_byte(player.occupation);
            message.base_mut().add_byte(player.level);
            message.base_mut().add_str(Some(&faction_name));
            message
                .base_mut()
                .add_byte(u8::from(player.is_faction_master));
        }
        let map_id = context.game_server_number_by_player_id(self.king.id);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_to_map_id(&message, map_id);
        Ok(CountryPlayersListReport {
            page,
            start_index,
            total,
            entries,
            map_id,
            wire,
            delivery,
            logs: vec![initial_log, sort_log, list_log],
            legacy_result: self.king.id,
        })
    }

    pub fn authorize_king<Context: CountryExileResultContext + ?Sized>(
        &self,
        candidate: i32,
        context: &mut Context,
    ) -> bool {
        self.authorize_king_for_job(candidate, context)
    }

 /// Повторяет `CCountry::HasJob`: сначала вызывает полный `IsKing`
 /// (включая его отрицательный `WS0034` king-log), затем ищет первый
 /// minister ID в unsigned job-order.
    pub fn has_job<Context: CountryHasJobContext + ?Sized>(
        &self,
        player_id: i32,
        context: &mut Context,
    ) -> u8 {
        if self.authorize_king_for_job(player_id, context) {
            return 1;
        }
        self.ministers
            .iter()
            .find_map(|(&job, minister)| (minister.snapshot.id == player_id).then_some(job))
            .unwrap_or(0)
    }

    fn authorize_king_for_job<Context: CountryHasJobContext + ?Sized>(
        &self,
        candidate: i32,
        context: &mut Context,
    ) -> bool {
        if candidate != 0 && self.king.id == candidate {
            return true;
        }
        let country_name = context.country_name(self.country_id);
        let string_id = if candidate == 0 { b"WS0033" } else { b"WS0034" };
        let text = legacy_country_text(context.format_world_string(
            string_id,
            &[CountryExileTextArgument::Text(&country_name)],
        ));
        context.put_king_log(&text);
        false
    }

    pub fn authorize_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        job: u8,
        context: &mut Context,
    ) -> bool {
        if player_id != 0 && self.has_minister_id(player_id) {
            return true;
        }
        let country_name = context.country_name(self.country_id);
        let identity_name = context.country_identity_name(job);
        let string_id = if player_id == 0 { b"WS0036" } else { b"WS0037" };
        let text = legacy_country_text(context.format_world_string(
            string_id,
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&identity_name),
            ],
        ));
        context.put_king_log(&text);
        false
    }

    pub fn can_demise<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanDemiseDisposition {
        if parameters.min_king_control_point().is_none() {
            return CountryCanDemiseDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        }
        let Some(required) = parameters.demise_required_control_point() else {
            return CountryCanDemiseDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_def_king_control_point_demise_need" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryDemiseRejection::CountryAtWar, b"WS0038" as &'static [u8], None))
        } else if self.king.control_point < required {
            Some((
                CountryDemiseRejection::InsufficientControlPoint,
                b"WS0039" as &'static [u8],
                Some(required),
            ))
        } else if self.king.appointed {
            Some((CountryDemiseRejection::AppointmentPending, b"WS0040" as &'static [u8], None))
        } else {
            None
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanDemiseDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanDemiseDisposition::Rejected { reason, text, private_delivery }
    }

 /// `0x60308 -> Demise`: возвращает прежнего короля даже если
 /// вложенный `RegisterKing` отказал, потому что старый caller его результат
 /// не проверял.
    pub fn demise<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryDemiseReport {
        if self.king.id == player_id {
            let country_name = context.country_name(self.country_id);
            let text = legacy_country_text(context.format_world_string(
                b"WS0058",
                &[CountryExileTextArgument::Text(&country_name)],
            ));
            context.put_king_log(&text);
            return CountryDemiseReport {
                target_player_id: player_id,
                legacy_result: 0,
                text,
                disposition: CountryDemiseDisposition::Rejected(
                    CountryDemiseRejection::SamePlayer,
                ),
            };
        }

        if let Err(block) = self.can_demise_target(player_id, parameters, context) {
            return match block {
                DemiseTargetBlock::Rejected(reason, text) => CountryDemiseReport {
                    target_player_id: player_id,
                    legacy_result: 0,
                    text,
                    disposition: CountryDemiseDisposition::Rejected(reason),
                },
                DemiseTargetBlock::Parameter(block) => CountryDemiseReport {
                    target_player_id: player_id,
                    legacy_result: 0,
                    text: Vec::new(),
                    disposition: CountryDemiseDisposition::ParameterUnavailable(block),
                },
                DemiseTargetBlock::Context(block) => CountryDemiseReport {
                    target_player_id: player_id,
                    legacy_result: 0,
                    text: Vec::new(),
                    disposition: CountryDemiseDisposition::ContextBlocked(block),
                },
            };
        }

        let old_king_id = self.king.id;
        let register = self.register_king_demise(player_id, parameters, context);
        let map_id = context.game_server_number_by_player_id(self.king.id);
        let control_point_delivery = self.send_king_control_point(map_id, context);
        CountryDemiseReport {
            target_player_id: player_id,
            legacy_result: old_king_id,
            text: register.text.clone(),
            disposition: CountryDemiseDisposition::Applied {
                old_king_id,
                register,
                control_point_delivery,
            },
        }
    }

    fn can_demise_target<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<(), DemiseTargetBlock> {
        let old_king = context.online_player(self.king.id);
        let candidate = context.online_player(player_id);
        if old_king.is_none() || candidate.is_none() {
            let text = legacy_country_text(context.format_world_string(b"WS0016", &[]));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::PlayerMissing,
                text,
            ));
        }
        let target_master_faction = context
            .faction_id_by_master_player(player_id)
            .map_err(DemiseTargetBlock::Context)?;
        if target_master_faction == 0 {
            self.demise_faction = true;
        }
        self.can_ascend_for_demise(player_id, parameters, context)
    }

    fn can_ascend_for_demise<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<(), DemiseTargetBlock> {
        let Some(player) = context.online_player(player_id) else {
            let text = legacy_country_text(context.format_world_string(b"WS0016", &[]));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::PlayerMissing,
                text,
            ));
        };
        if self.king.id == 0 {
            let mut text = context.country_name(self.country_id);
            text.extend_from_slice(b" : [Fatal ERROR] Ascend. KingID ==0!");
            let text = legacy_country_text(text);
            context.put_king_log(&text);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::KingMissing,
                text,
            ));
        }
        let required_credit = parameters
            .king_need_credit()
            .ok_or(DemiseTargetBlock::Parameter(CountryParameterUnavailable {
                field: "_king_need_credit",
            }))?;
        if player.credit < required_credit as u32 {
            let country_name = context.country_name(self.country_id);
            let text = legacy_country_text(context.format_world_string(
                b"WS0017",
                &[CountryExileTextArgument::Text(&country_name)],
            ));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::InsufficientCredit,
                text,
            ));
        }
        let required_level = parameters
            .king_need_level()
            .ok_or(DemiseTargetBlock::Parameter(CountryParameterUnavailable {
                field: "_king_need_level",
            }))?;
        if i32::from(player.level) < required_level {
            let text = legacy_country_text(context.format_world_string(
                b"WS0018",
                &[CountryExileTextArgument::Signed(required_level)],
            ));
            context.put_king_log(&text);
            let _ = self.send_private_message(&text, 0, context);
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::InsufficientLevel,
                text,
            ));
        }

        let mut faction_id = context
            .faction_id_by_master_player(player_id)
            .map_err(DemiseTargetBlock::Context)?;
        if faction_id == 0 {
            faction_id = context
                .faction_id_by_player(player_id)
                .map_err(DemiseTargetBlock::Context)?;
            let king_faction_id = context
                .faction_id_by_player(self.king.id)
                .map_err(DemiseTargetBlock::Context)?;
            if faction_id != king_faction_id {
                faction_id = 0;
            }
        }
        let Some(faction) = context.faction_snapshot(faction_id).filter(|_| faction_id != 0) else {
            return Err(DemiseTargetBlock::Rejected(
                CountryDemiseRejection::FactionMissing,
                Vec::new(),
            ));
        };
        if !self.demise_faction && !faction.owned_cities.is_empty() {
            // В EXE сюда ошибочно передаётся player ID короля, а не faction ID.
            let old_king_union = context
                .union_id_for_faction(self.king.id)
                .map_err(DemiseTargetBlock::Context)?;
            if old_king_union != faction_id {
                let text = legacy_country_text(context.format_world_string(b"WS0019", &[]));
                context.put_king_log(&text);
                let _ = self.send_private_message(&text, 0, context);
                return Err(DemiseTargetBlock::Rejected(
                    CountryDemiseRejection::OwnedCityConflict,
                    text,
                ));
            }
        }
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0020",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
            ],
        ));
        context.put_king_log(&text);
        Ok(())
    }

    fn register_king_demise<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryRegisterKingReport {
        let Some(player) = context.online_player(player_id) else {
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::PlayerMissing,
                b"WS0016",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryRegisterKingReport {
                player_id,
                text: Vec::new(),
                disposition: CountryRegisterKingDisposition::Rejected(
                    CountryDemiseRejection::TargetCountryUnavailable,
                ),
            };
        };
        if player_country != self.country_id {
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::TargetFromAnotherCountry,
                b"WS0022",
                &[],
                true,
                context,
            );
        }
        let faction_id = match context.faction_id_by_player(player_id) {
            Ok(faction_id) => faction_id,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
        };
        let Some(candidate_faction) = context
            .faction_snapshot(faction_id)
            .filter(|_| faction_id != 0)
        else {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::FactionMissing,
                b"WS0023",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&player.name),
                ],
                false,
                context,
            );
        };

        let old_king_name = self.king.name.clone();
        if old_king_name.is_empty() {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::OldKingNameMissing,
                b"WS0026",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let old_faction_id = match context.faction_id_by_master_player(self.king.id) {
            Ok(faction_id) => faction_id,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
        };
        if old_faction_id == 0 {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::OldKingNotFactionMaster,
                b"WS0027",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&old_king_name),
                ],
                false,
                context,
            );
        }

        let mut city_transfer = None;
        let faction_transferred;
        if !self.demise_faction {
            faction_transferred = false;
            let Some(old_faction) = context.faction_snapshot(old_faction_id) else {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::Rejected(
                        CountryDemiseRejection::OldFactionMissing,
                    ),
                };
            };
            let city_id = old_faction.owned_cities.first().copied().unwrap_or(0);
            self.city_id = city_id;
            if city_id == 0 {
                let country_name = context.country_name(self.country_id);
                return self.reject_register_king(
                    player_id,
                    CountryDemiseRejection::OldFactionCityMissing,
                    b"WS0029",
                    &[
                        CountryExileTextArgument::Text(&country_name),
                        CountryExileTextArgument::Text(&old_king_name),
                    ],
                    false,
                    context,
                );
            }
            if let Err(block) = context.clear_faction_owned_cities(old_faction_id) {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
            if let Err(block) = context.add_faction_owned_city(faction_id, city_id) {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
            let new_union_id = match context.union_id_for_faction(candidate_faction.faction_id) {
                Ok(union_id) => union_id,
                Err(block) => {
                    return CountryRegisterKingReport {
                        player_id,
                        text: Vec::new(),
                        disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                    };
                }
            };
            context.refresh_owned_city(city_id, faction_id, new_union_id, Some(self.country_id));
            let mut city_message = CMessage::new(0x0007_FE27);
            city_message.base_mut().add_long(city_id);
            city_message.base_mut().add_long(faction_id);
            city_message.base_mut().add_long(0);
            city_message.base_mut().add_byte(self.country_id);
            let wire = city_message.as_wire_bytes().to_vec();
            let delivery = context.send_all(&city_message);
            city_transfer = Some(CountryCityTransferReport {
                city_id,
                old_faction_id,
                cleared_old_cities: old_faction.owned_cities,
                new_faction_id: faction_id,
                new_union_id,
                wire,
                delivery,
            });
        } else {
            faction_transferred = match context.demise_faction(
                faction_id,
                self.king.id,
                player_id,
                self.country_id,
                self.king.id,
                self.demise_faction,
            ) {
                Ok(transferred) => transferred,
                Err(block) => {
                    return CountryRegisterKingReport {
                        player_id,
                        text: Vec::new(),
                        disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                    };
                }
            };
            if !faction_transferred {
                let country_name = context.country_name(self.country_id);
                return self.reject_register_king(
                    player_id,
                    CountryDemiseRejection::FactionTransferRejected,
                    b"WS0028",
                    &[
                        CountryExileTextArgument::Text(&country_name),
                        CountryExileTextArgument::Text(&player.name),
                    ],
                    false,
                    context,
                );
            }
        }

        let depose = match self.depose_king(2, parameters, context) {
            Ok(report) => report,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ContextBlocked(block),
                };
            }
        };
        if depose.legacy_result == 0 {
            let country_name = context.country_name(self.country_id);
            return self.reject_register_king(
                player_id,
                CountryDemiseRejection::OldKingDeposeFailed,
                b"WS0030",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&old_king_name),
                ],
                false,
                context,
            );
        }

        self.king.id = player_id;
        self.king.appointed = true;
        let Some(cost) = parameters.demise_control_point_cost() else {
            return CountryRegisterKingReport {
                player_id,
                text: Vec::new(),
                disposition: CountryRegisterKingDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_dec_king_control_point_demise" },
                ),
            };
        };
        let control_point_update = match change_control_point(
            &mut self.king,
            cost.wrapping_neg(),
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return CountryRegisterKingReport {
                    player_id,
                    text: Vec::new(),
                    disposition: CountryRegisterKingDisposition::ParameterUnavailable(block),
                };
            }
        };
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0031",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&old_king_name),
                CountryExileTextArgument::Text(&player.name),
            ],
        ));
        context.put_king_log(&text);
        self.demise_faction = false;
        self.king.name = player.name;
        self.king_timestamp_ms = context.current_tick_ms();

        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(player_id);
        appointment.base_mut().add_byte(1);
        appointment.base_mut().add_byte(1);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let world = self.send_world_message(&text, context);
        CountryRegisterKingReport {
            player_id,
            text,
            disposition: CountryRegisterKingDisposition::Applied {
                city_transfer,
                faction_transferred,
                depose,
                control_point_update,
                appointment_wire,
                appointment_delivery,
                world_wire: world.as_ref().map(|(wire, _)| wire.clone()),
                world_delivery: world.map(|(_, delivery)| delivery),
            },
        }
    }

    fn reject_register_king<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountryDemiseRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryRegisterKingReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        if notify_king {
            let _ = self.send_private_message(&text, 0, context);
        }
        CountryRegisterKingReport {
            player_id,
            text,
            disposition: CountryRegisterKingDisposition::Rejected(reason),
        }
    }

    fn depose_king<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        mode: u8,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> Result<CountryDeposeKingReport, CountryGovernanceContextBlock> {
        let old_king_id = self.king.id;
        if old_king_id == 0 && self.king.name.is_empty() {
            let country_name = context.country_name(self.country_id);
            let text = legacy_country_text(context.format_world_string(
                b"WS0053",
                &[CountryExileTextArgument::Text(&country_name)],
            ));
            context.put_king_log(&text);
            return Ok(CountryDeposeKingReport {
                old_king_id,
                legacy_result: 0,
                mode,
                minister_reports: Vec::new(),
                appointment_wire: None,
                appointment_delivery: None,
                world_wire: None,
                world_delivery: None,
            });
        }

        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(old_king_id);
        appointment.base_mut().add_byte(1);
        appointment.base_mut().add_byte(2);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);

        if !self.demise_faction {
            let faction_id = context.faction_id_by_player(old_king_id)?;
            if faction_id == 0 {
                self.king.id = 0;
                self.king.name.clear();
                let country_name = context.country_name(self.country_id);
                let text = legacy_country_text(context.format_world_string(
                    b"WS0054",
                    &[
                        CountryExileTextArgument::Text(&country_name),
                        CountryExileTextArgument::Signed(old_king_id),
                    ],
                ));
                context.put_king_log(&text);
            }
            if faction_id <= 0 || context.faction_snapshot(faction_id).is_none() {
                let country_name = context.country_name(self.country_id);
                let text = legacy_country_text(context.format_world_string(
                    b"WS0055",
                    &[CountryExileTextArgument::Text(&country_name)],
                ));
                context.put_king_log(&text);
                return Ok(CountryDeposeKingReport {
                    old_king_id,
                    legacy_result: 0,
                    mode,
                    minister_reports: Vec::new(),
                    appointment_wire: Some(appointment_wire),
                    appointment_delivery: Some(appointment_delivery),
                    world_wire: None,
                    world_delivery: None,
                });
            }
        }

        let old_king_name = self.king.name.clone();
        let text = if old_king_name.is_empty() {
            Vec::new()
        } else if mode == 3 || mode == 4 {
            let country_name = context.country_name(self.country_id);
            legacy_country_text(context.format_world_string(
                if mode == 3 { b"WS0056" } else { b"WS0057" },
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&old_king_name),
                ],
            ))
        } else {
            Vec::new()
        };
        let jobs = self
            .ministers
            .iter()
            .filter_map(|(&job, minister)| (minister.snapshot.id != 0).then_some(job))
            .collect::<Vec<_>>();
        let mut minister_reports = Vec::with_capacity(jobs.len());
        for job in jobs {
            minister_reports.push(self.depose_minister(job, mode, parameters, context));
        }
        context.put_king_log(&text);
        let world = self.send_world_message(&text, context);
        self.king.id = 0;
        self.king.name.clear();
        Ok(CountryDeposeKingReport {
            old_king_id,
            legacy_result: old_king_id,
            mode,
            minister_reports,
            appointment_wire: Some(appointment_wire),
            appointment_delivery: Some(appointment_delivery),
            world_wire: world.as_ref().map(|(wire, _)| wire.clone()),
            world_delivery: world.map(|(_, delivery)| delivery),
        })
    }

    pub fn can_depose_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanDeposeMinisterDisposition {
        if parameters.min_king_control_point().is_none() {
            return CountryCanDeposeMinisterDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        }
        if !self.is_warring {
            return CountryCanDeposeMinisterDisposition::Allowed;
        }
        let text = legacy_country_text(context.format_world_string(b"WS0043", &[]));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanDeposeMinisterDisposition::Rejected { text, private_delivery }
    }

    pub fn can_appoint_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanAppointMinisterDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanAppointMinisterDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryAppointMinisterRejection::CountryAtWar, b"WS0041" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountryAppointMinisterRejection::InsufficientControlPoint,
                b"WS0042" as &'static [u8],
                Some(minimum),
            ))
        } else {
            None
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanAppointMinisterDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanAppointMinisterDisposition::Rejected { reason, text, private_delivery }
    }

    pub fn appoint_minister<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        job: u8,
        mode: u8,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryAppointMinisterReport {
        if player_id != 0 && self.king.id == player_id {
            let country_name = context.country_name(self.country_id);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetIsKing,
                b"WS0059",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(minister) = self.get_minister(job) else {
            let country_name = context.country_name(self.country_id);
            let identity_name = context.country_identity_name(job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::JobUnavailable,
                b"WS0060",
                &[
                    CountryExileTextArgument::Text(&country_name),
                    CountryExileTextArgument::Text(&identity_name),
                ],
                false,
                context,
            );
        };
        if minister.snapshot.id != 0 {
            let identity_name = context.country_identity_name(job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::JobOccupied,
                b"WS0061",
                &[CountryExileTextArgument::Text(&identity_name)],
                true,
                context,
            );
        }
        if minister.snapshot.appointed {
            let identity_name = context.country_identity_name(job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::AppointmentPending,
                b"WS0062",
                &[CountryExileTextArgument::Text(&identity_name)],
                true,
                context,
            );
        }
        let Some(player) = context.online_player(player_id) else {
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetMissing,
                b"WS0063",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryAppointMinisterReport {
                player_id,
                job,
                mode,
                legacy_result: player_id,
                text: Vec::new(),
                disposition: CountryAppointMinisterDisposition::Rejected {
                    reason: CountryAppointMinisterRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetFromAnotherCountry,
                b"WS0064",
                &[],
                true,
                context,
            );
        }
        let existing_job = self.has_job_with_legacy_king_check(player_id, context);
        if existing_job != 0 {
            let identity_name = context.country_identity_name(existing_job);
            return self.reject_appoint_minister(
                player_id,
                job,
                mode,
                CountryAppointMinisterRejection::TargetAlreadyHasJob { job: existing_job },
                b"WS0065",
                &[
                    CountryExileTextArgument::Text(&player.name),
                    CountryExileTextArgument::Text(&identity_name),
                ],
                true,
                context,
            );
        }
        self.get_minister_mut(job)
            .expect("minister проверен выше")
            .snapshot
            .appointed = true;
        let Some(cost) = parameters.appoint_control_point_cost() else {
            return self.appoint_parameter_unavailable(
                player_id,
                job,
                mode,
                CountryParameterUnavailable { field: "_dec_king_control_point_appoint" },
                None,
            );
        };
        let control_point_update = match change_control_point(
            &mut self.king,
            cost.wrapping_neg(),
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return self.appoint_parameter_unavailable(player_id, job, mode, block, None);
            }
        };
        let country_name = context.country_name(self.country_id);
        let identity_name = context.country_identity_name(job);
        let text = legacy_country_text(context.format_world_string(
            b"WS0066",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
                CountryExileTextArgument::Text(&identity_name),
            ],
        ));
        let country_deliveries = self.send_country_message(&text, context);
        context.put_king_log(&text);
        let minister = self.get_minister_mut(job).expect("minister проверен выше");
        minister.snapshot.id = player_id;
        minister.snapshot.name = player.name;
        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(player_id);
        appointment.base_mut().add_byte(job);
        appointment.base_mut().add_byte(1);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let control_point_delivery = self.send_king_control_point(king_map_id, context);
        let base_info = self.send_base_info_to_client(parameters, context);
        CountryAppointMinisterReport {
            player_id,
            job,
            mode,
            legacy_result: player_id,
            text,
            disposition: CountryAppointMinisterDisposition::Applied {
                control_point_update,
                country_deliveries,
                appointment_wire,
                appointment_delivery,
                control_point_delivery,
                base_info,
            },
        }
    }

    fn has_job_with_legacy_king_check<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        context: &mut Context,
    ) -> u8 {
        self.has_job(player_id, context)
    }

    fn reject_appoint_minister<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        job: u8,
        mode: u8,
        reason: CountryAppointMinisterRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryAppointMinisterReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountryAppointMinisterReport {
            player_id,
            job,
            mode,
            legacy_result: player_id,
            text,
            disposition: CountryAppointMinisterDisposition::Rejected { reason, private_delivery },
        }
    }

    fn appoint_parameter_unavailable(
        &self,
        player_id: i32,
        job: u8,
        mode: u8,
        block: CountryParameterUnavailable,
        control_point_update: Option<KingPointUpdate>,
    ) -> CountryAppointMinisterReport {
        CountryAppointMinisterReport {
            player_id,
            job,
            mode,
            legacy_result: player_id,
            text: Vec::new(),
            disposition: CountryAppointMinisterDisposition::ParameterUnavailable {
                block,
                appointment_flag_set: true,
                control_point_update,
            },
        }
    }

    pub fn depose_minister<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        job: u8,
        mode: u8,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryDeposeMinisterReport {
        let Some(minister) = self.ministers.get(&job) else {
            let inserted_null_slot = self.null_minister_slots.insert(job);
            return CountryDeposeMinisterReport {
                job,
                mode,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryDeposeMinisterDisposition::SlotMissing { inserted_null_slot },
            };
        };
        if minister.snapshot.id == 0 {
            return CountryDeposeMinisterReport {
                job,
                mode,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryDeposeMinisterDisposition::SlotMissing {
                    inserted_null_slot: false,
                },
            };
        }
        let previous_player_id = minister.snapshot.id;
        let previous_name = minister.snapshot.name.clone();
        let country_name = context.country_name(self.country_id);
        let identity_name = context.country_identity_name(job);
        let string_id = if mode == 8 { b"WS0067" } else { b"WS0068" };
        let arguments = if mode == 8 {
            [
                CountryExileTextArgument::Text(&previous_name),
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&identity_name),
            ]
        } else {
            [
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&previous_name),
                CountryExileTextArgument::Text(&identity_name),
            ]
        };
        let text = legacy_country_text(context.format_world_string(string_id, &arguments));
        let country_deliveries = self.send_country_message(&text, context);
        context.put_king_log(&text);
        let minister = self.ministers.get_mut(&job).expect("minister проверен выше");
        minister.snapshot.id = 0;
        minister.snapshot.name.clear();

        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut appointment = CMessage::new(0x0007_FF04);
        appointment.base_mut().add_byte(self.country_id);
        appointment.base_mut().add_long(previous_player_id);
        appointment.base_mut().add_byte(job);
        appointment.base_mut().add_byte(2);
        let appointment_wire = appointment.as_wire_bytes().to_vec();
        let appointment_delivery = context.send_all(&appointment);
        let control_point_delivery = self.send_king_control_point(king_map_id, context);
        let base_info = self.send_base_info_to_client(parameters, context);
        CountryDeposeMinisterReport {
            job,
            mode,
            legacy_result: 0,
            text,
            disposition: CountryDeposeMinisterDisposition::Applied {
                previous_player_id,
                previous_name,
                country_deliveries,
                appointment_wire,
                appointment_delivery,
                control_point_delivery,
                base_info,
            },
        }
    }

    pub fn can_absolve<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanAbsolveDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanAbsolveDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryAbsolveRejection::CountryAtWar, b"WS0044" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountryAbsolveRejection::InsufficientControlPoint,
                b"WS0045" as &'static [u8],
                Some(minimum),
            ))
        } else {
            let Some(maximum) = parameters.max_absolve_count() else {
                return CountryCanAbsolveDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_max_absolve_num" },
                );
            };
            if self.absolve_count <= maximum.wrapping_sub(1) {
                None
            } else {
                Some((
                    CountryAbsolveRejection::DailyLimitReached,
                    b"WS0046" as &'static [u8],
                    Some(maximum),
                ))
            }
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanAbsolveDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanAbsolveDisposition::Rejected { reason, text, private_delivery }
    }

    pub fn absolve<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryAbsolveReport {
        let Some(player) = context.online_player(player_id) else {
            return self.reject_absolve(
                player_id,
                CountryAbsolveRejection::TargetMissing,
                b"WS0084",
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryAbsolveReport {
                player_id,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryAbsolveDisposition::Rejected {
                    reason: CountryAbsolveRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_absolve(
                player_id,
                CountryAbsolveRejection::TargetFromAnotherCountry,
                b"WS0085",
                context,
            );
        }
        let Some(counter_reset) = context.reset_online_player_murder_counters(player_id) else {
            return CountryAbsolveReport {
                player_id,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountryAbsolveDisposition::Rejected {
                    reason: CountryAbsolveRejection::TargetMutationUnavailable,
                    private_delivery: None,
                },
            };
        };
        let Some(control_point_cost) = parameters.absolve_control_point_cost() else {
            return self.absolve_parameter_unavailable(
                player_id,
                counter_reset,
                CountryParameterUnavailable { field: "_dec_king_control_point_absolve" },
                None,
            );
        };
        let control_point_update = match change_control_point(
            &mut self.king,
            control_point_cost.wrapping_neg(),
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return self.absolve_parameter_unavailable(
                    player_id,
                    counter_reset,
                    block,
                    None,
                );
            }
        };

        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut control_point_message = CMessage::new(0x0007_FF10);
        control_point_message.base_mut().add_long(self.king.id);
        control_point_message.base_mut().add_byte(self.country_id);
        control_point_message.base_mut().add_long(self.king.control_point);
        let control_point_delivery = CountryExileMessageDelivery {
            map_id: king_map_id,
            delivery: context.send_to_map_id(&control_point_message, king_map_id),
        };

        let previous_absolve_count = self.absolve_count;
        self.absolve_count = self.absolve_count.wrapping_add(1);
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0086",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
            ],
        ));
        context.put_king_log(&text);
        let mut broadcast = CMessage::new(0x0007_FF0C);
        broadcast.base_mut().add_byte(self.country_id);
        broadcast.base_mut().add_long(player_id);
        let broadcast_wire = broadcast.as_wire_bytes().to_vec();
        let broadcast_delivery = context.send_all(&broadcast);
        let country_deliveries = self.send_country_message(&text, context);
        CountryAbsolveReport {
            player_id,
            legacy_result: player_id,
            text,
            disposition: CountryAbsolveDisposition::Applied {
                counter_reset,
                control_point_update,
                control_point_delivery,
                previous_absolve_count,
                applied_absolve_count: self.absolve_count,
                broadcast_wire,
                broadcast_delivery,
                country_deliveries,
            },
        }
    }

    fn reject_absolve<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountryAbsolveRejection,
        string_id: &'static [u8],
        context: &mut Context,
    ) -> CountryAbsolveReport {
        let text = legacy_country_text(context.format_world_string(string_id, &[]));
        context.put_king_log(&text);
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryAbsolveReport {
            player_id,
            legacy_result: 0,
            text,
            disposition: CountryAbsolveDisposition::Rejected { reason, private_delivery },
        }
    }

    fn absolve_parameter_unavailable(
        &self,
        player_id: i32,
        counter_reset: CountryAbsolveCounterReset,
        block: CountryParameterUnavailable,
        control_point_update: Option<KingPointUpdate>,
    ) -> CountryAbsolveReport {
        CountryAbsolveReport {
            player_id,
            legacy_result: 0,
            text: Vec::new(),
            disposition: CountryAbsolveDisposition::ParameterUnavailable {
                block,
                counter_reset,
                control_point_update,
            },
        }
    }

    pub fn can_silence<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanSilenceDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanSilenceDisposition::ParameterUnavailable(
                CountryParameterUnavailable { field: "_min_king_control_point" },
            );
        };
        let rejection = if self.is_warring {
            Some((CountrySilenceRejection::CountryAtWar, b"WS0050" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountrySilenceRejection::InsufficientControlPoint,
                b"WS0051" as &'static [u8],
                Some(minimum),
            ))
        } else {
            let Some(maximum) = parameters.max_silence_count() else {
                return CountryCanSilenceDisposition::ParameterUnavailable(
                    CountryParameterUnavailable { field: "_max_silence_num" },
                );
            };
            if self.silence_count <= maximum.wrapping_sub(1) {
                None
            } else {
                Some((
                    CountrySilenceRejection::DailyLimitReached,
                    b"WS0052" as &'static [u8],
                    Some(maximum),
                ))
            }
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanSilenceDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanSilenceDisposition::Rejected { reason, text, private_delivery }
    }

    pub fn silence<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountrySilenceReport {
        if player_id == self.king.id {
            let country_name = context.country_name(self.country_id);
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetIsKing,
                b"WS0079",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(player) = context.online_player(player_id) else {
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetMissing,
                b"WS0080",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountrySilenceReport {
                player_id,
                legacy_result: 0,
                text: Vec::new(),
                disposition: CountrySilenceDisposition::Rejected {
                    reason: CountrySilenceRejection::TargetCountryUnavailable,
                    private_delivery: None,
                },
            };
        };
        if player_country != self.country_id {
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetFromAnotherCountry,
                b"WS0081",
                &[],
                true,
                context,
            );
        }
        if player.is_god {
            return self.reject_silence(
                player_id,
                CountrySilenceRejection::TargetIsGod,
                b"WS0082",
                &[],
                true,
                context,
            );
        }

        let previous_silence_count = self.silence_count;
        self.silence_count = self.silence_count.wrapping_add(1);
        let Some(control_point_cost) = parameters.silence_control_point_cost() else {
            return self.silence_parameter_unavailable(
                player_id,
                previous_silence_count,
                CountryParameterUnavailable { field: "_dec_king_control_point_silence" },
                None,
            );
        };
        let control_point_update = match change_control_point(
            &mut self.king,
            control_point_cost.wrapping_neg(),
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return self.silence_parameter_unavailable(
                    player_id,
                    previous_silence_count,
                    block,
                    None,
                );
            }
        };
        let Some(silence_time) = parameters.silence_time() else {
            return self.silence_parameter_unavailable(
                player_id,
                previous_silence_count,
                CountryParameterUnavailable { field: "_silence_time" },
                Some(control_point_update),
            );
        };

        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0083",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player.name),
                CountryExileTextArgument::Signed(silence_time),
            ],
        ));
        context.put_king_log(&text);

        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let mut control_point_message = CMessage::new(0x0007_FF10);
        control_point_message.base_mut().add_long(self.king.id);
        control_point_message.base_mut().add_byte(self.country_id);
        control_point_message.base_mut().add_long(self.king.control_point);
        let control_point_delivery = CountryExileMessageDelivery {
            map_id: king_map_id,
            delivery: context.send_to_map_id(&control_point_message, king_map_id),
        };

        let target_map_id = context.game_server_number_by_player_id(player_id);
        let mut target_message = CMessage::new(0x0007_FF0D);
        target_message.base_mut().add_byte(self.country_id);
        target_message.base_mut().add_long(player_id);
        target_message.base_mut().add_long(self.silence_count);
        let target_wire = target_message.as_wire_bytes().to_vec();
        let target_delivery = CountryExileMessageDelivery {
            map_id: target_map_id,
            delivery: context.send_to_map_id(&target_message, target_map_id),
        };
        let country_deliveries = self.send_country_message(&text, context);
        CountrySilenceReport {
            player_id,
            legacy_result: player_id,
            text,
            disposition: CountrySilenceDisposition::Applied {
                previous_silence_count,
                applied_silence_count: self.silence_count,
                control_point_update,
                control_point_delivery,
                target_delivery,
                target_wire,
                country_deliveries,
            },
        }
    }

    fn reject_silence<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        reason: CountrySilenceRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountrySilenceReport {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountrySilenceReport {
            player_id,
            legacy_result: 0,
            text,
            disposition: CountrySilenceDisposition::Rejected { reason, private_delivery },
        }
    }

    fn silence_parameter_unavailable(
        &self,
        player_id: i32,
        previous_silence_count: i32,
        block: CountryParameterUnavailable,
        control_point_update: Option<KingPointUpdate>,
    ) -> CountrySilenceReport {
        CountrySilenceReport {
            player_id,
            legacy_result: 0,
            text: Vec::new(),
            disposition: CountrySilenceDisposition::ParameterUnavailable {
                block,
                previous_silence_count,
                applied_silence_count: self.silence_count,
                control_point_update,
            },
        }
    }

    pub fn can_exile<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryCanExileDisposition {
        let Some(minimum) = parameters.min_king_control_point() else {
            return CountryCanExileDisposition::ParameterUnavailable(
                CountryParameterUnavailable {
                    field: "_min_king_control_point",
                },
            );
        };
        let rejection = if self.is_warring {
            Some((CountryExileRejection::CountryAtWar, b"WS0047" as &'static [u8], None))
        } else if self.king.control_point < minimum {
            Some((
                CountryExileRejection::InsufficientControlPoint,
                b"WS0048" as &'static [u8],
                Some(minimum),
            ))
        } else {
            let Some(maximum) = parameters.max_exile_count() else {
                return CountryCanExileDisposition::ParameterUnavailable(
                    CountryParameterUnavailable {
                        field: "_max_exile_num",
                    },
                );
            };
            if self.exile_count <= maximum.wrapping_sub(1) {
                None
            } else {
                Some((
                    CountryExileRejection::DailyLimitReached,
                    b"WS0049" as &'static [u8],
                    Some(maximum),
                ))
            }
        };
        let Some((reason, string_id, argument)) = rejection else {
            return CountryCanExileDisposition::Allowed;
        };
        let arguments = argument
            .as_ref()
            .map(|value| [CountryExileTextArgument::Signed(*value)]);
        let text = legacy_country_text(context.format_world_string(
            string_id,
            arguments.as_ref().map_or(&[], |arguments| arguments.as_slice()),
        ));
        let private_delivery = self.send_private_message(&text, 0, context);
        CountryCanExileDisposition::Rejected {
            reason,
            text,
            private_delivery,
        }
    }

    pub fn exile<Context: CountryExileResultContext + ?Sized>(
        &self,
        player_id: i32,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryExileRequestDisposition {
        if player_id == self.king.id {
            let country_name = context.country_name(self.country_id);
            return self.reject_exile_request(
                CountryExileRejection::TargetIsKing,
                b"WS0071",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(player) = context.online_player(player_id) else {
            return self.reject_exile_request(
                CountryExileRejection::TargetMissing,
                b"WS0072",
                &[],
                true,
                context,
            );
        };
        let Some(player_country) = player.country else {
            return CountryExileRequestDisposition::Rejected {
                reason: CountryExileRejection::TargetCountryUnavailable,
                text: Vec::new(),
                private_delivery: None,
            };
        };
        if player_country != self.country_id {
            return self.reject_exile_request(
                CountryExileRejection::TargetFromAnotherCountry,
                b"WS0073",
                &[],
                true,
                context,
            );
        }
        if !parameters.has_exile_rect(self.country_id) {
            let country_name = context.country_name(self.country_id);
            return self.reject_exile_request(
                CountryExileRejection::ExileRectMissing,
                b"WS0074",
                &[CountryExileTextArgument::Text(&country_name)],
                false,
                context,
            );
        }
        let Some(maximum_pk) = parameters.max_exile_pk() else {
            return CountryExileRequestDisposition::ParameterUnavailable(
                CountryParameterUnavailable {
                    field: "_max_exile_pk",
                },
            );
        };
        if i32::from(player.pk_count) > maximum_pk {
            return self.reject_exile_request(
                CountryExileRejection::TargetPkTooHigh,
                b"WS0075",
                &[CountryExileTextArgument::Text(&player.name)],
                true,
                context,
            );
        }
        let map_id = context.game_server_number_by_player_id(player_id);
        if map_id == 0 {
            return self.reject_exile_request(
                CountryExileRejection::TargetRouteMissing,
                b"WS0076",
                &[CountryExileTextArgument::Text(&player.name)],
                true,
                context,
            );
        }
        let mut message = CMessage::new(0x0007_FF0E);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_long(player_id);
        let wire = message.as_wire_bytes().to_vec();
        CountryExileRequestDisposition::Sent {
            map_id,
            wire,
            delivery: context.send_to_map_id(&message, map_id),
        }
    }

    fn reject_exile_request<Context: CountryExileResultContext + ?Sized>(
        &self,
        reason: CountryExileRejection,
        string_id: &'static [u8],
        arguments: &[CountryExileTextArgument<'_>],
        notify_king: bool,
        context: &mut Context,
    ) -> CountryExileRequestDisposition {
        let text = legacy_country_text(context.format_world_string(string_id, arguments));
        context.put_king_log(&text);
        let private_delivery = notify_king
            .then(|| self.send_private_message(&text, 0, context))
            .flatten();
        CountryExileRequestDisposition::Rejected {
            reason,
            text,
            private_delivery,
        }
    }

    pub fn has_king_id(&self, player_id: i32) -> bool {
        self.king.id == player_id
    }

 /// `IsMinister` игнорирует входной job при поиске и обходит всех
 /// живых министров; job используется только в сообщении отрицательного log.
    pub fn has_minister_id(&self, player_id: i32) -> bool {
        self.ministers
            .values()
            .any(|minister| minister.snapshot.id == player_id)
    }

    pub fn success_exiled<Context: CountryExileResultContext + ?Sized>(
        &mut self,
        player_id: i32,
        success: bool,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountrySuccessExiledReport {
        let Some(player_name) = context.map_player_name(player_id) else {
            let text = legacy_country_text(context.format_world_string(b"WS0072", &[]));
            context.put_king_log(&text);
            let private_delivery = self.send_private_message(&text, 0, context);
            return CountrySuccessExiledReport {
                player_id,
                success,
                text,
                disposition: CountrySuccessExiledDisposition::PlayerMissing {
                    private_delivery,
                },
            };
        };

        if !success {
            let text = legacy_country_text(context.format_world_string(
                b"WS0078",
                &[CountryExileTextArgument::Text(&player_name)],
            ));
            let private_delivery = self.send_private_message(&text, 0, context);
            context.put_king_log(&text);
            return CountrySuccessExiledReport {
                player_id,
                success,
                text,
                disposition: CountrySuccessExiledDisposition::Failed { private_delivery },
            };
        }

        // EXE вычисляет маршрут короля до чтения country-параметров.
        let king_map_id = context.game_server_number_by_player_id(self.king.id);
        let Some(control_point_cost) = parameters.exile_control_point_cost() else {
            return CountrySuccessExiledReport {
                player_id,
                success,
                text: Vec::new(),
                disposition: CountrySuccessExiledDisposition::ParameterUnavailable {
                    block: CountryParameterUnavailable {
                        field: "_dec_king_control_point_exile",
                    },
                    king_map_id,
                    control_point_update: None,
                    control_point_delivery: None,
                },
            };
        };
        let control_point_update = match change_control_point(
            &mut self.king,
            control_point_cost.wrapping_neg(),
            parameters,
        ) {
            Ok(update) => update,
            Err(block) => {
                return CountrySuccessExiledReport {
                    player_id,
                    success,
                    text: Vec::new(),
                    disposition: CountrySuccessExiledDisposition::ParameterUnavailable {
                        block,
                        king_map_id,
                        control_point_update: None,
                        control_point_delivery: None,
                    },
                };
            }
        };

        let mut control_point_message = CMessage::new(0x0007_FF10);
        control_point_message.base_mut().add_long(self.king.id);
        control_point_message.base_mut().add_byte(self.country_id);
        control_point_message
            .base_mut()
            .add_long(self.king.control_point);
        let control_point_delivery = CountryExileMessageDelivery {
            map_id: king_map_id,
            delivery: context.send_to_map_id(&control_point_message, king_map_id),
        };

        let Some(exile_time_ms) = parameters.exile_time_ms() else {
            return CountrySuccessExiledReport {
                player_id,
                success,
                text: Vec::new(),
                disposition: CountrySuccessExiledDisposition::ParameterUnavailable {
                    block: CountryParameterUnavailable {
                        field: "_exile_time",
                    },
                    king_map_id,
                    control_point_update: Some(control_point_update),
                    control_point_delivery: Some(control_point_delivery),
                },
            };
        };

        // Исходный timeGetTime здесь вызывался, но результат не сохранялся и
        // `ExileMap` не менялся. Чисто технический пустой вызов Rust не имитирует.
        let previous_exile_count = self.exile_count;
        self.exile_count = self.exile_count.wrapping_add(1);
        let country_name = context.country_name(self.country_id);
        let text = legacy_country_text(context.format_world_string(
            b"WS0077",
            &[
                CountryExileTextArgument::Text(&country_name),
                CountryExileTextArgument::Text(&player_name),
                CountryExileTextArgument::Signed(exile_time_ms / 60_000),
            ],
        ));
        let country_deliveries = self.send_country_message(&text, context);
        context.put_king_log(&text);
        CountrySuccessExiledReport {
            player_id,
            success,
            text,
            disposition: CountrySuccessExiledDisposition::Successful {
                control_point_update,
                control_point_delivery,
                previous_exile_count,
                applied_exile_count: self.exile_count,
                country_deliveries,
            },
        }
    }

    fn send_private_message<Context: CountryExileResultContext + ?Sized>(
        &self,
        text: &[u8],
        player_id: i32,
        context: &mut Context,
    ) -> Option<CountryExileMessageDelivery> {
        if text.is_empty() {
            return None;
        }
        let target_player_id = if player_id == 0 {
            self.king.id
        } else {
            player_id
        };
        let map_id = context.game_server_number_by_player_id(target_player_id);
        if map_id == 0 {
            return None;
        }
        let text = CString::new(text).expect("legacy country text не содержит NUL");
        let mut message = CMessage::new(0x0007_FF13);
        message.base_mut().add_long(target_player_id);
        message.base_mut().add_str(Some(&text));
        Some(CountryExileMessageDelivery {
            map_id,
            delivery: context.send_to_map_id(&message, map_id),
        })
    }

    fn send_king_control_point<Context: CountryExileResultContext + ?Sized>(
        &self,
        map_id: i32,
        context: &mut Context,
    ) -> CountryExileMessageDelivery {
        let mut message = CMessage::new(0x0007_FF10);
        message.base_mut().add_long(self.king.id);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_long(self.king.control_point);
        CountryExileMessageDelivery {
            map_id,
            delivery: context.send_to_map_id(&message, map_id),
        }
    }

    fn send_base_info_to_client<Context: CountryExileResultContext + ?Sized>(
        &self,
        parameters: &CCountryParam,
        context: &mut Context,
    ) -> CountryBaseInfoDisposition {
        if self.king.id == 0 {
            return CountryBaseInfoDisposition::KingMissing;
        }
        let parameter = |value: Option<i32>, field: &'static str| {
            value.ok_or(CountryParameterUnavailable { field })
        };
        let demise = match parameter(parameters.demise_control_point_cost(), "_dec_king_control_point_demise") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let appoint = match parameter(parameters.appoint_control_point_cost(), "_dec_king_control_point_appoint") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let exile = match parameter(parameters.exile_control_point_cost(), "_dec_king_control_point_exile") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let silence = match parameter(parameters.silence_control_point_cost(), "_dec_king_control_point_silence") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let absolve = match parameter(parameters.absolve_control_point_cost(), "_dec_king_control_point_absolve") {
            Ok(value) => value,
            Err(block) => return CountryBaseInfoDisposition::ParameterUnavailable(block),
        };
        let mut jobs = self.ministers.keys().copied().collect::<BTreeSet<_>>();
        jobs.extend(self.null_minister_slots.iter().copied());
        let Ok(minister_count) = i32::try_from(jobs.len()) else {
            return CountryBaseInfoDisposition::MinisterCountOutOfRange {
                minister_count: jobs.len(),
            };
        };
        let mut message = CMessage::new(0x0007_FF07);
        message.base_mut().add_long(self.king.id);
        message.base_mut().add_long(self.king.control_point);
        message.base_mut().add_long(self.treasury);
        message.base_mut().add_long(self.power);
        message.base_mut().add_long(self.king.material_point);
        message.base_mut().add_long(self.king.war_point);
        message.base_mut().add_long(self.tech_current_exp);
        message.base_mut().add_long(self.tech_level_up_exp);
        message.base_mut().add_long(self.tech_level);
        message.base_mut().add_byte(u8::from(self.king_quest_switch));
        message.base_mut().add_long(minister_count);
        for job in jobs {
            let minister = self.ministers.get(&job);
            let player_id = minister.map_or(0, |minister| minister.snapshot.id);
            message.base_mut().add_long(player_id);
            message.base_mut().add_byte(job);
            if let Some(minister) = minister.filter(|minister| minister.snapshot.id != 0) {
                let visible_name = minister
                    .snapshot
                    .name
                    .split(|&byte| byte == 0)
                    .next()
                    .unwrap_or_default();
                let name = CString::new(visible_name)
                    .expect("C-string prefix minister-а не содержит embedded NUL");
                message.base_mut().add_str(Some(&name));
                message.base_mut().add_byte(u8::from(minister.quest_switch));
                message.base_mut().add_byte(u8::from(minister.snapshot.appointed));
            }
        }
        for value in [demise, appoint, exile, silence, absolve] {
            message.base_mut().add_long(value);
        }
        let map_id = context.game_server_number_by_player_id(self.king.id);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_to_map_id(&message, map_id);
        let mut log = context.country_name(self.country_id);
        log.extend_from_slice(b" : Successfully SendBaseInfoToClient!");
        let log = legacy_country_text(log);
        context.put_king_log(&log);
        CountryBaseInfoDisposition::Sent { map_id, wire, delivery, log }
    }

    fn send_country_message<Context: CountryExileResultContext + ?Sized>(
        &self,
        text: &[u8],
        context: &mut Context,
    ) -> Vec<CountryExileMessageDelivery> {
        if text.is_empty() {
            return Vec::new();
        }
        let text = CString::new(text).expect("legacy country text не содержит NUL");
        let mut message = CMessage::new(0x0007_FF11);
        message.base_mut().add_byte(self.country_id);
        message.base_mut().add_str(Some(&text));
        context.send_to_connected_game_servers(&message)
    }

    fn send_world_message<Context: CountryExileResultContext + ?Sized>(
        &self,
        text: &[u8],
        context: &mut Context,
    ) -> Option<(Vec<u8>, Result<i32, SendMessageError>)> {
        if text.is_empty() {
            return None;
        }
        let text = CString::new(text).expect("legacy country text не содержит NUL");
        let mut message = CMessage::new(0x0007_FF12);
        message.base_mut().add_long(-0x1_0000);
        message.base_mut().add_long(-0x100);
        message.base_mut().add_str(Some(&text));
        let wire = message.as_wire_bytes().to_vec();
        let delivery = context.send_all(&message);
        Some((wire, delivery))
    }

 /// Повторяет signed 32-битную арифметику `GetExileResTime` после уже
 /// снятого `timeGetTime`; отсутствие записи не требует `_exile_time`.
    pub fn exile_remaining_time(
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

    pub fn set_quest_switch(
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
        let minister = self.get_minister_mut(job)?;
        let previous = minister.quest_switch;
        minister.quest_switch = enabled;
        Some(CountryQuestSwitchUpdate {
            target: CountryQuestSwitchTarget::Minister { job },
            previous,
            applied: enabled,
        })
    }

    pub fn apply_server_scalar(
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

    pub fn set_country_power(
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

    pub fn set_country_treasury(
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

    pub fn set_country_technology(&mut self, requested: i32) -> CountryScalarUpdate {
        let previous = self.tech_current_exp;
        let applied = requested.min(self.tech_level_up_exp);
        self.tech_current_exp = applied;
        CountryScalarUpdate::TechnologyExperience {
            requested,
            previous,
            applied,
        }
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), CountrySerializeError> {
        let mut minister_jobs = self.ministers.keys().copied().collect::<BTreeSet<_>>();
        minister_jobs.extend(self.null_minister_slots.iter().copied());
        let minister_count = minister_jobs.len();
        let minister_count_u8 = u8::try_from(minister_count)
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
        destination.push(minister_count_u8);
        for job in minister_jobs {
            destination.push(job);
            let player_id = self
                .ministers
                .get(&job)
                .map_or(0, |minister| minister.snapshot.id);
            destination.extend_from_slice(&player_id.to_le_bytes());
        }
        Ok(())
    }

    pub fn clone_save_data(&self, limits: CountryKingSaveLimits) -> CountrySaveSnapshot {
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

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    value
        .split(|byte| *byte == 0)
        .next()
        .unwrap_or_default()
}

fn legacy_country_text(mut text: Vec<u8>) -> Vec<u8> {
    if let Some(terminator) = text.iter().position(|byte| *byte == 0) {
        text.truncate(terminator);
    }
    // Старый `_sprintf` писал в `char[260]`; переполнение и последующий
    // overread были внутренним UB, а не Miracle wire-контрактом.
    text.truncate(259);
    text
}
