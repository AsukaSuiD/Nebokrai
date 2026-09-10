//! Общая граница достигнутых навыков-состояний игрока.
//!
//! Несколько конкретных владельцев используют одинаковый каркас пакета
//! отказа и начала/завершения каста. Значения `opcode`, `skill ID` и `action`
//! передаёт конкретный навык; здесь сохраняется только общий порядок полей.
//! Источник: gameserver.exe + GameServer.pdb, appserver/states/stateskill.cpp.
//! CStateSkill::End (0x005dfbd0) при успехе вызывает AfterUseSkill через +0x8c,
//! затем CSkill::End (0x004d84c0). Обычный AfterUseSkill (0x0053cf30) вызывает
//! только CPlayer::OnWeaponDamaged. Виртуальный +0x158 базового End у игрока
//! пуст: пересчёт при наложении состояния остаётся у конкретного навыка,
//! здесь он не повторяется. Хвост фиксирует cooldown, но не меняет выбранный
//! навык игрока: пустой слот 0x00485540 не сбрасывает m_pCurrentSkill.
//! Смена выбора принадлежит OnChangeSkill/OnLoseTarget, не End экземпляра.
//! AfterUse и reuse выполняет общая граница зарегистрированного экземпляра:
//! virtual override берётся из factory-каталога, износ относится к GetUser,
//! а ключ экземпляра сохраняется через вызванный износом пересчёт экипировки.

use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::gameserver::game::GameMainLoopRuntime;
use crate::nets::netserver::message::CMessage;

pub(crate) fn finish_state_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    game.after_use_player_skill(player_id, skill_id, runtime);
}

impl CGame {
    pub(crate) fn send_self_state_skill_failure(
        &self,
        message_type: i32,
        player_id: i32,
        action: u8,
    ) {
        let mut message = CMessage::new(message_type);
        message.add_byte(0);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

    pub(crate) fn send_self_state_skill_cast(
        &mut self,
        message_type: i32,
        player_id: i32,
        skill_id: u32,
        skill_level: i32,
        action: u8,
    ) {
        let Some(player) = self.find_player(player_id) else {
            return;
        };
        let identity = player.shape().identity();
        let mut message = CMessage::new(message_type);
        message.add_byte(action);
        message.add_long(skill_id as i32);
        message.base_mut().add_short(skill_level as i16);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        if action == 1 {
            message.add_long(player.shape().get_direction());
        } else {
            message.add_long(identity.object_type);
            message.add_long(identity.id);
            message.add_long(player.shape().get_tile_x().unwrap_or_default());
            message.add_long(player.shape().get_tile_y().unwrap_or_default());
        }
        let _ = self.send_player_shape_around(player_id, None, &message);
    }
}
