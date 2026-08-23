//! Country-сообщения `OnCountryMessage` из `countrymessage.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Ветки `0x60301`, `0x60304`, `0x60306..0x6031D` сохраняют управление
//! королём/министрами, scalar sync, player lists и country-war events.
//! Неизвестный opcode — no-op; relays меняют только type и вызывают `SendAll`.
//!
//! Governance идёт через `GetCountry -> permission -> operation`; byte job не
//! проверяется заранее. Scalar limits несимметричны: treasury/power зажимаются
//! с двух сторон, tech exp только сверху, tech level только снизу, king points
//! только сверху.
//!
//! Exile time вычисляется wrapping-миллисекундами и ограничивается нулём.
//! Player lists сохраняют page arithmetic и GM filter. War branches не вводят
//! source/tail checks. Явный main-loop context и safe codec заменяют singleton
//! и overread без изменения вызовов.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::tools::put_string_to_file;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::worldserver::appworld::country::country::{
    CountryAbsolveReport, CountryAppointMinisterReport, CountryBaseInfoDisposition,
    CountryCanAbsolveDisposition,
    CountryCanAppointMinisterDisposition, CountryCanExileDisposition,
    CountryCanDemiseDisposition, CountryCanDeposeMinisterDisposition, CountryCanSilenceDisposition,
    CountryDemiseReport, CountryGovernanceContextBlock, CountryInitialKingReport,
    CountryDeposeMinisterReport, CountryExileRequestDisposition,
    CountryExileResultContext, CountryExileTimeLookup, CountryQuestSwitchUpdate,
    CountryPlayersListContext, CountryPlayersListContextBlock, CountryPlayersListReport,
    CountrySetKingReport, CountrySetNewDayContext,
    CountryScalarUpdate, CountrySilenceReport, CountrySuccessExiledReport,
};
use crate::worldserver::appworld::country::countryhandler::{
    CCountryHandler, CountryHandlerNewDayReport,
};
use crate::worldserver::appworld::country::countryparam::{
    CCountryParam, CountryParameterUnavailable,
};
use crate::worldserver::appworld::country::king::{KingPointUpdate, set_control_point};
use crate::worldserver::appworld::organizingsystem::fournationwarsys::{
    CFourNationWarSys, FourNationCountryFailContext, FourNationCountryFailReport,
    FourNationExploitLoadedReport, FourNationSignUpDisposition, FourNationWarResultContext,
    FourNationWarResultReport, FourNationWarTimeReport,
};
use crate::worldserver::appworld::player::PlayerCountryChangeReport;
use crate::worldserver::worldserver::game::{CGame, legacy_tick_ms};
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

use super::super::country::countrywarsys::{
    CountryWarDeclarationContext, CountryWarDeclarationReport, CountryWarSys,
    CountryWarVictoryContext, CountryWarVictoryReport,
};

