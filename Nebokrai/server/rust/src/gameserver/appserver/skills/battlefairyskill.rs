//! Общая wire-граница семейства навыков боевого духа.
//! Диапазон 0x212..=0x224 не получает искусственный prepared: в 19 телах
//! AI (0x00517610..0x0052AAC0) нет записи флага владельца +0x44.
//! Summon BFBaseAttack (0x00517E40), Leiming2 (0x00520990), Thunder
//! (0x00521D30) и Tianhuo (0x00523280) также не выставляют этот флаг;
//! вызывающий AI завершает навык через End(1), а созданная область живёт
//! самостоятельно. Например, Leiming2 после virtual Summon(+0x8C)
//! в 0x005208AB сразу переходит к End(1). Общая prepared-ветвь CPlayerAI
//! не является основанием продлевать исполнение этих конкретных навыков.
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
//! Ошибки AI атак 0x21a..=0x21f вызывают End(0): Tianhuo (0x00522e5a),
//! Leiming2 (0x005206b1), FatalBlow (0x0051f1ac), BloodLoss (0x0051b4af),
//! PoisonArrow (0x00519e0c), Thunder (0x00521a51). Это не внешний
//! OnLoseTargetWarSoul, прерывающий ещё активный навык через End(1).
//! После собственного End(0) навыка повторный OnLoseTargetWarSoul видит
//! IsEnded и не добавляет отказ, износ оружия или cooldown.
//! Тот же End(0) подтверждён для отказов Po (AI 0x00527a40..0x0052aac0),
//! LifeShield (0x00518d66), Wangsheng (0x0051de92), Huoxieshu (0x0051d3a9)
//! и Lingzhishu (0x0051c76a). У Yu есть отдельный отказ с End(1) при смерти
//! цели; владелец передаёт его как RejectedAfterUse, не как обычный Rejected.
//! Базовая атака 0x224 также различает End(0) при потере цели и End(1)
//! после попытки Summon (AI 0x005178fb/0x005179df).

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

    /// Хвост уже материализованного навыка боевого духа. Его владелец уже
    /// отправил action `3` и очистил собственные поля. Подтверждённый `End(0)`
    /// не получает побочные эффекты успешного завершения и повторный отказ.
    pub(crate) fn finish_battle_fairy_skill_end_tail<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        dispatch: BattleFairySkillDispatch,
        player_ai: &mut CPlayerAI,
        reject_request: bool,
        runtime: &mut Runtime,
    ) {
        if reject_request && (0x212..=0x224).contains(&dispatch.skill_id()) {
            return;
        }
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
