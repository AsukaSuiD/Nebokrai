//! Переменные `CVariableList` из `variablelist.cpp`.
//!
//! Loader очищает список и читает scalar, array и quoted string definitions.
//! DB load заменяет current/saved значения первого byte-sensitive имени и может
//! заменить объявленный array строкой. Integer setter использует ASCII case-
//! insensitive имя и index 0; string setter меняет первое строковое совпадение.
//!
//! Wire — signed variable count, payload length и records: C-string name,
//! signed tag, затем `i32`, C-string либо tag элементов `i32`. Отрицательный
//! tag строки включает NUL. Typed enum заменяет C++ unions; недопустимая смена
//! scalar/array в string блокируется до состояния с dangling pointer.
//!
//! Save делегирует тому же `CRsGenVar` и caller connection без собственной
//! DB-семантики.

use std::error::Error;
use std::fmt;

use crate::persistence::rsgenvar::{
    GenVarLoadOutcome, GenVarSaveOutcome, RsGenVarOwner,
};
use crate::persistence::rssetup::WorldTdsClient;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariableValue {
    Integer { current: i32, saved: i32 },
    String { current: Vec<u8>, saved: Vec<u8> },
    IntegerArray { current: Vec<i32>, saved: Vec<i32> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariableEntry {
    pub name: Vec<u8>,
    pub value: VariableValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CVariableList {
    variables: Vec<VariableEntry>,
}

impl Default for CVariableList {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CVariableList {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

 /// Освобождает все записи переменных и публикует пустое состояние.
 ///
 /// Rust `Vec` и `VariableValue` освобождают имя и значения без ранних
 /// возвратов и утечек старого owner-а.
    pub fn release(&mut self) {
        self.variables.clear();
    }

 /// Возвращает имя C-строки без индексного суффикса от последней `[`.
 ///
 /// Оригинал сначала копировал вход в выходной буфер вызывающего, затем
 /// ставил NUL на последней `[` независимо от наличия закрывающей `]`.
 /// Rust возвращает владеющий массив байтов и не меняет входной срез.
    pub fn get_array_name(name: &[u8]) -> Vec<u8> {
        let name = visible_c_string(name);
        let end = name
            .iter()
            .rposition(|byte| *byte == b'[')
            .unwrap_or(name.len());
        name[..end].to_vec()
    }

 /// Создаёт действующий `LoadVarList` без оригинал allocation и union.
 ///
 /// `CIni::GetContinueDataNum` брал только непрерывный блок строк после
 /// index `GeneralVariableList`; пустая или отсутствующая resource поэтому
 /// публикует пустой список, как и исходный void owner.
    pub fn load_var_list(&mut self, source: Option<&[u8]>) -> VariableListLoadReport {
        self.release();
        let Some(source) = source else {
            return VariableListLoadReport::default();
        };

        let mut report = VariableListLoadReport {
            resource_found: true,
            ..VariableListLoadReport::default()
        };
        for (name, value) in general_variable_records(source) {
            let (name, array_length) = split_array_name(name);
            let value = trim_ascii(value);
            let entry = match array_length {
                Some(length) => {
                    let parsed = legacy_atoi(value);
                    VariableEntry {
                        name: name.to_vec(),
                        value: VariableValue::IntegerArray {
                            current: vec![parsed; length],
                            saved: vec![parsed; length],
                        },
                    }
                }
                None if value.first() == Some(&b'\"') => {
                    // EXE копировал всё между первым символом и последней
                    // позицией строки, не требуя парной closing quote.
                    let string = value
                        .get(1..value.len().saturating_sub(1))
                        .unwrap_or_default()
                        .to_vec();
                    VariableEntry {
                        name: name.to_vec(),
                        value: VariableValue::String {
                            current: string.clone(),
                            saved: string,
                        },
                    }
                }
                None => {
                    let parsed = legacy_atoi(value);
                    VariableEntry {
                        name: name.to_vec(),
                        value: VariableValue::Integer {
                            current: parsed,
                            saved: parsed,
                        },
                    }
                }
            };
            self.variables.push(entry);
            report.loaded_variables += 1;
        }
        report
    }

 /// Точный `CVariableList::LoadOneVar`: первая byte-sensitive запись с
 /// совпавшим именем получает DB SValue/CValue; array-объявления становятся
 /// scalar/string, как это делал исходный owner.
    pub fn load_one_var(
        &mut self,
        name: &[u8],
        saved_value: &[u8],
        current_value: &[u8],
    ) -> VariableDatabaseLoadDisposition {
        let Some(variable) = self.variables.iter_mut().find(|variable| variable.name == name) else {
            return VariableDatabaseLoadDisposition::NameNotDeclared;
        };
        let saved_value = visible_c_string(saved_value);
        let current_value = visible_c_string(current_value);
        if saved_value.first() == Some(&b'\"') || current_value.first() == Some(&b'\"') {
            let saved = unquote_legacy_value(saved_value);
            let current = unquote_legacy_value(current_value);
            variable.value = VariableValue::String { current, saved };
            return VariableDatabaseLoadDisposition::LoadedString;
        }
        variable.value = VariableValue::Integer {
            current: legacy_atoi(current_value),
            saved: legacy_atoi(saved_value),
        };
        VariableDatabaseLoadDisposition::LoadedInteger
    }

    pub fn push(&mut self, variable: VariableEntry) {
        self.variables.push(variable);
    }

    pub fn variables(&self) -> &[VariableEntry] {
        &self.variables
    }

    pub fn set_zero_index_integer(
        &mut self,
        name: &[u8],
        value: i32,
    ) -> VariableSetOutcome {
        let name = visible_c_string(name);
        for (variable_index, variable) in self.variables.iter_mut().enumerate() {
            if !visible_c_string(&variable.name).eq_ignore_ascii_case(name) {
                continue;
            }
            match &mut variable.value {
                VariableValue::Integer { current, .. } => {
                    *current = value;
                    return VariableSetOutcome::Updated { variable_index };
                }
                VariableValue::IntegerArray { current, .. } => {
                    let Some(first) = current.first_mut() else {
                        return VariableSetOutcome::Blocked(
                            VariableSetBlock::EmptyIntegerArray { variable_index },
                        );
                    };
                    *first = value;
                    return VariableSetOutcome::Updated { variable_index };
                }
                VariableValue::String { .. } => {}
            }
        }
        VariableSetOutcome::NotFound
    }

    pub fn set_string(&mut self, name: &[u8], value: &[u8]) -> VariableSetOutcome {
        let name = visible_c_string(name);
        let value = visible_c_string(value);
        for (variable_index, variable) in self.variables.iter_mut().enumerate() {
            if !visible_c_string(&variable.name).eq_ignore_ascii_case(name) {
                continue;
            }
            let VariableValue::String { current, .. } = &mut variable.value else {
                return VariableSetOutcome::Blocked(
                    VariableSetBlock::StringWriteWouldRetypeSavedUnion { variable_index },
                );
            };
            current.clear();
            current.extend_from_slice(value);
            return VariableSetOutcome::Updated { variable_index };
        }
        VariableSetOutcome::NotFound
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), VariableListSerializationBlock> {
        let count = i32::try_from(self.variables.len()).map_err(|_| {
            VariableListSerializationBlock::VariableCountOutOfRange {
                count: self.variables.len(),
            }
        })?;
        let mut payload = Vec::new();
        for (variable_index, variable) in self.variables.iter().enumerate() {
            write_variable_c_string(
                &mut payload,
                variable_index,
                VariableStringField::Name,
                &variable.name,
            )?;
            match &variable.value {
                VariableValue::Integer { current, .. } => {
                    payload.extend_from_slice(&0_i32.to_le_bytes());
                    payload.extend_from_slice(&current.to_le_bytes());
                }
                VariableValue::String { current, .. } => {
                    let wire_length = current.len().checked_add(1).ok_or(
                        VariableListSerializationBlock::StringLengthOutOfRange {
                            variable_index,
                            length: current.len(),
                        },
                    )?;
                    let wire_length = i32::try_from(wire_length).map_err(|_| {
                        VariableListSerializationBlock::StringLengthOutOfRange {
                            variable_index,
                            length: current.len(),
                        }
                    })?;
                    payload.extend_from_slice(&(-wire_length).to_le_bytes());
                    write_variable_c_string(
                        &mut payload,
                        variable_index,
                        VariableStringField::Value,
                        current,
                    )?;
                }
                VariableValue::IntegerArray { current, .. } => {
                    if current.is_empty() {
                        return Err(VariableListSerializationBlock::EmptyArray {
                            variable_index,
                        });
                    }
                    let array_length = i32::try_from(current.len()).map_err(|_| {
                        VariableListSerializationBlock::ArrayLengthOutOfRange {
                            variable_index,
                            length: current.len(),
                        }
                    })?;
                    payload.extend_from_slice(&array_length.to_le_bytes());
                    for value in current {
                        payload.extend_from_slice(&value.to_le_bytes());
                    }
                }
            }
        }
        let payload_length = i32::try_from(payload.len()).map_err(|_| {
            VariableListSerializationBlock::PayloadLengthOutOfRange {
                length: payload.len(),
            }
        })?;
        destination.extend_from_slice(&count.to_le_bytes());
        destination.extend_from_slice(&payload_length.to_le_bytes());
        destination.extend_from_slice(&payload);
        Ok(())
    }

 /// Точный `LoadVarData`: передаёт уже загруженный конфигурационный список
 /// DB-owner-у и не меняет его bool-результат.
 ///
 /// Старый код находил `CGame::m_pRsGenVar` через singleton и игнорировал
 /// итог `CRsGenVar::Load`; Rust делает владельца явным, поэтому caller
 /// может сохранить это игнорирование или наблюдать результат отдельно.
    pub async fn load_var_data<O: RsGenVarOwner>(
        &mut self,
        database: &mut O,
    ) -> GenVarLoadOutcome {
        database.load_general_variables(self).await
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VariableListLoadReport {
    pub resource_found: bool,
    pub loaded_variables: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableDatabaseLoadDisposition {
    NameNotDeclared,
    LoadedInteger,
    LoadedString,
}

fn general_variable_records(source: &[u8]) -> Vec<(&[u8], &[u8])> {
    let Some(start) = source
        .split(|byte| *byte == b'\n')
        .position(|line| {
            let line = trim_ascii(line.strip_suffix(b"\r").unwrap_or(line));
            line == b"GeneralVariableList" || line == b"[GeneralVariableList]"
        })
        .map(|line| line + 1)
    else {
        return Vec::new();
    };
    source
        .split(|byte| *byte == b'\n')
        .skip(start)
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .take_while(|line| {
            !matches!(line.first(), None | Some(b' ') | Some(b'\t') | Some(b'/') | Some(b'\r'))
        })
        .map(trim_ascii)
        .filter_map(|line| {
            let separator = line.iter().position(|byte| *byte == b'=')?;
            Some((trim_ascii(&line[..separator]), trim_ascii(&line[separator + 1..])))
        })
        .collect()
}

fn split_array_name(name: &[u8]) -> (&[u8], Option<usize>) {
    let Some(open) = name.iter().rposition(|byte| *byte == b'[') else {
        return (name, None);
    };
    let Some(closing) = name[open..].iter().position(|byte| *byte == b']') else {
        return (name, None);
    };
    if open + closing + 1 != name.len() {
        return (name, None);
    }
    let length = legacy_atoi(&name[open + 1..open + closing]);
    (name[..open].as_ref(), usize::try_from(length).ok().filter(|length| *length > 0))
}

fn trim_ascii(value: &[u8]) -> &[u8] {
    let start = value.iter().position(|byte| !byte.is_ascii_whitespace()).unwrap_or(value.len());
    let end = value.iter().rposition(|byte| !byte.is_ascii_whitespace()).map_or(start, |index| index + 1);
    &value[start..end]
}

fn legacy_atoi(value: &[u8]) -> i32 {
    let value = trim_ascii(value);
    let (negative, value) = match value.first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let magnitude = value.iter().take_while(|byte| byte.is_ascii_digit()).fold(0_i64, |value, byte| {
        value.saturating_mul(10).saturating_add(i64::from(*byte - b'0'))
    });
    let signed = if negative { magnitude.saturating_neg() } else { magnitude };
    signed.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn unquote_legacy_value(value: &[u8]) -> Vec<u8> {
    let value = value.strip_prefix(b"\"").unwrap_or(value);
    value.strip_suffix(b"\"").unwrap_or(value).to_vec()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariableSetOutcome {
    Updated { variable_index: usize },
    NotFound,
    Blocked(VariableSetBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariableSetBlock {
    EmptyIntegerArray { variable_index: usize },
    StringWriteWouldRetypeSavedUnion { variable_index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableStringField {
    Name,
    Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariableListSerializationBlock {
    VariableCountOutOfRange { count: usize },
    StringContainsNul {
        variable_index: usize,
        field: VariableStringField,
    },
    StringLengthOutOfRange {
        variable_index: usize,
        length: usize,
    },
    EmptyArray { variable_index: usize },
    ArrayLengthOutOfRange {
        variable_index: usize,
        length: usize,
    },
    PayloadLengthOutOfRange { length: usize },
}

impl fmt::Display for VariableListSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VariableCountOutOfRange { count } => write!(
                formatter,
                "VariableList содержит {count} записей вне signed 32-битного диапазона"
            ),
            Self::StringContainsNul {
                variable_index,
                field,
            } => write!(
                formatter,
                "VariableList #{variable_index}: поле {field:?} содержит внутренний NUL"
            ),
            Self::StringLengthOutOfRange {
                variable_index,
                length,
            } => write!(
                formatter,
                "VariableList #{variable_index}: строка длиной {length} не помещается в signed tag"
            ),
            Self::EmptyArray { variable_index } => write!(
                formatter,
                "VariableList #{variable_index}: пустой массив не имеет отдельного wire tag"
            ),
            Self::ArrayLengthOutOfRange {
                variable_index,
                length,
            } => write!(
                formatter,
                "VariableList #{variable_index}: массив длиной {length} вне signed 32-битного диапазона"
            ),
            Self::PayloadLengthOutOfRange { length } => write!(
                formatter,
                "VariableList payload длиной {length} вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for VariableListSerializationBlock {}

fn write_variable_c_string(
    destination: &mut Vec<u8>,
    variable_index: usize,
    field: VariableStringField,
    value: &[u8],
) -> Result<(), VariableListSerializationBlock> {
    if value.contains(&0) {
        return Err(VariableListSerializationBlock::StringContainsNul {
            variable_index,
            field,
        });
    }
    destination.extend_from_slice(value);
    destination.push(0);
    Ok(())
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

pub struct VariableSaveRow {
    pub name: Vec<u8>,
    pub initial_value: Vec<u8>,
    pub current_value: Vec<u8>,
}

pub trait VariableListSaveSource {
    fn variable_count(&self) -> usize;

    fn save_row(&self, index: usize) -> VariableSaveRow;
}

impl VariableListSaveSource for CVariableList {
    fn variable_count(&self) -> usize {
        self.variables.len()
    }

    fn save_row(&self, index: usize) -> VariableSaveRow {
        let variable = &self.variables[index];
        let (initial_value, current_value) = match &variable.value {
            VariableValue::Integer { current, saved } => (
                saved.to_string().into_bytes(),
                current.to_string().into_bytes(),
            ),
            VariableValue::String { current, saved } => {
                (quote_variable_string(saved), quote_variable_string(current))
            }
            VariableValue::IntegerArray { .. } => (Vec::new(), Vec::new()),
        };
        VariableSaveRow {
            name: variable.name.clone(),
            initial_value,
            current_value,
        }
    }
}

fn quote_variable_string(value: &[u8]) -> Vec<u8> {
    let mut quoted = Vec::with_capacity(value.len() + 2);
    quoted.push(b'"');
    quoted.extend_from_slice(value);
    quoted.push(b'"');
    quoted
}

pub async fn save_var_data<S: VariableListSaveSource, O: RsGenVarOwner>(
    variables: &S,
    database: &mut O,
    active_transaction: &mut WorldTdsClient,
) -> GenVarSaveOutcome {
    database.save(variables, active_transaction).await
}
