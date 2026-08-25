//! World → Game dispatcher `CJJcSystem`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/jjcsystemmessage.cpp`. `0x80502..0x8050A` теперь
//! разбираются из общего `CGame::ProcessMessage` FIFO и вызывают owned JJC,
//! player, region, script и network owners; `0x80506/07` остаются точными
//! no-op callback-ами оригинала.

use crate::gameserver::appserver::jjcsystem::JjcInfo;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const APPLICATION_RESPONSE: u32 = 0x0008_0502;
const MATCHED: u32 = 0x0008_0503;
const STARTED: u32 = 0x0008_0504;
const TIMEOUT: u32 = 0x0008_0505;
const FORWARD_RESULT: u32 = 0x0008_0508;
const WEEK_UPDATE: u32 = 0x0008_0509;
const SEASON_UPDATE: u32 = 0x0008_050a;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameJjcSystemMessageError {
    MissingField,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameJjcSystemMessageOutcome {
    ApplicationNotified,
    Matched,
    Started,
    Timeout,
    Forwarded,
    WeekUpdated,
    SeasonUpdated,
    Ignored,
}

#[must_use = "JJC message report сохраняет reached effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameJjcSystemMessageReport {
    pub(crate) source_type: u32,
    pub(crate) outcome: GameJjcSystemMessageOutcome,
    pub(crate) player_id: Option<i32>,
    pub(crate) delivery: Option<i32>,
    pub(crate) affected_players: usize,
}

fn decode_jjc_info(message: &mut CMessage) -> Option<JjcInfo> {
    Some(JjcInfo {
        jjc_level: message.base_mut().get_long()? as u32,
        old_region_id: message.base_mut().get_long()?,
        pos_x: message.base_mut().get_long()?,
        pos_y: message.base_mut().get_long()?,
        opponent_id: message.base_mut().get_long()?,
        jjc_region_id: message.base_mut().get_long()?,
        start_time: message.base_mut().get_long()?,
    })
}

pub(crate) fn dispatch_game_jjc_system_message<Runtime: GameMainLoopRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameJjcSystemMessageReport, GameJjcSystemMessageError>> {
    let source_type = message.message_type() as u32;
    let result = match source_type {
        APPLICATION_RESPONSE => {
            let Some(result) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let delivery = game.jjc_notify_application(player_id, result);
            GameJjcSystemMessageReport {
                source_type,
                outcome: GameJjcSystemMessageOutcome::ApplicationNotified,
                player_id: Some(player_id),
                delivery: Some(delivery),
                affected_players: usize::from(game.find_player(player_id).is_some()),
            }
        }
        MATCHED => {
            let Some(first) = decode_jjc_info(message) else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(second) = decode_jjc_info(message) else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            game.jjc_on_matched(first, second);
            GameJjcSystemMessageReport {
                source_type,
                outcome: GameJjcSystemMessageOutcome::Matched,
                player_id: None,
                delivery: None,
                affected_players: 2,
            }
        }
        STARTED => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(region_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let changed = game
                .jjc_start_player(region_id, player_id, runtime)
                .is_some();
            GameJjcSystemMessageReport {
                source_type,
                outcome: GameJjcSystemMessageOutcome::Started,
                player_id: Some(player_id),
                delivery: None,
                affected_players: usize::from(changed),
            }
        }
        TIMEOUT => {
            let Some(_region_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(first) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(second) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let scripts = game.jjc_timeout([first, second]);
            GameJjcSystemMessageReport {
                source_type,
                outcome: GameJjcSystemMessageOutcome::Timeout,
                player_id: None,
                delivery: None,
                affected_players: scripts.into_iter().flatten().count(),
            }
        }
        0x0008_0506 | 0x0008_0507 => GameJjcSystemMessageReport {
            source_type,
            outcome: GameJjcSystemMessageOutcome::Ignored,
            player_id: None,
            delivery: None,
            affected_players: 0,
        },
        FORWARD_RESULT => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            message.base_mut().set_message_type(0x000c_0801);
            let delivery = message.send_to_player(game.net_server(), player_id);
            GameJjcSystemMessageReport {
                source_type,
                outcome: GameJjcSystemMessageOutcome::Forwarded,
                player_id: Some(player_id),
                delivery: Some(delivery),
                affected_players: usize::from(game.find_player(player_id).is_some()),
            }
        }
        WEEK_UPDATE | SEASON_UPDATE => {
            let Some(_timestamp) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let affected_players = if source_type == WEEK_UPDATE {
                game.jjc_week_update().len()
            } else {
                game.jjc_season_update().len()
            };
            GameJjcSystemMessageReport {
                source_type,
                outcome: if source_type == WEEK_UPDATE {
                    GameJjcSystemMessageOutcome::WeekUpdated
                } else {
                    GameJjcSystemMessageOutcome::SeasonUpdated
                },
                player_id: None,
                delivery: None,
                affected_players,
            }
        }
        _ => return None,
    };
    Some(Ok(result))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\jjcsystemmessage.cpp

// ============================================================================
// FUNCTION: FUN_00497600
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\jjcsystemmessage.cpp:83
// RVA: 0x00097600
// ADDRESS: 00497600
// PROTOTYPE: undefined FUN_00497600()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: OnJJcSystemMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\jjcsystemmessage.cpp:13
// RVA: 0x000978E0
// ADDRESS: 004978e0
// PROTOTYPE: void __cdecl OnJJcSystemMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
