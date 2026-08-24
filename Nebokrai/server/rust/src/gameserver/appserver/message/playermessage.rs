//! Player-message dispatcher GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/playermessage.cpp`. Материализован friend lifecycle
//! `0x8FA0D..0x8FA0F`: byte-exact names, 40-entry ordered state, reciprocal
//! accept mutation, addressed `0xBF719/71A/71B` и WorldServer
//! `0x60501/0x60502`. Public identity `0x8FA11/17/18` замыкает headpiece
//! state/around publication, honor-country-appellation snapshot и attempt ID
//! до server-trusted change-appellation script boundary. Client timing
//! `0x8FA12/13/1A` замыкает quest countdown, heartbeat acknowledgement и exact
//! 16-byte Windows `SYSTEMTIME`; wall/local clocks остаются runtime owner-ом.
//! Остальные opcode ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::player::PlayerFriendAddOutcome;
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const REQUEST_FRIEND: u32 = 0x0008_fa0d;
const ANSWER_FRIEND: u32 = 0x0008_fa0e;
const DELETE_FRIEND: u32 = 0x0008_fa0f;
const SET_DISPLAY_HEAD_PIECE: u32 = 0x0008_fa11;
const QUERY_QUEST_TIME: u32 = 0x0008_fa12;
const ACKNOWLEDGE_HEARTBEAT: u32 = 0x0008_fa13;
const QUERY_HONOR_IDENTITY: u32 = 0x0008_fa17;
const REQUEST_CHANGE_APPELLATION: u32 = 0x0008_fa18;
const QUERY_LOCAL_TIME: u32 = 0x0008_fa1a;

pub(crate) trait GamePlayerMessageRuntime {
    /// Выполняет concrete `PlayerRunScript` с server-trusted path; VM и
    /// script-data owner ещё не материализованы в `CGame`.
    fn run_change_appellation_script(&mut self, game: &mut CGame, player_id: i32, path: &[u8]);

    /// Возвращает legacy 32-bit `_time` seconds для quest countdown.
    fn player_wall_time_seconds(&mut self) -> i32;

