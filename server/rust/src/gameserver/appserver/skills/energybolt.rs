//! Пошаговые снаряды `CEnergyBolt` (`0x1A0`), `CSnakeBolt` (`0x1A5`) и
//! `CZombieClaw` (`0x1A2`). Источник: точная пара `gameserver.exe +
//! GameServer.pdb`, исходные владельцы `energybolt.cpp`, `snakebolt.cpp` и
//! `zombieclaw.cpp`. Тела Check/AI и helpers перенесены буквально в Zone
//! `skills/pathprojectile.rs` (кластер B полосы Monster 0x19x; основание и
//! машинные статусы см. там). Здесь — объявленные швы переноса: фасадные
//! реализации трейтов Zone над прежними методами `CGame`/`CMoveShape` и
//! тонкие делегации с прежними сигнатурами; зарегистрированный вход идёт
//! общим playercast, MP-контракт — прежний rangedweaponcast, стихийный удар
//! — прежний directelementattack, семейный End — прежний stateskill.
//! Потребители не меняются.

use super::directelementattack::apply_direct_element_attack;
use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::spend_cast_mana_without_text;
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
    state_skill_outcome,
};
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::SkillLifecycle;
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::pathprojectile::{
    self, PathProjectileContact, PathProjectileGame, PathProjectileMoveShape,
    PathProjectileOutcome,
};

pub(crate) use nebokrai_zone::skills::pathprojectile::ENERGY_BOLT_SKILL_ID;

impl PathProjectileMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }
    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }
    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }
}

impl PathProjectileGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type MoveShape = CMoveShape;

    fn registered_skill(
        &self,
        address: RegisteredSkill,
    ) -> Option<&RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill(address)
    }

    fn registered_skill_mut(
        &mut self,
        address: RegisteredSkill,
    ) -> Option<&mut RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill_mut(address)
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        self.update_registered_skill_visual(address, mode)
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn resolve_skill_sufferer(&self, lifecycle: &SkillLifecycle) -> Option<(i32, ShapeIdentity)> {
        resolve_skill_sufferer(self, lifecycle)
    }

    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&CMoveShape> {
        resolve_state_move_shape(self, region_id, identity)
    }

    fn resolve_state_move_shape_mut(
        &mut self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&mut CMoveShape> {
        resolve_state_move_shape_mut(self, region_id, identity)
    }

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }

    fn skill_target_path_with_length(
        &self,
        lifecycle: &SkillLifecycle,
        maximum: u32,
    ) -> Vec<(i32, i32, u8)> {
        self.skill_target_path_with_length(lifecycle, maximum)
    }

    fn player_mana(&self, player_id: i32) -> Option<u32> {
        self.find_player(player_id).map(|player| player.mana())
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn send_skill_system_info_with_unsigned(&self, player_id: i32, text: &[u8], amount: u32) {
        self.send_skill_system_info_with_unsigned(player_id, text, amount)
    }

    fn move_shape_health(&self, region_id: i32, identity: ShapeIdentity) -> Option<u32> {
        self.move_shape_health(region_id, identity)
    }

    fn live_skill_target_attackable_between(
        &self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
    ) -> bool {
        self.live_skill_target_attackable_between(source, target)
    }

    fn path_projectile_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView> {
        super::flash::cell_views(self, region_id, x, y)
    }

    fn path_projectile_region_exists(&self, region_id: i32) -> bool {
        self.find_region(region_id).is_some()
    }

    fn path_projectile_skill_cell_block(&self, region_id: i32, x: i32, y: i32) -> Option<u8> {
        self.find_region(region_id).map(|owner| owner.base().skill_cell_block(x, y))
    }
}

impl<Runtime: GameMainLoopRuntime> PathProjectileContact<Runtime> for CGame {
    fn spend_path_projectile_cast_mana(
        &mut self,
        address: RegisteredSkill,
        player: Option<i32>,
        properties: &CSkillBaseProperties,
    ) -> bool {
        spend_cast_mana_without_text(self, address, player, properties)
    }

    fn apply_direct_element_attack(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) {
        apply_direct_element_attack(self, address, source, target, runtime);
    }
}

pub(crate) fn execute_player_path_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let begin_target = match dispatch {
        PlayerSkillDispatch::Point { .. } => StateSkillBeginTarget::Resolved,
        _ => StateSkillBeginTarget::Object(original_user.and_then(|(region, _)| {
            dispatch.object_target().and_then(|target| game.player_skill_begin_object(region, target))
        })),
    };
    execute_registered_player_cast(
        game,
        player_id,
        instance,
        dispatch,
        runtime,
        SkillVisualEffectKind::PathProjectile,
        |game, instance, _, _runtime| {
            let target = game.registered_skill(instance)
                .and_then(|skill| begin_target.resolve(game, skill, false));
            pathprojectile::check_cast(game, instance, original_user, target, game_tick_milliseconds)
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_path_projectile_ai,
    )
}

fn run_path_projectile_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let outcome = pathprojectile::run_ai(game, instance, runtime, game_tick_milliseconds);
    match outcome {
        PathProjectileOutcome::Rejected => end_state_skill(game, instance, 0, runtime),
        PathProjectileOutcome::Completed => end_state_skill(game, instance, 1, runtime),
        PathProjectileOutcome::Pending => state_skill_outcome(QueuedSkillExecutionState::Pending),
    }
}

struct PathProjectileSkill<const ID: u32>;

impl<const ID: u32> RegisteredStateSkill for PathProjectileSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::PathProjectile;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        target: StateSkillBeginTarget,
        _runtime: &mut Runtime,
    ) -> bool {
        let source = game.registered_skill(instance).and_then(|skill| {
            let (region, identity) = skill.lifecycle().user();
            resolve_state_move_shape(game, region, identity)
                .map(|source| (source.shape().get_region_id(), source.shape().identity()))
        });
        let target = game.registered_skill(instance)
            .and_then(|skill| target.resolve(game, skill, false));
        pathprojectile::check_cast(game, instance, source, target, game_tick_milliseconds)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        run_path_projectile_ai(game, instance, runtime)
    }
}

pub(crate) fn execute_owned_path_projectile<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<PathProjectileSkill<ID>, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}

pub(crate) fn execute_owned_monster_energy_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_path_projectile::<ENERGY_BOLT_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
