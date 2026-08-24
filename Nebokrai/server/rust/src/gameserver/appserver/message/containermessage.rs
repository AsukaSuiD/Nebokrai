//! Входной container dispatcher исторического GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/message/containermessage.cpp`. Материализован
//! полный player packet/equipment → enhancement-shadow проход `0x90301`:
//! одиннадцать wire-полей, outer changing/region/progress/death guards,
//! Receive-нормализация owner ID, точный source position/GUID/amount,
//! запрет stackable goods, однослотовый AddShadow, last-operated state и обе
//! адресные `0xC0101` публикации. Вторая self-move публикация нормализуется в
//! `OT_ROLL_BACK`, сохраняя исходный goods в его source slot. Shadow не
//! забирает ownership исходного goods;
//! native remove→re-add свёрнут в атомарную metadata-запись, поэтому отказ
//! эквивалентен успешному rollback без промежуточной потери предмета.
//!
//! Остальные container paths owner-а остаются RAW ниже и после восстановления
//! cursor продолжают проходить через прежнюю общую handler-границу.

use crate::gameserver::appserver::message::containermessage::EnhancementMoveReceiveBlock::{
    InvalidExtendId, InvalidObjectType, SameContainer, ZeroAmount,
};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::{
    EnhancementSelectionBlock, EnhancementSelectionReport, PlayerProgress,
};
use crate::gameserver::gameserver::game::{CGame, OldClientGoodsCodec};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const CONTAINER_OBJECT_MOVE: u32 = 0x0009_0301;
const CLIENT_CONTAINER_OBJECT_MOVE: i32 = 0x000c_0101;
const PLAYER_CONTAINER_TYPE: i32 = 400;
const GOODS_OBJECT_TYPE: i32 = 700;
const ENHANCEMENT_EXTEND_ID: i32 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameContainerMessageError {
    MissingField(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContainerObjectMoveRequest {
    pub(crate) source_container_type: i32,
    pub(crate) source_container_id: i32,
    pub(crate) source_container_extend_id: i32,
    pub(crate) source_position: u32,
    pub(crate) destination_container_type: i32,
    pub(crate) destination_container_id: i32,
    pub(crate) destination_container_extend_id: i32,
    pub(crate) destination_position: u32,
    pub(crate) object_type: i32,
    pub(crate) object_id: CGuid,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnhancementMoveReceiveBlock {
    InvalidObjectType,
    ZeroAmount,
    InvalidExtendId,
    SameContainer,
    ForbiddenRoute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameContainerMessageOutcome {
    MissingPlayer,
    MissingRegion,
    ChangingServer,
    ChangingRegion,
    Synthesis {
        notification_delivery: i32,
    },
    Died {
        notification_delivery: i32,
    },
    ReceiveRejected(EnhancementMoveReceiveBlock),
    RolledBack {
        reason: EnhancementSelectionBlock,
        delivery: i32,
    },
    EnhancementSelected {
        selection: EnhancementSelectionReport,
        old_client_payload: Vec<u8>,
        add_shadow_delivery: i32,
        move_delivery: i32,
    },
}

#[must_use = "container report сохраняет request, mutation и ordered client effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameContainerMessageReport {
    pub(crate) message_type: u32,
    pub(crate) socket_id: i32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) request: Option<ContainerObjectMoveRequest>,
    pub(crate) outcome: GameContainerMessageOutcome,
}

pub(crate) fn dispatch_game_container_message<Context: OldClientGoodsCodec>(
    message: &mut CMessage,
    game: &mut CGame,
    context: &mut Context,
) -> Option<Result<GameContainerMessageReport, GameContainerMessageError>> {
    let message_type = message.message_type() as u32;
    if message_type != CONTAINER_OBJECT_MOVE {
        return None;
    }

    message.resolve_player_context(game);
    let socket_id = message.socket_id();
    let player_id = message.player_id();
    let region_id = message.region_id();
    let Some(player_id) = player_id else {
        return Some(Ok(GameContainerMessageReport {
            message_type,
            socket_id,
            player_id: None,
            region_id,
            request: None,
            outcome: GameContainerMessageOutcome::MissingPlayer,
        }));
    };

    let start_cursor = message.base_mut().cursor();
    let request = match decode_container_object_move(message) {
        Ok(mut request) => {
            if request.source_container_type != PLAYER_CONTAINER_TYPE
                || request.destination_container_type != PLAYER_CONTAINER_TYPE
                || request.destination_container_extend_id != ENHANCEMENT_EXTEND_ID
            {
                let (_, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
                *cursor = start_cursor;
                return None;
            }
            request.source_container_id = player_id;
            request.destination_container_id = player_id;
            if matches!(request.source_container_extend_id, 3 | 4 | 5) {
                request.source_position = 0;
            }
            request
        }
        Err(error) => return Some(Err(error)),
    };

    let report = |outcome| GameContainerMessageReport {
        message_type,
        socket_id,
        player_id: Some(player_id),
        region_id,
        request: Some(request),
        outcome,
    };
    let Some(player) = game.find_player(player_id) else {
        return Some(Ok(report(GameContainerMessageOutcome::MissingPlayer)));
    };
    if player.in_changing_server() {
        return Some(Ok(report(GameContainerMessageOutcome::ChangingServer)));
    }
    if player.in_changing_region() {
        return Some(Ok(report(GameContainerMessageOutcome::ChangingRegion)));
    }
    if region_id.is_none() {
        return Some(Ok(report(GameContainerMessageOutcome::MissingRegion)));
    }
    if player.current_progress() == PlayerProgress::Synthesis {
        let text = game.get_string_by_id(b"GS1013").to_vec();
        let delivery = send_notify(game, player_id, &text, 0xffff_0000, 0);
        return Some(Ok(report(GameContainerMessageOutcome::Synthesis {
            notification_delivery: delivery,
        })));
    }
    if CMoveShape::is_died(player.health()) {
        let delivery = send_notify(
            game,
            player_id,
            b"you can`t pick up prop after died! ",
            0xffff_ffff,
            0,
        );
        return Some(Ok(report(GameContainerMessageOutcome::Died {
            notification_delivery: delivery,
        })));
    }
    if request.object_type != GOODS_OBJECT_TYPE {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            InvalidObjectType,
        ))));
    }
    if request.amount == 0 {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            ZeroAmount,
        ))));
    }
    if !(0..=17).contains(&request.source_container_extend_id)
        || !(0..=17).contains(&request.destination_container_extend_id)
    {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            InvalidExtendId,
        ))));
    }
    if request.source_container_extend_id == request.destination_container_extend_id {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            SameContainer,
        ))));
    }
    if request.source_container_extend_id == 4
        || request.source_container_extend_id == 5
        || matches!(request.source_container_extend_id, 8 | 15)
    {
        return Some(Ok(report(GameContainerMessageOutcome::ReceiveRejected(
            EnhancementMoveReceiveBlock::ForbiddenRoute,
        ))));
    }

    let selection = game.select_player_enhancement_goods(
        player_id,
        request.source_container_extend_id,
        request.source_position,
        request.object_id,
        request.amount,
    );
    let selection = match selection {
        Ok(selection) => selection,
        Err(reason) => {
            let delivery = send_rollback(game, player_id);
            return Some(Ok(report(GameContainerMessageOutcome::RolledBack {
                reason,
                delivery,
            })));
        }
    };

    let goods = game
        .find_player(player_id)
        .and_then(|player| player.get_goods_by_id(selection.goods.ex_id))
        .expect("enhancement shadow сохраняет live source goods");
    let old_client_payload = context.encode_goods_for_old_client(goods);
    let add_shadow_delivery = send_add_shadow(game, player_id, &selection, &old_client_payload);
    let move_delivery = send_move_result(game, player_id, request, &selection);
    Some(Ok(report(
        GameContainerMessageOutcome::EnhancementSelected {
            selection,
            old_client_payload,
            add_shadow_delivery,
            move_delivery,
        },
    )))
}

