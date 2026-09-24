//! Конфигурация объединения battle fairy из `cbattlefairyproperty.cpp/.h`,
//! подтверждённая `worldserver.exe` и `worldserver.pdb`.
//! Перенесена в Realm `content/`.
//!
//! После успешного открытия loader очищает список и возвращает true даже для
//! повреждённого хвоста. Один временный record переиспользуется: неполная
//! последняя запись наследует непрочитанные поля предыдущей и всё равно
//! добавляется.
//!
//! Wire subtype `0x2D` содержит signed count и сырые 0x7C-байтные MSVC records.
//! Rust воспроизводит SSO-layout строк длиной до 15 байт. Длинные строки
//! отклоняются: старый формат передавал межпроцессный heap pointer и не мог
//! корректно декодироваться. Caller-owned registry заменяет singleton.

use std::error::Error;
use std::fmt;
use std::path::Path;

const COMPOSE_RECORD_SIZE: usize = 0x7c;
const LEGACY_STRING_SIZE: usize = 0x1c;
const LEGACY_STRING_INLINE_CAPACITY: usize = 15;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BattleFairyCompose {
    pub fetch_stone: Vec<u8>,
    pub fetch_body: Vec<u8>,
    pub material: Vec<u8>,
    pub deplete_fetch: u32,
    pub success_rate: f32,
    pub battle_fairy: Vec<u8>,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BattleFairyLevel {
    pub equipment_level: i32,
    pub success_rate: f32,
    pub up_value: i32,
    pub total_value: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleFairyComposeWireError {
    CountOutsideLegacyRange {
        count: usize,
    },
    StringRequiresLegacyHeapPointer {
        record: usize,
        field: &'static str,
        length: usize,
    },
}

impl fmt::Display for BattleFairyComposeWireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutsideLegacyRange { count } => {
                write!(
                    formatter,
                    "compose count {count} не помещается в signed legacy long"
                )
            }
            Self::StringRequiresLegacyHeapPointer {
                record,
                field,
                length,
            } => write!(
                formatter,
                "compose record {record}, поле {field} длиной {length} требует непереносимый MSVC heap pointer"
            ),
        }
    }
}

impl Error for BattleFairyComposeWireError {}

pub struct CBattleFairyProperty {
    compose: Vec<BattleFairyCompose>,
    levels: Vec<BattleFairyLevel>,
}

impl CBattleFairyProperty {
    /// Создаёт два пустых vector-а; остальные scalar defaults равны нулю
    /// и не имеют consumer-а в matching World PDB.
    pub const fn with_constructor_defaults() -> Self {
        Self {
            compose: Vec::new(),
            levels: Vec::new(),
        }
    }

    pub fn compose(&self) -> &[BattleFairyCompose] {
        &self.compose
    }

    pub fn levels(&self) -> &[BattleFairyLevel] {
        &self.levels
    }

    pub fn load_combine_config_from_file(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        let source = std::fs::read(path)?;
        self.load_combine_config(&source);
        Ok(())
    }

    pub fn load_combine_config(&mut self, source: &[u8]) {
        self.compose.clear();
        let mut cursor = 0usize;
        let mut current = BattleFairyCompose::default();
        while let Some(marker) = source[cursor..].iter().position(|byte| *byte == b'#') {
            cursor += marker + 1;
            let mut failed = false;
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.fetch_stone);
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.fetch_body);
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.material);
            read_u32_field(source, &mut cursor, &mut failed, &mut current.deplete_fetch);
            read_f32_field(source, &mut cursor, &mut failed, &mut current.success_rate);
            read_bytes_field(source, &mut cursor, &mut failed, &mut current.battle_fairy);
            read_u32_field(source, &mut cursor, &mut failed, &mut current.index);
            self.compose.push(current.clone());
            if failed {
                break;
            }
        }
    }

    pub fn serialize_combine(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), BattleFairyComposeWireError> {
        let count = i32::try_from(self.compose.len()).map_err(|_| {
            BattleFairyComposeWireError::CountOutsideLegacyRange {
                count: self.compose.len(),
            }
        })?;
        for (record, compose) in self.compose.iter().enumerate() {
            validate_legacy_string(record, "strFetchStone", &compose.fetch_stone)?;
            validate_legacy_string(record, "strFetchBody", &compose.fetch_body)?;
            validate_legacy_string(record, "strMaterial", &compose.material)?;
            validate_legacy_string(record, "strBattleFairy", &compose.battle_fairy)?;
        }
        destination.extend_from_slice(&count.to_le_bytes());
        for compose in &self.compose {
            let mut record = [0u8; COMPOSE_RECORD_SIZE];
            write_legacy_string(&mut record, 0x00, &compose.fetch_stone);
            write_legacy_string(&mut record, 0x1c, &compose.fetch_body);
            write_legacy_string(&mut record, 0x38, &compose.material);
            record[0x54..0x58].copy_from_slice(&compose.deplete_fetch.to_le_bytes());
            record[0x58..0x5c].copy_from_slice(&compose.success_rate.to_bits().to_le_bytes());
            write_legacy_string(&mut record, 0x5c, &compose.battle_fairy);
            record[0x78..0x7c].copy_from_slice(&compose.index.to_le_bytes());
            destination.extend_from_slice(&record);
        }
        Ok(())
    }
}

fn next_token<'source>(source: &'source [u8], cursor: &mut usize) -> Option<&'source [u8]> {
    while source.get(*cursor).is_some_and(u8::is_ascii_whitespace) {
        *cursor += 1;
    }
    let start = *cursor;
    while source
        .get(*cursor)
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        *cursor += 1;
    }
    (start != *cursor).then_some(&source[start..*cursor])
}

fn read_bytes_field(source: &[u8], cursor: &mut usize, failed: &mut bool, out: &mut Vec<u8>) {
    if *failed {
        return;
    }
    let Some(token) = next_token(source, cursor) else {
        *failed = true;
        return;
    };
    out.clear();
    out.extend_from_slice(token);
}

fn read_u32_field(source: &[u8], cursor: &mut usize, failed: &mut bool, out: &mut u32) {
    if *failed {
        return;
    }
    let value = next_token(source, cursor)
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse().ok());
    match value {
        Some(value) => *out = value,
        None => *failed = true,
    }
}

fn read_f32_field(source: &[u8], cursor: &mut usize, failed: &mut bool, out: &mut f32) {
    if *failed {
        return;
    }
    let value = next_token(source, cursor)
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse().ok());
    match value {
        Some(value) => *out = value,
        None => *failed = true,
    }
}

fn validate_legacy_string(
    record: usize,
    field: &'static str,
    value: &[u8],
) -> Result<(), BattleFairyComposeWireError> {
    if value.len() <= LEGACY_STRING_INLINE_CAPACITY {
        return Ok(());
    }
    Err(
        BattleFairyComposeWireError::StringRequiresLegacyHeapPointer {
            record,
            field,
            length: value.len(),
        },
    )
}

fn write_legacy_string(record: &mut [u8; COMPOSE_RECORD_SIZE], offset: usize, value: &[u8]) {
    let buffer = offset + 4;
    record[buffer..buffer + value.len()].copy_from_slice(value);
    record[offset + 0x14..offset + 0x18].copy_from_slice(&(value.len() as u32).to_le_bytes());
    record[offset + 0x18..offset + LEGACY_STRING_SIZE]
        .copy_from_slice(&(LEGACY_STRING_INLINE_CAPACITY as u32).to_le_bytes());
}
