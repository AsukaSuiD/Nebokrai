//! Достигнутый ИИ обычного лучника `CArcher`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/archer.cpp` подтверждают выбор живой цели с
//! минимальным текущим HP внутри дальности охраны. Реальный путь сохраняет
//! девять соседних областей, порядок игроков перед питомцами, первое совпадение
//! при равных HP и точную `CShape::Distance`. Общие `Run`, `OnSchedule` и
//! реакция на полученный урон принадлежат уже действующему циклу монстра.
//!
//! `OnMoving` ниже остаётся RAW в части точного момента постановки события
//! `ASA_SEARCH_ENEMY`; достигнутый цикл выполняет поиск после утраты цели.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\archer.cpp
// ============================================================================
// FUNCTION: CArcher::OnMoving
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: общий цикл повторяет поиск для живого лучника без цели;
// точная граница постановки события после движения остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\archer.cpp:41
// RVA: 0x0020FEE0
// ADDRESS: 0060fee0
// PROTOTYPE: int __thiscall OnMoving(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::CGame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcherTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
    pub(crate) hit_points: u32,
}

/// `CArcher::OnSearchEnemy` сохраняет первого кандидата при равных HP и меняет
/// его только на следующую живую цель с меньшим текущим HP внутри дальности
/// охраны. Порядок игроков перед питомцами остаётся наблюдаемой частью выбора.
pub(crate) fn consider_archer_target(
    selected: Option<ArcherTarget>,
    candidate: ArcherTarget,
    guard_range: i32,
) -> Option<ArcherTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    match selected {
        Some(current) if current.hit_points <= candidate.hit_points => Some(current),
        _ => Some(candidate),
    }
}

/// Выполняет достигнутый `OnSearchEnemy` AI4 по упорядоченным индексам
/// игроков, затем питомцев. При одинаковом HP сохраняется первая цель.
pub(crate) fn select_archer_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected = consider_archer_target(
            selected,
            ArcherTarget {
                identity: candidate.identity,
                distance: owner.distance(candidate),
                hit_points: player.health(),
            },
            guard_range,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((candidate, hit_points)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, pet.hit_points()))
            })
        else {
            continue;
        };
        selected = consider_archer_target(
            selected,
            ArcherTarget {
                identity: candidate.identity,
                distance: owner.distance(candidate),
                hit_points,
            },
            guard_range,
        );
    }
    selected.map(|selected| selected.identity)
}
