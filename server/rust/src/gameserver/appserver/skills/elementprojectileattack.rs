//! Элементный контакт BaseMagic, FireBolt, FireBall и GodPunishment.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые phalanx.cpp.
//! Снимок конструктора, усиление душами и числовая формула перенесены в
//! `nebokrai_zone::skills::projectile` (основание и статусы см. там); обёртка
//! сохраняет прежнее имя и inherent-интерфейс для владельцев game/. Здесь
//! остаются CGame-разрешение живых полей: поиск игрока по attacker ID
//! независимо от сохранённого типа, уровень цели, живой weapon modifier,
//! source_property и порог смерти, затем доставка обычным контактом или
//! боевой фее. Отсутствие игрока оставляет пустую атаку, не отменяя контакт.

pub(crate) use nebokrai_zone::skills::SoulProjectileAmplification;

use nebokrai_zone::skills::{ElementProjectileAttack as ElementProjectileAttackRule,
    ElementProjectileLiveField};
use super::weaponattack::{SourceProperty, source_property};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ElementProjectileAttack {
    rule: ElementProjectileAttackRule,
}

impl ElementProjectileAttack {
    pub(super) const fn new(
        master: MasterInfo, skill_id: u32, skill_level: i32, minimum_attack: i32,
        maximum_attack: i32, element_modifier: i32, souls: Option<SoulProjectileAmplification>,
    ) -> Self {
        Self {
            rule: ElementProjectileAttackRule {
                master, skill_id, skill_level, minimum_attack, maximum_attack, element_modifier,
                souls,
            },
        }
    }

    pub(crate) const fn master(self) -> MasterInfo { self.rule.master }
    pub(super) const fn skill_level(self) -> i32 { self.rule.skill_level }

    fn calculate(self, game: &mut CGame, target: (i32, ShapeIdentity), attack: &mut AttackInformation) {
        let Some(player) = game.find_player(attack.attacker_id) else { return; };
        let element_modify = player.combat_properties().element_modify;
        let source = (player.shape().get_region_id(), player.shape().identity());
        let Some(element_modify) = self.rule.begin_calculation(attack, |field| match field {
            ElementProjectileLiveField::ElementModify => Some(element_modify),
            _ => None,
        }) else { return; };
        let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
        let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
        let weapon_modifier = player.weapon_modifier(
            game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
        );
        let critical_rate = game.globe_setup().critical_rate();
        self.rule.roll_damage(
            attack, element_modify, weapon_modifier, critical_rate,
            |field| match field {
                ElementProjectileLiveField::RandomBelow(bound) => Some(game.skill_random_below(bound)),
                ElementProjectileLiveField::AddElementAttack => {
                    source_property(game, source, SourceProperty::Element).map(|value| value as i32)
                }
                ElementProjectileLiveField::CriticalChance => {
                    source_property(game, source, SourceProperty::CriticalChance)
                        .map(|value| i32::from(value as u16))
                }
                ElementProjectileLiveField::ElementModify => Some(element_modify),
            },
        );
    }

    pub(crate) fn apply<Runtime: GameMainLoopRuntime>(
        self, game: &mut CGame, target: (i32, ShapeIdentity), war_soul: bool, runtime: &mut Runtime,
    ) {
        if game.move_shape_health(target.0, target.1).is_none_or(|hp| hp == 0) { return; }
        let master = self.rule.attack_master();
        let mut attack = AttackInformation::for_master(master);
        self.calculate(game, target, &mut attack);
        if war_soul {
            game.apply_owned_skill_attack_to_war_soul(master, target.1.id, target.0, attack, runtime);
        } else {
            game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
        }
    }
}
