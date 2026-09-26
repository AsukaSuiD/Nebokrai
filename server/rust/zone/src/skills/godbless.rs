//! Правила и исполнение CGodBless/CGodBless2 (0x12F/0x145), параметры и
//! создание их состояний. Источник: gameserver.exe + GameServer.pdb
//! (точная пара `4F5C98E0…` + RSDS match), `appserver/skills/
//! godbless{,2}.cpp/.h`. Машинные якоря семьи: CGodBless Check `0x1B02A0`
//! (не-Player возвращает 1 ДО MP/Move0; arg-null → тихий 0; cost0 → тихий
//! 0; signed-diff → visual7/GS0288 с number или Move0 → 1), AI `0x1B0480`;
//! CGodBless2 AI `0x150990` (visual10/GS0305 при не-Monster); End(H)
//! 3-fold `0x1502F0`. Прежний переходный владелец — `src/gameserver/
//! appserver/skills/godbless.rs`; тела Check/AI перенесены буквально
//! порцией №6a (разведка — запись аудита «Zone skills: машинная разведка
//! battlefairy-навыков (порция №6)», 26 сентября 2026).
//!
//! Оба Begin сохраняют исходную цель в общей базе, создают visual loop1
//! и проверяют только исходного U. Нет проверки S, пути или оружия.
//! После абсолютного reuse источник не типа Player проходит без Move0;
//! Player MP0 означает тихий отказ, иначе signed DWORD-разность допускает
//! Move0. Отказ Begin вызывает End0 без дополнительного visual2.
//!
//! AI удерживает одну таблицу и найденные U/S через callbacks. GodBless
//! при NULL S использует захваченного U; GodBless2 требует именно Monster
//! при каждом входе AI, иначе Player получает visual10/GS0305 и End0.
//! Обычный неприручённый монстр без Carriage AI заменяется свежим U,
//! с записью базовых type/id S. Поэтому GodBless2 после такой замены и
//! ожидания задержки может отвергнуть уже изменившуюся S на следующем AI.
//! Нет проверки смерти или смены направления. Первый AI выполняет
//! MP→OnChangeStates→CAN→visual0→condition; выпуск ждёт unsigned
//! start+delay.
//!
//! После visual1 захваченный U даёт живой уровень оружия либо ноль.
//! Свойства читаются в порядке MIN_COEFF→MIN→MAX_COEFF→MAX→ELEMENT_COEFF→
//! ELEMENT. Каждое unsigned wrapping-произведение с 0.01f сохраняется в
//! f32, затем прибавление константы отдельно сохраняется в f32. Значения
//! живут через удаление прежнего состояния; только потом PERSIST→FISTP
//! ELEMENT/MAX/MIN→ctor→primary Begin→append→безусловный UpdateProperty.
//! Различие DelExStateByType у GodBless принадлежит общему установщику.
//! Выпуск заканчивается End1 даже при отказе state Begin; ранние отказы —
//! End0.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::*`
//! реализован у прежнего владельца; первичная установка состояния остаётся
//! швом старого пакета (`CGame::install_god_bless_state` со своим обходом
//! Ex-состояний), потому часы и установка приходят вместе через трейт
//! `GodBlessCastRuntime` — у владельца остаётся и Begin часов состояния.

use crate::combat::truncate_original;
use crate::effects::{GOD_BLESS_STATE_2_ID, GodBlessState};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::lifecycle::{SkillStage, skill_is_restored};
use super::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastVisualContract,
    StateCastVisualTarget, publish_state_cast_visual, state_cast_participant,
};
use super::visualeffect::SkillVisualEffectKind;

pub const GOD_BLESS_SKILL_ID: u32 = 0x12f;
pub const GOD_BLESS_2_SKILL_ID: u32 = GOD_BLESS_STATE_2_ID;
const DELAY_TIME: u32 = 10_001;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;

const TARGET_ELEMENT_GAIN: u32 = 115;
const TARGET_MINIMUM_GAIN: u32 = 116;
const TARGET_MAXIMUM_GAIN: u32 = 117;
const TARGET_ELEMENT_COEFFICIENT: u32 = 120;
const TARGET_MINIMUM_COEFFICIENT: u32 = 121;
const TARGET_MAXIMUM_COEFFICIENT: u32 = 122;
const STATE_PERSIST_TIME: u32 = 10_002;

pub const fn is_god_bless_skill(skill_id: u32) -> bool {
    matches!(skill_id, GOD_BLESS_SKILL_ID | GOD_BLESS_2_SKILL_ID)
}

#[derive(Clone, Copy, Debug)]
pub struct GodBlessGains {
    minimum: f32,
    maximum: f32,
    element: f32,
}

