//! Общая идентичность и wire-конверт семейства `CSummonShape` — призванных
//! навыковых форм Zone.
//!
//! Размещение в `skills/`, а не в `regions/`: тип, счётчик и конверт
//! принадлежат семейству призванных навыковых фаланг, чьи конкретные
//! владельцы (`WeakPhalanx` и др.) уже лежат здесь; `regions/` хранит
//! универсальную spatial-часть `CShape` без скиллового конверта.
//!
//! Точная пара: `original/server/Miracle_server/GameServer/gameserver.exe`
//! (SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`)
//! + `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`
//! age 2, совпадение подтверждено `.local/evidence/symbols.py identity`).
//! Исходные владельцы PDB: `appserver/summonshape.cpp/.h`.
//!
//! `SUMMON_SHAPE_TYPE = 1000`: VERIFIED_DISASSEMBLY. Конструктор
//! `CSummonShape` (RVA `0x1E97A0`, pub `1:001e87a0`) пишет
//! `mov dword ptr [esi+4], 0x3e8` в поле type базового `CBaseObject`.
//!
//! `next_summon_shape_id`: VERIFIED_DISASSEMBLY. Pub
//! `3:0000671c ?g_lID@CSummonShape@@1JA` — знаковый статический `long`.
//! Конструктор читает `g_lID` в ID объекта (`[esi+8]`, RVA `0x1E9820`) и
//! лишь затем выполняет `add dword ptr [0x6a471c], 1`: выдаётся прежнее
//! значение, приращение по модулю 2^32 даёт `wrapping_add(1)`. Живой счётчик
//! принадлежит `CGame` старого пакета, поэтому смена региона не создаёт
//! повторные устаревшие ID; сюда перенесено только правило.
//!
//! `encode_related_phalanx_snapshot` / `encode_related_phalanx_prefix`:
//! VERIFIED_DISASSEMBLY для порядка конверта по двум членам семейства,
//! `CWeakPhalanx::AddToByteArray` (RVA `0x1F54D0`) и
//! `CFireBoltPhalanx::AddToByteArray` (RVA `0x1FBD20`): skill ID базового
//! `CSummonShape` (`[esi+0xB8]`; getter `GetSkillID` RVA `0x1E1030`, pub
//! `1:001e0030`), уровень навыка владельца (`+0xC8` у weak, `+0xD4` у
//! firebolt), первые два поля вложенного `tagMasterInfo` (`+0x84` type,
//! `+0x88` id; копия десяти DWORD через ctor/assign RVA `0x10A610`/`0x10A640`,
//! см. `zone/combat/masterinfo.rs`), оставшееся время (`GetRemainedTime`
//! RVA `0x1E9870`, pub `1:001e8870`) и финальный `CShape::AddToByteArray`
//! (RVA `0x5B250`). Тела владельцев НЕ слиты линкером: адреса и immediate
//! полей различаются, общим является только порядок конверта, поэтому
//! значения приходят параметрами. Исходная функция пробрасывает флаг
//! `include_child` в базовый `CShape`; перенесённый клиентский снимок
//! фиксирует `true` — поведение прежнего адаптера сохранено, пересмотр
//! остаётся за per-phalanx порциями владельцев.
//!
//! Двойное чтение часов: VERIFIED_DISASSEMBLY. `GetRemainedTime` обращается
//! к часам дважды через IAT `0x64B264` (это `WINMM!timeGetTime`): первое
//! чтение — в проверке истечения `now < lifetime + started` (беззнаковое
//! `jb`), второе — при вычитании `lifetime - now + started`; истёкший срок
//! даёт 0. Оба чтения, беззнаковое сравнение и арифметика по модулю 2^32
//! сохранены в `encode_related_phalanx_prefix` дословно. Запись полей —
//! побайтовый little-endian append (helper RVA `0x7AF00`), что совпадает с
//! `LegacyWriter::write_i32/write_u32`.
//!
//! Остальная поверхность `CSummonShape` (OnMessage, ForceMove, MoveStep,
//! End, `CScope`, decode) пока остаётся у прежнего владельца; метаданные её
//! исследования хранятся в evidence-блоке
//! `src/gameserver/appserver/summonshape.rs`.

use crate::regions::shape::CShape;
use nebokrai_shared::protocol::LegacyWriter;

/// Общий для процесса тип всех призванных навыковых форм (см. шапку).
pub const SUMMON_SHAPE_TYPE: i32 = 1000;

/// Правило прежнего `g_lID`: после выдачи текущего значения счётчик
/// увеличивается на единицу с переполнением. Сам счётчик и начальное нулевое
/// значение остаются у владельца `CGame`.
pub const fn next_summon_shape_id(current: i32) -> i32 {
    current.wrapping_add(1)
}

/// Общий точный wire-конверт снимков семейства призванных фаланг:
/// идентификатор и уровень навыка, два зависящих от владельца `long`
/// (type/id вложенного `tagMasterInfo`) и оставшееся время перед базовым
/// `CShape`. Значение пары определяет конкретный владелец; незавершённая
/// ветвь сохраняет два чтения часов (см. шапку).
pub fn encode_related_phalanx_snapshot(
    shape: &CShape,
    skill_id: i32,
    skill_level: i32,
    related_type: i32,
    related_id: i32,
    started_at_ms: u32,
    lifetime_ms: u32,
    now_milliseconds: impl FnMut() -> u32,
) -> Option<Vec<u8>> {
    let mut payload = encode_related_phalanx_prefix(
        skill_id, skill_level, related_type, related_id,
        started_at_ms, lifetime_ms, now_milliseconds,
    );
    shape
        .add_to_byte_array(&mut payload, true)
        .then_some(payload)
}

pub fn encode_related_phalanx_prefix(
    skill_id: i32, skill_level: i32, related_type: i32, related_id: i32,
    started_at_ms: u32, lifetime_ms: u32, mut now_milliseconds: impl FnMut() -> u32,
) -> Vec<u8> {
    let mut payload = Vec::new();
    let mut writer = LegacyWriter::new(&mut payload);
    writer.write_i32(skill_id);
    writer.write_i32(skill_level);
    writer.write_i32(related_type);
    writer.write_i32(related_id);
    let remained = if started_at_ms.wrapping_add(lifetime_ms) <= now_milliseconds() {
        0
    } else {
        lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(started_at_ms)
    };
    writer.write_u32(remained);
    payload
}
