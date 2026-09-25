//! Монстр `CMonster` из `monster.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//! Перенесён в Realm `characters/`.
//!
//! Владелец расширяет `CMoveShape`, задаёт object type `600` и хранит original
//! name, HP, leader fields, lifetime и died-remove. Scalar properties конструктор
//! оригинала не назначал; Rust использует нули, поскольку они не публикуются
//! до загрузки и неопределённая память не является контрактом.
//!
//! `GetFigure` требует найденную setup-запись; отсутствие возвращает `None`
//! вместо null-dereference. `Vec<u8>` и обычный Drop заменяют MSVC string и
//! destructor chain без изменения игрового состояния.

use nebokrai_shared::resources::{MonsterRegistry, get_monster_property_by_origin_name};

use crate::regions::moveshape::CMoveShape;

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

pub struct CMonster {
    move_shape_base: CMoveShape,
    property: MonsterProperty,
}

impl CMonster {
    pub fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(600);
        Self {
            move_shape_base,
            property: MonsterProperty::default(),
        }
    }

    pub const fn get_type(&self) -> i32 {
        self.move_shape_base.get_type()
    }

    pub const fn get_id(&self) -> i32 {
        self.move_shape_base.get_id()
    }

    pub const fn set_id(&mut self, id: i32) {
        self.move_shape_base.set_id(id);
    }

    pub fn set_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

    pub fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

    pub const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.move_shape_base.set_graphics_id(graphics_id);
    }

    pub fn set_original_name(&mut self, original_name: Vec<u8>) {
        self.property.original_name = original_name;
    }

 /// Возвращает low-byte setup `dwFigure` либо отсутствие setup-записи.
 ///
 /// Точное приведение `u32` к `uchar` сохраняет младшие восемь бит.
    pub fn get_figure(&self, monsters: &MonsterRegistry) -> Option<u8> {
        get_monster_property_by_origin_name(monsters, &self.property.original_name)
            .map(|properties| properties.figure as u8)
    }
}
