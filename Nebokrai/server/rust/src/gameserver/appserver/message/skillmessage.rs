//! Входные сообщения навыков GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/message/skillmessage.cpp`. Достигнутый
//! исполняемый контракт `0x90001` строго читает пять Windows `long`, сохраняет
//! contend notice, безусловный ClearEmotion, learned-skill authorization,
//! self/point/object target и адресный socket reject либо очередь `CPlayerAI`.
//! `0x90002` сохраняет current-skill/IsEnd/End gate, `0x90003` — exact три
//! script path family и battle-fairy feature reject, `0x90004` — client level,
//! ordered item-skill state и тот же target/AI route. `0x90005` добавляет
//! battle-fairy equipment gates; `0x90006` остаётся подтверждённым no-op.
//!
//! Безопасный decoder отклоняет оборванный payload вместо исходного чтения за
//! границей буфера. Skill-script path входит в reached `CGame::run_script_file`
//! с player/region context. Оба AI-dispatch теперь попадают в canonical
//! player-owned `CPlayerAI`. Canonical player AI, base/city
//! `SymbolIsAttackAble` virtual и все зарегистрированные region target
//! разрешаются самим `CGame`; отдельного внешнего shape-resolver-а нет.
//! Concrete `CSkill::IsEnd/End(true)` и исполнение target/skill очередей
//! остаются явно названными runtime-границами.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\skillmessage.cpp

use crate::gameserver::appserver::player::{BattleFairySkillRequest, PlayerSkillRequest};
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, MaterializedSkillEndCause, colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;
use tracing::trace;

const USE_PLAYER_SKILL: u32 = 0x0009_0001;
const END_PLAYER_SKILL: u32 = 0x0009_0002;
const RUN_SKILL_SCRIPT: u32 = 0x0009_0003;
const USE_ITEM_SKILL: u32 = 0x0009_0004;
const USE_BATTLE_FAIRY_SKILL: u32 = 0x0009_0005;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillEndRuntimeOutcome {
    AlreadyEnded,
    Ended,
}

/// Skill message route использует тот же script runtime; завершение любого
/// current player skill уже принадлежит materialized `CPlayerAI` owner-у.
pub(crate) trait GameSkillMessageRuntime: ScriptFunctionRuntime {}

