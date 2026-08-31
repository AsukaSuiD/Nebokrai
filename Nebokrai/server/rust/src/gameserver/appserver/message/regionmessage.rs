//! Владелец диспетчера сообщений региона GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/regionmessage.cpp`. Достигнутый `0x8F801` завершает
//! локальный `CPlayer::ChangeRegion`: проверяет действующий контекст игрока и
//! региона, снимает `m_bInChangingRegion`, переносит IP клиента, добавляет
//! игрока в пространственный реестр назначения, публикует снимки и погоду,
//! затем завершает состояния, спутников и WarSoul прямым владельцем.
//! `0x8F805` проверяет
//! прямое ребро канонического `RegionRouter`, диапазон перехода и
//! `CS_CHANGEREGION`, после чего входит в тот же полный владелец
//! `CPlayer::ChangeRegion` с игровыми, пространственными и локальными/World
//! wire-эффектами. Диагностический исход публикуется через `tracing` после
//! синхронного выполнения эффектов.

use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::shape::SHAPE_CHANGE_REGION;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::setup::regionrouter::RegionRoutePoint;
use tracing::{debug, trace};

const ENTER_CHANGED_REGION: u32 = 0x0008_f801;
const CHANGE_CONNECTED_REGION: u32 = 0x0008_f805;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameRegionMessageError {
    InvalidPayloadSize { expected: usize, actual: usize },
    MissingArgument,
}

pub(crate) fn dispatch_game_region_message<Context>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<(), GameRegionMessageError>>
where
    Context: ScriptFunctionRuntime,
{
    let message_type = message.message_type() as u32;
    if !matches!(message_type, ENTER_CHANGED_REGION | CHANGE_CONNECTED_REGION) {
        return None;
    }
    let actual = message
        .base_mut()
        .as_wire_bytes()
        .len()
        .saturating_sub(message.base_mut().cursor());
    if actual != 4 {
        return Some(Err(GameRegionMessageError::InvalidPayloadSize {
            expected: 4,
            actual,
        }));
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(argument) = message.base_mut().get_long() else {
        return Some(Err(GameRegionMessageError::MissingArgument));
    };
    if message_type == ENTER_CHANGED_REGION {
        let applied = match (player_id, region_id) {
            (Some(player_id), Some(region_id)) => game.enter_changed_player_region(
                player_id,
                region_id,
                argument,
                message.ip(),
                message.socket_id(),
                context,
            ),
            _ => false,
        };
        debug!(?player_id, ?region_id, applied, "завершён вход игрока в сменённый регион");
    } else {
        let target_region_id = argument;
        if let Some(player_id) = player_id {
            let player_facts = game.find_player(player_id).map(|player| {
                (
                    player.server_region_id(),
                    player.shape().get_tile_x(),
                    player.shape().get_tile_y(),
                    player.shape().get_direction(),
                    player.shape().change_state(),
                )
            });
            if let Some((Some(source_region_id), x, y, direction, change_state)) = player_facts {
                match x.and_then(|x| y.map(|y| RegionRoutePoint { x, y })) {
                    Err(block) => {
                        trace!(player_id, source_region_id, target_region_id, ?block, "смена связанного региона отклонена координатами");
                    }
                    Ok(current) => {
                        let destination = game.region_router().connected_region_destination(
                            source_region_id,
                            target_region_id,
                            current,
                        );
                        if let Some(destination) = destination {
                            if change_state == SHAPE_CHANGE_REGION {
                                trace!(player_id, source_region_id, target_region_id, "игрок уже меняет регион");
                            } else {
                                let _ = game.change_player_region(
                                    player_id,
                                    target_region_id,
                                    destination.x,
                                    destination.y,
                                    direction,
                                    0,
                                    0,
                                    0,
                                    context,
                                );
                                debug!(player_id, source_region_id, target_region_id, destination_x = destination.x, destination_y = destination.y, "запущена смена связанного региона");
                            }
                        } else {
                            trace!(player_id, source_region_id, target_region_id, "между регионами нет прямого перехода");
                        }
                    }
                }
            }
        }
    }
    Some(Ok(()))
}

// IMPLEMENTED: `CServerRegion::OnMessage` opcodes `0x8F801/0x8F805`
// материализованы выше; покрытый RAW и технический STL/SEH шум удалены.
