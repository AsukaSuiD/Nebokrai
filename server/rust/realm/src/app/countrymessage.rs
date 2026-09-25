//! Country-сообщения `OnCountryMessage` из `countrymessage.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Здесь чистые типы ветвей `0x60301`, `0x60304`, `0x60306..0x6031D` и сами
//! обработчики независимых ветвей: хвостовой диспетчер [`on_country_message`]
//! (relay `0x60310`/`0x60311`, игнорируемый `0x6030F`, no-op, four-nation
//! sign-up `0x6031B`, scalar `0x60314`, quest-switch `0x60315`, exile-time
//! `0x60316`), four-nation dispatch `0x60319`/`0x6031C`/`0x6031D` и exploit
//! `0x6031A`. Ветвям `0x60314/15/16` живые объекты страны подаёт узкий шов
//! [`WorldCountryMutGate`](crate::app::world_game_view::WorldCountryMutGate)
//! с адаптером `CCountryHandler` в старом пакете; exploit получает game-
//! контекст через [`WorldGameView`](crate::app::world_game_view::WorldGameView)
//! и DB-шов [`WorldExploitDbView`](crate::app::world_game_view::WorldExploitDbView)
//! по ADR-0013.
//!
//! В старом `appworld/message/countrymessage.rs` остаются governance- и
//! war-обработчики вместе со связкой `WorldCountryWarDeclarationSync`,
//! `WorldCountryWarVictorySync`, `WorldCountryMessageOutcome` и
//! `WorldCountryMessageDispatch`: их поля и варианты цитируют
//! `CountryWarDeclarationReport`/`CountryWarVictoryReport` из старого
//! `countrywarsys.rs`, который переносится вместе с war-системами. Старый
//! файл делегирует хвостовые ветви [`on_country_message`] и реэкспортирует
//! оба семейства для переходных потребителей.

use nebokrai_shared::resources::GlobeSetupSnapshot;
use nebokrai_shared::runtime::put_string_to_file;

use crate::activities::fournationwarsys::{
    CFourNationWarSys, FourNationCountryFailContext, FourNationCountryFailReport,
    FourNationExploitContext, FourNationExploitLoadedDisposition, FourNationExploitLoadedReport,
    FourNationSignUpDisposition, FourNationWarResultContext, FourNationWarResultReport,
    FourNationWarTimeReport,
};
use crate::app::world_game_view::{WorldCountryMutGate, WorldExploitDbView, WorldGameView};
use crate::app::world_message::{CMessage, SendMessageError};
use crate::app::worldserver::AddLogTextDisposition;
use crate::characters::player::PlayerCountryChangeReport;
use crate::characters::playerexploit::PlayerExploitUpdate;
use crate::content::countryparam::{CCountryParam, CountryParameterUnavailable};
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
use crate::persistence::rssetup::WorldTdsClient;

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

/// Исходы хвостового диспетчера [`on_country_message`]: relay-ветви
/// `0x60310`/`0x60311`, подтверждённо игнорируемый `0x6030F`, no-op по
/// неизвестному opcode, four-nation sign-up `0x6031B`, scalar sync
/// `0x60314`, quest-switch sync `0x60315` и exile-time sync `0x60316`.
/// Общий `WorldCountryMessageOutcome` с governance- и war-вариантами
/// остаётся в старом пакете до переноса war-систем; dispatcher старого
/// пакета отображает эти исходы на его варианты один к одному.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryMessageTailOutcome {
    NoOp {
        request_type: i32,
    },
    IgnoredGovernanceRequest {
        request_type: i32,
    },
    Relay(WorldCountryRelayOutcome),
    ScalarSynchronized(WorldCountryScalarSync),
    QuestSwitchSynchronized(WorldCountryQuestSwitchSync),
    ExileTimeSynchronized(WorldCountryExileTimeSync),
    FourNationSignUp(WorldFourNationSignUpSync),
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

