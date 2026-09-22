//! Канонический владелец четырёх автоматических состояний восстановления.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! исходные владельцы `CAutomaticRestoreHpState`,
//! `CAutomaticRestoreHpStateFight`, `CAutomaticRestoreMpState` и
//! `CAutomaticRestoreMpStateFight`. Сохранены порядок `HP peace → HP fight →
//! MP peace → MP fight`, строгое wrapping-сравнение таймера и второй вызов
//! часов, которым фиксируется момент срабатывания. Объём восстановления
//! читается из живого `PlayerCombatProperties`, как в исходных `AI`. Все
//! четыре бессрочных состояния входят в полный клиентский снимок с базовым
//! нулевым client-time. Общие `Unserialize/Serialize` по адресам
//! `0x004F9D80/0x005ECE70` задают 12-байтную DB-запись из ID, частоты и
//! сохранённого объёма. CPlayer::RestoreHpMp (0x004455D0) сначала вызывает
//! End всех четырёх типов вместе с Particular в общем порядке массива, затем
//! добавляет четыре свежих. Begin(nullptr, holder) не читает часы и оставляет
//! last_tick_ms=0 у свежего constructor; загруженный экземпляр сохраняет
//! собственный clock Unserialize 0x004F9D80. Peace-варианты сохраняют объём как биты
//! `float`, fight-варианты — как исходный `DWORD`. Живой `AI` по-прежнему
//! читает актуальный объём из свойств игрока.
//! HP AI (0x004FA8E0/0x004FA4D0) сначала проверяет смерть, HP/max HP и
//! peace/fight обычного CMoveShape; неподходящий RTTI игрока затем оставляет
//! состояние нетронутым без часов. MP AI (0x004FA130/0x004F9DB0) проверяет
//! CPlayer до этих gates и для non-player вызывает End без чтения часов.
//! Общий End 0x005EEBA0 только удаляет запись из живого sufferer без visual;
//! отсутствие sufferer не позволяет удалить чужую запись. В обоих случаях
//! новый общий dispatch передаёт один ключ существующей арены.
//! restart_automatic_restore_state переносит object Begin(NULL, holder)
//! четырёх owners (0x004FA860/0x004FA450/0x004FA0B0/0x004F9D00):
//! без guards, base Begin → visual SetRun(1), без Update/пакета и часов.
//! CPlayer::OnEnterRegion (0x0045A2C6/0x0045A376) до RestoreHpMp временно
//! создаёт только HP/MP peace: общий lResumeTimer и WORD recovery * 0.001_f32.
//! Точный x87 fmul читает float 0x3A83126F, затем fstp округляет аргумент
//! конструктора до float; его биты становятся persisted_volume без clock.

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;

pub(crate) const AUTOMATIC_RESTORE_STATE_BYTES: usize = 12;
pub(crate) const AUTOMATIC_RESTORE_HP_PEACE_STATE_ID: u32 = 0x186a2;
pub(crate) const AUTOMATIC_RESTORE_MP_PEACE_STATE_ID: u32 = 0x186a3;
pub(crate) const AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID: u32 = 0x186ad;
pub(crate) const AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID: u32 = 0x186ae;

pub(crate) const fn is_automatic_restore_state_id(state_id: u32) -> bool {
    matches!(
        state_id,
        AUTOMATIC_RESTORE_HP_PEACE_STATE_ID
            | AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID
            | AUTOMATIC_RESTORE_MP_PEACE_STATE_ID
            | AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID
    )
}

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
    persisted_volume: u32,
    last_tick_ms: u32,
}

pub(crate) fn restart_automatic_restore_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<AutomaticRestoreState>(key)).is_none()
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

impl AutomaticRestoreState {
    const fn new(
        kind: AutomaticRestoreKind,
        frequency_ms: u32,
        persisted_volume: u32,
    ) -> Self {
        Self {
            kind,
            frequency_ms,
            persisted_volume,
            last_tick_ms: 0,
        }
    }

    pub(crate) const fn restored(properties: PlayerCombatProperties) -> [Self; 4] {
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

    pub(crate) fn region_entry_peace(
        resume_timer_ms: u32,
        properties: PlayerCombatProperties,
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

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Option<Self> {
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

    pub(crate) fn encoded_for_install(self) -> [u8; AUTOMATIC_RESTORE_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(AUTOMATIC_RESTORE_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.state_id());
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.persisted_volume);
        bytes
            .try_into()
            .expect("размер автоматического состояния восстановления фиксирован")
    }


    pub(crate) const fn state_id(self) -> u32 {
        match self.kind {
            AutomaticRestoreKind::HealthPeace => AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
            AutomaticRestoreKind::HealthFight => AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID,
            AutomaticRestoreKind::ManaPeace => AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
            AutomaticRestoreKind::ManaFight => AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID,
        }
    }

    pub(crate) const fn is_health(self) -> bool {
        matches!(self.kind, AutomaticRestoreKind::HealthPeace | AutomaticRestoreKind::HealthFight)
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
