//! Правила применения разобранных клиентских shape-команд движения.
//!
//! Источник: та же точная пара, что в `commands`; поведенческая канва —
//! прежний `appserver/message/shapemessage.cpp`, доказательная база порядка
//! ветвей и wire-кадров — `docs/protocol/game-actions.md`. Буквальный порядок
//! мутаций и отправок прежнего обработчика сохранён: круговые кадры позиции
//! всегда предшествуют пространственной мутации, отказ координат наступает
//! уже после рассылки, а наличие `ChangeBody` при эмоции сначала публикует
//! `GS1038` и не прерывает последующую проверку цели.

use tracing::{debug, trace};

use crate::ai::AiShapeAction;
use crate::app::game_message::CMessage;
use crate::skills::BASE_ATTACK_SKILL_ID;

use super::commands::{ShapeMoveCommand, ShapeMovementError};
use super::hub::ShapeMovementGame;

const DIRECTION_CHANGED_MESSAGE: i32 = 0x000b_f601;
const SHAPE_RELOCATED_MESSAGE: i32 = 0x000b_f603;
const CANNOT_MOVE_MESSAGE: i32 = 0x000b_f605;
const EMOTION_MESSAGE: i32 = 0x000b_f611;
const MOVE_ROTATION_FIX_MESSAGE: i32 = 0x000b_f738;
const SHAPE_SNAPSHOT_MESSAGE: i32 = 0x000b_f502;

/// Тип живого игрока в пространственном реестре региона.
const PLAYER_OBJECT_TYPE: i32 = 400;

/// Script move-state `ChangeBody` блокирующей ветви эмоции.
const CHANGE_BODY_STATE_ID: i32 = 0x37;

/// Короткая форма `0xBF605` одному игроку: отказ шага с текущими координатами.
/// Владелец формы здесь ещё прежний `CPlayer`, поэтому чтение клетки идёт
/// через hub-шов; блок координат остаётся отказом команды, как и раньше.
pub fn send_shape_move_cannot_move<Game: ShapeMovementGame>(
    game: &Game,
    player_id: i32,
) -> Result<(), ShapeMovementError> {
    let Some(tile) = game.player_tile_coordinates(player_id) else {
        return Ok(());
    };
    let (tile_x, tile_y) = tile.map_err(ShapeMovementError::Coordinate)?;
    let mut response = CMessage::new(CANNOT_MOVE_MESSAGE);
    response.add_long(0);
    response.add_long(0);
    response.add_long(tile_x);
    response.add_long(tile_y);
    game.send_to_player(player_id, &response);
    Ok(())
}

