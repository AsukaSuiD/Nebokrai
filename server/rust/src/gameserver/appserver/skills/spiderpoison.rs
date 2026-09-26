//! Тонкий путь к ядовитому удару `CSpiderPoison` (`0x191`) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/spiderpoison.cpp` (payload — spiderpoisonstate.cpp,
//! тонкий alias `spiderpoisonstate.rs` не изменён). Собственные
//! CheckCastCondition, AI, прямой удар и позднее наложение яда перенесены
//! буквально в `nebokrai_zone::skills::spiderpoison` (основание и статусы
//! MATCH см. там; кластер D полосы Monster 0x19x).
//! Здесь — объявленные швы переноса: hub-реализации `SpiderPoisonGame`
//! (`GetTargetPath`), `SpiderPoisonStateArena` (Cure-факт и семейная замена
//! первого состояния 0x191 через `states/state.rs` + primary Begin общего
//! яда) и `SpiderPoisonMoveShape` (SetMoveable) над прежними
//! `CGame`/`CMoveShape`. Общая обвязка `stateskill` обслуживает Begin/End
//! игрока и монстра как раньше; делегации сохраняют прежние сигнатуры —
//! потребители (`states/skill.rs`, `monsterbaseattack.rs`, `game.rs`,
//! `heartlessarrow.rs`, `spriteburn.rs`, `monster.rs`) не меняются.

use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
    execute_player_state_skill, finish_player_state_skill, publish_state_skill_visual,
    state_skill_outcome,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapeSkill};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillTermination;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::SkillLifecycle;
use nebokrai_zone::skills::spiderpoison::{
    self as zone, SpiderPoisonBeginTarget, SpiderPoisonGame, SpiderPoisonMoveShape,
    SpiderPoisonStateArena,
};
use nebokrai_zone::skills::statefactory::SpiderPoisonState;

pub(crate) use nebokrai_zone::skills::spiderpoison::SPIDER_POISON_SKILL_ID;

impl SpiderPoisonMoveShape for CMoveShape {
    fn set_moveable(&mut self, moveable: bool) {
        CMoveShape::set_moveable(&mut *self, moveable);
    }
}

impl SpiderPoisonStateArena for CGame {
    fn poison_shape_has_state(
        &self,
        region_id: i32,
        identity: ShapeIdentity,
        skill_id: u32,
    ) -> Option<bool> {
        resolve_state_move_shape(self, region_id, identity)
            .map(|shape| shape.has_state_by_skill_id(skill_id))
    }

    fn replace_or_begin_spider_poison_state(
        &mut self,
        region_id: i32,
        holder: ShapeIdentity,
        user: Option<(i32, ShapeIdentity)>,
        sufferer: Option<(i32, ShapeIdentity)>,
        state: SpiderPoisonState,
        now: &mut dyn FnMut() -> u32,
    ) -> bool {
        // Порядок прежнего hub `states/state.rs`: первый слот 0x191 → End →
        // destructor остатка → Begin в тот же слот; без совпадения — append.
        // Слот найден, а локализация офсета нет — abort до End (ветвь
        // прежнего `corpseptomaine`; дубликат состояния не создаётся).
        let previous = resolve_state_move_shape(self, region_id, holder)
            .and_then(|shape| shape.find_state_position(|state| state.state_id() == SPIDER_POISON_SKILL_ID));
        let placement = previous.and_then(|(_, key)| {
            resolve_state_move_shape(self, region_id, holder)?.applied_state_replacement_location(key)
        });
        if previous.is_some() && placement.is_none() {
            return false;
        }
        if let Some((position, _)) = previous {
            let _ = end_and_destroy_state_at(self, region_id, holder, position);
        }
        super::spiderpoisonstate::begin_primary_spider_poison_state(
            self,
            region_id,
            holder,
            user,
            sufferer,
            state,
            placement,
            now,
        )
        .is_some()
    }
}

impl SpiderPoisonGame for CGame {
    fn skill_target_path(&self, lifecycle: &SkillLifecycle) -> Vec<(i32, i32, u8)> {
        self.skill_target_path(lifecycle)
    }
}

struct SpiderPoison;

impl RegisteredStateSkill for SpiderPoison {
    const ID: u32 = SPIDER_POISON_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::SpiderPoison;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 8, 10, 11, 13, 14, 15];

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        zone::check_spider_poison_cast(game, instance, begin_target.into(), runtime.now_milliseconds())
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        match zone::execute_spider_poison_ai(game, instance, runtime, game_tick_milliseconds) {
            Some(argument) => end_state_skill(game, instance, argument, runtime),
            None => state_skill_outcome(QueuedSkillExecutionState::Pending),
        }
    }
}

impl From<StateSkillBeginTarget> for SpiderPoisonBeginTarget {
    fn from(begin_target: StateSkillBeginTarget) -> Self {
        match begin_target {
            StateSkillBeginTarget::Object(target) => Self::Object(target),
            StateSkillBeginTarget::Resolved => Self::Resolved,
        }
    }
}

pub(crate) const fn is_player_spider_poison_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_player_spider_poison_dispatch(dispatch)
}

pub(crate) fn publish_spider_poison_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<SpiderPoison>(game, skill, mode);
}

pub(crate) fn execute_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<SpiderPoison, Runtime>(game, player_id, dispatch, ai, runtime)
}

pub(crate) fn cancel_player_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<SpiderPoison, Runtime>(
        game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_owned_spider_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<SpiderPoison, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
