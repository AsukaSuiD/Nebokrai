//! Данные и скалярные правила городских ворот `CCityGate` (factory type
//! `0x4B0`) исторического GameServer, перенесённые в Zone `regions/` первой
//! порцией волны citygate. Исходный владелец — `appserver/citygate.h/.cpp`.
//! Переходный агрегат `CCityGate` остаётся в старом пакете поверх `CBuild`,
//! хранит те же колонки и делегирует сюда их поведение без изменения
//! сигнатур; нематериальные accessor-ы, hub-публикация владельцев
//! `CServerCityRegion`/`ServerCountryRegion` и evidence-блок остаются у
//! старого пакета.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). `symbols.py pubs` по этой паре даёт 41 символ семейства
//! `CCityGate`: собственные constructor `1:001dcb00`, destructor `1:001dcb20`,
//! scalar deleting destructor `1:001dcb90`, `SetAction` `1:001dcb30`, общие
//! `AI_Stand`/`AI_BeAttack`/`AI_Died` `1:001dcb80`, `AI` `1:001dcbb0`,
//! `IsAttackAble` `1:001dcc10`, `OnBeenHurted` `1:001dcd80`, vtable
//! `2:000138c4` с RTTI-записями, внешний `AddCityGate@CServerCityRegion`
//! `1:001cff00`, обработчики владельцев `OperatorCityGate`/`CityGateIsClose`/
//! `UpdateCityGateToClient` обоих регионов и instantiation-ы `std::_Tree`
//! карт `long → CCityGate*` и `long → tagCityGate`. Статусы общих
//! `AI_Stand`/`AI_BeAttack`/`AI_Died` (`1:001dcb80`) и `AI` (`1:001dcbb0`)
//! сохранены по evidence-блоку старого владельца без повторной сверки.
//!
//! Принятое решение по vtable: slot `+0x178` ворот указывает на общий no-op
//! thunk `0x00485540` (pub-символ `?OnDied@CCityGate@@UAEXXZ` = `1:00084540`)
//! вместо inherited `CBuild::OnDied` (`0x001DD9D0`, у владельца постройки
//! статус UNKNOWN), поэтому сохранённое script-поле ворот не исполняется из
//! death-pipeline. Особенность зафиксирована как поведение оригинала и не
//! «улучшается»; сам death-контракт ворот остаётся открытым вопросом и
//! переносится будущей порцией death-pipeline.

use nebokrai_shared::values::CGuid;

use super::ShapeIdentity;
use super::build::BuildBlockUpdate;

pub const CITY_GATE_OBJECT_TYPE: u32 = 0x4B0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CityGateInit {
    pub id: i32,
    pub graphics_id: i32,
    pub region_id: i32,
    pub name: Vec<u8>,
    pub direction: i32,
    pub action: u16,
    pub max_hp: i32,
    pub defence: i32,
    pub width_increment: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub height_increment: i32,
    pub element_resistance: i32,
    pub script: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CityGateHurtOwnerUpdate {
    pub region_id: i32,
    pub attacker_type: i32,
    pub attacker_id: i32,
}

/// Identity-перезапись constructor `CCityGate::CCityGate` (`0x001DDB00`):
/// после базовой инициализации `CBuild` с factory type `0x44C` ворота
/// заменяют identity типом `0x4B0` с пустым GUID.
pub const fn created_identity(id: i32) -> ShapeIdentity {
    ShapeIdentity {
        object_type: CITY_GATE_OBJECT_TYPE as i32,
        id,
        ex_id: CGuid::GUID_INVALID,
    }
}

/// Выбор начального action constructor: значение берётся из factory init как
/// есть, без guard-ов, потому что factory boundary уже выполнил первый
/// `SetAction` со своим tile-map эффектом; повторный эффект карты не выдаётся.
pub const fn created_action(action: u16) -> u16 {
    action
}

/// Block-правило action ворот — расширение правила постройки: actions `6`
/// (разрушение) и `7` освобождают footprint (`0`), любой другой action ставит
/// block `3`.
pub const fn gate_action_block(action: u16) -> u16 {
    if action == 7 {
        0
    } else {
        super::build::action_block(action)
    }
}

/// Снимок текущего block-эффекта footprint ворот по действующему action;
/// регион применяет его напрямую.
pub const fn block_snapshot(
    action: u16,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
    width_increment: i32,
    height_increment: i32,
) -> BuildBlockUpdate {
    BuildBlockUpdate {
        region_id,
        tile_x,
        tile_y,
        width_increment: width_increment as u8,
        height_increment: height_increment as u8,
        block: gate_action_block(action),
    }
}

/// Форма footprint ворот для region scan-а свободных клеток: block всегда
/// `0`, значимы только координаты и increments.
pub const fn footprint_snapshot(
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
    width_increment: i32,
    height_increment: i32,
) -> BuildBlockUpdate {
    BuildBlockUpdate {
        region_id,
        tile_x,
        tile_y,
        width_increment: width_increment as u8,
        height_increment: height_increment as u8,
        block: 0,
    }
}

/// Решение `CCityGate::SetAction` (`0x001DDB30`): неизменившийся action
/// остаётся no-op без записи change-state и обновления карты; при смене
/// действия применяется `gate_action_block` нового действия.
pub const fn decide_set_action_update(
    current_action: u16,
    next_action: u16,
    region_id: i32,
    tile_x: i32,
    tile_y: i32,
    width_increment: i32,
    height_increment: i32,
) -> Option<BuildBlockUpdate> {
    if current_action == next_action {
        return None;
    }
    Some(block_snapshot(
        next_action,
        region_id,
        tile_x,
        tile_y,
        width_increment,
        height_increment,
    ))
}

/// Exact `CCityGate::IsAttackAble` (`0x001DDC10`): null-attacker и derived
/// action `7` закрывают цель до базовых guards постройки (action `6` и
/// death); concrete country/city faction policy передаётся уже разрешённым
/// результатом owning региона closure без erased указателей.
pub fn is_attackable_in_region(
    attacker_present: bool,
    action: u16,
    hp: u32,
    region_allows: impl FnOnce() -> bool,
) -> bool {
    attacker_present
        && action != 7
        && super::build::is_attackable_in_region(action, hp, region_allows)
}

/// Exact `CCityGate::OnBeenHurted` (`0x001DDD80`): ворота не меняют
/// собственные HP/action и только возвращают два DWORD attacker-а owning
/// `CServerCityRegion`; non-null region pointer оригинала выражен совпадающим
/// region ID, который проверяет владелец.
pub const fn hurt_owner_update(
    region_id: i32,
    attacker_type: i32,
    attacker_id: i32,
) -> CityGateHurtOwnerUpdate {
    CityGateHurtOwnerUpdate {
        region_id,
        attacker_type,
        attacker_id,
    }
}
