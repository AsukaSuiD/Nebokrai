//! Входящий JJC-owner WorldServer.
//!
//! `OnJJcSystemMessage` и leaf-ветви входят в контракт owner-а из
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`.
//!
//! Dispatcher сохраняет numeric gate `bUseJJc`, порядок cursor-чтений и
//! opcodes `0x60901..0x60907`. В частности, `0x60901` после отсутствующего
//! player не читает остаток payload, `0x60902` читает его до lookup-а, ответ
//! `0x80502` отправляется только при ненулевом result `ApplyPlayer`, а
//! `0x60905` остаётся намеренным no-op. Недостаточный payload вместо старого
//! out-of-bounds чтения даёт typed malformed outcome без частичных записей.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::appworld::jjcsystem::{
    CJJcSystem, JjcApplyReport, JjcEndPkReport, JjcInfo, JjcLogEvent, JjcQuitReport,
    JjcRunConfig, JjcRunContext, JjcUpdateBroadcastReport,
};
use crate::worldserver::worldserver::game::CGame;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct JjcApplyMessageReport {
    pub(crate) player_id: i32,
    pub(crate) apply: JjcApplyReport,
    pub(crate) rejection_delivery: Option<Result<i32, SendMessageError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum JjcSystemMessageOutcome {
    Disabled { message_type: i32 },
    PlayerSnapshot { player_id: i32, player_found: bool },
    Apply(JjcApplyMessageReport),
    ApplyPlayerMissing { player_id: i32 },
    Quit(JjcQuitReport),
    EndPk(JjcEndPkReport),
    WeekUpdate(JjcUpdateBroadcastReport),
    SeasonUpdate(JjcUpdateBroadcastReport),
    ReservedRankNoOp,
    UnknownOpcode { message_type: i32 },
    Malformed { message_type: i32, cursor: usize },
}

pub(crate) fn on_jjc_system_message<Context: JjcRunContext + ?Sized>(
    game: &mut CGame,
    system: &mut CJJcSystem,
    config: JjcRunConfig,
    context: &mut Context,
    message: &mut CMessage,
) -> JjcSystemMessageOutcome {
    let message_type = message.message_type();
    if config.use_jjc == 0 {
        return JjcSystemMessageOutcome::Disabled { message_type };
    }

    macro_rules! malformed {
        () => {
            return JjcSystemMessageOutcome::Malformed {
                message_type,
                cursor: message.base_mut().cursor(),
            }
        };
    }

    match message_type {
        0x0006_0901 => {
            let Some(player_id) = message.base_mut().get_long() else {
                malformed!();
            };
            if game.map_player(player_id as u32).is_none() {
                return JjcSystemMessageOutcome::PlayerSnapshot {
                    player_id,
                    player_found: false,
                };
            }
            let Some(level) = message.base_mut().get_char().map(|value| value as u8) else {
                malformed!();
            };
            let Some(jjc_level) = message.base_mut().get_long().map(|value| value as u32) else {
                malformed!();
            };
            let Some(jjc_score) = message.base_mut().get_long().map(|value| value as u32) else {
                malformed!();
            };
            let mut counters = [0; 0x10];
            if !message.base_mut().get(&mut counters) {
                malformed!();
            }
            let player_found = game.set_map_player_jjc_snapshot(
                player_id as u32,
                level,
                jjc_level,
                jjc_score,
                counters,
            );
            JjcSystemMessageOutcome::PlayerSnapshot {
                player_id,
                player_found,
            }
        }
        0x0006_0902 => {
            let Some(player_id) = message.base_mut().get_long() else {
                malformed!();
            };
            let Some(level) = message.base_mut().get_char().map(|value| value as u8) else {
                malformed!();
            };
            let Some(jjc_level) = message.base_mut().get_long().map(|value| value as u32) else {
                malformed!();
            };
            let Some(old_region_id) = message.base_mut().get_long() else {
                malformed!();
            };
            let Some(position_x) = message.base_mut().get_long() else {
                malformed!();
            };
            let Some(position_y) = message.base_mut().get_long() else {
                malformed!();
            };
            if !game.set_map_player_jjc_identity(player_id as u32, level, jjc_level) {
                context.log(JjcLogEvent::ApplyPlayerMissing { player_id });
                return JjcSystemMessageOutcome::ApplyPlayerMissing { player_id };
            }
            context.log(JjcLogEvent::ApplyStarted { player_id });
            let apply = system.apply_player(
                game,
                context,
                JjcInfo {
                    jjc_level,
                    old_region_id,
                    position_x,
                    position_y,
                    opponent_id: 0,
                    jjc_region_id: 0,
                    start_time: 0,
                },
                player_id,
                config,
            );
            let rejection_delivery = (apply.legacy_result != 0).then(|| {
                let mut response = CMessage::new(0x0008_0502);
                response.base_mut().add_long(apply.legacy_result);
                response.base_mut().add_long(player_id);
                game.send_msg_to_game_server(message.map_id(), &response)
            });
            context.log(JjcLogEvent::ApplyFinished { player_id });
            JjcSystemMessageOutcome::Apply(JjcApplyMessageReport {
                player_id,
                apply,
                rejection_delivery,
            })
        }
        0x0006_0903 => {
            let Some(player_id) = message.base_mut().get_long() else {
                malformed!();
            };
            let Some(region_id) = message.base_mut().get_long() else {
                malformed!();
            };
            JjcSystemMessageOutcome::Quit(system.quit_player(
                game, config, context, player_id, region_id,
            ))
        }
        0x0006_0904 => {
            let Some(region_id) = message.base_mut().get_long() else {
                malformed!();
            };
            let Some(player_id) = message.base_mut().get_long() else {
                malformed!();
            };
            JjcSystemMessageOutcome::EndPk(system.end_pk(
                config, context, region_id, player_id,
            ))
        }
        0x0006_0905 => JjcSystemMessageOutcome::ReservedRankNoOp,
        0x0006_0906 => {
            let Some(timestamp) = message.base_mut().get_long() else {
                malformed!();
            };
            JjcSystemMessageOutcome::WeekUpdate(system.week_update(game, timestamp, context))
        }
        0x0006_0907 => {
            let Some(timestamp) = message.base_mut().get_long() else {
                malformed!();
            };
            JjcSystemMessageOutcome::SeasonUpdate(system.season_update(game, timestamp, context))
        }
        _ => JjcSystemMessageOutcome::UnknownOpcode { message_type },
    }
}
