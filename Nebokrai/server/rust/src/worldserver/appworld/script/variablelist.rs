//! Владелец `CVariableList` исторического WorldServer из `variablelist.cpp`.
//!
//! Статусы `LoadVarList` RVA `0x000A1320`, `LoadOneVar` `0x000A1680`,
//! `SetVarValue` RVA `0x000A11B0/0x000A1240`, `SaveVarData` RVA `0x000A1930`
//! `AddToByteArray` RVA `0x000A1B10` и `LoadVarData` `0x000A1630` —
//! `IMPLEMENTED`; посторонние copy/destructor
//! `CBattleFairyProperty::tagCompose` ниже также выражены живым owner-ом
//! `goods::cbattlefairyproperty`, а оставшиеся блоки — compiler/STL cleanup.
//! Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp`.
//!
//! PDB задаёт `CVariableList` как `m_lVarNum: signed long` и
//! `m_pVarList: stVariable*`, а 16-байтный `stVariable` содержит `Name` по
//! `+0`, signed `Array` по `+4` и две union-пары `Value/strValue` и
//! `SValue/strSValue` по `+8/+0xc`. Достигнутый `GetOneVar` `0x000A19C0`
//! отдаёт имя и два уже форматированных ANSI-значения: `%d` для scalar,
//! `\"%s\"` для string и пустые значения для положительного `Array`.
//! `VariableListSaveSource` сохраняет именно этот узкий byte-exact view; сам
//! layout и остальные операции списка будут материализованы в их владельце.
//!
//! `LoadVarList` сначала очищает прежний список, затем читает непрерывные
//! строки после `GeneralVariableList`: `name=value`, `name[N]=value` и
//! `name="value"`. Scalar/array получают одинаковые current/saved values.
//! `LoadOneVar` ищет первое byte-sensitive имя из `CSL_GENVAR` и заменяет его
//! current/saved scalar либо строкой; именно так DB-строка может заменить
//! объявленный array. Rust не сохраняет исходные преждевременные return после
//! `delete` в декомпиляции — это внутренний compiler/lifetime defect, не
//! контракт списка.
//!
//! `SaveVarData` не имел собственной DB-семантики: копировал тот же ADO
//! connection, получал `GetGame()->m_pRsGenVar`, вызывал
//! `CRsGenVar::Save(this, connection)` и возвращал его `bool`. Rust заменяет
//! singleton явными owner/connection аргументами и исключает `nullptr` через
//! ссылки, но не меняет порядок, соединение либо результат. Условный
//! `BLOCKED_MISSING_FACT` переполнения исходного SQL scratch-буфера проходит
//! наружу отдельно и не маскируется придуманным `false`.
//!
//! Exact World serializer и Game decoder задают framing общего списка:
//! `signed variable count + signed payload length + payload`. Каждая запись
//! содержит C-string name, signed tag и значение: tag `0` — один `i32`, tag
//! `<0` — C-string, tag `>0` — ровно tag значений `i32`. Для строки исходный
//! tag равен `-(bytes + NUL)`. Typed enum заменяет две C++ union-пары, `Vec`
//! заменяет raw-массивы, сохраняя порядок и wire. Невозможные count/length,
//! внутренний NUL и пустой массив блокируются до изменения destination.
//!
//! Достигнутый World call-site integer-overload всегда передаёт index `0`.
//! Владелец линейно ищет первое ASCII-only `_strcmpi`-совпадение, меняет
//! scalar либо нулевой элемент непустого массива и продолжает поиск после
//! совпавшей string-записи. String-overload меняет первую совпавшую string.
//! EXE технически позволял переписать integer/array как string, но после этого
//! сохранённый union интерпретировался как `char*` и дальнейшее чтение имело UB;
//! Rust останавливает этот недопустимый owner-state typed-границей. `Vec` и
//! enum заменяют только raw allocation/union, не меняя штатную мутацию.

