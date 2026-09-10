//! Общий End зарегистрированного навыка GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/states/skill.cpp`, `attackskill.cpp`, `stateskill.cpp`,
//! `summonskill.cpp`. CSkill::End(int) (0x004D84C0) не проверяет IsEnded:
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
//! по сохранённой базе. FIFO, current selection и background здесь не меняются.
//! Concrete payload не удаляется общим End: освобождаются только доказанные
//! ресурсы/поля, HeartLessArrow может вместо End выпустить удерживаемую стрелу.
//! Терминальные границы расписания и explicit End отдельно завершают payload
//! по заранее захваченному ключу. Concrete command-tail не разрешает ID заново;
//! уже снятая им команда не подавляет финализацию того же экземпляра.
//!
//! Полный общий End пока вызывают Rage и Knight. Адресация охватывает экземпляры
//! реестра игрока независимо от payload, но BattleFairy сохраняет прежний End-tail.
//! Перенос общей границы в остальные concrete owners, callbacks временно
//! извлечённого региона/монстра и синхронные DelSkill/ClearSkills ещё требуется.
//! Наличие модели не заменяет эти callers.
//! Производные visual, кроме подключённых видов, не имитируются пустым пакетом.

use crate::gameserver::appserver::moveshape::{MoveShapeSkill, SkillSlot};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::skills::kernel::SkillTermination;
use crate::gameserver::appserver::skills::skillfactory::{
    SkillAfterUse, SkillCategory, SkillEndEffect, SkillEndMovement, SkillEndPathOrder,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

use super::state::{resolve_skill_sufferer, resolve_state_move_shape_mut, resolve_state_user};
use super::visualeffect::SkillVisualEffectKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegisteredPlayerSkill {
    player_id: i32,
    slot: SkillSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegisteredSkillEnd {
    Ended,
    Released,
}

impl CGame {
    pub(crate) fn registered_player_skill(&self, player_id: i32, skill_id: u32) -> Option<RegisteredPlayerSkill> {
        let slot = self.find_player(player_id)?.move_shape().skill_slot(skill_id, self.skill_factory())?;
        Some(RegisteredPlayerSkill { player_id, slot })
    }

    fn registered_skill(&self, address: RegisteredPlayerSkill) -> Option<&MoveShapeSkill> {
        self.find_player(address.player_id)?.move_shape().skill_at(address.slot)
    }

    fn registered_skill_mut(&mut self, address: RegisteredPlayerSkill) -> Option<&mut MoveShapeSkill> {
        self.find_player_mut(address.player_id)?.move_shape_mut().skill_at_mut(address.slot)
    }

    pub(crate) fn finish_registered_player_execution(
        &mut self,
        address: RegisteredPlayerSkill,
        dispatch: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        let Some(skill) = self.registered_skill_mut(address) else { return false };
        if skill.player_dispatch() != Some(dispatch) { return false; }
        skill.finish_base(termination);
        skill.clear_player_execution()
    }

    /// Завершение команды отделено от End самого экземпляра. Потерявшийся
    /// после callback ключ не разрешается заново по ID даже при том же dispatch.
    /// Payload завершается независимо от текущей команды: concrete хвост мог
    /// уже снять свою команду, либо callback мог выбрать другую.
    pub(crate) fn finish_registered_player_command(
        &mut self,
        address: Option<RegisteredPlayerSkill>,
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

    fn registered_skill_user(&self, address: RegisteredPlayerSkill) -> Option<(i32, ShapeIdentity)> {
        let (region, identity) = self.registered_skill(address)?.lifecycle().user();
        Some((region, resolve_state_user(self, region, identity)?))
    }

    fn registered_skill_sufferer(&self, address: RegisteredPlayerSkill) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, self.registered_skill(address)?.lifecycle())
    }

    pub(crate) fn update_player_skill_visual(&mut self, player_id: i32, skill_id: u32, mode: u32) {
        if let Some(address) = self.registered_player_skill(player_id, skill_id) {
            self.update_registered_skill_visual(address, mode);
        }
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredPlayerSkill, mode: u32) {
        let Some(skill) = self.registered_skill(address) else { return };
        let Some(effect) = skill.visual_effect() else { return };
        match effect.kind() {
            SkillVisualEffectKind::Rage =>
                crate::gameserver::appserver::skills::rage::publish_rage_visual(self, skill, mode),
            SkillVisualEffectKind::KnightCut =>
                crate::gameserver::appserver::skills::knightcut::publish_knight_cut_visual(self, skill, mode),
        }
        if let Some(effect) = self.registered_skill_mut(address).and_then(MoveShapeSkill::visual_effect_mut) {
            effect.update_base_tail();
        }
    }

    fn after_use_registered_skill<Runtime: GameMainLoopRuntime>(
        &mut self,
        address: RegisteredPlayerSkill,
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
        self.end_registered_player_instance(address, argument, termination, runtime)
    }

    fn end_registered_player_instance<Runtime: GameMainLoopRuntime>(
        &mut self,
        address: RegisteredPlayerSkill,
        argument: i32,
        termination: SkillTermination,
        runtime: &mut Runtime,
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
        if policy.path_order == SkillEndPathOrder::AfterMovement {
            self.registered_skill_mut(address)?.clear_end_paths();
        }
        if let Some(available) = policy.available {
            self.registered_skill_mut(address)?.lifecycle_mut().set_available(available);
        }
        match policy.effect {
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
        if argument != 0 { self.after_use_registered_skill(address, runtime)?; }
        // CPlayer/CMonster::OnEndSkill — пустой virtual 0x00485540.
        self.registered_skill_mut(address)?.finish_base(termination);
        Some(RegisteredSkillEnd::Ended)
    }
}
