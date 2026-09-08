//! Общий хвост завершения навыков игрока через CSummonSkill::End.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/states/summonskill.cpp`. End (0x005e0f40) при успехе вызывает
//! virtual AfterUseSkill (+0x90), затем CSkill::End (0x004d84c0). Обычный
//! AfterUseSkill (0x0053cf30) вызывает CPlayer::OnWeaponDamaged (0x00441d50).
//! Базовый End вызывает у источника virtual +0x158, который у CPlayer пуст:
//! здесь нет дополнительного пересчёта свойств или повторного Summon.
//! Этот слот — пустой ret 0x00485540. End не меняет m_pCurrentSkill игрока:
//! он очищает только поля собственного CSkill. Выбор default attack остаётся
//! у OnChangeSkill/OnLoseTarget, в том числе после завершения фонового навыка.
//! Игровые эффекты, пересчёты от изменения состояний и движение принадлежат
//! конкретному навыку. После успешного завершения фиксируется время reuse;
//! End(0) не вызывает AfterUseSkill и не меняет reuse. `CCorpsePtomaine`,
//! `CSpriteBurn`, `CGibe`, `CMonsterTaming` и `CPetsControl` переопределяют
//! `AfterUseSkill` пустой функцией; для них используется явный хвост без
//! износа оружия.
//! Конструктор исходного класса менял лишь vtable и техническую категорию `3`;
//! в Rust intrinsic-категория принадлежит фабричному owner-каталогу.
//! Этот хвост применяется к уже начатому исполнению; он не заменяет полный
//! End зарегистрированного экземпляра до Begin или после прежнего End.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

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
