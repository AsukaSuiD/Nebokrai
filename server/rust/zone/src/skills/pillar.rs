//! Защитная стойка CPillar (0x74): Check/AI и параметры создаваемой стойки.
//! Источник: `gameserver.exe` (SHA-256 `4F5C98E0…`) + `GameServer.pdb`
//! (RSDS match), `appserver/skills/pillar.cpp/.h`. Машинные якоря: Begin
//! `0x16FA00`, AI `0x170110` (запросы factor→persist — VA
//! 0x0057033C–0x0057039B); состояние принадлежит `pillarstate.cpp/.h`
//! (Begin `0x1F4B60`).
//!
//! Зарегистрированный Attack Begin сохраняет исходного U, раннее время
//! и visual loop1. Check проверяет только reuse и запрещает движение;
//! MP и оружие здесь не проверяются. Отказ Begin получает End(0).
//! Каждый AI сохраняет таблицу свойств и найденного U через callbacks.
//! Смерть U публикует visual2 и даёт End(1); недостаток MP — visual7/End(0).
//! Первый AI списывает MP со signed wrapping-проверкой до OnChangeStates,
//! затем записывает CAN, публикует visual0 и включает condition.
//!
//! Абсолютный unsigned срок start+delay предшествует visual1. Pillar —
//! переключатель: первый слот ID74 снимается без создания нового состояния.
//! Только при отсутствии такого слота читаются factor, затем persist;
//! PillarState получает Begin(U,U), append и отдельный UpdateProperty.
//! Беззнаковый factor умножается на сохранённую f32-константу 0.001
//! в расширенной точности до записи f32. Состояние владеет своим запретом
//! движения; общий End сбрасывает фазу до возврата движения свежему U
//! и выполняет унаследованный State End с исходным аргументом.
//! Ресурсы CPlayer не читаются у чужого живого CMoveShape: исходная ветка
//! использует непроверенное приведение, которое безопасная модель отклоняет.
//!
//! Объявленные швы переноса (не расхождения): hub `selfcast::SelfCastGame`
//! реализован у прежнего владельца; общий зарегистрированный вход и захват
//! исходного U остаются у `playercast` делегата; переключение состояния —
//! `skills/pillarstate.rs`.

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::lifecycle::{SkillStage, skill_is_restored};
use super::pillarstate::{PillarState, toggle_pillar_state};
use super::selfcast::{SelfCastExecutionOutcome, SelfCastGame, SelfCastMoveShape, SelfCastPlayer};

pub const PILLAR_SKILL_ID: u32 = 0x74;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

/// Запросы выполняются только при создании нового состояния: factor, затем срок.
pub fn pillar_state_parameters(mut query_property: impl FnMut(u32) -> u32) -> (u32, f32) {
    let factor =
        (f64::from(query_property(TARGET_DAMAGE_FACTOR)) * f64::from(0.001_f32)) as f32;
    let keep_time_ms = query_property(STATE_PERSIST_TIME);
    (keep_time_ms, factor)
}

fn mana_failure<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32, properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let amount = properties.query_property(USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount);
}

pub fn check_cast<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, original_user: (i32, ShapeIdentity),
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(source) = game.resolve_state_move_shape(original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if source.1.object_type == PLAYER_TYPE { game.send_skill_system_info(source.1.id, b"GS0278"); }
        return false;
    }
    let Some(source) = game.resolve_state_move_shape_mut(source.0, source.1) else { return false; };
    source.set_moveable(false);
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
    let Some(source) = game.resolve_state_move_shape(region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return SelfCastExecutionOutcome::Rejected; };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return SelfCastExecutionOutcome::RejectedAfterUse;
    }
    if stage == SkillStage::Begin {
        if source.1.object_type != PLAYER_TYPE { return SelfCastExecutionOutcome::Rejected; }
        let Some(player) = game.find_player(source.1.id) else { return SelfCastExecutionOutcome::Rejected; };
        let mana = player.mana();
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            mana_failure(game, instance, source.1.id, &properties);
            return SelfCastExecutionOutcome::Rejected;
        }
        if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
        game.publish_player_states(source.1.id);
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
    let _ = toggle_pillar_state(game, source, |_game| {
        let (keep, factor) = pillar_state_parameters(|key| properties.query_property(key));
        Some(PillarState::new(keep, factor))
    }, &mut || now_milliseconds());
    SelfCastExecutionOutcome::Completed
}
