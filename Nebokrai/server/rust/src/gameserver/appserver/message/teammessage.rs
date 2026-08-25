//! Team-message owner GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/teammessage.cpp`. Материализован полный достигнутый
//! recruitment-state selector `0x8FF08` и связная local team chain
//! `0x8FF01..07`: invite/answer/password join, leave, leader transfer, kick и
//! disband; typed `CTeam/CTeamate`, player membership, `0xBFD01/02/03/05/07`,
//! recruitment count, World session messages и conditional `0x60209` audit.
//! Остальные selectors и remote reconstruction сохранены ниже как RAW.

use crate::gameserver::appserver::teamstate::CTeamState;
use crate::gameserver::gameserver::game::{
    CGame, GameTeamJoinMutation, GameTeamJoinResult, GameTeamLifecycleMutation,
    colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;

const SET_RECRUITMENT_STATE: u32 = 0x0008_ff08;
const INVITE_BY_ID: u32 = 0x0008_ff01;
const ANSWER_INVITE: u32 = 0x0008_ff02;
const JOIN_RECRUITMENT: u32 = 0x0008_ff03;
const LEAVE_TEAM: u32 = 0x0008_ff04;
const SET_LEADER: u32 = 0x0008_ff05;
const KICK_PLAYER: u32 = 0x0008_ff06;
const DISBAND_TEAM: u32 = 0x0008_ff07;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameTeamMessageError {
    MissingEnabled,
    MissingLeaderId,
    MissingCandidateId,
    MissingInvitationResult,
    MissingTargetPlayerId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameTeamMessageOutcome {
    MissingPlayer,
    AlreadyDisabled,
    Disabled,
    RejectedByWordsFilter,
    Enabled,
    InvitationSent,
    InvitationRejected,
    TeamJoined,
    JoinRejected,
    RecruitmentPasswordRejected,
    TeamLeft,
    LeaderChanged,
    PlayerKicked,
    TeamDisbanded,
    LifecycleIgnored,
}

#[must_use = "team-message report сохраняет state и client effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameTeamMessageReport {
    pub(crate) player_id: Option<i32>,
    pub(crate) outcome: GameTeamMessageOutcome,
    pub(crate) state_count: usize,
    pub(crate) delivery:
        Option<Result<i32, crate::gameserver::appserver::shape::ShapeCoordinateBlock>>,
    pub(crate) direct_deliveries: Vec<i32>,
    pub(crate) notification_deliveries: Vec<i32>,
    pub(crate) join_result: Option<GameTeamJoinResult>,
    pub(crate) mutation: Option<GameTeamJoinMutation>,
    pub(crate) lifecycle: Option<GameTeamLifecycleMutation>,
}

fn empty_report(player_id: Option<i32>) -> GameTeamMessageReport {
    GameTeamMessageReport {
        player_id,
        outcome: GameTeamMessageOutcome::MissingPlayer,
        state_count: 0,
        delivery: None,
        direct_deliveries: Vec::new(),
        notification_deliveries: Vec::new(),
        join_result: None,
        mutation: None,
        lifecycle: None,
    }
}

fn notify(game: &CGame, report: &mut GameTeamMessageReport, player_id: i32, string_id: &[u8]) {
    if game.find_player(player_id).is_some() {
        report.notification_deliveries.push(
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(string_id))
                .send_to_player(game.net_server(), player_id),
        );
    }
}

fn notify_invite_error(
    game: &CGame,
    report: &mut GameTeamMessageReport,
    player_id: i32,
    result: GameTeamJoinResult,
) {
    let string_id = match result {
        GameTeamJoinResult::NoPermission => b"GS0091".as_slice(),
        GameTeamJoinResult::MaxMemberLimit => b"GS0092".as_slice(),
        GameTeamJoinResult::PlayerAlreadyInTeam => b"GS0093".as_slice(),
        GameTeamJoinResult::SamePlayer => b"GS0094".as_slice(),
        _ => b"GS0095".as_slice(),
    };
    notify(game, report, player_id, string_id);
}

