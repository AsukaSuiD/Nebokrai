//! Общий зарегистрированный вход активных навыков игрока.
//! Источник: gameserver.exe/GameServer.pdb, appserver/states/skill.cpp,
//! attackskill.cpp, stateskill.cpp и совместимые Begin. Игровые проверки
//! и AI остаются у конкретных навыков.
//!
//! База Begin, материализация, visual и End используют один поколенческий
//! ключ. Проверка видит данные исполнения с выключенной фазой; успешный
//! Begin включает её без второго отсчёта времени. Общий End выполняется
//! до возврата расписанию, которое только освобождает данные и ту же команду.
//! Удалённый callback-ом экземпляр не заменяется новым совпадением ID.
//! Released из concrete End оставляет исполнение активным; даже терминальный
//! результат AI не разрешает расписанию удалить выпущенный callback-ом cast.
//! Немедленные состояния и NonFun используют тот же вход без создания visual;
//! прежний visual при таком Begin не заменяется.

use super::kernel::{PlayerSkillExecution, SkillStage, SkillTermination};
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::{RegisteredSkill, RegisteredSkillEnd};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

/// Один каталог задаёт собственный Begin, исполнение и общий End.
/// Расписание не поддерживает отдельный список этих же владельцев.
#[derive(Clone, Copy)]
pub(crate) enum RegisteredPlayerCastOwner {
    Flash, LittleFlash, Rush, Rush2, ArmyBreak, GhostCut, Mosou, ThunderBlow2,
    Swallow, KnightCut, LeafCut, FrontCellSword, EnergyHolding, Pillar, Roar, ThunderSlash, Callosity,
    SelfState, LightingArrow, LightingArrow2, MeteorArrowMass, MeteorArrow, RainArrow, FallingStar,
    PoisonMoth, ScopedArrow, Scorpion, BoaLock, TargetedProjectile, Combustion, HeartlessArrow, HeartlessArrowArea, BaseProjectile, GodPunishment, ImmediateState, Heal, NonFun, GodBless, ZonalCast, Lightning, ChainLightning,
}

impl RegisteredPlayerCastOwner {
    pub(crate) fn from_skill_id(id: u32) -> Option<Self> {
        Some(match id {
            super::archery::ARCHERY_SKILL_ID | super::basemagic::BASE_MAGIC_SKILL_ID
                | super::firebolt::FIRE_BOLT_SKILL_ID | super::fireball::FIRE_BALL_SKILL_ID => Self::BaseProjectile,
            super::godpunishment::GOD_PUNISHMENT_SKILL_ID => Self::GodPunishment,
            super::flash::FLASH_SKILL_ID => Self::Flash,
            super::littleflash::LITTLE_FLASH_SKILL_ID | super::littleflash2::LITTLE_FLASH_2_SKILL_ID => Self::LittleFlash,
            super::rush::RUSH_SKILL_ID => Self::Rush,
            super::rush2::RUSH_2_SKILL_ID => Self::Rush2,
            super::armybreak::ARMY_BREAK_SKILL_ID | super::armybreak2::ARMY_BREAK_2_SKILL_ID => Self::ArmyBreak,
            super::ghostcut::GHOST_CUT_SKILL_ID | super::ghostcut2::GHOST_CUT_2_SKILL_ID
                | super::ghostcut3::GHOST_CUT_3_SKILL_ID => Self::GhostCut,
            super::mosou::MOSOU_SKILL_ID => Self::Mosou,
            super::thunderblow2::THUNDER_BLOW_2_SKILL_ID => Self::ThunderBlow2,
            super::swallow::SWALLOW_SKILL_ID => Self::Swallow,
            super::knightcut::KNIGHT_CUT_SKILL_ID => Self::KnightCut,
            super::leafcut::LEAF_CUT_SKILL_ID | super::leafcut2::LEAF_CUT_2_SKILL_ID
                | super::leafcut3::LEAF_CUT_3_SKILL_ID => Self::LeafCut,
            super::jucut::JU_CUT_SKILL_ID | super::lightningsword::LIGHTNING_SWORD_SKILL_ID
                | super::lightningsword2::LIGHTNING_SWORD_2_SKILL_ID
                | super::lightningsword3::LIGHTNING_SWORD_3_SKILL_ID
                | super::lightningsword4::LIGHTNING_SWORD_4_SKILL_ID
                | super::inversechopped::INVERSE_CHOPPED_SKILL_ID => Self::FrontCellSword,
            super::energyholding::ENERGY_HOLDING_SKILL_ID => Self::EnergyHolding,
            super::pillar::PILLAR_SKILL_ID => Self::Pillar,
            super::roar::ROAR_SKILL_ID => Self::Roar,
            super::thunderslash::THUNDER_SLASH_SKILL_ID => Self::ThunderSlash,
            super::callosity::CALLOSITY_SKILL_ID | super::callosity2::CALLOSITY_2_SKILL_ID => Self::Callosity,
            super::agility::AGILITY_SKILL_ID | super::agility2::AGILITY_2_SKILL_ID
                | super::natural::NATURAL_SKILL_ID | super::rapture::RAPTURE_SKILL_ID
                | super::daubpoison::DAUB_POISON_SKILL_ID
                | super::manashield::MANA_SHIELD_SKILL_ID
                | super::machineshield::MACHINE_SHIELD_SKILL_ID => Self::SelfState,
            super::lightingarrow::LIGHTING_ARROW_SKILL_ID => Self::LightingArrow,
            super::lightingarrow2::LIGHTING_ARROW_2_SKILL_ID => Self::LightingArrow2,
            super::meteorarrowmass::METEOR_ARROW_MASS_SKILL_ID => Self::MeteorArrowMass,
            super::meteorarrow::METEOR_ARROW_SKILL_ID => Self::MeteorArrow,
            super::rainarrowphalanx::RAIN_ARROW_SKILL_ID => Self::RainArrow,
            super::fallingstar::FALLING_STAR_SKILL_ID => Self::FallingStar,
            super::poisonmoth::POISON_MOTH_SKILL_ID => Self::PoisonMoth,
            super::scopedarrowcast::BLOOD_ROSE_SKILL_ID
                | super::scopedarrowcast::EXPLOSIVE_ARROW_SKILL_ID
                | super::scopedarrowcast::EXPLOSIVE_ARROW_2_SKILL_ID
                | super::scopedarrowcast::EXPLOSIVE_ARROW_3_SKILL_ID => Self::ScopedArrow,
            super::scorpion::SCORPION_SKILL_ID => Self::Scorpion,
            super::boalock::BOA_LOCK_SKILL_ID => Self::BoaLock,
            super::strike::STRIKE_SKILL_ID | super::yakshaslash::YAKSHA_SLASH_SKILL_ID => Self::TargetedProjectile,
            super::kerosene::KEROSENE_SKILL_ID | super::ignition::IGNITION_SKILL_ID => Self::Combustion,
            super::heartlessarrow::HEARTLESS_ARROW_SKILL_ID => Self::HeartlessArrow,
            super::heartlessarrow2::HEARTLESS_ARROW_2_SKILL_ID
                | super::heartlessarrow3::HEARTLESS_ARROW_3_SKILL_ID => Self::HeartlessArrowArea,
            id if super::immediatestate::is_immediate_state_skill(id) => Self::ImmediateState,
            id if super::heal::is_heal_skill(id) => Self::Heal,
            id if super::nonfun::is_non_fun_skill(id) => Self::NonFun,
            id if super::godbless::is_god_bless_skill(id) => Self::GodBless,
            id if super::zonalcast::is_zonal_cast_skill(id) => Self::ZonalCast,
            super::lightning::LIGHTNING_SKILL_ID => Self::Lightning,
            super::chainlightning::CHAIN_LIGHTNING_SKILL_ID => Self::ChainLightning,
            _ => return None,
        })
    }

