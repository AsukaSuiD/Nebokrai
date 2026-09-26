//! Правила и исполнение CHearten (0x144) — воодушевление.
//! Источник: gameserver.exe + GameServer.pdb (точная пара `4F5C98E0…` +
//! RSDS match), `appserver/skills/hearten.cpp/.h`; состояние принадлежит
//! `heartenstate.cpp/.h`. Машинные якоря семьи: CHearten Begin `0x150F30`,
//! AI `0x151950`; состояние — общий AI-fold `0x1D5BA0`. Тела Check/AI
//! перенесены буквально.
//!
//! Общий зарегистрированный Begin создаёт visual loop1 перед проверкой.
//! Объектный вызов проверяет исходную S; point/typed разрешает S с
//! fallback U. MP0 игрока — тихий отказ, положительная стоимость
//! проверяется по знаку DWORD-разности; монстр MP не проверяет и движение
//! не блокирует.
//!
//! AI не имеет задержки: первый проход расходует MP, задаёт прерываемость,
//! направление и сразу накладывает состояние. NULL S временно заменяется
//! U; дикий монстр, не являющийся повозкой, дополнительно меняет
//! сохранённые type/id цели на U. После этой замены игрок может усилить
//! только игрока. Прежний первый state144 проходит End и destructor свежей
//! той же позиции, затем создаётся новый: Begin(U,S) → append →
//! UpdateProperty цели → End(1). Отказ и отмена используют полный End того
//! же зарегистрированного экземпляра.
//!
//! Объявленные швы переноса (не расхождения): hub `statecast::*`
//! реализован у владельца старого пакета; разрешение `begin_target`,
//! материализация и полный End остаются в `stateskill.rs` делегата.

use nebokrai_shared::runtime::get_line_direction;

use crate::effects::{HEARTEN_STATE_ID, HeartenState};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::{MONSTER_TYPE, PLAYER_TYPE};

use super::heartenstate::begin_primary_hearten_state;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastMoveShape, StateCastPlayer,
    StateCastVisualContract, StateCastVisualTarget, publish_state_cast_visual, state_cast_participant,
};
use super::visualeffect::SkillVisualEffectKind;

pub const HEARTEN_SKILL_ID: u32 = HEARTEN_STATE_ID;
const MAX_HP_GAIN: u32 = 118;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const PERSIST: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

/// Снимок параметров нового состояния; вызывать после удаления прежнего
/// состояния: прибавка читается перед сроком (CHearten::AI
/// `0x00551CAE–0x00551CC7` той же пары).
pub fn hearten_state(mut query_property: impl FnMut(u32) -> u32) -> HeartenState {
    let gain = query_property(MAX_HP_GAIN) as i32;
    let keep_time_ms = query_property(PERSIST);
    HeartenState::new(0, keep_time_ms, gain)
}

static HEARTEN_VISUAL: StateCastVisualContract = StateCastVisualContract {
    skill_id: HEARTEN_SKILL_ID,
    kind: SkillVisualEffectKind::Hearten,
    failures: &[2, 7, 10, 11, 13, 15],
    target: StateCastVisualTarget::SuffererOrUser,
};

pub fn publish_hearten_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    publish_state_cast_visual(game, skill, mode, &HEARTEN_VISUAL);
}

fn failure<Game: StateCastGame>(
    game: &mut Game, address: Game::SkillAddress, source: (i32, ShapeIdentity),
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

pub fn check_cast<Game: StateCastGame>(
    game: &mut Game,
    address: Game::SkillAddress,
    begin_target: Option<(i32, ShapeIdentity)>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(skill) = game.registered_skill(address) else { return false; };
    let Some(source) = state_cast_participant(game, skill.lifecycle().user()) else { return false; };
    let Some(target) = begin_target else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, now()) {
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
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.set_moveable(false);
        }
    }
    true
}

pub fn run_hearten_ai<Game: StateCastGame>(
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
    let original_target = game.resolve_skill_sufferer(skill.lifecycle())
        .and_then(|target| state_cast_participant(game, target));
    let redirect_wild = original_target.is_some_and(|(region, identity)| {
        identity.object_type == MONSTER_TYPE
            && game.wild_untamed_non_carriage_monster(region, identity.id)
    });
    let target = if original_target.is_none() || redirect_wild {
        let fallback = game.registered_skill(address)
            .and_then(|skill| state_cast_participant(game, skill.lifecycle().user()));
        if redirect_wild
            && let Some((_, identity)) = fallback
            && let Some(skill) = game.registered_skill_mut(address)
        {
            skill.lifecycle_mut().set_sufferer_identity(identity);
        }
        fallback
    } else { original_target };
    let (Some(source), Some(target)) = (source, target) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(address, 10);
        return StateCastExecutionOutcome::EndRejected;
    }
    if game.registered_skill(address).and_then(|skill| skill.execution_stage()) == Some(SkillStage::Begin) {
        if source.1.object_type == PLAYER_TYPE {
            if target.1.object_type != PLAYER_TYPE {
                failure(game, address, source, 10, b"GS0306", None);
                return StateCastExecutionOutcome::EndRejected;
            }
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
        if source != target {
            let direction = (|| {
                let user = game.resolve_state_move_shape(source.0, source.1)?.shape();
                let target = game.resolve_state_move_shape(target.0, target.1)?.shape();
                Some(get_line_direction(
                    user.get_tile_x().unwrap_or(i32::MIN), user.get_tile_y().unwrap_or(i32::MIN),
                    target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN),
                ))
            })();
            if let Some(direction) = direction
                && let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1)
            { user.shape_mut().set_direction(direction); }
        }
        game.update_registered_skill_visual(address, 0);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(address).and_then(|skill| skill.execution_stage()) != Some(SkillStage::Check) {
        return StateCastExecutionOutcome::Pending;
    }
    game.update_registered_skill_visual(address, 1);
    if let Some((position, _)) = game.resolve_state_move_shape(target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == HEARTEN_SKILL_ID))
    {
        let _ = game.end_and_destroy_state_at(target.0, target.1, position);
    }
    let state = hearten_state(|key| properties.query_property(key));
    let _ = begin_primary_hearten_state(game, Some(source), target, state, now);
    game.update_move_shape_properties(target.0, target.1);
    StateCastExecutionOutcome::EndCompleted
}
