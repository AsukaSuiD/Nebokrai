//! Runtime-состояние городских ворот GameServer `CCityGate`.
//!
//! Конструктор RVA `0x001DDB00` и override `SetAction` RVA `0x001DDB30`
//! имеют статус `IMPLEMENTED`; исходник `citygate.cpp` и объявления PDB,
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`. `CCityGate`
//! наследовал `CBuild`, получал type `0x4B0` через factory и отличался от
//! обычной постройки тем, что action `6` и `7` освобождают footprint, а любой
//! другой action ставит block `3`.
//!
//! Rust хранит только уже достигнутое runtime-состояние base/build: identity,
//! позицию, action/state, HP и параметры footprint. `BuildBlockUpdate`
//! оставляет изменение карты владельцу региона, но сохраняет момент эффекта:
//! action и `m_lChangeState` меняются до вызова старого `SetBlock`. AI, damage,
//! attacker set и combat callbacks ниже пока остаются `UNKNOWN` (исследовательский декомпилят хранится локально).

use super::build::{BuildBlockUpdate, legacy_build_title_tile};
use super::shape::{ShapeFigure, ShapeIdentity, ShapeView};
use crate::public::guid::CGuid;

pub(crate) const CITY_GATE_OBJECT_TYPE: u32 = 0x4B0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CityGateInit {
    pub(crate) id: i32,
    pub(crate) graphics_id: i32,
    pub(crate) region_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) direction: i32,
    pub(crate) action: u16,
    pub(crate) max_hp: i32,
    pub(crate) defence: i32,
    pub(crate) width_increment: i32,
    pub(crate) tile_x: i32,
    pub(crate) tile_y: i32,
    pub(crate) height_increment: i32,
    pub(crate) element_resistance: i32,
    pub(crate) script: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CCityGate {
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

impl CCityGate {
    /// Создаёт локальное представление уже успешно созданного factory-объекта.
    /// Начальный `SetAction` выполнил сам factory boundary, поэтому повторный
    /// tile-map effect здесь не выдаётся.
    pub(crate) fn from_created(init: CityGateInit) -> Self {
        let direction = if (0..8).contains(&init.direction) {
            init.direction
        } else {
            0
        };
        let script = if init.script.is_empty() || init.script == b"0" {
            Vec::new()
        } else {
            init.script
        };
        Self {
            object_type: CITY_GATE_OBJECT_TYPE,
            id: init.id,
            graphics_id: init.graphics_id,
            region_id: init.region_id,
            name: init.name,
            direction,
            state: 0,
            action: init.action,
            change_state: 0,
            hp: init.max_hp as u32,
            max_hp: init.max_hp as u32,
            defence: init.defence as u32,
            width_increment: init.width_increment,
            tile_x: legacy_build_title_tile(init.tile_x),
            tile_y: legacy_build_title_tile(init.tile_y),
            height_increment: init.height_increment,
            element_resistance: init.element_resistance as u32,
            script,
        }
    }

    /// Меняет action и возвращает точный отложенный эффект старого `SetBlock`.
    /// При неизменившемся action оригинал не трогал ни change-state, ни карту.
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
            block: if action == 6 || action == 7 { 0 } else { 3 },
        })
    }

    pub(crate) fn refresh_hp(&mut self) {
        self.hp = self.max_hp;
    }

    pub(crate) fn footprint(&self) -> BuildBlockUpdate {
        BuildBlockUpdate {
            region_id: self.region_id,
            tile_x: self.tile_x,
            tile_y: self.tile_y,
            width_increment: self.width_increment as u8,
            height_increment: self.height_increment as u8,
            block: 0,
        }
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
            block: if self.action == 6 || self.action == 7 {
                0
            } else {
                3
            },
        }
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp

// ============================================================================
// FUNCTION: CCityGate::CCityGate
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001DDB00
//
// IMPLEMENTED выше: `from_created` сохраняет factory identity/type и
// достигнутые CShape/CBuild-поля; Rust ownership заменяет цепочку деструкторов.
//

// ============================================================================
// FUNCTION: CCityGate::~CCityGate
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001DDB20
//
// IMPLEMENTED: owned `Vec` и обычный Rust `Drop` воспроизводят освобождение
// строк; vtable/destructor thunks не являются наблюдаемым контрактом.
//

// ============================================================================
// FUNCTION: CCityGate::SetAction
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// RVA: 0x001DDB30
//
// IMPLEMENTED выше: no-op при равном action, затем change-state `0` и
// отложенный `SetBlock` с `0` для actions `6/7`, иначе `3`.
//

// ============================================================================
// FUNCTION: CCityGate::AI_Stand
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:68
// RVA: 0x001DDB80
// ADDRESS: 005ddb80
// PROTOTYPE: long __thiscall AI_Stand(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCityGate::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:48
// RVA: 0x001DDBB0
// ADDRESS: 005ddbb0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCityGate::IsAttackAble
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:91
// RVA: 0x001DDC10
// ADDRESS: 005ddc10
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCityGate::OnBeenHurted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:177
// RVA: 0x001DDD80
// ADDRESS: 005ddd80
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: GameServer
