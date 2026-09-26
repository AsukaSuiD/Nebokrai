//! Делегат ИИ неподвижного лучника `CFixedPositionArcher` (AI5) и
//! наследующей стационарной семьи в Zone.
//!
//! Предикаты очередей (`attack_completion_actions`,
//! `inherits_fixed_archer_change_skill`), точная постановка стационарного
//! `OnIdle`, restore-хвост `OnChangeSkill` и необычный выбор
//! `OnSearchEnemy` перенесены буквально в
//! `nebokrai_zone::ai::fixedpositionarcher` — машинная база `MATCH` по
//! точной паре `4F5C98E0…` + GameServer.pdb (RSDS match), RVA-якоря
//! (`0x0060F9F0` OnChangeSkill, `0x0060FA70` OnFighting, `0x0060FAA0`
//! OnIdle с `Hibernate` vt `+0x64`, `0x0060FBC0` OnSearchEnemy) описаны в её
//! шапке волной Z-AI. Здесь:
//!
//! - реализация hub-трейта `FixedArcherDispatcherMoveShape` над прежним
//!   `CMoveShape` — запись выбранного навыка разрешается прежним реестром;
//! - делегации с прежними сигнатурами и переэкспорт `FixedArcherTarget` —
//!   потребители старого пакета (`monsterbaseattack`, `monster.rs`,
//!   `guardwithbow`, `guardcountry`, `godsbattleguardwithsword`) не
//!   меняются.

use crate::gameserver::appserver::moveshape::{CMoveShape, MoveShapeSkill};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeView;
use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};
use crate::setup::monsterlist::MonsterProperties;

use nebokrai_zone::ai::fixedpositionarcher::FixedArcherDispatcherMoveShape;

pub(crate) use nebokrai_zone::ai::fixedpositionarcher::{
    FixedArcherTarget, attack_completion_actions, consider_fixed_archer_target,
    inherits_fixed_archer_change_skill,
};

impl FixedArcherDispatcherMoveShape for CMoveShape {
    fn skill(&self, skill_id: u32, factory: &CSkillFactory) -> Option<&MoveShapeSkill> {
        CMoveShape::skill(self, skill_id, factory)
    }
}

/// Ставит общую точную очередь стационарного `OnIdle` (прежняя сигнатура).
pub(crate) fn queue_stationary_guard_idle<Runtime: GameMainLoopRuntime>(
    region: &mut CServerRegion,
    monster_id: i32,
    stop_frame: u32,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::fixedpositionarcher::queue_stationary_guard_idle(
        region,
        monster_id,
        stop_frame,
        game_tick_milliseconds,
    )
}

/// Выполняет производный хвост `OnChangeSkill` AI5/AI23 (прежняя сигнатура).
pub(crate) fn queue_fixed_archer_skill_delay<Runtime: GameMainLoopRuntime>(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    selected_skill_id: u16,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::fixedpositionarcher::queue_fixed_archer_skill_delay(
        game,
        region,
        monster_id,
        property,
        selected_skill_id,
        game_tick_milliseconds,
    )
}

/// Выполняет достигнутый `OnSearchEnemy` AI5 с минимальной дистанцией
/// текущего навыка (прежняя сигнатура).
pub(crate) fn select_fixed_archer_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<crate::gameserver::appserver::shape::ShapeIdentity> {
    nebokrai_zone::ai::fixedpositionarcher::select_fixed_archer_enemy(
        game,
        region,
        owner,
        area_index,
        guard_range,
        minimum_skill_distance,
    )
}
