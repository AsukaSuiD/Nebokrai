//! Runtime-состояние обычной GameServer-постройки `CBuild`.
//!
//! Конструктор RVA `0x001DD570`, `SetScriptFile` `0x001CBAC0`, `SetAction`
//! `0x001DD1C0`, `SetTileXY` `0x001DD2B0`, property accessors
//! `0x001DD5F0..0x001DD630` и destructor `0x001DD640` имеют статус
//! `IMPLEMENTED`; исходники `build.h/.cpp`, точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`. Type `0x44C`
//! подтверждён factory RVA `0x000FC190` и country flag decoder
//! `ServerCountryRegion::DecordFromByteArray` RVA `0x001CD3F0`.
//!
//! Rust хранит достигнутые identity/shape/property поля и byte-exact script.
//! `BuildBlockUpdate` применяет owning `CServerRegion`:
//! обычный `CBuild` освобождает клетку только action `6`, тогда как subclass
//! `CCityGate` также освобождает её при `7`. x87 `i32 -> f32 -> trunc i32`
//! для title coordinates сохранён общим helper-ом этого владельца. Owned
//! `Vec<u8>` и `Drop` заменяют `std::string`/destructor noise. `GetFigure`
//! материализован в общий `ShapeView`, поэтому country flags участвуют в
//! region membership и общем поиске боевых целей. Runtime context хранит
//! только внешнюю client publication. AI, combat, client
//! serialization и остальная поверхность ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально).

use super::shape::{ShapeFigure, ShapeIdentity, ShapeView};
use crate::public::guid::CGuid;

pub(crate) const BUILD_OBJECT_TYPE: u32 = 0x44C;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BuildInit {
    pub(crate) id: i32,
    pub(crate) graphics_id: i32,
    pub(crate) region_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) direction: i32,
    pub(crate) max_hp: i32,
    pub(crate) defence: i32,
    pub(crate) width_increment: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) height_increment: i32,
    pub(crate) element_resistance: i32,
    pub(crate) script: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BuildBlockUpdate {
    pub(crate) region_id: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) width_increment: u8,
    pub(crate) height_increment: u8,
    pub(crate) block: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BuildClientUpdate {
    pub(crate) object_type: u32,
    pub(crate) object_id: u32,
    pub(crate) action: u16,
    pub(crate) max_hp: u32,
    pub(crate) hp: u32,
}

pub(crate) trait BuildRuntimeContext {
    /// Кодирует `0xBF60F(type, id, action, max_hp, hp)` и шлёт вокруг build.
    fn send_build_update(&mut self, region_id: i32, build_id: i32, update: BuildClientUpdate);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CBuild {
    pub(crate) object_type: u32,
    pub(crate) id: i32,
    pub(crate) graphics_id: i32,
    pub(crate) region_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) direction: i32,
    pub(crate) state: u16,
    pub(crate) action: u16,
    pub(crate) change_state: i32,
    pub(crate) hp: u32,
    pub(crate) max_hp: u32,
    pub(crate) defence: u32,
    pub(crate) width_increment: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) height_increment: i32,
    pub(crate) element_resistance: u32,
    pub(crate) script: Vec<u8>,
}

impl CBuild {
    /// Создаёт локальное состояние уже успешно созданного factory type `0x44C`.
    pub(crate) fn from_created(init: BuildInit) -> Self {
        let direction = if (0..8).contains(&init.direction) {
            init.direction
        } else {
            0
        };
        let mut build = Self {
            object_type: BUILD_OBJECT_TYPE,
            id: init.id,
            graphics_id: init.graphics_id,
            region_id: init.region_id,
            name: init.name,
            direction,
            state: 0,
            action: 0,
            change_state: 0,
            hp: init.max_hp as u32,
            max_hp: init.max_hp as u32,
            defence: init.defence as u32,
            width_increment: init.width_increment,
            tile_x: legacy_build_title_tile(init.tile_x),
            tile_y: legacy_build_title_tile(init.tile_y),
            height_increment: init.height_increment,
            element_resistance: init.element_resistance as u32,
            script: Vec::new(),
        };
        if !init.script.is_empty() && init.script != b"0" {
            build.set_script_file(init.script);
        }
        build
    }

    pub(crate) fn set_script_file(&mut self, script: Vec<u8>) {
        self.script = script;
    }

    /// При неизменившемся action оригинал не менял change-state и карту.
    pub(crate) fn set_action(&mut self, action: u16) -> Option<BuildBlockUpdate> {
        if self.action == action {
            return None;
        }
        self.change_state = 0;
        self.action = action;
        Some(BuildBlockUpdate {
            region_id: self.region_id,
            tile_x: self.tile_x,
            tile_y: self.tile_y,
            width_increment: self.width_increment as u8,
            height_increment: self.height_increment as u8,
            block: if action == 6 { 0 } else { 3 },
        })
    }

