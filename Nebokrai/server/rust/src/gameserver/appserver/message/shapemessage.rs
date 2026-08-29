//! Входные shape-команды GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message/shapemessage.cpp`. Материализован весь обработчик
//! `0x8F901..0x8F905`: точное декодирование полей фиксированной ширины,
//! направление и эмоция игрока, поиск региона, адресная и круговая wire-
//! рассылка, а также порядок внешних владельцев AI, пространства и
//! сериализации. Виртуальный `SymbolIsAttackAble` базового и городского
//! регионов разрешается каноническим владельцем региона. `SetTileXY` игрока
//! проходит конкретные изменения региона, области и блока, затем `GS0163`.
//! Движение к заданию сохраняет защиту атаки, исправление поворота, адресный
//! `OnCannotMove`, сброс эмоции и FIFO назначения в принадлежащем игроку
//! `CPlayerAI`. Разрешение клиентской позиции читается из действующего
//! `CGlobeSetup::bAllowClientChangePos`; исходный порядок проверки, поиска и
//! payload сохранён. Полиморфный `SetTileXY` не-игрока и полные сериализаторы
//! player/goods/shape остаются границами исполнения. Синхронные отправки не
//! дублируются в `Vec`; диагностические исходы публикуются через `tracing`.
//! Эмоция `0x8F905` сохраняет странность EXE: наличие `ChangeBody` сначала
//! публикует `GS1038`, но не прерывает последующую проверку цели и
//! `PerformEmotion`. `CBaseAI::GetTarget` выражен текущей объектной командой
//! игрока и её разрешением через тот же региональный владелец.

