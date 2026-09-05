//! Общая wire-граница семейства навыков боевого духа.
//!
//! Пакет `0xBFE01` с префиксом отказа `4` и общий `End(0)` с действием `3`
//! используются несколькими конкретными владельцами навыков. Точные общие
//! тела `End` по адресам `0x0051A700`, `0x0051BE50`, `0x0051E370`,
//! `0x005222A0` и `0x005246C0` делегируют в `CStateSkill::End` либо
//! `CSummonSkill::End`: успешный хвост изнашивает оружие, а cooldown
//! фиксируется последним. Источник — gameserver.exe + GameServer.pdb,
//! appserver/states/skill.cpp: CSkill::End (0x004d84c0) вызывает virtual
//! +0x158 источника, который у CPlayer пуст. Дополнительного пересчёта
//! свойств здесь нет; изменения состояний обрабатывают конкретные владельцы.
//! Здесь остаётся общий терминальный хвост и состав packet-ов; проверки и
//! формулы принадлежат соответствующим модулям-владельцам.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use super::basemagic::BASE_MAGIC_EFFECT_MESSAGE;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;

impl CGame {
    /// Общий `End(false)` пространственного удаления или повторного запроса:
    /// action `3` предшествует `CSkill::End`, но оружейный `AfterUseSkill` и
    /// cooldown не выполняются. Команда без concrete `Begin` не имеет `End`.
    pub(crate) fn cancel_active_battle_fairy_skill(
        &mut self,
        player_id: i32,
    ) -> Option<BattleFairySkillDispatch> {
        let (materialized, dispatch) = {
            let player = self.find_player_mut(player_id)?;
            let player_ai = player.player_ai_mut();
            let materialized = player_ai.battle_fairy_skill_execution_is_materialized();
            let dispatch = player_ai.cancel_active_battle_fairy_skill();
            (materialized, dispatch)
        };
        let dispatch = dispatch?;
        if materialized {
            self.send_battle_fairy_skill_end(player_id, dispatch);
        }
        Some(dispatch)
    }

    /// Общий хвост `End(true)` уже материализованного war-soul skill. Concrete
    /// owner к этому моменту отправил action `3` и очистил собственные поля;
    /// `OnLoseTargetWarSoul` добавляет общий отказ `4,2` после полного `End`.
    pub(crate) fn finish_battle_fairy_skill_end_tail<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        dispatch: BattleFairySkillDispatch,
        player_ai: &mut CPlayerAI,
        reject_request: bool,
        runtime: &mut Runtime,
    ) {
        self.damage_player_weapon(player_id, runtime);
        let _ = player_ai.mark_battle_fairy_skill_used(
            dispatch.skill_id(),
            runtime.now_milliseconds(),
        );
        if reject_request {
            self.send_battle_fairy_skill_failure(player_id, 2);
        }
    }

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
