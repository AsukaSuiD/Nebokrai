//! Каноническое состояние семейства `CSwordshipState`.
//!
//! PDB подтверждает одинаковый набор виртуальных операций у четырёх исходных
//! классов. Отличается только ID `0x6f/0xe0/0xe8/0xe9`. Для игрока обе
//! прибавки сначала знаково усекаются до `i16`, затем складываются через
//! `u32` и ограничиваются `i32::MAX`. Для монстра исходный владелец передаёт
//! полные значения `i32` методам `SetMinAtk/SetMaxAtk`, то есть выполняет
//! сложение с переполнением.
//! Состояния не имеют собственного таймера и визуального сообщения.

use super::swordship::is_swordship_skill;
use crate::gameserver::appserver::player::PlayerCombatProperties;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwordshipState {
    skill_id: u32,
    minimum_attack_gain: i32,
    maximum_attack_gain: i32,
}

impl SwordshipState {
    pub(crate) const fn new(
        skill_id: u32,
        minimum_attack_gain: i32,
        maximum_attack_gain: i32,
    ) -> Self {
        debug_assert!(is_swordship_skill(skill_id));
        Self {
            skill_id,
            minimum_attack_gain,
            maximum_attack_gain,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.minimum_attack = apply_player_gain(
            properties.minimum_attack,
            self.minimum_attack_gain,
        );
        properties.maximum_attack = apply_player_gain(
            properties.maximum_attack,
            self.maximum_attack_gain,
        );
        properties
    }

    pub(crate) const fn apply_to_monster(self, minimum: u32, maximum: u32) -> (u32, u32) {
        (
            minimum.wrapping_add(self.minimum_attack_gain as u32),
            maximum.wrapping_add(self.maximum_attack_gain as u32),
        )
    }
}

fn apply_player_gain(value: u32, gain: i32) -> u32 {
    value
        .wrapping_add((gain as i16 as i32) as u32)
        .min(i32::MAX as u32)
}
