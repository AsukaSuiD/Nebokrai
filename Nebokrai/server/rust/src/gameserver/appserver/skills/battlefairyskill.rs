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
//! Очередь и background сохраняют поколенческий ключ до Begin/AI/contacts.
//! Общий End читает GetUser из той же базы, выполняет AfterUse/reuse и лишь
//! затем очищает source/target/visual и освобождает BF-payload. Удаление навыка
//! из callback износа не переносит продолжение на новую регистрацию. AI на
//! время этого callback опубликован у игрока; его новая команда не снимается.
//! End(0) смены региона/отзыва использует ту же границу без runtime и часов.
//! Отказ Begin сбрасывает созданную базу даже без payload, после чего расписание
//! отправляет единственный внешний 4,2. Внутренние отказы владельцев сохранены.
//! BF owned visual ещё не материализован: concrete owners пока публикуют свои
//! wire-End сами. Пакет не подменяет ресурс, его Begin/Update/Drop и source-gates;
//! перенос этих публикаций в общий visual-dispatch остаётся отдельной зависимостью.

use super::basemagic::BASE_MAGIC_EFFECT_MESSAGE;
use super::kernel::SkillTermination;
use crate::gameserver::appserver::moveshape::RegisteredSkillDispatch;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredPlayerSkill;
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
            let player_ai = self.find_player(player_id)?.player_ai();
            let materialized = self.battle_fairy_skill_execution_is_materialized(player_id, player_ai);
            let dispatch = player_ai.current_battle_fairy_skill();
            (materialized, dispatch)
        };
        let dispatch = dispatch?;
        let instance = self.registered_player_skill(player_id, dispatch.skill_id());
        self.find_player_mut(player_id)?.player_ai_mut().release_current_battle_fairy_command();
        if materialized && let Some(instance) = instance {
            self.send_battle_fairy_skill_end(player_id, dispatch);
            let _ = self.end_registered_player_instance_without_after_use(instance, SkillTermination::Cancelled);
            if let Some(skill) = self.registered_skill_mut(instance) {
                skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
            }
        }
        Some(dispatch)
    }

    /// Терминальная граница после concrete owner и contacts. Wire-End пока
    /// публикует concrete owner; общий End сохраняет источник до AfterUse и
    /// только затем сбрасывает базу. Callback не может перенести reuse/cleanup
    /// на новую регистрацию того же ID. Неуспешный Begin не требует payload.
    pub(crate) fn finish_registered_battle_fairy_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        instance: RegisteredPlayerSkill,
        dispatch: BattleFairySkillDispatch,
        argument: i32,
        termination: SkillTermination,
        begin_attempted: bool,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = self.registered_skill(instance) else { return false };
        let materialized = skill.battle_fairy_dispatch() == Some(dispatch);
        let failed_begin = begin_attempted && !skill.lifecycle().is_ended()
            && skill.is_execution_inactive();
        if !materialized && !failed_begin {
            return false;
        }
        let argument = if materialized { argument } else { 0 };
        let _ = self.end_registered_player_instance(instance, argument, termination, runtime);
        if let Some(skill) = self.registered_skill_mut(instance) {
            skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
        }
        true
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
