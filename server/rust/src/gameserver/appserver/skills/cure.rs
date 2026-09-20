//! Очищение CCure (0x131), gameserver.exe + GameServer.pdb,
//! appserver/skills/cure.cpp. Общий stateskill владеет Begin, visual и End
//! игрока и монстра; здесь остаются выбор цели, MP, путь и CastCure.
//! AI сохраняет участников и таблицу свойств до callbacks. Потерянная S
//! заменяется U только для текущего вызова; дикий нетранспортный монстр
//! дополнительно переназначает сохранённую identity цели. CAN_BE_BREAKED
//! независим от фазы исполнения. Reuse и delay сравнивают wrapping DWORD.
//! CastCure перечитывает живой вектор и расходует RNG для каждой подходящей
//! позиции. Новый CureState начинает действие до End первого прежнего Cure;
//! после замены нет дополнительного UpdateProperty.

use super::curestate::{CureState, begin_and_replace_cure_state};
use super::fightdefense::truncate_original;
use super::kernel::{SkillStage, SkillTermination, skill_is_restored};
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill, execute_player_state_skill, finish_player_state_skill,
    publish_state_skill_visual, state_skill_outcome,
};
use crate::gameserver::appserver::ai::aifactory::{ActiveMonsterAi, MonsterAiKind};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, end_move_shape_state, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

pub(crate) const CURE_SKILL_ID: u32 = 0x131;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;
const CONSTANT: u32 = 20_010;
const EM_MODIFIER: u32 = 20_015;
const PROBABILITY: u32 = 40_001;

fn participant(game: &CGame, value: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn publish_cure_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<Cure>(game, skill, mode);
}

fn failure(
    game: &mut CGame, address: RegisteredSkill, source: (i32, ShapeIdentity),
    mode: u32, text: &[u8], amount: Option<u32>,
) {
    game.update_registered_skill_visual(address, mode);
    if source.1.object_type == PLAYER_TYPE {
        if let Some(amount) = amount {
            game.send_skill_system_info_with_unsigned(source.1.id, text, amount);
        } else { game.send_skill_system_info(source.1.id, text); }
    }
}

fn cure_threshold(element_modify: i32, probability: u32, constant: u32, em_modifier: u32) -> i32 {
    let scaled = truncate_original(
        f64::from(em_modifier) * f64::from(0.01_f32) * f64::from(element_modify),
    );
    (scaled as u32).wrapping_mul(constant).wrapping_add(probability) as i32
}

fn cast_cure_states(game: &mut CGame, region_id: i32, target: ShapeIdentity, threshold: i32) {
    let mut index = 0;
    loop {
        let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return; };
        if index >= shape.state_slot_count() { break; }
        let selected = shape.state_at(index).is_some_and(|(_, state)| state.is_curable());
        if selected && game.skill_random_below(100) < threshold {
            let _ = end_and_destroy_state_at(game, region_id, target, index);
        }
        index += 1;
    }
}

pub(crate) fn finish_curable_state(
    game: &mut CGame, region_id: i32, target: ShapeIdentity, state_id: u32, _now_ms: u32,
) -> bool {
    let Some(shape) = resolve_state_move_shape(game, region_id, target) else { return false; };
    let key = (0..shape.state_slot_count()).find_map(|index| {
        let (key, state) = shape.state_at(index)?;
        (state.is_curable() && state.state_id() == state_id).then_some(key)
    });
    let Some(key) = key else { return false; };
    end_move_shape_state(game, region_id, target, key)
}

struct Cure;

