//! Входные skill-сообщения GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/message/skillmessage.cpp`. Достигнутый
//! исполняемый контракт `0x90001` строго читает пять Windows `long`, сохраняет
//! contend notice, безусловный ClearEmotion, learned-skill authorization,
//! self/point/object target и адресный socket reject либо очередь `CPlayerAI`.
//! `0x90005` тем же route читает шесть `long` и добавляет battle-fairy gates.
//!
//! Безопасный decoder отклоняет оборванный payload вместо исходного чтения за
//! границей буфера. Остальные opcodes owner-а остаются RAW ниже и продолжают
//! маршрутизироваться прежней общей handler-границей.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\skillmessage.cpp

use crate::gameserver::appserver::player::{
    BattleFairySkillRequest, BattleFairySkillRequestFacts, BattleFairySkillRequestReport,
    PlayerSkillRequest, PlayerSkillRequestFacts, PlayerSkillRequestReport,
};
use crate::gameserver::gameserver::game::{
    BattleFairySkillRequestContext, CGame, PlayerSkillRequestContext,
};
use crate::nets::netserver::message::CMessage;

const USE_PLAYER_SKILL: u32 = 0x0009_0001;
const USE_BATTLE_FAIRY_SKILL: u32 = 0x0009_0005;

pub(crate) trait GameSkillMessageRuntime:
    BattleFairySkillRequestContext + PlayerSkillRequestContext
{
    fn player_skill_request_facts(
        &mut self,
        game: &CGame,
        player_id: i32,
        region_id: Option<i32>,
        request: PlayerSkillRequest,
    ) -> PlayerSkillRequestFacts;

    fn battle_fairy_skill_request_facts(
        &mut self,
        game: &CGame,
        player_id: i32,
        region_id: Option<i32>,
        request: BattleFairySkillRequest,
    ) -> BattleFairySkillRequestFacts;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameSkillMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameSkillMessageOutcome {
    MissingPlayer,
    PlayerSkill(PlayerSkillRequestReport),
    BattleFairy(BattleFairySkillRequestReport),
}

#[must_use = "skill-message report содержит decode, routing и gameplay result"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameSkillMessageReport {
    pub(crate) message_type: u32,
    pub(crate) socket_id: i32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) outcome: GameSkillMessageOutcome,
}

pub(crate) fn dispatch_game_skill_message<Runtime: GameSkillMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameSkillMessageReport, GameSkillMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(message_type, USE_PLAYER_SKILL | USE_BATTLE_FAIRY_SKILL) {
        return None;
    }

    message.resolve_player_context(game);
    let socket_id = message.socket_id();
    let player_id = message.player_id();
    let region_id = message.region_id();
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(GameSkillMessageError::MissingField(field))
    };
    let outcome = match message_type {
        USE_PLAYER_SKILL => {
            let request = match (|| {
                Ok(PlayerSkillRequest {
                    raw_skill_id: read_long(message, "skill id")?,
                    target_type: read_long(message, "target type")?,
                    target_id: read_long(message, "target id")?,
                    target_x: read_long(message, "target x")?,
                    target_y: read_long(message, "target y")?,
                })
            })() {
                Ok(request) => request,
                Err(error) => return Some(Err(error)),
            };
            let Some(player_id) = player_id else {
                return Some(Ok(GameSkillMessageReport {
                    message_type,
                    socket_id,
                    player_id: None,
                    region_id,
                    outcome: GameSkillMessageOutcome::MissingPlayer,
                }));
            };
            let facts = runtime.player_skill_request_facts(game, player_id, region_id, request);
            let report = game
                .request_player_skill(player_id, socket_id, request, facts, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
            GameSkillMessageOutcome::PlayerSkill(report)
        }
        USE_BATTLE_FAIRY_SKILL => {
            let request = match (|| {
                Ok(BattleFairySkillRequest {
                    raw_skill_id: read_long(message, "skill id")?,
                    target_type: read_long(message, "target type")?,
                    target_id: read_long(message, "target id")?,
                    property_offset: read_long(message, "property offset")?,
                    target_x: read_long(message, "target x")?,
                    target_y: read_long(message, "target y")?,
                })
            })() {
                Ok(request) => request,
                Err(error) => return Some(Err(error)),
            };
            let Some(player_id) = player_id else {
                return Some(Ok(GameSkillMessageReport {
                    message_type,
                    socket_id,
                    player_id: None,
                    region_id,
                    outcome: GameSkillMessageOutcome::MissingPlayer,
                }));
            };
            let facts =
                runtime.battle_fairy_skill_request_facts(game, player_id, region_id, request);
            let report = game
                .request_battle_fairy_skill(player_id, socket_id, request, facts, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
            GameSkillMessageOutcome::BattleFairy(report)
        }
        _ => unreachable!("unsupported skill message отфильтрован до decode"),
    };
    Some(Ok(GameSkillMessageReport {
        message_type,
        socket_id,
        player_id,
        region_id,
        outcome,
    }))
}

// ============================================================================
// FUNCTION: OnSkillMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\skillmessage.cpp:18
// RVA: 0x00088B60
// ADDRESS: 00488b60
// PROTOTYPE: void __cdecl OnSkillMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
