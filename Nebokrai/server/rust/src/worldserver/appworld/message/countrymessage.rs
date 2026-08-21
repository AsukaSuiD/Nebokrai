//! WorldServer dispatcher-owner country messages `OnCountryMessage`.
//!
//! Dispatcher RVA `0x000A47F0` остаётся `IMPLEMENTED_PARTIAL`: country relays
//! `0x60310 -> 0x7FF11` и `0x60311 -> 0x7FF12`, а также вход country victory
//! `0x60318` имеют статус `IMPLEMENTED`. Victory читает один
//! unsigned country byte и вызывает исходно названный
//! `CountryWarSys::on_flag_destory`; соседние opcodes helper не
//! интерпретирует. Точная пара
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходник
//! `appworld/message/countrymessage.cpp`. Exact `0x004A4FA3..0x004A4FC4`
//! подтверждает, что оба relay меняют type исходного сообщения и вызывают
//! общий `SendAll`, не читая payload, не вызывая `Update` и не добавляя
//! ownership/tail gates.

use crate::nets::networld::message::{CMessage, SendMessageError};
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

pub(crate) enum WorldCountryMessageDispatch {
    Handled(WorldCountryRelayOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутые in-place relay ветви `OnCountryMessage`.
pub(crate) fn on_country_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldCountryMessageDispatch {
    let request_type = message.message_type();
    let response_type = match request_type {
        COUNTRY_RELAY_FIRST => 0x0007_FF11,
        COUNTRY_RELAY_SECOND => 0x0007_FF12,
        _ => return WorldCountryMessageDispatch::Pending(message),
    };
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldCountryMessageDispatch::Handled(WorldCountryRelayOutcome {
        request_type,
        response_type,
        wire,
        delivery,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarVictoryDispatchError<ContextBlock> {
    UnexpectedEnd { offset: usize },
    Context(ContextBlock),
}

pub(crate) fn dispatch_country_war_victory_message<Context: CountryWarVictoryContext + ?Sized>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Result<Option<CountryWarVictoryReport>, CountryWarVictoryDispatchError<Context::Block>> {
    if opcode != 0x60318 {
        return Ok(None);
    }

    let offset = *cursor;
    let Some(&country) = payload.get(offset) else {
        return Err(CountryWarVictoryDispatchError::UnexpectedEnd { offset });
    };
    *cursor += 1;
    country_war_sys
        .on_flag_destory(i32::from(country), context)
        .map(Some)
        .map_err(CountryWarVictoryDispatchError::Context)
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
