//! Country-сообщения `OnCountryMessage` из `countrymessage.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Ветки `0x60301`, `0x60304`, `0x60306..0x6031D` сохраняют управление
//! королём/министрами, scalar sync, player lists и country-war events.
//! Неизвестный opcode — no-op; relays меняют только type и вызывают `SendAll`.
//! Отдельно подтверждённый `0x6030F` также ведёт прямо в общий выход dispatcher-а:
//! payload `[target, caller, country]` не читается, состояние и сеть не меняются.
//!
//! Governance идёт через `GetCountry -> permission -> operation`; byte job не
//! проверяется заранее. Scalar limits несимметричны: treasury/power зажимаются
//! с двух сторон, tech exp только сверху, tech level только снизу, king points
//! только сверху.
//!
//! Exile time вычисляется wrapping-миллисекундами и ограничивается нулём.
//! Точный `0x60316` сохраняет ошибочный original writer: `0x7FF15` с двумя
//! long вместо Game exile-list layout; Game safe boundary распознаёт этот
//! уникальный 8-байтовый retired payload без undefined overread.
//! Player lists сохраняют page arithmetic и GM filter. War branches не вводят
//! source/tail checks. Явный main-loop context и safe codec заменяют singleton
//! и overread без изменения вызовов.
//!
//! Наблюдаемые data-контракты ветвей, обработчики независимых ветвей
//! (хвостовой `on_country_message`: `0x6030F`, `0x6031B`, `0x60314..0x60316`,
//! relay `0x60310`/`0x60311`, no-op; four-nation dispatch
//! `0x60319`/`0x6031C`/`0x6031D`; decode `0x6031A`) и governance-ветви
//! `0x60301`, `0x60304`, `0x60306..0x6030E`, `0x60313` перенесены в
//! `nebokrai_realm::app::countrymessage`. Независимые и хвостовые ветви здесь
//! реэкспортированы, governance-ветви делегированы обёртками с теми же
//! именами и с отображением исходов один к одному. Хвостовой вызов идёт через
//! адаптер [`WorldCountryMutGate`] поверх живого `CCountryHandler`,
//! governance — через адаптер [`WorldCountryGovernanceGate`] поверх того же
//! живого `CCountryHandler`, ветвь `0x60301` — через
//! [`WorldCountryPlayerChangeView`] на `CGame` и [`WorldCountryView`] поверх
//! `CCountryHandler`; exploit `0x6031A` целиком обслуживается realm
//! async-обработчиком от точки вызова в `game.rs`. У этого владельца временно
//! остаются `WorldCountryWarDeclarationSync`, `WorldCountryWarVictorySync` и
//! связка `WorldCountryMessageOutcome`/`WorldCountryMessageDispatch`: они
//! цитируют `CountryWarDeclarationReport`/`CountryWarVictoryReport` из
//! `countrywarsys.rs` до переноса war-систем.

use nebokrai_realm::app::world_game_view::{WorldCountryMutGate, WorldCountryView};
use nebokrai_realm::content::countryparam::CountryParameterUnavailable;
use nebokrai_realm::organizations::country::{
    CountryExileTimeLookup, CountryQuestSwitchUpdate, CountryScalarUpdate, KingPointUpdate,
};

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::worldserver::appworld::country::country::{
    CountryAbsolveReport, CountryAppointMinisterReport, CountryBaseInfoDisposition,
    CountryCanAbsolveDisposition, CountryCanAppointMinisterDisposition, CountryCanDemiseDisposition,
    CountryCanDeposeMinisterDisposition, CountryCanExileDisposition, CountryCanSilenceDisposition,
    CountryDemiseReport, CountryDeposeMinisterReport, CountryExileRequestDisposition,
    CountryExileResultContext, CountryGovernanceContextBlock, CountryInitialKingReport,
    CountryPlayersListContext, CountryPlayersListContextBlock, CountryPlayersListReport,
    CountrySetKingReport, CountrySetNewDayContext, CountrySilenceReport, CountrySuccessExiledReport,
};
use crate::worldserver::appworld::country::countryhandler::{
    CCountryHandler, CountryHandlerNewDayReport,
};
use crate::worldserver::appworld::country::countryparam::CCountryParam;
use crate::worldserver::appworld::country::king::set_control_point;
use crate::worldserver::appworld::player::PlayerCountryChangeReport;
use crate::worldserver::worldserver::game::{CGame, legacy_tick_ms};

