//! Диспетчер World → Game для `CJJcSystem`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/message/jjcsystemmessage.cpp`. `0x80502..0x8050A` теперь
//! разбираются из общего FIFO `CGame::ProcessMessage` и вызывают владельцев
//! JJC, player, region, script и сети; `0x80506/07` остаются точными пустыми
//! callback оригинала. Синхронные эффекты не дублируются отчётом, а их итог
//! публикуется через `tracing`.

use crate::gameserver::appserver::jjcsystem::JjcInfo;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use tracing::{debug, trace};

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
) -> Option<Result<(), GameJjcSystemMessageError>> {
    let source_type = message.message_type() as u32;
    match source_type {
        APPLICATION_RESPONSE => {
            let Some(result) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let _ = game.jjc_notify_application(player_id, result);
            debug!(player_id, result, "игроку отправлен итог заявки JJC");
        }
        MATCHED => {
            let Some(first) = decode_jjc_info(message) else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(second) = decode_jjc_info(message) else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            game.jjc_on_matched(first, second);
            debug!("применено сопоставление участников JJC");
        }
        STARTED => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let Some(region_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            let changed = game.jjc_start_player(region_id, player_id, runtime).is_some();
            debug!(player_id, region_id, changed, "обработан старт JJC для игрока");
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
            debug!(first, second, affected_players = scripts.into_iter().flatten().count(), "обработан тайм-аут JJC");
        }
        0x0008_0506 | 0x0008_0507 => trace!(source_type, "пустой callback JJC сохранён"),
        FORWARD_RESULT => {
            let Some(player_id) = message.base_mut().get_long() else {
                return Some(Err(GameJjcSystemMessageError::MissingField));
            };
            message.base_mut().set_message_type(0x000c_0801);
            let _ = message.send_to_player(game.net_server(), player_id);
            debug!(player_id, "результат JJC перенаправлен игроку");
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
            debug!(source_type, affected_players, "обновлён период JJC");
        }
        _ => return None,
    }
    Some(Ok(()))
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
