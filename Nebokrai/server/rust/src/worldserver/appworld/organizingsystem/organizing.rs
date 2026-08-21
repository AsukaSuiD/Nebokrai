//! Владелец общего organizing-состояния исторического `WorldServer`.
//!
//! Статус PDB-layout `COrganizing::tagMemInfo` и его вложенного
//! `ePurview/ePurviewOwnState` — `IMPLEMENTED`; полный `COrganizing`,
//! billboard-типы и их методы ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp`.
//!
//! Полная PDB-запись type index `0x5AE8` задаёт размер `tagMemInfo` `0xF0` и
//! одиннадцать членов: signed `long lID` `+0x00`, `char strName[32]` `+0x04`,
//! signed `long lLvl/lOccu/lJobLvl` `+0x24/+0x28/+0x2C`,
//! `char strTitle[64]` `+0x30`, `ePurviewOwnState listPV[11]` `+0x70`,
//! `char strRegion[64]` `+0x9C`, `tagTime LastOnlineTime` `+0xDC` и
//! `bool bControbute` `+0xEC`. `tagTime` type index `0x3268` состоит из восьми
//! `unsigned short` в порядке `wYear..wMilliseconds` и занимает `0x10`;
//! `ePurviewOwnState` type index `0x5AE2` имеет signed 32-битную основу и
//! значения `PST_No=0`, `PST_Forbid=1`, `PST_Permit=2`. Три последних байта
//! старого `tagMemInfo` являются выравниванием, а не полем.
//!
//! Rust сохраняет доказанный layout через `repr(C)` и compile-time offsets,
//! потому что faction/union публикуют `listPV` сырым блоком `0x2C`, а
//! `LastOnlineTime` — блоком `0x10`. Сами блоки строятся явно в little-endian:
//! ни padding всего объекта, ни native Rust memory не отправляются в wire.
//! Plain `char` хранится как byte-exact `u8`; фиксированные массивы не
//! заменяются `String`/`Vec`, а C-string view заканчивается на первом NUL и
//! включает его. Отсутствующий NUL был бы старым чтением за границей массива;
//! безопасная граница возвращает локальный `BLOCKED_MISSING_FACT`, не
//! придумывая наблюдаемую реакцию.
//!
//! `CFaction::Initial` RVA `0x000BD950` полностью заполняет локальный
//! `tagMemInfo` мастера и копирует ровно `0xF0` bytes в `m_Members`.
//! Контейнерный `_Buynode` RVA `0x000B52C0` копирует key вместе со всеми
//! `0xF0` bytes значения; `Copy` заменяет этот trivially-copyable механизм без
//! отдельной STL-семантики. В отличие от полного значения, COMDAT
//! `map::operator[]` RVA `0x000BA580` вызывает только нулевой constructor
//! `LastOnlineTime` и оставляет остальные primitive/array bytes
//! неопределёнными. Поэтому Rust намеренно не реализует `Default`: безопасное
//! создание требует все поля сразу, а будущий `m_Members` не сможет незаметно
//! выбрать нули вместо старой странности.
//!
//! Полные порядки `CFaction::AddMembersToByteArray` RVA `0x000B53D0` и
//! `UpdateMemberInfoToClient` RVA `0x000BA7C0` принадлежат `faction.rs`. Этот
//! owner предоставляет доказанные общие `eOperator`, layout и wire-проекции
//! фиксированных C-буферов, `listPV` и `LastOnlineTime`; он не вводит
//! универсальный serializer и не смешивает faction/union форматы. PDB type
//! `0x5A22` задаёт `eOperator` как signed 32-bit enum со значениями
//! `OP_Delete=0`, `OP_Add=1`, `OP_Update=2`.
//! PDB также задаёт общий `eCityState`: `CIS_NO=0`, `CIS_DUTH=1`,
//! `CIS_Mass=2`, `CIS_Fight=3`; его используют Village/AttackCity owners.

use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

const MEMBER_NAME_CAPACITY: usize = 32;
const MEMBER_TEXT_CAPACITY: usize = 64;
const PURVIEW_COUNT: usize = 11;

/// Три общих organizing-оператора с точным signed wire-значением.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum EOperator {
    Delete = 0,
    Add = 1,
    Update = 2,
}

/// Четыре точных состояния organizing-war региона.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum ECityState {
    No = 0,
    Duth = 1,
    Mass = 2,
    Fight = 3,
}

/// Одиннадцать точных organizing-прав из PDB `COrganizing::ePurview`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum EPurview {
    Disband = 0,
    Exit = 1,
    DubJobLevel = 2,
    ConMem = 3,
    FireOut = 4,
    Pronounce = 5,
    LeaveWord = 6,
    EditLeaveWord = 7,
    ObtainTax = 8,
    OperCityGate = 9,
    EndueRor = 10,
}

