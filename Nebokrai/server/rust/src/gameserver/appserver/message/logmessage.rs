//! World/log-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/logmessage.cpp`. Friend presence `0x7F904/905` сохраняет
//! исходный `(playerId, friendName)` payload, меняет только message type на
//! `0xBF404/405` и адресно пересылает указанному игроку. Player login
//! `0x7F901` теперь проходит тот же live main-loop runtime: status/captain/
//! team читаются владельцем сообщения, полный `CPlayer` декодируется единым
//! GameSave codec-ом, после чего `CGame` выполняет map/region membership,
//! login/honor script scheduling, property/client publication, Billing
//! `0xEF201`, полный GoodsAI traversal и expired-equipment `0xBF928` tail.
//! Legacy pending `s_mapPlayer` placeholder заменён уже существующим transport
//! map-id: полноценный owned `CPlayer` появляется в canonical map только после
//! успешного decode, а любой malformed/duplicate/membership отказ очищает
//! route и публикует подтверждённые client/World результаты.
//! Client entry `0x8F702` теперь сам создаёт этот pending route после World
//! `0x5FB01`; duplicate live owner закрывает новый socket. До player decode
//! успешный `0x7F901` выдаёт optional validate `0xBF402` и sequence `0xBF403`,
//! а `0x6FA01` теперь проходит полный reached `OnLost` lifecycle: team/JJC,
//! scripts, nation timing, particular goods/states, immediate либо delayed
//! fight-state departure и region/map cleanup. Reached `OnExit` корректирует
//! silence, публикует позиционный `0xBF504`, исполняет узкий spatial/AI tail и
//! при обычном logout отправляет полный GameSave в World `0x5FB02`.
//! LoginServer kick
//! `0x7F903` полностью различает live, pending, orphan-region и missing player:
//! публикует `GS0041`, transport close либо World `0x5FB02` и очищает route.

use crate::gameserver::appserver::player::PlayerGameSaveCodecError;
use crate::gameserver::appserver::player::{PlayerLostDelayStarted, PlayerParticularGoodsDrop};
use crate::gameserver::appserver::serverregion::RegionMembershipBlock;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerExitReport, GamePlayerLoginBlock,
    GamePlayerLoginPreludeError, GamePlayerLoginPreludeReport, GamePlayerLoginReport,
    GroundGoodsMoveBlock, GroundGoodsMoveReport, colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;

const PLAYER_LOGIN: u32 = 0x0007_f901;
const PLAYER_KICK: u32 = 0x0007_f903;
const FRIEND_ONLINE: u32 = 0x0007_f904;
const FRIEND_OFFLINE: u32 = 0x0007_f905;
const CLIENT_ENTER: u32 = 0x0008_f702;
const PLAYER_LOST: u32 = 0x0006_fa01;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameLogMessageOutcome {
    Forwarded,
    ClientEnterRequested,
    DuplicateClientEnterRejected,
    LivePlayerKicked,
    PendingPlayerKicked,
    OrphanRegionPlayerKicked,
    MissingPlayerConfirmed,
    PlayerLogin,
    PlayerLoginRejected,
    IgnoredStatus,
    PlayerLost,
}

