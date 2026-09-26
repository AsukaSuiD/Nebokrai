//! Четыре состояния автоматического восстановления игрока Zone.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! классы `CAutomaticRestoreHpState`, `CAutomaticRestoreHpStateFight`,
//! `CAutomaticRestoreMpState`, `CAutomaticRestoreMpStateFight`.
//! Идентификаторы точной пары —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки.
//! `AI` HP peace проверяет смерть, полноту HP и мирный
//! режим до часов; при срабатывании читает живой объём восстановления,
//! меняет HP и повторно читает часы. Состояние не владеет игроком или источником часов.
//! Таблицы всех четырёх классов вызывают общие тела Serialize/Unserialize.
//! PDB называет общие тела по другим состояниям;
//! принадлежность здесь определяется vtable, а не именем символа.
//! Проекция из снимка `PlayerCombatProperties` расположена здесь (а не в
//! переходном адаптере старого пакета), потому что оба типа локальны Zone.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};

pub const AUTOMATIC_RESTORE_STATE_BYTES: usize = 12;
pub const AUTOMATIC_RESTORE_HP_PEACE_STATE_ID: u32 = 0x186a2;
pub const AUTOMATIC_RESTORE_MP_PEACE_STATE_ID: u32 = 0x186a3;
pub const AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID: u32 = 0x186ad;
pub const AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID: u32 = 0x186ae;

pub const fn is_automatic_restore_state_id(state_id: u32) -> bool {
    matches!(
        state_id,
        AUTOMATIC_RESTORE_HP_PEACE_STATE_ID
            | AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID
            | AUTOMATIC_RESTORE_MP_PEACE_STATE_ID
            | AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomaticRestoreKind {
    HealthPeace,
    HealthFight,
    ManaPeace,
    ManaFight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomaticRestoreMutation {
    Health(u32),
    Mana(u32),
}

/// Только значения, которые состояние читает из живых свойств игрока.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutomaticRestoreProperties {
    pub hp_recovery: u16,
    pub mp_recovery: u16,
    pub resume_hp_peace: i32,
    pub resume_hp_fight: i32,
    pub resume_mp_peace: i32,
    pub resume_mp_fight: i32,
    pub restored_hp_peace: i32,
    pub restored_hp_fight: i32,
    pub restored_mp_peace: i32,
    pub restored_mp_fight: i32,
}

impl From<crate::combat::PlayerCombatProperties> for AutomaticRestoreProperties {
    fn from(properties: crate::combat::PlayerCombatProperties) -> Self {
        Self {
            hp_recovery: properties.hp_recovery,
            mp_recovery: properties.mp_recovery,
            resume_hp_peace: properties.resume_hp_peace,
            resume_hp_fight: properties.resume_hp_fight,
            resume_mp_peace: properties.resume_mp_peace,
            resume_mp_fight: properties.resume_mp_fight,
            restored_hp_peace: properties.restored_hp_peace,
            restored_hp_fight: properties.restored_hp_fight,
            restored_mp_peace: properties.restored_mp_peace,
            restored_mp_fight: properties.restored_mp_fight,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutomaticRestoreState {
    kind: AutomaticRestoreKind,
    frequency_ms: u32,
    persisted_volume: u32,
    last_tick_ms: u32,
}

impl AutomaticRestoreState {
    const fn new(kind: AutomaticRestoreKind, frequency_ms: u32, persisted_volume: u32) -> Self {
        Self {
            kind,
            frequency_ms,
            persisted_volume,
            last_tick_ms: 0,
        }
    }

    pub const fn restored(properties: AutomaticRestoreProperties) -> [Self; 4] {
        [
            Self::new(
                AutomaticRestoreKind::HealthPeace,
                properties.resume_hp_peace as u32,
                (properties.restored_hp_peace as f32).to_bits(),
            ),
            Self::new(
                AutomaticRestoreKind::HealthFight,
                properties.resume_hp_fight as u32,
                properties.restored_hp_fight as u32,
            ),
            Self::new(
                AutomaticRestoreKind::ManaPeace,
                properties.resume_mp_peace as u32,
                (properties.restored_mp_peace as f32).to_bits(),
            ),
            Self::new(
                AutomaticRestoreKind::ManaFight,
                properties.resume_mp_fight as u32,
                properties.restored_mp_fight as u32,
            ),
        ]
    }

    pub fn region_entry_peace(
        resume_timer_ms: u32,
        properties: AutomaticRestoreProperties,
    ) -> [Self; 2] {
        [
            Self::new(
                AutomaticRestoreKind::HealthPeace,
                resume_timer_ms,
                (f32::from(properties.hp_recovery) * 0.001_f32).to_bits(),
            ),
            Self::new(
                AutomaticRestoreKind::ManaPeace,
                resume_timer_ms,
                (f32::from(properties.mp_recovery) * 0.001_f32).to_bits(),
            ),
        ]
    }

    pub fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Option<Self> {
        let mut reader = LegacyReader::at(payload, offset).ok()?;
        let state_id = reader.read_u32().ok()?;
        let kind = match state_id {
            AUTOMATIC_RESTORE_HP_PEACE_STATE_ID => AutomaticRestoreKind::HealthPeace,
            AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID => AutomaticRestoreKind::HealthFight,
            AUTOMATIC_RESTORE_MP_PEACE_STATE_ID => AutomaticRestoreKind::ManaPeace,
            AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID => AutomaticRestoreKind::ManaFight,
            _ => return None,
        };
        Some(Self {
            kind,
            frequency_ms: reader.read_u32().ok()?,
            persisted_volume: reader.read_u32().ok()?,
            last_tick_ms: now_ms,
        })
    }

    pub fn encoded_for_install(self) -> [u8; AUTOMATIC_RESTORE_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(AUTOMATIC_RESTORE_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.state_id());
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.persisted_volume);
        bytes
            .try_into()
            .expect("размер автоматического состояния восстановления фиксирован")
    }

    pub const fn state_id(self) -> u32 {
        match self.kind {
            AutomaticRestoreKind::HealthPeace => AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
            AutomaticRestoreKind::HealthFight => AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID,
            AutomaticRestoreKind::ManaPeace => AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
            AutomaticRestoreKind::ManaFight => AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID,
        }
    }

    pub const fn is_health(self) -> bool {
        matches!(
            self.kind,
            AutomaticRestoreKind::HealthPeace | AutomaticRestoreKind::HealthFight
        )
    }

    pub const fn should_check(
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

    pub const fn due(self, checked_at_ms: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_tick_ms) < checked_at_ms
    }

    pub fn apply(
        &mut self,
        recorded_at_ms: u32,
        properties: AutomaticRestoreProperties,
        health: u32,
        maximum_health: u32,
        mana: u32,
        maximum_mana: u32,
    ) -> Option<AutomaticRestoreMutation> {
        self.last_tick_ms = recorded_at_ms;
        let (current, maximum, amount, health_target) = match self.kind {
            AutomaticRestoreKind::HealthPeace => {
                (health, maximum_health, properties.restored_hp_peace, true)
            }
            AutomaticRestoreKind::HealthFight => {
                (health, maximum_health, properties.restored_hp_fight, true)
            }
            AutomaticRestoreKind::ManaPeace => {
                (mana, maximum_mana, properties.restored_mp_peace, false)
            }
            AutomaticRestoreKind::ManaFight => {
                (mana, maximum_mana, properties.restored_mp_fight, false)
            }
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
