//! Входящий Team-owner WorldServer.
//!
//! Источник контракта `OnTeamMessage` — `worldserver.exe` и
//! `worldserver.pdb`.
//!
//! Реализация сохраняет opcodes `0x60001..0x6000C`, условный порядок чтения
//! payload и все действующие virtual side effects. После отсутствующего
//! `CTeam` остаётся прочитан только team ID; `0x60009` игнорирует два средних
//! `long`; allocation scheme принимает любое signed значение `< 2`, включая
//! отрицательное. Ответ `0x7FD08` буквально содержит virtual `Serialize`.
//!
//! Полный tail-check, duplicate-team gate и cleanup при ошибке `InsertPlug`
//! отсутствуют в EXE. Единственные registry остаются внутри
//! `CSessionFactory`; конкретные `CTeam`/`CTeamate` и узкие trait-проекции
//! выражают исходные RTTI/virtual границы. Недостаточный scalar payload даёт
//! typed malformed до относящегося к нему эффекта.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::appworld::session::csessionfactory::{
    CSessionFactory, WorldSessionFactoryInputBlock,
};
use crate::worldserver::worldserver::game::CGame;

const TEAMATE_PLUG_TYPE: i32 = 5;
const PLAYER_OWNER_TYPE: i32 = 400;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldTeamMessageOutcome {
    SessionUnserialized {
        outcome: Result<i32, WorldSessionFactoryInputBlock>,
        cursor: usize,
    },
    TeamEnded { team_id: i32, session_id: i32, result: Option<i32> },
    TeamateAdded {
        team_id: i32,
        session_id: i32,
        plug_id: Option<i32>,
        inserted: Option<i32>,
    },
    TeamateExited {
        team_id: i32,
        session_id: i32,
        plug_id: Option<i32>,
        exited: bool,
    },
    TeamateRegionUpdated {
        team_id: i32,
        session_id: i32,
        plug_id: Option<i32>,
        updated: bool,
    },
    LeaderUpdated {
        team_id: i32,
        session_id: i32,
        player_id: i32,
        updated: bool,
    },
    PlayerKicked {
        team_id: i32,
        session_id: i32,
        player_id: i32,
        kicked: bool,
    },
    TeamSnapshot {
        team_id: i32,
        session_id: i32,
        serialized: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    TeamateExistenceMarked { plug_id: i32, marked: bool },
    AllocationUpdated {
        team_id: i32,
        session_id: i32,
        requested: i32,
        updated: bool,
    },
    PlugStateUpdated {
        team_id: i32,
        session_id: i32,
        plug_id: Option<i32>,
        state: i32,
        updated: bool,
    },
    TeamMissing { message_type: i32, team_id: i32, session_id: i32 },
    UnknownOpcode { message_type: i32 },
    Malformed { message_type: i32, cursor: usize },
}

pub(crate) fn on_team_message(
    game: &mut CGame,
    factory: &mut CSessionFactory,
    message: &mut CMessage,
) -> WorldTeamMessageOutcome {
    let message_type = message.message_type();

    macro_rules! malformed {
        () => {
            return WorldTeamMessageOutcome::Malformed {
                message_type,
                cursor: message.base_mut().cursor(),
            }
        };
    }
    macro_rules! team_session {
        () => {{
            let Some(team_id) = message.base_mut().get_long() else { malformed!() };
            let session_id = game.get_team_session_id(team_id as u32);
            if !factory.is_team(session_id) {
                return WorldTeamMessageOutcome::TeamMissing { message_type, team_id, session_id };
            }
            (team_id, session_id)
        }};
    }

    match message_type {
        0x0006_0001 => {
            let (wire, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
            let Ok(mut offset) = i32::try_from(*cursor) else {
                return WorldTeamMessageOutcome::Malformed { message_type, cursor: *cursor };
            };
            let outcome = factory.unserialize_session(game, Some(wire), &mut offset);
            if let Ok(next_cursor) = usize::try_from(offset) {
                *cursor = next_cursor;
            }
            WorldTeamMessageOutcome::SessionUnserialized { outcome, cursor: *cursor }
        }
        0x0006_0002 => {
            let Some(team_id) = message.base_mut().get_long() else { malformed!() };
            let session_id = game.get_team_session_id(team_id as u32);
            WorldTeamMessageOutcome::TeamEnded {
                team_id,
                session_id,
                result: factory.end_session(game, session_id),
            }
        }
        0x0006_0003 => {
            let (team_id, session_id) = team_session!();
            let Some(owner_type) = message.base_mut().get_long() else { malformed!() };
            let Some(owner_id) = message.base_mut().get_long() else { malformed!() };
            let Some(region_id) = message.base_mut().get_long() else { malformed!() };
            let owner_name = message.base_mut().get_str_bytes(0x100)
                .expect("ненулевая legacy-граница GetStr");
            let existing = factory.with_team(game, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            }).flatten();
            if existing.is_some() {
                return WorldTeamMessageOutcome::TeamateAdded {
                    team_id, session_id, plug_id: existing, inserted: None,
                };
            }
            let plug_id = factory.create_plug(TEAMATE_PLUG_TYPE, owner_type, owner_id);
            let initialized = factory.with_teamate(game, plug_id, |teamate| {
                teamate.set_owner_region_id(region_id);
                teamate.set_owner_name(&owner_name);
            }).is_some();
            let inserted = initialized.then(|| factory.insert_plug(game, session_id, plug_id));
            WorldTeamMessageOutcome::TeamateAdded {
                team_id,
                session_id,
                plug_id: initialized.then_some(plug_id),
                inserted,
            }
        }
        0x0006_0004 => {
            let (team_id, session_id) = team_session!();
            let Some(owner_type) = message.base_mut().get_long() else { malformed!() };
            let Some(owner_id) = message.base_mut().get_long() else { malformed!() };
            let plug_id = factory.with_team(game, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            }).flatten();
            let exited = plug_id.and_then(|plug_id| {
                factory.with_teamate(game, plug_id, |teamate| teamate.exit())
            }).is_some();
            WorldTeamMessageOutcome::TeamateExited {
                team_id, session_id, plug_id, exited,
            }
        }
        0x0006_0005 => {
            let (team_id, session_id) = team_session!();
            let Some(owner_type) = message.base_mut().get_long() else { malformed!() };
            let Some(owner_id) = message.base_mut().get_long() else { malformed!() };
            let Some(region_id) = message.base_mut().get_long() else { malformed!() };
            let plug_id = factory.with_team(game, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            }).flatten();
            let updated = plug_id.and_then(|plug_id| {
                factory.with_teamate(game, plug_id, |teamate| teamate.set_owner_region_id(region_id))
            }).is_some();
            WorldTeamMessageOutcome::TeamateRegionUpdated {
                team_id, session_id, plug_id, updated,
            }
        }
        0x0006_0006 => {
            let (team_id, session_id) = team_session!();
            let Some(player_id) = message.base_mut().get_long() else { malformed!() };
            let updated = factory.with_team(game, session_id, |team| {
                if team.query_plug_by_owner(PLAYER_OWNER_TYPE, player_id).is_some() {
                    team.set_leader(player_id);
                    true
                } else {
                    false
                }
            }).unwrap_or(false);
            WorldTeamMessageOutcome::LeaderUpdated {
                team_id, session_id, player_id, updated,
            }
        }
        0x0006_0007 => {
            let (team_id, session_id) = team_session!();
            let Some(player_id) = message.base_mut().get_long() else { malformed!() };
            let kicked = factory.with_team(game, session_id, |team| team.kick_player(player_id)).is_some();
            WorldTeamMessageOutcome::PlayerKicked {
                team_id, session_id, player_id, kicked,
            }
        }
        0x0006_0008 => {
            let (team_id, session_id) = team_session!();
            let serialized = factory.serialize_team(session_id);
            let delivery = serialized.as_ref().map(|bytes| {
                let mut response = CMessage::new(0x0007_FD08);
                response.base_mut().add(bytes);
                response.base_mut().update();
                response.send_to_socket(
                    game.current_game_server_sender().as_ref(), message.socket_id(),
                )
            });
            WorldTeamMessageOutcome::TeamSnapshot {
                team_id, session_id, serialized, delivery,
            }
        }
        0x0006_0009 => {
            let Some(plug_id) = message.base_mut().get_long() else { malformed!() };
            let Some(_) = message.base_mut().get_long() else { malformed!() };
            let Some(_) = message.base_mut().get_long() else { malformed!() };
            let Some(existed) = message.base_mut().get_long() else { malformed!() };
            let marked = existed != 0 && factory.with_teamate(game, plug_id, |teamate| {
                teamate.player_still_existed(1)
            }).is_some();
            WorldTeamMessageOutcome::TeamateExistenceMarked { plug_id, marked }
        }
        0x0006_000A => {
            let (team_id, session_id) = team_session!();
            let Some(requested) = message.base_mut().get_long() else { malformed!() };
            let updated = factory.with_team(game, session_id, |team| {
                if requested != team.allocation_scheme() && requested < 2 {
                    team.set_allocation_scheme(requested);
                    true
                } else {
                    false
                }
            }).unwrap_or(false);
            WorldTeamMessageOutcome::AllocationUpdated {
                team_id, session_id, requested, updated,
            }
        }
        0x0006_000B => {
            let (team_id, session_id) = team_session!();
            let Some(owner_type) = message.base_mut().get_long() else { malformed!() };
            let Some(owner_id) = message.base_mut().get_long() else { malformed!() };
            let plug_id = factory.with_team(game, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            }).flatten();
            let Some(plug_id_value) = plug_id else {
                return WorldTeamMessageOutcome::PlugStateUpdated {
                    team_id, session_id, plug_id, state: 8, updated: false,
                };
            };
            let text = message.base_mut().get_str_bytes(0x200)
                .expect("ненулевая legacy-граница GetStr");
            let updated = factory.with_team(game, session_id, |team| {
                team.on_plug_change_state(plug_id_value, 8, &text)
            }).is_some();
            WorldTeamMessageOutcome::PlugStateUpdated {
                team_id, session_id, plug_id, state: 8, updated,
            }
        }
        0x0006_000C => {
            let (team_id, session_id) = team_session!();
            let Some(owner_type) = message.base_mut().get_long() else { malformed!() };
            let Some(owner_id) = message.base_mut().get_long() else { malformed!() };
            let Some(value) = message.base_mut().get_float() else { malformed!() };
            let plug_id = factory.with_team(game, session_id, |team| {
                team.query_plug_by_owner(owner_type, owner_id)
            }).flatten();
            let teamate_id = plug_id.filter(|plug_id| {
                factory.with_teamate(game, *plug_id, |_| ()).is_some()
            });
            let updated = teamate_id
                .and_then(|plug_id| {
                    factory.with_team(game, session_id, |team| {
                        team.on_plug_change_state(plug_id, 9, &value.to_le_bytes())
                    })
                })
                .is_some();
            WorldTeamMessageOutcome::PlugStateUpdated {
                team_id, session_id, plug_id, state: 9, updated,
            }
        }
        _ => WorldTeamMessageOutcome::UnknownOpcode { message_type },
    }
}
