//! Общая идентичность и wire-конверт семейства `CSummonShape` — призванных
//! навыковых форм Zone: тип 1000, правило счётчика ID (выдаётся прежнее
//! значение, приращение по модулю 2^32; живой счётчик остаётся у `CGame`
//! старого пакета) и пятипольный конверт клиентского снимка (skill id,
//! уровень, master type/id, остаток времени, хвост `CShape`).
//!
//! Размещение в `skills/`, а не в `regions/`: конверт принадлежит семейству
//! призванных навыковых фаланг, чьи владельцы (`WeakPhalanx` и др.) уже лежат
//! здесь; `regions/` хранит универсальную spatial-часть `CShape` без
//! скиллового конверта.
//!
//! Расширенный хвост конверта (счётчик + raw-массив клеток у GodThunder,
//! SnowStorm, FallingStar) машинно снят, но в Zone не перенесён; клиентское
//! чтение массива — UNKNOWN. Остальная поверхность `CSummonShape` (OnMessage,
//! ForceMove, End, decode) остаётся у прежнего владельца.
//!
//! Исходный владелец PDB: `appserver/summonshape.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#summonshape-семейство-csummonshape-идентичность-и-wire-конверт

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