pub(crate) trait GamePlayerLostRuntime {
    fn detach_player_from_team_on_lost(&mut self, game: &mut CGame, player_id: i32) -> bool;
    fn quit_player_jjc_on_lost(&mut self, game: &mut CGame, player_id: i32) -> bool;
    /// Исполняет оставшийся polymorphic spatial/AI tail `OnExit` между
    /// owned around-publication и World-save.
    fn player_on_exit_spatial_tail(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        changing_server: bool,
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerLostParticularGoodsDrop {
    pub(crate) source: PlayerParticularGoodsDrop,
    pub(crate) result: Result<GroundGoodsMoveReport, GroundGoodsMoveBlock>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerLostDisposition {
    PendingLoginCleared,
    OrphanRegionEntry,
    Missing,
    Delayed,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerLostReport {
    pub(crate) player_id: i32,
    pub(crate) disposition: GamePlayerLostDisposition,
    pub(crate) changing_server: bool,
    pub(crate) changing_region: bool,
    pub(crate) scripts_removed: usize,
    pub(crate) team_detached: bool,
    pub(crate) jjc_quit: bool,
    pub(crate) nation_timing_finished: bool,
    pub(crate) particular_goods: Vec<GamePlayerLostParticularGoodsDrop>,
    pub(crate) change_body_states_ended: usize,
    pub(crate) exit: Option<GamePlayerExitReport>,
    pub(crate) delay: Option<PlayerLostDelayStarted>,
    pub(crate) departure: Option<Result<(), RegionMembershipBlock>>,
    pub(crate) route_command: Option<i32>,
}

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

#[must_use = "log-message report сохраняет presence forwarding"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameLogMessageReport {
    pub(crate) source_type: u32,
    pub(crate) outcome: GameLogMessageOutcome,
    pub(crate) client_type: u32,
    pub(crate) player_id: i32,
    pub(crate) delivery: i32,
    pub(crate) login_status: Option<i32>,
    pub(crate) decoded_bytes: usize,
    pub(crate) route_command: Option<i32>,
    pub(crate) login_prelude: Option<GamePlayerLoginPreludeReport>,
    pub(crate) login: Option<GamePlayerLoginReport>,
    pub(crate) lost: Option<GamePlayerLostReport>,
}

pub(crate) fn dispatch_game_log_message<Runtime: GameMainLoopRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameLogMessageReport, GameLogMessageError>> {
    let source_type = message.message_type() as u32;
    if source_type == PLAYER_LOST {
        let player_id = message
            .base_mut()
            .get_long()
            .ok_or(GameLogMessageError::MissingPlayerId);
        return Some(player_id.map(|player_id| {
            let lost = game.on_player_lost(player_id, runtime);
            GameLogMessageReport {
                source_type,
                outcome: GameLogMessageOutcome::PlayerLost,
                client_type: 0,
                player_id,
                delivery: 0,
                login_status: None,
                decoded_bytes: 0,
                route_command: lost.route_command,
                login_prelude: None,
                login: None,
                lost: Some(lost),
            }
        }));
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
    Some(Ok(GameLogMessageReport {
        source_type,
        outcome: GameLogMessageOutcome::Forwarded,
        client_type,
        player_id,
        delivery,
        login_status: None,
        decoded_bytes: 0,
        route_command: None,
        login_prelude: None,
        login: None,
        lost: None,
    }))
}

fn dispatch_client_enter(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<GameLogMessageReport, GameLogMessageError> {
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
        return Ok(GameLogMessageReport {
            source_type: CLIENT_ENTER,
            outcome: GameLogMessageOutcome::DuplicateClientEnterRejected,
            client_type: 0,
            player_id,
            delivery: 0,
            login_status: None,
            decoded_bytes: 0,
            route_command: Some(route_command),
            login_prelude: None,
            login: None,
            lost: None,
        });
    }

    message.base_mut().set_message_type(0x0005_fb01);
    let delivery = message.send(game, false).unwrap_or_default();
    let route_command = game
        .net_server()
        .command_handle()
        .set_client_map_id(socket_id, player_id);
    Ok(GameLogMessageReport {
        source_type: CLIENT_ENTER,
        outcome: GameLogMessageOutcome::ClientEnterRequested,
        client_type: 0,
        player_id,
        delivery,
        login_status: None,
        decoded_bytes: 0,
        route_command: Some(route_command),
        login_prelude: None,
        login: None,
        lost: None,
    })
}

fn dispatch_player_kick(
    message: &mut CMessage,
    game: &mut CGame,
) -> Result<GameLogMessageReport, GameLogMessageError> {
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
        return Ok(kick_report(
            player_id,
            GameLogMessageOutcome::LivePlayerKicked,
            delivery,
            Some(kick.command_result),
        ));
    }

    if game.net_server().has_player_map_id(player_id) {
        let delivery = publish_login_kick_confirmation(game, player_id);
        let (_, route_command) = game.discard_player_login(player_id);
        return Ok(kick_report(
            player_id,
            GameLogMessageOutcome::PendingPlayerKicked,
            delivery,
            Some(route_command),
        ));
    }

    if game.player_registered_in_region(player_id) {
        let kick = game.kick_player(player_id);
        return Ok(kick_report(
            player_id,
            GameLogMessageOutcome::OrphanRegionPlayerKicked,
            0,
            Some(kick.command_result),
        ));
    }

    Ok(kick_report(
        player_id,
        GameLogMessageOutcome::MissingPlayerConfirmed,
        publish_login_kick_confirmation(game, player_id),
        None,
    ))
}

fn publish_login_kick_confirmation(game: &CGame, player_id: i32) -> i32 {
    let mut confirmation = CMessage::new(0x0005_fb02);
    confirmation.add_long(player_id);
    confirmation.add_long(0);
    confirmation.send(game, false).unwrap_or_default()
}

fn kick_report(
    player_id: i32,
    outcome: GameLogMessageOutcome,
    delivery: i32,
    route_command: Option<i32>,
) -> GameLogMessageReport {
    GameLogMessageReport {
        source_type: PLAYER_KICK,
        outcome,
        client_type: 0,
        player_id,
        delivery,
        login_status: None,
        decoded_bytes: 0,
        route_command,
        login_prelude: None,
        login: None,
        lost: None,
    }
}

fn dispatch_player_login<Runtime: GameMainLoopRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Result<GameLogMessageReport, GameLogMessageError> {
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
        return Ok(GameLogMessageReport {
            source_type: PLAYER_LOGIN,
            outcome: GameLogMessageOutcome::PlayerLoginRejected,
            client_type: 0x000b_f401,
            player_id,
            delivery,
            login_status: Some(status),
            decoded_bytes: 0,
            route_command: None,
            login_prelude: None,
            login: None,
            lost: None,
        });
    }
    if status == -2 {
        return Ok(GameLogMessageReport {
            source_type: PLAYER_LOGIN,
            outcome: GameLogMessageOutcome::IgnoredStatus,
            client_type: 0,
            player_id: 0,
            delivery: 0,
            login_status: Some(status),
            decoded_bytes: 0,
            route_command: None,
            login_prelude: None,
            login: None,
            lost: None,
        });
    }

    let player_id = status;
    if !game.net_server().has_player_map_id(player_id) {
        let _delivery = reject_player_login(game, player_id, true);
        return Err(GameLogMessageError::MissingClientRoute { player_id });
    }
    let now_ms = runtime.get_tick_ms();
    let login_prelude = game
        .begin_player_login_validation(player_id, now_ms, runtime.wall_time_seconds())
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
    let login = game
        .complete_world_player_login(player_id, player, captain, team_id, runtime)
        .map_err(|block| {
            let _delivery = reject_player_login(game, player_id, true);
            GameLogMessageError::PlayerLogin(block)
        })?;
    Ok(GameLogMessageReport {
        source_type: PLAYER_LOGIN,
        outcome: GameLogMessageOutcome::PlayerLogin,
        client_type: 0x000b_f401,
        player_id,
        delivery: login.client_deliveries.first().copied().unwrap_or_default(),
        login_status: Some(status),
        decoded_bytes,
        route_command: None,
        login_prelude: Some(login_prelude),
        login: Some(login),
        lost: None,
    })
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
