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
//! chain. Goods-war `0x7FF20/21` маршрутизируются соседнему owned helper-у;
//! другие opcodes этот helper не интерпретирует. Достигнутая family проходит
//! живой FIFO: `CountryWarSys`, country regions/results и canonical player
//! traversal принадлежат `CGame`, а gate/guard/kick и virtual contender
//! effects остаются обязательной runtime-границей concrete owners.
//! Client governance `0x90502..0x90509` проверяет caller/target changing state,
//! сохраняет selector-specific `GS/WS` ошибки через `0xC030D` и пересылает
//! исходный payload с caller ID/country в достигнутые WorldServer
//! `0x60306..0x6030D`. `0x9050A -> 0x6030F` остаётся отдельно названной
//! границей до materialization соответствующего World permission owner-а.

use super::super::country::countrywarsys::{
    CountryWarPhaseContext, CountryWarRegionContext, CountryWarSys, CountryWarVictoryContext,
};
use super::super::servercountryregion::{CountryBattleStateBlock, CountryRegionRuntimeContext};
use crate::gameserver::gameserver::game::{CGame, ServerRegionOwner};
use crate::nets::netserver::message::{CMessage, SendMessageError};

pub(crate) trait GameCountryWarRuntime: CountryRegionRuntimeContext {
    /// Материализует virtual `UpdateContendPlayer` country-region owner-а.
    fn update_country_contend_player(&mut self, region_id: i32);
}

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarBroadcastIntent {
    All,
    Countries([u8; 2]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarMessageDispatchReport {
    pub(crate) opcode: u32,
    pub(crate) update_found: Option<bool>,
    pub(crate) message_type_effect: Option<u32>,
    pub(crate) broadcast: Option<CountryWarBroadcastIntent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarBroadcastOutcome {
    MissingServer,
    All(Result<i32, SendMessageError>),
    Countries {
        countries: [u8; 2],
        recipients: Vec<i32>,
        queue_results: Vec<i32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameCountryWarMessageReport {
    pub(crate) dispatched: Option<CountryWarMessageDispatchReport>,
    pub(crate) governance: Option<CountryGovernanceReport>,
    pub(crate) broadcast: Option<CountryWarBroadcastOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryGovernanceOutcome {
    MissingPlayer,
    Forwarded {
        world_type: u32,
        target_id: Option<i32>,
        delivery: Result<i32, SendMessageError>,
    },
    Rejected {
        string_id: &'static [u8],
        delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryGovernanceReport {
    pub(crate) opcode: u32,
    pub(crate) player_id: Option<i32>,
    pub(crate) country: Option<u8>,
    pub(crate) outcome: CountryGovernanceOutcome,
}

pub(crate) fn dispatch_country_war_message<Context: CountryWarMessageContext>(
    opcode: u32,
    payload: &[u8],
    cursor: &mut usize,
    country_war_sys: &mut CountryWarSys,
    context: &mut Context,
) -> Option<
    Result<CountryWarMessageDispatchReport, CountryWarMessageDispatchError<Context::SideError>>,
> {
    let mut update_found = None;
    let mut message_type_effect = None;
    let mut broadcast = None;
    match opcode {
        0x7ff17 => country_war_sys.on_declare_begin(&mut CountryPhaseAdapter(context)),
        0x7ff18 => country_war_sys.on_declare_end(&mut CountryPhaseAdapter(context)),
        0x7ff19 => country_war_sys.on_prepare_begin(&mut CountryPhaseAdapter(context)),
        0x7ff1a => country_war_sys.on_prepare_end(&mut CountryPhaseAdapter(context)),
        0x7ff1b => {
            country_war_sys.on_war_start(&mut CountryPhaseAdapter(context));
            message_type_effect = Some(0xc0312);
            let countries: Vec<u8> = (1..5)
                .filter(|&country| country_war_sys.is_already_declar(i32::from(country)))
                .collect();
            match countries.as_slice() {
                [_, _, _, _] => broadcast = Some(CountryWarBroadcastIntent::All),
                &[first, second] => {
                    broadcast = Some(CountryWarBroadcastIntent::Countries([first, second]));
                }
                _ => {}
            }
        }
        0x7ff1c => {
            if let Err(error) = country_war_sys.on_war_timeout(&mut CountryPhaseAdapter(context)) {
                return Some(Err(CountryWarMessageDispatchError::Side(error)));
            }
        }
        0x7ff1d => country_war_sys.on_war_end(&mut CountryPhaseAdapter(context)),
        0x7ff1e => country_war_sys.on_war_clear(&mut CountryPhaseAdapter(context)),
        0x7ff1f => {
            let region_id = match read_country_war_long(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let defend_country = match read_country_war_long(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let attack_country = match read_country_war_long(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            update_found = Some(country_war_sys.update_apply_war(
                region_id,
                defend_country,
                attack_country,
                &mut CountryRegionAdapter(context),
            ));
        }
        0x7ff22 => {
            let country = match read_country_war_byte(payload, cursor) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            country_war_sys
                .on_flag_destroy(i32::from(country), &mut CountryVictoryAdapter(context));
        }
        _ => return None,
    }
    Some(Ok(CountryWarMessageDispatchReport {
        opcode,
        update_found,
        message_type_effect,
        broadcast,
    }))
}

/// Подключает достигнутую country-war family к owned `CGame` state и тому же
/// входному `CMessage`, который исходный dispatcher переиспользует для send.
pub(crate) fn dispatch_game_country_war_message<Runtime: GameCountryWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<
    Result<GameCountryWarMessageReport, CountryWarMessageDispatchError<CountryBattleStateBlock>>,
> {
    let opcode = message.message_type() as u32;
    if matches!(opcode, 0x90502..=0x90509) {
        let governance = dispatch_country_governance_message(message, game, opcode);
        return Some(Ok(GameCountryWarMessageReport {
            dispatched: None,
            governance: Some(governance),
            broadcast: None,
        }));
    }
    if !matches!(opcode, 0x7ff17..=0x7ff1f | 0x7ff22) {
        return None;
    }

    let mut owners = game.take_war_startup_owners();
    let dispatched = {
        let mut context = GameCountryWarContext { game, runtime };
        let (payload, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        dispatch_country_war_message(opcode, payload, cursor, &mut owners.country, &mut context)
            .expect("country-war opcode проверен перед dispatcher-ом")
    };
    game.restore_war_startup_owners(owners);

    let dispatched = match dispatched {
        Ok(dispatched) => dispatched,
        Err(error) => return Some(Err(error)),
    };
    if let Some(message_type) = dispatched.message_type_effect {
        message.set_message_type(message_type as i32);
    }
    let broadcast = dispatched.broadcast.map(|intent| {
        let Some(net_server) = game.current_net_server() else {
            return CountryWarBroadcastOutcome::MissingServer;
        };
        match intent {
            CountryWarBroadcastIntent::All => {
                CountryWarBroadcastOutcome::All(message.send_all(Some(net_server)))
            }
            CountryWarBroadcastIntent::Countries(countries) => {
                let recipients = game.player_ids_in_countries(countries);
                let queue_results = recipients
                    .iter()
                    .map(|&player_id| message.send_to_player(net_server, player_id))
                    .collect();
                CountryWarBroadcastOutcome::Countries {
                    countries,
                    recipients,
                    queue_results,
                }
            }
        }
    });
    Some(Ok(GameCountryWarMessageReport {
        dispatched: Some(dispatched),
        governance: None,
        broadcast,
    }))
}

fn dispatch_country_governance_message(
    message: &mut CMessage,
    game: &CGame,
    opcode: u32,
) -> CountryGovernanceReport {
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let Some(player_id) = player_id else {
        return CountryGovernanceReport {
            opcode,
            player_id,
            country: None,
            outcome: CountryGovernanceOutcome::MissingPlayer,
        };
    };
    let Some(player) = game.find_player(player_id) else {
        return CountryGovernanceReport {
            opcode,
            player_id: Some(player_id),
            country: None,
            outcome: CountryGovernanceOutcome::MissingPlayer,
        };
    };
    let country = player.country();
    let target_id = match opcode {
        0x90504..=0x90507 | 0x90509 => Some(message.base_mut().get_long().unwrap_or(0)),
        0x90508 => Some(player_id),
        _ => None,
    };
    let target = target_id.and_then(|target_id| game.find_player(target_id));
    let rejection = match opcode {
        0x90504 if target.is_none() => Some(b"GS0047".as_slice()),
        0x90504 if target.is_some_and(changing_location) => Some(b"GS0017".as_slice()),
        0x90505 if target.is_none() => Some(b"WS0063".as_slice()),
        0x90505 if target.is_some_and(changing_location) => Some(b"GS0018".as_slice()),
        0x90506 if target.is_some_and(changing_location) => Some(b"GS0019".as_slice()),
        0x90507 if target.is_none() => Some(b"WS0084".as_slice()),
        0x90507 if target.is_some_and(changing_location) => Some(b"GS0020".as_slice()),
        0x90508 if target.is_none() => Some(b"WS0080".as_slice()),
        0x90508 if target.is_some_and(changing_location) => Some(b"GS0021".as_slice()),
        0x90509 if target.is_none() => Some(b"WS0072".as_slice()),
        0x90509 if target.is_some_and(changing_location) => Some(b"GS0021".as_slice()),
        _ => None,
    };
    if let Some(string_id) = rejection {
        let mut response = CMessage::new(0x000c_030d);
        response.base_mut().add(game.get_string_by_id(string_id));
        response.add_byte(0);
        return CountryGovernanceReport {
            opcode,
            player_id: Some(player_id),
            country: Some(country),
            outcome: CountryGovernanceOutcome::Rejected {
                string_id,
                delivery: response.send_to_player(game.net_server(), player_id),
            },
        };
    }

    let world_type = match opcode {
        0x90502 => 0x0006_0306,
        0x90503 => 0x0006_0307,
        0x90504 => 0x0006_0308,
        0x90505 => 0x0006_0309,
        0x90506 => 0x0006_030a,
        0x90507 => 0x0006_030b,
        0x90508 => 0x0006_030c,
        0x90509 => 0x0006_030d,
        _ => unreachable!("governance opcode проверен перед dispatcher-ом"),
    };
    message.set_message_type(world_type);
    message.add_long(player_id);
    message.add_byte(country);
    CountryGovernanceReport {
        opcode,
        player_id: Some(player_id),
        country: Some(country),
        outcome: CountryGovernanceOutcome::Forwarded {
            world_type: world_type as u32,
            target_id,
            delivery: message.send(game, false),
        },
    }
}

fn changing_location(player: &crate::gameserver::appserver::player::CPlayer) -> bool {
    player.in_changing_region() || player.in_changing_server()
}

struct GameCountryWarContext<'a, Runtime> {
    game: &'a mut CGame,
    runtime: &'a mut Runtime,
}

impl<Runtime: GameCountryWarRuntime> CountryWarMessageContext
    for GameCountryWarContext<'_, Runtime>
{
    type Region = i32;
    type SideError = CountryBattleStateBlock;

    fn find_country_region(&mut self, region_id: i32) -> Option<Self::Region> {
        matches!(
            self.game.find_region(region_id),
            Some(ServerRegionOwner::Country(_))
        )
        .then_some(region_id)
    }

    fn set_country_sides(
        &mut self,
        region: Self::Region,
        defend_country: i32,
        attack_country: i32,
    ) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.set_country_sides(defend_country, attack_country);
        }
    }

    fn update_contend_player(&mut self, region: Self::Region) {
        self.runtime.update_country_contend_player(region);
    }

    fn on_declare_begin(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_declare_begin(region_id);
        }
    }

    fn on_declare_end(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_declare_end(region_id);
        }
    }

    fn on_prepare_begin(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_prepare_begin(region_id);
        }
    }

    fn on_prepare_end(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_prepare_end(region_id);
        }
    }

    fn on_war_start(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_war_start(region_id);
        }
    }

    fn on_war_timeout(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_war_timeout(region_id);
        }
    }

    fn on_war_end(&mut self, region: Self::Region, region_id: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_war_end(region_id);
        }
    }

    fn clear_country_region(&mut self, region: Self::Region) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.clear_region(self.runtime);
        }
    }

    fn country_region_side_bytes(
        &mut self,
        region: Self::Region,
    ) -> Result<(u8, u8), Self::SideError> {
        let Some(ServerRegionOwner::Country(region)) = self.game.find_region(region) else {
            unreachable!("country handle получен из того же синхронного CGame map")
        };
        region.country_side_bytes()
    }

    fn reset_country_war_result(&mut self, country: u8) {
        if let Some(country) = self.game.country_handler_mut().country_mut(country) {
            country.country_war_result = 0;
        }
    }

    fn on_country_flag_destroy(&mut self, region: Self::Region, region_id: i32, country: i32) {
        if let Some(ServerRegionOwner::Country(region)) = self.game.find_region_mut(region) {
            region.on_flag_destroy(region_id, country);
        }
    }

    fn set_country_war_result(&mut self, country: u8, result: i32) {
        if let Some(country) = self.game.country_handler_mut().country_mut(country) {
            country.country_war_result = result;
        }
    }
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
