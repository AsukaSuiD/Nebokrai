//! Элементальное попадание областей YinYang, GodThunder, FireWall и ChaosSphere.
//! Источник: gameserver.exe/GameServer.pdb, CalculateAttackPower/Attack владельцев
//! yinyangphalanx{,2}.cpp, godthunderphalanx{,2}.cpp, firewallphalanx.cpp
//! и chaosspherephalanx.cpp.
//! Тела Calculate/Deliver перенесены буквально в
//! `nebokrai_zone::skills::elementphalanx` порцией T5 «zonalcast-хаб»;
//! живые разрешения Player, уровня цели, оружейного множителя и PK-допуска —
//! швы `ZonalCastGame`/`ZonalCastContact` над прежними методами `CGame`
//! (`skills/zonalcast.rs`). Здесь — делегации с прежними сигнатурами;
//! потребители (обходы `game/{maskedelement,godthunder,chaossphere}.rs`)
//! не меняются.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::ElementPhalanxAttack;

pub(crate) fn apply_element_phalanx_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ElementPhalanxAttack, target: (i32, ShapeIdentity),
    war_soul: bool, runtime: &mut Runtime,
) {
    nebokrai_zone::skills::apply_element_phalanx_attack(game, snapshot, target, war_soul, runtime);
}

pub(crate) fn apply_element_phalanx_war_soul<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ElementPhalanxAttack, target_id: i32,
    runtime: &mut Runtime,
) {
    nebokrai_zone::skills::apply_element_phalanx_war_soul(game, snapshot, target_id, runtime);
}
