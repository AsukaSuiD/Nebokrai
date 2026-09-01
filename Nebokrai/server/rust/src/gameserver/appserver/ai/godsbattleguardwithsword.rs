//! Выбор цели охранника битвы богов `CGBGuardWithSward`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает тип ИИ `23`: допустим игрок другой фракции либо преступник
//! своей фракции. Выбор сохраняет дальность охраны, минимальную дистанцию
//! текущего навыка и исходный порядок подключённых игроков. Унаследованные от
//! неподвижного лучника idle, задержка смены навыка, поиск и post-attack
//! переход проходят через тот же FIFO с этим фракционным selector-ом.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\godsbattleguardwithsword.cpp
// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::ai::fixedpositionarcher::{
    FixedArcherTarget, consider_fixed_archer_target,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::CGame;

/// Применяет фракционный фильтр охраны перед подтверждённым выбором цели.
pub(crate) fn consider_gods_battle_guard_target(
    selected: Option<FixedArcherTarget>,
    candidate: FixedArcherTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    owner_faction: u32,
    candidate_faction: u32,
    candidate_badman: bool,
) -> Option<FixedArcherTarget> {
    if candidate_faction == owner_faction && !candidate_badman {
        selected
    } else {
        consider_fixed_archer_target(
            selected,
            candidate,
            guard_range,
            minimum_skill_distance,
        )
    }
}

/// Выполняет достигнутый player-поиск AI103 с фракционным исключением и
/// правилом преступника своей фракции.
pub(crate) fn select_gods_battle_guard_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
    owner_faction: u32,
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
        selected = consider_gods_battle_guard_target(
            selected,
            FixedArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
            owner_faction,
            player.gods_battle_faction() as u32,
            player.is_badman(game.globe_setup().pk_count_per_kill()),
        );
    }
    selected.map(|selected| selected.identity)
}
