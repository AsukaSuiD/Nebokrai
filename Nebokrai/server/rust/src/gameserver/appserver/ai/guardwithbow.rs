//! Поиск цели и реакция на урон лучника городской охраны `CGuardWithBow`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает приоритет преступных игроков перед неохранными монстрами.
//! Проверка политики атаки остаётся у GameServer, а необычный выбор по
//! минимальной дистанции навыка разделяется с неподвижным лучником. Немедленная
//! реакция на урон вызывает того же владельца до продолжения обработки попадания.
//! Общий сон включается только при отсутствии подключённых игроков во всех
//! девяти областях; при наличии игроков `OnIdle` ставит стационарную очередь
//! `ChangeSkill → Stand → SearchEnemy`.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\guardwithbow.cpp
// `OnIdle` сопоставлен с RVA 0x0020EE60.

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::ai::fixedpositionarcher::{
    FixedArcherTarget, consider_fixed_archer_target,
};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::appserver::skills::monsterattack::monster_attackable_by_monster;
use crate::gameserver::gameserver::game::CGame;
use crate::setup::monsterlist::MonsterProperties;

/// Выполняет подтверждённый порядок `SearchCriminal → SearchMonster`.
/// Возвращаемая цель ещё не назначена монстру: это остаётся короткой операцией
/// владельца цикла после завершения неизменяемого обхода индексов региона.
pub(crate) fn select_guard_with_bow_target(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity> {
    let monster = region.find_monster_by_id(monster_id)?;
    let monster_view = monster.shape_view(property)?;
    let area_index = monster.move_shape().shape().area_index()?;
    let guard_range = property.guard_range as i32;

    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id)
            || player.is_dead()
            || !player.is_badman(game.globe_setup().pk_count_per_kill())
            || !game.player_attackable_by_monster(
                player_id,
                region.id,
                property,
                false,
                monster.master_info(),
            )
        {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_fixed_archer_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    monster_view.tile_x,
                    monster_view.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    if let Some(selected) = selected {
        return Some(selected.identity);
    }

    select_guard_monster_target(game, region, monster_id, property, minimum_skill_distance)
}

/// Общая запасная ветвь охранников: ближайший неохранный монстр, которого
/// допускает действующая политика `CMonster::IsAttackAble`.
pub(crate) fn select_guard_monster_target(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    minimum_skill_distance: i32,
) -> Option<ShapeIdentity> {
    let monster = region.find_monster_by_id(monster_id)?;
    let monster_view = monster.shape_view(property)?;
    let area_index = monster.move_shape().shape().area_index()?;
    let guard_range = property.guard_range as i32;
    let attacker_master = monster.master_info();
    let mut selected = None;
    for target_id in region.monster_ids_around_area(area_index) {
        let Some(target) = region.find_monster_by_id(target_id) else {
            continue;
        };
        let Some(target_property) = target
            .base_property_key()
            .and_then(|key| game.find_monster_property_by_origin_name(key))
        else {
            continue;
        };
        if target_property.kind == 5 || CMoveShape::is_died(target.hit_points()) {
            continue;
        }
        if !monster_attackable_by_monster(
            game,
            property,
            false,
            attacker_master,
            target_property,
            target.is_tamed(),
            target.master_info(),
            region.id,
        ) {
            continue;
        }
        let Some(candidate) = target.shape_view(target_property) else {
            continue;
        };
        selected = consider_fixed_archer_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    monster_view.tile_x,
                    monster_view.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    selected.map(|selected| selected.identity)
}
