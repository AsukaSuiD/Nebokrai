//! Объявления по appserver/script/variablelist.cpp/.h: LoadVarList, GetArrayNum.
//! Game RVA 0xadc30/0xad840; правила и границы — docs/gameplay/scripting.md.
//! Сложные выражения размера требуют исполнителя; здесь поддержан десятичный литерал.

use super::ini::{IniListError, IniListErrorKind, IniRecord, IniRecords, decimal_i32};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableListErrorKind {
    Syntax(IniListErrorKind),
    MissingArrayClose,
    UnsupportedArrayExpression,
    NegativeArrayLength,
    ArraySizeOverflow,
    StringTooShort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VariableListError {
    pub offset: usize,
    pub kind: VariableListErrorKind,
}

impl From<IniListError> for VariableListError {
    fn from(error: IniListError) -> Self {
        Self {
            offset: error.offset,
            kind: VariableListErrorKind::Syntax(error.kind),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableDefault<'a> {
    Integer(i32),
    String(&'a [u8]),
    IntegerArray { length: usize, value: i32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VariableDefinition<'a> {
    pub offset: usize,
    pub name: &'a [u8],
    pub value: VariableDefault<'a>,
}

pub struct VariableListRecords<'a> {
    records: IniRecords<'a>,
    finished: bool,
}

impl<'a> VariableListRecords<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self, VariableListError> {
        Ok(Self {
            records: IniRecords::new(source, b"VariableList")?,
            finished: false,
        })
    }
}

impl<'a> Iterator for VariableListRecords<'a> {
    type Item = Result<VariableDefinition<'a>, VariableListError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let record = self
            .records
            .next()?
            .map_err(VariableListError::from)
            .and_then(definition);
        self.finished = record.is_err();
        Some(record)
    }
}

fn definition(record: IniRecord<'_>) -> Result<VariableDefinition<'_>, VariableListError> {
    let error = |kind| VariableListError {
        offset: record.offset,
        kind,
    };
    let mut name = record.caption;
    if let Some(open) = name.iter().position(|b| *b == b'[') {
        let expression = &name[open + 1..];
        let close = expression
            .iter()
            .position(|b| *b == b']')
            .ok_or_else(|| error(VariableListErrorKind::MissingArrayClose))?;
        let expression = &expression[..close];
        // GetArrayNum запускает CScript::RunLine. Не заменяем выражение его числовым префиксом.
        if expression.is_empty() || !expression.iter().all(u8::is_ascii_digit) {
            return Err(error(VariableListErrorKind::UnsupportedArrayExpression));
        }
        let length = decimal_i32(expression);
        if length < 0 {
            return Err(error(VariableListErrorKind::NegativeArrayLength));
        }
        if length > 0 {
            if (length as u32).checked_mul(4).is_none() {
                return Err(error(VariableListErrorKind::ArraySizeOverflow));
            }
            // Размер берётся из первой пары скобок; имя обрезается на последней '['.
            name = &name[..name
                .iter()
                .rposition(|b| *b == b'[')
                .expect("скобка найдена")];
            return Ok(VariableDefinition {
                offset: record.offset,
                name,
                value: VariableDefault::IntegerArray {
                    length: length as usize,
                    // CIni::ReadInt возвращает это значение при отсутствии '=' до CR/конца.
                    value: record.value.map_or(-99_999_999, decimal_i32),
                },
            });
        }
        // Нулевой размер идёт в scalar/string-ветвь без обрезания имени.
    }
    let text = record.value.ok_or_else(|| {
        error(VariableListErrorKind::Syntax(
            IniListErrorKind::MissingValueSeparator,
        ))
    })?;
    let value = if text.first() == Some(&b'"') {
        if text.len() < 2 {
            return Err(error(VariableListErrorKind::StringTooShort));
        }
        // Оригинал убирает первый и последний байт без проверки закрывающей кавычки.
        VariableDefault::String(&text[1..text.len() - 1])
    } else {
        VariableDefault::Integer(decimal_i32(text))
    };
    Ok(VariableDefinition {
        offset: record.offset,
        name,
        value,
    })
}
