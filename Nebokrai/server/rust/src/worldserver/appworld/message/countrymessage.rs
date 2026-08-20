//! WorldServer dispatcher-owner country messages `OnCountryMessage`.
//!
//! Весь dispatcher RVA `0x000A47F0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме входа
//! country victory `0x60318` со статусом `IMPLEMENTED`. Он читает один
//! unsigned country byte и вызывает исходно названный
//! `CountryWarSys::on_flag_destory`; соседние opcodes helper не
//! интерпретирует. Точная пара
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходник
//! `appworld/message/countrymessage.cpp`.

use super::super::country::countrywarsys::{
    CountryWarSys, CountryWarVictoryContext, CountryWarVictoryReport,
};

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
