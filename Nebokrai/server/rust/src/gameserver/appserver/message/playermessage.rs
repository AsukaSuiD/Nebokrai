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
//! Общий outer guard сохраняет исходный запрет player-message во время смены
//! сервера/региона; `0x8FA02` вызывает полный reached `CPlayer::OnRelive(0)`
//! через concrete `CGame` relive owner со всеми state/region/wire effects.
//! LeiTing claim `0x8FA19` сохраняет packet-space gate, exact thresholds,
//! `BF73E -> 5FD10 -> reward script` ordering; `0x8FA10` использует тот же
//! server-trusted script runtime для help script.
//! Player trade `0x8FA06/07/0B/0C` замыкает invitation/answer guards,
//! normal session с двумя trader plug-ами, ready toggle, синхронный commit,
//! Billing-pending YuanBao tail и terminal End/Abort публикации.
//! Equipment-state refresh `0x8FA16` сохраняет packed local-time decode,
//! strict grace-minute comparison, addon mutation и around `0xBF928`.
//! Остальные opcode ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_EQUIP_STATE;
use crate::gameserver::appserver::player::PlayerFriendAddOutcome;
use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{
    CGame, GameContainerMessageRuntime, PlayerReliveContext, PlayerReliveReport,
    PlayerTradeAbortReport, PlayerTradeReadyReport, colored_player_notice_message,
};
use crate::nets::netserver::message::{CMessage, SendMessageError};
use crate::public::guid::CGuid;

const REQUEST_RELIVE: u32 = 0x0008_fa02;
const REQUEST_TRADE: u32 = 0x0008_fa06;
const ANSWER_TRADE: u32 = 0x0008_fa07;
const TOGGLE_TRADE_READY: u32 = 0x0008_fa0b;
const ABORT_TRADE: u32 = 0x0008_fa0c;
const RUN_HELP_SCRIPT: u32 = 0x0008_fa10;
const REQUEST_FRIEND: u32 = 0x0008_fa0d;
const ANSWER_FRIEND: u32 = 0x0008_fa0e;
const DELETE_FRIEND: u32 = 0x0008_fa0f;
const SET_DISPLAY_HEAD_PIECE: u32 = 0x0008_fa11;
const QUERY_QUEST_TIME: u32 = 0x0008_fa12;
const ACKNOWLEDGE_HEARTBEAT: u32 = 0x0008_fa13;
const REFRESH_EXPIRED_EQUIPMENT_STATE: u32 = 0x0008_fa16;
const QUERY_HONOR_IDENTITY: u32 = 0x0008_fa17;
const REQUEST_CHANGE_APPELLATION: u32 = 0x0008_fa18;
const QUERY_LOCAL_TIME: u32 = 0x0008_fa1a;
const CLAIM_LEI_TING_REWARD: u32 = 0x0008_fa19;

const LEI_TING_REWARD_SCRIPTS: [&[u8]; 9] = [
    b"scripts/goods/leilifengxing_lingqu_20.script",
    b"scripts/goods/leilifengxing_lingqu_60.script",
    b"scripts/goods/leilifengxing_lingqu_80.script",
    b"scripts/goods/leilifengxing_lingqu_100.script",
    b"scripts/goods/leilifengxing_lingqu_4.script",
    b"scripts/goods/leilifengxing_lingqu_10.script",
    b"scripts/goods/leilifengxing_lingqu_16.script",
    b"scripts/goods/leilifengxing_lingqu_22.script",
    b"scripts/goods/leilifengxing_lingqu_28.script",
];

