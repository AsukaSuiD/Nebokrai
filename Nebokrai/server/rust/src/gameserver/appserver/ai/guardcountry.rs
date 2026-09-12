//! Национальный охранник `CGuardCountry`, типы ИИ `17` и `100`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает приоритет игрока другой страны и особое правило минимальной
//! дистанции. Сон использует общую проверку подключённых игроков в девяти
//! областях. `OnIdle` ставит стационарную очередь
//! `ChangeSkill → Stand → SearchEnemy`. Реакция на урон немедленно повторяет
//! ту же поисковую политику.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\guardcountry.cpp
// `OnIdle` сопоставлен с RVA 0x0020B770.
// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::ai::fixedpositionarcher::FixedArcherTarget;
use crate::gameserver::appserver::ai::guardwithbow::{
    select_guard_monster_target, select_guard_with_bow_target,
};
use crate::gameserver::gameserver::game::{CGame, ServerRegionOwner};
use crate::setup::monsterlist::MonsterProperties;

fn consider_other_country_target(
    selected: Option<FixedArcherTarget>,
    candidate: FixedArcherTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<FixedArcherTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    match selected {
        None => Some(candidate),
        Some(current)
            if candidate.distance < current.distance
                && minimum_skill_distance <= candidate.distance =>
        {
            Some(candidate)
        }
        Some(current) => Some(current),
    }
}

/// Сохраняет точный приоритет обоих вариантов: сначала игрок другой страны,
/// затем для типов `13/20` преступник и монстр, а для типа `14` только монстр.
pub(crate) fn select_country_guard_target(
    game: &CGame,
    owner: &ServerRegionOwner,
    monster_id: i32,
    property: &MonsterProperties,
    minimum_skill_distance: i32,
) -> Option<crate::gameserver::appserver::shape::ShapeIdentity> {
    let region = owner.base();
    let monster = region.find_monster_by_id(monster_id)?;
    let monster_view = monster.shape_view(property)?;
    let area_index = monster.move_shape().shape().area_index()?;
    let mut selected = None;
    if property.race != 0 {
        for player_id in region.player_ids_around_area(area_index) {
            let Some(player) = game.find_player(player_id) else {
                continue;
            };
            if player.server_region_id() != Some(region.id)
                || player.is_dead()
                || u32::from(player.country()) == property.race
            {
                continue;
            }
            let Some(candidate) = player.shape_view() else {
                continue;
            };
            selected = consider_other_country_target(
                selected,
                FixedArcherTarget {
                    identity: candidate.identity,
                    distance: monster_view.real_distance(Some(candidate)),
                },
                property.guard_range as i32,
                minimum_skill_distance,
            );
        }
    }
    if let Some(selected) = selected {
        return Some(selected.identity);
    }
    if matches!(property.ai, 17 | 100) {
        select_guard_with_bow_target(
            game,
            owner,
            monster_id,
            property,
            minimum_skill_distance,
        )
    } else {
        select_guard_monster_target(
            game,
            owner,
            monster_id,
            property,
            minimum_skill_distance,
        )
    }
}

/// Немедленно повторяет собственную поисковую политику охранника после урона,
/// не назначая нападавшего целью общей ветвью `CMonsterAI`.
pub(crate) fn retarget_special_guard_after_hurt(
    game: &CGame,
    owner: &mut ServerRegionOwner,
    monster_id: i32,
    property: &MonsterProperties,
) {
    let region = owner.base();
    if !matches!(property.ai, 8 | 17 | 100 | 101)
        || region
            .find_monster_by_id(monster_id)
            .is_none_or(|monster| monster.ai_target().is_some())
    {
        return;
    }
    let Some(current_skill_id) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.move_shape().current_skill_id())
    else {
        return;
    };
    let Some(skill) = property
        .skills
        .iter()
        .copied()
        .filter(|skill| u32::from(skill.id) == current_skill_id)
        .max_by_key(|skill| skill.level)
    else {
        return;
    };
    let minimum_skill_distance = game
        .skill_base_properties(current_skill_id, i32::from(skill.level))
        .map_or(0, |properties| properties.query_property(5_004) as i32);
    let selected = if property.ai == 8 {
        select_guard_with_bow_target(
            game,
            owner,
            monster_id,
            property,
            minimum_skill_distance,
        )
    } else {
        select_country_guard_target(
            game,
            owner,
            monster_id,
            property,
            minimum_skill_distance,
        )
    };
    if let (Some(selected), Some(monster)) =
        (selected, owner.base_mut().find_monster_by_id_mut(monster_id))
    {
        monster.set_ai_target(selected);
    }
}
