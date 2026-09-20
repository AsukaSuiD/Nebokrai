//! Элементный контакт BaseMagic, FireBolt, FireBall и GodPunishment.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые phalanx.cpp.
//! MIN/MAX/ELEMENT и усиление душами принадлежат снимку конструктора.
//! Calculate ищет игрока по attacker ID независимо от сохранённого типа;
//! отсутствие игрока оставляет пустую атаку, но не отменяет контакт.
//! Живой element_modify читается до уровня цели и weapon modifier.
//! Signed wrapping процент вычисляется до RNG(abs(MAX-MIN)+1), затем идут
//! сохранённый MIN, живой AddElement и необязательное усиление душами.
//! Усиление использует signed count/variable и x87-порядок до усечения,
//! без промежуточного float; нижняя граница ноль применяется после него.
//! Затем живой CCH и общий критический хвост. Повторной таблицы, SOUL,
//! допуска и RP здесь нет; флаг попадания по боевой фее передаёт caller.

use super::fightdefense::truncate_original;
use super::weaponattack::{SourceProperty, apply_weapon_critical, source_property};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SoulProjectileAmplification {
    count: i32,
    variable: i32,
}

impl SoulProjectileAmplification {
    pub(super) const fn new(count: i32, variable: i32) -> Self { Self { count, variable } }

    fn apply(self, damage: i32) -> i32 {
        if self.count == 0 || self.variable == 0 { return damage; }
        truncate_original(
            (f64::from(self.variable) * f64::from(self.count) * f64::from(0.01_f32) + 1.0)
                * f64::from(damage),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ElementProjectileAttack {
    master: MasterInfo,
    skill_id: u32,
    skill_level: i32,
    minimum_attack: i32,
    maximum_attack: i32,
    element_modifier: i32,
    souls: Option<SoulProjectileAmplification>,
}

impl ElementProjectileAttack {
    pub(super) const fn new(
        master: MasterInfo, skill_id: u32, skill_level: i32, minimum_attack: i32,
        maximum_attack: i32, element_modifier: i32, souls: Option<SoulProjectileAmplification>,
    ) -> Self {
        Self { master, skill_id, skill_level, minimum_attack, maximum_attack, element_modifier, souls }
    }

    pub(crate) const fn master(self) -> MasterInfo { self.master }
    pub(super) const fn skill_level(self) -> i32 { self.skill_level }

    fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    fn calculate(self, game: &mut CGame, target: (i32, ShapeIdentity), attack: &mut AttackInformation) {
        let Some(player) = game.find_player(attack.attacker_id) else { return; };
        let element_modify = player.combat_properties().element_modify;
        let source = (player.shape().get_region_id(), player.shape().identity());
        attack.skill_id = self.skill_id;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
        let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
        let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
        attack.damage_factor = player.weapon_modifier(
            game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
        );
        attack.hit_modifier = 100;

        let element = self.element_modifier.wrapping_mul(element_modify).wrapping_div(100);
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let rolled = game.skill_random_below(width).wrapping_add(self.minimum_attack);
        let Some(addition) = source_property(game, source, SourceProperty::Element) else { return; };
        let damage = element.wrapping_add((addition as i32).wrapping_add(rolled));
        let damage = self.souls.map_or(damage, |souls| souls.apply(damage)).max(0);
        attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 });
        let Some(chance) = source_property(game, source, SourceProperty::CriticalChance) else { return; };
        apply_weapon_critical(game, i32::from(chance as u16), attack);
    }

    pub(crate) fn apply<Runtime: GameMainLoopRuntime>(
        self, game: &mut CGame, target: (i32, ShapeIdentity), war_soul: bool, runtime: &mut Runtime,
    ) {
        if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
        let master = self.attack_master();
        let mut attack = AttackInformation::for_master(master);
        self.calculate(game, target, &mut attack);
        if war_soul {
            game.apply_owned_skill_attack_to_war_soul(master, target.1.id, target.0, attack, runtime);
        } else {
            game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
        }
    }
}