fn decode_container_object_move(
    message: &mut CMessage,
) -> Result<ContainerObjectMoveRequest, GameContainerMessageError> {
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(GameContainerMessageError::MissingField(field))
    };
    let source_container_type = read_long(message, "source container type")?;
    let source_container_id = read_long(message, "source container ID")?;
    let source_container_extend_id = read_long(message, "source container extend ID")?;
    let source_position = read_long(message, "source position")? as u32;
    let destination_container_type = read_long(message, "destination container type")?;
    let destination_container_id = read_long(message, "destination container ID")?;
    let destination_container_extend_id = read_long(message, "destination container extend ID")?;
    let destination_position = read_long(message, "destination position")? as u32;
    let object_type = read_long(message, "object type")?;
    let object_id_cursor = message.base_mut().cursor();
    let object_id = match message.base_mut().get_guid() {
        Some(guid) => guid,
        None if message.base_mut().cursor() == object_id_cursor + 1 => CGuid::GUID_INVALID,
        None => return Err(GameContainerMessageError::MissingField("object GUID")),
    };
    let amount = read_long(message, "amount")? as u32;
    Ok(ContainerObjectMoveRequest {
        source_container_type,
        source_container_id,
        source_container_extend_id,
        source_position,
        destination_container_type,
        destination_container_id,
        destination_container_extend_id,
        destination_position,
        object_type,
        object_id,
        amount,
    })
}

