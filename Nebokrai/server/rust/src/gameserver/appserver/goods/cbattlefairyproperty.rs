//! Game-side конфигурация объединения battle fairy из точных
//! `gameserver.exe + GameServer.pdb`; исходный owner
//! `gameserver/appserver/goods/cbattlefairyproperty.h/.cpp`.
//!
//! Достигнут selector `0x2D`: signed count и raw 0x7C MSVC records. Парный
//! World serializer допускает только SSO-строки до 15 байт, потому что heap
//! pointer другого процесса непереносим. Decoder сохраняет это ограничение,
//! очищает список до count и публикует только полные records. Остальные
//! BattleFairy gameplay/formula methods ниже остаются RAW.

use std::error::Error;
use std::fmt;

const COMPOSE_RECORD_SIZE: usize = 0x7c;
const LEGACY_STRING_SIZE: usize = 0x1c;
const LEGACY_STRING_INLINE_CAPACITY: u32 = 15;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct BattleFairyCompose {
    pub(crate) fetch_stone: Vec<u8>,
    pub(crate) fetch_body: Vec<u8>,
    pub(crate) material: Vec<u8>,
    pub(crate) deplete_fetch: u32,
    pub(crate) success_rate: f32,
    pub(crate) battle_fairy: Vec<u8>,
    pub(crate) index: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CBattleFairyProperty {
    compose: Vec<BattleFairyCompose>,
}

impl CBattleFairyProperty {
    pub(crate) fn compose(&self) -> &[BattleFairyCompose] {
        &self.compose
    }

    pub(crate) fn decord_byte_array_combine(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<usize, BattleFairyComposeDecodeError> {
        self.compose.clear();
        let count = read_wire_i32(source, cursor)?;
        for record_index in 0..count.max(0) {
            let record = read_wire_array::<COMPOSE_RECORD_SIZE>(source, cursor)?;
            self.compose.push(BattleFairyCompose {
                fetch_stone: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strFetchStone",
                    0x00,
                )?,
                fetch_body: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strFetchBody",
                    0x1c,
                )?,
                material: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strMaterial",
                    0x38,
                )?,
                deplete_fetch: u32::from_le_bytes(
                    record[0x54..0x58].try_into().expect("поле 4 байта"),
                ),
                success_rate: f32::from_bits(u32::from_le_bytes(
                    record[0x58..0x5c].try_into().expect("поле 4 байта"),
                )),
                battle_fairy: decode_legacy_string(
                    &record,
                    record_index as usize,
                    "strBattleFairy",
                    0x5c,
                )?,
                index: u32::from_le_bytes(record[0x78..0x7c].try_into().expect("поле 4 байта")),
            });
        }
        Ok(self.compose.len())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleFairyComposeDecodeError {
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        available: usize,
    },
    NonPortableString {
        record: usize,
        field: &'static str,
        length: u32,
        capacity: u32,
    },
}

impl fmt::Display for BattleFairyComposeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd {
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "BattleFairy combine snapshot обрывается на {offset}: нужно {needed}, доступно {available}"
            ),
            Self::NonPortableString {
                record,
                field,
                length,
                capacity,
            } => write!(
                formatter,
                "BattleFairy combine record {record}, {field}: непереносимый MSVC string length {length}, capacity {capacity}"
            ),
        }
    }
}

impl Error for BattleFairyComposeDecodeError {}

fn decode_legacy_string(
    record: &[u8; COMPOSE_RECORD_SIZE],
    record_index: usize,
    field: &'static str,
    offset: usize,
) -> Result<Vec<u8>, BattleFairyComposeDecodeError> {
    let length = u32::from_le_bytes(
        record[offset + 0x14..offset + 0x18]
            .try_into()
            .expect("MSVC string length содержит четыре байта"),
    );
    let capacity = u32::from_le_bytes(
        record[offset + 0x18..offset + LEGACY_STRING_SIZE]
            .try_into()
            .expect("MSVC string capacity содержит четыре байта"),
    );
    if capacity > LEGACY_STRING_INLINE_CAPACITY || length > LEGACY_STRING_INLINE_CAPACITY {
        return Err(BattleFairyComposeDecodeError::NonPortableString {
            record: record_index,
            field,
            length,
            capacity,
        });
    }
    let length = length as usize;
    Ok(record[offset + 4..offset + 4 + length].to_vec())
}

fn read_wire_i32(source: &[u8], cursor: &mut usize) -> Result<i32, BattleFairyComposeDecodeError> {
    Ok(i32::from_le_bytes(read_wire_array(source, cursor)?))
}

fn read_wire_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], BattleFairyComposeDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(bytes) = source.get(offset..offset.saturating_add(N)) else {
        return Err(BattleFairyComposeDecodeError::UnexpectedEnd {
            offset,
            needed: N,
            available,
        });
    };
    *cursor += N;
    Ok(bytes
        .try_into()
        .expect("размер BattleFairy combine record уже проверен"))
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.cpp

// ============================================================================
// FUNCTION: CBattleFairyProperty::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.h:96
// RVA: 0x0009CEE0
// ADDRESS: 0049cee0
// PROTOTYPE: CBattleFairyProperty * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004caf59
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.cpp
// RVA: 0x000CAF59
// ADDRESS: 004caf59
// PROTOTYPE: undefined Catch@004caf59()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::LevelUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.cpp:191
// RVA: 0x001C78F0
// ADDRESS: 005c78f0
// PROTOTYPE: eExpUpResult __thiscall LevelUp(CGoods * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::tagCompose::tagCompose
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.h:52
// RVA: 0x001C7E40
// ADDRESS: 005c7e40
// PROTOTYPE: undefined __thiscall tagCompose(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::CBattleFairyProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.cpp:56
// RVA: 0x001C7FF0
// ADDRESS: 005c7ff0
// PROTOTYPE: undefined __thiscall CBattleFairyProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::ExpUp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.cpp:103
// RVA: 0x001C83F0
// ADDRESS: 005c83f0
// PROTOTYPE: eExpUpResult __thiscall ExpUp(CGoods * param_1, int param_2, ulong * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyProperty::DecordByteArray_Combine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\goods\cbattlefairyproperty.cpp:461
// RVA: 0x001C87F0
// ADDRESS: 005c87f0
// PROTOTYPE: bool __thiscall DecordByteArray_Combine(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
