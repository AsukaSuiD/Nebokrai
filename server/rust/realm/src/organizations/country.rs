//! Data-контракты государства `CCountry` из `country.cpp/.h`, подтверждённые
//! точной парой `worldserver.exe` и `worldserver.pdb`. Data-уровень
//! перенесён в Realm `organizations/`.
//!
//! Сама `CCountry`, её методы и контекстные трейты остаются в старом
//! `appworld/country/country.rs` до волны владельца. Отчёты, ссылающиеся на
//! ещё не перенесённый `KingPointUpdate` из `appworld/country/king.rs`
//! (AI-, exile-, silence-, absolve-, appoint-, demise-семьи и new-day), а
//! также типы, ссылающиеся на саму `CCountry`, переносятся вместе с ней в следующих волнах.

use std::error::Error;
use std::fmt;

use crate::app::world_message::SendMessageError;
use crate::content::countryparam::{CountryParameterUnavailable, CountryTechLevelLookup};
use crate::organizations::dbcountry::CountryMinisterSaveSnapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryGovernanceContextBlock {
    FactionMasterLookup,
    PlayerFactionLookup,
    UnionLookup,
    OwnedCityMutation,
    FactionDemise,
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
