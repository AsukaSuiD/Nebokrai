//! Общий хвост завершения навыков игрока через CSummonSkill::End.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/summonskill.cpp`. End (0x005e0f40) при успехе вызывает
//! virtual AfterUseSkill (+0x90), затем CSkill::End (0x004d84c0). Обычный
//! AfterUseSkill (0x0053cf30) вызывает CPlayer::OnWeaponDamaged (0x00441d50).
//! Базовый End вызывает у источника virtual +0x158, который у CPlayer пуст:
//! здесь нет дополнительного пересчёта свойств или повторного Summon.
//! Игровые эффекты, пересчёты от изменения состояний и движение принадлежат
//! конкретному навыку. После успешного завершения фиксируется время reuse;
//! End(0) не вызывает AfterUseSkill и не меняет reuse. `CCorpsePtomaine`,
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