impl<T: ScriptFunctionRuntime> GameSkillMessageRuntime for T {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameSkillMessageError {
    MissingField(&'static str),
}

pub(crate) fn dispatch_game_skill_message<Runtime: GameMainLoopRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameSkillMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        USE_PLAYER_SKILL
            | END_PLAYER_SKILL
            | RUN_SKILL_SCRIPT
            | USE_ITEM_SKILL
            | USE_BATTLE_FAIRY_SKILL
    ) {
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
    match message_type {
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
                trace!(message_type, socket_id, "Команда навыка не имеет игрока");
                return Some(Ok(()));
            };
            let facts = game.player_skill_request_facts(player_id, region_id, request);
            game
                .request_player_skill(player_id, socket_id, request, facts, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
            trace!(player_id, skill_id = request.skill_id(), "Обработан запрос навыка игрока");
        }
        END_PLAYER_SKILL => {
            let requested_skill_id = match read_long(message, "skill id") {
                Ok(skill_id) => skill_id,
                Err(error) => return Some(Err(error)),
            };
            let Some(player_id) = player_id else {
                trace!(message_type, socket_id, "Команда завершения навыка не имеет игрока");
                return Some(Ok(()));
            };
            let current_skill_id = game
                .find_player(player_id)
                .and_then(|player| player.current_skill_id());
            let outcome = match current_skill_id {
                None => "нет текущего навыка",
                Some(current) if current != requested_skill_id as u32 => "идентификатор не совпал",
                Some(current) => match game
                    .end_materialized_player_skill(
                        player_id,
                        current,
                        MaterializedSkillEndCause::ClientRequest,
                        runtime,
                    )
                    .unwrap_or(PlayerSkillEndRuntimeOutcome::AlreadyEnded)
                {
                    PlayerSkillEndRuntimeOutcome::AlreadyEnded => "уже завершён",
                    PlayerSkillEndRuntimeOutcome::Ended => "завершён",
                },
            };
            trace!(player_id, requested_skill_id, ?current_skill_id, outcome, "Обработано завершение навыка");
        }
        RUN_SKILL_SCRIPT => {
            let (skill_id, level, variant) = match (|| {
                Ok((
                    read_long(message, "skill id")?,
                    read_long(message, "skill level")?,
                    read_long(message, "skill variant")?,
                ))
            })() {
                Ok(fields) => fields,
                Err(error) => return Some(Err(error)),
            };
            let Some(player_id) = player_id else {
                trace!(message_type, socket_id, skill_id, "Команда сценария навыка не имеет игрока");
                return Some(Ok(()));
            };
            let battle_fairy_script =
                (530..=545).contains(&skill_id) || (960..=962).contains(&skill_id);
            if battle_fairy_script && !game.battle_fairy_enabled() {
                let delivery = colored_player_notice_message(
                    0xffff_0000,
                    0,
                    game.get_string_by_id(b"ZHGS0037"),
                )
                .send_to_player(game.net_server(), player_id);
                trace!(player_id, skill_id, delivery, "Сценарий навыка боевой феи отклонён");
            } else {
                let path = if (530..=545).contains(&skill_id) {
                    format!("scripts/skills/{skill_id}0{level}.script").into_bytes()
                } else if (960..=962).contains(&skill_id) {
                    format!("scripts/skills/{skill_id}{variant}.script").into_bytes()
                } else {
                    format!("scripts/skills/{skill_id}.script").into_bytes()
                };
                let script_present = game.script_file_data(&path).is_some();
                let _ = game.run_script_file(
                    &path,
                    ScriptExecutionContext {
                        player_id: Some(player_id),
                        region_id,
                        ..ScriptExecutionContext::default()
                    },
                    runtime,
                );
                trace!(player_id, skill_id, level, variant, script_present, "Сценарий навыка передан исполнителю");
            }
        }
        USE_ITEM_SKILL => {
            let (request, skill_level) = match (|| {
                let raw_skill_id = read_long(message, "skill id")?;
                let skill_level = read_long(message, "skill level")?;
                Ok((
                    PlayerSkillRequest {
                        raw_skill_id,
                        target_type: read_long(message, "target type")?,
                        target_id: read_long(message, "target id")?,
                        target_x: read_long(message, "target x")?,
                        target_y: read_long(message, "target y")?,
                    },
                    skill_level,
                ))
            })() {
                Ok(fields) => fields,
                Err(error) => return Some(Err(error)),
            };
            let Some(player_id) = player_id else {
                trace!(message_type, socket_id, "Команда предметного навыка не имеет игрока");
                return Some(Ok(()));
            };
            let facts = game.player_skill_request_facts(player_id, region_id, request);
            game
                .request_item_skill(player_id, socket_id, request, skill_level, facts, runtime)
                .expect("resolved message player остаётся в CGame во время item-skill dispatch");
            trace!(player_id, skill_id = request.skill_id(), skill_level, "Обработан запрос предметного навыка");
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
                trace!(message_type, socket_id, "Команда навыка боевой феи не имеет игрока");
                return Some(Ok(()));
            };
            let facts = game.battle_fairy_skill_request_facts(player_id, region_id, request);
            game
                .request_battle_fairy_skill(player_id, socket_id, request, facts, runtime)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
            trace!(player_id, skill_id = request.skill_id(), "Обработан запрос навыка боевой феи");
        }
        _ => unreachable!("unsupported skill message отфильтрован до decode"),
    };
    Some(Ok(()))
}

// COMPONENT_VARIANT_END: GameServer
