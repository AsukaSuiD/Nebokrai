//! NPC `CNpc` из `npc.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Владелец расширяет `CMoveShape`, задаёт object type `500` и создаёт пустой
//! внутренний list. Текущий World-корпус не читает и не заполняет его, поэтому
//! элементам не назначается неподтверждённый доменный тип.
//!
//! Rust-композиция и Drop сохраняют один base-owner и порядок destructor chain
//! без копирования MSVC ABI, vtable или list internals.

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

pub(crate) struct CNpc {
    move_shape_base: CMoveShape,
    script_links: Vec<NpcScriptLink>,
}

impl CNpc {
    pub(crate) const fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(500);
        Self {
            move_shape_base,
            script_links: Vec::new(),
        }
    }

    pub(crate) const fn get_type(&self) -> i32 {
        self.move_shape_base.get_type()
    }

    pub(crate) const fn get_id(&self) -> i32 {
        self.move_shape_base.get_id()
    }

    pub(crate) const fn set_id(&mut self, id: i32) {
        self.move_shape_base.set_id(id);
    }

    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

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