impl GodBlessGains {
    /// Снимок трёх прибавок до удаления прежнего состояния.
    pub fn read(weapon: u32, mut query_property: impl FnMut(u32) -> u32) -> Self {
        let mut gain = |coefficient, constant| {
            let coefficient = query_property(coefficient);
            let scaled = (f64::from(coefficient.wrapping_mul(weapon)) * f64::from(0.01_f32)) as f32;
            let constant = query_property(constant);
            (f64::from(constant) + f64::from(scaled)) as f32
        };
        Self {
            minimum: gain(TARGET_MINIMUM_COEFFICIENT, TARGET_MINIMUM_GAIN),
            maximum: gain(TARGET_MAXIMUM_COEFFICIENT, TARGET_MAXIMUM_GAIN),
            element: gain(TARGET_ELEMENT_COEFFICIENT, TARGET_ELEMENT_GAIN),
        }
    }

    /// Вызывать после удаления прежнего состояния: срок запрашивается именно здесь.
    pub fn create_state(
        self, skill_id: u32, mut query_property: impl FnMut(u32) -> u32,
    ) -> GodBlessState {
        let keep_time_ms = query_property(STATE_PERSIST_TIME);
        let element = truncate_original(f64::from(self.element)) as u32;
        let maximum = truncate_original(f64::from(self.maximum)) as u32;
        let minimum = truncate_original(f64::from(self.minimum)) as u32;
        GodBlessState::new(skill_id, keep_time_ms, minimum, maximum, element)
    }
}

/// Часы и первичная установка состояния благословения: runtime главного
/// цикла остаётся у владельца (в нём же Begin часов состояния), поэтому
/// обе операции приходят как один шов, а не как два конфликтующих доступа.
pub trait GodBlessCastRuntime<Game> {
    fn now_milliseconds(&mut self) -> u32;

    fn install_god_bless_state(
        &mut self,
        game: &mut Game,
        user: (i32, ShapeIdentity),
        sufferer: (i32, ShapeIdentity),
        skill_id: u32,
        create: &mut dyn FnMut() -> GodBlessState,
    ) -> bool;
}

static GOD_BLESS_VISUAL: StateCastVisualContract = StateCastVisualContract {
    skill_id: GOD_BLESS_SKILL_ID,
    kind: SkillVisualEffectKind::GodBless,
    failures: &[2, 7, 13],
    target: StateCastVisualTarget::SuffererOrUser,
};

static GOD_BLESS_2_VISUAL: StateCastVisualContract = StateCastVisualContract {
    skill_id: GOD_BLESS_2_SKILL_ID,
    kind: SkillVisualEffectKind::GodBless,
    failures: &[2, 7, 10, 13],
    target: StateCastVisualTarget::SuffererOrUser,
};

pub fn publish_god_bless_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    match skill.id() {
        GOD_BLESS_SKILL_ID => publish_state_cast_visual(game, skill, mode, &GOD_BLESS_VISUAL),
        GOD_BLESS_2_SKILL_ID => publish_state_cast_visual(game, skill, mode, &GOD_BLESS_2_VISUAL),
        _ => {}
    }
}

pub fn check_cast<Game: StateCastGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    original_user: Option<(i32, ShapeIdentity)>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(source) = state_cast_participant(game, (region, identity)) else { return false; };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    game.check_cast_mana(instance, source, &properties)
}

pub fn run_god_bless_ai<Game: StateCastGame, Runtime: GodBlessCastRuntime<Game>>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
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
    let target = game.resolve_skill_sufferer(skill.lifecycle())
        .and_then(|target| state_cast_participant(game, target));
    let Some(source) = source else { return StateCastExecutionOutcome::EndRejected; };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let monster = target.filter(|(_, identity)| identity.object_type == MONSTER_TYPE)
        .and_then(|(region, identity)| game.monster_present(region, identity.id).then_some(()));
    if skill_id == GOD_BLESS_2_SKILL_ID && monster.is_none() {
        if let Some(player) = player {
            game.update_registered_skill_visual(instance, 10);
            game.send_skill_system_info(player, b"GS0305");
        }
        return StateCastExecutionOutcome::EndRejected;
    }
    let ordinary_monster = target.is_some_and(|(region, identity)| {
        identity.object_type == MONSTER_TYPE
            && game.wild_untamed_non_carriage_monster(region, identity.id)
    });
    let target = if ordinary_monster {
        let Some(target) = game.registered_skill(instance)
            .and_then(|skill| state_cast_participant(game, skill.lifecycle().user()))
        else {
            return StateCastExecutionOutcome::EndRejected;
        };
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_sufferer_identity(target.1);
        }
        target
    } else { target.unwrap_or(source) };
    if stage == SkillStage::Begin {
        if !game.spend_cast_mana(instance, player, &properties) {
            return StateCastExecutionOutcome::EndRejected;
        }
        let can_break = properties.query_property(CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return StateCastExecutionOutcome::EndRejected;
        };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return StateCastExecutionOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    let weapon = player.and_then(|player| game.find_player(player))
        .map_or(0, |player| game.player_weapon_damage_level(player));
    let gains = GodBlessGains::read(weapon, |key| properties.query_property(key));
    let _ = runtime.install_god_bless_state(game, source, target, skill_id, &mut || {
        gains.create_state(skill_id, |key| properties.query_property(key))
    });
    StateCastExecutionOutcome::EndCompleted
}
