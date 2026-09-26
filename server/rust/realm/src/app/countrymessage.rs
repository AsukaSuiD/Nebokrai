//! Country-сообщения `OnCountryMessage` из `countrymessage.cpp`, подтверждённые
//! `Nworldserver.exe` и `WorldServer.pdb`.
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
//! Governance-ветви `0x60301`, `0x60304`, `0x60306`, `0x60307`, `0x60308`,
//! `0x60309`, `0x6030A`, `0x6030B`, `0x6030C`, `0x6030D`, `0x6030E` и
//! `0x60313` держат исходные цепочки `authorize_king → can_* → действие`:
//! доступ к живому `CCountryHandler` даёт узкий шов
//! [`WorldCountryGovernanceGate`] с адаптером в старом пакете, смена страны
//! `0x60301` — [`WorldCountryPlayerChangeView`] поверх [`WorldGameView`] и
//! [`WorldCountryView`].
//!
//! War-ветви `0x60317` и `0x60318`
//! ([`dispatch_country_war_declaration_message`],
//! [`dispatch_country_war_victory_message`]) живут здесь вместе с типами
//! [`WorldCountryWarDeclarationSync`], [`WorldCountryWarVictorySync`] и
//! агрегатом [`WorldCountryMessageOutcome`]/[`WorldCountryMessageDispatch`];
//! владелец [`CountryWarSys`](crate::activities::countrywarsys::CountryWarSys)
//! — Realm. Адаптеры швов `CCountryHandler`/`CGame` остаются у старого
//! пакета.

use nebokrai_shared::resources::GlobeSetupSnapshot;
use nebokrai_shared::runtime::put_string_to_file;

use crate::activities::countrywarsys::{
    CountryWarDeclarationContext, CountryWarDeclarationReport, CountryWarSys,
    CountryWarVictoryContext, CountryWarVictoryReport,
};
use crate::activities::fournationwarsys::{
    CFourNationWarSys, FourNationCountryFailContext, FourNationCountryFailReport,
    FourNationExploitContext, FourNationExploitLoadedDisposition, FourNationExploitLoadedReport,
    FourNationSignUpDisposition, FourNationWarResultContext, FourNationWarResultReport,
    FourNationWarTimeReport,
};
use crate::app::world_game_view::{
    WorldCountryMutGate, WorldCountryView, WorldExploitDbView, WorldGameView,
};
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
    CountryExileResultContext, CountryExileTimeLookup, CountryGovernanceContextBlock,
    CountryInitialKingReport, CountryPlayersListContext, CountryPlayersListContextBlock,
    CountryPlayersListReport, CountryQuestSwitchUpdate, CountryScalarUpdate, CountrySetKingReport,
    CountrySetNewDayContext, CountrySilenceReport, CountrySuccessExiledReport, KingPointUpdate,
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

/// Исход ветви `0x60317` объявления войны: декодированная пара
/// `[player, target country]`, отчёт владельца `CountryWarSys::player_declare`
/// и исходная отправка ответа `0x7FF16` на source map.
#[derive(Debug, Eq, PartialEq)]
pub struct WorldCountryWarDeclarationSync {
    pub player_id: i32,
    pub player_id_complete: bool,
    pub target_country: i32,
    pub target_country_complete: bool,
    pub source_map_id: i32,
    pub declaration: CountryWarDeclarationReport,
    pub response_wire: Vec<u8>,
    pub response_delivery: Result<i32, SendMessageError>,
}

/// Исход ветви `0x60318` победы в войне: декодированная страна-флагоносец и
/// отчёт владельца `CountryWarSys::on_flag_destory`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldCountryWarVictorySync {
    pub country: u8,
    pub country_complete: bool,
    pub report: CountryWarVictoryReport,
}

/// Ошибка диспетчера ветви `0x60318`: прозрачная обёртка над `Block`
/// victory-контекста вызывающей стороны, сохраняющая исходную error-форму
/// `Result<Option<..>, DispatchError>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryWarVictoryDispatchError<ContextBlock> {
    pub source: ContextBlock,
}