fn send_notify(game: &CGame, player_id: i32, text: &[u8], first: u32, second: u32) -> i32 {
    let mut message = CMessage::new(0x000b_f806);
    message.add_ulong(first);
    message.add_ulong(second);
    message.base_mut().add(text);
    message.add_byte(0);
    message.send_to_player(game.net_server(), player_id)
}

fn send_rollback(game: &CGame, player_id: i32) -> i32 {
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(0);
    message.send_to_player(game.net_server(), player_id)
}

fn send_add_shadow(
    game: &CGame,
    player_id: i32,
    selection: &EnhancementSelectionReport,
    payload: &[u8],
) -> i32 {
    let presence = &selection.shadow.presence;
    let mut message = CMessage::new(CLIENT_CONTAINER_OBJECT_MOVE);
    message.add_byte(2);
    message.add_long(0);
    message.add_long(0);
    message.add_long(0);
    message.add_ulong(0);
    message.add_long(presence.owner_type);
    message.add_long(presence.owner_id);
    message.add_long(presence.container_extend_id);
    message.add_ulong(presence.position);
    message.add_long(0);
    message.base_mut().add_guid(CGuid::GUID_INVALID);
    message.add_long(GOODS_OBJECT_TYPE);
    message.base_mut().add_guid(selection.goods.ex_id);
    message.add_ulong(payload.len() as u32);
    message.base_mut().add(payload);
    message.send_to_player(game.net_server(), player_id)
}

fn send_move_result(
    game: &CGame,
    player_id: i32,
    _request: ContainerObjectMoveRequest,
    _selection: &EnhancementSelectionReport,
) -> i32 {
    // Shadow Add сначала возвращает тот же goods в PreviousContainer. Поэтому
    // response до `Send` содержит одинаковые source/destination owner, slot и
    // GUID; exact `NormalizeSelfMove` сворачивает его в однобайтовый rollback.
    send_rollback(game, player_id)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp

// ============================================================================
// FUNCTION: OnContainerMessage
// STATUS: PARTIAL_IMPLEMENTATION
// MATERIALIZED: enhancement-shadow route `0x90301`; остальные container routes RAW ниже
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp:14
// RVA: 0x00086240
// ADDRESS: 00486240
// PROTOTYPE: void __cdecl OnContainerMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00499691
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp
// RVA: 0x00099691
// ADDRESS: 00499691
// PROTOTYPE: undefined Catch@00499691()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004997a1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\message\containermessage.cpp
// RVA: 0x000997A1
// ADDRESS: 004997a1
// PROTOTYPE: undefined Catch@004997a1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
