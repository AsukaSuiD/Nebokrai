//! Печать `CSeal` (`0x138`) для Player и Monster. Источник: точная пара
//! `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/seal.cpp`. Тела Check/Attack, гейты и формула keep-time
//! перенесены буквально в Zone `skills/seal.rs` (кластер B полосы Monster
//! 0x19x; основание и машинные статусы см. там). Здесь — объявленные швы
//! переноса: фасадные реализации трейтов Zone над прежними методами
//! `CGame`/`CMoveShape` и тонкие делегации с прежними сигнатурами; полёт и
//! зарегистрированный цикл — прежний targetedprojectile/stateskill, проверка
//! пути/MP — прежний rangedweaponcast, стихийный удар — прежний
//! directelementattack, замена состояния — прежний blindstate. Потребители
//! не меняются.

use super::blindstate::replace_primary_blind_state;
use super::directelementattack::apply_direct_element_attack;
use super::rangedweaponcast::{CastPathBlock, check_cast_mana, check_skill_path};
use super::sealstate::SealState;
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::targetedprojectile::{TargetedProjectileProgress, run_targeted_projectile_ai};
use crate::gameserver::appserver::monster::MonsterSkillExecution;
use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapeSkill};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::SkillLifecycle;
use nebokrai_zone::skills::execution::RegisteredSkillRecord;
use nebokrai_zone::skills::seal::{self, SealContact, SealGame, SealMoveShape};

pub(crate) use nebokrai_zone::skills::seal::SEAL_SKILL_ID;

impl SealMoveShape for CMoveShape {
    fn shape(&self) -> &CShape { self.shape() }

    fn has_state_by_skill_id(&self, skill_id: u32) -> bool { self.has_state_by_skill_id(skill_id) }
}

impl SealGame for CGame {
    type MonsterExecution = MonsterSkillExecution;
    type SkillAddress = RegisteredSkill;
    type MoveShape = CMoveShape;

    fn registered_skill(
        &self,
        address: RegisteredSkill,
    ) -> Option<&RegisteredSkillRecord<MonsterSkillExecution>> {
        self.registered_skill(address)
    }

    fn update_registered_skill_visual(&mut self, address: RegisteredSkill, mode: u32) {
        self.update_registered_skill_visual(address, mode)
    }

    fn skill_base_properties(&self, skill_id: u32, level: i32) -> Option<&CSkillBaseProperties> {
        self.skill_base_properties(skill_id, level)
    }

    fn resolve_state_move_shape(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
    ) -> Option<&CMoveShape> {
        resolve_state_move_shape(self, region_id, identity)
    }

    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }

    fn send_skill_system_info(&self, player_id: i32, text: &[u8]) {
        self.send_skill_system_info(player_id, text)
    }

    fn move_shape_level(&self, region_id: i32, identity: ShapeIdentity) -> Option<u8> {
        self.move_shape_level(region_id, identity)
    }

    fn check_seal_path(
        &mut self,
        address: RegisteredSkill,
        properties: &CSkillBaseProperties,
        path: &[(i32, i32, u8)],
        player: Option<i32>,
        target: (i32, ShapeIdentity),
    ) -> bool {
        check_skill_path(
            self,
            address,
            properties,
            path,
            player,
            CastPathBlock::Named { target, message: b"GS0295" },
        )
    }

    fn check_seal_cast_mana(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        properties: &CSkillBaseProperties,
    ) -> bool {
        check_cast_mana(self, address, source, properties)
    }
}

impl<Runtime: GameMainLoopRuntime> SealContact<Runtime> for CGame {
    fn apply_seal_direct_attack(
        &mut self,
        address: RegisteredSkill,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        runtime: &mut Runtime,
    ) {
        apply_direct_element_attack(self, address, source, target, runtime);
    }

    fn replace_seal_state(
        &mut self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        keep_time_ms: u32,
        runtime: &mut Runtime,
    ) {
        let state = SealState::new(keep_time_ms);
        let _ = replace_primary_blind_state(
            self,
            source,
            target,
            state,
            &mut || runtime.now_milliseconds(),
        );
    }
}

pub(super) fn check_seal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>,
    _runtime: &mut Runtime,
) -> bool {
    seal::check_cast(game, instance, original_user, original_target, game_tick_milliseconds)
}

pub(super) fn apply_seal_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) {
    seal::apply_attack(game, instance, source, target, properties, runtime);
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
        _runtime: &mut Runtime,
    ) -> bool {
        let source = game.registered_skill(instance).and_then(|skill| {
            let (region, identity) = skill.lifecycle().user();
            resolve_state_move_shape(game, region, identity)
                .map(|source| (source.shape().get_region_id(), source.shape().identity()))
        });
        let target = game.registered_skill(instance)
            .and_then(|skill| begin_target.resolve(game, skill, false));
        seal::check_cast(game, instance, source, target, game_tick_milliseconds)
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
