//! Состояние восстановления маны `CRestoreMpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/restorempstate.cpp`. Состояние хранит исходные
//! `DWORD` срока, частоты, прироста и счётчика. Живой игрок получает один шаг,
//! когда `frequency * count + started < now`; сложение выполняется с
//! переполнением и ограничивается текущим максимумом MP. После второго чтения
//! часов состояние завершается при строгом `time_to_keep + started < now`.
//! Смерть приостанавливает и шаги, и истечение.

pub(crate) const RESTORE_MP_STATE_ID: i32 = 100_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RestoreMpState {
    time_to_keep_ms: u32,
    frequency_ms: u32,
    mana_gain: u32,
    restore_count: u32,
    started_at_ms: u32,
}

impl RestoreMpState {
    pub(crate) const fn new(
        time_to_keep_ms: u32,
        frequency_ms: u32,
        mana_gain: u32,
        started_at_ms: u32,
    ) -> Self {
        Self {
            time_to_keep_ms,
            frequency_ms,
            mana_gain,
            restore_count: 0,
            started_at_ms,
        }
    }

    pub(crate) const fn state_id(self) -> i32 {
        RESTORE_MP_STATE_ID
    }

    pub(crate) fn tick(
        &mut self,
        checked_at_ms: u32,
        current_mana: u32,
        maximum_mana: u32,
    ) -> Option<u32> {
        let due_at_ms = self
            .frequency_ms
            .wrapping_mul(self.restore_count)
            .wrapping_add(self.started_at_ms);
        if due_at_ms >= checked_at_ms {
            return None;
        }
        self.restore_count = self.restore_count.wrapping_add(1);
        Some(current_mana.wrapping_add(self.mana_gain).min(maximum_mana))
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp

// ============================================================================
// FUNCTION: CRestoreMpState::CRestoreMpState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp:27
// RVA: 0x000F8840
// ADDRESS: 004f8840
// PROTOTYPE: undefined __thiscall CRestoreMpState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CRestoreMpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp:60
// RVA: 0x000F88C0
// ADDRESS: 004f88c0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// ============================================================================
// FUNCTION: CRestoreMpState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\restorempstate.cpp:69
// RVA: 0x000F8950
// ADDRESS: 004f8950
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================

// COMPONENT_VARIANT_END: GameServer
