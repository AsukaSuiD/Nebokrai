//! WorldServer dispatcher-owner `OnOtherMessage`.
//!
//! Весь dispatcher RVA `0x000AC680` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме локального
//! transport leaves `0x5FD02`, `0x5FD06..0x5FD09`, cursor-only `0x5FD0E`,
//! honor-reset `0x5FD0C` и eliminate update `0x5FD0D` со статусом
//! `IMPLEMENTED`. Reset читает один Windows `long`, получает текущий `CGame`
//! и вызывает `ResetHonorElimilateInfo`.
//! Недостаточный payload сохраняет старое поведение numeric getter-а: значение
//! становится нулём без сдвига cursor; отчёт отдельно фиксирует неполноту.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\othermessage.cpp:43`.
//! Linux C++ подтверждает практическую границу dispatcher-а, но добавленную там
//! проверку synthetic owner-а и route-validation Rust не переносит: exact EXE
//! их не выполняет. Для `0x5FD0D` exact owner сначала читает player/eliminator,
//! проверяет online player и duplicate ledger, и только для новой пары читает
//! четыре прежних счётчика, прибавляет к каждому единицу, обновляет ranks и
//! отвечает `0x7FA16 + player + char(1)` в исходный socket. Дубликат прекращает
//! ветку до чтения счётчиков и ответа; этот cursor/order контракт сохранён.
//! `0x5FD02` читает target map, переписывает исходный type в `0x7FA05` и
//! маршрутизует то же сообщение; `0x5FD06..09` только переписывают type и
//! делают `SendAll`. `0x5FD0E` ровно один раз читает и отбрасывает signed long.
//! Donor-added ownership/tail validation отсутствует в EXE и не перенесена.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::{
    CGame, WorldHonorEliminatorRegistration,
};
use crate::worldserver::worldserver::honorranks::{
    CHonorRanks, HonorRankPushBlock, HonorRanksKilledPlayerReport,
};

const HONOR_ELIMINATE_RESET: i32 = 0x0005_FD0C;
const HONOR_ELIMINATE_UPDATE: i32 = 0x0005_FD0D;
const HONOR_ELIMINATE_ACKNOWLEDGEMENT: i32 = 0x0007_FA16;

/// Наблюдаемый итог одной уже восстановленной ветки `OnOtherMessage`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorEliminateReset {
    pub(crate) rank_mask: u32,
    pub(crate) payload_complete: bool,
    pub(crate) legacy_result: bool,
}

/// Наблюдаемый исход exact duplicate-ledger и rank-update ветки `0x5FD0D`.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldHonorEliminateUpdate {
    MissingOnlinePlayer {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
    },
    Duplicate {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
    },
    RankUpdateBlocked {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
        eliminate_counts: [u32; 4],
        counts_payload_complete: [bool; 4],
        source: HonorRankPushBlock,
    },
    Updated {
        player_id: u32,
        eliminator_id: u32,
        identity_payload_complete: [bool; 2],
        eliminate_counts: [u32; 4],
        counts_payload_complete: [bool; 4],
        ranks: Option<HonorRanksKilledPlayerReport>,
        acknowledgement_type: i32,
        acknowledgement: Result<i32, SendMessageError>,
    },
}

/// Один обработанный результат частично восстановленного other-owner-а.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldOtherMessageOutcome {
    Transport {
        request_type: i32,
        response_type: i32,
        target_map_id: Option<i32>,
        target_payload_complete: Option<bool>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    DiscardedLong {
        request_type: i32,
        value: i32,
        payload_complete: bool,
    },
    HonorEliminateReset(WorldHonorEliminateReset),
    HonorEliminateUpdate(WorldHonorEliminateUpdate),
}

