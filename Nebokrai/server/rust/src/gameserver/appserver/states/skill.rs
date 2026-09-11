//! Общий End зарегистрированного навыка GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/states/skill.cpp`, `attackskill.cpp`, `stateskill.cpp`,
//! `summonskill.cpp`, `appserver/moveshape.cpp`. CSkill::End(int) (0x004D84C0) не проверяет IsEnded:
//! GetUser/OnEndSkill предшествуют обнулению базы, удалению visual и ended=true.
//! Native End-политика берётся из единственного каталога concrete владельцев.
//! End(0) не вызывает AfterUse и не читает reuse-часы. Ненулевой End у
//! attack/state/summon вызывает свой AfterUse, затем независимо от GetUser
//! сохраняет свежие часы. Defense пропускает обе операции.
//!
//! Владельцем служит зарегистрированный экземпляр, а не AI/payload. Его
//! поколенческий SlotMap-ключ сохраняется через callbacks: износ оружия может
//! пересчитать TaoZhuang и удалить/заменить навыки. После такого удаления
//! продолжение не касается нового экземпляра с тем же ID или индексом.
//! Владельцы остаются опубликованными; источники и цели разрешаются отдельно
//! по сохранённой базе. Сам End не меняет FIFO, current selection и background.
//! Concrete payload не удаляется общим End: освобождаются только доказанные
//! ресурсы/поля, HeartLessArrow может вместо End выпустить удерживаемую стрелу.
//! Терминальные границы расписания и explicit End отдельно завершают payload
//! по заранее захваченному ключу. Concrete command-tail не разрешает ID заново;
//! уже снятая им команда не подавляет финализацию того же экземпляра.
//!
//! Rage, Knight и BattleFairy используют общий End вместе с owned visual.
//! Собственные End(bool) феи вызывают свой visual-пролог отдельно от политики
//! End(int); AfterUse/base и освобождение ресурса остаются общими.
//! Пролог феи переводит concrete AI в Idle до visual и AfterUse, не трогая
//! lifecycle.ended/source. FatalBlow здесь также обнуляет время полёта;
//! BFBaseAttack сохраняет его. Общая int-политика Po/Yu/transfer не получает
//! этого собственного bool-пролога и его сброса полей.
//! StopAllSkills (0x004CDF50) обходит живой реестр держателя независимо от
//! GetUser/IsEnded. Приручение вызывает его до назначения master. DelSkill
//! (0x004CF320) завершает только разрешённый current и затем удаляет первое
//! совпадение через Drop, без End остальных. Общий Add сохраняет implicit Del
//! при повышении уровня. Remote/script, realm и item-reuse проходят эти границы.
//! Вложенные equipment/war-soul мутации, callbacks извлечённого региона и
//! смерть ещё требуют подключения в точном порядке частичных изменений.
//! ClearSkills не заменяется StopAll: его native current-End и virtual
//! SetCurrentSkill перед деструкторами пока остаются в доказательствах owner-а.
//! Производные visual, кроме подключённых видов, не имитируются пустым пакетом.
//! Нулевой End имеет вход без runtime: он использует тот же derived/base код,
//! но не требует фиктивных часов или реализации износа для region-entry/recall.
//! SpiderMist снимает у держателя техническую curable-регистрацию после
//! возврата движения, до AfterUse/base End. SkillOwner задаёт это действие
//! вместе с остальной политикой; самостоятельная phalanx не принадлежит cast.