impl EPurview {
    /// Отделяет допустимый enum-контракт от произвольного входного `long`.
    pub(crate) const fn from_wire_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Disband),
            1 => Some(Self::Exit),
            2 => Some(Self::DubJobLevel),
            3 => Some(Self::ConMem),
            4 => Some(Self::FireOut),
            5 => Some(Self::Pronounce),
            6 => Some(Self::LeaveWord),
            7 => Some(Self::EditLeaveWord),
            8 => Some(Self::ObtainTax),
            9 => Some(Self::OperCityGate),
            10 => Some(Self::EndueRor),
            _ => None,
        }
    }

    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}

impl ECityState {
    /// Возвращает исходное signed значение enum для wire и message boundaries.
    pub(crate) const fn wire_value(self) -> i32 {
        self as i32
    }
}

impl EOperator {
    /// Возвращает значение исходного `eOperator` для `CBaseMessage::Add(long)`.
    pub(crate) const fn wire_value(self) -> i32 {
        self as i32
    }
}

/// Три состояния одного organizing-права с точным signed wire-значением.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum EPurviewOwnState {
    No = 0,
    Forbid = 1,
    Permit = 2,
}

impl EPurviewOwnState {
    /// Возвращает доказанное 32-битное значение элемента `listPV`.
    pub(crate) const fn wire_value(self) -> i32 {
        self as i32
    }
}

/// Общий typed-результат concrete мутации одного `tagMemInfo::listPV`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MemberPurviewMutation {
    InvalidPurview,
    MemberNotFound,
    Unchanged,
    Changed,
}

/// Вложенное значение `tagTime`, нужное полному layout `tagMemInfo`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(crate) struct TagTimeValue {
    pub(crate) year: u16,
    pub(crate) month: u16,
    pub(crate) day_of_week: u16,
    pub(crate) day: u16,
    pub(crate) hour: u16,
    pub(crate) minute: u16,
    pub(crate) second: u16,
    pub(crate) milliseconds: u16,
}

impl TagTimeValue {
    /// Строит ровно тот 16-байтовый little-endian блок, который отправлял owner.
    pub(crate) fn wire_bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        for (chunk, value) in bytes.chunks_exact_mut(2).zip([
            self.year,
            self.month,
            self.day_of_week,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.milliseconds,
        ]) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

/// Ошибка безопасного C-string view одного фиксированного member-поля.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnterminatedMemberField {
    pub(crate) field: &'static str,
}

impl fmt::Display for UnterminatedMemberField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "в фиксированном поле {} отсутствует завершающий NUL",
            self.field
        )
    }
}

impl Error for UnterminatedMemberField {}

/// Полное доказанное значение исходного `COrganizing::tagMemInfo`.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct TagMemInfo {
    pub(crate) id: i32,
    pub(crate) name: [u8; MEMBER_NAME_CAPACITY],
    pub(crate) level: i32,
    pub(crate) occupation: i32,
    pub(crate) job_level: i32,
    pub(crate) title: [u8; MEMBER_TEXT_CAPACITY],
    pub(crate) purview: [EPurviewOwnState; PURVIEW_COUNT],
    pub(crate) region: [u8; MEMBER_TEXT_CAPACITY],
    pub(crate) last_online_time: TagTimeValue,
    pub(crate) contribute: bool,
}

impl TagMemInfo {
    /// Создаёт только полностью определённое member-значение; старого частично
    /// неинициализированного `map::operator[]` аналога намеренно нет.
    #[allow(
        clippy::too_many_arguments,
        reason = "параметры один к одному сохраняют десять доказанных data-полей tagMemInfo"
    )]
    pub(crate) const fn from_complete_fields(
        id: i32,
        name: [u8; MEMBER_NAME_CAPACITY],
        level: i32,
        occupation: i32,
        job_level: i32,
        title: [u8; MEMBER_TEXT_CAPACITY],
        purview: [EPurviewOwnState; PURVIEW_COUNT],
        region: [u8; MEMBER_TEXT_CAPACITY],
        last_online_time: TagTimeValue,
        contribute: bool,
    ) -> Self {
        Self {
            id,
            name,
            level,
            occupation,
            job_level,
            title,
            purview,
            region,
            last_online_time,
            contribute,
        }
    }

    /// Возвращает `strName` до первого NUL включительно.
    pub(crate) fn name_wire_bytes(&self) -> Result<&[u8], UnterminatedMemberField> {
        terminated_field(&self.name, "strName")
    }

    /// Возвращает `strTitle` до первого NUL включительно.
    pub(crate) fn title_wire_bytes(&self) -> Result<&[u8], UnterminatedMemberField> {
        terminated_field(&self.title, "strTitle")
    }

    /// Возвращает `strRegion` до первого NUL включительно.
    pub(crate) fn region_wire_bytes(&self) -> Result<&[u8], UnterminatedMemberField> {
        terminated_field(&self.region, "strRegion")
    }

    /// Строит точный `0x2C` little-endian блок `listPV`.
    pub(crate) fn purview_wire_bytes(&self) -> [u8; 44] {
        let mut bytes = [0; 44];
        for (chunk, state) in bytes.chunks_exact_mut(4).zip(self.purview) {
            chunk.copy_from_slice(&state.wire_value().to_le_bytes());
        }
        bytes
    }

    /// Строит точный `0x10` блок `LastOnlineTime`.
    pub(crate) fn last_online_wire_bytes(&self) -> [u8; 16] {
        self.last_online_time.wire_bytes()
    }
}

