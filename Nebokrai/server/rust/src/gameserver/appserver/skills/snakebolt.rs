//! Владелец змеиного снаряда поверх общего механизма пошагового снаряда.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` и исходный владелец
//! `GameServer/appserver/skills/snakebolt.cpp` подтверждают единичную область
//! на первых двух уровнях, область 3×3 с третьего уровня, начальную позицию пути
//! ноль и коэффициент оружия для игрока. Проверка, MP, задержка, потребление
//! `CSoulCollectState`, единственный вызов RNG и сетевые стадии исполняются общим
//! механизмом `energybolt`; эта обёртка задаёт только различающийся контракт
//! змеиного снаряда. Унаследованный `CAttackSkill::AfterUseSkill` изнашивает
//! оружие один раз при `End(true)`.
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

pub(crate) const SNAKE_BOLT_SKILL_ID: u32 = 0x1a5;

pub(crate) fn execute_player_snake_bolt<Runtime: GameMainLoopRuntime>(
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
        PathProjectileSpec::new(SNAKE_BOLT_SKILL_ID, 3, true, 0),
        ai,
        runtime,
    )
}
#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn execute_owned_snake_bolt<Runtime: GameMainLoopRuntime>(
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
        PathProjectileSpec::new(SNAKE_BOLT_SKILL_ID, 3, true, 0),
        skill_level,
        properties,
        now_ms,
        runtime,
    )
}
