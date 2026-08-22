//! Владелец монстра исторического `WorldServer`.
//!
//! Статус base-подобъекта, полного `m_Property` и type-default внутри
//! `CMonster::CMonster` RVA `0x000E0490`, а также непосредственной
//! destructor-цепочки RVA `0x000E0410` и `GetFigure` RVA `0x000E0460` —
//! `IMPLEMENTED`. Остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.h` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp:11-16`.
//!
//! Exact PDB задаёт размеры старых `CMoveShape/CMonster` `0x80/0xB8` и
//! `CMonster::stProperty` `0x38`: `strOrginName` `+0x00`, unsigned `dwHP`
//! `+0x1C`, `wSign/wLeaderSign` `+0x20/+0x22`, signed
//! `lLeaderType/lLeaderID` `+0x24/+0x28`, `wLeaderDistance` `+0x2C`,
//! `lLiveTime` `+0x30` и `bDiedRemove` `+0x34`. Constructor по `0x004E0493`
//! передаёт неизменённый `this` в `CMoveShape::CMoveShape`, инициализирует
//! только собственную `strOrginName` в диапазоне `+0x84..+0x98`, а по
//! `0x004E04B6` выполняет
//! `mov [esi+4], 0x258`. Последняя запись является object type `600` в
//! унаследованном `CBaseObject::m_lType`, а не ошибочно подписанным
//! `_padding_`.
//!
//! Raw destructor ошибочно показывал ранний возврат после освобождения
//! heap-строки. Exact EXE `0x004E0419..0x004E044C` подтверждает, что обе формы
//! строки сходятся на сбросе её состояния и затем tail-jump вызывают
//! `CMoveShape::~CMoveShape`. Rust-композиция материализует только единственный
//! достигнутый base-подобъект, весь property-state и type-default. `Vec<u8>`
//! сохраняет C-string lookup без SSO/heap lifetime старого ABI. Оригинальный
//! constructor не назначал восемь scalar-полей property, поэтому их чтение
//! было внутренним UB; World Rust не имеет их потребителей, а practical C++ и
//! Linux-донор согласованно задают нули. Rust исправляет дефект сразу
//! детерминированными нулевыми defaults, не публикуя неопределённую память.
//! Original `GetFigure` разыменовывал null при отсутствующей setup-записи;
//! `Option` делает этот ошибочный внутренний путь явным и не выдумывает
//! внешне значимого figure. AI/region/container-семантика остаются raw, а
//! destructor не подменяется пустым `Drop`. Rust layout не объявляется копией
//! старого ABI.

use crate::setup::monsterlist::{MonsterRegistry, get_monster_property_by_origin_name};

use super::moveshape::CMoveShape;

/// Полное состояние старого `CMonster::stProperty` без MSVC string ABI.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct MonsterProperty {
    original_name: Vec<u8>,
    hp: u32,
    sign: u16,
    leader_sign: u16,
    leader_type: i32,
    leader_id: i32,
    leader_distance: u16,
    live_time: i32,
    died_remove: bool,
}

/// Достигнутая base-часть исходного `CMonster`.
pub(crate) struct CMonster {
    move_shape_base: CMoveShape,
    property: MonsterProperty,
}

impl CMonster {
    /// Создаёт только доказанный base-подобъект с object type `600`.
    pub(crate) fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(600);
        Self {
            move_shape_base,
            property: MonsterProperty::default(),
        }
    }

    /// Возвращает унаследованный object type без дополнительных эффектов.
    pub(crate) const fn get_type(&self) -> i32 {
        self.move_shape_base.get_type()
    }

    /// Возвращает унаследованный signed object ID.
    pub(crate) const fn get_id(&self) -> i32 {
        self.move_shape_base.get_id()
    }

    /// Присваивает унаследованный signed object ID.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.move_shape_base.set_id(id);
    }

    /// Присваивает унаследованное byte-exact имя до первого NUL.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

    /// Заимствует унаследованное byte-exact имя без завершающего NUL.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

    /// Присваивает унаследованный signed graphics ID.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.move_shape_base.set_graphics_id(graphics_id);
    }

    /// Присваивает единственное достигнутое строковое поле `m_Property`.
    pub(crate) fn set_original_name(&mut self, original_name: Vec<u8>) {
        self.property.original_name = original_name;
    }

    /// Возвращает low-byte setup `dwFigure` либо отсутствие setup-записи.
    ///
    /// Точное приведение `u32` к `uchar` сохраняет младшие восемь бит.
    pub(crate) fn get_figure(&self, monsters: &MonsterRegistry) -> Option<u8> {
        get_monster_property_by_origin_name(monsters, &self.property.original_name)
            .map(|properties| properties.figure as u8)
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.h

// ============================================================================
// FUNCTION: CMonster::~CMonster
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp:16
// RVA: 0x000E0410
// ADDRESS: 004e0410
// PROTOTYPE: void __thiscall ~CMonster(void)
//
// Реализовано обычным Rust `Drop` полей `property` и `move_shape_base`.
// Точная цепочка не имеет внешних побочных действий; SSO/heap-ветки старого ABI
// являются только внутренней технической деталью владения.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetFigure
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.h:36
// RVA: 0x000E0460
// ADDRESS: 004e0460
// PROTOTYPE: uchar __thiscall GetFigure(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CMonster
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp:11
// RVA: 0x000E0490
// ADDRESS: 004e0490
// PROTOTYPE: undefined __thiscall CMonster(void)
//
// Реализовано выше как `with_constructor_base_and_type`: конструктор создаёт
// достигнутый `CMoveShape`, полный zeroed property-state (исправление
// неинициализированных scalar-полей) и затем ставит object type `600`. Rust
// не переносит старый SSO layout строки и vtable-назначение.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
