//! Входные skill-сообщения GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
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
//! player-owned `CPlayerAI`; concrete `CSkill::IsEnd/End(true)` и исполнение
//! target/skill очередей остаются явно названными runtime-границами.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\skillmessage.cpp

use crate::gameserver::appserver::player::{
    BattleFairySkillRequest, BattleFairySkillRequestFacts, BattleFairySkillRequestReport,
    PlayerSkillRequest, PlayerSkillRequestFacts, PlayerSkillRequestReport,
};
use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillEndOutcome {
    MissingCurrentSkill,
    SkillMismatch,
    AlreadyEnded,
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSkillEndReport {
    pub(crate) player_id: i32,
    pub(crate) requested_skill_id: i32,
    pub(crate) current_skill_id: Option<u32>,
    pub(crate) outcome: PlayerSkillEndOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerSkillScriptOutcome {
    MissingPlayer,
    FeatureDisabled,
    Dispatched { script_present: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSkillScriptReport {
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) skill_id: i32,
    pub(crate) level: i32,
    pub(crate) variant: i32,
    pub(crate) path: Option<Vec<u8>>,
    pub(crate) notice_delivery: Option<i32>,
    pub(crate) outcome: PlayerSkillScriptOutcome,
}

pub(crate) trait GameSkillMessageRuntime: ScriptFunctionRuntime {
    fn end_current_player_skill(
        &mut self,
        game: &mut CGame,
        player_id: i32,
        skill_id: u32,
    ) -> PlayerSkillEndRuntimeOutcome;

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
    PlayerSkillEnd(PlayerSkillEndReport),
    PlayerSkillScript(PlayerSkillScriptReport),
    ItemSkill(PlayerSkillRequestReport),
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
                .request_player_skill(player_id, socket_id, request, facts)
                .expect("resolved message player остаётся в CGame во время synchronous dispatch");
            GameSkillMessageOutcome::PlayerSkill(report)
        }
        END_PLAYER_SKILL => {
            let requested_skill_id = match read_long(message, "skill id") {
                Ok(skill_id) => skill_id,
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
            let current_skill_id = game
                .find_player(player_id)
                .and_then(|player| player.current_skill_id());
            let outcome = match current_skill_id {
                None => PlayerSkillEndOutcome::MissingCurrentSkill,
                Some(current) if current != requested_skill_id as u32 => {
                    PlayerSkillEndOutcome::SkillMismatch
                }
                Some(current) => match runtime.end_current_player_skill(game, player_id, current) {
                    PlayerSkillEndRuntimeOutcome::AlreadyEnded => {
                        PlayerSkillEndOutcome::AlreadyEnded
                    }
                    PlayerSkillEndRuntimeOutcome::Ended => PlayerSkillEndOutcome::Ended,
                },
            };
            GameSkillMessageOutcome::PlayerSkillEnd(PlayerSkillEndReport {
                player_id,
                requested_skill_id,
                current_skill_id,
                outcome,
            })
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
            let mut report = PlayerSkillScriptReport {
                player_id,
                region_id,
                skill_id,
                level,
                variant,
                path: None,
                notice_delivery: None,
                outcome: PlayerSkillScriptOutcome::MissingPlayer,
            };
            let Some(player_id) = player_id else {
                return Some(Ok(GameSkillMessageReport {
                    message_type,
                    socket_id,
                    player_id: None,
                    region_id,
                    outcome: GameSkillMessageOutcome::PlayerSkillScript(report),
                }));
            };
            let battle_fairy_script =
                (530..=545).contains(&skill_id) || (960..=962).contains(&skill_id);
            if battle_fairy_script && !game.battle_fairy_enabled() {
                report.notice_delivery = Some(
                    colored_player_notice_message(
                        0xffff_0000,
                        0,
                        game.get_string_by_id(b"ZHGS0037"),
                    )
                    .send_to_player(game.net_server(), player_id),
                );
                report.outcome = PlayerSkillScriptOutcome::FeatureDisabled;
                GameSkillMessageOutcome::PlayerSkillScript(report)
            } else {
                let path = if (530..=545).contains(&skill_id) {
                    format!("scripts/skills/{skill_id}0{level}.script").into_bytes()
                } else if (960..=962).contains(&skill_id) {
                    format!("scripts/skills/{skill_id}{variant}.script").into_bytes()
                } else {
                    format!("scripts/skills/{skill_id}.script").into_bytes()
                };
                let script_data = game.script_file_data(&path).map(<[u8]>::to_vec);
                report.path = Some(path.clone());
                let _ = game.run_script_file(
                    &path,
                    ScriptExecutionContext {
                        player_id: Some(player_id),
                        region_id,
                        ..ScriptExecutionContext::default()
                    },
                    runtime,
                );
                report.outcome = PlayerSkillScriptOutcome::Dispatched {
                    script_present: script_data.is_some(),
                };
                GameSkillMessageOutcome::PlayerSkillScript(report)
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
                .request_item_skill(player_id, socket_id, request, skill_level, facts)
                .expect("resolved message player остаётся в CGame во время item-skill dispatch");
            GameSkillMessageOutcome::ItemSkill(report)
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
                .request_battle_fairy_skill(player_id, socket_id, request, facts)
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

// COMPONENT_VARIANT_END: GameServer
