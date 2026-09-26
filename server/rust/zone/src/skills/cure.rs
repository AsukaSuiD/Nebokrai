//! Правила и исполнение CCure (0x131) — очищение состояний, и выбор
//! снимаемых состояний CastCure. Источник: GameServer/gameserver.exe +
//! GameServer/GameServer.pdb (точная пара `4F5C98E0…` + RSDS match),
//! `appserver/skills/cure.cpp/.h`. Машинные якоря семьи: CCure Begin
//! `0x1AD3A0`, собственное DoesTargetEffective `0x1AD590` (NULL S с
//! fallback U), CastCure `0x1ADB10` (порядок обхода позиций и расход RNG
//! на каждую подходящую), AI `0x1AE110`, CCureState 8-байтный codec fold
//! (Serialize `0x1F51E0`, Unserialize `0x1E9AC0`). Прежний переходный
//! владелец — `src/gameserver/appserver/skills/cure.rs`; тела Check/AI,
//! диагностика и обход состояний перенесены буквально порцией №6a
//! (разведка — запись аудита «Zone skills: машинная разведка
//! battlefairy-навыков (порция №6)», 26 сентября 2026).
//!
//! Общий stateskill владеет Begin, visual loop1 и полным End игрока и
//! монстра; здесь выбор цели, MP, путь и CastCure. AI сохраняет участников
//! и таблицу свойств до callbacks. Потерянная S заменяется U только для
//! текущего вызова; дикий нетранспортный монстр дополнительно переназначает
//! сохранённую identity цели. CAN_BE_BREAKED независим от фазы исполнения.
//! Reuse и delay сравнивают wrapping DWORD. CastCure перечитывает живой
//! вектор и расходует RNG для каждой подходящей позиции. Новый CureState
//! начинает действие до End первого прежнего Cure; после замены нет
//! дополнительного UpdateProperty.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::{StateCastGame,
//! StateCastPlayer, StateCastMoveShape}` реализован у прежнего владельца;
//! разрешение `begin_target`, материализация исполнения и общий полный End
//! остаются в `stateskill.rs` делегата. Безусловный базовый callback visual
//! остаётся у зарегистрированного ресурса.
//!
//! `cure_threshold` — `FILD` unsigned-модификатора с поправкой 2^32, затем
//! `FMUL 0.01`, `FIMUL` свойства игрока и `FISTP DWORD` перед DWORD-арифметикой.

use nebokrai_shared::runtime::get_line_direction;

use crate::combat::truncate_original;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::curestate::begin_and_replace_cure_state;
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastMoveShape, StateCastPlayer,
    StateCastVisualContract, StateCastVisualTarget, publish_state_cast_visual, state_cast_participant,
};
use super::visualeffect::SkillVisualEffectKind;

pub const CURE_SKILL_ID: u32 = crate::effects::CURE_STATE_SKILL_ID;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;
const CONSTANT: u32 = 20_010;
const EM_MODIFIER: u32 = 20_015;
const PROBABILITY: u32 = 40_001;

pub const fn is_cure_removable_state_id(state_id: u32) -> bool {
    matches!(
        state_id,
        0x138 | 0xd2 | 0xc9 | 0x67 | 0x192 | 0x191 | 0x198 | 0x199 | 0x1a6 | 0x73 | 0x7c | 0x1f8
    )
}

/// `FILD` unsigned-модификатора с поправкой 2^32, затем `FMUL 0.01`,
/// `FIMUL` свойства игрока и `FISTP DWORD` перед DWORD-арифметикой.
pub fn cure_threshold(element_modify: i32, probability: u32, constant: u32, em_modifier: u32) -> i32 {
    let scaled = truncate_original(
        f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    (scaled as u32).wrapping_mul(constant).wrapping_add(probability) as i32
}

static CURE_VISUAL: StateCastVisualContract = StateCastVisualContract {
    skill_id: CURE_SKILL_ID,
    kind: SkillVisualEffectKind::Cure,
    failures: &[2, 7, 10, 11, 13, 15],
    target: StateCastVisualTarget::SuffererOrUser,
};

pub fn publish_cure_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    publish_state_cast_visual(game, skill, mode, &CURE_VISUAL);
}

fn failure<Game: StateCastGame>(
    game: &mut Game, address: Game::SkillAddress, source: (i32, ShapeIdentity),
    mode: u32, text: &[u8], amount: Option<u32>,
) {
    game.update_registered_skill_visual(address, mode);
    if source.1.object_type == PLAYER_TYPE {
        if let Some(amount) = amount {
            game.send_skill_system_info_with_unsigned(source.1.id, text, amount);
        } else { game.send_skill_system_info(source.1.id, text); }
    }
}

fn cast_cure_states<Game: StateCastGame>(
    game: &mut Game, region_id: i32, target: ShapeIdentity, threshold: i32,
) {
    let mut index = 0;
    loop {
        let Some(shape) = game.resolve_state_move_shape(region_id, target) else { return; };
        if index >= shape.state_slot_count() { break; }
        let selected = shape.state_at(index).is_some_and(|(_, state)| state.is_curable());
        if selected && game.skill_random_below(100) < threshold {
            let _ = game.end_and_destroy_state_at(region_id, target, index);
        }
        index += 1;
    }
}

