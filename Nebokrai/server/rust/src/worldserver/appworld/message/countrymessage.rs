//! WorldServer dispatcher-owner country messages `OnCountryMessage`.
//!
//! Dispatcher RVA `0x000A47F0` остаётся `IMPLEMENTED_PARTIAL`: country relays
//! `0x60310 -> 0x7FF11` и `0x60311 -> 0x7FF12`, а также вход country victory
//! `0x60318`, scalar-sync `0x60314` и quest-switch `0x60315` имеют статус
//! `IMPLEMENTED`. Victory читает один unsigned country byte и вызывает исходно
//! названный `CountryWarSys::on_flag_destory`; соседние opcodes helper не
//! интерпретирует. Точная пара
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходник
//! `appworld/message/countrymessage.cpp`. Exact `0x004A4FA3..0x004A4FC4`
//! подтверждает, что оба relay меняют type исходного сообщения и вызывают
//! общий `SendAll`, не читая payload, не вызывая `Update` и не добавляя
//! ownership/tail gates. Exact `0x004A48C6..0x004A496E` задаёт для `0x60314`
//! wire `unsigned char country, signed char selector, signed long value` и
//! тихие no-op на отсутствующей стране или неизвестном selector. Scalar-setter
//! сохраняет несимметричные исходные ограничения: treasury/power ограничены
//! снизу нулём и сверху максимумом, tech-exp только сверху, tech-level только
//! снизу, king points только сверху. Вместо singleton `CCountryParam` Rust
//! принимает уже принадлежащий main-loop параметр явно.
//! Exact `0x004A4EF1..0x004A4F9E` и PDB layout `COfficer` подтверждают для
//! `0x60315` три unsigned byte `country/job/raw switch`, выбор встроенного king
//! при job `1`, `GetMinister` только для `2..=7` и запись именно
//! `_bQuestSwitch +0x25`. Строка `king`-лога сохраняет исходный raw switch и
//! byte-exact хвост `A1 A3`; безопасный accessor `CGlobeSetup` заменяет только
//! старое адресное вычисление country-name slot.
//! Exact switch target `0x004A504E..0x004A5065` подтверждает, что `0x60318`
//! читает один unsigned country byte и сразу передаёт его достигнутому
//! `CountryWarSys`; конкретный region/country/localization/network context
//! подключён в общем `ProcessMessage`, а не оставлен отдельным helper-ом.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::public::tools::put_string_to_file;
use crate::setup::globesetup::GlobeSetupSnapshot;
use crate::worldserver::appworld::country::country::{
    CountryQuestSwitchUpdate, CountryScalarUpdate,
};
use crate::worldserver::appworld::country::countryhandler::CCountryHandler;
use crate::worldserver::appworld::country::countryparam::{
    CCountryParam, CountryParameterUnavailable,
};
use crate::worldserver::worldserver::game::CGame;

use super::super::country::countrywarsys::{
    CountryWarSys, CountryWarVictoryContext, CountryWarVictoryReport,
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
pub(crate) enum WorldCountryMessageOutcome {
    Relay(WorldCountryRelayOutcome),
    ScalarSynchronized(WorldCountryScalarSync),
    QuestSwitchSynchronized(WorldCountryQuestSwitchSync),
    CountryWarVictory(WorldCountryWarVictorySync),
}

pub(crate) enum WorldCountryMessageDispatch {
    Handled(WorldCountryMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутые scalar-sync и in-place relay ветви `OnCountryMessage`.
pub(crate) fn on_country_message(
    game: &CGame,
    country_handler: &mut CCountryHandler,
    country_parameters: &CCountryParam,
    globe_setup: &GlobeSetupSnapshot,
    mut message: CMessage,
) -> WorldCountryMessageDispatch {
    let request_type = message.message_type();
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
    let response_type = match request_type {
        COUNTRY_RELAY_FIRST => 0x0007_FF11,
        COUNTRY_RELAY_SECOND => 0x0007_FF12,
        _ => return WorldCountryMessageDispatch::Pending(message),
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\countrymessage.cpp

// ============================================================================
// FUNCTION: OnCountryMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\countrymessage.cpp:21
// RVA: 0x000A47F0
// ADDRESS: 004a47f0
// PROTOTYPE: void __cdecl OnCountryMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: WorldServer
