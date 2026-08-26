//! Диспетчер сообщений LoginServer и потери игрока.
//!
//! Источник: `gameserver.exe`, `GameServer.pdb` и исходный владелец
//! `appserver/message/logmessage.cpp`. Реализованные ветви сохраняют точное
//! чтение входных полей, адресную пересылку состояния друзей, создание и очистку
//! маршрута входа, порядок регистрации игрока, его удаления из региона и
//! client/World/Billing-отправок. Отложенное удаление в боевом состоянии остаётся
//! частью канонического жизненного цикла `CPlayer`.
//!
//! Все эффекты выполняются синхронно их владельцами. Их результаты публикуются
//! через `tracing`; диспетчер возвращает только ошибку разбора или семантического
//! входа и не создаёт отчёт о уже выполненных действиях.

use crate::gameserver::appserver::player::PlayerGameSaveCodecError;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerLoginBlock, GamePlayerLoginPreludeError,
    colored_player_notice_message, game_wall_time_seconds,
};
use crate::nets::netserver::message::CMessage;

const PLAYER_LOGIN: u32 = 0x0007_f901;
const PLAYER_KICK: u32 = 0x0007_f903;
const FRIEND_ONLINE: u32 = 0x0007_f904;
const FRIEND_OFFLINE: u32 = 0x0007_f905;
const CLIENT_ENTER: u32 = 0x0008_f702;
const PLAYER_LOST: u32 = 0x0006_fa01;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameLogMessageError {
    MissingPlayerId,
    MissingCaptain,
    MissingTeamId,
    MissingClientRoute { player_id: i32 },
    PlayerCodec(PlayerGameSaveCodecError),
    PlayerLoginPrelude(GamePlayerLoginPreludeError),
    PlayerLogin(GamePlayerLoginBlock),
}

pub(crate) fn dispatch_game_log_message<Runtime: GameMainLoopRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameLogMessageError>> {
    let source_type = message.message_type() as u32;
    if source_type == PLAYER_LOST {
        let player_id = message
            .base_mut()
            .get_long()
            .ok_or(GameLogMessageError::MissingPlayerId);
        return Some(player_id.map(|player_id| game.on_player_lost(player_id, runtime)));
    }
    if source_type == PLAYER_KICK {
        return Some(dispatch_player_kick(message, game));
    }
    if source_type == CLIENT_ENTER {
        return Some(dispatch_client_enter(message, game));
    }
    if source_type == PLAYER_LOGIN {
        return Some(dispatch_player_login(message, game, runtime));
    }
    let client_type = match source_type {
        FRIEND_ONLINE => 0x000b_f404,
        FRIEND_OFFLINE => 0x000b_f405,
        _ => return None,
    };
    let Some(player_id) = message.base_mut().get_long() else {
        return Some(Err(GameLogMessageError::MissingPlayerId));
    };
    message.base_mut().set_message_type(client_type as i32);
    let delivery = message.send_to_player(game.net_server(), player_id);
    tracing::trace!(source_type, client_type, player_id, delivery, "состояние друга передано игроку");
    Some(Ok(()))
}

fn dispatch_client_enter(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), GameLogMessageError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(GameLogMessageError::MissingPlayerId)?;
    let socket_id = message.socket_id();
    if game.find_player(player_id).is_some() {
        let route_command = game
            .net_server()
            .command_handle()
            .quit_by_socket_id(socket_id);
        tracing::debug!(player_id, socket_id, route_command, "повторный вход клиента отклонён");
        return Ok(());
    }

    message.base_mut().set_message_type(0x0005_fb01);
    let delivery = message.send(game, false).unwrap_or_default();
    let route_command = game
        .net_server()
        .command_handle()
        .set_client_map_id(socket_id, player_id);
    tracing::trace!(player_id, socket_id, delivery, route_command, "вход клиента запрошен у World");
    Ok(())
}