pub fn check_cast<Game: StateCastGame>(
    game: &mut Game,
    address: Game::SkillAddress,
    begin_target: Option<(i32, ShapeIdentity)>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(skill) = game.registered_skill(address) else { return false; };
    let Some(source) = state_cast_participant(game, skill.lifecycle().user()) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let Some(target) = begin_target.and_then(|target| state_cast_participant(game, target)) else {
        failure(game, address, source, 10, b"GS0286", None);
        return false;
    };
    if !skill_is_restored(skill.last_used_ms(), properties.query_property(REUSE), now()) {
        failure(game, address, source, 13, b"GS0278", None);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if source != target && properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, address, source, 11, b"GS0290", None);
        return false;
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_registered_skill_visual(address, 15);
        if source.1.object_type == PLAYER_TYPE {
            let name = game.base_magic_target_name(target.0, target.1);
            game.send_skill_system_info_with_text(source.1.id, b"GS0295", &name);
        }
        return false;
    }
    if source.1.object_type == PLAYER_TYPE {
        if properties.query_property(MP_LOSS) == 0 { return false; }
        let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else { return false; };
        if (mana.wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0 {
            failure(game, address, source, 7, b"GS0288", Some(properties.query_property(MP_LOSS)));
            return false;
        }
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.set_moveable(false);
        }
    }
    true
}

pub fn run_cure_ai<Game: StateCastGame>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> StateCastExecutionOutcome {
    let Some(skill) = game.registered_skill(address) else {
        return StateCastExecutionOutcome::Rejected;
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return StateCastExecutionOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return StateCastExecutionOutcome::EndRejected;
    };
    let source = state_cast_participant(game, skill.lifecycle().user());
    let mut target = game.resolve_skill_sufferer(skill.lifecycle())
        .and_then(|target| state_cast_participant(game, target));
    if let Some(value) = target {
        let redirect = value.1.object_type == MONSTER_TYPE
            && game.wild_untamed_non_carriage_monster(value.0, value.1.id);
        if redirect {
            target = source;
            if let Some(source) = source && let Some(skill) = game.registered_skill_mut(address) {
                skill.lifecycle_mut().set_sufferer_identity(source.1);
            }
        }
    } else { target = source; }
    let (Some(source), Some(target)) = (source, target) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    if game.base_magic_target_dead(target.0, target.1) {
        failure(game, address, source, 10, b"GS0285", None);
        return StateCastExecutionOutcome::EndRejected;
    }
    if game.registered_skill(address).and_then(|skill| skill.execution_stage()) == Some(SkillStage::Begin) {
        if source.1.object_type == PLAYER_TYPE {
            let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else {
                return StateCastExecutionOutcome::EndRejected;
            };
            let remaining = mana.wrapping_sub(properties.query_property(MP_LOSS));
            if (remaining as i32) < 0 {
                failure(game, address, source, 7, b"GS0288", Some(properties.query_property(MP_LOSS)));
                return StateCastExecutionOutcome::EndRejected;
            }
            if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
            game.publish_player_states(source.1.id);
        }
        if let Some(skill) = game.registered_skill_mut(address) {
            skill.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
        }
        let direction = (|| {
            let target = game.resolve_state_move_shape(target.0, target.1)?.shape();
            let target_y = target.get_tile_y().unwrap_or(i32::MIN);
            let target_x = target.get_tile_x().unwrap_or(i32::MIN);
            let source = game.resolve_state_move_shape(source.0, source.1)?.shape();
            let source_y = source.get_tile_y().unwrap_or(i32::MIN);
            let source_x = source.get_tile_x().unwrap_or(i32::MIN);
            Some(get_line_direction(source_x, source_y, target_x, target_y))
        })();
        if let Some(direction) = direction
            && let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1)
        {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(address, 0);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(skill) = game.registered_skill(address) else {
        return StateCastExecutionOutcome::Rejected;
    };
    if skill.execution_stage() != Some(SkillStage::Check) {
        return StateCastExecutionOutcome::Pending;
    }
    let delay = properties.query_property(DELAY);
    let started = skill.lifecycle().started_at_ms();
    if now() < started.wrapping_add(delay) {
        return StateCastExecutionOutcome::Pending;
    }
    game.update_registered_skill_visual(address, 1);
    let em_modifier = properties.query_property(EM_MODIFIER);
    let constant = properties.query_property(CONSTANT);
    let probability = properties.query_property(PROBABILITY);
    let element_modify = if source.1.object_type == PLAYER_TYPE {
        game.find_player(source.1.id).map_or(0, |player| player.combat_properties().element_modify)
    } else { 0 };
    if game.resolve_state_move_shape(source.0, source.1).is_some() {
        cast_cure_states(game, target.0, target.1, cure_threshold(element_modify, probability, constant, em_modifier));
    }
    let state = crate::effects::CureState::new(properties.query_property(PERSIST));
    let _ = begin_and_replace_cure_state(game, Some(source), Some(target), state, now);
    StateCastExecutionOutcome::EndCompleted
}

/// Завершение первого curable-состояния с заданным ID через полный End
/// ветки арены; ничего не начинает взамен.
pub fn finish_curable_state<Game: StateCastGame>(
    game: &mut Game,
    region_id: i32,
    target: ShapeIdentity,
    state_id: u32,
    _now_ms: u32,
) -> bool {
    let Some(shape) = game.resolve_state_move_shape(region_id, target) else { return false; };
    let key = (0..shape.state_slot_count()).find_map(|index| {
        let (key, state) = shape.state_at(index)?;
        (state.is_curable() && state.state_id() == state_id).then_some(key)
    });
    let Some(key) = key else { return false; };
    game.end_move_shape_state(region_id, target, key)
}

pub const fn is_cure_target(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == CURE_SKILL_ID
}
