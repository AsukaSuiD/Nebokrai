//! Описание одного навыка, передаваемое из WorldServer в GameServer.
//!
//!
//! EXE разрешает противоречие с очищенным C++ reference: wire начинается с
//! `type, id, level, target`, имя передаётся как `length + bytes` без NUL,
//! описание вообще не передаётся, затем идут `usage_count` и пары
//! `(usage, cost)`. Поля — little-endian 32-битные слова.
//!
//! Исторический serializer объявлял длину на восемь байт больше записанных
//! полей. Точный Game `CSkillFactory::Rebuild` использует эту длину как границу
//! отдельного record-а, читает только известные поля и пропускает остаток,
//! поэтому padding является частью framing. Старый World отправлял в нём
//! неинициализированные байты heap; Rust сохраняет восемь байт, но заполняет их
//! нулями, исключая внутреннюю утечку без изменения принимаемой структуры.
//!
//! Фиксированные `char[256]` заменены bounded byte strings. Вход трактуется как
//! C-строка до первого NUL и не может переполнить owner; `Vec` и `Drop`
//! заменяют ручное владение usage-массивом.

use std::error::Error;
use std::fmt;

pub(crate) const UNKNOWN_SKILL_TYPE: u32 = 0x7fff_ffff;
const MAX_SKILL_TEXT_LENGTH: usize = 255;
const SKILL_WIRE_FIXED_LENGTH: usize = 24;
const SKILL_WIRE_TRAILING_PADDING: usize = 8;

/// Одна точная восьмибайтная пара `tagUsage`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillUsage {
    pub(crate) usage: u32,
    pub(crate) cost: u32,
}

/// Safe owner полей исходного `CSkill`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CSkill {
    usages: Vec<SkillUsage>,
    name: Vec<u8>,
    description: Vec<u8>,
    skill_type: u32,
    skill_id: u32,
    level: u32,
    is_target_self: i32,
}

impl CSkill {
    pub(crate) fn new(skill_id: i32) -> Self {
        Self {
            usages: Vec::new(),
            name: Vec::new(),
            description: Vec::new(),
            skill_type: UNKNOWN_SKILL_TYPE,
            skill_id: skill_id as u32,
            level: 0,
            is_target_self: 0,
        }
    }

    pub(crate) fn skill_id(&self) -> u32 {
        self.skill_id
    }

    pub(crate) fn level(&self) -> u32 {
        self.level
    }

 /// Точный composite key World cache: `id << 16 | level & 0xffff`.
    pub(crate) fn cache_key(&self) -> u32 {
        self.skill_id.wrapping_shl(16) | (self.level & 0xffff)
    }

    pub(crate) fn set_skill_type(&mut self, skill_type: i32) {
        self.skill_type = skill_type as u32;
    }

    pub(crate) fn set_level(&mut self, level: i32) {
        self.level = level as u32;
    }

    pub(crate) fn set_skill_name(&mut self, name: &[u8]) -> Result<(), SkillTextError> {
        set_bounded_c_string(&mut self.name, name, SkillTextField::Name)
    }

 /// Описание сохраняется для owner-а, но намеренно отсутствует в wire.
    pub(crate) fn set_description(&mut self, description: &[u8]) -> Result<(), SkillTextError> {
        set_bounded_c_string(
            &mut self.description,
            description,
            SkillTextField::Description,
        )
    }

    pub(crate) fn set_target_type(&mut self, is_target_self: i32) {
        self.is_target_self = is_target_self;
    }

    pub(crate) fn add_usage(&mut self, usage: SkillUsage) {
        self.usages.push(usage);
    }

 /// Возвращает `None` для invalid-state: unknown type либо нулевой ID.
    pub(crate) fn serialize(&self) -> Result<Option<Vec<u8>>, SkillSerializeError> {
        if self.skill_type == UNKNOWN_SKILL_TYPE || self.skill_id == 0 {
            return Ok(None);
        }

        let usage_count =
            u32::try_from(self.usages.len()).map_err(|_| SkillSerializeError::TooManyUsages {
                count: self.usages.len(),
            })?;
        let usage_bytes =
            self.usages
                .len()
                .checked_mul(8)
                .ok_or(SkillSerializeError::RecordTooLarge {
                    name_length: self.name.len(),
                    usage_count: self.usages.len(),
                })?;
        let wire_length = SKILL_WIRE_FIXED_LENGTH
            .checked_add(self.name.len())
            .and_then(|length| length.checked_add(usage_bytes))
            .and_then(|length| length.checked_add(SKILL_WIRE_TRAILING_PADDING))
            .ok_or(SkillSerializeError::RecordTooLarge {
                name_length: self.name.len(),
                usage_count: self.usages.len(),
            })?;
        u32::try_from(wire_length).map_err(|_| SkillSerializeError::RecordTooLarge {
            name_length: self.name.len(),
            usage_count: self.usages.len(),
        })?;

        let mut output = Vec::with_capacity(wire_length);
        output.extend_from_slice(&self.skill_type.to_le_bytes());
        output.extend_from_slice(&self.skill_id.to_le_bytes());
        output.extend_from_slice(&self.level.to_le_bytes());
        output.extend_from_slice(&self.is_target_self.to_le_bytes());
        output.extend_from_slice(&(self.name.len() as u32).to_le_bytes());
        output.extend_from_slice(&self.name);
        output.extend_from_slice(&usage_count.to_le_bytes());
        for usage in &self.usages {
            output.extend_from_slice(&usage.usage.to_le_bytes());
            output.extend_from_slice(&usage.cost.to_le_bytes());
        }
        debug_assert_eq!(output.len() + SKILL_WIRE_TRAILING_PADDING, wire_length);
        output.resize(wire_length, 0);
        Ok(Some(output))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SkillTextField {
    Name,
    Description,
}

impl fmt::Display for SkillTextField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Name => "имя навыка",
            Self::Description => "описание навыка",
        })
    }
}

/// Вместо исходного безграничного `strcpy`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SkillTextError {
    field: SkillTextField,
    length: usize,
}

impl fmt::Display for SkillTextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} содержит {} байт при максимуме {}",
            self.field, self.length, MAX_SKILL_TEXT_LENGTH
        )
    }
}

impl Error for SkillTextError {}

/// Невозможный в 32-битном wire размер skill record-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SkillSerializeError {
    TooManyUsages {
        count: usize,
    },
    RecordTooLarge {
        name_length: usize,
        usage_count: usize,
    },
}

impl fmt::Display for SkillSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyUsages { count } => {
                write!(formatter, "навык содержит {count} usage-записей вне u32")
            }
            Self::RecordTooLarge {
                name_length,
                usage_count,
            } => write!(
                formatter,
                "skill record с именем {name_length} байт и {usage_count} usage-записями не помещается в u32"
            ),
        }
    }
}

impl Error for SkillSerializeError {}

fn set_bounded_c_string(
    destination: &mut Vec<u8>,
    source: &[u8],
    field: SkillTextField,
) -> Result<(), SkillTextError> {
    let bytes = &source[..source
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(source.len())];
    if bytes.len() > MAX_SKILL_TEXT_LENGTH {
        return Err(SkillTextError {
            field,
            length: bytes.len(),
        });
    }
    destination.clear();
    destination.extend_from_slice(bytes);
    Ok(())
}

// материализованного owner-а, а не как Rust-реализация.
