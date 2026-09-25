//! Городские ворота CCityGate (0x4B0) поверх постройки CBuild.
//! Источник: gameserver.exe + GameServer.pdb, appserver/citygate.cpp.
//!
//! Определения данных и скалярные правила перенесены в Zone
//! `regions/citygate`; здесь переходный агрегат `CCityGate` с прежними
//! сигнатурами, реэкспорт семейства для старого пакета и evidence-блок.
//!
//! Rust хранит derived-owner поверх canonical `CBuild/CMoveShape`: identity,
//! позиция, action/state и общий property block не дублируются. `BuildBlockUpdate`
//! оставляет изменение карты владельцу региона, но сохраняет момент эффекта:
//! action и `m_lChangeState` меняются до вызова старого `SetBlock`. Inherited
//! client serializer и damage принадлежат `CBuild`; достигнутая базовая атака
//! сохраняет derived attackability и hurt callback. Все три action-ветви
//! `CCityGate::AI` вызывают один `AI_BeAttack`, который в точном EXE является
//! нулевым no-op; отдельный runtime ворот не требуется. Vtable slot `+0x178`
//! также заменяет inherited `CBuild::OnDied` точным no-op `0x00485540`, так
//! что сохранённое script-поле ворот не исполняется из death pipeline.

pub(crate) use nebokrai_zone::regions::citygate::*;

use super::build::{BuildBlockUpdate, BuildInit, CBuild};
use super::moveshape::CMoveShape;
use super::shape::ShapeView;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CCityGate {
    build: CBuild,
}

impl CCityGate {
    /// Создаёт локальное представление уже успешно созданного factory-объекта.
    /// Начальный `SetAction` выполнил сам factory boundary, поэтому повторный
    /// tile-map effect здесь не выдаётся; identity-перезапись `0x4B0` и выбор
    /// начального action принадлежат общим правилам Zone citygate.
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
        build
            .move_shape_mut()
            .shape_mut()
            .set_identity(nebokrai_zone::regions::citygate::created_identity(init.id));
        build
            .move_shape_mut()
            .shape_mut()
            .set_action(nebokrai_zone::regions::citygate::created_action(init.action));
        Self { build }
    }

    /// Меняет action и возвращает точный отложенный эффект старого `SetBlock`.
    /// При неизменившемся action оригинал не трогал ни change-state, ни карту;
    /// решение и block-эффект принадлежат общей операции Zone citygate.
    pub(crate) fn set_action(&mut self, action: u16) -> Option<BuildBlockUpdate> {
        let update = nebokrai_zone::regions::citygate::decide_set_action_update(
            self.action(),
            action,
            self.region_id(),
            self.build.tile_x(),
            self.build.tile_y(),
            self.build.width_increment,
            self.build.height_increment,
        )?;
        self.build.move_shape_mut().shape_mut().set_change_state(0);
        self.build.move_shape_mut().shape_mut().set_action(action);
        Some(update)
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
        nebokrai_zone::regions::citygate::is_attackable_in_region(
            attacker_present,
            self.action(),
            self.build.hp(),
            region_allows,
        )
    }

    pub(crate) fn footprint(&self) -> BuildBlockUpdate {
        nebokrai_zone::regions::citygate::footprint_snapshot(
            self.region_id(),
            self.build.tile_x(),
            self.build.tile_y(),
            self.build.width_increment,
            self.build.height_increment,
        )
    }

    pub(crate) fn shape_view(&self) -> ShapeView {
        self.build.shape_view()
    }

    pub(crate) fn current_block_update(&self) -> BuildBlockUpdate {
        nebokrai_zone::regions::citygate::block_snapshot(
            self.action(),
            self.region_id(),
            self.build.tile_x(),
            self.build.tile_y(),
            self.build.width_increment,
            self.build.height_increment,
        )
    }

    /// Exact `CCityGate::OnBeenHurted`: сами ворота не меняют HP/action и
    /// только возвращают два DWORD для owning `CServerCityRegion`.
    pub(crate) const fn on_been_hurted(
        &self,
        attacker_type: i32,
        attacker_id: i32,
    ) -> CityGateHurtOwnerUpdate {
        nebokrai_zone::regions::citygate::hurt_owner_update(
            self.region_id(),
            attacker_type,
            attacker_id,
        )
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
        timed_state_now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.build
            .encode_client_snapshot(include_child, timed_state_now_milliseconds)
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
// FUNCTION: CCityGate::AI_BeAttack
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:68
// RVA: 0x001DDB80
// ADDRESS: 005ddb80
// PROTOTYPE: long __thiscall AI_BeAttack(void)
//
// PDB-символ `AI_BeAttack` по `0x005DDB80` — tail-jump в общий
// `0x00601200` (`xor eax,eax; ret`). Старое имя `AI_Stand` было ошибкой
// декомпилятора.
//

// ============================================================================
// FUNCTION: CCityGate::AI
// STATUS: IMPLEMENTED, VERIFIED_DISASSEMBLY
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\citygate.cpp:48
// RVA: 0x001DDBB0
// ADDRESS: 005ddbb0
// PROTOTYPE: void __thiscall AI(void)
//
// Vtable `0x0065E8C4`: slots `+0x1A4/+0x1A8/+0x1AC` все указывают на
// `AI_BeAttack` выше. Action-dispatch не имеет наблюдаемого side effect.
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