/// Узкая диспетчеризация уже выбранного other-owner-а.
pub(crate) enum WorldOtherMessageDispatch {
    Handled(WorldOtherMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутые transport/cursor/honor ветви other-owner-а.
pub(crate) fn on_other_message(
    game: &mut CGame,
    honor_ranks: &mut CHonorRanks,
    mut message: CMessage,
) -> WorldOtherMessageDispatch {
    match message.message_type() {
        0x0005_FD02 => {
            let decoded_target_map_id = message.base_mut().get_long();
            let target_map_id = decoded_target_map_id.unwrap_or(0);
            message.set_message_type(0x0007_FA05);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(target_map_id, &message);
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::Transport {
                request_type: 0x0005_FD02,
                response_type: 0x0007_FA05,
                target_map_id: Some(target_map_id),
                target_payload_complete: Some(decoded_target_map_id.is_some()),
                wire,
                delivery,
            })
        }
        request_type @ (0x0005_FD06 | 0x0005_FD07 | 0x0005_FD08 | 0x0005_FD09) => {
            let response_type = match request_type {
                0x0005_FD06 => 0x0007_FA03,
                0x0005_FD07 => 0x0007_FA0F,
                0x0005_FD08 => 0x0007_FA10,
                0x0005_FD09 => 0x0007_FA11,
                _ => unreachable!("match pattern ограничивает exact transport set"),
            };
            message.set_message_type(response_type);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_all(game.current_game_server_sender().as_ref());
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::Transport {
                request_type,
                response_type,
                target_map_id: None,
                target_payload_complete: None,
                wire,
                delivery,
            })
        }
        0x0005_FD0E => {
            let decoded_value = message.base_mut().get_long();
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::DiscardedLong {
                request_type: 0x0005_FD0E,
                value: decoded_value.unwrap_or(0),
                payload_complete: decoded_value.is_some(),
            })
        }
        HONOR_ELIMINATE_RESET => {
            let decoded = message.base_mut().get_long();
            let rank_mask = decoded.unwrap_or(0) as u32;
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::HonorEliminateReset(
                WorldHonorEliminateReset {
                    rank_mask,
                    payload_complete: decoded.is_some(),
                    legacy_result: game.reset_honor_eliminate_info(rank_mask),
                },
            ))
        }
        HONOR_ELIMINATE_UPDATE => {
            let decoded_player = message.base_mut().get_long();
            let decoded_eliminator = message.base_mut().get_long();
            let player_id = decoded_player.unwrap_or(0) as u32;
            let eliminator_id = decoded_eliminator.unwrap_or(0) as u32;
            let identity_payload_complete =
                [decoded_player.is_some(), decoded_eliminator.is_some()];

            let early = match game.register_honor_eliminator(player_id, eliminator_id) {
                WorldHonorEliminatorRegistration::MissingOnlinePlayer => {
                    Some(WorldHonorEliminateUpdate::MissingOnlinePlayer {
                        player_id,
                        eliminator_id,
                        identity_payload_complete,
                    })
                }
                WorldHonorEliminatorRegistration::Duplicate => {
                    Some(WorldHonorEliminateUpdate::Duplicate {
                        player_id,
                        eliminator_id,
                        identity_payload_complete,
                    })
                }
                WorldHonorEliminatorRegistration::Accepted => None,
            };
            if let Some(outcome) = early {
                return WorldOtherMessageDispatch::Handled(
                    WorldOtherMessageOutcome::HonorEliminateUpdate(outcome),
                );
            }

            let decoded_counts = [
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
                message.base_mut().get_long(),
            ];
            let counts_payload_complete = decoded_counts.map(|value| value.is_some());
            let eliminate_counts =
                decoded_counts.map(|value| value.unwrap_or(0).wrapping_add(1) as u32);
            let player = game
                .online_player_by_id(player_id)
                .expect("accepted honor pair сохраняет прежнего online player");
            let ranks = match honor_ranks.killed_one_player(player, eliminate_counts) {
                Ok(ranks) => ranks,
                Err(source) => {
                    return WorldOtherMessageDispatch::Handled(
                        WorldOtherMessageOutcome::HonorEliminateUpdate(
                            WorldHonorEliminateUpdate::RankUpdateBlocked {
                                player_id,
                                eliminator_id,
                                identity_payload_complete,
                                eliminate_counts,
                                counts_payload_complete,
                                source,
                            },
                        ),
                    );
                }
            };

            let mut acknowledgement = CMessage::new(HONOR_ELIMINATE_ACKNOWLEDGEMENT);
            acknowledgement.base_mut().add_ulong(player_id);
            acknowledgement.base_mut().add_char(1);
            let socket_id = message.socket_id();
            let sender = game.current_game_server_sender();
            let delivery = acknowledgement.send_to_socket(sender.as_ref(), socket_id);
            WorldOtherMessageDispatch::Handled(WorldOtherMessageOutcome::HonorEliminateUpdate(
                WorldHonorEliminateUpdate::Updated {
                    player_id,
                    eliminator_id,
                    identity_payload_complete,
                    eliminate_counts,
                    counts_payload_complete,
                    ranks,
                    acknowledgement_type: HONOR_ELIMINATE_ACKNOWLEDGEMENT,
                    acknowledgement: delivery,
                },
            ))
        }
        _ => WorldOtherMessageDispatch::Pending(message),
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\othermessage.cpp

// ============================================================================
// FUNCTION: OnOtherMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\othermessage.cpp:43
// RVA: 0x000AC680
// ADDRESS: 004ac680
// PROTOTYPE: void __cdecl OnOtherMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
