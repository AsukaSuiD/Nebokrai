//! Воодушевление CHearten (0x144), gameserver.exe/GameServer.pdb,
//! appserver/skills/hearten.cpp; состояние принадлежит heartenstate.cpp.
//!
//! Общий зарегистрированный Begin создаёт visual loop1 перед проверкой.
//! Объектный вызов проверяет исходную S; point/typed разрешает S с fallback U.
//! MP0 игрока — тихий отказ, положительная стоимость проверяется по знаку
//! DWORD-разности; монстр MP не проверяет и движение не блокирует.
//!
//! AI не имеет задержки: первый проход расходует MP, задаёт прерываемость,
//! направление и сразу накладывает состояние. NULL S временно заменяется U;
//! дикий монстр, не являющийся повозкой, дополнительно меняет сохранённые
//! type/id цели на U. После этой замены игрок может усилить только игрока.
//! Прежний первый state144 проходит End и destructor свежей той же позиции,
//! затем создаётся новый: Begin(U,S) → append → UpdateProperty цели → End(1).
//! Отказ и отмена используют полный End того же зарегистрированного экземпляра.

use super::heartenstate::{HeartenState, begin_primary_hearten_state};
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
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

pub(crate) use nebokrai_zone::effects::HEARTEN_STATE_ID as HEARTEN_SKILL_ID;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const MAX_HP_GAIN: u32 = 118;
const PERSIST: u32 = 10_002;
const MAX_DISTANCE: u32 = 5_003;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

fn participant(game: &CGame, value: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn publish_hearten_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<Hearten>(game, skill, mode);
}

fn failure(
    game: &mut CGame, address: RegisteredSkill, source: (i32, ShapeIdentity),
    mode: u32, text: &[u8], amount: Option<u32>,
) {
    game.update_registered_skill_visual(address, mode);
    if source.1.object_type == PLAYER_TYPE {
        if let Some(amount) = amount {
            game.send_skill_system_info_with_unsigned(source.1.id, text, amount);
        } else {
            game.send_skill_system_info(source.1.id, text);
        }
    }
}

struct Hearten;

impl RegisteredStateSkill for Hearten {
    const ID: u32 = HEARTEN_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Hearten;
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(address) else { return false; };
        let Some(source) = participant(game, skill.lifecycle().user()) else { return false; };
        let Some(target) = begin_target.resolve(game, skill, true) else { return false; };
        let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
        let reuse = properties.query_property(REUSE);
        if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
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
        let original_target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| participant(game, target));
        let redirect_wild = original_target.is_some_and(|(region, identity)| {
            identity.object_type == 600
                && game.find_region(region).and_then(|region| region.base().find_monster_by_id(identity.id))
                    .is_some_and(|monster| !monster.is_tamed() && !matches!(
                        monster.active_ai(),
                        Some(ActiveMonsterAi::Carriage | ActiveMonsterAi::Primary(MonsterAiKind::Carriage))
                    ))
        });
        let target = if original_target.is_none() || redirect_wild {
            let fallback = game.registered_skill(address)
                .and_then(|skill| participant(game, skill.lifecycle().user()));
            if redirect_wild
                && let Some((_, identity)) = fallback
                && let Some(skill) = game.registered_skill_mut(address)
            {
                skill.lifecycle_mut().set_sufferer_identity(identity);
            }
            fallback
        } else { original_target };
        let (Some(source), Some(target)) = (source, target) else {
            return end_state_skill(game, address, 0, runtime);
        };
        if game.move_shape_health(target.0, target.1) == Some(0) {
            game.update_registered_skill_visual(address, 10);
            return end_state_skill(game, address, 0, runtime);
        }
        if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) == Some(SkillStage::Begin) {
            if source.1.object_type == PLAYER_TYPE {
                if target.1.object_type != PLAYER_TYPE {
                    failure(game, address, source, 10, b"GS0306", None);
                    return end_state_skill(game, address, 0, runtime);
                }
                let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else {
                    return end_state_skill(game, address, 0, runtime);
                };
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
            if source != target {
                let direction = (|| {
                    let user = resolve_state_move_shape(game, source.0, source.1)?.shape();
                    let target = resolve_state_move_shape(game, target.0, target.1)?.shape();
                    Some(get_line_direction(
                        user.get_tile_x().unwrap_or(i32::MIN), user.get_tile_y().unwrap_or(i32::MIN),
                        target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN),
                    ))
                })();
                if let Some(direction) = direction
                    && let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1)
                { user.shape_mut().set_direction(direction); }
            }
            game.update_registered_skill_visual(address, 0);
            if let Some(skill) = game.registered_skill_mut(address) {
                let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
            }
        }
        if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) != Some(SkillStage::Check) {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        game.update_registered_skill_visual(address, 1);
        if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
            .and_then(|shape| shape.find_state_position(|state| state.state_id() == HEARTEN_SKILL_ID))
        {
            let _ = end_and_destroy_state_at(game, target.0, target.1, position);
        }
        let gain = properties.query_property(MAX_HP_GAIN) as i32;
        let keep = properties.query_property(PERSIST);
        let state = HeartenState::new(0, keep, gain);
        let _ = begin_primary_hearten_state(
            game, Some(source), target, state, &mut || runtime.now_milliseconds(),
        );
        let _ = game.update_move_shape_properties(target.0, target.1);
        end_state_skill(game, address, 1, runtime)
    }
}

pub(crate) fn cancel_player_hearten<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Hearten, Runtime>(
        game, player_id, player_ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_hearten<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<Hearten, Runtime>(game, player_id, dispatch, player_ai, runtime)
}

pub(crate) fn execute_owned_monster_hearten<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<Hearten, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
