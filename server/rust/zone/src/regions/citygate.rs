//! Данные и скалярные правила городских ворот `CCityGate` (factory type
//! `0x4B0`) и их региональное гейтовое тело. Исходный владелец —
//! `appserver/citygate.h/.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Переходный агрегат `CCityGate` остаётся в старом пакете
//! поверх `CBuild` и делегирует сюда поведение колонок; нематериальные
//! accessor-ы, hub-публикация владельцев `CServerCityRegion`/
//! `ServerCountryRegion`, region-обходы карт и применение block-эффектов
//! остаются у переходных aggregate-ов. Статусы общих AI-семьи унаследованы от
//! шапки старого владельца без повторной сверки.
//!
//! Принятое решение по vtable: slot `+0x178` ворот указывает на общий no-op
//! thunk вместо inherited `CBuild::OnDied`, поэтому сохранённое script-поле
//! ворот не исполняется из death-pipeline. Особенность зафиксирована как
//! поведение оригинала и не «улучшается»; сам death-контракт ворот остаётся
//! открытым вопросом.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#npc-и-базовые-фигуры

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

/// Opcode-константы семьи `OperatorCityGate`; city владелец дополнительно
/// достигает `OC_Died`, country pointer-overload для него и для неизвестных
/// операций успешно ничего не делает.
pub const GATE_OP_OPEN: i32 = 0;
pub const GATE_OP_CLOSE: i32 = 1;
pub const GATE_OP_REFRESH: i32 = 2;
pub const GATE_OP_DIED: i32 = 3;

/// Wire-проекция действующего action ворот в client gate state: `7` —
/// открытые (`0`), `0 | 1` — закрытые (`1`), `6` — разрушенные (`2`),
/// остальные действия исходно дают `-1`. Обе region-семьи (city и country)
/// читают одно и то же отображение.
pub const fn gate_client_state(action: u16) -> i32 {
    match action {
        7 => 0,
        0 | 1 => 1,
        6 => 2,
        _ => -1,
    }
}

/// Скалярное решение gate-операции owning региона: выполнить ли предварительный
/// `RefreshHP` и какой `SetAction` применять затем. Сам эффект карты и
/// выбор concrete gate остаются у region-владельца.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GateOperationUpdate {
    pub refresh_hp: bool,
    pub next_action: Option<u16>,
}

/// City `OperatorCityGate` (`0x001CF370..0x001CF640`): refresh семьи обновляет
/// HP до action `7`; `OC_Died` применяет action `6`.
pub const fn city_gate_operation_update(operation: i32) -> GateOperationUpdate {
    let empty = GateOperationUpdate {
        refresh_hp: false,
        next_action: None,
    };
    match operation {
        GATE_OP_OPEN => GateOperationUpdate {
            refresh_hp: false,
            next_action: Some(7),
        },
        GATE_OP_CLOSE => GateOperationUpdate {
            refresh_hp: false,
            next_action: Some(1),
        },
        GATE_OP_REFRESH => GateOperationUpdate {
            refresh_hp: true,
            next_action: Some(7),
        },
        GATE_OP_DIED => GateOperationUpdate {
            refresh_hp: false,
            next_action: Some(6),
        },
        _ => empty,
    }
}

/// Country pointer-overload (`0x001CAC80/0x001CADD0`): в отличие от
/// city-владельца, для `OC_Died` и неизвестных operation успешно ничего не
/// делает.
pub const fn country_gate_operation_update(operation: i32) -> GateOperationUpdate {
    let empty = GateOperationUpdate {
        refresh_hp: false,
        next_action: None,
    };
    match operation {
        GATE_OP_OPEN => GateOperationUpdate {
            refresh_hp: false,
            next_action: Some(7),
        },
        GATE_OP_CLOSE => GateOperationUpdate {
            refresh_hp: false,
            next_action: Some(1),
        },
        GATE_OP_REFRESH => GateOperationUpdate {
            refresh_hp: true,
            next_action: Some(7),
        },
        _ => empty,
    }
}

/// Общий PDB-symbol `CServerCityRegion::CityGateIsClose` RVA `0x001CAAA0`:
/// country-region вызывает именно его, поэтому обе region-цепочки используют
/// один доказанный x-major footprint scan без объединения самих владельцев.
/// Footprint — block-снимок ворот с `block=0`; block-lookup owning региона
/// передаётся closure-швом и возвращает packed cell block.
pub fn footprint_is_clear(
    footprint: &BuildBlockUpdate,
    mut block_at: impl FnMut(i32, i32) -> Option<u8>,
) -> bool {
    let width = i32::from(footprint.width_increment);
    let height = i32::from(footprint.height_increment);
    // VERIFIED_DISASSEMBLY RVA 0x001CAAA0: x86 `sub/add` и loop increment
    // работают по DWORD с wrapping; это определяет поведение точнее, чем
    // потенциальный signed-overflow UB исходного C++. Существенный фрагмент:
    // `sub ebx,edi; add edi,eax; add edi,1; cmp edi,ebp; jle ...`.
    let left = footprint.tile_x.wrapping_sub(width);
    let right = footprint.tile_x.wrapping_add(width);
    let top = footprint.tile_y.wrapping_sub(height);
    let bottom = footprint.tile_y.wrapping_add(height);

    let mut tile_x = left;
    while tile_x <= right {
        let mut tile_y = top;
        while tile_y <= bottom {
            if block_at(tile_x, tile_y).is_some_and(|cell| cell & 7 == 3) {
                return false;
            }
            tile_y = tile_y.wrapping_add(1);
        }
        tile_x = tile_x.wrapping_add(1);
    }
    true
}
