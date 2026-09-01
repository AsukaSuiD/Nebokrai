//! Runtime-состояние городских ворот GameServer `CCityGate`.
//!
//! Конструктор RVA `0x001DDB00` и override `SetAction` RVA `0x001DDB30`
//! имеют статус `IMPLEMENTED`; исходник `citygate.cpp` и объявления PDB,
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`. `CCityGate`
//! наследовал `CBuild`, получал type `0x4B0` через factory и отличался от
//! обычной постройки тем, что action `6` и `7` освобождают footprint, а любой
//! другой action ставит block `3`.
//!
//! Rust хранит derived-owner поверх canonical `CBuild/CMoveShape`: identity,
//! позиция, action/state и общий property block не дублируются. `BuildBlockUpdate`
//! оставляет изменение карты владельцу региона, но сохраняет момент эффекта:
//! action и `m_lChangeState` меняются до вызова старого `SetBlock`. Inherited
//! client serializer и damage принадлежат `CBuild`; достигнутая базовая атака
//! сохраняет derived attackability и hurt callback. Автономный AI ниже пока
//! остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).

use super::build::{BuildBlockUpdate, BuildInit, CBuild};
use super::moveshape::CMoveShape;
use super::shape::{ShapeIdentity, ShapeView};

pub(crate) const CITY_GATE_OBJECT_TYPE: u32 = 0x4B0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CityGateHurtOwnerUpdate {
    pub(crate) region_id: i32,
    pub(crate) attacker_type: i32,
    pub(crate) attacker_id: i32,
}

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
    build: CBuild,
}

impl CCityGate {
    /// Создаёт локальное представление уже успешно созданного factory-объекта.
    /// Начальный `SetAction` выполнил сам factory boundary, поэтому повторный
    /// tile-map effect здесь не выдаётся.
    pub(crate) fn from_created(init: CityGateInit) -> Self {
        let mut build = CBuild::from_created(BuildInit {
            id: init.id,
            graphics_id: init.graphics_id,
            region_id: init.region_id,
            name: init.name,
            direction: init.direction,
            max_hp: init.max_hp,
            defence: init.defence,
            width_increment: init.width_increment,
            tile_x: init.tile_x,
            tile_y: init.tile_y,
            height_increment: init.height_increment,
            element_resistance: init.element_resistance,
            script: init.script,
        });
        build.move_shape_mut().shape_mut().set_identity(ShapeIdentity {
            object_type: CITY_GATE_OBJECT_TYPE as i32,
            id: init.id,
            ex_id: crate::public::guid::CGuid::GUID_INVALID,
        });
        build.move_shape_mut().shape_mut().set_action(init.action);
        Self { build }
    }

    /// Меняет action и возвращает точный отложенный эффект старого `SetBlock`.
    /// При неизменившемся action оригинал не трогал ни change-state, ни карту.
    pub(crate) fn set_action(&mut self, action: u16) -> Option<BuildBlockUpdate> {
        if self.action() == action {
            return None;
        }
        self.build.move_shape_mut().shape_mut().set_change_state(0);
        self.build.move_shape_mut().shape_mut().set_action(action);
        Some(BuildBlockUpdate {
            region_id: self.region_id(),
            tile_x: self.build.tile_x,
            tile_y: self.build.tile_y,
            width_increment: self.build.width_increment as u8,
            height_increment: self.build.height_increment as u8,
            block: if action == 6 || action == 7 { 0 } else { 3 },
        })
    }

    pub(crate) fn refresh_hp(&mut self) {
        self.build.refresh_hp();
    }

    /// Exact reached `CCityGate::IsAttackAble` для уже разрешённого attacker-а:
    /// derived action `7` закрывает цель дополнительно к базовым action `6` и
    /// death guard. Country/city faction policy вычисляет concrete region
    /// owner, поэтому здесь нет обратной ссылки на erased `CRegion`.
    pub(crate) fn is_attackable_in_region(
        &self,
        attacker_present: bool,
        region_allows: impl FnOnce() -> bool,
    ) -> bool {
        attacker_present
            && self.action() != 7
            && self.build.is_attackable_in_region(region_allows)
    }

    pub(crate) fn footprint(&self) -> BuildBlockUpdate {
        BuildBlockUpdate {
            region_id: self.region_id(),
            tile_x: self.build.tile_x,
            tile_y: self.build.tile_y,
            width_increment: self.build.width_increment as u8,
            height_increment: self.build.height_increment as u8,
            block: 0,
        }
    }

    pub(crate) fn shape_view(&self) -> ShapeView {
        self.build.shape_view()
    }

    pub(crate) fn current_block_update(&self) -> BuildBlockUpdate {
        BuildBlockUpdate {
            region_id: self.region_id(),
            tile_x: self.build.tile_x,
            tile_y: self.build.tile_y,
            width_increment: self.build.width_increment as u8,
            height_increment: self.build.height_increment as u8,
            block: if self.action() == 6 || self.action() == 7 {
                0
            } else {
                3
            },
        }
    }

    /// Exact `CCityGate::OnBeenHurted`: сами ворота не меняют HP/action и
    /// только возвращают два DWORD для owning `CServerCityRegion`.
    pub(crate) const fn on_been_hurted(
        &self,
        attacker_type: i32,
        attacker_id: i32,
    ) -> CityGateHurtOwnerUpdate {
        CityGateHurtOwnerUpdate {
            region_id: self.region_id(),
            attacker_type,
            attacker_id,
        }
    }

    pub(crate) const fn move_shape(&self) -> &CMoveShape {
        self.build.move_shape()
    }

    pub(crate) const fn build(&self) -> &CBuild {
        &self.build
    }

    pub(crate) fn build_mut(&mut self) -> &mut CBuild {
        &mut self.build
    }

    pub(crate) const fn object_type(&self) -> u32 {
        self.build.object_type()
    }

    pub(crate) const fn id(&self) -> i32 {
        self.build.id()
    }

    pub(crate) const fn region_id(&self) -> i32 {
        self.build.region_id()
    }

    pub(crate) const fn action(&self) -> u16 {
        self.build.action()
    }

    pub(crate) fn name(&self) -> &[u8] {
        self.build.name()
    }

    pub(crate) const fn hp(&self) -> u32 {
        self.build.hp()
    }

    pub(crate) const fn max_hp(&self) -> u32 {
        self.build.max_hp()
    }

    pub(crate) fn encode_client_snapshot(
        &self,
        include_child: bool,
        now_ms: u32,
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.build
            .encode_client_snapshot(include_child, now_ms, timed_state_now_milliseconds)
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
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:91
// RVA: 0x001DDC10
// ADDRESS: 005ddc10
// PROTOTYPE: bool __thiscall IsAttackAble(CMoveShape * param_1)
//
// Реализовано выше как `is_attackable_in_region`: null-attacker, action
// `6/7` и death guards принадлежат derived owner-у; country/city faction
// решение передаёт concrete region caller. Достигнутый caller пока атакует
// игроком; monster/pet ветвь остаётся у соответствующего AI-прохода.
//
//

// ============================================================================
// FUNCTION: CCityGate::OnBeenHurted
// STATUS: IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:177
// RVA: 0x001DDD80
// ADDRESS: 005ddd80
// PROTOTYPE: void __thiscall OnBeenHurted(long param_1, long param_2)
//
// Реализовано выше как typed owner-update: non-null region pointer в safe
// модели выражен совпадающим region ID, два DWORD сохраняет CServerCityRegion.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// COMPONENT_VARIANT_END: GameServer
