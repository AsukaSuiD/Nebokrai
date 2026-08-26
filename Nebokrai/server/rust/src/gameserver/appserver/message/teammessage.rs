//! Владелец команд GameServer для групп игроков.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/message/teammessage.cpp`. Реализованы клиентские команды
//! `0x8FF01..0B`, репликация World→Game `0x7FD02..0C`, состояния `CTeam` и
//! `CTeamate`, ограничения частоты чата и аудит `0x60209`. Изменения группы и
//! отправка сообщений выполняются сразу в исходном порядке; подробности
//! завершённых действий публикуются через `tracing`, а не возвращаются
//! вызывающей стороне в накопительном отчёте.

use crate::gameserver::appserver::session::csessionfactory::{
    TeamMemberSnapshot, TeamSessionSnapshot,
};
use crate::gameserver::appserver::teamstate::CTeamState;
use crate::gameserver::gameserver::game::{
    CGame, GameTeamChatResult, GameTeamJoinResult, colored_player_notice_message,
    game_tick_milliseconds,
};
use crate::nets::netserver::message::CMessage;
use tracing::trace;

const SET_RECRUITMENT_STATE: u32 = 0x0008_ff08;
const INVITE_BY_ID: u32 = 0x0008_ff01;
const ANSWER_INVITE: u32 = 0x0008_ff02;
const JOIN_RECRUITMENT: u32 = 0x0008_ff03;
const LEAVE_TEAM: u32 = 0x0008_ff04;
const SET_LEADER: u32 = 0x0008_ff05;
const KICK_PLAYER: u32 = 0x0008_ff06;
const DISBAND_TEAM: u32 = 0x0008_ff07;
const SET_ALLOCATION_SCHEME: u32 = 0x0008_ff09;
const TEAM_CHAT: u32 = 0x0008_ff0a;
const INVITE_BY_NAME: u32 = 0x0008_ff0b;
const WORLD_ABORT_TEAM: u32 = 0x0007_fd02;
const WORLD_INSERT_MEMBER: u32 = 0x0007_fd03;
const WORLD_REMOVE_MEMBER: u32 = 0x0007_fd04;
const WORLD_CHANGE_REGION: u32 = 0x0007_fd05;
const WORLD_KICK_PLAYER: u32 = 0x0007_fd06;
const WORLD_CHANGE_LEADER: u32 = 0x0007_fd07;
const WORLD_TEAM_SNAPSHOT: u32 = 0x0007_fd08;
const WORLD_ONLINE_QUERY: u32 = 0x0007_fd09;
const WORLD_ALLOCATION_SCHEME: u32 = 0x0007_fd0a;
const WORLD_TEAM_CHAT: u32 = 0x0007_fd0b;
const WORLD_MEMBER_STATE: u32 = 0x0007_fd0c;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameTeamMessageError {
    MissingEnabled,
    MissingLeaderId,
    MissingCandidateId,
    MissingInvitationResult,
    MissingTargetPlayerId,
    MissingAllocationScheme,
    MissingChatText,
    MissingInvitedName,
    MissingRemoteField,
    InvalidRemoteTeamSnapshot,
}

fn decode_remote_team_snapshot(message: &mut CMessage) -> Option<TeamSessionSnapshot> {
    let session_type = message.base_mut().get_long()?;
    if session_type != 1 {
        return None;
    }
    let minimum_plugs = message.base_mut().get_long()? as u32;
    let maximum_plugs = message.base_mut().get_long()? as u32;
    let lifetime = message.base_mut().get_long()? as u32;
    let team_id = message.base_mut().get_long()? as u32;
    let team_name = message.base_mut().get_str_bytes(0x100)?;
    let password = message.base_mut().get_str_bytes(0x100)?;
    let leader_id = message.base_mut().get_long()?;
    let member_count = usize::try_from(message.base_mut().get_long()? as u32).ok()?;
    if minimum_plugs > maximum_plugs || maximum_plugs > 8 || member_count > maximum_plugs as usize {
        return None;
    }
    let mut members = Vec::with_capacity(member_count);
    for _ in 0..member_count {
        if message.base_mut().get_long()? != 5 {
            return None;
        }
        let owner_type = message.base_mut().get_long()?;
        let owner_id = message.base_mut().get_long()?;
        let _plug_state = message.base_mut().get_long()?;
        let owner_region_id = message.base_mut().get_long()?;
        let owner_name = message.base_mut().get_str_bytes(0x100)?;
        members.push(TeamMemberSnapshot {
            owner_type,
            owner_id,
            owner_region_id,
            owner_name,
        });
    }
    Some(TeamSessionSnapshot {
        minimum_plugs,
        maximum_plugs,
        lifetime,
        team_id,
        team_name,
        password,
        leader_id,
        members,
    })
}

