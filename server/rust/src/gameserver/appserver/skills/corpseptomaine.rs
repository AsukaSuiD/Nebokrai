//! Тонкий путь к трупному яду `CCorpsePtomaine` (`0x19F`) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/corpseptomaine.cpp`. Обе ветви и payload-цепочка
//! AddState перенесены буквально в `nebokrai_zone::skills::corpseptomaine`
//! (основание и статусы MATCH см. там; кластер D полосы Monster 0x19x);
//! **FIX F2** — player-скан клеток больше не ограничен Rust-allowlist типов
//! `{400, 500, 600, 1100, 1200}`: нативный scan принимает любой живой
//! RTTI-CMoveShape по виртуальному IsAttackAble (AI `0x53A230`); остаток
//! домена (разрешители owner-а) зафиксирован в шапке zone-владельца.
//! Здесь — объявленные швы переноса: hub-фасады `CorpsePtomaineGame`
//! (региональная форма `live_skill_target_attackable`) и
//! `CorpsePtomaineContact` (`finish_summon_skill` прежнего hub
//! `states/summonskill`) над прежним `CGame`; арена состояний — общий hub
//! `spiderpoison` (`SpiderPoisonStateArena`, реализован рядом), остальное —
//! hub `monsterattack` кластера A2. Делегации сохраняют прежние сигнатуры —
//! потребители (`monsterbaseattack.rs`, `game.rs`, `monster.rs`) не меняются.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::corpseptomaine::{
    self as zone, CorpsePtomaineContact, CorpsePtomaineGame, CorpsePtomaineOutcome,
};

pub(crate) use nebokrai_zone::skills::corpseptomaine::CORPSE_PTOMAINE_SKILL_ID;

impl CorpsePtomaineGame for CGame {
    fn ptomaine_target_attackable(
        &self,
        region_id: i32,
        source: ShapeIdentity,
        target: ShapeIdentity,
    ) -> bool {
        self.live_skill_target_attackable(region_id, source, target)
    }
}

impl<Runtime: GameMainLoopRuntime> CorpsePtomaineContact<Runtime> for CGame {
    fn finish_corpse_ptomaine_summon_skill(
        &mut self,
        player_id: i32,
        skill_id: u32,
        runtime: &mut Runtime,
    ) {
        finish_summon_skill(self, player_id, skill_id, runtime);
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) const fn is_player_corpse_ptomaine_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_player_corpse_ptomaine_dispatch(dispatch)
}

pub(crate) fn execute_player_corpse_ptomaine<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let state = match zone::execute_player_corpse_ptomaine(game, player_id, dispatch, ai, runtime, game_tick_milliseconds) {
        CorpsePtomaineOutcome::Begun => QueuedSkillExecutionState::Begun,
        CorpsePtomaineOutcome::Pending => QueuedSkillExecutionState::Pending,
        CorpsePtomaineOutcome::Completed => QueuedSkillExecutionState::Completed,
        CorpsePtomaineOutcome::Rejected => QueuedSkillExecutionState::Rejected,
    };
    terminal(state)
}

pub(crate) fn cancel_player_corpse_ptomaine<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> bool {
    zone::cancel_player_corpse_ptomaine(game, player_id, ai, runtime)
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт")]
pub(crate) fn execute_owned_corpse_ptomaine<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    zone::execute_owned_corpse_ptomaine(
        game, owner, monster_id, target_identity, skill_level, properties, now_ms, runtime,
        game_tick_milliseconds,
    )
}
