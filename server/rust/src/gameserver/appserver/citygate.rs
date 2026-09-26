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
