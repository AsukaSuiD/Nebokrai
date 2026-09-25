//! Data-контракты country-сообщений `OnCountryMessage` из `countrymessage.cpp`,
//! подтверждённые `worldserver.exe` и `worldserver.pdb`.
//!
//! Здесь чистые типы ветвей `0x60301`, `0x60304`, `0x60306..0x6031D`:
//! governance sync/disposition, player lists, scalar sync, four-nation и
//! relay-константы `COUNTRY_RELAY_FIRST`/`COUNTRY_RELAY_SECOND`, а также
//! helper строки quest-switch журнала. Сам диспетчер `on_country_message`,
//! fn-обработчики и связка `WorldCountryWarDeclarationSync`,
//! `WorldCountryWarVictorySync`, `WorldCountryMessageOutcome` и
//! `WorldCountryMessageDispatch` остаются в старом
//! `appworld/message/countrymessage.rs`: их поля и варианты цитируют
//! `CountryWarDeclarationReport` и `CountryWarVictoryReport` из старого
//! `countrywarsys.rs`, которые переносятся вместе с war-системами. Старый
//! файл реэкспортирует эти типы для переходных потребителей.

use crate::activities::fournationwarsys::{
    FourNationCountryFailReport, FourNationExploitLoadedReport, FourNationSignUpDisposition,
    FourNationWarResultReport, FourNationWarTimeReport,
};
use crate::app::world_message::SendMessageError;
use crate::app::worldserver::AddLogTextDisposition;
use crate::characters::player::PlayerCountryChangeReport;
use crate::content::countryparam::CountryParameterUnavailable;
use crate::organizations::country::{
    CountryAbsolveReport, CountryAppointMinisterReport, CountryBaseInfoDisposition,
    CountryCanAbsolveDisposition, CountryCanAppointMinisterDisposition, CountryCanDemiseDisposition,
    CountryCanDeposeMinisterDisposition, CountryCanExileDisposition, CountryCanSilenceDisposition,
    CountryDemiseReport, CountryDeposeMinisterReport, CountryExileRequestDisposition,
    CountryExileTimeLookup, CountryGovernanceContextBlock, CountryInitialKingReport,
    CountryPlayersListContextBlock, CountryPlayersListReport, CountryQuestSwitchUpdate,
    CountryScalarUpdate, CountrySetKingReport, CountrySilenceReport, CountrySuccessExiledReport,
    KingPointUpdate,
};
use crate::organizations::countryhandler::CountryHandlerNewDayReport;

