//! Владелец region-message dispatcher-а GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/regionmessage.cpp`. Достигнутый `0x8F801` завершает
//! локальный `CPlayer::ChangeRegion`: проверяет live player/region context,
//! снимает `m_bInChangingRegion`, переносит client IP, добавляет player в
//! destination spatial registry и лишь затем передаёт serialization/weather/
//! state tail runtime-owner-у. `0x8F805` проверяет direct edge canonical
//! `RegionRouter`, transition range и `CS_CHANGEREGION`, после чего входит в
//! тот же полный `CPlayer::ChangeRegion` owner с business/state/spatial и
//! local/World wire effects.

use crate::gameserver::appserver::script::function::ScriptFunctionRuntime;
use crate::gameserver::appserver::shape::{SHAPE_CHANGE_REGION, ShapeCoordinateBlock};
use crate::gameserver::gameserver::game::{
    CGame, GameRegionEnterContext, GameRegionEnterReport, PlayerRegionChangeReport,
};
use crate::nets::netserver::message::CMessage;
use crate::setup::regionrouter::RegionRoutePoint;

const ENTER_CHANGED_REGION: u32 = 0x0008_f801;
const CHANGE_CONNECTED_REGION: u32 = 0x0008_f805;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameRegionMessageError {
    InvalidPayloadSize { expected: usize, actual: usize },
    MissingArgument,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameRegionMessageReport {
    pub(crate) message_type: u32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) entry: Option<GameRegionEnterReport>,
    pub(crate) connected_change: Option<GameConnectedRegionChangeReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameConnectedRegionChangeOutcome {
    MissingContext,
    CoordinateBlocked(ShapeCoordinateBlock),
    NotConnected,
    AlreadyChanging,
    Dispatched,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameConnectedRegionChangeReport {
    pub(crate) player_id: Option<i32>,
    pub(crate) source_region_id: Option<i32>,
    pub(crate) target_region_id: i32,
    pub(crate) current: Option<RegionRoutePoint>,
    pub(crate) destination: Option<RegionRoutePoint>,
    pub(crate) outcome: GameConnectedRegionChangeOutcome,
    pub(crate) change: Option<PlayerRegionChangeReport>,
}

pub(crate) fn dispatch_game_region_message<
    Context: GameRegionEnterContext + ScriptFunctionRuntime,
>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<GameRegionMessageReport, GameRegionMessageError>> {
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
    let (entry, connected_change) = if message_type == ENTER_CHANGED_REGION {
        let entry = match (player_id, region_id) {
            (Some(player_id), Some(region_id)) => game.enter_changed_player_region(
                player_id,
                region_id,
                argument,
                message.ip(),
                message.socket_id(),
                context,
            ),
            _ => None,
        };
        (entry, None)
    } else {
        let target_region_id = argument;
        let mut report = GameConnectedRegionChangeReport {
            player_id,
            source_region_id: None,
            target_region_id,
            current: None,
            destination: None,
            outcome: GameConnectedRegionChangeOutcome::MissingContext,
            change: None,
        };
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
                report.source_region_id = Some(source_region_id);
                match x.and_then(|x| y.map(|y| RegionRoutePoint { x, y })) {
                    Err(block) => {
                        report.outcome = GameConnectedRegionChangeOutcome::CoordinateBlocked(block);
                    }
                    Ok(current) => {
                        report.current = Some(current);
                        report.destination = game.region_router().connected_region_destination(
                            source_region_id,
                            target_region_id,
                            current,
                        );
                        if let Some(destination) = report.destination {
                            if change_state == SHAPE_CHANGE_REGION {
                                report.outcome = GameConnectedRegionChangeOutcome::AlreadyChanging;
                            } else {
                                report.change = Some(game.change_player_region(
                                    player_id,
                                    target_region_id,
                                    destination.x,
                                    destination.y,
                                    direction,
                                    0,
                                    0,
                                    0,
                                    context,
                                ));
                                report.outcome = GameConnectedRegionChangeOutcome::Dispatched;
                            }
                        } else {
                            report.outcome = GameConnectedRegionChangeOutcome::NotConnected;
                        }
                    }
                }
            }
        }
        (None, Some(report))
    };
    Some(Ok(GameRegionMessageReport {
        message_type,
        player_id,
        region_id,
        entry,
        connected_change,
    }))
}

// IMPLEMENTED: `CServerRegion::OnMessage` opcodes `0x8F801/0x8F805`
// материализованы выше; покрытый RAW и технический STL/SEH шум удалены.
