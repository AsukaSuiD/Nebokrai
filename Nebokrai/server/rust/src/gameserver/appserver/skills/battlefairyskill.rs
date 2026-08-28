//! Общая wire-граница семейства навыков боевого духа.
//!
//! Пакет отказа `0xBFE01` с префиксом `4` используется несколькими
//! конкретными владельцами навыков. Здесь остаётся только общий состав пакета;
//! проверки, формулы и жизненный цикл принадлежат соответствующим модулям-владельцам.

use super::basemagic::BASE_MAGIC_EFFECT_MESSAGE;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

impl CGame {
    pub(crate) fn send_battle_fairy_skill_failure(&self, player_id: i32, action: u8) {
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(4);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }
}
