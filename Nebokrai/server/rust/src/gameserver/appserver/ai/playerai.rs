//! Достигнутая часть очередей и исполнения `CPlayerAI` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/ai/playerai.cpp`. Трёхаргументный virtual `MoveTo` RVA
//! `0x0010A480` для живого игрока очищает эмоцию, удаляет старейшие назначения
//! до длины не более трёх и затем добавляет `(direction, is_run)`;
//! так очередь после вызова содержит не более четырёх элементов. Здоровье
//! владельца и `ClearEmotion` остаются у вызывающей стороны, чтобы не хранить сырые
//! указатели внутри ИИ. Канонический `CPlayer` владеет очередями навыков;
//! `CMoveShape::AI` передаёт первый элемент конкретному исполнителю и удаляет
//! его только после завершения либо отказа. Базовая атака, базовая магия,
//! стрельба, семейство ловкости, парная закалка, воодушевление, управление
//! питомцами, усиление, периодическое лечение, огненная стрела, огненная
//! стена, огненный круг, молния, печать, инь-ян, божественная кара, сбор душ
//! и зеркало душ,
//! сфера хаоса и семь падающих звёзд,
//! машинный и мана-щит,
//! оглушение, ослабление, очищение,
//! атака боевой феи и её призываемые области
//! и приручение монстров сохраняют незавершённое состояние между проходами ИИ.
//! Хвост
//! `CPlayerAI::Run` хранит часы
//! автоматического прироста,
//! использует сохранённые факты игрока и фракции и соблюдает беззнаковую
//! проверку срока. Прирост опыта возвращается в полный `CGame::CheckLevel`,
//! энергия публикуется адресным сообщением `0xBF72C`.
//! Четырёхаргументный поиск пути `MoveTo` и остальные
//! методы ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use std::collections::VecDeque;

