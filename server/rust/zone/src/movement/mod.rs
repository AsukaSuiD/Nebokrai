//! Компонент `movement`: клиентские команды движения фигур региона
//! `0x8F901..0x8F905` — decode, типы и правила применения прежнего
//! `appserver/message/shapemessage.cpp` GameServer. Доказательная база полей,
//! ветвей и ответных кадров — `docs/protocol/game-actions.md`.
//! Живой игрок, пространственный реестр региона, региональные агрегаты и
//! net-owner остаются hub-владельцами и достигаются узкими фасадами `hub`;
//! сетевой край прежнего пакета подключает команды через тонкий адаптер.

mod apply; // правила применения разобранных команд и ответные wire-кадры.
mod commands; // opcode-фильтр, точные размеры payload и decode пяти команд.
mod hub; // переходный фасад живых реестров и отправки прежнего `CGame`.

pub use apply::{apply_shape_move_command, send_shape_move_cannot_move};
pub use commands::{
    parse_shape_move_command, ShapeMoveCommand, ShapeMovementError, CHANGE_POSITION,
    MOVE_DIRECTION, PERFORM_EMOTION, QUERY_SHAPE_SNAPSHOT, QUEST_MOVE_STEP,
};
pub use hub::ShapeMovementGame;