use super::super::country::countrywarsys::{
    CountryWarDeclarationContext, CountryWarDeclarationReport, CountryWarSys,
    CountryWarVictoryContext, CountryWarVictoryReport,
};

pub(crate) use nebokrai_realm::app::countrymessage::*;

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
pub(crate) enum WorldCountryMessageOutcome {
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

pub(crate) enum WorldCountryMessageDispatch {
    Handled(WorldCountryMessageOutcome),
    Pending(CMessage),
}

/// Адаптер [`WorldCountryMutGate`] поверх живого `CCountryHandler`: ровно те
/// же `get_country_mut`/`get_country` цепочки, что выполнял прежний
/// диспетчер. Тик `legacy_tick_ms` для exile-lookup снимается в исходной
/// позиции — внутри ветки найденной страны, до вызова
/// `CCountry::exile_remaining_time`; отсутствующая страна тик не тратит.
struct CountryHandlerMutGate<'a> {
    handler: &'a mut CCountryHandler,
}

impl WorldCountryMutGate for CountryHandlerMutGate<'_> {
    fn apply_server_scalar(
        &mut self,
        country: u8,
        selector: i8,
        value: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<Option<CountryScalarUpdate>, CountryParameterUnavailable>> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.apply_server_scalar(selector, value, country_parameters))
    }

    fn set_quest_switch(
        &mut self,
        country: u8,
        job: u8,
        enabled: bool,
    ) -> Option<Option<CountryQuestSwitchUpdate>> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.set_quest_switch(job, enabled))
    }

    fn exile_remaining_time(
        &self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<CountryExileTimeLookup, CountryParameterUnavailable>> {
        self.handler.get_country(country).map(|country| {
            country.exile_remaining_time(player_id, legacy_tick_ms(), country_parameters)
        })
    }
}

/// Хвостовые ветви `OnCountryMessage` (`0x6030F`, `0x6031B`,
/// `0x60314..0x60316`, relay `0x60310`/`0x60311`, no-op) перенесены в realm
/// `nebokrai_realm::app::countrymessage::on_country_message`; здесь только
/// отображение tail-исходов на общий `WorldCountryMessageOutcome`, который
/// пока цитирует war-типы старого `countrywarsys.rs`.
pub(crate) fn on_country_message(
    game: &CGame,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    message: CMessage,
) -> WorldCountryMessageDispatch {
    let mut gate = CountryHandlerMutGate {
        handler: country_handler,
    };
    WorldCountryMessageDispatch::Handled(
        match nebokrai_realm::app::countrymessage::on_country_message(
            game,
            &mut gate,
            country_parameters,
            globe_setup,
            message,
        ) {
            WorldCountryMessageTailOutcome::NoOp { request_type } => {
                WorldCountryMessageOutcome::NoOp { request_type }
            }
            WorldCountryMessageTailOutcome::IgnoredGovernanceRequest { request_type } => {
                WorldCountryMessageOutcome::IgnoredGovernanceRequest { request_type }
            }
            WorldCountryMessageTailOutcome::Relay(outcome) => {
                WorldCountryMessageOutcome::Relay(outcome)
            }
            WorldCountryMessageTailOutcome::ScalarSynchronized(sync) => {
                WorldCountryMessageOutcome::ScalarSynchronized(sync)
            }
            WorldCountryMessageTailOutcome::QuestSwitchSynchronized(sync) => {
                WorldCountryMessageOutcome::QuestSwitchSynchronized(sync)
            }
            WorldCountryMessageTailOutcome::ExileTimeSynchronized(sync) => {
                WorldCountryMessageOutcome::ExileTimeSynchronized(sync)
            }
            WorldCountryMessageTailOutcome::FourNationSignUp(sync) => {
                WorldCountryMessageOutcome::FourNationSignUp(sync)
            }
        },
    )
}

/// Адаптер [`WorldCountryGovernanceGate`] поверх живого `CCountryHandler`:
/// ровно те цепочки `get_country`/`get_country_mut → метод CCountry`, что
/// выполнял прежний диспетчер governance-ветвей. Receiver gate-метода
/// повторяет receiver метода `CCountry`: `&self`-шаги идут через
/// `get_country`, `&mut self`-шаги через `get_country_mut`; единая
/// `get_country_mut`-разрешение ветви машинного оригинала поведения не
/// меняет — за время ветви таблица недоступна никому (заимствование).
struct CountryHandlerGovernanceGate<'a> {
    handler: &'a mut CCountryHandler,
}

