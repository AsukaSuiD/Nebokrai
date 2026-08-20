//! Ordered emotion setup исторического Miracle.
//!
//! Статус World `LoadSetup` RVA `0x000A0CA0` и `Serialize` RVA
//! `0x000A03F0`: `IMPLEMENTED`; Game decoder ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\setup\emotion.cpp:21,46`.
//!
//! World loader не очищает static map: каждая `*`-запись заменяет только свой
//! signed ID, остальные прежние keys сохраняются. Отсутствующий файл возвращал
//! `0` без мутации, открытый файл — `1` даже без записей. Это отличается от
//! очищенного C++ reference с transactional replacement и проверкой non-empty;
//! Rust сохраняет exact merge/state-transition, но использует `std::fs` и
//! caller-owned `BTreeMap`.
//!
//! Wire — signed `long` count, затем пары signed `long id/value` в ascending
//! signed-key порядке. Повреждённая formatted запись исходно могла вставить
//! неинициализированные locals; безопасный parser вместо этого возвращает
//! typed error после уже полностью применённых записей. Универсальные parser,
//! map и file lifetime отданы стандартной библиотеке; собственным остаётся
//! только Miracle token/wire контракт.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::public::readwrite::read_to;

/// Value-owner исходного `std::map<long, int>`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CEmotion {
    emotions: BTreeMap<i32, i32>,
}

impl CEmotion {
    /// Читает файл без предварительной мутации; отсутствие сохраняет map.
    pub(crate) fn load_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<usize, EmotionFileLoadError> {
        let source = std::fs::read(path).map_err(EmotionFileLoadError::Io)?;
        self.load_from_bytes(&source)
            .map_err(EmotionFileLoadError::Format)
    }

    /// Применяет exact ordered insert-or-replace для каждой `*`-записи.
    pub(crate) fn load_from_bytes(&mut self, source: &[u8]) -> Result<usize, EmotionFormatError> {
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());
        let mut applied = 0;
        while read_to(&mut tokens, b"*") {
            let emotion_id = read_i32(&mut tokens, "emotion ID")?;
            let value = read_i32(&mut tokens, "emotion value")?;
            self.emotions.insert(emotion_id, value);
            applied += 1;
        }
        Ok(applied)
    }

    /// Возвращает повторяющее значение либо исходный ноль отсутствия.
    pub(crate) fn repeated(&self, emotion_id: i32) -> i32 {
        self.emotions.get(&emotion_id).copied().unwrap_or(0)
    }

    /// Дописывает exact `count + ordered (id, value)` wire.
    pub(crate) fn serialize(&self, destination: &mut Vec<u8>) -> Result<(), EmotionSerializeError> {
        let count = i32::try_from(self.emotions.len()).map_err(|_| EmotionSerializeError {
            count: self.emotions.len(),
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        for (&emotion_id, &value) in &self.emotions {
            destination.extend_from_slice(&emotion_id.to_le_bytes());
            destination.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }
}

/// Ошибка безопасного formatted parser-а.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EmotionFormatError {
    UnexpectedEnd { field: &'static str },
    InvalidLong { field: &'static str, token: Vec<u8> },
}

impl fmt::Display for EmotionFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { field } => {
                write!(formatter, "после маркера отсутствует поле {field}")
            }
            Self::InvalidLong { field, token } => write!(
                formatter,
                "поле {field} не является signed long: {}",
                String::from_utf8_lossy(token)
            ),
        }
    }
}

impl Error for EmotionFormatError {}

/// Ошибка file-adapter-а с сохранённым источником.
#[derive(Debug)]
pub(crate) enum EmotionFileLoadError {
    Io(std::io::Error),
    Format(EmotionFormatError),
}

impl fmt::Display for EmotionFileLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Format(error) => error.fmt(formatter),
        }
    }
}

impl Error for EmotionFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Format(error) => Some(error),
        }
    }
}

/// Невозможный в MSVC32 signed count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EmotionSerializeError {
    pub(crate) count: usize,
}

impl fmt::Display for EmotionSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "emotion map содержит {} записей вне signed 32-битного диапазона",
            self.count
        )
    }
}

impl Error for EmotionSerializeError {}

fn read_i32<'a>(
    tokens: &mut impl Iterator<Item = &'a [u8]>,
    field: &'static str,
) -> Result<i32, EmotionFormatError> {
    let token = tokens
        .next()
        .ok_or(EmotionFormatError::UnexpectedEnd { field })?;
    let text = std::str::from_utf8(token).map_err(|_| EmotionFormatError::InvalidLong {
        field,
        token: token.to_vec(),
    })?;
    text.parse::<i32>()
        .map_err(|_| EmotionFormatError::InvalidLong {
            field,
            token: token.to_vec(),
        })
}

// Остальной сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\emotion.cpp

// ============================================================================
// FUNCTION: CEmotion::IsEmotionRepeated
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\emotion.cpp:74
// RVA: 0x000D7870
// ADDRESS: 004d7870
// PROTOTYPE: int __cdecl IsEmotionRepeated(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEmotion::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\emotion.cpp:60
// RVA: 0x000D80C0
// ADDRESS: 004d80c0
// PROTOTYPE: int __cdecl Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\setup\emotion.cpp

// ============================================================================
// FUNCTION: CEmotion::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\emotion.cpp:46
// RVA: 0x000A03F0
// ADDRESS: 004a03f0
// PROTOTYPE: int __cdecl Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEmotion::LoadSetup
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\setup\emotion.cpp:21
// RVA: 0x000A0CA0
// ADDRESS: 004a0ca0
// PROTOTYPE: int __cdecl LoadSetup(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