/// Исходы диспетчера [`on_country_message`] и специализированных
/// обработчиков country-ветвей: no-op по неизвестному opcode, подтверждённо
/// игнорируемый `0x6030F`, relay `0x60310`/`0x60311`, governance `0x60301`,
/// `0x60304`, `0x60306..0x6030E` и `0x60313`, scalar/quest-switch/exile-time
/// sync `0x60314..0x60316`, war `0x60317`/`0x60318` и four-nation
/// `0x60319`..`0x6031D`. Переходный хвостовой под-тип слит сюда при
/// переносе war-ветвей: варианты хвоста сохраняют прежние имена.
#[derive(Debug, Eq, PartialEq)]
pub enum WorldCountryMessageOutcome {
    NoOp {
        request_type: i32,
    },
    IgnoredGovernanceRequest {
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

/// Результат хвостового каскада: обработанный исход или сообщение,
/// возвращённое внешнему диспетчеру для следующих владельцев opcode.
pub enum WorldCountryMessageDispatch {
    Handled(WorldCountryMessageOutcome),
    Pending(CMessage),
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
) -> WorldCountryMessageOutcome {
    let request_type = message.message_type();
    if request_type == 0x0006_030f {
        return WorldCountryMessageOutcome::IgnoredGovernanceRequest { request_type };
    }
    if request_type == 0x0006_031b {
        let source_map_id = message.map_id();
        let source_socket_id = message.socket_id();
        let decoded_country = message.base_mut().get_long();
        let country = decoded_country.unwrap_or(0);
        return WorldCountryMessageOutcome::FourNationSignUp(WorldFourNationSignUpSync {
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
        return WorldCountryMessageOutcome::ScalarSynchronized(WorldCountryScalarSync {
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
        return WorldCountryMessageOutcome::QuestSwitchSynchronized(
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
        return WorldCountryMessageOutcome::ExileTimeSynchronized(
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
            return WorldCountryMessageOutcome::NoOp { request_type };
        }
    };
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldCountryMessageOutcome::Relay(WorldCountryRelayOutcome {
        request_type,
        response_type,
        wire,
        delivery,
    })
}

/// Узкий dyn-шов governance-ветвей `OnCountryMessage`
/// (`0x60304`, `0x60306..0x6030E`, `0x60313`): живые объекты `CCountry` и сама
/// таблица `CCountryHandler` остаются в старом пакете, а обработчики в Realm
/// повторяют исходные цепочки `get_country(_mut) → метод CCountry` один вызов
/// на цепочку, по прецеденту [`WorldCountryMutGate`]. Receiver метода шва
/// повторяет receiver метода `CCountry`: `&self`-шаги идут через
/// `get_country`, `&mut self`-шаги через `get_country_mut`. Реализация живёт
/// в адаптере `CCountryHandler` у dispatcher-а старого пакета.
///
/// Каждый метод возвращает несвёрнутый результат соответствующей цепочки:
/// внешний `Option` — страна отсутствует. Исходная форма ветви разрешала
/// страну один раз и удерживала её заимствованием до конца ветви, поэтому
/// первый `None` обработчик отображает на `CountryMissing`, а повторный
/// `None` внутри ветви структурно недостижим (контексты игры доступа к
/// таблице стран не имеют) и свёрнут в тот же `CountryMissing`.
#[allow(clippy::type_complexity, reason = "вложенные формы повторяют исходные цепочки get_country_mut/get_country один к одному")]
pub trait WorldCountryGovernanceGate {
    fn authorize_king(
        &self,
        country: u8,
        candidate: i32,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<bool>;

    fn authorize_minister(
        &self,
        country: u8,
        player_id: i32,
        job: u8,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<bool>;

    fn authorize_king_for_players(
        &self,
        country: u8,
        candidate: i32,
        context: &mut dyn CountryPlayersListContext,
    ) -> Option<bool>;

    fn can_exile(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanExileDisposition>;

    fn exile(
        &self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryExileRequestDisposition>;

    fn can_silence(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanSilenceDisposition>;

    fn silence(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountrySilenceReport>;

    fn can_absolve(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanAbsolveDisposition>;

    fn absolve(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryAbsolveReport>;

    fn can_depose_minister(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanDeposeMinisterDisposition>;

    fn depose_minister(
        &mut self,
        country: u8,
        job: u8,
        mode: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryDeposeMinisterReport>;

    fn can_appoint_minister(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanAppointMinisterDisposition>;

    fn appoint_minister(
        &mut self,
        country: u8,
        player_id: i32,
        job: u8,
        mode: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryAppointMinisterReport>;

    fn can_demise(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanDemiseDisposition>;

    fn demise(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryDemiseReport>;

    fn get_info(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryBaseInfoDisposition>;

    fn get_players_list(
        &self,
        country: u8,
        page: i32,
        context: &mut dyn CountryPlayersListContext,
    ) -> Option<Result<CountryPlayersListReport, CountryPlayersListContextBlock>>;

    /// Цепочка `get_country_mut → set_control_point(&mut country.king, …)`
    /// free-функции `king.rs` ветви direct-appointment.
    fn set_control_point(
        &mut self,
        country: u8,
        requested: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<KingPointUpdate, CountryParameterUnavailable>>;

    fn set_king(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<Result<CountrySetKingReport, CountryGovernanceContextBlock>>;

    fn register_initial_king(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryInitialKingReport>;

    fn success_exiled(
        &mut self,
        country: u8,
        player_id: i32,
        success: bool,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountrySuccessExiledReport>;

    /// Handler-уровень ветви `0x60313`: цепочка без разрешения страны —
    /// `CCountryHandler::set_new_day` всегда выполняется, поэтому внешнего
    /// `Option` нет.
    fn set_new_day(
        &mut self,
        requested_day: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountrySetNewDayContext,
    ) -> CountryHandlerNewDayReport;
}

/// Узкий dyn-шов ветви `0x60301` `OnCountryMessage`: единственный вызов
/// `CGame::change_online_player_country` с предикатом `country_exists`,
/// который исходный диспетчер строил замыканием
/// `|requested| country_handler.get_country(requested).is_some()`. Реализация
/// живёт на `CGame` старого пакета и делегирует одноимённый inherent-метод;
/// отправка ответа идёт методами [`WorldGameView`] того же владельца.
pub trait WorldCountryPlayerChangeView: WorldGameView {
    fn change_online_player_country(
        &mut self,
        player_id: u32,
        requested_country: u8,
        country_exists: &mut dyn FnMut(u8) -> bool,
    ) -> Option<PlayerCountryChangeReport>;
}

/// Ветвь `0x60301`: смена страны игрока и ответ `0x7FF01` на source map.
/// Предикат `country_exists` сохранён замыканием поверх [`WorldCountryView`]
/// в исходной позиции — внутри `change_online_player_country`.
pub fn dispatch_country_player_change_message(
    message: &mut CMessage,
    game: &mut dyn WorldCountryPlayerChangeView,
    countries: &dyn WorldCountryView,
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
    let mut country_exists = |requested_country: u8| countries.country_exists(requested_country);
    let disposition = match game.change_online_player_country(
        player_id as u32,
        country_id,
        &mut country_exists,
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

/// Ветвь `0x60313`: смена дня всеми странами. Цепочка
/// `set_new_day(10, …)` без разрешения страны повторена буквально.
pub fn dispatch_country_new_day_message(
    message: &CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountrySetNewDayContext,
) -> Option<WorldCountryNewDaySync> {
    if message.message_type() != 0x60313 {
        return None;
    }
    Some(WorldCountryNewDaySync {
        source_map_id: message.map_id(),
        source_socket_id: message.socket_id(),
        report: countries.set_new_day(10, country_parameters, context),
    })
}

/// Ветвь `0x60304`: прямое назначение короля (`appoint == 1`) или замена
/// министра. Исходная цепочка: `set_control_point(100_000) → set_king →
/// register_initial_king` для короля и `depose_minister(appoint, 7) →
/// appoint_minister(player, appoint, 6)` для министра; отказы параметров и
/// контекста — на своих позициях.
pub fn dispatch_country_direct_appointment_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        if appoint == 1 {
            let initial_control_point = match countries.set_control_point(
                country_id,
                100_000,
                country_parameters,
            ) {
                None => {
                    break 'fallback WorldCountryDirectAppointmentDisposition::CountryMissing
                }
                Some(Err(block)) => {
                    break 'fallback WorldCountryDirectAppointmentDisposition::ParameterUnavailable(
                        block,
                    )
                }
                Some(Ok(update)) => update,
            };
            let set = match countries.set_king(country_id, player_id, country_parameters, context) {
                None => {
                    break 'fallback WorldCountryDirectAppointmentDisposition::CountryMissing
                }
                Some(Err(block)) => {
                    break 'fallback WorldCountryDirectAppointmentDisposition::ContextBlocked(block)
                }
                Some(Ok(report)) => report,
            };
            let Some(register) = countries.register_initial_king(
                country_id,
                player_id,
                country_parameters,
                context,
            ) else {
                break 'fallback WorldCountryDirectAppointmentDisposition::CountryMissing;
            };
            WorldCountryDirectAppointmentDisposition::King {
                initial_control_point,
                set,
                register,
            }
        } else {
            let Some(depose) = countries.depose_minister(
                country_id,
                appoint,
                7,
                country_parameters,
                context,
            ) else {
                break 'fallback WorldCountryDirectAppointmentDisposition::CountryMissing;
            };
            let Some(appoint_report) = countries.appoint_minister(
                country_id,
                player_id,
                appoint,
                6,
                country_parameters,
                context,
            ) else {
                break 'fallback WorldCountryDirectAppointmentDisposition::CountryMissing;
            };
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

/// Ветвь `0x60306`: base-info страны королю после `authorize_king`.
pub fn dispatch_country_info_message(
    message: &mut CMessage,
    countries: &dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = match countries.authorize_king(country_id, king_player_id, context) {
        None => WorldCountryInfoDisposition::CountryMissing,
        Some(false) => WorldCountryInfoDisposition::KingRejected,
        Some(true) => match countries.get_info(country_id, country_parameters, context) {
            Some(report) => WorldCountryInfoDisposition::Sent(report),
            None => WorldCountryInfoDisposition::CountryMissing,
        },
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

/// Ветвь `0x60307`: страница списка игроков страны после
/// `authorize_king_for_players`.
pub fn dispatch_country_players_list_message(
    message: &mut CMessage,
    countries: &dyn WorldCountryGovernanceGate,
    context: &mut dyn CountryPlayersListContext,
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
    let disposition = match countries.authorize_king_for_players(country_id, king_player_id, context)
    {
        None => WorldCountryPlayersListDisposition::CountryMissing,
        Some(false) => WorldCountryPlayersListDisposition::KingRejected,
        Some(true) => match countries.get_players_list(country_id, page, context) {
            Some(Ok(report)) => WorldCountryPlayersListDisposition::Sent(report),
            Some(Err(block)) => WorldCountryPlayersListDisposition::ContextBlocked(block),
            None => WorldCountryPlayersListDisposition::CountryMissing,
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

/// Ветвь `0x60308`: передача престола. Цепочка
/// `authorize_king → can_demise → demise` с отказами операции на
/// исходных позициях.
pub fn dispatch_country_demise_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        match countries.authorize_king(country_id, king_player_id, context) {
            None => WorldCountryDemiseDisposition::CountryMissing,
            Some(false) => WorldCountryDemiseDisposition::KingRejected,
            Some(true) => {
                let Some(operation) =
                    countries.can_demise(country_id, country_parameters, context)
                else {
                    break 'fallback WorldCountryDemiseDisposition::CountryMissing;
                };
                if !matches!(operation, CountryCanDemiseDisposition::Allowed) {
                    WorldCountryDemiseDisposition::OperationRejected(operation)
                } else {
                    let Some(report) =
                        countries.demise(country_id, target_player_id, country_parameters, context)
                    else {
                        break 'fallback WorldCountryDemiseDisposition::CountryMissing;
                    };
                    WorldCountryDemiseDisposition::Applied { operation, report }
                }
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

/// Ветвь `0x60309`: назначение министра. Цепочка
/// `authorize_king → can_appoint_minister → appoint_minister(target, job, 6)`.
pub fn dispatch_country_appoint_minister_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        match countries.authorize_king(country_id, king_player_id, context) {
            None => WorldCountryAppointMinisterDisposition::CountryMissing,
            Some(false) => WorldCountryAppointMinisterDisposition::KingRejected,
            Some(true) => {
                let Some(operation) =
                    countries.can_appoint_minister(country_id, country_parameters, context)
                else {
                    break 'fallback WorldCountryAppointMinisterDisposition::CountryMissing;
                };
                if !matches!(operation, CountryCanAppointMinisterDisposition::Allowed) {
                    WorldCountryAppointMinisterDisposition::OperationRejected(operation)
                } else {
                    let Some(report) = countries.appoint_minister(
                        country_id,
                        target_player_id,
                        job,
                        6,
                        country_parameters,
                        context,
                    ) else {
                        break 'fallback WorldCountryAppointMinisterDisposition::CountryMissing;
                    };
                    WorldCountryAppointMinisterDisposition::Applied { operation, report }
                }
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

/// Ветвь `0x6030A`: смещение министра. Цепочка
/// `authorize_king → can_depose_minister → authorize_minister →
/// depose_minister(job, 7)`.
pub fn dispatch_country_depose_minister_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        match countries.authorize_king(country_id, king_player_id, context) {
            None => WorldCountryDeposeMinisterDisposition::CountryMissing,
            Some(false) => WorldCountryDeposeMinisterDisposition::KingRejected,
            Some(true) => {
                let Some(operation) =
                    countries.can_depose_minister(country_id, country_parameters, context)
                else {
                    break 'fallback WorldCountryDeposeMinisterDisposition::CountryMissing;
                };
                if !matches!(operation, CountryCanDeposeMinisterDisposition::Allowed) {
                    WorldCountryDeposeMinisterDisposition::OperationRejected(operation)
                } else {
                    let Some(is_minister) = countries.authorize_minister(
                        country_id,
                        target_player_id,
                        job,
                        context,
                    ) else {
                        break 'fallback WorldCountryDeposeMinisterDisposition::CountryMissing;
                    };
                    if !is_minister {
                        WorldCountryDeposeMinisterDisposition::MinisterRejected
                    } else {
                        let Some(report) = countries.depose_minister(
                            country_id,
                            job,
                            7,
                            country_parameters,
                            context,
                        ) else {
                            break 'fallback WorldCountryDeposeMinisterDisposition::CountryMissing;
                        };
                        WorldCountryDeposeMinisterDisposition::Applied { operation, report }
                    }
                }
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

/// Ветвь `0x6030B`: амнистия PK. Цепочка
/// `authorize_king → can_absolve → absolve`.
pub fn dispatch_country_absolve_request_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        match countries.authorize_king(country_id, king_player_id, context) {
            None => WorldCountryAbsolveRequestDisposition::CountryMissing,
            Some(false) => WorldCountryAbsolveRequestDisposition::KingRejected,
            Some(true) => {
                let Some(operation) =
                    countries.can_absolve(country_id, country_parameters, context)
                else {
                    break 'fallback WorldCountryAbsolveRequestDisposition::CountryMissing;
                };
                if !matches!(operation, CountryCanAbsolveDisposition::Allowed) {
                    WorldCountryAbsolveRequestDisposition::OperationRejected(operation)
                } else {
                    let Some(report) = countries.absolve(
                        country_id,
                        target_player_id,
                        country_parameters,
                        context,
                    ) else {
                        break 'fallback WorldCountryAbsolveRequestDisposition::CountryMissing;
                    };
                    WorldCountryAbsolveRequestDisposition::Applied { operation, report }
                }
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

/// Ветвь `0x6030C`: молчание игрока. Цепочка
/// `authorize_king → can_silence → silence`.
pub fn dispatch_country_silence_request_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        match countries.authorize_king(country_id, king_player_id, context) {
            None => WorldCountrySilenceRequestDisposition::CountryMissing,
            Some(false) => WorldCountrySilenceRequestDisposition::KingRejected,
            Some(true) => {
                let Some(operation) =
                    countries.can_silence(country_id, country_parameters, context)
                else {
                    break 'fallback WorldCountrySilenceRequestDisposition::CountryMissing;
                };
                if !matches!(operation, CountryCanSilenceDisposition::Allowed) {
                    WorldCountrySilenceRequestDisposition::OperationRejected(operation)
                } else {
                    let Some(report) = countries.silence(
                        country_id,
                        target_player_id,
                        country_parameters,
                        context,
                    ) else {
                        break 'fallback WorldCountrySilenceRequestDisposition::CountryMissing;
                    };
                    WorldCountrySilenceRequestDisposition::Applied { operation, report }
                }
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

/// Ветвь `0x6030D`: запрос exile. Цепочка
/// `authorize_king → can_exile → exile` с legacy-свёрткой результата в
/// `target_player_id` при отправке запроса и `0` иначе.
pub fn dispatch_country_exile_request_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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
    let disposition = 'fallback: {
        match countries.authorize_king(country_id, king_player_id, context) {
            None => WorldCountryExileRequestDisposition::CountryMissing,
            Some(false) => WorldCountryExileRequestDisposition::KingRejected,
            Some(true) => {
                let Some(operation) = countries.can_exile(country_id, country_parameters, context)
                else {
                    break 'fallback WorldCountryExileRequestDisposition::CountryMissing;
                };
                if !matches!(operation, CountryCanExileDisposition::Allowed) {
                    WorldCountryExileRequestDisposition::OperationRejected(operation)
                } else {
                    let Some(request) = countries.exile(
                        country_id,
                        target_player_id,
                        country_parameters,
                        context,
                    ) else {
                        break 'fallback WorldCountryExileRequestDisposition::CountryMissing;
                    };
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

/// Ветвь `0x6030E`: результат exile от GameServer: `success_exiled` до
/// чтения списка игроков, safe-decode списка (проверка усечения по остатку
/// байт и guard аллокации до цикла) и ответ `0x7FF15` формы
/// `[country byte, count long, ids…]` через `send_all` контекста. Порядок
/// повторяет исходный диспетчер; guard-ы списка — добавленная Rust-закалка:
/// их у машины нет (цикл `count×GetDWord` без проверок, при усечении
/// GetDWord отдаёт 0 и ответ всё равно уходит) — дивергенция только на
/// порченом wire (машинная досверка диспетчера).
pub fn dispatch_country_exile_result_message(
    message: &mut CMessage,
    countries: &mut dyn WorldCountryGovernanceGate,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
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

    let Some(country_report) = countries.success_exiled(
        country_id,
        player_id,
        raw_success != 0,
        country_parameters,
        context,
    ) else {
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

/// Ветвь `0x60317`: объявление войны государству. Decode пары long, вызов
/// владельца `CountryWarSys::player_declare` с его authority-гейтами
/// (`online_player_country → declaration_authority` внутри владельца) и
/// исходная отправка ответа `0x7FF16` `[accepted byte, player, target]` на
/// source map после отчёта владельца — в этой же позиции каскада.
pub fn dispatch_country_war_declaration_message<
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

/// Ветвь `0x60318`: победа в войне по уничтоженному флагу. Decode одного
/// байта страны и вызов владельца `CountryWarSys::on_flag_destory`, который
/// для каждой активной пары проставляет `set_country_war_result` победителю
/// 2 и проигравшему 1 через victory-gate вызывающей стороны. Error-форма
/// `CountryWarVictoryDispatchError` сохраняет исходную обёртку над `Block`
/// контекста; у машинного владельца ошибки dispatch-уровня нет.
pub fn dispatch_country_war_victory_message<Context: CountryWarVictoryContext + ?Sized>(
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