/// Ветви `0x6030F`, `0x6031B`, `0x60314`..`0x60316`, relay `0x60310`/`0x60311`
/// и no-op по неизвестному opcode. Порядок проверок, декодирования и запись
/// king-журнала quest-switch повторяют исходный диспетчер буквально; живые
/// объекты страны приходят через [`WorldCountryMutGate`], отправка и online-
/// проверки — через [`WorldGameView`]. Dispatcher старого пакета передаёт
/// сюда остаток каскада после специализированных обработчиков.
pub fn on_country_message(
    game: &dyn WorldGameView,
    country: &mut dyn WorldCountryMutGate,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    mut message: CMessage,
) -> WorldCountryMessageTailOutcome {
    let request_type = message.message_type();
    if request_type == 0x0006_030f {
        return WorldCountryMessageTailOutcome::IgnoredGovernanceRequest { request_type };
    }
    if request_type == 0x0006_031b {
        let source_map_id = message.map_id();
        let source_socket_id = message.socket_id();
        let decoded_country = message.base_mut().get_long();
        let country = decoded_country.unwrap_or(0);
        return WorldCountryMessageTailOutcome::FourNationSignUp(WorldFourNationSignUpSync {
            country,
            country_complete: decoded_country.is_some(),
            source_map_id,
            source_socket_id,
            disposition: CFourNationWarSys::one_country_sign_up(country),
        });
    }
    if request_type == 0x0006_0314 {
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);
        let decoded_selector = message.base_mut().get_char();
        let selector = decoded_selector.unwrap_or(0);
        let decoded_value = message.base_mut().get_long();
        let value = decoded_value.unwrap_or(0);
        let disposition =
            match country.apply_server_scalar(country_id, selector, value, country_parameters) {
                None => WorldCountryScalarDisposition::CountryMissing,
                Some(Ok(Some(update))) => WorldCountryScalarDisposition::Updated(update),
                Some(Ok(None)) => WorldCountryScalarDisposition::SelectorIgnored,
                Some(Err(block)) => WorldCountryScalarDisposition::ParameterUnavailable(block),
            };
        return WorldCountryMessageTailOutcome::ScalarSynchronized(WorldCountryScalarSync {
            country_id,
            country_id_complete: decoded_country_id.is_some(),
            selector,
            selector_complete: decoded_selector.is_some(),
            value,
            value_complete: decoded_value.is_some(),
            disposition,
        });
    }
    if request_type == 0x0006_0315 {
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);
        let decoded_job = message.base_mut().get_byte();
        let job = decoded_job.unwrap_or(0);
        let decoded_raw_switch = message.base_mut().get_byte();
        let raw_switch = decoded_raw_switch.unwrap_or(0);

        let update = country.set_quest_switch(country_id, job, raw_switch != 0);
        let (disposition, log) = match update {
            None => (WorldCountryQuestSwitchDisposition::CountryMissing, None),
            Some(None) => (WorldCountryQuestSwitchDisposition::OfficerMissing, None),
            Some(Some(update)) => {
                let country_name = globe_setup.country_name(country_id);
                let line = country_quest_switch_log_line(
                    country_name.unwrap_or_default(),
                    job,
                    raw_switch,
                );
                put_string_to_file("king", &line);
                (
                    WorldCountryQuestSwitchDisposition::Updated(update),
                    Some(WorldCountryQuestSwitchLog {
                        country_name_complete: country_name.is_some(),
                        line,
                    }),
                )
            }
        };
        return WorldCountryMessageTailOutcome::QuestSwitchSynchronized(
            WorldCountryQuestSwitchSync {
                country_id,
                country_id_complete: decoded_country_id.is_some(),
                job,
                job_complete: decoded_job.is_some(),
                raw_switch,
                raw_switch_complete: decoded_raw_switch.is_some(),
                disposition,
                log,
            },
        );
    }
    if request_type == 0x0006_0316 {
        let decoded_player_id = message.base_mut().get_long();
        let player_id = decoded_player_id.unwrap_or(0);
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);

        let disposition =
            match country.exile_remaining_time(country_id, player_id, country_parameters) {
                None => WorldCountryExileTimeDisposition::CountryMissing,
                Some(Err(block)) => WorldCountryExileTimeDisposition::ParameterUnavailable(block),
                Some(Ok(lookup)) if game.online_player_by_id(player_id as u32).is_none() => {
                    WorldCountryExileTimeDisposition::PlayerMissing { lookup }
                }
                Some(Ok(lookup)) => {
                    let mut response = CMessage::new(0x0007_FF15);
                    response.base_mut().add_long(lookup.remaining_seconds);
                    response.base_mut().add_long(player_id);
                    let wire = response.as_wire_bytes().to_vec();
                    let delivery =
                        response.send_all(game.current_game_server_sender().as_ref());
                    WorldCountryExileTimeDisposition::Broadcast {
                        lookup,
                        wire,
                        delivery,
                    }
                }
            };
        return WorldCountryMessageTailOutcome::ExileTimeSynchronized(
            WorldCountryExileTimeSync {
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                country_id,
                country_id_complete: decoded_country_id.is_some(),
                disposition,
            },
        );
    }
    let response_type = match request_type {
        COUNTRY_RELAY_FIRST => 0x0007_FF11,
        COUNTRY_RELAY_SECOND => 0x0007_FF12,
        _ => {
            return WorldCountryMessageTailOutcome::NoOp { request_type };
        }
    };
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldCountryMessageTailOutcome::Relay(WorldCountryRelayOutcome {
        request_type,
        response_type,
        wire,
        delivery,
    })
}