const COUNTRY_RELAY_FIRST: i32 = 0x0006_0310;
const COUNTRY_RELAY_SECOND: i32 = 0x0006_0311;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryRelayOutcome {
    pub(crate) request_type: i32,
    pub(crate) response_type: i32,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryPlayerChangeDisposition {
    PlayerMissing,
    Responded {
        change: PlayerCountryChangeReport,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryPlayerChangeSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) player_id: i32,
    pub(crate) player_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryPlayerChangeDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryNewDaySync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) report: CountryHandlerNewDayReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryScalarDisposition {
    CountryMissing,
    SelectorIgnored,
    ParameterUnavailable(CountryParameterUnavailable),
    Updated(CountryScalarUpdate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryScalarSync {
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) selector: i8,
    pub(crate) selector_complete: bool,
    pub(crate) value: i32,
    pub(crate) value_complete: bool,
    pub(crate) disposition: WorldCountryScalarDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryQuestSwitchDisposition {
    CountryMissing,
    OfficerMissing,
    Updated(CountryQuestSwitchUpdate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryQuestSwitchLog {
    pub(crate) country_name_complete: bool,
    pub(crate) line: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryQuestSwitchSync {
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) job: u8,
    pub(crate) job_complete: bool,
    pub(crate) raw_switch: u8,
    pub(crate) raw_switch_complete: bool,
    pub(crate) disposition: WorldCountryQuestSwitchDisposition,
    pub(crate) log: Option<WorldCountryQuestSwitchLog>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryExileTimeDisposition {
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
pub(crate) struct WorldCountryExileTimeSync {
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) disposition: WorldCountryExileTimeDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryExileResultDisposition {
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
pub(crate) struct WorldCountryExileResultSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) raw_success: i8,
    pub(crate) success_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_id_complete: bool,
    pub(crate) country_report: Option<CountrySuccessExiledReport>,
    pub(crate) count_complete: Option<bool>,
    pub(crate) disposition: WorldCountryExileResultDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryExileRequestDisposition {
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
pub(crate) struct WorldCountryExileRequestSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryExileRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountrySilenceRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanSilenceDisposition),
    Applied {
        operation: CountryCanSilenceDisposition,
        report: CountrySilenceReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountrySilenceRequestSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountrySilenceRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryAbsolveRequestDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanAbsolveDisposition),
    Applied {
        operation: CountryCanAbsolveDisposition,
        report: CountryAbsolveReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryAbsolveRequestSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryAbsolveRequestDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryDeposeMinisterDisposition {
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
pub(crate) struct WorldCountryDeposeMinisterSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) job: u8,
    pub(crate) job_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryDeposeMinisterDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryAppointMinisterDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanAppointMinisterDisposition),
    Applied {
        operation: CountryCanAppointMinisterDisposition,
        report: CountryAppointMinisterReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryAppointMinisterSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) job: u8,
    pub(crate) job_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryAppointMinisterDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryDemiseDisposition {
    CountryMissing,
    KingRejected,
    OperationRejected(CountryCanDemiseDisposition),
    Applied {
        operation: CountryCanDemiseDisposition,
        report: CountryDemiseReport,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryDemiseSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) target_player_id: i32,
    pub(crate) target_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryDemiseDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryDirectAppointmentDisposition {
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
pub(crate) struct WorldCountryDirectAppointmentSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) player_id: i32,
    pub(crate) player_complete: bool,
    pub(crate) appoint: u8,
    pub(crate) appoint_complete: bool,
    pub(crate) disposition: WorldCountryDirectAppointmentDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryInfoDisposition {
    CountryMissing,
    KingRejected,
    Sent(CountryBaseInfoDisposition),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryInfoSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryInfoDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryPlayersListDisposition {
    CountryMissing,
    KingRejected,
    ContextBlocked(CountryPlayersListContextBlock),
    Sent(CountryPlayersListReport),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryPlayersListSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) page: i32,
    pub(crate) page_complete: bool,
    pub(crate) king_player_id: i32,
    pub(crate) king_complete: bool,
    pub(crate) country_id: u8,
    pub(crate) country_complete: bool,
    pub(crate) disposition: WorldCountryPlayersListDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryWarDeclarationSync {
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) target_country: i32,
    pub(crate) target_country_complete: bool,
    pub(crate) source_map_id: i32,
    pub(crate) declaration: CountryWarDeclarationReport,
    pub(crate) response_wire: Vec<u8>,
    pub(crate) response_delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationWarResultSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) report: FourNationWarResultReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationExploitRequest {
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) increment: i32,
    pub(crate) increment_complete: bool,
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldFourNationExploitDatabaseDisposition {
    NotRequired,
    ConnectionUnavailable { log: AddLogTextDisposition },
    Applied,
    ExecutionFailed { error: String },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationExploitSync {
    pub(crate) request: WorldFourNationExploitRequest,
    pub(crate) initial: FourNationExploitLoadedReport,
    pub(crate) database: WorldFourNationExploitDatabaseDisposition,
    pub(crate) after_database: Option<FourNationExploitLoadedReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationSignUpSync {
    pub(crate) country: i32,
    pub(crate) country_complete: bool,
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) disposition: FourNationSignUpDisposition,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationWarTimeSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) numeric_payload_complete: [bool; 3],
    pub(crate) report: FourNationWarTimeReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldFourNationCountryFailSync {
    pub(crate) source_map_id: i32,
    pub(crate) source_socket_id: i32,
    pub(crate) numeric_payload_complete: [bool; 2],
    pub(crate) report: FourNationCountryFailReport,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryMessageOutcome {
    NoOp {
        request_type: i32,
    },
    Relay(WorldCountryRelayOutcome),
    PlayerCountryChanged(WorldCountryPlayerChangeSync),
    NewDaySet(WorldCountryNewDaySync),
    ScalarSynchronized(WorldCountryScalarSync),
    QuestSwitchSynchronized(WorldCountryQuestSwitchSync),
    ExileTimeSynchronized(WorldCountryExileTimeSync),
    ExileRequested(WorldCountryExileRequestSync),
    ExileResultSynchronized(WorldCountryExileResultSync),
    SilenceRequested(WorldCountrySilenceRequestSync),
    AbsolveRequested(WorldCountryAbsolveRequestSync),
    MinisterDeposed(WorldCountryDeposeMinisterSync),
    MinisterAppointed(WorldCountryAppointMinisterSync),
    KingDemised(WorldCountryDemiseSync),
    CountryAppointedDirectly(WorldCountryDirectAppointmentSync),
    CountryInfoSent(WorldCountryInfoSync),
    CountryPlayersListed(WorldCountryPlayersListSync),
    CountryWarDeclared(WorldCountryWarDeclarationSync),
    CountryWarVictory(WorldCountryWarVictorySync),
    FourNationWarResult(WorldFourNationWarResultSync),
    FourNationExploit(WorldFourNationExploitSync),
    FourNationSignUp(WorldFourNationSignUpSync),
    FourNationWarTime(WorldFourNationWarTimeSync),
    FourNationCountryFail(WorldFourNationCountryFailSync),
}

pub(crate) enum WorldCountryMessageDispatch {
    Handled(WorldCountryMessageOutcome),
    Pending(CMessage),
}

pub(crate) fn on_country_message(
    game: &CGame,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    mut message: CMessage,
) -> WorldCountryMessageDispatch {
    let request_type = message.message_type();
    if request_type == 0x0006_031b {
        let source_map_id = message.map_id();
        let source_socket_id = message.socket_id();
        let decoded_country = message.base_mut().get_long();
        let country = decoded_country.unwrap_or(0);
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::FourNationSignUp(WorldFourNationSignUpSync {
                country,
                country_complete: decoded_country.is_some(),
                source_map_id,
                source_socket_id,
                disposition: CFourNationWarSys::one_country_sign_up(country),
            }),
        );
    }
    if request_type == 0x0006_0314 {
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);
        let decoded_selector = message.base_mut().get_char();
        let selector = decoded_selector.unwrap_or(0);
        let decoded_value = message.base_mut().get_long();
        let value = decoded_value.unwrap_or(0);
        let disposition = match country_handler.get_country_mut(country_id) {
            None => WorldCountryScalarDisposition::CountryMissing,
            Some(country) => match country.apply_server_scalar(selector, value, country_parameters)
            {
                Ok(Some(update)) => WorldCountryScalarDisposition::Updated(update),
                Ok(None) => WorldCountryScalarDisposition::SelectorIgnored,
                Err(block) => WorldCountryScalarDisposition::ParameterUnavailable(block),
            },
        };
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::ScalarSynchronized(WorldCountryScalarSync {
                country_id,
                country_id_complete: decoded_country_id.is_some(),
                selector,
                selector_complete: decoded_selector.is_some(),
                value,
                value_complete: decoded_value.is_some(),
                disposition,
            }),
        );
    }
    if request_type == 0x0006_0315 {
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);
        let decoded_job = message.base_mut().get_byte();
        let job = decoded_job.unwrap_or(0);
        let decoded_raw_switch = message.base_mut().get_byte();
        let raw_switch = decoded_raw_switch.unwrap_or(0);

        let update = country_handler
            .get_country_mut(country_id)
            .map(|country| country.set_quest_switch(job, raw_switch != 0));
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
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::QuestSwitchSynchronized(
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
            ),
        );
    }
    if request_type == 0x0006_0316 {
        let decoded_player_id = message.base_mut().get_long();
        let player_id = decoded_player_id.unwrap_or(0);
        let decoded_country_id = message.base_mut().get_byte();
        let country_id = decoded_country_id.unwrap_or(0);

        let disposition = match country_handler.get_country(country_id) {
            None => WorldCountryExileTimeDisposition::CountryMissing,
            Some(country) => {
                let sampled_at_ms = legacy_tick_ms();
                match country.exile_remaining_time(
                    player_id,
                    sampled_at_ms,
                    country_parameters,
                ) {
                    Err(block) => {
                        WorldCountryExileTimeDisposition::ParameterUnavailable(block)
                    }
                    Ok(lookup) if game.online_player_by_id(player_id as u32).is_none() => {
                        WorldCountryExileTimeDisposition::PlayerMissing { lookup }
                    }
                    Ok(lookup) => {
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
                }
            }
        };
        return WorldCountryMessageDispatch::Handled(
            WorldCountryMessageOutcome::ExileTimeSynchronized(WorldCountryExileTimeSync {
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                country_id,
                country_id_complete: decoded_country_id.is_some(),
                disposition,
            }),
        );
    }
    let response_type = match request_type {
        COUNTRY_RELAY_FIRST => 0x0007_FF11,
        COUNTRY_RELAY_SECOND => 0x0007_FF12,
        _ => {
            return WorldCountryMessageDispatch::Handled(WorldCountryMessageOutcome::NoOp {
                request_type,
            });
        }
    };
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldCountryMessageDispatch::Handled(WorldCountryMessageOutcome::Relay(
        WorldCountryRelayOutcome {
            request_type,
            response_type,
            wire,
            delivery,
        },
    ))
}

fn country_quest_switch_log_line(country_name: &[u8], job: u8, raw_switch: u8) -> Vec<u8> {
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
pub(crate) struct WorldCountryWarVictorySync {
    pub(crate) country: u8,
    pub(crate) country_complete: bool,
    pub(crate) report: CountryWarVictoryReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarVictoryDispatchError<ContextBlock> {
    pub(crate) source: ContextBlock,
}

pub(crate) fn dispatch_country_exile_result_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryExileResultSync> {
    if message.message_type() != 0x6030e {
        return None;
    }

    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_success = message.base_mut().get_char();
    let raw_success = decoded_success.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let Some(country) = country_handler.get_country_mut(country_id) else {
        return Some(WorldCountryExileResultSync {
            source_map_id,
            source_socket_id,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            raw_success,
            success_complete: decoded_success.is_some(),
            country_id,
            country_id_complete: decoded_country.is_some(),
            country_report: None,
            count_complete: None,
            disposition: WorldCountryExileResultDisposition::CountryMissing,
        });
    };

    let country_report = country.success_exiled(
        player_id,
        raw_success != 0,
        country_parameters,
        context,
    );
    let decoded_count = message.base_mut().get_long();
    let advertised_count = decoded_count.unwrap_or(0);

    let mut player_ids = Vec::new();
    if advertised_count > 0 {
        let cursor = message.base_mut().cursor();
        let remaining_bytes = message.as_wire_bytes().len().saturating_sub(cursor);
        let advertised_count_usize = advertised_count as usize;
        let required_bytes = advertised_count_usize.saturating_mul(size_of::<i32>());
        if required_bytes > remaining_bytes {
            return Some(WorldCountryExileResultSync {
                source_map_id,
                source_socket_id,
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                raw_success,
                success_complete: decoded_success.is_some(),
                country_id,
                country_id_complete: decoded_country.is_some(),
                country_report: Some(country_report),
                count_complete: Some(decoded_count.is_some()),
                disposition: WorldCountryExileResultDisposition::PlayerListTruncated {
                    advertised_count,
                    available_complete_ids: remaining_bytes / size_of::<i32>(),
                },
            });
        }
        if player_ids.try_reserve_exact(advertised_count_usize).is_err() {
            return Some(WorldCountryExileResultSync {
                source_map_id,
                source_socket_id,
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                raw_success,
                success_complete: decoded_success.is_some(),
                country_id,
                country_id_complete: decoded_country.is_some(),
                country_report: Some(country_report),
                count_complete: Some(decoded_count.is_some()),
                disposition: WorldCountryExileResultDisposition::PlayerListAllocationBlocked {
                    advertised_count,
                },
            });
        }
        for _ in 0..advertised_count_usize {
            player_ids.push(
                message
                    .base_mut()
                    .get_long()
                    .expect("полнота exile player-list проверена до декодирования"),
            );
        }
    }

    let mut response = CMessage::new(0x0007_FF15);
    response.base_mut().add_byte(country_id);
    response.base_mut().add_long(advertised_count);
    for &exiled_player_id in &player_ids {
        response.base_mut().add_long(exiled_player_id);
    }
    let wire = response.as_wire_bytes().to_vec();
    let delivery = context.send_all(&response);
    Some(WorldCountryExileResultSync {
        source_map_id,
        source_socket_id,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        raw_success,
        success_complete: decoded_success.is_some(),
        country_id,
        country_id_complete: decoded_country.is_some(),
        country_report: Some(country_report),
        count_complete: Some(decoded_count.is_some()),
        disposition: WorldCountryExileResultDisposition::Broadcast {
            advertised_count,
            player_ids,
            wire,
            delivery,
        },
    })
}

pub(crate) fn dispatch_country_exile_request_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryExileRequestSync> {
    if message.message_type() != 0x6030d {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let disposition = match country_handler.get_country(country_id) {
        None => WorldCountryExileRequestDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryExileRequestDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_exile(country_parameters, context);
            if !matches!(operation, CountryCanExileDisposition::Allowed) {
                WorldCountryExileRequestDisposition::OperationRejected(operation)
            } else {
                let request = country.exile(target_player_id, country_parameters, context);
                let legacy_result = if matches!(request, CountryExileRequestDisposition::Sent { .. }) {
                    target_player_id
                } else {
                    0
                };
                WorldCountryExileRequestDisposition::Requested {
                    operation,
                    legacy_result,
                    request,
                }
            }
        }
    };
    Some(WorldCountryExileRequestSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_silence_request_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountrySilenceRequestSync> {
    if message.message_type() != 0x6030c {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountrySilenceRequestDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountrySilenceRequestDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_silence(country_parameters, context);
            if !matches!(operation, CountryCanSilenceDisposition::Allowed) {
                WorldCountrySilenceRequestDisposition::OperationRejected(operation)
            } else {
                let report = country.silence(target_player_id, country_parameters, context);
                WorldCountrySilenceRequestDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountrySilenceRequestSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_absolve_request_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryAbsolveRequestSync> {
    if message.message_type() != 0x6030b {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;

    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryAbsolveRequestDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryAbsolveRequestDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_absolve(country_parameters, context);
            if !matches!(operation, CountryCanAbsolveDisposition::Allowed) {
                WorldCountryAbsolveRequestDisposition::OperationRejected(operation)
            } else {
                let report = country.absolve(target_player_id, country_parameters, context);
                WorldCountryAbsolveRequestDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountryAbsolveRequestSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_depose_minister_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryDeposeMinisterSync> {
    if message.message_type() != 0x6030a {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_job = message.base_mut().get_char();
    let job = decoded_job.unwrap_or(0) as u8;
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryDeposeMinisterDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryDeposeMinisterDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_depose_minister(country_parameters, context);
            if !matches!(operation, CountryCanDeposeMinisterDisposition::Allowed) {
                WorldCountryDeposeMinisterDisposition::OperationRejected(operation)
            } else if !country.authorize_minister(target_player_id, job, context) {
                WorldCountryDeposeMinisterDisposition::MinisterRejected
            } else {
                let report = country.depose_minister(job, 7, country_parameters, context);
                WorldCountryDeposeMinisterDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountryDeposeMinisterSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        job,
        job_complete: decoded_job.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_appoint_minister_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryAppointMinisterSync> {
    if message.message_type() != 0x60309 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_job = message.base_mut().get_char();
    let job = decoded_job.unwrap_or(0) as u8;
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryAppointMinisterDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryAppointMinisterDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_appoint_minister(country_parameters, context);
            if !matches!(operation, CountryCanAppointMinisterDisposition::Allowed) {
                WorldCountryAppointMinisterDisposition::OperationRejected(operation)
            } else {
                let report = country.appoint_minister(
                    target_player_id,
                    job,
                    6,
                    country_parameters,
                    context,
                );
                WorldCountryAppointMinisterDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountryAppointMinisterSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        job,
        job_complete: decoded_job.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_players_list_message<
    Context: CountryPlayersListContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &CCountryHandler,
    context: &mut Context,
) -> Option<WorldCountryPlayersListSync> {
    if message.message_type() != 0x60307 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_page = message.base_mut().get_long();
    let page = decoded_page.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country(country_id) {
        None => WorldCountryPlayersListDisposition::CountryMissing,
        Some(country) if !country.authorize_king_for_players(king_player_id, context) => {
            WorldCountryPlayersListDisposition::KingRejected
        }
        Some(country) => match country.get_players_list(page, context) {
            Ok(report) => WorldCountryPlayersListDisposition::Sent(report),
            Err(block) => WorldCountryPlayersListDisposition::ContextBlocked(block),
        },
    };
    Some(WorldCountryPlayersListSync {
        source_map_id,
        source_socket_id,
        page,
        page_complete: decoded_page.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_player_change_message(
    message: &mut CMessage,
    game: &mut CGame,
    country_handler: &CCountryHandler,
) -> Option<WorldCountryPlayerChangeSync> {
    if message.message_type() != 0x60301 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_player = message.base_mut().get_long();
    let player_id = decoded_player.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match game.change_online_player_country(
        player_id as u32,
        country_id,
        |requested_country| country_handler.get_country(requested_country).is_some(),
    ) {
        None => WorldCountryPlayerChangeDisposition::PlayerMissing,
        Some(change) => {
            let mut response = CMessage::new(0x7ff01);
            response.base_mut().add_long(player_id);
            response.base_mut().add_long(change.legacy_result);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                source_map_id,
            );
            WorldCountryPlayerChangeDisposition::Responded {
                change,
                wire,
                delivery,
            }
        }
    };
    Some(WorldCountryPlayerChangeSync {
        source_map_id,
        source_socket_id,
        player_id,
        player_complete: decoded_player.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_new_day_message<Context: CountrySetNewDayContext + ?Sized>(
    message: &CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryNewDaySync> {
    if message.message_type() != 0x60313 {
        return None;
    }
    Some(WorldCountryNewDaySync {
        source_map_id: message.map_id(),
        source_socket_id: message.socket_id(),
        report: country_handler.set_new_day(10, country_parameters, context),
    })
}

pub(crate) fn dispatch_country_direct_appointment_message<
    Context: CountryExileResultContext + ?Sized,
>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryDirectAppointmentSync> {
    if message.message_type() != 0x60304 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let decoded_player = message.base_mut().get_long();
    let player_id = decoded_player.unwrap_or(0);
    let decoded_appoint = message.base_mut().get_char();
    let appoint = decoded_appoint.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryDirectAppointmentDisposition::CountryMissing,
        Some(country) if appoint == 1 => {
            let initial_control_point = match set_control_point(
                &mut country.king,
                100_000,
                country_parameters,
            ) {
                Ok(update) => update,
                Err(block) => {
                    return Some(WorldCountryDirectAppointmentSync {
                        source_map_id,
                        source_socket_id,
                        country_id,
                        country_complete: decoded_country.is_some(),
                        player_id,
                        player_complete: decoded_player.is_some(),
                        appoint,
                        appoint_complete: decoded_appoint.is_some(),
                        disposition: WorldCountryDirectAppointmentDisposition::ParameterUnavailable(block),
                    });
                }
            };
            let set = match country.set_king(player_id, country_parameters, context) {
                Ok(report) => report,
                Err(block) => {
                    return Some(WorldCountryDirectAppointmentSync {
                        source_map_id,
                        source_socket_id,
                        country_id,
                        country_complete: decoded_country.is_some(),
                        player_id,
                        player_complete: decoded_player.is_some(),
                        appoint,
                        appoint_complete: decoded_appoint.is_some(),
                        disposition: WorldCountryDirectAppointmentDisposition::ContextBlocked(block),
                    });
                }
            };
            let register = country.register_initial_king(player_id, country_parameters, context);
            WorldCountryDirectAppointmentDisposition::King {
                initial_control_point,
                set,
                register,
            }
        }
        Some(country) => {
            let depose = country.depose_minister(appoint, 7, country_parameters, context);
            let appoint_report = country.appoint_minister(
                player_id,
                appoint,
                6,
                country_parameters,
                context,
            );
            WorldCountryDirectAppointmentDisposition::Minister {
                depose,
                appoint: appoint_report,
            }
        }
    };
    Some(WorldCountryDirectAppointmentSync {
        source_map_id,
        source_socket_id,
        country_id,
        country_complete: decoded_country.is_some(),
        player_id,
        player_complete: decoded_player.is_some(),
        appoint,
        appoint_complete: decoded_appoint.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_info_message<Context: CountryExileResultContext + ?Sized>(
    message: &mut CMessage,
    country_handler: &CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryInfoSync> {
    if message.message_type() != 0x60306 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country(country_id) {
        None => WorldCountryInfoDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryInfoDisposition::KingRejected
        }
        Some(country) => WorldCountryInfoDisposition::Sent(
            country.get_info(country_parameters, context),
        ),
    };
    Some(WorldCountryInfoSync {
        source_map_id,
        source_socket_id,
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_demise_message<Context: CountryExileResultContext + ?Sized>(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut Context,
) -> Option<WorldCountryDemiseSync> {
    if message.message_type() != 0x60308 {
        return None;
    }
    let source_map_id = message.map_id();
    let source_socket_id = message.socket_id();
    let decoded_target = message.base_mut().get_long();
    let target_player_id = decoded_target.unwrap_or(0);
    let decoded_king = message.base_mut().get_long();
    let king_player_id = decoded_king.unwrap_or(0);
    let decoded_country = message.base_mut().get_char();
    let country_id = decoded_country.unwrap_or(0) as u8;
    let disposition = match country_handler.get_country_mut(country_id) {
        None => WorldCountryDemiseDisposition::CountryMissing,
        Some(country) if !country.authorize_king(king_player_id, context) => {
            WorldCountryDemiseDisposition::KingRejected
        }
        Some(country) => {
            let operation = country.can_demise(country_parameters, context);
            if !matches!(operation, CountryCanDemiseDisposition::Allowed) {
                WorldCountryDemiseDisposition::OperationRejected(operation)
            } else {
                let report = country.demise(target_player_id, country_parameters, context);
                WorldCountryDemiseDisposition::Applied { operation, report }
            }
        }
    };
    Some(WorldCountryDemiseSync {
        source_map_id,
        source_socket_id,
        target_player_id,
        target_complete: decoded_target.is_some(),
        king_player_id,
        king_complete: decoded_king.is_some(),
        country_id,
        country_complete: decoded_country.is_some(),
        disposition,
    })
}

pub(crate) fn dispatch_country_war_victory_message<Context: CountryWarVictoryContext + ?Sized>(
    message: &mut CMessage,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Result<Option<WorldCountryWarVictorySync>, CountryWarVictoryDispatchError<Context::Block>> {
    if message.message_type() != 0x60318 {
        return Ok(None);
    }

    let decoded_country = message.base_mut().get_byte();
    let country = decoded_country.unwrap_or(0);
    let report = country_war_sys
        .on_flag_destory(i32::from(country), context)
        .map_err(|source| CountryWarVictoryDispatchError { source })?;
    Ok(Some(WorldCountryWarVictorySync {
        country,
        country_complete: decoded_country.is_some(),
        report,
    }))
}

pub(crate) fn dispatch_country_war_declaration_message<
    Context: CountryWarDeclarationContext + ?Sized,
>(
    message: &mut CMessage,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Option<WorldCountryWarDeclarationSync> {
    if message.message_type() != 0x60317 {
        return None;
    }

    let source_map_id = message.map_id();
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_target_country = message.base_mut().get_long();
    let target_country = decoded_target_country.unwrap_or(0);
    let declaration = country_war_sys.player_declare(player_id, target_country, context);

    let mut response = CMessage::new(0x7ff16);
    response
        .base_mut()
        .add_char(if declaration.accepted() { 1 } else { 0 });
    response.base_mut().add_long(player_id);
    response.base_mut().add_long(target_country);
    let response_wire = response.as_wire_bytes().to_vec();
    let response_delivery = context.send_to_map_id(&response, source_map_id);

    Some(WorldCountryWarDeclarationSync {
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        target_country,
        target_country_complete: decoded_target_country.is_some(),
        source_map_id,
        declaration,
        response_wire,
        response_delivery,
    })
}

pub(crate) fn dispatch_four_nation_war_result_message<
    Context: FourNationWarResultContext + ?Sized,
>(
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

pub(crate) fn dispatch_four_nation_war_time_message<
    Context: FourNationWarResultContext + ?Sized,
>(
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

pub(crate) fn dispatch_four_nation_country_fail_message<
    Context: FourNationCountryFailContext + ?Sized,
>(
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

pub(crate) fn decode_four_nation_exploit_message(
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
