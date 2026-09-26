//! Данные, wire-stream decoder-семья и скалярные операции общего war-региона
//! `CServerWarRegion`. Исходный владелец — `appserver/serverwarregion.h/.cpp`;
//! сверка по точной паре `gameserver.exe` + `GameServer.pdb`. Переходный агрегат
//! остаётся в старом пакете: хранит живой региональный реестр `CServerRegion`,
//! а contender/symbol колонки читают и переписывают hub-потребители. Сюда
//! делегированы data-типы contender-ов и payload-логов, typed-границы
//! арифметики, потоковые readers с `RegionDecodeInputBlock`, error-семейство
//! `WarRegionDecodeError` и чистые context-контракты; decode-contend-context
//! над живым `CServerRegion` остаётся у старого пакета.
//!
//! `DWORD`-время и signed умнения сохраняют wrapping. Неопределённый x86-край
//! `INT_MIN / -1` возвращает локальный `ContendArithmeticBlock`, а недоказанная
//! реакция x87 `fistp i32` на NaN/inf/out-of-range — `WarDamageArithmeticBlock`
//! (BLOCKED_MISSING_FACT наследия, без придуманного saturating cast). Decoder
//! обновляет только keys `0..total`: старые map-keys за новым total оригинал не
//! очищает. Записей `m_lVicSymbolNum` в constructor-е нет — исходное значение
//! остаётся `UNKNOWN` (zero в `Default` не утверждает содержимое
//! неинициализированной памяти оригинала).
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#war-регионы

use nebokrai_shared::protocol::LegacyReader;

