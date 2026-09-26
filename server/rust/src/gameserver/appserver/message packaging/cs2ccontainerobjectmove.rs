//! Клиентский container-move `0xC0101` старого GameServer перенесён в Zone
//! items волной Z-C5 (build-only: `SetOperation` подкоды, self-move
//! normalization и объектный кодек с tail по операции). Здесь реэкспорт для
//! старого пакета и sender-шов доставки: RLE send-family собранного кадра
//! одному player и разрешение player-owner-ов session ordered plug registry
//! `CSessionFactory` остаются у `CGame`.

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::GameMessageDomainOps;
use nebokrai_zone::app::game_message::CMessage;

pub(crate) use nebokrai_zone::items::cs2ccontainerobjectmove::*;

impl ContainerObjectMessageSender for CGame {
    fn send_container_object_message(&self, message: &CMessage, player_id: i32) -> i32 {
        message.send_to_player(self.net_server(), player_id)
    }

    fn session_player_ids(&self, session_id: i32) -> Vec<i32> {
        CGame::session_player_ids(self, session_id)
    }
}