fn dispatch_player_kick(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<(), GameLogMessageError> {
    let player_id = message
        .base_mut()
        .get_long()
        .ok_or(GameLogMessageError::MissingPlayerId)?;
    game.clear_player_login_validation(player_id);

    if game.find_player(player_id).is_some() {
        let notice = colored_player_notice_message(
            0xffff_ffff,
            0xffff_0000,
            game.get_string_by_id(b"GS0041"),
        );
        let delivery = notice.send_to_player(game.net_server(), player_id);
        let kick = game.kick_player(player_id);
        tracing::trace!(player_id, delivery, route_command = kick.command_result, "активный игрок отключён");
        return Ok(());
    }

    if game.net_server().has_player_map_id(player_id) {
        let delivery = publish_login_kick_confirmation(game, player_id);
        let (_, route_command) = game.discard_player_login(player_id);
        tracing::trace!(player_id, delivery, route_command, "ожидающий входа игрок отключён");
        return Ok(());
    }

    if game.player_registered_in_region(player_id) {
        let kick = game.kick_player(player_id);
        tracing::warn!(player_id, route_command = kick.command_result, "игрок без основного владельца удалён из региона");
        return Ok(());
    }

    let delivery = publish_login_kick_confirmation(game, player_id);
    tracing::trace!(player_id, delivery, "отсутствие игрока подтверждено LoginServer");
    Ok(())
}

fn publish_login_kick_confirmation(game: &CGame, player_id: i32) -> i32 {
    let mut confirmation = CMessage::new(0x0005_fb02);
    confirmation.add_long(player_id);
    confirmation.add_long(0);
    confirmation.send(game, false).unwrap_or_default()
}

fn dispatch_player_login<Runtime: GameMainLoopRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<(), GameLogMessageError> {
    let status = message
        .base_mut()
        .get_long()
        .ok_or(GameLogMessageError::MissingPlayerId)?;
    if matches!(status, 0 | -1) {
        let player_id = message
            .base_mut()
            .get_long()
            .ok_or(GameLogMessageError::MissingPlayerId)?;
        let delivery = reject_player_login(game, player_id, false);
        tracing::debug!(player_id, status, delivery, "вход игрока отклонён World");
        return Ok(());
    }
    if status == -2 {
        tracing::trace!(status, "служебный результат входа игрока проигнорирован");
        return Ok(());
    }

    let player_id = status;
    if !game.net_server().has_player_map_id(player_id) {
        let _delivery = reject_player_login(game, player_id, true);
        return Err(GameLogMessageError::MissingClientRoute { player_id });
    }
    let now_ms = runtime.now_milliseconds();
    game
        .begin_player_login_validation(player_id, now_ms, game_wall_time_seconds() as u32)
        .map_err(|error| {
            let _delivery = reject_player_login(game, player_id, true);
            GameLogMessageError::PlayerLoginPrelude(error)
        })?;
    let Some(captain) = message.base_mut().get_byte() else {
        let _delivery = reject_player_login(game, player_id, true);
        return Err(GameLogMessageError::MissingCaptain);
    };
    let captain = captain != 0;
    let Some(team_id) = message.base_mut().get_long() else {
        let _delivery = reject_player_login(game, player_id, true);
        return Err(GameLogMessageError::MissingTeamId);
    };
    let (player, codec) = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        match game.decode_player_game_save(source, cursor, now_ms) {
            Ok(decoded) => decoded,
            Err(error) => {
                let _delivery = reject_player_login(game, player_id, true);
                return Err(GameLogMessageError::PlayerCodec(error));
            }
        }
    };
    let decoded_bytes = codec.consumed_bytes;
    game
        .complete_world_player_login(player_id, player, captain, team_id, runtime)
        .map_err(|block| {
            let _delivery = reject_player_login(game, player_id, true);
            GameLogMessageError::PlayerLogin(block)
        })?;
    tracing::trace!(
        player_id,
        status,
        captain,
        team_id,
        decoded_bytes,
        "вход игрока завершён"
    );
    Ok(())
}

fn reject_player_login(game: &mut CGame, player_id: i32, notify_world: bool) -> i32 {
    if notify_world {
        let mut kick = CMessage::new(0x0005_fb02);
        kick.add_long(player_id);
        kick.add_long(0);
        let _ = kick.send(game, false);
    }
    let mut failed = CMessage::new(0x000b_f401);
    failed.add_long(0);
    let delivery = failed.send_to_player(game.net_server(), player_id);
    let _ = game.discard_player_login(player_id);
    delivery
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\logmessage.cpp

// ============================================================================
// FUNCTION: OnLogMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\logmessage.cpp:30
// RVA: 0x0009F140
// ADDRESS: 0049f140
// PROTOTYPE: void __cdecl OnLogMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a0620
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\logmessage.cpp:219
// RVA: 0x000A0620
// ADDRESS: 004a0620
// PROTOTYPE: undefined Catch@004a0620()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
