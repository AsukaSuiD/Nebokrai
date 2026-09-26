//! Правила и исполнение CHeal/CHeal2/CSuperHeal/CSuperHeal2
//! (0xD3/0xE3/0xD9/0xE4) — периодическое лечение.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/heal{,2}.cpp` и `superheal{,2}.cpp`.
//! Тела Check/AI перенесены буквально. Машинные якоря: CHeal Check — reuse(0x2715)+tick
//! visual13/GS0278 → GetTargetPath безусловно → distance-пара → MP
//! (cost>0 signed-diff, Move0 только при успехе; cost0 → visual7/GS0288);
//! AI: MP→OnChangeStates→CAN→направление→visual0→delay→visual1→формула
//! `coeff*weapon_level*0.01+const` (FISTP) → QueryProperty(6001) затем
//! QueryProperty(10002) → ctor state(J,J) → Begin(U,S).
//!
//! **Установленный FIX:** `TARGET_AFFECT_FREQUENCY` — машинно `6001`
//! (во всех четырёх AI, якорь CHeal `0x581861`: прямое `QueryProperty(6001)`
//! перед `10002`), не `5002`, как держала прежняя реконструкция.
//!
//! Зарегистрированный экземпляр игрока или монстра хранит единственную
//! базу и фазу; отдельного payload и снимка цели нет. Объектный Begin
//! проверяет исходные U/S, координатный — свежую S либо U после loop1
//! visual. Check требует обе фигуры, абсолютный reuse и свежий
//! GetTargetPath даже при самоцели. Дальность проверяется только для
//! разных U/S; препятствия не запрещают лечение. Player MP0 — тихий отказ,
//! положительная цена проверяется по знаку DWORD-разности перед Move0.
//! Источник другого типа проходит без MP и запрета движения.
//!
//! Каждый AI удерживает таблицу свойств и найденные U/S через callbacks.
//! Отсутствующая S заменяется свежим U; обычный неприручённый монстр без
//! Carriage AI дополнительно переназначает базовые type/id цели на U.
//! Только смерть выбранной S отменяет лечение. Первый AI выполняет
//! MP→OnChangeStates→CAN→S.Y/X→U.Y/X→направление→visual0→condition.
//! После unsigned start+delay visual1 заново разрешает S либо U, тогда как
//! состояние устанавливается на захваченную этим AI цель.
//!
//! Прибавка Player считается после visual1: COEFF→живой уровень оружия,
//! unsigned wrapping-произведение с 0.01f сохраняется в f32; CONST
//! прибавляется с отдельным сохранением f32. У источника другого типа
//! прибавка равна нулю. После End/destructor первого прежнего состояния
//! значение усекается FISTP, затем читаются FREQ→PERSIST. Новый primary
//! Begin предшествует append; отдельного UpdateProperty или пакета
//! состояния здесь нет. SuperHeal заменяет только Heal; SuperHeal2 хранится
//! у выбранной S, но primary Begin получает U/U. Эти различия принадлежат
//! `super::healstate`. Выпуск всегда завершает навык End1, ранние отказы —
//! End0. Общий End сбрасывает фазу, разрешает свежий U Move1 и сохраняет
//! исходный аргумент. Vec, общий kernel и каноническое хранилище заменяют
//! native STL/указатели.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::*`
//! реализован у владельца старого пакета; Check-скелет `rangedweaponcast`
//! (`check_skill_path` ветки Ignore и MP-контракт) и материализация
//! исполнения остаются в старом пакете и объявлены швами.

use nebokrai_shared::runtime::get_line_direction;

use crate::combat::truncate_original;
use crate::effects::HealState;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::healstate::replace_heal_state;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastMoveShape,
    StateCastVisualContract, StateCastVisualTarget, publish_state_cast_visual, state_cast_participant,
};
use super::visualeffect::SkillVisualEffectKind;

pub const HEAL_SKILL_ID: u32 = 0xd3;
pub const HEAL_2_SKILL_ID: u32 = 0xe3;
pub const SUPER_HEAL_SKILL_ID: u32 = 0xd9;
pub const SUPER_HEAL_2_SKILL_ID: u32 = 0xe4;
const CONST: u32 = 20_010;
const HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
/// Машинный якорь `0x581861` (CHeal): `QueryProperty(6001)` перед `10002`.
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;

