//! Достигнутый вход открытия personal shop GameServer.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и owner
//! `server/gameserver/appserver/message/playershopmessage.cpp` подтверждают
//! `0x90201`: guards changing-state/death/progress, region block `2`, local
//! seller-session либо World auction arbitration, client `0xC0001` и ordered
//! `0x60812` completion. Rust handler подключён к реальному message FIFO;
//! malformed coordinate/payload выражены typed error вместо invalid access.
//! Остальные shop selectors остаются RAW ниже до своих вертикальных сценариев.

use crate::gameserver::appserver::player::PlayerProgress;
use crate::gameserver::appserver::region::RegionCellAccessBlock;
use crate::gameserver::appserver::shape::ShapeCoordinateBlock;
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::{CMessage, SendMessageError};

const PLAYER_SHOP_OPEN_MESSAGE: i32 = 0x0009_0201;
const CLIENT_PLAYER_SHOP_OPENED_MESSAGE: i32 = 0x000c_0001;
const WORLD_PLAYER_SHOP_REQUEST_MESSAGE: i32 = 0x0006_0811;
const WORLD_PLAYER_SHOP_OVER_MESSAGE: i32 = 0x0006_0812;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopIgnoreReason {
    MissingPlayerContext,
    MissingPlayer,
    ChangingServer,
    ChangingRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopMessageError {
    MissingSourceWorldServerId,
    Coordinate(ShapeCoordinateBlock),
    RegionCell(RegionCellAccessBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopOpenOutcome {
    Dead,
    ProgressConflict,
    MissingRegion,
    OutsideStallArea,
    SessionCreationFailed,
    LocalSession {
        session_id: i32,
        plug_id: i32,
        seller_volume: u32,
        seller_extend_id: i32,
        seller_shop_opened: bool,
        seller_name_empty: bool,
        listener_attach: [bool; 2],
        client_delivery: i32,
    },
    WorldRequested,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerShopMessageReport {
    Ignored {
        player_id: Option<i32>,
        reason: PlayerShopIgnoreReason,
    },
    Opened {
        player_id: i32,
        source_world_server_id: i32,
        outcome: PlayerShopOpenOutcome,
        notice_delivery: Option<i32>,
        world_request: Option<Result<i32, SendMessageError>>,
        world_completion: Option<Result<i32, SendMessageError>>,
    },
}

pub(crate) fn dispatch_player_shop_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<PlayerShopMessageReport, PlayerShopMessageError>> {
    if message.message_type() != PLAYER_SHOP_OPEN_MESSAGE {
        return None;
    }

    message.resolve_player_context(game);
    let Some(player_id) = message.player_id() else {
        return Some(Ok(PlayerShopMessageReport::Ignored {
            player_id: None,
            reason: PlayerShopIgnoreReason::MissingPlayerContext,
        }));
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(Ok(PlayerShopMessageReport::Ignored {
            player_id: Some(player_id),
            reason: PlayerShopIgnoreReason::MissingPlayer,
        }));
    };
    if player.in_changing_server() || player.in_changing_region() {
        return Some(Ok(PlayerShopMessageReport::Ignored {
            player_id: Some(player_id),
            reason: if player.in_changing_server() {
                PlayerShopIgnoreReason::ChangingServer
            } else {
                PlayerShopIgnoreReason::ChangingRegion
            },
        }));
    }

    let Some(source_world_server_id) = message.base_mut().get_long() else {
        return Some(Err(PlayerShopMessageError::MissingSourceWorldServerId));
    };
    Some(open_player_shop(
        message,
        game,
        player_id,
        source_world_server_id,
    ))
}

fn open_player_shop(
    message: &CMessage,
    game: &mut CGame,
    player_id: i32,
    source_world_server_id: i32,
) -> Result<PlayerShopMessageReport, PlayerShopMessageError> {
    let (_, world_server_id) = game.server_ids();
    let (dead, progress, region_id) = {
        let player = game
            .find_player(player_id)
            .expect("player проверен до открытия personal shop");
        (
            player.is_dead(),
            player.current_progress(),
            message.region_id(),
        )
    };

    let mut notice_delivery = None;
    let mut world_request = None;
    let mut world_completion = None;
    let outcome = if dead {
        PlayerShopOpenOutcome::Dead
    } else if progress != PlayerProgress::None {
        notice_delivery = Some(
            colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0072"))
                .send_to_player(game.net_server(), player_id),
        );
        PlayerShopOpenOutcome::ProgressConflict
    } else if let Some(region_id) = region_id {
        let Some(region) = game.find_region(region_id) else {
            if source_world_server_id == world_server_id {
                world_completion = Some(send_world_shop_completion(game, player_id));
            }
            return Ok(PlayerShopMessageReport::Opened {
                player_id,
                source_world_server_id,
                outcome: PlayerShopOpenOutcome::MissingRegion,
                notice_delivery,
                world_request,
                world_completion,
            });
        };
        let (tile_x, tile_y) = {
            let player = game
                .find_player(player_id)
                .expect("player проверен перед region lookup");
            (
                player
                    .shape()
                    .get_tile_x()
                    .map_err(PlayerShopMessageError::Coordinate)?,
                player
                    .shape()
                    .get_tile_y()
                    .map_err(PlayerShopMessageError::Coordinate)?,
            )
        };
        let block = region
            .base()
            .region
            .get_block(tile_x, tile_y)
            .map_err(PlayerShopMessageError::RegionCell)?;
        if block != 2 {
            notice_delivery = Some(
                colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS0073"))
                    .send_to_player(game.net_server(), player_id),
            );
            if source_world_server_id == world_server_id {
                world_completion = Some(send_world_shop_completion(game, player_id));
            }
            return Ok(PlayerShopMessageReport::Opened {
                player_id,
                source_world_server_id,
                outcome: PlayerShopOpenOutcome::OutsideStallArea,
                notice_delivery,
                world_request,
                world_completion,
            });
        }

        if !game.globe_setup().auction_enabled() || source_world_server_id != 0 {
            let Some((session_id, plug_id)) = game
                .session_factory_mut()
                .create_personal_shop_seller_session(player_id)
            else {
                return Ok(PlayerShopMessageReport::Opened {
                    player_id,
                    source_world_server_id,
                    outcome: PlayerShopOpenOutcome::SessionCreationFailed,
                    notice_delivery,
                    world_request,
                    world_completion,
                });
            };
            let seller = game
                .session_factory()
                .personal_shop_seller(plug_id)
                .expect("personal-shop factory публикует typed seller");
            let seller_volume = seller.goods().size();
            let seller_extend_id = seller.goods().base().base().container_extend_id();
            let seller_shop_opened = seller.shop_opened();
            let seller_name_empty = seller.shop_name().is_empty();
            let listener_attach = {
                let player = game
                    .find_player_mut(player_id)
                    .expect("player жив во время session insertion");
                let listener_attach = player.attach_equipment_session_listener(plug_id);
                player.set_current_progress_snapshot(PlayerProgress::OpenStall);
                listener_attach
            };
            let mut response = CMessage::new(CLIENT_PLAYER_SHOP_OPENED_MESSAGE);
            response.base_mut().add_long(session_id);
            response.base_mut().add_long(plug_id);
            let client_delivery = response.send_to_player(game.net_server(), player_id);
            PlayerShopOpenOutcome::LocalSession {
                session_id,
                plug_id,
                seller_volume,
                seller_extend_id,
                seller_shop_opened,
                seller_name_empty,
                listener_attach,
                client_delivery,
            }
        } else {
            let mut request = CMessage::new(WORLD_PLAYER_SHOP_REQUEST_MESSAGE);
            request.base_mut().add_long(player_id);
            request.base_mut().add_ulong(message.ip());
            world_request = Some(request.send(game, false));
            PlayerShopOpenOutcome::WorldRequested
        }
    } else {
        PlayerShopOpenOutcome::MissingRegion
    };

    if matches!(
        outcome,
        PlayerShopOpenOutcome::Dead
            | PlayerShopOpenOutcome::ProgressConflict
            | PlayerShopOpenOutcome::MissingRegion
    ) && source_world_server_id == world_server_id
    {
        world_completion = Some(send_world_shop_completion(game, player_id));
    }
    Ok(PlayerShopMessageReport::Opened {
        player_id,
        source_world_server_id,
        outcome,
        notice_delivery,
        world_request,
        world_completion,
    })
}

fn send_world_shop_completion(game: &CGame, player_id: i32) -> Result<i32, SendMessageError> {
    let client_ip = game
        .find_player(player_id)
        .expect("completion относится к live player")
        .client_ip();
    let mut completion = CMessage::new(WORLD_PLAYER_SHOP_OVER_MESSAGE);
    completion.base_mut().add_long(player_id);
    completion.base_mut().add_ulong(client_ip);
    completion.send(game, false)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\playershopmessage.cpp

// ============================================================================
// FUNCTION: OnPlayerShopMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\playershopmessage.cpp:21
// RVA: 0x0008B310
// ADDRESS: 0048b310
// PROTOTYPE: void __cdecl OnPlayerShopMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