#[allow(clippy::type_complexity, reason = "вложенные формы повторяют исходные цепочки get_country_mut/get_country один к одному")]
impl WorldCountryGovernanceGate for CountryHandlerGovernanceGate<'_> {
    fn authorize_king(
        &self,
        country: u8,
        candidate: i32,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<bool> {
        self.handler
            .get_country(country)
            .map(|country| country.authorize_king(candidate, context))
    }

    fn authorize_minister(
        &self,
        country: u8,
        player_id: i32,
        job: u8,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<bool> {
        self.handler
            .get_country(country)
            .map(|country| country.authorize_minister(player_id, job, context))
    }

    fn authorize_king_for_players(
        &self,
        country: u8,
        candidate: i32,
        context: &mut dyn CountryPlayersListContext,
    ) -> Option<bool> {
        self.handler
            .get_country(country)
            .map(|country| country.authorize_king_for_players(candidate, context))
    }

    fn can_exile(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanExileDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_exile(country_parameters, context))
    }

    fn exile(
        &self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryExileRequestDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.exile(player_id, country_parameters, context))
    }

    fn can_silence(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanSilenceDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_silence(country_parameters, context))
    }

    fn silence(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountrySilenceReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.silence(player_id, country_parameters, context))
    }

    fn can_absolve(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanAbsolveDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_absolve(country_parameters, context))
    }

    fn absolve(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryAbsolveReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.absolve(player_id, country_parameters, context))
    }

    fn can_depose_minister(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanDeposeMinisterDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_depose_minister(country_parameters, context))
    }

    fn depose_minister(
        &mut self,
        country: u8,
        job: u8,
        mode: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryDeposeMinisterReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.depose_minister(job, mode, country_parameters, context))
    }

    fn can_appoint_minister(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanAppointMinisterDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_appoint_minister(country_parameters, context))
    }

    fn appoint_minister(
        &mut self,
        country: u8,
        player_id: i32,
        job: u8,
        mode: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryAppointMinisterReport> {
        self.handler.get_country_mut(country).map(|country| {
            country.appoint_minister(player_id, job, mode, country_parameters, context)
        })
    }

    fn can_demise(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryCanDemiseDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.can_demise(country_parameters, context))
    }

    fn demise(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryDemiseReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.demise(player_id, country_parameters, context))
    }

    fn get_info(
        &self,
        country: u8,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryBaseInfoDisposition> {
        self.handler
            .get_country(country)
            .map(|country| country.get_info(country_parameters, context))
    }

    fn get_players_list(
        &self,
        country: u8,
        page: i32,
        context: &mut dyn CountryPlayersListContext,
    ) -> Option<Result<CountryPlayersListReport, CountryPlayersListContextBlock>> {
        self.handler
            .get_country(country)
            .map(|country| country.get_players_list(page, context))
    }

    fn set_control_point(
        &mut self,
        country: u8,
        requested: i32,
        country_parameters: &CCountryParam,
    ) -> Option<Result<KingPointUpdate, CountryParameterUnavailable>> {
        self.handler
            .get_country_mut(country)
            .map(|country| set_control_point(&mut country.king, requested, country_parameters))
    }

    fn set_king(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<Result<CountrySetKingReport, CountryGovernanceContextBlock>> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.set_king(player_id, country_parameters, context))
    }

    fn register_initial_king(
        &mut self,
        country: u8,
        player_id: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountryInitialKingReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.register_initial_king(player_id, country_parameters, context))
    }

    fn success_exiled(
        &mut self,
        country: u8,
        player_id: i32,
        success: bool,
        country_parameters: &CCountryParam,
        context: &mut dyn CountryExileResultContext,
    ) -> Option<CountrySuccessExiledReport> {
        self.handler
            .get_country_mut(country)
            .map(|country| country.success_exiled(player_id, success, country_parameters, context))
    }

    fn set_new_day(
        &mut self,
        requested_day: i32,
        country_parameters: &CCountryParam,
        context: &mut dyn CountrySetNewDayContext,
    ) -> CountryHandlerNewDayReport {
        self.handler
            .set_new_day(requested_day, country_parameters, context)
    }
}

