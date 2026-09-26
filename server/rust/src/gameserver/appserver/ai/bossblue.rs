//! Делегат ИИ синего босса `CBossBlue` (AI103) в Zone.
//!
//! Восемь одноразовых HP-порогов ярости, пороговый выбор навыка и общий
//! enemy-проход перенесены буквально в `nebokrai_zone::ai::bossblue` —
//! машинная база `MATCH` по точной паре `4F5C98E0…` + GameServer.pdb (RSDS
//! match), RVA-якоря (ctor `0x00609CB0`, `WakeUp` `0x00609CD0`, Select
//! `0x0060A0E0`), ICF-связи и граница tick hub (`OnSchedule` `0x00609FE0`,
//! `Hibernate` `0x006093B0`) описаны в её шапке. Здесь:
//!
//! - реализации hub-трейтов Zone над прежними `CMonster` и `CMoveShape` —
//!   состояние порогов остаётся полем переходного `CMonster`, признак
//!   fury-состояния читается из прежнего хранилища состояний формы;
//! - делегации с прежними сигнатурами и переэкспорт `BossBlueAiState` —
//!   потребители старого пакета (`monster.rs`, `monsterbaseattack`) не
//!   меняются.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

use nebokrai_zone::ai::bossblue::{BossBlueDispatcherMonster, BossBlueDispatcherMoveShape};

pub(crate) use nebokrai_zone::ai::bossblue::BossBlueAiState;

impl BossBlueDispatcherMoveShape for CMoveShape {
    fn has_boss_blue_fury_state(&self) -> bool {
        self.boss_blue_fury_state().is_some()
    }
}

impl BossBlueDispatcherMonster for CMonster {
    fn boss_blue_ai_mut(&mut self) -> &mut BossBlueAiState {
        self.boss_blue_ai_mut()
    }
}

/// Выполняет подтверждённый `OnSearchEnemy` синего босса общим проходом
/// игроков, затем питомцев (прежняя сигнатура).
pub(crate) fn select_boss_blue_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    nebokrai_zone::ai::bossblue::select_boss_blue_enemy(
        game, region, owner, area_index, guard_range,
    )
}

/// Разрешает состояние конкретного синего босса и выполняет его пороговый
/// выбор после единственного RNG-броска runtime (прежняя сигнатура).
pub(crate) fn choose_boss_blue_attack_skill(
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    hit_points: u32,
    roll: i32,
    default_skill_id: u16,
) -> Option<u16> {
    nebokrai_zone::ai::bossblue::choose_boss_blue_attack_skill(
        region, monster_id, property, hit_points, roll, default_skill_id,
    )
}
