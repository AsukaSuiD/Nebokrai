//! Узкий game-view для обработчиков мировых сообщений Realm.

use nebokrai_shared::network::ServerCommandHandle;

/// Точка чтения текущего game-server sender-а, нужная handler-ветвям
/// мира. Реализация живёт у владельца игры (старый `CGame`) и делегирует
/// его inherent-методу; имя намеренно совпадает — inherent priority
/// исключает рекурсию.
pub trait WorldGameView {
    fn current_game_server_sender(&self) -> Option<ServerCommandHandle>;
}
