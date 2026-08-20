//! Владелец GameServer dispatcher-а country messages `OnCountryMessage`.
//!
//! Весь dispatcher RVA `0x000997C0` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме связной
//! country-war цепочек opcodes `0x7FF17..0x7FF1F` и `0x7FF22` со статусом
//! `IMPLEMENTED`.
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`, исходник
//! `message/countrymessage.cpp`.
//!
//! Declare/prepare/start/timeout/end/clear cases вызывают `CountryWarSys` без
//! чтения payload. Start затем меняет текущий message ID на `0xC0312`, собирает
//! страны `1..4` в ascending order и шлёт всем при четырёх участниках либо
//! player-ам двух участвующих стран при count `2`; count `0/1/3` не отправляет.
//! `0x7FF1F` читает ровно три signed long и игнорирует legacy bool writer-а;
//! `0x7FF22` читает unsigned country byte и запускает flag-destroy victory
//! chain. Независимые goods-war `0x7FF20/21` и другие opcodes этот helper не
//! интерпретирует. Message/player traversal,
//! singleton storage и concrete region/country registries остаются context-
//! границами своих owners.

use super::super::country::countrywarsys::{
    CountryWarPhaseContext, CountryWarRegionContext, CountryWarSys, CountryWarVictoryContext,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarMessageDispatchError<SideError> {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    Side(SideError),
}

pub(crate) trait CountryWarMessageContext {
    type Region: Copy;
    type SideError;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region>;
    fn set_country_sides(&mut self, region: Self::Region, defend_country: i32, attack_country: i32);
    fn update_contend_player(&mut self, region: Self::Region);

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32);
    fn on_declare_end(&mut self, region: Self::Region, region_id: i32);
    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32);
    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32);
    fn on_war_start(&mut self, region: Self::Region, region_id: i32);
    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32);
    fn on_war_end(&mut self, region: Self::Region, region_id: i32);
    fn clear_country_region(&mut self, region: Self::Region);
    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError>;
    fn reset_country_war_result(&mut self, country: u8);
    fn on_country_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32);
    fn set_country_war_result(&mut self, country: u8, result: i32);

    /// Меняет ID того же входного `CMessage` перед исходной отправкой.
    fn set_country_message_id(&mut self, message_id: u32);
    fn send_current_country_message_all(&mut self);
    /// Обходит `CGame::s_mapPlayer` в map-order и шлёт только двум странам.
    fn send_current_country_message_to_countries(&mut self, countries: [u8; 2]);
}

pub(crate) fn dispatch_country_war_message<Context: CountryWarMessageContext>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Result<bool, CountryWarMessageDispatchError<Context::SideError>> {
    match opcode {
        0x7ff17 => country_war_sys.on_declare_begin(&mut CountryPhaseAdapter(context)),
        0x7ff18 => country_war_sys.on_declare_end(&mut CountryPhaseAdapter(context)),
        0x7ff19 => country_war_sys.on_prepare_begin(&mut CountryPhaseAdapter(context)),
        0x7ff1a => country_war_sys.on_prepare_end(&mut CountryPhaseAdapter(context)),
        0x7ff1b => {
            country_war_sys.on_war_start(&mut CountryPhaseAdapter(context));
            context.set_country_message_id(0xc0312);
            let countries: Vec<u8> = (1..5)
                .filter(|&country| country_war_sys.is_already_declar(i32::from(country)))
                .collect();
            match countries.as_slice() {
                [_, _, _, _] => context.send_current_country_message_all(),
                &[first, second] => {
                    context.send_current_country_message_to_countries([first, second]);
                }
                _ => {}
            }
        }
        0x7ff1c => country_war_sys
            .on_war_timeout(&mut CountryPhaseAdapter(context))
            .map_err(CountryWarMessageDispatchError::Side)?,
        0x7ff1d => country_war_sys.on_war_end(&mut CountryPhaseAdapter(context)),
        0x7ff1e => country_war_sys.on_war_clear(&mut CountryPhaseAdapter(context)),
        0x7ff1f => {
            let region_id = read_country_war_long(payload, cursor)?;
            let defend_country = read_country_war_long(payload, cursor)?;
            let attack_country = read_country_war_long(payload, cursor)?;
            let _ = country_war_sys.update_apply_war(
                region_id,
                defend_country,
                attack_country,
                &mut CountryRegionAdapter(context),
            );
        }
        0x7ff22 => {
            let country = read_country_war_byte(payload, cursor)?;
            country_war_sys
                .on_flag_destroy(i32::from(country), &mut CountryVictoryAdapter(context));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn read_country_war_byte<SideError>(
    payload: &[u8],
    cursor: &mut usize,
) -> Result<u8, CountryWarMessageDispatchError<SideError>> {
    let offset = *cursor;
    let Some(&value) = payload.get(offset) else {
        return Err(CountryWarMessageDispatchError::UnexpectedEnd {
            offset,
            needed: 1,
            available: 0,
        });
    };
    *cursor += 1;
    Ok(value)
}

fn read_country_war_long<SideError>(
    payload: &[u8],
    cursor: &mut usize,
) -> Result<i32, CountryWarMessageDispatchError<SideError>> {
    let offset = *cursor;
    let available = payload.len().saturating_sub(offset);
    let Some(bytes) = payload.get(offset..offset.saturating_add(4)) else {
        return Err(CountryWarMessageDispatchError::UnexpectedEnd {
            offset,
            needed: 4,
            available,
        });
    };
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("country-war long содержит 4 байта"),
    ))
}

struct CountryRegionAdapter<'a, Context>(&'a mut Context);

impl<Context: CountryWarMessageContext> CountryWarRegionContext
    for CountryRegionAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }

    fn set_country_sides(
        &mut self,
        region: Self::Region,
        defend_country: i32,
        attack_country: i32,
    ) {
        self.0
            .set_country_sides(region, defend_country, attack_country);
    }

    fn update_contend_player(&mut self, region: Self::Region) {
        self.0.update_contend_player(region);
    }
}

struct CountryPhaseAdapter<'a, Context>(&'a mut Context);

impl<Context: CountryWarMessageContext> CountryWarPhaseContext
    for CountryPhaseAdapter<'_, Context>
{
    type Region = Context::Region;
    type SideError = Context::SideError;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_declare_begin(region, region_id);
    }

    fn on_declare_end(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_declare_end(region, region_id);
    }

    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_prepare_begin(region, region_id);
    }

    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_prepare_end(region, region_id);
    }

    fn on_war_start(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_war_start(region, region_id);
    }

    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_war_timeout(region, region_id);
    }

    fn on_war_end(&mut self, region: Self::Region, region_id: i32) {
        self.0.on_war_end(region, region_id);
    }

    fn clear_country_region(&mut self, region: Self::Region) {
        self.0.clear_country_region(region);
    }

    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError> {
        self.0.country_region_side_bytes(region)
    }

    fn reset_country_war_result(&mut self, country: u8) {
        self.0.reset_country_war_result(country);
    }
}

struct CountryVictoryAdapter<'a, Context>(&'a mut Context);

impl<Context: CountryWarMessageContext> CountryWarVictoryContext
    for CountryVictoryAdapter<'_, Context>
{
    type Region = Context::Region;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        self.0.find_country_region(region_id)
    }

    fn on_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32) {
        self.0.on_country_flag_destroy(region, region_id, country);
    }

    fn set_country_war_result(&mut self, country: u8, result: i32) {
        self.0.set_country_war_result(country, result);
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\countrymessage.cpp

// ============================================================================
// FUNCTION: OnCountryMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\countrymessage.cpp:22
// RVA: 0x000997C0
// ADDRESS: 004997c0
// PROTOTYPE: void __cdecl OnCountryMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//







// COMPONENT_VARIANT_END: GameServer