/// Применяет разобранную клиентскую команду движения от имени уже связанного
/// игрока, исполняя шаги через hub-владельца в исходном порядке обработчика.
pub fn apply_shape_move_command<Game: ShapeMovementGame>(
    game: &mut Game,
    player_id: i32,
    region_id: i32,
    command: ShapeMoveCommand,
    now_milliseconds: &mut dyn FnMut() -> u32,
) -> Result<(), ShapeMovementError> {
    match command {
        ShapeMoveCommand::MoveDirection { direction } => {
            let Some(identity) = game.apply_player_client_direction(player_id, direction) else {
                return Ok(());
            };
            let mut changed = CMessage::new(DIRECTION_CHANGED_MESSAGE);
            changed.add_byte(direction);
            changed.add_long(identity.object_type);
            changed.add_long(identity.id);
            game.send_player_shape_around(player_id, Some(player_id), &changed);
            game.clear_player_emotion(player_id);
            let mut cleared = CMessage::new(EMOTION_MESSAGE);
            cleared.add_long(identity.object_type);
            cleared.add_long(identity.id);
            cleared.add_long(0);
            game.send_player_shape_around(player_id, Some(player_id), &cleared);
            if game.player_contend_state(player_id)
                && game.cancel_player_contend_in_region(region_id, player_id)
            {
                game.send_colored_player_notice(player_id, 0xffff_ffff, 0xffff_0000, b"GS0331");
            }
            debug!(player_id, region_id, direction, "изменено направление игрока");
        }
        ShapeMoveCommand::ChangePosition {
            target,
            tile_x,
            tile_y,
        } => {
            if !game.allow_client_change_position() {
                trace!(player_id, region_id, "клиентское перемещение отключено");
                return Ok(());
            }
            let Some(shape) = game.find_shape_in_region(region_id, target) else {
                trace!(player_id, region_id, target_type = target.object_type, target_id = target.id, "цель перемещения не найдена");
                return Ok(());
            };
            let mut relocation = CMessage::new(SHAPE_RELOCATED_MESSAGE);
            relocation.add_long(target.object_type);
            relocation.add_long(target.id);
            relocation.add_long(tile_x);
            relocation.add_long(tile_y);
            game.send_shape_position_around(region_id, shape.tile_x, shape.tile_y, &relocation);
            match game.relocate_region_shape(region_id, target, tile_x, tile_y) {
                Some(Ok(())) => {}
                Some(Err(_)) => {
                    trace!(player_id, region_id, target_id = target.id, "перемещение заблокировано координатами");
                    return Ok(());
                }
                None => {
                    return Ok(());
                }
            }
            debug!(player_id, region_id, target_type = target.object_type, target_id = target.id, tile_x, tile_y, "изменена позиция shape");
        }
        ShapeMoveCommand::QuestMoveStep {
            move_mode,
            direction,
            client_rotation,
        } => {
            // Запрет шага соответствует CMoveShape::OnMessage (0x004972D1):
            // только active Attack + выбранный базовый навык; ожидающая
            // команда и фоновые навыки движение не запрещают.
            let blocked_by_base_attack = game.player_active_action(player_id)
                == Some(AiShapeAction::Attack)
                && game.player_current_skill_id(player_id) == Some(BASE_ATTACK_SKILL_ID);
            if blocked_by_base_attack {
                send_shape_move_cannot_move(game, player_id)?;
                trace!(player_id, region_id, "шаг движения заблокирован атакой");
                return Ok(());
            }
            let server_rotation = game.quest_move_rotation();
            if client_rotation != server_rotation {
                let mut response = CMessage::new(MOVE_ROTATION_FIX_MESSAGE);
                response.add_byte(server_rotation);
                game.send_to_player(player_id, &response);
            }
            if game.player_is_dead(player_id) {
                trace!(player_id, region_id, "шаг движения мёртвого игрока пропущен");
                return Ok(());
            }
            game.clear_player_emotion(player_id);
            if !game.queue_player_ai_destination(player_id, i32::from(direction), move_mode != 2) {
                send_shape_move_cannot_move(game, player_id)?;
                trace!(player_id, region_id, "очередь шага движения отклонила цель");
                return Ok(());
            }
            debug!(player_id, region_id, direction, move_mode, "шаг движения поставлен в AI-очередь");
        }
        ShapeMoveCommand::QueryShapeSnapshot { target } => {
            let snapshot = match game.find_shape_in_region(region_id, target) {
                Some(shape) if target.object_type == PLAYER_OBJECT_TYPE => {
                    let Some(snapshot) = game.serialize_player_shape_snapshot(
                        region_id,
                        shape,
                        &mut *now_milliseconds,
                    ) else {
                        trace!(player_id, region_id, target_type = target.object_type, target_id = target.id, "снимок игрока не сериализован");
                        return Ok(());
                    };
                    snapshot
                }
                Some(_) => {
                    let Some(snapshot) = game.serialize_owned_shape_snapshot(
                        region_id,
                        target,
                        &mut *now_milliseconds,
                    ) else {
                        trace!(player_id, region_id, target_type = target.object_type, target_id = target.id, "снимок формы не сериализован владельцем");
                        return Ok(());
                    };
                    snapshot
                }
                None => {
                    trace!(player_id, region_id, target_type = target.object_type, target_id = target.id, "цель снимка shape не найдена");
                    return Ok(());
                }
            };
            let (identity, payload) = snapshot;
            let Ok(size) = i32::try_from(payload.len()) else {
                trace!(player_id, region_id, payload_len = payload.len(), "размер снимка shape не представим в wire-формате");
                return Ok(());
            };
            let mut response = CMessage::new(SHAPE_SNAPSHOT_MESSAGE);
            response.add_long(identity.object_type);
            response.add_long(identity.id);
            response.base_mut().add_guid(identity.ex_id);
            response.add_long(size);
            response.base_mut().add(&payload);
            response.base_mut().add_char(0);
            game.send_to_player(player_id, &response);
            debug!(player_id, region_id, target_type = identity.object_type, target_id = identity.id, payload_len = payload.len(), "снимок shape отправлен");
        }
        ShapeMoveCommand::PerformEmotion {
            target,
            emotion_id,
        } => {
            let Some(player_identity) = game.player_shape_identity(player_id) else {
                return Ok(());
            };
            // Странность EXE: наличие ChangeBody сначала публикует GS1038,
            // но не прерывает последующую проверку цели и PerformEmotion.
            if game.player_script_state_present(player_id, CHANGE_BODY_STATE_ID) {
                game.send_colored_player_notice(player_id, 0xffff_ffff, 0, b"GS1038");
            }
            if player_identity.object_type != target.object_type || player_identity.id != target.id
            {
                trace!(player_id, region_id, target_type = target.object_type, target_id = target.id, "цель эмоции не совпадает с игроком");
                return Ok(());
            }
            let repeated = game.player_emotion_repeated(emotion_id);
            let ai_has_target = game.player_ai_has_resolved_target(player_id, region_id);
            let now_ms = now_milliseconds();
            let publish = game.perform_player_emotion(
                player_id,
                emotion_id,
                repeated,
                now_ms,
                true,
                ai_has_target,
            );
            if !publish {
                trace!(player_id, region_id, emotion_id, "эмоция отклонена состоянием игрока");
                return Ok(());
            }
            let mut response = CMessage::new(EMOTION_MESSAGE);
            response.add_long(player_identity.object_type);
            response.add_long(player_identity.id);
            response.add_long(emotion_id);
            game.send_player_shape_around(player_id, Some(player_id), &response);
            debug!(player_id, region_id, emotion_id, "эмоция опубликована вокруг игрока");
        }
    }
    Ok(())
}
