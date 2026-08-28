//! Канонический владелец четырёх автоматических состояний восстановления.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! исходные владельцы `CAutomaticRestoreHpState`,
//! `CAutomaticRestoreHpStateFight`, `CAutomaticRestoreMpState` и
//! `CAutomaticRestoreMpStateFight`. Сохранены порядок `HP peace → HP fight →
//! MP peace → MP fight`, строгое wrapping-сравнение таймера и второй вызов
//! часов, которым фиксируется момент срабатывания. Объём восстановления
//! читается из живого `PlayerCombatProperties`, как в исходных `AI`.

use crate::gameserver::appserver::player::PlayerCombatProperties;

pub(crate) const AUTOMATIC_RESTORE_HP_PEACE_STATE_ID: u32 = 0x186a2;
pub(crate) const AUTOMATIC_RESTORE_MP_PEACE_STATE_ID: u32 = 0x186a3;
pub(crate) const AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID: u32 = 0x186ad;
pub(crate) const AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID: u32 = 0x186ae;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AutomaticRestoreKind {
    HealthPeace,
    HealthFight,
    ManaPeace,
    ManaFight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AutomaticRestoreMutation {
    Health(u32),
    Mana(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AutomaticRestoreState {
    kind: AutomaticRestoreKind,
    frequency_ms: u32,
    last_tick_ms: u32,
}

impl AutomaticRestoreState {
    const fn new(kind: AutomaticRestoreKind, frequency_ms: i32) -> Self {
        Self {
            kind,
            frequency_ms: frequency_ms as u32,
            last_tick_ms: 0,
        }
    }

    pub(crate) const fn restored(properties: PlayerCombatProperties) -> [Self; 4] {
        [
            Self::new(AutomaticRestoreKind::HealthPeace, properties.resume_hp_peace),
            Self::new(AutomaticRestoreKind::HealthFight, properties.resume_hp_fight),
            Self::new(AutomaticRestoreKind::ManaPeace, properties.resume_mp_peace),
            Self::new(AutomaticRestoreKind::ManaFight, properties.resume_mp_fight),
        ]
    }

    pub(crate) const fn should_check(
        self,
        dead: bool,
        shape_state: u16,
        health: u32,
        maximum_health: u32,
        mana: u32,
        maximum_mana: u32,
    ) -> bool {
        if dead {
            return false;
        }
        match self.kind {
            AutomaticRestoreKind::HealthPeace => health != maximum_health && shape_state == 0,
            AutomaticRestoreKind::HealthFight => health != maximum_health && shape_state == 1,
            AutomaticRestoreKind::ManaPeace => mana != maximum_mana && shape_state == 0,
            AutomaticRestoreKind::ManaFight => mana != maximum_mana && shape_state == 1,
        }
    }

    pub(crate) const fn due(self, checked_at_ms: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_tick_ms) < checked_at_ms
    }

    pub(crate) fn apply(
        &mut self,
        recorded_at_ms: u32,
        properties: PlayerCombatProperties,
        health: u32,
        maximum_health: u32,
        mana: u32,
        maximum_mana: u32,
    ) -> Option<AutomaticRestoreMutation> {
        self.last_tick_ms = recorded_at_ms;
        let (current, maximum, amount, health_target) = match self.kind {
            AutomaticRestoreKind::HealthPeace => (
                health,
                maximum_health,
                properties.restored_hp_peace,
                true,
            ),
            AutomaticRestoreKind::HealthFight => (
                health,
                maximum_health,
                properties.restored_hp_fight,
                true,
            ),
            AutomaticRestoreKind::ManaPeace => (
                mana,
                maximum_mana,
                properties.restored_mp_peace,
                false,
            ),
            AutomaticRestoreKind::ManaFight => (
                mana,
                maximum_mana,
                properties.restored_mp_fight,
                false,
            ),
        };
        if amount == 0 {
            return None;
        }
        let restored = current.wrapping_add(amount as u32).min(maximum);
        Some(if health_target {
            AutomaticRestoreMutation::Health(restored)
        } else {
            AutomaticRestoreMutation::Mana(restored)
        })
    }
}
