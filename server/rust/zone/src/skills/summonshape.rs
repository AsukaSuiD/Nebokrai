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
//! VERIFIED_DISASSEMBLY для порядка конверта по всему семейству — машинно
//! досмотрены все 11 уникальных тел `AddToByteArray@C*Phalanx` (таблица
//! ниже): skill ID базового `CSummonShape` (`[esi+0xB8]`; getter
//! `GetSkillID` RVA `0x1E1030`, pub `1:001e0030`), уровень навыка владельца
//! (immediate чтения зависит от владельца, см. таблицу), первые два поля
//! вложенного `tagMasterInfo` (`+0x84` type, `+0x88` id; копия десяти DWORD
//! через ctor/assign RVA `0x10A610`/`0x10A640`, см.
//! `zone/combat/masterinfo.rs`), оставшееся время (`GetRemainedTime`
//! RVA `0x1E9870`, pub `1:001e8870`; вызов `call 0x5E9870`) и финальный
//! `CShape::AddToByteArray` (RVA `0x5B250`). Исходная функция пробрасывает
//! флаг `include_child` (второй аргумент) в базовый `CShape`; перенесённый
//! клиентский снимок фиксирует `true` — поведение прежнего адаптера
//! сохранено, пересмотр остаётся за per-phalanx порциями владельцев.
//!
//! Семейство тел `AddToByteArray@C*Phalanx` (S_PUB32 PDB). Линкер частично
//! сфолдовал 29 имён в 11 уникальных тел: внутри группы машинный код
//! байт-идентичен, между группами адреса и immediate различаются. Во всех
//! телах уровень читается как поле самого объекта фаланги (`this` в esi),
//! а не через указатель мастера: различие immediate (`+0xC8`, `+0xD4`, …)
//! отражает разный layout владельцев, а не разный контракт.
//!
//! - RVA `0x1E4100` (VA `0x5E4100`): `CFatalBlowPhalanx`,
//!   `CHeartLessArrowPhalanx2`, `CHeartLessArrowPhalanx3`; уровень
//!   `[esi+0xBC]`. Пятипольный конверт — соответствует zone-конверту.
//! - RVA `0x1E47A0` (VA `0x5E47A0`): `CArcheryPhalanx`,
//!   `CBaseMagicPhalanx`, `CBFBaseAttackPhalanx`, `CSpiderMistPhalanx`,
//!   `CLeimingPhalanx2`, `CYinYangPhalanx`, `CYinYangPhalanx2`; уровень
//!   `[esi+0xCC]`. Соответствует.
//! - RVA `0x1EF0C0` (VA `0x5EF0C0`): `CGodThunderPhalanx`,
//!   `CGodThunderPhalanx2`, `CThunderPhalanx`; уровень `[esi+0xD8]`.
//!   Расширенный хвост до `CShape`, см. ниже: счётчик
//!   `n = ([esi+0xB0] / [esi+0xC4]) * [esi+0xDC]` (беззнаковое деление).
//!   Пятипольный префикс совпадает, расширение к zone-конверту не
//!   перенесено — частичное соответствие, полный перенос отложен.
//! - RVA `0x1F54D0` (VA `0x5F54D0`): `CWeakPhalanx`,
//!   `CGodPunishmentPhalanx`, `CThunderBlowPhalanx`, `CTianhuoPhalanx`;
//!   уровень `[esi+0xC8]`. Соответствует (базовый спотчек ранней порции).
//! - RVA `0x1F76A0` (VA `0x5F76A0`): `CThunderSlashPhalanx`; уровень
//!   `[esi+0xE4]`. Соответствует.
//! - RVA `0x1F8ED0` (VA `0x5F8ED0`): `CSnowStormPhalanx`; уровень
//!   `[esi+0xD8]`. Расширенный хвост: счётчик
//!   `n = ([esi+0xB0] / [esi+0xC4]) * k2(level) * k1(level)`, где k1/k2
//!   выбираются по уровню 1/2/3 (иначе k1(1)/k2(1)) из глобальных RVA
//!   `0x2A52C4`/`0x2A52CC`/`0x2A52D4` (k1 = `g_dwLevelOne/Two/ThreeLength`)
//!   и `0x2A52C8`/`0x2A52D0`/`0x2A52D8` (k2 = `…Height`); семантика
//!   снята машинно: это размеры области по уровням, для этой сборки 5×5
//!   на всех уровнях → `n = ticks·25` независимо от уровня. Префикс
//!   совпадает, расширение не перенесено.
//! - RVA `0x1F9750` (VA `0x5F9750`): `CFallingStarPhalanx`,
//!   `CMeteorArrowPhalanx`; уровень `[esi+0xEC]`. Расширенный хвост:
//!   счётчик `n = [esi+0xE4]`. Префикс совпадает, расширение не перенесено.
//! - RVA `0x1F9F30` (VA `0x5F9F30`): `CRainArrowPhalanx`; уровень
//!   `[esi+0x108]`. Соответствует.
//! - RVA `0x1FA850` (VA `0x5FA850`): `CLightingArrowPhalanx`; уровень
//!   `[esi+0xF4]`. Соответствует.
//! - RVA `0x1FBD20` (VA `0x5FBD20`): `CFireBoltPhalanx`,
//!   `CFireBallPhalanx`, `CFireWallPhalanx`, `CPoisonFogPhalanx`,
//!   `CThunderFirePhalanx`; уровень `[esi+0xD4]`. Соответствует (базовый
//!   спотчек ранней порции).
//! - RVA `0x1FECB0` (VA `0x5FECB0`): `CChaosSpherePhalanx`; уровень
//!   `[esi+0xD8]`. Соответствует.
//!
//! Расширенный хвост трёх тел (godthunder, snowstorm, fallingstar) имеет
//! одну схему: после оставшегося времени дописываются `[esi+0xB0]` и
//! `[esi+0xC4]` как i32, счётчик `n` как i32 и `8*n` сырых байт по
//! указателю из `[esi+0xBC]` через helper RVA `0x7AD90` (VA `0x47AD90`) —
//! raw-append `(vector, ptr, byte_count)`; различается только вычисление
//! `n`. Семантика полей снята машинно (PDB TPI + дизассембл write-sites):
//! `+0xB0` = `m_dwLifeTime` (полный срок жизни, мс timeGetTime-шкалы),
//! `+0xBC` = `tagCell* m_pCells` (массив клеток `{long lX, long lY}` —
//! абсолютные tile-координаты точек удара; источник raw-блока — сам
//! указатель, не двойное разыменование), `+0xC4` = `m_dwFrequency`
//! (период тика, мс; из `QueryProperty(6001)`), `+0xDC` = `m_dwNumTargets`
//! (клеток на тик; из `QueryProperty(20010)`; у FallingStar-семейства то
//! же смещение — `m_lCCH`), `+0xE4` = `m_dwNumArrows` (FallingStar/Meteor;
//! у GT-семейства — `m_lCCH`). Серверный массив имеет страйд `H·L` на
//! тик; wire-счётчик GT = `ticks·NumTargets` — самосогласован при конфиге
//! `NumTargets == Length·Height` (INFERRED: значения конфига вне машинного
//! основания). Wire-dword после префикса сериализуется, но на декоде
//! пропускается (skip 8 байт — читается `m_dwFrequency`); `NumTargets`,
//! `NumArrows`, `m_dwAttackCount`, `m_pScope` декодом НЕ восстанавливаются
//! (особенность оригинала). Клиентское визуальное чтение массива —
//! UNKNOWN (клиентская сборка не разбиралась). Символов `*Masked*` в PDB
//! этой сборки нет.
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
//!
//! Ветви диспетчера живых форм `SummonedSkillShape` для ChaosSphere,
//! GodThunder, MaskedElement (FireWall и YinYang), FireBall и GodPunishment
//! порцией замыкания оканчиваются в владельцах `chaossphere`, `godthunder`,
//! `masked_area` и `projectile`: живые композиты перенесены туда буквально.
//! Сам enum и правило счётчика ID остаются у прежнего владельца `CGame` до
//! последней per-phalanx порции.

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
