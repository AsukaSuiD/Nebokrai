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
//! End(0) пространственных SetWarSoulXY/DelWarSoul использует ту же границу
//! без runtime и часов. Выбор идёт из m_tgCurrentWarSoulSkill, без gates по
//! current-команде/payload и без снятия команды; внешний int-End не заменяется
//! concrete bool-End. Set требует target area, Delete проверяет навык до area.
//! Отказ Begin сбрасывает созданную базу даже без payload, после чего расписание
//! отправляет единственный внешний 4,2. Внутренние отказы владельцев сохранены.
//! Все 19 skill-owned эффектов боевой феи выделяют ровно 0xC байт: только
//! CVisualEffect, без собственных полей. Общий ресурс создаётся после базового
//! Begin и до concrete checks; ID, уровень, GetUser/GetSufferer берутся из
//! живого экземпляра, не из команды. Все Update, включая silent/ended/missing
//! source, проходят общий base tail; wire mode 3 сам по себе не завершает ресурс.
//! Различия wire сведены в один статический контракт: они не создают новых
//! контейнеров или копий lifecycle. В частности, BloodLoss mode13 = [0,13],
//! Leiming2 mode7 = [0,7], Po/Yu/transfer mode8 = DWORD(4), byte(8).
//! Tianhuo mode1 сохраняет wire-ID 0x13A, LifeShield mode1 — лишь target 0,0.
//! Native Fatal/Leiming временно меняют source.type на 700; around определяет
//! CPlayer через RTTI (0x0041460F), не это поле. Rust сохраняет живую форму и
//! пишет 700 в wire: existing around сохраняет также дальний team-tail, без
//! мутации типа, копии owning shape или отдельного транспорта.
//! Po/Yu/Wangsheng/Huoxieshu/Lingzhishu вызывают из Begin/AI End(bool)+0x94:
//! Po/Yu (0x005246C0) rearm/update3 при source, transfer/Wang (0x0051BE50)
//! update3 без rearm, затем inherited End. Этот вызов не подменяет их внешний
//! End(int)+0x68, которому по-прежнему принадлежит отдельная фабричная политика.
//! Собственный пролог перед visual отключает concrete AI, не завершая базу
//! заранее. Idle не означает Begin и не служит отметкой tick: единый вход AI
//! в этой фазе молчит, пока source/effect остаются доступны AfterUse.

use super::basemagic::BASE_MAGIC_EFFECT_MESSAGE;
use super::kernel::{BattleFairyExecution, SkillTermination};
use super::skillfactory::{SkillEndEffect, SkillOwner};
use crate::gameserver::appserver::moveshape::{MoveShapeSkill, RegisteredSkillDispatch};
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;

#[derive(Clone, Copy)]
enum BattleFairyFireTarget { Optional, Required, Point, Source, LifeShield }

/// Только различия wire конкретных классов. База ресурса и её lifecycle
/// общие; здесь нет копий source, target, уровня или исполнения навыка.
struct BattleFairyVisualContract {
    target: BattleFairyFireTarget,
    failure_modes: &'static [u32],
    fire_time: bool,
    source_type_on_fire: bool,
    fire_id: Option<u32>,
    zero_prefix_mode: Option<u32>,
    wide_failure_eight: bool,
    boolean_end_effect: Option<SkillEndEffect>,
}

impl BattleFairyVisualContract {
    fn for_owner(owner: SkillOwner) -> Option<Self> {
        use BattleFairyFireTarget as Target;
        use SkillOwner::*;
        let mut result = Self {
            target: Target::Required, failure_modes: &[2, 7, 10, 11, 13, 15],
            fire_time: false, source_type_on_fire: false, fire_id: None,
            zero_prefix_mode: None, wide_failure_eight: false,
            boolean_end_effect: None,
        };
        match owner {
            BFBaseAttack => { result.target = Target::Optional; result.fire_time = true; }
            CFatalBlow => {
                result.fire_time = true; result.failure_modes = &[2, 7, 10, 11, 13, 14, 15];
            }
            CThunder | CLeiming2 => {
                result.target = Target::Point;
                if owner == CLeiming2 { result.zero_prefix_mode = Some(7); }
            }
            CTianhuo => { result.target = Target::Optional; result.fire_id = Some(0x13a); }
            CBloodLoss | CPoisonArrow => {
                result.failure_modes = &[2, 7, 8, 10, 11, 13, 14, 15];
                if owner == CBloodLoss { result.zero_prefix_mode = Some(13); }
            }
            CLifeShield => { result.target = Target::LifeShield; result.failure_modes = &[2, 7, 8, 13, 14]; }
            CPojia | CPobing | CPomo | CPofa | CYujia | CYubing | CYumo | CYufa
            | CHuoxieshu | CLingzhishu | CWangsheng => {
                if !matches!(owner, CPojia | CPobing | CPomo | CPofa) { result.target = Target::Source; }
                result.source_type_on_fire = true; result.wide_failure_eight = true;
                result.failure_modes = if owner == CHuoxieshu { &[2, 6, 8, 13] } else { &[2, 7, 8, 13] };
                result.boolean_end_effect = Some(if matches!(owner, CHuoxieshu | CLingzhishu | CWangsheng) {
                    SkillEndEffect::BattleFairySummon
                } else {
                    SkillEndEffect::BattleFairyState
                });
            }
            _ => return None,
        }
        Some(result)
    }
}

