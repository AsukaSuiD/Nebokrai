//! Тонкий путь к отравленной стреле боевого духа CPoisonArrow (0x21E) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/poisonarrow.cpp`. Собственные Check/AI, общая с
//! CBloodLoss обёртка `execute_periodic_battle_fairy_arrow` и позднее
//! наложение яда перенесены буквально в `nebokrai_zone::skills::poisonarrow`
//! (основание и статусы MATCH — в шапке Zone-файла; кластер D, порция D5).
//! Здесь — объявленные швы переноса: hub-реализации `PoisonArrowStateArena`
//! (замена первого слота ID через `states/state.rs` и primary Begin прежнего
//! hub `states/poison.rs`, как у соседнего `SpiderPoisonStateArena`) и
//! `PoisonArrowContact` (PK `player_on_first_skill_at_position`) над прежним
//! `CGame`; делегации сохраняют прежние сигнатуры — потребители
//! (`bloodloss.rs`, `poisonarrowstate.rs`, `game.rs`) не меняются.
//! Координаторский End-контракт и BF918-доставка (решение C) остаются hub
//! порции №6b.

use super::battlefairyskill::battle_fairy_outcome;
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use nebokrai_zone::skills::poisonarrow::{
    self as zone, PoisonArrowContact, PoisonArrowState, PoisonArrowStateArena,
};
use nebokrai_zone::skills::state::StateKey;

pub(crate) use nebokrai_zone::skills::poisonarrow::POISON_ARROW_SKILL_ID;
pub(super) use nebokrai_zone::skills::poisonarrow::ArrowEffect;

impl PoisonArrowStateArena for CGame {
    fn arrow_state_replacement_slot(
        &mut self,
        target: (i32, ShapeIdentity),
        state_id: u32,
    ) -> Option<Option<(usize, usize)>> {
        let shape = resolve_state_move_shape(self, target.0, target.1)?;
        if let Some((position, key)) = shape.find_state_position(|state| state.state_id() == state_id) {
            let location = shape.applied_state_replacement_location(key)?;
            let _ = end_and_destroy_state_at(self, target.0, target.1, position);
            Some(Some(location))
        } else {
            Some(None)
        }
    }

    fn begin_primary_poison_arrow_state(
        &mut self,
        holder_region: i32,
        holder: ShapeIdentity,
        user: Option<(i32, ShapeIdentity)>,
        sufferer: Option<(i32, ShapeIdentity)>,
        state: PoisonArrowState,
        placement: Option<(usize, usize)>,
        now: &mut dyn FnMut() -> u32,
    ) -> Option<StateKey> {
        super::poisonarrowstate::begin_primary_poison_arrow_state(
            self, holder_region, holder, user, sufferer, state, placement, now,
        )
    }
}

impl<Runtime: GameMainLoopRuntime> PoisonArrowContact<Runtime> for CGame {
    fn poison_arrow_first_skill_at_position(
        &mut self,
        attacker_id: i32,
        victim_id: i32,
        region_id: i32,
        x: i32,
        y: i32,
        runtime: &mut Runtime,
    ) {
        let _ = self.player_on_first_skill_at_position(
            attacker_id, victim_id, region_id, x, y, runtime,
        );
    }
}

pub(super) fn execute_periodic_battle_fairy_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime, obstacle_string: &[u8],
    apply: impl FnOnce(&mut CGame, ArrowEffect, &mut Runtime),
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(zone::execute_periodic_battle_fairy_arrow(
        game, player_id, instance, dispatch, begin_target, runtime, obstacle_string,
        apply, |runtime: &mut Runtime| runtime.now_milliseconds(),
    ))
}

pub(crate) fn execute_battle_fairy_poison_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != POISON_ARROW_SKILL_ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    battle_fairy_outcome(zone::execute_battle_fairy_poison_arrow(
        game, player_id, instance, dispatch, begin_target, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
    ))
}

/// Замена первого состояния `state_id` у цели (scan → локализация офсета до
/// End → End + destructor остатка); делегация шва арены для `bloodloss.rs`
/// до его волны.
pub(super) fn arrow_replacement_slot(
    game: &mut CGame, target: (i32, ShapeIdentity), state_id: u32,
) -> Option<Option<(usize, usize)>> {
    game.arrow_state_replacement_slot(target, state_id)
}
