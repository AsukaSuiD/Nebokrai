//! Городской охранник с луком `CCityGuardWithBow` (AI11): hurt-повторный
//! поиск вне боя парой виртуальных selector-ов с минимальной дистанцией
//! текущего навыка.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithbow.cpp`.
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `WhenBeenHurted`: базовый hurt (`0x004C93E0`) всегда; `HasTarget != 0` → выход. Иначе `SearchEnemyGuildMember` (vt `+0x90`) и `SearchEnemyGuildPet` (vt `+0x94`), ближайший, игрок при равной дистанции, назначение через virtual `SetTarget`. `SearchEnemyGuildCarriage` (vt `+0x98`) hurt-путь **не вызывает** — прежний hub комбинировал и повозки (установленное расхождение hub, здесь исправлено) | VA `0x0060DA90`, vtable `0x00662BCC` | [`retarget_city_bow_guard_after_hurt`] | `MATCH` |
//! | selector-пара совпадает с городским общим поиском `0x0060E290`/`0x0060DB10` (фильтры владельца города, минимальная дистанция навыка) | VA `0x0060DB90`/`0x0060DD50` (не перечитаны построчно; форма подтверждена телами sword-пары `0x0060E350`/`0x0060E510` и единым слотом vtable-пары) | [`super::cityguardwithsword::select_city_guard_enemy`] | `PARTIAL` ( bow-тела selector-ов предполагают sword-форму) |
//! | минимальная дистанция текущего навыка: hub-форма выбирает запись setup с наибольшим уровнем среди совпадающих ID и читает `QueryProperty(5004)` | hub-контракт прежнего владельца (лифт `+0x70` навыка) | [`retarget_city_bow_guard_after_hurt`] | `PARTIAL` (маршрут через setup-уровень — прежний машинный вывод) |
//!
//! Разрешение текущего навыка реестром `CMoveShape`, базовые FIFO и применение
//! цели остаются hub-владением.

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