use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::rsgenvar::{
    GenVarLoadOutcome, GenVarSaveOutcome, RsGenVarOwner,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VariableValue {
    Integer { current: i32, saved: i32 },
    String { current: Vec<u8>, saved: Vec<u8> },
    IntegerArray { current: Vec<i32>, saved: Vec<i32> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VariableEntry {
    pub(crate) name: Vec<u8>,
    pub(crate) value: VariableValue,
}

/// Value-owner вместо `m_lVarNum + stVariable*` и двух raw union-ов.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CVariableList {
    variables: Vec<VariableEntry>,
}

impl Default for CVariableList {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CVariableList {
    /// Создаёт пустой список, как конструктор с `m_lVarNum = 0` и null-массивом.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    /// Освобождает все записи переменных и публикует пустое состояние.
    ///
    /// Rust `Vec` и `VariableValue` освобождают имя и значения без ранних
    /// возвратов и утечек старого owner-а.
    pub(crate) fn release(&mut self) {
        self.variables.clear();
    }

    /// Возвращает имя C-строки без индексного суффикса от последней `[`.
    ///
    /// Оригинал сначала копировал вход в выходной буфер вызывающего, затем
    /// ставил NUL на последней `[` независимо от наличия закрывающей `]`.
    /// Rust возвращает владеющий массив байтов и не меняет входной срез.
    pub(crate) fn get_array_name(name: &[u8]) -> Vec<u8> {
        let name = visible_c_string(name);
        let end = name
            .iter()
            .rposition(|byte| *byte == b'[')
            .unwrap_or(name.len());
        name[..end].to_vec()
    }

    /// Материализует достигнутый `LoadVarList` без raw allocation и union.
    ///
    /// `CIni::GetContinueDataNum` брал только непрерывный блок строк после
    /// index `GeneralVariableList`; пустая или отсутствующая resource поэтому
    /// публикует пустой список, как и исходный void owner.
    pub(crate) fn load_var_list(&mut self, source: Option<&[u8]>) -> VariableListLoadReport {
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
    pub(crate) fn load_one_var(
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

    pub(crate) fn push(&mut self, variable: VariableEntry) {
        self.variables.push(variable);
    }

    pub(crate) fn variables(&self) -> &[VariableEntry] {
        &self.variables
    }

    /// Точный достигнутый `SetVarValue(name, 0, value)` World call-site.
    pub(crate) fn set_zero_index_integer(
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

    /// Штатная часть `SetVarValue(name, string)` без unsafe union-retyping.
    pub(crate) fn set_string(&mut self, name: &[u8], value: &[u8]) -> VariableSetOutcome {
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

    pub(crate) fn add_to_byte_array(
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
    pub(crate) async fn load_var_data<O: RsGenVarOwner>(
        &mut self,
        database: &mut O,
    ) -> GenVarLoadOutcome {
        database.load_general_variables(self).await
    }
}

/// Наблюдаемый итог configuration-половины `LoadVarList`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct VariableListLoadReport {
    pub(crate) resource_found: bool,
    pub(crate) loaded_variables: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VariableDatabaseLoadDisposition {
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
pub(crate) enum VariableSetOutcome {
    Updated { variable_index: usize },
    NotFound,
    Blocked(VariableSetBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VariableSetBlock {
    EmptyIntegerArray { variable_index: usize },
    StringWriteWouldRetypeSavedUnion { variable_index: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VariableStringField {
    Name,
    Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VariableListSerializationBlock {
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

/// Три byte-exact строки, которые исходный `GetOneVar` отдавал DB-owner-у.
pub(crate) struct VariableSaveRow {
    pub(crate) name: Vec<u8>,
    pub(crate) initial_value: Vec<u8>,
    pub(crate) current_value: Vec<u8>,
}

/// Узкий read-only view достигнутой части `CVariableList`.
pub(crate) trait VariableListSaveSource {
    /// Число выполняемых исходным signed-циклом итераций.
    fn variable_count(&self) -> usize;

    /// Возвращает результат `GetOneVar` для доказанно допустимого индекса.
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

/// Делегирует сохранение тому же DB-owner-у на том же активном connection.
pub(crate) async fn save_var_data<S: VariableListSaveSource, O: RsGenVarOwner>(
    variables: &S,
    database: &mut O,
    active_transaction: &mut WorldTdsClient,
) -> GenVarSaveOutcome {
    database.save(variables, active_transaction).await
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::~tagCompose
// STATUS: IMPLEMENTED_API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RUST: четыре строки `BattleFairyCompose` освобождаются структурным Drop;
// MSVC string cleanup не имеет самостоятельного игрового эффекта.
// RVA: 0x0003FED0
// ADDRESS: 0043fed0
// PROTOTYPE: void __thiscall ~tagCompose(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::tagCompose
// STATUS: IMPLEMENTED_API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RUST: `BattleFairyCompose: Clone` копирует четыре byte-строки, оба scalar
// и `f32` success rate; промежуточный MSVC string lifecycle заменён Rust.
// RVA: 0x0003FFD0
// ADDRESS: 0043ffd0
// PROTOTYPE: undefined __thiscall tagCompose(tagCompose * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440286
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x00040286
// ADDRESS: 00440286
// PROTOTYPE: undefined Catch@00440286()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440388
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x00040388
// ADDRESS: 00440388
// PROTOTYPE: undefined Catch@00440388()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0044066a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0004066A
// ADDRESS: 0044066a
// PROTOTYPE: undefined Catch@0044066a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00440736
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x00040736
// ADDRESS: 00440736
// PROTOTYPE: undefined Catch@00440736()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::CVariableList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:18
// RVA: 0x000A1090
// ADDRESS: 004a1090
// PROTOTYPE: undefined __thiscall CVariableList(void)
//
// Реализовано выше как `with_constructor_defaults`: пустой `Vec` заменяет
// `m_lVarNum = 0` и нулевой raw-массив без ABI/vtable-техники.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:29
// RVA: 0x000A10B0
// ADDRESS: 004a10b0
// PROTOTYPE: void __thiscall Release(void)
//
// Реализовано выше как `release`: Rust освобождает все типизированные записи; ранние
// возвраты старого декомпилята после первого `delete` не являются контрактом.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetArrayName
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:510
// RVA: 0x000A1160
// ADDRESS: 004a1160
// PROTOTYPE: void __thiscall GetArrayName(char * param_1, char * param_2)
//
// Реализовано выше как `get_array_name`: отдельное Rust-значение заменяет
// выходной буфер вызывающего, а последний `[` отрезает суффикс.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::SetVarValue
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:577
// RVA: 0x000A11B0
// ADDRESS: 004a11b0
// PROTOTYPE: int __thiscall SetVarValue(char * param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::SetVarValue
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:601
// RVA: 0x000A1240
// ADDRESS: 004a1240
// PROTOTYPE: int __thiscall SetVarValue(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::LoadVarList
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:177
// RVA: 0x000A1320
// ADDRESS: 004a1320
// PROTOTYPE: void __thiscall LoadVarList(char * param_1, char * param_2, char * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::LoadVarData
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:447
// RVA: 0x000A1630
// ADDRESS: 004a1630
// PROTOTYPE: void __thiscall LoadVarData(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::LoadOneVar
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:60
// RVA: 0x000A1680
// ADDRESS: 004a1680
// PROTOTYPE: void __thiscall LoadOneVar(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetOneVar
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:144
// RVA: 0x000A19C0
// ADDRESS: 004a19c0
// PROTOTYPE: void __thiscall GetOneVar(long param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_3, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp:644
// RVA: 0x000A1B10
// ADDRESS: 004a1b10
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: Unwind@0052e010
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0012E010
// ADDRESS: 0052e010
// PROTOTYPE: undefined Unwind@0052e010()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052e030
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\script\variablelist.cpp
// RVA: 0x0012E030
// ADDRESS: 0052e030
// PROTOTYPE: undefined Unwind@0052e030()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
