//! Общая wire-граница семейства навыков боевого духа.
//!
//! Пакет `0xBFE01` с префиксом отказа `4` и общий `End(0)` с действием `3`
//! используются несколькими конкретными владельцами навыков. Здесь остаётся
//! только общий состав packet-ов; проверки, формулы и жизненный цикл принадлежат
//! соответствующим модулям-владельцам.

use super::basemagic::BASE_MAGIC_EFFECT_MESSAGE;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;

impl CGame {
    pub(crate) fn send_battle_fairy_skill_failure(&self, player_id: i32, action: u8) {
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(4);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn send_battle_fairy_skill_end(
        &mut self,
        player_id: i32,
        dispatch: BattleFairySkillDispatch,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(3);
        message.add_long(dispatch.skill_id() as i32);
        message.add_short(dispatch.skill_level() as i16);
        message.add_long(BATTLE_FAIRY_VISUAL_OBJECT_TYPE);
        message.add_long(player_id);
        message.add_long(player.shape().get_direction());
        let _ = self.send_player_shape_around(player_id, None, &message);
    }
}