fn notify_colored(
    game: &CGame,
    player_id: i32,
    string_id: &[u8],
    color: u32,
) {
    if game.find_player(player_id).is_some() {
        let delivery = colored_player_notice_message(color, 0, game.get_string_by_id(string_id))
            .send_to_player(game.net_server(), player_id);
        trace!(player_id, delivery, string_id = ?String::from_utf8_lossy(string_id), "Отправлено уведомление группы");
    }
}

fn notify(game: &CGame, player_id: i32, string_id: &[u8]) {
    notify_colored(game, player_id, string_id, 0xffff_ffff);
}

fn notify_invite_error(
    game: &CGame,
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
    notify(game, player_id, string_id);
}

pub(crate) fn dispatch_game_team_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<(), GameTeamMessageError>> {
    match message.message_type() as u32 {
        WORLD_ABORT_TEAM => {
            let Some(team_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game.abort_remote_team(team_id as u32).is_some();
            trace!(team_id, applied, "Обработано удалённое завершение группы");
            return Some(Ok(()));
        }
        WORLD_INSERT_MEMBER => {
            let (Some(team_id), Some(owner_type), Some(owner_id), Some(owner_region_id)) = (
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let Some(owner_name) = message.base_mut().get_str_bytes(0x100) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game
                .insert_remote_team_member(
                team_id as u32,
                owner_type,
                owner_id,
                owner_region_id,
                owner_name,
            )
                .is_some();
            trace!(team_id, owner_type, owner_id, owner_region_id, applied, "Обработано удалённое добавление участника группы");
            return Some(Ok(()));
        }
        WORLD_REMOVE_MEMBER => {
            let (Some(team_id), Some(owner_type), Some(owner_id)) = (
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game
                .remove_remote_team_member(team_id as u32, owner_type, owner_id)
                .is_some();
            trace!(team_id, owner_type, owner_id, applied, "Обработано удалённое исключение участника группы");
            return Some(Ok(()));
        }
        WORLD_CHANGE_REGION => {
            let (Some(team_id), Some(owner_type), Some(owner_id), Some(owner_region_id)) = (
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game
                .change_remote_team_region(
                team_id as u32,
                owner_type,
                owner_id,
                owner_region_id,
            )
                .is_some();
            trace!(team_id, owner_type, owner_id, owner_region_id, applied, "Обработана смена региона участника группы");
            return Some(Ok(()));
        }
        WORLD_KICK_PLAYER | WORLD_CHANGE_LEADER => {
            let (Some(team_id), Some(player_id)) =
                (message.base_mut().get_long(), message.base_mut().get_long())
            else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = if message.message_type() as u32 == WORLD_KICK_PLAYER {
                game.kick_remote_team_player(team_id as u32, player_id)
            } else {
                game.change_remote_team_leader(team_id as u32, player_id)
            }
            .is_some();
            trace!(team_id, player_id, applied, message_type = message.message_type(), "Обработано удалённое управление группой");
            return Some(Ok(()));
        }
        WORLD_TEAM_SNAPSHOT => {
            let Some(snapshot) = decode_remote_team_snapshot(message) else {
                return Some(Err(GameTeamMessageError::InvalidRemoteTeamSnapshot));
            };
            let team_id = snapshot.team_id;
            let applied = game.restore_remote_team(snapshot).is_some();
            trace!(team_id, applied, "Восстановлен снимок удалённой группы");
            return Some(Ok(()));
        }
        WORLD_ONLINE_QUERY => {
            let (Some(first), Some(second), Some(player_id)) = (
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let delivery = game.answer_remote_team_online_query(first, second, player_id);
            trace!(first, second, player_id, ?delivery, "Отправлен ответ о присутствии участника группы");
            return Some(Ok(()));
        }
        WORLD_ALLOCATION_SCHEME => {
            let (Some(team_id), Some(allocation_scheme)) =
                (message.base_mut().get_long(), message.base_mut().get_long())
            else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game
                .change_remote_team_allocation(team_id as u32, allocation_scheme)
                .is_some();
            trace!(team_id, allocation_scheme, applied, "Обновлена удалённая схема распределения группы");
            return Some(Ok(()));
        }
        WORLD_TEAM_CHAT => {
            let (Some(team_id), Some(owner_type), Some(owner_id)) = (
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let Some(text) = message.base_mut().get_str_bytes(0x200) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game
                .relay_remote_team_chat(team_id as u32, owner_type, owner_id, &text)
                .is_some();
            trace!(team_id, owner_type, owner_id, applied, "Передано удалённое сообщение группы");
            return Some(Ok(()));
        }
        WORLD_MEMBER_STATE => {
            let (Some(team_id), Some(owner_type), Some(owner_id), Some(state)) = (
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_float(),
            ) else {
                return Some(Err(GameTeamMessageError::MissingRemoteField));
            };
            let applied = game
                .relay_remote_team_state(team_id as u32, owner_type, owner_id, state)
                .is_some();
            trace!(team_id, owner_type, owner_id, state, applied, "Передано удалённое состояние участника группы");
            return Some(Ok(()));
        }
        INVITE_BY_ID => {
            let Some(leader_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingLeaderId));
            };
            let Some(candidate_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingCandidateId));
            };
            let result = game.check_team_join(leader_id, candidate_id);
            if result == GameTeamJoinResult::Succeed {
                if game.find_player(candidate_id).is_some() {
                    let mut invite = CMessage::new(0x000b_fd01);
                    invite.add_long(candidate_id);
                    invite.add_long(leader_id);
                    let delivery = invite.send_to_player(game.net_server(), candidate_id);
                    trace!(leader_id, candidate_id, delivery, "Отправлено приглашение в группу");
                }
            } else {
                notify_invite_error(game, leader_id, result);
                trace!(leader_id, candidate_id, ?result, "Приглашение в группу отклонено проверкой");
            }
            return Some(Ok(()));
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
            if accepted != 1 {
                if game.find_player(leader_id).is_some() {
                    let mut answer = CMessage::new(0x000b_fd02);
                    answer.add_long(candidate_id);
                    answer.add_long(leader_id);
                    answer.add_byte(accepted as u8);
                    let delivery = answer.send_to_player(game.net_server(), leader_id);
                    trace!(leader_id, candidate_id, accepted, delivery, "Отправлен отказ от приглашения в группу");
                }
                return Some(Ok(()));
            }
            let result = game.check_team_join(leader_id, candidate_id);
            if result == GameTeamJoinResult::Succeed {
                let applied = game
                    .join_team_accepted(leader_id, candidate_id, true)
                    .is_some();
                trace!(leader_id, candidate_id, applied, "Обработано принятие приглашения в группу");
            } else {
                match result {
                    GameTeamJoinResult::NoPermission => {
                        notify(game, leader_id, b"GS0091");
                        notify(game, candidate_id, b"GS0097");
                    }
                    GameTeamJoinResult::MaxMemberLimit => {
                        notify(game, leader_id, b"GS0092");
                        notify(game, candidate_id, b"GS0092");
                    }
                    GameTeamJoinResult::PlayerAlreadyInTeam => {
                        notify(game, leader_id, b"GS0093");
                        notify(game, candidate_id, b"GS0098");
                    }
                    GameTeamJoinResult::SamePlayer => {
                        notify(game, leader_id, b"GS0094");
                        notify(game, candidate_id, b"GS0099");
                    }
                    _ => notify(game, leader_id, b"GS0095"),
                }
                trace!(leader_id, candidate_id, ?result, "Присоединение к группе отклонено проверкой");
            }
            return Some(Ok(()));
        }
        JOIN_RECRUITMENT => {
            let Some(leader_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingLeaderId));
            };
            message.resolve_player_context(game);
            let candidate_id = message.player_id();
            let password = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
            let Some(candidate_id) = candidate_id else {
                trace!("Команда присоединения к набору не имеет игрока");
                return Some(Ok(()));
            };
            let password_matches = game
                .find_player(leader_id)
                .and_then(|leader| leader.first_team_recruitment_state())
                .is_some_and(|state| state.team_password() == password);
            if !password_matches {
                notify(game, candidate_id, b"GS0102");
                trace!(leader_id, candidate_id, "Пароль набора в группу не совпал");
                return Some(Ok(()));
            }
            let result = game.check_team_join(leader_id, candidate_id);
            if result == GameTeamJoinResult::Succeed {
                let applied = game
                    .join_team_accepted(leader_id, candidate_id, false)
                    .is_some();
                trace!(leader_id, candidate_id, applied, "Обработано присоединение к набору в группу");
            } else {
                match result {
                    GameTeamJoinResult::MaxMemberLimit => {
                        notify(game, candidate_id, b"GS0100")
                    }
                    GameTeamJoinResult::PlayerAlreadyInTeam => {
                        notify(game, candidate_id, b"GS0101")
                    }
                    _ => {}
                }
                trace!(leader_id, candidate_id, ?result, "Присоединение к набору в группу отклонено");
            }
            return Some(Ok(()));
        }
        LEAVE_TEAM => {
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let applied = player_id.and_then(|id| game.leave_team(id)).is_some();
            trace!(?player_id, applied, "Обработан выход игрока из группы");
            return Some(Ok(()));
        }
        SET_LEADER | KICK_PLAYER => {
            let Some(target_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingTargetPlayerId));
            };
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let applied = player_id
                .and_then(|actor_id| {
                if message.message_type() as u32 == SET_LEADER {
                    game.change_team_leader(actor_id, target_id)
                } else {
                    game.kick_team_member(actor_id, target_id)
                }
            })
                .is_some();
            trace!(?player_id, target_id, applied, message_type = message.message_type(), "Обработано управление составом группы");
            return Some(Ok(()));
        }
        DISBAND_TEAM => {
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let applied = player_id.and_then(|id| game.disband_team(id)).is_some();
            trace!(?player_id, applied, "Обработан роспуск группы");
            return Some(Ok(()));
        }
        SET_ALLOCATION_SCHEME => {
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let Some(player_id) = player_id else {
                trace!("Команда схемы распределения не имеет игрока");
                return Some(Ok(()));
            };
            let Some(allocation_scheme) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingAllocationScheme));
            };
            let applied = game
                .change_team_allocation_scheme(player_id, allocation_scheme)
                .is_some();
            trace!(player_id, allocation_scheme, applied, "Обновлена схема распределения группы");
            return Some(Ok(()));
        }
        TEAM_CHAT => {
            message.resolve_player_context(game);
            let player_id = message.player_id();
            let Some(player_id) = player_id else {
                trace!("Команда чата группы не имеет игрока");
                return Some(Ok(()));
            };
            let Some(player) = game.find_player(player_id) else {
                trace!(player_id, "Игрок чата группы не найден");
                return Some(Ok(()));
            };
            if player.team_id() == 0 {
                trace!(player_id, "Игрок без группы попытался отправить сообщение группы");
                return Some(Ok(()));
            }
            if player.silence_minutes() >= 1 {
                notify_colored(game, player_id, b"GS0330", 0xffff_0000);
                trace!(player_id, "Сообщение группы отклонено из-за молчания");
                return Some(Ok(()));
            }
            let Some(text) = message.base_mut().get_str_bytes(0x200) else {
                return Some(Err(GameTeamMessageError::MissingChatText));
            };
            match game.send_team_chat(player_id, text, game_tick_milliseconds()) {
                GameTeamChatResult::Sent => {
                    trace!(player_id, "Сообщение группы отправлено");
                }
                GameTeamChatResult::Cooldown => {
                    notify_colored(game, player_id, b"GS0049", 0xffff_0000);
                    trace!(player_id, "Сообщение группы отклонено ограничением частоты");
                }
                GameTeamChatResult::MissingContext => {
                    trace!(player_id, "Для сообщения группы отсутствует контекст");
                }
            }
            return Some(Ok(()));
        }
        INVITE_BY_NAME => {
            let Some(candidate_name) = message.base_mut().get_str_bytes(0x100) else {
                return Some(Err(GameTeamMessageError::MissingInvitedName));
            };
            let Some(leader_id) = message.base_mut().get_long() else {
                return Some(Err(GameTeamMessageError::MissingLeaderId));
            };
            let candidate_id = game
                .find_player_by_name(&candidate_name)
                .map(|player| player.player_id());
            let Some(candidate_id) = candidate_id else {
                notify(game, leader_id, b"GS0096");
                trace!(leader_id, "Игрок для приглашения по имени не найден");
                return Some(Ok(()));
            };
            if game.find_player(leader_id).is_none() {
                trace!(leader_id, candidate_id, "Инициатор приглашения по имени не найден");
                return Some(Ok(()));
            }
            let result = game.check_team_join(leader_id, candidate_id);
            if result == GameTeamJoinResult::Succeed {
                let mut invite = CMessage::new(0x000b_fd01);
                invite.add_long(candidate_id);
                invite.add_long(leader_id);
                let delivery = invite.send_to_player(game.net_server(), candidate_id);
                trace!(leader_id, candidate_id, delivery, "Отправлено приглашение в группу по имени");
            } else {
                notify_invite_error(game, leader_id, result);
                trace!(leader_id, candidate_id, ?result, "Приглашение в группу по имени отклонено проверкой");
            }
            return Some(Ok(()));
        }
        SET_RECRUITMENT_STATE => {}
        _ => return None,
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let Some(player_id) = player_id else {
        trace!("Команда набора в группу не имеет игрока");
        return Some(Ok(()));
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
            trace!(player_id, "Набор в группу уже выключен");
            return Some(Ok(()));
        };
        let mut ended = CMessage::new(0x000b_fe04);
        ended.add_long(400);
        ended.add_long(player_id);
        ended.add_long(state.state_id());
        let delivery = game.send_player_shape_around(player_id, None, &ended);
        let _ended = game
            .find_player_mut(player_id)
            .and_then(|player| player.end_first_team_recruitment_state());
        let state_count = game
            .find_player(player_id)
            .map(|player| player.team_recruitment_state_count())
            .unwrap_or_default();
        trace!(player_id, ?delivery, state_count, "Набор в группу выключен");
        return Some(Ok(()));
    }

    let mut team_name = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
    let team_password = message.base_mut().get_str_bytes(0x100).unwrap_or_default();
    if !game
        .words_filter()
        .check_with_numeric_gate(&mut team_name, false, true)
    {
        let delivery = colored_player_notice_message(
            0xffff_ffff,
            0,
            game.get_string_by_id(b"GS0333"),
        )
        .send_to_player(game.net_server(), player_id);
        trace!(player_id, delivery, "Название группы отклонено фильтром слов");
        return Some(Ok(()));
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
    let delivery = game.send_player_shape_around(player_id, None, &begun);
    game.find_player_mut(player_id)
        .expect("team message player разрешён до state attach")
        .attach_team_recruitment_state(state);
    let state_count = game
        .find_player(player_id)
        .map(|player| player.team_recruitment_state_count())
        .unwrap_or_default();
    trace!(player_id, ?delivery, state_count, "Набор в группу включён");
    Some(Ok(()))
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
