//! Достигнутая часть городского охранника с луком (AI11).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/cityguardwithbow.cpp подтверждают общий городской поиск игроков
//! и питомцев по владельцу фракции и союза, преимущество игрока при равной
//! дистанции и повторный поиск после урона только вне боя. Точные очереди
//! OnSchedule/OnFighting/OnIdle и не достигнутый из OnSearch поиск повозок
//! сохранены как RAW.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithbow.cpp

// ============================================================================
// FUNCTION: CCityGuardWithBow::OnSchedule
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithbow.cpp:26
// RVA: 0x0020B890
// ADDRESS: 0060b890
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCityGuardWithBow::OnFighting
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithbow.cpp:183
// RVA: 0x0020D930
// ADDRESS: 0060d930
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCityGuardWithBow::OnIdle
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithbow.cpp:93
// RVA: 0x0020D970
// ADDRESS: 0060d970
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCityGuardWithBow::SearchEnemyGuildCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithbow.cpp:350
// RVA: 0x0020DFC0
// ADDRESS: 0060dfc0
// PROTOTYPE: CMoveShape * __thiscall SearchEnemyGuildCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

use super::cityguardwithsword::select_city_guard_enemy;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

/// `WhenBeenHurted` AI11 не принимает атакующего напрямую: вне боя он заново
/// выполняет общий городской поиск игроков и питомцев текущим навыком.
pub(crate) fn retarget_city_bow_guard_after_hurt(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    now_ms: u32,
) {
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
        monster.when_been_hurted(now_ms);
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