pub(crate) fn dispatch_game_team_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<GameTeamMessageReport, GameTeamMessageError>> {
    match message.message_type() as u32 {
        INVITE_BY_ID => {
            let Some(leader_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingLeaderId));
            };
            let Some(candidate_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingCandidateId));
            };
            let mut report = empty_report(Some(leader_id));
            let result = game.check_team_join(leader_id, candidate_id);
            report.join_result = Some(result);
            if result == GameTeamJoinResult::Succeed {
                if game.find_player(candidate_id).is_some() {
                    let mut invite = CMessage::new(0x000b_fd01);
                    invite.add_long(candidate_id);
                    invite.add_long(leader_id);
                    report
                        .direct_deliveries
                        .push(invite.send_to_player(game.net_server(), candidate_id));
                }
                report.outcome = GameTeamMessageOutcome::InvitationSent;
            } else {
                notify_invite_error(game, &mut report, leader_id, result);
                report.outcome = GameTeamMessageOutcome::JoinRejected;
            }
            return Some(Ok(report));
        }
        ANSWER_INVITE => {
            let Some(candidate_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingCandidateId));
            };
            let Some(leader_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingLeaderId));
            };
            let Some(accepted) = message.base_mut().get_char() else {
                return Some(Err(GameTeamMessageError::MissingInvitationResult));
            };
            let mut report = empty_report(Some(candidate_id));
            if accepted != 1 {
                if game.find_player(leader_id).is_some() {
                    let mut answer = CMessage::new(0x000b_fd02);
                    answer.add_long(candidate_id);
                    answer.add_long(leader_id);
                    answer.add_byte(accepted as u8);
                    report
                        .direct_deliveries
                        .push(answer.send_to_player(game.net_server(), leader_id));
                }
                report.outcome = GameTeamMessageOutcome::InvitationRejected;
                return Some(Ok(report));
            }
            let result = game.check_team_join(leader_id, candidate_id);
            report.join_result = Some(result);
            if result == GameTeamJoinResult::Succeed {
                report.mutation = game.join_team_accepted(leader_id, candidate_id, true);
                report.outcome = if report.mutation.is_some() {
                    GameTeamMessageOutcome::TeamJoined
                } else {
                    GameTeamMessageOutcome::JoinRejected
                };
            } else {
                match result {
                    GameTeamJoinResult::NoPermission => {
                        notify(game, &mut report, leader_id, b"GS0091");
                        notify(game, &mut report, candidate_id, b"GS0097");
                    }
                    GameTeamJoinResult::MaxMemberLimit => {
                        notify(game, &mut report, leader_id, b"GS0092");
                        notify(game, &mut report, candidate_id, b"GS0092");
                    }
                    GameTeamJoinResult::PlayerAlreadyInTeam => {
                        notify(game, &mut report, leader_id, b"GS0093");
                        notify(game, &mut report, candidate_id, b"GS0098");
                    }
                    GameTeamJoinResult::SamePlayer => {
                        notify(game, &mut report, leader_id, b"GS0094");
                        notify(game, &mut report, candidate_id, b"GS0099");
                    }
                    _ => notify(game, &mut report, leader_id, b"GS0095"),
                }
                report.outcome = GameTeamMessageOutcome::JoinRejected;
            }
            return Some(Ok(report));
        }
        JOIN_RECRUITMENT => {
            let Some(leader_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingLeaderId));
            };
            message.resolve_player_context(game);
            let candidate_id = message.player_id();
            let password = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
            let mut report = empty_report(candidate_id);
            let Some(candidate_id) = candidate_id else {
                return Some(Ok(report));
            };
            let password_matches = game
                .find_player(leader_id)
                .and_then(|leader| leader.first_team_recruitment_state())
                .is_some_and(|state| state.team_password() == password);
            if !password_matches {
                notify(game, &mut report, candidate_id, b"GS0102");
                report.outcome = GameTeamMessageOutcome::RecruitmentPasswordRejected;
                return Some(Ok(report));
            }
            let result = game.check_team_join(leader_id, candidate_id);
            report.join_result = Some(result);
            if result == GameTeamJoinResult::Succeed {
                report.mutation = game.join_team_accepted(leader_id, candidate_id, false);
                report.outcome = if report.mutation.is_some() {
                    GameTeamMessageOutcome::TeamJoined
                } else {
                    GameTeamMessageOutcome::JoinRejected
                };
            } else {
                match result {
                    GameTeamJoinResult::MaxMemberLimit => {
                        notify(game, &mut report, candidate_id, b"GS0100")
                    }
                    GameTeamJoinResult::PlayerAlreadyInTeam => {
                        notify(game, &mut report, candidate_id, b"GS0101")
                    }
                    _ => {}
                }
                report.outcome = GameTeamMessageOutcome::JoinRejected;
            }
            return Some(Ok(report));
        }
        LEAVE_TEAM => {
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let mut report = empty_report(player_id);
            report.lifecycle = player_id.and_then(|id| game.leave_team(id));
            report.outcome = if report.lifecycle.is_some() {
                GameTeamMessageOutcome::TeamLeft
            } else {
                GameTeamMessageOutcome::LifecycleIgnored
            };
            return Some(Ok(report));
        }
        SET_LEADER | KICK_PLAYER => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingTargetPlayerId));
            };
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let mut report = empty_report(player_id);
            report.lifecycle = player_id.and_then(|actor_id| {
                if message.message_type() as u32 == SET_LEADER {
                    game.change_team_leader(actor_id, target_id)
                } else {
                    game.kick_team_member(actor_id, target_id)
                }
            });
            report.outcome = if report.lifecycle.is_some() {
                if message.message_type() as u32 == SET_LEADER {
                    GameTeamMessageOutcome::LeaderChanged
                } else {
                    GameTeamMessageOutcome::PlayerKicked
                }
            } else {
                GameTeamMessageOutcome::LifecycleIgnored
            };
            return Some(Ok(report));
        }
        DISBAND_TEAM => {
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let mut report = empty_report(player_id);
            report.lifecycle = player_id.and_then(|id| game.disband_team(id));
            report.outcome = if report.lifecycle.is_some() {
                GameTeamMessageOutcome::TeamDisbanded
            } else {
                GameTeamMessageOutcome::LifecycleIgnored
            };
            return Some(Ok(report));
        }
        SET_RECRUITMENT_STATE => {}
        _ => return None,
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let mut report = empty_report(player_id);
    let Some(player_id) = player_id else {
        return Some(Ok(report));
    };
    let Some(enabled) = message.base_mut().get_long() else {
        return Some(Err(GameTeamMessageError::MissingEnabled));
    };
    if enabled == 0 {
        let state = game
            .find_player(player_id)
            .and_then(|player| player.first_team_recruitment_state())
            .cloned();
        let Some(state) = state else {
            report.outcome = GameTeamMessageOutcome::AlreadyDisabled;
            return Some(Ok(report));
        };
        let mut ended = CMessage::new(0x000b_fe04);
        ended.add_long(400);
        ended.add_long(player_id);
        ended.add_long(state.state_id());
        report.delivery = game.send_player_shape_around(player_id, None, &ended);
        let _ended = game
            .find_player_mut(player_id)
            .and_then(|player| player.end_first_team_recruitment_state());
        report.state_count = game
            .find_player(player_id)
            .map(|player| player.team_recruitment_state_count())
            .unwrap_or_default();
        report.outcome = GameTeamMessageOutcome::Disabled;
        return Some(Ok(report));
    }

    let mut team_name = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
    let team_password = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
    if !game
        .words_filter()
        .check_with_numeric_gate(&mut team_name, false, true)
    {
        report.notification_deliveries.push(
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0333"))
                .send_to_player(game.net_server(), player_id),
        );
        report.outcome = GameTeamMessageOutcome::RejectedByWordsFilter;
        return Some(Ok(report));
    }

    let state = CTeamState::new(team_name, team_password);
    let mut begun = CMessage::new(0x000b_fe03);
    begun.add_long(400);
    begun.add_long(player_id);
    begun.add_long(state.state_id());
    begun.add_long(state.client_state_time());
    begun.add_ulong(state.initial_additional_data());
    begun.base_mut().add(state.team_name());
    begun.add_byte(0);
    report.delivery = game.send_player_shape_around(player_id, None, &begun);
    game.find_player_mut(player_id)
        .expect("team message player разрешён до state attach")
        .attach_team_recruitment_state(state);
    report.state_count = game
        .find_player(player_id)
        .map(|player| player.team_recruitment_state_count())
        .unwrap_or_default();
    report.outcome = GameTeamMessageOutcome::Enabled;
    Some(Ok(report))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\teammessage.cpp

// ============================================================================
// FUNCTION: JoinTeam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\teammessage.cpp:61
// RVA: 0x0008BEF0
// ADDRESS: 0048bef0
// PROTOTYPE: tagCreateTeamResult __cdecl JoinTeam(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: NewTeamMemberJoin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\teammessage.cpp:152
// RVA: 0x0008C090
// ADDRESS: 0048c090
// PROTOTYPE: void __cdecl NewTeamMemberJoin(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CreateNewTeam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\teammessage.cpp:192
// RVA: 0x0008C250
// ADDRESS: 0048c250
// PROTOTYPE: void __cdecl CreateNewTeam(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: OnTeamMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\teammessage.cpp:249
// RVA: 0x0008C4C0
// ADDRESS: 0048c4c0
// PROTOTYPE: void __cdecl OnTeamMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
