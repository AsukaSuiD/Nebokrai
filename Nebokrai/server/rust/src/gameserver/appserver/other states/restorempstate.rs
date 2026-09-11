//! Состояние восстановления маны `CRestoreMpState`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/other states/restorempstate.cpp`. Состояние хранит исходные
//! `DWORD` срока, частоты, прироста и счётчика. Живой игрок получает один шаг,
//! когда `frequency * count + started < now`; сложение выполняется с
//! переполнением и ограничивается текущим максимумом MP. После второго чтения
//! часов состояние завершается при строгом `time_to_keep + started < now`.
//! Смерть приостанавливает и шаги, и истечение. Exact EXE подтверждает общие
//! тела `Serialize/Unserialize` по `0x005F65F0/0x005EEC70`: DB-запись состоит
//! из ID, оставшегося срока, частоты и прироста. Vtable также направляет
//! `GetRemainedTime` на `0x005F2CD0`; visual-effect update `0x004F86F0`
//! сетевых пакетов не создаёт. Exact AI 0x004F8AA0 сначала проверяет RTTI
//! CPlayer: для живого non-player вызывается End без чтения часов даже при
//! нулевом HP. Только игрок проходит death-gate и два последовательных clock.
//! Общий End 0x005EEBA0 удаляет точный ключ sufferer без visual; отсутствие
//! sufferer не даёт права удалить запись другого владельца.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const RESTORE_MP_STATE_ID: i32 = 100_001;
pub(crate) const RESTORE_MP_STATE_BYTES: usize = 16;

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

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_i32()?;
        Ok(Self::new(
            reader.read_u32()?,
            reader.read_u32()?,
            reader.read_u32()?,
            0,
        ))
    }

    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; RESTORE_MP_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now_milliseconds) as u32)
    }

    pub(crate) fn encoded_for_install(self) -> [u8; RESTORE_MP_STATE_BYTES] {
        self.encoded_with_remaining(self.time_to_keep_ms)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; RESTORE_MP_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(RESTORE_MP_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(RESTORE_MP_STATE_ID);
        writer.write_u32(remaining_time_ms);
        writer.write_u32(self.frequency_ms);
        writer.write_u32(self.mana_gain);
        bytes.try_into().expect("размер состояния восстановления MP фиксирован")
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
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
