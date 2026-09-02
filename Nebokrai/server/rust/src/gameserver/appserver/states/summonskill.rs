//! Общий достигнутый хвост `CSummonSkill::End`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/summonskill.cpp`. При `End(1)` конкретный навык сначала
//! выполняет свой виртуальный `Summon`, включая унаследованный оружейный
//! `AfterUseSkill`, после чего базовый `CSkill::End` обновляет свойства,
//! очищает текущий навык и фиксирует время восстановления. В достигнутых
//! владельцах виртуальный `Summon` является синхронным обновлением свойств
//! игрока; skill-specific состояние и движение остаются у конкретного owner-а.
//! При `End(0)` `Summon` и `AfterUseSkill` не выполняются, но базовый
//! `CSkill::End` всё равно обновляет свойства перед очисткой current skill;
//! concrete owner завершает свои флаги через отдельный abort-путь. `CCorpsePtomaine`,
//! `CSpriteBurn`, `CGibe`, `CMonsterTaming` и `CPetsControl` переопределяют
//! `AfterUseSkill` пустой функцией; для них используется явный хвост без
//! износа оружия.
//! Конструктор исходного класса менял лишь vtable и техническую категорию `3`;
//! в Rust категория читается из `CSkillBaseProperties`, а общий lifecycle
//! принадлежит `skills/kernel.rs` и `CPlayerAI`. Неизвестных частей в
//! достигнутом хвосте нет.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) fn abort_skill(game: &mut CGame, player_id: i32) {
    let _ = game.update_player_properties(player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
}

fn finish_summon_skill_owner<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    damage_weapon: bool,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    if damage_weapon {
        game.damage_player_weapon(player_id, runtime);
    }
    let _ = game.update_player_properties(player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_current_skill_id(None);
    }
    mark_used(player_ai, runtime.now_milliseconds());
}

pub(crate) fn finish_summon_skill<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    finish_summon_skill_owner(game, player_id, player_ai, runtime, true, mark_used);
}

pub(crate) fn finish_summon_skill_without_weapon_wear<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    finish_summon_skill_owner(game, player_id, player_ai, runtime, false, mark_used);
}
