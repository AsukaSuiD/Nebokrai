//! Состояние восстановления здоровья `CRestoreHpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/restorehpstate.cpp`. Состояние хранит исходные
//! `DWORD` срока, частоты, прироста и счётчика. Живой игрок получает один шаг,
//! когда `frequency * count + started < now`; сложение выполняется с
//! переполнением и ограничивается текущим максимумом HP. После второго чтения
//! часов состояние завершается при строгом `time_to_keep + started < now`.
//! Смерть приостанавливает и шаги, и истечение. Vtable exact EXE подтверждает
//! общий `CBlindState::GetRemainedTime` по `0x005F2CD0`; visual-effect update
//! `0x004F86F0` сетевых пакетов не создаёт.

use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const RESTORE_HP_STATE_ID: i32 = 100_000;

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
