//! Общий прямой снаряд `CChuckStone` (`0x19D`) и `CSkeletonArchery`
//! (`0x1A1`). Источник: точная пара `gameserver.exe + GameServer.pdb`,
//! исходные владельцы `chuckstone.cpp` и `skeletonarchery.cpp`. Тела Check/AI
//! и helpers перенесены буквально в Zone `skills/directprojectile.rs`
//! (кластер B полосы Monster 0x19x; основание и машинные статусы см. там).
//! Здесь — объявленные швы переноса: фасадные реализации трейтов Zone над
//! прежними методами `CGame`/`CMoveShape` и тонкие делегации с прежними
//! сигнатурами; зарегистрированный вход идёт общим playercast, удар — прежний
//! weaponattack, семейный End — прежний stateskill. Потребители не меняются.
//!
//! `DirectProjectileProgress` заменяет поля C++-объекта и хранится в том же
//! зарегистрированном экземпляре игрока или монстра (Zone `skills/execution`).
//! Вектор и SlotMap реестра заменяют временные C++-контейнеры и указатели без
//! копии.

use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
    state_skill_outcome,
};
use super::weaponattack::apply_direct_projectile_attack;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
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
    RegionShapeResolver, ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::SkillLifecycle;
use nebokrai_zone::skills::directprojectile::{
    self, DirectProjectileContact, DirectProjectileGame, DirectProjectileMoveShape,
    DirectProjectileOutcome,
};
use nebokrai_zone::skills::execution::RegisteredSkillRecord;

impl DirectProjectileMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }
    fn shape_mut(&mut self) -> &mut CShape { self.shape_mut() }
    fn set_moveable(&mut self, moveable: bool) { self.set_moveable(moveable) }
}

impl DirectProjectileGame for CGame {
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

    fn player_weapon_addon_category(&self, player_id: i32) -> Option<i32> {
        self.find_player(player_id)
            .and_then(|player| player.equipment().get_goods(2))
            .map(|weapon| weapon.addon_property_value(self.goods_factory(), GAP_WEAPON_CATEGORY, 1))
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

    fn direct_projectile_shape_view_at(&self, region_id: i32, x: i32, y: i32) -> Option<ShapeView> {
        let (area_width, area_height) = self.area_dimensions();
        let owner = self.find_region(region_id)?;
        let resolver = RegionShapeResolver { game: self, owner };
        owner.base().get_shape(x, y, area_width, area_height, &resolver).ok().flatten()
    }

    fn direct_projectile_cell_views(&self, region_id: i32, x: i32, y: i32) -> Vec<ShapeView> {
        let Some(owner) = self.find_region(region_id) else { return Vec::new(); };
        let resolver = RegionShapeResolver { game: self, owner };
        let (area_width, area_height) = self.area_dimensions();
        let mut views = Vec::new();
        if owner.base().get_shapes(x, y, area_width, area_height, &resolver, &mut views).is_err() { return Vec::new(); }
        views
    }
}

impl<Runtime: GameMainLoopRuntime> DirectProjectileContact<Runtime> for CGame {
    fn apply_direct_projectile_attack(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) {
        apply_direct_projectile_attack(self, address, source, target, runtime);
    }

    fn end_direct_projectile_instance(&mut self, address: RegisteredSkill, runtime: &mut Runtime) {
        let _ = end_state_skill(self, address, 1, runtime);
    }
}

pub(crate) fn execute_player_direct_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game,
        player_id,
        instance,
        dispatch,
        runtime,
        SkillVisualEffectKind::DirectProjectile,
        |game, instance, _, _| directprojectile::check_cast(game, instance, original_user),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_direct_projectile_ai,
    )
}

fn run_direct_projectile_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let outcome = directprojectile::run_ai(game, instance, runtime, game_tick_milliseconds);
    match outcome {
        DirectProjectileOutcome::Rejected => end_state_skill(game, instance, 0, runtime),
        DirectProjectileOutcome::Completed => end_state_skill(game, instance, 1, runtime),
        DirectProjectileOutcome::Pending => state_skill_outcome(QueuedSkillExecutionState::Pending),
    }
}

struct DirectProjectileSkill<const ID: u32>;

impl<const ID: u32> RegisteredStateSkill for DirectProjectileSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::DirectProjectile;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        _target: StateSkillBeginTarget,
        _runtime: &mut Runtime,
    ) -> bool {
        let source = game.registered_skill(instance).and_then(|skill| {
            let (region, identity) = skill.lifecycle().user();
            resolve_state_move_shape(game, region, identity)
                .map(|source| (source.shape().get_region_id(), source.shape().identity()))
        });
        directprojectile::check_cast(game, instance, source)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        run_direct_projectile_ai(game, instance, runtime)
    }
}

pub(crate) fn execute_owned_monster_direct_projectile<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<DirectProjectileSkill<ID>, Runtime>(
        game,
        owner,
        monster_id,
        target,
        skill_level,
        runtime,
    )
}