    fn completion_end_argument(self, skill_id: u32) -> i32 {
        // Выпуск RainArrow, AI Swordship и NonFun передают End(0); внешний End(1)
        // по-прежнему выполняет AfterUse. Это не политика смены региона.
        match self {
            Self::RainArrow | Self::NonFun => 0,
            Self::ImmediateState => super::immediatestate::immediate_completion_end_argument(skill_id),
            _ => 1,
        }
    }

    pub(crate) fn execute<Runtime: GameMainLoopRuntime>(
        self, game: &mut CGame, player_id: i32, instance: RegisteredSkill,
        dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let execute = match self {
            Self::BaseProjectile => super::baseprojectilecast::execute_player_base_projectile::<Runtime>,
            Self::GodPunishment => super::godpunishment::execute_player_god_punishment::<Runtime>,
            Self::ImmediateState => super::immediatestate::execute_player_immediate_state::<Runtime>,
            Self::Heal => super::heal::execute_player_heal::<Runtime>,
            Self::NonFun => super::nonfun::execute_player_non_fun::<Runtime>,
            Self::GodBless => super::godbless::execute_player_god_bless::<Runtime>,
            Self::ZonalCast => super::zonalcast::execute_player_zonal_cast::<Runtime>,
            Self::Lightning => super::lightning::execute_player_lightning::<Runtime>,
            Self::ChainLightning => super::chainlightning::execute_player_chain_lightning::<Runtime>,
            Self::Flash => super::flash::execute_player_flash::<Runtime>,
            Self::LittleFlash => super::littleflash::execute_player_little_flash::<Runtime>,
            Self::Rush => super::rush::execute_player_rush::<Runtime>,
            Self::Rush2 => super::rush2::execute_player_rush_2::<Runtime>,
            Self::ArmyBreak => super::armybreak::execute_player_army_break::<Runtime>,
            Self::GhostCut => super::ghostcut::execute_player_ghost_cut::<Runtime>,
            Self::Mosou => super::mosou::execute_player_mosou::<Runtime>,
            Self::ThunderBlow2 => super::thunderblow2::execute_player_thunder_blow_2::<Runtime>,
            Self::Swallow => super::swallow::execute_player_swallow::<Runtime>,
            Self::KnightCut => super::knightcut::execute_player_knight_cut::<Runtime>,
            Self::LeafCut => super::leafcut::execute_player_leaf_cut::<Runtime>,
            Self::FrontCellSword => super::frontcellswordcast::execute_player_front_cell_sword::<Runtime>,
            Self::EnergyHolding => super::energyholding::execute_player_energy_holding::<Runtime>,
            Self::Pillar => super::pillar::execute_player_pillar::<Runtime>,
            Self::Roar => super::roar::execute_player_roar::<Runtime>,
            Self::ThunderSlash => super::thunderslash::execute_player_thunder_slash::<Runtime>,
            Self::Callosity => super::callosity::execute_player_callosity::<Runtime>,
            Self::SelfState => super::selfstatecast::execute_player_self_state::<Runtime>,
            Self::LightingArrow => super::lightingarrow::execute_player_lighting_arrow::<Runtime>,
            Self::LightingArrow2 => super::lightingarrow2::execute_player_lighting_arrow_2::<Runtime>,
            Self::MeteorArrowMass => super::meteorarrowmass::execute_player_meteor_arrow_mass::<Runtime>,
            Self::MeteorArrow => super::meteorarrow::execute_player_meteor_arrow::<Runtime>,
            Self::RainArrow => super::rainarrow::execute_player_rain_arrow::<Runtime>,
            Self::FallingStar => super::fallingstar::execute_player_falling_star::<Runtime>,
            Self::PoisonMoth => super::poisonmoth::execute_player_poison_moth::<Runtime>,
            Self::ScopedArrow => super::scopedarrowcast::execute_player_scoped_arrow::<Runtime>,
            Self::Scorpion => super::scorpion::execute_player_scorpion::<Runtime>,
            Self::BoaLock => super::boalock::execute_player_boa_lock::<Runtime>,
            Self::TargetedProjectile => super::targetedprojectile::execute_player_targeted_projectile::<Runtime>,
            Self::Combustion => super::combustioncast::execute_player_combustion::<Runtime>,
            Self::HeartlessArrow => super::heartlessarrow::execute_player_heartless_arrow::<Runtime>,
            Self::HeartlessArrowArea => super::heartlessarrow2::execute_player_heartless_arrow_area::<Runtime>,
        };
        execute(game, player_id, instance, dispatch, runtime)
    }
}

