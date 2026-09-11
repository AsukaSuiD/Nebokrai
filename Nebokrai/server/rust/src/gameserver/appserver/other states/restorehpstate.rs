//! Состояние восстановления здоровья `CRestoreHpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/restorehpstate.cpp`. Состояние хранит исходные
//! `DWORD` срока, частоты, прироста и счётчика. Живой CMoveShape получает один шаг,
//! когда `frequency * count + started < now`; сложение выполняется с
//! переполнением и ограничивается текущим максимумом HP. После второго чтения
//! часов состояние завершается при строгом `time_to_keep + started < now`.
//! Смерть приостанавливает и шаги, и истечение. Exact EXE подтверждает общие
//! тела `Serialize/Unserialize` по `0x005F65F0/0x005EEC70`: DB-запись состоит
//! из ID, оставшегося срока, частоты и прироста. Vtable также направляет
//! `GetRemainedTime` на `0x005F2CD0`; visual-effect update `0x004F86F0`
//! сетевых пакетов не создаёт. Exact AI 0x004F8650 не содержит CPlayer RTTI:
//! HP читается и записывается virtual slots +0xD0/+0xD8/+0xD4, затем вызывается
//! OnChangeStates +0x164. Поэтому общий ключ лечит также монстра и постройку;
//! NPC vtable 0x0065DA1C возвращает HP=0 и останавливается на death-gate.
//! Vtable Monster 0x00652AE4, Build 0x0065E704 и CityGate 0x0065E8C4
//! направляют публикацию на базовый 0x004CD3E0 (BFE02), без player team-route.
//! End 0x005EEBA0 разрешает sufferer и удаляет только его точный ключ,
//! без visual; свойства игрока пересчитываются после фактического удаления.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const RESTORE_HP_STATE_ID: i32 = 100_000;
pub(crate) const RESTORE_HP_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RestoreHpState {
    time_to_keep_ms: u32,
    frequency_ms: u32,
    health_gain: u32,
    restore_count: u32,
    started_at_ms: u32,
}

impl RestoreHpState {
    pub(crate) const fn new(
        time_to_keep_ms: u32,
        frequency_ms: u32,
        health_gain: u32,
        started_at_ms: u32,
    ) -> Self {
        Self {
            time_to_keep_ms,
            frequency_ms,
            health_gain,
            restore_count: 0,
            started_at_ms,
        }
    }

    pub(crate) const fn state_id(self) -> i32 {
        RESTORE_HP_STATE_ID
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_i32()?;
        Ok(Self::new(
            reader.read_u32()?,
            reader.read_u32()?,
            reader.read_u32()?,
            now_ms,
        ))
    }

    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; RESTORE_HP_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now_milliseconds) as u32)
    }

    pub(crate) fn encoded_for_install(self) -> [u8; RESTORE_HP_STATE_BYTES] {
        self.encoded_with_remaining(self.time_to_keep_ms)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; RESTORE_HP_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(RESTORE_HP_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(RESTORE_HP_STATE_ID);
        writer.write_u32(remaining_time_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.health_gain);
        bytes.try_into().expect("размер состояния восстановления HP фиксирован")
    }


    pub(crate) fn reset_restore_count(&mut self) {
        self.restore_count = 0;
    }

    pub(crate) fn client_state_time(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.time_to_keep_ms,
            now_milliseconds,
        ) as i32
    }

    pub(crate) fn tick(
        &mut self,
        checked_at_ms: u32,
        current_health: u32,
        maximum_health: u32,
    ) -> Option<u32> {
        let due_at_ms = self
            .frequency_ms
            .wrapping_mul(self.restore_count)
            .wrapping_add(self.started_at_ms);
        if due_at_ms >= checked_at_ms {
            return None;
        }
        self.restore_count = self.restore_count.wrapping_add(1);
        Some(current_health.wrapping_add(self.health_gain).min(maximum_health))
    }

    pub(crate) const fn expired(self, checked_at_ms: u32) -> bool {
        self.time_to_keep_ms
            .wrapping_add(self.started_at_ms)
            < checked_at_ms
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp

// ============================================================================
// FUNCTION: CRestoreHpState::CRestoreHpState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp:27
// RVA: 0x000F84A0
// ADDRESS: 004f84a0
// PROTOTYPE: undefined __thiscall CRestoreHpState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CRestoreHpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp:60
// RVA: 0x000F8520
// ADDRESS: 004f8520
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CRestoreHpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorehpstate.cpp:69
// RVA: 0x000F85B0
// ADDRESS: 004f85b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