pub const fn is_heal_skill(skill_id: u32) -> bool {
    matches!(skill_id, HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID)
}

const HEAL_VISUAL_FAILURES: &[u32] = &[2, 7, 10, 11, 13, 15];

pub fn publish_heal_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if is_heal_skill(skill.id()) {
        let contract = StateCastVisualContract {
            skill_id: skill.id(),
            kind: SkillVisualEffectKind::Heal,
            failures: HEAL_VISUAL_FAILURES,
            target: StateCastVisualTarget::SuffererOrUser,
        };
        publish_state_cast_visual(game, skill, mode, &contract);
    }
}

/// Check: обе фигуры, абсолютный reuse, свежий GetTargetPath даже при
/// самоцели; distance-пара только для разных U/S, затем MP-контракт.
pub fn check_heal_cast<Game: StateCastGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let (Some(user), Some(target)) = (original_user, original_target) else { return false; };
    let Some(source_shape) = game.resolve_state_move_shape(user.0, user.1) else { return false; };
    let Some(target_shape) = game.resolve_state_move_shape(target.0, target.1) else { return false; };
    let same_target = std::ptr::eq(source_shape, target_shape);
    let source = (source_shape.shape().get_region_id(), source_shape.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !same_target && !game.check_skill_path(instance, &properties, &path, player) {
        return false;
    }
    game.check_cast_mana(instance, source, &properties)
}

/// AI: MP→CAN→направление→visual0→delay→visual1→формула→установка состояния.
/// Только смерть выбранной S отменяет лечение; выпуск всегда End1.
pub fn run_heal_ai<Game: StateCastGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> StateCastExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return StateCastExecutionOutcome::Pending;
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return StateCastExecutionOutcome::EndRejected;
    };
    let source = state_cast_participant(game, skill.lifecycle().user());
    let mut target = game.resolve_skill_sufferer(skill.lifecycle())
        .and_then(|target| state_cast_participant(game, target));
    if target.is_none() {
        target = state_cast_participant(game, skill.lifecycle().user());
    } else if target.is_some_and(|(region, identity)| {
        identity.object_type == MONSTER_TYPE
            && game.wild_untamed_non_carriage_monster(region, identity.id)
    }) {
        target = state_cast_participant(game, skill.lifecycle().user());
        if let Some(target) = target
            && let Some(skill) = game.registered_skill_mut(instance)
        { skill.lifecycle_mut().set_sufferer_identity(target.1); }
    }
    let (Some(source), Some(target)) = (source, target) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(instance, 10);
        return StateCastExecutionOutcome::EndRejected;
    }
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    if stage == SkillStage::Begin {
        if !game.spend_cast_mana(instance, player, &properties) {
            return StateCastExecutionOutcome::EndRejected;
        }
        let can_break = properties.query_property(CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return StateCastExecutionOutcome::EndRejected;
        };
        skill.lifecycle_mut().set_available(can_break != 0);
        if source != target {
            let Some(sufferer) = game.resolve_state_move_shape(target.0, target.1) else {
                return StateCastExecutionOutcome::EndRejected;
            };
            let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
            let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
            let Some(user) = game.resolve_state_move_shape(source.0, source.1) else {
                return StateCastExecutionOutcome::EndRejected;
            };
            let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
            let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
            let direction = get_line_direction(source_x, source_y, target_x, target_y);
            if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
                user.shape_mut().set_direction(direction);
            }
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    if now() < started.wrapping_add(delay) {
        return StateCastExecutionOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    let hp_gain = if let Some(user) = player.and_then(|player| game.find_player(player)) {
        let coefficient = properties.query_property(HEAL_RECOVER_COEFFICIENT);
        let weapon = game.player_weapon_damage_level(user) as u32;
        let scaled = (f64::from(coefficient.wrapping_mul(weapon)) * f64::from(0.01_f32)) as f32;
        let constant = properties.query_property(CONST);
        (f64::from(constant) + f64::from(scaled)) as f32
    } else { 0.0 };
    let _ = replace_heal_state(game, source, target, skill_id, || {
        let hp_gain = truncate_original(f64::from(hp_gain)) as u32;
        let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
        let keep = properties.query_property(STATE_PERSIST_TIME);
        HealState::new(skill_id, keep, frequency, hp_gain)
    }, now);
    StateCastExecutionOutcome::EndCompleted
}
