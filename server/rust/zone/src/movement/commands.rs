//! Parse входящих shape-команд движения `0x8F901..0x8F905`.
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `appserver/message/shapemessage.cpp`; доказательная база полей и ветвей —
//! `docs/protocol/game-actions.md`. Порядок чтения полей и точный размер
//! payload сохранены; реакция EXE на лишний/оборванный payload в этих ветвях
//! не установлена (`UNKNOWN`), поэтому строгое отклонение здесь — техническая
//! защита текущего сервиса, а не заявленная идентичность ошибочного пути.

use nebokrai_shared::protocol::LegacyReader;
use nebokrai_shared::values::CGuid;

use crate::regions::ShapeIdentity;
use crate::regions::shape::ShapeCoordinateBlock;

pub const MOVE_DIRECTION: u32 = 0x0008_f901;
pub const CHANGE_POSITION: u32 = 0x0008_f902;
pub const QUEST_MOVE_STEP: u32 = 0x0008_f903;
pub const QUERY_SHAPE_SNAPSHOT: u32 = 0x0008_f904;
pub const PERFORM_EMOTION: u32 = 0x0008_f905;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapeMoveCommand {
    /// `0x8F901`: новое направление игрока.
    MoveDirection {
        direction: u8,
    },
    /// `0x8F902`: клиентская позиция фигуры (только при разрешении setup).
    ChangePosition {
        target: ShapeIdentity,
        tile_x: i32,
        tile_y: i32,
    },
    /// `0x8F903`: шаг движения к заданию; `move_mode != 2` — бег.
    QuestMoveStep {
        move_mode: i8,
        direction: i8,
        client_rotation: u8,
    },
    /// `0x8F904`: запрос клиентского снимка фигуры региона.
    QueryShapeSnapshot {
        target: ShapeIdentity,
    },
    /// `0x8F905`: эмоция самого игрока.
    PerformEmotion {
        target: ShapeIdentity,
        emotion_id: i32,
    },
}

/// Отказ decode/применения shape-команды. Варианты держатся равными бывшему
/// `GameShapeMessageError`: decode-ответственность перешла владельцу Zone
/// без смены диагностического контракта.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeMovementError {
    MissingField(&'static str),
    InvalidPayloadSize { expected: usize, actual: usize },
    Coordinate(ShapeCoordinateBlock),
}

/// Фильтр семейства и точный размер payload одной команды.
pub const fn shape_move_payload_size(message_type: u32) -> Option<usize> {
    match message_type {
        MOVE_DIRECTION => Some(1),
        CHANGE_POSITION => Some(16),
        QUEST_MOVE_STEP => Some(3),
        QUERY_SHAPE_SNAPSHOT => Some(8),
        PERFORM_EMOTION => Some(12),
        _ => None,
    }
}

/// Разбирает payload без header. `Ok(None)` — опкод не из этого семейства;
/// при несовпадении размера поля не читаются, как в прежнем обработчике.
pub fn parse_shape_move_command(
    message_type: u32,
    payload: &[u8],
) -> Result<Option<ShapeMoveCommand>, ShapeMovementError> {
    let Some(expected) = shape_move_payload_size(message_type) else {
        return Ok(None);
    };
    if payload.len() != expected {
        return Err(ShapeMovementError::InvalidPayloadSize {
            expected,
            actual: payload.len(),
        });
    }
    let mut reader = LegacyReader::new(payload);
    let command = match message_type {
        MOVE_DIRECTION => ShapeMoveCommand::MoveDirection {
            direction: take_u8(&mut reader, "direction")?,
        },
        CHANGE_POSITION => ShapeMoveCommand::ChangePosition {
            target: ShapeIdentity {
                object_type: take_i32(&mut reader, "target type")?,
                id: take_i32(&mut reader, "target id")?,
                ex_id: CGuid::GUID_INVALID,
            },
            tile_x: take_i32(&mut reader, "tile x")?,
            tile_y: take_i32(&mut reader, "tile y")?,
        },
        QUEST_MOVE_STEP => ShapeMoveCommand::QuestMoveStep {
            move_mode: take_i8(&mut reader, "move mode")?,
            direction: take_i8(&mut reader, "direction")?,
            client_rotation: take_u8(&mut reader, "rotation")?,
        },
        QUERY_SHAPE_SNAPSHOT => ShapeMoveCommand::QueryShapeSnapshot {
            target: ShapeIdentity {
                object_type: take_i32(&mut reader, "target type")?,
                id: take_i32(&mut reader, "target id")?,
                ex_id: CGuid::GUID_INVALID,
            },
        },
        PERFORM_EMOTION => ShapeMoveCommand::PerformEmotion {
            target: ShapeIdentity {
                object_type: take_i32(&mut reader, "target type")?,
                id: take_i32(&mut reader, "target id")?,
                ex_id: CGuid::GUID_INVALID,
            },
            emotion_id: take_i32(&mut reader, "emotion id")?,
        },
        _ => unreachable!("shape command отфильтрована размером payload"),
    };
    Ok(Some(command))
}

fn take_u8(reader: &mut LegacyReader<'_>, field: &'static str) -> Result<u8, ShapeMovementError> {
    reader
        .read_u8()
        .map_err(|_| ShapeMovementError::MissingField(field))
}

fn take_i8(reader: &mut LegacyReader<'_>, field: &'static str) -> Result<i8, ShapeMovementError> {
    reader
        .read_i8()
        .map_err(|_| ShapeMovementError::MissingField(field))
}

fn take_i32(reader: &mut LegacyReader<'_>, field: &'static str) -> Result<i32, ShapeMovementError> {
    reader
        .read_i32()
        .map_err(|_| ShapeMovementError::MissingField(field))
}
