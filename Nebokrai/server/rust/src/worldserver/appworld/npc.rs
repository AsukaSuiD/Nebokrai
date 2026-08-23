//! Владелец NPC исторического `WorldServer`.
//!
//! Base-подобъект, type-default и lifecycle собственного списка внутри
//! `CNpc::CNpc/~CNpc` представлены действующим
//! Rust-owner-ом. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Layout сохраняет размеры старых `CMoveShape/CNpc` `0x80/0x8C`. Конструктор
//! по первым передаёт неизменённый `this` в
//! `CMoveShape::CMoveShape`, создаёт собственный `m_listScript` с offset
//! `+0x80`, а по выполняет `mov [esi+4], 0x1F4`. Последняя запись
//! является object type `500` в унаследованном `CBaseObject::m_lType`, а не
//! ошибочно подписанным `_padding_`. Destructor сначала
//! очищает тот же список, затем вызывает `CMoveShape::~CMoveShape`.
//!
//! Rust-композиция материализует единственный действующий `CMoveShape`
//! base-подобъект, type-default и пустой non-owning `m_listScript`.
//! корпус не записывает в этот список и не читает его элементов; их
//! временное имя `tagEnemyFaction*` не выдаётся за подтверждённый Rust-тип.
//! `Drop` сначала очищает storage списка, затем автоматически освобождает
//! base-owner, как исходный destructor. AI, region и container-семантика
//! остаются в оригинал-блоках; Rust layout не является копией старого ABI.

use super::moveshape::CMoveShape;

/// Opaque non-owning entry точного `CNpc::m_listScript`.
///
/// В исследованном World-корпусе нет writer/reader этого списка, поэтому
/// хранится только host-address без выдуманного типа или ownership pointee.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NpcScriptLink {
    address: usize,
}

/// Действующий owner `CNpc`.
pub(crate) struct CNpc {
    move_shape_base: CMoveShape,
    script_links: Vec<NpcScriptLink>,
}

impl CNpc {
 /// Создаёт `CMoveShape`, пустой `m_listScript` и object type `500`.
    pub(crate) const fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(500);
        Self {
            move_shape_base,
            script_links: Vec::new(),
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
}

impl Drop for CNpc {
    fn drop(&mut self) {
 // `~CNpc` сначала вызывает `_Tidy(m_listScript)`, и лишь затем
 // `CMoveShape::~CMoveShape`. Pointee не owned старым list, поэтому
 // очистка Rust storage не освобождает неизвестные external objects.
        self.script_links.clear();
    }
}
