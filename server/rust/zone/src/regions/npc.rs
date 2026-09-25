//! Data-профиль и скалярные правила `CNpc` исторического GameServer,
//! перенесённые в Zone `regions/` первой порцией волны npc. Исходный
//! владелец — `appserver/npc.h/.cpp`. Переходный агрегат `CNpc` остаётся в
//! старом пакете, хранит те же колонки и делегирует сюда их поведение без
//! изменения сигнатур; нематериальные accessor-ы чтения/записи полей
//! остаются у переходного владельца. Формирование кадра `Talk` через hub
//! `CMessage`, client encode через `move_shape` и spawn-семья
//! `CServerRegion::AddNpc` этой порцией не переносятся.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2; RSDS-запись внутри EXE задаёт этот же GUID и age). Pubs
//! зафиксированы в нотации section:offset; `.text` этой сборки начинается с
//! RVA `0x1000`, поэтому, например, `1:001d2e90` = RVA `0x001D3E90` = VA
//! `0x005D3E90`. Дизассембли и декомпилят машинного кода пары подтверждают
//! переносимые правила:
//!
//! - ctor `1:001d2e90` (VA `0x005D3E90`) — VERIFIED: вызывает базовый
//!   `CMoveShape`, вешает vtable `0x0065DA1C` и записывает только type
//!   `500` (`+0x4`), пустую script-строку (MSVC `std::string` на `+0x1D4`:
//!   буфер `+0x1D8=0`, size `+0x1E8=0`, capacity `+0x1EC=0xF`) и
//!   `m_bShowList = true` (байт `+0x1F0=1`). Колонки live `+0x1F4` и born
//!   `+0x1F8` ctor не трогает — Rust хранит их как `Option<u32>`, а
//!   назначает region spawn до публикации объекта.
//! - script-колонка — VERIFIED: отдельного pub `CNpc::SetScriptFile` в
//!   функциях пары нет (соседний `SetScriptFile` VA `0x005CBAC0`
//!   принадлежит `CBuild` и основание не подтверждает); запись
//!   `m_strScriptFile` выполняет inline-часть `CServerRegion::AddNpc` (VA
//!   `0x00480A40`): strlen-цикл до первого NUL и `std::string::assign` по
//!   `+0x1D4` без завершающего нуля — та же strcpy-семантика, что у
//!   подтверждённых `CMonster::SetScriptFile` (RVA `0x00038C70`) и
//!   `CBuild::SetScriptFile` (VA `0x005CBAC0`).
//! - lifetime-предикат из `AI` `1:001d2dd0` (VA `0x005D3DD0`) — VERIFIED:
//!   `live == 0` отключает срок; `EAX = GetTickCount() - born` как DWORD
//!   с естественным wrapping; unsigned `CMP live, EAX; JNC exit` —
//!   истечение при `live < now - born`. Сам AI дальше шлёт around
//!   `0xBF504(type, id, 0)` и удаляет объект виртуальным слотом `+0x24` —
//!   это остаётся у регионального владельца.
//! - `LossHP` `1:001d2ed0` (VA `0x005D3ED0`) — VERIFIED: `XOR AX,AX;
//!   RET 8` — всегда ноль, оба аргумента не читаются.
//! - `DecordFromByteArray` `1:001e8840` (VA `0x005E9840`) — VERIFIED:
//!   `MOV AL,1; RET 0xC` — ничего не читает и возвращает true.
//! - `Talk` `1:001d2f50` (VA `0x005D3F50`) — VERIFIED: девять area по
//!   таблице `_area` `0x006A3B98..0x006A3BE0`, только type `0x190` (400),
//!   строгие фильтры `abs(dx) < AREA_WIDTH` и `abs(dy) < AREA_HEIGHT`
//!   (глобалы `0x0069EFC8`/`0x0069EFCC`), кадр `0xBF801`: long `0`, long
//!   `500`, long id, строка имени, строка текста, `SendToPlayer` каждому
//!   игроку. Формирование кадра остаётся у hub `CMessage`.
//! - `AddToByteArray` `1:001d2dc0` (VA `0x005D3DC0`) — VERIFIED:
//!   `JMP 0x0045B250` — tail-jump в `CShape::AddToByteArray` без
//!   NPC-specific полей; encode остаётся у переходного владельца.
//! - Проекции `shape_view`/`movement_position_facts` — PARTIAL: это
//!   локальные проекции zone `ShapeView`/RTTI-фактов. Прямое основание —
//!   отсутствие у `CNpc` собственных HP/figure-колонок (ctor их не пишет,
//!   `LossHP` нулевой); сравнение vtable-слотов figure с базовым
//!   `CMoveShape` отдельно не проводилось.

use super::moveshape::MoveShapePositionFacts;
use super::shape::{CShape, ShapeFigure, ShapeView};

pub const NPC_TYPE: i32 = 500;

/// Data-профиль exact ctor-порции `CNpc` (VA `0x005D3E90`): type `500`,
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
/// нет — используется figure базового `CMoveShape` по умолчанию.
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
/// здоровье всегда нулевое (`CNpc` не имеет HP-колонки), figure по
/// умолчанию, текущую area разрешает dispatcher.
pub fn movement_position_facts(area_width: i32, area_height: i32) -> MoveShapePositionFacts {
    MoveShapePositionFacts {
        current_hit_points: 0,
        figure: ShapeFigure::default(),
        current_area: None,
        area_width,
        area_height,
    }
}