pub(crate) fn publish_battle_fairy_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    let Some(contract) = BattleFairyVisualContract::for_owner(skill.owner()) else { return };
    if skill.visual_effect().is_none_or(|effect| effect.kind() != SkillVisualEffectKind::BattleFairy || effect.is_ended()) {
        return;
    }
    let (region_id, source_identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region_id, source_identity) else { return };
    let source = source.shape();
    let identity = source.identity();
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    if contract.failure_modes.contains(&mode) {
        if identity.object_type != 400 { return; }
        if mode == 8 && contract.wide_failure_eight { message.add_long(4); }
        else { message.add_byte(if contract.zero_prefix_mode == Some(mode) { 0 } else { 4 }); }
        message.add_byte(mode as u8);
        let _ = message.send_to_player(game.net_server(), identity.id);
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    message.add_byte(action);
    message.add_long(if action == 2 { contract.fire_id.unwrap_or(skill.id()) } else { skill.id() } as i32);
    message.add_short(skill.level() as i16);
    message.add_long(if action == 2 && contract.source_type_on_fire { identity.object_type } else { BATTLE_FAIRY_VISUAL_OBJECT_TYPE });
    message.add_long(identity.id);
    if action == 2 {
        let sufferer = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, target)| resolve_state_move_shape(game, region, target))
            .map(|target| target.shape());
        if matches!(contract.target, BattleFairyFireTarget::Required | BattleFairyFireTarget::LifeShield)
            && sufferer.is_none() { return; }
        let target = if matches!(contract.target, BattleFairyFireTarget::Source) { Some(source) } else { sufferer };
        let target_identity = target.map(|target| target.identity());
        let zero_identity = matches!(contract.target, BattleFairyFireTarget::Point | BattleFairyFireTarget::LifeShield);
        message.add_long(if zero_identity { 0 } else { target_identity.map_or(0, |identity| identity.object_type) });
        message.add_long(if zero_identity { 0 } else { target_identity.map_or(0, |identity| identity.id) });
        if !matches!(contract.target, BattleFairyFireTarget::LifeShield) {
            let (x, y) = match target {
                Some(target) => {
                    let (Ok(x), Ok(y)) = (target.get_tile_x(), target.get_tile_y()) else { return };
                    (x, y)
                }
                None => skill.lifecycle().destination(),
            };
            message.add_long(x); message.add_long(y);
        }
        if contract.fire_time {
            let time = match skill.battle_fairy_execution_state() {
                Some(BattleFairyExecution::BaseMagic(state)) => state.attack_time(),
                Some(BattleFairyExecution::FatalBlow(state)) => state.missile_flying_time(),
                _ => 0,
            };
            message.add_ulong(time);
        }
    } else { message.add_long(source.get_direction()); }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

impl CGame {
    /// SetWarSoulXY (0x0042DF50) и DelWarSoul (0x0042E0A0) вызывают End(int,0)
    /// выбранного зарегистрированного навыка, если его база ещё не ended.
    /// Payload и текущая команда не являются gates; очередь и target не снимаются.
    /// Собственный End(bool) Po/Yu/transfer сюда не подставляется.
    pub(crate) fn cancel_active_battle_fairy_skill(
        &mut self,
        player_id: i32,
    ) -> bool {
        let Some(skill_id) = self.find_player(player_id).map(|player| player.player_ai().selected_battle_fairy_skill_id()) else { return false };
        let Some(instance) = self.registered_player_skill(player_id, skill_id) else { return false };
        let Some(skill) = self.registered_skill(instance) else { return false };
        if skill.lifecycle().is_ended() { return false; }
        let dispatch = skill.battle_fairy_dispatch();
        let _ = self.end_registered_instance_without_after_use(instance, SkillTermination::Cancelled);
        if let Some(dispatch) = dispatch && let Some(skill) = self.registered_skill_mut(instance) {
            skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
        }
        true
    }

    /// Терминальная граница после concrete owner и contacts. Выполняет точный
    /// concrete End(bool/int); общий End сохраняет источник до AfterUse и
    /// только затем сбрасывает базу. Callback не может перенести reuse/cleanup
    /// на новую регистрацию того же ID. Неуспешный Begin не требует payload.
    pub(crate) fn finish_registered_battle_fairy_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        instance: RegisteredSkill,
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
        self.prepare_battle_fairy_boolean_end(instance);
        let _ = self.end_registered_instance(instance, argument, termination, runtime);
        if let Some(skill) = self.registered_skill_mut(instance) {
            skill.clear_execution(RegisteredSkillDispatch::BattleFairy(dispatch));
        }
        true
    }

    fn prepare_battle_fairy_boolean_end(&mut self, instance: RegisteredSkill) {
        if let Some(effect) = self.registered_skill(instance)
            .and_then(|skill| BattleFairyVisualContract::for_owner(skill.owner()))
            .and_then(|contract| contract.boolean_end_effect)
        {
            let _ = self.prepare_registered_skill_end_effect(instance, effect, 0);
        }
    }

    pub(crate) fn send_battle_fairy_skill_failure(&self, player_id: i32, action: u8) {
        let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
        message.add_byte(4);
        message.add_byte(action);
        let _ = message.send_to_player(self.net_server(), player_id);
    }

}