use super::serverregion::spawnsetup::ServerRegionMonsterRectBlock;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContendState {
    pub id: i32,
    pub name: String,
    pub player_id: i32,
    pub faction_id: i32,
    pub current_time: i32,
    pub max_time: i32,
    pub start_time_ms: u32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContendPlayerState {
    pub player_id: i32,
    pub faction_id: i32,
    pub union_id: i32,
    pub country: u8,
    pub faction_name: String,
    pub shape_type: i32,
    pub is_dead: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolCaptureLog<'a> {
    pub war_number: i32,
    pub owned_faction_id: i32,
    pub owned_union_id: i32,
    pub faction_name: &'a str,
    pub faction_id: i32,
    pub player_id: i32,
    pub symbol_id: i32,
    pub union_id: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WarRegionOwnership {
    pub faction_id: i32,
    pub union_id: i32,
}

/// BLOCKED_MISSING_FACT: exact EXE выполняет signed `idiv`; реакция процесса
/// на единственную пару `INT_MIN / -1` не доказана текущим owner-ом.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContendArithmeticBlock {
    pub current_time: i32,
    pub max_time: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WarDamagePlayer {
    pub player_id: i32,
    pub max_hp: u32,
}

/// BLOCKED_MISSING_FACT: для NaN/inf/out-of-range x87 `fistp i32` точная
/// реакция процесса не доказана; safe Rust не назначает ей saturating cast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WarDamageArithmeticBlock {
    pub max_time: i32,
    pub damage: i32,
    pub max_hp: u32,
    pub dec_time_param_bits: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WarDamageError {
    X87(WarDamageArithmeticBlock),
    Percentage(ContendArithmeticBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContendAiError<MembershipError> {
    Base(ServerRegionMonsterRectBlock),
    Arithmetic(ContendArithmeticBlock),
    Membership(MembershipError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionDecodeInputBlock {
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
    LegacyStringOverflow {
        field: &'static str,
        first_out_of_bounds_offset: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WarRegionDecodeError<BaseError> {
    Base(BaseError),
    Input(RegionDecodeInputBlock),
}

pub trait WarRegionContext {
    type MembershipError;

    /// Возвращает текущую faction найденного player; `None` сохраняет contender.
    fn player_faction_id(&mut self, player_id: i32) -> Option<i32>;

    /// Может остановить обновление на локальной неизвестной schedule-owner-а.
    fn is_apply_war_faction(&mut self, faction_id: i32) -> Result<bool, Self::MembershipError>;

    /// Шлёт player-у `0xBFF29` с одним signed значением времени.
    fn send_contend_time(&mut self, player_id: i32, time: i32);

    /// Меняет contend-state player из глобального `s_mapPlayer`, если он существует.
    fn set_global_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Меняет state только у region-local player, найденного через `FindChildObject`.
    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool);
}

pub trait WarRegionClearContext {
    /// Узкий exact tail `ClearRegion`: schedule membership и global lookup
    /// здесь не вызываются, нужны только reset wire и region-local state.
    fn send_contend_time(&mut self, player_id: i32, time: i32);
    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool);
}

impl<Context: WarRegionContext + ?Sized> WarRegionClearContext for Context {
    fn send_contend_time(&mut self, player_id: i32, time: i32) {
        WarRegionContext::send_contend_time(self, player_id, time);
    }

    fn set_region_player_contend_state(&mut self, region_id: i32, player_id: i32, state: bool) {
        WarRegionContext::set_region_player_contend_state(self, region_id, player_id, state);
    }
}

/// Узкий context входа в захват. Script caller не обязан подменять заглушками
/// AI/victory effects, которые `OnEnterContend` никогда не вызывает.
pub trait WarContendEntryContext: WarRegionContext {
    /// Возвращает младшие 32 бита монотонного миллисекундного счётчика.
    fn now_millis(&mut self) -> u32;

    /// Вызывает concrete City/Village `IsOwner` для текущего региона.
    fn is_owner(&mut self, faction_id: i32) -> bool;

    fn player_has_good(&mut self, player_id: i32, good_name: &str) -> bool;

    /// Меняет contend-state у уже известного non-null player pointer.
    fn set_known_player_contend_state(&mut self, player_id: i32, state: bool);

    /// Шлёт player-localized строку без форматных аргументов.
    fn notify_player(&mut self, player_id: i32, string_id: &'static str);

    /// Concrete virtual slot `+0x118`: Village сохраняет строку, City — no-op.
    fn register_needed_good(&mut self, good_name: &str);

    /// Шлёт region `GS0246(country-name, faction-name, symbol-name)`.
    fn send_first_faction_contender_notice(
        &mut self,
        country: u8,
        faction_name: &str,
        symbol_name: &str,
    );
}

/// Signed процент прогресса захвата: нулевой max возвращает `0`, а пара
/// `INT_MIN / -1` остаётся локальным `ContendArithmeticBlock`.
pub fn contend_percentage(
    current_time: i32,
    max_time: i32,
) -> Result<i32, ContendArithmeticBlock> {
    if max_time == 0 {
        return Ok(0);
    }
    current_time
        .wrapping_mul(100)
        .checked_div(max_time)
        .ok_or(ContendArithmeticBlock {
            current_time,
            max_time,
        })
}

/// VERIFIED_DISASSEMBLY RVA `0x001D2510`: damage и unsigned MaxHP сначала
/// становятся f32; ratio сохраняется как f32, затем x87 умножает его на
/// signed max_time и выгружает i32 с установленным процессом truncation RC.
pub fn legacy_war_damage_decrement(
    max_time: i32,
    damage: i32,
    max_hp: u32,
    dec_time_param: f32,
) -> Result<i32, WarDamageArithmeticBlock> {
    let block = || WarDamageArithmeticBlock {
        max_time,
        damage,
        max_hp,
        dec_time_param_bits: dec_time_param.to_bits(),
    };

    let damage_as_float = damage as f32;
    let max_hp_as_float = max_hp as f32;
    let ratio = ((f64::from(damage_as_float) / f64::from(max_hp_as_float))
        * f64::from(dec_time_param)) as f32;
    let scaled = f64::from(max_time) * f64::from(ratio);
    if !scaled.is_finite() || scaled < f64::from(i32::MIN) || scaled >= 2_147_483_648.0_f64 {
        return Err(block());
    }
    Ok(scaled.trunc() as i32)
}

pub fn read_region_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], RegionDecodeInputBlock> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: N,
            available: block.available,
        }
    })?;
    let bytes = reader
        .read_bytes(N)
        .map_err(|block| RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

pub fn read_region_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, RegionDecodeInputBlock> {
    let mut reader = LegacyReader::at(source, *cursor).map_err(|block| {
        RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: 4,
            available: block.available,
        }
    })?;
    let value = reader
        .read_i32()
        .map_err(|block| RegionDecodeInputBlock::UnexpectedEnd {
            field,
            offset: block.offset,
            needed: block.needed,
            available: block.available,
        })?;
    *cursor = reader.position();
    Ok(value)
}
