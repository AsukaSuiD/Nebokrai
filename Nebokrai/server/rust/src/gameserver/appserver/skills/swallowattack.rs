//! Два независимых прохода направленной атаки Swallow.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/swallow.cpp.
//!
//! Восемь масок 3×3 одинаковы для всех трёх групп уровней. Центр и регион
//! фиксируются для одного прохода, но маска каждой клетки выбирается по
//! живому направлению U. X идёт снаружи, Y внутри; GetShapes снимает только
//! текущую клетку. Локальный Vec отмечает цель после допуска и Attack, даже
//! при отказе. Attack отдельно проверяет смерть до PK-seed, общего оружейного
//! Calculate, сырого OnBeenAttacked и RP; второй проход имеет свой Vec.

use super::flash::cell_views;
use super::weaponattack::apply_player_weapon_attack;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const DIRECTIONAL_SCOPE: [[u8; 9]; 8] = [
    [1, 1, 1, 0, 0, 0, 0, 0, 0], [0, 1, 1, 0, 0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0, 1, 0, 0, 1], [0, 0, 0, 0, 0, 1, 0, 1, 1],
    [0, 0, 0, 0, 0, 0, 1, 1, 1], [0, 0, 0, 1, 0, 0, 1, 1, 0],
    [1, 0, 0, 1, 0, 0, 1, 0, 0], [1, 1, 0, 1, 0, 0, 0, 0, 0],
];

pub(super) fn run_swallow_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let shape = user.shape();
    if !shape.is_assigned_to_server_region() { return; }
    let region_id = shape.get_region_id();
    if game.find_region(region_id).is_none() { return; }
    let origin_x = shape.get_tile_x().unwrap_or(i32::MIN).wrapping_sub(1);
    let origin_y = shape.get_tile_y().unwrap_or(i32::MIN).wrapping_sub(1);
    let mut seen = Vec::new();
    for x in 0..3_usize {
        for y in 0..3_usize {
            let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
            let direction = user.shape().get_direction();
            let Some(scope) = usize::try_from(direction).ok().and_then(|index| DIRECTIONAL_SCOPE.get(index)) else { return; };
            if scope[x + 3 * y] == 0 { continue; }
            for view in cell_views(game, region_id, origin_x.wrapping_add(x as i32), origin_y.wrapping_add(y as i32)) {
                let Some(sufferer) = resolve_state_move_shape(game, region_id, view.identity) else { continue; };
                let target = (sufferer.shape().get_region_id(), sufferer.shape().identity());
                if target.1 == source.1 || seen.contains(&target.1) { continue; }
                if game.live_skill_target_attackable(target.0, source.1, target.1)
                    && game.move_shape_health(target.0, target.1).is_some_and(|health| health != 0)
                {
                    apply_player_weapon_attack(game, instance, source, target, TARGET_DAMAGE_FACTOR, runtime);
                }
                seen.push(target.1);
            }
        }
    }
}
