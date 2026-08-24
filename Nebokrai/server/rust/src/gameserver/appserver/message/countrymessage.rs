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
//! Client governance `0x90502..0x9050A` проверяет caller/target changing state,
//! сохраняет selector-specific `GS/WS` ошибки через `0xC030D` и пересылает
//! исходный payload с caller ID/country в достигнутые WorldServer
//! `0x60306..0x6030D`. Ветка `0x9050A` также сохраняет исходную проверку
//! target changing-state и wire `[target, caller, country]`; подтверждённый
//! WorldServer `0x6030F` принимает пакет как намеренный no-op без ответа.
//! `0x9050B` замыкает вход в country-war: ordered camp-area и region RNG,
//! `ChangeRegion`, wrapping/clamped exploit и client `0xBF72E/0xBF816`.
//! Ответ World `0x7FF01` применяет country только в диапазоне `1..4`, но для
//! любого decoded результата найденного player публикует around `0xC0301`.
//! `0x7FF04(country, player, job, active)` сначала меняет country-information,
//! затем для live player публикует job через `0xC0302`; active `2` скрывает job.
//! Family `0x7FF05/07/08` синхронизирует king ID либо, сохраняя исходный
//! payload, адресно меняет type на client `0xC0304/0xC0305` и отправляет королю.

use super::super::country::country::{
    CountryInformationMutationReport, CountryKingIdMutationReport,
};
use super::super::country::countrywarsys::{
    CountryWarPhaseContext, CountryWarRegionContext, CountryWarSys, CountryWarVictoryContext,
};
use super::super::player::{PlayerCountryMutationReport, PlayerExploitMutationReport};
use super::super::region::{RegionCellAccessBlock, RegionRandomContext, RegionRandomPosition};
use super::super::servercountryregion::{CountryBattleStateBlock, CountryRegionRuntimeContext};
use super::super::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{CGame, ServerRegionOwner};
use crate::nets::netserver::message::{CMessage, SendMessageError};

