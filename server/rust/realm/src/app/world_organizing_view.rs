//! Узкий organizing-view для обработчиков мировых сообщений Realm.

use crate::app::world_game_view::WorldGameView;
use crate::organizations::faction::{
    FactionInitialPropertyBlock, FactionTalkDelivery, OwnedCityAddOutcome,
    OwnedCityMutationBuildError,
};
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

/// Точки обратного вызова `CWorldRegion::init_owner_relation` в organizing
/// и игру. Inherent `add_owned_city_to_faction` требует `&CGame` (обновление
/// owned-city wire и faction info), поэтому реализация живёт в адаптере-
/// мосте у единственного call-site инициализации, который связывает
/// `&mut COrganizingCtrl` и `&CGame`; отдельный трейт вместо методов
/// `WorldOrganizingView`, чтобы не перетаскивать `&CGame` в чужой контракт.
pub trait WorldRegionOwnerOrganizingView {
    fn has_faction(&self, faction_id: i32) -> bool;

    fn has_confederation(&self, union_id: i32) -> bool;

    fn add_owned_city_to_faction(
        &mut self,
        faction_id: i32,
        region_id: i32,
        update_player: &mut dyn FnMut(i32),
    ) -> Result<Option<OwnedCityAddOutcome>, OwnedCityMutationBuildError>;

    fn country_by_faction(
        &self,
        faction_id: i32,
    ) -> Result<Option<u8>, FactionInitialPropertyBlock>;
}
