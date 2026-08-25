//! Входные shape-команды GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/shapemessage.cpp`. Материализован весь handler
//! `0x8F901..0x8F905`: exact fixed-width decode, direction/emotion player
//! state, region lookup, around/addressed wire и ordering внешних AI/spatial/
//! serialization owners. Player `SetTileXY` проходит concrete region/area/
//! block mutation и post-move `GS0163`. Quest movement замыкает attack guard,
//! rotation correction, addressed `OnCannotMove`, emotion reset и canonical
//! player-owned `CPlayerAI` destination FIFO. Non-player polymorphic `SetTileXY` и полные
//! player/goods/shape serializers остаются runtime-границами.

use crate::gameserver::appserver::shape::{ShapeCoordinateBlock, ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{CGame, colored_player_notice_message};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const MOVE_DIRECTION: u32 = 0x0008_f901;
const CHANGE_POSITION: u32 = 0x0008_f902;
const QUEST_MOVE_STEP: u32 = 0x0008_f903;
const QUERY_SHAPE_SNAPSHOT: u32 = 0x0008_f904;
const PERFORM_EMOTION: u32 = 0x0008_f905;
const PLAYER_TYPE: i32 = 400;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShapeQuestMoveFacts {
    pub(crate) blocked_by_breakable_attack: bool,
    pub(crate) server_rotation: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShapeEmotionFacts {
    pub(crate) blocked_state: bool,
    pub(crate) ai_available: bool,
    pub(crate) ai_has_target: bool,
    pub(crate) now_ms: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShapeSnapshot {
    pub(crate) identity: ShapeIdentity,
    pub(crate) payload: Vec<u8>,
}

pub(crate) trait GameShapeMessageRuntime {
    fn shape_symbol_attackable(&mut self, game: &CGame, player_id: i32, region_id: i32) -> bool;
    fn allow_client_change_position(&mut self, game: &CGame) -> bool;
    fn resolve_external_shape_view(
        &mut self,
        game: &CGame,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<ShapeView>;
    fn relocate_external_shape(
        &mut self,
        game: &mut CGame,
        region_id: i32,
        identity: ShapeIdentity,
        tile_x: i32,
        tile_y: i32,
    );
    fn shape_quest_move_facts(
        &mut self,
        game: &CGame,
        player_id: i32,
        region_id: i32,
    ) -> ShapeQuestMoveFacts;
    fn serialize_shape_snapshot(
        &mut self,
        game: &CGame,
        region_id: i32,
        shape: ShapeView,
    ) -> Option<ShapeSnapshot>;
    fn resolve_external_shape_snapshot(
        &mut self,
        game: &CGame,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<ShapeSnapshot>;
    fn shape_emotion_facts(
        &mut self,
        game: &CGame,
        player_id: i32,
        region_id: i32,
        emotion_id: i32,
    ) -> ShapeEmotionFacts;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameShapeMessageError {
    MissingField(&'static str),
    InvalidPayloadSize { expected: usize, actual: usize },
    Coordinate(ShapeCoordinateBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameShapeMessageOutcome {
    MissingContext,
    DirectionChanged,
    PositionFeatureDisabled,
    PositionTargetMissing,
    PositionMutationBlocked,
    PositionChanged,
    QuestMoveBlocked,
    QuestMoveIgnoredDead,
    QuestMoveQueued,
    SnapshotTargetMissing,
    SnapshotSerializationFailed,
    SnapshotSent,
    EmotionTargetMismatch,
    EmotionRejected,
    EmotionSent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameShapeMessageDelivery {
    Around(Option<Result<i32, ShapeCoordinateBlock>>),
    AroundPosition(Option<i32>),
    Player(i32),
}

#[must_use = "shape-message report содержит state, spatial и network effects"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameShapeMessageReport {
    pub(crate) message_type: u32,
    pub(crate) player_id: Option<i32>,
    pub(crate) region_id: Option<i32>,
    pub(crate) outcome: GameShapeMessageOutcome,
    pub(crate) target: Option<ShapeIdentity>,
    pub(crate) deliveries: Vec<GameShapeMessageDelivery>,
}

fn send_player_cannot_move(game: &CGame, player_id: i32) -> Result<i32, GameShapeMessageError> {
    let Some(player) = game.find_player(player_id) else {
        return Ok(0);
    };
    let tile_x = player
        .shape()
        .get_tile_x()
        .map_err(GameShapeMessageError::Coordinate)?;
    let tile_y = player
        .shape()
        .get_tile_y()
        .map_err(GameShapeMessageError::Coordinate)?;
    let mut response = CMessage::new(0x000b_f605);
    response.add_long(0);
    response.add_long(0);
    response.add_long(tile_x);
    response.add_long(tile_y);
    Ok(response.send_to_player(game.net_server(), player_id))
}

pub(crate) fn dispatch_game_shape_message<Runtime: GameShapeMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<GameShapeMessageReport, GameShapeMessageError>> {
    let message_type = message.message_type() as u32;
    if !matches!(
        message_type,
        MOVE_DIRECTION | CHANGE_POSITION | QUEST_MOVE_STEP | QUERY_SHAPE_SNAPSHOT | PERFORM_EMOTION
    ) {
        return None;
    }
    let expected_payload = match message_type {
        MOVE_DIRECTION => 1,
        CHANGE_POSITION => 16,
        QUEST_MOVE_STEP => 3,
        QUERY_SHAPE_SNAPSHOT => 8,
        PERFORM_EMOTION => 12,
        _ => unreachable!("shape command отфильтрована выше"),
    };
    let base = message.base_mut();
    let actual_payload = base.as_wire_bytes().len().saturating_sub(base.cursor());
    if actual_payload != expected_payload {
        return Some(Err(GameShapeMessageError::InvalidPayloadSize {
            expected: expected_payload,
            actual: actual_payload,
        }));
    }
    message.resolve_player_context(game);
    let player_id = message.player_id();
    let region_id = message.region_id();
    let mut report = GameShapeMessageReport {
        message_type,
        player_id,
        region_id,
        outcome: GameShapeMessageOutcome::MissingContext,
        target: None,
        deliveries: Vec::new(),
    };
    let (Some(player_id), Some(region_id)) = (player_id, region_id) else {
        return Some(Ok(report));
    };
    let read_long = |message: &mut CMessage, field| {
        message
            .base_mut()
            .get_long()
            .ok_or(GameShapeMessageError::MissingField(field))
    };

    match message_type {
        MOVE_DIRECTION => {
            let direction = match message.base_mut().get_byte() {
                Some(value) => value,
                None => return Some(Err(GameShapeMessageError::MissingField("direction"))),
            };
            let Some((identity, contend_state)) = game.find_player_mut(player_id).map(|player| {
                player.apply_client_direction(direction);
                (player.shape().identity(), player.contend_state())
            }) else {
                return Some(Ok(report));
            };
            let mut changed = CMessage::new(0x000b_f601);
            changed.add_byte(direction);
            changed.add_long(identity.object_type);
            changed.add_long(identity.id);
            report.deliveries.push(GameShapeMessageDelivery::Around(
                game.send_player_shape_around(player_id, Some(player_id), &changed),
            ));
            game.find_player_mut(player_id)
                .expect("shape player сохранён до ClearEmotion")
                .clear_emotion_state();
            let mut cleared = CMessage::new(0x000b_f611);
            cleared.add_long(identity.object_type);
            cleared.add_long(identity.id);
            cleared.add_long(0);
            report.deliveries.push(GameShapeMessageDelivery::Around(
                game.send_player_shape_around(player_id, Some(player_id), &cleared),
            ));
            if contend_state && runtime.shape_symbol_attackable(game, player_id, region_id) {
                let delivery = colored_player_notice_message(
                    0xffff_ffff,
                    0xffff_0000,
                    game.get_string_by_id(b"GS0331"),
                )
                .send_to_player(game.net_server(), player_id);
                report
                    .deliveries
                    .push(GameShapeMessageDelivery::Player(delivery));
            }
            report.outcome = GameShapeMessageOutcome::DirectionChanged;
        }
        CHANGE_POSITION => {
            let fields = match (|| {
                Ok((
                    read_long(message, "target type")?,
                    read_long(message, "target id")?,
                    read_long(message, "tile x")?,
                    read_long(message, "tile y")?,
                ))
            })() {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            if !runtime.allow_client_change_position(game) {
                report.outcome = GameShapeMessageOutcome::PositionFeatureDisabled;
                return Some(Ok(report));
            }
            let identity = ShapeIdentity {
                object_type: fields.0,
                id: fields.1,
                ex_id: CGuid::GUID_INVALID,
            };
            report.target = Some(identity);
            let target = game
                .find_shape_in_region(region_id, identity)
                .or_else(|| runtime.resolve_external_shape_view(game, region_id, identity));
            let Some(target) = target else {
                report.outcome = GameShapeMessageOutcome::PositionTargetMissing;
                return Some(Ok(report));
            };
            let mut relocation = CMessage::new(0x000b_f603);
            relocation.add_long(fields.0);
            relocation.add_long(fields.1);
            relocation.add_long(fields.2);
            relocation.add_long(fields.3);
            report
                .deliveries
                .push(GameShapeMessageDelivery::AroundPosition(
                    game.send_shape_position_around(
                        region_id,
                        target.tile_x,
                        target.tile_y,
                        &relocation,
                    ),
                ));
            if identity.object_type == PLAYER_TYPE && game.find_player(identity.id).is_some() {
                match game.relocate_player_shape(identity.id, region_id, fields.2, fields.3) {
                    Some(Ok(())) => {}
                    Some(Err(_)) => {
                        report.outcome = GameShapeMessageOutcome::PositionMutationBlocked;
                        return Some(Ok(report));
                    }
                    None => {
                        report.outcome = GameShapeMessageOutcome::PositionTargetMissing;
                        return Some(Ok(report));
                    }
                }
                let contend_state = game
                    .find_player(identity.id)
                    .is_some_and(|player| player.contend_state());
                if contend_state && runtime.shape_symbol_attackable(game, identity.id, region_id) {
                    let delivery = colored_player_notice_message(
                        0xffff_ffff,
                        0xffff_0000,
                        game.get_string_by_id(b"GS0163"),
                    )
                    .send_to_player(game.net_server(), identity.id);
                    report
                        .deliveries
                        .push(GameShapeMessageDelivery::Player(delivery));
                }
            } else {
                runtime.relocate_external_shape(game, region_id, identity, fields.2, fields.3);
            }
            report.outcome = GameShapeMessageOutcome::PositionChanged;
        }
        QUEST_MOVE_STEP => {
            let move_mode = message
                .base_mut()
                .get_char()
                .ok_or(GameShapeMessageError::MissingField("move mode"));
            let direction = message
                .base_mut()
                .get_char()
                .ok_or(GameShapeMessageError::MissingField("direction"));
            let rotation = message
                .base_mut()
                .get_byte()
                .ok_or(GameShapeMessageError::MissingField("rotation"));
            let (move_mode, direction, client_rotation) = match (move_mode, direction, rotation) {
                (Ok(a), Ok(b), Ok(c)) => (a, b, c),
                (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                    return Some(Err(error));
                }
            };
            let facts = runtime.shape_quest_move_facts(game, player_id, region_id);
            if facts.blocked_by_breakable_attack {
                let delivery = match send_player_cannot_move(game, player_id) {
                    Ok(delivery) => delivery,
                    Err(error) => return Some(Err(error)),
                };
                report
                    .deliveries
                    .push(GameShapeMessageDelivery::Player(delivery));
                report.outcome = GameShapeMessageOutcome::QuestMoveBlocked;
                return Some(Ok(report));
            }
            if client_rotation != facts.server_rotation {
                let mut response = CMessage::new(0x000b_f738);
                response.add_byte(facts.server_rotation);
                report.deliveries.push(GameShapeMessageDelivery::Player(
                    response.send_to_player(game.net_server(), player_id),
                ));
            }
            if game
                .find_player(player_id)
                .is_some_and(|player| player.is_dead())
            {
                report.outcome = GameShapeMessageOutcome::QuestMoveIgnoredDead;
                return Some(Ok(report));
            }
            game.find_player_mut(player_id)
                .expect("quest-move player сохранён после dead guard")
                .clear_emotion_state();
            if !game.queue_player_ai_destination(player_id, i32::from(direction), move_mode != 2) {
                let delivery = match send_player_cannot_move(game, player_id) {
                    Ok(delivery) => delivery,
                    Err(error) => return Some(Err(error)),
                };
                report
                    .deliveries
                    .push(GameShapeMessageDelivery::Player(delivery));
                report.outcome = GameShapeMessageOutcome::QuestMoveBlocked;
                return Some(Ok(report));
            }
            report.outcome = GameShapeMessageOutcome::QuestMoveQueued;
        }
        QUERY_SHAPE_SNAPSHOT => {
            let identity = match (|| {
                Ok(ShapeIdentity {
                    object_type: read_long(message, "target type")?,
                    id: read_long(message, "target id")?,
                    ex_id: CGuid::GUID_INVALID,
                })
            })() {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            report.target = Some(identity);
            let snapshot = match game.find_shape_in_region(region_id, identity) {
                Some(shape) => {
                    let Some(snapshot) = runtime.serialize_shape_snapshot(game, region_id, shape)
                    else {
                        report.outcome = GameShapeMessageOutcome::SnapshotSerializationFailed;
                        return Some(Ok(report));
                    };
                    snapshot
                }
                None => {
                    let Some(snapshot) =
                        runtime.resolve_external_shape_snapshot(game, region_id, identity)
                    else {
                        report.outcome = GameShapeMessageOutcome::SnapshotTargetMissing;
                        return Some(Ok(report));
                    };
                    snapshot
                }
            };
            let Ok(size) = i32::try_from(snapshot.payload.len()) else {
                report.outcome = GameShapeMessageOutcome::SnapshotSerializationFailed;
                return Some(Ok(report));
            };
            let mut response = CMessage::new(0x000b_f502);
            response.add_long(snapshot.identity.object_type);
            response.add_long(snapshot.identity.id);
            response.base_mut().add_guid(snapshot.identity.ex_id);
            response.add_long(size);
            response.base_mut().add(&snapshot.payload);
            response.base_mut().add_char(0);
            report.deliveries.push(GameShapeMessageDelivery::Player(
                response.send_to_player(game.net_server(), player_id),
            ));
            report.outcome = GameShapeMessageOutcome::SnapshotSent;
        }
        PERFORM_EMOTION => {
            let fields = match (|| {
                Ok((
                    read_long(message, "target type")?,
                    read_long(message, "target id")?,
                    read_long(message, "emotion id")?,
                ))
            })() {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let identity = ShapeIdentity {
                object_type: fields.0,
                id: fields.1,
                ex_id: CGuid::GUID_INVALID,
            };
            report.target = Some(identity);
            let Some(player_identity) = game
                .find_player(player_id)
                .map(|player| player.shape().identity())
            else {
                return Some(Ok(report));
            };
            let facts = runtime.shape_emotion_facts(game, player_id, region_id, fields.2);
            if facts.blocked_state {
                let delivery =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS1038"))
                        .send_to_player(game.net_server(), player_id);
                report
                    .deliveries
                    .push(GameShapeMessageDelivery::Player(delivery));
            }
            if player_identity.object_type != fields.0 || player_identity.id != fields.1 {
                report.outcome = GameShapeMessageOutcome::EmotionTargetMismatch;
                return Some(Ok(report));
            }
            let repeated = game.emotion_repeated(fields.2);
            let publish = game
                .find_player_mut(player_id)
                .expect("emotion player сохранён после identity lookup")
                .perform_emotion_state(
                    fields.2,
                    repeated,
                    facts.now_ms,
                    facts.ai_available,
                    facts.ai_has_target,
                );
            if !publish {
                report.outcome = GameShapeMessageOutcome::EmotionRejected;
                return Some(Ok(report));
            }
            let mut response = CMessage::new(0x000b_f611);
            response.add_long(player_identity.object_type);
            response.add_long(player_identity.id);
            response.add_long(fields.2);
            report.deliveries.push(GameShapeMessageDelivery::Around(
                game.send_player_shape_around(player_id, Some(player_id), &response),
            ));
            report.outcome = GameShapeMessageOutcome::EmotionSent;
        }
        _ => unreachable!("shape command отфильтрована до decode"),
    }
    Some(Ok(report))
}

// COMPONENT_VARIANT_END: GameServer
