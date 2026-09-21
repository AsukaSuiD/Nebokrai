//! Достигнутая wire-часть базового `CBaseObject` исторического GameServer.
//!
//! `AddToByteArray` RVA `0x000FC300`, scalar/name часть constructor-а
//! `0x000FC3C0` и `DecordFromByteArray` `0x000FC440` имеют статус
//! `IMPLEMENTED, VERIFIED_DISASSEMBLY`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`, SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, PDB
//! `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`.
//! Исходники: `server/gameserver/appserver/baseobject.h/.cpp`.
//!
//! Exact EXE подтверждает signed `m_lType +0x4`, `m_lID +0x8`, нулевой
//! `CGUID +0xC`, `m_lGraphicsID +0x1C`, byte-string `+0x20`, include-child
//! `+0x3C == true` и null father `+0x40`. Достигнутый codec пишет три
//! little-endian `long`, затем имя с NUL; decoder использует локальный
//! `char[256]`, сохраняет тот же порядок и на normal return явно ставит
//! `AL=1` по `0x004FC4BD`. Входной include-child оба тела не читают.
//! `Vec<u8>` заменяет `std::string` без навязывания UTF-8, а safe decoder
//! останавливает отсутствие NUL/выход за старый 256-байтовый буфер локальным
//! `BLOCKED_MISSING_FACT`; уже прочитанные scalar-поля и cursor сохраняются.
//! Child-list/father ownership и полный destructor остаются только в локальном исследовательском корпусе: helper
//! конструктора материализует только достигнутую region-chain часть и не
//! объявляет Rust layout копией старого ABI.
//! Identity helpers `GetHashValue/CalculateType/CalculateID` RVA
//! `0x000FC0C0/0x000FC0E0/0x000FC0F0` также имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`: верхний DWORD хранит type с исходным sign-extension
//! отрицательного ID, нижний — битовый образ ID.

use std::fmt;
use thiserror::Error;

use crate::public::guid::CGuid;

use super::legacycodec::{LegacyReader, LegacyWriter};
use super::monster::CMonster;
use super::npc::CNpc;

const LEGACY_NAME_CAPACITY: usize = 0x100;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum BaseObjectDecodeError {
    #[error("base object обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("base object name вышло за legacy buffer в {first_out_of_bounds_offset}")]
    LegacyNameOverflow {
        first_out_of_bounds_offset: usize,
    },
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct CBaseObject {
    object_type: i32,
    id: i32,
    ex_id: CGuid,
    graphics_id: i32,
    name: Vec<u8>,
    pub(crate) include_child: bool,
}

impl fmt::Debug for CBaseObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CBaseObject")
            .field("object_type", &self.object_type)
            .field("id", &self.id)
            .field("graphics_id", &self.graphics_id)
            .field("name", &self.name)
            .field("include_child", &self.include_child)
            .finish_non_exhaustive()
    }
}

impl CBaseObject {
    pub(crate) const fn with_reached_constructor_defaults() -> Self {
        Self {
            object_type: 0,
            id: 0,
            ex_id: CGuid::GUID_INVALID,
            graphics_id: 0,
            name: Vec::new(),
            include_child: true,
        }
    }

    pub(crate) const fn get_type(&self) -> i32 {
        self.object_type
    }

    pub(crate) const fn set_type(&mut self, object_type: i32) {
        self.object_type = object_type;
    }

    pub(crate) const fn get_id(&self) -> i32 {
        self.id
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub(crate) const fn get_ex_id(&self) -> CGuid {
        self.ex_id
    }

    pub(crate) const fn set_ex_id(&mut self, ex_id: CGuid) {
        self.ex_id = ex_id;
    }

    pub(crate) const fn get_graphics_id(&self) -> i32 {
        self.graphics_id
    }

    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.graphics_id = graphics_id;
    }

    pub(crate) const fn get_hash_value(object_type: i32, id: i32) -> i64 {
        let high = object_type | (id >> 31);
        ((high as u32 as u64) << 32 | id as u32 as u64) as i64
    }

    pub(crate) const fn calculate_type(hash: i64) -> i32 {
        (hash >> 32) as i32
    }

    pub(crate) const fn calculate_id(hash: i64) -> i32 {
        hash as i32
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        let prefix_len = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        self.name.clear();
        self.name.extend_from_slice(&name[..prefix_len]);
    }

