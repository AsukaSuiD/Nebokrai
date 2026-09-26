//! Накопление энергии CEnergyHolding (0x89).
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/energyholding.cpp`. Машинные якоря:
//! Begin `0x149BF0`, Check `0x14A190`, AI `0x14A4A0`; skill End(H) 3-fold
//! `0x1502F0` — общий зарегистрированный End, здесь не дублируется
//! (порядок clear+End исполняет прежний kernel/вход). Прежний переходный
//! владелец — `src/gameserver/appserver/skills/energyholding.rs`; тела
//! Check/AI перенесены буквально порцией №6c «self/zone-касты» (разведка —
//! запись аудита «Zone skills: машинная разведка battlefairy-навыков
//! (порция №6)», 26 сентября 2026).
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
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::SelfCastGame`
//! реализован у прежнего владельца; общий зарегистрированный вход остаётся
//! у `playercast` делегата; machine накопления `accumulatedstate` объявлена
//! швом `add_energy_holding_state` (владелец поделён с SoulCollect).

use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::dispatch::PlayerSkillDispatch;
use super::energyholdingstate::{EnergyHoldingState, add_energy_holding, typed_first_energy_holding};
use super::lifecycle::{SkillStage, skill_is_restored};
use super::selfcast::{SelfCastExecutionOutcome, SelfCastGame, SelfCastMoveShape, SelfCastPlayer};
use crate::content::CSkillBaseProperties;

pub const ENERGY_HOLDING_SKILL_ID: u32 = crate::effects::ENERGY_HOLDING_STATE_ID;
const USER_MP_LOSE: u32 = 2;
pub const PARAMETER_PERCENT: u32 = 20_020;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

fn weapon_is_valid<Game: SelfCastGame>(game: &Game, player: &Game::Player) -> bool {
    game.player_weapon_addon_category(player).is_some_and(|category| category == 2)
}

fn failure<Game: SelfCastGame>(game: &mut Game, instance: Game::SkillAddress, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 13 => b"GS0278", 14 => b"GS0292", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn resource_failure<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

pub fn check_cast<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, now_milliseconds: fn() -> u32,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let source = (player.shape().get_region_id(), player.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
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

pub fn run_ai<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, now_milliseconds: fn() -> u32,
) -> SelfCastExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return SelfCastExecutionOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return SelfCastExecutionOutcome::Pending;
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(region, identity).or_else(|| {
        let (region, identity) = game.resolve_skill_sufferer(skill.lifecycle())?;
        game.resolve_state_move_shape(region, identity)
    }).map(|source| (source.shape().get_region_id(), source.shape().identity())) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return SelfCastExecutionOutcome::RejectedAfterUse;
    }
    if stage == SkillStage::Begin {
        if source.1.object_type == PLAYER_TYPE {
            let Some(player) = game.find_player(source.1.id) else { return SelfCastExecutionOutcome::Rejected; };
            let mana = player.mana();
            let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
            if (remaining as i32) < 0 {
                resource_failure(game, instance, source.1.id, &properties);
                return SelfCastExecutionOutcome::Rejected;
            }
            if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
            game.publish_player_states(source.1.id);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return SelfCastExecutionOutcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    if now_milliseconds() < started.wrapping_add(delay) {
        return SelfCastExecutionOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    let _ = add_energy_holding(game, source, |game| {
        let percent = properties.query_property(PARAMETER_PERCENT);
        let level = game.registered_skill(instance)?.level() as u32;
        Some(EnergyHoldingState::new(level, percent))
    }, &mut || now_milliseconds());
    SelfCastExecutionOutcome::Completed
}

/// Дисциплина dispatch-а сохранена у зарегистрированного входа делегата.
pub const fn is_energy_holding_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == ENERGY_HOLDING_SKILL_ID
}
