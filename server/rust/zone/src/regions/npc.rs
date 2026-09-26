//! Data-профиль и скалярные правила `CNpc` (type `500`). Исходный владелец —
//! `appserver/npc.h/.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb`. Переходный агрегат `CNpc` остаётся в старом пакете: хранит
//! те же колонки и делегирует сюда их поведение без изменения сигнатур;
//! нематериальные accessor-ы полей, кадр `Talk` через hub `CMessage`, client
//! encode через `move_shape` и spawn-семья `CServerRegion::AddNpc` остаются у
//! него.
//!
//! Колонки live/born хранятся `Option<u32>`: ctor их не пишет, а назначает
//! region spawn до публикации объекта. Проекции figure/HP берут значения только
//! разрешённых ICF-слотов vtable (виртуально-константный ноль), а не полей ctor.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#npc-и-базовые-фигуры

use super::moveshape::MoveShapePositionFacts;
use super::shape::{CShape, ShapeFigure, ShapeView};

pub const NPC_TYPE: i32 = 500;

/// Data-профиль exact ctor-части `CNpc` (VA `0x005D3E90`): type `500`,
/// включённый show list и пустой script; live/born ctor не записывает —
/// `None` до назначения region spawn-ом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NpcConstructorProfile {
    pub object_type: i32,
    pub show_list: bool,
    pub live_time_ms: Option<u32>,
    pub born_time_ms: Option<u32>,
}

pub const NPC_CONSTRUCTOR_PROFILE: NpcConstructorProfile = NpcConstructorProfile {
    object_type: NPC_TYPE,
    show_list: true,
    live_time_ms: None,
    born_time_ms: None,
};

/// Запись `m_strScriptFile` семантикой inline-writer `CServerRegion::AddNpc`
/// (VA `0x00480A40`): strcpy-копия до первого NUL без завершающего нуля.
pub fn set_script_file(script_file: &mut Vec<u8>, value: &[u8]) {
    let prefix_len = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    script_file.clear();
    script_file.extend_from_slice(&value[..prefix_len]);
}

/// Exact lifetime-предикат `CNpc::AI` (VA `0x005D3DD0`): `live == 0`
/// отключает срок; истечение при unsigned `live < now - born` с DWORD
/// wrapping. Неназначенные колонки не истекают.
pub const fn lifetime_expired(
    live_time_ms: Option<u32>,
    born_time_ms: Option<u32>,
    now_ms: u32,
) -> bool {
    match (live_time_ms, born_time_ms) {
        (Some(live_time), Some(born_time)) if live_time != 0 => {
            live_time < now_ms.wrapping_sub(born_time)
        }
        _ => false,
    }
}

/// Exact virtual `CNpc::LossHP` (VA `0x005D3ED0`): NPC не получает урон
/// через combat-chain, результат всегда ноль.
pub const fn loss_hp(_amount: i32) -> u16 {
    0
}

/// Exact virtual `CNpc::DecordFromByteArray` (VA `0x005E9840`): ничего не
/// читает и возвращает true.
pub const fn decord_from_byte_array() -> bool {
    true
}

/// Проекция NPC в zone `ShapeView`. Собственной figure-колонки у `CNpc`
/// нет: vtable-слот `GetFigure` `+0x8C` наследует `CShape`-функцию, свёрнутую
/// ICF в `XOR AL,AL; RET 4` (RVA `0x000856A0`), — нулевая figure по всем
/// направлениям (базис и статусы — в шапке файла).
pub fn shape_view(shape: &CShape) -> Option<ShapeView> {
    Some(ShapeView {
        identity: shape.identity(),
        tile_x: shape.get_tile_x().ok()?,
        tile_y: shape.get_tile_y().ok()?,
        pos_x_bits: shape.get_pos_x().to_bits(),
        pos_y_bits: shape.get_pos_y().to_bits(),
        figure: ShapeFigure::default(),
    })
}

/// RTTI-факты NPC для пространственной регистрации `CMoveShape::SetPosXY`:
/// здоровье всегда нулевое (vtable-слот `GetHP` `+0xD0` свёрнут ICF в
/// `XOR EAX,EAX; RET` RVA `0x00201200`), figure нулевая по тому же слоту
/// `+0x8C`, а `current_area` фиксирует ctor-состояние `[+0x60] = 0`;
/// назначение и смену area после регистрации разрешает dispatcher.
pub fn movement_position_facts(area_width: i32, area_height: i32) -> MoveShapePositionFacts {
    MoveShapePositionFacts {
        current_hit_points: 0,
        figure: ShapeFigure::default(),
        current_area: None,
        area_width,
        area_height,
    }
}
