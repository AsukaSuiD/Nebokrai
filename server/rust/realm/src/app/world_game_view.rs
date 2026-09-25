//! Узкий game-view для обработчиков мировых сообщений Realm.

use nebokrai_shared::network::ServerCommandHandle;

use crate::app::auction::WorldBaiTanRemoval;
use crate::app::world_client::CMyNetClient;
use crate::app::world_message::{CMessage, SendMessageError};
use crate::persistence::writelog::WorldWriteLogCommand;
use crate::characters::player::{CPlayer, PlayerCodecError, PlayerPropertyCoefficients};
use crate::content::goods::GoodsBasePropertiesRegistry;

/// Снимок состояния game server-а: наличие коннекта и числовой индекс маршрута.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldGameServerSnapshot {
    pub connected: bool,
    pub index: u32,
}

/// Точки обратного вызова handler-ветвей мира в владельца игры. Реализация
/// живёт у владельца игры (старый `CGame`) и делегирует его inherent-
/// методам; имена намеренно совпадают — inherent priority исключает рекурсию.
pub trait WorldGameView {
    fn current_game_server_sender(&self) -> Option<ServerCommandHandle>;

    fn send_msg_to_game_server(
        &self,
        map_id: i32,
        message: &CMessage,
    ) -> Result<i32, SendMessageError>;

    fn publish_team_session(&mut self, team_id: u32, session_id: i32);

    fn remove_team_session(&mut self, team_id: u32);

    fn get_team_session_id(&self, team_id: u32) -> i32;

    fn player_game_server(&self, player_id: i32) -> Option<WorldGameServerSnapshot>;

    fn map_player(&self, player_id: u32) -> Option<&CPlayer>;

    fn online_player_id_by_name(&self, name: &[u8]) -> u32;

    fn online_player_by_cdkey(&self, cdkey: &[u8]) -> Option<&CPlayer>;

    fn configured_world_number(&self) -> Option<u32>;

    fn current_login_client(&self) -> Option<&CMyNetClient>;

    fn game_server_number_by_player_id(&self, player_id: i32) -> i32;

    fn legacy_tick_ms(&self) -> u32;

    fn set_map_player_jjc_identity(&mut self, player_id: u32, level: u8, jjc_level: u32) -> bool;

    fn set_map_player_jjc_snapshot(
        &mut self,
        player_id: u32,
        level: u8,
        jjc_level: u32,
        jjc_score: u32,
        counters: [u8; 0x10],
    ) -> bool;

    fn online_player_by_id(&self, player_id: u32) -> Option<&CPlayer>;

    fn decord_online_player_by_id(
        &mut self,
        player_id: u32,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError>;

    fn add_item_to_bai_tan_request_list(&mut self, ip: u32, player_id: i32) -> bool;

    fn del_item_from_bai_tan_list(&mut self, player_id: i32) -> WorldBaiTanRemoval;

    fn game_server(&self, index: u32) -> Option<WorldGameServerSnapshot>;

    fn push_write_log_command(&self, command: WorldWriteLogCommand) -> usize;
}

