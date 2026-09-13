//! Элементальное попадание областей YinYang и GodThunder.
//! Источник: gameserver.exe/GameServer.pdb, CalculateAttackPower/Attack владельцев
//! yinyangphalanx{,2}.cpp и godthunderphalanx{,2}.cpp.
//! Урон берётся из снимка формы, но Player по attacker ID и оружейный множитель
//! разрешаются при каждом попадании. Поиск Player не зависит от master type;
//! его отсутствие сохраняет пустую исходную атаку, не отменяя raw receipt.
//! Допуск принадлежит обходу; здесь IsDied→PK seed→Calculate→receipt, без RP.
//! Только второй GodThunder передаёт признак попадания по боевому духу.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ElementPhalanxAttack {
    pub(crate) master: MasterInfo,
    pub(crate) skill_id: u32,
    pub(crate) skill_level: i32,
    pub(crate) minimum: i32,
    pub(crate) maximum: i32,
    pub(crate) element: i32,
    pub(crate) critical_chance: i32,
}

impl ElementPhalanxAttack {
    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    fn calculate(self, game: &mut CGame, target: (i32, ShapeIdentity), attack: &mut AttackInformation) {
        let Some(player) = game.find_player(attack.attacker_id) else { return; };
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
        let Some(level) = game.move_shape_level(target.0, target.1) else { return; };
        let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
        attack.damage_factor = player.weapon_modifier(game.goods_factory(), i32::from(level), divisor, minimum);
        attack.hit_modifier = 100;
        let width = self.maximum.wrapping_sub(self.minimum).wrapping_abs().wrapping_add(1);
        let damage = game.skill_random_below(width).wrapping_add(self.minimum)
            .wrapping_add(self.element).max(0);
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0,
        });
        if game.skill_random_below(100) < self.critical_chance {
            attack.critical = true;
            for power in &mut attack.damages {
                if matches!(power.kind, AttackPowerType::Physical | AttackPowerType::Element | AttackPowerType::Soul) {
                    power.hp_damage = truncate_original(
                        f64::from(power.hp_damage) * f64::from(game.globe_setup().critical_rate()),
                    );
                }
            }
        }
    }
}

pub(crate) fn apply_element_phalanx_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: ElementPhalanxAttack, target: (i32, ShapeIdentity),
    war_soul: bool, runtime: &mut Runtime,
) {
    if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
    let master = snapshot.attack_master();
    let mut attack = AttackInformation::for_master(master);
    snapshot.calculate(game, target, &mut attack);
    if war_soul {
        game.apply_owned_skill_attack_to_war_soul(master, target.1.id, target.0, attack, runtime);
    } else {
        game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    }
}
