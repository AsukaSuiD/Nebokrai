//! Делегат ИИ умного гладиатора `CSmartGladiator` (AI2) в Zone.
//!
//! Очередь шагов отхода, уязвимый выбор цели (HP < 40%), hurt-ответ и
//! готовый-idle шлюз `[+0x8C] == 0` перенесены буквально в
//! `nebokrai_zone::ai::smartgladiator` — машинная база `MATCH` по точной
//! паре `4F5C98E0…` + GameServer.pdb (RSDS match), RVA-якоря (`0x006103B0`
//! WhenBeenHurted, `0x00610660` OnIdle, `0x006106E0` OnSchedule,
//! `0x00610850` ctor, `0x00610AC0` OnSearchEnemy). Два исправленных этой
//! волной расхождения прежнего hub (шаг к ближайшему монстру hurt-ответа и
//! запись `CShape::SetDir` перед `GetDirPos`/`MoveTo`) и граница
//! tick/hibernate описаны в её шапке. Здесь:
//!
//! - реализации hub-трейтов Zone над прежними `CMonster` и `CPlayer` —
//!   очередь шагов остаётся полем переходного `CMonster`, HP-проекции
//!   игрока читаются прежними методами;
//! - делегации с прежними сигнатурами и переэкспорт типов выбора —
//!   потребители старого пакета (`monsterbaseattack`, `periodicattack`,
//!   runtime-вход `game.rs`, `monster.rs`) не меняются.

use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeView;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, game_tick_milliseconds};
use crate::setup::monsterlist::MonsterProperties;

use nebokrai_zone::ai::smartgladiator::{
    SmartGladiatorDispatcherMonster, SmartGladiatorDispatcherPlayer,
};

pub(crate) use nebokrai_zone::ai::smartgladiator::{
    SmartGladiatorSelection, SmartGladiatorState,
};

impl SmartGladiatorDispatcherMonster for CMonster {
    fn smart_gladiator_ai(&self) -> Option<&SmartGladiatorState> {
        CMonster::smart_gladiator_ai(self)
    }

    fn smart_gladiator_ai_mut(&mut self) -> Option<&mut SmartGladiatorState> {
        CMonster::smart_gladiator_ai_mut(self)
    }

    fn maximum_hit_points(&self, property: &MonsterProperties) -> u32 {
        CMonster::maximum_hp(self, property)
    }
}

impl SmartGladiatorDispatcherPlayer for CPlayer {
    fn hit_points(&self) -> u32 {
        CPlayer::health(self)
    }

    fn maximum_hit_points(&self) -> u32 {
        CPlayer::combat_properties(self).maximum_hp
    }
}

/// Один шаг schedule-фазы AI2: очередь потребляет обработанную запись даже
/// при неуспешном движении (прежняя сигнатура).
pub(crate) fn execute_smart_gladiator_retreat<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    _runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::ai::smartgladiator::execute_smart_gladiator_retreat(
        game, region, monster_id, game_tick_milliseconds,
    )
}

/// Собирает достигнутый выбор AI2 по упорядоченным индексам игроков, затем
/// питомцев (прежняя сигнатура).
pub(crate) fn select_smart_gladiator_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> SmartGladiatorSelection {
    nebokrai_zone::ai::smartgladiator::select_smart_gladiator_enemy(
        game, region, owner, area_index, guard_range,
    )
}

pub(crate) use nebokrai_zone::ai::smartgladiator::apply_monster_hurt_response
    as smart_gladiator_monster_hurt_response_impl;

/// Применяет подтверждённую реакцию AI2 на удар игрока: защита всегда, цель
/// или немедленный шаг — только вне боя; шаг к ближайшему монстру идёт по
/// направлению `owner → monster` с записью `SetDir` (прежняя сигнатура).
pub(crate) fn apply_player_hurt_response<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    player_id: i32,
    _runtime: &mut Runtime,
) {
    nebokrai_zone::ai::smartgladiator::apply_player_hurt_response(
        game,
        region,
        monster_id,
        property,
        player_id,
        game_tick_milliseconds,
    );
}

/// AI2 принимает в цель только приручённого монстра или повозку и только
/// вне боя (прежняя сигнатура).
pub(crate) fn apply_monster_hurt_response(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    attacker_id: i32,
    now_ms: u32,
) {
    smart_gladiator_monster_hurt_response_impl(game, region, monster_id, attacker_id, now_ms);
}
