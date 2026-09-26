//! Городской охранник с луком `CCityGuardWithBow` (AI11): hurt-повторный
//! поиск вне боя парой виртуальных selector-ов с минимальной дистанцией текущего
//! навыка. Исходный владелец PDB: `appserver/ai/cityguardwithbow.cpp`; сверка
//! по точной паре `gameserver.exe` + `GameServer.pdb`.
//!
//! `SearchEnemyGuildCarriage` hurt-путь не вызывает — прежний hub комбинировал и
//! повозки (установленное расхождение hub, здесь исправлено). VERIFIED: тела
//! bow-selector-ов `0x60DB90`/`0x60DD50` попарно сверены с sword-эталонами
//! `0x60E350`/`0x60E510` — инструкционно идентичные близнецы (441/616 байт,
//! те же фильтры faction/union и два вызова min-дистанции навыка vt `+0x70`;
//! компилятор не сложил их только из-за разных SEH-таблиц раскадровки).
//! Разрешение текущего навыка реестром `CMoveShape`, базовые FIFO и применение
//! цели остаются hub-владением.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#ai-расписаний-и-поведение

use nebokrai_shared::resources::MonsterProperties;

use super::cityguardwithsword::{
    CityGuardDispatcherMoveShape, CityGuardDispatcherPlayer, CityGuardDispatcherRegion,
    GuardStationDispatcherMonster, select_city_guard_enemy,
};
use super::monsterai::{
    MonsterDispatcherGame, MonsterDispatcherMonster, MonsterDispatcherMoveShape,
    MonsterDispatcherRegion,
};

/// `WhenBeenHurted` AI11 не принимает атакующего напрямую: вне боя он заново
/// выполняет общий городской поиск игроков и питомцев текущим навыком
/// (`SearchEnemyGuildMember` + `SearchEnemyGuildPet`).
pub fn retarget_city_bow_guard_after_hurt<Game, Region>(
    game: &Game,
    region: &mut Region,
    monster_id: i32,
    property: &MonsterProperties,
    now_ms: u32,
) where
    Game: MonsterDispatcherGame,
    Game::Player: CityGuardDispatcherPlayer,
    Region: CityGuardDispatcherRegion + MonsterDispatcherRegion,
    Region::Monster: GuardStationDispatcherMonster<MoveShape: CityGuardDispatcherMoveShape>,
{
    let Some((owner, area_index, current_skill_id, was_fighting)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.shape_view(property)?,
                monster.move_shape().shape().area_index(),
                monster.move_shape().current_skill_id(),
                monster.ai_target().is_some(),
            ))
        })
    else {
        return;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        super::jiumai::JiuMaiDispatcherMonster::when_been_hurted(monster, now_ms);
    }
    if was_fighting {
        return;
    }
    let Some((area_index, current_skill_id)) = area_index.zip(current_skill_id) else {
        return;
    };
    let Some(skill) = property
        .skills
        .iter()
        .filter(|skill| u32::from(skill.id) == current_skill_id)
        .max_by_key(|skill| skill.level)
    else {
        return;
    };
    let Some(skill_properties) =
        game.skill_base_properties(current_skill_id, i32::from(skill.level))
    else {
        return;
    };
    let selected = select_city_guard_enemy(
        game,
        region,
        owner,
        area_index,
        property.guard_range as i32,
        skill_properties.query_property(5_004) as i32,
    );
    if let (Some(selected), Some(monster)) =
        (selected, region.find_monster_by_id_mut(monster_id))
    {
        monster.set_ai_target(selected.identity);
    }
}
