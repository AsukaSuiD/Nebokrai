//! Узкий organizing-view для обработчиков мировых сообщений Realm.

use crate::app::world_game_view::WorldGameView;
use crate::organizations::faction::FactionTalkDelivery;
use crate::organizations::organizingctrl::FreePlayerLookup;

/// Точки обратного вызова handler-ветвей мира в organizing-владельца.
/// Реализация живёт у старого `COrganizingCtrl` и делегирует его inherent-
/// методам и `CFaction::talk`; отдельный трейт, а не `WorldGameView`,
/// потому что organizing передаётся диспетчерам отдельным owner-параметром.
pub trait WorldOrganizingView {
    fn is_free_player(&self, player_id: i32) -> FreePlayerLookup;

    /// Разговор фракции `0x7FA02`: `None` — фракция не найдена. Цикл по
    /// членам и wire-формат остаются у владельца (`CFaction::talk`).
    fn faction_talk(
        &self,
        game: &dyn WorldGameView,
        faction_id: i32,
        speaker_id: i32,
        first_text: &[u8],
        second_text: &[u8],
    ) -> Option<Vec<FactionTalkDelivery>>;
}