fn terminated_field<'a>(
    field: &'a [u8],
    name: &'static str,
) -> Result<&'a [u8], UnterminatedMemberField> {
    let Some(terminator) = field.iter().position(|byte| *byte == 0) else {
        // BLOCKED_MISSING_FACT: исходные char*-overload-ы продолжали бы чтение
        // за фиксированным массивом. Достижимость и наблюдаемая реакция такого
        // состояния не доказаны и не заменяются добавленным NUL или unsafe.
        return Err(UnterminatedMemberField { field: name });
    };
    Ok(&field[..=terminator])
}

const _: () = {
    assert!(size_of::<EOperator>() == 4);
    assert!(size_of::<EPurview>() == 4);
    assert!(size_of::<EPurviewOwnState>() == 4);
    assert!(size_of::<TagTimeValue>() == 0x10);
    assert!(size_of::<TagMemInfo>() == 0xF0);
    assert!(offset_of!(TagMemInfo, id) == 0x00);
    assert!(offset_of!(TagMemInfo, name) == 0x04);
    assert!(offset_of!(TagMemInfo, level) == 0x24);
    assert!(offset_of!(TagMemInfo, occupation) == 0x28);
    assert!(offset_of!(TagMemInfo, job_level) == 0x2C);
    assert!(offset_of!(TagMemInfo, title) == 0x30);
    assert!(offset_of!(TagMemInfo, purview) == 0x70);
    assert!(offset_of!(TagMemInfo, region) == 0x9C);
    assert!(offset_of!(TagMemInfo, last_online_time) == 0xDC);
    assert!(offset_of!(TagMemInfo, contribute) == 0xEC);
};

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.h

// ============================================================================
// FUNCTION: Catch@00433ca1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x00033CA1
// ADDRESS: 00433ca1
// PROTOTYPE: undefined Catch@00433ca1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerList::tagPropertiesUpgrade::~tagPropertiesUpgrade
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x00033F90
// ADDRESS: 00433f90
// PROTOTYPE: void __thiscall ~tagPropertiesUpgrade(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_enum_eCrOrgResult___thiscall_COrganizingCtrl::CreateConfederation(long,long,std::basic_string<char,std::char_traits<char>,std::allocator<char>_>&)'::__l28::CreateUnion::Release`adjustor{4}'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x00034FF0
// ADDRESS: 00434ff0
// PROTOTYPE: void __thiscall Release`adjustor{4}'(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: `public:_bool___thiscall_COrganizingCtrl::TransferIOwnerCity(long,long,long)'::__l42::PlayerTransferOwnerCity::Release`adjustor{4}'
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x00035000
// ADDRESS: 00435000
// PROTOTYPE: void __thiscall Release`adjustor{4}'(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043579a
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x0003579A
// ADDRESS: 0043579a
// PROTOTYPE: undefined Catch@0043579a()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004358ba
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x000358BA
// ADDRESS: 004358ba
// PROTOTYPE: undefined Catch@004358ba()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0043622e
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x0003622E
// ADDRESS: 0043622e
// PROTOTYPE: undefined Catch@0043622e()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// ============================================================================
// FUNCTION: Unwind@0052d560
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x0012D560
// ADDRESS: 0052d560
// PROTOTYPE: undefined Unwind@0052d560()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052d580
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x0012D580
// ADDRESS: 0052d580
// PROTOTYPE: undefined Unwind@0052d580()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@0052d5c0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\organizingsystem\organizing.cpp
// RVA: 0x0012D5C0
// ADDRESS: 0052d5c0
// PROTOTYPE: undefined Unwind@0052d5c0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


























// COMPONENT_VARIANT_END: WorldServer