use crate::gameserver::appserver::shape::{ShapeCoordinateBlock, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use crate::gameserver::gameserver::game::{
    CGame, GameClockContext, colored_player_notice_message,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use tracing::{debug, trace};

const MOVE_DIRECTION: u32 = 0x0008_f901;
const CHANGE_POSITION: u32 = 0x0008_f902;
const QUEST_MOVE_STEP: u32 = 0x0008_f903;
const QUERY_SHAPE_SNAPSHOT: u32 = 0x0008_f904;
const PERFORM_EMOTION: u32 = 0x0008_f905;
const PLAYER_TYPE: i32 = 400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShapeSnapshot {
    pub(crate) identity: ShapeIdentity,
    pub(crate) payload: Vec<u8>,
}

pub(crate) trait GameShapeMessageRuntime: GameClockContext {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameShapeMessageError {
    MissingField(&'static str),
    InvalidPayloadSize { expected: usize, actual: usize },
    Coordinate(ShapeCoordinateBlock),
}

fn send_player_cannot_move(game: &CGame, player_id: i32) -> Result<(), GameShapeMessageError> {
    let Some(player) = game.find_player(player_id) else {
        return Ok(());
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
    let _ = response.send_to_player(game.net_server(), player_id);
    Ok(())
}

pub(crate) fn dispatch_game_shape_message<Runtime: GameShapeMessageRuntime>(
    message: &mut CMessage,
    game: &mut CGame,
    runtime: &mut Runtime,
) -> Option<Result<(), GameShapeMessageError>> {
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
    let (Some(player_id), Some(region_id)) = (player_id, region_id) else {
        trace!(message_type, ?player_id, ?region_id, "shape-команда пропущена: нет контекста");
        return Some(Ok(()));
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
                return Some(Ok(()));
            };
            let mut changed = CMessage::new(0x000b_f601);
            changed.add_byte(direction);
            changed.add_long(identity.object_type);
            changed.add_long(identity.id);
            let _ = game.send_player_shape_around(player_id, Some(player_id), &changed);
            game.find_player_mut(player_id)
                .expect("shape player сохранён до ClearEmotion")
                .clear_emotion_state();
            let mut cleared = CMessage::new(0x000b_f611);
            cleared.add_long(identity.object_type);
            cleared.add_long(identity.id);
            cleared.add_long(0);
            let _ = game.send_player_shape_around(player_id, Some(player_id), &cleared);
            if contend_state && game.region_symbol_attackable(region_id) {
                let _ = colored_player_notice_message(
                    0xffff_ffff,
                    0xffff_0000,
                    game.get_string_by_id(b"GS0331"),
                )
                .send_to_player(game.net_server(), player_id);
            }
            debug!(player_id, region_id, direction, "изменено направление игрока");
        }
        CHANGE_POSITION => {
            if !game.allow_client_change_position() {
                trace!(player_id, region_id, "клиентское перемещение отключено");
                return Some(Ok(()));
            }
            let target_fields = match (|| {
                Ok((
                    read_long(message, "target type")?,
                    read_long(message, "target id")?,
                ))
            })() {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            let identity = ShapeIdentity {
                object_type: target_fields.0,
                id: target_fields.1,
                ex_id: CGuid::GUID_INVALID,
            };
            let target = game
                .find_shape_in_region(region_id, identity)
                .or_else(|| runtime.resolve_external_shape_view(game, region_id, identity));
            let Some(target) = target else {
                trace!(player_id, region_id, target_type = identity.object_type, target_id = identity.id, "цель перемещения не найдена");
                return Some(Ok(()));
            };
            let position_fields =
                match (|| Ok((read_long(message, "tile x")?, read_long(message, "tile y")?)))() {
                    Ok(value) => value,
                    Err(error) => return Some(Err(error)),
                };
            let mut relocation = CMessage::new(0x000b_f603);
            relocation.add_long(target_fields.0);
            relocation.add_long(target_fields.1);
            relocation.add_long(position_fields.0);
            relocation.add_long(position_fields.1);
            let _ = game.send_shape_position_around(
                region_id,
                target.tile_x,
                target.tile_y,
                &relocation,
            );
            if identity.object_type == PLAYER_TYPE && game.find_player(identity.id).is_some() {
                match game.relocate_player_shape(
                    identity.id,
                    region_id,
                    position_fields.0,
                    position_fields.1,
                ) {
                    Some(Ok(())) => {}
                    Some(Err(_)) => {
                        trace!(player_id, region_id, target_id = identity.id, "перемещение заблокировано координатами");
                        return Some(Ok(()));
                    }
                    None => {
                        return Some(Ok(()));
                    }
                }
                let contend_state = game
                    .find_player(identity.id)
                    .is_some_and(|player| player.contend_state());
                if contend_state && game.region_symbol_attackable(region_id) {
                    let _ = colored_player_notice_message(
                        0xffff_ffff,
                        0xffff_0000,
                        game.get_string_by_id(b"GS0163"),
                    )
                    .send_to_player(game.net_server(), identity.id);
                }
            } else {
                runtime.relocate_external_shape(
                    game,
                    region_id,
                    identity,
                    position_fields.0,
                    position_fields.1,
                );
            }
            debug!(player_id, region_id, target_type = identity.object_type, target_id = identity.id, tile_x = position_fields.0, tile_y = position_fields.1, "изменена позиция shape");
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
            let blocked_by_breakable_attack = game
                .find_player(player_id)
                .and_then(|player| {
                    let skill_id = player.current_skill_id()?;
                    let level = player.learned_skill_level(skill_id);
                    game.skill_base_properties(skill_id, level)
                })
                .is_some_and(|properties| {
                    properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) == 1
                });
            if blocked_by_breakable_attack {
                match send_player_cannot_move(game, player_id) {
                    Ok(()) => {}
                    Err(error) => return Some(Err(error)),
                }
                trace!(player_id, region_id, "шаг движения заблокирован атакой");
                return Some(Ok(()));
            }
            let server_rotation = game.quest_move_rotation();
            if client_rotation != server_rotation {
                let mut response = CMessage::new(0x000b_f738);
                response.add_byte(server_rotation);
                let _ = response.send_to_player(game.net_server(), player_id);
            }
            if game
                .find_player(player_id)
                .is_some_and(|player| player.is_dead())
            {
                trace!(player_id, region_id, "шаг движения мёртвого игрока пропущен");
                return Some(Ok(()));
            }
            game.find_player_mut(player_id)
                .expect("quest-move player сохранён после dead guard")
                .clear_emotion_state();
            if !game.queue_player_ai_destination(player_id, i32::from(direction), move_mode != 2) {
                match send_player_cannot_move(game, player_id) {
                    Ok(()) => {}
                    Err(error) => return Some(Err(error)),
                }
                trace!(player_id, region_id, "очередь шага движения отклонила цель");
                return Some(Ok(()));
            }
            debug!(player_id, region_id, direction, move_mode, "шаг движения поставлен в AI-очередь");
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
            let snapshot = match game.find_shape_in_region(region_id, identity) {
                Some(shape) => {
                    let Some(snapshot) = runtime.serialize_shape_snapshot(game, region_id, shape)
                    else {
                        trace!(player_id, region_id, target_type = identity.object_type, target_id = identity.id, "снимок shape не сериализован");
                        return Some(Ok(()));
                    };
                    snapshot
                }
                None => {
                    let Some(snapshot) =
                        runtime.resolve_external_shape_snapshot(game, region_id, identity)
                    else {
                        trace!(player_id, region_id, target_type = identity.object_type, target_id = identity.id, "цель снимка shape не найдена");
                        return Some(Ok(()));
                    };
                    snapshot
                }
            };
            let Ok(size) = i32::try_from(snapshot.payload.len()) else {
                trace!(player_id, region_id, payload_len = snapshot.payload.len(), "размер снимка shape не представим в wire-формате");
                return Some(Ok(()));
            };
            let mut response = CMessage::new(0x000b_f502);
            response.add_long(snapshot.identity.object_type);
            response.add_long(snapshot.identity.id);
            response.base_mut().add_guid(snapshot.identity.ex_id);
            response.add_long(size);
            response.base_mut().add(&snapshot.payload);
            response.base_mut().add_char(0);
            let _ = response.send_to_player(game.net_server(), player_id);
            debug!(player_id, region_id, target_type = identity.object_type, target_id = identity.id, payload_len = snapshot.payload.len(), "снимок shape отправлен");
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
            let Some(player_identity) = game
                .find_player(player_id)
                .map(|player| player.shape().identity())
            else {
                return Some(Ok(()));
            };
            let blocked_state = game
                .find_player(player_id)
                .is_some_and(|player| player.script_move_state_count(0x37) != 0);
            if blocked_state {
                let _ =
                    colored_player_notice_message(0xffff_ffff, 0, game.get_string_by_id(b"GS1038"))
                        .send_to_player(game.net_server(), player_id);
            }
            if player_identity.object_type != fields.0 || player_identity.id != fields.1 {
                trace!(player_id, region_id, target_type = fields.0, target_id = fields.1, "цель эмоции не совпадает с игроком");
                return Some(Ok(()));
            }
            let repeated = game.emotion_repeated(fields.2);
            let ai_has_target = game.player_ai_has_resolved_target(player_id, region_id);
            let now_ms = runtime.now_milliseconds();
            let publish = game
                .find_player_mut(player_id)
                .expect("emotion player сохранён после identity lookup")
                .perform_emotion_state(
                    fields.2,
                    repeated,
                    now_ms,
                    true,
                    ai_has_target,
                );
            if !publish {
                trace!(player_id, region_id, emotion_id = fields.2, "эмоция отклонена состоянием игрока");
                return Some(Ok(()));
            }
            let mut response = CMessage::new(0x000b_f611);
            response.add_long(player_identity.object_type);
            response.add_long(player_identity.id);
            response.add_long(fields.2);
            let _ = game.send_player_shape_around(player_id, Some(player_id), &response);
            debug!(player_id, region_id, emotion_id = fields.2, "эмоция опубликована вокруг игрока");
        }
        _ => unreachable!("shape command отфильтрована до decode"),
    }
    Some(Ok(()))
}

// COMPONENT_VARIANT_END: GameServer