pub(crate) trait GameCountryWarRuntime:
    CountryRegionRuntimeContext + RegionRandomContext
{
    /// Материализует virtual `UpdateContendPlayer` country-region owner-а.
    fn update_country_contend_player(&mut self, region_id: i32);

    /// Исполняет `CPlayer::ChangeRegion(region, x, y, -1, 0, 0, 0)`; legacy
    /// bool наблюдается в отчёте, но не gate-ит последующую exploit-награду.
    fn change_country_war_player_region(
        &mut self,
        player_id: i32,
        region_id: i32,
        x: i32,
        y: i32,
    ) -> bool;
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
    pub(crate) entry: Option<CountryWarEntryReport>,
    pub(crate) player_country_change: Option<GamePlayerCountryChangeReport>,
    pub(crate) country_information_change: Option<GameCountryInformationChangeReport>,
    pub(crate) direct_response: Option<GameCountryDirectResponseReport>,
    pub(crate) broadcast: Option<CountryWarBroadcastOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameCountryDirectResponseOutcome {
    CountryMissing,
    KingIdUpdated(CountryKingIdMutationReport),
    Relayed {
        player_id: i32,
        player_id_complete: bool,
        response_type: u32,
        delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameCountryDirectResponseReport {
    pub(crate) opcode: u32,
    pub(crate) country: Option<u8>,
    pub(crate) country_complete: Option<bool>,
    pub(crate) king_id: Option<i32>,
    pub(crate) king_id_complete: Option<bool>,
    pub(crate) outcome: GameCountryDirectResponseOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameCountryInformationChangeOutcome {
    CountryMissing,
    PlayerMissing {
        mutation: CountryInformationMutationReport,
    },
    Published {
        mutation: CountryInformationMutationReport,
        client_job: u8,
        around_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameCountryInformationChangeReport {
    pub(crate) opcode: u32,
    pub(crate) country: u8,
    pub(crate) country_complete: bool,
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) job: u8,
    pub(crate) job_complete: bool,
    pub(crate) active: u8,
    pub(crate) active_complete: bool,
    pub(crate) outcome: GameCountryInformationChangeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerCountryChangeOutcome {
    MissingPlayer,
    Published {
        mutation: PlayerCountryMutationReport,
        around_delivery: Option<Result<i32, ShapeCoordinateBlock>>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerCountryChangeReport {
    pub(crate) opcode: u32,
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) country: i32,
    pub(crate) country_complete: bool,
    pub(crate) outcome: GamePlayerCountryChangeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryWarEntryOutcome {
    MissingPlayer,
    MissingRegion {
        region_id: i32,
    },
    WrongRegionKind {
        region_id: i32,
    },
    MissingCamp {
        country: u8,
        camp: i32,
    },
    MissingEntryArea {
        region_id: i32,
        camp: i32,
    },
    PositionBlocked(RegionCellAccessBlock),
    CountryParametersUnavailable {
        increment: Option<i32>,
        maximum: Option<i32>,
    },
    Completed {
        region_id: i32,
        camp: i32,
        position: RegionRandomPosition,
        change_region_result: bool,
        exploit: PlayerExploitMutationReport,
        exploit_delivery: i32,
        notice_delivery: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CountryWarEntryReport {
    pub(crate) opcode: u32,
    pub(crate) player_id: i32,
    pub(crate) player_id_complete: bool,
    pub(crate) outcome: CountryWarEntryOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CountryGovernanceOutcome {
    MissingPlayer,
    MissingTarget {
        target_id: i32,
    },
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
    if matches!(opcode, 0x7ff05 | 0x7ff07 | 0x7ff08) {
        let direct_response = dispatch_country_direct_response_message(message, game, opcode);
        return Some(Ok(GameCountryWarMessageReport {
            dispatched: None,
            governance: None,
            entry: None,
            player_country_change: None,
            country_information_change: None,
            direct_response: Some(direct_response),
            broadcast: None,
        }));
    }
    if opcode == 0x7ff04 {
        let country_information_change = dispatch_country_information_change_message(message, game);
        return Some(Ok(GameCountryWarMessageReport {
            dispatched: None,
            governance: None,
            entry: None,
            player_country_change: None,
            country_information_change: Some(country_information_change),
            direct_response: None,
            broadcast: None,
        }));
    }
    if opcode == 0x7ff01 {
        let player_country_change = dispatch_player_country_change_message(message, game);
        return Some(Ok(GameCountryWarMessageReport {
            dispatched: None,
            governance: None,
            entry: None,
            player_country_change: Some(player_country_change),
            country_information_change: None,
            direct_response: None,
            broadcast: None,
        }));
    }
    if opcode == 0x9050b {
        let entry = dispatch_country_war_entry_message(message, game, runtime);
        return Some(Ok(GameCountryWarMessageReport {
            dispatched: None,
            governance: None,
            entry: Some(entry),
            player_country_change: None,
            country_information_change: None,
            direct_response: None,
            broadcast: None,
        }));
    }
    if matches!(opcode, 0x90502..=0x9050a) {
        let governance = dispatch_country_governance_message(message, game, opcode);
        return Some(Ok(GameCountryWarMessageReport {
            dispatched: None,
            governance: Some(governance),
            entry: None,
            player_country_change: None,
            country_information_change: None,
            direct_response: None,
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
        entry: None,
        player_country_change: None,
        country_information_change: None,
        direct_response: None,
        broadcast,
    }))
}

fn dispatch_country_direct_response_message(
    message: &mut CMessage,
    game: &mut CGame,
    opcode: u32,
) -> GameCountryDirectResponseReport {
    if opcode == 0x7ff05 {
        let decoded_country = message.base_mut().get_char();
        let country = decoded_country.unwrap_or(0) as u8;
        let decoded_king_id = message.base_mut().get_long();
        let king_id = decoded_king_id.unwrap_or(0);
        let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
            return GameCountryDirectResponseReport {
                opcode,
                country: Some(country),
                country_complete: Some(decoded_country.is_some()),
                king_id: Some(king_id),
                king_id_complete: Some(decoded_king_id.is_some()),
                outcome: GameCountryDirectResponseOutcome::CountryMissing,
            };
        };
        let mutation = country_owner.set_king_id(king_id);
        return GameCountryDirectResponseReport {
            opcode,
            country: Some(country),
            country_complete: Some(decoded_country.is_some()),
            king_id: Some(king_id),
            king_id_complete: Some(decoded_king_id.is_some()),
            outcome: GameCountryDirectResponseOutcome::KingIdUpdated(mutation),
        };
    }

    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let response_type = match opcode {
        0x7ff07 => 0x000c_0304,
        0x7ff08 => 0x000c_0305,
        _ => unreachable!("direct country response opcode проверен caller-ом"),
    };
    message.set_message_type(response_type);
    let delivery = message.send_to_player(game.net_server(), player_id);
    GameCountryDirectResponseReport {
        opcode,
        country: None,
        country_complete: None,
        king_id: None,
        king_id_complete: None,
        outcome: GameCountryDirectResponseOutcome::Relayed {
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            response_type: response_type as u32,
            delivery,
        },
    }
}

fn dispatch_country_information_change_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> GameCountryInformationChangeReport {
    let decoded_country = message.base_mut().get_char();
    let country = decoded_country.unwrap_or(0) as u8;
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_job = message.base_mut().get_char();
    let job = decoded_job.unwrap_or(0) as u8;
    let decoded_active = message.base_mut().get_char();
    let active = decoded_active.unwrap_or(0) as u8;
    let base_report = |outcome| GameCountryInformationChangeReport {
        opcode: 0x7ff04,
        country,
        country_complete: decoded_country.is_some(),
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        job,
        job_complete: decoded_job.is_some(),
        active,
        active_complete: decoded_active.is_some(),
        outcome,
    };
    let Some(country_owner) = game.country_handler_mut().country_mut(country) else {
        return base_report(GameCountryInformationChangeOutcome::CountryMissing);
    };
    let mutation = country_owner.set_country_information(job, player_id, active);
    if game.find_player(player_id).is_none() {
        return base_report(GameCountryInformationChangeOutcome::PlayerMissing { mutation });
    }
    let client_job = if active == 2 { 0 } else { job };
    let mut publication = CMessage::new(0x000c_0302);
    publication.add_long(player_id);
    publication.add_byte(client_job);
    let around_delivery = game.send_player_shape_around(player_id, None, &publication);
    base_report(GameCountryInformationChangeOutcome::Published {
        mutation,
        client_job,
        around_delivery,
    })
}

fn dispatch_player_country_change_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> GamePlayerCountryChangeReport {
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let decoded_country = message.base_mut().get_long();
    let country = decoded_country.unwrap_or(0);
    let Some(player) = game.find_player_mut(player_id) else {
        return GamePlayerCountryChangeReport {
            opcode: 0x7ff01,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            country,
            country_complete: decoded_country.is_some(),
            outcome: GamePlayerCountryChangeOutcome::MissingPlayer,
        };
    };
    let mutation = player.apply_world_country(country);
    let mut publication = CMessage::new(0x000c_0301);
    publication.add_long(country);
    publication.add_long(player_id);
    let around_delivery = game.send_player_shape_around(player_id, None, &publication);
    GamePlayerCountryChangeReport {
        opcode: 0x7ff01,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        country,
        country_complete: decoded_country.is_some(),
        outcome: GamePlayerCountryChangeOutcome::Published {
            mutation,
            around_delivery,
        },
    }
}

fn dispatch_country_war_entry_message<Runtime: GameCountryWarRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> CountryWarEntryReport {
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0);
    let missing_player = || CountryWarEntryReport {
        opcode: 0x9050b,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        outcome: CountryWarEntryOutcome::MissingPlayer,
    };
    let Some(player) = game.find_player(player_id) else {
        return missing_player();
    };
    let country = player.country();
    let war_region_id = game
        .country_war_sys()
        .get_war_region_for_country(i32::from(country));
    let camp = game.country_war_sys().get_war_camp(i32::from(country));
    if !matches!(camp, 0 | 1) {
        return CountryWarEntryReport {
            opcode: 0x9050b,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            outcome: CountryWarEntryOutcome::MissingCamp { country, camp },
        };
    }
    let Some(owner) = game.take_region_owner(war_region_id) else {
        return CountryWarEntryReport {
            opcode: 0x9050b,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            outcome: CountryWarEntryOutcome::MissingRegion {
                region_id: war_region_id,
            },
        };
    };
    let ServerRegionOwner::Country(region) = &owner else {
        game.restore_region_owner(owner);
        return CountryWarEntryReport {
            opcode: 0x9050b,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            outcome: CountryWarEntryOutcome::WrongRegionKind {
                region_id: war_region_id,
            },
        };
    };
    let position = region.country_war_entry_position(camp, runtime);
    game.restore_region_owner(owner);
    let position = match position {
        Ok(Some(position)) => position,
        Ok(None) => {
            return CountryWarEntryReport {
                opcode: 0x9050b,
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                outcome: CountryWarEntryOutcome::MissingEntryArea {
                    region_id: war_region_id,
                    camp,
                },
            };
        }
        Err(block) => {
            return CountryWarEntryReport {
                opcode: 0x9050b,
                player_id,
                player_id_complete: decoded_player_id.is_some(),
                outcome: CountryWarEntryOutcome::PositionBlocked(block),
            };
        }
    };

    let increment = game.country_param().exploit_increment();
    let maximum = game.country_param().max_exploit();
    let (Some(increment), Some(maximum)) = (increment, maximum) else {
        return CountryWarEntryReport {
            opcode: 0x9050b,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            outcome: CountryWarEntryOutcome::CountryParametersUnavailable { increment, maximum },
        };
    };
    let change_region_result =
        runtime.change_country_war_player_region(player_id, war_region_id, position.x, position.y);
    let exploit = {
        let player = game
            .find_player_mut(player_id)
            .expect("0x9050B сохраняет live player до ChangeRegion boundary");
        player.set_exploit(player.exploit().wrapping_add(increment as u32), maximum)
    };
    let mut exploit_message = CMessage::new(0x000b_f72e);
    exploit_message.add_ulong(exploit.applied);
    let exploit_delivery = exploit_message.send_to_player(game.net_server(), player_id);

    let notice = format_country_war_exploit_notice(game.get_string_by_id(b"GS0024"), increment);
    let mut notice_message = CMessage::new(0x000b_f816);
    notice_message.add_byte(0);
    add_country_legacy_c_string(&mut notice_message, &notice);
    let notice_delivery = notice_message.send_to_player(game.net_server(), player_id);

    CountryWarEntryReport {
        opcode: 0x9050b,
        player_id,
        player_id_complete: decoded_player_id.is_some(),
        outcome: CountryWarEntryOutcome::Completed {
            region_id: war_region_id,
            camp,
            position,
            change_region_result,
            exploit,
            exploit_delivery,
            notice_delivery,
        },
    }
}

fn format_country_war_exploit_notice(template: &[u8], increment: i32) -> Vec<u8> {
    let template = &template[..template
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(template.len())];
    let value = increment.to_string();
    let mut text = Vec::with_capacity(template.len().saturating_add(value.len()));
    if let Some(marker) = template.windows(2).position(|window| window == b"%d") {
        text.extend_from_slice(&template[..marker]);
        text.extend_from_slice(value.as_bytes());
        text.extend_from_slice(&template[marker + 2..]);
    } else {
        text.extend_from_slice(template);
    }
    text.truncate(255);
    text
}

fn add_country_legacy_c_string(message: &mut CMessage, value: &[u8]) {
    let value = &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())];
    message.base_mut().add(value);
    message.add_byte(0);
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
        0x90504..=0x90507 | 0x90509..=0x9050a => Some(message.base_mut().get_long().unwrap_or(0)),
        0x90508 => Some(player_id),
        _ => None,
    };
    let target = target_id.and_then(|target_id| game.find_player(target_id));
    if opcode == 0x9050a && target.is_none() {
        return CountryGovernanceReport {
            opcode,
            player_id: Some(player_id),
            country: Some(country),
            outcome: CountryGovernanceOutcome::MissingTarget {
                target_id: target_id.expect("0x9050A всегда читает target ID"),
            },
        };
    }
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
        0x9050a if target.is_some_and(changing_location) => Some(b"GS0022".as_slice()),
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
        0x9050a => 0x0006_030f,
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