pub const COUNTRY_RELAY_FIRST: i32 = 0x0006_0310;
pub const COUNTRY_RELAY_SECOND: i32 = 0x0006_0311;

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryRelayOutcome {
    pub request_type: i32,
    pub response_type: i32,
    pub wire: Vec<u8>,
    pub delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryPlayerChangeDisposition {
    PlayerMissing,
    Responded {
        change: PlayerCountryChangeReport,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryPlayerChangeSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub player_id: i32,
    pub player_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryPlayerChangeDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryNewDaySync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub report: CountryHandlerNewDayReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCountryScalarDisposition {
    CountryMissing,
    SelectorIgnored,
    ParameterUnavailable(CountryParameterUnavailable),
    Updated(CountryScalarUpdate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldCountryScalarSync {
    pub country_id: u8,
    pub country_id_complete: bool,
    pub selector: i8,
    pub selector_complete: bool,
    pub value: i32,
    pub value_complete: bool,
    pub disposition: WorldCountryScalarDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCountryQuestSwitchDisposition {
    CountryMissing,
    OfficerMissing,
    Updated(CountryQuestSwitchUpdate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCountryQuestSwitchLog {
    pub country_name_complete: bool,
    pub line: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCountryQuestSwitchSync {
    pub country_id: u8,
    pub country_id_complete: bool,
    pub job: u8,
    pub job_complete: bool,
    pub raw_switch: u8,
    pub raw_switch_complete: bool,
    pub disposition: WorldCountryQuestSwitchDisposition,
    pub log: Option<WorldCountryQuestSwitchLog>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryExileTimeDisposition {
    CountryMissing,
    ParameterUnavailable(CountryParameterUnavailable),
    PlayerMissing {
        lookup: CountryExileTimeLookup,
    },
    Broadcast {
        lookup: CountryExileTimeLookup,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryExileTimeSync {
    pub player_id: i32,
    pub player_id_complete: bool,
    pub country_id: u8,
    pub country_id_complete: bool,
    pub disposition: WorldCountryExileTimeDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryExileResultDisposition {
    CountryMissing,
    PlayerListTruncated {
        advertised_count: i32,
        available_complete_ids: usize,
    },
    PlayerListAllocationBlocked {
        advertised_count: i32,
    },
    Broadcast {
        advertised_count: i32,
        player_ids: Vec<i32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryExileResultSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub player_id: i32,
    pub player_id_complete: bool,
    pub raw_success: i8,
    pub success_complete: bool,
    pub country_id: u8,
    pub country_id_complete: bool,
    pub country_report: Option<CountrySuccessExiledReport>,
    pub count_complete: Option<bool>,
    pub disposition: WorldCountryExileResultDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryExileRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanExileDisposition),
    Requested {
        operation: CountryCanExileDisposition,
        legacy_result: i32,
        request: CountryExileRequestDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryExileRequestSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub target_player_id: i32,
    pub target_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryExileRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountrySilenceRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanSilenceDisposition),
    Applied {
        operation: CountryCanSilenceDisposition,
        report: CountrySilenceReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountrySilenceRequestSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub target_player_id: i32,
    pub target_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountrySilenceRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryAbsolveRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanAbsolveDisposition),
    Applied {
        operation: CountryCanAbsolveDisposition,
        report: CountryAbsolveReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryAbsolveRequestSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub target_player_id: i32,
    pub target_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryAbsolveRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryDeposeMinisterDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanDeposeMinisterDisposition),
    MinisterRejected,
    Applied {
        operation: CountryCanDeposeMinisterDisposition,
        report: CountryDeposeMinisterReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryDeposeMinisterSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub target_player_id: i32,
    pub target_complete: bool,
    pub job: u8,
    pub job_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryDeposeMinisterDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryAppointMinisterDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanAppointMinisterDisposition),
    Applied {
        operation: CountryCanAppointMinisterDisposition,
        report: CountryAppointMinisterReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryAppointMinisterSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub target_player_id: i32,
    pub target_complete: bool,
    pub job: u8,
    pub job_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryAppointMinisterDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryDemiseDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanDemiseDisposition),
    Applied {
        operation: CountryCanDemiseDisposition,
        report: CountryDemiseReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryDemiseSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub target_player_id: i32,
    pub target_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryDemiseDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryDirectAppointmentDisposition {
    CountryMissing,
    ParameterUnavailable(CountryParameterUnavailable),
    ContextBlocked(CountryGovernanceContextBlock),
    King {
        initial_control_point: KingPointUpdate,
        set: CountrySetKingReport,
        register: CountryInitialKingReport,
    },
    Minister {
        depose: CountryDeposeMinisterReport,
        appoint: CountryAppointMinisterReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryDirectAppointmentSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub country_id: u8,
    pub country_complete: bool,
    pub player_id: i32,
    pub player_complete: bool,
    pub appoint: u8,
    pub appoint_complete: bool,
    pub disposition: WorldCountryDirectAppointmentDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryInfoDisposition {
    CountryMissing,
    KingRejected,
    Sent(CountryBaseInfoDisposition),
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryInfoSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryInfoDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryPlayersListDisposition {
    CountryMissing,
    KingRejected,
    ContextBlocked(CountryPlayersListContextBlock),
    Sent(CountryPlayersListReport),
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryPlayersListSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub page: i32,
    pub page_complete: bool,
    pub king_player_id: i32,
    pub king_complete: bool,
    pub country_id: u8,
    pub country_complete: bool,
    pub disposition: WorldCountryPlayersListDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldFourNationWarResultSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub report: FourNationWarResultReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldFourNationExploitRequest {
    pub player_id: i32,
    pub player_id_complete: bool,
    pub increment: i32,
    pub increment_complete: bool,
    pub source_map_id: i32,
    pub source_socket_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldFourNationExploitDatabaseDisposition {
    NotRequired,
    ConnectionUnavailable { log: AddLogTextDisposition },
    Applied,
    ExecutionFailed { error: String },
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldFourNationExploitSync {
    pub request: WorldFourNationExploitRequest,
    pub initial: FourNationExploitLoadedReport,
    pub database: WorldFourNationExploitDatabaseDisposition,
    pub after_database: Option<FourNationExploitLoadedReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldFourNationSignUpSync {
    pub country: i32,
    pub country_complete: bool,
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub disposition: FourNationSignUpDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldFourNationWarTimeSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub numeric_payload_complete: [bool; 3],
    pub report: FourNationWarTimeReport,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WorldFourNationCountryFailSync {
    pub source_map_id: i32,
    pub source_socket_id: i32,
    pub numeric_payload_complete: [bool; 2],
    pub report: FourNationCountryFailReport,
}

pub fn country_quest_switch_log_line(country_name: &[u8], job: u8, raw_switch: u8) -> Vec<u8> {
    let mut line = Vec::with_capacity(country_name.len() + 40);
    line.extend_from_slice(country_name);
    line.extend_from_slice(b" : Country Task: ");
    line.extend_from_slice(job.to_string().as_bytes());
    line.extend_from_slice(b" ( ");
    line.extend_from_slice(raw_switch.to_string().as_bytes());
    line.extend_from_slice(b" )");
    line.extend_from_slice(&[0xa1, 0xa3]);
    line
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryWarVictoryDispatchError<ContextBlock> {
    pub source: ContextBlock,
}