/// Ветвь `0x60319`: приём результата four-nation войны от GameServer и
/// публикация morale по маршрутам стран в исходном interleaving-е.
pub fn dispatch_four_nation_war_result_message<Context: FourNationWarResultContext + ?Sized>(
    message: &mut CMessage,
    four_nation_war: &mut CFourNationWarSys,
    context: &mut Context,
) -> Option<WorldFourNationWarResultSync> {
    if message.message_type() != 0x60319 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let report = four_nation_war.receive_result_from_game_server(message, context);
    Some(WorldFourNationWarResultSync {
        source_map_id,
        source_socket_id,
        report,
    })
}

/// Ветвь `0x6031C`: доставка war time игрока на GameServer его страны.
pub fn dispatch_four_nation_war_time_message<Context: FourNationWarResultContext + ?Sized>(
    message: &mut CMessage,
    four_nation_war: &mut CFourNationWarSys,
    context: &mut Context,
) -> Option<WorldFourNationWarTimeSync> {
    if message.message_type() != 0x6031c {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_war_time = message.base_mut().get_long();
    let war_time = decoded_war_time.unwrap_or(0) as u32;
    let decoded_country = message.base_mut().get_long();
    let country = decoded_country.unwrap_or(0);
    let report = four_nation_war.send_player_war_time_to_game_server(
        player_id, war_time, country, context,
    );
    Some(WorldFourNationWarTimeSync {
        source_map_id,
        source_socket_id,
        numeric_payload_complete: [
            decoded_player_id.is_some(),
            decoded_war_time.is_some(),
            decoded_country.is_some(),
        ],
        report,
    })
}

/// Ветвь `0x6031D`: top-info о проигрыше страны. Текст и его доставка
/// остаются за контекстом вызывающей стороны (`send_top_info` идёт через
/// organizing-очередь владельца игры), здесь только decode и порядок вызова.
pub fn dispatch_four_nation_country_fail_message<Context: FourNationCountryFailContext + ?Sized>(
    message: &mut CMessage,
    four_nation_war: &CFourNationWarSys,
    context: &mut Context,
) -> Option<WorldFourNationCountryFailSync> {
    if message.message_type() != 0x6031d {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_country = message.base_mut().get_long();
    let country = decoded_country.unwrap_or(0);
    let decoded_failed_country = message.base_mut().get_long();
    let failed_country = decoded_failed_country.unwrap_or(0);
    let report = four_nation_war.one_country_fail(country, failed_country, context);
    Some(WorldFourNationCountryFailSync {
        source_map_id,
        source_socket_id,
        numeric_payload_complete: [
            decoded_country.is_some(),
            decoded_failed_country.is_some(),
        ],
        report,
    })
}

/// Decode ветви `0x6031A`: фиксированная пара long без source/tail проверок.
/// Возвращает `None` для чужого opcode.
pub fn decode_four_nation_exploit_message(
    message: &mut CMessage,
) -> Option<WorldFourNationExploitRequest> {
    if message.message_type() != 0x6031a {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_increment = message.base_mut().get_long();
    let increment = decoded_increment.unwrap_or(0);
    Some(WorldFourNationExploitRequest {
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        increment,
        increment_complete: decoded_increment.is_some(),
        source_map_id,
        source_socket_id,
    })
}

/// Game-контекст exploit-ветви поверх [`WorldGameView`]: повторяет четыре
/// вызова effects-структуры старого пакета через методы view с теми же
/// значениями (`player_game_server` свёрнут в индекс маршрута).
struct WorldGameExploitContext<'a> {
    game: &'a mut dyn WorldGameView,
}

impl FourNationExploitContext for WorldGameExploitContext<'_> {
    fn map_player_exists(&mut self, player_id: u32) -> bool {
        self.game.map_player(player_id).is_some()
    }

    fn player_game_server_map_id(&mut self, player_id: i32) -> Option<i32> {
        self.game
            .player_game_server(player_id)
            .map(|game_server| game_server.index as i32)
    }

    fn add_local_player_exploit(
        &mut self,
        player_id: u32,
        increment: i32,
    ) -> Option<PlayerExploitUpdate> {
        self.game.add_map_player_exploit_wrapping(player_id, increment)
    }

    fn send_to_map_id(
        &mut self,
        message: &CMessage,
        map_id: i32,
    ) -> Result<i32, SendMessageError> {
        message.send_to_map_id(self.game.current_game_server_sender().as_ref(), map_id)
    }
}

/// Ветвь `0x6031A`: конвертация загруженного morale в exploit. При игроке
/// вне карты выполняется offline UPDATE через [`WorldExploitDbView`], после
/// успешного или невозможного (нет подключения к БД) обращения convert
/// повторяется, как у машинного owner-а: первичный `PlayerMissing` мог
/// разрешиться публикацией игрока за время DB-вызова. Порядок decode,
/// convert, журнала ошибки подключения и повторных convert сохранён
/// буквально из исходного диспетчера.
pub async fn dispatch_four_nation_exploit_message(
    message: &mut CMessage,
    game: &mut dyn WorldGameView,
    four_nation_war: &mut CFourNationWarSys,
    rs_player: &mut dyn WorldExploitDbView,
    player_database: Option<&mut WorldTdsClient>,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
) -> Option<WorldFourNationExploitSync> {
    let request = decode_four_nation_exploit_message(message)?;
    let initial = {
        let mut effects = WorldGameExploitContext { game: &mut *game };
        four_nation_war.convert_loaded_morale_to_exploit(
            request.player_id,
            request.increment,
            &mut effects,
        )
    };
    let mut database = WorldFourNationExploitDatabaseDisposition::NotRequired;
    let mut after_database = None;

    if matches!(
        &initial.disposition,
        FourNationExploitLoadedDisposition::PlayerMissing
    ) {
        match player_database {
            None => {
                let log = add_log_text(b"Error:failed to connect to DB!!");
                database =
                    WorldFourNationExploitDatabaseDisposition::ConnectionUnavailable {
                        log,
                    };
                let mut effects = WorldGameExploitContext { game: &mut *game };
                after_database = Some(
                    four_nation_war.convert_loaded_morale_to_exploit(
                        request.player_id,
                        request.increment,
                        &mut effects,
                    ),
                );
            }
            Some(active_database) => {
                match rs_player
                    .add_player_exploit(request.increment, request.player_id, active_database)
                    .await
                {
                    Ok(()) => {
                        database = WorldFourNationExploitDatabaseDisposition::Applied;
                        let mut effects = WorldGameExploitContext { game: &mut *game };
                        after_database = Some(
                            four_nation_war.convert_loaded_morale_to_exploit(
                                request.player_id,
                                request.increment,
                                &mut effects,
                            ),
                        );
                    }
                    Err(error) => {
                        database = WorldFourNationExploitDatabaseDisposition::ExecutionFailed {
                            error,
                        };
                    }
                }
            }
        }
    }

    Some(WorldFourNationExploitSync {
        request,
        initial,
        database,
        after_database,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryWarVictoryDispatchError<ContextBlock> {
    pub source: ContextBlock,
}
