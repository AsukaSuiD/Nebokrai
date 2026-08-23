//! Владелец монстра исторического `WorldServer`.
//!
//! Base-подобъект, полный `m_Property` и type-default внутри
//! `CMonster::CMonster`, а также непосредственной
//! destructor-цепочки и `GetFigure`
//! входят в контракт owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Layout сохраняет размеры старых `CMoveShape/CMonster` `0x80/0xB8` и
//! `CMonster::stProperty` `0x38`: `strOrginName` `+0x00`, unsigned `dwHP`
//! `+0x1C`, `wSign/wLeaderSign` `+0x20/+0x22`, signed
//! `lLeaderType/lLeaderID` `+0x24/+0x28`, `wLeaderDistance` `+0x2C`,
//! `lLiveTime` `+0x30` и `bDiedRemove` `+0x34`. Constructor по
//! передаёт неизменённый `this` в `CMoveShape::CMoveShape`, инициализирует
//! только собственную `strOrginName` в диапазоне `+0x84..+0x98`, а по
//! выполняет
//! `mov [esi+4], 0x258`. Последняя запись является object type `600` в
//! унаследованном `CBaseObject::m_lType`, а не ошибочно подписанным
//! `_padding_`.
//!
//! оригинал destructor ошибочно показывал ранний возврат после освобождения
//! heap-строки. подтверждает, что обе формы
//! строки сходятся на сбросе её состояния и затем tail-jump вызывают
//! `CMoveShape::~CMoveShape`. Rust-композиция материализует только единственный
//! действующий base-подобъект, весь property-state и type-default. `Vec<u8>`
//! сохраняет C-string lookup без SSO/heap lifetime старого ABI. Оригинальный
//! constructor не назначал восемь scalar-полей property, поэтому их чтение
//! было внутренним UB; World Rust не имеет их потребителей, а practical C++ и
//! детерминированными нулевыми defaults, не публикуя неопределённую память.
//! Original `GetFigure` разыменовывал null при отсутствующей setup-записи;
//! `Option` делает этот ошибочный внутренний путь явным и не выдумывает
//! внешне значимого figure. AI/region/container-семантика остаются оригинал, а
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

/// Действующая base-часть исходного `CMonster`.
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

 /// Присваивает унаследованное byte- имя до первого NUL.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

 /// Заимствует унаследованное byte- имя без завершающего NUL.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

 /// Присваивает унаследованный signed graphics ID.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.move_shape_base.set_graphics_id(graphics_id);
    }

 /// Присваивает единственное действующее строковое поле `m_Property`.
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
