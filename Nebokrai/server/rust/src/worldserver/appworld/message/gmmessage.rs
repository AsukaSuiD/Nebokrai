//! GM-сообщения `OnGMMessage` из `gmmessage.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Dispatcher безусловно читает request ID, затем обслуживает queries, relays,
//! region broadcast, reload, map kick, silence и ban для `0x5FF01..0x5FF16`.
//! Неизвестный opcode завершается после request ID без эффектов.
//! `0x5FF06` имеет достигнутого GameServer caller-а: script selector
//! `5001 / Reload` передаёт player ID как request ID и profile C-string;
//! dispatcher запускает тот же concrete reload owner, что и GM route.
//!
//! Online count/name queries отвечают исходному socket. Region query рассылает
//! `0x7FC04` всем; kick-map проходит регионы по фактическому ID. Silence меняет
//! player state до route, а missing target отвечает requester-у `WS0114`.
//!
//! Ban ищет account в полном player map, при пустом значении запрашивает БД и
//! только затем отправляет `0x20001` LoginServer. Requester не проверяется и
//! ответа не получает. Tiberius заменяет ADO, сохраняя этот порядок.
//! Сценарный запрос списка блокировок `0x5FF17` проверяет принадлежность игрока
//! исходному GameServer и передаёт ID игрока и сценария в LoginServer как
//! `0x20002`; отказ очереди немедленно возвращается исходной карте.

use std::ffi::CString;

use crate::dbaccess::worlddb::rsgodsbattle::TiberiusRsGodsBattle;
use crate::dbaccess::worlddb::rsplayer::{RsPlayerOwner, TiberiusRsPlayer};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::setup::godsbattleconf::CGodsBattleConf;
use crate::worldserver::appworld::jjcsystem::CJJcSystem;
use crate::worldserver::appworld::skills::skillfactory::CSkillFactory;
use crate::worldserver::worldserver::game::{
    CGame, WorldNamedRegionLookup, WorldRegionIdRouteScan, WorldReloadBlock, WorldReloadContext,
};