fn finish_outcome<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, outcome: QueuedSkillExecutionOutcome,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let end = match outcome.state {
        QueuedSkillExecutionState::Rejected => Some((0, SkillTermination::Rejected)),
        QueuedSkillExecutionState::RejectedAfterUse => Some((1, SkillTermination::Completed)),
        QueuedSkillExecutionState::Completed => {
            let argument = game.registered_skill(instance).map_or(1, |skill| {
                RegisteredPlayerCastOwner::from_skill_id(skill.id())
                    .map_or(1, |owner| owner.completion_end_argument(skill.id()))
            });
            Some((argument, SkillTermination::Completed))
        }
        QueuedSkillExecutionState::Begun | QueuedSkillExecutionState::Pending => None,
    };
    if let Some((argument, termination)) = end {
        if game.end_registered_instance(instance, argument, termination, runtime) == Some(RegisteredSkillEnd::Released) {
            return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Pending, ..outcome };
        }
    }
    outcome
}

pub(super) fn execute_registered_player_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime, visual_kind: SkillVisualEffectKind,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(PlayerSkillDispatch, u32) -> PlayerSkillExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    execute_registered_player_cast_impl(
        game, player_id, instance, dispatch, runtime, Some(visual_kind), check, materialize, run_ai,
    )
}

pub(super) fn execute_registered_player_cast_without_visual<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(PlayerSkillDispatch, u32) -> PlayerSkillExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    execute_registered_player_cast_impl(
        game, player_id, instance, dispatch, runtime, None, check, materialize, run_ai,
    )
}

fn execute_registered_player_cast_impl<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime, visual_kind: Option<SkillVisualEffectKind>,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(PlayerSkillDispatch, u32) -> PlayerSkillExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.id() != dispatch.skill_id() {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    if let Some(previous) = skill.player_dispatch() {
        if previous != dispatch {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let outcome = run_ai(game, instance, runtime);
        return finish_outcome(game, instance, outcome, runtime);
    }
    if !game.begin_registered_player_skill_with_combat(
        instance, player_id, dispatch, runtime.now_milliseconds(),
    ) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    let Some(skill) = game.registered_skill_mut(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if let Some(visual_kind) = visual_kind {
        skill.replace_visual_effect(SkillVisualEffect::new(visual_kind, 1));
    }
    let mut execution = materialize(dispatch, skill.lifecycle().started_at_ms());
    execution.kernel_mut().clear_phase_for_end();
    if !skill.install_player_execution(execution) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    if !check(game, instance, player_id, runtime) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Idle, SkillStage::Begin);
    }
    state_skill_outcome(QueuedSkillExecutionState::Begun)
}