    pub(crate) fn set_hp(&mut self, hp: u32) {
        self.hp = hp;
    }

    pub(crate) fn refresh_hp(&mut self) {
        self.set_hp(self.max_hp);
    }

    pub(crate) fn hp(&self) -> u32 {
        self.hp
    }

    pub(crate) fn max_hp(&self) -> u32 {
        self.max_hp
    }

    pub(crate) fn defence(&self) -> u32 {
        self.defence
    }

    pub(crate) fn element_resistance(&self) -> u32 {
        self.element_resistance
    }

    pub(crate) fn shape_view(&self) -> ShapeView {
        ShapeView {
            identity: ShapeIdentity {
                object_type: self.object_type as i32,
                id: self.id,
                ex_id: CGuid::GUID_INVALID,
            },
            tile_x: self.tile_x,
            tile_y: self.tile_y,
            pos_x_bits: (self.tile_x as f32).to_bits(),
            pos_y_bits: (self.tile_y as f32).to_bits(),
            figure: ShapeFigure::from_directions([
                self.height_increment as u8,
                self.height_increment as u8,
                self.width_increment as u8,
                self.width_increment as u8,
            ]),
        }
    }

    pub(crate) fn current_block_update(&self) -> BuildBlockUpdate {
        BuildBlockUpdate {
            region_id: self.region_id,
            tile_x: self.tile_x,
            tile_y: self.tile_y,
            width_increment: self.width_increment as u8,
            height_increment: self.height_increment as u8,
            block: if self.action == 6 { 0 } else { 3 },
        }
    }
}

pub(crate) fn legacy_build_title_tile(value: i32) -> i32 {
    // VERIFIED_DISASSEMBLY RVA 0x001DD2B0: `fild i32 -> fstp f32 ->
    // fld f32 -> fistp i32` под truncation control word. Округлённое вверх
    // `i32::MAX` становится x87 integer-indefinite `0x80000000`.
    let stored_float = value as f32;
    if stored_float >= 2_147_483_648.0_f32 {
        i32::MIN
    } else {
        stored_float as i32
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.h

// ============================================================================
// FUNCTION: CBuild::SetScriptFile
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001CBAC0
//
// Реализовано выше: byte-exact script assignment.
//

// ============================================================================
// FUNCTION: CBuild::AddToByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:32
// RVA: 0x001DD140
// ADDRESS: 005dd140
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:41
// RVA: 0x001DD180
// ADDRESS: 005dd180
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::SetAction
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD1C0
//
// Реализовано выше: change-state/action и single-cell block effect.
//

// ============================================================================
// FUNCTION: CBuild::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:67
// RVA: 0x001DD210
// ADDRESS: 005dd210
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::ApplyFinalDamage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:208
// RVA: 0x001DD270
// ADDRESS: 005dd270
// PROTOTYPE: void __thiscall ApplyFinalDamage(tagAttackInformation * param_1, vector<CMoveShape::tagDamage*,std::allocator<CMoveShape::tagDamage*>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::GetFigure
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:277
// RVA: 0x001DD280
// Реализовано выше в `shape_view`: directions `0/1` используют младший byte
// `height_increment`, `2/3` — `width_increment`, остальные дают ноль.

// ============================================================================
// FUNCTION: CBuild::SetTileXY
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD2B0
//
// Реализовано выше: x87-compatible coordinate conversion.
//

// ============================================================================
// FUNCTION: CBuild::GetAttackerDir
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:297
// RVA: 0x001DD300
// ADDRESS: 005dd300
// PROTOTYPE: long __thiscall GetAttackerDir(long param_1, long param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::GetBeAttackedPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:309
// RVA: 0x001DD350
// ADDRESS: 005dd350
// PROTOTYPE: void __thiscall GetBeAttackedPoint(long param_1, long param_2, long * param_3, long * param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:254
// RVA: 0x001DD520
// ADDRESS: 005dd520
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::CBuild
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD570
//
// Реализовано выше: type 0x44C и zeroed property/action state.
//

// ============================================================================
// FUNCTION: CBuild::GetHP
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD5F0
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::SetHP
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD600
//
// Реализовано выше: property mutation.
//

// ============================================================================
// FUNCTION: CBuild::GetMaxHP
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD610
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::GetDef
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD620
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::GetElementResistant
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD630
//
// Реализовано выше: property accessor.
//

// ============================================================================
// FUNCTION: CBuild::~CBuild
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp
// RVA: 0x001DD640
//
// Реализовано выше: owned Vec/Drop replacement.
//

// ============================================================================
// FUNCTION: CBuild::OnBeenAttacked
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:104
// RVA: 0x001DD6B0
// ADDRESS: 005dd6b0
// PROTOTYPE: void __thiscall OnBeenAttacked(tagAttackInformation * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBuild::OnDied
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\build.cpp:214
// RVA: 0x001DD9D0
// ADDRESS: 005dd9d0
// PROTOTYPE: void __thiscall OnDied(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