pub(crate) trait GamePlayerMessageRuntime:
    PlayerReliveContext + GameContainerMessageRuntime
{
    /// Выполняет concrete `PlayerRunScript` с server-trusted path; VM и
    /// script-data owner ещё не материализованы в `CGame`.
    fn run_player_script(&mut self, game: &mut CGame, player_id: i32, path: &[u8]);

    /// Возвращает legacy 32-bit `_time` seconds для quest countdown.
    fn player_wall_time_seconds(&mut self) -> i32;

    /// Возвращает поля Windows `SYSTEMTIME` в native field order.
    fn player_local_system_time(&mut self) -> [u16; 8];

    /// Выполняет exact local `mktime`/`time`/`difftime` для packed equipment
    /// timestamp. `None` соответствует `_mktime == -1`; DST выбирает CRT.
    fn player_elapsed_seconds_from_local_time(
        &mut self,
        local: PlayerPackedLocalTime,
    ) -> Option<f64>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPackedLocalTime {
    pub(crate) year_since_1900: i32,
    pub(crate) zero_based_month: i32,
    pub(crate) day: i32,
    pub(crate) hour: i32,
    pub(crate) minute: i32,
    pub(crate) second: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GamePlayerMessageOutcome {
    MissingContext,
    ChangingLocation,
    TargetMissing,
    Relived,
    PlayerScriptRun,
    TradeRequested,
    TradeAnswered,
    TradeStateChanged,
    TradeAborted,
    FriendRequested,
    FriendAnswered,
    FriendMissing,
    FriendDeleted,
    DisplayHeadPieceChanged,
    QuestTimeSent,
    HeartbeatAcknowledged,
    HonorIdentitySent,
    AppellationChangeRequested,
    ExpiredEquipmentMissing,
    ExpiredEquipmentStateIgnored,
    ExpiredEquipmentTimeInvalid,
    ExpiredEquipmentStillActive,
    ExpiredEquipmentPublished,
    LocalTimeSent,
    LeiTingInvalidReward,
    LeiTingPacketFull,
    LeiTingRewardUnavailable,
    LeiTingRewardClaimed,
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
    pub(crate) lei_ting_reward: Option<u16>,
    pub(crate) equipment_goods_id: Option<CGuid>,
    pub(crate) equipment_source: Option<i8>,
    pub(crate) equipment_source_position: Option<i32>,
    pub(crate) equipment_elapsed_seconds: Option<i32>,
    pub(crate) equipment_state_mutated: Option<bool>,
    pub(crate) outcome: GamePlayerMessageOutcome,
    pub(crate) relive: Option<PlayerReliveReport>,
    pub(crate) trade_session: Option<(i32, i32, i32)>,
    pub(crate) trade_ready: Option<PlayerTradeReadyReport>,
    pub(crate) trade_abort: Option<PlayerTradeAbortReport>,
    pub(crate) deliveries: Vec<GamePlayerMessageDelivery>,
}

fn add_c_string(message: &mut CMessage, value: &[u8]) {
    let value = value.split(|byte| *byte == 0).next().unwrap_or_default();
    message.base_mut().add(value);
    message.base_mut().add_byte(0);
}

fn send_trade_notice(game: &CGame, player_id: i32, string_id: &[u8]) -> i32 {
    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id))
        .send_to_player(game.net_server(), player_id)
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

fn publish_lei_ting_update(
    game: &CGame,
    player_id: i32,
    deliveries: &mut Vec<GamePlayerMessageDelivery>,
) {
    let payload = game
        .find_player(player_id)
        .expect("LeiTing player сохранён после flag mutation")
        .encode_lei_ting();
    let mut client = CMessage::new(0x000b_f73e);
    client.base_mut().add(&payload);
    deliveries.push(GamePlayerMessageDelivery::Player(
        client.send_to_player(game.net_server(), player_id),
    ));
    let mut world = CMessage::new(0x0005_fd10);
    world.add_long(player_id);
    world.base_mut().add(&payload);
    deliveries.push(GamePlayerMessageDelivery::World(world.send(game, false)));
}

fn decode_equipment_state_local_time(packed: i32) -> PlayerPackedLocalTime {
    PlayerPackedLocalTime {
        year_since_1900: packed >> 23,
        zero_based_month: (packed >> 19) & 0x0f,
        day: (packed >> 14) & 0x1f,
        hour: (packed >> 9) & 0x1f,
        minute: (packed >> 3) & 0x3f,
        second: 0,
    }
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
        REQUEST_RELIVE
            | REQUEST_TRADE
            | ANSWER_TRADE
            | TOGGLE_TRADE_READY
            | ABORT_TRADE
            | RUN_HELP_SCRIPT
            | REQUEST_FRIEND
            | ANSWER_FRIEND
            | DELETE_FRIEND
            | SET_DISPLAY_HEAD_PIECE
            | QUERY_QUEST_TIME
            | ACKNOWLEDGE_HEARTBEAT
            | REFRESH_EXPIRED_EQUIPMENT_STATE
            | QUERY_HONOR_IDENTITY
            | REQUEST_CHANGE_APPELLATION
            | QUERY_LOCAL_TIME
            | CLAIM_LEI_TING_REWARD
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
        lei_ting_reward: None,
        equipment_goods_id: None,
        equipment_source: None,
        equipment_source_position: None,
        equipment_elapsed_seconds: None,
        equipment_state_mutated: None,
        outcome: GamePlayerMessageOutcome::MissingContext,
        relive: None,
        trade_session: None,
        trade_ready: None,
        trade_abort: None,
        deliveries: Vec::new(),
    };
    let Some(player_id) = player_id else {
        return Some(Ok(report));
    };
    if game
        .find_player(player_id)
        .is_some_and(|player| player.in_changing_server() || player.in_changing_region())
    {
        report.outcome = GamePlayerMessageOutcome::ChangingLocation;
        return Some(Ok(report));
    }

    match message_type {
        REQUEST_RELIVE => {
            report.relive = Some(game.relive_gods_battle_player(player_id, 0, runtime));
            report.outcome = GamePlayerMessageOutcome::Relived;
        }
        RUN_HELP_SCRIPT => {
            runtime.run_player_script(game, player_id, b"scripts/help/help.script");
            report.outcome = GamePlayerMessageOutcome::PlayerScriptRun;
        }
        REQUEST_TRADE => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade target player id",
                )));
            };
            report.target_player_id = Some(target_id);
            let Some(requester) = game.find_player(player_id) else {
                return Some(Ok(report));
            };
            let notice = if requester.is_dead() {
                Some(b"GS0065".as_slice())
            } else if target_id == player_id {
                Some(b"GS0058".as_slice())
            } else if requester.current_progress() != PlayerProgress::None {
                Some(b"GS0059".as_slice())
            } else {
                match game.find_player(target_id) {
                    None => Some(b"GS0064".as_slice()),
                    Some(target) if target.is_dead() => Some(b"GS0063".as_slice()),
                    Some(target) if target.current_progress() != PlayerProgress::None => {
                        Some(b"GS0062".as_slice())
                    }
                    Some(_)
                        if game
                            .player_trade_distance(player_id, target_id)
                            .unwrap_or(9)
                            >= 9 =>
                    {
                        Some(b"GS0061".as_slice())
                    }
                    Some(_) => None,
                }
            };
            if let Some(string_id) = notice {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                        game, player_id, string_id,
                    )));
                report.outcome = GamePlayerMessageOutcome::TradeRequested;
                return Some(Ok(report));
            }
            let mut invitation = CMessage::new(0x000b_f70f);
            invitation.add_long(player_id);
            report.deliveries.push(GamePlayerMessageDelivery::Player(
                invitation.send_to_player(game.net_server(), target_id),
            ));
            report
                .deliveries
                .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                    game, player_id, b"GS0060",
                )));
            report.outcome = GamePlayerMessageOutcome::TradeRequested;
        }
        ANSWER_TRADE => {
            let Some(inviter_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade inviter player id",
                )));
            };
            let Some(accepted) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField("trade answer")));
            };
            report.target_player_id = Some(inviter_id);
            if inviter_id == player_id {
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            let Some(answerer) = game.find_player(player_id) else {
                return Some(Ok(report));
            };
            if answerer.is_dead() {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                        game, player_id, b"GS0065",
                    )));
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            let inviter_exists = game.find_player(inviter_id).is_some();
            if answerer.current_progress() != PlayerProgress::None {
                for string_id in [b"GS0068".as_slice(), b"GS0071".as_slice()]
                    .into_iter()
                    .take(if inviter_exists { 2 } else { 1 })
                {
                    report
                        .deliveries
                        .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                            game, player_id, string_id,
                        )));
                }
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            let Some(inviter) = game.find_player(inviter_id) else {
                report
                    .deliveries
                    .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                        game, player_id, b"GS0070",
                    )));
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            };
            let notices: &[&[u8]] = if inviter.is_dead() {
                &[b"GS0069", b"GS0065"]
            } else if inviter.current_progress() != PlayerProgress::None {
                &[b"GS0067", b"GS0068"]
            } else if game
                .player_trade_distance(player_id, inviter_id)
                .unwrap_or(9)
                >= 9
            {
                &[b"GS0061", b"GS0061"]
            } else if accepted == 0 {
                &[b"GS0066"]
            } else {
                &[]
            };
            if !notices.is_empty() {
                for string_id in notices {
                    report
                        .deliveries
                        .push(GamePlayerMessageDelivery::Player(send_trade_notice(
                            game, player_id, string_id,
                        )));
                }
                report.outcome = GamePlayerMessageOutcome::TradeAnswered;
                return Some(Ok(report));
            }
            game.find_player_mut(player_id)
                .expect("trade answerer проверен")
                .set_current_progress_snapshot(PlayerProgress::Trading);
            game.find_player_mut(inviter_id)
                .expect("trade inviter проверен")
                .set_current_progress_snapshot(PlayerProgress::Trading);
            let session = game.create_player_trade_session(inviter_id, player_id);
            report.trade_session = session;
            if let Some((session_id, inviter_plug_id, answerer_plug_id)) = session {
                let mut opened = CMessage::new(0x000b_f710);
                opened.add_long(session_id);
                opened.add_long(inviter_id);
                opened.add_long(inviter_plug_id);
                opened.add_long(player_id);
                opened.add_long(answerer_plug_id);
                for owner_id in [inviter_id, player_id] {
                    report.deliveries.push(GamePlayerMessageDelivery::Player(
                        opened.send_to_player(game.net_server(), owner_id),
                    ));
                }
            }
            report.outcome = GamePlayerMessageOutcome::TradeAnswered;
        }
        TOGGLE_TRADE_READY => {
            let Some(session_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade session id",
                )));
            };
            let Some(plug_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("trade plug id")));
            };
            report.trade_ready =
                Some(game.toggle_player_trade_ready(player_id, session_id, plug_id, runtime));
            report.outcome = GamePlayerMessageOutcome::TradeStateChanged;
        }
        ABORT_TRADE => {
            let Some(session_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "trade session id",
                )));
            };
            let Some(plug_id) = message.base_mut().get_long() else {
                return Some(Err(GamePlayerMessageError::MissingField("trade plug id")));
            };
            report.trade_abort = Some(game.abort_player_trade(player_id, session_id, plug_id));
            report.outcome = GamePlayerMessageOutcome::TradeAborted;
        }
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
        REFRESH_EXPIRED_EQUIPMENT_STATE => {
            let Some(goods_id) = message.base_mut().get_guid() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "expired equipment guid",
                )));
            };
            report.equipment_goods_id = Some(goods_id);
            let Some((state, packed_time)) = game.find_player(player_id).and_then(|player| {
                player.get_goods_by_id(goods_id).map(|goods| {
                    (
                        goods.addon_property_value(game.goods_factory(), GAP_EQUIP_STATE, 1),
                        goods.addon_property_value(game.goods_factory(), GAP_EQUIP_STATE, 2),
                    )
                })
            }) else {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentMissing;
                return Some(Ok(report));
            };
            if state != 2 || packed_time == 0 {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentStateIgnored;
                return Some(Ok(report));
            }
            let Some(source) = message.base_mut().get_char() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "expired equipment source",
                )));
            };
            report.equipment_source = Some(source);
            report.equipment_source_position = game.find_player(player_id).map(|player| {
                if source != 0 {
                    return -1;
                }
                player
                    .packet()
                    .query_goods_position(goods_id)
                    .map(|position| position as i32)
                    .unwrap_or_else(|| {
                        player
                            .equipment()
                            .query_goods_position_by_id(goods_id)
                            .map(|position| position.position())
                            .unwrap_or(u32::MAX)
                            .wrapping_add(2) as i32
                    })
            });
            let Some(elapsed_seconds) = runtime.player_elapsed_seconds_from_local_time(
                decode_equipment_state_local_time(packed_time),
            ) else {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentTimeInvalid;
                return Some(Ok(report));
            };
            let elapsed_seconds = elapsed_seconds.round() as i32;
            report.equipment_elapsed_seconds = Some(elapsed_seconds);
            if elapsed_seconds / 60 <= 0x275f {
                report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentStillActive;
                return Some(Ok(report));
            }
            let (identity, payload, mutated) = {
                let goods = game
                    .find_player_mut(player_id)
                    .expect("expired-equipment player сохранён после context lookup")
                    .get_goods_by_id_mut(goods_id)
                    .expect("expired equipment сохранён после addon validation");
                let mutated = goods.set_addon_property_modifier_core(GAP_EQUIP_STATE, 1, 3);
                (
                    goods.identity(),
                    runtime.encode_goods_for_old_client(goods),
                    mutated,
                )
            };
            report.equipment_state_mutated = Some(mutated);
            let mut response = CMessage::new(0x000b_f928);
            response.add_long(player_id);
            response.base_mut().add_guid(identity.ex_id);
            response.base_mut().add_ulong(payload.len() as u32);
            response.base_mut().add(&payload);
            report.deliveries.push(GamePlayerMessageDelivery::Around(
                game.send_player_shape_around(player_id, None, &response),
            ));
            report.outcome = GamePlayerMessageOutcome::ExpiredEquipmentPublished;
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
            runtime.run_player_script(
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
        CLAIM_LEI_TING_REWARD => {
            let Some(reward) = message.base_mut().get_word() else {
                return Some(Err(GamePlayerMessageError::MissingField(
                    "LeiTing reward index",
                )));
            };
            report.lei_ting_reward = Some(reward);
            let Some(script) = LEI_TING_REWARD_SCRIPTS.get(usize::from(reward)) else {
                report.outcome = GamePlayerMessageOutcome::LeiTingInvalidReward;
                return Some(Ok(report));
            };
            if !game
                .find_player(player_id)
                .expect("LeiTing player сохранён после context lookup")
                .packet()
                .check_space(3)
            {
                let notice =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"E19681"));
                report.deliveries.push(GamePlayerMessageDelivery::Player(
                    notice.send_to_player(game.net_server(), player_id),
                ));
                report.outcome = GamePlayerMessageOutcome::LeiTingPacketFull;
                return Some(Ok(report));
            }
            if !game
                .find_player_mut(player_id)
                .expect("LeiTing player сохранён после packet-space lookup")
                .change_fy_energy_flag(reward)
            {
                report.outcome = GamePlayerMessageOutcome::LeiTingRewardUnavailable;
                return Some(Ok(report));
            }
            publish_lei_ting_update(game, player_id, &mut report.deliveries);
            runtime.run_player_script(game, player_id, script);
            report.outcome = GamePlayerMessageOutcome::LeiTingRewardClaimed;
        }
        _ => unreachable!("player opcode отфильтрован до decode"),
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
