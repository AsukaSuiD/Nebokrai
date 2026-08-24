//! General-variable storage GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `server/gameserver/appserver/script/variablelist.cpp`. Startup сначала
//! загружает declarations из полученного `VariableList` resource, затем
//! `DecordFromByteArray` применяет World snapshot: signed count, ignored
//! payload length, C-string name, signed scalar/string/array tag и values.
//! `CScript::LoadGeneralVariable` передаёт cursor по значению, поэтому decoder
//! двигает только локальную копию и не меняет позицию внешнего `CMessage`.
//! `Vec` заменяет ручные union/allocation массивы, сохраняя insertion order,
//! first exact-name update и cursor ordering. Malformed wire возвращает typed
//! ошибку вместо чтения за границей; остальные expression operations ниже RAW.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameVariableValue {
    Integer(i32),
    String(Vec<u8>),
    IntegerArray(Vec<i32>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameVariable {
    pub(crate) name: Vec<u8>,
    pub(crate) value: GameVariableValue,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CVariableList {
    variables: Vec<GameVariable>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GameVariableSnapshotReport {
    pub(crate) declared_variables: usize,
    pub(crate) snapshot_variables: usize,
    pub(crate) declared_payload_length: i32,
    pub(crate) consumed_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameVariableSnapshotError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    NegativeCount(i32),
    NameTooLong {
        length: usize,
    },
    InvalidArrayLength(i32),
}

impl CVariableList {
    pub(crate) fn variables(&self) -> &[GameVariable] {
        &self.variables
    }

    pub(crate) fn release(&mut self) -> usize {
        let count = self.variables.len();
        self.variables.clear();
        count
    }

    pub(crate) fn decode_world_snapshot(
        &mut self,
        definitions: Option<&[u8]>,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<GameVariableSnapshotReport, GameVariableSnapshotError> {
        self.load_definitions(definitions);
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
        Ok(GameVariableSnapshotReport {
            declared_variables,
            snapshot_variables: count as usize,
            declared_payload_length,
            consumed_bytes: cursor.saturating_sub(start),
        })
    }

    fn load_definitions(&mut self, definitions: Option<&[u8]>) {
        self.variables.clear();
        let Some(definitions) = definitions else {
            return;
        };
        for (name, value) in section_records(definitions, b"VariableList") {
            let (name, array_length) = split_array_name(name);
            let value = trim_ascii(value);
            let variable = if let Some(length) = array_length {
                GameVariableValue::IntegerArray(vec![legacy_atoi(value); length])
            } else if name.first() == Some(&b'#') || value.first() == Some(&b'\"') {
                GameVariableValue::String(unquote(value))
            } else {
                GameVariableValue::Integer(legacy_atoi(value))
            };
            self.insert_or_update(name.to_vec(), variable);
        }
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
    let offset = *cursor;
    let bytes = source.get(offset..offset.saturating_add(4)).ok_or(
        GameVariableSnapshotError::UnexpectedEnd {
            offset,
            needed: 4,
            available: source.len().saturating_sub(offset),
        },
    )?;
    *cursor += 4;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("slice длиной 4"),
    ))
}

fn read_c_string(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, GameVariableSnapshotError> {
    let offset = *cursor;
    let tail = source.get(offset..).unwrap_or_default();
    let Some(length) = tail.iter().position(|byte| *byte == 0) else {
        return Err(GameVariableSnapshotError::UnexpectedEnd {
            offset,
            needed: tail.len().saturating_add(1),
            available: tail.len(),
        });
    };
    *cursor += length + 1;
    Ok(tail[..length].to_vec())
}

pub(crate) fn section_records<'a>(source: &'a [u8], section: &[u8]) -> Vec<(&'a [u8], &'a [u8])> {
    let bracketed = [b"[".as_slice(), section, b"]".as_slice()].concat();
    let Some(start) = source
        .split(|byte| *byte == b'\n')
        .position(|line| {
            let line = trim_ascii(line.strip_suffix(b"\r").unwrap_or(line));
            line == section || line == bracketed
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
            !matches!(
                line.first(),
                None | Some(b' ') | Some(b'\t') | Some(b'/') | Some(b'\r')
            ) && !trim_ascii(line).starts_with(b"[")
        })
        .filter_map(|line| {
            let line = trim_ascii(line);
            let separator = line.iter().position(|byte| *byte == b'=')?;
            Some((
                trim_ascii(&line[..separator]),
                trim_ascii(&line[separator + 1..]),
            ))
        })
        .collect()
}

fn split_array_name(name: &[u8]) -> (&[u8], Option<usize>) {
    let Some(open) = name.iter().rposition(|byte| *byte == b'[') else {
        return (name, None);
    };
    let Some(close) = name[open..].iter().position(|byte| *byte == b']') else {
        return (name, None);
    };
    if open + close + 1 != name.len() {
        return (name, None);
    }
    let length = usize::try_from(legacy_atoi(&name[open + 1..open + close]))
        .ok()
        .filter(|length| *length > 0);
    (&name[..open], length)
}

fn trim_ascii(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &value[start..end]
}

fn legacy_atoi(value: &[u8]) -> i32 {
    let value = trim_ascii(value);
    let (negative, digits) = match value.first() {
        Some(b'-') => (true, &value[1..]),
        Some(b'+') => (false, &value[1..]),
        _ => (false, value),
    };
    let magnitude =
        digits
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .fold(0_i64, |current, byte| {
                current
                    .saturating_mul(10)
                    .saturating_add(i64::from(*byte - b'0'))
            });
    let value = if negative { -magnitude } else { magnitude };
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn unquote(value: &[u8]) -> Vec<u8> {
    if value.first() == Some(&b'\"') {
        value
            .get(1..value.len().saturating_sub(1))
            .unwrap_or_default()
            .to_vec()
    } else {
        value.to_vec()
    }
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

// ============================================================================
// FUNCTION: CVariableList::SetVarValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:547
// RVA: 0x000ADA90
// ADDRESS: 004ada90
// PROTOTYPE: int __thiscall SetVarValue(char * param_1, int param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CVariableList::SetVarValue
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\script\variablelist.cpp:587
// RVA: 0x000ADB50
// ADDRESS: 004adb50
// PROTOTYPE: int __thiscall SetVarValue(char * param_1, char * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CVariableList::LoadVarList` материализован выше как definition pass `load_definitions`.

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
