//! Накопление энергии CEnergyHolding (0x89).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/energyholding.cpp.
//!
//! Зарегистрированный Attack Begin сохраняет раннее время и visual loop1;
//! Check проверяет исходного игрока, reuse, оружие категории 2, signed MP
//! и RTTI первого state с ID 0x89. Нулевая цена разрешена, отсутствие
//! состояния не подменяется нулём зарядов при проверке предела. Только
//! успешный Check запрещает движение; отказ сразу заканчивается End(0).
//!
//! AI сохраняет таблицу свойств и найденного U либо S через callbacks.
//! Смерть даёт visual2 и End(1); недостаток MP — visual7 и End(0).
//! Первый AI списывает MP до OnChangeStates, затем задаёт CAN, visual0
//! и condition без повторной проверки оружия. Абсолютный unsigned срок
//! start+delay предшествует visual1 и накоплению. Типизированный первый
//! state с ID 0x89 увеличивается без чтения параметров нового;
//! создание читает процент, затем свежий уровень навыка. Успех накопления
//! не меняет завершающий End(1). Общий End сбрасывает фазу, разрешает
//! движение свежему U либо S и передаёт исходный аргумент в Attack End.
//! Kernel владеет единственным исполнением; отдельного пути здесь нет.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::energyholdingstate::{EnergyHoldingState, add_energy_holding, typed_first_energy_holding};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const ENERGY_HOLDING_SKILL_ID: u32 = 0x89;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
pub(crate) const PARAMETER_PERCENT: u32 = 20_020;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0292", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn resource_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let source = (player.shape().get_region_id(), player.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    let Some(player) = game.find_player(player_id) else { return false; };
    if !weapon_is_valid(game, player) {
        failure(game, instance, player_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties);
            return false;
        }
    }
    if let Some(state) = typed_first_energy_holding(game, source) {
        let count = state.energy_count();
        let Some(level) = game.registered_skill(instance).map(|skill| skill.level() as u32) else { return false; };
        if count >= level {
            game.send_skill_system_info(player_id, b"GS0299");
            return false;
        }
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity).or_else(|| {
        let (region, identity) = resolve_skill_sufferer(game, skill.lifecycle())?;
        resolve_state_move_shape(game, region, identity)
    }).map(|source| (source.shape().get_region_id(), source.shape().identity())) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    }
    if stage == SkillStage::Begin {
        if source.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(source.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                resource_failure(game, instance, source.1.id, &properties);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
            game.publish_player_states(source.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    let _ = add_energy_holding(game, source, |game| {
        let percent = properties.query_property(PARAMETER_PERCENT);
        let level = game.registered_skill(instance)?.level() as u32;
        Some(EnergyHoldingState::new(level, percent))
    }, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_energy_holding<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != ENERGY_HOLDING_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        check_cast, |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