/// Адаптер [`WorldCountryView`] поверх `CCountryHandler` ветви `0x60301`:
/// ровно `get_country(...).is_some()` прежнего замыкания `country_exists`,
/// по прецеденту `CreateRoleCountryViewAdapter` в `logmessage.rs`.
struct CountryHandlerExistsView<'a> {
    handler: &'a CCountryHandler,
}

impl WorldCountryView for CountryHandlerExistsView<'_> {
    fn country_exists(&self, country: u8) -> bool {
        self.handler.get_country(country).is_some()
    }
}

/// Bridge [`WorldCountryPlayerChangeView`] владельца игры: делегирует
/// одноимённый inherent-метод `CGame`; closure-предикат передаётся без
/// обёрток — `&mut dyn FnMut(u8) -> bool` удовлетворяет исходному
/// `impl FnOnce(u8) -> bool` через std-blanket для `&mut F`.
impl WorldCountryPlayerChangeView for CGame {
    fn change_online_player_country(
        &mut self,
        player_id: u32,
        requested_country: u8,
        country_exists: &mut dyn FnMut(u8) -> bool,
    ) -> Option<PlayerCountryChangeReport> {
        CGame::change_online_player_country(self, player_id, requested_country, country_exists)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldCountryWarVictorySync {
    pub(crate) country: u8,
    pub(crate) country_complete: bool,
    pub(crate) report: CountryWarVictoryReport,
}

/// Ветвь `0x6030E` перенесена в realm; обёртка сохраняет прежнюю сигнатуру и
/// отображение исхода один к одному.
pub(crate) fn dispatch_country_exile_result_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryExileResultSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_exile_result_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030D` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_exile_request_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryExileRequestSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_exile_request_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030C` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_silence_request_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountrySilenceRequestSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_silence_request_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030B` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_absolve_request_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryAbsolveRequestSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_absolve_request_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x6030A` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_depose_minister_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryDeposeMinisterSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_depose_minister_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60309` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_appoint_minister_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryAppointMinisterSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_appoint_minister_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60307` перенесена в realm; обёртка сохраняет прежнюю сигнатуру,
/// только shared handler заменён на mutable для адаптера шва.
pub(crate) fn dispatch_country_players_list_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    context: &mut dyn CountryPlayersListContext,
) -> Option<WorldCountryPlayersListSync> {
    let countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_players_list_message(
        message,
        &countries,
        context,
    )
}

/// Ветвь `0x60301` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_player_change_message(
    message: &mut CMessage,
    game: &mut CGame,
    country_handler: &CCountryHandler,
) -> Option<WorldCountryPlayerChangeSync> {
    let countries = CountryHandlerExistsView {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_player_change_message(
        message,
        game,
        &countries,
    )
}

/// Ветвь `0x60313` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_new_day_message(
    message: &CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountrySetNewDayContext,
) -> Option<WorldCountryNewDaySync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_new_day_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60304` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_direct_appointment_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryDirectAppointmentSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_direct_appointment_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60306` перенесена в realm; обёртка сохраняет прежнюю сигнатуру,
/// только shared handler заменён на mutable для адаптера шва.
pub(crate) fn dispatch_country_info_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryInfoSync> {
    let countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_info_message(
        message,
        &countries,
        country_parameters,
        context,
    )
}

/// Ветвь `0x60308` перенесена в realm; обёртка сохраняет прежнюю сигнатуру.
pub(crate) fn dispatch_country_demise_message(
    message: &mut CMessage,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    context: &mut dyn CountryExileResultContext,
) -> Option<WorldCountryDemiseSync> {
    let mut countries = CountryHandlerGovernanceGate {
        handler: country_handler,
    };
    nebokrai_realm::app::countrymessage::dispatch_country_demise_message(
        message,
        &mut countries,
        country_parameters,
        context,
    )
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

// Four-nation dispatch `0x60319`/`0x6031C`/`0x6031D`, decode `0x6031A`,
// governance-ветви `0x60301`, `0x60304`, `0x60306..0x6030E`, `0x60313` и
// хвостовой `on_country_message` перенесены в realm; имена доступны здесь
// через glob re-export и делегирующие обёртки выше.