    /// Материализует точную ветвь `CreateObject(500, id)`: сначала constructor
    /// `CNpc`, затем общая factory-tail запись type/ID.
    pub(crate) fn create_npc(id: i32) -> CNpc {
        let mut npc = CNpc::with_constructor_defaults();
        npc.move_shape_mut()
            .shape_mut()
            .base_object_mut()
            .set_id(id);
        npc
    }

    /// Материализует ветвь `CreateObject(600, id)` до derived skills/AI Init.
    pub(crate) fn create_monster(id: i32) -> CMonster {
        let mut monster = CMonster::with_constructor_defaults();
        monster
            .move_shape_mut()
            .shape_mut()
            .base_object_mut()
            .set_id(id);
        monster
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        _include_child: bool,
    ) -> bool {
        let mut writer = LegacyWriter::new(destination);
        writer.write_i32(self.object_type);
        writer.write_i32(self.id);
        writer.write_i32(self.graphics_id);
        writer.write_c_string(&self.name);
        true
    }

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
    ) -> Result<bool, BaseObjectDecodeError> {
        self.object_type = read_i32(source, cursor, "m_lType")?;
        self.id = read_i32(source, cursor, "m_lID")?;
        self.graphics_id = read_i32(source, cursor, "m_lGraphicsID")?;
        self.name = read_name(source, cursor)?;
        Ok(true)
    }
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, BaseObjectDecodeError> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        BaseObjectDecodeError::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader.read_i32().map_err(|block| BaseObjectDecodeError::UnexpectedEnd {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })?;
    *cursor = reader.position();
    Ok(value)
}

fn read_name(source: &[u8], cursor: &mut usize) -> Result<Vec<u8>, BaseObjectDecodeError> {
    let mut name = Vec::new();
    loop {
        let offset = *cursor;
        let mut reader = LegacyReader::at(source, offset).map_err(|block| {
            BaseObjectDecodeError::UnexpectedEnd {
                field: "m_strName",
                offset: block.offset,
                needed: 1,
                available: block.available,
            }
        })?;
        let byte = reader.read_u8().map_err(|block| BaseObjectDecodeError::UnexpectedEnd {
            field: "m_strName",
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
        *cursor = reader.position();
        if name.len() == LEGACY_NAME_CAPACITY {
            // BLOCKED_MISSING_FACT: этот байт уже выходил за local char[256].
            return Err(BaseObjectDecodeError::LegacyNameOverflow {
                first_out_of_bounds_offset: offset,
            });
        }
        if byte == 0 {
            return Ok(name);
        }
        name.push(byte);
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp

// ============================================================================
// FUNCTION: CBaseObject::FindChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:106
// RVA: 0x000856A0
// ADDRESS: 004856a0
// PROTOTYPE: bool __thiscall FindChildObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::GetName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.h:70
// RVA: 0x000C99B0
// ADDRESS: 004c99b0
// PROTOTYPE: char * __thiscall GetName(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CBaseObject::GetHashValue` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CBaseObject::CalculateType` материализован выше; покрытый raw-блок удалён.

// IMPLEMENTED: `CBaseObject::CalculateID` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CBaseObject::FindChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:121
// RVA: 0x000FC100
// ADDRESS: 004fc100
// PROTOTYPE: CBaseObject * __thiscall FindChildObject(long param_1, long param_2, CGUID * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::CreateObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// IMPLEMENTED_SUBCHAIN: ветви type 500/600 материализованы выше как
// `create_npc/create_monster`; остальные variants и erased return остаются RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:178
// RVA: 0x000FC110
// ADDRESS: 004fc110
// PROTOTYPE: CBaseObject * __cdecl CreateObject(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CBaseObject::AddToByteArray` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CBaseObject::~CBaseObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:36
// RVA: 0x000FC360
// ADDRESS: 004fc360
// PROTOTYPE: void __thiscall ~CBaseObject(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::CBaseObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:25
// RVA: 0x000FC3C0
// ADDRESS: 004fc3c0
// PROTOTYPE: undefined __thiscall CBaseObject(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED: `CBaseObject::DecordFromByteArray` материализован выше; покрытый raw-блок удалён.

// ============================================================================
// FUNCTION: CBaseObject::CreateChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:240
// RVA: 0x000FC4D0
// ADDRESS: 004fc4d0
// PROTOTYPE: CBaseObject * __thiscall CreateChildObject(long param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseObject::DeleteChildObject
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\baseobject.cpp:326
// RVA: 0x001FECA0
// ADDRESS: 005feca0
// PROTOTYPE: void __thiscall DeleteChildObject(long param_1, long param_2, CGUID * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