const ONLINE_PLAYER_COUNT_REQUEST: i32 = 0x0005_FF01;
const ONLINE_PLAYER_COUNT_RESPONSE: i32 = 0x0007_FC01;
const ONLINE_PLAYER_ID_REQUEST: i32 = 0x0005_FF05;
const ONLINE_PLAYER_ID_RESPONSE: i32 = 0x0007_FC05;
const ACTIVE_BAN_LIST_REQUEST: i32 = 0x0005_ff17;
const ACTIVE_BAN_LIST_LOGIN_REQUEST: i32 = 0x0002_0002;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmTransportOutcome {
    PlayerNameBroadcast {
        request_type: i32,
        response_type: i32,
        request_id: i32,
        player_name: Vec<u8>,
        online_player_id: u32,
        text: Vec<u8>,
        source_map_id: i32,
        value: i32,
        numeric_payload_complete: [bool; 2],
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    FreshMapRoute {
        request_type: i32,
        response_type: i32,
        request_id: i32,
        first_value: i32,
        target_map_id: i32,
        second_value: i32,
        payload_complete: [bool; 4],
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Broadcast {
        request_type: i32,
        response_type: i32,
        reused_request: bool,
        request_id_field: Option<i32>,
        appended_map_id: Option<i32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    PlayerRoute {
        request_type: i32,
        response_type: i32,
        player_id: i32,
        player_id_complete: bool,
        game_server_id: i32,
        wire: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    DirectMapRoute {
        request_type: i32,
        response_type: i32,
        request_id: i32,
        discarded_value: i32,
        target_map_id: i32,
        payload_complete: [bool; 3],
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    NamedPlayerRoute {
        request_type: i32,
        request_id: i32,
        request_id_complete: bool,
        player_name: Vec<u8>,
        online_player_id: u32,
        target_game_server_id: i32,
        disposition: WorldGmNamedPlayerRouteDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmNamedPlayerRouteDisposition {
    TargetRouted {
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterFallback {
        response_type: i32,
        requester_game_server_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterUnroutable,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmSilienceDisposition {
    TargetRouted {
        previous_silience_time: i32,
        target_game_server_id: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterFallback {
        requester_game_server_id: i32,
        response_type: i32,
        notice: Vec<u8>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterUnroutable,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmOnlinePlayerCountOutcome {
    CountOutsideLegacyRange {
        request_id: i32,
        script_id: i32,
        payload_complete: [bool; 2],
        count: usize,
    },
    Responded {
        request_id: i32,
        script_id: i32,
        payload_complete: [bool; 2],
        online_count: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmMessageOutcome {
    NoOp {
        request_type: i32,
        request_id: i32,
        request_id_complete: bool,
    },
    OnlinePlayerCount(WorldGmOnlinePlayerCountOutcome),
    OnlinePlayerId {
        request_id: i32,
        player_name: Vec<u8>,
        script_id: i32,
        numeric_payload_complete: [bool; 2],
        online_player_id: u32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RegionLookup {
        request_id: i32,
        region_name: Vec<u8>,
        script_id: i32,
        numeric_payload_complete: [bool; 2],
        lookup: WorldNamedRegionLookup,
        returned_region_id: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Reload {
        request_id: i32,
        request_id_complete: bool,
        profile: Vec<u8>,
        send_to_game_servers: bool,
        reload_server_resources: bool,
        result: Result<i32, WorldReloadBlock>,
    },
    KickMap {
        request_id: i32,
        region_id: i32,
        payload_complete: [bool; 2],
        response_type: i32,
        scan: WorldRegionIdRouteScan,
        wire: Option<Vec<u8>>,
        deliveries: Vec<Result<i32, SendMessageError>>,
    },
    Silience {
        request_id: i32,
        player_name: Vec<u8>,
        silience_time: i32,
        numeric_payload_complete: [bool; 2],
        online_player_id: u32,
        disposition: WorldGmSilienceDisposition,
    },
    Ban {
        request_id: i32,
        player_name: Vec<u8>,
        minutes: i32,
        payload_complete: [bool; 2],
        map_player_id: u32,
        map_player_found: bool,
        account_source: WorldGmBanAccountSource,
        account: Vec<u8>,
        response_type: i32,
        wire: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    ActiveBanListRequested {
        requester_player_id: i32,
        script_id: i32,
        source_map_id: i32,
        requester_map_id: i32,
        payload_complete: [bool; 2],
        wire: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
        failure_delivery: Option<Result<i32, SendMessageError>>,
    },
    Transport(WorldGmTransportOutcome),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGmBanAccountSource {
    MapPlayer,
    Database,
    Missing,
}

pub(crate) enum WorldGmMessageDispatch {
    Handled(WorldGmMessageOutcome),
    Pending(CMessage),
}

pub(crate) async fn on_gm_message(
    game: &mut CGame,
    jjc: &mut CJJcSystem,
    gods_battle: &mut CGodsBattleConf,
    skills: &mut CSkillFactory,
    rs_gods_battle: Option<&mut TiberiusRsGodsBattle>,
    rs_player: &mut TiberiusRsPlayer,
    player_database: Option<&mut WorldTdsClient>,
    reload_context: &mut dyn WorldReloadContext,
    mut message: CMessage,
) -> WorldGmMessageDispatch {
    let decoded_request_id = message.base_mut().get_long();
    let request_id = decoded_request_id.unwrap_or(0);
    match message.message_type() {
        ACTIVE_BAN_LIST_REQUEST => {
            let decoded_script_id = message.base_mut().get_long();
            let script_id = decoded_script_id.unwrap_or(0);
            let source_map_id = message.map_id();
            let requester_map_id = game.game_server_number_by_player_id(request_id);
            let (wire, delivery) =
                if request_id > 0 && script_id > 0 && requester_map_id == source_map_id {
                    let mut request = CMessage::new(ACTIVE_BAN_LIST_LOGIN_REQUEST);
                    request.base_mut().add_long(request_id);
                    request.base_mut().add_long(script_id);
                    let wire = request.as_wire_bytes().to_vec();
                    let delivery = request.send(
                        game.current_login_client()
                            .map(|client| client.send_queue()),
                        false,
                    );
                    (Some(wire), Some(delivery))
                } else {
                    (None, None)
                };
            let failure_delivery = if delivery
                .as_ref()
                .is_some_and(|result| !matches!(result, Ok(1)))
            {
                let mut failure = CMessage::new(0x0007_fc14);
                failure.base_mut().add_long(request_id);
                failure.base_mut().add_long(script_id);
                failure.base_mut().add_byte(0);
                failure.base_mut().add_long(0);
                failure.base_mut().add_byte(0);
                failure.base_mut().add_long(0);
                Some(game.send_msg_to_game_server(source_map_id, &failure))
            } else {
                None
            };
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::ActiveBanListRequested {
                requester_player_id: request_id,
                script_id,
                source_map_id,
                requester_map_id,
                payload_complete: [decoded_request_id.is_some(), decoded_script_id.is_some()],
                wire,
                delivery,
                failure_delivery,
            })
        }
        ONLINE_PLAYER_COUNT_REQUEST => {
            let decoded_script_id = message.base_mut().get_long();
            let script_id = decoded_script_id.unwrap_or(0);
            let payload_complete = [decoded_request_id.is_some(), decoded_script_id.is_some()];
            let count = game.online_player_count();
            let Ok(online_count_bits) = u32::try_from(count) else {
                return WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::OnlinePlayerCount(
                    WorldGmOnlinePlayerCountOutcome::CountOutsideLegacyRange {
                        request_id,
                        script_id,
                        payload_complete,
                        count,
                    },
                ));
            };
            let online_count = online_count_bits as i32;

            let mut response = CMessage::new(ONLINE_PLAYER_COUNT_RESPONSE);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(online_count);
            response.base_mut().add_long(script_id);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_socket(
                game.current_game_server_sender().as_ref(),
                message.socket_id(),
            );
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::OnlinePlayerCount(
                WorldGmOnlinePlayerCountOutcome::Responded {
                    request_id,
                    script_id,
                    payload_complete,
                    online_count,
                    response_type: ONLINE_PLAYER_COUNT_RESPONSE,
                    wire,
                    delivery,
                },
            ))
        }
        ONLINE_PLAYER_ID_REQUEST => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_script_id = message.base_mut().get_long();
            let script_id = decoded_script_id.unwrap_or(0);
            let online_player_id = game.online_player_id_by_name(&player_name);

            let mut response = CMessage::new(ONLINE_PLAYER_ID_RESPONSE);
            response.base_mut().add_long(request_id);
            response.base_mut().add_ulong(online_player_id);
            response.base_mut().add_long(script_id);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_socket(
                game.current_game_server_sender().as_ref(),
                message.socket_id(),
            );
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::OnlinePlayerId {
                request_id,
                player_name,
                script_id,
                numeric_payload_complete: [
                    decoded_request_id.is_some(),
                    decoded_script_id.is_some(),
                ],
                online_player_id,
                response_type: ONLINE_PLAYER_ID_RESPONSE,
                wire,
                delivery,
            })
        }
        0x0005_FF06 => {
            let profile = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let result = game
                .reload(
                    reload_context,
                    jjc,
                    gods_battle,
                    skills,
                    rs_gods_battle,
                    &profile,
                    true,
                    true,
                )
                .await;
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Reload {
                request_id,
                request_id_complete: decoded_request_id.is_some(),
                profile,
                send_to_game_servers: true,
                reload_server_resources: true,
                result,
            })
        }
        0x0005_FF02 => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let text = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_value = message.base_mut().get_long();
            let value = decoded_value.unwrap_or(0);
            let online_player_id = game.online_player_id_by_name(&player_name);
            let source_map_id = message.map_id();

            let mut response = CMessage::new(0x0007_FC03);
            response.base_mut().add_long(request_id);
            response.base_mut().add_ulong(online_player_id);
            add_c_string(&mut response, &text);
            response.base_mut().add_long(source_map_id);
            response.base_mut().add_long(value);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_all(game.current_game_server_sender().as_ref());
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::PlayerNameBroadcast {
                    request_type: 0x0005_FF02,
                    response_type: 0x0007_FC03,
                    request_id,
                    player_name,
                    online_player_id,
                    text,
                    source_map_id,
                    value,
                    numeric_payload_complete: [
                        decoded_request_id.is_some(),
                        decoded_value.is_some(),
                    ],
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FF03 => {
            let decoded_first = message.base_mut().get_long();
            let decoded_target = message.base_mut().get_long();
            let decoded_second = message.base_mut().get_long();
            let first_value = decoded_first.unwrap_or(0);
            let target_map_id = decoded_target.unwrap_or(0);
            let second_value = decoded_second.unwrap_or(0);
            let mut response = CMessage::new(0x0007_FC02);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(first_value);
            response.base_mut().add_long(second_value);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(target_map_id, &response);
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::FreshMapRoute {
                    request_type: 0x0005_FF03,
                    response_type: 0x0007_FC02,
                    request_id,
                    first_value,
                    target_map_id,
                    second_value,
                    payload_complete: [
                        decoded_request_id.is_some(),
                        decoded_first.is_some(),
                        decoded_target.is_some(),
                        decoded_second.is_some(),
                    ],
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FF04 => {
            let region_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_script_id = message.base_mut().get_long();
            let script_id = decoded_script_id.unwrap_or(0);
            let lookup = game.named_region_lookup(&region_name);
            let returned_region_id = lookup
                .matched
                .filter(|matched| matched.game_server_connected)
                .map_or(0, |matched| matched.region_id);

            let mut response = CMessage::new(0x0007_FC04);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(returned_region_id);
            response.base_mut().add_long(script_id);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_all(game.current_game_server_sender().as_ref());
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::RegionLookup {
                request_id,
                region_name,
                script_id,
                numeric_payload_complete: [
                    decoded_request_id.is_some(),
                    decoded_script_id.is_some(),
                ],
                lookup,
                returned_region_id,
                response_type: 0x0007_FC04,
                wire,
                delivery,
            })
        }
        0x0005_FF07 => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let online_player_id = game.online_player_id_by_name(&player_name);
            let target_game_server_id =
                game.game_server_number_by_player_id(online_player_id as i32);
            let disposition = if target_game_server_id != 0 {
                message.set_message_type(0x0007_FC06);
                let wire = message.as_wire_bytes().to_vec();
                let delivery = message.send_to_map_id(
                    game.current_game_server_sender().as_ref(),
                    target_game_server_id,
                );
                WorldGmNamedPlayerRouteDisposition::TargetRouted {
                    response_type: 0x0007_FC06,
                    wire,
                    delivery,
                }
            } else {
                let requester_game_server_id = game.game_server_number_by_player_id(request_id);
                if requester_game_server_id == 0 {
                    WorldGmNamedPlayerRouteDisposition::RequesterUnroutable
                } else {
                    let mut response = CMessage::new(0x0007_FC08);
                    response.base_mut().add_long(request_id);
                    response.base_mut().add_char(0);
                    add_c_string(&mut response, &player_name);
                    let wire = response.as_wire_bytes().to_vec();
                    let delivery = response.send_to_map_id(
                        game.current_game_server_sender().as_ref(),
                        requester_game_server_id,
                    );
                    WorldGmNamedPlayerRouteDisposition::RequesterFallback {
                        response_type: 0x0007_FC08,
                        requester_game_server_id,
                        wire,
                        delivery,
                    }
                }
            };
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::NamedPlayerRoute {
                    request_type: 0x0005_FF07,
                    request_id,
                    request_id_complete: decoded_request_id.is_some(),
                    player_name,
                    online_player_id,
                    target_game_server_id,
                    disposition,
                },
            ))
        }
        0x0005_FF08 => {
            let map_id = message.map_id();
            message.set_message_type(0x0007_FC07);
            message.base_mut().add_long(map_id);
            handled_broadcast(
                game,
                message,
                0x0005_FF08,
                0x0007_FC07,
                true,
                None,
                Some(map_id),
            )
        }
        0x0005_FF09 => handled_player_route(
            game,
            message,
            0x0005_FF09,
            0x0007_FC08,
            request_id,
            decoded_request_id.is_some(),
        ),
        0x0005_FF0A => {
            let mut response = CMessage::new(0x0007_FC09);
            response.base_mut().add_long(request_id);
            handled_broadcast(
                game,
                response,
                0x0005_FF0A,
                0x0007_FC09,
                false,
                Some(request_id),
                None,
            )
        }
        0x0005_FF0B => {
            let decoded_region_id = message.base_mut().get_long();
            let region_id = decoded_region_id.unwrap_or(0);
            let scan = game.region_routes_by_owner_id(region_id);
            let wire = (!scan.routes.is_empty()).then(|| {
                message.set_message_type(0x0007_FC0A);
                message.as_wire_bytes().to_vec()
            });
            let deliveries = scan
                .routes
                .iter()
                .map(|route| {
                    message.send_to_map_id(
                        game.current_game_server_sender().as_ref(),
                        route.game_server_id,
                    )
                })
                .collect();
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::KickMap {
                request_id,
                region_id,
                payload_complete: [decoded_request_id.is_some(), decoded_region_id.is_some()],
                response_type: 0x0007_FC0A,
                scan,
                wire,
                deliveries,
            })
        }
        0x0005_FF0C => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_silience_time = message.base_mut().get_long();
            let silience_time = decoded_silience_time.unwrap_or(0);
            let online_player_id = game.online_player_id_by_name(&player_name);
            let previous_silience_time =
                game.replace_online_player_silience_time(online_player_id, silience_time);
            let disposition = if let Some(previous_silience_time) = previous_silience_time {
                message.set_message_type(0x0007_FC0B);
                let target_game_server_id =
                    game.game_server_number_by_player_id(online_player_id as i32);
                let wire = message.as_wire_bytes().to_vec();
                let delivery = message.send_to_map_id(
                    game.current_game_server_sender().as_ref(),
                    target_game_server_id,
                );
                WorldGmSilienceDisposition::TargetRouted {
                    previous_silience_time,
                    target_game_server_id,
                    response_type: 0x0007_FC0B,
                    wire,
                    delivery,
                }
            } else {
                let requester_game_server_id = game.game_server_number_by_player_id(request_id);
                if requester_game_server_id == 0 {
                    WorldGmSilienceDisposition::RequesterUnroutable
                } else {
                    let notice = game.get_string_by_id(b"WS0114").to_vec();
                    let mut response = CMessage::new(0x0007_FC0C);
                    response.base_mut().add_long(request_id);
                    add_c_string(&mut response, &player_name);
                    response.base_mut().add_long(silience_time);
                    response.base_mut().add_char(0);
                    add_c_string(&mut response, &notice);
                    let wire = response.as_wire_bytes().to_vec();
                    let delivery = response.send_to_map_id(
                        game.current_game_server_sender().as_ref(),
                        requester_game_server_id,
                    );
                    WorldGmSilienceDisposition::RequesterFallback {
                        requester_game_server_id,
                        response_type: 0x0007_FC0C,
                        notice,
                        wire,
                        delivery,
                    }
                }
            };
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Silience {
                request_id,
                player_name,
                silience_time,
                numeric_payload_complete: [
                    decoded_request_id.is_some(),
                    decoded_silience_time.is_some(),
                ],
                online_player_id,
                disposition,
            })
        }
        0x0005_FF0D => handled_player_route(
            game,
            message,
            0x0005_FF0D,
            0x0007_FC0C,
            request_id,
            decoded_request_id.is_some(),
        ),
        0x0005_FF0E => handled_rewritten_broadcast(game, message, 0x0005_FF0E, 0x0007_FC0D),
        0x0005_FF0F => handled_rewritten_broadcast(game, message, 0x0005_FF0F, 0x0007_FC0E),
        0x0005_FF10 => handled_player_route(
            game,
            message,
            0x0005_FF10,
            0x0007_FC0F,
            request_id,
            decoded_request_id.is_some(),
        ),
        0x0005_FF11 => handled_rewritten_broadcast(game, message, 0x0005_FF11, 0x0007_FC10),
        0x0005_FF12 => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_minutes = message.base_mut().get_long();
            let minutes = decoded_minutes.unwrap_or(0);
            let map_player_id = game.map_player_id_by_name(&player_name);
            let map_player = game.map_player(map_player_id);
            let map_player_found = map_player.is_some();
            let mut account = map_player
                .map(|player| legacy_c_string_prefix(player.get_account()).to_vec())
                .unwrap_or_default();
            let mut account_source = if account.is_empty() {
                WorldGmBanAccountSource::Missing
            } else {
                WorldGmBanAccountSource::MapPlayer
            };
            if account.is_empty() {
                account = rs_player.get_cd_key(&player_name, player_database).await;
                if !account.is_empty() {
                    account_source = WorldGmBanAccountSource::Database;
                }
            }

            let (wire, delivery) = if account.is_empty() {
                (None, None)
            } else {
                let mut response = CMessage::new(0x0002_0001);
                add_c_string(&mut response, &account);
                response.base_mut().add_long(minutes);
                let wire = response.as_wire_bytes().to_vec();
                let delivery = response.send(
                    game.current_login_client()
                        .map(|client| client.send_queue()),
                    false,
                );
                (Some(wire), Some(delivery))
            };
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Ban {
                request_id,
                player_name,
                minutes,
                payload_complete: [decoded_request_id.is_some(), decoded_minutes.is_some()],
                map_player_id,
                map_player_found,
                account_source,
                account,
                response_type: 0x0002_0001,
                wire,
                delivery,
            })
        }
        0x0005_FF13 => handled_broadcast(
            game,
            CMessage::new(0x0007_F803),
            0x0005_FF13,
            0x0007_F803,
            false,
            None,
            None,
        ),
        0x0005_FF14 => {
            let map_id = message.map_id();
            let mut response = CMessage::new(0x0007_FC11);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(map_id);
            handled_broadcast(
                game,
                response,
                0x0005_FF14,
                0x0007_FC11,
                false,
                Some(request_id),
                Some(map_id),
            )
        }
        0x0005_FF15 => {
            let decoded_discarded = message.base_mut().get_long();
            let decoded_target = message.base_mut().get_long();
            let discarded_value = decoded_discarded.unwrap_or(0);
            let target_map_id = decoded_target.unwrap_or(0);
            message.set_message_type(0x0007_FC12);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(target_map_id, &message);
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::DirectMapRoute {
                    request_type: 0x0005_FF15,
                    response_type: 0x0007_FC12,
                    request_id,
                    discarded_value,
                    target_map_id,
                    payload_complete: [
                        decoded_request_id.is_some(),
                        decoded_discarded.is_some(),
                        decoded_target.is_some(),
                    ],
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FF16 => handled_rewritten_broadcast(game, message, 0x0005_FF16, 0x0007_FC13),
        request_type => WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::NoOp {
            request_type,
            request_id,
            request_id_complete: decoded_request_id.is_some(),
        }),
    }
}

fn handled_rewritten_broadcast(
    game: &CGame,
    mut message: CMessage,
    request_type: i32,
    response_type: i32,
) -> WorldGmMessageDispatch {
    message.set_message_type(response_type);
    handled_broadcast(game, message, request_type, response_type, true, None, None)
}

fn handled_broadcast(
    game: &CGame,
    message: CMessage,
    request_type: i32,
    response_type: i32,
    reused_request: bool,
    request_id_field: Option<i32>,
    appended_map_id: Option<i32>,
) -> WorldGmMessageDispatch {
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
        WorldGmTransportOutcome::Broadcast {
            request_type,
            response_type,
            reused_request,
            request_id_field,
            appended_map_id,
            wire,
            delivery,
        },
    ))
}

fn handled_player_route(
    game: &CGame,
    mut message: CMessage,
    request_type: i32,
    response_type: i32,
    player_id: i32,
    player_id_complete: bool,
) -> WorldGmMessageDispatch {
    let game_server_id = game.game_server_number_by_player_id(player_id);
    let (wire, delivery) = if game_server_id == 0 {
        (None, None)
    } else {
        message.set_message_type(response_type);
        let wire = message.as_wire_bytes().to_vec();
        let delivery =
            message.send_to_map_id(game.current_game_server_sender().as_ref(), game_server_id);
        (Some(wire), Some(delivery))
    };
    WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
        WorldGmTransportOutcome::PlayerRoute {
            request_type,
            response_type,
            player_id,
            player_id_complete,
            game_server_id,
            wire,
            delivery,
        },
    ))
}

fn add_c_string(message: &mut CMessage, bytes: &[u8]) {
    let value = CString::new(bytes).expect("bounded GetStr возвращает bytes до первого NUL");
    message.base_mut().add_str(Some(&value));
}

fn legacy_c_string_prefix(bytes: &[u8]) -> &[u8] {
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..length]
}