use crate::gameserver::appserver::player::{
    BattleFairySkillDispatch, CPlayer, PlayerSkillDispatch,
};
use crate::gameserver::appserver::skills::agility::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AgilityFamilyExecutionState,
};
use crate::gameserver::appserver::skills::archery::ArcheryExecutionState;
use crate::gameserver::appserver::skills::baseattack::BaseAttackExecutionState;
use crate::gameserver::appserver::skills::basemagic::BaseMagicExecutionState;
use crate::gameserver::appserver::skills::lightning::LightningExecutionState;
use crate::gameserver::appserver::skills::seal::SealExecutionState;
use crate::gameserver::appserver::skills::battlefairybasemagic::BattleFairyBaseMagicExecutionState;
use crate::gameserver::appserver::skills::battlefairytransfer::BattleFairyTransferKind;
use crate::gameserver::appserver::skills::callosity::CallosityExecutionState;
use crate::gameserver::appserver::skills::chaossphere::ChaosSphereExecutionState;
use crate::gameserver::appserver::skills::chainlightning::ChainLightningExecutionState;
use crate::gameserver::appserver::skills::kernel::{
    SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::appserver::skills::natural::NATURAL_SKILL_ID;
use crate::gameserver::appserver::skills::rapture::RAPTURE_SKILL_ID;
use crate::gameserver::appserver::skills::sevenshootingstar::SevenShootingStarExecutionState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAiDestination {
    pub(crate) direction: i32,
    pub(crate) is_run: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerAI {
    destinations: VecDeque<PlayerAiDestination>,
    player_skills: VecDeque<PlayerSkillDispatch>,
    battle_fairy_skills: VecDeque<BattleFairySkillDispatch>,
    base_attack: Option<BaseAttackExecutionState>,
    base_attack_last_used_ms: u32,
    archery: Option<ArcheryExecutionState>,
    archery_last_used_ms: u32,
    agility_family: Option<AgilityFamilyExecutionState>,
    agility_family_last_used_ms: [u32; 4],
    base_magic: Option<BaseMagicExecutionState>,
    base_magic_last_used_ms: u32,
    fire_bolt: Option<BaseMagicExecutionState>,
    fire_bolt_last_used_ms: u32,
    fire_ball: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    fire_ball_last_used_ms: u32,
    chain_lightning: Option<ChainLightningExecutionState>,
    chain_lightning_last_used_ms: u32,
    thunder_blow: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    thunder_blow_last_used_ms: u32,
    thunder_blow_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    thunder_blow_2_last_used_ms: u32,
    mosou: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    mosou_last_used_ms: u32,
    fire_wall: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    fire_wall_last_used_ms: u32,
    infernol: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    infernol_last_used_ms: u32,
    seven_shooting_star: Option<SevenShootingStarExecutionState>,
    seven_shooting_star_last_used_ms: u32,
    chaos_sphere: Option<ChaosSphereExecutionState>,
    chaos_sphere_last_used_ms: u32,
    lightning: Option<LightningExecutionState>,
    lightning_last_used_ms: u32,
    seal: Option<SealExecutionState>,
    seal_last_used_ms: u32,
    yin_yang: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    yin_yang_last_used_ms: u32,
    yin_yang_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    yin_yang_2_last_used_ms: u32,
    god_punishment: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_punishment_last_used_ms: u32,
    god_thunder: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_thunder_last_used_ms: u32,
    god_thunder_2: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_thunder_2_last_used_ms: u32,
    soul_collect: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    soul_collect_last_used_ms: u32,
    soul_mirror: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    soul_mirror_last_used_ms: u32,
    battle_fairy_base_magic: Option<BattleFairyBaseMagicExecutionState>,
    battle_fairy_base_magic_last_used_ms: u32,
    life_shield: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    life_shield_last_used_ms: u32,
    battle_fairy_transfer: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    battle_fairy_transfer_last_used_ms: [u32; 2],
    wangsheng: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    wangsheng_last_used_ms: u32,
    poison_arrow: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    poison_arrow_last_used_ms: u32,
    blood_loss: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    blood_loss_last_used_ms: u32,
    fatal_blow: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    fatal_blow_last_used_ms: u32,
    thunder: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    thunder_last_used_ms: u32,
    leiming2: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    leiming2_last_used_ms: u32,
    tianhuo: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    tianhuo_last_used_ms: u32,
    battle_fairy_attribute: Option<SkillExecutionKernel<BattleFairySkillDispatch>>,
    battle_fairy_attribute_last_used_ms: [u32; 8],
    callosity: Option<CallosityExecutionState>,
    callosity_last_used_ms: [u32; 2],
    hearten: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    hearten_last_used_ms: u32,
    promotion: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    promotion_last_used_ms: u32,
    heal_family: [Option<SkillExecutionKernel<PlayerSkillDispatch>>; 4],
    heal_family_last_used_ms: [u32; 4],
    pets_control: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    pets_control_last_used_ms: u32,
    monster_taming: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    monster_taming_last_used_ms: u32,
    knock_out: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    knock_out_last_used_ms: u32,
    snow_storm: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    snow_storm_last_used_ms: u32,
    weak: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    weak_last_used_ms: u32,
    god_bless: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    god_bless_last_used_ms: [u32; 2],
    cure: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    cure_last_used_ms: u32,
    machine_shield: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    machine_shield_last_used_ms: u32,
    mana_shield: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    mana_shield_last_used_ms: u32,
    immediate_state: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    non_fun: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    swordship: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    gibe: Option<SkillExecutionKernel<PlayerSkillDispatch>>,
    gibe_last_used_ms: u32,
    auto_inc_last_time_ms: u32,
    auto_inc_energy_last_time_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerAutoProgress {
    pub(crate) player_id: i32,
    pub(crate) sampled_at_ms: u32,
    pub(crate) experience_gain: u32,
    pub(crate) vigour_gain: u32,
    pub(crate) previous_experience: u32,
    pub(crate) current_experience: u32,
    pub(crate) previous_vigour: u32,
    pub(crate) current_vigour: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerEnergyRegeneration {
    pub(crate) player_id: i32,
    pub(crate) sampled_at_ms: u32,
    pub(crate) increment: u32,
    pub(crate) previous_energy: u32,
    pub(crate) current_energy: u32,
}

impl CPlayerAI {
    pub(crate) fn destinations(&self) -> &VecDeque<PlayerAiDestination> {
        &self.destinations
    }

    pub(crate) fn queue_client_destination(&mut self, direction: i32, is_run: bool) {
        while 3 < self.destinations.len() {
            self.destinations.pop_front();
        }
        self.destinations
            .push_back(PlayerAiDestination { direction, is_run });
    }

    pub(crate) fn queue_player_skill(&mut self, dispatch: PlayerSkillDispatch) -> usize {
        if self.player_skills.front().copied() == Some(dispatch) {
            return 0;
        }
        let rejected = self.player_skills.len();
        self.player_skills.clear();
        self.base_attack = None;
        self.archery = None;
        self.agility_family = None;
        self.base_magic = None;
        self.fire_bolt = None;
        self.fire_ball = None;
        self.chain_lightning = None;
        self.thunder_blow = None;
        self.fire_wall = None;
        self.infernol = None;
        self.seven_shooting_star = None;
        self.chaos_sphere = None;
        self.lightning = None;
        self.seal = None;
        self.yin_yang = None;
        self.yin_yang_2 = None;
        self.god_punishment = None;
        self.god_thunder = None;
        self.god_thunder_2 = None;
        self.soul_collect = None;
        self.soul_mirror = None;
        self.callosity = None;
        self.hearten = None;
        self.promotion = None;
        self.heal_family = [None; 4];
        self.pets_control = None;
        self.monster_taming = None;
        self.knock_out = None;
        self.snow_storm = None;
        self.weak = None;
        self.god_bless = None;
        self.cure = None;
        self.machine_shield = None;
        self.mana_shield = None;
        self.immediate_state = None;
        self.non_fun = None;
        self.swordship = None;
        self.gibe = None;
        self.player_skills.push_back(dispatch);
        rejected
    }

    pub(crate) fn queue_battle_fairy_skill(&mut self, dispatch: BattleFairySkillDispatch) -> usize {
        if self.battle_fairy_skills.front().copied() == Some(dispatch) {
            return 0;
        }
        let replaced = self.battle_fairy_skills.len();
        self.battle_fairy_skills.clear();
        self.battle_fairy_base_magic = None;
        self.life_shield = None;
        self.battle_fairy_transfer = None;
        self.wangsheng = None;
        self.poison_arrow = None;
        self.blood_loss = None;
        self.fatal_blow = None;
        self.thunder = None;
        self.leiming2 = None;
        self.tianhuo = None;
        self.battle_fairy_attribute = None;
        self.battle_fairy_skills.push_back(dispatch);
        replaced
    }

    pub(crate) fn player_skills(&self) -> &VecDeque<PlayerSkillDispatch> {
        &self.player_skills
    }

    pub(crate) fn next_player_skill(&self) -> Option<PlayerSkillDispatch> {
        self.player_skills.front().copied()
    }

    pub(crate) fn finish_player_skill(
        &mut self,
        expected: PlayerSkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        if self.player_skills.front().copied() != Some(expected) {
            return false;
        }
        self.player_skills.pop_front();
        if let Some(mut execution) = self.base_attack.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение навыка игрока завершено");
        }
        if let Some(mut execution) = self.archery.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(
                ?expected,
                ?termination,
                stage = ?execution.kernel().stage(),
                "выполнение базовой стрельбы завершено"
            );
        }
        if let Some(mut execution) = self.agility_family.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение навыка ловкости завершено");
        }
        if let Some(mut execution) = self.base_magic.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение базовой магии завершено");
        }
        if let Some(mut execution) = self.fire_bolt.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение огненной стрелы завершено");
        }
        if let Some(mut execution) = self.fire_ball.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение огненного шара завершено");
        }
        if let Some(mut execution) = self.chain_lightning.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение цепной молнии завершено");
        }
        if let Some(mut execution) = self.thunder_blow.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение громового удара завершено");
        }
        if let Some(mut execution) = self.thunder_blow_2.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго громового удара завершено");
        }
        if let Some(mut execution) = self.mosou.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение Мо-шоу завершено");
        }
        if let Some(mut execution) = self.fire_wall.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение огненной стены завершено");
        }
        if let Some(mut execution) = self.infernol.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение огненного круга завершено");
        }
        if let Some(mut execution) = self.seven_shooting_star.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение семи падающих звёзд завершено");
        }
        if let Some(mut execution) = self.chaos_sphere.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение сферы хаоса завершено");
        }
        if let Some(mut execution) = self.lightning.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение молнии завершено");
        }
        if let Some(mut execution) = self.seal.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение печати завершено");
        }
        if let Some(mut execution) = self.yin_yang.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение инь-ян завершено");
        }
        if let Some(mut execution) = self.yin_yang_2.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго инь-ян завершено");
        }
        if let Some(mut execution) = self.god_punishment.take() { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение божественной кары завершено"); }
        if let Some(mut execution) = self.god_thunder.take() { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение божественного грома завершено"); }
        if let Some(mut execution) = self.god_thunder_2.take() { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение второго божественного грома завершено"); }
        if let Some(mut execution) = self.soul_collect.take() { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение сбора душ завершено"); }
        if let Some(mut execution) = self.soul_mirror.take() { let _ = execution.terminate(termination); tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение зеркала душ завершено"); }
        if let Some(mut execution) = self.callosity.take() {
            let _ = execution.kernel_mut().terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение навыка закалки завершено");
        }
        if let Some(mut execution) = self.hearten.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение воодушевления завершено");
        }
        if let Some(mut execution) = self.promotion.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение усиления завершено");
        }
        for mut execution in self
            .heal_family
            .each_mut()
            .into_iter()
            .filter_map(Option::take)
        {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение периодического лечения завершено");
        }
        if let Some(mut execution) = self.pets_control.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение управления питомцами завершено");
        }
        if let Some(mut execution) = self.monster_taming.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение приручения монстра завершено");
        }
        if let Some(mut execution) = self.knock_out.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение оглушения завершено");
        }
        if let Some(mut execution) = self.snow_storm.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение снежной бури завершено");
        }
        if let Some(mut execution) = self.weak.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ослабления завершено");
        }
        if let Some(mut execution) = self.god_bless.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение божественного благословения завершено");
        }
        if let Some(mut execution) = self.cure.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение очищения завершено");
        }
        if let Some(mut execution) = self.machine_shield.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение машинного щита завершено");
        }
        if let Some(mut execution) = self.mana_shield.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение мана-щита завершено");
        }
        if let Some(mut execution) = self.immediate_state.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение немедленного состояния завершено");
        }
        if let Some(mut execution) = self.non_fun.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение пустого навыка завершено");
        }
        if let Some(mut execution) = self.swordship.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение корабля мечей завершено");
        }
        if let Some(mut execution) = self.gibe.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение провокации завершено");
        }
        true
    }

    /// Материализует встречный `CPlayerAI::OnLoseTarget`, который вызывает
    /// `CPet::ReleaseReciprocalTarget`: удаляет только текущую object-команду,
    /// действительно направленную на отказавшегося питомца.
    pub(crate) fn release_object_target(
        &mut self,
        target: super::super::shape::ShapeIdentity,
    ) -> bool {
        let matches_target = matches!(
            self.player_skills.front(),
            Some(PlayerSkillDispatch::Object { target: current, .. }) if *current == target
        );
        if !matches_target {
            return false;
        }
        self.player_skills.pop_front();
        self.base_attack = None;
        self.archery = None;
        self.agility_family = None;
        self.base_magic = None;
        self.fire_bolt = None;
        self.fire_ball = None;
        self.chain_lightning = None;
        self.thunder_blow = None;
        self.fire_wall = None;
        self.infernol = None;
        self.seven_shooting_star = None;
        self.chaos_sphere = None;
        self.lightning = None;
        self.seal = None;
        self.yin_yang = None;
        self.yin_yang_2 = None;
        self.god_punishment = None;
        self.god_thunder = None;
        self.god_thunder_2 = None;
        self.soul_collect = None;
        self.soul_mirror = None;
        self.callosity = None;
        self.hearten = None;
        self.promotion = None;
        self.heal_family = [None; 4];
        self.pets_control = None;
        self.monster_taming = None;
        self.knock_out = None;
        self.snow_storm = None;
        self.weak = None;
        self.god_bless = None;
        self.cure = None;
        self.machine_shield = None;
        self.mana_shield = None;
        self.immediate_state = None;
        self.non_fun = None;
        self.swordship = None;
        self.gibe = None;
        self.poison_arrow = None;
        self.blood_loss = None;
        self.fatal_blow = None;
        self.thunder = None;
        self.leiming2 = None;
        self.tianhuo = None;
        self.battle_fairy_attribute = None;
        true
    }

    pub(crate) const fn base_attack(&self) -> Option<BaseAttackExecutionState> {
        self.base_attack
    }

    pub(crate) const fn archery(&self) -> Option<ArcheryExecutionState> {
        self.archery
    }

    pub(crate) const fn begin_archery(&mut self, state: ArcheryExecutionState) {
        self.archery = Some(state);
    }

    pub(crate) fn archery_mut(&mut self) -> Option<&mut ArcheryExecutionState> {
        self.archery.as_mut()
    }

    pub(crate) const fn archery_last_used_ms(&self) -> u32 {
        self.archery_last_used_ms
    }

    pub(crate) const fn agility_family(&self) -> Option<AgilityFamilyExecutionState> {
        self.agility_family
    }
    pub(crate) const fn begin_agility_family(&mut self, state: AgilityFamilyExecutionState) {
        self.agility_family = Some(state);
    }
    pub(crate) fn agility_family_mut(&mut self) -> Option<&mut AgilityFamilyExecutionState> {
        self.agility_family.as_mut()
    }
    const fn agility_family_index(skill_id: u32) -> usize {
        match skill_id {
            AGILITY_SKILL_ID => 0,
            AGILITY_2_SKILL_ID => 1,
            RAPTURE_SKILL_ID => 2,
            NATURAL_SKILL_ID => 3,
            _ => unreachable!(),
        }
    }
    pub(crate) const fn agility_family_last_used_ms(&self, skill_id: u32) -> u32 {
        self.agility_family_last_used_ms[Self::agility_family_index(skill_id)]
    }
    pub(crate) fn mark_agility_family_used(&mut self, skill_id: u32, now_ms: u32) {
        let index = Self::agility_family_index(skill_id);
        self.agility_family_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn mark_archery_used(&mut self, now_ms: u32) {
        self.archery_last_used_ms = now_ms;
    }

    pub(crate) const fn begin_base_attack(&mut self, state: BaseAttackExecutionState) {
        self.base_attack = Some(state);
    }

    pub(crate) fn advance_base_attack(
        &mut self,
        expected: SkillStage,
        next: SkillStage,
    ) -> bool {
        self.base_attack
            .as_mut()
            .is_some_and(|state| state.advance(expected, next))
    }

    pub(crate) const fn base_attack_last_used_ms(&self) -> u32 {
        self.base_attack_last_used_ms
    }

    pub(crate) const fn mark_base_attack_used(&mut self, now_ms: u32) {
        self.base_attack_last_used_ms = now_ms;
    }

    pub(crate) const fn base_magic(&self) -> Option<BaseMagicExecutionState> {
        self.base_magic
    }

    pub(crate) const fn begin_base_magic(&mut self, state: BaseMagicExecutionState) {
        self.base_magic = Some(state);
    }

    pub(crate) fn base_magic_mut(&mut self) -> Option<&mut BaseMagicExecutionState> {
        self.base_magic.as_mut()
    }

    pub(crate) const fn base_magic_last_used_ms(&self) -> u32 {
        self.base_magic_last_used_ms
    }

    pub(crate) const fn mark_base_magic_used(&mut self, now_ms: u32) {
        self.base_magic_last_used_ms = now_ms;
    }

    pub(crate) const fn fire_bolt(&self) -> Option<BaseMagicExecutionState> {
        self.fire_bolt
    }

    pub(crate) const fn begin_fire_bolt(&mut self, state: BaseMagicExecutionState) {
        self.fire_bolt = Some(state);
    }

    pub(crate) fn fire_bolt_mut(&mut self) -> Option<&mut BaseMagicExecutionState> {
        self.fire_bolt.as_mut()
    }

    pub(crate) const fn fire_bolt_last_used_ms(&self) -> u32 {
        self.fire_bolt_last_used_ms
    }

    pub(crate) const fn mark_fire_bolt_used(&mut self, now_ms: u32) {
        self.fire_bolt_last_used_ms = now_ms;
    }

    pub(crate) const fn fire_ball(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_ball
    }

    pub(crate) const fn begin_fire_ball(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) {
        self.fire_ball = Some(state);
    }

    pub(crate) fn fire_ball_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_ball.as_mut()
    }

    pub(crate) const fn fire_ball_last_used_ms(&self) -> u32 { self.fire_ball_last_used_ms }

    pub(crate) const fn mark_fire_ball_used(&mut self, now_ms: u32) {
        self.fire_ball_last_used_ms = now_ms;
    }

    pub(crate) const fn chain_lightning(&self) -> Option<ChainLightningExecutionState> {
        self.chain_lightning
    }

    pub(crate) const fn begin_chain_lightning(&mut self, state: ChainLightningExecutionState) {
        self.chain_lightning = Some(state);
    }

    pub(crate) fn chain_lightning_mut(&mut self) -> Option<&mut ChainLightningExecutionState> {
        self.chain_lightning.as_mut()
    }

    pub(crate) const fn chain_lightning_last_used_ms(&self) -> u32 { self.chain_lightning_last_used_ms }

    pub(crate) const fn mark_chain_lightning_used(&mut self, now_ms: u32) {
        self.chain_lightning_last_used_ms = now_ms;
    }

    pub(crate) const fn thunder_blow(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow }
    pub(crate) const fn begin_thunder_blow(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.thunder_blow = Some(state); }
    pub(crate) fn thunder_blow_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow.as_mut() }
    pub(crate) const fn thunder_blow_last_used_ms(&self) -> u32 { self.thunder_blow_last_used_ms }
    pub(crate) const fn mark_thunder_blow_used(&mut self, now_ms: u32) { self.thunder_blow_last_used_ms = now_ms; }

    pub(crate) const fn thunder_blow_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow_2 }
    pub(crate) const fn begin_thunder_blow_2(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.thunder_blow_2 = Some(state); }
    pub(crate) fn thunder_blow_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.thunder_blow_2.as_mut() }
    pub(crate) const fn thunder_blow_2_last_used_ms(&self) -> u32 { self.thunder_blow_2_last_used_ms }
    pub(crate) const fn mark_thunder_blow_2_used(&mut self, now_ms: u32) { self.thunder_blow_2_last_used_ms = now_ms; }

    pub(crate) const fn mosou(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.mosou }
    pub(crate) const fn begin_mosou(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.mosou = Some(state); }
    pub(crate) fn mosou_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.mosou.as_mut() }
    pub(crate) const fn mosou_last_used_ms(&self) -> u32 { self.mosou_last_used_ms }
    pub(crate) const fn mark_mosou_used(&mut self, now_ms: u32) { self.mosou_last_used_ms = now_ms; }

    pub(crate) const fn fire_wall(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_wall
    }

    pub(crate) const fn begin_fire_wall(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.fire_wall = Some(state);
    }

    pub(crate) fn fire_wall_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.fire_wall.as_mut()
    }

    pub(crate) const fn fire_wall_last_used_ms(&self) -> u32 {
        self.fire_wall_last_used_ms
    }

    pub(crate) const fn mark_fire_wall_used(&mut self, now_ms: u32) {
        self.fire_wall_last_used_ms = now_ms;
    }

    pub(crate) const fn infernol(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.infernol
    }

    pub(crate) const fn begin_infernol(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.infernol = Some(state);
    }

    pub(crate) fn infernol_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.infernol.as_mut()
    }

    pub(crate) const fn infernol_last_used_ms(&self) -> u32 {
        self.infernol_last_used_ms
    }

    pub(crate) const fn mark_infernol_used(&mut self, now_ms: u32) {
        self.infernol_last_used_ms = now_ms;
    }

    pub(crate) const fn seven_shooting_star(&self) -> Option<&SevenShootingStarExecutionState> {
        self.seven_shooting_star.as_ref()
    }

    pub(crate) fn begin_seven_shooting_star(
        &mut self,
        state: SevenShootingStarExecutionState,
    ) {
        self.seven_shooting_star = Some(state);
    }

    pub(crate) fn seven_shooting_star_mut(
        &mut self,
    ) -> Option<&mut SevenShootingStarExecutionState> {
        self.seven_shooting_star.as_mut()
    }

    pub(crate) const fn seven_shooting_star_last_used_ms(&self) -> u32 {
        self.seven_shooting_star_last_used_ms
    }

    pub(crate) const fn mark_seven_shooting_star_used(&mut self, now_ms: u32) {
        self.seven_shooting_star_last_used_ms = now_ms;
    }

    pub(crate) const fn chaos_sphere(&self) -> Option<&ChaosSphereExecutionState> {
        self.chaos_sphere.as_ref()
    }

    pub(crate) const fn begin_chaos_sphere(&mut self, state: ChaosSphereExecutionState) {
        self.chaos_sphere = Some(state);
    }

    pub(crate) fn chaos_sphere_mut(&mut self) -> Option<&mut ChaosSphereExecutionState> {
        self.chaos_sphere.as_mut()
    }

    pub(crate) const fn chaos_sphere_last_used_ms(&self) -> u32 {
        self.chaos_sphere_last_used_ms
    }

    pub(crate) const fn mark_chaos_sphere_used(&mut self, now_ms: u32) {
        self.chaos_sphere_last_used_ms = now_ms;
    }

    pub(crate) const fn lightning(&self) -> Option<LightningExecutionState> {
        self.lightning
    }

    pub(crate) const fn begin_lightning(&mut self, state: LightningExecutionState) {
        self.lightning = Some(state);
    }

    pub(crate) fn lightning_mut(&mut self) -> Option<&mut LightningExecutionState> {
        self.lightning.as_mut()
    }

    pub(crate) const fn lightning_last_used_ms(&self) -> u32 {
        self.lightning_last_used_ms
    }

    pub(crate) const fn mark_lightning_used(&mut self, now_ms: u32) {
        self.lightning_last_used_ms = now_ms;
    }

    pub(crate) const fn seal(&self) -> Option<SealExecutionState> {
        self.seal
    }

    pub(crate) const fn begin_seal(&mut self, state: SealExecutionState) {
        self.seal = Some(state);
    }

    pub(crate) fn seal_mut(&mut self) -> Option<&mut SealExecutionState> {
        self.seal.as_mut()
    }

    pub(crate) const fn seal_last_used_ms(&self) -> u32 {
        self.seal_last_used_ms
    }

    pub(crate) const fn mark_seal_used(&mut self, now_ms: u32) {
        self.seal_last_used_ms = now_ms;
    }

    pub(crate) const fn yin_yang(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.yin_yang
    }

    pub(crate) const fn begin_yin_yang(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) {
        self.yin_yang = Some(state);
    }

    pub(crate) fn yin_yang_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.yin_yang.as_mut()
    }

    pub(crate) const fn yin_yang_last_used_ms(&self) -> u32 { self.yin_yang_last_used_ms }

    pub(crate) const fn mark_yin_yang_used(&mut self, now_ms: u32) {
        self.yin_yang_last_used_ms = now_ms;
    }
    pub(crate) const fn yin_yang_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.yin_yang_2 }
    pub(crate) const fn begin_yin_yang_2(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.yin_yang_2 = Some(state); }
    pub(crate) fn yin_yang_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.yin_yang_2.as_mut() }
    pub(crate) const fn yin_yang_2_last_used_ms(&self) -> u32 { self.yin_yang_2_last_used_ms }
    pub(crate) const fn mark_yin_yang_2_used(&mut self, now_ms: u32) { self.yin_yang_2_last_used_ms = now_ms; }
    pub(crate) const fn god_punishment(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_punishment }
    pub(crate) const fn begin_god_punishment(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.god_punishment = Some(state); }
    pub(crate) fn god_punishment_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_punishment.as_mut() }
    pub(crate) const fn god_punishment_last_used_ms(&self) -> u32 { self.god_punishment_last_used_ms }
    pub(crate) const fn mark_god_punishment_used(&mut self, now: u32) { self.god_punishment_last_used_ms = now; }
    pub(crate) const fn god_thunder(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder }
    pub(crate) const fn begin_god_thunder(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.god_thunder = Some(state); }
    pub(crate) fn god_thunder_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder.as_mut() }
    pub(crate) const fn god_thunder_last_used_ms(&self) -> u32 { self.god_thunder_last_used_ms }
    pub(crate) const fn mark_god_thunder_used(&mut self, now: u32) { self.god_thunder_last_used_ms = now; }
    pub(crate) const fn god_thunder_2(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder_2 }
    pub(crate) const fn begin_god_thunder_2(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.god_thunder_2 = Some(state); }
    pub(crate) fn god_thunder_2_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_thunder_2.as_mut() }
    pub(crate) const fn god_thunder_2_last_used_ms(&self) -> u32 { self.god_thunder_2_last_used_ms }
    pub(crate) const fn mark_god_thunder_2_used(&mut self, now: u32) { self.god_thunder_2_last_used_ms = now; }
    pub(crate) const fn soul_collect(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_collect }
    pub(crate) const fn begin_soul_collect(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.soul_collect = Some(state); }
    pub(crate) fn soul_collect_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_collect.as_mut() }
    pub(crate) const fn soul_collect_last_used_ms(&self) -> u32 { self.soul_collect_last_used_ms }
    pub(crate) const fn mark_soul_collect_used(&mut self, now: u32) { self.soul_collect_last_used_ms = now; }
    pub(crate) const fn soul_mirror(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_mirror }
    pub(crate) const fn begin_soul_mirror(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.soul_mirror = Some(state); }
    pub(crate) fn soul_mirror_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.soul_mirror.as_mut() }
    pub(crate) const fn soul_mirror_last_used_ms(&self) -> u32 { self.soul_mirror_last_used_ms }
    pub(crate) const fn mark_soul_mirror_used(&mut self, now: u32) { self.soul_mirror_last_used_ms = now; }

    pub(crate) const fn callosity(&self) -> Option<CallosityExecutionState> {
        self.callosity
    }

    pub(crate) const fn begin_callosity(&mut self, state: CallosityExecutionState) {
        self.callosity = Some(state);
    }

    pub(crate) fn callosity_mut(&mut self) -> Option<&mut CallosityExecutionState> {
        self.callosity.as_mut()
    }

    pub(crate) const fn callosity_last_used_ms(&self, skill_id: u32) -> u32 {
        if skill_id == crate::gameserver::appserver::skills::callosity::CALLOSITY_2_SKILL_ID {
            self.callosity_last_used_ms[1]
        } else {
            self.callosity_last_used_ms[0]
        }
    }

    pub(crate) fn mark_callosity_used(&mut self, skill_id: u32, now_ms: u32) {
        let index = if skill_id
            == crate::gameserver::appserver::skills::callosity::CALLOSITY_2_SKILL_ID
        {
            1
        } else {
            0
        };
        self.callosity_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn hearten(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.hearten
    }
    pub(crate) const fn begin_hearten(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.hearten = Some(state);
    }
    pub(crate) fn hearten_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.hearten.as_mut()
    }
    pub(crate) const fn hearten_last_used_ms(&self) -> u32 { self.hearten_last_used_ms }
    pub(crate) const fn mark_hearten_used(&mut self, now_ms: u32) {
        self.hearten_last_used_ms = now_ms;
    }

    pub(crate) const fn promotion(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.promotion
    }

    pub(crate) const fn begin_promotion(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.promotion = Some(state);
    }

    pub(crate) fn promotion_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.promotion.as_mut()
    }

    pub(crate) const fn promotion_last_used_ms(&self) -> u32 {
        self.promotion_last_used_ms
    }

    pub(crate) const fn mark_promotion_used(&mut self, now_ms: u32) {
        self.promotion_last_used_ms = now_ms;
    }

    pub(crate) const fn heal_family(
        &self,
        index: usize,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.heal_family[index]
    }

    pub(crate) fn begin_heal_family(
        &mut self,
        index: usize,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.heal_family[index] = Some(state);
    }

    pub(crate) fn heal_family_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.heal_family[index].as_mut()
    }

    pub(crate) const fn heal_family_last_used_ms(&self, index: usize) -> u32 {
        self.heal_family_last_used_ms[index]
    }

    pub(crate) const fn mark_heal_family_used(&mut self, index: usize, now_ms: u32) {
        self.heal_family_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn pets_control(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.pets_control
    }

    pub(crate) const fn begin_pets_control(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.pets_control = Some(state);
    }

    pub(crate) fn pets_control_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.pets_control.as_mut()
    }

    pub(crate) const fn pets_control_last_used_ms(&self) -> u32 {
        self.pets_control_last_used_ms
    }

    pub(crate) const fn mark_pets_control_used(&mut self, now_ms: u32) {
        self.pets_control_last_used_ms = now_ms;
    }

    pub(crate) const fn monster_taming(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_taming
    }

    pub(crate) const fn begin_monster_taming(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.monster_taming = Some(state);
    }

    pub(crate) fn monster_taming_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.monster_taming.as_mut()
    }

    pub(crate) const fn monster_taming_last_used_ms(&self) -> u32 {
        self.monster_taming_last_used_ms
    }

    pub(crate) const fn mark_monster_taming_used(&mut self, now_ms: u32) {
        self.monster_taming_last_used_ms = now_ms;
    }

    pub(crate) const fn knock_out(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.knock_out
    }

    pub(crate) const fn begin_knock_out(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) {
        self.knock_out = Some(state);
    }

    pub(crate) fn knock_out_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.knock_out.as_mut()
    }

    pub(crate) const fn knock_out_last_used_ms(&self) -> u32 { self.knock_out_last_used_ms }

    pub(crate) const fn mark_knock_out_used(&mut self, now_ms: u32) {
        self.knock_out_last_used_ms = now_ms;
    }

    pub(crate) const fn snow_storm(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.snow_storm
    }

    pub(crate) const fn begin_snow_storm(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) {
        self.snow_storm = Some(state);
    }

    pub(crate) fn snow_storm_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.snow_storm.as_mut()
    }

    pub(crate) const fn snow_storm_last_used_ms(&self) -> u32 { self.snow_storm_last_used_ms }

    pub(crate) const fn mark_snow_storm_used(&mut self, now_ms: u32) {
        self.snow_storm_last_used_ms = now_ms;
    }

    pub(crate) const fn weak(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.weak
    }

    pub(crate) const fn begin_weak(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) {
        self.weak = Some(state);
    }

    pub(crate) fn weak_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.weak.as_mut()
    }

    pub(crate) const fn weak_last_used_ms(&self) -> u32 { self.weak_last_used_ms }

    pub(crate) const fn mark_weak_used(&mut self, now_ms: u32) {
        self.weak_last_used_ms = now_ms;
    }

    pub(crate) const fn god_bless(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.god_bless }
    pub(crate) const fn begin_god_bless(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.god_bless = Some(state); }
    pub(crate) fn god_bless_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.god_bless.as_mut() }
    pub(crate) const fn god_bless_last_used_ms(&self, index: usize) -> u32 { self.god_bless_last_used_ms[index] }
    pub(crate) const fn mark_god_bless_used(&mut self, index: usize, now_ms: u32) { self.god_bless_last_used_ms[index] = now_ms; }
    pub(crate) const fn cure(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> { self.cure }
    pub(crate) const fn begin_cure(&mut self, state: SkillExecutionKernel<PlayerSkillDispatch>) { self.cure = Some(state); }
    pub(crate) fn cure_mut(&mut self) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> { self.cure.as_mut() }
    pub(crate) const fn cure_last_used_ms(&self) -> u32 { self.cure_last_used_ms }
    pub(crate) const fn mark_cure_used(&mut self, now_ms: u32) { self.cure_last_used_ms = now_ms; }

    pub(crate) const fn machine_shield(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.machine_shield
    }

    pub(crate) const fn begin_machine_shield(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.machine_shield = Some(state);
    }

    pub(crate) fn machine_shield_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.machine_shield.as_mut()
    }

    pub(crate) const fn machine_shield_last_used_ms(&self) -> u32 {
        self.machine_shield_last_used_ms
    }

    pub(crate) const fn mark_machine_shield_used(&mut self, now_ms: u32) {
        self.machine_shield_last_used_ms = now_ms;
    }

    pub(crate) const fn mana_shield(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.mana_shield
    }

    pub(crate) const fn begin_mana_shield(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.mana_shield = Some(state);
    }
    pub(crate) fn mana_shield_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.mana_shield.as_mut()
    }

    pub(crate) const fn mana_shield_last_used_ms(&self) -> u32 {
        self.mana_shield_last_used_ms
    }

    pub(crate) const fn mark_mana_shield_used(&mut self, now_ms: u32) {
        self.mana_shield_last_used_ms = now_ms;
    }

    pub(crate) const fn immediate_state(
        &self,
    ) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.immediate_state
    }
    pub(crate) const fn begin_immediate_state(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.immediate_state = Some(state);
    }
    pub(crate) fn immediate_state_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.immediate_state.as_mut()
    }

    pub(crate) const fn non_fun(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.non_fun
    }

    pub(crate) const fn begin_non_fun(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.non_fun = Some(state);
    }

    pub(crate) fn non_fun_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.non_fun.as_mut()
    }

    pub(crate) const fn swordship(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.swordship
    }

    pub(crate) const fn begin_swordship(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.swordship = Some(state);
    }

    pub(crate) fn swordship_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.swordship.as_mut()
    }

    pub(crate) const fn gibe(&self) -> Option<SkillExecutionKernel<PlayerSkillDispatch>> {
        self.gibe
    }

    pub(crate) const fn begin_gibe(
        &mut self,
        state: SkillExecutionKernel<PlayerSkillDispatch>,
    ) {
        self.gibe = Some(state);
    }

    pub(crate) fn gibe_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<PlayerSkillDispatch>> {
        self.gibe.as_mut()
    }

    pub(crate) const fn gibe_last_used_ms(&self) -> u32 {
        self.gibe_last_used_ms
    }

    pub(crate) const fn mark_gibe_used(&mut self, now_ms: u32) {
        self.gibe_last_used_ms = now_ms;
    }

    pub(crate) fn battle_fairy_skills(&self) -> &VecDeque<BattleFairySkillDispatch> {
        &self.battle_fairy_skills
    }

    pub(crate) fn next_battle_fairy_skill(&self) -> Option<BattleFairySkillDispatch> {
        self.battle_fairy_skills.front().copied()
    }

    pub(crate) fn finish_battle_fairy_skill(
        &mut self,
        expected: BattleFairySkillDispatch,
        termination: SkillTermination,
    ) -> bool {
        if self.battle_fairy_skills.front().copied() != Some(expected) {
            return false;
        }
        self.battle_fairy_skills.pop_front();
        if let Some(mut execution) = self.battle_fairy_base_magic.take() {
            let _ = execution
                .kernel_mut()
                .terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.kernel().stage(), "выполнение базовой атаки боевой феи завершено");
        }
        if let Some(mut execution) = self.life_shield.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение щита жизни завершено");
        }
        if let Some(mut execution) = self.battle_fairy_transfer.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение переноса здоровья боевому духу завершено");
        }
        if let Some(mut execution) = self.wangsheng.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение восстановления здоровья игрока завершено");
        }
        if let Some(mut execution) = self.poison_arrow.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение ядовитой стрелы завершено");
        }
        if let Some(mut execution) = self.blood_loss.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение потери крови завершено");
        }
        if let Some(mut execution) = self.fatal_blow.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение смертельного удара завершено");
        }
        if let Some(mut execution) = self.thunder.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение грома завершено");
        }
        if let Some(mut execution) = self.leiming2.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение отложенного грома завершено");
        }
        if let Some(mut execution) = self.tianhuo.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение небесного огня завершено");
        }
        if let Some(mut execution) = self.battle_fairy_attribute.take() {
            let _ = execution.terminate(termination);
            tracing::trace!(?expected, ?termination, stage = ?execution.stage(), "выполнение атрибутного навыка боевого духа завершено");
        }
        true
    }

    pub(crate) const fn battle_fairy_base_magic(
        &self,
    ) -> Option<BattleFairyBaseMagicExecutionState> {
        self.battle_fairy_base_magic
    }

    pub(crate) const fn battle_fairy_attribute(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_attribute
    }

    pub(crate) const fn begin_battle_fairy_attribute(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.battle_fairy_attribute = Some(state);
    }

    pub(crate) fn battle_fairy_attribute_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_attribute.as_mut()
    }

    const fn battle_fairy_attribute_index(skill_id: u32) -> usize {
        match skill_id {
            0x212..=0x219 => (skill_id - 0x212) as usize,
            _ => unreachable!(),
        }
    }

    pub(crate) const fn battle_fairy_attribute_last_used_ms(&self, skill_id: u32) -> u32 {
        self.battle_fairy_attribute_last_used_ms[Self::battle_fairy_attribute_index(skill_id)]
    }

    pub(crate) fn mark_battle_fairy_attribute_used(&mut self, skill_id: u32, now_ms: u32) {
        let index = Self::battle_fairy_attribute_index(skill_id);
        self.battle_fairy_attribute_last_used_ms[index] = now_ms;
    }

    pub(crate) const fn begin_battle_fairy_base_magic(
        &mut self,
        state: BattleFairyBaseMagicExecutionState,
    ) {
        self.battle_fairy_base_magic = Some(state);
    }

    pub(crate) fn battle_fairy_base_magic_mut(
        &mut self,
    ) -> Option<&mut BattleFairyBaseMagicExecutionState> {
        self.battle_fairy_base_magic.as_mut()
    }

    pub(crate) const fn battle_fairy_base_magic_last_used_ms(&self) -> u32 {
        self.battle_fairy_base_magic_last_used_ms
    }

    pub(crate) const fn mark_battle_fairy_base_magic_used(&mut self, now_ms: u32) {
        self.battle_fairy_base_magic_last_used_ms = now_ms;
    }

    pub(crate) const fn life_shield(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.life_shield
    }

    pub(crate) const fn begin_life_shield(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.life_shield = Some(state);
    }

    pub(crate) fn life_shield_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.life_shield.as_mut()
    }

    pub(crate) const fn life_shield_last_used_ms(&self) -> u32 {
        self.life_shield_last_used_ms
    }

    pub(crate) const fn mark_life_shield_used(&mut self, now_ms: u32) {
        self.life_shield_last_used_ms = now_ms;
    }

    pub(crate) const fn battle_fairy_transfer(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_transfer
    }

    pub(crate) const fn begin_battle_fairy_transfer(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.battle_fairy_transfer = Some(state);
    }

    pub(crate) fn battle_fairy_transfer_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.battle_fairy_transfer.as_mut()
    }

    const fn battle_fairy_transfer_index(kind: BattleFairyTransferKind) -> usize {
        match kind {
            BattleFairyTransferKind::Health => 0,
            BattleFairyTransferKind::Mana => 1,
        }
    }

    pub(crate) const fn battle_fairy_transfer_last_used_ms(
        &self,
        kind: BattleFairyTransferKind,
    ) -> u32 {
        self.battle_fairy_transfer_last_used_ms[Self::battle_fairy_transfer_index(kind)]
    }

    pub(crate) fn mark_battle_fairy_transfer_used(
        &mut self,
        kind: BattleFairyTransferKind,
        now_ms: u32,
    ) {
        self.battle_fairy_transfer_last_used_ms[Self::battle_fairy_transfer_index(kind)] = now_ms;
    }

    pub(crate) const fn wangsheng(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.wangsheng
    }

    pub(crate) const fn begin_wangsheng(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.wangsheng = Some(state);
    }

    pub(crate) fn wangsheng_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.wangsheng.as_mut()
    }

    pub(crate) const fn wangsheng_last_used_ms(&self) -> u32 {
        self.wangsheng_last_used_ms
    }

    pub(crate) const fn mark_wangsheng_used(&mut self, now_ms: u32) {
        self.wangsheng_last_used_ms = now_ms;
    }

    pub(crate) const fn poison_arrow(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.poison_arrow
    }

    pub(crate) const fn begin_poison_arrow(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.poison_arrow = Some(state);
    }

    pub(crate) fn poison_arrow_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.poison_arrow.as_mut()
    }

    pub(crate) const fn poison_arrow_last_used_ms(&self) -> u32 {
        self.poison_arrow_last_used_ms
    }

    pub(crate) const fn mark_poison_arrow_used(&mut self, now_ms: u32) {
        self.poison_arrow_last_used_ms = now_ms;
    }

    pub(crate) const fn blood_loss(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.blood_loss
    }

    pub(crate) const fn begin_blood_loss(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.blood_loss = Some(state);
    }

    pub(crate) fn blood_loss_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.blood_loss.as_mut()
    }

    pub(crate) const fn blood_loss_last_used_ms(&self) -> u32 {
        self.blood_loss_last_used_ms
    }

    pub(crate) const fn mark_blood_loss_used(&mut self, now_ms: u32) {
        self.blood_loss_last_used_ms = now_ms;
    }

    pub(crate) const fn fatal_blow(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.fatal_blow
    }

    pub(crate) const fn begin_fatal_blow(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.fatal_blow = Some(state);
    }

    pub(crate) fn fatal_blow_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.fatal_blow.as_mut()
    }

    pub(crate) const fn fatal_blow_last_used_ms(&self) -> u32 {
        self.fatal_blow_last_used_ms
    }

    pub(crate) const fn mark_fatal_blow_used(&mut self, now_ms: u32) {
        self.fatal_blow_last_used_ms = now_ms;
    }

    pub(crate) const fn thunder(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.thunder
    }

    pub(crate) const fn begin_thunder(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.thunder = Some(state);
    }

    pub(crate) fn thunder_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.thunder.as_mut()
    }

    pub(crate) const fn thunder_last_used_ms(&self) -> u32 {
        self.thunder_last_used_ms
    }

    pub(crate) const fn mark_thunder_used(&mut self, now_ms: u32) {
        self.thunder_last_used_ms = now_ms;
    }

    pub(crate) const fn leiming2(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.leiming2
    }

    pub(crate) const fn begin_leiming2(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.leiming2 = Some(state);
    }

    pub(crate) fn leiming2_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.leiming2.as_mut()
    }

    pub(crate) const fn leiming2_last_used_ms(&self) -> u32 {
        self.leiming2_last_used_ms
    }

    pub(crate) const fn mark_leiming2_used(&mut self, now_ms: u32) {
        self.leiming2_last_used_ms = now_ms;
    }

    pub(crate) const fn tianhuo(
        &self,
    ) -> Option<SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.tianhuo
    }

    pub(crate) const fn begin_tianhuo(
        &mut self,
        state: SkillExecutionKernel<BattleFairySkillDispatch>,
    ) {
        self.tianhuo = Some(state);
    }

    pub(crate) fn tianhuo_mut(
        &mut self,
    ) -> Option<&mut SkillExecutionKernel<BattleFairySkillDispatch>> {
        self.tianhuo.as_mut()
    }

    pub(crate) const fn tianhuo_last_used_ms(&self) -> u32 {
        self.tianhuo_last_used_ms
    }

    pub(crate) const fn mark_tianhuo_used(&mut self, now_ms: u32) {
        self.tianhuo_last_used_ms = now_ms;
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn increment_player_progress(
        &mut self,
        player: &mut CPlayer,
        interval_ms: u32,
        auto_exp_1: f32,
        auto_exp_2: f32,
        exp_to_vigour_x: u32,
        exp_to_vigour_y: u32,
        maximum_vigour_once: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerAutoProgress> {
        if player.is_dead() || player.faction_id() == 0 {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_last_time_ms = get_tick_ms();

        let level = f64::from(player.level());
        let experience_gain = (((f64::from(player.faction_level()) * 0.05 + 1.0)
            * level.powi(3)
            * f64::from(auto_exp_2)
            + f64::from(auto_exp_1))
            * f64::from(0.000_115_740_74_f32))
        .trunc() as u32;
        if experience_gain == 0 {
            return None;
        }
        let vigour_raw = f64::from(exp_to_vigour_x)
            * f64::from(experience_gain.wrapping_add(600)).log10()
            - f64::from(exp_to_vigour_y);
        let vigour_gain = (vigour_raw.trunc() as i32 as u32).min(maximum_vigour_once);
        let previous_experience = player.experience();
        let previous_vigour = player.vigour();
        player.set_experience(previous_experience.wrapping_add(experience_gain));
        player.set_vigour_clamped(previous_vigour.wrapping_add(vigour_gain));
        Some(PlayerAutoProgress {
            player_id: player.player_id(),
            sampled_at_ms,
            experience_gain,
            vigour_gain,
            previous_experience,
            current_experience: player.experience(),
            previous_vigour,
            current_vigour: player.vigour(),
        })
    }

    /// Exact energy tail `CPlayerAI::Run`: первый живой tick только заводит
    /// clock; full energy не двигает его дальше. Due comparison намеренно не
    /// wrap-safe (`last < now - interval`) — это наблюдаемая native-семантика.
    pub(crate) fn regenerate_player_energy(
        &mut self,
        player: &mut CPlayer,
        interval_ms: u32,
        get_tick_ms: &mut dyn FnMut() -> u32,
    ) -> Option<PlayerEnergyRegeneration> {
        if player.is_dead() {
            return None;
        }
        if self.auto_inc_energy_last_time_ms == 0 {
            self.auto_inc_energy_last_time_ms = get_tick_ms();
        }
        let previous_energy = player.energy();
        if previous_energy == player.maximum_energy() {
            return None;
        }
        let sampled_at_ms = get_tick_ms();
        if self.auto_inc_energy_last_time_ms >= sampled_at_ms.wrapping_sub(interval_ms) {
            return None;
        }
        self.auto_inc_energy_last_time_ms = get_tick_ms();

        let faction_bonus = if player.faction_id() == 0 {
            0.0
        } else {
            (f64::from(player.level()) * f64::from(0.01_f32))
                .min(1.0)
                .mul_add(f64::from(player.faction_level()) * 0.5, 0.0)
                .max(1.0)
        };
        // MSVC меняет x87 rounding mode на truncation перед `__ftol2`.
        let increment =
            ((f64::from(player.level()) * f64::from(0.1_f32) - 1.0) * 5.0 + faction_bonus + 10.0)
                .trunc() as u32;
        if increment == 0 {
            return None;
        }
        player.set_energy(previous_energy.wrapping_add(increment));
        let current_energy = player.energy();
        (current_energy != previous_energy).then_some(PlayerEnergyRegeneration {
            player_id: player.player_id(),
            sampled_at_ms,
            increment,
            previous_energy,
            current_energy,
        })
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp

// ============================================================================
// FUNCTION: CPlayerAI::Tracing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:414
// RVA: 0x00108DC0
// ADDRESS: 00508dc0
// PROTOTYPE: int __thiscall Tracing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnChangeSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:608
// RVA: 0x00108E40
// ADDRESS: 00508e40
// PROTOTYPE: int __thiscall OnChangeSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnMoving
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:658
// RVA: 0x00108E90
// ADDRESS: 00508e90
// PROTOTYPE: int __thiscall OnMoving(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnStanding
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:685
// RVA: 0x00108ED0
// ADDRESS: 00508ed0
// PROTOTYPE: int __thiscall OnStanding(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::MoveTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:952
// RVA: 0x00108F10
// ADDRESS: 00508f10
// PROTOTYPE: void __thiscall MoveTo(CRegion * param_1, long param_2, long param_3, int param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnLoseTarget
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:450
// RVA: 0x00109130
// ADDRESS: 00509130
// PROTOTYPE: int __thiscall OnLoseTarget(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnLoseTargetWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:484
// RVA: 0x001091B0
// ADDRESS: 005091b0
// PROTOTYPE: int __thiscall OnLoseTargetWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnFightingWithWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:517
// RVA: 0x00109230
// ADDRESS: 00509230
// PROTOTYPE: int __thiscall OnFightingWithWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnFighting
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:563
// RVA: 0x001092B0
// ADDRESS: 005092b0
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnChangeSkillWithWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:629
// RVA: 0x00109310
// ADDRESS: 00509310
// PROTOTYPE: int __thiscall OnChangeSkillWithWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::IsCanMoveTo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:1057
// RVA: 0x00109360
// ADDRESS: 00509360
// PROTOTYPE: int __thiscall IsCanMoveTo(long param_1, long * param_2, long * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Run
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:27
// RVA: 0x001093E0
// ADDRESS: 005093e0
// PROTOTYPE: AI_EXEC_STATE __thiscall Run(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnScheduleAboutWarSoul
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:143
// RVA: 0x00109730
// ADDRESS: 00509730
// PROTOTYPE: void __thiscall OnScheduleAboutWarSoul(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::OnSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:256
// RVA: 0x001098D0
// ADDRESS: 005098d0
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::CPlayerAI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:19
// RVA: 0x00109B70
// ADDRESS: 00509b70
// PROTOTYPE: undefined __thiscall CPlayerAI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:711
// RVA: 0x00109FF0
// ADDRESS: 00509ff0
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerAI::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\playerai.cpp:841
// RVA: 0x0010A230
// ADDRESS: 0050a230
// PROTOTYPE: void __thiscall Attack(tagSkillID param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
