//! Элементальное попадание областей YinYang, GodThunder, FireWall и ChaosSphere.
//! Источник: gameserver.exe/GameServer.pdb, CalculateAttackPower/Attack владельцев
//! yinyangphalanx{,2}.cpp, godthunderphalanx{,2}.cpp, firewallphalanx.cpp
//! и chaosspherephalanx.cpp.
//! Числовой снимок и расчёт находятся в zone/skills/elementphalanx.rs.
//! Player по attacker ID и оружейный множитель разрешаются при каждом попадании.
//! Поиск Player не зависит от master type;
//! его отсутствие сохраняет пустую исходную атаку, не отменяя raw receipt.
//! Допуск принадлежит обходу; здесь IsDied→PK seed→Calculate→receipt, без RP.
//! GodThunder2 и ChaosSphere сначала проверяют глобальных Player цели/источника
//! и передают признак попадания по боевому духу; body-допуск хранится в обходе.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::ElementPhalanxAttack;

fn calculate(
    game: &mut CGame, snapshot: ElementPhalanxAttack,
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(player) = game.find_player(attack.attacker_id) else { return; };
    snapshot.begin_calculation(attack);
    let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
    let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
    let weapon_modifier = player.weapon_modifier(game.goods_factory(), i32::from(level), divisor, minimum);
    if snapshot.roll_damage(attack, weapon_modifier, |width| game.skill_random_below(width)) {
        ElementPhalanxAttack::scale_critical(attack, game.globe_setup().critical_rate());
    }
}

pub(crate) fn apply_element_phalanx_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ElementPhalanxAttack, target: (i32, ShapeIdentity),
    war_soul: bool, runtime: &mut Runtime,
) {
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    calculate(game, snapshot, target, &mut attack);
    if war_soul {
        game.apply_owned_skill_attack_to_war_soul(master, target.1.id, target.0, attack, runtime);
    } else {
        game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    }
}

pub(crate) fn apply_element_phalanx_war_soul<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ElementPhalanxAttack, target_id: i32,
    runtime: &mut Runtime,
) {
    let target = game.find_player(target_id);
    let source = game.find_player(snapshot.master.master_id);
    let (Some(target), Some(source)) = (target, source) else { return; };
    if target_id == snapshot.master.master_id { return; }
    if target.shape().get_action() == 6 || target.is_dead() { return; }
    let target = (target.shape().get_region_id(), target.shape().identity());
    let source = (source.shape().get_region_id(), source.shape().identity());
    if !game.live_skill_target_attackable_between(source, target) { return; }
    apply_element_phalanx_attack(game, snapshot, target, true, runtime);
}
