//! Печать `CSeal` (`0x138`) для Player и Monster.
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/seal.cpp`.
//!
//! Общий TargetedProjectile хранит зарегистрированные U/S, первую проверку,
//! unsigned delay и flight, prepared-background и хвост Attack End. Этот owner
//! оставляет только подтверждённые различия Seal: Check принимает лишь S типа
//! CMonster; только Player требует MP и Move0. AI повторно проверяет
//! DoesTargetEffective(U) перед проверкой смерти S. После первого срока Move1 предшествует
//! проверке уровней и свежего пути; невозможный уровень завершает End(1),
//! остальные отказы — End(0). Выпуск visual1 предшествует prepared.
//! Общий путь берёт точку фигуры S по сохранённому user_region Begin.
//!
//! Impact пропускается при Cure. Иначе общий DirectElement-профиль сохраняет
//! generic MasterInfo, Player-only EM, единственный RNG и расширенную
//! EM/FISTP-прибавку с f32-константой. После Attack уровни читаются S→U даже
//! после фатального попадания; новый SealState создаётся до полного
//! End/destructor прежнего ID и занимает его прежнее место. State Begin
//! устанавливает timestamp и visual, поэтому ctor получает только keep-time.

use super::basemagic::SKILL_USAGE_REUSE_DELAY_TIME;
use super::blindstate::replace_primary_blind_state;
use super::directelementattack::apply_direct_element_attack;
use super::kernel::skill_is_restored;
use super::rangedweaponcast::{CastPathBlock, check_cast_mana, check_skill_path};
use super::sealstate::SealState;
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::targetedprojectile::{TargetedProjectileProgress, run_targeted_projectile_ai};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};

pub(crate) const SEAL_SKILL_ID: u32 = 0x138;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const CURE_SKILL_ID: u32 = 0x131;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_CONST: u32 = 20_010;

fn resolved_object(game: &CGame, object: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, object.0, object.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(super) fn check_seal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool {
    let Some(source) = original_user.and_then(|source| resolved_object(game, source)) else {
        return false;
    };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some((skill_id, skill_level, last_used)) = game.registered_skill(instance).map(|skill| {
        (skill.id(), skill.level(), skill.last_used_ms())
    }) else {
        return false;
    };
    let Some(properties) = game.skill_base_properties(skill_id, skill_level).cloned() else {
        return false;
    };
    let target = original_target.and_then(|target| resolved_object(game, target));
    let Some(target) = target.filter(|target| target.1.object_type == MONSTER_TYPE) else {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player {
            game.send_skill_system_info(player, b"GS0317");
        }
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(last_used, reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player {
            game.send_skill_system_info(player, b"GS0278");
        }
        return false;
    }
    let Some(lifecycle) = game.registered_skill(instance).map(|skill| *skill.lifecycle()) else {
        return false;
    };
    let path = game.skill_target_path(&lifecycle);
    if !check_skill_path(
        game,
        instance,
        &properties,
        &path,
        player,
        CastPathBlock::Named {
            target,
            message: b"GS0295",
        },
    ) {
        return false;
    }
    check_cast_mana(game, instance, source, &properties)
}

pub(super) fn apply_seal_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) {
    let target_has_cure = resolve_state_move_shape(game, target.0, target.1)
        .is_some_and(|target| target.has_state_by_skill_id(CURE_SKILL_ID));
    if target_has_cure {
        return;
    }
    // Native Attack гасит только raw contact при U == S; state-tail остаётся.
    apply_direct_element_attack(game, instance, source, target, runtime);

    // Native читает S, затем U после OnBeenAttacked; смерть S не отменяет SealState.
    let Some(target_level) = game.move_shape_level(target.0, target.1) else {
        return;
    };
    let Some(source_level) = game.move_shape_level(source.0, source.1) else {
        return;
    };
    let multiplier = i32::from(source_level)
        .wrapping_sub(i32::from(target_level))
        .wrapping_add(properties.query_property(SKILL_USAGE_CONST) as i32)
        .max(1);
    let keep = properties
        .query_property(SKILL_USAGE_STATE_PERSIST_TIME)
        .wrapping_mul(multiplier as u32);
    let state = SealState::new(keep);
    let _ = replace_primary_blind_state(
        game,
        source,
        target,
        state,
        &mut || runtime.now_milliseconds(),
    );
}

struct SealSkill;

impl RegisteredStateSkill for SealSkill {
    const ID: u32 = SEAL_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::TargetedProjectile;

    fn prepare_monster(skill: &mut MoveShapeSkill) {
        skill.set_monster_progress(TargetedProjectileProgress::default());
    }

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let source = game
            .registered_skill(instance)
            .and_then(|skill| resolved_object(game, skill.lifecycle().user()));
        let target = game
            .registered_skill(instance)
            .and_then(|skill| begin_target.resolve(game, skill, false));
        check_seal_cast(game, instance, source, target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_targeted_projectile_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
                end_state_skill(game, instance, 1, runtime)
            }
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_seal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<SealSkill, Runtime>(
        game,
        owner,
        monster_id,
        target,
        skill_level,
        runtime,
    )
}
