//! Data-профиль и скалярные правила `CNpc` (type `500`). Исходный владелец —
//! `appserver/npc.h/.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb` (идентификаторы сборки —
//! `server/rust/src/manifest/_gameserver_export_manifest.toml`; RSDS-запись
//! внутри EXE задаёт тот же GUID и age, что PDB). Pubs зафиксированы в
//! нотации section:offset; `.text` этой сборки начинается с RVA `0x1000`,
//! т.е. `1:001d2e90` = RVA `0x001D3E90` = VA `0x005D3E90`.
//! Переходный агрегат `CNpc` остаётся в старом пакете: хранит те же колонки
//! и делегирует сюда их поведение без изменения сигнатур; нематериальные
//! accessor-ы чтения/записи полей, формирование кадра `Talk` через hub
//! `CMessage`, client encode через `move_shape` и spawn-семья
//! `CServerRegion::AddNpc` остаются у него.
//!
//! Дизассембл и декомпилят машинного кода пары подтверждают правила:
//!
//! - ctor `1:001d2e90` (VA `0x005D3E90`) — VERIFIED: вызывает базовый
//!   `CMoveShape`, вешает vtable `0x0065DA1C` и записывает только type
//!   `500` (`+0x4`), пустую script-строку (MSVC `std::string` на `+0x1D4`:
//!   буфер `+0x1D8=0`, size `+0x1E8=0`, capacity `+0x1EC=0xF`) и
//!   `m_bShowList = true` (байт `+0x1F0=1`). Колонки live `+0x1F4` и born
//!   `+0x1F8` ctor не трогает — Rust хранит их как `Option<u32>`, а
//!   назначает region spawn до публикации объекта. Вызов базы — `CALL
//!   0x4D0C60` (ctor `CMoveShape` `1:000cfc60` = RVA `0x000D0C60`); полей
//!   figure/HP в layout `CMoveShape` нет, их значения виртуально-константные
//!   (слоты `+0x8C`/`+0xD0` ниже), и ctor-цепочка их не хранит, а current
//!   area `[+0x60]` и change-state `[+0x80]` зануляет ctor `CShape` (RVA
//!   `0x0005B9A0`).
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
//!   RET 8` — всегда ноль, оба аргумента не читаются. Это новый виртуал
//!   `CNpc`: витабла `CMoveShape` заканчивается на 105 слотах, расширение
//!   `CNpc` — шесть слотов `+0x1A4..+0x1B8`, и `LossHP` — единственный из
//!   них с собственным телом (слот `+0x1B4`); остальные пять, включая
//!   объявленный у `CNpc` виртуальный `IsDied`, свёрнуты ICF в общий
//!   funclet RVA `0x00201200` (`XOR EAX,EAX; RET`).
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
//! - Витаблы `CNpc` ↔ `CMoveShape` — VERIFIED_DISASSEMBLY (сырой дамп сверки
//!   — локальный `.local/evidence/audit-npc-vtable.txt`, драйвер
//!   `audit-npc-vtable.py`): `CNpc` — pub `2:00012a1c` = RVA `0x0025DA1C`
//!   = VA `0x0065DA1C`, 111 слотов, единственная витабла класса; `CMoveShape`
//!   — `2:00006e7c` = RVA `0x00251E7C` = VA `0x00651E7C`, 105 слотов. На
//!   всех слотах figure/combat-гнезда таблицы совпадают; собственные
//!   отличия `CNpc` — dtor, codec-пара, `AI`, `IsAttackAble` (слот `+0x134`,
//!   у `CMoveShape` там `_purecall`) и шесть виртуалов расширения конца
//!   таблицы. Проекции `shape_view`/`movement_position_facts` принимают
//!   значения именно этих разрешённых слотов:
//! - figure-факт (`ShapeFigure::default()`) — VERIFIED_DISASSEMBLY:
//!   `GetFigure` — слот `+0x8C` (машинная привязка: `CShape::SetBlock` RVA
//!   `0x0005BA60` вызывает `[this+0x8C]` с аргументами 2 и 0 как
//!   горизонтальный и вертикальный half-extents; overrides семейства —
//!   `CBuild` слот `+0x8C` → RVA `0x001DD280`, `CMonster` → `0x000E7D20`).
//!   У `CNpc`, как и у `CMoveShape`/`CShape`, слот `+0x8C` указывает на
//!   общий ICF-funclet RVA `0x000856A0` — `XOR AL,AL; RET 4` (ICF x7:
//!   `GetFigure@CShape`, `IsAttackAble@CNpc`, `FindChildObject@CBaseObject`
//!   и четыре region/goods-метода с телом «константный ноль»). figure по
//!   всем четырём направлениям нулевая — виртуально-константная, а не
//!   ctor-поле. Совпадение слота `+0x18` (`FindChildObject@CBaseObject`) с
//!   тем же funclet — свёртка идентичных тел, другой слот, не `GetFigure`.
//! - HP-факт (`current_hit_points: 0`) — VERIFIED_DISASSEMBLY: `GetHP` —
//!   слот `+0xD0` (машинная привязка: невиртуальный `CMoveShape::IsDied`
//!   RVA `0x000CCF20` вызывает `[vftable+0xD0]` и сравнивает с нулём;
//!   overrides — `CBuild` слот `+0xD0` → RVA `0x001DD5F0`, `CMonster` →
//!   `0x000E7D40`). У `CNpc`/`CMoveShape` слот `+0xD0` ведёт в общий
//!   ICF-funclet RVA `0x00201200` — `XOR EAX,EAX; RET` (ICF x55, включая
//!   `GetHP@CMoveShape` и `GetHP@CNpc`), то есть виртуальный `GetHP()` NPC
//!   всегда 0. Условие записи блока в `SetPosXY` (RVA `0x000CD050`):
//!   `CALL [+0xD0]; TEST EAX,EAX; JA`, иначе `CMP [ESI+4],0x1F4; JNE` —
//!   в точности проецируемое `hp != 0 || type == 500`, причём type пишет
//!   ctor (`+0x4` = 500).
//! - `GetBeAttackedPoint` — VERIFIED_DISASSEMBLY: слот `+0xAC` → RVA
//!   `0x0004A250` (pub `1:00049250`); `CNpc` override не объявляет и
//!   наследует слот с тем же target — реальное тело base-impl, не thunk:
//!   out-параметры заполняются собственными `GetTileX` RVA `0x0005B110` и
//!   `GetTileY` RVA `0x0005B140`, figure там не читается. Собственные
//!   overrides только у `CBuild` (слот `+0xAC` → RVA `0x001DD350`) и
//!   `CMonster` (→ `0x000E6AA0`), поэтому `nearest_figure_attack_point`
//!   остаётся их геометрией, а NPC отвечает своей клеткой.
//! - `IsDied` — VERIFIED_DISASSEMBLY (ICF-layout): два раздельных паблика.
//!   Невиртуальный `IsDied@CMoveShape` `1:000cbf20` (RVA `0x000CCF20`) —
//!   `return virtual GetHP() == 0` (`CALL [vftable+0xD0]`, `SBB EAX,EAX`,
//!   `ADD EAX,1`). Собственный виртуальный `IsDied@CNpc` (`1:00200200`) —
//!   один из новых виртуалов расширения и свёрнут ICF в funclet RVA
//!   `0x00201200` (`XOR EAX,EAX; RET` → константная false); точный слот
//!   внутри пятёрки return-0 (`+0x1A4/+0x1A8/+0x1AC/+0x1B0/+0x1B8`) по
//!   машинному коду неразличим — тела идентичны, значение слота одно и то
//!   же, статус ординала не ставится. Zone `moveshape::is_died` моделирует
//!   базовый предикат поверх facts `current_hit_points`.
//! - `current_area: None` — VERIFIED_DISASSEMBLY как ctor-состояние плюс
//!   NULL-branch регистрации: ctor `CShape` (RVA `0x0005B9A0`) пишет
//!   `[+0x60] = 0` (current area-указатель) и `[+0x80] = 0` (change-state);
//!   `SetPosXY` при `[+0x60] == 0` (`TEST ECX,ECX; JE` на запись нуля)
//!   ставит change-state `SHAPE_CHANGE_NONE` и не пишет next-area
//!   `+0x68/+0x6C`. Проекция фиксирует это состояние; назначение и смена
//!   area после регистрации разрешаются dispatcher-ом region-слоя — это
//!   граница проекции, а не машинное утверждение о поздних состояниях.
//!   Прежний PARTIAL снят: расхождений проекций с машинными значениями не
//!   установлено.

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