use crate::gameserver::appserver::moveshape::{MoveShapeSkill, RegisteredSkillDispatch, SkillSlot};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::skills::kernel::SkillTermination;
use crate::gameserver::appserver::skills::skillfactory::{
    SkillAfterUse, SkillCategory, SkillEndEffect, SkillEndMovement, SkillEndPathOrder, UNKNOWN_SKILL_ID,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

use super::state::{resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut, resolve_state_user};
use super::visualeffect::SkillVisualEffectKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegisteredSkill {
    holder: (i32, ShapeIdentity),
    slot: SkillSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegisteredSkillEnd {
    Ended,
    Released,
}

impl CGame {
    pub(crate) fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<RegisteredSkill> {
        let player = self.find_player(player_id)?;
        self.registered_move_shape_skill(player.shape().get_region_id(), player.shape().identity(), skill_id)
    }

    pub(crate) fn registered_move_shape_skill(
        &self, region_id: i32, holder: ShapeIdentity, skill_id: u32,
    ) -> Option<RegisteredSkill> {
        let slot = resolve_state_move_shape(self, region_id, holder)?.skill_slot(skill_id, self.skill_factory())?;
        Some(RegisteredSkill { holder: (region_id, holder), slot })
    }

    pub(crate) fn registered_skill(&self, address: RegisteredSkill) -> Option<&MoveShapeSkill> {
        resolve_state_move_shape(self, address.holder.0, address.holder.1)?.skill_at(address.slot)
    }

    pub(crate) fn registered_skill_mut(&mut self, address: RegisteredSkill) -> Option<&mut MoveShapeSkill> {
        resolve_state_move_shape_mut(self, address.holder.0, address.holder.1)?.skill_at_mut(address.slot)
    }

    /// CMoveShape::StopAllSkills (0x004CDF50): без IsEnded-gate и без
    /// удаления экземпляров или AI-команд. Первые три категории читают длину
    /// заново; State фиксирует её перед циклом, но перечитывает каждый индекс.
    pub(crate) fn stop_all_move_shape_skills(&mut self, region_id: i32, holder: ShapeIdentity) {
        for category in [SkillCategory::Attack, SkillCategory::Defense, SkillCategory::Summon, SkillCategory::State] {
            let Some(shape) = resolve_state_move_shape(self, region_id, holder) else { return };
            let state_length = (category == SkillCategory::State).then(|| shape.skill_count_in_category(category));
            let mut index = 0;
            loop {
                let Some(shape) = resolve_state_move_shape(self, region_id, holder) else { return };
                let length = state_length.unwrap_or_else(|| shape.skill_count_in_category(category));
                if index >= length { break; }
                let instance = shape.skill_slot_at(category, index)
                    .map(|slot| RegisteredSkill { holder: (region_id, holder), slot });
                if let Some(instance) = instance {
                    let _ = self.end_registered_instance_without_after_use(instance, SkillTermination::Cancelled);
                }
                index += 1;
            }
        }
    }

    fn current_registered_skill(&self, region_id: i32, holder: ShapeIdentity) -> Option<RegisteredSkill> {
        let id = resolve_state_move_shape(self, region_id, holder)?.current_skill_id()?;
        self.registered_move_shape_skill(region_id, holder, id)
    }

    /// DelSkill (0x004CF320) завершает только разрешённый current, даже если
    /// удаляется другой ID. Удаляемый экземпляр получает destructor, не End.
    pub(crate) fn delete_move_shape_skill(&mut self, region_id: i32, holder: ShapeIdentity, skill_id: u32) -> bool {
        if skill_id == UNKNOWN_SKILL_ID { return false; }
        if let Some(instance) = self.current_registered_skill(region_id, holder) {
            if self.registered_skill(instance).is_some_and(|skill| !skill.lifecycle().is_ended()) {
                let _ = self.end_registered_instance_without_after_use(instance, SkillTermination::Cancelled);
            }
            if let Some(shape) = resolve_state_move_shape_mut(self, region_id, holder) {
                shape.set_current_skill_id(None);
            }
        }
        let Some(category) = SkillCategory::from_raw(self.skill_factory().query_skill_type(skill_id, 1)) else { return false };
        let Some(shape) = resolve_state_move_shape_mut(self, region_id, holder) else { return false };
        shape.delete_skill_in_category(skill_id, category);
        true
    }

    /// AddSkill (0x004D1C70): implicit Del выполняется только при повышении
    /// ненулевого уровня, до новой фабричной регистрации в хвост категории.
    pub(crate) fn add_move_shape_skill(&mut self, region_id: i32, holder: ShapeIdentity, skill_id: u32, level: i32) -> bool {
        if let Some(instance) = self.registered_move_shape_skill(region_id, holder, skill_id)
            && let Some(existing) = self.registered_skill(instance)
            && existing.level() != 0
        {
            if level <= existing.level() { return true; }
            let _ = self.delete_move_shape_skill(region_id, holder, skill_id);
        }
        resolve_state_move_shape_mut(self, region_id, holder)
            .is_some_and(|shape| shape.insert_new_skill(skill_id, level))
    }

    pub(crate) fn finish_registered_player_execution(
        &mut self,
        address: RegisteredSkill,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        let Some(skill) = self.registered_skill_mut(address) else { return false };
        if skill.player_dispatch() != Some(dispatch) { return false; }
        skill.finish_base(termination);
        skill.clear_execution(RegisteredSkillDispatch::Player(dispatch))
    }

    /// Завершение команды отделено от End самого экземпляра. Потерявшийся
    /// после callback ключ не разрешается заново по ID даже при том же dispatch.
    /// Payload завершается независимо от текущей команды: concrete хвост мог
    /// уже снять свою команду, либо callback мог выбрать другую.
    pub(crate) fn finish_registered_player_command(
        &mut self,
        address: Option<RegisteredSkill>,
        ai: &mut CPlayerAI,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        let finished = address.is_some_and(|address| self.finish_registered_player_execution(address, dispatch, termination));
        if ai.current_player_skill() == Some(dispatch) {
            ai.release_current_player_command();
            return true;
        }
        finished
    }

    fn registered_skill_user(&self, address: RegisteredSkill) -> Option<(i32, ShapeIdentity)> {
        let (region, identity) = self.registered_skill(address)?.lifecycle().user();
        Some((region, resolve_state_user(self, region, identity)?))
    }

    fn registered_skill_sufferer(&self, address: RegisteredSkill) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, self.registered_skill(address)?.lifecycle())
    }

    pub(crate) fn update_player_skill_visual(&mut self, player_id: i32, skill_id: u32, mode: u32) {
        if let Some(address) = self.registered_player_skill(player_id, skill_id) {
            self.update_registered_skill_visual(address, mode);
        }
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        let Some(skill) = self.registered_skill(address) else { return };
        let Some(effect) = skill.visual_effect() else { return };
        match effect.kind() {
            SkillVisualEffectKind::Rage =>
                crate::gameserver::appserver::skills::rage::publish_rage_visual(self, skill, mode),
            SkillVisualEffectKind::KnightCut =>
                crate::gameserver::appserver::skills::knightcut::publish_knight_cut_visual(self, skill, mode),
            SkillVisualEffectKind::BattleFairy =>
                crate::gameserver::appserver::skills::battlefairyskill::publish_battle_fairy_visual(self, skill, mode),
        }
        if let Some(effect) = self.registered_skill_mut(address).and_then(MoveShapeSkill::visual_effect_mut) {
            effect.update_base_tail();
        }
    }

    fn after_use_registered_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        address: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let skill = self.registered_skill(address)?;
        if skill.owner().category() == SkillCategory::Defense { return Some(()); }
        match skill.owner().after_use() {
            SkillAfterUse::Weapon => {
                if let Some((_, source)) = self.registered_skill_user(address)
                    && source.object_type == 400
                {
                    self.damage_player_weapon(source.id, runtime);
                }
            }
            SkillAfterUse::ItemGroup => {
                if let Some((_, source)) = self.registered_skill_user(address)
                    && source.object_type == 400
                {
                    let skill = self.registered_skill(address)?;
                    if let Some(properties) = self.skill_base_properties(skill.id(), skill.level()) {
                        let group = properties.query_property(0xC351);
                        let now = runtime.now_milliseconds();
                        if let Some(player) = self.find_player_mut(source.id) {
                            player.mark_skill_item_used(group, now);
                        }
                    }
                }
            }
            SkillAfterUse::None => {}
        }
        // Callback мог удалить исходный экземпляр. Его замена не наследует reuse.
        self.registered_skill(address)?;
        let now = runtime.now_milliseconds();
        self.registered_skill_mut(address)?.mark_used(now);
        Some(())
    }

    /// Общий AfterUse/reuse для concrete End, ещё выполняющих свои остальные
    /// части самостоятельно. Это не полный End и не изменение AI-команды.
    pub(crate) fn after_use_player_skill<Runtime: GameMainLoopRuntime>(
        &mut self, player_id: i32, skill_id: u32, runtime: &mut Runtime,
    ) {
        if let Some(address) = self.registered_player_skill(player_id, skill_id) {
            let _ = self.after_use_registered_skill(address, runtime);
        }
    }

    pub(crate) fn end_registered_player_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        skill_id: u32,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
    ) -> Option<RegisteredSkillEnd> {
        let address = self.registered_player_skill(player_id, skill_id)?;
        self.end_registered_instance(address, argument, termination, runtime)
    }

    pub(crate) fn end_registered_instance<Runtime: GameMainLoopRuntime>(
        &mut self,
        address: RegisteredSkill,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
    ) -> Option<RegisteredSkillEnd> {
        if self.prepare_registered_end(address, argument)? == RegisteredSkillEnd::Released {
            return Some(RegisteredSkillEnd::Released);
        }
        if argument != 0 { self.after_use_registered_skill(address, runtime)?; }
        self.finish_registered_base_end(address, termination)
    }

    /// End(0) не требует часов, RNG или контекста износа. Region-entry и отзыв
    /// феи используют ту же derived/base границу, не создавая фиктивный runtime.
    pub(crate) fn end_registered_instance_without_after_use(
        &mut self,
        address: RegisteredSkill,
        termination: SkillTermination,
    ) -> Option<RegisteredSkillEnd> {
        if self.prepare_registered_end(address, 0)? == RegisteredSkillEnd::Released {
            return Some(RegisteredSkillEnd::Released);
        }
        self.finish_registered_base_end(address, termination)
    }

    fn finish_registered_base_end(
        &mut self,
        address: RegisteredSkill,
        termination: SkillTermination,
    ) -> Option<RegisteredSkillEnd> {
        // CPlayer/CMonster::OnEndSkill — пустой virtual 0x00485540.
        self.registered_skill_mut(address)?.finish_base(termination);
        Some(RegisteredSkillEnd::Ended)
    }

    fn prepare_registered_end(
        &mut self,
        address: RegisteredSkill,
        argument: i32,
    ) -> Option<RegisteredSkillEnd> {
        let policy = self.registered_skill(address)?.owner().end_policy();
        if !self.registered_skill_mut(address)?.prepare_derived_end(argument) {
            return Some(RegisteredSkillEnd::Released);
        }
        if policy.path_order == SkillEndPathOrder::BeforeMovement {
            self.registered_skill_mut(address)?.clear_end_paths();
        }
        let movement_target = match policy.movement {
            SkillEndMovement::None => None,
            SkillEndMovement::User => self.registered_skill_user(address),
            SkillEndMovement::UserOrSufferer => self.registered_skill_user(address)
                .or_else(|| self.registered_skill_sufferer(address)),
        };
        if let Some((region, target)) = movement_target
            && let Some(target) = resolve_state_move_shape_mut(self, region, target)
        {
            target.set_moveable(true);
        }
        if policy.release_curable_registration {
            resolve_state_move_shape_mut(self, address.holder.0, address.holder.1)?
                .finish_curable_skill_slot(address.slot);
        }
        if policy.path_order == SkillEndPathOrder::AfterMovement {
            self.registered_skill_mut(address)?.clear_end_paths();
        }
        if let Some(available) = policy.available {
            self.registered_skill_mut(address)?.lifecycle_mut().set_available(available);
        }
        self.prepare_registered_skill_end_effect(address, policy.effect, argument)?;
        Some(RegisteredSkillEnd::Ended)
    }

    pub(crate) fn prepare_registered_skill_end_effect(
        &mut self, address: RegisteredSkill, effect: SkillEndEffect, argument: i32,
    ) -> Option<()> {
        if matches!(effect, SkillEndEffect::BattleFairyBaseMagic | SkillEndEffect::BattleFairyState
            | SkillEndEffect::BattleFairyFatal | SkillEndEffect::BattleFairySummon)
            && let Some(execution) = self.registered_skill_mut(address)?.battle_fairy_execution_state_mut()
        {
            execution.prepare_derived_end();
        }
        match effect {
            SkillEndEffect::None => {}
            SkillEndEffect::Rage => {
                if let Some((_, source)) = self.registered_skill_user(address)
                    && source.object_type == 400
                {
                    let _ = self.publish_player_states(source.id);
                }
                self.update_registered_skill_visual(address, 3);
            }
            SkillEndEffect::ScorpionOnlyZero if argument != 0 => {}
            SkillEndEffect::BattleFairyBaseMagic | SkillEndEffect::BattleFairyState => {
                if self.registered_skill_user(address).is_some() {
                    if let Some(effect) = self.registered_skill_mut(address)?.visual_effect_mut() {
                        effect.begin_visual_effect(1);
                    }
                    self.update_registered_skill_visual(address, 3);
                }
            }
            SkillEndEffect::ScorpionOnlyZero | SkillEndEffect::Star
            | SkillEndEffect::BattleFairyFatal | SkillEndEffect::BattleFairySummon => {
                self.update_registered_skill_visual(address, 3);
            }
        }
        Some(())
    }
}
