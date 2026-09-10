//! Владелец когтя зомби поверх общего механизма пошагового снаряда.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` и исходный владелец
//! `GameServer/appserver/skills/zombieclaw.cpp` подтверждают единичную область
//! на первых двух уровнях, область 3×3 с третьего уровня, начальную позицию пути
//! один и отсутствие коэффициента оружия в формуле игрока. Проверка, MP, задержка,
//! потребление `CSoulCollectState`, единственный вызов RNG и сетевые стадии
//! исполняются общим механизмом `energybolt`; эта обёртка задаёт только
//! различающийся контракт когтя зомби. Унаследованный
//! `CAttackSkill::AfterUseSkill` изнашивает оружие один раз при `End(true)`.
//! Monster-обёртка передаёт полное временное владение регионом общему
//! пошаговому снаряду: death/End не ограничивается заимствованием его базы.

use super::energybolt::{
    PathProjectileSpec, execute_owned_path_projectile, execute_player_path_projectile,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, ServerRegionOwner, QueuedSkillExecutionOutcome,
};

pub(crate) const ZOMBIE_CLAW_SKILL_ID: u32 = 0x1a2;

pub(crate) fn execute_player_zombie_claw<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_path_projectile(
        game,
        player_id,
        dispatch,
        PathProjectileSpec::new(ZOMBIE_CLAW_SKILL_ID, 3, true, 1),
        ai,
        runtime,
    )
}
#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn execute_owned_zombie_claw<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_path_projectile(
        game,
        owner,
        monster_id,
        target_identity,
        PathProjectileSpec::new(ZOMBIE_CLAW_SKILL_ID, 3, true, 1),
        skill_level,
        properties,
        now_ms,
        runtime,
    )
}
