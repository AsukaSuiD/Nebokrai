//! Диспетчер server-сообщений WorldServer (`appworld/message/servermessage.cpp`)
//! целиком перенесён в Realm: типы, decode-helpers, `on_server_message`,
//! `on_game_server_connected`, семейство `continue_game_server_*_configuration`
//! и reconnect-цепочка живут в `nebokrai_realm::app::servermessage`; обращения
//! к владельцу игры — через `WorldGameView` и шов `WorldServerMessageGameView`
//! с делегациями inherent-методам `CGame`. Completion-волна закрыла машинно
//! установленный пропуск ветви `0x3FC02` (GameServer disconnect): швы
//! `disconnect_game_server` и `on_game_server_lost` делегируют одноимённым
//! inherent-методам `CGame`.
//!
//! Внутрипроцессный reconnect заменяет GameServer client в той же FIFO-позиции:
//! старое соединение закрывается и уничтожается до публикации нового, после
//! чего ставятся CD-key snapshot и регистрация. Уже выполненная замена не
//! откатывается при ошибке последующей отправки. Первичная ветвь `0x5FA01`
//! отправляет конфигурацию в исходном порядке; ошибки отдельных `Send` не
//! прерывают цепочку; неизвестный тип записи потребляет только свой tag.
//!
//! Здесь реэкспорт для переходных потребителей старого пакета.

pub use nebokrai_realm::app::servermessage::*;