    /// Возвращает поля Windows `SYSTEMTIME` в native field order.
    fn player_local_system_time(&mut self) -> [u16; 8];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageOutcome {
    MissingContext,
    TargetMissing,
    FriendRequested,
    FriendAnswered,
    FriendMissing,
    FriendDeleted,
    DisplayHeadPieceChanged,
    QuestTimeSent,
    HeartbeatAcknowledged,
    HonorIdentitySent,
    AppellationChangeRequested,
    LocalTimeSent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageDelivery {
    Player(i32),
    Around(Option<Result<i32, ShapeCoordinateBlock>>),
    World(Result<i32, SendMessageError>),
}

#[must_use = "player-message report сохраняет friend state и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GamePlayerMessageReport {
    pub(crate) message_type: u32,
    pub(crate) player_id: Option<i32>,
    pub(crate) target_player_id: Option<i32>,
    pub(crate) friend_name: Vec<u8>,
    pub(crate) outcome: GamePlayerMessageOutcome,
    pub(crate) deliveries: Vec<GamePlayerMessageDelivery>,
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(value);
    message.base_mut().add_byte(0);
}

fn publish_friend_add(
    game: &CGame,
    owner_id: i32,
    friend_id: i32,
) -> Result<i32, SendMessageError> {
    let mut message = CMessage::new(0x0006_0501);
    message.add_long(owner_id);
    message.add_long(friend_id);
    message.send(game, false)
}

fn publish_friend_delete(
    game: &CGame,
    owner_id: i32,
    friend_id: i32,
    friend_name: &[u8],
) -> Result<i32, SendMessageError> {
    let mut message = CMessage::new(0x0006_0502);
    message.add_long(owner_id);
    message.add_long(friend_id);
    add_c_string(&mut message, friend_name);
    message.send(game, false)
}

fn apply_friend_add(
    game: &mut CGame,
    owner_id: i32,
    friend_name: &[u8],
    online_friend_id: Option<i32>,
    deliveries: &mut Vec<GamePlayerMessageDelivery>,
) {
    let Some(outcome) = game
        .find_player_mut(owner_id)
        .map(|player| player.add_friend_state(friend_name))
    else {
        return;
    };
    match outcome {
        PlayerFriendAddOutcome::Added => {
            if let Some(friend_id) = online_friend_id {
                deliveries.push(GamePlayerMessageDelivery::World(publish_friend_add(
                    game, owner_id, friend_id,
                )));
            }
        }
        PlayerFriendAddOutcome::AlreadyPresent => {}
        PlayerFriendAddOutcome::LimitReached => {
            let delivery =
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0155"))
                    .send_to_player(game.net_server(), owner_id);
            deliveries.push(GamePlayerMessageDelivery::Player(delivery));
        }
    }
}

pub(crate) fn dispatch_game_player_message<Runtime: GamePlayerMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GamePlayerMessageReport, GamePlayerMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        REQUEST_FRIEND
            | ANSWER_FRIEND
            | DELETE_FRIEND
            | SET_DISPLAY_HEAD_PIECE
            | QUERY_QUEST_TIME
            | ACKNOWLEDGE_HEARTBEAT
            | QUERY_HONOR_IDENTITY
            | REQUEST_CHANGE_APPELLATION
            | QUERY_LOCAL_TIME
    ) {
        return None;
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let mut report = GamePlayerMessageReport {
        message_type,
        player_id,
        target_player_id: None,
        friend_name: Vec::new(),
        outcome: GamePlayerMessageOutcome::MissingContext,
        deliveries: Vec::new(),
    };
    let Some(player_id) = player_id else {
        return Some(Ok(report));
    };

    match message_type {
        REQUEST_FRIEND => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "target player id",
                )));
            };
            report.target_player_id = Some(target_id);
            let Some(requester_name) = game
                .find_player(player_id)
                .map(|player| player.player_name().to_vec())
            else {
                return Some(Ok(report));
            };
            if game.find_player(target_id).is_none() {
                report.outcome = GamePlayerMessageOutcome::TargetMissing;
                return Some(Ok(report));
            }
            let mut response = CMessage::new(0x000b_f719);
            add_c_string(&mut response, &requester_name);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), target_id),
            ));
            report.outcome = GamePlayerMessageOutcome::FriendRequested;
        }
        ANSWER_FRIEND => {
            let friend_name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
            let Some(accepted) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField("friend answer")));
            };
            report.friend_name.clone_from(&friend_name);
            let target_id = game
                .find_player_by_name(&friend_name)
                .map(|player| player.player_id());
            report.target_player_id = target_id;
            if accepted == 1 {
                apply_friend_add(
                    game,
                    player_id,
                    &friend_name,
                    target_id,
                    &mut report.deliveries,
                );
            }
            let Some(target_id) = target_id else {
                report.outcome = GamePlayerMessageOutcome::TargetMissing;
                return Some(Ok(report));
            };
            let requester_name = game
                .find_player(player_id)
                .expect("friend answer requester сохранён после context lookup")
                .player_name()
                .to_vec();
            let target_name = game
                .find_player(target_id)
                .expect("friend answer target сохранён после name lookup")
                .player_name()
                .to_vec();
            if accepted == 1 {
                apply_friend_add(
                    game,
                    target_id,
                    &requester_name,
                    Some(player_id),
                    &mut report.deliveries,
                );
            }
            let mut to_target = CMessage::new(0x000b_f71a);
            add_c_string(&mut to_target, &requester_name);
            to_target.base_mut().add_byte(accepted as u8);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                to_target.send_to_player(game.net_server(), target_id),
            ));
            let mut to_requester = CMessage::new(0x000b_f71a);
            add_c_string(&mut to_requester, &target_name);
            to_requester.base_mut().add_byte(accepted as u8);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                to_requester.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::FriendAnswered;
        }
        DELETE_FRIEND => {
            let friend_name = message.base_mut().get_str_bytes(0x32).unwrap_or_default();
            report.friend_name.clone_from(&friend_name);
            let online_friend_id = game
                .find_player_by_name(&friend_name)
                .map(|player| player.player_id());
            report.target_player_id = online_friend_id;
            let exists = game
                .find_player(player_id)
                .is_some_and(|player| player.has_friend(&friend_name));
            if !exists {
                report.outcome = GamePlayerMessageOutcome::FriendMissing;
                return Some(Ok(report));
            }
            report
                .deliveries
                .push(GamePlayerMessageDelivery::World(publish_friend_delete(
                    game,
                    player_id,
                    online_friend_id.unwrap_or(0),
                    &friend_name,
                )));
            game.find_player_mut(player_id)
                .expect("friend owner сохранён после existence lookup")
                .delete_friend_state(&friend_name);
            let mut response = CMessage::new(0x000b_f71b);
            add_c_string(&mut response, &friend_name);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::FriendDeleted;
        }
        SET_DISPLAY_HEAD_PIECE => {
            let Some(display) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "display head piece",
                )));
            };
            let display = display != 0;
            game.find_player_mut(player_id)
                .expect("display-head player сохранён после context lookup")
                .set_display_head_piece(display);
            let mut response = CMessage::new(0x000b_f722);
            response.add_long(player_id);
            response.base_mut().add_byte(u8::from(display));
            report.deliveries.push(GamePlayerMessageDelivery::Around(
                game.send_player_shape_around(player_id, Some(player_id), &response),
            ));
            report.outcome = GamePlayerMessageOutcome::DisplayHeadPieceChanged;
        }
        QUERY_QUEST_TIME => {
            let remaining = game
                .find_player(player_id)
                .expect("quest-time player сохранён после context lookup")
                .quest_time_remaining(runtime.player_wall_time_seconds());
            let mut response = CMessage::new(0x000b_f72b);
            response.add_long(remaining);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::QuestTimeSent;
        }
        ACKNOWLEDGE_HEARTBEAT => {
            game.find_player_mut(player_id)
                .expect("heartbeat player сохранён после context lookup")
                .acknowledge_heartbeat();
            report.outcome = GamePlayerMessageOutcome::HeartbeatAcknowledged;
        }
        QUERY_HONOR_IDENTITY => {
            let country_identity = game.player_country_identity(player_id);
            let honor = game
                .find_player(player_id)
                .expect("honor player сохранён после context lookup")
                .honor_snapshot();
            let mut response = CMessage::new(0x000b_f737);
            response.base_mut().add_ulong(honor.rank_of_nobility_id);
            response.base_mut().add_ulong(u32::from(country_identity));
            response.base_mut().add_ulong(honor.appellation_id);
            response.base_mut().add_ulong(honor.days_eliminate);
            response.base_mut().add_ulong(honor.weeks_eliminate);
            response.base_mut().add_ulong(honor.months_eliminate);
            response.base_mut().add_ulong(honor.total_eliminate);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::HonorIdentitySent;
        }
        REQUEST_CHANGE_APPELLATION => {
            let Some(appellation_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("appellation id")));
            };
            let Some(_legacy_ignored) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "appellation request tail",
                )));
            };
            game.find_player_mut(player_id)
                .expect("appellation player сохранён после context lookup")
                .request_change_appellation_state(appellation_id as u32);
            runtime.run_change_appellation_script(
                game,
                player_id,
                b"scripts/circle/honorrank/changeappellation.script",
            );
            report.outcome = GamePlayerMessageOutcome::AppellationChangeRequested;
        }
        QUERY_LOCAL_TIME => {
            let system_time = runtime.player_local_system_time();
            let mut response = CMessage::new(0x000b_f73f);
            for field in system_time {
                response.base_mut().add(&field.to_le_bytes());
            }
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GamePlayerMessageOutcome::LocalTimeSent;
        }
        _ => unreachable!("friend opcode отфильтрован до decode"),
    }
    Some(Ok(report))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\playermessage.cpp

// ============================================================================
// FUNCTION: CPlayer::OnMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\playermessage.cpp:26
// RVA: 0x000FAAB0
// ADDRESS: 004faab0
// PROTOTYPE: void __thiscall OnMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
