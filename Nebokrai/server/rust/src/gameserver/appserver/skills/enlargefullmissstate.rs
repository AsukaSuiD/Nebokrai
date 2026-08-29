//! Каноническая достигнутая часть `CEnlargeFullMissState`.
//!
//! Для игрока состояние `603` прибавляет младшие 16 бит знакового параметра
//! к `full_miss` точным WORD-сложением с переполнением. Собственного
//! визуального сообщения и таймера нет. DB-запись состоит ровно из
//! little-endian ID и знаковой 32-битной прибавки.

use super::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub(crate) const ENLARGE_FULL_MISS_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnlargeFullMissState {
    gain: i32,
}

impl EnlargeFullMissState {
    pub(crate) const fn new(gain: i32) -> Self { Self { gain } }
    pub(crate) const fn skill_id(self) -> u32 { ENLARGE_FULL_MISS_SKILL_ID }
    pub(crate) const fn apply(self, value: u16) -> u16 {
        value.wrapping_add(self.gain as u16)
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ENLARGE_FULL_MISS_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(reader.read_i32()?))
    }

    pub(crate) fn encoded(self) -> [u8; ENLARGE_FULL_MISS_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(ENLARGE_FULL_MISS_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(ENLARGE_FULL_MISS_SKILL_ID);
        writer.write_i32(self.gain);
        bytes.try_into().expect("размер состояния полного уклонения фиксирован")
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.h

// ============================================================================
// FUNCTION: CEnlargeFullMissState::CEnlargeFullMissState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:24
// RVA: 0x001E2080
// ADDRESS: 005e2080
// PROTOTYPE: undefined __thiscall CEnlargeFullMissState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
