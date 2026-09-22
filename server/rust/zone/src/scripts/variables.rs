//! Механика и wire-формат списков сценарных переменных Zone.
//! Экземпляры списка по-прежнему принадлежат игроку или копии общих значений Game;
//! объявления разбирает Shared scripting/variablelist.rs.
//!
//! Перенесено из старого Game-модуля по исходному владельцу
//! `server/gameserver/appserver/script/variablelist.cpp`; основание — точная пара
//! `gameserver.exe + GameServer.pdb`. Startup сначала
//! загружает объявления из полученного ресурса `VariableList`, затем
//! `DecordFromByteArray` применяет World snapshot: signed count, ignored
//! длину payload, C-string имени, знаковый tag и значения scalar/string/array.
//! `CScript::LoadGeneralVariable` передаёт cursor по значению, поэтому decoder
//! двигает только локальную копию и не меняет позицию внешнего `CMessage`.
//! `Vec` заменяет ручные union/allocation массивы, сохраняя insertion order,
//! первое обновление снимка по точному имени и порядок курсора. Ветка `0x7F805`
//! сохраняет поиск `_stricmp`, правила индекса scalar/array и смену типа строки.
//! Повреждённый wire возвращает типизированную ошибку вместо чтения за границей;
//! остальные операции над выражениями сохранены только в локальном исследовательском корпусе.

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};
use nebokrai_shared::scripting::{VariableDefault, VariableListError, VariableListRecords};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameVariableValue {
    Integer(i32),
    String(Vec<u8>),
    IntegerArray(Vec<i32>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameVariable {
    pub name: Vec<u8>,
    pub value: GameVariableValue,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CVariableList {
    variables: Vec<GameVariable>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameVariableMutationOutcome {
    UpdatedInteger {
        variable_index: usize,
    },
    UpdatedArrayElement {
        variable_index: usize,
        element_index: usize,
    },
    UpdatedString {
        variable_index: usize,
        retyped: bool,
    },
    TypeMismatch {
        variable_index: usize,
    },
    NameNotFound,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GameVariableSnapshotError {
    #[error("отказ объявлений VariableList: {0:?}")]
    Definitions(VariableListError),
    #[error("не удалось выделить массив из {length} элементов для объявления в {offset}")]
    DefinitionAllocation { offset: usize, length: usize },
    #[error("variable snapshot обрывается в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("variable snapshot содержит отрицательное count {0}")]
    NegativeCount(i32),
    #[error("variable snapshot содержит имя длиной {length}")]
    NameTooLong { length: usize },
    #[error("variable snapshot содержит недопустимую длину массива {0}")]
    InvalidArrayLength(i32),
}

impl CVariableList {
    pub fn variables(&self) -> &[GameVariable] {
        &self.variables
    }

    pub fn integer(&self, name: &[u8], element_index: usize) -> Option<i32> {
        let variable = self
            .variables
            .iter()
            .find(|variable| variable.name.eq_ignore_ascii_case(name))?;
        match &variable.value {
            GameVariableValue::Integer(value) if element_index == 0 => Some(*value),
            GameVariableValue::IntegerArray(values) => values.get(element_index).copied(),
            GameVariableValue::Integer(_) | GameVariableValue::String(_) => None,
        }
    }

    pub fn string(&self, name: &[u8]) -> Option<&[u8]> {
        let variable = self
            .variables
            .iter()
            .find(|variable| variable.name.eq_ignore_ascii_case(name))?;
        match &variable.value {
            GameVariableValue::String(value) => Some(value),
            GameVariableValue::Integer(_) | GameVariableValue::IntegerArray(_) => None,
        }
    }

    pub fn release(&mut self) -> usize {
        let count = self.variables.len();
        self.variables.clear();
        count
    }

    /// Exact integer `SetVarValue(name, index, value)`: первый
    /// ASCII-case-insensitive owner, scalar только при index `0`, массив только
    /// внутри длины; строка с совпавшим именем блокирует дальнейший поиск.
    pub fn set_integer(
        &mut self,
        name: &[u8],
        element_index: usize,
        value: i32,
    ) -> GameVariableMutationOutcome {
        let Some((variable_index, variable)) = self
            .variables
            .iter_mut()
            .enumerate()
            .find(|(_, variable)| variable.name.eq_ignore_ascii_case(name))
        else {
            return GameVariableMutationOutcome::NameNotFound;
        };
        match &mut variable.value {
            GameVariableValue::Integer(current) if element_index == 0 => {
                *current = value;
                GameVariableMutationOutcome::UpdatedInteger { variable_index }
            }
            GameVariableValue::IntegerArray(current) => {
                let Some(current) = current.get_mut(element_index) else {
                    return GameVariableMutationOutcome::TypeMismatch { variable_index };
                };
                *current = value;
                GameVariableMutationOutcome::UpdatedArrayElement {
                    variable_index,
                    element_index,
                }
            }
            GameVariableValue::Integer(_) | GameVariableValue::String(_) => {
                GameVariableMutationOutcome::TypeMismatch { variable_index }
            }
        }
    }

    /// Exact string `SetVarValue(name, value)` переводит первую совпавшую
    /// scalar/array запись в строковый layout и заменяет существующую строку.
    pub fn set_string(&mut self, name: &[u8], value: &[u8]) -> GameVariableMutationOutcome {
        let Some((variable_index, variable)) = self
            .variables
            .iter_mut()
            .enumerate()
            .find(|(_, variable)| variable.name.eq_ignore_ascii_case(name))
        else {
            return GameVariableMutationOutcome::NameNotFound;
        };
        let retyped = !matches!(variable.value, GameVariableValue::String(_));
        variable.value = GameVariableValue::String(value.to_vec());
        GameVariableMutationOutcome::UpdatedString {
            variable_index,
            retyped,
        }
    }

    /// Exact `AddVar(name, value)`: существующее имя проверяется побайтно,
    /// иначе новая scalar-запись добавляется в конец insertion-order списка.
    pub fn add_integer(&mut self, name: &[u8], value: i32) -> GameVariableMutationOutcome {
        if self.variables.iter().any(|variable| variable.name == name) {
            return self.set_integer(name, 0, value);
        }
        let variable_index = self.variables.len();
        self.variables.push(GameVariable {
            name: name.to_vec(),
            value: GameVariableValue::Integer(value),
        });
        GameVariableMutationOutcome::UpdatedInteger { variable_index }
    }

    /// Строковая перегрузка `AddVar` сохраняет тот же exact-name append и
    /// штатную смену типа уже существующей записи через `SetVarValue`.
    pub fn add_string(&mut self, name: &[u8], value: &[u8]) -> GameVariableMutationOutcome {
        if self.variables.iter().any(|variable| variable.name == name) {
            return self.set_string(name, value);
        }
        let variable_index = self.variables.len();
        self.variables.push(GameVariable {
            name: name.to_vec(),
            value: GameVariableValue::String(value.to_vec()),
        });
        GameVariableMutationOutcome::UpdatedString {
            variable_index,
            retyped: false,
        }
    }

    pub fn decode_world_snapshot(
        &mut self,
        definitions: Option<&[u8]>,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), GameVariableSnapshotError> {
        self.load_definitions(definitions)?;
        let declared_variables = self.variables.len();
        let start = *cursor;
        let count = read_i32(source, cursor)?;
        if count < 0 {
            return Err(GameVariableSnapshotError::NegativeCount(count));
        }
        let declared_payload_length = read_i32(source, cursor)?;
        for _ in 0..count {
            let name = read_c_string(source, cursor)?;
            if name.len() >= 0x100 {
                return Err(GameVariableSnapshotError::NameTooLong { length: name.len() });
            }
            let tag = read_i32(source, cursor)?;
            let value = if tag == 0 {
                GameVariableValue::Integer(read_i32(source, cursor)?)
            } else if tag < 0 {
                GameVariableValue::String(read_c_string(source, cursor)?)
            } else {
                let length = usize::try_from(tag)
                    .map_err(|_| GameVariableSnapshotError::InvalidArrayLength(tag))?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(read_i32(source, cursor)?);
                }
                GameVariableValue::IntegerArray(values)
            };
            self.insert_or_update(name, value);
        }
        tracing::trace!(
            declared_variables,
            snapshot_variables = count as usize,
            declared_payload_length,
            consumed_bytes = cursor.saturating_sub(start),
            "снимок сценарных переменных применён"
        );
        Ok(())
    }

    /// Exact `AddToByteArray`: count, размер временного payload и сами records.
    /// Строковые значения используют tag `-1`; scalar — `0`, массив — длину.
    pub fn encode_world_snapshot(&self, destination: &mut Vec<u8>) -> bool {
        let Ok(count) = i32::try_from(self.variables.len()) else {
            return false;
        };
        LegacyWriter::new(destination).write_i32(count);
        let mut payload = Vec::new();
        for variable in &self.variables {
            LegacyWriter::new(&mut payload).write_c_string(&variable.name);
            match &variable.value {
                GameVariableValue::Integer(value) => {
                    let mut writer = LegacyWriter::new(&mut payload);
                    writer.write_i32(0);
                    writer.write_i32(*value);
                }
                GameVariableValue::String(value) => {
                    let mut writer = LegacyWriter::new(&mut payload);
                    writer.write_i32(-1);
                    writer.write_c_string(value);
                }
                GameVariableValue::IntegerArray(values) => {
                    let Ok(length) = i32::try_from(values.len()) else {
                        return false;
                    };
                    let mut writer = LegacyWriter::new(&mut payload);
                    writer.write_i32(length);
                    for value in values {
                        writer.write_i32(*value);
                    }
                }
            }
        }
        let Ok(payload_length) = i32::try_from(payload.len()) else {
            return false;
        };
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(payload_length);
        writer.write_bytes(&payload);
        true
    }

    pub(crate) fn load_definitions(
        &mut self,
        definitions: Option<&[u8]>,
    ) -> Result<(), GameVariableSnapshotError> {
        self.variables.clear();
        let Some(definitions) = definitions else {
            return Ok(());
        };
        let records = VariableListRecords::new(definitions)
            .map_err(GameVariableSnapshotError::Definitions)?;
        for record in records {
            let record = record.map_err(GameVariableSnapshotError::Definitions)?;
            let value = match record.value {
                VariableDefault::Integer(value) => GameVariableValue::Integer(value),
                VariableDefault::String(value) => GameVariableValue::String(value.to_vec()),
                VariableDefault::IntegerArray { length, value } => {
                    let mut values = Vec::new();
                    values.try_reserve_exact(length).map_err(|_| {
                        GameVariableSnapshotError::DefinitionAllocation {
                            offset: record.offset,
                            length,
                        }
                    })?;
                    values.resize(length, value);
                    GameVariableValue::IntegerArray(values)
                }
            };
            // LoadVarList заполняет отдельную запись каждой строки, включая повторы имён.
            self.variables.push(GameVariable {
                name: record.name.to_vec(),
                value,
            });
        }
        Ok(())
    }

    fn insert_or_update(&mut self, name: Vec<u8>, value: GameVariableValue) {
        if let Some(variable) = self.variables.iter_mut().find(|entry| entry.name == name) {
            variable.value = value;
        } else {
            self.variables.push(GameVariable { name, value });
        }
    }
}

fn read_i32(source: &[u8], cursor: &mut usize) -> Result<i32, GameVariableSnapshotError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        GameVariableSnapshotError::UnexpectedEnd {
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader
        .read_i32()
        .map_err(|block| GameVariableSnapshotError::UnexpectedEnd {
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_c_string(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, GameVariableSnapshotError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let mut reader = LegacyReader::at(source, offset).map_err(|block| {
        GameVariableSnapshotError::UnexpectedEnd {
            offset: block.offset,
            needed: 1,
            available: block.available,
        }
    })?;
    let value = reader.read_c_string(available).map_err(|block| {
        GameVariableSnapshotError::UnexpectedEnd {
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        }
    })?;
    *cursor = reader.position();
    Ok(value.to_vec())
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp

// IMPLEMENTED: `CVariableList::CVariableList` заменён безопасным `Default` owned storage.

// IMPLEMENTED: `CVariableList::Release` материализован выше как `release`.

// ============================================================================
// FUNCTION: CVariableList::SetVarList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:254
// RVA: 0x000AD4B0
// ADDRESS: 004ad4b0
// PROTOTYPE: void __thiscall SetVarList(stVariable * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::isExist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:323
// RVA: 0x000AD680
// ADDRESS: 004ad680
// PROTOTYPE: bool __thiscall isExist(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::RemoveVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:343
// RVA: 0x000AD6E0
// ADDRESS: 004ad6e0
// PROTOTYPE: bool __thiscall RemoveVar(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetArrayNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:433
// RVA: 0x000AD840
// ADDRESS: 004ad840
// PROTOTYPE: int __thiscall GetArrayNum(char * param_1, CPlayer * param_2, CRegion * param_3, CNpc * param_4, CGUID * param_5, ulong param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetArrayName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:464
// RVA: 0x000AD960
// ADDRESS: 004ad960
// PROTOTYPE: void __thiscall GetArrayName(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetVarValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:483
// RVA: 0x000AD9B0
// ADDRESS: 004ad9b0
// PROTOTYPE: int __thiscall GetVarValue(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::GetVarValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:526
// RVA: 0x000ADA30
// ADDRESS: 004ada30
// PROTOTYPE: char * __thiscall GetVarValue(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: integer `CVariableList::SetVarValue` материализован выше как `set_integer`.

// IMPLEMENTED: string `CVariableList::SetVarValue` материализован выше как `set_string`.

// ============================================================================
// FUNCTION: CVariableList::AddVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:132
// RVA: 0x000ADF50
// ADDRESS: 004adf50
// PROTOTYPE: void __thiscall AddVar(char * param_1, int param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::AddVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:167
// RVA: 0x000AE0A0
// ADDRESS: 004ae0a0
// PROTOTYPE: void __thiscall AddVar(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::AddVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:211
// RVA: 0x000AE240
// ADDRESS: 004ae240
// PROTOTYPE: void __thiscall AddVar(char * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::UpdateVarList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:301
// RVA: 0x000AE370
// ADDRESS: 004ae370
// PROTOTYPE: void __thiscall UpdateVarList(stVariable * param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CVariableList::DecordFromByteArray` материализован выше как `decode_world_snapshot`.

// ============================================================================
// FUNCTION: CVariableList::AddVar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:120
// RVA: 0x000AE510
// ADDRESS: 004ae510
// PROTOTYPE: bool __thiscall AddVar(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:630
// RVA: 0x000AE540
// ADDRESS: 004ae540
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::tagCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp
// RVA: 0x001C7C70
// ADDRESS: 005c7c70
// PROTOTYPE: undefined __thiscall tagCompose(tagCompose * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c7e06
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp
// RVA: 0x001C7E06
// ADDRESS: 005c7e06
// PROTOTYPE: undefined Catch@005c7e06()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c7fb8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp
// RVA: 0x001C7FB8
// ADDRESS: 005c7fb8
// PROTOTYPE: undefined Catch@005c7fb8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c828a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp
// RVA: 0x001C828A
// ADDRESS: 005c828a
// PROTOTYPE: undefined Catch@005c828a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005c8356
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp
// RVA: 0x001C8356
// ADDRESS: 005c8356
// PROTOTYPE: undefined Catch@005c8356()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
