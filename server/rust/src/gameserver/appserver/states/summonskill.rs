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
//! `CGibe`, `CMonsterTaming` и `CPetsControl` переопределяют
//! `AfterUseSkill` пустой функцией. Все overrides выбираются одним concrete
//! factory-каталогом; служебная передача флага из каждого caller больше не нужна.
//! Конструктор исходного класса менял лишь vtable и техническую категорию `3`;
//! в Rust intrinsic-категория принадлежит фабричному owner-каталогу.
//! Этот хвост применяется к уже начатому исполнению; он не заменяет полный
//! End зарегистрированного экземпляра до Begin или после прежнего End.

use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) fn finish_summon_skill<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) {
    game.after_use_player_skill(player_id, skill_id, runtime);
}
