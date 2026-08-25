//! Team-message owner GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/teammessage.cpp`. Материализован полный достигнутый
//! recruitment-state selector `0x8FF08`: socket player context, enable flag,
//! две bounded C-строки, `CWordsFilter::Check(false, true)`, `GS0333`, typed
//! `CTeamState` attach/end и around `0xBFE03/04`. State становится canonical
//! gate для equipment-session и synthesis callers. Остальные team selectors
//! зависят от ещё не восстановленных `CTeam/CTeamate` session owners и
//! сохранены ниже как RAW.

use crate::gameserver::appserver::teamstate::CTeamState;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;

const SET_RECRUITMENT_STATE: u32 = 0x0008_ff08;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameTeamMessageError {
    MissingEnabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameTeamMessageOutcome {
    MissingPlayer,
    AlreadyDisabled,
    Disabled,
    RejectedByWordsFilter,
    Enabled,
}

#[must_use = "team-message report сохраняет state и client effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameTeamMessageReport {
    pub(crate) player_id: Option<i32>,
    pub(crate) outcome: GameTeamMessageOutcome,
    pub(crate) state_count: usize,
    pub(crate) delivery:
        Option<Result<i32, crate::gameserver::appserver::shape::ShapeCoordinateBlock>>,
    pub(crate) notification_delivery: Option<i32>,
}

pub(crate) fn dispatch_game_team_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<GameTeamMessageReport, GameTeamMessageError>> {
    if message.message_type() as u32 != SET_RECRUITMENT_STATE {
        return None;
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let mut report = GameTeamMessageReport {
        player_id,
        outcome: GameTeamMessageOutcome::MissingPlayer,
        state_count: 0,
        delivery: None,
        notification_delivery: None,
    };
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
        report.notification_delivery = Some(
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
