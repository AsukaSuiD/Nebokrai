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

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::gameserver::game::GameMainLoopRuntime;
use crate::nets::netserver::message::CMessage;

pub(crate) fn finish_state_skill<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    game.damage_player_weapon(player_id, runtime);
    mark_used(player_ai, runtime.now_milliseconds());
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