impl RegisteredStateSkill for Cure {
    const ID: u32 = CURE_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Cure;
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(address) else { return false; };
        let Some(source) = participant(game, skill.lifecycle().user()) else { return false; };
        let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
        let Some(target) = begin_target.resolve(game, skill, true).and_then(|target| participant(game, target)) else {
            failure(game, address, source, 10, b"GS0286", None);
            return false;
        };
        if !skill_is_restored(skill.last_used_ms(), properties.query_property(REUSE), runtime.now_milliseconds()) {
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
                let name = game.base_magic_target_name(target.0, target.1).unwrap_or_default().to_vec();
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
            if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
                user.set_moveable(false);
            }
        }
        true
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let Some(skill) = game.registered_skill(address) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
            return end_state_skill(game, address, 0, runtime);
        };
        let source = participant(game, skill.lifecycle().user());
        let mut target = resolve_skill_sufferer(game, skill.lifecycle()).and_then(|target| participant(game, target));
        if let Some(value) = target {
            let redirect = value.1.object_type == 600
                && game.find_region(value.0).and_then(|region| region.base().find_monster_by_id(value.1.id))
                    .is_some_and(|monster| !monster.is_tamed() && !matches!(
                        monster.active_ai(), Some(ActiveMonsterAi::Carriage | ActiveMonsterAi::Primary(MonsterAiKind::Carriage)),
                    ));
            if redirect {
                target = source;
                if let Some(source) = source && let Some(skill) = game.registered_skill_mut(address) {
                    skill.lifecycle_mut().set_sufferer_identity(source.1);
                }
            }
        } else { target = source; }
        let (Some(source), Some(target)) = (source, target) else { return end_state_skill(game, address, 0, runtime); };
        if game.base_magic_target_dead(target.0, target.1) {
            failure(game, address, source, 10, b"GS0285", None);
            return end_state_skill(game, address, 0, runtime);
        }
        if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) == Some(SkillStage::Begin) {
            if source.1.object_type == PLAYER_TYPE {
                let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else { return end_state_skill(game, address, 0, runtime); };
                let remaining = mana.wrapping_sub(properties.query_property(MP_LOSS));
                if (remaining as i32) < 0 {
                    failure(game, address, source, 7, b"GS0288", Some(properties.query_property(MP_LOSS)));
                    return end_state_skill(game, address, 0, runtime);
                }
                if let Some(player) = game.find_player_mut(source.1.id) { player.set_mana(remaining); }
                let _ = game.publish_player_states(source.1.id);
            }
            if let Some(skill) = game.registered_skill_mut(address) {
                skill.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
            }
            let direction = (|| {
                let target = resolve_state_move_shape(game, target.0, target.1)?.shape();
                let target_y = target.get_tile_y().unwrap_or(i32::MIN);
                let target_x = target.get_tile_x().unwrap_or(i32::MIN);
                let source = resolve_state_move_shape(game, source.0, source.1)?.shape();
                let source_y = source.get_tile_y().unwrap_or(i32::MIN);
                let source_x = source.get_tile_x().unwrap_or(i32::MIN);
                Some(get_line_direction(source_x, source_y, target_x, target_y))
            })();
            if let Some(direction) = direction && let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
                user.shape_mut().set_direction(direction);
            }
            game.update_registered_skill_visual(address, 0);
            if let Some(skill) = game.registered_skill_mut(address) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
        }
        let Some(skill) = game.registered_skill(address) else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };
        if skill.execution_stage() != Some(SkillStage::Check) { return state_skill_outcome(QueuedSkillExecutionState::Pending); }
        let delay = properties.query_property(DELAY);
        let started = skill.lifecycle().started_at_ms();
        if runtime.now_milliseconds() < started.wrapping_add(delay) { return state_skill_outcome(QueuedSkillExecutionState::Pending); }
        game.update_registered_skill_visual(address, 1);
        let em_modifier = properties.query_property(EM_MODIFIER);
        let constant = properties.query_property(CONSTANT);
        let probability = properties.query_property(PROBABILITY);
        let element_modify = if source.1.object_type == PLAYER_TYPE {
            game.find_player(source.1.id).map_or(0, |player| player.combat_properties().element_modify)
        } else { 0 };
        if resolve_state_move_shape(game, source.0, source.1).is_some() {
            cast_cure_states(game, target.0, target.1, cure_threshold(element_modify, probability, constant, em_modifier));
        }
        let state = CureState::new(properties.query_property(PERSIST));
        let _ = begin_and_replace_cure_state(game, Some(source), Some(target), state, &mut || runtime.now_milliseconds());
        end_state_skill(game, address, 1, runtime)
    }
}

pub(crate) const fn is_cure_target(dispatch: PlayerSkillDispatch) -> bool { dispatch.skill_id() == CURE_SKILL_ID }

pub(crate) fn complete_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Cure, Runtime>(game, player_id, ai, 1, SkillTermination::Completed, runtime)
}

pub(crate) fn cancel_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Cure, Runtime>(game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime)
}

pub(crate) fn execute_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<Cure, Runtime>(game, player_id, dispatch, ai, runtime)
}

pub(crate) fn execute_owned_monster_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<Cure, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
